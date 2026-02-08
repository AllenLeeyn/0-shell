//! Command registry and shared types for 0-shell built-in commands.
use std::collections::HashMap;

mod cat;
mod cd;
mod clear;
mod cp;
mod echo;
mod exit;
mod export;
mod ls;
mod mkdir;
mod mv;
mod pwd;
mod rm;
mod util;

/// The result of a command execution.
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

/// A single command with its metadata and callback.
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

/// Collection of registered commands.
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
        if cmd_name == "help" {
            let mut help_text = String::from("Available commands:\n");
            for (name, cmd) in &self.cmds {
                help_text.push_str(&format!("  {:10} - {}\n", name, cmd.help));
            }
            return CommandResult::with_stdout(help_text);
        }

        let cmd = match self.cmds.get(&cmd_name) {
            Some(c) => c,
            None => {
                return CommandResult::with_stderr(format!(
                    "0-shell: {}: command not found",
                    cmd_name
                ));
            }
        };

        if flags.iter().any(|flag| flag == "--help" || flag == "-h") {
            return CommandResult::with_stdout(format!("Usage: {}\n", cmd.help));
        }

        if cmd.require_args && args.is_empty() {
            return CommandResult::with_stderr(format!(
                "{}: missing operand.\nTry 'help' or '{} --help' for more information.",
                cmd_name, cmd_name
            ));
        }

        (cmd.callback)(flags, args)
    }
}

/// Creates and registers all available commands.
pub fn command_list() -> CommandList {
    let mut cmds = CommandList::new();

    cmds.register(
        "exit".to_string(),
        Command::new("exit - cause the shell to exit", false, exit::exit_callback),
    );
    cmds.register(
        "export".to_string(),
        Command::new(
            "export [VAR=value | VAR]... - set or export environment variables",
            false,
            export::export_callback,
        ),
    );
    cmds.register(
        "echo".to_string(),
        Command::new(
            "echo [-e] [text ...] - display a line of text (-e interpret escapes)",
            false,
            echo::echo_callback,
        ),
    );
    cmds.register(
        "pwd".to_string(),
        Command::new(
            "pwd - print name of current/working directory",
            false,
            pwd::pwd_callback,
        ),
    );
    cmds.register(
        "cd".to_string(),
        Command::new(
            "cd [DIRECTORY] - change the working directory",
            false,
            cd::cd_callback,
        ),
    );
    cmds.register(
        "mkdir".to_string(),
        Command::new(
            "mkdir [-p] DIRECTORY... - create directories (-p parents)",
            true,
            mkdir::mkdir_callback,
        ),
    );
    cmds.register(
        "cat".to_string(),
        Command::new(
            "cat [FILE...] - concatenate files and print on the standard output",
            false,
            cat::cat_callback,
        ),
    );
    cmds.register(
        "clear".to_string(),
        Command::new(
            "clear - clear the terminal screen",
            false,
            clear::clear_callback,
        ),
    );
    cmds.register(
        "cp".to_string(),
        Command::new(
            "cp SOURCE DEST or cp -r SOURCE... DIRECTORY - copy files (use -r for directories)",
            true,
            cp::cp_callback,
        ),
    );
    cmds.register(
        "mv".to_string(),
        Command::new(
            "mv SOURCE DEST or mv SOURCE... DIRECTORY - move (rename) files",
            true,
            mv::mv_callback,
        ),
    );
    cmds.register(
        "rm".to_string(),
        Command::new(
            "rm [-r] FILE... - remove files or directories",
            true,
            rm::rm_callback,
        ),
    );
    cmds.register(
        "ls".to_string(),
        Command::new(
            "ls [-a] [-l] [-F] [FILE...] - list directory contents",
            false,
            ls::ls_callback,
        ),
    );

    cmds
}
