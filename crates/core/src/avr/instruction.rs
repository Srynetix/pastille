// SPDX-License-Identifier: MIT OR Apache-2.0
// Ported from avr8js, src/cpu/instruction.ts.
// Copyright (c) 2019-2025 Uri Shaked, MIT licence (see NOTICE.md beside this file).

//! The AVR instruction set: one instruction per call, cycle counts as the
//! ATmega328P's datasheet gives them (ADR 0017).
//!
//! avr8js decodes with one long chain of `if`s, mask and compare, in an order
//! where earlier entries win — `LDD Y+0` is `LD Y`, and the plain `LD` forms
//! are tested before the displaced ones. That order is the specification, so
//! `decode` keeps it entry for entry. What changes is when it runs: once per
//! possible opcode, into a 64K table built on first use, rather than once per
//! instruction executed. An instruction near the bottom of the chain — `SUBI`
//! is the ninety-sixth test — then costs what one near the top does, which is
//! most of what a port to a compiled language can buy over V8 (the table is
//! shared by every CPU, and SPM, which could change the program, does
//! nothing).
//!
//! [`avr_instruction`] is then the bodies of the chain, as a `match`, with the
//! flags computed as upstream computes them. Where JavaScript's numbers do the
//! work — a subtraction going negative, a `>>>` — the arithmetic is done in
//! `i32` or `u32` so the same expressions give the same bits.
//!
//! **Where this differs from avr8js 0.21.1**, each marked `avr8js:` below:
//!
//! - **H, the half-carry**, is computed from bit 0 upstream (`1 & (…)`) where
//!   the datasheet takes bit 3 (`8 & (…)`), in `ADC`, `ADD`, `CP`, `CPC`,
//!   `CPI`, `NEG`, `SBC`, `SBCI`, `SUB` and `SUBI`; here it is bit 3. `SBIW`
//!   leaves H alone, as the datasheet says, rather than computing it.
//! - **`NEG`'s V** is never set upstream (it compares the negative `0 - Rd`
//!   with 128); here it is set when the result is 0x80.
//! - **`FMULSU`'s Z** is inverted upstream; here it is set on a zero product,
//!   as `FMUL` and `FMULS` set it.
//! - **`SBC`, `SBCI` and `CPC`** keep Z when the result is zero even when the
//!   unmasked difference is −256 (0 − 255 − 1), which upstream misses.
//! - **`CBI`** takes two cycles, as the datasheet and `SBI` do, not one.
//! - **A jump to a negative address** wraps round the end of flash, as the
//!   hardware's program counter does; upstream it leaves `pc` negative.
//!
//! `SLEEP`, `SPM` and `BREAK` do nothing, as upstream.

use std::sync::OnceLock;

use super::cpu::{Cpu, Hooks, SREG};

/// Each instruction avr8js tells apart: the arms of its `if` chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Op {
    Adc,
    Add,
    Adiw,
    And,
    Andi,
    Asr,
    Bclr,
    Bld,
    Brbc,
    Brbs,
    Bset,
    Bst,
    Call,
    Cbi,
    Com,
    Cp,
    Cpc,
    Cpi,
    Cpse,
    Dec,
    Eicall,
    Eijmp,
    Elpm,
    ElpmReg,
    ElpmInc,
    Eor,
    Fmul,
    Fmuls,
    Fmulsu,
    Icall,
    Ijmp,
    In,
    Inc,
    Jmp,
    Lac,
    Las,
    Lat,
    Ldi,
    Lds,
    Ldx,
    LdxInc,
    LdxDec,
    Ldy,
    LdyInc,
    LdyDec,
    Lddy,
    Ldz,
    LdzInc,
    LdzDec,
    Lddz,
    Lpm,
    LpmReg,
    LpmInc,
    Lsr,
    Mov,
    Movw,
    Mul,
    Muls,
    Mulsu,
    Neg,
    Nop,
    Or,
    Sbr,
    Out,
    Pop,
    Push,
    Rcall,
    Ret,
    Reti,
    Rjmp,
    Ror,
    Sbc,
    Sbci,
    Sbi,
    Sbic,
    Sbis,
    Sbiw,
    Sbrc,
    Sbrs,
    Sleep,
    Spm,
    SpmInc,
    Sts,
    Stx,
    StxInc,
    StxDec,
    Sty,
    StyInc,
    StyDec,
    Stdy,
    Stz,
    StzInc,
    StzDec,
    Stdz,
    Sub,
    Subi,
    Swap,
    Wdr,
    Xch,
    /// Matched by nothing, `BREAK` among them: executes as a `NOP`.
    Unknown,
}

/// The displacement `q` of `LDD` and `STD`: `10q0 qq.. .... .qqq`.
#[inline]
fn displacement(opcode: u16) -> u16 {
    (opcode & 7) | ((opcode & 0xc00) >> 7) | ((opcode & 0x2000) >> 8)
}

/// avr8js's chain of tests, in its order.
fn decode(opcode: u16) -> Op {
    use Op::*;
    let o = opcode;
    if (o & 0xfc00) == 0x1c00 {
        Adc
    } else if (o & 0xfc00) == 0xc00 {
        Add
    } else if (o & 0xff00) == 0x9600 {
        Adiw
    } else if (o & 0xfc00) == 0x2000 {
        And
    } else if (o & 0xf000) == 0x7000 {
        Andi
    } else if (o & 0xfe0f) == 0x9405 {
        Asr
    } else if (o & 0xff8f) == 0x9488 {
        Bclr
    } else if (o & 0xfe08) == 0xf800 {
        Bld
    } else if (o & 0xfc00) == 0xf400 {
        Brbc
    } else if (o & 0xfc00) == 0xf000 {
        Brbs
    } else if (o & 0xff8f) == 0x9408 {
        Bset
    } else if (o & 0xfe08) == 0xfa00 {
        Bst
    } else if (o & 0xfe0e) == 0x940e {
        Call
    } else if (o & 0xff00) == 0x9800 {
        Cbi
    } else if (o & 0xfe0f) == 0x9400 {
        Com
    } else if (o & 0xfc00) == 0x1400 {
        Cp
    } else if (o & 0xfc00) == 0x400 {
        Cpc
    } else if (o & 0xf000) == 0x3000 {
        Cpi
    } else if (o & 0xfc00) == 0x1000 {
        Cpse
    } else if (o & 0xfe0f) == 0x940a {
        Dec
    } else if o == 0x9519 {
        Eicall
    } else if o == 0x9419 {
        Eijmp
    } else if o == 0x95d8 {
        Elpm
    } else if (o & 0xfe0f) == 0x9006 {
        ElpmReg
    } else if (o & 0xfe0f) == 0x9007 {
        ElpmInc
    } else if (o & 0xfc00) == 0x2400 {
        Eor
    } else if (o & 0xff88) == 0x308 {
        Fmul
    } else if (o & 0xff88) == 0x380 {
        Fmuls
    } else if (o & 0xff88) == 0x388 {
        Fmulsu
    } else if o == 0x9509 {
        Icall
    } else if o == 0x9409 {
        Ijmp
    } else if (o & 0xf800) == 0xb000 {
        In
    } else if (o & 0xfe0f) == 0x9403 {
        Inc
    } else if (o & 0xfe0e) == 0x940c {
        Jmp
    } else if (o & 0xfe0f) == 0x9206 {
        Lac
    } else if (o & 0xfe0f) == 0x9205 {
        Las
    } else if (o & 0xfe0f) == 0x9207 {
        Lat
    } else if (o & 0xf000) == 0xe000 {
        Ldi
    } else if (o & 0xfe0f) == 0x9000 {
        Lds
    } else if (o & 0xfe0f) == 0x900c {
        Ldx
    } else if (o & 0xfe0f) == 0x900d {
        LdxInc
    } else if (o & 0xfe0f) == 0x900e {
        LdxDec
    } else if (o & 0xfe0f) == 0x8008 {
        Ldy
    } else if (o & 0xfe0f) == 0x9009 {
        LdyInc
    } else if (o & 0xfe0f) == 0x900a {
        LdyDec
    } else if (o & 0xd208) == 0x8008 && displacement(o) != 0 {
        Lddy
    } else if (o & 0xfe0f) == 0x8000 {
        Ldz
    } else if (o & 0xfe0f) == 0x9001 {
        LdzInc
    } else if (o & 0xfe0f) == 0x9002 {
        LdzDec
    } else if (o & 0xd208) == 0x8000 && displacement(o) != 0 {
        Lddz
    } else if o == 0x95c8 {
        Lpm
    } else if (o & 0xfe0f) == 0x9004 {
        LpmReg
    } else if (o & 0xfe0f) == 0x9005 {
        LpmInc
    } else if (o & 0xfe0f) == 0x9406 {
        Lsr
    } else if (o & 0xfc00) == 0x2c00 {
        Mov
    } else if (o & 0xff00) == 0x100 {
        Movw
    } else if (o & 0xfc00) == 0x9c00 {
        Mul
    } else if (o & 0xff00) == 0x200 {
        Muls
    } else if (o & 0xff88) == 0x300 {
        Mulsu
    } else if (o & 0xfe0f) == 0x9401 {
        Neg
    } else if o == 0 {
        Nop
    } else if (o & 0xfc00) == 0x2800 {
        Or
    } else if (o & 0xf000) == 0x6000 {
        Sbr
    } else if (o & 0xf800) == 0xb800 {
        Out
    } else if (o & 0xfe0f) == 0x900f {
        Pop
    } else if (o & 0xfe0f) == 0x920f {
        Push
    } else if (o & 0xf000) == 0xd000 {
        Rcall
    } else if o == 0x9508 {
        Ret
    } else if o == 0x9518 {
        Reti
    } else if (o & 0xf000) == 0xc000 {
        Rjmp
    } else if (o & 0xfe0f) == 0x9407 {
        Ror
    } else if (o & 0xfc00) == 0x800 {
        Sbc
    } else if (o & 0xf000) == 0x4000 {
        Sbci
    } else if (o & 0xff00) == 0x9a00 {
        Sbi
    } else if (o & 0xff00) == 0x9900 {
        Sbic
    } else if (o & 0xff00) == 0x9b00 {
        Sbis
    } else if (o & 0xff00) == 0x9700 {
        Sbiw
    } else if (o & 0xfe08) == 0xfc00 {
        Sbrc
    } else if (o & 0xfe08) == 0xfe00 {
        Sbrs
    } else if o == 0x9588 {
        Sleep
    } else if o == 0x95e8 {
        Spm
    } else if o == 0x95f8 {
        SpmInc
    } else if (o & 0xfe0f) == 0x9200 {
        Sts
    } else if (o & 0xfe0f) == 0x920c {
        Stx
    } else if (o & 0xfe0f) == 0x920d {
        StxInc
    } else if (o & 0xfe0f) == 0x920e {
        StxDec
    } else if (o & 0xfe0f) == 0x8208 {
        Sty
    } else if (o & 0xfe0f) == 0x9209 {
        StyInc
    } else if (o & 0xfe0f) == 0x920a {
        StyDec
    } else if (o & 0xd208) == 0x8208 && displacement(o) != 0 {
        Stdy
    } else if (o & 0xfe0f) == 0x8200 {
        Stz
    } else if (o & 0xfe0f) == 0x9201 {
        StzInc
    } else if (o & 0xfe0f) == 0x9202 {
        StzDec
    } else if (o & 0xd208) == 0x8200 && displacement(o) != 0 {
        Stdz
    } else if (o & 0xfc00) == 0x1800 {
        Sub
    } else if (o & 0xf000) == 0x5000 {
        Subi
    } else if (o & 0xfe0f) == 0x9402 {
        Swap
    } else if o == 0x95a8 {
        Wdr
    } else if (o & 0xfe0f) == 0x9204 {
        Xch
    } else {
        Unknown
    }
}

/// [`decode`] for every opcode, built once.
fn decode_table() -> &'static [Op; 0x10000] {
    static TABLE: OnceLock<Box<[Op; 0x10000]>> = OnceLock::new();
    TABLE.get_or_init(|| {
        (0..=u16::MAX)
            .map(decode)
            .collect::<Vec<_>>()
            .into_boxed_slice()
            .try_into()
            .expect("one entry per opcode")
    })
}

fn is_two_word_instruction(opcode: u16) -> bool {
    /* LDS */
    (opcode & 0xfe0f) == 0x9000
        /* STS */
        || (opcode & 0xfe0f) == 0x9200
        /* CALL */
        || (opcode & 0xfe0e) == 0x940e
        /* JMP */
        || (opcode & 0xfe0e) == 0x940c
}

const X: u16 = 26;
const Y: u16 = 28;
const Z: u16 = 30;
const RAMPZ: usize = 0x5b;
const EIND: usize = 0x5c;
const S: usize = SREG as usize;

/// Rd, five bits: `.... ...d dddd ....`.
#[inline]
fn rd(opcode: u16) -> usize {
    ((opcode & 0x1f0) >> 4) as usize
}

/// Rr, five bits: `.... ..r. .... rrrr`.
#[inline]
fn rr(opcode: u16) -> usize {
    ((opcode & 0xf) | ((opcode & 0x200) >> 5)) as usize
}

/// Rd of the immediate forms, r16 to r31: `.... .... dddd ....`.
#[inline]
fn rd_high(opcode: u16) -> usize {
    (((opcode & 0xf0) >> 4) + 16) as usize
}

/// K of the immediate forms: `.... KKKK .... KKKK`.
#[inline]
fn imm8(opcode: u16) -> i32 {
    ((opcode & 0xf) | ((opcode & 0xf00) >> 4)) as i32
}

/// A relative jump's offset, sign-extended from `bits` bits.
#[inline]
fn signed(value: u16, bits: u32) -> i32 {
    let shift = 32 - bits;
    ((value as i32) << shift) >> shift
}

/// S = N ⊕ V, from the N and V already in `sreg`.
#[inline]
fn with_s(sreg: u8) -> u8 {
    sreg | if ((sreg >> 2) & 1) ^ ((sreg >> 3) & 1) != 0 {
        0x10
    } else {
        0
    }
}

#[inline]
fn flag(condition: bool, bit: u8) -> u8 {
    if condition { bit } else { 0 }
}

/// Pushes a return address: two bytes, or three with a 22-bit PC.
#[inline]
fn push_return(cpu: &mut Cpu, ret: u32) {
    let sp = cpu.sp();
    cpu.data[sp as usize] = ret as u8;
    cpu.data[sp.wrapping_sub(1) as usize] = (ret >> 8) as u8;
    if cpu.pc22_bits {
        cpu.data[sp.wrapping_sub(2) as usize] = (ret >> 16) as u8;
    }
    cpu.set_sp(sp.wrapping_sub(if cpu.pc22_bits { 3 } else { 2 }));
}

/// Pops a return address into `pc`, less one for the increment to come.
#[inline]
fn pop_return(cpu: &mut Cpu) {
    let i = cpu.sp().wrapping_add(if cpu.pc22_bits { 3 } else { 2 });
    cpu.set_sp(i);
    let high = cpu.data[i.wrapping_sub(1) as usize] as u32;
    let low = cpu.data[i as usize] as u32;
    cpu.pc = ((high << 8) + low).wrapping_sub(1);
    if cpu.pc22_bits {
        cpu.pc |= (cpu.data[i.wrapping_sub(2) as usize] as u32) << 16;
    }
}

/// Skips the next instruction, one word or two.
#[inline]
fn skip(cpu: &mut Cpu) {
    let next_opcode = cpu.prog_word(cpu.pc + 1);
    let skip_size = if is_two_word_instruction(next_opcode) {
        2
    } else {
        1
    };
    cpu.pc += skip_size;
    cpu.cycles += skip_size as u64;
}

/// `pc + offset`, where the offset may be negative.
#[inline]
fn relative(pc: u32, offset: i32) -> u32 {
    pc.wrapping_add(offset as u32)
}

/// Executes the instruction at `pc`.
pub fn avr_instruction<H: Hooks + ?Sized>(cpu: &mut Cpu, hooks: &mut H) {
    let opcode = cpu.prog_word(cpu.pc);

    match decode_table()[opcode as usize] {
        Op::Adc => {
            /* ADC, 0001 11rd dddd rrrr */
            let d = cpu.data[rd(opcode)] as u32;
            let r = cpu.data[rr(opcode)] as u32;
            let sum = d + r + (cpu.data[S] & 1) as u32;
            let result = sum & 255;
            cpu.data[rd(opcode)] = result as u8;
            let mut sreg = cpu.data[S] & 0xc0;
            sreg |= flag(result == 0, 2);
            sreg |= flag(128 & result != 0, 4);
            sreg |= flag((result ^ r) & (d ^ result) & 128 != 0, 8);
            sreg = with_s(sreg);
            sreg |= flag(sum & 256 != 0, 1);
            // avr8js: `1 & (…)`, bit 0; the half-carry is out of bit 3.
            sreg |= flag(8 & ((d & r) | (r & !result) | (!result & d)) != 0, 0x20);
            cpu.data[S] = sreg;
        }
        Op::Add => {
            /* ADD, 0000 11rd dddd rrrr */
            let d = cpu.data[rd(opcode)] as u32;
            let r = cpu.data[rr(opcode)] as u32;
            let result = (d + r) & 255;
            cpu.data[rd(opcode)] = result as u8;
            let mut sreg = cpu.data[S] & 0xc0;
            sreg |= flag(result == 0, 2);
            sreg |= flag(128 & result != 0, 4);
            sreg |= flag((result ^ r) & (result ^ d) & 128 != 0, 8);
            sreg = with_s(sreg);
            sreg |= flag((d + r) & 256 != 0, 1);
            // avr8js: bit 0, as in ADC.
            sreg |= flag(8 & ((d & r) | (r & !result) | (!result & d)) != 0, 0x20);
            cpu.data[S] = sreg;
        }
        Op::Adiw => {
            /* ADIW, 1001 0110 KKdd KKKK */
            let addr = 2 * ((opcode & 0x30) >> 4) + 24;
            let value = cpu.data16(addr) as u32;
            let result = (value + ((opcode & 0xf) | ((opcode & 0xc0) >> 2)) as u32) & 0xffff;
            cpu.set_data16(addr, result as u16);
            let mut sreg = cpu.data[S] & 0xe0;
            sreg |= flag(result == 0, 2);
            sreg |= flag(0x8000 & result != 0, 4);
            sreg |= flag(!value & result & 0x8000 != 0, 8);
            sreg = with_s(sreg);
            sreg |= flag(!result & value & 0x8000 != 0, 1);
            cpu.data[S] = sreg;
            cpu.cycles += 1;
        }
        Op::And => {
            /* AND, 0010 00rd dddd rrrr */
            let result = cpu.data[rd(opcode)] & cpu.data[rr(opcode)];
            cpu.data[rd(opcode)] = result;
            logic_flags(cpu, result);
        }
        Op::Andi => {
            /* ANDI, 0111 KKKK dddd KKKK */
            let result = cpu.data[rd_high(opcode)] & imm8(opcode) as u8;
            cpu.data[rd_high(opcode)] = result;
            logic_flags(cpu, result);
        }
        Op::Asr => {
            /* ASR, 1001 010d dddd 0101 */
            let value = cpu.data[rd(opcode)];
            let result = (value >> 1) | (128 & value);
            cpu.data[rd(opcode)] = result;
            shift_flags(cpu, result, value & 1);
        }
        Op::Bclr => {
            /* BCLR, 1001 0100 1sss 1000 */
            cpu.data[S] &= !(1 << ((opcode & 0x70) >> 4));
        }
        Op::Bld => {
            /* BLD, 1111 100d dddd 0bbb */
            let b = opcode & 7;
            let d = rd(opcode);
            cpu.data[d] = (!(1 << b) & cpu.data[d]) | (((cpu.data[S] >> 6) & 1) << b);
        }
        Op::Brbc => {
            /* BRBC, 1111 01kk kkkk ksss */
            if cpu.data[S] & (1 << (opcode & 7)) == 0 {
                cpu.pc = relative(cpu.pc, signed((opcode & 0x3f8) >> 3, 7));
                cpu.cycles += 1;
            }
        }
        Op::Brbs => {
            /* BRBS, 1111 00kk kkkk ksss */
            if cpu.data[S] & (1 << (opcode & 7)) != 0 {
                cpu.pc = relative(cpu.pc, signed((opcode & 0x3f8) >> 3, 7));
                cpu.cycles += 1;
            }
        }
        Op::Bset => {
            /* BSET, 1001 0100 0sss 1000 */
            cpu.data[S] |= 1 << ((opcode & 0x70) >> 4);
        }
        Op::Bst => {
            /* BST, 1111 101d dddd 0bbb */
            let d = cpu.data[rd(opcode)];
            let b = opcode & 7;
            cpu.data[S] = (cpu.data[S] & 0xbf) | flag((d >> b) & 1 != 0, 0x40);
        }
        Op::Call => {
            /* CALL, 1001 010k kkkk 111k kkkk kkkk kkkk kkkk */
            let k = cpu.prog_word(cpu.pc + 1) as u32
                | ((opcode as u32 & 1) << 16)
                | ((opcode as u32 & 0x1f0) << 13);
            let ret = cpu.pc + 2;
            push_return(cpu, ret);
            cpu.pc = k.wrapping_sub(1);
            cpu.cycles += if cpu.pc22_bits { 4 } else { 3 };
        }
        Op::Cbi => {
            /* CBI, 1001 1000 AAAA Abbb */
            let a = ((opcode & 0xf8) >> 3) + 32;
            let mask = 1 << (opcode & 7);
            let value = cpu.read_data(hooks, a);
            cpu.write_data(hooks, a, value & !mask, mask);
            // avr8js: one cycle; the datasheet, and SBI here, take two.
            cpu.cycles += 1;
        }
        Op::Com => {
            /* COM, 1001 010d dddd 0000 */
            let d = rd(opcode);
            let result = 255 - cpu.data[d];
            cpu.data[d] = result;
            let mut sreg = (cpu.data[S] & 0xe1) | 1;
            sreg |= flag(result == 0, 2);
            sreg |= flag(128 & result != 0, 4);
            cpu.data[S] = with_s(sreg);
        }
        Op::Cp => {
            /* CP, 0001 01rd dddd rrrr */
            let val1 = cpu.data[rd(opcode)] as i32;
            let val2 = cpu.data[rr(opcode)] as i32;
            cpu.data[S] = subtract_flags(cpu.data[S], val1, val2, val1 - val2);
        }
        Op::Cpc => {
            /* CPC, 0000 01rd dddd rrrr */
            let arg1 = cpu.data[rd(opcode)] as i32;
            let arg2 = cpu.data[rr(opcode)] as i32;
            let result = arg1 - arg2 - (cpu.data[S] & 1) as i32;
            cpu.data[S] = subtract_carry_flags(cpu.data[S], arg1, arg2, result);
        }
        Op::Cpi => {
            /* CPI, 0011 KKKK dddd KKKK */
            let arg1 = cpu.data[rd_high(opcode)] as i32;
            let arg2 = imm8(opcode);
            cpu.data[S] = subtract_flags(cpu.data[S], arg1, arg2, arg1 - arg2);
        }
        Op::Cpse => {
            /* CPSE, 0001 00rd dddd rrrr */
            if cpu.data[rd(opcode)] == cpu.data[rr(opcode)] {
                skip(cpu);
            }
        }
        Op::Dec => {
            /* DEC, 1001 010d dddd 1010 */
            let value = cpu.data[rd(opcode)];
            let result = value.wrapping_sub(1);
            cpu.data[rd(opcode)] = result;
            let mut sreg = cpu.data[S] & 0xe1;
            sreg |= flag(result == 0, 2);
            sreg |= flag(128 & result != 0, 4);
            sreg |= flag(value == 128, 8);
            cpu.data[S] = with_s(sreg);
        }
        Op::Eicall => {
            /* EICALL, 1001 0101 0001 1001 */
            let ret = cpu.pc + 1;
            let sp = cpu.sp();
            let eind = cpu.data[EIND] as u32;
            cpu.data[sp as usize] = ret as u8;
            cpu.data[sp.wrapping_sub(1) as usize] = (ret >> 8) as u8;
            cpu.data[sp.wrapping_sub(2) as usize] = (ret >> 16) as u8;
            cpu.set_sp(sp.wrapping_sub(3));
            cpu.pc = ((eind << 16) | cpu.data16(Z) as u32).wrapping_sub(1);
            cpu.cycles += 3;
        }
        Op::Eijmp => {
            /* EIJMP, 1001 0100 0001 1001 */
            let eind = cpu.data[EIND] as u32;
            cpu.pc = ((eind << 16) | cpu.data16(Z) as u32).wrapping_sub(1);
            cpu.cycles += 1;
        }
        Op::Elpm => {
            /* ELPM, 1001 0101 1101 1000 */
            let rampz = cpu.data[RAMPZ] as u32;
            cpu.data[0] = cpu.prog_byte((rampz << 16) | cpu.data16(Z) as u32);
            cpu.cycles += 2;
        }
        Op::ElpmReg => {
            /* ELPM(REG), 1001 000d dddd 0110 */
            let rampz = cpu.data[RAMPZ] as u32;
            cpu.data[rd(opcode)] = cpu.prog_byte((rampz << 16) | cpu.data16(Z) as u32);
            cpu.cycles += 2;
        }
        Op::ElpmInc => {
            /* ELPM(INC), 1001 000d dddd 0111 */
            let rampz = cpu.data[RAMPZ] as u32;
            let i = cpu.data16(Z);
            cpu.data[rd(opcode)] = cpu.prog_byte((rampz << 16) | i as u32);
            cpu.set_data16(Z, i.wrapping_add(1));
            if i == 0xffff {
                // Upstream divides by zero, and stores 0, on a part with
                // less than 64 KB of flash; so does this.
                let pages = (cpu.prog_bytes_len() >> 16) as u32;
                cpu.data[RAMPZ] = (rampz + 1).checked_rem(pages).unwrap_or(0) as u8;
            }
            cpu.cycles += 2;
        }
        Op::Eor => {
            /* EOR, 0010 01rd dddd rrrr */
            let result = cpu.data[rd(opcode)] ^ cpu.data[rr(opcode)];
            cpu.data[rd(opcode)] = result;
            logic_flags(cpu, result);
        }
        Op::Fmul => {
            /* FMUL, 0000 0011 0ddd 1rrr */
            let v1 = cpu.data[(((opcode & 0x70) >> 4) + 16) as usize] as i32;
            let v2 = cpu.data[((opcode & 7) + 16) as usize] as i32;
            multiply(cpu, v1 * v2, true);
        }
        Op::Fmuls => {
            /* FMULS, 0000 0011 1ddd 0rrr */
            let v1 = cpu.data[(((opcode & 0x70) >> 4) + 16) as usize] as i8 as i32;
            let v2 = cpu.data[((opcode & 7) + 16) as usize] as i8 as i32;
            multiply(cpu, v1 * v2, true);
        }
        Op::Fmulsu => {
            /* FMULSU, 0000 0011 1ddd 1rrr */
            let v1 = cpu.data[(((opcode & 0x70) >> 4) + 16) as usize] as i8 as i32;
            let v2 = cpu.data[((opcode & 7) + 16) as usize] as i32;
            // avr8js: Z is set on a non-zero product; here as FMUL sets it.
            multiply(cpu, v1 * v2, true);
        }
        Op::Icall => {
            /* ICALL, 1001 0101 0000 1001 */
            let ret = cpu.pc + 1;
            push_return(cpu, ret);
            cpu.pc = (cpu.data16(Z) as u32).wrapping_sub(1);
            cpu.cycles += if cpu.pc22_bits { 3 } else { 2 };
        }
        Op::Ijmp => {
            /* IJMP, 1001 0100 0000 1001 */
            cpu.pc = (cpu.data16(Z) as u32).wrapping_sub(1);
            cpu.cycles += 1;
        }
        Op::In => {
            /* IN, 1011 0AAd dddd AAAA */
            let i = cpu.read_data(hooks, ((opcode & 0xf) | ((opcode & 0x600) >> 5)) + 32);
            cpu.data[rd(opcode)] = i;
        }
        Op::Inc => {
            /* INC, 1001 010d dddd 0011 */
            let d = cpu.data[rd(opcode)];
            let result = d.wrapping_add(1);
            cpu.data[rd(opcode)] = result;
            let mut sreg = cpu.data[S] & 0xe1;
            sreg |= flag(result == 0, 2);
            sreg |= flag(128 & result != 0, 4);
            sreg |= flag(d == 127, 8);
            cpu.data[S] = with_s(sreg);
        }
        Op::Jmp => {
            /* JMP, 1001 010k kkkk 110k kkkk kkkk kkkk kkkk */
            let k = cpu.prog_word(cpu.pc + 1) as u32
                | ((opcode as u32 & 1) << 16)
                | ((opcode as u32 & 0x1f0) << 13);
            cpu.pc = k.wrapping_sub(1);
            cpu.cycles += 2;
        }
        Op::Lac => {
            /* LAC, 1001 001r rrrr 0110 */
            let r = rd(opcode);
            let clear = cpu.data[r];
            let z = cpu.data16(Z);
            let value = cpu.read_data(hooks, z);
            cpu.write_data(hooks, z, value & (255 - clear), 0xff);
            cpu.data[r] = value;
        }
        Op::Las => {
            /* LAS, 1001 001r rrrr 0101 */
            let r = rd(opcode);
            let set = cpu.data[r];
            let z = cpu.data16(Z);
            let value = cpu.read_data(hooks, z);
            cpu.write_data(hooks, z, value | set, 0xff);
            cpu.data[r] = value;
        }
        Op::Lat => {
            /* LAT, 1001 001r rrrr 0111 */
            let r = cpu.data[rd(opcode)];
            let z = cpu.data16(Z);
            let value = cpu.read_data(hooks, z);
            cpu.write_data(hooks, z, r ^ value, 0xff);
            cpu.data[rd(opcode)] = value;
        }
        Op::Ldi => {
            /* LDI, 1110 KKKK dddd KKKK */
            cpu.data[rd_high(opcode)] = imm8(opcode) as u8;
        }
        Op::Lds => {
            /* LDS, 1001 000d dddd 0000 kkkk kkkk kkkk kkkk */
            cpu.cycles += 1;
            let addr = cpu.prog_word(cpu.pc + 1);
            let value = cpu.read_data(hooks, addr);
            cpu.data[rd(opcode)] = value;
            cpu.pc += 1;
        }
        Op::Ldx => load(cpu, hooks, opcode, X, 0, 0),
        Op::LdxInc => load(cpu, hooks, opcode, X, 0, 1),
        Op::LdxDec => load(cpu, hooks, opcode, X, -1, 0),
        Op::Ldy => load(cpu, hooks, opcode, Y, 0, 0),
        Op::LdyInc => load(cpu, hooks, opcode, Y, 0, 1),
        Op::LdyDec => load(cpu, hooks, opcode, Y, -1, 0),
        Op::Lddy => {
            /* LDDY, 10q0 qq0d dddd 1qqq */
            cpu.cycles += 1;
            let addr = cpu.data16(Y).wrapping_add(displacement(opcode));
            cpu.data[rd(opcode)] = cpu.read_data(hooks, addr);
        }
        Op::Ldz => load(cpu, hooks, opcode, Z, 0, 0),
        Op::LdzInc => load(cpu, hooks, opcode, Z, 0, 1),
        Op::LdzDec => load(cpu, hooks, opcode, Z, -1, 0),
        Op::Lddz => {
            /* LDDZ, 10q0 qq0d dddd 0qqq */
            cpu.cycles += 1;
            let addr = cpu.data16(Z).wrapping_add(displacement(opcode));
            cpu.data[rd(opcode)] = cpu.read_data(hooks, addr);
        }
        Op::Lpm => {
            /* LPM, 1001 0101 1100 1000 */
            cpu.data[0] = cpu.prog_byte(cpu.data16(Z) as u32);
            cpu.cycles += 2;
        }
        Op::LpmReg => {
            /* LPM(REG), 1001 000d dddd 0100 */
            cpu.data[rd(opcode)] = cpu.prog_byte(cpu.data16(Z) as u32);
            cpu.cycles += 2;
        }
        Op::LpmInc => {
            /* LPM(INC), 1001 000d dddd 0101 */
            let i = cpu.data16(Z);
            cpu.data[rd(opcode)] = cpu.prog_byte(i as u32);
            cpu.set_data16(Z, i.wrapping_add(1));
            cpu.cycles += 2;
        }
        Op::Lsr => {
            /* LSR, 1001 010d dddd 0110 */
            let value = cpu.data[rd(opcode)];
            let result = value >> 1;
            cpu.data[rd(opcode)] = result;
            shift_flags(cpu, result, value & 1);
        }
        Op::Mov => {
            /* MOV, 0010 11rd dddd rrrr */
            cpu.data[rd(opcode)] = cpu.data[rr(opcode)];
        }
        Op::Movw => {
            /* MOVW, 0000 0001 dddd rrrr */
            let r2 = 2 * (opcode & 0xf) as usize;
            let d2 = 2 * ((opcode & 0xf0) >> 4) as usize;
            cpu.data[d2] = cpu.data[r2];
            cpu.data[d2 + 1] = cpu.data[r2 + 1];
        }
        Op::Mul => {
            /* MUL, 1001 11rd dddd rrrr */
            let product = cpu.data[rd(opcode)] as i32 * cpu.data[rr(opcode)] as i32;
            multiply(cpu, product, false);
        }
        Op::Muls => {
            /* MULS, 0000 0010 dddd rrrr */
            let v1 = cpu.data[rd_high(opcode)] as i8 as i32;
            let v2 = cpu.data[((opcode & 0xf) + 16) as usize] as i8 as i32;
            multiply(cpu, v1 * v2, false);
        }
        Op::Mulsu => {
            /* MULSU, 0000 0011 0ddd 0rrr */
            let v1 = cpu.data[(((opcode & 0x70) >> 4) + 16) as usize] as i8 as i32;
            let v2 = cpu.data[((opcode & 7) + 16) as usize] as i32;
            multiply(cpu, v1 * v2, false);
        }
        Op::Neg => {
            /* NEG, 1001 010d dddd 0001 */
            let d = rd(opcode);
            let value = cpu.data[d] as i32;
            let result = 0 - value;
            cpu.data[d] = result as u8;
            let mut sreg = cpu.data[S] & 0xc0;
            sreg |= flag(result == 0, 2);
            sreg |= flag(128 & result != 0, 4);
            // avr8js: `128 === R`, never true of a negated byte.
            sreg |= flag(result & 0xff == 0x80, 8);
            sreg = with_s(sreg);
            sreg |= flag(result != 0, 1);
            // avr8js: bit 0, as in ADC.
            sreg |= flag(8 & (result | value) != 0, 0x20);
            cpu.data[S] = sreg;
        }
        Op::Nop => { /* NOP, 0000 0000 0000 0000 */ }
        Op::Or => {
            /* OR, 0010 10rd dddd rrrr */
            let result = cpu.data[rd(opcode)] | cpu.data[rr(opcode)];
            cpu.data[rd(opcode)] = result;
            logic_flags(cpu, result);
        }
        Op::Sbr => {
            /* SBR, 0110 KKKK dddd KKKK */
            let result = cpu.data[rd_high(opcode)] | imm8(opcode) as u8;
            cpu.data[rd_high(opcode)] = result;
            logic_flags(cpu, result);
        }
        Op::Out => {
            /* OUT, 1011 1AAr rrrr AAAA */
            let value = cpu.data[rd(opcode)];
            cpu.write_data(
                hooks,
                ((opcode & 0xf) | ((opcode & 0x600) >> 5)) + 32,
                value,
                0xff,
            );
        }
        Op::Pop => {
            /* POP, 1001 000d dddd 1111 */
            let value = cpu.sp().wrapping_add(1);
            cpu.set_sp(value);
            cpu.data[rd(opcode)] = cpu.data[value as usize];
            cpu.cycles += 1;
        }
        Op::Push => {
            /* PUSH, 1001 001d dddd 1111 */
            let value = cpu.sp();
            cpu.data[value as usize] = cpu.data[rd(opcode)];
            cpu.set_sp(value.wrapping_sub(1));
            cpu.cycles += 1;
        }
        Op::Rcall => {
            /* RCALL, 1101 kkkk kkkk kkkk */
            let k = signed(opcode & 0xfff, 12);
            let ret = cpu.pc + 1;
            push_return(cpu, ret);
            cpu.pc = relative(cpu.pc, k);
            cpu.cycles += if cpu.pc22_bits { 3 } else { 2 };
        }
        Op::Ret => {
            /* RET, 1001 0101 0000 1000 */
            pop_return(cpu);
            cpu.cycles += if cpu.pc22_bits { 4 } else { 3 };
        }
        Op::Reti => {
            /* RETI, 1001 0101 0001 1000 */
            pop_return(cpu);
            cpu.cycles += if cpu.pc22_bits { 4 } else { 3 };
            cpu.data[S] |= 0x80; // Enable interrupts
        }
        Op::Rjmp => {
            /* RJMP, 1100 kkkk kkkk kkkk */
            cpu.pc = relative(cpu.pc, signed(opcode & 0xfff, 12));
            cpu.cycles += 1;
        }
        Op::Ror => {
            /* ROR, 1001 010d dddd 0111 */
            let d = cpu.data[rd(opcode)];
            let result = (d >> 1) | ((cpu.data[S] & 1) << 7);
            cpu.data[rd(opcode)] = result;
            shift_flags(cpu, result, d & 1);
        }
        Op::Sbc => {
            /* SBC, 0000 10rd dddd rrrr */
            let val1 = cpu.data[rd(opcode)] as i32;
            let val2 = cpu.data[rr(opcode)] as i32;
            let result = val1 - val2 - (cpu.data[S] & 1) as i32;
            cpu.data[rd(opcode)] = result as u8;
            cpu.data[S] = subtract_carry_flags(cpu.data[S], val1, val2, result);
        }
        Op::Sbci => {
            /* SBCI, 0100 KKKK dddd KKKK */
            let val1 = cpu.data[rd_high(opcode)] as i32;
            let val2 = imm8(opcode);
            let result = val1 - val2 - (cpu.data[S] & 1) as i32;
            cpu.data[rd_high(opcode)] = result as u8;
            cpu.data[S] = subtract_carry_flags(cpu.data[S], val1, val2, result);
        }
        Op::Sbi => {
            /* SBI, 1001 1010 AAAA Abbb */
            let target = ((opcode & 0xf8) >> 3) + 32;
            let mask = 1 << (opcode & 7);
            let value = cpu.read_data(hooks, target);
            cpu.write_data(hooks, target, value | mask, mask);
            cpu.cycles += 1;
        }
        Op::Sbic => {
            /* SBIC, 1001 1001 AAAA Abbb */
            let value = cpu.read_data(hooks, ((opcode & 0xf8) >> 3) + 32);
            if value & (1 << (opcode & 7)) == 0 {
                skip(cpu);
            }
        }
        Op::Sbis => {
            /* SBIS, 1001 1011 AAAA Abbb */
            let value = cpu.read_data(hooks, ((opcode & 0xf8) >> 3) + 32);
            if value & (1 << (opcode & 7)) != 0 {
                skip(cpu);
            }
        }
        Op::Sbiw => {
            /* SBIW, 1001 0111 KKdd KKKK */
            let i = 2 * ((opcode & 0x30) >> 4) + 24;
            let a = cpu.data16(i) as i32;
            let l = ((opcode & 0xf) | ((opcode & 0xc0) >> 2)) as i32;
            let result = a - l;
            cpu.set_data16(i, result as u16);
            // avr8js: H is cleared and recomputed; SBIW does not touch it.
            let mut sreg = cpu.data[S] & 0xe0;
            sreg |= flag(result == 0, 2);
            sreg |= flag(0x8000 & result != 0, 4);
            sreg |= flag(a & !result & 0x8000 != 0, 8);
            sreg = with_s(sreg);
            sreg |= flag(l > a, 1);
            cpu.data[S] = sreg;
            cpu.cycles += 1;
        }
        Op::Sbrc => {
            /* SBRC, 1111 110r rrrr 0bbb */
            if cpu.data[rd(opcode)] & (1 << (opcode & 7)) == 0 {
                skip(cpu);
            }
        }
        Op::Sbrs => {
            /* SBRS, 1111 111r rrrr 0bbb */
            if cpu.data[rd(opcode)] & (1 << (opcode & 7)) != 0 {
                skip(cpu);
            }
        }
        Op::Sleep => {
            /* SLEEP, 1001 0101 1000 1000 */
            /* not implemented */
        }
        Op::Spm => {
            /* SPM, 1001 0101 1110 1000 */
            /* not implemented */
        }
        Op::SpmInc => {
            /* SPM(INC), 1001 0101 1111 1000 */
            /* not implemented */
        }
        Op::Sts => {
            /* STS, 1001 001d dddd 0000 kkkk kkkk kkkk kkkk */
            let value = cpu.data[rd(opcode)];
            let addr = cpu.prog_word(cpu.pc + 1);
            cpu.write_data(hooks, addr, value, 0xff);
            cpu.pc += 1;
            cpu.cycles += 1;
        }
        Op::Stx => store(cpu, hooks, opcode, X, 0, 0),
        Op::StxInc => store(cpu, hooks, opcode, X, 0, 1),
        Op::StxDec => store(cpu, hooks, opcode, X, -1, 0),
        Op::Sty => store(cpu, hooks, opcode, Y, 0, 0),
        Op::StyInc => store(cpu, hooks, opcode, Y, 0, 1),
        Op::StyDec => store(cpu, hooks, opcode, Y, -1, 0),
        Op::Stdy => {
            /* STDY, 10q0 qq1r rrrr 1qqq */
            let addr = cpu.data16(Y).wrapping_add(displacement(opcode));
            let value = cpu.data[rd(opcode)];
            cpu.write_data(hooks, addr, value, 0xff);
            cpu.cycles += 1;
        }
        Op::Stz => store(cpu, hooks, opcode, Z, 0, 0),
        Op::StzInc => store(cpu, hooks, opcode, Z, 0, 1),
        Op::StzDec => store(cpu, hooks, opcode, Z, -1, 0),
        Op::Stdz => {
            /* STDZ, 10q0 qq1r rrrr 0qqq */
            let addr = cpu.data16(Z).wrapping_add(displacement(opcode));
            let value = cpu.data[rd(opcode)];
            cpu.write_data(hooks, addr, value, 0xff);
            cpu.cycles += 1;
        }
        Op::Sub => {
            /* SUB, 0001 10rd dddd rrrr */
            let val1 = cpu.data[rd(opcode)] as i32;
            let val2 = cpu.data[rr(opcode)] as i32;
            let result = val1 - val2;
            cpu.data[rd(opcode)] = result as u8;
            cpu.data[S] = subtract_flags(cpu.data[S], val1, val2, result);
        }
        Op::Subi => {
            /* SUBI, 0101 KKKK dddd KKKK */
            let val1 = cpu.data[rd_high(opcode)] as i32;
            let val2 = imm8(opcode);
            let result = val1 - val2;
            cpu.data[rd_high(opcode)] = result as u8;
            cpu.data[S] = subtract_flags(cpu.data[S], val1, val2, result);
        }
        Op::Swap => {
            /* SWAP, 1001 010d dddd 0010 */
            let d = rd(opcode);
            let i = cpu.data[d];
            cpu.data[d] = i.rotate_left(4);
        }
        Op::Wdr => {
            /* WDR, 1001 0101 1010 1000 */
            hooks.watchdog_reset(cpu);
        }
        Op::Xch => {
            /* XCH, 1001 001r rrrr 0100 */
            let r = rd(opcode);
            let z = cpu.data16(Z) as usize;
            let val1 = cpu.data[r];
            let val2 = cpu.data[z];
            cpu.data[z] = val1;
            cpu.data[r] = val2;
        }
        Op::Unknown => {}
    }

    let next = cpu.pc.wrapping_add(1);
    let len = cpu.prog_mem.len() as u32;
    cpu.pc = if next < len {
        next
    } else {
        // Past the end of flash, or before its start: round to it.
        (next as i32 as i64).rem_euclid(len.max(1) as i64) as u32
    };
    cpu.cycles += 1;
}

/// Z, N, S of AND, EOR, OR and their immediates; V cleared.
#[inline]
fn logic_flags(cpu: &mut Cpu, result: u8) {
    let mut sreg = cpu.data[S] & 0xe1;
    sreg |= flag(result == 0, 2);
    sreg |= flag(128 & result != 0, 4);
    cpu.data[S] = with_s(sreg);
}

/// Z, N, C, V = N ⊕ C, S of ASR, LSR and ROR.
#[inline]
fn shift_flags(cpu: &mut Cpu, result: u8, carry: u8) {
    let mut sreg = cpu.data[S] & 0xe0;
    sreg |= flag(result == 0, 2);
    sreg |= flag(128 & result != 0, 4);
    sreg |= carry;
    sreg |= flag(((sreg >> 2) & 1) ^ (sreg & 1) != 0, 8);
    cpu.data[S] = with_s(sreg);
}

/// The flags of `CP`, `CPI`, `SUB` and `SUBI`, given `val1 - val2`.
#[inline]
fn subtract_flags(sreg: u8, val1: i32, val2: i32, result: i32) -> u8 {
    let mut sreg = sreg & 0xc0;
    sreg |= flag(result == 0, 2);
    sreg |= flag(128 & result != 0, 4);
    sreg |= flag((val1 ^ val2) & (val1 ^ result) & 128 != 0, 8);
    sreg = with_s(sreg);
    sreg |= flag(val2 > val1, 1);
    // avr8js: bit 0, as in ADC.
    sreg |= flag(
        8 & ((!val1 & val2) | (val2 & result) | (result & !val1)) != 0,
        0x20,
    );
    sreg
}

/// The flags of `CPC`, `SBC` and `SBCI`, given `val1 - val2 - C`: Z is only
/// ever cleared.
#[inline]
fn subtract_carry_flags(sreg: u8, val1: i32, val2: i32, result: i32) -> u8 {
    let carry = (sreg & 1) as i32;
    // avr8js: `!R`, which misses a result of -256.
    let mut new = (sreg & 0xc0)
        | flag(result & 0xff == 0 && (sreg >> 1) & 1 != 0, 2)
        | flag(val2 + carry > val1, 1);
    new |= flag(128 & result != 0, 4);
    new |= flag((val1 ^ val2) & (val1 ^ result) & 128 != 0, 8);
    new = with_s(new);
    // avr8js: bit 0, as in ADC.
    new |= flag(
        8 & ((!val1 & val2) | (val2 & result) | (result & !val1)) != 0,
        0x20,
    );
    new
}

/// The multiplications: the product (doubled for the fractional ones) to
/// r1:r0, Z and C, and the extra cycle. C is bit 15 of the product before it
/// is doubled.
#[inline]
fn multiply(cpu: &mut Cpu, product: i32, fractional: bool) {
    let result = if fractional { product << 1 } else { product };
    cpu.set_data16(0, result as u16);
    cpu.data[S] =
        (cpu.data[S] & 0xfc) | flag(0xffff & result == 0, 2) | flag(product & 0x8000 != 0, 1);
    cpu.cycles += 1;
}

/// `LD Rd, X|Y|Z`, with the pointer moved by `pre` before and `post` after.
#[inline]
fn load<H: Hooks + ?Sized>(
    cpu: &mut Cpu,
    hooks: &mut H,
    opcode: u16,
    pointer: u16,
    pre: i16,
    post: i16,
) {
    let addr = cpu.data16(pointer).wrapping_add_signed(pre);
    if pre != 0 {
        cpu.set_data16(pointer, addr);
    }
    cpu.cycles += 1;
    cpu.data[rd(opcode)] = cpu.read_data(hooks, addr);
    if post != 0 {
        cpu.set_data16(pointer, addr.wrapping_add_signed(post));
    }
}

/// `ST X|Y|Z, Rr`, with the pointer moved by `pre` before and `post` after.
/// The register is read before the pointer moves, as upstream reads it.
#[inline]
fn store<H: Hooks + ?Sized>(
    cpu: &mut Cpu,
    hooks: &mut H,
    opcode: u16,
    pointer: u16,
    pre: i16,
    post: i16,
) {
    let value = cpu.data[rd(opcode)];
    let addr = cpu.data16(pointer).wrapping_add_signed(pre);
    if pre != 0 {
        cpu.set_data16(pointer, addr);
    }
    cpu.write_data(hooks, addr, value, 0xff);
    if post != 0 {
        cpu.set_data16(pointer, addr.wrapping_add_signed(post));
    }
    cpu.cycles += 1;
}
