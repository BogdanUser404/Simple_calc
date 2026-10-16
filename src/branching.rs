// Copyright (C) 2026 Bogdan Yachmenev.
// License: AGPL v3.0 or later

//! Branching (conditional) logic for the math engine.
//!
//! Syntax:
//! `IF[EXPRESSION]:(EXPRESSION IF NOT ZERO):(EXPRESSION IF ZERO)`
//!
//! The condition is considered true when it evaluates to a value that is
//! NOT zero. Evaluation is lazy: only the condition and the single taken
//! branch are ever parsed and evaluated — the untaken branch is skipped
//! entirely (so errors such as division-by-zero inside it never surface).
//!
//! Fixed-point note: the engine stores every number scaled by 10^8, so a
//! literal `0` is represented internally as `0` — checking `value != 0` on
//! the raw fixed-point representation is exactly "not equal to zero".

use crate::find_if_construct;
use crate::math_proccessor::MathProcessor;
use crate::parser::ParsingEngine;

// The engine stores every number scaled by 10^8 (see module docs), so the
// tests below compare against fixed-point values via the `fx!` macro.

/// Fixed-point scale: raw units per whole number. Only referenced by the
/// test helper below, hence `cfg(test)` to avoid dead-code warnings.
#[cfg(test)]
const SCALE: i128 = 100_000_000;

/// Helper for tests: converts a plain number into its fixed-point raw form.
#[cfg(test)]
macro_rules! fx {
    ($n:expr) => {
        ($n) * SCALE
    };
}

/// Errors produced while parsing an IF construct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchError {
    /// The condition bracket `]` was never closed.
    UnterminatedCondition,
    /// Missing `:` separator after the condition / before the THEN group.
    MissingThenSeparator,
    /// Missing `:` separator between the THEN and ELSE groups.
    MissingElseSeparator,
    /// A part of the construct (condition or branch) is empty.
    EmptyPart(&'static str),
}

impl core::fmt::Display for BranchError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BranchError::UnterminatedCondition => write!(f, "IF: unterminated condition, expected ']'"),
            BranchError::MissingThenSeparator => write!(f, "IF: missing ':' after condition"),
            BranchError::MissingElseSeparator => write!(f, "IF: missing ':' between THEN and ELSE expressions"),
            BranchError::EmptyPart(name) => write!(f, "IF: empty {} expression", name),
        }
    }
}

/// A fully parsed branching construct: condition + two lazily-evaluated arms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    pub condition: String,
    pub then_expr: String,
    pub else_expr: String,
}

/// Returns the index of the first occurrence of `target` in `s[start..]`
/// that is not enclosed by any *parenthesis* group (depth counted over
/// `(`/`)` only). Square brackets are intentionally NOT tracked here:
/// inside an IF construct every `[`/`]` pair has already been consumed by
/// the caller's scanning logic, so any remaining square bracket must be
/// treated as a plain character. This also keeps byte-slice indices on
/// UTF-8 boundaries (multi-byte operators like `¬|` are skipped safely).
fn find_colon_at_depth(s: &[u8], start: usize, target: u8) -> Option<usize> {
    let mut depth = 0i32;
    let mut i = start;
    while i < s.len() {
        let c = s[i];
        if c == b'(' {
            depth += 1;
        } else if c == b')' {
            depth -= 1;
        } else if c == target && depth == 0 {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Given the index of an opening parenthesis `open_idx`, returns the index
/// of its matching `)` (nesting-aware), or `None` if unbalanced.
fn matching_close_paren(s: &[u8], open_idx: usize) -> Option<usize> {
    if s.get(open_idx) != Some(&b'(') {
        return None;
    }
    let mut depth = 0i32;
    let mut i = open_idx;
    while i < s.len() {
        let c = s[i];
        if c == b'(' {
            depth += 1;
        } else if c == b')' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

impl Branch {
    /// Parses the *body* of an IF construct — everything that follows the
    /// `IF[` prefix — i.e. a string of the form:
    /// `CONDITION]:(THEN):(ELSE)`
    ///
    /// All three parts must be present; THEN and ELSE must each be wrapped
    /// in parentheses. Nested parentheses inside any part are handled via
    /// depth tracking.
    pub fn parse(if_body: &str) -> Result<Self, BranchError> {
        let bytes = if_body.as_bytes();

        // 1. Condition: text up to its closing `]`. Nested complete IF
        //    constructs inside the condition are skipped atomically so that
        //    their inner `:` separators never end the outer condition scan.
        let mut pos = 0usize;
        let cond_close = loop {
            if pos >= bytes.len() {
                return Err(BranchError::UnterminatedCondition);
            }
            // Skip an inner "IF[..]:(..):(..)" as a single unit.
            if bytes[pos] == b'I'
                && bytes.get(pos + 1) == Some(&b'F')
                && bytes.get(pos + 2) == Some(&b'[')
            {
                if let Some((_, inner_end)) = find_if_construct(if_body, pos) {
                    pos = inner_end;
                    continue;
                }
                // Malformed inner construct — fall through to normal scan.
            }
            if bytes[pos] == b']' {
                break pos;
            }
            pos += 1;
        };
        let condition = if_body[..cond_close].trim().to_string();
        if condition.is_empty() {
            return Err(BranchError::EmptyPart("condition"));
        }

        // 2. First `:` at depth 0 after the condition — introduces THEN.
        let sep1 = find_colon_at_depth(bytes, cond_close + 1, b':').ok_or(BranchError::MissingThenSeparator)?;
        let then_open = sep1 + 1;
        if *bytes.get(then_open).ok_or(BranchError::MissingThenSeparator)? != b'(' {
            return Err(BranchError::MissingThenSeparator);
        }
        let then_close = matching_close_paren(bytes, then_open).ok_or(BranchError::MissingThenSeparator)?;
        let then_expr = if_body[then_open + 1..then_close].trim().to_string();
        if then_expr.is_empty() {
            return Err(BranchError::EmptyPart("THEN"));
        }

        // 3. Second `:` at depth 0 after the THEN group — introduces ELSE.
        let sep2 = find_colon_at_depth(bytes, then_close + 1, b':').ok_or(BranchError::MissingElseSeparator)?;
        let else_open = sep2 + 1;
        if *bytes.get(else_open).ok_or(BranchError::MissingElseSeparator)? != b'(' {
            return Err(BranchError::MissingElseSeparator);
        }
        let else_close = matching_close_paren(bytes, else_open).ok_or(BranchError::UnterminatedCondition)?;
        let else_expr = if_body[else_open + 1..else_close].trim().to_string();
        if else_expr.is_empty() {
            return Err(BranchError::EmptyPart("ELSE"));
        }

        Ok(Self { condition, then_expr, else_expr })
    }

    /// Lazily evaluates the construct:
    ///   * evaluate ONLY the condition first;
    ///   * then evaluate EXACTLY ONE arm (the chosen one);
    ///   * the other arm is never parsed nor evaluated.
    ///
    /// `eval_fn` decides how a sub-expression string becomes a fixed-point
    /// integer (this lets the caller expand `{placeholder}` variables first).
    pub fn eval_lazily_with(&self, mut eval_fn: impl FnMut(&str) -> Result<i128, String>) -> Result<i128, String> {
        let cond_val = eval_fn(&self.condition)?;
        // Non-zero (in the engine's fixed-point representation) means "true".
        let taken = if cond_val != 0 { &self.then_expr } else { &self.else_expr };
        eval_fn(taken)
    }

    /// Convenience wrapper: lazy evaluation using the engine's own parser
    /// and processor directly (no placeholder expansion).
    pub fn eval_lazily(&self) -> Result<i128, String> {
        self.eval_lazily_with(Self::eval_subexpression)
    }

    /// Evaluates a standalone sub-expression through the engine's
    /// recursive-stack math processor.
    pub fn eval_subexpression(expr: &str) -> Result<i128, String> {
        let mut processor = MathProcessor::new();
        ParsingEngine::parse_into_processor(expr, &mut processor)
            .map_err(|e| format!("Parse error in branch expression '{}': {}", expr, e))?;
        processor
            .finalize()
            .map_err(|e| format!("Evaluation error in branch expression '{}': {}", expr, e))
    }
}

/// Parses and lazily evaluates a complete `IF[...]:(...):(...)` construct
/// given its body (the text after `IF[`).
pub fn evaluate_branch_from_body(if_body: &str) -> Result<i128, String> {
    let branch = Branch::parse(if_body).map_err(|e| e.to_string())?;
    branch.eval_lazily()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_branch() {
        // Body form: everything after "IF[".
        let b = Branch::parse("1+1]:(2*3):(4+5)").unwrap();
        assert_eq!(b.condition, "1+1");
        assert_eq!(b.then_expr, "2*3");
        assert_eq!(b.else_expr, "4+5");
    }

    #[test]
    fn picks_then_when_nonzero() {
        assert_eq!(evaluate_branch_from_body("5-4]:(10+1):(20+2)").unwrap(), fx!(11));
    }

    #[test]
    fn picks_else_when_zero() {
        assert_eq!(evaluate_branch_from_body("5-5]:(10+1):(20+2)").unwrap(), fx!(22));
    }

    #[test]
    fn lazy_evaluation_skips_other_branch() {
        // THEN divides by zero — but the condition is false, so it is skipped.
        assert_eq!(evaluate_branch_from_body("1-1]:(1/0):(7*7)").unwrap(), fx!(49));
        // Mirror case: ELSE is broken but never evaluated.
        assert_eq!(evaluate_branch_from_body("2-1]:(7*7):(1/0)").unwrap(), fx!(49));
    }

    #[test]
    fn nested_parentheses_in_branches() {
        assert_eq!(evaluate_branch_from_body("2]:( (3+4)*2 ):(0*(1+2))").unwrap(), fx!(14));
    }

    #[test]
    fn fractional_condition_nonzero_is_true() {
        // 0.5 != 0 in fixed-point ⇒ THEN arm is taken.
        assert_eq!(evaluate_branch_from_body("0.5]:(1+1):(2+2)").unwrap(), fx!(2));
    }

    #[test]
    fn errors_on_missing_separator() {
        assert!(Branch::parse("1]:(2)").is_err());
        assert!(Branch::parse("1]:(2)).").is_err());
        assert!(Branch::parse("1:(2):(3)").is_err()); // no ']' closing the condition
    }
}
