//! `cd` - change the working directory.

use std::env;
use std::path::Path;

use super::CommandResult;

pub fn cd_callback(_flags: Vec<String>, args: Vec<String>) -> CommandResult {
    let destination = if args.is_empty() {
        env::var("HOME").unwrap_or_else(|_| "/".to_string())
    } else {
        args[0].clone()
    };

    let new_path = Path::new(&destination);
    match env::set_current_dir(new_path) {
        Ok(_) => CommandResult::new(),
        Err(e) => CommandResult::with_stderr(format!("cd: {}: {}", destination, e)),
    }
}
