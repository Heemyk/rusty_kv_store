//! Interactive CLI (read-eval-print loop) for the key-value store.
//!
//! Commands:
//! - GET <key>     — fetch value, print as UTF-8 (lossy)
//! - SET <key> <value> — store key=value (value can be multi-word)
//! - DELETE <key>  — remove key, print previous value or "deleted"
//! - EXIT / QUIT   — exit the REPL

use crate::error::CliError;
use crate::store::MemStore;
use std::io::{self, BufRead, Write};
use std::sync::Arc;

/// Run the interactive REPL. Returns on I/O error or when stdin is closed.
pub fn run(store: Arc<MemStore>) -> Result<(), CliError> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut lines = stdin.lock().lines();

    println!("KV Store CLI. Commands: GET <key> | SET <key> <value> | DELETE <key> | EXIT");
    print!("> ");
    stdout.flush()?;

    while let Some(line_result) = lines.next() {
        let line = line_result?;
        let line = line.trim();

        if line.is_empty() {
            print!("> ");
            stdout.flush()?;
            continue;
        }

        match execute(&store, line) {
            Ok(response) => println!("{}", response),
            Err(CliError::Parse(msg)) => eprintln!("Error: {}", msg),
            Err(e) => eprintln!("Error: {}", e),
        }

        print!("> ");
        stdout.flush()?;
    }

    Ok(())
}

/// Parse the line and execute the command. Returns a string to print or an error.
fn execute(store: &MemStore, line: &str) -> Result<String, CliError> {
    let mut iter = line.splitn(2, ' ');
    let cmd = iter.next().unwrap_or("").to_uppercase();
    let rest = iter.next().unwrap_or("").trim();

    match cmd.as_str() {
        "GET" => {
            let key = validate_key(rest, "GET")?;
            match store.get(key)? {
                Some(bytes) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
                None => Ok("(nil)".to_string()),
            }
        }
        "SET" => {
            let (key, value) = parse_set_args(rest)?;
            store.set(key, value.as_bytes().to_vec())?;
            Ok("OK".to_string())
        }
        "DELETE" => {
            let key = validate_key(rest, "DELETE")?;
            match store.delete(key)? {
                Some(bytes) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
                None => Ok("(nil)".to_string()),
            }
        }
        "EXIT" | "QUIT" => {
            std::process::exit(0);
        }
        "" => Ok("(nil)".to_string()),
        _ => Err(CliError::Parse(format!(
            "unknown command '{}'. Use GET, SET, DELETE, or EXIT",
            cmd
        ))),
    }
}

/// Validate a key: non-empty, no newlines or null bytes.
fn validate_key<'a>(key: &'a str, cmd: &str) -> Result<&'a str, CliError> {
    let key = key.trim();
    if key.is_empty() {
        return Err(CliError::Parse(format!("{} requires a non-empty key", cmd)));
    }
    if key.contains('\0') {
        return Err(CliError::Parse("key must not contain null bytes".to_string()));
    }
    if key.contains('\n') || key.contains('\r') {
        return Err(CliError::Parse("key must not contain newlines".to_string()));
    }
    Ok(key)
}

/// Parse "key value" for SET. Key is required; value may be empty.
fn parse_set_args(rest: &str) -> Result<(&str, &str), CliError> {
    let rest = rest.trim_start();
    if rest.is_empty() {
        return Err(CliError::Parse("SET requires a key".to_string()));
    }

    let key = if let Some(space) = rest.find(' ') {
        rest[..space].trim()
    } else {
        rest.trim()
    };

    let key = validate_key(key, "SET")?;

    let value = if let Some(space) = rest.find(' ') {
        &rest[space + 1..]
    } else {
        ""
    };

    Ok((key, value))
}
