//! `mkdir` - create directories.

use super::CommandResult;

pub fn mkdir_callback(_flags: Vec<String>, args: Vec<String>) -> CommandResult {
    let mut result = CommandResult::new();
    for path in args {
        if let Err(e) = std::fs::create_dir_all(&path) {
            if !result.stderr.is_empty() {
                result.stderr.push('\n');
            }
            result
                .stderr
                .push_str(&format!("mkdir: cannot create directory '{}': {}", path, e));
        }
    }
    result
}
