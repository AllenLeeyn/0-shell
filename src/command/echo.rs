//! `echo` - display a line of text.

use super::CommandResult;

pub fn echo_callback(flags: Vec<String>, args: Vec<String>) -> CommandResult {
    let interpret = flags.iter().any(|f| f == "-e");
    let mut result = CommandResult::new();

    let input = args.join(" ");

    if !interpret {
        result.stdout = format!("{}\n", input);
        return result;
    }

    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('c') => {
                    result.stdout = output;
                    return result;
                }
                Some(next) => {
                    if let Some(mapped) = map_echo_escape(next) {
                        output.push(mapped);
                    } else {
                        output.push('\\');
                        output.push(next);
                    }
                }
                None => output.push('\\'),
            }
        } else {
            output.push(c);
        }
    }

    result.stdout = format!("{}\n", output);
    result
}

fn map_echo_escape(c: char) -> Option<char> {
    match c {
        'a' => Some('\x07'),
        'b' => Some('\x08'),
        'e' => Some('\x1b'),
        'f' => Some('\x0c'),
        'n' => Some('\n'),
        'r' => Some('\r'),
        't' => Some('\t'),
        'v' => Some('\x0b'),
        '\\' => Some('\\'),
        _ => None,
    }
}
