//! `ls` - list directory contents.

use std::fs;
use std::path::Path;

use chrono::{DateTime, Local};
use std::ffi::CString;
use xattr;

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

use super::util;
use super::CommandResult;

pub fn ls_callback(flags: Vec<String>, mut args: Vec<String>) -> CommandResult {
    let all = flags.iter().any(|f| f == "-a");
    let long = flags.iter().any(|f| f == "-l");
    let classify = flags.iter().any(|f| f == "-F");

    if args.is_empty() {
        args.push(".".to_string());
    }

    let mut result = CommandResult::new();
    let multi_path = args.len() > 1;

    for (i, path_str) in args.iter().enumerate() {
        if multi_path {
            if i > 0 {
                result.stdout.push('\n');
            }
            result.stdout.push_str(&format!("{}:\n", path_str));
        }

        match fs::read_dir(path_str) {
            Ok(entries) => {
                let mut rows: Vec<(String, std::fs::Metadata)> = Vec::new();

                if all {
                    let dir_path = Path::new(path_str);
                    if let Ok(meta) = dir_path.metadata() {
                        rows.push((".".to_string(), meta));
                    }
                    let parent_path = dir_path.join("..");
                    if parent_path != dir_path {
                        if let Ok(meta) = parent_path.metadata() {
                            rows.push(("..".to_string(), meta));
                        }
                    }
                }

                for entry in entries {
                    match entry {
                        Ok(e) => {
                            let name = e.file_name().to_string_lossy().into_owned();
                            if all || !name.starts_with('.') {
                                if let Ok(meta) = e.path().metadata() {
                                    rows.push((name, meta));
                                }
                            }
                        }
                        Err(e) => util::append_stderr(&mut result, &format!("ls: {}", e)),
                    }
                }

                rows.sort_by(|a, b| a.0.cmp(&b.0));

                if long {
                    let total_blocks: u64 = rows.iter().map(|(_, m)| metadata_blocks_1k(m)).sum();
                    result
                        .stdout
                        .push_str(&format!("total {}\n", total_blocks));
                    for (mut name, metadata) in rows {
                        let full_path = Path::new(path_str).join(&name);
                        if classify {
                            let ft = metadata.file_type();
                            if ft.is_dir() {
                                name.push('/');
                            } else if ft.is_symlink() {
                                name.push('@');
                            } else if is_executable(&metadata) {
                                name.push('*');
                            }
                        }
                        let mode = format!(
                            "{}{}",
                            parse_permissions(&metadata),
                            permission_suffix(&full_path)
                        );
                        let nlink = metadata_nlink(&metadata);
                        let owner = metadata_owner(&metadata);
                        let group = metadata_group(&metadata);
                        let size = metadata_size(&metadata);
                        let modified: DateTime<Local> = metadata.modified().unwrap().into();
                        let time_str = modified.format("%b %e %H:%M").to_string();
                        result.stdout.push_str(&format!(
                            "{:11} {:>2} {} {} {:>8} {} {}\n",
                            mode, nlink, owner, group, size, time_str, name
                        ));
                    }
                } else {
                    let names: Vec<String> = rows
                        .into_iter()
                        .map(|(mut name, metadata)| {
                            if classify {
                                let ft = metadata.file_type();
                                if ft.is_dir() {
                                    name.push('/');
                                } else if ft.is_symlink() {
                                    name.push('@');
                                } else if is_executable(&metadata) {
                                    name.push('*');
                                }
                            }
                            name
                        })
                        .collect();

                    if names.is_empty() {
                        return result;
                    }

                    let (width, _) = term_size::dimensions().unwrap_or((80, 24));
                    let max_len = names.iter().map(|n| n.len()).max().unwrap_or(0) + 2;
                    let cols = (width as usize / max_len).max(1);
                    let rows = (names.len() + cols - 1) / cols;

                    for row in 0..rows {
                        for col in 0..cols {
                            if let Some(name) = names.get(row + col * rows) {
                                result.stdout.push_str(name);
                                for _ in 0..(max_len - name.len()) {
                                    result.stdout.push(' ');
                                }
                            }
                        }
                        result.stdout.push('\n');
                    }
                }
            }
            Err(e) => util::append_stderr(
                &mut result,
                &format!("ls: cannot access '{}': {}", path_str, e),
            ),
        }
    }

    result
}

fn metadata_size(metadata: &std::fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        metadata.size()
    }
    #[cfg(not(unix))]
    {
        metadata.len()
    }
}

fn metadata_nlink(metadata: &std::fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        metadata.nlink()
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        1
    }
}

fn metadata_blocks_1k(metadata: &std::fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        let blocks = metadata.blocks();
        #[cfg(target_os = "macos")]
        {
            blocks
        }
        #[cfg(not(target_os = "macos"))]
        {
            blocks / 2
        }
    }
    #[cfg(not(unix))]
    {
        (metadata.len() + 1023) / 1024
    }
}

fn metadata_owner(metadata: &std::fs::Metadata) -> String {
    #[cfg(unix)]
    {
        users::get_user_by_uid(metadata.uid())
            .map(|u| u.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| metadata.uid().to_string())
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        "-".to_string()
    }
}

fn metadata_group(metadata: &std::fs::Metadata) -> String {
    #[cfg(unix)]
    {
        users::get_group_by_gid(metadata.gid())
            .map(|g| g.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| metadata.gid().to_string())
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        "-".to_string()
    }
}

fn is_executable(metadata: &std::fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        false
    }
}

fn parse_permissions(metadata: &std::fs::Metadata) -> String {
    let mut s = String::with_capacity(10);
    let ft = metadata.file_type();

    #[cfg(unix)]
    {
        let mode = metadata.permissions().mode();
        s.push(if ft.is_dir() {
            'd'
        } else if ft.is_symlink() {
            'l'
        } else {
            '-'
        });
        let rwx = ["---", "--x", "-w-", "-wx", "r--", "r-x", "rw-", "rwx"];
        s.push_str(rwx[((mode >> 6) & 7) as usize]);
        s.push_str(rwx[((mode >> 3) & 7) as usize]);
        s.push_str(rwx[(mode & 7) as usize]);
    }

    #[cfg(not(unix))]
    {
        s.push(if ft.is_dir() {
            'd'
        } else if ft.is_symlink() {
            'l'
        } else {
            '-'
        });
        s.push_str("rw-rw-rw-");
    }

    s
}

fn permission_suffix(path: &Path) -> &'static str {
    let has_xattr = xattr::list(path)
        .map(|mut list| list.next().is_some())
        .unwrap_or(false);
    #[cfg(target_os = "macos")]
    let has_acl = has_acl_macos(path);
    #[cfg(not(target_os = "macos"))]
    let has_acl = false;

    match (has_xattr, has_acl) {
        (true, true) => "@+",
        (true, false) => "@",
        (false, true) => "+",
        (false, false) => "",
    }
}

#[cfg(target_os = "macos")]
fn has_acl_macos(path: &Path) -> bool {
    use libc::{c_int, c_void};
    let path = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };
    let path_c = match path.to_str().and_then(|s| CString::new(s).ok()) {
        Some(c) => c,
        None => return false,
    };
    unsafe extern "C" {
        fn acl_get_file(path: *const libc::c_char, type_: c_int) -> *mut c_void;
        fn acl_free(acl: *mut c_void) -> c_int;
    }
    const ACL_TYPE_EXTENDED: c_int = 0x100;
    let acl = unsafe { acl_get_file(path_c.as_ptr(), ACL_TYPE_EXTENDED) };
    if acl.is_null() {
        return false;
    }
    unsafe { acl_free(acl) };
    true
}
