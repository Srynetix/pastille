// SPDX-License-Identifier: MIT OR Apache-2.0
// Ported from avr8js, src/peripherals/usart.spec.ts.
// Copyright (c) 2019-2025 Uri Shaked, MIT licence (see avr/NOTICE.md).

//! usart.spec.ts, every case. Where the spec spies on `onByteTransmit`, this
//! reads [`AvrUsart::transmitted`]; on `onConfigurationChange`,
//! [`AvrUsart::configuration_changed`]; on `onRxComplete`, RXC. The
//! `onLineTransmit` cases check the bytes sent, since the line buffer is the
//! listener's here. Then the cases this port adds.

use crate::avr::Avr;
use crate::avr::cpu::Cpu;
use crate::avr::usart::{AvrUsart, USART0_CONFIG};

const FREQ_16MHZ: u64 = 16_000_000;
const FREQ_11_0529MHZ: u64 = 11_059_200;

// CPU registers
const SREG: usize = 95;

// USART0 Registers
const UCSR0A: u16 = 0xc0;
const UCSR0B: u16 = 0xc1;
const UCSR0C: u16 = 0xc2;
const UBRR0L: u16 = 0xc4;
const UBRR0H: u16 = 0xc5;
const UDR0: u16 = 0xc6;

// Register bit names
const U2X0: u8 = 2;
const TXEN: u8 = 8;
const RXEN: u8 = 16;
const UDRIE: u8 = 0x20;
const TXCIE: u8 = 0x40;
const RXC: u8 = 0x80;
const TXC: u8 = 0x40;
const UDRE: u8 = 0x20;
const USBS: u8 = 0x08;
const UPM0: u8 = 0x10;
const UPM1: u8 = 0x20;

// Interrupt address
const PC_INT_UDRE: u32 = 0x26;
const PC_INT_TXC: u32 = 0x28;
const UCSZ0: u8 = 2;
const UCSZ1: u8 = 4;
const UCSZ2: u8 = 4;

/// `new CPU(new Uint16Array(1024))` and `new AVRUSART(cpu, usart0Config, freq)`.
fn usart(freq: u64) -> Avr {
    let mut cpu = Cpu::new(vec![0; 1024], 8192);
    let mut io = crate::avr::Peripherals::new(&mut cpu, &[], &[]);
    io.usart = Some(AvrUsart::new(&mut cpu, USART0_CONFIG, freq));
    Avr { cpu, io }
}

fn the(avr: &Avr) -> &AvrUsart {
    avr.io.usart.as_ref().unwrap()
}

fn changed(avr: &mut Avr) -> bool {
    std::mem::take(&mut avr.io.usart.as_mut().unwrap().configuration_changed)
}

fn sent(avr: &mut Avr) -> Vec<u16> {
    std::mem::take(&mut avr.io.usart.as_mut().unwrap().transmitted)
        .into_iter()
        .map(|sent| sent.value)
        .collect()
}

fn write_byte(avr: &mut Avr, value: u16) -> bool {
    let Avr { cpu, io } = avr;
    io.usart.as_mut().unwrap().write_byte(cpu, value, false)
}

#[test]
fn should_correctly_calculate_the_baud_rate_from_ubrr() {
    let mut avr = usart(FREQ_11_0529MHZ);
    avr.write_data(UBRR0H, 0);
    avr.write_data(UBRR0L, 5);
    assert_eq!(the(&avr).baud_rate(&avr.cpu), 115200);
}

#[test]
fn should_correctly_calculate_the_baud_rate_from_ubrr_in_double_speed_mode() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UBRR0H, 3);
    avr.write_data(UBRR0L, 64);
    avr.write_data(UCSR0A, U2X0);
    assert_eq!(the(&avr).baud_rate(&avr.cpu), 2400);
}

#[test]
fn should_call_on_configuration_change_when_the_baud_rate_changes() {
    let mut avr = usart(FREQ_16MHZ);
    changed(&mut avr);
    avr.write_data(UBRR0H, 0);
    assert!(changed(&mut avr));
    avr.write_data(UBRR0L, 5);
    assert!(changed(&mut avr));
    avr.write_data(UCSR0A, U2X0);
    assert!(changed(&mut avr));
}

#[test]
fn bits_per_char_should_return_5_bits_per_byte_when_ucsz_is_0() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0C, 0);
    assert_eq!(the(&avr).bits_per_char(&avr.cpu), 5);
}

#[test]
fn bits_per_char_should_return_6_bits_per_byte_when_ucsz_is_1() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0C, UCSZ0);
    assert_eq!(the(&avr).bits_per_char(&avr.cpu), 6);
}

#[test]
fn bits_per_char_should_return_7_bits_per_byte_when_ucsz_is_2() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0C, UCSZ1);
    assert_eq!(the(&avr).bits_per_char(&avr.cpu), 7);
}

#[test]
fn bits_per_char_should_return_8_bits_per_byte_when_ucsz_is_3() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0C, UCSZ0 | UCSZ1);
    assert_eq!(the(&avr).bits_per_char(&avr.cpu), 8);
}

#[test]
fn bits_per_char_should_return_9_bits_per_byte_when_ucsz_is_7() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0C, UCSZ0 | UCSZ1);
    avr.write_data(UCSR0B, UCSZ2);
    assert_eq!(the(&avr).bits_per_char(&avr.cpu), 9);
}

#[test]
fn bits_per_char_should_call_on_configuration_change_when_bits_per_char_change() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0C, UCSZ0 | UCSZ1);
    assert!(changed(&mut avr));
    avr.write_data(UCSR0B, UCSZ2);
    assert!(changed(&mut avr));
    avr.write_data(UCSR0B, UCSZ2);
    assert!(!changed(&mut avr));
}

#[test]
fn stop_bits_should_return_1_when_usbs_is_0() {
    let avr = usart(FREQ_16MHZ);
    assert_eq!(the(&avr).stop_bits(&avr.cpu), 1);
}

#[test]
fn stop_bits_should_return_2_when_usbs_is_1() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0C, USBS);
    assert_eq!(the(&avr).stop_bits(&avr.cpu), 2);
}

#[test]
fn parity_enabled_should_return_false_when_upm1_is_0() {
    let avr = usart(FREQ_16MHZ);
    assert!(!the(&avr).parity_enabled(&avr.cpu));
}

#[test]
fn parity_enabled_should_return_true_when_upm1_is_1() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0C, UPM1);
    assert!(the(&avr).parity_enabled(&avr.cpu));
}

#[test]
fn parity_odd_should_return_false_when_upm0_is_0() {
    let avr = usart(FREQ_16MHZ);
    assert!(!the(&avr).parity_odd(&avr.cpu));
}

#[test]
fn parity_odd_should_return_true_when_upm0_is_1() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0C, UPM0);
    assert!(the(&avr).parity_odd(&avr.cpu));
}

#[test]
fn should_invoke_on_byte_transmit_when_udr0_is_written_to() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0B, TXEN);
    avr.write_data(UDR0, 0x61);
    assert_eq!(sent(&mut avr), [0x61]);
}

#[test]
fn tx_enable_should_equal_true_when_the_transitter_is_enabled() {
    let mut avr = usart(FREQ_16MHZ);
    assert!(!the(&avr).tx_enable(&avr.cpu));
    avr.write_data(UCSR0B, TXEN);
    assert!(the(&avr).tx_enable(&avr.cpu));
}

#[test]
fn rx_enable_should_equal_true_when_the_transitter_is_enabled() {
    let mut avr = usart(FREQ_16MHZ);
    assert!(!the(&avr).rx_enable(&avr.cpu));
    avr.write_data(UCSR0B, RXEN);
    assert!(the(&avr).rx_enable(&avr.cpu));
}

#[test]
fn tick_should_trigger_data_register_empty_interrupt_if_udre_is_set() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0B, UDRIE | TXEN);
    avr.cpu.data[SREG] = 0x80; // SREG: I-------
    avr.tick();
    assert_eq!(avr.cpu.pc, PC_INT_UDRE);
    assert_eq!(avr.cpu.cycles, 2);
    assert_eq!(avr.cpu.data[UCSR0A as usize] & UDRE, 0);
}

#[test]
fn tick_should_trigger_data_tx_complete_interrupt_if_txcie_is_set() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0B, TXCIE | TXEN);
    avr.write_data(UDR0, 0x61);
    avr.cpu.data[SREG] = 0x80; // SREG: I-------
    avr.cpu.cycles = 1_000_000;
    avr.tick();
    assert_eq!(avr.cpu.pc, PC_INT_TXC);
    assert_eq!(avr.cpu.cycles, 1_000_000 + 2);
    assert_eq!(avr.cpu.data[UCSR0A as usize] & TXC, 0);
}

#[test]
fn tick_should_not_trigger_data_tx_complete_interrupt_if_udr_was_not_written_to() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0B, TXCIE | TXEN);
    avr.cpu.data[SREG] = 0x80; // SREG: I-------
    avr.tick();
    assert_eq!(avr.cpu.pc, 0);
    assert_eq!(avr.cpu.cycles, 0);
}

#[test]
fn tick_should_not_trigger_any_interrupt_if_interrupts_are_disabled() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0B, UDRIE | TXEN);
    avr.write_data(UDR0, 0x61);
    avr.cpu.data[SREG] = 0; // SREG: 0 (disable interrupts)
    avr.cpu.cycles = 1_000_000;
    avr.tick();
    assert_eq!(avr.cpu.pc, 0);
    assert_eq!(avr.cpu.cycles, 1_000_000);
    assert_eq!(avr.cpu.data[UCSR0A as usize], TXC | UDRE);
}

#[test]
fn on_line_transmit_should_send_every_byte_of_a_line_and_its_newline() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0B, TXEN);
    for byte in *b"Hello\n" {
        avr.write_data(UDR0, byte);
    }
    assert_eq!(sent(&mut avr), b"Hello\n".map(u16::from));
}

#[test]
fn on_line_transmit_should_send_bytes_without_a_newline() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0B, TXEN);
    avr.write_data(UDR0, 0x48); // 'H'
    avr.write_data(UDR0, 0x69); // 'i'
    assert_eq!(sent(&mut avr), [0x48, 0x69]);
}

#[test]
fn write_byte_should_return_false_if_called_when_rx_is_busy() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0B, RXEN);
    avr.write_data(UBRR0L, 103); // baud: 9600
    assert!(write_byte(&mut avr, 10));
    assert!(!write_byte(&mut avr, 10));
    avr.tick();
    assert!(!write_byte(&mut avr, 10));
}

#[test]
fn should_set_the_txc_bit_after_1_04ms_when_baud_rate_set_to_9600() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0B, TXEN);
    avr.write_data(UBRR0L, 103); // baud: 9600
    avr.write_data(UDR0, 0x48); // 'H'
    avr.cpu.cycles += 16000; // 1ms
    avr.tick();
    assert_eq!(avr.cpu.data[UCSR0A as usize] & TXC, 0);
    avr.cpu.cycles += 800; // 0.05ms
    avr.tick();
    assert_eq!(avr.cpu.data[UCSR0A as usize] & TXC, TXC);
}

#[test]
fn should_be_ready_to_recieve_the_next_byte_after_1_04ms_when_baudrate_set_to_9600() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0B, RXEN);
    avr.write_data(UBRR0L, 103); // baud: 9600
    assert!(write_byte(&mut avr, 0x42));
    avr.cpu.cycles += 16000; // 1ms
    avr.tick();
    assert_eq!(avr.cpu.data[UCSR0A as usize] & RXC, 0); // byte not received yet
    assert!(the(&avr).rx_busy());
    avr.cpu.cycles += 800; // 0.05ms
    avr.tick();
    assert_eq!(avr.cpu.data[UCSR0A as usize] & RXC, RXC);
    assert!(!the(&avr).rx_busy());
    assert_eq!(avr.read_data(UDR0), 0x42);
    assert_eq!(avr.read_data(UDR0), 0);
}

// ---- not in avr8js's spec

#[test]
fn writing_ucsra_keeps_rxc_and_udre_and_clears_txc_only_when_written_1() {
    let mut avr = usart(FREQ_16MHZ);
    avr.write_data(UCSR0B, TXEN | RXEN);
    avr.write_data(UDR0, 0x61);
    avr.cpu.cycles += 20_000;
    avr.tick();
    assert_eq!(avr.cpu.data[UCSR0A as usize], TXC | UDRE);
    // U2X written, TXC written 0: both flags stay.
    avr.write_data(UCSR0A, U2X0);
    assert_eq!(avr.cpu.data[UCSR0A as usize], TXC | UDRE | U2X0);
    // TXC written 1: it goes, UDRE stays.
    avr.write_data(UCSR0A, U2X0 | TXC);
    assert_eq!(avr.cpu.data[UCSR0A as usize], UDRE | U2X0);
}

#[test]
fn turning_the_transmitter_on_or_off_flags_a_pin_change() {
    let mut avr = usart(FREQ_16MHZ);
    assert!(!avr.io.outputs_changed());
    avr.write_data(UCSR0B, TXEN);
    assert!(avr.io.outputs_changed());
    avr.io.usart.as_mut().unwrap().pins_changed = false;
    avr.write_data(UCSR0B, TXEN | UDRIE);
    assert!(!avr.io.outputs_changed());
}

#[test]
fn each_byte_sent_carries_its_cycle_and_its_frame() {
    let mut avr = usart(FREQ_16MHZ);
    // Serial.begin(9600) as the Arduino core does it: U2X, UBRR 207.
    avr.write_data(UCSR0A, U2X0);
    avr.write_data(UBRR0L, 207);
    avr.write_data(UCSR0B, TXEN);
    avr.cpu.cycles = 1234;
    avr.write_data(UDR0, b'A');
    let sent = the(&avr).transmitted[0];
    assert_eq!(sent.cycles, 1234);
    assert_eq!(sent.shape.cycles_per_bit, 208 * 8);
    assert_eq!(sent.shape.data_bits, 8);
    assert_eq!(sent.shape.parity, None);
    assert_eq!(sent.shape.stop_bits, 1);
    assert_eq!(the(&avr).baud_rate(&avr.cpu), 9615);
}
