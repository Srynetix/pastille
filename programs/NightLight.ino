// SPDX-License-Identifier: MIT OR Apache-2.0

// NightLight: a light-dependent resistor from A0 to GND, under 10 kΩ from
// 5V. The darker it gets, the higher the LDR's resistance and the higher A0
// reads; D13 lights past 600 and goes out below 500. The gap between the two
// keeps it from flickering on the edge of dark.
bool lit = false;

void setup() {
  pinMode(13, OUTPUT);
}

void loop() {
  int dark = analogRead(A0);
  if (dark > 600) {
    lit = true;
  } else if (dark < 500) {
    lit = false;
  }
  digitalWrite(13, lit ? HIGH : LOW);
  delay(50);
}
