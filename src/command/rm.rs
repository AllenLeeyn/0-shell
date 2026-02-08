//! `rm` - remove files or directories.

use std::fs;
use std::path::Path;

use super::{CommandResult, util};

pub fn rm_callback(flags: Vec<String>, args: Vec<String>) -> CommandResult {
    let recursive = flags
        .iter()
        .any(|f| f == "-r" || f == "-R" || f == "--recursive");
    let mut result = CommandResult::new();

    for path_str in args {
        let path = Path::new(&path_str);

        let remove_res = if !path.exists() {
            Err(format!(
                "rm: cannot remove '{}': No such file or directory",
                path_str
            ))
        } else if path.is_dir() {
            if recursive {
                fs::remove_dir_all(path).map_err(|e| format!("rm: {}: {}", path_str, e))
            } else {
                Err(format!("rm: cannot remove '{}': Is a directory", path_str))
            }
        } else {
            fs::remove_file(path).map_err(|e| format!("rm: {}: {}", path_str, e))
        };

        if let Err(e) = remove_res {
            util::append_stderr(&mut result, &e);
        }
    }

    result
}
