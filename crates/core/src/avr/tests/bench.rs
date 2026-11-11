// SPDX-License-Identifier: MIT OR Apache-2.0

//! How fast the port runs, in simulated megahertz: not a test, a measurement
//! (ADR 0017 budgets the program inside every frame's cost). Ignored, so CI
//! does not run it; run it in release to mean anything:
//!
//! ```text
//! cargo test -p pastille-core --release --lib avr::tests::bench -- --ignored --nocapture
//! ```

use std::time::Instant;

use crate::avr::Avr;

/// A delay loop as avr-gcc compiles `_delay_ms` and Arduino's busy waits,
/// with timer0 running as the Arduino core runs it for `millis()`: prescaler
/// 64, overflow interrupt enabled.
///
/// ```text
///     ldi r16, 0x03      ; TCCR0B = CS01 | CS00
///     out 0x25, r16
///     ldi r16, 0x01      ; TIMSK0 = TOIE0
///     sts 0x6e, r16
///     sei
/// loop:
///     ldi r24, 0xff
///     ldi r25, 0xff
/// inner:
///     sbiw r24, 1        ; 2 cycles
///     brne inner         ; 2 cycles taken
///     in r17, 0x26       ; read TCNT0
///     rjmp loop
/// ```
///
/// The overflow vector (word 0x20) is a `reti`; the reset vector jumps over
/// the table to 0x40.
fn program() -> Vec<u16> {
    let mut flash = vec![0u16; 0x60];
    flash[0] = 0xc03f; // rjmp 0x40
    flash[0x20] = 0x9518; // reti
    let body = [
        0xe003, // ldi r16, 0x03
        0xbd05, // out 0x25, r16
        0xe001, // ldi r16, 0x01
        0x9300, 0x006e, // sts 0x6e, r16
        0x9478, // sei
        0xef8f, // loop: ldi r24, 0xff
        0xef9f, // ldi r25, 0xff
        0x9701, // inner: sbiw r24, 1
        0xf7f1, // brne inner
        0xb516, // in r17, 0x26
        0xcffa, // rjmp loop
    ];
    flash[0x40..0x40 + body.len()].copy_from_slice(&body);
    flash
}

#[test]
#[ignore = "a measurement: run in release with --ignored"]
fn simulated_megahertz_of_a_delay_loop_with_timer0_running() {
    let mut avr = Avr::atmega328p(program());
    let cycles = 400_000_000;
    let start = Instant::now();
    while avr.cpu.cycles < cycles {
        avr.step();
    }
    let seconds = start.elapsed().as_secs_f64();
    let mhz = avr.cpu.cycles as f64 / seconds / 1e6;
    println!(
        "{} cycles in {seconds:.3} s: {mhz:.1} MHz simulated, {:.1}× an Uno's 16 MHz",
        avr.cpu.cycles,
        mhz / 16.0
    );
    assert_eq!(avr.cpu.data[0x6e], 1, "the loop ran with TOIE0 set");
}

/// The other extreme: a pin toggled every three cycles, and the CPU stopped
/// on each toggle as the engine will stop it — `run_until`, then the change
/// taken.
///
/// ```text
///     ldi r16, 0x20      ; PB5, the Uno's LED
///     out 0x04, r16      ; DDRB
/// loop:
///     out 0x03, r16      ; PINB: toggle PB5
///     rjmp loop
/// ```
#[test]
#[ignore = "a measurement: run in release with --ignored"]
fn simulated_megahertz_stopping_on_every_pin_change() {
    let mut avr = Avr::atmega328p(vec![0xe200, 0xb904, 0xb903, 0xcffe]);
    let cycles = 100_000_000;
    let mut changes = 0u64;
    let start = Instant::now();
    while avr.run_until(cycles) {
        changes += avr.io.ports[crate::avr::PORT_B].take_change().is_some() as u64;
    }
    let seconds = start.elapsed().as_secs_f64();
    let mhz = avr.cpu.cycles as f64 / seconds / 1e6;
    println!(
        "{} cycles, {changes} stops, in {seconds:.3} s: {mhz:.1} MHz simulated",
        avr.cpu.cycles
    );
    assert!(changes > cycles / 4);
}
