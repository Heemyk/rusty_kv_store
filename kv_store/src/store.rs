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
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.data.read().unwrap().get(key).cloned()
    }

    /// Borrows self (write lock). key.into() converts &str or String to owned
    /// String; value is moved into the map. impl Into<String> lets callers pass
    /// "foo" directly instead of String::from("foo").
    pub fn set(&self, key: impl Into<String>, value: Vec<u8>) {
        self.data.write().unwrap().insert(key.into(), value);
    }

    /// Write lock (exclusive). Removes and returns the previous value if any.
    pub fn delete(&self, key: &str) -> Option<Vec<u8>> {
        self.data.write().unwrap().remove(key)
    }

    /// Read lock. Used by Debug/Display and for introspection.
    pub fn len(&self) -> usize {
        self.data.read().unwrap().len()
    }
}

impl fmt::Debug for MemStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.data.read().unwrap().len();
        f.debug_struct("MemStore").field("entries", &count).finish()
    }
}

impl fmt::Display for MemStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MemStore({} entries)", self.len())
    }
}
