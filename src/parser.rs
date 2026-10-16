// Copyright (C) 2026 Bogdan Yachmenev.
// License: AGPL v3.0 or later
pub struct Lexer<'a> {
input: std::iter::Peekable<std::str::Chars<'a>>,
}
/// Reserved comparison operator words (lowercase form). Matched greedily
/// by `try_read_comparison_word` before any generic identifier scan, so
/// `БОЛЬШЕ`, `МЕНЬШЕ`, `РАВНО` and their variants always tokenize as
/// operators instead of variable names.
const RESERVED_COMPARISON_WORDS: &[&str] = &[
"больше", "меньше", "равно",
"не_равно", "не_больше", "не_меньше", "не_и", "не_или",
];

impl<'a> Lexer<'a> {
pub fn new(input: &'a str) -> Self {
Self {
input: input.chars().peekable(),
}
}
pub fn next_token(&mut self) -> Result<Option<String>, &'static str> {
 while let Some(&c) = self.input.peek() {
 if c.is_whitespace() {
 self.input.next();
 } else {
 break;
 }
 }
 let c = match self.input.next() {
 Some(ch) => ch,
 None => return Ok(None),
 };
 // 1. Numbers with floating point auto-scaled to exactly 10^8
 if c.is_ascii_digit() {
 let mut int_digits = String::new();
 let mut frac_digits = String::new();
 int_digits.push(c);
 while let Some(&next_c) = self.input.peek() {
 if next_c.is_ascii_digit() {
 int_digits.push(self.input.next().unwrap());
 } else {
 break;
 }
 }
 if let Some(&'.') = self.input.peek() {
 self.input.next(); // Consume '.'
 while let Some(&next_c) = self.input.peek() {
 if next_c.is_ascii_digit() {
 frac_digits.push(self.input.next().unwrap());
 } else {
 break;
 }
 }
 }
 let mut integer_part: i128 = int_digits.parse().map_err(|_| "Invalid integer format")?;
 integer_part = integer_part.checked_mul(100_000_000).ok_or("Overflow")?;
 let mut fractional_part: i128 = 0;
 if !frac_digits.is_empty() {
 let len = frac_digits.len();
 if len <= 8 {
 let frac_val: i128 = frac_digits.parse().map_err(|_| "Invalid fractional format")?;
 let multiplier = 10i128.pow((8 - len) as u32);
 fractional_part = frac_val.checked_mul(multiplier).ok_or("Overflow")?;
 } else {
 let truncated: String = frac_digits.chars().take(8).collect();
 fractional_part = truncated.parse().map_err(|_| "Invalid fractional format")?;
 }
 }
 let final_fixed_val = integer_part.checked_add(fractional_part).ok_or("Overflow")?;
 return Ok(Some(final_fixed_val.to_string()));
 }
 // 2. Strict Multicharacter Word Tokenization (No implicit multiplication like xyz)
 //    Identifiers may also contain digits and underscores after the leading
 //    letter (e.g. `x1`, `my_var`), so real variables can be tokenized;
 //    operator words (`xor`, `gcd`, `lcm`, ¬, `|`, `&`) are unaffected.
 //    Cyrillic letters count as alphabetic, so Russian identifiers AND
 //    the reserved comparison words БОЛЬШЕ/МЕНЬШЕ/РАВНО tokenize here.
 if c.is_alphabetic() || c == '¬' {
 // Reserved comparison words win over identifiers: `5 больше 3`
 // becomes [5][больше][3], never a variable named "больше".
 if c != '¬' {
 let lowered = c.to_lowercase().next().unwrap_or(c);
 if lowered == 'б' || lowered == 'м' || lowered == 'р' || lowered == 'н' {
 let mut probe = String::new();
 probe.push(lowered);
 if RESERVED_COMPARISON_WORDS.iter().any(|w| w.starts_with(probe.as_str())) {
 // `c` is already consumed; feed it back into the scanner.
 let mut rest = self.input.clone();
 if let Some(word) = try_read_from(&mut rest, probe) {
 self.input = rest;
 return Ok(Some(word));
 }
  }
 }
 }
 let mut word = String::new();
 word.push(c);
 // `_` is accepted as a *continuation* character only (identifiers must
 // start with a letter), which keeps Russian operator aliases like
 // НЕ_РАВНО tokenizing as single words.
 while let Some(&next_c) = self.input.peek() {
 if next_c.is_alphabetic() || next_c == '|' || next_c == '&' || next_c == '_' {
 word.push(self.input.next().unwrap());
 } else if next_c.is_ascii_digit() && c != '¬' {
 word.push(self.input.next().unwrap());
 } else {
 break;
 }
 }
 return Ok(Some(word));
 }
 // 3a. Unary minus: a '-' directly attached to the following number
 // (no whitespace between, e.g. `-5`, `2.5-(-1)`) lexes as ONE negative
 // number token. A standalone `-` (with spaces around it) stays the
 // binary subtraction operator, so `5 - 8` keeps working as before.
 if c == '-' {
 if let Some(&next_c) = self.input.peek() {
 if next_c.is_ascii_digit() || (next_c == '.' && self.input.clone().nth(1).map_or(false, |n| n.is_ascii_digit())) {
 let mut num_str = String::from("-");
 while let Some(&nc) = self.input.peek() {
 if nc.is_ascii_digit() || nc == '.' {
 num_str.push(self.input.next().unwrap());
 } else {
 break;
 }
 }
 // Split into integer/fraction parts and scale exactly like the
 // positive-number branch above (fixed-point, scale 10^8).
 let (int_s, frac_s) = match num_str.find('.') {
 Some(p) => (&num_str[..p], &num_str[p + 1..]),
 None => (num_str.as_str(), ""),
 };
 let mut integer_part: i128 = int_s.parse().map_err(|_| "Invalid negative integer format")?;
 integer_part = integer_part.checked_mul(100_000_000).ok_or("Overflow")?;
 let mut fractional_part: i128 = 0;
 if !frac_s.is_empty() {
 let len = frac_s.len();
 if len <= 8 {
 let frac_val: i128 = frac_s.parse().map_err(|_| "Invalid fractional format")?;
 // `integer_part` already carries the sign; subtract the scaled
 // fraction instead of adding it so -0.5 → -(0.5 * SCALE).
 fractional_part = frac_val.checked_mul(10i128.pow((8 - len) as u32)).ok_or("Overflow")?;
 } else {
 let truncated: String = frac_s.chars().take(8).collect();
 fractional_part = truncated.parse().map_err(|_| "Invalid fractional format")?;
 }
 }
 let final_fixed_val = if integer_part < 0 {
 integer_part.checked_sub(fractional_part).ok_or("Overflow")?
 } else {
 integer_part.checked_add(fractional_part).ok_or("Overflow")?
 };
 return Ok(Some(final_fixed_val.to_string()));
 }
 }
 }
 // 4. Multicharacter operators and single characters
 match c {
 // Comparisons vs. bit shifts:
 //   `<` / `>`            — МЕНЬШЕ / БОЛЬШЕ (single char is comparison)
 //   `<<` / `>>`          — bit shift left/right (double char is shift)
 //   `<=` / `>=`          — НЕ_БОЛЬШЕ / НЕ_МЕНЬШЕ
 // A third repeated character (`<<<`, `>>>`) is rejected outright.
 '<' | '>' => {
 let second = self.input.peek().copied();
 if second == Some('=') {
 self.input.next();
 let mut op = String::new();
 op.push(c);
 op.push('=');
 return Ok(Some(op));
 }
 if second == Some(c) {
 self.input.next();
 if self.input.peek() == Some(&c) {
 return Err("Invalid operator sequence '<<<' / '>>>': use '<'/'>' for comparison (МЕНЬШЕ/БОЛЬШЕ) or '<<'/'>>' for bit shifts");
 }
 let mut op = String::new();
 op.push(c);
 op.push(c);
 return Ok(Some(op));
 }
 Ok(Some(c.to_string()))
 }
 // Equality family: `=` РАВНО, `==` alias of РАВНО.
 // Single `=` is safe here because REPL assignments are intercepted
 // before this lexer ever sees them.
 '=' => {
 if self.input.peek() == Some(&'=') {
 self.input.next();
 return Ok(Some("==".to_string()));
 }
 Ok(Some("=".to_string()))
 }
 // `≠` is the Unicode spelling of НЕ_РАВНО; `!` keeps its postfix
 // factorial role but `!=` is disambiguated up front.
 '≠' => Ok(Some("≠".to_string())),
 '!' => {
 if self.input.peek() == Some(&'=') {
 self.input.next();
 return Ok(Some("!=".to_string()));
 }
 let mut op = String::new();
 op.push(c);
 if let Some(&'s') = self.input.peek() {
 let remaining: String = self.input.clone().take(3).collect();
 if remaining == "sub" {
 for _ in 0..3 { self.input.next(); }
 op.push_str("sub");
 }
 }
 Ok(Some(op))
 }
 '+' | '-' | '/' | '&' | '|' | '(' | ')' | '[' | ']' | ',' | ';' => {
 let mut op = String::new();
 op.push(c);
 Ok(Some(op))
 }
 '*' => {
 let mut op = String::new();
 op.push(c);
 if let Some(&'*') = self.input.peek() {
 op.push(self.input.next().unwrap());
 }
 Ok(Some(op))
 }
 _ => Err("Unknown token character encountered"),
 }
 }
}
/// Continuation of `try_read_comparison_word` for an iterator whose first
/// character was already consumed by the caller. `prefix` holds the
/// lowercased leading character(s). On success `rest` ends just past the
/// matched word; on failure `rest` is untouched relative to its entry
/// state minus what `prefix` already covered.
fn try_read_from(rest: &mut std::iter::Peekable<std::str::Chars>, mut prefix: String) -> Option<String> {
if !RESERVED_COMPARISON_WORDS.iter().any(|w| w.starts_with(prefix.as_str())) {
return None;
}
let mut best: Option<String> = None;
if RESERVED_COMPARISON_WORDS.contains(&prefix.as_str()) {
best = Some(prefix.clone());
}
loop {
match rest.peek().copied() {
Some(nc) if nc.is_alphabetic() || nc == '_' => {
prefix.push(nc.to_lowercase().next().unwrap_or(nc));
rest.next();
if RESERVED_COMPARISON_WORDS.contains(&prefix.as_str()) {
best = Some(prefix.clone());
}
if !RESERVED_COMPARISON_WORDS.iter().any(|w| w.starts_with(prefix.as_str())) {
break;
}
}
_ => break,
}
}
best
}
pub struct ParsingEngine;
/// Canonicalizes operator *words* (including their Russian aliases) into
/// the engine's internal operator tokens. Returns `None` when `word` is
/// not an operator word — the caller then treats it as a variable name.
pub fn normalize_operator_word(word: &str) -> Option<&'static str> {
// Case-insensitive matching so `больше`, `Больше` and `БОЛЬШЕ` all work.
let lower = word.to_lowercase();
Some(match lower.as_str() {
// Словесные англоязычные операторы
"xor" => "xor",
"not" => "not",
"gcd" => "gcd",
"lcm" => "lcm",
"rand" => "rand",
// УСЛОВИЯ: БОЛЬШЕ / МЕНЬШЕ / РАВНО / НЕ_РАВНО (+ короткие формы)
"больше" | "gt" | "greater" => ">",
"меньше" | "lt" | "less" => "<",
"равно" | "eq" | "equal" => "=",
"не_равно" | "ne_ravno" | "neravno" => "!=",
"не_больше" | "nb" => "<=",
"не_меньше" | "nm" => ">=",
// Русские синонимы побитовых операторов
"и" => "&",
"или" => "|",
"исключительное_или" => "xor",
"не_и" | "nand" => "¬&",
"не_или" | "nor" => "¬|",
_ => return None,
})
}
impl ParsingEngine {
pub fn parse_into_processor(input: &str, processor: &mut crate::math_proccessor::MathProcessor) -> Result<(), &'static str> {
        Self::parse_into_processor_with_vars(input, processor, &std::collections::HashMap::new())
}

/// Parses `input` into `processor`, resolving bare identifiers (real
/// variables) through `vars`. Known operator words (`xor`, `gcd`, `lcm`,
/// `not`, `rand`, `¬`, `¬|`, `¬&`) and their Russian aliases (`БОЛЬШЕ`,
/// `МЕНЬШЕ`, `РАВНО`, `НЕ_РАВНО`, ...) are always treated as operators;
/// any other word is looked up in `vars` and pushed onto the stack as a
/// fixed-point number. Unknown names produce an error listing the defined
/// variables.
    pub fn parse_into_processor_with_vars(
        input: &str,
        processor: &mut crate::math_proccessor::MathProcessor,
        vars: &std::collections::HashMap<String, i128>,
) -> Result<(), &'static str> {
let mut lexer = Lexer::new(input);
while let Some(token) = lexer.next_token()? {
// Operator words (also Russian aliases like БОЛЬШЕ/МЕНЬШЕ/РАВНО)
// are canonicalized first, so they can never be confused with
// variable names.
let op_alias = normalize_operator_word(&token);
let token_ref: &str = op_alias.unwrap_or(token.as_str());
match token_ref {
"(" => processor.open_bracket(),
")" => processor.close_bracket()?,
"+" | "-" | "*" | "/" | "**" | "&" | "|" | "xor" | "not" | "¬" | "!" | "!sub" | "<<" | ">>" | "gcd" | "lcm" | "¬|" | "¬&" | "rand"
| ">" | "<" | "=" | "==" | "!=" | "≠" | "<=" | ">=" => {
processor.current_mut()?.push_op(token_ref);
}
_ => {
if let Ok(val) = token.parse::<i128>() {
processor.current_mut()?.push_num(val);
} else if let Some(&val) = vars.get(token.as_str()) {
// Real variable: push its fixed-point value directly.
processor.current_mut()?.push_num(val);
} else {
let mut defined: Vec<&String> = vars.keys().collect();
defined.sort();
let known = if defined.is_empty() {
"(no variables are defined yet)".to_string()
} else {
format!("defined variables: {}", defined.iter().map(|k| k.as_str()).collect::<Vec<_>>().join(", "))
};
return Err(Box::leak(
format!("Syntax Error: Unknown variable '{}'. {}", token, known).into_boxed_str(),
) as &'static str);
}
}
}
}
Ok(())
}
}
