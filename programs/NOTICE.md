# The Uno's programs

Real Arduino builds, which the app offers an Uno (A-07) and the core's tests
run on the emulated ATmega328P (A-03, ADR 0017): `Blink`, `Button`,
`TrafficLight`, `Fade`, `Smoothed`, `Sweep`, and, reading A0 (A-04),
`AnalogInput`, `Dimmer`, `NightLight` and `Thermometer`, and, printing to
the serial monitor (A-06), `AnalogReadSerial`, `SerialLed`,
`SerialThermometer` and `TwoPots` — each its sketch (`.ino`, this
repository's, MIT OR Apache-2.0) and the Intel HEX built from it. `just
programs` rebuilds them; each example circuit carries its program's HEX
verbatim, and a test holds the two together.

**Each `.hex` contains Arduino's AVR core**, statically linked as every
Arduino program is: ArduinoCore-avr **1.8.8**, licensed under the **GNU
Lesser General Public License, version 2.1 or later** — its text is
`LGPL-2.1.txt` beside this file. Its source is at
<https://github.com/arduino/ArduinoCore-avr/tree/1.8.8>. The sketches here
are the rest of what the programs are made of, so they can be rebuilt —
against that core or a modified one — with `build.sh`.

**`Sweep.hex` also contains Arduino's Servo library**, version **1.3.0**,
likewise LGPL-2.1 or later; its source is at
<https://github.com/arduino-libraries/Servo/tree/1.3.0>.

Built on 2026-10-04 with `build.sh`: avr-gcc 9.5.0 (Homebrew's
`osx-cross/avr`), avr-libc (BSD-licensed), the flags the Arduino IDE uses for
an Uno. The GCC Runtime Library Exception covers what the compiler adds.
