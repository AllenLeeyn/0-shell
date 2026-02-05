//! `cd` - change the working directory.
//!
//! With no arguments, changes to `$HOME` (or `/` if `HOME` is unset).

use std::env;
use std::path::Path;

use super::CommandResult;

pub fn cd_callback(_flags: Vec<String>, args: Vec<String>) -> CommandResult {
    let dest = args
        .first()
        .cloned()
        .unwrap_or_else(|| env::var("HOME").unwrap_or_else(|_| "/".to_string()));

    match env::set_current_dir(Path::new(&dest)) {
        Ok(()) => CommandResult::new(),
        Err(e) => CommandResult::with_stderr(format!("cd: {}: {}", dest, e)),
    }
}
