use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

use chrono::{DateTime, Local};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// The result of a command execution, containing output and error streams.
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    /// If true, the shell should terminate.
    pub should_exit: bool,
}

impl CommandResult {
    pub fn new() -> Self {
        Self {
            stdout: String::new(),
            stderr: String::new(),
            should_exit: false,
        }
    }

    pub fn with_stdout(stdout: String) -> Self {
        Self {
            stdout,
            stderr: String::new(),
            should_exit: false,
        }
    }

    pub fn with_stderr(stderr: String) -> Self {
        Self {
            stdout: String::new(),
            stderr,
            should_exit: false,
        }
    }

    pub fn exit() -> Self {
        Self {
            stdout: String::new(),
            stderr: String::new(),
            should_exit: true,
        }
    }
}

/// Represents a single command with its metadata and callback function
pub struct Command {
    help: String,
    pub require_args: bool,
    callback: fn(Vec<String>, Vec<String>) -> CommandResult,
}

impl Command {
    pub fn new(
        help: &str,
        require_args: bool,
        callback: fn(Vec<String>, Vec<String>) -> CommandResult,
    ) -> Self {
        Self {
            help: help.to_string(),
            require_args,
            callback,
        }
    }
}

/// Collection of registered commands
pub struct CommandList {
    cmds: HashMap<String, Command>,
}

impl CommandList {
    pub fn new() -> Self {
        Self {
            cmds: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, cmd: Command) {
        self.cmds.insert(name, cmd);
    }

    pub fn execute(
        &self,
        cmd_name: String,
        flags: Vec<String>,
        args: Vec<String>,
    ) -> CommandResult {
        // 1. Global 'help' list
        if cmd_name == "help" {
            let mut help_text = String::from("Available commands:\n");
            for (name, cmd) in &self.cmds {
                help_text.push_str(&format!("  {:10} - {}\n", name, cmd.help));
            }
            return CommandResult::with_stdout(help_text);
        }

        // 2. Command Lookup
        let cmd = match self.cmds.get(&cmd_name) {
            Some(c) => c,
            None => {
                return CommandResult::with_stderr(format!(
                    "0-shell: {}: command not found",
                    cmd_name
                ));
            }
        };

        // 3. Specific '--help' flag check
        if flags.iter().any(|flag| flag == "--help" || flag == "-h") {
            return CommandResult::with_stdout(format!("Usage: {}\n", cmd.help));
        }

        // 4. Centralized Argument Validation
        if cmd.require_args && args.is_empty() {
            return CommandResult::with_stderr(format!(
                "{}: missing operand.\nTry 'help' or '{} --help' for more information.",
                cmd_name, cmd_name
            ));
        }

        // 5. Trigger the callback
        (cmd.callback)(flags, args)
    }
}

/// Creates and registers all available commands
pub fn command_list() -> CommandList {
    let mut cmds = CommandList::new();

    cmds.register(
        "exit".to_string(),
        Command::new("exit - cause the shell to exit", false, exit_callback),
    );

    cmds.register(
        "echo".to_string(),
        Command::new(
            "echo [-e] [text ...] - display a line of text",
            false,
            echo_callback,
        ),
    );

    cmds.register(
        "pwd".to_string(),
        Command::new(
            "pwd - print name of current/working directory",
            false,
            pwd_callback,
        ),
    );

    cmds.register(
        "cd".to_string(),
        Command::new(
            "cd [DIRECTORY] - change the working directory",
            false,
            cd_callback,
        ),
    );

    cmds.register(
        "mkdir".to_string(),
        Command::new(
            "mkdir DIRECTORY... - create directories",
            true,
            mkdir_callback,
        ),
    );

    cmds.register(
        "cat".to_string(),
        Command::new(
            "cat [FILE...] - concatenate files and print on the standard output",
            false,
            cat_callback,
        ),
    );

    cmds.register(
        "cp".to_string(),
        Command::new(
            "cp SOURCE DEST or cp SOURCE... DIRECTORY - copy files and directories",
            true,
            cp_callback,
        ),
    );

    cmds.register(
        "mv".to_string(),
        Command::new(
            "mv SOURCE DEST or mv SOURCE... DIRECTORY - move (rename) files",
            true,
            mv_callback,
        ),
    );

    cmds.register(
        "rm".to_string(),
        Command::new(
            "rm [-r] FILE... - remove files or directories",
            true,
            rm_callback,
        ),
    );

    cmds.register(
        "ls".to_string(),
        Command::new(
            "ls [-a] [-l] [-F] [FILE...] - list directory contents",
            false,
            ls_callback,
        ),
    );

    cmds
}

// ============================================================================
// Command Callback Functions
// ============================================================================

/// Causes the shell to exit.
///
/// Returns a special `CommandResult` that indicates the shell should terminate.
fn exit_callback(_flags: Vec<String>, _args: Vec<String>) -> CommandResult {
    CommandResult::exit()
}

/// Displays a line of text.
///
/// Supports the `-e` flag to interpret backslash escape sequences.
fn echo_callback(flags: Vec<String>, args: Vec<String>) -> CommandResult {
    let mut interpret = false;
    let mut result = CommandResult::new();

    // Check for the -e flag in flags
    if flags.iter().any(|f| f == "-e") {
        interpret = true;
    }

    let input = args.join(" ");

    if !interpret {
        // Default Bash behavior: print literally
        result.stdout = format!("{}\n", input);
        return result;
    }

    // -e behavior: interpret backslash sequences
    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('c') => {
                    result.stdout = output;
                    return result; // "Stop" signal: return immediately without \n
                }
                Some(next) => {
                    if let Some(mapped) = map_echo_escape(next) {
                        output.push(mapped);
                    } else {
                        // Not a recognized escape; push both literally
                        output.push('\\');
                        output.push(next);
                    }
                }
                None => output.push('\\'), // Trailing backslash
            }
        } else {
            output.push(c);
        }
    }

    result.stdout = format!("{}\n", output);
    result
}

/// Maps echo escape sequences to their corresponding characters.
fn map_echo_escape(c: char) -> Option<char> {
    match c {
        'a' => Some('\x07'), // BEL
        'b' => Some('\x08'), // Backspace
        'e' => Some('\x1b'), // Escape
        'f' => Some('\x0c'), // Form feed
        'n' => Some('\n'),
        'r' => Some('\r'),
        't' => Some('\t'),
        'v' => Some('\x0b'), // Vertical tab
        '\\' => Some('\\'),
        _ => None,
    }
}

/// Prints the current working directory.
fn pwd_callback(_flags: Vec<String>, _args: Vec<String>) -> CommandResult {
    match env::current_dir() {
        Ok(path) => CommandResult::with_stdout(format!("{}\n", path.display())),
        Err(e) => {
            CommandResult::with_stderr(format!("pwd: error retrieving current directory: {}", e))
        }
    }
}

/// Changes the current working directory.
///
/// If no arguments are provided, it defaults to the `HOME` environment variable,
/// or `/` if `HOME` is not set.
fn cd_callback(_flags: Vec<String>, args: Vec<String>) -> CommandResult {
    let destination = if args.is_empty() {
        env::var("HOME").unwrap_or_else(|_| "/".to_string())
    } else {
        args[0].clone()
    };

    let new_path = Path::new(&destination);
    match env::set_current_dir(new_path) {
        Ok(_) => CommandResult::new(),
        Err(e) => CommandResult::with_stderr(format!("cd: {}: {}", destination, e)),
    }
}

/// Creates one or more directories.
///
/// Uses `create_dir_all` to support nested paths and skip existing directories.
fn mkdir_callback(_flags: Vec<String>, args: Vec<String>) -> CommandResult {
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

/// Concatenates and prints files to standard output.
///
/// If no files are provided, it reads from standard input until EOF.
fn cat_callback(_flags: Vec<String>, args: Vec<String>) -> CommandResult {
    let mut result = CommandResult::new();
    if args.is_empty() {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut line = String::new();
        let mut stdout = io::stdout();

        // In interactive mode, echo lines immediately to stdout
        while let Ok(n) = handle.read_line(&mut line) {
            if n == 0 {
                break;
            }
            if let Err(e) = stdout.write_all(line.as_bytes()) {
                result.stderr = format!("cat: {}", e);
                break;
            }
            let _ = stdout.flush();
            line.clear();
        }
    } else {
        for file_path in args {
            match File::open(&file_path) {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    let mut contents = String::new();
                    if let Err(e) = reader.read_to_string(&mut contents) {
                        if !result.stderr.is_empty() {
                            result.stderr.push('\n');
                        }
                        result
                            .stderr
                            .push_str(&format!("cat: {}: {}", file_path, e));
                    } else {
                        result.stdout.push_str(&contents);
                    }
                }
                Err(e) => {
                    if !result.stderr.is_empty() {
                        result.stderr.push('\n');
                    }
                    result
                        .stderr
                        .push_str(&format!("cat: {}: {}", file_path, e));
                }
            }
        }
    }
    result
}

/// Resolves the final destination path for copy/move operations.
///
/// If the `dest_path` is a directory, the source's file name is appended to it.
fn resolve_destination(src_path: &Path, dest_path: &Path) -> Result<std::path::PathBuf, String> {
    if dest_path.is_dir() {
        let file_name = src_path
            .file_name()
            .ok_or_else(|| format!("invalid source path: {}", src_path.display()))?;
        Ok(dest_path.join(file_name))
    } else {
        Ok(dest_path.to_path_buf())
    }
}

/// Copies files and directories.
///
/// Supports multiple sources if the destination is a directory.
fn cp_callback(_flags: Vec<String>, args: Vec<String>) -> CommandResult {
    if args.len() < 2 {
        return CommandResult::with_stderr(
            "cp: missing destination file operand after source".to_string(),
        );
    }

    let mut result = CommandResult::new();
    let (sources, destination) = args.split_at(args.len() - 1);
    let dest_path = Path::new(&destination[0]);

    if sources.len() > 1 && !dest_path.is_dir() {
        return CommandResult::with_stderr(format!(
            "cp: target '{}' is not a directory",
            destination[0]
        ));
    }

    for source_str in sources {
        let src_path = Path::new(source_str);
        match resolve_destination(src_path, dest_path) {
            Ok(final_dest) => {
                if let Err(e) = fs::copy(src_path, final_dest) {
                    if !result.stderr.is_empty() {
                        result.stderr.push('\n');
                    }
                    result
                        .stderr
                        .push_str(&format!("cp: {}: {}", source_str, e));
                }
            }
            Err(e) => {
                if !result.stderr.is_empty() {
                    result.stderr.push('\n');
                }
                result.stderr.push_str(&format!("cp: {}", e));
            }
        }
    }

    result
}

/// Moves or renames files and directories.
///
/// Supports multiple sources if the destination is a directory.
fn mv_callback(_flags: Vec<String>, args: Vec<String>) -> CommandResult {
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
                    if !result.stderr.is_empty() {
                        result.stderr.push('\n');
                    }
                    result.stderr.push_str(&format!(
                        "mv: cannot move '{}' to '{}': {}",
                        source_str, destination[0], e
                    ));
                }
            }
            Err(e) => {
                if !result.stderr.is_empty() {
                    result.stderr.push('\n');
                }
                result.stderr.push_str(&format!("mv: {}", e));
            }
        }
    }

    result
}

/// Removes files or directories.
///
/// Supports the `-r` or `-R` flag for recursive removal of directories.
fn rm_callback(flags: Vec<String>, args: Vec<String>) -> CommandResult {
    let recursive = flags.iter().any(|f| f == "-r" || f == "-R");
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
            if !result.stderr.is_empty() {
                result.stderr.push('\n');
            }
            result.stderr.push_str(&e);
        }
    }

    result
}

/// Lists directory contents.
///
/// Supports the following flags:
/// - `-a`: List all entries, including those starting with `.`.
/// - `-l`: Use a long listing format.
/// - `-F`: Append a character to each entry indicating its type.
fn ls_callback(flags: Vec<String>, mut args: Vec<String>) -> CommandResult {
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
                let mut entry_list = Vec::new();
                for entry in entries {
                    match entry {
                        Ok(e) => {
                            let name = e.file_name().to_string_lossy().into_owned();
                            if all || !name.starts_with('.') {
                                entry_list.push(e);
                            }
                        }
                        Err(e) => {
                            if !result.stderr.is_empty() {
                                result.stderr.push('\n');
                            }
                            result.stderr.push_str(&format!("ls: {}", e));
                        }
                    }
                }

                entry_list.sort_by_key(|e| e.file_name());

                for entry in entry_list {
                    match entry.metadata() {
                        Ok(metadata) => {
                            let mut name = entry.file_name().to_string_lossy().into_owned();
                            if classify {
                                if metadata.is_dir() {
                                    name.push('/');
                                } else if is_executable(&metadata) {
                                    name.push('*');
                                }
                            }

                            if long {
                                let mode = parse_permissions(&metadata);
                                let size = metadata.len();
                                let modified: DateTime<Local> = metadata.modified().unwrap().into();
                                let time_str = modified.format("%b %d %H:%M").to_string();
                                result.stdout.push_str(&format!(
                                    "{} {:>8} {} {}\n",
                                    mode, size, time_str, name
                                ));
                            } else {
                                result.stdout.push_str(&format!("{}  ", name));
                            }
                        }
                        Err(e) => {
                            if !result.stderr.is_empty() {
                                result.stderr.push('\n');
                            }
                            result.stderr.push_str(&format!("ls: {}", e));
                        }
                    }
                }
                if !long {
                    result.stdout.push('\n');
                }
            }
            Err(e) => {
                if !result.stderr.is_empty() {
                    result.stderr.push('\n');
                }
                result
                    .stderr
                    .push_str(&format!("ls: cannot access '{}': {}", path_str, e));
            }
        }
    }

    result
}

/// Checks if a file is executable.
///
/// On Unix, checks the permission bits. On Windows, currently returns false.
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

/// Generates a human-readable permissions string (e.g., `drwxr-xr-x`).
fn parse_permissions(metadata: &std::fs::Metadata) -> String {
    let mut s = String::with_capacity(10);

    #[cfg(unix)]
    {
        let mode = metadata.permissions().mode();
        s.push(if mode & 0o40000 != 0 { 'd' } else { '-' });
        let rwx = ["---", "--x", "-w-", "-wx", "r--", "r-x", "rw-", "rwx"];
        s.push_str(rwx[((mode >> 6) & 7) as usize]);
        s.push_str(rwx[((mode >> 3) & 7) as usize]);
        s.push_str(rwx[(mode & 7) as usize]);
    }

    #[cfg(not(unix))]
    {
        s.push(if metadata.is_dir() { 'd' } else { '-' });
        s.push_str("rw-rw-rw-");
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_echo_basic() {
        let res = echo_callback(vec![], vec!["hello".to_string(), "world".to_string()]);
        assert_eq!(res.stdout, "hello world\n");
    }

    #[test]
    fn test_echo_escapes() {
        let res = echo_callback(vec!["-e".to_string()], vec!["hello\\nworld".to_string()]);
        assert_eq!(res.stdout, "hello\nworld\n");
    }

    #[test]
    fn test_pwd() {
        let res = pwd_callback(vec![], vec![]);
        let current = std::env::current_dir().unwrap();
        assert_eq!(res.stdout, format!("{}\n", current.display()));
    }

    #[test]
    fn test_exit() {
        let res = exit_callback(vec![], vec![]);
        assert!(res.should_exit);
    }

    #[test]
    fn test_mkdir_and_ls() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_dir");

        // Test mkdir
        let res = mkdir_callback(vec![], vec![path.to_str().unwrap().to_string()]);
        assert!(res.stderr.is_empty());
        assert!(path.exists());

        // Test ls
        let res = ls_callback(vec![], vec![dir.path().to_str().unwrap().to_string()]);
        assert!(res.stdout.contains("test_dir"));
    }

    #[test]
    fn test_cp_and_mv() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src.txt");
        let dest = dir.path().join("dest.txt");
        let moved = dir.path().join("moved.txt");

        fs::write(&src, "hello").unwrap();

        // Test cp
        cp_callback(
            vec![],
            vec![
                src.to_str().unwrap().to_string(),
                dest.to_str().unwrap().to_string(),
            ],
        );
        assert!(dest.exists());
        assert_eq!(fs::read_to_string(&dest).unwrap(), "hello");

        // Test mv
        mv_callback(
            vec![],
            vec![
                dest.to_str().unwrap().to_string(),
                moved.to_str().unwrap().to_string(),
            ],
        );
        assert!(!dest.exists());
        assert!(moved.exists());
        assert_eq!(fs::read_to_string(&moved).unwrap(), "hello");
    }

    #[test]
    fn test_rm() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("to_remove.txt");
        fs::write(&file, "bye").unwrap();

        rm_callback(vec![], vec![file.to_str().unwrap().to_string()]);
        assert!(!file.exists());

        let sub_dir = dir.path().join("sub");
        fs::create_dir(&sub_dir).unwrap();
        let res = rm_callback(vec![], vec![sub_dir.to_str().unwrap().to_string()]);
        assert!(!res.stderr.is_empty()); // Should fail without -r
        assert!(sub_dir.exists());

        rm_callback(
            vec!["-r".to_string()],
            vec![sub_dir.to_str().unwrap().to_string()],
        );
        assert!(!sub_dir.exists());
    }

    #[test]
    fn test_cat() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("cat_test.txt");
        fs::write(&file, "meow").unwrap();

        let res = cat_callback(vec![], vec![file.to_str().unwrap().to_string()]);
        assert_eq!(res.stdout, "meow");
    }

    #[test]
    fn test_command_list_execute() {
        let cmds = command_list();

        // Test help
        let res = cmds.execute("help".to_string(), vec![], vec![]);
        assert!(res.stdout.contains("Available commands"));

        // Test unrecognized
        let res = cmds.execute("nope".to_string(), vec![], vec![]);
        assert!(res.stderr.contains("command not found"));

        // Test command help flag
        let res = cmds.execute("ls".to_string(), vec!["-h".to_string()], vec![]);
        assert!(res.stdout.contains("Usage: ls [-a] [-l] [-F] [FILE...]"));

        // Test required args
        let res = cmds.execute("mkdir".to_string(), vec![], vec![]);
        assert!(res.stderr.contains("missing operand"));
    }
}
