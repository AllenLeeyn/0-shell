//! 0-shell: minimal Unix-style shell (read-eval-print loop).

mod command;
mod command_call;

use command::command_list;
use command_call::{parse_line, unclosed_quote_prompt};
use std::env;
use std::io::{self, Write};
use std::path::Path;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    let cmds = command_list();

    loop {
        write!(stdout, "{}", get_prompt())?;
        stdout.flush()?;

        let raw_input = match read_line_with_continuation(&mut stdout)? {
            None => break,
            Some(s) => s,
        };

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

/// Reads a full line from stdin, showing continuation prompts (e.g. `dquote> `) until quotes balance.
/// Returns `None` on EOF.
fn read_line_with_continuation<W: Write>(out: &mut W) -> io::Result<Option<String>> {
    let mut raw_input = String::new();
    loop {
        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 {
            break Ok(if raw_input.is_empty() {
                None
            } else {
                Some(raw_input.trim_end().to_string())
            });
        }
        raw_input.push_str(&line);
        match unclosed_quote_prompt(&raw_input) {
            None => break Ok(Some(raw_input.trim_end().to_string())),
            Some(cont_prompt) => {
                out.write_all(cont_prompt.as_bytes())?;
                out.flush()?;
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
