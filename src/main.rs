// Copyright (C) 2026 Bogdan Yachmenev.
// License: AGPL v3.0 or later

pub mod math_proccessor;
pub mod math_operations;
mod parser;

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

fn execute_expression(expr: &str) {
	if expr.trim().is_empty() {
		return;
	}
	let mut processor = MathProcessor::new();
	match ParsingEngine::parse_into_processor(expr, &mut processor) {
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
	println!("Type your expression and press Enter. Type 'exit' or 'quit' to stop.\n");

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
		if trimmed == "exit" || trimmed == "quit" {
			break;
		}

		execute_expression(trimmed);
	}
}

fn main() {
	let args: Vec<String> = env::args().collect();

	if args.len() > 1 {
		// One-shot execution from command line arguments
		let expression = args[1..].join(" ");
		execute_expression(&expression);
	} else {
		// Fallback to Interactive REPL mode
		run_repl();
	}
}
