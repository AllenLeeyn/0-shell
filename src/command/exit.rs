//! `exit` - cause the shell to exit.

use super::CommandResult;

pub fn exit_callback(_flags: Vec<String>, _args: Vec<String>) -> CommandResult {
    CommandResult::exit()
}
