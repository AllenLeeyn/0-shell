//! Quote and escape state for shell parsing.
//!
//! Shared by history expansion and line tokenization: tracks single/double quotes
//! and backslash escape so we know when to expand `!` or treat characters literally.

/// Continuation prompt when input has unclosed quote (bash PS2 style: just `> `).
pub const CONTINUATION_PROMPT: &str = "> ";

/// Tracks quote/escape state for a single left-to-right pass over input.
#[derive(Default)]
pub struct QuoteState {
    pub in_single_quote: bool,
    pub in_double_quote: bool,
    pub escaped: bool,
}

impl QuoteState {
    /// Update state for the next character (quote toggles, backslash escape).
    pub fn advance(&mut self, c: char) {
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

    /// Returns the continuation prompt if a quote is still open, otherwise `None`.
    pub fn unclosed_prompt(&self) -> Option<&'static str> {
        if self.in_double_quote || self.in_single_quote {
            Some(CONTINUATION_PROMPT)
        } else {
            None
        }
    }

    /// True when `!` should trigger history expansion (not inside single quotes, not escaped).
    pub fn should_expand_bang(&self) -> bool {
        !self.in_single_quote && !self.escaped
    }
}
