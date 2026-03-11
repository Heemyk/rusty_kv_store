//! Error types for the KV store.

use std::fmt;
use std::sync::PoisonError;

/// Errors that can occur in store operations.
#[derive(Debug)]
pub enum StoreError {
    /// The RwLock was poisoned (a thread panicked while holding the lock).
    LockPoisoned,
    /// I/O error (e.g. disk read/write).
    Io(std::io::Error),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::LockPoisoned => write!(f, "store lock poisoned (previous thread panicked)"),
            StoreError::Io(e) => write!(f, "i/o error: {}", e),
        }
    }
}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e)
    }
}

impl std::error::Error for StoreError {}

impl<T> From<PoisonError<T>> for StoreError {
    fn from(_: PoisonError<T>) -> Self {
        StoreError::LockPoisoned
    }
}

/// Errors that can occur in the CLI (parsing, validation, store).
#[derive(Debug)]
pub enum CliError {
    /// Parse/validation error with a user-facing message.
    Parse(String),
    /// Store operation failed.
    Store(StoreError),
    /// I/O error (e.g. stdin read, stdout write).
    Io(std::io::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Parse(msg) => write!(f, "{}", msg),
            CliError::Store(e) => write!(f, "store error: {}", e),
            CliError::Io(e) => write!(f, "i/o error: {}", e),
        }
    }
}

impl std::error::Error for CliError {}

impl From<StoreError> for CliError {
    fn from(e: StoreError) -> Self {
        CliError::Store(e)
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::Io(e)
    }
}
