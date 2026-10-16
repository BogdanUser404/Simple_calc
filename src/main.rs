// Copyright (C) 2026 Bogdan Yachmenev.
// License: AGPL v3.0 or later
pub mod math_proccessor;
pub mod math_operations;
mod parser;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
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
fn get_functions_dir() -> PathBuf {
	let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
	PathBuf::from(home).join(".local").join("share").join("calc").join("functions")
}
fn load_math_function(name: &str) -> Result<(usize, String), String> {
	let func_dir = get_functions_dir();
	let file_path = func_dir.join(format!("{}.math", name));
	let content = fs::read_to_string(&file_path)
		.map_err(|e| format!("Failed to read .math function '{}': {}", name, e))?;
	
	let mut args_count = 0;
	let mut expr_start = 0;
	
	for (i, line) in content.lines().enumerate() {
		let trimmed = line.trim();
		if trimmed.starts_with("ARGS:") {
			let args_part = trimmed.trim_start_matches("ARGS:").trim();
			let args_str = args_part.trim_end_matches(';').trim();
			args_count = args_str.parse::<usize>()
				.map_err(|_| format!("Invalid ARGS count in '{}'", name))?;
			expr_start = i + 1;
			break;
		}
	}
	
	if expr_start == 0 {
		return Err(format!("No ARGS declaration found in '{}'", name));
	}
	
	let expr: String = content.lines().skip(expr_start)
		.map(|l| l.trim())
		.filter(|l| !l.is_empty())
		.collect::<Vec<_>>()
		.join(" ");
	
	Ok((args_count, expr))
}
fn execute_math_function(name: &str, args: &[i128], vars: &HashMap<String, i128>) -> Result<i128, String> {
	let (expected_args, expr_template) = load_math_function(name)?;
	
	if args.len() != expected_args {
		return Err(format!("Function '{}' expects {} arguments, got {}", name, expected_args, args.len()));
	}
	
	let mut local_vars = vars.clone();
	for (i, &arg_val) in args.iter().enumerate() {
		local_vars.insert(format!("ARG_{}", i + 1), arg_val);
	}
	
	let expanded = apply_placeholders(&expr_template, &local_vars)?;
	
	let mut processor = MathProcessor::new();
	ParsingEngine::parse_into_processor(&expanded, &mut processor)
		.map_err(|e| format!("Parse error in function '{}': {}", name, e))?;
	
	processor.finalize()
		.map_err(|e| format!("Evaluation error in function '{}': {}", name, e))
}
fn execute_rhai_function(name: &str, args: &[i128]) -> Result<i128, String> {
	use rhai::{Engine, Array, Dynamic, Scope};
	
	let func_dir = get_functions_dir();
	let file_path = func_dir.join(format!("{}.rhai", name));
	let script = fs::read_to_string(&file_path)
		.map_err(|e| format!("Failed to read .rhai function '{}': {}", name, e))?;
	
	let mut engine = Engine::new();
	
	// Register fixed-point type and operations
	engine.register_type_with_name::<i128>("FixedPoint");
	
	engine.register_fn("to_fixed", |x: i64| -> i128 { (x as i128) * SCALE });
	engine.register_fn("to_fixed", |x: f64| -> i128 { (x * SCALE as f64) as i128 });
	engine.register_fn("to_raw", |x: i128| -> i64 { (x / SCALE) as i64 });
	engine.register_fn("to_float", |x: i128| -> f64 { x as f64 / SCALE as f64 });
	
	engine.register_fn("+", |a: i128, b: i128| -> i128 { a + b });
	engine.register_fn("-", |a: i128, b: i128| -> i128 { a - b });
	engine.register_fn("*", |a: i128, b: i128| -> i128 { a * b / SCALE });
	engine.register_fn("/", |a: i128, b: i128| -> i128 { a * SCALE / b });
	engine.register_fn("**", |a: i128, b: i128| -> i128 { 
		crate::math_operations::pow(a, b).unwrap_or(0)
	});
	
	let args_array: Array = args.iter().map(|&x| Dynamic::from(x)).collect();
	
	let mut scope = Scope::new();
	scope.push("args", args_array);
	
	let result = engine.eval_with_scope::<i128>(&mut scope, &script)
		.map_err(|e| format!("Rhai execution error in '{}': {}", name, e))?;
	
	Ok(result)
}
fn parse_function_call(expr: &str) -> Option<(String, Vec<String>)> {
	let bracket_pos = expr.find('[')?;
	let name = expr[..bracket_pos].trim().to_string();
	
	let rest = &expr[bracket_pos + 1..];
	let end_pos = rest.find(']')?;
	let args_str = &rest[..end_pos];
	
	let mut args = Vec::new();
	let mut current = String::new();
	let mut depth = 0;
	
	for c in args_str.chars() {
		match c {
			'(' => {
				depth += 1;
				current.push(c);
			}
			')' => {
				depth -= 1;
				current.push(c);
			}
			',' if depth == 0 => {
				args.push(current.trim().to_string());
				current.clear();
			}
			_ => {
				current.push(c);
			}
		}
	}
	
	if !current.trim().is_empty() {
		args.push(current.trim().to_string());
	}
	
	Some((name, args))
}
fn execute_expression(expr: &str, vars: &HashMap<String, i128>) {
	if expr.trim().is_empty() {
		return;
	}
	
	// Check if it's a function call
	if let Some((func_name, arg_strs)) = parse_function_call(expr) {
		let mut arg_values = Vec::new();
		for arg_str in arg_strs {
			let expanded = match apply_placeholders(&arg_str, vars) {
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
						Ok(result) => arg_values.push(result),
						Err(err) => {
							eprintln!("Evaluation Error in argument: {}", err);
							return;
						}
					}
				}
				Err(err) => {
					eprintln!("{}", err);
					return;
				}
			}
		}
		
		let func_dir = get_functions_dir();
		let math_path = func_dir.join(format!("{}.math", func_name));
		let rhai_path = func_dir.join(format!("{}.rhai", func_name));
		
		let result = if math_path.exists() {
			execute_math_function(&func_name, &arg_values, vars)
		} else if rhai_path.exists() {
			execute_rhai_function(&func_name, &arg_values)
		} else {
			Err(format!("Function '{}' not found", func_name))
		};
		
		match result {
			Ok(val) => println!("{}", format_fixed_point(val)),
			Err(e) => eprintln!("Function Error: {}", e),
		}
		return;
	}
	
	// Regular expression
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
			let mut parts = rest.splitn(2, char::is_whitespace);
			let name = parts.next().unwrap_or("").trim();
			let val_str = parts.next().unwrap_or("").trim();
			if name.is_empty() || val_str.is_empty() {
				eprintln!("Usage: set NAME VALUE");
				continue;
			}
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
			} else {
				expr_parts.push(arg.clone());
			}
		}
		let expression = expr_parts.join(" ");
		execute_expression(&expression, &vars);
	} else {
		run_repl();
	}
}