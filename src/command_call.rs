//! Line parsing and tokenization for 0-shell.
//!
//! Handles semicolon-separated commands, quoted strings, backslash escapes,
//! environment variable expansion (`$VAR`, `${VAR}`), and separation of flags from args.

use std::collections::HashMap;

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
/// Handles: command chaining with `;`, tokenization (quotes/escapes, env expansion), and separation of flags vs args.
pub fn parse_line(input: &str, env: &HashMap<String, String>) -> Vec<CommandCall> {
    input
        .split(';')
        .filter_map(|chunk| parse_chunk(chunk.trim(), env))
        .collect()
}

/// Parses one semicolon-separated segment into a single command call, or `None` if empty.
fn parse_chunk(chunk: &str, env: &HashMap<String, String>) -> Option<CommandCall> {
    if chunk.is_empty() {
        return None;
    }
    let mut tokens = tokenize(chunk, env);
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
/// - Single quotes (`'`): Everything inside is treated literally (no expansion).
/// - Double quotes (`"`): Supports backslash escaping for `"`, `\`, and `$`; `$VAR` is expanded.
/// - Backslash escapes (`\`): Outside of quotes, escapes any following character.
/// - Environment variables: `$VAR` and `${VAR}` expanded (empty if unset); not expanded inside single quotes.
/// - Whitespace: Separates tokens unless escaped or quoted.
pub fn tokenize(input: &str, env: &HashMap<String, String>) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut state = QuoteState::default();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if state.escaped {
            current.push_str(&handle_escape(c, state.in_double_quote));
            state.escaped = false;
            continue;
        }

        state.advance(c);

        match c {
            '$' if !state.in_single_quote => {
                let name = read_var_name(&mut chars);
                current.push_str(get_env_var(env, &name));
            }
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

/// Returns the value of an environment variable, or the empty string if unset.
fn get_env_var(env: &HashMap<String, String>, name: &str) -> &str {
    env.get(name).map(String::as_str).unwrap_or("")
}

/// Reads a variable name after `$`: either `${name}` or `name` (alphanumeric + underscore).
/// Variable names cannot contain quotes or backslashes, so we don't need to update `QuoteState`.
fn read_var_name(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut name = String::new();
    match chars.peek().copied() {
        Some('{') => {
            chars.next();
            while let Some(c) = chars.next() {
                if c == '}' {
                    break;
                }
                name.push(c);
            }
        }
        Some(c) if c.is_alphabetic() || c == '_' => {
            name.push(c);
            chars.next();
            while let Some(&c) = chars.peek() {
                if !c.is_alphanumeric() && c != '_' {
                    break;
                }
                chars.next();
                name.push(c);
            }
        }
        _ => {}
    }
    name
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

    fn empty_env() -> HashMap<String, String> {
        HashMap::new()
    }

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("ls -la /home", &empty_env());
        assert_eq!(tokens, vec!["ls", "-la", "/home"]);
    }

    #[test]
    fn test_tokenize_quotes() {
        let tokens = tokenize("echo \"hello world\" 'single quote'", &empty_env());
        assert_eq!(tokens, vec!["echo", "hello world", "single quote"]);
    }

    #[test]
    fn test_tokenize_quote_inside_quote() {
        // Single quote inside double quotes is literal (bash behavior)
        let tokens = tokenize("echo \"'something\"", &empty_env());
        assert_eq!(tokens, vec!["echo", "'something"]);
        let tokens = tokenize("echo '\"double\"'", &empty_env());
        assert_eq!(tokens, vec!["echo", "\"double\""]);
    }

    #[test]
    fn test_tokenize_escapes() {
        let tokens = tokenize("echo \\\"hello\\ world\\\"", &empty_env());
        assert_eq!(tokens, vec!["echo", "\"hello world\""]);
    }

    #[test]
    fn test_tokenize_env_expansion() {
        let mut env = HashMap::new();
        env.insert("HOME".to_string(), "/home/user".to_string());
        env.insert("X".to_string(), "val".to_string());
        assert_eq!(tokenize("echo $HOME", &env), vec!["echo", "/home/user"]);
        assert_eq!(tokenize("echo ${HOME}", &env), vec!["echo", "/home/user"]);
        assert_eq!(tokenize("echo a$X b", &env), vec!["echo", "aval", "b"]);
    }

    #[test]
    fn test_tokenize_env_no_expand_single_quote() {
        let mut env = HashMap::new();
        env.insert("HOME".to_string(), "/home/user".to_string());
        let tokens = tokenize("echo '$HOME'", &env);
        assert_eq!(tokens, vec!["echo", "$HOME"]);
    }

    #[test]
    fn test_tokenize_env_unset_empty() {
        let tokens = tokenize("echo $MISSING", &empty_env());
        assert_eq!(tokens, vec!["echo", ""]);
    }

    #[test]
    fn test_parse_line_chaining() {
        let calls = parse_line("ls -l; echo hi", &empty_env());
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "ls");
        assert_eq!(calls[0].flags, vec!["-l"]);
        assert_eq!(calls[1].name, "echo");
        assert_eq!(calls[1].args, vec!["hi"]);
    }

    #[test]
    fn test_parse_line_flags_expansion() {
        let calls = parse_line("ls -la /tmp", &empty_env());
        assert_eq!(calls[0].flags, vec!["-l", "-a"]);
        assert_eq!(calls[0].args, vec!["/tmp"]);
    }

    #[test]
    fn test_parse_line_long_flags() {
        let calls = parse_line("ls --all /tmp", &empty_env());
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
