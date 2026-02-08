//! 0-shell: minimal Unix-style shell (read-eval-print loop).

mod command;
mod command_call;
mod history_expansion;
mod quote_state;

use command::command_list;
use command_call::{parse_line, unclosed_quote_prompt};
use history_expansion::expand_history;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::collections::HashMap;
use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

// -----------------------------------------------------------------------------
// Main REPL
// -----------------------------------------------------------------------------

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    let stderr_is_tty = stderr.is_terminal();
    let cmds = command_list();
    let mut state = ShellState::new();

    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => {
            let _ = write_error(&mut stderr, &format!("Failed to initialize line editor: {}", e), stderr_is_tty);
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to initialize editor"));
        }
    };

    loop {
        let prompt = get_prompt();
        let raw_input = match read_line_with_history(&mut rl, &prompt) {
            Ok(Some(input)) => input,
            Ok(None) => break,
            Err(e) => {
                let _ = write_error(&mut stderr, &format!("Error reading line: {}", e), stderr_is_tty);
                continue;
            }
        };

        let expanded = match expand_line(&raw_input, state.histexpand, &rl) {
            Ok(s) => s,
            Err(e) => {
                let _ = write_error(&mut stderr, &e, stderr_is_tty);
                continue;
            }
        };

        if !expanded.trim().is_empty() {
            if let Err(e) = rl.add_history_entry(expanded.as_str()) {
                let _ = write_error(&mut stderr, &format!("Warning: Failed to add to history: {}", e), stderr_is_tty);
            }
        }

        if execute_commands(&expanded, &mut state, &cmds, &mut stdout, &mut stderr, stderr_is_tty)? {
            return Ok(());
        }
    }

    Ok(())
}

/// Shell state shared across the REPL: environment and options.
struct ShellState {
    env: HashMap<String, String>,
    /// History expansion: true = `set -H`, false = `set +H`.
    histexpand: bool,
}

impl ShellState {
    fn new() -> Self {
        Self {
            env: env::vars().collect(),
            histexpand: true,
        }
    }
}

// -----------------------------------------------------------------------------
// REPL helpers: input, expansion, prompt
// -----------------------------------------------------------------------------

/// Expands history references in the line when histexpand is on; otherwise returns input as-is.
fn expand_line(
    raw_input: &str,
    histexpand: bool,
    rl: &DefaultEditor,
) -> Result<String, String> {
    if histexpand {
        expand_history(
            raw_input,
            rl.history().len(),
            |i| rl.history().get(i).map(String::as_str),
        )
    } else {
        Ok(raw_input.to_string())
    }
}

/// Reads a full line with history and continuation prompts until quotes balance. Returns `None` on EOF.
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
                raw_input.clear();
                current_prompt = prompt;
                let _ = io::stdout().write_all(b"\n");
                let _ = io::stdout().flush();
                continue;
            }
            Err(ReadlineError::Eof) => {
                return Ok(if raw_input.is_empty() {
                    None
                } else {
                    Some(raw_input.trim_end().to_string())
                });
            }
            Err(e) => return Err(e),
        };

        if !raw_input.is_empty() {
            raw_input.push('\n');
        }
        raw_input.push_str(&line);

        match unclosed_quote_prompt(&raw_input) {
            None => return Ok(Some(raw_input.trim_end().to_string())),
            Some(cont_prompt) => current_prompt = cont_prompt,
        }
    }
}

/// Prompt with current directory (home as `~`). Path in cyan when stdout is a TTY.
fn get_prompt() -> String {
    let cwd = env::current_dir().unwrap_or_default();
    let home = env::var("HOME").unwrap_or_else(|_| String::new());
    let path = if home.is_empty() {
        format!("{}", cwd.display())
    } else {
        let home_path = Path::new(&home);
        match cwd.strip_prefix(home_path) {
            Ok(suffix) => format!("~{}", suffix.display()),
            Err(_) => format!("{}", cwd.display()),
        }
    };
    if io::stdout().is_terminal() {
        format!("{}{}{} $ ", ANSI_CYAN, path, ANSI_RESET)
    } else {
        format!("{} $ ", path)
    }
}

// -----------------------------------------------------------------------------
// Command execution and builtins
// -----------------------------------------------------------------------------

/// Parses the line, runs builtins (export, set) or commands. Returns `true` if shell should exit.
fn execute_commands(
    line: &str,
    state: &mut ShellState,
    cmds: &command::CommandList,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
    stderr_is_tty: bool,
) -> io::Result<bool> {
    for call in parse_line(line, &state.env) {
        if call.name == "export" {
            do_export(&mut state.env, &call.args);
            continue;
        }
        if call.name == "set" {
            do_set(&mut state.histexpand, &call.flags);
            continue;
        }
        let result = cmds.execute(call.name, call.flags, call.args);
        if result.should_exit {
            return Ok(true);
        }
        write_command_result(stdout, stderr, &result, stderr_is_tty)?;
    }
    Ok(false)
}

/// `set [-H] [+H] ...`: -H enables history expansion, +H disables it.
fn do_set(histexpand: &mut bool, flags: &[String]) {
    for f in flags {
        match f.as_str() {
            "-H" => *histexpand = true,
            "+H" => *histexpand = false,
            _ => {}
        }
    }
}

/// `export [VAR=value | VAR] ...`: set or export variables in shell and process env.
fn do_export(env: &mut HashMap<String, String>, args: &[String]) {
    for arg in args {
        if let Some((name, value)) = arg.split_once('=') {
            let name = name.to_string();
            let value = value.to_string();
            env.insert(name.clone(), value.clone());
            let _ = env::set_var(&name, &value);
        } else if !arg.is_empty() {
            let name = arg.clone();
            let value = env
                .get(&name)
                .cloned()
                .unwrap_or_else(|_| env::var(&name).unwrap_or_default());
            env.insert(name.clone(), value.clone());
            let _ = env::set_var(&name, &value);
        }
    }
}

// -----------------------------------------------------------------------------
// I/O: ANSI and writing
// -----------------------------------------------------------------------------

const ANSI_RED: &str = "\x1b[31m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_RESET: &str = "\x1b[0m";

/// Writes an error to stderr; red when stderr is a TTY.
fn write_error(stderr: &mut impl Write, msg: &str, is_tty: bool) -> io::Result<()> {
    if is_tty {
        stderr.write_all(ANSI_RED.as_bytes())?;
    }
    stderr.write_all(msg.as_bytes())?;
    if is_tty {
        stderr.write_all(ANSI_RESET.as_bytes())?;
    }
    stderr.write_all(b"\n")?;
    stderr.flush()
}

/// Writes command stdout and stderr; stderr in red when a TTY.
fn write_command_result(
    stdout: &mut impl Write,
    stderr: &mut impl Write,
    result: &command::CommandResult,
    stderr_is_tty: bool,
) -> io::Result<()> {
    if !result.stdout.is_empty() {
        stdout.write_all(result.stdout.as_bytes())?;
        stdout.flush()?;
    }
    if !result.stderr.is_empty() {
        if stderr_is_tty {
            stderr.write_all(ANSI_RED.as_bytes())?;
        }
        stderr.write_all(result.stderr.as_bytes())?;
        if stderr_is_tty {
            stderr.write_all(ANSI_RESET.as_bytes())?;
        }
        stderr.write_all(b"\n")?;
        stderr.flush()?;
    }
    Ok(())
}
