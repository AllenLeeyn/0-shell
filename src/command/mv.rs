//! `mv` - move (rename) files and directories.

use std::fs;
use std::path::Path;

use super::CommandResult;
use super::util::{self, resolve_destination};

pub fn mv_callback(_flags: Vec<String>, args: Vec<String>) -> CommandResult {
    if args.len() < 2 {
        return CommandResult::with_stderr(
            "mv: missing destination file operand after source".to_string(),
        );
    }

    let mut result = CommandResult::new();
    let (sources, destination) = args.split_at(args.len() - 1);
    let dest_path = Path::new(&destination[0]);

    if sources.len() > 1 && !dest_path.is_dir() {
        return CommandResult::with_stderr(format!(
            "mv: target '{}' is not a directory",
            destination[0]
        ));
    }

    for source_str in sources {
        let src_path = Path::new(source_str);
        match resolve_destination(src_path, dest_path) {
            Ok(final_dest) => {
                if let Err(e) = fs::rename(src_path, final_dest) {
                    util::append_stderr(
                        &mut result,
                        &format!(
                            "mv: cannot move '{}' to '{}': {}",
                            source_str, destination[0], e
                        ),
                    );
                }
            }
            Err(e) => util::append_stderr(&mut result, &format!("mv: {}", e)),
        }
    }

    result
}
