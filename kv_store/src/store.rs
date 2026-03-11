use crate::error::StoreError;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

/// In-memory key-value store. Thread-safe via Arc + RwLock.
/// - Arc: shared ownership so multiple threads can hold a handle.
/// - RwLock: coordinates access (many readers OR one writer).
pub struct MemStore {
    /// HashMap lives on heap; RwLock guards it; Arc shares it across threads.
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MemStore {
    pub fn new() -> Self {
        Self {
            // Arc::new allocates on heap; RwLock wraps the map for concurrent access.
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Borrows self (read lock). Returns owned Vec<u8> via .cloned() so we can
    /// release the lock before returning. key is &str (borrowed, not owned).
    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let guard = self.data.read()?;
        Ok(guard.get(key).cloned())
    }

    /// Borrows self (write lock). key.into() converts &str or String to owned
    /// String; value is moved into the map. impl Into<String> lets callers pass
    /// "foo" directly instead of String::from("foo").
    pub fn set(&self, key: impl Into<String>, value: Vec<u8>) -> Result<(), StoreError> {
        let mut guard = self.data.write()?;
        guard.insert(key.into(), value);
        Ok(())
    }

    /// Write lock (exclusive). Removes and returns the previous value if any.
    pub fn delete(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let mut guard = self.data.write()?;
        Ok(guard.remove(key))
    }

    /// Read lock. Used by Debug/Display and for introspection.
    pub fn len(&self) -> Result<usize, StoreError> {
        let guard = self.data.read()?;
        Ok(guard.len())
    }
}

impl fmt::Debug for MemStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.data.read().map(|g| g.len()).unwrap_or(0);
        f.debug_struct("MemStore").field("entries", &count).finish()
    }
}

impl fmt::Display for MemStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.data.read().map(|g| g.len()).unwrap_or(0);
        write!(f, "MemStore({} entries)", count)
    }
}
