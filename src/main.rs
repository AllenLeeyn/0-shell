mod command;
mod command_call;

use command::command_list;
use command_call::parse_line;
use std::env;
use std::io::{self, Write};

/// Main entry point for the 0-shell
/// Implements a read-eval-print loop (REPL) for command execution
fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    let cmds = command_list();

    loop {
        let prompt = get_prompt();
        stdout.write_all(prompt.as_bytes())?;
        stdout.flush()?;

        let mut line = String::new();
        // Lock stdin only long enough to read the command line
        let bytes_read = io::stdin().read_line(&mut line)?;

        if bytes_read == 0 {
            break; // EOF (Ctrl+D)
        }

        // Remove trailing newline
        let raw_input = line.trim_end();

        // Layer 1: Parse the line into individual calls (with flags separated)
        let calls = parse_line(raw_input);

        // Layer 2: Dispatch calls one by one
        for call in calls {
            let result = cmds.execute(call.name, call.flags, call.args);

            if result.should_exit {
                return Ok(());
            }

            if !result.stdout.is_empty() {
                stdout.write_all(result.stdout.as_bytes())?;
                stdout.flush()?;
            }

            if !result.stderr.is_empty() {
                stderr.write_all(format!("{}\n", result.stderr).as_bytes())?;
                stderr.flush()?;
            }
        }
    }

    Ok(())
}

/// Generates the shell prompt, showing the current directory
/// Replaces the home directory path with ~ for brevity
fn get_prompt() -> String {
    let cwd = env::current_dir().unwrap_or_default();
    let home = env::var("HOME").unwrap_or_default();

    let path_str = cwd.to_string_lossy();

    if !home.is_empty() && path_str.starts_with(&home) {
        format!("~{} $ ", &path_str[home.len()..])
    } else {
        format!("{} $ ", path_str)
    }
}
