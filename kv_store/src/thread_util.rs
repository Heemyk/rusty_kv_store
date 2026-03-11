use std::thread::{self, JoinHandle};

/// Spawns a new thread that runs the given closure. Returns a JoinHandle you
/// can `.join()` to block and receive the return value.
///
/// # Type parameters
/// - F: the closure type. Must be callable as f() -> T.
/// - T: the closure's return type, sent back across the thread boundary.
///
/// # Bounds
/// - F: FnOnce() -> T  — callable once, takes no args, returns T.
/// - F: Send + 'static — safe to move to another thread; no stack borrows.
/// - T: Send + 'static — return value safe to send back; no short-lived refs.
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    thread::spawn(f)
}
