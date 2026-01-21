use std::collections::HashMap;

pub struct Command {
    help: String,
    pub require_args: bool,
    callback: fn(Vec<String>) -> Result<String, String>,
}

impl Command {
    pub fn new(
        help: &str, 
        require_args: bool, 
        callback: fn(Vec<String>) -> Result<String, String>
    ) -> Self {
        Self {
            help: help.to_string(),
            require_args,
            callback,
        }
    }
}

pub struct CommandList {
    cmds: HashMap<String, Command>,
}

impl CommandList {
    pub fn new() -> Self {
        Self { 
            cmds: HashMap::new()
        }
    }

    pub fn register(
        &mut self,
        name: String,
        cmd: Command,
    ) {
        self.cmds.insert(name, cmd);
    }

    pub fn execute(&self, cmd_name: String, args: Vec<String>) -> Result<String, String> {
        // 1. Global 'help' list
        if cmd_name == "help" {
            let mut help_text = String::from("Available commands:\n");
            for (name, cmd) in &self.cmds {
                help_text.push_str(&format!("  {:10} - {}\n", name, cmd.help));
            }
            return Ok(help_text);
        }

        // 2. Command Lookup
        let cmd = self.cmds.get(&cmd_name)
            .ok_or_else(|| format!("0-shell: {}: command not found", cmd_name))?;

        // 3. Specific '--help' flag check
        if args.iter().any(|arg| arg == "--help" || arg == "-h") {
            return Ok(format!("Usage: {}\n", cmd.help));
        }

        // 4. Centralized Argument Validation
        if cmd.require_args && args.is_empty() {
            return Err(format!(
                "{}: missing operand.\nTry 'help' or '{} --help' for more information.", 
                cmd_name, 
                cmd_name
            ));
        }

        // 5. Trigger the callback
        (cmd.callback)(args)
    }
}

pub fn command_list() -> CommandList {
    let mut cmds = CommandList::new();

    cmds.register(
        "exit".to_string(),
        Command::new(
            "exit - cause the shell to exit",
            false,
            |_args| {
            // Logic directly here
            Ok("EXIT_SHELL".to_string())
        }
        ),
    );

    cmds.register(
        "echo".to_string(),
        Command::new(
            "echo [text] - display a line of text",
            false,
            echo_callback
        ),
    );

    cmds.register(
        "pwd".to_string(),
        Command::new(
            "pwd - print name of current/working directory",
            false,
            pwd_callback
        ),
    );

    cmds.register(
        "cd".to_string(),
        Command::new(
            "cd — change the working directory",
            false,
            cd_callback
        ),
    );

    // mkdir REQUIRES an argument
    cmds.register(
        "mkdir".to_string(),
        Command::new(
            "mkdir DIRECTORY... - Create directories recursively.", 
            true, 
            mkdir_callback
        ),
    );

    cmds.register(
        "cat".to_string(),
        Command::new(
            "cat - concatenate files and print on the standard output", 
            false, 
            cat_callback
        ),
    );

    cmds.register(
        "cp".to_string(),
        Command::new(
            "cp - copy files and directories", 
            true, 
            cp_callback
        ),
    );

    cmds.register(
        "mv".to_string(),
        Command::new(
            "mv - move (rename) files", 
            true, 
            mv_callback
        ),
    );

    cmds.register(
        "rm".to_string(),
        Command::new(
            "rm - remove files or directories", 
            true, 
            rm_callback
        ),
    );

    cmds.register(
        "ls".to_string(),
        Command::new(
            "ls - list directory contents",
            false,
            ls_callback)
    );

    cmds
}

fn echo_callback(args: Vec<String>) -> Result<String, String> {
    let mut interpret = false;
    let mut start_idx = 0;

    // Check for the -e flag
    if let Some(first_arg) = args.get(0) {
        if first_arg == "-e" {
            interpret = true;
            start_idx = 1;
        }
    }

    let input = args[start_idx..].join(" ");
    
    if !interpret {
        // Default Bash behavior: print literally
        return Ok(format!("{}\n", input));
    }

    // -e behavior: interpret backslash sequences
    let mut output = String::new();
    let mut chars = input.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('c') => return Ok(output), // "Stop" signal: return immediately without \n
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

    Ok(format!("{}\n", output))
}

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
use std::env;

fn pwd_callback(_args: Vec<String>) -> Result<String, String> {
    // env::current_dir() returns a Result<PathBuf, Error>
    match env::current_dir() {
        Ok(path) => {
            // Convert PathBuf to String. 
            // .display() handles the formatting for different OS platforms.
            Ok(format!("{}\n", path.display()))
        }
        Err(e) => Err(format!("pwd: error retrieving current directory: {}", e)),
    }
}

use std::path::Path;

fn cd_callback(args: Vec<String>) -> Result<String, String> {
    // 1. Determine the destination
    let destination = if args.is_empty() {
        // In Unix, 'cd' with no args goes to $HOME
        env::var("HOME").unwrap_or_else(|_| "/".to_string())
    } else {
        args[0].clone()
    };

    // 2. Attempt to change the directory
    let new_path = Path::new(&destination);
    match env::set_current_dir(new_path) {
        Ok(_) => Ok(String::new()), // Success: cd usually prints nothing
        Err(e) => Err(format!("cd: {}: {}", destination, e)),
    }
}

fn mkdir_callback(args: Vec<String>) -> Result<String, String> {
    for path in args {
        // create_dir_all handles nested paths and doesn't 
        // error if the directory already exists.
        if let Err(e) = std::fs::create_dir_all(&path) {
            return Err(format!("mkdir: cannot create directory '{}': {}", path, e));
        }
    }
    Ok(String::new())
}

use std::io::{self, BufRead, BufReader, Write};
use std::fs::File;

fn cat_callback(args: Vec<String>) -> Result<String, String> {
    let mut stdout = io::stdout();

    if args.is_empty() {
        // --- Interactive Mode ---
        let stdin = io::stdin();
        let handle = stdin.lock();
        
        // Use a loop to echo line-by-line immediately
        let mut line = String::new();
        let mut reader = io::BufReader::new(handle);
        
        while reader.read_line(&mut line).map_err(|e| e.to_string())? > 0 {
            stdout.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
            stdout.flush().map_err(|e| e.to_string())?;
            line.clear();
        }
    } else {
        // --- File Mode ---
        for file_path in args {
            let file = File::open(&file_path)
                .map_err(|e| format!("cat: {}: {}", file_path, e))?;
            let mut reader = BufReader::new(file);

            // io::copy streams directly from disk to screen
            io::copy(&mut reader, &mut stdout)
                .map_err(|e| format!("cat: {}: {}", file_path, e))?;
        }
    }

    // Return empty string because we've already written to stdout
    Ok(String::new())
}

use std::fs;

fn cp_callback(args: Vec<String>) -> Result<String, String> {
    // CommandList::execute handles "missing operand" check, 
    // but cp specifically needs at least TWO: source and dest.
    if args.len() < 2 {
        return Err("cp: missing destination file operand after source".to_string());
    }

    let (sources, destination) = args.split_at(args.len() - 1);
    let dest_path = Path::new(&destination[0]);

    // Check if the destination is an existing directory
    let is_dest_dir = dest_path.is_dir();

    if sources.len() > 1 && !is_dest_dir {
        return Err(format!("cp: target '{}' is not a directory", destination[0]));
    }

    for source_str in sources {
        let src_path = Path::new(source_str);
        
        // Construct the actual final path
        let final_dest = if is_dest_dir {
            let file_name = src_path.file_name()
                .ok_or_else(|| format!("cp: invalid source name '{}'", source_str))?;
            dest_path.join(file_name)
        } else {
            dest_path.to_path_buf()
        };

        // Perform the copy
        fs::copy(src_path, final_dest)
            .map_err(|e| format!("cp: {}: {}", source_str, e))?;
    }

    Ok(String::new())
}

fn mv_callback(args: Vec<String>) -> Result<String, String> {
    // We need at least a source and a destination
    if args.len() < 2 {
        return Err("mv: missing destination file operand after source".to_string());
    }

    // Split args into sources and the final destination
    let (sources, destination) = args.split_at(args.len() - 1);
    let dest_path = Path::new(&destination[0]);

    // Check if we are moving multiple things into a directory
    let is_dest_dir = dest_path.is_dir();

    if sources.len() > 1 && !is_dest_dir {
        return Err(format!("mv: target '{}' is not a directory", destination[0]));
    }

    for source_str in sources {
        let src_path = Path::new(source_str);
        
        // Determine the actual destination path
        let final_dest = if is_dest_dir {
            let file_name = src_path.file_name()
                .ok_or_else(|| format!("mv: invalid source '{}'", source_str))?;
            dest_path.join(file_name)
        } else {
            dest_path.to_path_buf()
        };

        // fs::rename handles files and folders across the same partition
        fs::rename(src_path, final_dest).map_err(|e| {
            format!("mv: cannot move '{}' to '{}': {}", source_str, destination[0], e)
        })?;
    }

    Ok(String::new())
}

fn rm_callback(args: Vec<String>) -> Result<String, String> {
    let mut recursive = false;
    let mut paths = Vec::new();
    let mut exit_error = false;
    let mut error_message = String::new();

    // 1. Separate flags from paths
    for arg in args {
        if arg == "-r" || arg == "-R" {
            recursive = true;
        } else {
            paths.push(arg);
        }
    }

    // 2. Process each path
    for path_str in paths {
        let path = Path::new(&path_str);
        
        // Define a small helper to capture errors without stopping the loop
        let result = if !path.exists() {
            Err(format!("rm: cannot remove '{}': No such file or directory", path_str))
        } else if path.is_dir() {
            if recursive {
                fs::remove_dir_all(path).map_err(|e| format!("rm: {}: {}", path_str, e))
            } else {
                Err(format!("rm: cannot remove '{}': Is a directory", path_str))
            }
        } else {
            fs::remove_file(path).map_err(|e| format!("rm: {}: {}", path_str, e))
        };

        // If an error happened, store it and mark the command as failed, but CONTINUE
        if let Err(e) = result {
            if !error_message.is_empty() { error_message.push('\n'); }
            error_message.push_str(&e);
            exit_error = true;
        }
    }

    if exit_error {
        Err(error_message)
    } else {
        Ok(String::new())
    }
}

use std::os::unix::fs::PermissionsExt;  
use chrono::{DateTime, Local};

fn ls_callback(args: Vec<String>) -> Result<String, String> {
    let mut all = false;
    let mut long = false;
    let mut classify = false;
    let mut paths = Vec::new();

    // 1. Manual Flag Parsing
    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 {
            for c in arg.chars().skip(1) {
                match c {
                    'a' => all = true,
                    'l' => long = true,
                    'F' => classify = true,
                    _ => return Err(format!("ls: invalid option -- '{}'", c)),
                }
            }
        } else {
            paths.push(arg);
        }
    }

    if paths.is_empty() { paths.push(".".to_string()); }

    let mut final_output = String::new();

    for path_str in paths {
        let entries = fs::read_dir(&path_str)
            .map_err(|e| format!("ls: cannot access '{}': {}", path_str, e))?;

        let mut entry_list = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().into_owned();

            // Handle -a
            if !all && name.starts_with('.') { continue; }
            entry_list.push(entry);
        }

        // Sort alphabetically
        entry_list.sort_by_key(|e| e.file_name());

        for entry in entry_list {
            let metadata = entry.metadata().map_err(|e| e.to_string())?;
            let mut name = entry.file_name().to_string_lossy().into_owned();

            // Handle -F (Classify)
            if classify {
                if metadata.is_dir() { name.push('/'); }
                else if metadata.permissions().mode() & 0o111 != 0 { name.push('*'); }
            }

            if long {
                // Handle -l (Long format)
                let mode = parse_permissions(metadata.permissions().mode());
                let size = metadata.len();
                let modified: DateTime<Local> = metadata.modified().unwrap().into();
                let time_str = modified.format("%b %d %H:%M").to_string();
                
                final_output.push_str(&format!("{} {:>8} {} {}\n", mode, size, time_str, name));
            } else {
                final_output.push_str(&format!("{}  ", name));
            }
        }
        final_output.push('\n');
    }

    Ok(final_output)
}

// Helper for -l permissions string
fn parse_permissions(mode: u32) -> String {
    let mut s = String::with_capacity(10);
    s.push(if mode & 0o40000 != 0 { 'd' } else { '-' }); // Basic dir check
    let rwx = ["---", "--x", "-w-", "-wx", "r--", "r-x", "rw-", "rwx"];
    s.push_str(rwx[((mode >> 6) & 7) as usize]);
    s.push_str(rwx[((mode >> 3) & 7) as usize]);
    s.push_str(rwx[(mode & 7) as usize]);
    s
}
