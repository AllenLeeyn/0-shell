//! `export` - set or export environment variables (handled in main, not via this callback).

use super::CommandResult;

/// Not used: export is intercepted in main and handled by do_export().
pub fn export_callback(_flags: Vec<String>, _args: Vec<String>) -> CommandResult {
    CommandResult::new()
}
