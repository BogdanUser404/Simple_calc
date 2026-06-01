// Copyright (C) 2026 Bogdan Yachmenev.
// License: AGPL v3.0 or later

pub struct Lexer<'a> {
	input: std::iter::Peekable<std::str::Chars<'a>>,
}

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
		if c.is_ascii_alphabetic() || c == '¬' {
			let mut word = String::new();
			word.push(c);
			while let Some(&next_c) = self.input.peek() {
				if next_c.is_ascii_alphabetic() || next_c == '|' || next_c == '&' {
					word.push(self.input.next().unwrap());
				} else {
					break;
				}
			}
			return Ok(Some(word));
		}

		// 3. Multicharacter operators and single characters
		match c {
			'<' | '>' => {
				if let Some(&next_c) = self.input.peek() {
					if next_c == c {
						let mut op = String::new();
						op.push(c);
						op.push(self.input.next().unwrap());
						return Ok(Some(op));
					}
				}
				Err("Invalid shift operator usage")
			}
			'+' | '-' | '/' | '&' | '|' | '(' | ')' => {
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
			'!' => {
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
			_ => Err("Unknown token character encountered"),
		}
	}
}

pub struct ParsingEngine;

impl ParsingEngine {
	pub fn parse_into_processor(input: &str, processor: &mut crate::math_proccessor::MathProcessor) -> Result<(), &'static str> {
		let mut lexer = Lexer::new(input);
		
		while let Some(token) = lexer.next_token()? {
			match token.as_str() {
				"(" => processor.open_bracket(),
				")" => processor.close_bracket()?,
				"+" | "-" | "*" | "/" | "**" | "&" | "|" | "xor" | "not" | "¬" | "!" | "!sub" | "<<" | ">>" | "gcd" | "lcm" | "¬|" | "¬&" | "rand" => {
					processor.current_mut()?.push_op(&token);
				}
				_ => {
					if let Ok(val) = token.parse::<i128>() {
						processor.current_mut()?.push_num(val);
					} else {
						return Err("Syntax Error: Unknown variables or implicit multiplication like xyz are disabled.");
					}
				}
			}
		}
		Ok(())
	}
}
