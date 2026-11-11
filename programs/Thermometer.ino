// SPDX-License-Identifier: MIT OR Apache-2.0

// Thermometer: a TMP36 on A0, five LEDs on D2 to D6, one more lit for every
// 5 °C from 20 °C. The TMP36 gives 0.5 V at 0 °C and 10 mV a degree more, so
// it is read against the chip's own 1.1 V rather than 5 V: four times finer.
void setup() {
  analogReference(INTERNAL);
  for (int pin = 2; pin <= 6; pin++) {
    pinMode(pin, OUTPUT);
  }
}

void loop() {
  long millivolts = analogRead(A0) * 1100L / 1024;
  long celsius = (millivolts - 500) / 10;
  for (int led = 0; led < 5; led++) {
    digitalWrite(2 + led, celsius >= 20 + 5 * led ? HIGH : LOW);
  }
  delay(100);
}
