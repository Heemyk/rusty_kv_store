//! TCP server for the KV store.
//!
//! Protocol (line-based, one command per line):
//! - GET <key>
//! - SET <key> <value>
//! - DELETE <key>
//! Responses end with newline.

use kv_store::disk_store::DiskStore;
use kv_store::store::KvStore;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("KV_PORT").unwrap_or_else(|_| "6380".to_string());
    let path = std::env::var("KV_STORE_PATH").unwrap_or_else(|_| "./kv_store.data".to_string());
    let path = PathBuf::from(path);

    let store = Arc::new(DiskStore::open(path)?);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

    println!("KV server listening on 127.0.0.1:{}", port);

    loop {
        let (stream, addr) = listener.accept().await?;
        let store = Arc::clone(&store);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, store).await {
                eprintln!("Error handling {}: {}", addr, e);
            }
        });
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    store: Arc<DiskStore>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? != 0 {
        let cmd_line = line.trim();
        if cmd_line.is_empty() {
            line.clear();
            continue;
        }

        let response = execute(&store, cmd_line);
        writer.write_all(response.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        line.clear();
    }

    Ok(())
}

fn execute(store: &DiskStore, line: &str) -> String {
    let mut iter = line.splitn(2, ' ');
    let cmd = iter.next().unwrap_or("").to_uppercase();
    let rest = iter.next().unwrap_or("").trim();

    let result = match cmd.as_str() {
        "GET" => {
            if rest.is_empty() {
                Err("GET requires a key".to_string())
            } else {
                store.get(rest).map_err(|e| e.to_string()).and_then(|opt| {
                    Ok(opt
                        .map(|b| String::from_utf8_lossy(&b).into_owned())
                        .unwrap_or_else(|| "(nil)".to_string()))
                })
            }
        }
        "SET" => {
            let mut parts = rest.splitn(2, ' ');
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("");
            if key.is_empty() {
                Err("SET requires a key".to_string())
            } else {
                store.set(key, value.as_bytes().to_vec()).map(|_| "OK".to_string()).map_err(|e| e.to_string())
            }
        }
        "DELETE" => {
            if rest.is_empty() {
                Err("DELETE requires a key".to_string())
            } else {
                store.delete(rest)
                    .map(|opt| opt.map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_else(|| "(nil)".to_string()))
                    .map_err(|e| e.to_string())
            }
        }
        "" => Ok("(nil)".to_string()),
        _ => Err(format!("unknown command '{}'", cmd)),
    };

    match result {
        Ok(msg) => msg,
        Err(e) => format!("ERR {}", e),
    }
}
