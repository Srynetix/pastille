// SPDX-License-Identifier: MIT OR Apache-2.0

//! A magnet, and the two parts that feel it: a reed switch and an A3144 Hall
//! sensor (C-115).
//!
//! **The dial is a distance**, from one magnet: the 10 mm by 3 mm neodymium
//! disc in every sensor kit, taken as N35 (a remanence of 1.2 T). Neither part
//! answers to "a magnet": a Hall sensor switches at a flux density and a reed
//! is rated in the ampere-turns of a test coil, so the field each one feels is
//! worked out from where the magnet is — on the disc's axis, by the textbook
//! formula for a uniformly magnetised cylinder,
//!
//! B(z) = Br/2 · ((D + z)/√(R² + (D + z)²) − z/√(R² + z²)),
//!
//! 309 mT touching, 23 mT at 10 mm, 4 mT at 20 mm. Off the axis, sideways,
//! or the other pole facing, is not modelled: the magnet is always square on.
//!
//! **The reed switch** closes at 5 mT, which with this magnet is about
//! 18.5 mm. An estimate, and said: reeds are sold by the ampere-turns that
//! close them in a test coil, not by a field, and 5 mT is where a common small
//! glass reed of some 15 AT is said to close. Its hysteresis — it lets go at a
//! weaker field than it closes at — is not modelled: it is a contact that
//! follows the dial.
//!
//! **The A3144** (Allegro, 27621.6B, read): an open-collector output that
//! pulls low while the field passes its operate point and lets go once it
//! falls below its release point. The sheet gives the operate point only as
//! 7 to 35 mT at 25 °C, with no typical; the middle, 21 mT, is taken, and the
//! release point is its typical 5.5 mT of hysteresis below, 15.5 mT — so about
//! 10.5 mm closer to switch on, about 12 mm away to switch off, and in between
//! it stays as it was. Its output saturates at 175 mV at 20 mA (some 9 Ω) and
//! leaks under a microamp off; it runs on 4.5 to 24 V and draws 4.4 mA, taken
//! here as the resistance that draws it at 5 V, so it draws more than the real
//! part does from a higher supply. 25 mA out at most.

/// The furthest the dial puts the magnet, in millimetres.
pub const FARTHEST_MM: f64 = 30.0;

/// The magnet's remanence in tesla, and its radius and thickness in mm.
const REMANENCE: f64 = 1.2;
const RADIUS_MM: f64 = 5.0;
const THICKNESS_MM: f64 = 3.0;

/// Where the dial puts the magnet: 0 is as far as it goes, 1 is touching.
pub fn distance_mm(dial: f64) -> f64 {
    FARTHEST_MM * (1.0 - dial.clamp(0.0, 1.0))
}

/// The field at the part from a magnet `mm` away on its axis, in mT.
pub fn field_at(mm: f64) -> f64 {
    let z = mm.max(0.0);
    let (r, d) = (RADIUS_MM, THICKNESS_MM);
    1000.0 * REMANENCE / 2.0
        * ((d + z) / (r * r + (d + z) * (d + z)).sqrt() - z / (r * r + z * z).sqrt())
}

/// The field the dial gives, in mT.
pub fn field(dial: f64) -> f64 {
    field_at(distance_mm(dial))
}

/// The field a reed switch closes at, in mT: an estimate (see the module).
pub const REED_CLOSES: f64 = 5.0;

/// Whether a reed switch with the magnet where `dial` puts it is shut.
pub fn reed_shut(dial: f64) -> bool {
    field(dial) >= REED_CLOSES
}

/// The A3144's operate and release points, in mT.
pub const OPERATE: f64 = 21.0;
pub const RELEASE: f64 = 15.5;

/// Pin order: the sheet's, viewed from the branded side.
pub const PINS: [&str; 3] = ["VCC", "GND", "OUT"];
pub const VCC: usize = 0;
pub const GND: usize = 1;
pub const OUT: usize = 2;

/// What is on the part.
pub const PART: &str = "A3144";

/// Behind its output while it pulls low, in ohms: 175 mV at 20 mA.
pub const ON_OHMS: f64 = 8.75;
/// Its output let go, in siemens: under a microamp at 24 V.
pub const OFF_LEAK: f64 = 1e-9;
/// Its supply, in ohms: 4.4 mA at 5 V.
pub const SUPPLY_OHMS: f64 = 5.0 / 4.4e-3;
/// It does nothing below this supply, in volts.
pub const LEAST_SUPPLY: f64 = 4.5;
/// Nor above this one.
pub const MOST_SUPPLY: f64 = 24.0;

/// Whether the A3144 pulls its output low after a step: on past the operate
/// point, off under the release point, as it was in between; never without
/// its supply.
pub fn hall_on(was_on: bool, dial: f64, supply: f64) -> bool {
    if supply < LEAST_SUPPLY {
        return false;
    }
    let field = field(dial);
    if field >= OPERATE {
        true
    } else if field <= RELEASE {
        false
    } else {
        was_on
    }
}

/// The dial for a magnet `mm` away.
pub fn dial_for(mm: f64) -> f64 {
    (1.0 - mm / FARTHEST_MM).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_field_falls_away_as_the_textbook_says() {
        // 309 mT touching, 23 mT at 10 mm, 4.2 mT at 20 mm, 1.4 mT at 30.
        for (mm, mt) in [(0.0, 308.7), (10.0, 23.35), (20.0, 4.22), (30.0, 1.39)] {
            assert!(
                (field_at(mm) - mt).abs() < 0.05,
                "{} mT at {mm} mm",
                field_at(mm)
            );
        }
        assert_eq!(distance_mm(0.0), FARTHEST_MM);
        assert_eq!(distance_mm(1.0), 0.0);
        assert!((dial_for(12.0) - 0.6).abs() < 1e-12);
    }

    #[test]
    fn a_reed_closes_within_about_18_mm() {
        assert!(reed_shut(dial_for(18.0)));
        assert!(!reed_shut(dial_for(19.0)));
    }

    #[test]
    fn the_hall_sensor_switches_on_closer_than_it_switches_off() {
        // On at 10 mm, off at 13 mm, and at 11.5 mm whatever it was (C-115).
        let between = dial_for(11.5);
        assert!(hall_on(false, dial_for(10.0), 5.0));
        assert!(!hall_on(true, dial_for(13.0), 5.0));
        assert!(hall_on(true, between, 5.0), "kept on");
        assert!(!hall_on(false, between, 5.0), "kept off");
        assert!(
            !hall_on(true, dial_for(0.0), 4.0),
            "never without its supply"
        );
    }
}
