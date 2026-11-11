// SPDX-License-Identifier: MIT OR Apache-2.0
// Ported from avr8js, src/peripherals/usart.ts.
// Copyright (c) 2019-2025 Uri Shaked, MIT licence (see NOTICE.md beside this file).

//! The ATmega328P's USART0: what `Serial` talks through (A-06, ADR 0017).
//!
//! A byte written to UDR is sent at the rate UBRR and U2X set, framed as
//! UCSRC says — a start bit, five to nine data bits, a parity bit or none,
//! one or two stop bits; UDRE and TXC come back when it has gone. A byte
//! arriving is put in UDR with RXC raised.
//!
//! As in avr8js, nothing is shifted out bit by bit. Where avr8js calls
//! `onByteTransmit`, this keeps each byte in [`AvrUsart::transmitted`] with
//! the cycle it was written at and the shape of its frame, so that whoever
//! listens can work out what a receiver at another baud makes of it
//! (`crate::serial`). A byte arriving is [`AvrUsart::write_byte`], or, at a
//! cycle the listener has worked out, [`AvrUsart::receive_at`].
//!
//! **Where this differs from avr8js 0.21.1**:
//!
//! - writing UCSRA leaves RXC and UDRE as they were — they are read-only —
//!   and TXC unless it is written 1. Upstream stores only the U2X and MPCM
//!   bits written, which clears both flags; the Arduino core gets away with
//!   it because it writes UCSRA just before UCSRB raises UDRE again;
//! - `onByteTransmit`, `onLineTransmit`, `onRxComplete` and
//!   `onConfigurationChange` are not callbacks: the first is
//!   [`AvrUsart::transmitted`], the last [`AvrUsart::configuration_changed`],
//!   and the line buffer is the listener's business;
//! - turning the transmitter or the receiver on or off raises
//!   [`AvrUsart::pins_changed`], because each takes a pin over from its port
//!   (§20.3, §20.6): TXD (PD1) is driven, idling high; RXD (PD0) is an input.

use super::cpu::{ClockEvent, Cpu, Hook, InterruptConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AvrUsartConfig {
    pub rx_complete_interrupt: u8,
    pub data_register_empty_interrupt: u8,
    pub tx_complete_interrupt: u8,
    pub ucsra: u16,
    pub ucsrb: u16,
    pub ucsrc: u16,
    pub ubrrl: u16,
    pub ubrrh: u16,
    pub udr: u16,
}

pub const USART0_CONFIG: AvrUsartConfig = AvrUsartConfig {
    rx_complete_interrupt: 0x24,
    data_register_empty_interrupt: 0x26,
    tx_complete_interrupt: 0x28,
    ucsra: 0xc0,
    ucsrb: 0xc1,
    ucsrc: 0xc2,
    ubrrl: 0xc4,
    ubrrh: 0xc5,
    udr: 0xc6,
};

// Register bits
const UCSRA_RXC: u8 = 0x80; // USART Receive Complete
const UCSRA_TXC: u8 = 0x40; // USART Transmit Complete
const UCSRA_UDRE: u8 = 0x20; // USART Data Register Empty
const UCSRA_U2X: u8 = 0x2; // Double the USART Transmission Speed
const UCSRA_MPCM: u8 = 0x1; // Multi-processor Communication Mode
const UCSRA_CFG_MASK: u8 = UCSRA_U2X;
const UCSRB_RXCIE: u8 = 0x80; // RX Complete Interrupt Enable
const UCSRB_TXCIE: u8 = 0x40; // TX Complete Interrupt Enable
const UCSRB_UDRIE: u8 = 0x20; // USART Data Register Empty Interrupt Enable
const UCSRB_RXEN: u8 = 0x10; // Receiver Enable
const UCSRB_TXEN: u8 = 0x8; // Transmitter Enable
const UCSRB_UCSZ2: u8 = 0x4; // Character Size 2
const UCSRB_CFG_MASK: u8 = UCSRB_UCSZ2 | UCSRB_RXEN | UCSRB_TXEN;
const UCSRC_UPM1: u8 = 0x20; // Parity Mode 1
const UCSRC_UPM0: u8 = 0x10; // Parity Mode 0
const UCSRC_USBS: u8 = 0x8; // Stop Bit Select
const UCSRC_UCSZ1: u8 = 0x4; // Character Size 1
const UCSRC_UCSZ0: u8 = 0x2; // Character Size 0

/// How a character is framed on the line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameShape {
    /// CPU cycles a bit lasts.
    pub cycles_per_bit: u64,
    /// Five to nine.
    pub data_bits: u8,
    /// `Some(true)` for odd parity, `Some(false)` for even, `None` for none.
    pub parity: Option<bool>,
    /// One or two.
    pub stop_bits: u8,
}

impl FrameShape {
    /// Bits in a frame, start and stop included.
    pub fn bits(&self) -> u64 {
        1 + u64::from(self.data_bits) + u64::from(self.parity.is_some()) + u64::from(self.stop_bits)
    }
}

/// A byte the program sent: when it was written to UDR, and how it went.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sent {
    pub cycles: u64,
    pub value: u16,
    pub shape: FrameShape,
}

#[derive(Debug, Clone)]
pub struct AvrUsart {
    pub config: AvrUsartConfig,
    pub freq_hz: u64,
    /// What has been sent since the listener last took it.
    pub transmitted: Vec<Sent>,
    /// The baud, the frame or an enable changed: avr8js's
    /// `onConfigurationChange`. Stays set until someone clears it.
    pub configuration_changed: bool,
    /// The transmitter or the receiver was turned on or off, so a pin
    /// changed hands. Cleared with the ports' changes.
    pub pins_changed: bool,
    rx_busy: bool,
    rx_byte: u16,
    rxc: InterruptConfig,
    udre: InterruptConfig,
    txc: InterruptConfig,
}

impl AvrUsart {
    pub fn new(cpu: &mut Cpu, config: AvrUsartConfig, freq_hz: u64) -> AvrUsart {
        let interrupt = |address, flag_mask, enable_mask, constant| InterruptConfig {
            address,
            flag_register: config.ucsra,
            flag_mask,
            enable_register: config.ucsrb,
            enable_mask,
            constant,
            ..InterruptConfig::default()
        };
        let mut usart = AvrUsart {
            config,
            freq_hz,
            transmitted: Vec::new(),
            configuration_changed: false,
            pins_changed: false,
            rx_busy: false,
            rx_byte: 0,
            rxc: interrupt(config.rx_complete_interrupt, UCSRA_RXC, UCSRB_RXCIE, true),
            udre: interrupt(
                config.data_register_empty_interrupt,
                UCSRA_UDRE,
                UCSRB_UDRIE,
                false,
            ),
            txc: interrupt(config.tx_complete_interrupt, UCSRA_TXC, UCSRB_TXCIE, false),
        };
        usart.reset(cpu);
        cpu.write_hooks[config.ucsra as usize] = Hook::UsartStatus;
        cpu.write_hooks[config.ucsrb as usize] = Hook::UsartControlB;
        cpu.write_hooks[config.ucsrc as usize] = Hook::UsartControlC;
        cpu.read_hooks[config.udr as usize] = Hook::UsartData;
        cpu.write_hooks[config.udr as usize] = Hook::UsartData;
        cpu.write_hooks[config.ubrrh as usize] = Hook::UsartBaud;
        cpu.write_hooks[config.ubrrl as usize] = Hook::UsartBaud;
        usart
    }

    pub fn reset(&mut self, cpu: &mut Cpu) {
        cpu.data[self.config.ucsra as usize] = UCSRA_UDRE;
        cpu.data[self.config.ucsrb as usize] = 0;
        cpu.data[self.config.ucsrc as usize] = UCSRC_UCSZ1 | UCSRC_UCSZ0; // default: 8 bits per byte
        self.rx_busy = false;
        self.rx_byte = 0;
        self.transmitted.clear();
    }

    /// A write to one of its registers. Returns whether it stored the value
    /// itself.
    pub fn write(
        &mut self,
        cpu: &mut Cpu,
        hook: Hook,
        value: u8,
        old_value: u8,
        addr: u16,
    ) -> bool {
        let config = self.config;
        match hook {
            Hook::UsartStatus => {
                let flags = old_value & (UCSRA_RXC | UCSRA_UDRE | UCSRA_TXC);
                cpu.data[config.ucsra as usize] = flags | (value & (UCSRA_MPCM | UCSRA_U2X));
                cpu.clear_interrupt_by_flag(&self.txc, value);
                if (value & UCSRA_CFG_MASK) != (old_value & UCSRA_CFG_MASK) {
                    self.configuration_changed = true;
                }
                true
            }
            Hook::UsartControlB => {
                cpu.update_interrupt_enable(&self.rxc, value);
                cpu.update_interrupt_enable(&self.udre, value);
                cpu.update_interrupt_enable(&self.txc, value);
                if value & UCSRB_RXEN != 0 && old_value & UCSRB_RXEN != 0 {
                    cpu.clear_interrupt(&self.rxc, true);
                }
                if value & UCSRB_TXEN != 0 && old_value & UCSRB_TXEN == 0 {
                    // Enabling the transmission - mark UDR as empty
                    cpu.set_interrupt_flag(&self.udre);
                }
                cpu.data[config.ucsrb as usize] = value;
                if (value & UCSRB_CFG_MASK) != (old_value & UCSRB_CFG_MASK) {
                    self.configuration_changed = true;
                }
                if (value ^ old_value) & (UCSRB_RXEN | UCSRB_TXEN) != 0 {
                    self.pins_changed = true;
                }
                true
            }
            Hook::UsartControlC => {
                cpu.data[config.ucsrc as usize] = value;
                self.configuration_changed = true;
                true
            }
            Hook::UsartBaud => {
                cpu.data[addr as usize] = value;
                self.configuration_changed = true;
                true
            }
            Hook::UsartData => {
                self.transmitted.push(Sent {
                    cycles: cpu.cycles,
                    value: u16::from(value),
                    shape: self.shape(cpu),
                });
                cpu.add_clock_event(ClockEvent::UsartTransmitted, self.cycles_per_char(cpu));
                cpu.clear_interrupt(&self.txc, true);
                cpu.clear_interrupt(&self.udre, true);
                false
            }
            _ => false,
        }
    }

    /// A read of UDR: the byte received, which empties it.
    pub fn read_data(&mut self, cpu: &mut Cpu) -> u8 {
        let mask = match self.bits_per_char(cpu) {
            5 => 0x1f,
            6 => 0x3f,
            7 => 0x7f,
            _ => 0xff,
        };
        let result = (self.rx_byte & mask) as u8;
        self.rx_byte = 0;
        cpu.clear_interrupt(&self.rxc, true);
        result
    }

    /// The byte written to UDR has gone.
    pub fn transmitted_event(&mut self, cpu: &mut Cpu) {
        cpu.set_interrupt_flag(&self.udre);
        cpu.set_interrupt_flag(&self.txc);
    }

    /// A byte has arrived: avr8js's deferred `writeByte(value, true)`.
    pub fn received_event(&mut self, cpu: &mut Cpu, value: u16) {
        self.rx_busy = false;
        self.write_byte(cpu, value, true);
    }

    pub fn rx_busy(&self) -> bool {
        self.rx_busy
    }

    /// Something on the line sends the chip `value`: at once if `immediate`,
    /// otherwise a character's time from now. False if the receiver is off,
    /// or busy with the last one.
    pub fn write_byte(&mut self, cpu: &mut Cpu, value: u16, immediate: bool) -> bool {
        if self.rx_busy || !self.rx_enable(cpu) {
            return false;
        }
        if immediate {
            self.rx_byte = value;
            cpu.set_interrupt_flag(&self.rxc);
        } else {
            self.rx_busy = true;
            cpu.add_clock_event(ClockEvent::UsartReceive(value), self.cycles_per_char(cpu));
        }
        true
    }

    /// `value` finishes arriving `cycles` from now, as a receiver listening
    /// to the line has worked out. Dropped then if the receiver is off.
    pub fn receive_at(&mut self, cpu: &mut Cpu, value: u16, cycles: u64) {
        cpu.add_clock_event(ClockEvent::UsartReceive(value), cycles);
    }

    /// How a character is framed now.
    pub fn shape(&self, cpu: &Cpu) -> FrameShape {
        FrameShape {
            cycles_per_bit: (u64::from(self.ubrr(cpu)) + 1) * self.multiplier(cpu),
            data_bits: self.bits_per_char(cpu),
            parity: self.parity_enabled(cpu).then(|| self.parity_odd(cpu)),
            stop_bits: self.stop_bits(cpu),
        }
    }

    pub fn cycles_per_char(&self, cpu: &Cpu) -> u64 {
        let shape = self.shape(cpu);
        shape.cycles_per_bit * shape.bits()
    }

    pub fn ubrr(&self, cpu: &Cpu) -> u16 {
        u16::from(cpu.data[self.config.ubrrh as usize]) << 8
            | u16::from(cpu.data[self.config.ubrrl as usize])
    }

    pub fn multiplier(&self, cpu: &Cpu) -> u64 {
        if cpu.data[self.config.ucsra as usize] & UCSRA_U2X != 0 {
            8
        } else {
            16
        }
    }

    pub fn rx_enable(&self, cpu: &Cpu) -> bool {
        cpu.data[self.config.ucsrb as usize] & UCSRB_RXEN != 0
    }

    pub fn tx_enable(&self, cpu: &Cpu) -> bool {
        cpu.data[self.config.ucsrb as usize] & UCSRB_TXEN != 0
    }

    pub fn baud_rate(&self, cpu: &Cpu) -> u64 {
        self.freq_hz / (self.multiplier(cpu) * (1 + u64::from(self.ubrr(cpu))))
    }

    pub fn bits_per_char(&self, cpu: &Cpu) -> u8 {
        let ucsz = ((cpu.data[self.config.ucsrc as usize] & (UCSRC_UCSZ1 | UCSRC_UCSZ0)) >> 1)
            | (cpu.data[self.config.ucsrb as usize] & UCSRB_UCSZ2);
        match ucsz {
            0 => 5,
            1 => 6,
            2 => 7,
            3 => 8,
            // 4..6 are reserved
            _ => 9,
        }
    }

    pub fn stop_bits(&self, cpu: &Cpu) -> u8 {
        if cpu.data[self.config.ucsrc as usize] & UCSRC_USBS != 0 {
            2
        } else {
            1
        }
    }

    pub fn parity_enabled(&self, cpu: &Cpu) -> bool {
        cpu.data[self.config.ucsrc as usize] & UCSRC_UPM1 != 0
    }

    pub fn parity_odd(&self, cpu: &Cpu) -> bool {
        cpu.data[self.config.ucsrc as usize] & UCSRC_UPM0 != 0
    }
}
