//! `mkdir` - create directories.
//!
//! By default creates only the last path component (fails if parent is missing).
//! Use `-p` or `--parents` to create intermediate directories as needed.

use std::fs;

use super::util;
use super::CommandResult;

pub fn mkdir_callback(flags: Vec<String>, args: Vec<String>) -> CommandResult {
    let parents = flags.iter().any(|f| f == "-p" || f == "--parents");
    let mut result = CommandResult::new();

    for path in args {
        let create_res = if parents {
            fs::create_dir_all(&path)
        } else {
            fs::create_dir(&path)
        };
        if let Err(e) = create_res {
            util::append_stderr(
                &mut result,
                &format!("mkdir: cannot create directory '{}': {}", path, e),
            );
        }
    }
    result
}
