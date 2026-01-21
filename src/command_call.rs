/// Represents a parsed command call with its name, flags, and arguments.
///
/// A command call is generated from a single command segment (e.g., between semicolons).
/// Flags are separated from arguments to allow for easier command processing.
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

impl CommandCall {}

/// Parses a line of input into a sequence of command calls.
///
/// This function handles:
/// 1. Command chaining with semicolons (`;`).
/// 2. Tokenization with support for quotes and escapes.
/// 3. Separation of flags from positional arguments.
///
/// # Example
/// ```
/// let calls = parse_line("ls -la; echo \"hello world\"");
/// ```
pub fn parse_line(input: &str) -> Vec<CommandCall> {
    input
        .split(';') // Split by semicolon to support command chaining
        .filter_map(|chunk| {
            let chunk = chunk.trim();
            if chunk.is_empty() {
                return None;
            }

            let mut tokens = tokenize(chunk);
            if tokens.is_empty() {
                return None;
            }

            // The first token is always the command name
            let name = tokens.remove(0).to_lowercase();

            // Separate remaining tokens into flags and positional arguments
            let (flags, args) = separate_flags_from_args(tokens);

            Some(CommandCall { name, flags, args })
        })
        .collect()
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
            // Handle combined short flags like -al (split into -a and -l)
            // But skip long flags starting with --
            if token.len() > 2 && !token.starts_with("--") {
                for c in token.chars().skip(1) {
                    flags.push(format!("-{}", c));
                }
            } else {
                // Single flag or long flag: -a, --help
                flags.push(token);
            }
        } else {
            // Treat as a positional argument
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
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if escaped {
            current.push_str(&handle_escape(c, in_double_quote));
            escaped = false;
            continue;
        }

        match c {
            // Enter/Exit escaping state (only outside single quotes)
            '\\' if !in_single_quote => {
                escaped = true;
            }
            // Single quotes: strictly literal until the next single quote
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            // Double quotes: toggle state, allows certain escapes
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            // Whitespace: splits tokens if not inside quotes
            c if c.is_whitespace() && !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            // All other characters are part of the current token
            _ => current.push(c),
        }
    }

    // Push the final token if it exists
    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
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
}
