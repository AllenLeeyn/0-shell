mod command;
mod command_call;

use std::env;
use command::{
    command_list
};

use command_call::{
    parse_line
};

use std::io::{
    Read, Write, stdin, stdout, stderr, Result
};

fn main() -> Result<()> {
    let mut stdin = stdin();
    let mut stdout = stdout();
    let mut stderr = stderr();
    let mut buffer = [0; 1024];
    let cmds = command_list();

    loop {
        let prompt = get_prompt(); 
        stdout.write_all(prompt.as_bytes())?;
        stdout.flush()?;

        let n = stdin.read(&mut buffer)?;
        if n == 0 { break; }

        let raw_input = String::from_utf8_lossy(&buffer[..n]);

        // Layer 1: Parse the line into individual calls
        let calls = parse_line(&raw_input);

        // Layer 2: Dispatch calls one by one
        for call in calls {
            match cmds.execute(call.name, call.args) {
                Ok(output) => {
                    if output == "EXIT_SHELL" { return Ok(()); }
                    stdout.write_all(output.as_bytes())?;
                    stdout.flush()?;
                }
                Err(e) => {
                    stderr.write_all(format!("{}\n", e).as_bytes())?;
                    stderr.flush()?;
                }
            }
        }
    }

    Ok(())
}

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