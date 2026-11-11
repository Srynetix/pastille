// SPDX-License-Identifier: MIT OR Apache-2.0

//! An Arduino Uno's pins, as the circuit sees them (A-01, ADR 0017).
//!
//! The program runs on the emulated ATmega328P (`crate::avr`); this is the
//! other half: what each of its pins is to the solver, and how a solved
//! voltage becomes the level the program reads. It is the 74HC parts' shape
//! (C-108): within a solve each pin is linear, driven high or low behind a
//! resistance or a leak, and what it should be is carried from the moment
//! the program last changed it.
//!
//! **Powered over USB.** The board makes its own 5 V, so its pins drive from
//! that rather than from a supply pin the circuit has to wire: each is
//! stamped as its Norton equivalent against the board's GND pin, and the 5 V
//! pin is 5 V behind a little resistance — USB's, its fuse's and the trace's.
//!
//! **From Microchip's ATmega328P sheet** (DS40002061B, rev. B, 09/2020), at
//! 5 V and 25 °C, read on 2026-10-04 (A-02):
//!
//! - an output's resistance from its typical curves, Figs 31-355 and 31-353
//!   (p. 513–514): about 26 Ω high and 24 Ω low, close to straight up to
//!   20 mA. Table 30-1's guaranteed figures (VOH ≥ 4.2 V at −20 mA, VOL ≤
//!   0.9 V at 20 mA, at 85 °C) bound them at 40 and 45 Ω;
//! - an input's switching points, typical, from Figs 31-356 to 31-358 (p.
//!   514–515): up at about 2.6 V, down at about 2.07 V — the hysteresis the
//!   sheet draws. Table 30-1 guarantees only 0.6 VCC and 0.3 VCC;
//! - the pull-up, 20 to 50 kΩ (Table 30-1), about 36 kΩ on Fig 31-348;
//! - an input's leak, 1 µA at most (Table 30-1);
//! - 40 mA a pin and 200 mA through VCC and GND at most (§29.1); Arduino
//!   rates a pin at 20 mA.
//!
//! **From the Uno R3's schematic** (UNO-TH_Rev3e): USB's 5 V reaches the
//! board through a 500 mA polyfuse and a MOSFET, a fraction of an ohm in
//! all (estimated, not read off their sheets); 3V3 is an LP2985-33 off 5 V,
//! 150 mA rated (TI SLVS522S); VIN is the barrel jack's, after its diode,
//! and on USB alone nothing feeds it but a 10 k + 10 k divider to GND; D0 is
//! the USB-serial chip's TX through 1 kΩ, which idles high, so D0 sits at
//! 5 V unless something pulls it down hard; D1 sees that chip's input
//! through 1 kΩ, which loads it with nothing worth stamping. The yellow L on
//! D13 hangs off an op-amp follower, so it loads D13 with nothing either.

/// The pins, as the board's headers name them, in the registry's order.
pub const PINS: [&str; 24] = [
    "D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8", "D9", "D10", "D11", "D12", "D13", "A0",
    "A1", "A2", "A3", "A4", "A5", "5V", "3V3", "VIN", "GND",
];

/// How many of them are the program's: D0–D13 and A0–A5.
pub const IO: usize = 20;
pub const FIVE_VOLTS: usize = 20;
pub const THREE_VOLTS: usize = 21;
pub const VIN: usize = 22;
pub const GND: usize = 23;

/// The board's supply, and what is behind it.
pub const SUPPLY: f64 = 5.0;
pub const SUPPLY_OHMS: f64 = 0.5;
/// The 3V3 pin: its regulator's output, and what is behind it.
pub const THREE_VOLTS_SUPPLY: f64 = 3.3;
pub const THREE_VOLTS_OHMS: f64 = 1.0;
/// What VIN is on USB alone: the divider the board watches it through.
pub const VIN_OHMS: f64 = 20e3;
/// Behind an output, high and low.
pub const HIGH_OHMS: f64 = 26.0;
pub const LOW_OHMS: f64 = 24.0;
/// An input's pull-up, when the program turns it on.
pub const PULL_UP_OHMS: f64 = 36e3;
/// What an input takes when it is only an input.
pub const LEAK: f64 = 1e-9;
/// What a pin is rated for (Arduino's figure), what damages it, and what the
/// chip passes through its VCC and its GND at most (§29.1).
pub const PIN_RATED_AMPS: f64 = 20e-3;
pub const PIN_MOST_AMPS: f64 = 40e-3;
pub const CHIP_MOST_AMPS: f64 = 200e-3;
/// The most any pin can be above GND: VCC + 0.5 V (§29.1).
pub const MOST_PIN_VOLTS: f64 = 5.5;
/// Where an input switches, as fractions of the supply: up past the higher,
/// down past the lower.
pub const RISING: f64 = 2.6 / SUPPLY;
pub const FALLING: f64 = 2.07 / SUPPLY;
/// The analog pins: A0 is the I/O pin after D13, and there are six.
pub const A0: usize = 14;
pub const ANALOG: usize = 6;
/// The ADC's sample-and-hold, as the sheet gives it (A-02's reading): 14 pF
/// on whichever analog pin the mux is on.
pub const SAMPLE_HOLD_FARADS: f64 = 14e-12;

/// The pin the sample-and-hold is on, from what the run carries for it: the
/// channel plus one, 0 for none (A-04).
pub fn sampled_pin(carried: f64) -> Option<usize> {
    let channel = carried.round() as usize;
    (1..=ANALOG).contains(&channel).then(|| A0 + channel - 1)
}

/// D0's line from the USB-serial chip, idling high.
pub const SERIAL_OHMS: f64 = 1e3;
pub const D0: usize = 0;
/// D1, the USART's TXD while `Serial` is begun.
pub const D1: usize = 1;

/// Where each I/O pin is on the ATmega328P: its port (`crate::avr::PORT_B` …)
/// and bit. D0–D7 are port D, D8–D13 port B, A0–A5 port C.
pub fn port_bit(pin: usize) -> (usize, u8) {
    match pin {
        0..=7 => (crate::avr::PORT_D, pin as u8),
        8..=13 => (crate::avr::PORT_B, (pin - 8) as u8),
        _ => (crate::avr::PORT_C, (pin - 14) as u8),
    }
}

/// What a pin is doing, as the solver stamps it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drive {
    /// An input, and nothing else: a leak.
    Input = 0,
    /// An input with its pull-up on.
    PullUp = 1,
    Low = 2,
    High = 3,
}

impl Drive {
    fn from_bits(bits: u64) -> Drive {
        match bits & 3 {
            1 => Drive::PullUp,
            2 => Drive::Low,
            3 => Drive::High,
            _ => Drive::Input,
        }
    }
}

impl From<crate::avr::gpio::PinState> for Drive {
    fn from(state: crate::avr::gpio::PinState) -> Drive {
        use crate::avr::gpio::PinState;
        match state {
            PinState::Low => Drive::Low,
            PinState::High => Drive::High,
            PinState::Input => Drive::Input,
            PinState::InputPullUp => Drive::PullUp,
        }
    }
}

/// Every I/O pin's drive, as the run carries it (C-48): two bits a pin,
/// forty in all, which a double holds exactly.
pub fn encode(drives: &[Drive; IO]) -> f64 {
    let bits = drives.iter().enumerate().fold(0u64, |bits, (pin, drive)| {
        bits | (*drive as u64) << (2 * pin)
    });
    bits as f64
}

pub fn decode(value: f64) -> [Drive; IO] {
    let bits = value.max(0.0) as u64;
    std::array::from_fn(|pin| Drive::from_bits(bits >> (2 * pin)))
}

/// What an input reads at `volts` above the board's GND, given what it read
/// last: it has to climb past the higher threshold to read high, and fall
/// past the lower one to read low.
pub fn reads_high(volts: f64, was: bool) -> bool {
    if was {
        volts > FALLING * SUPPLY
    } else {
        volts > RISING * SUPPLY
    }
}

/// What is behind a pin, as the solver stamps it against the board's GND:
/// each a resistance and the voltage it drives towards — its 5 V pin the USB
/// supply, an output high or low, an input a leak, and D0 its drive and the
/// USB-serial chip's TX besides. Nothing for GND itself. The stamp and the
/// currents the linter holds the pins to are both read from here (A-10).
pub fn branches(pin: usize, drives: &[Drive; IO]) -> [Option<(f64, f64)>; 2] {
    let own = match pin {
        FIVE_VOLTS => (SUPPLY_OHMS, SUPPLY),
        THREE_VOLTS => (THREE_VOLTS_OHMS, THREE_VOLTS_SUPPLY),
        // On USB, nothing drives VIN but the divider the board watches it
        // through.
        VIN => (VIN_OHMS, 0.0),
        GND => return [None, None],
        _ => match drives[pin] {
            Drive::High => (HIGH_OHMS, SUPPLY),
            Drive::Low => (LOW_OHMS, 0.0),
            Drive::PullUp => (PULL_UP_OHMS, SUPPLY),
            Drive::Input => (1.0 / LEAK, 0.0),
        },
    };
    // D0 is held up by the USB-serial chip's idle TX.
    let serial = (pin == D0).then_some((SERIAL_OHMS, SUPPLY));
    [Some(own), serial]
}

/// The current into each pin at these voltages above the board's GND, as
/// [`branches`] has them stamped; GND takes back what the rest give. A pin
/// that is not wired carries nothing.
pub fn currents(drives: &[Drive; IO], above_ground: &[Option<f64>]) -> Vec<f64> {
    let mut into = vec![0.0; PINS.len()];
    for (pin, amps) in into.iter_mut().enumerate().take(GND) {
        let Some(volts) = above_ground.get(pin).copied().flatten() else {
            continue;
        };
        *amps = branches(pin, drives)
            .into_iter()
            .flatten()
            .map(|(ohms, drives_to)| (volts - drives_to) / ohms)
            .sum();
    }
    into[GND] = -into.iter().sum::<f64>();
    into
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pin_high_into_a_short_gives_what_its_resistance_lets_it() {
        let mut drives = [Drive::Input; IO];
        drives[13] = Drive::High;
        let mut volts = [None; PINS.len()];
        volts[13] = Some(0.0);
        let into = currents(&drives, &volts);
        assert!((into[13] + SUPPLY / HIGH_OHMS).abs() < 1e-12);
        assert!((into[GND] - SUPPLY / HIGH_OHMS).abs() < 1e-12);
    }

    #[test]
    fn every_pin_s_drive_survives_the_carry() {
        let mut drives = [Drive::Input; IO];
        drives[13] = Drive::High;
        drives[2] = Drive::PullUp;
        drives[19] = Drive::Low;
        assert_eq!(decode(encode(&drives)), drives);
    }

    #[test]
    fn the_headers_map_onto_the_ports() {
        assert_eq!(port_bit(13), (crate::avr::PORT_B, 5));
        assert_eq!(port_bit(6), (crate::avr::PORT_D, 6));
        assert_eq!(port_bit(14), (crate::avr::PORT_C, 0));
    }

    #[test]
    fn an_input_switches_up_and_down_at_two_voltages() {
        // Up at about 2.6 V, down at about 2.07 V (Figs 31-356, 31-357).
        assert!(!reads_high(2.5, false));
        assert!(reads_high(2.7, false));
        assert!(reads_high(2.2, true));
        assert!(!reads_high(2.0, true));
    }
}

/// A-01's spike: a program on the AVR, the circuit following it.
#[cfg(test)]
mod spike {
    use std::sync::Arc;

    use crate::engine::engine_for;
    use crate::netlist::{ElementKind, NetId, Netlist};
    use crate::scheduler::Scheduler;

    /// D13 (PB5) toggled every half second by a busy loop of `sbiw`/`brne`:
    /// `ldi r16,0x20; out DDRB,r16; loop: out PINB,r16; ldi r18,40;
    /// outer: ldi r24,0x50; ldi r25,0xC3; inner: sbiw r24,1; brne inner;
    /// dec r18; brne outer; rjmp loop` — 40 × 50,000 × 4 cycles.
    const BLINK: [u16; 11] = [
        0xe200, 0xb904, 0xb903, 0xe228, 0xe580, 0xec93, 0x9701, 0xf7f1, 0x952a, 0xf7d1, 0xcff7,
    ];

    /// Timer0 in fast PWM on OC0A (D6), clock / 64 — 976.6 Hz — at 64/256:
    /// `ldi r16,0x40; out DDRD,r16; ldi r16,0x83; out TCCR0A,r16;
    /// ldi r16,63; out OCR0A,r16; ldi r16,3; out TCCR0B,r16; loop: rjmp loop`.
    const PWM: [u16; 9] = [
        0xe400, 0xb90a, 0xe803, 0xbd04, 0xe30f, 0xbd07, 0xe003, 0xbd05, 0xcfff,
    ];

    /// An Uno running `program`, a 220 Ω and a red LED from D13 to its GND,
    /// and 10 kΩ and 1 µF from D6 to its GND: the nets of D13, of the
    /// capacitor and of GND. Which net is the reference is the solver's to
    /// pick (ADR 0003), so every voltage is read against GND's.
    fn bench(program: &[u16]) -> (Netlist, NetId, NetId, NetId) {
        let mut netlist = Netlist::new();
        let uno = netlist.add_element(ElementKind::Uno);
        netlist.set_program(
            uno,
            Arc::new(crate::netlist::Program {
                name: "test".to_string(),
                hex: String::new(),
                words: Arc::from(program),
            }),
        );
        let resistor = netlist.add_element(ElementKind::Resistor { ohms: 220.0 });
        let led = netlist.add_element(ElementKind::Led {
            colour: crate::netlist::LedColour::Red,
        });
        let filter = netlist.add_element(ElementKind::Resistor { ohms: 10e3 });
        let capacitor = netlist.add_element(ElementKind::Capacitor {
            farads: 1e-6,
            initial_volts: 0.0,
        });
        let (d13, lit, ground, d6, smooth) = (
            netlist.add_net(),
            netlist.add_net(),
            netlist.add_net(),
            netlist.add_net(),
            netlist.add_net(),
        );
        for (element, pin, net) in [
            (uno, "D13", d13),
            (resistor, "a", d13),
            (resistor, "b", lit),
            (led, "anode", lit),
            (led, "cathode", ground),
            (uno, "GND", ground),
            (uno, "D6", d6),
            (filter, "a", d6),
            (filter, "b", smooth),
            (capacitor, "a", smooth),
            (capacitor, "b", ground),
        ] {
            netlist.connect_named(element, pin, net).unwrap();
        }
        (netlist, d13, smooth, ground)
    }

    /// A net's voltage above the Uno's GND.
    fn above(run: &Scheduler, net: NetId, ground: NetId) -> f64 {
        let engine = run.engine();
        engine.node_voltage(net).unwrap() - engine.node_voltage(ground).unwrap()
    }

    #[test]
    fn blink_toggles_d13_every_half_second_of_simulated_time() {
        let (netlist, d13, _, ground) = bench(&BLINK);
        let mut run = Scheduler::new(engine_for(netlist).unwrap());
        let mut edges = Vec::new();
        let mut was = false;
        for _ in 0..2600 {
            run.run(1).unwrap();
            let high = above(&run, d13, ground) > 2.5;
            if high != was {
                edges.push(run.engine().time());
                was = high;
            }
        }
        // 0, 0.5, 1.0, 1.5, 2.0, 2.5 s, within a step and the loop's
        // overhead.
        assert_eq!(edges.len(), 6, "{edges:?}");
        for (k, at) in edges.iter().enumerate() {
            assert!((at - 0.5 * k as f64).abs() < 3e-3, "{edges:?}");
        }
    }

    #[test]
    fn pwm_through_an_rc_settles_at_its_duty_and_ripples() {
        let (netlist, _, smooth, ground) = bench(&PWM);
        let mut run = Scheduler::new(engine_for(netlist).unwrap());
        // Five time constants to settle.
        run.run(60).unwrap();
        let samples: Vec<f64> = (0..200)
            .map(|_| {
                run.run(1).unwrap();
                above(&run, smooth, ground)
            })
            .collect();
        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        let (low, high) = samples
            .iter()
            .fold((f64::MAX, f64::MIN), |(low, high), &v| {
                (low.min(v), high.max(v))
            });
        // 64/256 of 5 V.
        assert!((mean - 1.25).abs() < 0.05, "{mean}");
        // About 5 V × ¼ × ¾ / (976 Hz × 10 ms) ≈ 96 mV peak to peak, which
        // the 1 ms steps sample at shifting phases of the 1.024 ms period.
        assert!(high - low > 0.04 && high - low < 0.12, "{low}..{high}");
    }

    /// D2 read with its pull-up on and copied to D13, for ever:
    /// `ldi r16,0x20; out DDRB,r16; ldi r16,0x04; out PORTD,r16;
    /// loop: sbic PIND,2; rjmp high; cbi PORTB,5; rjmp loop;
    /// high: sbi PORTB,5; rjmp loop`.
    const FOLLOW: [u16; 10] = [
        0xe200, 0xb904, 0xe004, 0xb90b, 0x994a, 0xc002, 0x982d, 0xcffc, 0x9a2d, 0xcffa,
    ];

    #[test]
    fn an_input_reads_the_circuit_and_the_program_runs_on_through_an_edit() {
        // A switch from D2 to GND, open: the pull-up holds D2 high, so D13
        // is.
        let (mut netlist, d13, _, ground) = bench(&FOLLOW);
        let uno = netlist
            .elements()
            .find(|(_, element)| matches!(element.kind, ElementKind::Uno))
            .unwrap()
            .0;
        let switch = netlist.add_element(ElementKind::Switch { closed: false });
        let d2 = netlist.add_net();
        netlist.connect_named(uno, "D2", d2).unwrap();
        netlist.connect_named(switch, "a", d2).unwrap();
        let gnd = netlist.element(uno).unwrap().pins[super::GND].net.unwrap();
        netlist.connect_named(switch, "b", gnd).unwrap();
        let mut run = Scheduler::new(engine_for(netlist.clone()).unwrap());
        run.run(5).unwrap();
        assert!(above(&run, d13, ground) > 4.0);

        // Closed, as an edit: the program reads D2 low within a step.
        netlist
            .set_kind(switch, ElementKind::Switch { closed: true })
            .unwrap();
        run.engine_mut().adopt(netlist);
        run.run(2).unwrap();
        assert!(above(&run, d13, ground) < 0.5);
    }

    #[test]
    fn an_edit_does_not_restart_the_program() {
        // Blink, edited at 0.3 s: its next edge is still at 0.5 s, not 0.8.
        let (mut netlist, d13, _, ground) = bench(&BLINK);
        let mut run = Scheduler::new(engine_for(netlist.clone()).unwrap());
        run.run(300).unwrap();
        let resistor = netlist
            .elements()
            .find(|(_, element)| matches!(element.kind, ElementKind::Resistor { ohms } if ohms == 220.0))
            .unwrap()
            .0;
        netlist
            .set_kind(resistor, ElementKind::Resistor { ohms: 330.0 })
            .unwrap();
        run.engine_mut().adopt(netlist);
        let mut fell = None;
        for _ in 0..400 {
            run.run(1).unwrap();
            if fell.is_none() && above(&run, d13, ground) < 2.5 {
                fell = Some(run.engine().time());
            }
        }
        let fell = fell.expect("D13 fell");
        assert!((fell - 0.5).abs() < 3e-3, "{fell}");
    }

    /// How fast the two run together, natively: `cargo test --release -p
    /// pastille-core --lib spike_speed -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn spike_speed() {
        for (name, program) in [("blink", &BLINK[..]), ("pwm", &PWM[..])] {
            let (netlist, ..) = bench(program);
            let mut run = Scheduler::new(engine_for(netlist).unwrap());
            let start = std::time::Instant::now();
            run.run(2000).unwrap();
            let wall = start.elapsed().as_secs_f64();
            println!(
                "{name}: 2 s simulated in {wall:.3} s = {:.1}x real time",
                2.0 / wall
            );
        }
    }
}
