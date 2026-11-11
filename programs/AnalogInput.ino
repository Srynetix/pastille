// SPDX-License-Identifier: MIT OR Apache-2.0

// AnalogInput: a potentiometer on A0 sets how fast D13 blinks. analogRead
// gives the wiper's voltage as a number from 0 (0 V) to 1023 (5 V), and the
// program waits that many milliseconds between each change.
void setup() {
  pinMode(13, OUTPUT);
}

void loop() {
  int wait = analogRead(A0);
  digitalWrite(13, HIGH);
  delay(wait);
  digitalWrite(13, LOW);
  delay(wait);
}
