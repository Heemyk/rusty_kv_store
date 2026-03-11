//! Interactive CLI (read-eval-print loop) for the key-value store.
//!
//! Commands:
//! - GET <key>     — fetch value, print as UTF-8 (lossy)
//! - SET <key> <value> — store key=value (value can be multi-word)
//! - DELETE <key>  — remove key, print previous value or "deleted"
//! - EXIT / QUIT   — exit the REPL

use crate::store::MemStore;
use std::io::{self, BufRead, Write};
use std::sync::Arc;

/// Run the interactive REPL. Blocks until the user types EXIT or QUIT.
pub fn run(store: Arc<MemStore>) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut lines = stdin.lock().lines();

    println!("KV Store CLI. Commands: GET <key> | SET <key> <value> | DELETE <key> | EXIT");
    print!("> ");
    stdout.flush().unwrap();

    while let Some(Ok(line)) = lines.next() {
        let line = line.trim();
        if line.is_empty() {
            print!("> ");
            stdout.flush().unwrap();
            continue;
        }

        let response = execute(&store, line);
        println!("{}", response);

        print!("> ");
        stdout.flush().unwrap();
    }
}

/// Parse the line and execute the command. Returns a string to print.
fn execute(store: &MemStore, line: &str) -> String {
    let mut iter = line.splitn(2, ' ');
    let cmd = iter.next().unwrap_or("").to_uppercase();
    let rest = iter.next().unwrap_or("").trim();

    match cmd.as_str() {
        "GET" => {
            if rest.is_empty() {
                return "Error: GET requires a key".to_string();
            }
            match store.get(rest) {
                Some(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                None => "(nil)".to_string(),
            }
        }
        "SET" => {
            let mut parts = rest.splitn(2, ' ');
            let key = parts.next().unwrap_or("");
            let value = parts.next().unwrap_or("");
            if key.is_empty() {
                return "Error: SET requires a key".to_string();
            }
            store.set(key, value.as_bytes().to_vec());
            "OK".to_string()
        }
        "DELETE" => {
            if rest.is_empty() {
                return "Error: DELETE requires a key".to_string();
            }
            match store.delete(rest) {
                Some(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                None => "(nil)".to_string(),
            }
        }
        "EXIT" | "QUIT" => {
            std::process::exit(0);
        }
        "" => "(nil)".to_string(),
        _ => format!("Error: unknown command '{}'", cmd),
    }
}
