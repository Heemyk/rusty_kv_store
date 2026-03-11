//! KV Store with interactive CLI.
//!
//! Roadmap: CLI (done) → KvStore trait → Error handling (done) → Persistence (done) → Networking → Consensus

use kv_store::cli;
use kv_store::disk_store::DiskStore;
use kv_store::error::CliError;
use std::path::PathBuf;
use std::sync::Arc;

fn main() -> Result<(), CliError> {
    let path = std::env::var("KV_STORE_PATH").unwrap_or_else(|_| "./kv_store.data".to_string());
    let path = PathBuf::from(path);
    let store = Arc::new(DiskStore::open(path)?);
    cli::run(store)
}
