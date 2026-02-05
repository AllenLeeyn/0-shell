//! 0-shell: minimal Unix-style shell (read-eval-print loop).

mod command;
mod command_call;

use command::command_list;
use command_call::{parse_line, unclosed_quote_prompt};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;
use std::io::{self, Write};
use std::path::Path;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    let cmds = command_list();

    // Initialize rustyline editor with history support
    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => {
            eprintln!("Failed to initialize line editor: {}", e);
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to initialize editor"));
        }
    };

    loop {
        let prompt = get_prompt();
        
        let raw_input = match read_line_with_history(&mut rl, &prompt) {
            Ok(Some(input)) => input,
            Ok(None) => break, // EOF (Ctrl+D)
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                continue;
            }
        };

        // Add non-empty commands to history (excluding empty lines)
        if !raw_input.trim().is_empty() {
            if let Err(e) = rl.add_history_entry(raw_input.as_str()) {
                eprintln!("Warning: Failed to add to history: {}", e);
            }
        }

        for call in parse_line(&raw_input) {
            let result = cmds.execute(call.name, call.flags, call.args);
            if result.should_exit {
                return Ok(());
            }
            write_command_result(&mut stdout, &mut stderr, &result)?;
        }
    }

    Ok(())
}

/// Writes a command's stdout and stderr to the given writers.
fn write_command_result(
    stdout: &mut impl Write,
    stderr: &mut impl Write,
    result: &command::CommandResult,
) -> io::Result<()> {
    if !result.stdout.is_empty() {
        stdout.write_all(result.stdout.as_bytes())?;
        stdout.flush()?;
    }
    if !result.stderr.is_empty() {
        stderr.write_all(result.stderr.as_bytes())?;
        stderr.write_all(b"\n")?;
        stderr.flush()?;
    }
    Ok(())
}

/// Reads a full line using rustyline with history support, showing continuation prompts
/// (e.g. `dquote> `) until quotes balance. Returns `None` on EOF.
fn read_line_with_history(
    rl: &mut DefaultEditor,
    prompt: &str,
) -> Result<Option<String>, ReadlineError> {
    let mut raw_input = String::new();
    let mut current_prompt = prompt;
    
    loop {
        let line = match rl.readline(current_prompt) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C - clear current input and start fresh
                raw_input.clear();
                current_prompt = prompt;
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D - return None if no input, otherwise return what we have
                return Ok(if raw_input.is_empty() {
                    None
                } else {
                    Some(raw_input.trim_end().to_string())
                });
            }
            Err(e) => return Err(e),
        };
        
        // Append the line to our accumulated input
        if !raw_input.is_empty() {
            raw_input.push('\n');
        }
        raw_input.push_str(&line);
        
        // Check if quotes are balanced
        match unclosed_quote_prompt(&raw_input) {
            None => {
                // Quotes are balanced, return the input
                return Ok(Some(raw_input.trim_end().to_string()));
            }
            Some(cont_prompt) => {
                // Quotes not balanced, continue reading with continuation prompt
                current_prompt = cont_prompt;
            }
        }
    }
}

/// Shell prompt with current directory; home is abbreviated as `~`.
fn get_prompt() -> String {
    let cwd = env::current_dir().unwrap_or_default();
    let home = env::var("HOME").unwrap_or_else(|_| String::new());

    if home.is_empty() {
        return format!("{} $ ", cwd.display());
    }

    let home_path = Path::new(&home);
    match cwd.strip_prefix(home_path) {
        Ok(suffix) => format!("~{} $ ", suffix.display()),
        Err(_) => format!("{} $ ", cwd.display()),
    }
}
