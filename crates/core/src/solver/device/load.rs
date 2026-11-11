// SPDX-License-Identifier: MIT OR Apache-2.0

//! The two loads in every starter kit that are not lights: an active buzzer
//! and a small DC motor (C-66).
//!
//! Each is, electrically, a resistance, and whether it is doing what it is for
//! — sounding, turning — is read from the voltage across it. That is all a
//! beginner's circuit asks of them.
//!
//! What they honour: the current each draws; the voltage each needs before it
//! does anything; that an active buzzer is polarised and stays silent reversed.
//! What they ignore: the buzzer's internal oscillator — the app plays its one
//! note while it is sounding (C-71), which is all the tone there is; the motor's
//! inertia, so it spins up at once, and its load, so it draws its unloaded
//! running current whatever it is turning — no stall, no back-EMF that moves.

/// An active buzzer's resistance, in ohms: 25 mA at its rated 5 V.
pub const BUZZER_OHMS: f64 = 200.0;

/// The least voltage it sounds at, the right way round.
pub const BUZZER_SOUNDS_FROM: f64 = 3.0;

/// A 130 motor's resistance running unloaded, in ohms: 200 mA at 3 V.
pub const MOTOR_OHMS: f64 = 15.0;

/// The least voltage it turns at, either way round.
pub const MOTOR_TURNS_FROM: f64 = 1.0;

/// Whether a buzzer with `volts` from `+` to `−` is sounding.
pub fn buzzer_sounds(volts: f64) -> bool {
    volts >= BUZZER_SOUNDS_FROM
}

/// Whether a motor with `volts` across it is turning.
pub fn motor_turns(volts: f64) -> bool {
    volts.abs() >= MOTOR_TURNS_FROM
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_buzzer_sounds_the_right_way_round_and_no_other() {
        assert!(buzzer_sounds(5.0));
        assert!(!buzzer_sounds(2.0), "under its least voltage");
        assert!(!buzzer_sounds(-9.0), "reversed");
    }

    #[test]
    fn a_motor_turns_either_way() {
        assert!(motor_turns(3.0) && motor_turns(-3.0));
        assert!(!motor_turns(0.5));
    }
}
