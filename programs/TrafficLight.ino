// SPDX-License-Identifier: MIT OR Apache-2.0

// Traffic light: green on D10 for two seconds, amber on D11 for half a
// second, red on D12 for two seconds, and round again.
const int RED = 12;
const int AMBER = 11;
const int GREEN = 10;

void show(int lit, unsigned long ms) {
  digitalWrite(RED, lit == RED ? HIGH : LOW);
  digitalWrite(AMBER, lit == AMBER ? HIGH : LOW);
  digitalWrite(GREEN, lit == GREEN ? HIGH : LOW);
  delay(ms);
}

void setup() {
  pinMode(RED, OUTPUT);
  pinMode(AMBER, OUTPUT);
  pinMode(GREEN, OUTPUT);
}

void loop() {
  show(GREEN, 2000);
  show(AMBER, 500);
  show(RED, 2000);
}
