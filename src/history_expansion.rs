//! Simple history expansion: `!!`, `!n`, `!-n`, and `!+n`.
//!
//! Not expanded inside single quotes or when `!` is escaped. See docs/HISTORY_EXPANSION.md.

use crate::quote_state::QuoteState;

/// Result of history expansion: expanded string or a bash-style error message.
pub type ExpandResult = Result<String, String>;

// -----------------------------------------------------------------------------
// Public API
// -----------------------------------------------------------------------------

/// Expands history references in `input` using direct access to history.
///
/// - `!!` → previous command (last entry)
/// - `!n` / `!+n` → command number n (1-based)
/// - `!-n` → the command n steps back
///
/// `get_entry(i)` returns the entry at 0-based index `i`, or `None` if out of range.
/// No expansion inside single quotes; backslash before `!` inhibits expansion.
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
            let replacement = parse_designator(&mut chars, history_len, &get_entry)?;
            out.push_str(&replacement);
            continue;
        }

        if c == '\\' && !state.in_single_quote {
            continue; // next char will be literal
        }

        out.push(c);
    }

    Ok(out)
}

// -----------------------------------------------------------------------------
// Designator parsing (!!, !n, !-n, !+n)
// -----------------------------------------------------------------------------

/// Parses one designator after the leading `!`; returns replacement text or error.
fn parse_designator<'a, F>(
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

    if chars.peek().copied() == Some('!') {
        chars.next();
        return get_entry(history_len - 1)
            .map(|s| s.to_string())
            .ok_or_else(|| "bash: !!: no previous command".to_string());
    }

    if matches!(chars.peek().copied(), Some('-') | Some('+') | Some(c) if c.is_ascii_digit()) {
        let n = read_number(chars)?;
        if n == 0 {
            return Err("bash: !0: command not found".to_string());
        }
        let (idx, err_label) = resolve_index(history_len, n)?;
        return get_entry(idx)
            .map(|s| s.to_string())
            .ok_or_else(|| format!("bash: !{}: event not found", err_label));
    }

    Err("bash: !: event not found".to_string())
}

/// Converts a signed history offset to a 0-based index and error label. `n < 0` ⇒ !-n, `n > 0` ⇒ !n/!+n.
fn resolve_index(history_len: usize, n: i64) -> Result<(usize, String), String> {
    if n < 0 {
        let abs_n = n.unsigned_abs() as usize;
        if abs_n > history_len {
            return Err(format!(
                "bash: !-{}: event not found (history has {} entries)",
                abs_n, history_len
            ));
        }
        Ok((history_len - abs_n, format!("-{}", abs_n)))
    } else {
        let n = n as usize;
        if n > history_len {
            return Err(format!(
                "bash: !{}: event not found (history has {} entries)",
                n, history_len
            ));
        }
        Ok((n - 1, n.to_string()))
    }
}

// -----------------------------------------------------------------------------
// Number parsing (optional sign + digits)
// -----------------------------------------------------------------------------

/// Parses optional `-` or `+` then one or more ASCII digits. Returns signed value.
fn read_number(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Result<i64, String> {
    let mut s = String::new();
    match chars.peek().copied() {
        Some('-') => s.push(chars.next().unwrap()),
        Some('+') => {
            chars.next();
        }
        _ => {}
    }
    while chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        s.push(chars.next().unwrap());
    }
    if s.is_empty() || s == "-" {
        return Err("bash: invalid history reference".to_string());
    }
    s.parse::<i64>().map_err(|_| "bash: invalid history reference".to_string())
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

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
    fn test_escaped_bang() {
        let h = hist(&["echo previous"]);
        assert_eq!(expand_with(&h, r"\!!").unwrap(), "!!");
    }

    #[test]
    fn test_empty_history() {
        assert!(expand_history("!!", 0, |_| None).is_err());
    }

    #[test]
    fn test_event_not_found() {
        let h = hist(&["echo one"]);
        assert!(expand_with(&h, "!2").is_err());
        assert!(expand_with(&h, "!-2").is_err());
    }
}
