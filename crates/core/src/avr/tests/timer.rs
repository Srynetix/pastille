// SPDX-License-Identifier: MIT OR Apache-2.0
// Ported from avr8js, src/peripherals/timer.spec.ts.
// Copyright (c) 2019-2025 Uri Shaked, MIT licence (see avr/NOTICE.md).

//! timer.spec.ts, every case: all of them use the ATmega328P's timers, some
//! with bits or an OCRC channel changed to stand for another part's. Where the
//! spec spies on `timerOverridePin`, this reads the port's log of the same
//! calls, emptied where the spec creates its spy.

use super::programs::*;
use super::{avr_with, blank_avr, run_instructions, run_to_address};
use crate::avr::Avr;
use crate::avr::gpio::{PORT_B_CONFIG, PORT_D_CONFIG, PinOverrideMode};
use crate::avr::timer::{AvrTimerConfig, TIMER0_CONFIG, TIMER1_CONFIG, TIMER2_CONFIG};

use PinOverrideMode::{Clear, Enable, Set, Toggle};

// CPU registers
const R1: u16 = 1;
const R17: u16 = 17;
const R18: u16 = 18;
const R19: u16 = 19;
const R20: u16 = 20;
const R21: u16 = 21;
const R22: u16 = 22;
const SREG: usize = 95;

// Timer 0 Registers
const TIFR0: usize = 0x35;
const TCCR0A: u16 = 0x44;
const TCCR0B: u16 = 0x45;
const TCNT0: u16 = 0x46;
const OCR0A: u16 = 0x47;
const OCR0B: u16 = 0x48;
const TIMSK0: u16 = 0x6e;
const TIMSK1: u16 = 0x6f;

// Timer 1 Registers
const TIFR1: usize = 0x36;
const TCCR1A: u16 = 0x80;
const TCCR1B: u16 = 0x81;
const TCCR1C: u16 = 0x82;
const TCNT1: u16 = 0x84;
const TCNT1H: u16 = 0x85;
const ICR1: u16 = 0x86;
const ICR1H: u16 = 0x87;
const OCR1A: u16 = 0x88;
const OCR1AH: u16 = 0x89;
const OCR1B: u16 = 0x8a;
const OCR1C: u16 = 0x8c;
const OCR1CH: u16 = 0x8d;

// Timer 2 Registers
const TCCR2B: u16 = 0xb1;
const TCNT2: u16 = 0xb2;

// Register bit names
const TOV0: u8 = 1;
const TOV1: u8 = 1;
const OCIE0A: u8 = 2;
const OCIE0B: u8 = 4;
const TOIE0: u8 = 1;
const OCF0A: u8 = 2;
const OCF0B: u8 = 4;
const OCF1A: u8 = 1 << 1;
const OCF1B: u8 = 1 << 2;
const OCF1C: u8 = 1 << 3;
const WGM00: u8 = 1;
const WGM10: u8 = 1;
const WGM01: u8 = 2;
const WGM11: u8 = 2;
const WGM12: u8 = 8;
const WGM13: u8 = 16;
const CS00: u8 = 1;
const CS01: u8 = 2;
const CS02: u8 = 4;
const CS10: u8 = 1;
const CS21: u8 = 2;
const CS22: u8 = 4;
const COM0B1: u8 = 1 << 5;
const COM1C0: u8 = 1 << 2;
const FOC0B: u8 = 1 << 6;
const FOC1C: u8 = 1 << 5;

const T0: u8 = 4; // PD4 on ATmega328p

/// `new CPU(new Uint16Array(0x1000))` and `new AVRTimer(cpu, config)`.
fn timer(config: AvrTimerConfig) -> Avr {
    blank_avr(0x1000, &[], &[config])
}

/// The same, with port B or D for the timer to drive, as index 0.
fn timer_and_port(config: AvrTimerConfig, port: crate::avr::gpio::AvrPortConfig) -> Avr {
    blank_avr(0x1000, &[port], &[config])
}

/// A program, timer0 and port D.
fn timer0_program(program: &[u16]) -> Avr {
    avr_with(program, &[PORT_D_CONFIG], &[TIMER0_CONFIG])
}

/// Sets the cycle count and runs what is due, as the spec does by hand.
fn tick_at(avr: &mut Avr, cycles: u64) {
    avr.cpu.cycles = cycles;
    avr.tick();
}

/// Everything the timer told port 0, since the last call.
fn overrides(avr: &mut Avr) -> Vec<(u8, PinOverrideMode)> {
    std::mem::take(&mut avr.io.ports[0].override_log)
}

/// timer1 with an OCRC channel on PB3, as an ATmega2560's has.
const TIMER1_WITH_OCRC: AvrTimerConfig = AvrTimerConfig {
    ocrc: OCR1C,
    ocfc: OCF1C,
    comp_port_c: PORT_B_CONFIG.port,
    comp_pin_c: 3,
    ..TIMER1_CONFIG
};

#[test]
fn updates_every_tick_when_prescaler_is_1() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 2);
    assert_eq!(avr.read_data(TCNT0), 1);
}

#[test]
fn updates_every_64_ticks_when_prescaler_is_3() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCCR0B, CS01 | CS00); // Set prescaler to 64
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 1 + 64);
    assert_eq!(avr.read_data(TCNT0), 1);
}

#[test]
fn does_not_update_when_disabled() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCCR0B, 0); // No prescaler (timer disabled)
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 100000);
    assert_eq!(avr.read_data(TCNT0), 0); // TCNT should stay 0
}

#[test]
fn sets_tov_when_wrapping_above_top() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0xff);
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1

    tick_at(&mut avr, 1);
    assert_eq!(avr.read_data(TCNT0), 0xff);
    assert_eq!(avr.cpu.data[TIFR0] & TOV0, 0);

    tick_at(&mut avr, 2);
    assert_eq!(avr.read_data(TCNT0), 0);
    assert_eq!(avr.cpu.data[TIFR0] & TOV0, TOV0);
}

#[test]
fn sets_tov_when_overflowing_past_top_without_reaching_it() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0xfe);
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    tick_at(&mut avr, 1);
    assert_eq!(avr.read_data(TCNT0), 0xfe);
    tick_at(&mut avr, 5);
    assert_eq!(avr.read_data(TCNT0), 0x2);
    assert_eq!(avr.cpu.data[TIFR0] & TOV0, TOV0);
}

#[test]
fn clears_tov_when_writing_1_to_it_without_taking_the_interrupt() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0xff);
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 2);
    assert_eq!(avr.cpu.data[TIFR0] & TOV0, TOV0);
    avr.write_data(TIFR0 as u16, TOV0);
    assert_eq!(avr.cpu.data[TIFR0] & TOV0, 0);
}

#[test]
fn writing_tifr_leaves_the_flags_written_as_0() {
    // Not in avr8js's spec, where TOV0 is cleared here too.
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0xff);
    avr.write_data(OCR0A, 0);
    avr.write_data(OCR0B, 0x80);
    avr.write_data(TCCR0B, CS00);
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 2);
    assert_eq!(avr.cpu.data[TIFR0], TOV0 | OCF0A);
    avr.write_data(TIFR0 as u16, OCF0A);
    assert_eq!(avr.cpu.data[TIFR0], TOV0);
}

#[test]
fn sets_tov_when_overflowing_in_fast_pwm() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0xff);
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    tick_at(&mut avr, 1);
    avr.write_data(OCR0A, 0x7f);
    avr.write_data(TCCR0A, WGM01 | WGM00); // WGM: Fast PWM
    tick_at(&mut avr, 2);
    assert_eq!(avr.read_data(TCNT0), 0);
    assert_eq!(avr.cpu.data[TIFR0] & TOV0, TOV0);
}

#[test]
fn takes_the_overflow_interrupt_when_enabled() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0xff);
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    tick_at(&mut avr, 1);
    avr.write_data(TIMSK0, TOIE0);
    avr.cpu.data[SREG] = 0x80; // SREG: I-------
    tick_at(&mut avr, 2);
    assert_eq!(avr.read_data(TCNT0), 2); // TCNT should be 2 (one tick above + 2 cycles for interrupt)
    assert_eq!(avr.cpu.data[TIFR0] & TOV0, 0);
    assert_eq!(avr.cpu.pc, 0x20);
    assert_eq!(avr.cpu.cycles, 4);
}

#[test]
fn supports_overriding_the_tov_and_toie_bits() {
    // avr8js issue #64.
    let mut avr = timer(AvrTimerConfig {
        // The following values correspond ATtiny85 config:
        tov: 2,
        ocfa: 2,
        ocfb: 8,
        toie: 2,
        ociea: 16,
        ocieb: 8,
        ..TIMER0_CONFIG
    });
    avr.write_data(TCNT0, 0xff);
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    tick_at(&mut avr, 1);
    avr.write_data(TIMSK0, 2);
    avr.cpu.data[SREG] = 0x80; // SREG: I-------
    tick_at(&mut avr, 2);
    assert_eq!(avr.read_data(TCNT0), 2); // TCNT should be 2 (one tick above + 2 cycles for interrupt)
    assert_eq!(avr.cpu.data[TIFR0] & 2, 0);
    assert_eq!(avr.cpu.pc, 0x20);
    assert_eq!(avr.cpu.cycles, 4);
}

#[test]
fn no_overflow_interrupt_when_global_interrupts_are_disabled() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0xff);
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    tick_at(&mut avr, 1);
    avr.cpu.data[TIMSK0 as usize] = TOIE0;
    avr.cpu.data[SREG] = 0x0; // SREG: --------
    tick_at(&mut avr, 2);
    assert_eq!(avr.cpu.data[TIFR0] & TOV0, TOV0);
    assert_eq!(avr.cpu.pc, 0);
    assert_eq!(avr.cpu.cycles, 2);
}

#[test]
fn no_overflow_interrupt_when_toie0_is_clear() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0xff);
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    tick_at(&mut avr, 1);
    avr.cpu.data[TIMSK0 as usize] = 0;
    avr.cpu.data[SREG] = 0x80; // SREG: I-------
    tick_at(&mut avr, 2);
    assert_eq!(avr.cpu.data[TIFR0] & TOV0, TOV0);
    assert_eq!(avr.cpu.pc, 0);
    assert_eq!(avr.cpu.cycles, 2);
}

#[test]
fn sets_ocf0a_and_ocf0b_when_they_are_0_and_the_timer_equals_them() {
    // avr8js issue #74.
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0xff);
    avr.write_data(OCR0A, 0x0);
    avr.write_data(OCR0B, 0x0);
    avr.write_data(TCCR0A, 0x0); // WGM: Normal
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 2);
    assert_eq!(avr.read_data(TCNT0), 0);
    assert_eq!(avr.cpu.data[TIFR0] & (OCF0A | OCF0B), OCF0A | OCF0B);
    assert_eq!(avr.cpu.pc, 0);
    assert_eq!(avr.cpu.cycles, 2);
}

#[test]
fn sets_ocf1a_when_the_timer_overflowed_past_it_in_wgm_15() {
    // avr8js issue #94.
    let mut avr = timer(TIMER1_CONFIG);
    avr.write_data(TCNT1, 118);
    avr.write_data(OCR1A, 120);
    avr.write_data(OCR1B, 4); // To avoid getting the OCF1B flag set
    avr.write_data(TCCR1A, WGM10 | WGM11); // WGM: Fast PWM
    avr.write_data(TCCR1B, WGM12 | WGM13 | CS10); // Set prescaler to 1
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 5);
    assert_eq!(avr.read_data(TCNT1), 1);
    assert_eq!(avr.cpu.data[TIFR1] & (OCF1A | OCF1B), OCF1A);
    assert_eq!(avr.cpu.pc, 0);
    assert_eq!(avr.cpu.cycles, 5);
}

#[test]
fn sets_ocf0a_when_the_timer_equals_ocra() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0x10);
    avr.write_data(OCR0A, 0x11);
    avr.write_data(TCCR0A, 0x0); // WGM: Normal
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 2);
    assert_eq!(avr.cpu.data[TIFR0], OCF0A);
    assert_eq!(avr.cpu.pc, 0);
    assert_eq!(avr.cpu.cycles, 2);
}

#[test]
fn resets_the_counter_in_ctc_mode_when_it_equals_ocra() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0x10);
    avr.write_data(OCR0A, 0x11);
    avr.write_data(TCCR0A, WGM01); // WGM: CTC
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 3);
    assert_eq!(avr.read_data(TCNT0), 0);
    assert_eq!(avr.cpu.pc, 0);
    assert_eq!(avr.cpu.cycles, 3);
}

#[test]
fn no_tov_when_top_is_below_max_in_ctc_mode() {
    // avr8js issue #75.
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0x1e);
    avr.write_data(OCR0A, 0x1f);
    avr.write_data(TCCR0A, WGM01); // WGM: CTC
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 2);
    tick_at(&mut avr, 3);
    assert_eq!(avr.read_data(TCNT0), 0);
    assert_eq!(avr.cpu.data[TIFR0] & TOV0, 0); // TOV0 clear
}

/// Issues #75 and #80, which the spec tests with the same steps: TOV is set
/// once, on the wrap, when TOP is MAX in CTC mode.
fn tov_once_when_top_is_max_in_ctc_mode() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0xfe);
    avr.write_data(OCR0A, 0xff);
    avr.write_data(TCCR0A, WGM01); // WGM: CTC
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    tick_at(&mut avr, 1);

    tick_at(&mut avr, 2);
    assert_eq!(avr.read_data(TCNT0), 0xff);
    assert_eq!(avr.cpu.data[TIFR0] & TOV0, 0); // TOV clear

    tick_at(&mut avr, 3);
    assert_eq!(avr.read_data(TCNT0), 0);
    assert_eq!(avr.cpu.data[TIFR0] & TOV0, TOV0); // TOV set
}

#[test]
fn sets_tov_when_top_is_max_in_ctc_mode() {
    // avr8js issue #75.
    tov_once_when_top_is_max_in_ctc_mode();
}

#[test]
fn does_not_set_tov_twice_on_overflow() {
    // avr8js issue #80.
    tov_once_when_top_is_max_in_ctc_mode();
}

#[test]
fn sets_ocf0b_when_the_timer_equals_ocrb() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0x10);
    avr.write_data(OCR0B, 0x11);
    avr.write_data(TCCR0A, 0x0); // WGM: (Normal)
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 2);
    assert_eq!(avr.cpu.data[TIFR0], OCF0B);
    assert_eq!(avr.cpu.pc, 0);
    assert_eq!(avr.cpu.cycles, 2);
}

#[test]
fn takes_compare_a_when_tcnt0_equals_ocr0a() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0x20);
    avr.write_data(OCR0A, 0x21);
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    avr.write_data(TIMSK0, OCIE0A);
    avr.write_data(95, 0x80); // SREG: I-------
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 2);
    assert_eq!(avr.read_data(TCNT0), 0x23); // TCNT should be 0x23 (one tick above + 2 cycles for interrupt)
    assert_eq!(avr.cpu.data[TIFR0] & OCF0A, 0);
    assert_eq!(avr.cpu.pc, 0x1c);
    assert_eq!(avr.cpu.cycles, 4);
}

#[test]
fn no_compare_a_when_ocie0a_is_disabled() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0x20);
    avr.write_data(OCR0A, 0x21);
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    avr.write_data(TIMSK0, 0);
    avr.write_data(95, 0x80); // SREG: I-------
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 2);
    assert_eq!(avr.read_data(TCNT0), 0x21);
    assert_eq!(avr.cpu.pc, 0);
    assert_eq!(avr.cpu.cycles, 2);
}

#[test]
fn takes_compare_b_when_tcnt0_equals_ocr0b() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0x20);
    avr.write_data(OCR0B, 0x21);
    avr.write_data(TCCR0B, CS00); // Set prescaler to 1
    avr.write_data(TIMSK0, OCIE0B);
    avr.write_data(95, 0x80); // SREG: I-------
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 2);
    assert_eq!(avr.read_data(TCNT0), 0x23); // TCNT should be 0x23 (0x23 + 2 cycles for interrupt)
    assert_eq!(avr.cpu.data[TIFR0] & OCF0B, 0);
    assert_eq!(avr.cpu.pc, 0x1e);
    assert_eq!(avr.cpu.cycles, 4);
}

#[test]
fn does_not_count_on_the_cycle_tcnt_is_written() {
    // avr8js issue #36. At the end of this short program, R17 should contain
    // 0x31. Verified against a physical ATmega328p.
    let mut avr = avr_with(TIMER_ISSUE_36, &[], &[TIMER0_CONFIG]);
    run_instructions(&mut avr, TIMER_ISSUE_36_INSTRUCTIONS);
    assert_eq!(avr.cpu.data[R17 as usize], 0x31);
}

#[test]
fn timer2_counts_every_256_ticks_when_prescaler_is_6() {
    // avr8js issue #5.
    let mut avr = timer(TIMER2_CONFIG);
    avr.write_data(TCCR2B, CS22 | CS21); // Set prescaler to 256
    tick_at(&mut avr, 1);

    tick_at(&mut avr, 1 + 511);
    assert_eq!(avr.read_data(TCNT2), 1);

    tick_at(&mut avr, 1 + 512);
    assert_eq!(avr.read_data(TCNT2), 2);
}

#[test]
fn updates_tcnt_as_a_two_cycle_instruction_reads_it() {
    // avr8js issue #40.
    let mut avr = avr_with(TIMER_ISSUE_40, &[], &[TIMER0_CONFIG]);
    run_instructions(&mut avr, TIMER_ISSUE_40_INSTRUCTIONS);
    assert_eq!(avr.cpu.data[R1 as usize], 2);
}

#[test]
fn does_not_count_before_the_prescaler_is_first_set() {
    // avr8js issue #41.
    let mut avr = avr_with(TIMER_ISSUE_41, &[], &[TIMER2_CONFIG]);
    run_instructions(&mut avr, TIMER_ISSUE_41_INSTRUCTIONS);
    assert_eq!(avr.read_data(R17), 2);
}

#[test]
fn stops_counting_on_the_instruction_that_disables_it() {
    // avr8js issue #72.
    let mut avr = avr_with(TIMER_ISSUE_72, &[], &[TIMER2_CONFIG]);
    run_instructions(&mut avr, TIMER_ISSUE_72_INSTRUCTIONS);
    assert_eq!(avr.read_data(R17), 2);
}

#[test]
fn foc0b_clears_oc0b() {
    let mut avr = timer_and_port(TIMER0_CONFIG, PORT_D_CONFIG);
    avr.write_data(TCCR0A, COM0B1);

    // Listen to Port B's internal callback
    overrides(&mut avr);

    avr.write_data(TCCR0B, FOC0B);

    assert!(overrides(&mut avr).contains(&(5, Clear)));
}

#[test]
fn fast_pwm_sets_oc0a_on_compare_match_and_clears_it_on_bottom() {
    // avr8js issue #78.
    let mut avr = timer0_program(FAST_PWM_SET_CLEAR);

    run_to_address(&mut avr, FAST_PWM_SET_CLEAR_BEFORE_MATCH);
    assert_eq!(avr.read_data(TCNT0), 0xfd);
    assert_eq!(overrides(&mut avr), [(6, Enable)]); // OC0A: Enable

    run_to_address(&mut avr, FAST_PWM_SET_CLEAR_AFTER_MATCH);
    assert_eq!(avr.read_data(TCNT0), 0xfe);
    assert_eq!(overrides(&mut avr), [(6, Set)]); // OC0A: Set

    run_to_address(&mut avr, FAST_PWM_SET_CLEAR_BEFORE_BOTTOM);
    assert_eq!(avr.read_data(TCNT0), 0xff);
    assert_eq!(overrides(&mut avr), []);

    run_to_address(&mut avr, FAST_PWM_SET_CLEAR_AFTER_BOTTOM);
    assert_eq!(avr.read_data(TCNT0), 0x0);
    assert_eq!(overrides(&mut avr), [(6, Clear)]); // OC0A: Clear
}

#[test]
fn fast_pwm_toggles_oc0a_on_compare_match_when_com0a_is_1() {
    // avr8js issue #78.
    let mut avr = timer0_program(FAST_PWM_TOGGLE);

    run_to_address(&mut avr, FAST_PWM_TOGGLE_BEFORE_MATCH);
    assert_eq!(avr.read_data(TCNT0), 0xfd);
    assert_eq!(overrides(&mut avr), [(6, Enable)]); // OC0A: Enable

    run_to_address(&mut avr, FAST_PWM_TOGGLE_AFTER_MATCH);
    assert_eq!(avr.read_data(TCNT0), 0xfe);
    assert_eq!(overrides(&mut avr), [(6, Toggle)]); // OC0A: Toggle

    run_to_address(&mut avr, FAST_PWM_TOGGLE_AFTER_OVERFLOW);
    assert_eq!(avr.read_data(TCNT0), 0);
    assert_eq!(overrides(&mut avr), []);
}

#[test]
fn fast_pwm_leaves_oc0a_disconnected_when_com0a_is_1_and_wgm02_is_0() {
    // avr8js issue #78.
    let mut avr = timer0_program(FAST_PWM_DISCONNECT);

    // First, run with the bit set and assert that the Pin Override was enabled (OC0A connected)
    run_to_address(&mut avr, FAST_PWM_DISCONNECT_BEFORE_CLEAR_WGM02);
    assert_eq!(overrides(&mut avr), [(6, Enable)]);

    // Now clear WGM02 and observe that Pin Override was disabled (OC0A disconnected)
    run_to_address(&mut avr, FAST_PWM_DISCONNECT_AFTER_CLEAR_WGM02);
    assert_eq!(overrides(&mut avr), [(6, PinOverrideMode::None)]);
}

#[test]
fn phase_correct_counts_up_to_top_down_to_0_then_sets_tov() {
    let mut avr = avr_with(PHASE_PWM_UP_DOWN, &[], &[TIMER0_CONFIG]);
    run_instructions(&mut avr, PHASE_PWM_UP_DOWN_INSTRUCTIONS);

    assert_eq!(avr.read_data(R17), 2);
    assert_eq!(avr.read_data(R18), 3);
    assert_eq!(avr.read_data(R19), 2);
    assert_eq!(avr.read_data(R20), 1);
    assert_eq!(avr.read_data(R21), 0);
    assert_eq!(avr.read_data(R22), 1);
    assert_eq!(avr.cpu.data[TIFR0] & TOV0, TOV0);
}

#[test]
fn phase_correct_clears_oc0a_on_match_counting_up() {
    let mut avr = timer0_program(PHASE_PWM_CLEAR_OC0A);

    let nop_count = 3;
    run_instructions(&mut avr, PHASE_PWM_CLEAR_OC0A_INSTRUCTIONS - nop_count);

    assert_eq!(avr.read_data(TCNT0), 0xfd);
    assert!(overrides(&mut avr).contains(&(6, Enable)));

    run_instructions(&mut avr, 1);
    assert_eq!(avr.read_data(TCNT0), 0xfe);
    assert!(overrides(&mut avr).contains(&(6, Clear)));

    run_instructions(&mut avr, 1);
    assert_eq!(avr.read_data(TCNT0), 0xff);
    assert_eq!(overrides(&mut avr), []);

    run_instructions(&mut avr, 1);
    assert_eq!(avr.read_data(TCNT0), 0xfe);
    assert!(overrides(&mut avr).contains(&(6, Set)));
}

#[test]
fn phase_correct_toggles_oc0a_on_match_when_com0a_is_1() {
    // avr8js issue #78.
    let mut avr = timer0_program(PHASE_PWM_TOGGLE_OC0A);

    run_to_address(&mut avr, PHASE_PWM_TOGGLE_OC0A_BEFORE_MATCH);
    assert_eq!(avr.read_data(TCNT0), 0xfd);
    assert!(overrides(&mut avr).contains(&(6, Enable)));

    run_to_address(&mut avr, PHASE_PWM_TOGGLE_OC0A_AFTER_MATCH);
    assert_eq!(avr.read_data(TCNT0), 0xfe);
    assert!(overrides(&mut avr).contains(&(6, Toggle)));
}

#[test]
fn phase_correct_leaves_oc0a_disconnected_with_com0a_1_in_wgm_1() {
    // avr8js issue #78.
    let mut avr = timer0_program(PHASE_PWM_MODE_1);
    run_instructions(&mut avr, PHASE_PWM_MODE_1_INSTRUCTIONS);

    // Assert that the pin callback wasn't called (thus it's disconnected)
    assert_eq!(overrides(&mut avr), []);
}

#[test]
fn does_not_miss_a_compare_match_during_a_multi_cycle_instruction() {
    // avr8js issue #79.
    let mut avr = timer0_program(TIMER_ISSUE_79);
    run_instructions(&mut avr, TIMER_ISSUE_79_INSTRUCTIONS);

    assert_eq!(avr.read_data(TCNT0), 0x11);
    let log = overrides(&mut avr);
    assert!(log.contains(&(6, Enable)));

    // Verify that Compare Match has occured and set the OC0A pin (PD6 on ATmega328p)
    assert!(log.contains(&(6, Set)));
}

#[test]
fn phase_correct_updates_ocr0a_only_at_top() {
    // avr8js issue #76.
    let mut avr = avr_with(PHASE_PWM_ISSUE_76, &[], &[TIMER0_CONFIG]);
    run_instructions(&mut avr, PHASE_PWM_ISSUE_76_INSTRUCTIONS);

    assert_eq!(avr.read_data(R17), 0x4);
    assert_eq!(avr.read_data(R18), 0x1);
}

#[test]
fn phase_correct_updates_ocr0a_when_top_is_0() {
    // avr8js issue #119.
    let mut avr = avr_with(PHASE_PWM_ISSUE_119_TOP_ZERO, &[], &[TIMER0_CONFIG]);
    run_instructions(&mut avr, PHASE_PWM_ISSUE_119_TOP_ZERO_INSTRUCTIONS);

    assert_eq!(avr.read_data(R17), 0);
    assert_eq!(avr.read_data(R18), 0);
    assert_eq!(avr.read_data(R19), 0x1);
}

#[test]
fn phase_correct_does_not_overrun_when_top_is_below_the_count() {
    // avr8js issue #119.
    let mut avr = avr_with(PHASE_PWM_ISSUE_119_OVERRUN, &[], &[TIMER0_CONFIG]);
    run_instructions(&mut avr, PHASE_PWM_ISSUE_119_OVERRUN_INSTRUCTIONS);

    assert_eq!(avr.read_data(R17), 255);
    assert_eq!(avr.io.timers[0].debug_tcnt(), 0); // TCNT should wrap
}

#[test]
fn timer1_increments_tcnt_by_1() {
    let mut avr = timer(TIMER1_CONFIG);
    avr.write_data(TCNT1H, 0x22); // TCNT1 <- 0x2233
    avr.write_data(TCNT1, 0x33); // ...
    let timer_low = avr.read_data(TCNT1);
    let timer_high = avr.read_data(TCNT1H);
    assert_eq!(((timer_high as u16) << 8) | timer_low as u16, 0x2233);
    avr.write_data(TCCR1A, 0x0); // WGM: Normal
    avr.write_data(TCCR1B, CS10); // Set prescaler to 1
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 2);
    avr.read_data(TCNT1);
    assert_eq!(avr.cpu.data16(TCNT1), 0x2234); // TCNT1 should increment
}

#[test]
fn timer1_sets_ocf1a_when_the_timer_equals_ocr1a() {
    let mut avr = timer(TIMER1_CONFIG);
    avr.write_data(TCNT1H, 0x10); // TCNT1 <- 0x10ee
    avr.write_data(TCNT1, 0xee); // ...
    avr.write_data(OCR1AH, 0x10); // OCR1 <- 0x10ef
    avr.write_data(OCR1A, 0xef); // ...
    avr.write_data(TCCR1A, 0x0); // WGM: Normal
    avr.write_data(TCCR1B, CS10); // Set prescaler to 1
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 2);
    assert_eq!(avr.cpu.data[TIFR1], OCF1A); // TIFR1 should have OCF1A bit on
    assert_eq!(avr.cpu.pc, 0);
    assert_eq!(avr.cpu.cycles, 2);
}

#[test]
fn timer1_sets_ocf1c_when_the_timer_equals_ocr1c() {
    let mut avr = timer(AvrTimerConfig {
        ocrc: OCR1C,
        ocfc: OCF1C,
        ..TIMER1_CONFIG
    });
    avr.write_data(TCNT1H, 0);
    avr.write_data(TCNT1, 0x10);
    avr.write_data(OCR1C, 0x11);
    avr.write_data(OCR1CH, 0x11);
    avr.write_data(TCCR1A, 0x0); // WGM: (Normal)
    avr.write_data(TCCR1B, CS00); // Set prescaler to 1
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 2);
    assert_eq!(avr.cpu.data[TIFR1], OCF1C);
    assert_eq!(avr.cpu.pc, 0);
    assert_eq!(avr.cpu.cycles, 2);
}

#[test]
fn timer1_takes_the_overflow_interrupt_when_enabled() {
    let mut avr = timer(TIMER1_CONFIG);
    avr.write_data(TCCR1A, 0x3); // TCCR1A <- WGM10 | WGM11 (Fast PWM, 10-bit)
    avr.write_data(TCCR1B, 0x9); // TCCR1B <- WGM12 | CS10
    avr.write_data(TIMSK1, 0x1); // TIMSK1: TOIE1
    avr.cpu.data[SREG] = 0x80; // SREG: I-------
    avr.write_data(TCNT1H, 0x3); // TCNT1 <- 0x3ff
    tick_at(&mut avr, 1);
    avr.write_data(TCNT1, 0xff); // ...
    tick_at(&mut avr, 2); // This cycle shouldn't be counted
    tick_at(&mut avr, 3); // This is where we cause the overflow
    avr.read_data(TCNT1); // Refresh TCNT1
    assert_eq!(avr.cpu.data16(TCNT1), 2);
    assert_eq!(avr.cpu.data[TIFR1] & TOV1, 0);
    assert_eq!(avr.cpu.pc, 0x1a);
    assert_eq!(avr.cpu.cycles, 5);
}

#[test]
fn timer1_resets_at_icr1_in_mode_12() {
    let mut avr = timer(TIMER1_CONFIG);
    avr.write_data(TCNT1H, 0x50); // TCNT1 <- 0x500f
    avr.write_data(TCNT1, 0x0f); // ...
    avr.write_data(ICR1H, 0x50); // ICR1 <- 0x5010
    avr.write_data(ICR1, 0x10); // ...
    avr.write_data(TCCR1B, WGM13 | WGM12 | CS10); // Set prescaler to 1, WGM: CTC
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 3); // 2 cycles should increment timer twice, beyond ICR1
    avr.read_data(TCNT1); // Refresh TCNT1
    assert_eq!(avr.cpu.data16(TCNT1), 0); // TCNT should be 0
    assert_eq!(avr.cpu.data[TIFR1] & TOV1, 0);
    assert_eq!(avr.cpu.cycles, 3);
}

#[test]
fn timer1_does_not_take_the_high_byte_written_after_the_low_byte() {
    // avr8js issue #37.
    let mut avr = timer(TIMER1_CONFIG);
    avr.write_data(TCNT1, 0x22);
    avr.write_data(TCNT1H, 0x55);
    tick_at(&mut avr, 1);
    let timer_low = avr.read_data(TCNT1);
    let timer_high = avr.read_data(TCNT1H);
    assert_eq!(((timer_high as u16) << 8) | timer_low as u16, 0x22);
}

#[test]
fn timer1_high_byte_read_before_the_low_byte_is_the_old_one() {
    // avr8js issue #37.
    let mut avr = timer(TIMER1_CONFIG);
    avr.write_data(TCNT1H, 0xff);
    avr.write_data(TCNT1, 0xff);
    avr.write_data(TCCR1B, WGM12 | CS10); // Set prescaler to 1, WGM: CTC
    tick_at(&mut avr, 1);
    tick_at(&mut avr, 2);
    // We read the high byte before the low byte, so the high byte should still have
    // the previous value:
    let timer_high = avr.read_data(TCNT1H);
    let timer_low = avr.read_data(TCNT1);
    assert_eq!(((timer_high as u16) << 8) | timer_low as u16, 0xff00);
}

#[test]
fn timer1_toggles_oc1b_on_compare_match() {
    let mut avr = avr_with(TIMER1_TOGGLE_OC1B, &[PORT_B_CONFIG], &[TIMER1_CONFIG]);

    let nop_count = 2;
    run_instructions(&mut avr, TIMER1_TOGGLE_OC1B_INSTRUCTIONS - nop_count);

    assert_eq!(avr.read_data(TCNT1), 0x49);
    assert!(overrides(&mut avr).contains(&(2, Enable)));

    run_instructions(&mut avr, 1);
    assert_eq!(avr.read_data(TCNT1), 0x4a);
    assert!(overrides(&mut avr).contains(&(2, Toggle)));
}

#[test]
fn timer1_toggles_oc1c_on_compare_match() {
    let mut avr = avr_with(TIMER1_TOGGLE_OC1C, &[PORT_B_CONFIG], &[TIMER1_WITH_OCRC]);

    let nop_count = 2;
    run_instructions(&mut avr, TIMER1_TOGGLE_OC1C_INSTRUCTIONS - nop_count);

    assert_eq!(avr.read_data(TCNT1), 0x49);
    assert!(overrides(&mut avr).contains(&(3, Enable)));

    run_instructions(&mut avr, 1);
    assert_eq!(avr.read_data(TCNT1), 0x4a);
    assert!(overrides(&mut avr).contains(&(3, Toggle)));
}

#[test]
fn timer1_foc1c_toggles_oc1c() {
    let mut avr = timer_and_port(TIMER1_WITH_OCRC, PORT_B_CONFIG);
    avr.write_data(TCCR1A, COM1C0);

    // Listen to Port B's internal callback
    overrides(&mut avr);

    avr.write_data(TCCR1C, FOC1C);

    assert!(overrides(&mut avr).contains(&(3, Toggle)));
}

#[test]
fn timer1_foc1c_does_nothing_in_pwm_mode() {
    let mut avr = timer_and_port(TIMER1_WITH_OCRC, PORT_B_CONFIG);
    avr.write_data(TCCR1A, COM1C0 | WGM11);

    // Listen to Port B's internal callback
    overrides(&mut avr);

    avr.write_data(TCCR1C, FOC1C);

    assert_eq!(overrides(&mut avr), []);
}

#[test]
fn timer1_updates_ocr1a_only_at_bottom_in_phase_and_frequency_correct_mode() {
    // avr8js issue #76.
    let mut avr = avr_with(TIMER1_ISSUE_76, &[], &[TIMER1_CONFIG]);
    run_instructions(&mut avr, TIMER1_ISSUE_76_INSTRUCTIONS);

    assert_eq!(avr.read_data(R17), 0x4);
    assert_eq!(avr.read_data(R18), 0x2);
    assert_eq!(avr.read_data(R19), 0x6);
    assert_eq!(avr.read_data(R20), 0x7);
}

#[test]
fn timer1_updates_ocr1a_when_tcnt_is_set_to_0() {
    // avr8js issue #111.
    let mut avr = avr_with(TIMER1_ISSUE_111, &[], &[TIMER1_CONFIG]);
    run_instructions(&mut avr, TIMER1_ISSUE_111_INSTRUCTIONS);

    assert_eq!(avr.read_data(R17), 0x1);
    assert_eq!(avr.read_data(R18), 0x3);
    assert_eq!(avr.read_data(R19), 0x5);
    assert_eq!(avr.read_data(R20), 0x3);
}

#[test]
fn timer1_masks_the_unused_bits_of_ocr1a_with_a_fixed_top() {
    let mut avr = timer(TIMER1_CONFIG);
    avr.write_data(TCCR1A, WGM10 | WGM11); // WGM: FastPWM, top 0x3ff
    avr.write_data(TCCR1B, WGM12);
    avr.write_data(OCR1AH, 0xff);
    avr.write_data(OCR1A, 0xff);
    assert_eq!(avr.read_data(OCR1A), 0xff);
    assert_eq!(avr.read_data(OCR1AH), 0x03);
}

#[test]
fn counts_on_the_falling_edge_of_t0_when_cs_is_110() {
    let mut avr = timer_and_port(TIMER0_CONFIG, PORT_D_CONFIG);
    avr.write_data(TCCR0B, CS02 | CS01); // Count on falling edge
    tick_at(&mut avr, 1);

    avr.io.set_pin(&mut avr.cpu, 0, T0, true); // Rising edge
    tick_at(&mut avr, 2);
    assert_eq!(avr.read_data(TCNT0), 0);

    avr.io.set_pin(&mut avr.cpu, 0, T0, false); // Falling edge
    tick_at(&mut avr, 3);
    assert_eq!(avr.read_data(TCNT0), 1);
}

#[test]
fn counts_on_the_rising_edge_of_t0_when_cs_is_111() {
    let mut avr = timer_and_port(TIMER0_CONFIG, PORT_D_CONFIG);
    avr.write_data(TCCR0B, CS02 | CS01 | CS00); // Count on rising edge
    tick_at(&mut avr, 1);

    avr.io.set_pin(&mut avr.cpu, 0, T0, true); // Rising edge
    tick_at(&mut avr, 2);
    assert_eq!(avr.read_data(TCNT0), 1);

    avr.io.set_pin(&mut avr.cpu, 0, T0, false); // Falling edge
    tick_at(&mut avr, 3);
    assert_eq!(avr.read_data(TCNT0), 1);
}

// ---- Not in avr8js's spec.

#[test]
fn timer0_compare_output_drives_pd6_in_fast_pwm() {
    // analogWrite(6, 64) as the Arduino core sets it up: fast PWM, clear on
    // match, set at bottom, prescaler 64, 256 counts a period. avr8js keeps
    // the pin high for OCR0A counts of them, a quarter (the datasheet says
    // OCR0A + 1); an LED on it would see the pulses.
    const PERIOD: u64 = 64 * 256;
    let mut avr = Avr::atmega328p(vec![0; 16]);
    avr.write_data(0x2a, 1 << 6); // DDRD: PD6 output
    avr.write_data(OCR0A, 64);
    avr.write_data(TCCR0A, 0x83); // COM0A1 | WGM01 | WGM00
    avr.write_data(TCCR0B, CS01 | CS00);
    let port_d = crate::avr::PORT_D;
    // The first period starts low: the pin is only set at the first bottom.
    while avr.cpu.cycles < PERIOD + 64 {
        avr.step();
    }
    let (mut high, mut edges) = (0, 0);
    let start = avr.cpu.cycles;
    while avr.cpu.cycles < start + 4 * PERIOD {
        let before = avr.io.ports[port_d].pin_state(&avr.cpu, 6);
        avr.step();
        let after = avr.io.ports[port_d].pin_state(&avr.cpu, 6);
        edges += (before != after) as u32;
        high += (after == crate::avr::gpio::PinState::High) as u64;
    }
    assert_eq!(edges, 8, "two edges a period");
    let duty = high as f64 / (4 * PERIOD) as f64;
    assert!((duty - 0.25).abs() < 0.001, "duty {duty}");
}

#[test]
fn timer0_oc0b_clocks_timer1_through_t1() {
    // OC0B is PD5, and so is T1: a timer driving a pin that clocks another,
    // which avr8js does by calling one from inside the other and this port
    // does by queueing the edge.
    let mut avr = Avr::atmega328p(vec![0; 16]);
    avr.write_data(0x2a, 1 << 5); // DDRD: PD5 output
    avr.write_data(TCCR1B, CS02 | CS01 | CS00); // timer1 on T1's rising edge
    avr.step();
    avr.write_data(TCCR0A, 0x12); // COM0B0 | WGM01: CTC, toggle OC0B
    avr.write_data(OCR0A, 9);
    avr.write_data(TCCR0B, CS00);
    for _ in 0..200 {
        avr.step();
    }
    // Ten counts a toggle, two toggles a rising edge: about 200 / 20.
    let count = avr.read_data(TCNT1);
    assert!((9..=11).contains(&count), "timer1 counted {count}");
}

#[test]
fn reset_counts_up_from_zero() {
    let mut avr = timer(TIMER0_CONFIG);
    avr.write_data(TCNT0, 0x10);
    avr.io.timers[0].reset();
    // Phase correct, TOP 0xff: up from 0 after a reset.
    avr.write_data(TCCR0A, WGM00);
    avr.write_data(TCCR0B, CS00);
    for cycles in 1..=4 {
        tick_at(&mut avr, cycles);
    }
    assert_eq!(avr.read_data(TCNT0), 3);
}
