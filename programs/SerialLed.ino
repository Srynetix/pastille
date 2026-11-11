// SPDX-License-Identifier: MIT OR Apache-2.0

// SerialLed: type "on" or "off" in the serial monitor and press Send; D13
// does as it is told, and the program says what it did. Anything else, it
// says it did not understand.
String line;

void setup() {
  pinMode(13, OUTPUT);
  Serial.begin(9600);
  Serial.println("Type on or off.");
}

void loop() {
  while (Serial.available() > 0) {
    char c = Serial.read();
    if (c == '\r') {
      continue;
    }
    if (c != '\n') {
      line += c;
      continue;
    }
    line.trim();
    if (line == "on") {
      digitalWrite(13, HIGH);
      Serial.println("D13 is on.");
    } else if (line == "off") {
      digitalWrite(13, LOW);
      Serial.println("D13 is off.");
    } else {
      Serial.print("I do not know \"");
      Serial.print(line);
      Serial.println("\". Type on or off.");
    }
    line = "";
  }
}
