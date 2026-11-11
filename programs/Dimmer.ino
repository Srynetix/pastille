// SPDX-License-Identifier: MIT OR Apache-2.0

// Dimmer: a potentiometer on A0 sets the brightness of an LED on D9.
// analogRead gives 0 to 1023 and analogWrite takes 0 to 255, so the reading
// is divided by four on its way across.
void setup() {
  pinMode(9, OUTPUT);
}

void loop() {
  analogWrite(9, analogRead(A0) / 4);
}
