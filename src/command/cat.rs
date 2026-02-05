//! `cat` - concatenate files and print on the standard output.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};

use super::CommandResult;

pub fn cat_callback(_flags: Vec<String>, args: Vec<String>) -> CommandResult {
    let mut result = CommandResult::new();
    if args.is_empty() {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut line = String::new();
        let mut stdout = io::stdout();

        while let Ok(n) = handle.read_line(&mut line) {
            if n == 0 {
                break;
            }
            if let Err(e) = stdout.write_all(line.as_bytes()) {
                result.stderr = format!("cat: {}", e);
                break;
            }
            let _ = stdout.flush();
            line.clear();
        }
    } else {
        for file_path in args {
            match File::open(&file_path) {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    let mut contents = String::new();
                    if let Err(e) = reader.read_to_string(&mut contents) {
                        if !result.stderr.is_empty() {
                            result.stderr.push('\n');
                        }
                        result
                            .stderr
                            .push_str(&format!("cat: {}: {}", file_path, e));
                    } else {
                        result.stdout.push_str(&contents);
                    }
                }
                Err(e) => {
                    if !result.stderr.is_empty() {
                        result.stderr.push('\n');
                    }
                    result
                        .stderr
                        .push_str(&format!("cat: {}: {}", file_path, e));
                }
            }
        }
    }
    result
}
