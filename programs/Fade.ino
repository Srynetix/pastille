// SPDX-License-Identifier: MIT OR Apache-2.0

// Fade: two LEDs, on D9 and D10, swap brightness back and forth. analogWrite
// does not make a voltage in between: it switches the pin on and off 490
// times a second, on for longer the higher the number.
void setup() {
  pinMode(9, OUTPUT);
  pinMode(10, OUTPUT);
}

void loop() {
  for (int level = 0; level <= 255; level += 5) {
    analogWrite(9, level);
    analogWrite(10, 255 - level);
    delay(20);
  }
  for (int level = 255; level >= 0; level -= 5) {
    analogWrite(9, level);
    analogWrite(10, 255 - level);
    delay(20);
  }
}
