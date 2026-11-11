// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

/// D13 toggled every half second (A-01's hand-assembled Blink).
const BLINK: [u16; 11] = [
    0xe200, 0xb904, 0xb903, 0xe228, 0xe580, 0xec93, 0x9701, 0xf7f1, 0x952a, 0xf7d1, 0xcff7,
];

/// `words` as the Intel HEX the Arduino IDE exports: sixteen bytes a
/// line, little-endian words, each line's checksum, and the end record.
fn intel_hex(words: &[u16]) -> String {
    let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
    let mut out = String::new();
    for (line, chunk) in bytes.chunks(16).enumerate() {
        let address = (line * 16) as u16;
        let mut record = vec![chunk.len() as u8, (address >> 8) as u8, address as u8, 0];
        record.extend_from_slice(chunk);
        let sum = record.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte));
        record.push(sum.wrapping_neg());
        out.push(':');
        for byte in record {
            out.push_str(&format!("{byte:02X}"));
        }
        out.push('\n');
    }
    out.push_str(":00000001FF\n");
    out
}

/// An Uno with an LED behind 220 Ω on D13: the Uno, and the nets of D13
/// and of its GND.
fn blinking() -> (Circuit, String, String, String) {
    let mut circuit = Circuit::new();
    let uno = circuit.try_add("Uno", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 100.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 200.0, 0.0).unwrap();
    circuit.try_set_value(&resistor, "220").unwrap();
    let d13 = circuit.try_connect(&uno, "D13", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    let ground = circuit.try_connect(&led, "cathode", &uno, "GND").unwrap();
    (circuit, uno, d13, ground)
}

/// D13's voltage above GND, after running `seconds` more.
fn d13_after(circuit: &mut Circuit, seconds: f64, d13: &str, ground: &str) -> f64 {
    // A frame at a time: one call makes up a quarter of a second at most.
    for _ in 0..(seconds / 0.01).round() as usize {
        circuit.advance_by(0.01).unwrap();
    }
    let solution = circuit.solution();
    let volts = |net: &str| {
        solution
            .net_voltages
            .iter()
            .find(|voltage| voltage.net_id == net)
            .map_or(0.0, |voltage| voltage.volts)
    };
    volts(d13) - volts(ground)
}

#[test]
fn a_hex_loaded_into_an_uno_runs_and_its_led_blinks() {
    let (mut circuit, uno, d13, ground) = blinking();
    circuit
        .try_set_program(&uno, "Blink.hex", &intel_hex(&BLINK))
        .unwrap();
    // High from the start, low after half a second, high again after one.
    assert!(d13_after(&mut circuit, 0.25, &d13, &ground) > 4.0);
    assert!(d13_after(&mut circuit, 0.5, &d13, &ground) < 0.5);
    assert!(d13_after(&mut circuit, 0.5, &d13, &ground) > 4.0);
    let views = circuit.views();
    let shown = views.iter().find(|element| element.id == uno).unwrap();
    assert_eq!(shown.program.as_deref(), Some("Blink.hex"));
}

#[test]
fn an_uno_s_gnd_is_the_circuit_s_zero_volts() {
    // A pin high reads 5 V on its own, as a meter on the bench would
    // read it against GND — not D13's net taken as zero.
    let (mut circuit, uno, d13, ground) = blinking();
    circuit
        .try_set_program(&uno, "Blink.hex", &intel_hex(&BLINK))
        .unwrap();
    circuit.advance_by(0.05).unwrap();
    let solution = circuit.solution();
    let volts = |net: &str| {
        solution
            .net_voltages
            .iter()
            .find(|voltage| voltage.net_id == net)
            .unwrap()
            .volts
    };
    assert_eq!(volts(&ground), 0.0);
    assert!(volts(&d13) > 4.0, "{}", volts(&d13));
}

#[test]
fn arduino_s_blink_lights_d13_a_second_on_a_second_off() {
    // A real Arduino build (programs/ at the root): `digitalWrite` and
    // `delay(1000)`, so the core's start-up, Timer0's interrupt and
    // `millis()` all run on the emulated chip.
    let (mut circuit, uno, d13, ground) = blinking();
    circuit
        .try_set_program(
            &uno,
            "Blink.hex",
            include_str!("../../../../programs/Blink.hex"),
        )
        .unwrap();
    let mut edges = Vec::new();
    let mut was = false;
    for _ in 0..450 {
        let high = d13_after(&mut circuit, 0.01, &d13, &ground) > 2.5;
        if high != was {
            edges.push(circuit.run_time());
            was = high;
        }
    }
    // On at once (after the core's start-up), then every second.
    assert_eq!(edges.len(), 5, "{edges:?}");
    for (k, at) in edges.iter().enumerate() {
        assert!((at - k as f64).abs() < 0.03, "{edges:?}");
    }
}

#[test]
fn arduino_s_button_reads_d2_through_its_pull_up() {
    // `pinMode(2, INPUT_PULLUP)`, and D13 lit while D2 reads low: a
    // switch from D2 to GND, open then shut.
    let (mut circuit, uno, d13, ground) = blinking();
    let switch = circuit.try_add("Switch", 0.0, 200.0).unwrap();
    circuit.try_connect(&uno, "D2", &switch, "a").unwrap();
    circuit.try_connect(&switch, "b", &uno, "GND").unwrap();
    circuit
        .try_set_program(
            &uno,
            "Button.hex",
            include_str!("../../../../programs/Button.hex"),
        )
        .unwrap();
    assert!(d13_after(&mut circuit, 0.05, &d13, &ground) < 0.5);
    circuit.try_set_closed(&switch, true).unwrap();
    assert!(d13_after(&mut circuit, 0.02, &d13, &ground) > 4.0);
    circuit.try_set_closed(&switch, false).unwrap();
    assert!(d13_after(&mut circuit, 0.02, &d13, &ground) < 0.5);
}

/// An Uno running one of `programs/`, and `leds` red LEDs each behind
/// 220 Ω from its pin to GND: the circuit, the Uno, and each resistor.
fn running(program: &str, hex: &str, leds: &[&str]) -> (Circuit, String, Vec<String>) {
    let mut circuit = Circuit::new();
    let uno = circuit.try_add("Uno", 0.0, 0.0).unwrap();
    let mut resistors = Vec::new();
    for (row, pin) in leds.iter().enumerate() {
        let resistor = circuit
            .try_add("Resistor", 100.0, row as f64 * 60.0)
            .unwrap();
        let led = circuit.try_add("Led", 200.0, row as f64 * 60.0).unwrap();
        circuit.try_set_value(&resistor, "220").unwrap();
        circuit.try_connect(&uno, pin, &resistor, "a").unwrap();
        circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
        circuit.try_connect(&led, "cathode", &uno, "GND").unwrap();
        resistors.push(resistor);
    }
    circuit.try_set_program(&uno, program, hex).unwrap();
    (circuit, uno, resistors)
}

/// The average current through `id` over the next `seconds`, sampled at
/// every millisecond: PWM is on and off, and an LED's light is the average.
fn average_amps(circuit: &mut Circuit, id: &str, seconds: f64) -> f64 {
    let samples = (seconds / 1e-3).round() as usize;
    let mut total = 0.0;
    for _ in 0..samples {
        circuit.advance_by(1e-3).unwrap();
        total += circuit
            .solution()
            .element_currents
            .iter()
            .find(|current| current.element_id == id)
            .map_or(0.0, |current| current.amps.abs());
    }
    total / samples as f64
}

#[test]
fn arduino_s_fade_swaps_two_leds_brightness_with_pwm() {
    // A-05: D9 climbs from off while D10 falls from full, 5 of 255 every
    // 20 ms; half a second in, D9 is at about half and D10 likewise.
    let (mut circuit, _, resistors) = running(
        "Fade",
        include_str!("../../../../programs/Fade.hex"),
        &["D9", "D10"],
    );
    let d9_early = average_amps(&mut circuit, &resistors[0], 0.1);
    circuit.advance_by(0.25).ok();
    let d9_later = average_amps(&mut circuit, &resistors[0], 0.1);
    let d10_later = average_amps(&mut circuit, &resistors[1], 0.1);
    assert!(d9_early < 0.004, "{d9_early}");
    assert!(d9_later > 2.0 * d9_early, "{d9_early} then {d9_later}");
    assert!(d10_later > 0.004, "{d10_later}");
}

#[test]
fn arduino_s_smoothed_pwm_settles_at_a_quarter_and_ripples() {
    // A-05: D9 at 64/255 through 10 kΩ into 1 µF — about 1.25 V, with
    // 490 Hz ripple on it.
    let mut circuit = Circuit::new();
    let uno = circuit.try_add("Uno", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 100.0, 0.0).unwrap();
    let capacitor = circuit.try_add("Capacitor", 200.0, 0.0).unwrap();
    circuit.try_set_value(&resistor, "10k").unwrap();
    circuit.try_set_value(&capacitor, "1u").unwrap();
    circuit.try_connect(&uno, "D9", &resistor, "a").unwrap();
    let smooth = circuit
        .try_connect(&resistor, "b", &capacitor, "a")
        .unwrap();
    circuit.try_connect(&capacitor, "b", &uno, "GND").unwrap();
    circuit
        .try_set_program(
            &uno,
            "Smoothed",
            include_str!("../../../../programs/Smoothed.hex"),
        )
        .unwrap();
    for _ in 0..10 {
        circuit.advance_by(0.01).unwrap();
    }
    let mut samples = Vec::new();
    for _ in 0..200 {
        circuit.advance_by(1e-4).unwrap();
        let solution = circuit.solution();
        samples.push(
            solution
                .net_voltages
                .iter()
                .find(|voltage| voltage.net_id == smooth)
                .unwrap()
                .volts,
        );
    }
    let mean = samples.iter().sum::<f64>() / samples.len() as f64;
    let (low, high) = samples
        .iter()
        .fold((f64::MAX, f64::MIN), |(low, high), &v| {
            (low.min(v), high.max(v))
        });
    // 64/255 of 5 V less what 26 Ω drops: about 1.25 V.
    assert!((mean - 1.25).abs() < 0.08, "{mean}");
    // 5 V × ¼ × ¾ / (490 Hz × 10 ms) ≈ 0.19 V peak to peak.
    assert!(high - low > 0.1 && high - low < 0.3, "{low}..{high}");
}

#[test]
fn arduino_s_traffic_light_goes_green_amber_red() {
    let (mut circuit, _, resistors) = running(
        "TrafficLight",
        include_str!("../../../../programs/TrafficLight.hex"),
        &["D12", "D11", "D10"],
    );
    let lit = |circuit: &Circuit| -> Vec<bool> {
        let solution = circuit.solution();
        resistors
            .iter()
            .map(|id| {
                solution
                    .element_currents
                    .iter()
                    .find(|current| &current.element_id == id)
                    .is_some_and(|current| current.amps.abs() > 0.005)
            })
            .collect()
    };
    let after = |circuit: &mut Circuit, seconds: f64| {
        for _ in 0..(seconds / 0.01).round() as usize {
            circuit.advance_by(0.01).unwrap();
        }
    };
    // Red, amber, green.
    after(&mut circuit, 1.0);
    assert_eq!(lit(&circuit), [false, false, true]);
    after(&mut circuit, 1.25);
    assert_eq!(lit(&circuit), [false, true, false]);
    after(&mut circuit, 1.0);
    assert_eq!(lit(&circuit), [true, false, false]);
}

#[test]
fn arduino_s_sweep_turns_a_servo() {
    // The Servo library's pulses on D9: the horn climbs a degree every
    // 10 ms from 0.
    let mut circuit = Circuit::new();
    let uno = circuit.try_add("Uno", 0.0, 0.0).unwrap();
    let servo = circuit.try_add("Servo", 100.0, 0.0).unwrap();
    circuit.try_connect(&uno, "D9", &servo, "SIG").unwrap();
    circuit.try_connect(&uno, "5V", &servo, "+5V").unwrap();
    circuit.try_connect(&servo, "GND", &uno, "GND").unwrap();
    circuit
        .try_set_program(
            &uno,
            "Sweep",
            include_str!("../../../../programs/Sweep.hex"),
        )
        .unwrap();
    let angle = |circuit: &mut Circuit, seconds: f64| {
        for _ in 0..(seconds / 0.01).round() as usize {
            circuit.advance_by(0.01).unwrap();
        }
        circuit
            .solution()
            .angles
            .iter()
            .find(|entry| entry.element_id == servo)
            .map_or(f64::NAN, |entry| entry.degrees)
    };
    let early = angle(&mut circuit, 0.5);
    let later = angle(&mut circuit, 0.5);
    assert!(later > early + 20.0, "{early} then {later}");
}

#[test]
fn loading_a_program_is_one_step_of_undo() {
    let (mut circuit, uno, d13, ground) = blinking();
    circuit
        .try_set_program(&uno, "Blink.hex", &intel_hex(&BLINK))
        .unwrap();
    assert!(circuit.undo());
    assert!(circuit.netlist.program(decode(&uno).unwrap()).is_none());
    assert!(d13_after(&mut circuit, 0.25, &d13, &ground) < 0.5);
}

#[test]
fn a_program_is_saved_with_its_circuit() {
    let (mut circuit, uno, ..) = blinking();
    circuit
        .try_set_program(&uno, "Blink.hex", &intel_hex(&BLINK))
        .unwrap();
    let text = serde_json::to_string(&circuit.to_document()).unwrap();
    assert!(text.contains("\"name\":\"Blink.hex\""), "{text}");
    let mut reopened = Circuit::new();
    reopened.try_open(&text).unwrap();
    let (id, _) = reopened
        .netlist
        .elements()
        .find(|(_, element)| matches!(element.kind, ElementKind::Uno))
        .unwrap();
    let program = reopened.netlist.program(id).unwrap();
    assert_eq!(program.name, "Blink.hex");
    assert_eq!(&program.words[..BLINK.len()], &BLINK[..]);
}

#[test]
fn a_file_that_is_not_a_program_is_refused_with_how_to_get_one() {
    let (mut circuit, uno, ..) = blinking();
    let error = circuit
        .try_set_program(&uno, "notes.txt", "not a hex file")
        .unwrap_err();
    let CircuitError::Refused(sentence) = error else {
        panic!("{error:?}");
    };
    assert!(sentence.contains("Export Compiled Binary"), "{sentence}");
    assert!(circuit.netlist.program(decode(&uno).unwrap()).is_none());
}

#[test]
fn reset_starts_the_program_again() {
    // At 0.75 s Blink is low; reset there, and 0.25 s later it is high
    // again, as at its start.
    let (mut circuit, uno, d13, ground) = blinking();
    circuit
        .try_set_program(&uno, "Blink.hex", &intel_hex(&BLINK))
        .unwrap();
    assert!(d13_after(&mut circuit, 0.75, &d13, &ground) < 0.5);
    circuit.try_reset_program(&uno).unwrap();
    assert!(d13_after(&mut circuit, 0.25, &d13, &ground) > 4.0);
}

/// A 10 kΩ potentiometer across the Uno's 5V and GND, its wiper on A0
/// and turned to `wiper`: 0 is 0 V.
fn pot_on_a0(circuit: &mut Circuit, uno: &str, wiper: f64) -> String {
    let pot = circuit.try_add("Potentiometer", 100.0, 200.0).unwrap();
    circuit.try_set_value(&pot, "10k").unwrap();
    circuit.try_connect(&pot, "a", uno, "GND").unwrap();
    circuit.try_connect(&pot, "b", uno, "5V").unwrap();
    circuit.try_connect(&pot, "wiper", uno, "A0").unwrap();
    circuit.try_set_wiper(&pot, wiper).unwrap();
    pot
}

#[test]
fn arduino_s_analog_input_blinks_as_fast_as_the_pot_says() {
    // A-04: `delay(analogRead(A0))` between each change of D13. A fifth
    // of the way round is 1 V, which reads 204: on 204 ms, off 204 ms.
    // Turned to four fifths, 818.
    let (mut circuit, uno, d13, ground) = blinking();
    let pot = pot_on_a0(&mut circuit, &uno, 0.2);
    circuit
        .try_set_program(
            &uno,
            "AnalogInput",
            include_str!("../../../../programs/AnalogInput.hex"),
        )
        .unwrap();
    let halves = |circuit: &mut Circuit, seconds: f64| {
        let mut edges = Vec::new();
        let mut was = d13_after(circuit, 0.0, &d13, &ground) > 2.5;
        for _ in 0..(seconds / 0.002).round() as usize {
            circuit.advance_by(0.002).unwrap();
            let solution = circuit.solution();
            let volts = |net: &str| {
                solution
                    .net_voltages
                    .iter()
                    .find(|voltage| voltage.net_id == net)
                    .map_or(0.0, |voltage| voltage.volts)
            };
            let high = volts(&d13) - volts(&ground) > 2.5;
            if high != was {
                edges.push(circuit.run_time());
                was = high;
            }
        }
        edges
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .collect::<Vec<_>>()
    };
    let fast = halves(&mut circuit, 1.5);
    assert!(fast.len() >= 5, "{fast:?}");
    for half in &fast {
        assert!((half - 0.204).abs() < 0.005, "{fast:?}");
    }

    circuit.try_set_wiper(&pot, 0.8).unwrap();
    // Out of the wait it was in when the knob turned.
    halves(&mut circuit, 0.5);
    let slow = halves(&mut circuit, 3.0);
    assert!(slow.len() >= 2, "{slow:?}");
    for half in &slow {
        assert!((half - 0.818).abs() < 0.005, "{slow:?}");
    }
}

#[test]
fn arduino_s_dimmer_sets_an_led_by_the_pot() {
    // `analogWrite(9, analogRead(A0) / 4)`: a quarter of the way round
    // is on 63/255 of the time, three quarters 191/255 — three times
    // the light.
    let (mut circuit, uno, resistors) = running(
        "Dimmer",
        include_str!("../../../../programs/Dimmer.hex"),
        &["D9"],
    );
    let pot = pot_on_a0(&mut circuit, &uno, 0.25);
    circuit.advance_by(0.05).unwrap();
    let dim = average_amps(&mut circuit, &resistors[0], 0.1);
    circuit.try_set_wiper(&pot, 0.75).unwrap();
    circuit.advance_by(0.05).unwrap();
    let bright = average_amps(&mut circuit, &resistors[0], 0.1);
    assert!(dim > 0.001, "{dim}");
    assert!((bright / dim - 3.0).abs() < 0.3, "{dim} then {bright}");
}

#[test]
fn arduino_s_night_light_comes_on_in_the_dark() {
    // An LDR from A0 to GND under 10 kΩ from 5V: a lit room (30 lux,
    // ~4.5 kΩ) reads about 320, out; a dark one (1 lux, ~50 kΩ) about
    // 850, lit.
    let (mut circuit, uno, d13, ground) = blinking();
    let top = circuit.try_add("Resistor", 100.0, 200.0).unwrap();
    circuit.try_set_value(&top, "10k").unwrap();
    let ldr = circuit.try_add("Ldr", 100.0, 300.0).unwrap();
    circuit.try_connect(&top, "a", &uno, "5V").unwrap();
    circuit.try_connect(&top, "b", &uno, "A0").unwrap();
    circuit.try_connect(&ldr, "a", &uno, "A0").unwrap();
    circuit.try_connect(&ldr, "b", &uno, "GND").unwrap();
    circuit.try_set_wiper(&ldr, 0.5).unwrap();
    circuit
        .try_set_program(
            &uno,
            "NightLight",
            include_str!("../../../../programs/NightLight.hex"),
        )
        .unwrap();
    assert!(d13_after(&mut circuit, 1.0, &d13, &ground) < 0.5);
    circuit.try_set_wiper(&ldr, 0.2).unwrap();
    assert!(d13_after(&mut circuit, 1.5, &d13, &ground) > 4.0);
    circuit.try_set_wiper(&ldr, 0.5).unwrap();
    assert!(d13_after(&mut circuit, 1.5, &d13, &ground) < 0.5);
}

#[test]
fn arduino_s_thermometer_reads_a_tmp36_against_1v1() {
    // `analogReference(INTERNAL)`: 27 °C is 770 mV, which reads 716
    // against 1.1 V and counts as 26 °C — two LEDs; 38 °C, four. Against
    // 5 V it would read 157 and light none.
    let (mut circuit, uno, resistors) = running(
        "Thermometer",
        include_str!("../../../../programs/Thermometer.hex"),
        &["D2", "D3", "D4", "D5", "D6"],
    );
    let sensor = circuit.try_add("Tmp36", 100.0, 400.0).unwrap();
    circuit.try_connect(&sensor, "+Vs", &uno, "5V").unwrap();
    circuit.try_connect(&sensor, "GND", &uno, "GND").unwrap();
    circuit.try_connect(&sensor, "Vout", &uno, "A0").unwrap();
    let lit = |circuit: &mut Circuit, celsius: f64| {
        circuit
            .try_set_wiper(&sensor, crate::solver::device::tmp36::dial_for(celsius))
            .unwrap();
        for _ in 0..30 {
            circuit.advance_by(0.01).unwrap();
        }
        let solution = circuit.solution();
        resistors
            .iter()
            .map(|id| {
                solution
                    .element_currents
                    .iter()
                    .find(|current| &current.element_id == id)
                    .is_some_and(|current| current.amps.abs() > 1e-3)
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(lit(&mut circuit, 27.0), [true, true, false, false, false]);
    assert_eq!(lit(&mut circuit, 38.0), [true, true, true, true, false]);
}

#[test]
fn an_uno_s_sample_and_hold_carries_charge_from_one_pin_to_the_next() {
    // TwoPots reads A0, then A1, five times a second. A1 is at 0 V; A0
    // is fed through 100 MΩ with 100 pF on it, a filter, and sits at
    // 4.55 V against the input's leak. Each time the converter moves
    // from A1 back to A0, its 14 pF brings A1's 0 V with it and shares
    // it with the filter: A0 dips to about 4.55 × 100 / 114 = 4.0 V and
    // climbs back over 100 MΩ × 114 pF ≈ 11 ms. No dip at all if it were
    // nothing. (The sheet asks for a source under 10 kΩ, where it is
    // over in nanoseconds.)
    let mut circuit = Circuit::new();
    let uno = circuit.try_add("Uno", 0.0, 0.0).unwrap();
    let source = circuit.try_add("Resistor", 100.0, 0.0).unwrap();
    circuit.try_set_value(&source, "100M").unwrap();
    circuit.try_connect(&source, "a", &uno, "5V").unwrap();
    let a0 = circuit.try_connect(&source, "b", &uno, "A0").unwrap();
    let earth = circuit.try_add("Ground", 0.0, 200.0).unwrap();
    circuit.try_connect(&earth, "gnd", &uno, "GND").unwrap();
    circuit.try_connect(&uno, "A1", &uno, "GND").unwrap();
    let filter = circuit.try_add("Capacitor", 100.0, 100.0).unwrap();
    circuit.try_set_value(&filter, "100p").unwrap();
    circuit.try_connect(&filter, "a", &uno, "A0").unwrap();
    circuit.try_connect(&filter, "b", &uno, "GND").unwrap();
    circuit
        .try_set_program(
            &uno,
            "TwoPots",
            include_str!("../../../../programs/TwoPots.hex"),
        )
        .unwrap();
    let mut samples = Vec::new();
    for _ in 0..500 {
        circuit.advance_by(1e-3).unwrap();
        let solution = circuit.solution();
        samples.push(
            solution
                .net_voltages
                .iter()
                .find(|voltage| voltage.net_id == a0)
                .map_or(0.0, |voltage| voltage.volts),
        );
    }
    // The filter charges from 0 V at first, over 11 ms.
    let samples = &samples[100..];
    let dips = samples
        .windows(2)
        .filter(|pair| pair[1] < pair[0] - 0.3)
        .count();
    assert!((2..=3).contains(&dips), "{dips}");
    let low = samples.iter().copied().fold(f64::MAX, f64::min);
    assert!((low - 4.0).abs() < 0.15, "{low}");
    let last = *samples.last().unwrap();
    assert!((last - 4.55).abs() < 0.05, "{last}");
}

/// What the monitor has heard after `seconds` more, a frame at a time.
fn serial_after(circuit: &mut Circuit, uno: &str, seconds: f64) -> String {
    let mut bytes = Vec::new();
    for _ in 0..(seconds / 0.01).round() as usize {
        circuit.advance_by(0.01).unwrap();
        bytes.extend(circuit.try_take_serial(uno).unwrap());
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

#[test]
fn arduino_s_analog_read_serial_prints_the_pot() {
    // A-06: `Serial.println(analogRead(A0))` ten times a second at
    // 9600, the monitor's own: half way round is 2.5 V, 511 or 512.
    let mut circuit = Circuit::new();
    let uno = circuit.try_add("Uno", 0.0, 0.0).unwrap();
    let pot = pot_on_a0(&mut circuit, &uno, 0.5);
    circuit
        .try_set_program(
            &uno,
            "AnalogReadSerial",
            include_str!("../../../../programs/AnalogReadSerial.hex"),
        )
        .unwrap();
    let text = serial_after(&mut circuit, &uno, 0.55);
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.len() >= 5, "{text:?}");
    for line in &lines {
        assert!(*line == "511" || *line == "512", "{text:?}");
    }
    circuit.try_set_wiper(&pot, 0.25).unwrap();
    let text = serial_after(&mut circuit, &uno, 0.3);
    assert!(
        text.lines().last() == Some("255") || text.lines().last() == Some("256"),
        "{text:?}"
    );
    // The program's baud, as the panel shows it: 16 MHz / (8 × 208).
    let serial = circuit.solution().serial;
    assert_eq!(serial.len(), 1);
    assert_eq!(serial[0].monitor_baud, 9600);
    assert_eq!(serial[0].program_baud, Some(9615));
}

#[test]
fn arduino_s_serial_led_does_what_it_is_told() {
    // A line typed into the monitor reaches `Serial.read()`; the reply
    // comes back.
    let (mut circuit, uno, d13, ground) = blinking();
    circuit
        .try_set_program(
            &uno,
            "SerialLed",
            include_str!("../../../../programs/SerialLed.hex"),
        )
        .unwrap();
    assert_eq!(serial_after(&mut circuit, &uno, 0.1), "Type on or off.\r\n");
    circuit.try_send_serial(&uno, "on\n").unwrap();
    assert_eq!(serial_after(&mut circuit, &uno, 0.1), "D13 is on.\r\n");
    assert!(d13_after(&mut circuit, 0.0, &d13, &ground) > 4.0);
    circuit.try_send_serial(&uno, "maybe\n").unwrap();
    assert!(serial_after(&mut circuit, &uno, 0.1).contains("I do not know \"maybe\""));
    circuit.try_send_serial(&uno, "off\n").unwrap();
    assert_eq!(serial_after(&mut circuit, &uno, 0.1), "D13 is off.\r\n");
    assert!(d13_after(&mut circuit, 0.0, &d13, &ground) < 0.5);

    // Typed at the wrong baud, the program hears garbage and D13 stays.
    circuit.try_set_monitor_baud(&uno, 115_200).unwrap();
    circuit.try_send_serial(&uno, "on\n").unwrap();
    serial_after(&mut circuit, &uno, 0.1);
    assert!(d13_after(&mut circuit, 0.0, &d13, &ground) < 0.5);
}

#[test]
fn a_monitor_at_the_wrong_baud_reads_garbage_and_the_right_one_reads_text() {
    // SerialThermometer prints at 115200; the monitor starts at 9600.
    let mut circuit = Circuit::new();
    let uno = circuit.try_add("Uno", 0.0, 0.0).unwrap();
    let sensor = circuit.try_add("Tmp36", 100.0, 400.0).unwrap();
    circuit.try_connect(&sensor, "+Vs", &uno, "5V").unwrap();
    circuit.try_connect(&sensor, "GND", &uno, "GND").unwrap();
    circuit.try_connect(&sensor, "Vout", &uno, "A0").unwrap();
    circuit
        .try_set_wiper(&sensor, crate::solver::device::tmp36::dial_for(27.0))
        .unwrap();
    circuit
        .try_set_program(
            &uno,
            "SerialThermometer",
            include_str!("../../../../programs/SerialThermometer.hex"),
        )
        .unwrap();
    let garbage = serial_after(&mut circuit, &uno, 1.1);
    assert!(!garbage.contains("°C"), "{garbage:?}");
    assert!(!garbage.contains("27"), "{garbage:?}");

    circuit.try_set_monitor_baud(&uno, 115_200).unwrap();
    // Out of whatever line it was in the middle of.
    serial_after(&mut circuit, &uno, 0.6);
    let text = serial_after(&mut circuit, &uno, 1.1);
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.len() >= 2, "{text:?}");
    for line in lines {
        // 770 mV reads 716 against 1.1 V: 769.1 mV, 26.9 °C.
        assert_eq!(line, "26.9 °C", "{text:?}");
    }
}

#[test]
fn while_serial_is_begun_d1_is_driven_high_and_d0_listens() {
    // AnalogReadSerial never touches D0 or D1's DDR: the USART takes
    // them. D1 idles high behind its output resistance into 1 kΩ; D0,
    // held up by the USB chip's 1 kΩ, is an input.
    let mut circuit = Circuit::new();
    let uno = circuit.try_add("Uno", 0.0, 0.0).unwrap();
    let load = circuit.try_add("Resistor", 100.0, 0.0).unwrap();
    circuit.try_set_value(&load, "1k").unwrap();
    let d1 = circuit.try_connect(&uno, "D1", &load, "a").unwrap();
    let ground = circuit.try_connect(&load, "b", &uno, "GND").unwrap();
    let d1_volts = |circuit: &mut Circuit| d13_after(circuit, 0.05, &d1, &ground);
    assert!(d1_volts(&mut circuit) < 0.1);
    pot_on_a0(&mut circuit, &uno, 0.5);
    circuit
        .try_set_program(
            &uno,
            "AnalogReadSerial",
            include_str!("../../../../programs/AnalogReadSerial.hex"),
        )
        .unwrap();
    // 5 V × 1 kΩ / (1 kΩ + 26 Ω).
    assert!((d1_volts(&mut circuit) - 4.87).abs() < 0.05);
}

#[test]
fn an_uno_with_no_program_powers_the_circuit_and_drives_nothing() {
    // A-02: placed from the palette, its 5V lights an LED through 220 Ω;
    // D13, an input with no program, takes nothing from a second one.
    let mut circuit = Circuit::new();
    let uno = circuit.try_add("Uno", 0.0, 0.0).unwrap();
    let resistor = circuit.try_add("Resistor", 100.0, 0.0).unwrap();
    let led = circuit.try_add("Led", 200.0, 0.0).unwrap();
    let idle = circuit.try_add("Resistor", 100.0, 80.0).unwrap();
    let dark = circuit.try_add("Led", 200.0, 80.0).unwrap();
    circuit.try_set_value(&resistor, "220").unwrap();
    circuit.try_set_value(&idle, "220").unwrap();
    circuit.try_connect(&uno, "5V", &resistor, "a").unwrap();
    circuit.try_connect(&resistor, "b", &led, "anode").unwrap();
    circuit.try_connect(&led, "cathode", &uno, "GND").unwrap();
    circuit.try_connect(&uno, "D13", &idle, "a").unwrap();
    circuit.try_connect(&idle, "b", &dark, "anode").unwrap();
    circuit.try_connect(&dark, "cathode", &uno, "GND").unwrap();
    circuit.advance_by(0.05).unwrap();
    let solution = circuit.solution();
    assert!(solution.failure.is_none(), "{:?}", solution.failure);
    let amps = |id: &str| {
        solution
            .element_currents
            .iter()
            .find(|current| current.element_id == id)
            .map(|current| current.amps.abs())
            .unwrap()
    };
    // (5 V less a red LED's ~1.9 V) over 220 Ω and the supply's 0.5 Ω.
    assert!(
        (0.012..0.016).contains(&amps(&resistor)),
        "{}",
        amps(&resistor)
    );
    assert!(amps(&idle) < 1e-6, "{}", amps(&idle));
}
