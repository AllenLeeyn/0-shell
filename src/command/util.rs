//! Shared helpers for commands.

use std::path::{Path, PathBuf};

use super::CommandResult;

/// Appends a message to command stderr, with a newline before it if stderr is non-empty.
pub fn append_stderr(result: &mut CommandResult, msg: &str) {
    if !result.stderr.is_empty() {
        result.stderr.push('\n');
    }
    result.stderr.push_str(msg);
}

/// Resolves the final destination path for copy/move operations.
/// If the destination is a directory, the source's file name is appended.
pub fn resolve_destination(src_path: &Path, dest_path: &Path) -> Result<PathBuf, String> {
    if dest_path.is_dir() {
        let file_name = src_path
            .file_name()
            .ok_or_else(|| format!("invalid source path: {}", src_path.display()))?;
        Ok(dest_path.join(file_name))
    } else {
        Ok(dest_path.to_path_buf())
    }
}

/// Resolves `path` to an absolute path (canonical if it exists, else cwd.join(path)).
pub fn abs_path(path: &Path) -> Option<PathBuf> {
    if path.exists() {
        path.canonicalize().ok()
    } else {
        std::env::current_dir().ok().map(|cwd| cwd.join(path))
    }
}

/// Returns true if the two paths refer to the same file or directory.
pub fn is_same_path(src: &Path, dest: &Path) -> bool {
    match (abs_path(src), abs_path(dest)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// Returns true if `dest` is strictly under `src` (dest inside src, not equal).
pub fn dest_under_src(src: &Path, dest: &Path) -> bool {
    match (abs_path(src), abs_path(dest)) {
        (Some(abs_src), Some(abs_dest)) => {
            abs_src != abs_dest && abs_dest.strip_prefix(&abs_src).is_ok()
        }
        _ => false,
    }
}
