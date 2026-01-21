mod command;
mod command_call;

use command::{
    command_list
};

use command_call::{
    parse_line
};

use std::io::{
    Read, Write, stdin, stdout, stderr, Result
};

// split commands.
// tokenizer
// - no quote, single quote, double quote
// parser

fn main() -> Result<()> {
    let mut stdin = stdin();
    let mut stdout = stdout();
    let mut stderr = stderr();
    let mut buffer = [0; 1024];
    let cmds = command_list();

    loop {
        stdout.write_all(b"$ ")?;
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
