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

    cmds
}

fn echo_callback(args: Vec<String>) -> Result<String, String> {
    // Arguments are already unescaped by the tokenizer!
    Ok(format!("{}\n", args.join(" ")))
}
