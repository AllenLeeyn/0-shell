//! Line parsing and tokenization for 0-shell.
//!
//! Handles semicolon-separated commands, quoted strings, backslash escapes,
//! and separation of flags from positional arguments.

/// A parsed command: name, flags, and args (one `;`-separated segment).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandCall {
    /// The name of the command (e.g., "ls", "echo"). Always lowercase.
    pub name: String,
    /// Individual flags found in the command (e.g., "-l", "-a").
    /// Short flags combined as "-la" are expanded into ["-l", "-a"].
    pub flags: Vec<String>,
    /// Positional arguments for the command (e.g., file paths, text).
    pub args: Vec<String>,
}

/// Tracks quote/escape state for tokenization and unclosed-quote detection.
#[derive(Default)]
struct QuoteState {
    in_single_quote: bool,
    in_double_quote: bool,
    escaped: bool,
}

impl QuoteState {
    fn advance(&mut self, c: char) {
        if self.escaped {
            self.escaped = false;
            return;
        }
        if c == '\\' && !self.in_single_quote {
            self.escaped = true;
            return;
        }
        if c == '\'' && !self.in_double_quote {
            self.in_single_quote = !self.in_single_quote;
        } else if c == '"' && !self.in_single_quote {
            self.in_double_quote = !self.in_double_quote;
        }
    }

    fn unclosed_prompt(&self) -> Option<&'static str> {
        if self.in_double_quote || self.in_single_quote {
            Some(CONTINUATION_PROMPT)
        } else {
            None
        }
    }
}

/// Continuation prompt when input has unclosed quote (bash PS2 style: just `> `).
const CONTINUATION_PROMPT: &str = "> ";

/// Parses a line of input into a sequence of command calls.
///
/// Handles: command chaining with `;`, tokenization (quotes/escapes), and separation of flags vs args.
pub fn parse_line(input: &str) -> Vec<CommandCall> {
    input.split(';').filter_map(|chunk| parse_chunk(chunk.trim())).collect()
}

/// Parses one semicolon-separated segment into a single command call, or `None` if empty.
fn parse_chunk(chunk: &str) -> Option<CommandCall> {
    if chunk.is_empty() {
        return None;
    }
    let mut tokens = tokenize(chunk);
    if tokens.is_empty() {
        return None;
    }
    let name = tokens.remove(0).to_lowercase();
    let (flags, args) = separate_flags_from_args(tokens);
    Some(CommandCall { name, flags, args })
}

/// Separates command tokens into flags and positional arguments.
///
/// Flags are tokens starting with `-`. Short flags (single `-` followed by multiple characters)
/// are automatically expanded (e.g., `-al` -> `["-a", "-l"]`).
/// Long flags (starting with `--`) are preserved as-is.
fn separate_flags_from_args(tokens: Vec<String>) -> (Vec<String>, Vec<String>) {
    let mut flags = Vec::new();
    let mut args = Vec::new();

    for token in tokens {
        if token.starts_with('-') && token != "-" {
            if token.len() > 2 && !token.starts_with("--") {
                flags.extend(token.chars().skip(1).map(|c| format!("-{}", c)));
            } else {
                flags.push(token);
            }
        } else {
            args.push(token);
        }
    }

    (flags, args)
}

/// Tokenizes a raw command string into individual arguments.
///
/// This implementation supports:
/// - Single quotes (`'`): Everything inside is treated literally.
/// - Double quotes (`"`): Supports backslash escaping for `"`, `\`, and `$`.
/// - Backslash escapes (`\`): Outside of quotes, escapes any following character.
/// - Whitespace: Separates tokens unless escaped or quoted.
pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut state = QuoteState::default();

    for c in input.chars() {
        if state.escaped {
            current.push_str(&handle_escape(c, state.in_double_quote));
            state.escaped = false;
            continue;
        }

        state.advance(c);

        match c {
            '\\' if !state.in_single_quote => { /* advance() set escaped; don't push \ */ }
            '\'' if state.in_double_quote => current.push(c), // literal ' inside "..."
            '"' if state.in_single_quote => current.push(c),  // literal " inside '...'
            '\'' | '"' => { /* delimiter; already toggled in advance */ }
            c if c.is_whitespace() && !state.in_single_quote && !state.in_double_quote => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// Returns the continuation prompt if the input has an unclosed quote, otherwise `None`.
pub fn unclosed_quote_prompt(input: &str) -> Option<&'static str> {
    let mut state = QuoteState::default();
    for c in input.chars() {
        state.advance(c);
    }
    state.unclosed_prompt()
}

/// Logic for handling backslash escape sequences.
///
/// Inside double quotes, only `"`, `\`, and `$` are special when escaped.
/// Outside quotes, any character is treated literally when escaped.
fn handle_escape(c: char, in_double_quote: bool) -> String {
    if in_double_quote {
        match c {
            // These characters lose their special meaning in "" when escaped
            '"' | '\\' | '$' => c.to_string(),
            // Other characters retain the backslash (standard Bash behavior)
            _ => format!("\\{}", c),
        }
    } else {
        // Outside of quotes, \ simply makes the next char literal
        c.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("ls -la /home");
        assert_eq!(tokens, vec!["ls", "-la", "/home"]);
    }

    #[test]
    fn test_tokenize_quotes() {
        let tokens = tokenize("echo \"hello world\" 'single quote'");
        assert_eq!(tokens, vec!["echo", "hello world", "single quote"]);
    }

    #[test]
    fn test_tokenize_quote_inside_quote() {
        // Single quote inside double quotes is literal (bash behavior)
        let tokens = tokenize("echo \"'something\"");
        assert_eq!(tokens, vec!["echo", "'something"]);
        let tokens = tokenize("echo '\"double\"'");
        assert_eq!(tokens, vec!["echo", "\"double\""]);
    }

    #[test]
    fn test_tokenize_escapes() {
        let tokens = tokenize("echo \\\"hello\\ world\\\"");
        assert_eq!(tokens, vec!["echo", "\"hello world\""]);
    }

    #[test]
    fn test_parse_line_chaining() {
        let calls = parse_line("ls -l; echo hi");
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "ls");
        assert_eq!(calls[0].flags, vec!["-l"]);
        assert_eq!(calls[1].name, "echo");
        assert_eq!(calls[1].args, vec!["hi"]);
    }

    #[test]
    fn test_parse_line_flags_expansion() {
        let calls = parse_line("ls -la /tmp");
        assert_eq!(calls[0].flags, vec!["-l", "-a"]);
        assert_eq!(calls[0].args, vec!["/tmp"]);
    }

    #[test]
    fn test_parse_line_long_flags() {
        let calls = parse_line("ls --all /tmp");
        assert_eq!(calls[0].flags, vec!["--all"]);
    }

    #[test]
    fn test_unclosed_quote_prompt() {
        assert_eq!(unclosed_quote_prompt("echo \"hello"), Some("> "));
        assert_eq!(unclosed_quote_prompt("echo 'hello"), Some("> "));
        assert_eq!(unclosed_quote_prompt("echo \"hello\""), None);
        assert_eq!(unclosed_quote_prompt("echo 'hello'"), None);
    }
}
