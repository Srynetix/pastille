// SPDX-License-Identifier: MIT OR Apache-2.0

// AnalogReadSerial: the voltage on A0, as analogRead gives it (0 to 1023),
// printed to the serial monitor ten times a second. Turn the potentiometer
// and watch the numbers follow.
void setup() {
  Serial.begin(9600);
}

void loop() {
  Serial.println(analogRead(A0));
  delay(100);
}
