// SPDX-License-Identifier: MIT OR Apache-2.0
// Ported from avr8js, src/cpu/cpu.ts and src/cpu/interrupt.ts.
// Copyright (c) 2019-2025 Uri Shaked, MIT licence (see NOTICE.md beside this file).

//! The AVR's CPU: its memories, its clock, its interrupts (ADR 0017).
//!
//! A port of avr8js's `CPU` class and of `avrInterrupt`, kept in the same shape
//! and under the same names so that a fix upstream can be found here and
//! brought across. What had to change is how the peripherals reach it.
//!
//! **Hooks.** In avr8js a peripheral claims a register by putting a closure in
//! `cpu.readHooks[addr]` or `cpu.writeHooks[addr]`, and schedules work by
//! handing the CPU a callback to run after so many cycles. Closures that
//! capture the peripheral and also the CPU they are stored in are exactly what
//! Rust will not let two owners share without `Rc<RefCell<…>>` everywhere —
//! and a borrow checked at run time on every `OUT` is a cost paid sixteen
//! million times a simulated second. So the closures become data:
//!
//! - a hooked register holds a [`Hook`], a small `Copy` tag naming which
//!   peripheral's which register it is. Most addresses hold [`Hook::None`] and
//!   cost one load and one compare;
//! - a scheduled callback is a [`ClockEvent`], a tag of what to do, and the
//!   queue compares tags where avr8js compared function identity;
//! - whoever owns the peripherals implements [`Hooks`] and is passed alongside
//!   the CPU to [`Cpu::read_data`], [`Cpu::write_data`], [`Cpu::tick`] and the
//!   instruction. A `match` on the tag reaches the peripheral's method, which
//!   gets `&mut Cpu` as its argument where avr8js used `this.cpu`.
//!
//! The CPU owns nothing but its own state, the peripherals own theirs, and
//! neither holds a reference to the other: the two are borrowed side by side
//! for one call at a time. Everything is monomorphised over the [`Hooks`]
//! type, so the dispatch is a jump table and not a virtual call. The tests of
//! the instruction set pass `()`, which hooks nothing.
//!
//! **Memory.** Data memory is 64 KB whatever the part, so that every 16-bit
//! address the program can form indexes it without a bounds check or a panic;
//! avr8js's typed array silently drops a write past its end and reads
//! `undefined` back as 0, and a program with a runaway pointer here instead
//! reads back what it wrote. The part's own size still sets where the stack
//! starts. The clock-event queue is a vector kept sorted latest-first, popped
//! from the end; it is rarely more than a handful long.

use std::fmt;

/// The register file and I/O registers, below SRAM.
pub const REGISTER_SPACE: usize = 0x100;

/// Enough for an ATmega2560.
pub const MAX_INTERRUPTS: usize = 128;

/// Addresses a peripheral may hook: the register file, I/O and extended I/O
/// up to the end of the ATmega2560's, the largest part avr8js describes.
pub const HOOK_SPACE: usize = 0x200;

/// Data memory as the CPU addresses it: all of a 16-bit address space.
pub const DATA_SPACE: usize = 0x10000;

/// The stack pointer, low byte first.
pub const SP: u16 = 93;
/// The status register: `I T H S V N Z C`, bit 7 to bit 0.
pub const SREG: u16 = 95;

/// Which peripheral register an address belongs to, where avr8js stored a
/// closure. Each peripheral adds its variants here as it is ported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Hook {
    #[default]
    None,

    // GPIO (gpio.rs): the port's index among the owner's ports.
    PortDdr(u8),
    PortPort(u8),
    PortPin(u8),
    /// PCIFR, shared by every port.
    PinChangeFlag,
    /// PCICR, shared by every port.
    PinChangeControl,
    /// EICRA (or EICRB).
    ExternalInterruptControl,
    /// EIMSK.
    ExternalInterruptMask,
    /// EIFR.
    ExternalInterruptFlag,

    // Timers (timer.rs): the timer's index among the owner's timers.
    /// TCNT's low byte, read and written.
    TimerCounter(u8),
    /// The high byte of TCNT or ICR, which goes to the shared TEMP register.
    TimerTemp(u8),
    TimerOcrA(u8),
    TimerOcrB(u8),
    TimerOcrC(u8),
    /// The high byte of an OCR, which is masked to the top value.
    TimerOcrHigh(u8),
    TimerIcr(u8),
    TimerControlA(u8),
    TimerControlB(u8),
    TimerControlC(u8),
    TimerFlags(u8),
    TimerMask(u8),
    /// The ADC's ADCSRA.
    AdcControl,
    /// Its ADMUX.
    AdcMux,
    /// The USART's UCSRA, UCSRB and UCSRC.
    UsartStatus,
    UsartControlB,
    UsartControlC,
    /// Its UDR, read and written.
    UsartData,
    /// Its UBRRH and UBRRL.
    UsartBaud,
}

/// Something to do once a number of cycles has passed, where avr8js stored a
/// callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockEvent {
    /// A timer's `count`, by the timer's index.
    TimerCount(u8),
    /// The ADC's conversion ending.
    AdcComplete,
    /// The USART's byte gone.
    UsartTransmitted,
    /// A byte arriving at the USART.
    UsartReceive(u16),
    /// Anything the owner of the peripherals wants to be woken for; the CPU
    /// hands it back through [`Hooks::clock_event`] untouched.
    External(u32),
}

/// An interrupt: its vector, and the registers and bits that enable and flag
/// it. `AVRInterruptConfig` in avr8js.
///
/// avr8js shares one mutable object between a peripheral and the pending
/// queue; here the queue holds a copy taken when the interrupt was queued.
/// The one field that changes after creation, `constant`, is only ever
/// changed just before the interrupt is queued again, so the two agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InterruptConfig {
    /// The vector, in words: the index into the pending table.
    pub address: u8,
    pub enable_register: u16,
    pub enable_mask: u8,
    pub flag_register: u16,
    pub flag_mask: u8,
    /// Stays pending after it is taken: a level-triggered interrupt.
    pub constant: bool,
    /// The flag is set by clearing its bit.
    pub inverse_flag: bool,
}

/// What a peripheral does when the CPU touches its registers or its time
/// comes. Implemented by whoever owns the peripherals, and by `()` for a CPU
/// with none.
pub trait Hooks {
    /// A read of a register whose read hook is `hook`.
    fn read(&mut self, cpu: &mut Cpu, hook: Hook, addr: u16) -> u8;

    /// A write to a register whose write hook is `hook`. Returns `true` when
    /// the hook stored the value itself, as avr8js's hooks do; otherwise the
    /// CPU stores `value` as written.
    fn write(
        &mut self,
        cpu: &mut Cpu,
        hook: Hook,
        value: u8,
        old_value: u8,
        addr: u16,
        mask: u8,
    ) -> bool;

    /// A scheduled event has come due.
    fn clock_event(&mut self, cpu: &mut Cpu, event: ClockEvent);

    /// `WDR` was executed. avr8js's `onWatchdogReset`.
    fn watchdog_reset(&mut self, _cpu: &mut Cpu) {}
}

/// No peripherals at all.
impl Hooks for () {
    fn read(&mut self, cpu: &mut Cpu, _hook: Hook, addr: u16) -> u8 {
        cpu.data[addr as usize]
    }

    fn write(&mut self, _: &mut Cpu, _: Hook, _: u8, _: u8, _: u16, _: u8) -> bool {
        false
    }

    fn clock_event(&mut self, _cpu: &mut Cpu, _event: ClockEvent) {}
}

#[derive(Debug, Clone, Copy)]
struct ClockEventEntry {
    cycles: u64,
    event: ClockEvent,
}

/// An AVR CPU.
#[derive(Clone)]
pub struct Cpu {
    /// Registers, I/O and SRAM, at their data-space addresses.
    pub data: Box<[u8; DATA_SPACE]>,
    /// Flash, in 16-bit words.
    pub prog_mem: Vec<u16>,
    pub read_hooks: Box<[Hook; HOOK_SPACE]>,
    pub write_hooks: Box<[Hook; HOOK_SPACE]>,
    pending_interrupts: Box<[Option<InterruptConfig>; MAX_INTERRUPTS]>,
    /// Latest first, so the next one due is at the end.
    clock_events: Vec<ClockEventEntry>,

    /// Whether the program counter has 22 bits (the default is 16).
    pub pc22_bits: bool,

    /// Program counter, in words.
    pub pc: u32,
    /// Clock cycles since reset.
    pub cycles: u64,

    pub next_interrupt: i16,
    pub max_interrupt: i16,

    /// Registers, I/O and the part's SRAM: where the stack starts.
    data_len: usize,
}

impl Cpu {
    /// A CPU running `prog_mem`, with `sram_bytes` of SRAM after the 256 bytes
    /// of registers. avr8js's default is 8192; the ATmega328P has 2048.
    pub fn new(prog_mem: Vec<u16>, sram_bytes: usize) -> Cpu {
        let pc22_bits = prog_mem.len() * 2 > 0x20000;
        let mut cpu = Cpu {
            data: zeroed(),
            prog_mem,
            read_hooks: Box::new([Hook::None; HOOK_SPACE]),
            write_hooks: Box::new([Hook::None; HOOK_SPACE]),
            pending_interrupts: Box::new([None; MAX_INTERRUPTS]),
            clock_events: Vec::with_capacity(16),
            pc22_bits,
            pc: 0,
            cycles: 0,
            next_interrupt: -1,
            max_interrupt: 0,
            data_len: (sram_bytes + REGISTER_SPACE).min(DATA_SPACE),
        };
        cpu.reset();
        cpu
    }

    pub fn reset(&mut self) {
        self.set_sp((self.data_len - 1) as u16);
        self.pc = 0;
        self.pending_interrupts.fill(None);
        self.next_interrupt = -1;
        self.clock_events.clear();
    }

    /// Reads data memory as an instruction does, through any read hook.
    #[inline]
    pub fn read_data<H: Hooks + ?Sized>(&mut self, hooks: &mut H, addr: u16) -> u8 {
        let index = addr as usize;
        if (32..HOOK_SPACE).contains(&index) {
            let hook = self.read_hooks[index];
            if hook != Hook::None {
                return hooks.read(self, hook, addr);
            }
        }
        self.data[index]
    }

    /// Writes data memory as an instruction does, through any write hook.
    /// `mask` is the bits the instruction meant to change: `SBI` and `CBI`
    /// write one, everything else all eight.
    #[inline]
    pub fn write_data<H: Hooks + ?Sized>(&mut self, hooks: &mut H, addr: u16, value: u8, mask: u8) {
        let index = addr as usize;
        if index < HOOK_SPACE {
            let hook = self.write_hooks[index];
            if hook != Hook::None {
                let old_value = self.data[index];
                if hooks.write(self, hook, value, old_value, addr, mask) {
                    return;
                }
            }
        }
        self.data[index] = value;
    }

    /// A 16-bit register pair or I/O pair, low byte first.
    #[inline]
    pub fn data16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.data[addr as usize], self.data[addr as usize + 1]])
    }

    #[inline]
    pub fn set_data16(&mut self, addr: u16, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.data[addr as usize] = low;
        self.data[addr as usize + 1] = high;
    }

    /// A word of flash; 0 past its end, as avr8js's typed array reads.
    #[inline]
    pub fn prog_word(&self, addr: u32) -> u16 {
        self.prog_mem.get(addr as usize).copied().unwrap_or(0)
    }

    /// A byte of flash, as `LPM` addresses it: little-endian within each word.
    #[inline]
    pub fn prog_byte(&self, addr: u32) -> u8 {
        let word = self.prog_word(addr >> 1);
        (word >> ((addr & 1) * 8)) as u8
    }

    /// Writes a byte of flash; past its end, nothing. avr8js's `progBytes`.
    pub fn set_prog_byte(&mut self, addr: u32, value: u8) {
        if let Some(word) = self.prog_mem.get_mut((addr >> 1) as usize) {
            let shift = (addr & 1) * 8;
            *word = (*word & !(0xff << shift)) | ((value as u16) << shift);
        }
    }

    /// Flash in bytes.
    pub fn prog_bytes_len(&self) -> usize {
        self.prog_mem.len() * 2
    }

    #[inline]
    pub fn sp(&self) -> u16 {
        self.data16(SP)
    }

    #[inline]
    pub fn set_sp(&mut self, value: u16) {
        self.set_data16(SP, value);
    }

    #[inline]
    pub fn sreg(&self) -> u8 {
        self.data[SREG as usize]
    }

    #[inline]
    pub fn interrupts_enabled(&self) -> bool {
        self.sreg() & 0x80 != 0
    }

    pub fn set_interrupt_flag(&mut self, interrupt: &InterruptConfig) {
        let flag = interrupt.flag_register as usize;
        if interrupt.inverse_flag {
            self.data[flag] &= !interrupt.flag_mask;
        } else {
            self.data[flag] |= interrupt.flag_mask;
        }
        if self.data[interrupt.enable_register as usize] & interrupt.enable_mask != 0 {
            self.queue_interrupt(interrupt);
        }
    }

    pub fn update_interrupt_enable(&mut self, interrupt: &InterruptConfig, register_value: u8) {
        if register_value & interrupt.enable_mask != 0 {
            let bit_set = self.data[interrupt.flag_register as usize] & interrupt.flag_mask != 0;
            if bit_set != interrupt.inverse_flag {
                self.queue_interrupt(interrupt);
            }
        } else {
            self.clear_interrupt(interrupt, false);
        }
    }

    pub fn queue_interrupt(&mut self, interrupt: &InterruptConfig) {
        let address = interrupt.address as i16;
        self.pending_interrupts[interrupt.address as usize] = Some(*interrupt);
        if self.next_interrupt == -1 || self.next_interrupt > address {
            self.next_interrupt = address;
        }
        if address > self.max_interrupt {
            self.max_interrupt = address;
        }
    }

    pub fn clear_interrupt(&mut self, interrupt: &InterruptConfig, clear_flag: bool) {
        if clear_flag {
            self.data[interrupt.flag_register as usize] &= !interrupt.flag_mask;
        }
        let address = interrupt.address as usize;
        if self.pending_interrupts[address].is_none() {
            return;
        }
        self.pending_interrupts[address] = None;
        if self.next_interrupt == address as i16 {
            self.next_interrupt = -1;
            for i in address + 1..=self.max_interrupt as usize {
                if self.pending_interrupts[i].is_some() {
                    self.next_interrupt = i as i16;
                    break;
                }
            }
        }
    }

    pub fn clear_interrupt_by_flag(&mut self, interrupt: &InterruptConfig, register_value: u8) {
        if register_value & interrupt.flag_mask != 0 {
            self.data[interrupt.flag_register as usize] &= !interrupt.flag_mask;
            self.clear_interrupt(interrupt, true);
        }
    }

    /// Schedules `event` for `cycles` from now — at least one. Returns the
    /// event, as avr8js returns the callback, so it can be cleared later.
    pub fn add_clock_event(&mut self, event: ClockEvent, cycles: u64) -> ClockEvent {
        let cycles = self.cycles + cycles.max(1);
        // Before every event due at the same cycle or later, as avr8js's list
        // inserts: in this latest-first vector, after all of them.
        let at = self
            .clock_events
            .partition_point(|entry| entry.cycles >= cycles);
        self.clock_events
            .insert(at, ClockEventEntry { cycles, event });
        event
    }

    /// Moves a scheduled event to `cycles` from now. False, and nothing
    /// scheduled, if it was not scheduled.
    pub fn update_clock_event(&mut self, event: ClockEvent, cycles: u64) -> bool {
        if self.clear_clock_event(event) {
            self.add_clock_event(event, cycles);
            return true;
        }
        false
    }

    /// Unschedules the soonest instance of `event`. False if there was none.
    pub fn clear_clock_event(&mut self, event: ClockEvent) -> bool {
        match self
            .clock_events
            .iter()
            .rposition(|entry| entry.event == event)
        {
            Some(at) => {
                self.clock_events.remove(at);
                true
            }
            None => false,
        }
    }

    /// Runs what is due after an instruction: at most one clock event, then
    /// at most one interrupt.
    #[inline]
    pub fn tick<H: Hooks + ?Sized>(&mut self, hooks: &mut H) {
        if let Some(next) = self.clock_events.last()
            && next.cycles <= self.cycles
        {
            let event = next.event;
            self.clock_events.pop();
            hooks.clock_event(self, event);
        }

        // `next_interrupt` always names a pending one; upstream would throw
        // if it did not, and a panic here would take the app down with it.
        if self.interrupts_enabled()
            && self.next_interrupt >= 0
            && let Some(interrupt) = self.pending_interrupts[self.next_interrupt as usize]
        {
            avr_interrupt(self, interrupt.address);
            if !interrupt.constant {
                self.clear_interrupt(&interrupt, true);
            }
        }
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cpu")
            .field("pc", &self.pc)
            .field("cycles", &self.cycles)
            .field("sp", &self.sp())
            .field("sreg", &self.sreg())
            .finish_non_exhaustive()
    }
}

/// Takes an interrupt: pushes the return address, clears `I`, jumps to the
/// vector. avr8js's `avrInterrupt`.
pub fn avr_interrupt(cpu: &mut Cpu, addr: u8) {
    let sp = cpu.sp();
    let pc = cpu.pc;
    cpu.data[sp as usize] = pc as u8;
    cpu.data[sp.wrapping_sub(1) as usize] = (pc >> 8) as u8;
    if cpu.pc22_bits {
        cpu.data[sp.wrapping_sub(2) as usize] = (pc >> 16) as u8;
    }
    cpu.set_sp(sp.wrapping_sub(if cpu.pc22_bits { 3 } else { 2 }));
    cpu.data[SREG as usize] &= 0x7f; // clear global interrupt flag
    cpu.cycles += 2;
    cpu.pc = addr as u32;
}

/// 64 KB of zeroes on the heap, without building them on the stack first.
fn zeroed() -> Box<[u8; DATA_SPACE]> {
    vec![0; DATA_SPACE]
        .into_boxed_slice()
        .try_into()
        .expect("the vector has DATA_SPACE bytes")
}

#[cfg(test)]
mod tests {
    //! cpu.spec.ts and interrupt.spec.ts.

    use super::*;

    /// Records each event as it fires: what it was, and the cycle it fired on.
    #[derive(Default)]
    struct Recorder {
        events: Vec<(u32, u64)>,
    }

    impl Hooks for Recorder {
        fn read(&mut self, cpu: &mut Cpu, _: Hook, addr: u16) -> u8 {
            cpu.data[addr as usize]
        }
        fn write(&mut self, _: &mut Cpu, _: Hook, _: u8, _: u8, _: u16, _: u8) -> bool {
            false
        }
        fn clock_event(&mut self, cpu: &mut Cpu, event: ClockEvent) {
            if let ClockEvent::External(id) = event {
                self.events.push((id, cpu.cycles));
            }
        }
    }

    fn run(cpu: &mut Cpu, recorder: &mut Recorder, cycles: u64) {
        for _ in 0..cycles {
            cpu.cycles += 1;
            cpu.tick(recorder);
        }
    }

    #[test]
    fn sets_the_initial_sp_to_the_last_byte_of_internal_sram() {
        let cpu = Cpu::new(vec![0; 1024], 0x1000);
        assert_eq!(cpu.sp(), 0x10ff);
    }

    #[test]
    fn executes_queued_events_after_the_given_number_of_cycles() {
        let mut cpu = Cpu::new(vec![0; 1024], 0x1000);
        let mut recorder = Recorder::default();
        for i in [1, 4, 10] {
            cpu.add_clock_event(ClockEvent::External(i), i as u64);
        }
        run(&mut cpu, &mut recorder, 10);
        assert_eq!(recorder.events, [(1, 1), (4, 4), (10, 10)]);
    }

    #[test]
    fn sorts_events_added_in_reverse_order() {
        let mut cpu = Cpu::new(vec![0; 1024], 0x1000);
        let mut recorder = Recorder::default();
        for i in [10, 4, 1] {
            cpu.add_clock_event(ClockEvent::External(i), i as u64);
        }
        run(&mut cpu, &mut recorder, 10);
        assert_eq!(recorder.events, [(1, 1), (4, 4), (10, 10)]);
    }

    #[test]
    fn update_clock_event_moves_the_given_event() {
        let mut cpu = Cpu::new(vec![0; 1024], 0x1000);
        let mut recorder = Recorder::default();
        for i in [1, 4, 10] {
            cpu.add_clock_event(ClockEvent::External(i), i as u64);
        }
        assert!(cpu.update_clock_event(ClockEvent::External(4), 2));
        assert!(cpu.update_clock_event(ClockEvent::External(1), 12));
        run(&mut cpu, &mut recorder, 14);
        assert_eq!(recorder.events, [(4, 2), (10, 10), (1, 12)]);
    }

    #[test]
    fn clear_clock_event_removes_the_given_event() {
        let mut cpu = Cpu::new(vec![0; 1024], 0x1000);
        let mut recorder = Recorder::default();
        for i in [1, 4, 10] {
            cpu.add_clock_event(ClockEvent::External(i), i as u64);
        }
        cpu.clear_clock_event(ClockEvent::External(4));
        run(&mut cpu, &mut recorder, 10);
        assert_eq!(recorder.events, [(1, 1), (10, 10)]);
    }

    #[test]
    fn clear_clock_event_is_false_for_an_event_not_scheduled() {
        let mut cpu = Cpu::new(vec![0; 1024], 0x1000);
        let event4 = cpu.add_clock_event(ClockEvent::External(4), 4);
        let event10 = cpu.add_clock_event(ClockEvent::External(10), 10);
        cpu.add_clock_event(ClockEvent::External(1), 1);
        assert!(cpu.clear_clock_event(event4));
        assert!(cpu.clear_clock_event(event10));
        assert!(!cpu.clear_clock_event(event4));
        assert!(!cpu.clear_clock_event(event10));
        assert!(!cpu.update_clock_event(event4, 3));
    }

    #[test]
    fn events_due_on_the_same_cycle_run_newest_first_one_per_tick() {
        // Not in avr8js's spec: the order its linked list gives, which the
        // vector has to keep.
        let mut cpu = Cpu::new(vec![0; 1024], 0x1000);
        let mut recorder = Recorder::default();
        cpu.add_clock_event(ClockEvent::External(1), 2);
        cpu.add_clock_event(ClockEvent::External(2), 2);
        run(&mut cpu, &mut recorder, 3);
        assert_eq!(recorder.events, [(2, 2), (1, 3)]);
    }

    #[test]
    fn executes_an_interrupt_handler() {
        let mut cpu = Cpu::new(vec![0; 0x8000], 8192);
        cpu.pc = 0x520;
        cpu.data[94] = 0;
        cpu.data[93] = 0x80; // SP <- 0x80
        cpu.data[95] = 0b1000_0001; // SREG <- I------C
        avr_interrupt(&mut cpu, 5);
        assert_eq!(cpu.cycles, 2);
        assert_eq!(cpu.pc, 5);
        assert_eq!(cpu.data[93], 0x7e); // SP
        assert_eq!(cpu.data[0x80], 0x20); // Return addr low
        assert_eq!(cpu.data[0x7f], 0x5); // Return addr high
        assert_eq!(cpu.data[95], 0b0000_0001); // SREG: -------C
    }

    #[test]
    fn pushes_a_three_byte_return_address_with_a_22_bit_pc() {
        // avr8js issue #58.
        let mut cpu = Cpu::new(vec![0; 0x80000], 8192);
        assert!(cpu.pc22_bits);
        cpu.pc = 0x10520;
        cpu.data[94] = 0;
        cpu.data[93] = 0x80; // SP <- 0x80
        cpu.data[95] = 0b1000_0001; // SREG <- I------C
        avr_interrupt(&mut cpu, 5);
        assert_eq!(cpu.cycles, 2);
        assert_eq!(cpu.pc, 5);
        assert_eq!(cpu.data[93], 0x7d); // SP should decrement by 3
        assert_eq!(cpu.data[0x80], 0x20); // Return addr low
        assert_eq!(cpu.data[0x7f], 0x05); // Return addr high
        assert_eq!(cpu.data[0x7e], 0x1); // Return addr extended
        assert_eq!(cpu.data[95], 0b0000_0001); // SREG: -------C
    }

    #[test]
    fn takes_the_lowest_pending_vector_first_and_only_when_enabled() {
        let mut cpu = Cpu::new(vec![0; 1024], 2048);
        let low = InterruptConfig {
            address: 4,
            enable_register: 0x60,
            enable_mask: 1,
            flag_register: 0x61,
            flag_mask: 1,
            ..InterruptConfig::default()
        };
        let high = InterruptConfig {
            address: 10,
            flag_mask: 2,
            enable_mask: 2,
            ..low
        };
        cpu.data[0x60] = 3;
        cpu.set_interrupt_flag(&high);
        cpu.set_interrupt_flag(&low);
        assert_eq!(cpu.data[0x61], 3);

        cpu.tick(&mut ());
        assert_eq!(cpu.pc, 0, "I is clear");

        cpu.data[SREG as usize] = 0x80;
        cpu.tick(&mut ());
        assert_eq!(cpu.pc, 4);
        assert_eq!(cpu.data[0x61], 2, "the flag is cleared as it is taken");
        assert_eq!(cpu.next_interrupt, 10);
    }
}
