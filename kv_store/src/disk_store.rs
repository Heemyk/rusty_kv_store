//! Disk-backed key-value store. Uses an append-only log for durability.
//!
//! Log format (binary):
//! - SET:   0x01 | key_len (u32 LE) | key | value_len (u32 LE) | value
//! - DELETE: 0x02 | key_len (u32 LE) | key
//!
//! On open, the log is replayed to rebuild the in-memory state.

use crate::error::StoreError;
use crate::store::KvStore;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::sync::Mutex;

const CMD_SET: u8 = 0x01;
const CMD_DELETE: u8 = 0x02;

/// Disk-backed store. Writes go to an append-only log; reads hit the in-memory cache.
pub struct DiskStore {
    /// In-memory state + log file. Mutex ensures write-order: log first, then mem.
    inner: Mutex<DiskStoreInner>,
}

struct DiskStoreInner {
    mem: HashMap<String, Vec<u8>>,
    log: BufWriter<File>,
}

impl DiskStore {
    /// Open or create a disk store at the given path. Replays existing log on open.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref();
        let mut mem = HashMap::new();

        // Replay existing log (open separately so BufReader can take ownership)
        if path.exists() {
            let file = File::open(path)?;
            let mut reader = BufReader::new(file);
            replay_log(&mut reader, &mut mem)?;
        }

        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)?;
        let log = BufWriter::new(file);

        Ok(Self {
            inner: Mutex::new(DiskStoreInner { mem, log }),
        })
    }
}

/// Read and apply all records from the log into mem.
fn replay_log(reader: &mut impl Read, mem: &mut HashMap<String, Vec<u8>>) -> Result<(), StoreError> {
    loop {
        let cmd = match read_u8(reader) {
            Ok(b) => b,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        };

        let key_len = read_u32_le(reader)? as usize;
        let mut key_buf = vec![0u8; key_len];
        reader.read_exact(&mut key_buf)?;
        let key = String::from_utf8(key_buf).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "key is not valid UTF-8")
        })?;

        match cmd {
            CMD_SET => {
                let value_len = read_u32_le(reader)? as usize;
                let mut value = vec![0u8; value_len];
                reader.read_exact(&mut value)?;
                mem.insert(key, value);
            }
            CMD_DELETE => {
                mem.remove(&key);
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unknown log cmd: 0x{:02x}", cmd),
                )
                .into());
            }
        }
    }
    Ok(())
}

fn read_u8(r: &mut impl Read) -> io::Result<u8> {
    let mut buf = [0u8; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_u32_le(r: &mut impl Read) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn write_u32_le(w: &mut impl Write, n: u32) -> io::Result<()> {
    w.write_all(&n.to_le_bytes())
}

impl KvStore for DiskStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let inner = self.inner.lock().map_err(|_| {
            StoreError::Io(io::Error::new(
                io::ErrorKind::Other,
                "mutex poisoned",
            ))
        })?;
        Ok(inner.mem.get(key).cloned())
    }

    fn set(&self, key: &str, value: Vec<u8>) -> Result<(), StoreError> {
        let mut inner = self.inner.lock().map_err(|_| {
            StoreError::Io(io::Error::new(
                io::ErrorKind::Other,
                "mutex poisoned",
            ))
        })?;

        // Write log first (durability); then update mem.
        inner.log.write_all(&[CMD_SET])?;
        write_u32_le(&mut inner.log, key.len() as u32)?;
        inner.log.write_all(key.as_bytes())?;
        write_u32_le(&mut inner.log, value.len() as u32)?;
        inner.log.write_all(&value)?;
        inner.log.flush()?;

        inner.mem.insert(key.to_string(), value);
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let mut inner = self.inner.lock().map_err(|_| {
            StoreError::Io(io::Error::new(
                io::ErrorKind::Other,
                "mutex poisoned",
            ))
        })?;

        let old = inner.mem.remove(key);

        // Write delete record (even if key wasn't present, for idempotent replay).
        inner.log.write_all(&[CMD_DELETE])?;
        write_u32_le(&mut inner.log, key.len() as u32)?;
        inner.log.write_all(key.as_bytes())?;
        inner.log.flush()?;

        Ok(old)
    }
}
