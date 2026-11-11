// SPDX-License-Identifier: MIT OR Apache-2.0

// Smoothed: D9 on a quarter of the time (64 of 255), 490 times a second. A
// resistor and a capacitor after it smooth the pulses into about 1.25 V, with
// a ripple left on top that a bigger capacitor makes smaller.
void setup() {
  pinMode(9, OUTPUT);
  analogWrite(9, 64);
}

void loop() {
}
