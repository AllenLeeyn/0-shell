//! Shared helpers for commands.

use std::path::{Path, PathBuf};

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
