// Copyright (C) 2026 Bogdan Yachmenev.
// License: AGPL v3.0 or later
use std::time::SystemTime;

pub fn pow(base: i128, exp: i128) -> Result<i128, &'static str> {
	let mut e = exp / 100_000_000; 

	if e == 0 {
		return Ok(100_000_000); // x^0 = 1
	}

	if e < 0 {
		// For negative powers: 1 / (base^abs(e))
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

	// Fast binary exponentiation (O(log N))
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

pub struct RandomEngine {
	pub app_start_ms: i128,
}

impl RandomEngine {
	pub fn new() -> Self {
		let start = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as i128;
		Self { app_start_ms: start }
	}

	pub fn next_rand(&self, op_num: i128) -> i128 {
		// 1. Snapshot all General Purpose Registers, RFLAGS, and XMM low 64-bits
		let mut rflags_val: usize = 0;
		let mut rax_val: usize = 0; let mut rbx_val: usize = 0;
		let mut rcx_val: usize = 0; let mut rdx_val: usize = 0;
		let mut rsi_val: usize = 0; let mut rdi_val: usize = 0;
		let mut rbp_val: usize = 0; let mut rsp_addr: usize = 0;
		let mut r8_val: usize = 0;  let mut r9_val: usize = 0;
		let mut r10_val: usize = 0; let mut r11_val: usize = 0;
		let mut r12_val: usize = 0; let mut r13_val: usize = 0;
		let mut r14_val: usize = 0; let mut r15_val: usize = 0;

		let mut xmm: [u64; 16] = [0; 16];

		unsafe {
			std::arch::asm!("pushf", "pop {}", out(reg) rflags_val);
			std::arch::asm!("mov {}, rax", out(reg) rax_val);
			std::arch::asm!("mov {}, rbx", out(reg) rbx_val);
			std::arch::asm!("mov {}, rcx", out(reg) rcx_val);
			std::arch::asm!("mov {}, rdx", out(reg) rdx_val);
			std::arch::asm!("mov {}, rsi", out(reg) rsi_val);
			std::arch::asm!("mov {}, rdi", out(reg) rdi_val);
			std::arch::asm!("mov {}, rbp", out(reg) rbp_val);
			std::arch::asm!("mov {}, rsp", out(reg) rsp_addr);
			std::arch::asm!("mov {}, r8", out(reg) r8_val);
			std::arch::asm!("mov {}, r9", out(reg) r9_val);
			std::arch::asm!("mov {}, r10", out(reg) r10_val);
			std::arch::asm!("mov {}, r11", out(reg) r11_val);
			std::arch::asm!("mov {}, r12", out(reg) r12_val);
			std::arch::asm!("mov {}, r13", out(reg) r13_val);
			std::arch::asm!("mov {}, r14", out(reg) r14_val);
			std::arch::asm!("mov {}, r15", out(reg) r15_val);

			std::arch::asm!("movq {}, xmm0", out(reg) xmm[0]);
			std::arch::asm!("movq {}, xmm1", out(reg) xmm[1]);
			std::arch::asm!("movq {}, xmm2", out(reg) xmm[2]);
			std::arch::asm!("movq {}, xmm3", out(reg) xmm[3]);
			std::arch::asm!("movq {}, xmm4", out(reg) xmm[4]);
			std::arch::asm!("movq {}, xmm5", out(reg) xmm[5]);
			std::arch::asm!("movq {}, xmm6", out(reg) xmm[6]);
			std::arch::asm!("movq {}, xmm7", out(reg) xmm[7]);
			std::arch::asm!("movq {}, xmm8", out(reg) xmm[8]);
			std::arch::asm!("movq {}, xmm9", out(reg) xmm[9]);
			std::arch::asm!("movq {}, xmm10", out(reg) xmm[10]);
			std::arch::asm!("movq {}, xmm11", out(reg) xmm[11]);
			std::arch::asm!("movq {}, xmm12", out(reg) xmm[12]);
			std::arch::asm!("movq {}, xmm13", out(reg) xmm[13]);
			std::arch::asm!("movq {}, xmm14", out(reg) xmm[14]);
			std::arch::asm!("movq {}, xmm15", out(reg) xmm[15]);
		}

		// 2. Check Zero Flag (6th bit of RFLAGS)
		let zf_is_one = (rflags_val & (1 << 6)) != 0;

		// 3. Directional shifting driven by Zero Flag
		let shift_amount = ((!op_num).wrapping_mul(self.app_start_ms) & 0x7F) as u32;
		let shifted_part = if zf_is_one {
			op_num.wrapping_shr(shift_amount)
		} else {
			op_num.wrapping_shl(shift_amount & 0x3F)
		};

		let current_time = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_nanos() as i128;

		// 4. Create raw entropy block to feed our primitive hash function
		let core_math = shifted_part.wrapping_mul(op_num) ^ op_num;

		// Pack everything into a transient slice for linear hashing
		let registers_snapshot: [i128; 19] = [
			rax_val as i128, rbx_val as i128, rcx_val as i128, rdx_val as i128,
			rsi_val as i128, rdi_val as i128, rbp_val as i128, rsp_addr as i128,
			r8_val as i128,  r9_val as i128,  r10_val as i128, r11_val as i128,
			r12_val as i128, r13_val as i128, r14_val as i128, r15_val as i128,
			(xmm[0] ^ xmm[1] ^ xmm[2] ^ xmm[3] ^ xmm[4] ^ xmm[5] ^ xmm[6] ^ xmm[7] ^
			xmm[8] ^ xmm[9] ^ xmm[10] ^ xmm[11] ^ xmm[12] ^ xmm[13] ^ xmm[14] ^ xmm[15]) as i128,
			core_math,
			current_time
		];

		// 5. Dynamic FNV-1a 128-bit style hash driven by RAX and RDI hardware state
		// Instead of standard constants, the initial hash state and multiplier depend on the CPU
		let mut hash: u128 = (rax_val as u128) << 64 | (rdi_val as u128);
		
		// Ensure the prime multiplier is always an odd number to maintain hash coverage
		let mut dynamic_prime: u128 = (rax_val as u128 ^ rdi_val as u128) | 1;
		if dynamic_prime < 3 {
			dynamic_prime = 0x1000000000000000000013b; // Safe fallback prime constant
		}

		// Treat our snapshot array as raw byte stream
		let byte_ptr = registers_snapshot.as_ptr() as *const u8;
		let byte_len = std::mem::size_of::<[i128; 19]>();

		for idx in 0..byte_len {
			unsafe {
				let byte_val = *byte_ptr.add(idx);
				hash ^= byte_val as u128;
				hash = hash.wrapping_mul(dynamic_prime);
			}
		}

		let final_raw_rand = hash as i128 | current_time;
		let final_scaled = final_raw_rand.abs().wrapping_mul(100_000_000);

		final_scaled
	}
}