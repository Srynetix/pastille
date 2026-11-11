// SPDX-License-Identifier: MIT OR Apache-2.0

// SerialThermometer: a TMP36 on A0, its temperature printed twice a second
// — at 115200 baud, twelve times faster than 9600. The monitor has to be set
// to the same speed, or what arrives is garbage.
void setup() {
  analogReference(INTERNAL);
  Serial.begin(115200);
}

void loop() {
  float millivolts = analogRead(A0) * 1100.0 / 1024;
  float celsius = (millivolts - 500) / 10;
  Serial.print(celsius, 1);
  Serial.println(" °C");
  delay(500);
}
