//! Simple history expansion: `!!`, `!n`, and `!-n`.
//!
//! Expansion is not performed inside single quotes or when `!` is escaped by backslash.
//! Matches the behavior described in docs/HISTORY_EXPANSION.md.

use crate::quote_state::QuoteState;

/// Result of history expansion: either the expanded string or an error message.
pub type ExpandResult = Result<String, String>;

/// Expands history references in `input` using direct access to history.
///
/// - `!!` → previous command (last entry)
/// - `!n` → command number n (1-based; e.g. `!1` is first entry)
/// - `!-n` → the command n steps back (e.g. `!-1` is previous, `!-2` is two ago)
///
/// `history_len` is the number of entries; `get_entry(i)` returns the entry at index `i` (0-based), or `None` if out of range.
/// History is not expanded inside single quotes. A backslash before `!` prevents expansion.
/// If any reference is invalid (e.g. empty history, index out of range), returns `Err` with a message.
pub fn expand_history<'a, F>(input: &str, history_len: usize, get_entry: F) -> ExpandResult
where
    F: Fn(usize) -> Option<&'a str>,
{
    let mut out = String::new();
    let mut state = QuoteState::default();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if state.escaped {
            out.push(c);
            state.escaped = false;
            continue;
        }

        state.advance(c);

        if c == '!' && state.should_expand_bang() {
            let replacement = expand_one(&mut chars, history_len, &get_entry)?;
            out.push_str(&replacement);
            continue;
        }

        if c == '\\' && !state.in_single_quote {
            // advance() set escaped; don't push backslash, next char will be literal
            continue;
        }

        out.push(c);
    }

    Ok(out)
}

/// Parses one history designator after the leading `!` and returns the replacement text.
fn expand_one<'a, F>(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    history_len: usize,
    get_entry: &F,
) -> ExpandResult
where
    F: Fn(usize) -> Option<&'a str>,
{
    if history_len == 0 {
        return Err("bash: !!: no previous command".to_string());
    }

    // !!
    if chars.peek().copied() == Some('!') {
        chars.next();
        let s = get_entry(history_len - 1).ok_or("bash: !!: no previous command")?;
        return Ok(s.to_string());
    }

    // !-n, !+n, !n (optional sign then digits)
    if matches!(chars.peek().copied(), Some('-') | Some('+') | Some(c) if c.is_ascii_digit()) {
        let n = read_number(chars)?;
        if n == 0 {
            return Err("bash: !0: command not found".to_string());
        }
        let (idx, err_label): (usize, String) = if n < 0 {
            let abs_n = n.unsigned_abs() as usize;
            if abs_n > history_len {
                return Err(format!(
                    "bash: !-{}: event not found (history has {} entries)",
                    abs_n, history_len
                ));
            }
            (history_len - abs_n, format!("-{}", abs_n))
        } else {
            if n as usize > history_len {
                return Err(format!(
                    "bash: !{}: event not found (history has {} entries)",
                    n, history_len
                ));
            }
            ((n as usize) - 1, n.to_string())
        };
        let s = get_entry(idx).ok_or_else(|| format!("bash: !{}: event not found", err_label))?;
        return Ok(s.to_string());
    }

    // No other designators (e.g. !string) for this simple implementation
    Err("bash: !: event not found".to_string())
}

/// Parses an optional leading `-` or `+`, then one or more ASCII digits.
/// Returns a signed value: negative for `-n`, positive for `n` or `+n`.
fn read_number(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Result<i64, String> {
    let mut s = String::new();
    match chars.peek().copied() {
        Some('-') => {
            s.push(chars.next().unwrap());
        }
        Some('+') => {
            chars.next();
            // don't push '+' into s; we'll parse as positive
        }
        _ => {}
    }
    while chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        s.push(chars.next().unwrap());
    }
    if s.is_empty() || s == "-" {
        return Err("bash: invalid history reference".to_string());
    }
    s.parse::<i64>()
        .map_err(|_| "bash: invalid history reference".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hist(entries: &[&str]) -> Vec<String> {
        entries.iter().map(|s| s.to_string()).collect()
    }

    fn expand_with(history: &[String], input: &str) -> ExpandResult {
        expand_history(input, history.len(), |i| history.get(i).map(String::as_str))
    }

    #[test]
    fn test_double_bang() {
        let h = hist(&["echo first", "echo second"]);
        assert_eq!(expand_with(&h, "!!").unwrap(), "echo second");
    }

    #[test]
    fn test_bang_n() {
        let h = hist(&["echo one", "echo two", "echo three"]);
        assert_eq!(expand_with(&h, "!1").unwrap(), "echo one");
        assert_eq!(expand_with(&h, "!2").unwrap(), "echo two");
        assert_eq!(expand_with(&h, "!3").unwrap(), "echo three");
    }

    #[test]
    fn test_bang_minus_n() {
        let h = hist(&["echo one", "echo two", "echo three"]);
        assert_eq!(expand_with(&h, "!-1").unwrap(), "echo three");
        assert_eq!(expand_with(&h, "!-2").unwrap(), "echo two");
        assert_eq!(expand_with(&h, "!-3").unwrap(), "echo one");
    }

    #[test]
    fn test_bang_plus_n() {
        let h = hist(&["echo one", "echo two", "echo three"]);
        assert_eq!(expand_with(&h, "!+1").unwrap(), "echo one");
        assert_eq!(expand_with(&h, "!+2").unwrap(), "echo two");
        assert_eq!(expand_with(&h, "!+3").unwrap(), "echo three");
    }

    #[test]
    fn test_no_expand_inside_single_quotes() {
        let h = hist(&["echo previous"]);
        assert_eq!(expand_with(&h, "echo '!!'").unwrap(), "echo '!!'");
    }

    #[test]
    fn test_empty_history() {
        let h: Vec<String> = vec![];
        assert!(expand_history("!!", 0, |_| None).is_err());
    }

    #[test]
    fn test_event_not_found() {
        let h = hist(&["echo one"]);
        assert!(expand_with(&h, "!2").is_err());
        assert!(expand_with(&h, "!-2").is_err());
    }

    #[test]
    fn test_escaped_bang() {
        let h = hist(&["echo previous"]);
        assert_eq!(expand_with(&h, r"\!!").unwrap(), "!!");
    }
}
