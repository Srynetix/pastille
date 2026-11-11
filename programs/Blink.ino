// SPDX-License-Identifier: MIT OR Apache-2.0

// Blink: the LED on D13 on for a second, off for a second.
void setup() {
  pinMode(13, OUTPUT);
}

void loop() {
  digitalWrite(13, HIGH);
  delay(1000);
  digitalWrite(13, LOW);
  delay(1000);
}
