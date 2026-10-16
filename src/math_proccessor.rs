// Copyright (C) 2026 Bogdan Yachmenev.
// License: AGPL v3.0 or later

pub struct MathStack {
	pub numbers: Vec<i128>,
	pub operators: Vec<String>,
}

impl MathStack {
	pub fn new() -> Self {
		Self {
			numbers: Vec::new(),
			operators: Vec::new(),
		}
	}

	pub fn push_num(&mut self, val: i128) {
		self.numbers.push(val);
	}

	pub fn push_op(&mut self, op: &str) {
		self.operators.push(op.to_string());
	}

	fn fixed_pow(base: i128, exp: i128, scale: i128) -> Result<i128, &'static str> {
		let mut e = exp / scale; 
		if e == 0 { return Ok(scale); }
		if e < 0 {
			let positive_res = Self::fixed_pow(base, (-e) * scale, scale)?;
			if positive_res == 0 { return Err("Division by zero in negative power"); }
			return scale.checked_mul(scale).ok_or("Overflow")?.checked_div(positive_res).ok_or("Overflow");
		}
		let mut res = scale;
		let mut b = base;
		while e > 0 {
			if e % 2 == 1 {
				res = res.checked_mul(b).ok_or("Overflow")? / scale;
			}
			b = b.checked_mul(b).ok_or("Overflow")? / scale;
			e /= 2;
		}
		Ok(res)
	}

	pub fn evaluate_with_processor(
		mut self, 
		op_count: &mut i128, 
		rand_engine: &crate::math_operations::RandomEngine
	) -> Result<i128, &'static str> {
		const SCALE: i128 = 100_000_000;
		let to_raw = |x: i128| x / SCALE;
		let to_fixed = |x: i128| x * SCALE;

		// 1. Highest priority: Token generators ("rand")
		let mut i = 0;
		while i < self.operators.len() {
			if self.operators[i] == "rand" {
				*op_count += 1;
				let generated_raw = rand_engine.next_rand(*op_count);
				let final_rand = (generated_raw.abs() % SCALE) * (SCALE / 100_000_000);
				self.numbers.insert(i, final_rand);
				self.operators.remove(i);
			} else {
				i += 1;
			}
		}

		if self.numbers.is_empty() {
			return Err("Empty stack");
		}

		// 2. High priority: Unary operations (not, ¬, !, !sub) from right to left
		let mut i = self.operators.len();
		while i > 0 {
			i -= 1;
			let op = &self.operators[i];
			if op == "not" || op == "¬" || op == "!" || op == "!sub" {
				if i >= self.numbers.len() { return Err("Missing operand for unary operator"); }
				let a = self.numbers[i];
				let res = match op.as_str() {
					"not" | "¬" => to_fixed(!to_raw(a)),
					"!" => {
						let n = to_raw(a);
						if n < 0 { return Err("Factorial of negative number"); }
						let mut f: i128 = 1;
						for idx in 1..=n { f = f.checked_mul(idx).ok_or("Overflow")?; }
						to_fixed(f)
					},
					"!sub" => {
						let n = to_raw(a);
						if n < 0 { return Err("Subfactorial of negative number"); }
						let mut sub: i128 = 1;
						if n > 0 {
							let mut p1: i128 = 1; let mut p2: i128 = 0; sub = p2;
							for idx in 2..=n {
								sub = (idx - 1).checked_mul(p1 + p2).ok_or("Overflow")?;
								p1 = p2; p2 = sub;
							}
						}
						to_fixed(sub)
					},
					_ => unreachable!(),
				};
				self.numbers[i] = res;
				self.operators.remove(i);
			}
		}

		// 3. Medium-High priority: Power operator ("**")
		let mut i = 0;
		while i < self.operators.len() {
			if self.operators[i] == "**" {
				if i + 1 >= self.numbers.len() { return Err("Missing operand for power"); }
				let a = self.numbers[i];
				let b = self.numbers[i + 1];
				let res = Self::fixed_pow(a, b, SCALE)?;
				self.numbers[i] = res;
				self.numbers.remove(i + 1);
				self.operators.remove(i);
			} else {
				i += 1;
			}
		}

		// 4. Medium priority: Binary math (*, /, <<, >>, gcd, lcm)
		let mut i = 0;
		while i < self.operators.len() {
			let op = &self.operators[i];
			if op == "*" || op == "/" || op == "<<" || op == ">>" || op == "gcd" || op == "lcm" {
				if i + 1 >= self.numbers.len() { return Err("Missing operand"); }
				let a = self.numbers[i];
				let b = self.numbers[i + 1];

				let res = match op.as_str() {
					"*" => a.checked_mul(b).ok_or("Overflow")? / SCALE,
					"/" => {
						if b == 0 { return Err("Division by zero"); }
						a.checked_mul(SCALE).ok_or("Overflow")? / b
					},
					"<<" => to_fixed(to_raw(a).checked_shl(to_raw(b) as u32).ok_or("Shift overflow")?),
					">>" => to_fixed(to_raw(a).checked_shr(to_raw(b) as u32).ok_or("Shift overflow")?),
					"gcd" => {
						let mut x = to_raw(a).abs(); let mut y = to_raw(b).abs();
						while y != 0 { let t = x % y; x = y; y = t; }
						to_fixed(x)
					},
					"lcm" => {
						let x = to_raw(a).abs(); let y = to_raw(b).abs();
						if x == 0 || y == 0 { to_fixed(0) } else {
							let (mut g_x, mut g_y) = (x, y);
							while g_y != 0 { let t = g_x % g_y; g_x = g_y; g_y = t; }
							to_fixed((x / g_x).checked_mul(y).ok_or("Overflow")?)
						}
					},
					_ => unreachable!(),
				};
				self.numbers[i] = res;
				self.numbers.remove(i + 1);
				self.operators.remove(i);
			} else {
				i += 1;
			}
		}

		// 5. Low priority: Basic & bitwise logic (+, -, &, |, xor, ¬|, ¬&).
		let mut total_val = self.numbers[0];
		for (i, op) in self.operators.iter().enumerate() {
			if i + 1 >= self.numbers.len() {
				return Err("Missing operand for trailing operator");
			}
			let next_val = self.numbers[i + 1];
			total_val = match op.as_str() {
				"+" => total_val.checked_add(next_val).ok_or("Overflow")?,
				"-" => total_val.checked_sub(next_val).ok_or("Overflow")?,
				"&" => to_fixed(to_raw(total_val) & to_raw(next_val)),
				"|" => to_fixed(to_raw(total_val) | to_raw(next_val)),
				"xor" => to_fixed(to_raw(total_val) ^ to_raw(next_val)),
				"¬|" => to_fixed(!(to_raw(total_val) | to_raw(next_val))),
				"¬&" => to_fixed(!(to_raw(total_val) & to_raw(next_val))),
				_ => return Err("Invalid operator"),
			};
		}

		Ok(total_val)
	}

	/// Collapses every comparison on this stack into fixed-point booleans
	/// (1 or 0). Comparison operators: БОЛЬШЕ (`>`), МЕНЬШЕ (`<`), РАВНО
	/// (`=`/`==`), НЕ_РАВНО (`!=`/`≠`), НЕ_БОЛЬШЕ (`<=`), НЕ_МЕНЬШЕ (`>=`).
	///
	/// Semantics:
	///   * Chained comparisons behave like Rust: `a < b < c` means
	///     `(a < b) AND (b < c)`; the shared middle value `b` is reused as
	///     the left operand of the next link and the boolean results are
	///     combined with a spliced bitwise `&` (exact for 0/SCALE values).
	///   * Result type is numeric: 1 (истина) или 0 (ложь) — совместимо со
	///     всей остальной арифметикой движка, поэтому `(1 > 2) + 1` = 1.
	pub fn evaluate_comparisons(&mut self) -> Result<(), &'static str> {
		while let Some(pos) = self.operators.iter().position(|o| is_comparison_op(o)) {
			if pos + 1 >= self.numbers.len() {
				return Err("Missing operand for comparison operator");
			}
			let a = self.numbers[pos];
			let b = self.numbers[pos + 1];
			let res = compare_values(&self.operators[pos], a, b)?;
			let chained_right = self.operators.get(pos + 1)
				.map(|o| is_comparison_op(o.as_str()))
				.unwrap_or(false);
			if chained_right {
				// Drop `a`, KEEP `b` as the shared left operand of the next
				// link, and splice `&` so the final fold computes
				// `raw(a<b) & raw(b<c)` — logical AND on our 0/SCALE booleans.
				self.numbers.remove(pos);
				self.operators.insert(pos, "&".to_string());
				self.operators.remove(pos + 2); // the comparison just folded
			} else {
				self.numbers[pos] = res; // standalone (or last link): store bool
				self.operators.remove(pos);
			}
		}
		Ok(())
	}
}

/// True when `op` is one of the comparison operators: БОЛЬШЕ (`>`),
/// МЕНЬШЕ (`<`), РАВНО (`=`/`==`), НЕ_РАВНО (`!=`/`≠`), НЕ_БОЛЬШЕ (`<=`),
/// НЕ_МЕНЬШЕ (`>=`).
pub fn is_comparison_op(op: &str) -> bool {
	matches!(op, ">" | "<" | "=" | "==" | "!=" | "≠" | "<=" | ">=")
}

/// Compares two fixed-point values (scale 10^8) and returns the boolean
/// result scaled back to fixed-point (1 or 0). Exact integer comparison on
/// the raw representation — no epsilon needed since every value shares the
/// same scale.
pub fn compare_values(op: &str, a: i128, b: i128) -> Result<i128, &'static str> {
	const SCALE: i128 = 100_000_000;
	let result = match op {
		">" => a > b,
		"<" => a < b,
		"=" | "==" => a == b,
		"!=" | "≠" => a != b,
		"<=" => a <= b,
		">=" => a >= b,
		_ => return Err("Not a comparison operator"),
	};
	Ok(if result { SCALE } else { 0 })
}

pub struct MathProcessor {
	pub storage: Vec<MathStack>,
	pub op_count: i128,
	pub rand_engine: crate::math_operations::RandomEngine,
}

impl MathProcessor {
	pub fn new() -> Self {
		Self {
			storage: vec![MathStack::new()],
			op_count: 0,
			rand_engine: crate::math_operations::RandomEngine::new(),
		}
	}

	pub fn current_mut(&mut self) -> Result<&mut MathStack, &'static str> {
		self.storage.last_mut().ok_or("No active stack context")
	}

	pub fn open_bracket(&mut self) {
		self.storage.push(MathStack::new());
	}

	pub fn close_bracket(&mut self) -> Result<(), &'static str> {
		if self.storage.len() < 2 {
			return Err("Unmatched closing bracket");
		}
		let mut current = self.storage.pop().unwrap();
		// Resolve comparisons at THIS bracket level before handing a single
		// value upward, so `(1 > 2) + 1` becomes `0 + 1` → 1.
		current.evaluate_comparisons()?;
		let result = current.evaluate_with_processor(&mut self.op_count, &self.rand_engine)?;
		self.current_mut()?.push_num(result);
		Ok(())
	}

	pub fn finalize(mut self) -> Result<i128, &'static str> {
		if self.storage.len() != 1 {
			return Err("Unmatched opening brackets");
		}
		let mut current = self.storage.pop().unwrap();
		// Top-level expression: comparisons collapse into 1/0 booleans
		// first, then the remaining arithmetic/bitwise fold produces the
		// final value.
		current.evaluate_comparisons()?;
		current.evaluate_with_processor(&mut self.op_count, &self.rand_engine)
	}
}
