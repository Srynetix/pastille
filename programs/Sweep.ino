// SPDX-License-Identifier: MIT OR Apache-2.0

// Sweep: the servo on D9 turns from 0 to 180 degrees and back, a degree every
// 10 ms. The Servo library sends it a pulse every 20 ms; how long the pulse
// lasts, 1 to 2 ms, is the angle.
#include <Servo.h>

Servo servo;

void setup() {
  servo.attach(9);
}

void loop() {
  for (int angle = 0; angle <= 180; angle++) {
    servo.write(angle);
    delay(10);
  }
  for (int angle = 180; angle >= 0; angle--) {
    servo.write(angle);
    delay(10);
  }
}
