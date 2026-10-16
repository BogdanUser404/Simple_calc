// Copyright (C) 2026 Bogdan Yachmenev.
// License: AGPL v3.0 or later
pub mod math_proccessor;
pub mod math_operations;
mod parser;
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use crate::math_proccessor::MathProcessor;
use crate::parser::ParsingEngine;
const SCALE: i128 = 100_000_000;
fn format_fixed_point(val: i128) -> String {
	let sign = if val < 0 { "-" } else { "" };
	let abs_val = val.abs();
	let integer_part = abs_val / SCALE;
	let fractional_part = abs_val % SCALE;
	if fractional_part == 0 {
		format!("{}{}", sign, integer_part)
	} else {
		let frac_str = format!("{:08}", fractional_part);
		let trimmed_frac = frac_str.trim_end_matches('0');
		if trimmed_frac.is_empty() {
			format!("{}{}", sign, integer_part)
		} else {
			format!("{}{}.{}", sign, integer_part, trimmed_frac)
		}
	}
}
fn apply_placeholders(expr: &str, vars: &HashMap<String, i128>) -> Result<String, String> {
	let mut result = String::with_capacity(expr.len());
	let mut chars = expr.chars().peekable();
	while let Some(c) = chars.next() {
		if c == '{' {
			let mut name = String::new();
			let mut closed = false;
			while let Some(&nc) = chars.peek() {
				chars.next();
				if nc == '}' {
					closed = true;
					break;
				}
				name.push(nc);
			}
			if !closed {
				return Err(format!("Unmatched '{{' in placeholder: {{{}", name));
			}
			let name = name.trim().to_string();
			if name.is_empty() {
				return Err("Empty placeholder name".to_string());
			}
			match vars.get(&name) {
				Some(&val) => result.push_str(&format_fixed_point(val)),
				None => return Err(format!("Undefined placeholder: {{{}}}", name)),
			}
		} else {
			result.push(c);
		}
	}
	Ok(result)
}
fn parse_var_assignment(s: &str) -> Option<(String, i128)> {
	if let Some(eq_pos) = s.find('=') {
		let name = s[..eq_pos].trim().to_string();
		let val_str = s[eq_pos + 1..].trim();
		if name.is_empty() {
			return None;
		}
		if let Some(dot_pos) = val_str.find('.') {
			let int_part: i128 = val_str[..dot_pos].parse().ok()?;
			let frac_str = &val_str[dot_pos + 1..];
			let len = frac_str.len();
			let frac_val: i128 = frac_str.parse().ok()?;
			let multiplier = if len <= 8 {
				10i128.pow((8 - len) as u32)
			} else {
				let truncated: String = frac_str.chars().take(8).collect();
				truncated.parse().ok()?
			};
			let frac_scaled = frac_val.checked_mul(multiplier)?;
			let total = int_part.checked_mul(SCALE)?.checked_add(frac_scaled)?;
			Some((name, total))
		} else {
			let int_val: i128 = val_str.parse().ok()?;
			Some((name, int_val.checked_mul(SCALE)?))
		}
	} else {
		None
	}
}
fn execute_expression(expr: &str, vars: &HashMap<String, i128>) {
	if expr.trim().is_empty() {
		return;
	}
	let expanded = match apply_placeholders(expr, vars) {
		Ok(s) => s,
		Err(e) => {
			eprintln!("Placeholder Error: {}", e);
			return;
		}
	};
	let mut processor = MathProcessor::new();
	match ParsingEngine::parse_into_processor(&expanded, &mut processor) {
		Ok(_) => {
			match processor.finalize() {
				Ok(result) => println!("{}", format_fixed_point(result)),
				Err(err) => eprintln!("Evaluation Error: {}", err),
			}
		}
		Err(err) => eprintln!("{}", err),
	}
}
fn run_repl() {
	println!("Simple Fixed-Point Calculator (8 decimals) - Bogdan Yachmenev");
	println!("Type your expression and press Enter. Type 'exit' or 'quit' to stop.");
	println!("Use 'set NAME VALUE' to define placeholders like {{NAME}}.\n");
	let mut vars: HashMap<String, i128> = HashMap::new();
	let mut input = String::new();
	loop {
		print!("calc> ");
		if io::stdout().flush().is_err() {
			eprintln!("Error flushing stdout");
			break;
		}
		input.clear();
		if io::stdin().read_line(&mut input).is_err() {
			eprintln!("Error reading input");
			break;
		}
		let trimmed = input.trim();
		if trimmed.is_empty() {
			continue;
		}
		if trimmed == "exit" || trimmed == "quit" {
			break;
		}
		if trimmed.starts_with("set ") {
			let rest = trimmed[4..].trim();
			// set NAME VALUE
			let mut parts = rest.splitn(2, char::is_whitespace);
			let name = parts.next().unwrap_or("").trim();
			let val_str = parts.next().unwrap_or("").trim();
			if name.is_empty() || val_str.is_empty() {
				eprintln!("Usage: set NAME VALUE");
				continue;
			}
			// Используем тот же парсер, что и в CLI
			let assign = format!("{}={}", name, val_str);
			match parse_var_assignment(&assign) {
				Some((n, v)) => {
					vars.insert(n.clone(), v);
					println!("{} = {}", n, format_fixed_point(v));
				}
				None => eprintln!("Invalid value for '{}'", name),
			}
			continue;
		}
		if trimmed == "list" {
			if vars.is_empty() {
				println!("(no placeholders defined)");
			} else {
				for (k, v) in &vars {
					println!("{} = {}", k, format_fixed_point(*v));
				}
			}
			continue;
		}
		if trimmed == "clear" {
			vars.clear();
			println!("All placeholders cleared.");
			continue;
		}
		execute_expression(trimmed, &vars);
	}
}
fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() > 1 {
		let mut expr_parts: Vec<String> = Vec::new();
		let mut vars: HashMap<String, i128> = HashMap::new();
		let mut found_expr = false;
		for arg in &args[1..] {
			if !found_expr {
				expr_parts.push(arg.clone());
				found_expr = true;
				continue;
			}
			if let Some((name, val)) = parse_var_assignment(arg) {
				vars.insert(name, val);
			}
			else {
				expr_parts.push(arg.clone());
			}
		}
		let expression = expr_parts.join(" ");
		execute_expression(&expression, &vars);
	}
	else {
		run_repl();
	}
}