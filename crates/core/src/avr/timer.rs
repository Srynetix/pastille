// SPDX-License-Identifier: MIT OR Apache-2.0
// Ported from avr8js, src/peripherals/timer.ts.
// Copyright (c) 2019-2025 Uri Shaked, MIT licence (see NOTICE.md beside this file).

//! The ATmega328P's timer/counters: timer0 and timer2, eight bits, and
//! timer1, sixteen (ADR 0017).
//!
//! What `millis()` counts with, what `analogWrite` makes its PWM with, and
//! what `tone()` toggles a pin with. A timer is a counter clocked by the CPU
//! through a prescaler (or by an edge on its T*n* pin); each waveform
//! generation mode decides where it turns round, when it raises its overflow
//! flag, when a new compare value takes effect, and what a compare match does
//! to the OC pins it can drive.
//!
//! The counter is not advanced every cycle. Like avr8js, it schedules itself
//! as a clock event at the next cycle the prescaler lets through, and catches
//! up on whatever it is behind when it runs — or when the program reads TCNT
//! in between, which is why reading it is a hook.
//!
//! **Where this differs from avr8js 0.21.1**:
//!
//! - writing TIFR clears the flags written as 1 and leaves the others; upstream
//!   first stores the written value, which clears every flag written as 0
//!   while leaving its interrupt pending;
//! - [`AvrTimer::reset`] leaves the counter counting up, as the hardware comes
//!   out of reset; upstream leaves it counting down.
//!
//! The ATtiny's timer, and the OCRC channel that only an ATmega2560's timers
//! have, are kept where avr8js's code has them but the ATmega328P never uses
//! them.

use super::cpu::{ClockEvent, Cpu, Hook, InterruptConfig};
use super::gpio::{AvrIoPort, PORT_B_CONFIG, PORT_D_CONFIG, PinOverrideMode};

/// Prescaler divisions by CS bits; 0 is stopped, and 6 and 7 of timer0 and
/// timer1 are the external clock.
pub type TimerDividers = [u32; 8];

const TIMER01_DIVIDERS: TimerDividers = [0, 1, 8, 64, 256, 1024, 0, 0];

/// CS values clocking from the T*n* pin.
const EXTERNAL_CLOCK_FALLING_EDGE: u8 = 6;
const EXTERNAL_CLOCK_RISING_EDGE: u8 = 7;

/// A timer's registers, bits, vectors and pins. Zero means "not on this
/// part" wherever an address or vector could be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AvrTimerConfig {
    pub bits: u8,
    pub dividers: TimerDividers,

    // Interrupt vectors
    pub capture_interrupt: u8,
    pub comp_a_interrupt: u8,
    pub comp_b_interrupt: u8,
    pub comp_c_interrupt: u8,
    pub ovf_interrupt: u8,

    // Register addresses
    pub tifr: u16,
    pub ocra: u16,
    pub ocrb: u16,
    pub ocrc: u16,
    pub icr: u16,
    pub tcnt: u16,
    pub tccra: u16,
    pub tccrb: u16,
    pub tccrc: u16,
    pub timsk: u16,

    // TIFR bits
    pub tov: u8,
    pub ocfa: u8,
    pub ocfb: u8,
    pub ocfc: u8,

    // TIMSK bits
    pub toie: u8,
    pub ociea: u8,
    pub ocieb: u8,
    pub ociec: u8,

    // Output compare pins: the PORT register's address, and the bit.
    pub comp_port_a: u16,
    pub comp_pin_a: u8,
    pub comp_port_b: u16,
    pub comp_pin_b: u8,
    pub comp_port_c: u16,
    pub comp_pin_c: u8,

    // External clock pin
    pub external_clock_port: u16,
    pub external_clock_pin: u8,
}

pub const TIMER0_CONFIG: AvrTimerConfig = AvrTimerConfig {
    bits: 8,
    capture_interrupt: 0, // not available
    comp_a_interrupt: 0x1c,
    comp_b_interrupt: 0x1e,
    comp_c_interrupt: 0,
    ovf_interrupt: 0x20,
    tifr: 0x35,
    ocra: 0x47,
    ocrb: 0x48,
    ocrc: 0, // not available
    icr: 0,  // not available
    tcnt: 0x46,
    tccra: 0x44,
    tccrb: 0x45,
    tccrc: 0, // not available
    timsk: 0x6e,
    dividers: TIMER01_DIVIDERS,
    comp_port_a: PORT_D_CONFIG.port,
    comp_pin_a: 6,
    comp_port_b: PORT_D_CONFIG.port,
    comp_pin_b: 5,
    comp_port_c: 0, // Not available
    comp_pin_c: 0,
    external_clock_port: PORT_D_CONFIG.port,
    external_clock_pin: 4,
    tov: 1,
    ocfa: 2,
    ocfb: 4,
    ocfc: 0,
    toie: 1,
    ociea: 2,
    ocieb: 4,
    ociec: 0,
};

pub const TIMER1_CONFIG: AvrTimerConfig = AvrTimerConfig {
    bits: 16,
    capture_interrupt: 0x14,
    comp_a_interrupt: 0x16,
    comp_b_interrupt: 0x18,
    comp_c_interrupt: 0,
    ovf_interrupt: 0x1a,
    tifr: 0x36,
    ocra: 0x88,
    ocrb: 0x8a,
    ocrc: 0, // not available
    icr: 0x86,
    tcnt: 0x84,
    tccra: 0x80,
    tccrb: 0x81,
    tccrc: 0x82,
    timsk: 0x6f,
    dividers: TIMER01_DIVIDERS,
    comp_port_a: PORT_B_CONFIG.port,
    comp_pin_a: 1,
    comp_port_b: PORT_B_CONFIG.port,
    comp_pin_b: 2,
    comp_port_c: 0, // Not available
    comp_pin_c: 0,
    external_clock_port: PORT_D_CONFIG.port,
    external_clock_pin: 5,
    ..TIMER0_CONFIG
};

pub const TIMER2_CONFIG: AvrTimerConfig = AvrTimerConfig {
    bits: 8,
    capture_interrupt: 0, // not available
    comp_a_interrupt: 0x0e,
    comp_b_interrupt: 0x10,
    comp_c_interrupt: 0,
    ovf_interrupt: 0x12,
    tifr: 0x37,
    ocra: 0xb3,
    ocrb: 0xb4,
    ocrc: 0, // not available
    icr: 0,  // not available
    tcnt: 0xb2,
    tccra: 0xb0,
    tccrb: 0xb1,
    tccrc: 0, // not available
    timsk: 0x70,
    dividers: [0, 1, 8, 32, 64, 128, 256, 1024],
    comp_port_a: PORT_B_CONFIG.port,
    comp_pin_a: 3,
    comp_port_b: PORT_D_CONFIG.port,
    comp_pin_b: 3,
    comp_port_c: 0, // Not available
    comp_pin_c: 0,
    external_clock_port: 0, // Not available
    external_clock_pin: 0,
    ..TIMER0_CONFIG
};

/* All the following types and constants are related to WGM (Waveform Generation Mode) bits: */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimerMode {
    Normal,
    PwmPhaseCorrect,
    Ctc,
    FastPwm,
    PwmPhaseFrequencyCorrect,
    Reserved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TovUpdateMode {
    Max,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OcrUpdateMode {
    Immediate,
    Top,
    Bottom,
}

/// Where the counter turns round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimerTopValue {
    Fixed(u16),
    OcrA,
    Icr,
}

/// Enable Toggle mode for OCxA in PWM Wave Generation mode
const OC_TOGGLE: u8 = 1;

type WgmConfig = (TimerMode, TimerTopValue, OcrUpdateMode, TovUpdateMode, u8);

use OcrUpdateMode as Ocr;
use TimerMode::*;
use TimerTopValue::{Fixed, Icr, OcrA};
use TovUpdateMode as Tov;

const WGM_MODES_8BIT: [WgmConfig; 8] = [
    /*0*/ (Normal, Fixed(0xff), Ocr::Immediate, Tov::Max, 0),
    /*1*/ (PwmPhaseCorrect, Fixed(0xff), Ocr::Top, Tov::Bottom, 0),
    /*2*/ (Ctc, OcrA, Ocr::Immediate, Tov::Max, 0),
    /*3*/ (FastPwm, Fixed(0xff), Ocr::Bottom, Tov::Max, 0),
    /*4*/ (Reserved, Fixed(0xff), Ocr::Immediate, Tov::Max, 0),
    /*5*/ (PwmPhaseCorrect, OcrA, Ocr::Top, Tov::Bottom, OC_TOGGLE),
    /*6*/ (Reserved, Fixed(0xff), Ocr::Immediate, Tov::Max, 0),
    /*7*/ (FastPwm, OcrA, Ocr::Bottom, Tov::Top, OC_TOGGLE),
];

// Table 16-4 in the datasheet
const WGM_MODES_16BIT: [WgmConfig; 16] = [
    /*0 */ (Normal, Fixed(0xffff), Ocr::Immediate, Tov::Max, 0),
    /*1 */ (PwmPhaseCorrect, Fixed(0x00ff), Ocr::Top, Tov::Bottom, 0),
    /*2 */ (PwmPhaseCorrect, Fixed(0x01ff), Ocr::Top, Tov::Bottom, 0),
    /*3 */ (PwmPhaseCorrect, Fixed(0x03ff), Ocr::Top, Tov::Bottom, 0),
    /*4 */ (Ctc, OcrA, Ocr::Immediate, Tov::Max, 0),
    /*5 */ (FastPwm, Fixed(0x00ff), Ocr::Bottom, Tov::Top, 0),
    /*6 */ (FastPwm, Fixed(0x01ff), Ocr::Bottom, Tov::Top, 0),
    /*7 */ (FastPwm, Fixed(0x03ff), Ocr::Bottom, Tov::Top, 0),
    /*8 */ (PwmPhaseFrequencyCorrect, Icr, Ocr::Bottom, Tov::Bottom, 0),
    /*9 */
    (
        PwmPhaseFrequencyCorrect,
        OcrA,
        Ocr::Bottom,
        Tov::Bottom,
        OC_TOGGLE,
    ),
    /*10*/ (PwmPhaseCorrect, Icr, Ocr::Top, Tov::Bottom, 0),
    /*11*/ (PwmPhaseCorrect, OcrA, Ocr::Top, Tov::Bottom, OC_TOGGLE),
    /*12*/ (Ctc, Icr, Ocr::Immediate, Tov::Max, 0),
    /*13*/ (Reserved, Fixed(0xffff), Ocr::Immediate, Tov::Max, 0),
    /*14*/ (FastPwm, Icr, Ocr::Bottom, Tov::Top, OC_TOGGLE),
    /*15*/ (FastPwm, OcrA, Ocr::Bottom, Tov::Top, OC_TOGGLE),
];

/// COM bits to what a compare match does in the non-PWM modes.
fn comp_to_override(comp: u8) -> PinOverrideMode {
    match comp {
        1 => PinOverrideMode::Toggle,
        2 => PinOverrideMode::Clear,
        3 => PinOverrideMode::Set,
        _ => PinOverrideMode::Enable,
    }
}

// Force Output Compare (FOC) bits
const FOCA: u8 = 1 << 7;
const FOCB: u8 = 1 << 6;
const FOCC: u8 = 1 << 5;

/// The compare channels, where avr8js passes `'A' | 'B' | 'C'`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Channel {
    A,
    B,
    C,
}

/// A timer/counter. avr8js's `AVRTimer`.
#[derive(Debug, Clone)]
pub struct AvrTimer {
    pub config: AvrTimerConfig,
    /// Its index among its owner's timers: what its hooks and clock event
    /// carry.
    index: u8,
    max: u16,
    last_cycle: u64,
    ocr_a: u16,
    next_ocr_a: u16,
    ocr_b: u16,
    next_ocr_b: u16,
    has_ocrc: bool,
    ocr_c: u16,
    next_ocr_c: u16,
    ocr_update_mode: OcrUpdateMode,
    tov_update_mode: TovUpdateMode,
    icr: u16, // only for 16-bit timers
    timer_mode: TimerMode,
    top_value: TimerTopValue,
    tcnt: u16,
    tcnt_next: u16,
    comp_a: u8,
    comp_b: u8,
    comp_c: u8,
    tcnt_updated: bool,
    update_divider: bool,
    counting_up: bool,
    divider: u32,
    /// The ports, by index among the owner's, that the compare outputs and
    /// the external clock are on: avr8js looks them up by address each time.
    comp_ports: [Option<usize>; 3],
    external_clock_port: Option<usize>,
    external_clock_rising_edge: bool,

    // This is the temporary register used to access 16-bit registers (section 16.3 of the datasheet)
    high_byte_temp: u8,

    // Interrupts
    ovf: InterruptConfig,
    ocfa: InterruptConfig,
    ocfb: InterruptConfig,
    ocfc: InterruptConfig,
}

impl AvrTimer {
    /// The timer, its registers hooked as the `index`th of its owner's
    /// timers. `ports` are the PORT addresses of the owner's ports, in order,
    /// for the compare outputs and the external clock to find theirs.
    pub fn new(cpu: &mut Cpu, config: AvrTimerConfig, index: u8, ports: &[u16]) -> AvrTimer {
        let port_index = |address: u16| {
            (address != 0)
                .then(|| ports.iter().position(|&port| port == address))
                .flatten()
        };
        let interrupt = |address, flag_mask, enable_mask| InterruptConfig {
            address,
            flag_register: config.tifr,
            flag_mask,
            enable_register: config.timsk,
            enable_mask,
            ..InterruptConfig::default()
        };
        let (timer_mode, top_value, ocr_update_mode, tov_update_mode, _) = WGM_MODES_8BIT[0];
        let mut timer = AvrTimer {
            config,
            index,
            max: if config.bits == 16 { 0xffff } else { 0xff },
            last_cycle: 0,
            ocr_a: 0,
            next_ocr_a: 0,
            ocr_b: 0,
            next_ocr_b: 0,
            has_ocrc: config.ocrc > 0,
            ocr_c: 0,
            next_ocr_c: 0,
            ocr_update_mode,
            tov_update_mode,
            icr: 0,
            timer_mode,
            top_value,
            tcnt: 0,
            tcnt_next: 0,
            comp_a: 0,
            comp_b: 0,
            comp_c: 0,
            tcnt_updated: false,
            update_divider: false,
            counting_up: true,
            divider: 0,
            comp_ports: [
                port_index(config.comp_port_a),
                port_index(config.comp_port_b),
                port_index(config.comp_port_c),
            ],
            external_clock_port: port_index(config.external_clock_port),
            external_clock_rising_edge: false,
            high_byte_temp: 0,
            ovf: interrupt(config.ovf_interrupt, config.tov, config.toie),
            ocfa: interrupt(config.comp_a_interrupt, config.ocfa, config.ociea),
            ocfb: interrupt(config.comp_b_interrupt, config.ocfb, config.ocieb),
            ocfc: interrupt(config.comp_c_interrupt, config.ocfc, config.ociec),
        };

        // No port is told anything yet: the compare outputs start
        // disconnected, as TCCRA reads.
        timer.update_wgm_config(cpu, &mut []);

        cpu.read_hooks[config.tcnt as usize] = Hook::TimerCounter(index);
        let hooks = &mut cpu.write_hooks;
        hooks[config.tcnt as usize] = Hook::TimerCounter(index);
        hooks[config.ocra as usize] = Hook::TimerOcrA(index);
        hooks[config.ocrb as usize] = Hook::TimerOcrB(index);
        if timer.has_ocrc {
            hooks[config.ocrc as usize] = Hook::TimerOcrC(index);
        }
        if config.bits == 16 {
            hooks[config.icr as usize] = Hook::TimerIcr(index);
            hooks[config.tcnt as usize + 1] = Hook::TimerTemp(index);
            hooks[config.ocra as usize + 1] = Hook::TimerOcrHigh(index);
            hooks[config.ocrb as usize + 1] = Hook::TimerOcrHigh(index);
            if timer.has_ocrc {
                hooks[config.ocrc as usize + 1] = Hook::TimerOcrHigh(index);
            }
            hooks[config.icr as usize + 1] = Hook::TimerTemp(index);
        }
        hooks[config.tccra as usize] = Hook::TimerControlA(index);
        hooks[config.tccrb as usize] = Hook::TimerControlB(index);
        if config.tccrc != 0 {
            hooks[config.tccrc as usize] = Hook::TimerControlC(index);
        }
        hooks[config.tifr as usize] = Hook::TimerFlags(index);
        hooks[config.timsk as usize] = Hook::TimerMask(index);

        timer
    }

    fn event(&self) -> ClockEvent {
        ClockEvent::TimerCount(self.index)
    }

    pub fn reset(&mut self) {
        self.divider = 0;
        self.last_cycle = 0;
        self.ocr_a = 0;
        self.next_ocr_a = 0;
        self.ocr_b = 0;
        self.next_ocr_b = 0;
        self.ocr_c = 0;
        self.next_ocr_c = 0;
        self.icr = 0;
        self.tcnt = 0;
        self.tcnt_next = 0;
        self.tcnt_updated = false;
        // avr8js: `false`, which sends a phase-correct counter down from 0.
        self.counting_up = true;
        self.update_divider = true;
    }

    fn tccra(&self, cpu: &Cpu) -> u8 {
        cpu.data[self.config.tccra as usize]
    }

    fn tccrb(&self, cpu: &Cpu) -> u8 {
        cpu.data[self.config.tccrb as usize]
    }

    fn cs(&self, cpu: &Cpu) -> u8 {
        self.tccrb(cpu) & 0x7
    }

    fn wgm(&self, cpu: &Cpu) -> usize {
        let mask = if self.config.bits == 16 { 0x18 } else { 0x8 };
        (((self.tccrb(cpu) & mask) >> 1) | (self.tccra(cpu) & 0x3)) as usize
    }

    fn top(&self) -> u16 {
        match self.top_value {
            OcrA => self.ocr_a,
            Icr => self.icr,
            Fixed(value) => value,
        }
    }

    fn ocr_mask(&self) -> u16 {
        match self.top_value {
            OcrA | Icr => 0xffff,
            Fixed(value) => value,
        }
    }

    /// The raw value of TCNT, for the tests.
    pub fn debug_tcnt(&self) -> u16 {
        self.tcnt
    }

    /// A read of TCNT's low byte: the count brought up to date, and for a
    /// 16-bit timer, its high byte latched for the read that follows.
    pub(crate) fn read_tcnt(&mut self, cpu: &mut Cpu, ports: &mut [AvrIoPort], addr: u16) -> u8 {
        self.count(cpu, ports, false, false);
        if self.config.bits == 16 {
            cpu.data[addr as usize + 1] = (self.tcnt >> 8) as u8;
        }
        cpu.data[addr as usize] = self.tcnt as u8;
        self.tcnt as u8
    }

    /// A write to one of the timer's registers; true if it stored the value
    /// itself, as a hook returns.
    pub(crate) fn write(
        &mut self,
        cpu: &mut Cpu,
        ports: &mut [AvrIoPort],
        hook: Hook,
        value: u8,
        addr: u16,
    ) -> bool {
        let config = self.config;
        match hook {
            Hook::TimerCounter(_) => {
                self.tcnt_next = ((self.high_byte_temp as u16) << 8) | value as u16;
                self.counting_up = true;
                self.tcnt_updated = true;
                cpu.update_clock_event(self.event(), 0);
                if self.divider != 0 {
                    self.timer_updated(cpu, ports, self.tcnt_next, self.tcnt_next);
                }
                false
            }
            Hook::TimerOcrA(_) => {
                self.next_ocr_a = ((self.high_byte_temp as u16) << 8) | value as u16;
                if self.ocr_update_mode == Ocr::Immediate {
                    self.ocr_a = self.next_ocr_a;
                }
                false
            }
            Hook::TimerOcrB(_) => {
                self.next_ocr_b = ((self.high_byte_temp as u16) << 8) | value as u16;
                if self.ocr_update_mode == Ocr::Immediate {
                    self.ocr_b = self.next_ocr_b;
                }
                false
            }
            Hook::TimerOcrC(_) => {
                self.next_ocr_c = ((self.high_byte_temp as u16) << 8) | value as u16;
                if self.ocr_update_mode == Ocr::Immediate {
                    self.ocr_c = self.next_ocr_c;
                }
                false
            }
            Hook::TimerIcr(_) => {
                self.icr = ((self.high_byte_temp as u16) << 8) | value as u16;
                false
            }
            Hook::TimerTemp(_) => {
                self.high_byte_temp = value;
                false
            }
            Hook::TimerOcrHigh(_) => {
                self.high_byte_temp = value & (self.ocr_mask() >> 8) as u8;
                cpu.data[addr as usize] = self.high_byte_temp;
                true
            }
            Hook::TimerControlA(_) => {
                cpu.data[config.tccra as usize] = value;
                self.update_wgm_config(cpu, ports);
                true
            }
            Hook::TimerControlB(_) => {
                let mut value = value;
                if config.tccrc == 0 {
                    self.check_force_compare(cpu, ports, value);
                    value &= !(FOCA | FOCB);
                }
                cpu.data[config.tccrb as usize] = value;
                self.update_divider = true;
                cpu.clear_clock_event(self.event());
                cpu.add_clock_event(self.event(), 0);
                self.update_wgm_config(cpu, ports);
                true
            }
            Hook::TimerControlC(_) => {
                self.check_force_compare(cpu, ports, value);
                false
            }
            Hook::TimerFlags(_) => {
                // avr8js: stores `value` first, clearing the flags written as 0.
                cpu.clear_interrupt_by_flag(&self.ovf, value);
                cpu.clear_interrupt_by_flag(&self.ocfa, value);
                cpu.clear_interrupt_by_flag(&self.ocfb, value);
                true
            }
            Hook::TimerMask(_) => {
                cpu.update_interrupt_enable(&self.ovf, value);
                cpu.update_interrupt_enable(&self.ocfa, value);
                cpu.update_interrupt_enable(&self.ocfb, value);
                false
            }
            _ => false,
        }
    }

    fn update_wgm_config(&mut self, cpu: &mut Cpu, ports: &mut [AvrIoPort]) {
        let wgm = self.wgm(cpu);
        let tccra = cpu.data[self.config.tccra as usize];
        let (timer_mode, top_value, ocr_update_mode, tov_update_mode, flags) =
            if self.config.bits == 16 {
                WGM_MODES_16BIT[wgm]
            } else {
                WGM_MODES_8BIT[wgm]
            };
        self.timer_mode = timer_mode;
        self.top_value = top_value;
        self.ocr_update_mode = ocr_update_mode;
        self.tov_update_mode = tov_update_mode;

        let pwm_mode = matches!(
            timer_mode,
            FastPwm | PwmPhaseCorrect | PwmPhaseFrequencyCorrect
        );

        let prev_comp_a = self.comp_a;
        self.comp_a = (tccra >> 6) & 0x3;
        if self.comp_a == 1 && pwm_mode && flags & OC_TOGGLE == 0 {
            self.comp_a = 0;
        }
        if (prev_comp_a != 0) != (self.comp_a != 0) {
            let mode = if self.comp_a != 0 {
                PinOverrideMode::Enable
            } else {
                PinOverrideMode::None
            };
            self.update_comp(cpu, ports, Channel::A, mode);
        }

        let prev_comp_b = self.comp_b;
        self.comp_b = (tccra >> 4) & 0x3;
        if self.comp_b == 1 && pwm_mode {
            self.comp_b = 0; // Reserved, according to the datasheet
        }
        if (prev_comp_b != 0) != (self.comp_b != 0) {
            let mode = if self.comp_b != 0 {
                PinOverrideMode::Enable
            } else {
                PinOverrideMode::None
            };
            self.update_comp(cpu, ports, Channel::B, mode);
        }

        if self.has_ocrc {
            let prev_comp_c = self.comp_c;
            self.comp_c = (tccra >> 2) & 0x3;
            if self.comp_c == 1 && pwm_mode {
                self.comp_c = 0; // Reserved, according to the datasheet
            }
            if (prev_comp_c != 0) != (self.comp_c != 0) {
                let mode = if self.comp_c != 0 {
                    PinOverrideMode::Enable
                } else {
                    PinOverrideMode::None
                };
                self.update_comp(cpu, ports, Channel::C, mode);
            }
        }
    }

    /// Brings the counter up to date. Runs as the timer's clock event, with
    /// `reschedule`, and when TCNT is read, without; `external` is one edge
    /// of the T*n* pin.
    pub(crate) fn count(
        &mut self,
        cpu: &mut Cpu,
        ports: &mut [AvrIoPort],
        reschedule: bool,
        external: bool,
    ) {
        let divider = self.divider as u64;
        let delta = cpu.cycles - self.last_cycle;
        if (divider != 0 && delta >= divider) || external {
            let counter_delta = if external { 1 } else { delta / divider };
            self.last_cycle += counter_delta * divider;
            let val = self.tcnt;
            let timer_mode = self.timer_mode;
            let top = self.top();
            let phase_pwm = matches!(timer_mode, PwmPhaseCorrect | PwmPhaseFrequencyCorrect);
            let new_val = if phase_pwm {
                self.phase_pwm_count(cpu, ports, val, counter_delta)
            } else {
                ((val as u64 + counter_delta) % (top as u64 + 1)) as u16
            };
            let overflow = val as u64 + counter_delta > top as u64;
            // A CPU write overrides (has priority over) all counter clear or count operations.
            if !self.tcnt_updated {
                self.tcnt = new_val;
                if !phase_pwm {
                    self.timer_updated(cpu, ports, new_val, val);
                }
            }

            if !phase_pwm {
                if timer_mode == FastPwm && overflow {
                    let (comp_a, comp_b) = (self.comp_a, self.comp_b);
                    if comp_a != 0 {
                        self.update_comp_pin(cpu, ports, comp_a, Channel::A, true);
                    }
                    if comp_b != 0 {
                        self.update_comp_pin(cpu, ports, comp_b, Channel::B, true);
                    }
                }

                if self.ocr_update_mode == Ocr::Bottom && overflow {
                    // OCRUpdateMode.Top only occurs in Phase Correct modes, handled by phasePwmCount()
                    self.ocr_a = self.next_ocr_a;
                    self.ocr_b = self.next_ocr_b;
                    self.ocr_c = self.next_ocr_c;
                }

                // OCRUpdateMode.Bottom only occurs in Phase Correct modes, handled by phasePwmCount().
                // Thus we only handle TOVUpdateMode.Top or TOVUpdateMode.Max here.
                if overflow && (self.tov_update_mode == Tov::Top || top == self.max) {
                    cpu.set_interrupt_flag(&self.ovf);
                }
            }
        }
        if self.tcnt_updated {
            self.tcnt = self.tcnt_next;
            self.tcnt_updated = false;
            if (self.tcnt == 0 && self.ocr_update_mode == Ocr::Bottom)
                || (self.tcnt == self.top() && self.ocr_update_mode == Ocr::Top)
            {
                self.ocr_a = self.next_ocr_a;
                self.ocr_b = self.next_ocr_b;
                self.ocr_c = self.next_ocr_c;
            }
        }
        if self.update_divider {
            let cs = self.cs(cpu);
            let external_clock_pin = self.config.external_clock_pin as usize;
            let new_divider = self.config.dividers[cs as usize];
            self.last_cycle = if new_divider != 0 { cpu.cycles } else { 0 };
            self.update_divider = false;
            self.divider = new_divider;
            if let Some(port) = self.external_clock_port {
                ports[port].external_clock_listeners[external_clock_pin] = None;
            }
            if new_divider != 0 {
                let event = self.event();
                cpu.add_clock_event(
                    event,
                    (self.last_cycle + new_divider as u64).saturating_sub(cpu.cycles),
                );
            } else if let Some(port) = self.external_clock_port
                && (cs == EXTERNAL_CLOCK_FALLING_EDGE || cs == EXTERNAL_CLOCK_RISING_EDGE)
            {
                ports[port].external_clock_listeners[external_clock_pin] = Some(self.index);
                self.external_clock_rising_edge = cs == EXTERNAL_CLOCK_RISING_EDGE;
            }
            return;
        }
        if reschedule && divider != 0 {
            let event = self.event();
            cpu.add_clock_event(
                event,
                (self.last_cycle + divider).saturating_sub(cpu.cycles),
            );
        }
    }

    /// An edge on the T*n* pin, `value` its new level.
    pub(crate) fn external_clock_callback(
        &mut self,
        cpu: &mut Cpu,
        ports: &mut [AvrIoPort],
        value: bool,
    ) {
        if value == self.external_clock_rising_edge {
            self.count(cpu, ports, false, true);
        }
    }

    fn phase_pwm_count(
        &mut self,
        cpu: &mut Cpu,
        ports: &mut [AvrIoPort],
        value: u16,
        delta: u64,
    ) -> u16 {
        let (ocr_a, ocr_b, ocr_c, has_ocrc, top, max, tcnt_updated) = (
            self.ocr_a as i32,
            self.ocr_b as i32,
            self.ocr_c as i32,
            self.has_ocrc,
            self.top() as i32,
            self.max as i32,
            self.tcnt_updated,
        );
        // Signed, as avr8js's number is: a counter above TOP (or below 0 while
        // counting down) runs on past it rather than wrapping at 16 bits.
        let mut value = value as i32;
        let mut delta = delta;
        if value == 0 && top == 0 {
            delta = 0;
            if self.ocr_update_mode == Ocr::Top {
                self.ocr_a = self.next_ocr_a;
                self.ocr_b = self.next_ocr_b;
                self.ocr_c = self.next_ocr_c;
            }
        }
        while delta > 0 {
            if self.counting_up {
                value += 1;
                if value == top && !tcnt_updated {
                    self.counting_up = false;
                    if self.ocr_update_mode == Ocr::Top {
                        self.ocr_a = self.next_ocr_a;
                        self.ocr_b = self.next_ocr_b;
                        self.ocr_c = self.next_ocr_c;
                    }
                }
            } else {
                value -= 1;
                if value == 0 && !tcnt_updated {
                    self.counting_up = true;
                    cpu.set_interrupt_flag(&self.ovf);
                    if self.ocr_update_mode == Ocr::Bottom {
                        self.ocr_a = self.next_ocr_a;
                        self.ocr_b = self.next_ocr_b;
                        self.ocr_c = self.next_ocr_c;
                    }
                }
            }
            if !tcnt_updated {
                if value == ocr_a {
                    cpu.set_interrupt_flag(&self.ocfa);
                    if self.comp_a != 0 {
                        self.update_comp_pin(cpu, ports, self.comp_a, Channel::A, false);
                    }
                }
                if value == ocr_b {
                    cpu.set_interrupt_flag(&self.ocfb);
                    if self.comp_b != 0 {
                        self.update_comp_pin(cpu, ports, self.comp_b, Channel::B, false);
                    }
                }
                if has_ocrc && value == ocr_c {
                    cpu.set_interrupt_flag(&self.ocfc);
                    if self.comp_c != 0 {
                        self.update_comp_pin(cpu, ports, self.comp_c, Channel::C, false);
                    }
                }
            }
            delta -= 1;
        }
        (value & max) as u16
    }

    fn timer_updated(
        &mut self,
        cpu: &mut Cpu,
        ports: &mut [AvrIoPort],
        value: u16,
        prev_value: u16,
    ) {
        let (ocr_a, ocr_b, ocr_c, has_ocrc) = (self.ocr_a, self.ocr_b, self.ocr_c, self.has_ocrc);
        let overflow = prev_value > value;
        let matched = |ocr: u16| {
            ((prev_value < ocr || overflow) && value >= ocr) || (prev_value < ocr && overflow)
        };
        if matched(ocr_a) {
            cpu.set_interrupt_flag(&self.ocfa);
            if self.comp_a != 0 {
                self.update_comp_pin(cpu, ports, self.comp_a, Channel::A, false);
            }
        }
        if matched(ocr_b) {
            cpu.set_interrupt_flag(&self.ocfb);
            if self.comp_b != 0 {
                self.update_comp_pin(cpu, ports, self.comp_b, Channel::B, false);
            }
        }
        if has_ocrc && matched(ocr_c) {
            cpu.set_interrupt_flag(&self.ocfc);
            if self.comp_c != 0 {
                self.update_comp_pin(cpu, ports, self.comp_c, Channel::C, false);
            }
        }
    }

    fn check_force_compare(&mut self, cpu: &mut Cpu, ports: &mut [AvrIoPort], value: u8) {
        if matches!(
            self.timer_mode,
            FastPwm | PwmPhaseCorrect | PwmPhaseFrequencyCorrect
        ) {
            // The FOCnA/FOCnB/FOCnC bits are only active when the WGMn3:0 bits specifies a non-PWM mode
            return;
        }
        if value & FOCA != 0 {
            self.update_comp_pin(cpu, ports, self.comp_a, Channel::A, false);
        }
        if value & FOCB != 0 {
            self.update_comp_pin(cpu, ports, self.comp_b, Channel::B, false);
        }
        if self.config.comp_port_c != 0 && value & FOCC != 0 {
            self.update_comp_pin(cpu, ports, self.comp_c, Channel::C, false);
        }
    }

    fn update_comp_pin(
        &mut self,
        cpu: &mut Cpu,
        ports: &mut [AvrIoPort],
        comp_value: u8,
        channel: Channel,
        bottom: bool,
    ) {
        let inverting_mode = comp_value == 3;
        let is_set = self.counting_up == inverting_mode;
        let new_value = match self.timer_mode {
            Normal | Ctc => comp_to_override(comp_value),
            FastPwm => {
                if comp_value == 1 {
                    if bottom {
                        PinOverrideMode::None
                    } else {
                        PinOverrideMode::Toggle
                    }
                } else if inverting_mode != bottom {
                    PinOverrideMode::Set
                } else {
                    PinOverrideMode::Clear
                }
            }
            PwmPhaseCorrect | PwmPhaseFrequencyCorrect => {
                if comp_value == 1 {
                    PinOverrideMode::Toggle
                } else if is_set {
                    PinOverrideMode::Set
                } else {
                    PinOverrideMode::Clear
                }
            }
            Reserved => PinOverrideMode::None,
        };

        if new_value != PinOverrideMode::None {
            self.update_comp(cpu, ports, channel, new_value);
        }
    }

    /// avr8js's `updateCompA`, `updateCompB` and `updateCompC`.
    fn update_comp(
        &self,
        cpu: &mut Cpu,
        ports: &mut [AvrIoPort],
        channel: Channel,
        value: PinOverrideMode,
    ) {
        let (port, pin) = match channel {
            Channel::A => (self.comp_ports[0], self.config.comp_pin_a),
            Channel::B => (self.comp_ports[1], self.config.comp_pin_b),
            Channel::C => (self.comp_ports[2], self.config.comp_pin_c),
        };
        if let Some(port) = port.and_then(|port| ports.get_mut(port)) {
            port.timer_override_pin(cpu, pin, value);
        }
    }
}
