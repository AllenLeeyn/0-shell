//! `pwd` - print name of current/working directory.

use std::env;

use super::CommandResult;

pub fn pwd_callback(_flags: Vec<String>, _args: Vec<String>) -> CommandResult {
    match env::current_dir() {
        Ok(path) => CommandResult::with_stdout(format!("{}\n", path.display())),
        Err(e) => CommandResult::with_stderr(format!(
            "pwd: error retrieving current directory: {}",
            e
        )),
    }
}
