//! `clear` - clear the terminal screen.

use std::io::{self, Write};
use super::CommandResult;

pub fn clear_callback(_flags: Vec<String>, _args: Vec<String>) -> CommandResult {
    // Write ANSI escape sequence directly to stdout for immediate effect
    // This ensures the sequence is sent to the terminal without buffering issues
    // Using VT100 Reset Device (\x1bc) which completely resets the terminal
    // This actually removes all lines from the buffer, not just hides them
    // This is more aggressive than \x1b[2J but ensures complete clearing
    let mut stdout = io::stdout();
    let _ = write!(stdout, "\x1bc");
    let _ = stdout.flush();
    
    // Return empty result since we wrote directly
    CommandResult::new()
}
