// SPDX-License-Identifier: MIT OR Apache-2.0

//! The ATmega328P an Arduino Uno is built on, emulated (ADR 0017).
//!
//! A port of avr8js 0.21.1, Wokwi's AVR emulator, to Rust, so that the
//! program runs inside the core with the circuit and seam 4 still holds: the
//! core alone decides what simulated time is worth. Each ported file says
//! which of avr8js's it comes from and carries its notice; NOTICE.md beside
//! them is avr8js's licence in full.
//!
//! - [`cpu`]: memories, clock events, interrupts — and how peripherals hook
//!   the CPU without closures;
//! - [`instruction`]: the instruction set;
//! - [`gpio`]: ports B, C and D, external and pin-change interrupts;
//! - [`timer`]: timer0, timer1, timer2;
//! - [`adc`]: the analog-to-digital converter;
//! - [`usart`]: the serial port;
//! - [`hex`]: an Intel HEX loader, which avr8js leaves to its embedder.
//!
//! [`Peripherals`] owns the ports, timers, ADC and USART and is what the CPU calls back
//! into; [`Avr`] is the two together, one instruction at a time. Nothing here
//! knows about the circuit: wiring a pin to a net is the engine's business.

pub mod adc;
pub mod cpu;
pub mod gpio;
pub mod hex;
pub mod instruction;
pub mod timer;
pub mod usart;

use adc::{ADC_CONFIG, AvrAdc};
use cpu::{ClockEvent, Cpu, Hook, Hooks};
use gpio::{
    AvrIoPort, AvrPortConfig, InterruptRegister, PORT_B_CONFIG, PORT_C_CONFIG, PORT_D_CONFIG,
};
use instruction::avr_instruction;
use timer::{AvrTimer, AvrTimerConfig, TIMER0_CONFIG, TIMER1_CONFIG, TIMER2_CONFIG};
use usart::{AvrUsart, USART0_CONFIG};

/// The ATmega328P's flash, in 16-bit words: 32 KB.
pub const ATMEGA328P_FLASH_WORDS: usize = 0x4000;
/// Its SRAM, after the 256 bytes of registers and I/O.
pub const ATMEGA328P_SRAM_BYTES: usize = 2048;
/// The Uno's crystal.
pub const ATMEGA328P_CLOCK_HZ: u64 = 16_000_000;

/// The ATmega328P's ports, in the order [`Peripherals::atmega328p`] puts them.
pub const PORT_B: usize = 0;
pub const PORT_C: usize = 1;
pub const PORT_D: usize = 2;

/// An AVR's ports, timers, ADC and USART, as the CPU's [`Hooks`].
///
/// One owner for all of them because they reach each other: a timer's
/// compare output drives a port's pin, and a port's pin can clock a timer.
#[derive(Debug, Clone)]
pub struct Peripherals {
    pub ports: Vec<AvrIoPort>,
    pub timers: Vec<AvrTimer>,
    /// Its converter, where the part has one and the caller asked for it.
    pub adc: Option<AvrAdc>,
    /// Its serial port, likewise.
    pub usart: Option<AvrUsart>,
}

impl Peripherals {
    /// Ports and timers on `cpu`, their registers hooked.
    pub fn new(cpu: &mut Cpu, ports: &[AvrPortConfig], timers: &[AvrTimerConfig]) -> Peripherals {
        let port_addresses: Vec<u16> = ports.iter().map(|config| config.port).collect();
        Peripherals {
            ports: ports
                .iter()
                .enumerate()
                .map(|(index, &config)| AvrIoPort::new(cpu, config, index as u8))
                .collect(),
            timers: timers
                .iter()
                .enumerate()
                .map(|(index, &config)| AvrTimer::new(cpu, config, index as u8, &port_addresses))
                .collect(),
            adc: None,
            usart: None,
        }
    }

    /// The ATmega328P's: ports B, C and D, timers 0, 1 and 2, the ADC and
    /// USART0.
    pub fn atmega328p(cpu: &mut Cpu) -> Peripherals {
        let mut io = Peripherals::new(
            cpu,
            &[PORT_B_CONFIG, PORT_C_CONFIG, PORT_D_CONFIG],
            &[TIMER0_CONFIG, TIMER1_CONFIG, TIMER2_CONFIG],
        );
        io.adc = Some(AvrAdc::new(cpu, ADC_CONFIG));
        io.usart = Some(AvrUsart::new(cpu, USART0_CONFIG, ATMEGA328P_CLOCK_HZ));
        io
    }

    /// Whether any port's outputs changed since they were last taken, the
    /// USART took a pin over or gave one back, or the ADC's sample-and-hold
    /// moved: the test to make after each instruction.
    #[inline]
    pub fn outputs_changed(&self) -> bool {
        self.ports.iter().any(|port| port.changed)
            || self.usart.as_ref().is_some_and(|usart| usart.pins_changed)
            || self.adc.as_ref().is_some_and(|adc| adc.selection_changed)
    }

    /// Puts a level on an input pin, as the circuit drives it. Raises
    /// whatever interrupts and timer counts the edge causes.
    pub fn set_pin(&mut self, cpu: &mut Cpu, port: usize, pin: u8, value: bool) {
        self.ports[port].set_pin(cpu, pin, value);
        self.dispatch_clock_edges(cpu);
    }

    /// A timer's compare output taking a pin over. For the tests; the timers
    /// do it themselves.
    pub fn timer_override_pin(
        &mut self,
        cpu: &mut Cpu,
        port: usize,
        pin: u8,
        mode: gpio::PinOverrideMode,
    ) {
        self.ports[port].timer_override_pin(cpu, pin, mode);
        self.dispatch_clock_edges(cpu);
    }

    /// Hands each edge a port queued on a T*n* pin to the timer it clocks.
    ///
    /// Almost always nothing to do. The edges are delivered after the port
    /// that saw them is finished, and before the CPU runs again; a timer
    /// counting one may drive a pin that queues another.
    fn dispatch_clock_edges(&mut self, cpu: &mut Cpu) {
        while let Some(port) = self
            .ports
            .iter()
            .position(|port| !port.clock_edges.is_empty())
        {
            let (timer, value) = self.ports[port].clock_edges.remove(0);
            self.timers[timer as usize].external_clock_callback(cpu, &mut self.ports, value);
        }
    }
}

impl Hooks for Peripherals {
    fn read(&mut self, cpu: &mut Cpu, hook: Hook, addr: u16) -> u8 {
        let value = match hook {
            Hook::TimerCounter(timer) => {
                self.timers[timer as usize].read_tcnt(cpu, &mut self.ports, addr)
            }
            Hook::UsartData => match &mut self.usart {
                Some(usart) => usart.read_data(cpu),
                None => cpu.data[addr as usize],
            },
            _ => cpu.data[addr as usize],
        };
        self.dispatch_clock_edges(cpu);
        value
    }

    fn write(
        &mut self,
        cpu: &mut Cpu,
        hook: Hook,
        value: u8,
        old_value: u8,
        addr: u16,
        mask: u8,
    ) -> bool {
        let ports = &mut self.ports;
        let stored = match hook {
            Hook::None => false,
            Hook::PortDdr(port) => {
                ports[port as usize].write_ddr(cpu, value);
                true
            }
            Hook::PortPort(port) => {
                ports[port as usize].write_port(cpu, value);
                true
            }
            Hook::PortPin(port) => {
                ports[port as usize].write_pin(cpu, value, mask);
                true
            }
            Hook::PinChangeFlag => {
                gpio::write_pin_change_flag(ports, cpu, value);
                true
            }
            Hook::PinChangeControl => {
                gpio::write_pin_change_control(ports, cpu, addr, value);
                true
            }
            Hook::ExternalInterruptControl => {
                gpio::write_interrupt_register(ports, cpu, InterruptRegister::Other, addr, value);
                true
            }
            Hook::ExternalInterruptMask => {
                gpio::write_interrupt_register(ports, cpu, InterruptRegister::Mask, addr, value);
                true
            }
            Hook::ExternalInterruptFlag => {
                gpio::write_interrupt_register(ports, cpu, InterruptRegister::Flag, addr, value);
                true
            }
            Hook::TimerCounter(timer)
            | Hook::TimerTemp(timer)
            | Hook::TimerOcrA(timer)
            | Hook::TimerOcrB(timer)
            | Hook::TimerOcrC(timer)
            | Hook::TimerOcrHigh(timer)
            | Hook::TimerIcr(timer)
            | Hook::TimerControlA(timer)
            | Hook::TimerControlB(timer)
            | Hook::TimerControlC(timer)
            | Hook::TimerFlags(timer)
            | Hook::TimerMask(timer) => {
                self.timers[timer as usize].write(cpu, ports, hook, value, addr)
            }
            Hook::AdcControl => match &mut self.adc {
                Some(adc) => adc.write_control(cpu, value, old_value),
                None => false,
            },
            Hook::AdcMux => match &mut self.adc {
                Some(adc) => adc.write_mux(cpu, value),
                None => false,
            },
            Hook::UsartStatus
            | Hook::UsartControlB
            | Hook::UsartControlC
            | Hook::UsartData
            | Hook::UsartBaud => match &mut self.usart {
                Some(usart) => usart.write(cpu, hook, value, old_value, addr),
                None => false,
            },
        };
        self.dispatch_clock_edges(cpu);
        stored
    }

    fn clock_event(&mut self, cpu: &mut Cpu, event: ClockEvent) {
        match event {
            ClockEvent::TimerCount(timer) => {
                self.timers[timer as usize].count(cpu, &mut self.ports, true, false);
                self.dispatch_clock_edges(cpu);
            }
            ClockEvent::AdcComplete => {
                if let Some(adc) = &mut self.adc {
                    adc.complete(cpu);
                }
            }
            ClockEvent::UsartTransmitted => {
                if let Some(usart) = &mut self.usart {
                    usart.transmitted_event(cpu);
                }
            }
            ClockEvent::UsartReceive(value) => {
                if let Some(usart) = &mut self.usart {
                    usart.received_event(cpu, value);
                }
            }
            ClockEvent::External(_) => {}
        }
    }
}

/// A CPU and its peripherals.
#[derive(Debug, Clone)]
pub struct Avr {
    pub cpu: Cpu,
    pub io: Peripherals,
}

impl Avr {
    /// An ATmega328P running `program`, padded or cut to its 32 KB of flash.
    pub fn atmega328p(mut program: Vec<u16>) -> Avr {
        program.resize(ATMEGA328P_FLASH_WORDS, 0);
        let mut cpu = Cpu::new(program, ATMEGA328P_SRAM_BYTES);
        let io = Peripherals::atmega328p(&mut cpu);
        Avr { cpu, io }
    }

    /// Executes one instruction, then whatever clock event and interrupt
    /// are due — avr8js's `avrInstruction(cpu); cpu.tick()`.
    #[inline]
    pub fn step(&mut self) {
        avr_instruction(&mut self.cpu, &mut self.io);
        self.cpu.tick(&mut self.io);
    }

    /// Steps until the cycle count reaches `cycles` or an output changes,
    /// whichever comes first; true if an output changed. The instruction that
    /// changed it has run, and the cycle count is where it left it.
    ///
    /// A change stays flagged until its port's [`AvrIoPort::take_change`]
    /// takes it, so take them all before calling again, or this returns after
    /// one instruction.
    pub fn run_until(&mut self, cycles: u64) -> bool {
        while self.cpu.cycles < cycles {
            self.step();
            if self.io.outputs_changed() {
                return true;
            }
        }
        false
    }

    pub fn read_data(&mut self, addr: u16) -> u8 {
        self.cpu.read_data(&mut self.io, addr)
    }

    pub fn write_data(&mut self, addr: u16, value: u8) {
        self.cpu.write_data(&mut self.io, addr, value, 0xff);
    }

    pub fn tick(&mut self) {
        self.cpu.tick(&mut self.io);
    }
}

#[cfg(test)]
mod tests;
