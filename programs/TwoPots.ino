// SPDX-License-Identifier: MIT OR Apache-2.0

// TwoPots: A0 and A1 read one after the other, printed side by side five
// times a second. There is one converter: between the two readings it is
// switched from one pin to the other.
void setup() {
  Serial.begin(9600);
}

void loop() {
  int a0 = analogRead(A0);
  int a1 = analogRead(A1);
  Serial.print("A0 ");
  Serial.print(a0);
  Serial.print("   A1 ");
  Serial.println(a1);
  delay(200);
}
