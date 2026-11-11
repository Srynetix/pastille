// SPDX-License-Identifier: MIT OR Apache-2.0
// Ported from avr8js, src/utils/test-utils.ts.
// Copyright (c) 2019-2025 Uri Shaked, MIT licence (see avr/NOTICE.md).

//! avr8js's spec tests, ported: the evidence that the port is the emulator
//! (ADR 0017).
//!
//! One file per spec file, in the order and under the names avr8js gives its
//! cases, so the two can be read side by side. Where a case could not be
//! ported, or this port changed the behaviour it pins, the case says so.

mod adc;
mod bench;
mod gpio;
mod instruction;
mod programs;
mod timer;
mod usart;

use super::cpu::Cpu;
use super::gpio::AvrPortConfig;
use super::timer::AvrTimerConfig;
use super::{Avr, Peripherals};

/// avr8js's `TestProgramRunner` stops on this rather than run it.
const BREAK_OPCODE: u16 = 0x9598;

/// `new CPU(program)` with the given peripherals: flash as long as the
/// program, avr8js's default 8 KB of SRAM.
fn avr_with(program: &[u16], ports: &[AvrPortConfig], timers: &[AvrTimerConfig]) -> Avr {
    let mut cpu = Cpu::new(program.to_vec(), 8192);
    let io = Peripherals::new(&mut cpu, ports, timers);
    Avr { cpu, io }
}

/// `new CPU(new Uint16Array(words))`, with the given peripherals.
fn blank_avr(words: usize, ports: &[AvrPortConfig], timers: &[AvrTimerConfig]) -> Avr {
    avr_with(&vec![0; words], ports, timers)
}

/// `TestProgramRunner.runInstructions`.
fn run_instructions(avr: &mut Avr, count: usize) {
    for _ in 0..count {
        assert_ne!(
            avr.cpu.prog_word(avr.cpu.pc),
            BREAK_OPCODE,
            "BREAK instruction encountered"
        );
        avr.step();
    }
}

/// `TestProgramRunner.runUntil`.
fn run_until(avr: &mut Avr, predicate: impl Fn(&Cpu) -> bool) {
    for _ in 0..5000 {
        assert_ne!(
            avr.cpu.prog_word(avr.cpu.pc),
            BREAK_OPCODE,
            "BREAK instruction encountered"
        );
        if predicate(&avr.cpu) {
            return;
        }
        avr.step();
    }
    panic!("Test program ran for too long, check your predicate");
}

/// `TestProgramRunner.runToAddress`, in bytes.
fn run_to_address(avr: &mut Avr, byte_addr: u32) {
    run_until(avr, |cpu| cpu.pc * 2 == byte_addr);
}
