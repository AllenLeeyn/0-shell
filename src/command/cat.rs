//! `cat` - concatenate files and print on the standard output.
//!
//! With no arguments, reads from stdin and writes to stdout.

use std::fs;
use std::io;

use super::util;
use super::CommandResult;

pub fn cat_callback(_flags: Vec<String>, args: Vec<String>) -> CommandResult {
    let mut result = CommandResult::new();
    if args.is_empty() {
        let mut stdin = io::stdin().lock();
        let mut stdout = io::stdout().lock();
        if let Err(e) = io::copy(&mut stdin, &mut stdout) {
            result.stderr = format!("cat: {}", e);
        }
    } else {
        for path in &args {
            match fs::read_to_string(path) {
                Ok(contents) => result.stdout.push_str(&contents),
                Err(e) => util::append_stderr(&mut result, &format!("cat: {}: {}", path, e)),
            }
        }
    }
    result
}
