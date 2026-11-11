// SPDX-License-Identifier: MIT OR Apache-2.0
// Ported from avr8js, src/peripherals/gpio.ts.
// Copyright (c) 2019-2025 Uri Shaked, MIT licence (see NOTICE.md beside this file).

//! The AVR's I/O ports: what the program drives onto its pins and what it
//! reads back from them (ADR 0017).
//!
//! A port is three registers. DDR says which pins are outputs; PORT is what an
//! output drives, or for an input whether its pull-up is on; PIN is what the
//! pins read, and writing a 1 to it toggles PORT. A timer's compare output can
//! take a pin over from PORT ([`AvrIoPort::timer_override_pin`]), and an edge
//! on a pin can raise INT0, INT1 or a pin-change interrupt, or clock a timer.
//!
//! **Listeners become a flag.** avr8js calls its listeners whenever what the
//! port drives changes. The embedder here needs one thing from that: to stop
//! on the instruction that changed a pin, so the circuit can be solved up to
//! that moment. So a change sets [`AvrIoPort::changed`], one `bool` per port to
//! test after each instruction, and [`AvrIoPort::take_change`] says what the
//! port drives now and what it drove when it was last asked — the arguments
//! avr8js's listener would have had, for a single change.
//!
//! **An external clock is an edge queued for later.** A pin that clocks a
//! timer cannot call the timer from inside the port, since the timer may be
//! the one driving the pin; the edge is queued on the port and the owner of
//! both hands it to the timer once the port is done (see `Peripherals`).
//!
//! **Where this differs from avr8js 0.21.1**: upstream, writing PCMSK*n* calls
//! `updateInterruptEnable` for every port's pin-change interrupt with the
//! *mask's* value, which can drop another port's pending interrupt, and
//! writing PCICR — the register that actually enables them — has no hook, so
//! a flag raised before the enable is never taken. Here PCMSK is a plain
//! register and PCICR does what PCMSK's hook did, with PCICR's value.
//!
//! Only the ATmega328P's ports, B, C and D, are described.

use super::cpu::{Cpu, Hook, InterruptConfig};

/// An external interrupt, INT*n*: which bits of which registers drive it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AvrExternalInterrupt {
    /// EICRA or EICRB, whichever holds this interrupt's ISC bits.
    pub eicr: u16,
    pub eimsk: u16,
    pub eifr: u16,
    /// Offset of the ISC*n*0/ISC*n*1 bits in EICR.
    pub isc_offset: u8,
    /// Bit index in EIMSK and EIFR.
    pub index: u8,
    /// Interrupt vector.
    pub interrupt: u8,
}

/// A pin-change interrupt, PCINT*n*, covering one port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AvrPinChangeInterrupt {
    /// Bit index in PCICR and PCIFR.
    pub pcie: u8,
    pub pcicr: u16,
    pub pcifr: u16,
    pub pcmsk: u16,
    pub pin_change_interrupt: u8,
    pub mask: u8,
    pub offset: u8,
}

/// A port's registers and interrupts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AvrPortConfig {
    pub pin: u16,
    pub ddr: u16,
    pub port: u16,
    pub pin_change: Option<AvrPinChangeInterrupt>,
    /// By pin.
    pub external_interrupts: [Option<AvrExternalInterrupt>; 8],
}

pub const INT0: AvrExternalInterrupt = AvrExternalInterrupt {
    eicr: 0x69,
    eimsk: 0x3d,
    eifr: 0x3c,
    index: 0,
    isc_offset: 0,
    interrupt: 2,
};

pub const INT1: AvrExternalInterrupt = AvrExternalInterrupt {
    eicr: 0x69,
    eimsk: 0x3d,
    eifr: 0x3c,
    index: 1,
    isc_offset: 2,
    interrupt: 4,
};

pub const PCINT0: AvrPinChangeInterrupt = AvrPinChangeInterrupt {
    pcie: 0,
    pcicr: 0x68,
    pcifr: 0x3b,
    pcmsk: 0x6b,
    pin_change_interrupt: 6,
    mask: 0xff,
    offset: 0,
};

pub const PCINT1: AvrPinChangeInterrupt = AvrPinChangeInterrupt {
    pcie: 1,
    pcicr: 0x68,
    pcifr: 0x3b,
    pcmsk: 0x6c,
    pin_change_interrupt: 8,
    mask: 0xff,
    offset: 0,
};

pub const PCINT2: AvrPinChangeInterrupt = AvrPinChangeInterrupt {
    pcie: 2,
    pcicr: 0x68,
    pcifr: 0x3b,
    pcmsk: 0x6d,
    pin_change_interrupt: 10,
    mask: 0xff,
    offset: 0,
};

pub const PORT_B_CONFIG: AvrPortConfig = AvrPortConfig {
    pin: 0x23,
    ddr: 0x24,
    port: 0x25,
    pin_change: Some(PCINT0),
    external_interrupts: [None; 8],
};

pub const PORT_C_CONFIG: AvrPortConfig = AvrPortConfig {
    pin: 0x26,
    ddr: 0x27,
    port: 0x28,
    pin_change: Some(PCINT1),
    external_interrupts: [None; 8],
};

pub const PORT_D_CONFIG: AvrPortConfig = AvrPortConfig {
    pin: 0x29,
    ddr: 0x2a,
    port: 0x2b,
    pin_change: Some(PCINT2),
    external_interrupts: [None, None, Some(INT0), Some(INT1), None, None, None, None],
};

/// What a pin is doing, as the circuit sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinState {
    Low,
    High,
    Input,
    InputPullUp,
}

/// How a timer's compare output takes over a pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinOverrideMode {
    /// Released: PORT drives the pin again.
    None,
    /// Taken over, starting from PORT's level.
    Enable,
    Set,
    Clear,
    Toggle,
}

/// EICR's ISC bits.
const LOW_LEVEL: u8 = 0;
const CHANGE: u8 = 1;
const FALLING_EDGE: u8 = 2;
const RISING_EDGE: u8 = 3;

/// One of the external-interrupt registers, by what a write to it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InterruptRegister {
    /// EIFR.
    Flag,
    /// EIMSK.
    Mask,
    /// EICR.
    Other,
}

/// An I/O port. avr8js's `AVRIOPort`.
#[derive(Debug, Clone)]
pub struct AvrIoPort {
    pub config: AvrPortConfig,
    /// Pins that only pull low: an output 1 is left floating.
    pub open_collector: u8,
    /// Set when what the port drives changes — where avr8js calls its
    /// listeners. Cleared by [`AvrIoPort::take_change`].
    pub changed: bool,

    external_ints: [Option<InterruptConfig>; 8],
    pcint: Option<InterruptConfig>,
    pin_value: u8,
    override_mask: u8,
    override_value: u8,
    last_value: u8,
    last_ddr: u8,
    last_pin: u8,
    /// What the port drove when [`AvrIoPort::take_change`] last ran.
    reported_value: u8,
    /// The timer each pin clocks, if any.
    pub(crate) external_clock_listeners: [Option<u8>; 8],
    /// Edges on those pins, for the timers: timer index and new level.
    pub(crate) clock_edges: Vec<(u8, bool)>,

    /// Every call to `timer_override_pin`, for the tests to spy on as avr8js's
    /// spec spies on the method.
    #[cfg(test)]
    pub(crate) override_log: Vec<(u8, PinOverrideMode)>,
}

impl AvrIoPort {
    /// The port, its registers hooked as the `index`th of its owner's ports.
    pub fn new(cpu: &mut Cpu, config: AvrPortConfig, index: u8) -> AvrIoPort {
        let hooks = &mut cpu.write_hooks;
        hooks[config.ddr as usize] = Hook::PortDdr(index);
        hooks[config.port as usize] = Hook::PortPort(index);
        hooks[config.pin as usize] = Hook::PortPin(index);

        // External interrupts
        let external_ints = config.external_interrupts.map(|external| {
            external.map(|external| InterruptConfig {
                address: external.interrupt,
                flag_register: external.eifr,
                flag_mask: 1 << external.index,
                enable_register: external.eimsk,
                enable_mask: 1 << external.index,
                ..InterruptConfig::default()
            })
        });
        for external in config.external_interrupts.iter().flatten() {
            hooks[external.eicr as usize] = Hook::ExternalInterruptControl;
            hooks[external.eimsk as usize] = Hook::ExternalInterruptMask;
            hooks[external.eifr as usize] = Hook::ExternalInterruptFlag;
        }

        // Pin change interrupts
        let pcint = config.pin_change.map(|pin_change| InterruptConfig {
            address: pin_change.pin_change_interrupt,
            flag_register: pin_change.pcifr,
            flag_mask: 1 << pin_change.pcie,
            enable_register: pin_change.pcicr,
            enable_mask: 1 << pin_change.pcie,
            ..InterruptConfig::default()
        });
        if let Some(pin_change) = config.pin_change {
            hooks[pin_change.pcifr as usize] = Hook::PinChangeFlag;
            // avr8js: hooks PCMSK, with the effect this hook has.
            hooks[pin_change.pcicr as usize] = Hook::PinChangeControl;
        }

        AvrIoPort {
            config,
            open_collector: 0,
            changed: false,
            external_ints,
            pcint,
            pin_value: 0,
            override_mask: 0xff,
            override_value: 0,
            last_value: 0,
            last_ddr: 0,
            last_pin: 0,
            reported_value: 0,
            external_clock_listeners: [None; 8],
            clock_edges: Vec::new(),
            #[cfg(test)]
            override_log: Vec::new(),
        }
    }

    /// What the port drives — outputs after any timer override, and for the
    /// inputs, PORT: their pull-ups.
    pub fn value(&self) -> u8 {
        self.last_value
    }

    /// Whether what the port drives has changed since the last call, and if
    /// so, what it drives now and what it drove then. Clears
    /// [`AvrIoPort::changed`].
    pub fn take_change(&mut self) -> Option<(u8, u8)> {
        if !self.changed {
            return None;
        }
        self.changed = false;
        let previous = self.reported_value;
        self.reported_value = self.last_value;
        Some((self.last_value, previous))
    }

    /// The state of pin `index`, 0 to 7: low or high if it is an output, an
    /// input with or without its pull-up otherwise.
    pub fn pin_state(&self, cpu: &Cpu, index: u8) -> PinState {
        let ddr = cpu.data[self.config.ddr as usize];
        let port = cpu.data[self.config.port as usize];
        let bit_mask = 1 << index;
        let open_state = if port & bit_mask != 0 {
            PinState::InputPullUp
        } else {
            PinState::Input
        };
        let high_value = if self.open_collector & bit_mask != 0 {
            open_state
        } else {
            PinState::High
        };
        if ddr & bit_mask != 0 {
            if self.last_value & bit_mask != 0 {
                high_value
            } else {
                PinState::Low
            }
        } else {
            open_state
        }
    }

    /// Sets what pin `index` reads when it is an input: the level the circuit
    /// puts on it.
    pub fn set_pin(&mut self, cpu: &mut Cpu, index: u8, value: bool) {
        let bit_mask = 1 << index;
        self.pin_value &= !bit_mask;
        if value {
            self.pin_value |= bit_mask;
        }
        let ddr = cpu.data[self.config.ddr as usize];
        self.update_pin_register(cpu, ddr);
    }

    /// A timer's compare output taking pin `pin` over, or letting it go.
    pub fn timer_override_pin(&mut self, cpu: &mut Cpu, pin: u8, mode: PinOverrideMode) {
        #[cfg(test)]
        self.override_log.push((pin, mode));

        let pin_mask = 1 << pin;
        if mode == PinOverrideMode::None {
            self.override_mask |= pin_mask;
            self.override_value &= !pin_mask;
        } else {
            self.override_mask &= !pin_mask;
            match mode {
                PinOverrideMode::Enable => {
                    self.override_value &= !pin_mask;
                    self.override_value |= cpu.data[self.config.port as usize] & pin_mask;
                }
                PinOverrideMode::Set => self.override_value |= pin_mask,
                PinOverrideMode::Clear => self.override_value &= !pin_mask,
                PinOverrideMode::Toggle => self.override_value ^= pin_mask,
                PinOverrideMode::None => unreachable!(),
            }
        }
        let ddr = cpu.data[self.config.ddr as usize];
        self.write_gpio(cpu.data[self.config.port as usize], ddr);
        self.update_pin_register(cpu, ddr);
    }

    /// A write to DDR.
    pub(crate) fn write_ddr(&mut self, cpu: &mut Cpu, value: u8) {
        let port_value = cpu.data[self.config.port as usize];
        cpu.data[self.config.ddr as usize] = value;
        self.write_gpio(port_value, value);
        self.update_pin_register(cpu, value);
    }

    /// A write to PORT.
    pub(crate) fn write_port(&mut self, cpu: &mut Cpu, value: u8) {
        let ddr_mask = cpu.data[self.config.ddr as usize];
        cpu.data[self.config.port as usize] = value;
        self.write_gpio(value, ddr_mask);
        self.update_pin_register(cpu, ddr_mask);
    }

    /// A write to PIN: each 1 toggles PORT — of the bits `mask` says the
    /// instruction meant, so `SBI` toggles one pin and not all those set.
    pub(crate) fn write_pin(&mut self, cpu: &mut Cpu, value: u8, mask: u8) {
        let old_port_value = cpu.data[self.config.port as usize];
        let ddr_mask = cpu.data[self.config.ddr as usize];
        let port_value = old_port_value ^ (value & mask);
        cpu.data[self.config.port as usize] = port_value;
        self.write_gpio(port_value, ddr_mask);
        self.update_pin_register(cpu, ddr_mask);
    }

    fn update_pin_register(&mut self, cpu: &mut Cpu, ddr: u8) {
        let new_pin = (self.pin_value & !ddr) | (self.last_value & ddr);
        cpu.data[self.config.pin as usize] = new_pin;
        if self.last_pin != new_pin {
            for index in 0..8 {
                if (new_pin & (1 << index)) != (self.last_pin & (1 << index)) {
                    let value = new_pin & (1 << index) != 0;
                    self.toggle_interrupt(cpu, index, value);
                    if let Some(timer) = self.external_clock_listeners[index as usize] {
                        self.clock_edges.push((timer, value));
                    }
                }
            }
            self.last_pin = new_pin;
        }
    }

    fn toggle_interrupt(&mut self, cpu: &mut Cpu, pin: u8, rising_edge: bool) {
        let config = self.config;
        if let (Some(external_config), Some(external)) = (
            config.external_interrupts[pin as usize],
            self.external_ints[pin as usize].as_mut(),
        ) && cpu.data[external_config.eimsk as usize] & (1 << external_config.index) != 0
        {
            let configuration =
                (cpu.data[external_config.eicr as usize] >> external_config.isc_offset) & 0x3;
            external.constant = false;
            let generate_interrupt = match configuration {
                LOW_LEVEL => {
                    external.constant = true;
                    !rising_edge
                }
                CHANGE => true,
                FALLING_EDGE => !rising_edge,
                RISING_EDGE => rising_edge,
                _ => unreachable!("two bits"),
            };
            let external = *external;
            if generate_interrupt {
                cpu.set_interrupt_flag(&external);
            } else if external.constant {
                cpu.clear_interrupt(&external, true);
            }
        }

        if let (Some(pin_change), Some(pcint)) = (config.pin_change, self.pcint)
            && pin_change.mask & (1 << pin) != 0
            && cpu.data[pin_change.pcmsk as usize] & (1 << (pin + pin_change.offset)) != 0
        {
            cpu.set_interrupt_flag(&pcint);
        }
    }

    /// Queues a low-level INT*n* whose pin is already low.
    fn check_external_interrupts(&self, cpu: &mut Cpu) {
        for pin in 0..8 {
            let Some(external) = self.config.external_interrupts[pin] else {
                continue;
            };
            let pin_value = self.last_pin & (1 << pin) != 0;
            if cpu.data[external.eimsk as usize] & (1 << external.index) == 0 || pin_value {
                continue;
            }
            let configuration = (cpu.data[external.eicr as usize] >> external.isc_offset) & 0x3;
            if configuration == LOW_LEVEL {
                cpu.queue_interrupt(&InterruptConfig {
                    address: external.interrupt,
                    flag_register: external.eifr,
                    flag_mask: 1 << external.index,
                    enable_register: external.eimsk,
                    enable_mask: 1 << external.index,
                    constant: true,
                    inverse_flag: false,
                });
            }
        }
    }

    fn write_gpio(&mut self, value: u8, ddr: u8) {
        let new_value =
            (((value & self.override_mask) | self.override_value) & ddr) | (value & !ddr);
        let prev_value = self.last_value;
        if new_value != prev_value || ddr != self.last_ddr {
            self.last_value = new_value;
            self.last_ddr = ddr;
            self.changed = true;
        }
    }
}

/// A write to EICR, EIMSK or EIFR, which every port's external interrupts
/// share. avr8js's `attachInterruptHook`.
pub(crate) fn write_interrupt_register(
    ports: &mut [AvrIoPort],
    cpu: &mut Cpu,
    register_type: InterruptRegister,
    register: u16,
    value: u8,
) {
    if register_type != InterruptRegister::Flag {
        cpu.data[register as usize] = value;
    }
    for gpio in ports.iter() {
        for external in gpio.external_ints.iter().flatten() {
            if register_type == InterruptRegister::Mask {
                cpu.update_interrupt_enable(external, value);
            }
            if !external.constant && register_type == InterruptRegister::Flag {
                cpu.clear_interrupt_by_flag(external, value);
            }
        }
        gpio.check_external_interrupts(cpu);
    }
}

/// A write to PCIFR, which every port's pin-change interrupt shares.
pub(crate) fn write_pin_change_flag(ports: &[AvrIoPort], cpu: &mut Cpu, value: u8) {
    for gpio in ports {
        if let Some(pcint) = gpio.pcint {
            cpu.clear_interrupt_by_flag(&pcint, value);
        }
    }
}

/// A write to PCICR, which enables each port's pin-change interrupt.
pub(crate) fn write_pin_change_control(
    ports: &[AvrIoPort],
    cpu: &mut Cpu,
    register: u16,
    value: u8,
) {
    cpu.data[register as usize] = value;
    for gpio in ports {
        if let Some(pcint) = gpio.pcint {
            cpu.update_interrupt_enable(&pcint, value);
        }
    }
}
