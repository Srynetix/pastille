// SPDX-License-Identifier: MIT OR Apache-2.0
// Ported from avr8js, src/peripherals/adc.ts.
// Copyright (c) 2019-2025 Uri Shaked, MIT licence (see NOTICE.md beside this file).

//! The ATmega328P's analog-to-digital converter: what `analogRead` asks
//! (A-04, ADR 0017).
//!
//! Ten bits, against AVCC, AREF or the internal 1.1 V, through a mux of eight
//! channels (A0–A5 on the Uno, A6 and A7 only on the surface-mount part), the
//! temperature sensor and two constants. A conversion is started by setting
//! ADSC; it takes 13 ADC clocks, 25 for the first after ADEN, each clock the
//! CPU's divided by ADPS — 128 under the Arduino core, so ~104 µs at 16 MHz.
//!
//! As in avr8js, the input is sampled when the conversion starts, from
//! [`AvrAdc::channel_values`], which whoever owns the chip keeps up to date;
//! the result is written and the flag raised `sample_cycles` later. The chip
//! samples 1.5 ADC clocks after the start, 12 µs under the Arduino core:
//! the difference is not one the circuit's steps can see.
//!
//! **Where this differs from avr8js 0.21.1**:
//!
//! - writing ADCSRA with ADIF set clears ADIF, as the sheet says (§24.9.2);
//!   upstream stores the 1, so the flag stays raised and, with ADIE on, an
//!   interrupt is queued that nothing asked for;
//! - `onADCRead` is not a callback to override: the sample is
//!   [`AvrAdc::sample`], and an embedder that wants another voltage sets
//!   `channel_values`;
//! - the differential inputs are left out, as the ATmega328P has none;
//! - ADMUX is hooked, only to note when the sample-and-hold moves to another
//!   pin ([`AvrAdc::selection_changed`]), since what it loads is the
//!   circuit's business.

use super::cpu::{ClockEvent, Cpu, Hook, InterruptConfig};

/// What a voltage is measured against: ADMUX's REFS bits, through the part's
/// table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdcReference {
    Avcc,
    Aref,
    Internal1V1,
    Internal2V56,
    Reserved,
}

/// What a mux setting connects the converter to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdcMuxInput {
    /// A pin, by channel.
    SingleEnded(u8),
    /// A fixed voltage: the bandgap, or GND.
    Constant(f64),
    /// The temperature sensor.
    Temperature,
}

/// The ATmega328P's mux (Table 24-4): channels 0–7, the sensor, 1.1 V and
/// 0 V; the rest are reserved, and read as 0 V.
pub fn atmega328_channel(mux: u8) -> AdcMuxInput {
    match mux {
        0..=7 => AdcMuxInput::SingleEnded(mux),
        8 => AdcMuxInput::Temperature,
        14 => AdcMuxInput::Constant(1.1),
        _ => AdcMuxInput::Constant(0.0),
    }
}

/// The ADC's registers, vector and tables.
#[derive(Debug, Clone, Copy)]
pub struct AvrAdcConfig {
    pub admux: u16,
    pub adcsra: u16,
    pub adcsrb: u16,
    pub adcl: u16,
    pub adch: u16,
    pub didr0: u16,
    pub adc_interrupt: u8,
    pub num_channels: usize,
    pub mux_input_mask: u8,
    pub mux_channels: fn(u8) -> AdcMuxInput,
    /// REFS1:0 → reference.
    pub adc_references: [AdcReference; 4],
}

pub const ADC_CONFIG: AvrAdcConfig = AvrAdcConfig {
    admux: 0x7c,
    adcsra: 0x7a,
    adcsrb: 0x7b,
    adcl: 0x78,
    adch: 0x79,
    didr0: 0x7e,
    adc_interrupt: 0x2a,
    num_channels: 8,
    mux_input_mask: 0xf,
    mux_channels: atmega328_channel,
    adc_references: [
        AdcReference::Aref,
        AdcReference::Avcc,
        AdcReference::Reserved,
        AdcReference::Internal1V1,
    ],
};

// Register bits
const ADPS_MASK: u8 = 0x7;
const ADIE: u8 = 0x8;
const ADIF: u8 = 0x10;
const ADSC: u8 = 0x40;
const ADEN: u8 = 0x80;
const MUX_MASK: u8 = 0x1f;
const ADLAR: u8 = 0x20;
const MUX5: u8 = 0x8;
const REFS_MASK: u8 = 0x3;
const REFS_SHIFT: u8 = 6;

/// What the temperature sensor reads at 25 °C, as avr8js has it.
const TEMPERATURE_VOLTS: f64 = 0.378125;

/// The converter.
#[derive(Debug, Clone)]
pub struct AvrAdc {
    pub config: AvrAdcConfig,
    /// Each channel's voltage, against the chip's GND. What the next
    /// conversion of that channel samples.
    pub channel_values: [f64; 8],
    /// The AVCC and AREF pins' voltages.
    pub avcc: f64,
    pub aref: f64,
    converting: bool,
    conversion_cycles: u64,
    /// The result of the conversion running, sampled when it started.
    result: u16,
    /// The mux moved the sample-and-hold to another pin, or the converter
    /// was turned on or off: what it loads changed. Cleared with the ports'
    /// changes.
    pub selection_changed: bool,
    interrupt: InterruptConfig,
}

impl AvrAdc {
    pub fn new(cpu: &mut Cpu, config: AvrAdcConfig) -> AvrAdc {
        cpu.write_hooks[config.adcsra as usize] = Hook::AdcControl;
        cpu.write_hooks[config.admux as usize] = Hook::AdcMux;
        AvrAdc {
            config,
            channel_values: [0.0; 8],
            avcc: 5.0,
            aref: 5.0,
            converting: false,
            conversion_cycles: 25,
            result: 0,
            selection_changed: false,
            interrupt: InterruptConfig {
                address: config.adc_interrupt,
                flag_register: config.adcsra,
                flag_mask: ADIF,
                enable_register: config.adcsra,
                enable_mask: ADIE,
                ..InterruptConfig::default()
            },
        }
    }

    /// A write to ADMUX: stored, and whether the sample-and-hold moved noted.
    pub fn write_mux(&mut self, cpu: &mut Cpu, value: u8) -> bool {
        let before = self.selected_channel(cpu);
        cpu.data[self.config.admux as usize] = value;
        if self.selected_channel(cpu) != before {
            self.selection_changed = true;
        }
        true
    }

    /// A write to ADCSRA. Always stores the value itself.
    pub fn write_control(&mut self, cpu: &mut Cpu, value: u8, old_value: u8) -> bool {
        let adcsra = self.config.adcsra;
        if (value ^ old_value) & ADEN != 0 {
            self.selection_changed = true;
        }
        if value & ADEN != 0 && old_value & ADEN == 0 {
            self.conversion_cycles = 25;
        }
        // ADIF is cleared by writing it 1, and kept by writing it 0.
        let stored = (value & !ADIF) | (old_value & ADIF & !value);
        cpu.data[adcsra as usize] = stored;
        if value & ADIF != 0 {
            cpu.clear_interrupt(&self.interrupt, true);
        }
        cpu.update_interrupt_enable(&self.interrupt, stored);
        if !self.converting && value & ADSC != 0 {
            if value & ADEN == 0 {
                // Reading while the ADC is not enabled returns 0, which
                // avr8js checked on real hardware.
                self.result = 0;
                cpu.add_clock_event(ClockEvent::AdcComplete, self.sample_cycles(cpu));
                return true;
            }
            self.converting = true;
            let input = (self.config.mux_channels)(self.mux(cpu));
            self.result = self.sample(cpu, input);
            cpu.add_clock_event(ClockEvent::AdcComplete, self.sample_cycles(cpu));
        }
        true
    }

    /// The mux setting, MUX5 included where a part has it.
    fn mux(&self, cpu: &Cpu) -> u8 {
        let mut channel = cpu.data[self.config.admux as usize] & MUX_MASK;
        if cpu.data[self.config.adcsrb as usize] & MUX5 != 0 {
            channel |= 0x20;
        }
        channel & self.config.mux_input_mask
    }

    /// The pin channel the sample-and-hold is on, if the converter is
    /// enabled and its mux is on one.
    pub fn selected_channel(&self, cpu: &Cpu) -> Option<u8> {
        if cpu.data[self.config.adcsra as usize] & ADEN == 0 {
            return None;
        }
        match (self.config.mux_channels)(self.mux(cpu)) {
            AdcMuxInput::SingleEnded(channel) => Some(channel),
            _ => None,
        }
    }

    /// What a conversion of `input` gives, now: avr8js's default `onADCRead`.
    pub fn sample(&self, cpu: &Cpu, input: AdcMuxInput) -> u16 {
        let voltage = match input {
            AdcMuxInput::Constant(volts) => volts,
            AdcMuxInput::SingleEnded(channel) => self
                .channel_values
                .get(channel as usize)
                .copied()
                .unwrap_or(0.0),
            AdcMuxInput::Temperature => TEMPERATURE_VOLTS,
        };
        let raw = voltage / self.reference_voltage(cpu) * 1024.0;
        raw.floor().clamp(0.0, 1023.0) as u16
    }

    /// The conversion's end: the result into ADCL/ADCH, ADSC cleared, ADIF
    /// raised.
    pub fn complete(&mut self, cpu: &mut Cpu) {
        let AvrAdcConfig {
            adcl,
            adch,
            admux,
            adcsra,
            ..
        } = self.config;
        let value = self.result;
        self.converting = false;
        self.conversion_cycles = 13;
        if cpu.data[admux as usize] & ADLAR != 0 {
            cpu.data[adcl as usize] = (value << 6) as u8;
            cpu.data[adch as usize] = (value >> 2) as u8;
        } else {
            cpu.data[adcl as usize] = value as u8;
            cpu.data[adch as usize] = ((value >> 8) & 0x3) as u8;
        }
        cpu.data[adcsra as usize] &= !ADSC;
        cpu.set_interrupt_flag(&self.interrupt);
    }

    pub fn prescaler(&self, cpu: &Cpu) -> u64 {
        match cpu.data[self.config.adcsra as usize] & ADPS_MASK {
            0 | 1 => 2,
            2 => 4,
            3 => 8,
            4 => 16,
            5 => 32,
            6 => 64,
            _ => 128,
        }
    }

    pub fn reference_voltage_type(&self, cpu: &Cpu) -> AdcReference {
        let refs = (cpu.data[self.config.admux as usize] >> REFS_SHIFT) & REFS_MASK;
        self.config.adc_references[refs as usize]
    }

    pub fn reference_voltage(&self, cpu: &Cpu) -> f64 {
        match self.reference_voltage_type(cpu) {
            AdcReference::Avcc => self.avcc,
            AdcReference::Aref => self.aref,
            AdcReference::Internal1V1 => 1.1,
            AdcReference::Internal2V56 => 2.56,
            AdcReference::Reserved => self.avcc,
        }
    }

    /// How long a conversion takes, in CPU cycles.
    pub fn sample_cycles(&self, cpu: &Cpu) -> u64 {
        self.conversion_cycles * self.prescaler(cpu)
    }
}
