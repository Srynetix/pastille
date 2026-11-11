// SPDX-License-Identifier: MIT OR Apache-2.0

// Button: D13 lights while the button on D2 is pressed. The button goes from
// D2 to GND, and D2's own pull-up holds it high while it is not.
void setup() {
  pinMode(2, INPUT_PULLUP);
  pinMode(13, OUTPUT);
}

void loop() {
  digitalWrite(13, digitalRead(2) == LOW ? HIGH : LOW);
}
