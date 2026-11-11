// SPDX-License-Identifier: MIT OR Apache-2.0
// Ported from avr8js, src/peripherals/gpio.spec.ts.
// Copyright (c) 2019-2025 Uri Shaked, MIT licence (see avr/NOTICE.md).

//! gpio.spec.ts. Where the spec checks what a listener was called with, this
//! checks what [`AvrIoPort::take_change`] reports, taking the change first
//! wherever the spec adds its listener late. `removeListener`'s case has no
//! counterpart: there is nothing to remove.
//!
//! [`AvrIoPort::take_change`]: crate::avr::gpio::AvrIoPort::take_change

use super::programs::*;
use super::{avr_with, blank_avr, run_instructions};
use crate::avr::gpio::{PORT_B_CONFIG, PORT_C_CONFIG, PORT_D_CONFIG, PinOverrideMode, PinState};
use crate::avr::{Avr, PORT_B, PORT_C, PORT_D};

// CPU registers
const SREG: usize = 95;

// GPIO registers
const PINB: u16 = 0x23;
const DDRB: u16 = 0x24;
const PORTB: u16 = 0x25;
const PORTD: u16 = 0x2b;
const EIFR: u16 = 0x3c;
const EIMSK: u16 = 0x3d;
const PCICR: u16 = 0x68;
const EICRA: u16 = 0x69;
const PCIFR: u16 = 0x3b;
const PCMSK0: u16 = 0x6b;
const PCMSK1: u16 = 0x6c;

// Register bit names
const INT0: u8 = 0;
const INT1: u8 = 1;
const ISC00: u8 = 0;
const ISC01: u8 = 1;
const ISC11: u8 = 3;
const PCIE0: u8 = 0;
const PCIE1: u8 = 1;
const PCINT3: u8 = 3;

// Pin names
const PB0: u8 = 0;
const PB1: u8 = 1;
const PB3: u8 = 3;
const PB4: u8 = 4;
const PC2: u8 = 2;
const PD2: u8 = 2;
const PD3: u8 = 3;

// Interrupt vector addresses
const PC_INT_INT0: u32 = 2;
const PC_INT_INT1: u32 = 4;
const PC_INT_PCINT0: u32 = 6;
const PC_INT_PCINT1: u32 = 8;

/// `new CPU(new Uint16Array(1024))` and `new AVRIOPort(cpu, portBConfig)`:
/// the port is index 0.
fn port_b() -> Avr {
    blank_avr(1024, &[PORT_B_CONFIG], &[])
}

/// The same with port D.
fn port_d() -> Avr {
    blank_avr(1024, &[PORT_D_CONFIG], &[])
}

fn set_pin(avr: &mut Avr, pin: u8, value: bool) {
    avr.io.set_pin(&mut avr.cpu, 0, pin, value);
}

fn pin_state(avr: &Avr, pin: u8) -> PinState {
    avr.io.ports[0].pin_state(&avr.cpu, pin)
}

fn take_change(avr: &mut Avr) -> Option<(u8, u8)> {
    avr.io.ports[0].take_change()
}

#[test]
fn reports_a_change_when_the_port_is_written_to() {
    let mut avr = port_b();
    avr.write_data(DDRB, 0x0f);
    take_change(&mut avr);
    avr.write_data(PORTB, 0x55);
    assert_eq!(take_change(&mut avr), Some((0x55, 0)));
    assert_eq!(avr.cpu.data[0x23], 0x5); // PINB should return port value
}

#[test]
fn reports_a_change_when_ddr_changes() {
    // avr8js issue #28.
    let mut avr = port_b();
    avr.write_data(PORTB, 0x55);
    take_change(&mut avr);
    avr.write_data(DDRB, 0xf0);
    assert_eq!(take_change(&mut avr), Some((0x55, 0x55)));
}

#[test]
fn reports_a_change_when_a_pull_up_is_enabled() {
    // avr8js issue #62.
    let mut avr = port_b();
    avr.write_data(PORTB, 0x55);
    assert_eq!(take_change(&mut avr), Some((0x55, 0)));
}

#[test]
fn toggles_the_pin_when_writing_to_the_pin_register() {
    let mut avr = port_b();
    avr.write_data(DDRB, 0x0f);
    avr.write_data(PORTB, 0x55);
    take_change(&mut avr);
    avr.write_data(PINB, 0x01);
    assert_eq!(take_change(&mut avr), Some((0x54, 0x55)));
    assert_eq!(avr.cpu.data[PINB as usize], 0x4); // PINB should return port value
}

#[test]
fn only_affects_one_pin_when_writing_to_pin_using_sbi() {
    // avr8js issue #103.
    let mut avr = avr_with(GPIO_SBI_ON_PIN, &[PORT_D_CONFIG], &[]);

    // Setup: pins 6, 3 are output, set to HIGH
    run_instructions(&mut avr, 3);
    assert_eq!(take_change(&mut avr), Some((0x48, 0x0)));
    assert_eq!(avr.cpu.data[PORTD as usize], 0x48);

    // Now we toggle pin 6
    run_instructions(&mut avr, 1);
    assert_eq!(take_change(&mut avr), Some((0x08, 0x48)));
    assert_eq!(avr.cpu.data[PORTD as usize], 0x8);
}

#[test]
fn updates_the_pin_register_on_output_compare_match() {
    // avr8js issue #102.
    let mut avr = port_b();
    avr.write_data(DDRB, 1 << 1);
    avr.io
        .timer_override_pin(&mut avr.cpu, 0, 1, PinOverrideMode::Set);
    assert_eq!(pin_state(&avr, 1), PinState::High);
    assert_eq!(avr.cpu.data[PINB as usize], 1 << 1);
    avr.io
        .timer_override_pin(&mut avr.cpu, 0, 1, PinOverrideMode::Clear);
    assert_eq!(pin_state(&avr, 1), PinState::Low);
    assert_eq!(avr.cpu.data[PINB as usize], 0);
}

#[test]
fn reports_nothing_when_nothing_changed() {
    // In place of removeListener's case.
    let mut avr = port_b();
    avr.write_data(DDRB, 0x0f);
    assert!(avr.io.outputs_changed());
    take_change(&mut avr);
    assert!(!avr.io.outputs_changed());
    avr.write_data(DDRB, 0x0f);
    assert_eq!(take_change(&mut avr), None);
}

#[test]
fn pin_state_is_high_when_set_to_output_and_high() {
    let mut avr = port_b();
    avr.write_data(DDRB, 0x1);
    avr.write_data(PORTB, 0x1);
    assert_eq!(pin_state(&avr, PB0), PinState::High);
}

#[test]
fn pin_state_is_low_when_set_to_output_and_low() {
    let mut avr = port_b();
    avr.write_data(DDRB, 0x8);
    avr.write_data(PORTB, 0xf7);
    assert_eq!(pin_state(&avr, PB3), PinState::Low);
}

#[test]
fn pin_state_is_input_by_default() {
    let avr = port_b();
    assert_eq!(pin_state(&avr, PB1), PinState::Input);
}

#[test]
fn pin_state_is_input_pull_up_when_set_to_input_with_pull_up() {
    let mut avr = port_b();
    avr.write_data(DDRB, 0);
    avr.write_data(PORTB, 0x2);
    assert_eq!(pin_state(&avr, PB1), PinState::InputPullUp);
}

#[test]
fn pin_state_is_current_as_soon_as_the_change_is_flagged() {
    // avr8js issue #9: the state a listener sees.
    let mut avr = port_b();
    assert_eq!(pin_state(&avr, PB0), PinState::Input);
    avr.write_data(DDRB, 0x01);
    take_change(&mut avr);
    avr.write_data(PORTB, 0x01);
    assert!(avr.io.outputs_changed());
    assert_eq!(pin_state(&avr, PB0), PinState::High);
}

#[test]
fn pin_state_is_current_as_soon_as_a_ddr_change_is_flagged() {
    // avr8js issue #47.
    let mut avr = port_b();
    assert_eq!(pin_state(&avr, PB0), PinState::Input);
    avr.write_data(DDRB, 0x01);
    assert!(avr.io.outputs_changed());
    assert_eq!(pin_state(&avr, PB0), PinState::Low);
}

#[test]
fn an_open_collector_pin_floats_when_driven_high() {
    // Not in avr8js's spec.
    let mut avr = port_b();
    avr.io.ports[0].open_collector = 1 << PB0;
    avr.write_data(DDRB, 0x01);
    avr.write_data(PORTB, 0x01);
    assert_eq!(pin_state(&avr, PB0), PinState::InputPullUp);
    avr.write_data(PORTB, 0x00);
    assert_eq!(pin_state(&avr, PB0), PinState::Low);
}

#[test]
fn set_pin_sets_the_value_of_the_given_pin() {
    let mut avr = port_b();
    avr.write_data(DDRB, 0);
    set_pin(&mut avr, PB4, true);
    assert_eq!(avr.cpu.data[0x23], 0x10);
    set_pin(&mut avr, PB4, false);
    assert_eq!(avr.cpu.data[0x23], 0x0);
}

#[test]
fn set_pin_only_updates_pin_when_the_pin_is_an_input() {
    let mut avr = port_b();
    avr.write_data(DDRB, 0x10);
    avr.write_data(PORTB, 0x0);
    set_pin(&mut avr, PB4, true);
    assert_eq!(avr.cpu.data[PINB as usize], 0x0);
    avr.write_data(DDRB, 0x0);
    assert_eq!(avr.cpu.data[PINB as usize], 0x10);
}

#[test]
fn int0_on_rising_edge() {
    let mut avr = port_d();
    avr.write_data(EIMSK, 1 << INT0);
    avr.write_data(EICRA, (1 << ISC01) | (1 << ISC00));

    assert_eq!(avr.cpu.data[EIFR as usize], 0);
    set_pin(&mut avr, PD2, true);
    assert_eq!(avr.cpu.data[EIFR as usize], 1 << INT0);

    avr.cpu.data[SREG] = 0x80; // SREG: I------- (enable interrupts)
    avr.tick();
    assert_eq!(avr.cpu.pc, PC_INT_INT0);
    assert_eq!(avr.cpu.cycles, 2);
    assert_eq!(avr.cpu.data[EIFR as usize], 0);

    set_pin(&mut avr, PD2, false);
    assert_eq!(avr.cpu.data[EIFR as usize], 0);
}

#[test]
fn int0_on_falling_edge() {
    let mut avr = port_d();
    avr.write_data(EIMSK, 1 << INT0);
    avr.write_data(EICRA, 1 << ISC01);

    assert_eq!(avr.cpu.data[EIFR as usize], 0);
    set_pin(&mut avr, PD2, true);
    assert_eq!(avr.cpu.data[EIFR as usize], 0);
    set_pin(&mut avr, PD2, false);
    assert_eq!(avr.cpu.data[EIFR as usize], 1 << INT0);

    avr.cpu.data[SREG] = 0x80; // SREG: I------- (enable interrupts)
    avr.tick();
    assert_eq!(avr.cpu.pc, PC_INT_INT0);
    assert_eq!(avr.cpu.cycles, 2);
    assert_eq!(avr.cpu.data[EIFR as usize], 0);
}

#[test]
fn int0_on_level_change() {
    let mut avr = port_d();
    avr.write_data(EIMSK, 1 << INT0);
    avr.write_data(EICRA, 1 << ISC00);

    assert_eq!(avr.cpu.data[EIFR as usize], 0);
    set_pin(&mut avr, PD2, true);
    assert_eq!(avr.cpu.data[EIFR as usize], 1 << INT0);
    avr.write_data(EIFR, 1 << INT0);
    assert_eq!(avr.cpu.data[EIFR as usize], 0);
    set_pin(&mut avr, PD2, false);
    assert_eq!(avr.cpu.data[EIFR as usize], 1 << INT0);
}

#[test]
fn int0_is_sticky_while_the_pin_is_low() {
    let mut avr = port_d();
    avr.write_data(EIMSK, 1 << INT0);
    avr.write_data(EICRA, 0);
    assert_eq!(avr.cpu.data[EIFR as usize], 0);

    set_pin(&mut avr, PD2, true);
    assert_eq!(avr.cpu.data[EIFR as usize], 0);

    set_pin(&mut avr, PD2, false);
    assert_eq!(avr.cpu.data[EIFR as usize], 1 << INT0);

    // This is a sticky interrupt, verify we can't clear the flag:
    avr.write_data(EIFR, 1 << INT0);
    assert_eq!(avr.cpu.data[EIFR as usize], 1 << INT0);

    avr.cpu.data[SREG] = 0x80; // SREG: I------- (enable interrupts)
    avr.tick();
    assert_eq!(avr.cpu.pc, PC_INT_INT0);
    assert_eq!(avr.cpu.cycles, 2);

    // Flag shouldn't be cleared, as the interrupt is sticky
    assert_eq!(avr.cpu.data[EIFR as usize], 1 << INT0);

    // But it will be cleared as soon as the pin goes high.
    set_pin(&mut avr, PD2, true);
    assert_eq!(avr.cpu.data[EIFR as usize], 0);
}

#[test]
fn int1_on_falling_edge() {
    // Not in avr8js's spec: INT1, on PD3, at its own vector.
    let mut avr = port_d();
    avr.write_data(EIMSK, 1 << INT1);
    avr.write_data(EICRA, 1 << ISC11);
    set_pin(&mut avr, PD3, true);
    set_pin(&mut avr, PD3, false);
    assert_eq!(avr.cpu.data[EIFR as usize], 1 << INT1);
    avr.cpu.data[SREG] = 0x80;
    avr.tick();
    assert_eq!(avr.cpu.pc, PC_INT_INT1);
}

#[test]
fn pcint_when_pb3_goes_high() {
    let mut avr = port_b();
    avr.write_data(PCICR, 1 << PCIE0);
    avr.write_data(PCMSK0, 1 << PCINT3);

    set_pin(&mut avr, PB3, true);
    assert_eq!(avr.cpu.data[PCIFR as usize], 1 << PCIE0);

    avr.cpu.data[SREG] = 0x80; // SREG: I-------
    avr.tick();
    assert_eq!(avr.cpu.pc, PC_INT_PCINT0);
    assert_eq!(avr.cpu.cycles, 2);
    assert_eq!(avr.cpu.data[PCIFR as usize], 0);
}

#[test]
fn pcint_when_pb3_goes_low() {
    let mut avr = port_b();

    set_pin(&mut avr, PB3, true);
    avr.write_data(PCICR, 1 << PCIE0);
    avr.write_data(PCMSK0, 1 << PCINT3);
    assert_eq!(avr.cpu.data[PCIFR as usize], 0);

    set_pin(&mut avr, PB3, false);
    assert_eq!(avr.cpu.data[PCIFR as usize], 1 << PCIE0);

    avr.cpu.data[SREG] = 0x80; // SREG: I-------
    avr.tick();
    assert_eq!(avr.cpu.pc, PC_INT_PCINT0);
    assert_eq!(avr.cpu.cycles, 2);
    assert_eq!(avr.cpu.data[PCIFR as usize], 0);
}

#[test]
fn pcint_flag_clears_when_writing_to_pcifr() {
    let mut avr = port_b();
    avr.write_data(PCICR, 1 << PCIE0);
    avr.write_data(PCMSK0, 1 << PCINT3);

    set_pin(&mut avr, PB3, true);
    assert_eq!(avr.cpu.data[PCIFR as usize], 1 << PCIE0);

    avr.write_data(PCIFR, 1 << PCIE0);
    assert_eq!(avr.cpu.data[PCIFR as usize], 0);
}

#[test]
fn pcint_flagged_before_pcicr_is_taken_once_it_is_enabled() {
    // Not in avr8js's spec, where PCICR has no hook and this interrupt is
    // never taken.
    let mut avr = port_b();
    avr.write_data(PCMSK0, 1 << PCINT3);
    set_pin(&mut avr, PB3, true);
    assert_eq!(avr.cpu.data[PCIFR as usize], 1 << PCIE0);
    avr.cpu.data[SREG] = 0x80;
    avr.tick();
    assert_eq!(avr.cpu.pc, 0, "not enabled yet");

    avr.write_data(PCICR, 1 << PCIE0);
    avr.tick();
    assert_eq!(avr.cpu.pc, PC_INT_PCINT0);
}

#[test]
fn writing_one_ports_pcmsk_keeps_anothers_pending_interrupt() {
    // Not in avr8js's spec, where writing PCMSK0 = 0x08 unqueues PCINT1
    // (its enable bit, 0x02, is not in 0x08).
    let mut avr = blank_avr(1024, &[PORT_B_CONFIG, PORT_C_CONFIG], &[]);
    avr.write_data(PCICR, (1 << PCIE0) | (1 << PCIE1));
    avr.write_data(PCMSK1, 1 << PC2);
    avr.io.set_pin(&mut avr.cpu, 1, PC2, true);
    avr.write_data(PCMSK0, 1 << PCINT3);
    avr.cpu.data[SREG] = 0x80;
    avr.tick();
    assert_eq!(avr.cpu.pc, PC_INT_PCINT1);
}

#[test]
fn run_until_stops_on_the_instruction_that_changed_an_output() {
    // Not in avr8js's spec: what the embedder relies on.
    let mut avr = Avr::atmega328p(GPIO_SBI_ON_PIN.to_vec());
    assert!(avr.run_until(1000), "out DDRD changes what the port drives");
    assert_eq!(avr.cpu.pc, 2);
    assert_eq!(avr.io.ports[PORT_D].take_change(), Some((0, 0)));
    assert!(avr.run_until(1000));
    assert_eq!(avr.cpu.pc, 3);
    assert_eq!(avr.io.ports[PORT_D].take_change(), Some((0x48, 0)));
    assert!(!avr.io.ports[PORT_B].changed && !avr.io.ports[PORT_C].changed);
    assert!(avr.run_until(1000), "sbi PIND toggles pin 6");
    assert_eq!((avr.cpu.pc, avr.cpu.cycles), (4, 5));
    // A change stays flagged until it is taken.
    assert_eq!(avr.io.ports[PORT_D].take_change(), Some((0x08, 0x48)));
    assert!(!avr.run_until(20), "nothing after it changes anything");
    assert_eq!(avr.cpu.cycles, 20);
}
