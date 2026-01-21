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
            .ok_or_else(|| format!("Command '{}' not found", cmd_name))?;

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
            true,
            cd_callback
        ),
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
                Some('\\') => output.push('\\'),
                Some('a')  => output.push('\x07'), // BEL
                Some('b')  => output.push('\x08'), // Backspace
                Some('c')  => return Ok(output),   // Produce no further output
                Some('e')  => output.push('\x1b'), // Escape
                Some('f')  => output.push('\x0c'), // Form feed
                Some('n')  => output.push('\n'),
                Some('r')  => output.push('\r'),
                Some('t')  => output.push('\t'),
                Some('v')  => output.push('\x0b'), // Vertical tab
                Some(next) => {
                    output.push('\\');
                    output.push(next);
                }
                None => output.push('\\'),
            }
        } else {
            output.push(c);
        }
    }

    Ok(format!("{}\n", output))
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