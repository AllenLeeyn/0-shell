pub struct CommandCall {
    pub name: String,
    pub args: Vec<String>,
}

pub fn parse_line(input: &str) -> Vec<CommandCall> {
    input
        .split(';') // Split by semicolon
        .filter_map(|chunk| {
            let mut tokens = tokenize(chunk.trim());
            
            if tokens.is_empty() {
                return None;
            }

            // The first token is the command name
            let name = tokens.remove(0);
            // The rest are the arguments
            let args = tokens;

            Some(CommandCall { name, args })
        })
        .collect()
}

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
            // Enter/Exit escaping state (only inside double quotes or naked text)
            '\\' if !in_single_quote => {
                escaped = true;
            }
            // Single quotes: strictly literal
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            // Double quotes: toggle state
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            // Whitespace: only splits if NOT inside any quotes
            c if c.is_whitespace() && !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            // Any other character
            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn handle_escape(c: char, in_double_quote: bool) -> String {
    if in_double_quote {
        match c {
            'n' => "\n".to_string(),
            't' => "\t".to_string(),
            '"' | '\\' | '$' => c.to_string(),
            // POSIX rule: if not a special char in "", keep backslash
            _ => format!("\\{}", c),
        }
    } else {
        // Naked context: backslash always escapes the next char literally
        c.to_string()
    }
}