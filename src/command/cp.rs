//! `cp` - copy files and directories.
//!
//! With `-r`/`-R`/`--recursive`, copies directories recursively.
//! Copying a directory into itself is allowed; the self-subdirectory is skipped to avoid infinite recursion.

use std::fs;
use std::path::Path;

use super::CommandResult;
use super::util::{self, dest_under_src, is_same_path};

const RECURSIVE_FLAGS: &[&str] = &["-r", "-R", "--recursive"];

pub fn cp_callback(flags: Vec<String>, args: Vec<String>) -> CommandResult {
    if args.len() < 2 {
        return CommandResult::with_stderr(
            "cp: missing destination file operand after source".to_string(),
        );
    }

    let recursive = flags.iter().any(|f| RECURSIVE_FLAGS.contains(&f.as_str()));
    let mut result = CommandResult::new();
    let (sources, destination) = args.split_at(args.len() - 1);
    let dest_path = Path::new(&destination[0]);
    let dest_str = &destination[0];

    if sources.len() > 1 && (!dest_path.exists() || !dest_path.is_dir()) {
        return CommandResult::with_stderr(format!("cp: target '{}' is not a directory", dest_str));
    }

    for source_str in sources {
        let src_path = Path::new(source_str);
        if !src_path.exists() {
            util::append_stderr(
                &mut result,
                &format!("cp: {}: No such file or directory", source_str),
            );
            continue;
        }
        if src_path.is_dir() {
            if !recursive {
                util::append_stderr(
                    &mut result,
                    &format!("cp: omitting directory '{}'", source_str),
                );
                continue;
            }
            cp_recursive(
                &mut result,
                src_path,
                dest_path,
                source_str,
                dest_str,
                sources.len() == 1,
            );
        } else {
            cp_std(&mut result, src_path, dest_path, source_str);
        }
    }

    result
}

/// Copies a single file to the destination (or into the destination directory).
fn cp_std(result: &mut CommandResult, src_path: &Path, dest_path: &Path, source_str: &str) {
    let final_dest = match util::resolve_destination(src_path, dest_path) {
        Ok(d) => d,
        Err(e) => {
            util::append_stderr(result, &format!("cp: {}", e));
            return;
        }
    };
    if is_same_path(src_path, &final_dest) {
        return;
    }
    if let Err(e) = fs::copy(src_path, &final_dest) {
        util::append_stderr(result, &format!("cp: {}: {}", source_str, e));
    }
}

/// Recursively copies a directory. When `single_source` is true and dest doesn't exist,
/// creates dest and copies contents into it; otherwise copies as dest/source_name.
/// Copying into self (e.g. cp -r dir dir/sub) is allowed; copy_tree skips the self-subdir.
fn cp_recursive(
    result: &mut CommandResult,
    src_path: &Path,
    dest_path: &Path,
    source_str: &str,
    dest_str: &str,
    single_source: bool,
) {
    let final_dest = if dest_path.exists() && dest_path.is_dir() {
        dest_path.join(src_path.file_name().unwrap_or_default())
    } else if single_source {
        dest_path.to_path_buf()
    } else {
        util::append_stderr(
            result,
            &format!("cp: target '{}' is not a directory", dest_str),
        );
        return;
    };
    if is_same_path(src_path, &final_dest) {
        return;
    }
    if let Err(e) = copy_tree(src_path, &final_dest) {
        util::append_stderr(result, &format!("cp: {}: {}", source_str, e));
    }
}

/// Recursively copies a directory tree from `src` to `dest`.
/// Creates `dest` and replicates the structure and file contents of `src`.
/// Skips any subdirectory that would be copied into itself to prevent infinite recursion.
fn copy_tree(src: &Path, dest: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let name = entry.file_name();
        let dest_entry = dest.join(&name);
        if ty.is_dir() {
            if dest_under_src(&entry.path(), &dest_entry) {
                continue; // skip to prevent copying directory into itself
            }
            copy_tree(&entry.path(), &dest_entry)?;
        } else {
            fs::copy(entry.path(), &dest_entry)?;
        }
    }
    Ok(())
}
