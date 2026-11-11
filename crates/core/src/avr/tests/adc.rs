// SPDX-License-Identifier: MIT OR Apache-2.0
// Ported from avr8js, src/peripherals/adc.spec.ts.
// Copyright (c) 2019-2025 Uri Shaked, MIT licence (see avr/NOTICE.md).

//! adc.spec.ts, both cases; where the spec spies on `onADCRead`, this reads
//! whether a conversion is running. Then the cases this port adds: ADIF
//! cleared by writing it 1, and the internal reference.

use super::programs::*;
use super::{avr_with, run_instructions};
use crate::avr::Avr;
use crate::avr::adc::{ADC_CONFIG, AvrAdc};

const R16: usize = 16;
const R17: usize = 17;

const ADMUX: u16 = 0x7c;
const ADCSRA: u16 = 0x7a;
const ADCL: u16 = 0x78;
const ADCH: u16 = 0x79;

const REFS0: u8 = 1 << 6;
const REFS1: u8 = 1 << 7;
const ADEN: u8 = 1 << 7;
const ADSC: u8 = 1 << 6;
const ADIF: u8 = 1 << 4;
const ADIE: u8 = 1 << 3;
const ADPS: u8 = 7;

/// `new CPU(program)` and `new AVRADC(cpu, adcConfig)`.
fn adc_program(program: &[u16]) -> Avr {
    let mut avr = avr_with(program, &[], &[]);
    avr.io.adc = Some(AvrAdc::new(&mut avr.cpu, ADC_CONFIG));
    avr
}

fn adc(avr: &mut Avr) -> &mut AvrAdc {
    avr.io.adc.as_mut().unwrap()
}

/// Skips to the end of a conversion at the 128 prescaler, as the spec does.
fn skip_conversion(avr: &mut Avr) {
    avr.cpu.cycles += 128 * 25;
    avr.tick();
}

fn result(avr: &Avr) -> u16 {
    (avr.cpu.data[R17] as u16) << 8 | avr.cpu.data[R16] as u16
}

#[test]
fn should_successfuly_perform_an_adc_conversion() {
    let mut avr = adc_program(ADC_CONVERSION);
    adc(&mut avr).channel_values[0] = 2.56; // should result in 2.56/5*1024 = 524

    // Setup
    run_instructions(&mut avr, 4);
    assert_eq!(avr.cpu.data[ADCSRA as usize] & ADSC, ADSC);

    // Run the "waitComplete" loop for a few cycles
    run_instructions(&mut avr, 12);

    skip_conversion(&mut avr);

    // Now read the result
    run_instructions(&mut avr, 5);
    assert_eq!(result(&avr), 524); // 2.56 volts - see above
}

#[test]
fn should_read_0_when_the_adc_peripheral_is_not_enabled() {
    let mut avr = adc_program(ADC_NOT_ENABLED);
    adc(&mut avr).channel_values[0] = 2.56;

    // Setup
    run_instructions(&mut avr, 6);

    // Run the "waitComplete" loop for a few cycles
    run_instructions(&mut avr, 12);

    skip_conversion(&mut avr);

    // Now read the result, up to the break.
    run_instructions(&mut avr, 5);
    assert_eq!(avr.cpu.prog_word(avr.cpu.pc), 0x9598);
    assert_eq!(result(&avr), 0); // We should read 0 since the ADC hasn't been enabled
}

// ---- not in avr8js's spec

#[test]
fn writing_adif_as_1_clears_it_and_its_interrupt() {
    let mut avr = adc_program(&[0; 0x100]);
    adc(&mut avr).channel_values[0] = 1.0;
    avr.write_data(ADMUX, REFS0);
    avr.write_data(ADCSRA, ADEN | ADSC | ADPS);
    skip_conversion(&mut avr);
    assert_eq!(avr.cpu.data[ADCSRA as usize] & ADIF, ADIF);

    // Written back as read, with ADIE now on: the flag goes, and no
    // interrupt is left pending.
    let read = avr.read_data(ADCSRA);
    avr.write_data(ADCSRA, read | ADIE);
    assert_eq!(avr.cpu.data[ADCSRA as usize] & ADIF, 0);
    assert_eq!(avr.cpu.next_interrupt, -1);

    // Written 0, it stays.
    avr.write_data(ADCSRA, ADEN | ADSC | ADPS);
    avr.cpu.cycles += 128 * 13;
    avr.tick();
    avr.write_data(ADCSRA, ADEN | ADPS);
    assert_eq!(avr.cpu.data[ADCSRA as usize] & ADIF, ADIF);
}

#[test]
fn reads_against_the_internal_1v1_and_left_adjusted() {
    let mut avr = adc_program(&[0; 0x100]);
    adc(&mut avr).channel_values[3] = 0.55;
    // REFS1 | REFS0: the internal 1.1 V; ADLAR; channel 3.
    avr.write_data(ADMUX, REFS1 | REFS0 | 0x20 | 3);
    avr.write_data(ADCSRA, ADEN | ADSC | ADPS);
    skip_conversion(&mut avr);
    let value = (avr.read_data(ADCH) as u16) << 2 | (avr.read_data(ADCL) as u16) >> 6;
    assert_eq!(value, 512);
    assert_eq!(
        avr.io.adc.as_ref().unwrap().selected_channel(&avr.cpu),
        Some(3)
    );
}

#[test]
fn a_voltage_past_the_reference_reads_full_scale_and_the_bandgap_1v1() {
    let mut avr = adc_program(&[0; 0x100]);
    adc(&mut avr).channel_values[0] = 7.0;
    avr.write_data(ADMUX, REFS0);
    avr.write_data(ADCSRA, ADEN | ADSC | ADPS);
    skip_conversion(&mut avr);
    assert_eq!(
        (avr.read_data(ADCH) as u16) << 8 | avr.read_data(ADCL) as u16,
        1023
    );
    // Mux 14 is the bandgap, against AVCC: 1.1 / 5 × 1024.
    avr.write_data(ADMUX, REFS0 | 14);
    avr.write_data(ADCSRA, ADEN | ADSC | ADPS);
    avr.cpu.cycles += 128 * 13;
    avr.tick();
    assert_eq!(
        (avr.read_data(ADCH) as u16) << 8 | avr.read_data(ADCL) as u16,
        225
    );
    assert_eq!(
        avr.io.adc.as_ref().unwrap().selected_channel(&avr.cpu),
        None
    );
}
