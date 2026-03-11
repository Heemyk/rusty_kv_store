//! KV Store with interactive CLI.
//!
//! Roadmap: CLI (done) → KvStore trait → Error handling → Persistence → Networking → Consensus

use kv_store::cli;
use kv_store::store::MemStore;
use std::sync::Arc;

fn main() {
    let store = Arc::new(MemStore::new());
    cli::run(store);
}
