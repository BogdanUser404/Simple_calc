// Copyright (C) 2026 Bogdan Yachmenev.
// License: AGPL v3.0 or later
use rand::Rng;

pub fn pow(base: i128, exp: i128) -> Result<i128, &'static str> {
	let mut e = exp / 100_000_000;
	if e == 0 {
		return Ok(100_000_000); // x^0 = 1
	}
	if e < 0 {
		let positive_res = pow(base, (-e) * 100_000_000)?;
		if positive_res == 0 {
			return Err("Division by zero in negative power");
		}
		return 100_000_000i128
			.checked_mul(100_000_000i128)
			.ok_or("Overflow")?
			.checked_div(positive_res)
			.ok_or("Overflow");
	}
	let mut res = 100_000_000i128;
	let mut b = base;
	while e > 0 {
		if e % 2 == 1 {
			res = res
				.checked_mul(b)
				.ok_or("Overflow")?
				.checked_div(100_000_000)
				.ok_or("Overflow")?;
		}
		b = b
			.checked_mul(b)
			.ok_or("Overflow")?
			.checked_div(100_000_000)
			.ok_or("Overflow")?;
		e /= 2;
	}
	Ok(res)
}

pub struct RandomEngine;

impl RandomEngine {
	pub fn new() -> Self {
		Self
	}
	pub fn next_rand(&self, _op_num: i128) -> i128 {
		const SCALE: u128 = 100_000_000;
		let raw: u128 = rand::random();
		(raw % SCALE) as i128
	}
}