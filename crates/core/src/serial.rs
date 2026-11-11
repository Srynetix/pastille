// SPDX-License-Identifier: MIT OR Apache-2.0

//! A serial line, and what a receiver listening to it hears (A-06).
//!
//! The program's USART sends frames at its baud; the serial monitor listens
//! at its own. When the two agree, every byte arrives as it was sent. When
//! they do not, the monitor still finds start bits and samples the line in
//! the middle of where it thinks each bit is — and gets other bytes, or the
//! same edge read as two, which is the garbage a mismatch is. Nothing here
//! makes garbage up: it is what sampling the line at the wrong rate gives.
//!
//! A [`Frame`] is a character on the line: when it starts, how long a bit
//! lasts, and its bits. Between frames the line idles high. A [`Receiver`]
//! is a UART's receive side: it waits for the line to go low, checks it is
//! still low half a bit later, samples each data bit (and parity) at its
//! middle, then the stop bit; a stop bit read low is a framing error, and the
//! byte is delivered anyway, as a USB-serial adapter's driver passes it on.

use crate::avr::usart::FrameShape;

/// A character on the line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    /// When its start bit begins, in seconds.
    pub start: f64,
    /// How long each bit lasts, in seconds.
    pub bit: f64,
    pub value: u16,
    pub data_bits: u8,
    /// `Some(true)` odd, `Some(false)` even, `None` none.
    pub parity: Option<bool>,
    pub stop_bits: u8,
}

impl Frame {
    /// `value` sent at `baud`, eight data bits, no parity, one stop bit: how
    /// the serial monitor sends, as the Arduino IDE's does.
    pub fn eight_n_one(start: f64, baud: f64, value: u8) -> Frame {
        Frame {
            start,
            bit: 1.0 / baud,
            value: u16::from(value),
            data_bits: 8,
            parity: None,
            stop_bits: 1,
        }
    }

    /// `value` sent by a USART at `start`, framed as `shape` on a chip
    /// clocked at `hz`.
    pub fn sent(start: f64, value: u16, shape: FrameShape, hz: f64) -> Frame {
        Frame {
            start,
            bit: shape.cycles_per_bit as f64 / hz,
            value,
            data_bits: shape.data_bits,
            parity: shape.parity,
            stop_bits: shape.stop_bits,
        }
    }

    fn len(&self) -> usize {
        1 + usize::from(self.data_bits)
            + usize::from(self.parity.is_some())
            + usize::from(self.stop_bits)
    }

    pub fn end(&self) -> f64 {
        self.start + self.len() as f64 * self.bit
    }

    /// The level of its `k`th bit: the start bit low, the data least
    /// significant first, the parity, the stop bits high.
    fn level(&self, k: usize) -> bool {
        let data = usize::from(self.data_bits);
        match k {
            0 => false,
            k if k <= data => self.value >> (k - 1) & 1 != 0,
            k if k == data + 1 && self.parity.is_some() => {
                let ones = (self.value & ((1 << data) - 1)).count_ones() % 2 == 1;
                ones != (self.parity == Some(true))
            }
            _ => true,
        }
    }
}

/// A byte as a receiver heard it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Heard {
    pub value: u16,
    /// When its stop bit was sampled, in seconds: when it arrived.
    pub at: f64,
    /// The stop bit was low.
    pub framing_error: bool,
}

/// A UART's receive side, at one baud and one framing.
#[derive(Debug, Clone, PartialEq)]
pub struct Receiver {
    pub bit: f64,
    pub data_bits: u8,
    pub parity: bool,
    /// The frames on the line it has not finished with, in the order sent.
    frames: Vec<Frame>,
    /// From when it is looking for a start bit.
    resume: f64,
}

impl Receiver {
    /// Listening at `baud`, eight data bits, no parity.
    pub fn eight_n_one(baud: f64) -> Receiver {
        Receiver::new(1.0 / baud, 8, false)
    }

    pub fn new(bit: f64, data_bits: u8, parity: bool) -> Receiver {
        Receiver {
            bit,
            data_bits,
            parity,
            frames: Vec::new(),
            resume: f64::NEG_INFINITY,
        }
    }

    /// Listens at another rate from now on, as the monitor's baud is changed.
    pub fn set_bit(&mut self, bit: f64) {
        self.bit = bit;
    }

    /// A frame put on the line. Frames are sent in order; one that starts
    /// before the last has ended is moved to its end, as a transmitter's
    /// shift register would hold it.
    pub fn push(&mut self, mut frame: Frame) {
        if let Some(last) = self.frames.last() {
            frame.start = frame.start.max(last.end());
        }
        self.frames.push(frame);
    }

    /// Where the line is at `at`.
    fn level(&self, at: f64) -> bool {
        self.frames
            .iter()
            .find(|frame| frame.start <= at && at < frame.end())
            .is_none_or(|frame| frame.level(((at - frame.start) / frame.bit) as usize))
    }

    /// The first moment from `from` on that the line is low, if any frame
    /// takes it low.
    fn next_low(&self, from: f64) -> Option<f64> {
        self.frames
            .iter()
            .filter(|frame| frame.end() > from)
            .find_map(|frame| {
                (0..frame.len()).find_map(|k| {
                    let begins = frame.start + k as f64 * frame.bit;
                    (!frame.level(k) && begins + frame.bit > from).then(|| begins.max(from))
                })
            })
    }

    /// Every byte whose stop bit falls before `until`, in order. What is
    /// still arriving at `until` waits for the next call, since a frame sent
    /// later may yet change what it reads.
    pub fn hear(&mut self, until: f64) -> Vec<Heard> {
        let mut heard = Vec::new();
        let data = usize::from(self.data_bits);
        let stop = 1 + data + usize::from(self.parity);
        while let Some(begins) = self.next_low(self.resume) {
            let sample = |k: usize| begins + (k as f64 + 0.5) * self.bit;
            if sample(stop) >= until {
                break;
            }
            // Not low any more half a bit in: a glitch, not a start bit.
            if self.level(sample(0)) {
                self.resume = sample(0);
                continue;
            }
            let value = (1..=data).fold(0u16, |value, k| {
                value | u16::from(self.level(sample(k))) << (k - 1)
            });
            heard.push(Heard {
                value,
                at: sample(stop),
                framing_error: !self.level(sample(stop)),
            });
            self.resume = sample(stop);
        }
        let resume = self.resume;
        self.frames.retain(|frame| frame.end() > resume);
        heard
    }
}

/// The serial monitor on a board's USB port: what it has heard from the
/// program, at the baud it is set to, and not yet shown.
#[derive(Debug, Clone, PartialEq)]
pub struct Monitor {
    baud: u32,
    receiver: Receiver,
    heard: Vec<u8>,
    /// When the last byte it sent has finished going out, in seconds.
    sending_until: f64,
}

/// The Arduino IDE's serial monitor starts at 9600, as the examples do.
pub const MONITOR_BAUD: u32 = 9600;

/// Bytes heard and not yet taken, at most: a program printing in a loop
/// with nobody reading must not grow the heap for ever. The oldest go.
const MOST_HEARD: usize = 1 << 16;

impl Default for Monitor {
    fn default() -> Monitor {
        Monitor {
            baud: MONITOR_BAUD,
            receiver: Receiver::eight_n_one(f64::from(MONITOR_BAUD)),
            heard: Vec::new(),
            sending_until: f64::NEG_INFINITY,
        }
    }
}

impl Monitor {
    pub fn baud(&self) -> u32 {
        self.baud
    }

    pub fn set_baud(&mut self, baud: u32) {
        let baud = baud.max(1);
        self.baud = baud;
        self.receiver.set_bit(1.0 / f64::from(baud));
    }

    /// A frame the program put on its TX line.
    pub fn listen(&mut self, frame: Frame) {
        self.receiver.push(frame);
    }

    /// Hears what has arrived by `until`.
    pub fn hear(&mut self, until: f64) {
        for heard in self.receiver.hear(until) {
            self.heard.push(heard.value as u8);
        }
        if self.heard.len() > MOST_HEARD {
            let extra = self.heard.len() - MOST_HEARD;
            self.heard.drain(..extra);
        }
    }

    /// What it has heard since this was last asked.
    pub fn take(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.heard)
    }

    /// `bytes` sent from `now`, after whatever it is still sending, at its
    /// baud, 8N1: the frames on the program's RX line.
    pub fn send(&mut self, now: f64, bytes: &[u8]) -> Vec<Frame> {
        let baud = f64::from(self.baud);
        let mut at = now.max(self.sending_until);
        let frames = bytes
            .iter()
            .map(|&byte| {
                let frame = Frame::eight_n_one(at, baud, byte);
                at = frame.end();
                frame
            })
            .collect();
        self.sending_until = at;
        frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(baud: f64, text: &[u8]) -> Vec<Frame> {
        let bit = 1.0 / baud;
        text.iter()
            .enumerate()
            .map(|(k, &byte)| Frame::eight_n_one(0.01 + k as f64 * 10.0 * bit, baud, byte))
            .collect()
    }

    fn hear(baud: f64, frames: &[Frame]) -> Vec<Heard> {
        let mut receiver = Receiver::eight_n_one(baud);
        for frame in frames {
            receiver.push(*frame);
        }
        receiver.hear(1.0)
    }

    #[test]
    fn at_the_same_baud_every_byte_arrives_as_sent() {
        let heard = hear(9600.0, &line(9600.0, b"Hello, world\n"));
        let bytes: Vec<u8> = heard.iter().map(|heard| heard.value as u8).collect();
        assert_eq!(bytes, b"Hello, world\n");
        assert!(heard.iter().all(|heard| !heard.framing_error));
    }

    #[test]
    fn the_arduino_s_9615_baud_reads_cleanly_at_9600() {
        // Serial.begin(9600) makes 16 MHz / (8 × 208) = 9615 baud.
        let heard = hear(9600.0, &line(9615.4, b"0123456789abcdef\n"));
        let bytes: Vec<u8> = heard.iter().map(|heard| heard.value as u8).collect();
        assert_eq!(bytes, b"0123456789abcdef\n");
    }

    #[test]
    fn at_the_wrong_baud_it_is_garbage() {
        // 115200 sent, 9600 heard: a twelfth as many bytes, none of them
        // what was sent. 9600 sent, 115200 heard: each low bit read as a
        // run of zeros, broken frames.
        let text = b"Hello, world, this is a line\n";
        let fast = hear(9600.0, &line(115_200.0, text));
        assert!(fast.len() < text.len() / 4, "{fast:?}");
        assert!(
            fast.iter()
                .all(|heard| !text.contains(&(heard.value as u8)))
                || fast.len() < 4
        );

        let slow = hear(115_200.0, &line(9600.0, text));
        assert!(slow.len() > text.len(), "{}", slow.len());
        let bytes: Vec<u8> = slow.iter().map(|heard| heard.value as u8).collect();
        assert_ne!(&bytes[..text.len()], text);
        assert!(slow.iter().any(|heard| heard.framing_error));
    }

    #[test]
    fn a_byte_still_arriving_waits() {
        let frames = line(9600.0, b"AB");
        let mut receiver = Receiver::eight_n_one(9600.0);
        for frame in &frames {
            receiver.push(*frame);
        }
        // Half way through B.
        let first = receiver.hear(frames[1].start + 5.0 / 9600.0);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].value, u16::from(b'A'));
        let second = receiver.hear(1.0);
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].value, u16::from(b'B'));
    }

    #[test]
    fn parity_is_sent_and_checked_for_its_place() {
        // 7E1: seven data bits and even parity, heard as 7 bits + parity.
        let frame = Frame {
            start: 0.0,
            bit: 1.0 / 9600.0,
            value: 0b100_0001,
            data_bits: 7,
            parity: Some(false),
            stop_bits: 1,
        };
        // Two ones: even parity bit 0.
        assert!(!frame.level(8));
        let odd = Frame {
            parity: Some(true),
            ..frame
        };
        assert!(odd.level(8));
        let mut receiver = Receiver::new(1.0 / 9600.0, 7, true);
        receiver.push(frame);
        let heard = receiver.hear(1.0);
        assert_eq!(heard[0].value, 0b100_0001);
        assert!(!heard[0].framing_error);
    }
}
