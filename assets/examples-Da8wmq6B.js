var e=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 60.0, "y": 260.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 180.0, "y": 160.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 180.0, "y": 360.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 260.0, "y": 260.0, "turns": 1},
    {"kind": "Voltmeter", "x": 320.0, "y": 260.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 420.0, "y": 160.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 420.0, "y": 360.0, "turns": 1},
    {"kind": "OpAmp", "x": 520.0, "y": 260.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 620.0, "y": 340.0, "turns": 1},
    {"kind": "Voltmeter", "x": 680.0, "y": 340.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}, {"element": 5, "pin": "a"}, {"element": 7, "pin": "V+"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}, {"element": 3, "pin": "a"}, {"element": 4, "pin": "+"}]},
    {"pins": [{"element": 5, "pin": "b"}, {"element": 6, "pin": "a"}, {"element": 7, "pin": "in+"}]},
    {"pins": [{"element": 7, "pin": "out"}, {"element": 7, "pin": "in-"}, {"element": 8, "pin": "a"}, {"element": 9, "pin": "+"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 2, "pin": "b"}, {"element": 3, "pin": "b"}, {"element": 4, "pin": "-"}, {"element": 6, "pin": "b"}, {"element": 7, "pin": "V-"}, {"element": 8, "pin": "b"}, {"element": 9, "pin": "-"}]}
  ]
}
`,t=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    { "kind": { "DcSource": { "volts": 9.0, "series_ohms": 0.0 } }, "x": 120.0, "y": 240.0, "turns": 0 },
    { "kind": { "Resistor": { "ohms": 100000.0 } }, "x": 240.0, "y": 120.0, "turns": 0 },
    { "kind": { "Capacitor": { "farads": 0.00001, "initial_volts": 0.0 } }, "x": 400.0, "y": 240.0, "turns": 0 },
    { "kind": "Voltmeter", "x": 400.0, "y": 360.0, "turns": 0 }
  ],
  "nets": [
    { "pins": [{ "element": 0, "pin": "+" }, { "element": 1, "pin": "a" }] },
    { "pins": [{ "element": 1, "pin": "b" }, { "element": 2, "pin": "a" }, { "element": 3, "pin": "+" }] },
    { "pins": [{ "element": 2, "pin": "b" }, { "element": 0, "pin": "-" }, { "element": 3, "pin": "-" }] }
  ]
}
`,n=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 1.5, "series_ohms": 0.0}}, "x": 100.0, "y": 220.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 10.0}}, "x": 240.0, "y": 100.0, "turns": 0},
    {"kind": {"Inductor": {"henries": 10.0, "initial_amps": 0.0}}, "x": 380.0, "y": 220.0, "turns": 1},
    {"kind": "Ammeter", "x": 240.0, "y": 340.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "+"}]},
    {"pins": [{"element": 3, "pin": "-"}, {"element": 0, "pin": "-"}]}
  ]
}
`,ee=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 120.0, "y": 200.0, "turns": 0},
    {"kind": {"Potentiometer": {"ohms": 1000.0, "wiper": 0.5}}, "x": 240.0, "y": 120.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 220.0}}, "x": 360.0, "y": 120.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 480.0, "y": 200.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "wiper"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "anode"}]},
    {"pins": [{"element": 3, "pin": "cathode"}, {"element": 0, "pin": "-"}]}
  ]
}
`,te=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    { "kind": { "DcSource": { "volts": 9.0, "series_ohms": 0.0 } }, "x": 120.0, "y": 240.0, "turns": 0 },
    { "kind": "Ammeter", "x": 240.0, "y": 120.0, "turns": 0 },
    { "kind": { "Resistor": { "ohms": 1000.0 } }, "x": 360.0, "y": 120.0, "turns": 0 },
    { "kind": { "Resistor": { "ohms": 2000.0 } }, "x": 480.0, "y": 240.0, "turns": 0 },
    { "kind": "Voltmeter", "x": 480.0, "y": 360.0, "turns": 0 }
  ],
  "nets": [
    { "pins": [{ "element": 0, "pin": "+" }, { "element": 1, "pin": "+" }] },
    { "pins": [{ "element": 1, "pin": "-" }, { "element": 2, "pin": "a" }] },
    { "pins": [{ "element": 2, "pin": "b" }, { "element": 3, "pin": "a" }, { "element": 4, "pin": "+" }] },
    { "pins": [{ "element": 3, "pin": "b" }, { "element": 0, "pin": "-" }, { "element": 4, "pin": "-" }] }
  ]
}
`,r=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 120.0, "y": 200.0, "turns": 0},
    {"kind": {"Regulator": {"part": "L7812"}}, "x": 240.0, "y": 120.0, "turns": 0},
    {"kind": "Voltmeter", "x": 360.0, "y": 200.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 460.0, "y": 200.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "in"}]},
    {"pins": [{"element": 1, "pin": "out"}, {"element": 2, "pin": "+"}, {"element": 3, "pin": "a"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 2, "pin": "-"}, {"element": 1, "pin": "gnd"}, {"element": 0, "pin": "-"}]}
  ]
}
`,i=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"Capacitor": {"farads": 0.0047, "initial_volts": 9.0}}, "x": 120.0, "y": 200.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 220.0}}, "x": 240.0, "y": 120.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 360.0, "y": 200.0, "turns": 1},
    {"kind": "Voltmeter", "x": 40.0, "y": 200.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "a"}, {"element": 1, "pin": "a"}, {"element": 3, "pin": "+"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "anode"}]},
    {"pins": [{"element": 2, "pin": "cathode"}, {"element": 0, "pin": "b"}, {"element": 3, "pin": "-"}]}
  ]
}
`,a=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 120.0, "y": 220.0, "turns": 1},
    {"kind": {"PushButton": {"closed": false}}, "x": 240.0, "y": 120.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 100.0}}, "x": 340.0, "y": 120.0, "turns": 0},
    {"kind": {"Electrolytic": {"farads": 0.0001, "initial_volts": 0.0, "rated": "V25"}}, "x": 440.0, "y": 220.0, "turns": 1},
    {"kind": "Voltmeter", "x": 540.0, "y": 220.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "+"}, {"element": 4, "pin": "+"}]},
    {"pins": [{"element": 3, "pin": "-"}, {"element": 0, "pin": "-"}, {"element": 4, "pin": "-"}]}
  ]
}
`,o=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "room_celsius": 25.0,
  "elements": [
    { "kind": { "DcSource": { "volts": 12.0, "series_ohms": 0.0 } }, "x": 120.0, "y": 200.0, "turns": 0 },
    { "kind": { "Regulator": { "part": "L7805" } }, "x": 240.0, "y": 120.0, "turns": 0 },
    { "kind": "Voltmeter", "x": 360.0, "y": 200.0, "turns": 1 },
    { "kind": "Motor", "x": 480.0, "y": 200.0, "turns": 1 }
  ],
  "nets": [
    { "pins": [{ "element": 0, "pin": "+" }, { "element": 1, "pin": "in" }] },
    { "pins": [{ "element": 1, "pin": "out" }, { "element": 2, "pin": "+" }, { "element": 3, "pin": "a" }] },
    { "pins": [{ "element": 3, "pin": "b" }, { "element": 2, "pin": "-" }, { "element": 1, "pin": "gnd" }, { "element": 0, "pin": "-" }] }
  ]
}
`,s=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 60.0, "y": 280.0, "turns": 1},
    {"kind": {"Tmp36": {"celsius": 25.0}}, "x": 180.0, "y": 200.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 39000.0}}, "x": 260.0, "y": 320.0, "turns": 1},
    {"kind": {"Potentiometer": {"ohms": 10000.0, "wiper": 0.784}}, "x": 260.0, "y": 400.0, "turns": 3},
    {"kind": "Comparator", "x": 360.0, "y": 260.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 440.0, "y": 180.0, "turns": 1},
    {"kind": {"Mosfet": {"channel": "N"}}, "x": 540.0, "y": 280.0, "turns": 0},
    {"kind": {"Relay": {"part": "V5"}}, "x": 660.0, "y": 180.0, "turns": 0},
    {"kind": "Diode", "x": 540.0, "y": 140.0, "turns": 3},
    {"kind": "Motor", "x": 800.0, "y": 280.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "+Vs"}, {"element": 2, "pin": "a"}, {"element": 4, "pin": "V+"}, {"element": 5, "pin": "a"}, {"element": 7, "pin": "coil+"}, {"element": 8, "pin": "cathode"}, {"element": 7, "pin": "com"}]},
    {"pins": [{"element": 1, "pin": "Vout"}, {"element": 4, "pin": "in+"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "b"}]},
    {"pins": [{"element": 3, "pin": "wiper"}, {"element": 4, "pin": "in-"}]},
    {"pins": [{"element": 4, "pin": "out"}, {"element": 5, "pin": "b"}, {"element": 6, "pin": "g"}]},
    {"pins": [{"element": 7, "pin": "coil-"}, {"element": 8, "pin": "anode"}, {"element": 6, "pin": "d"}]},
    {"pins": [{"element": 7, "pin": "no"}, {"element": 9, "pin": "a"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "GND"}, {"element": 3, "pin": "a"}, {"element": 4, "pin": "V-"}, {"element": 6, "pin": "s"}, {"element": 9, "pin": "b"}]}
  ]
}
`,c=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 120.0, "y": 200.0, "turns": 0},
    {"kind": {"PushButton": {"closed": false}}, "x": 240.0, "y": 120.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 360.0, "y": 120.0, "turns": 0},
    {"kind": {"Led": {"colour": "White"}}, "x": 480.0, "y": 200.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "anode"}]},
    {"pins": [{"element": 3, "pin": "cathode"}, {"element": 0, "pin": "-"}]}
  ]
}
`,l=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "room_celsius": 25.0,
  "elements": [
    { "kind": { "DcSource": { "volts": 9.0, "series_ohms": 0.0 } }, "x": 120.0, "y": 160.0, "turns": 0 },
    { "kind": { "Resistor": { "ohms": 150.0 } }, "x": 240.0, "y": 100.0, "turns": 0 },
    { "kind": "Ammeter", "x": 360.0, "y": 160.0, "turns": 1 }
  ],
  "nets": [
    { "pins": [{ "element": 0, "pin": "+" }, { "element": 1, "pin": "a" }] },
    { "pins": [{ "element": 1, "pin": "b" }, { "element": 2, "pin": "+" }] },
    { "pins": [{ "element": 2, "pin": "-" }, { "element": 0, "pin": "-" }] }
  ]
}
`,u=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 100.0, "y": 220.0, "turns": 0},
    {"kind": "Ammeter", "x": 200.0, "y": 100.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 320.0, "y": 100.0, "turns": 0},
    {"kind": "Ammeter", "x": 440.0, "y": 100.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 2000.0}}, "x": 520.0, "y": 220.0, "turns": 1},
    {"kind": "Ammeter", "x": 320.0, "y": 340.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "+"}]},
    {"pins": [{"element": 1, "pin": "-"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "+"}]},
    {"pins": [{"element": 3, "pin": "-"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 5, "pin": "+"}]},
    {"pins": [{"element": 5, "pin": "-"}, {"element": 0, "pin": "-"}]}
  ]
}
`,d=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    { "kind": { "DcSource": { "volts": 9.0, "series_ohms": 0.0 } }, "x": 120.0, "y": 160.0, "turns": 0 },
    { "kind": { "Resistor": { "ohms": 470.0 } }, "x": 240.0, "y": 100.0, "turns": 0 },
    { "kind": { "Led": { "colour": "Red" } }, "x": 360.0, "y": 160.0, "turns": 0 }
  ],
  "nets": [
    { "pins": [{ "element": 0, "pin": "+" }, { "element": 1, "pin": "a" }] },
    { "pins": [{ "element": 1, "pin": "b" }, { "element": 2, "pin": "anode" }] },
    { "pins": [{ "element": 2, "pin": "cathode" }, { "element": 0, "pin": "-" }] }
  ]
}
`,f=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 120.0, "y": 240.0, "turns": 1},
    {"kind": {"RgbLed": {"common": "Cathode"}}, "x": 480.0, "y": 240.0, "turns": 0},
    {"kind": {"Potentiometer": {"ohms": 10000.0, "wiper": 1.0}}, "x": 220.0, "y": 120.0, "turns": 2},
    {"kind": {"Potentiometer": {"ohms": 10000.0, "wiper": 0.7}}, "x": 220.0, "y": 220.0, "turns": 2},
    {"kind": {"Potentiometer": {"ohms": 10000.0, "wiper": 0.0}}, "x": 220.0, "y": 320.0, "turns": 2},
    {"kind": {"Resistor": {"ohms": 390.0}}, "x": 320.0, "y": 160.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 320.0, "y": 260.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 320.0, "y": 360.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 2, "pin": "b"}, {"element": 3, "pin": "b"}, {"element": 4, "pin": "b"}]},
    {"pins": [{"element": 2, "pin": "wiper"}, {"element": 5, "pin": "a"}]},
    {"pins": [{"element": 3, "pin": "wiper"}, {"element": 6, "pin": "a"}]},
    {"pins": [{"element": 4, "pin": "wiper"}, {"element": 7, "pin": "a"}]},
    {"pins": [{"element": 5, "pin": "b"}, {"element": 1, "pin": "R"}], "waypoints": [{"x": 420.0, "y": 160.0}, {"x": 420.0, "y": 230.0}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 1, "pin": "G"}]},
    {"pins": [{"element": 7, "pin": "b"}, {"element": 1, "pin": "B"}], "waypoints": [{"x": 420.0, "y": 360.0}, {"x": 420.0, "y": 250.0}]},
    {"pins": [{"element": 1, "pin": "com"}, {"element": 0, "pin": "-"}], "waypoints": [{"x": 520.0, "y": 240.0}, {"x": 520.0, "y": 420.0}, {"x": 120.0, "y": 420.0}]}
  ]
}
`,p=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 100.0, "y": 240.0, "turns": 0},
    {"kind": "Diode", "x": 240.0, "y": 120.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 360.0, "y": 120.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 480.0, "y": 120.0, "turns": 0},
    {"kind": "Diode", "x": 240.0, "y": 300.0, "turns": 2},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 360.0, "y": 300.0, "turns": 0},
    {"kind": {"Led": {"colour": "Green"}}, "x": 480.0, "y": 300.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "anode"}, {"element": 4, "pin": "cathode"}]},
    {"pins": [{"element": 1, "pin": "cathode"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "anode"}]},
    {"pins": [{"element": 4, "pin": "anode"}, {"element": 5, "pin": "a"}]},
    {"pins": [{"element": 5, "pin": "b"}, {"element": 6, "pin": "anode"}]},
    {"pins": [{"element": 3, "pin": "cathode"}, {"element": 6, "pin": "cathode"}, {"element": 0, "pin": "-"}]}
  ]
}
`,m=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 120.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 470.0}}, "x": 260.0, "y": 120.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 400.0, "y": 120.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 470.0}}, "x": 260.0, "y": 280.0, "turns": 0},
    {"kind": {"Led": {"colour": "Green"}}, "x": 400.0, "y": 280.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}, {"element": 3, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "anode"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 4, "pin": "anode"}]},
    {"pins": [{"element": 2, "pin": "cathode"}, {"element": 4, "pin": "cathode"}, {"element": 0, "pin": "-"}]}
  ]
}
`,h=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 200.0, "y": 300.0, "turns": 1},
    {"kind": "Timer", "x": 440.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 340.0, "y": 190.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 360.0, "y": 250.0, "turns": 0},
    {"kind": {"Capacitor": {"farads": 4.7e-05, "initial_volts": 0.0}}, "x": 380.0, "y": 310.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 520.0, "y": 240.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 580.0, "y": 300.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 2, "pin": "a"}, {"element": 1, "pin": "VCC"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 1, "pin": "DIS"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 1, "pin": "TRIG"}, {"element": 1, "pin": "THR"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "OUT"}, {"element": 5, "pin": "a"}]},
    {"pins": [{"element": 5, "pin": "b"}, {"element": 6, "pin": "anode"}]},
    {"pins": [{"element": 6, "pin": "cathode"}, {"element": 4, "pin": "b"}, {"element": 1, "pin": "GND"}, {"element": 0, "pin": "-"}]}
  ]
}
`,g=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 200.0, "y": 300.0, "turns": 1},
    {"kind": "Timer", "x": 440.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 300.0, "y": 300.0, "turns": 1},
    {"kind": {"PushButton": {"closed": false}}, "x": 420.0, "y": 330.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 22000.0}}, "x": 360.0, "y": 170.0, "turns": 1},
    {"kind": {"Capacitor": {"farads": 4.7e-05, "initial_volts": 0.0}}, "x": 380.0, "y": 240.0, "turns": 2},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 520.0, "y": 240.0, "turns": 0},
    {"kind": {"Led": {"colour": "Green"}}, "x": 580.0, "y": 300.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "VCC"}, {"element": 2, "pin": "a"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 1, "pin": "TRIG"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 1, "pin": "DIS"}, {"element": 1, "pin": "THR"}, {"element": 5, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "OUT"}, {"element": 6, "pin": "a"}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 7, "pin": "anode"}]},
    {"pins": [{"element": 7, "pin": "cathode"}, {"element": 3, "pin": "b"}, {"element": 5, "pin": "b"}, {"element": 1, "pin": "GND"}, {"element": 0, "pin": "-"}]}
  ]
}
`,_=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 200.0, "y": 300.0, "turns": 1},
    {"kind": "Timer", "x": 440.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 4700.0}}, "x": 340.0, "y": 190.0, "turns": 1},
    {"kind": {"Potentiometer": {"ohms": 50000.0, "wiper": 0.276}}, "x": 360.0, "y": 270.0, "turns": 0},
    {"kind": {"Capacitor": {"farads": 1e-07, "initial_volts": 0.0}}, "x": 400.0, "y": 330.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 520.0, "y": 240.0, "turns": 0},
    {"kind": {"Bjt": {"polarity": "Npn"}}, "x": 580.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 100.0}}, "x": 580.0, "y": 120.0, "turns": 1},
    {"kind": "PassiveBuzzer", "x": 580.0, "y": 180.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 2, "pin": "a"}, {"element": 1, "pin": "VCC"}, {"element": 7, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 1, "pin": "DIS"}, {"element": 3, "pin": "a"}]},
    {"pins": [{"element": 3, "pin": "wiper"}, {"element": 1, "pin": "THR"}, {"element": 1, "pin": "TRIG"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "OUT"}, {"element": 5, "pin": "a"}]},
    {"pins": [{"element": 5, "pin": "b"}, {"element": 6, "pin": "b"}]},
    {"pins": [{"element": 7, "pin": "b"}, {"element": 8, "pin": "+"}]},
    {"pins": [{"element": 8, "pin": "-"}, {"element": 6, "pin": "c"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 4, "pin": "b"}, {"element": 1, "pin": "GND"}, {"element": 6, "pin": "e"}]}
  ]
}
`,v=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 160.0, "y": 300.0, "turns": 1},
    {"kind": "Timer", "x": 440.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 4700.0}}, "x": 340.0, "y": 190.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 15000.0}}, "x": 360.0, "y": 270.0, "turns": 0},
    {"kind": {"Capacitor": {"farads": 1e-07, "initial_volts": 0.0}}, "x": 400.0, "y": 330.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 520.0, "y": 240.0, "turns": 0},
    {"kind": {"Bjt": {"polarity": "Npn"}}, "x": 580.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 100.0}}, "x": 580.0, "y": 120.0, "turns": 1},
    {"kind": "PassiveBuzzer", "x": 580.0, "y": 180.0, "turns": 1},
    {"kind": "Timer", "x": 280.0, "y": 460.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 220.0, "y": 410.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 33000.0}}, "x": 200.0, "y": 490.0, "turns": 0},
    {"kind": {"Capacitor": {"farads": 1e-05, "initial_volts": 0.0}}, "x": 240.0, "y": 550.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 56000.0}}, "x": 360.0, "y": 400.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 2, "pin": "a"}, {"element": 1, "pin": "VCC"}, {"element": 7, "pin": "a"}, {"element": 9, "pin": "VCC"}, {"element": 10, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 1, "pin": "DIS"}, {"element": 3, "pin": "a"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 1, "pin": "THR"}, {"element": 1, "pin": "TRIG"}, {"element": 4, "pin": "a"}, {"element": 13, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "OUT"}, {"element": 5, "pin": "a"}]},
    {"pins": [{"element": 5, "pin": "b"}, {"element": 6, "pin": "b"}]},
    {"pins": [{"element": 7, "pin": "b"}, {"element": 8, "pin": "+"}]},
    {"pins": [{"element": 8, "pin": "-"}, {"element": 6, "pin": "c"}]},
    {"pins": [{"element": 10, "pin": "b"}, {"element": 9, "pin": "DIS"}, {"element": 11, "pin": "a"}]},
    {"pins": [{"element": 11, "pin": "b"}, {"element": 9, "pin": "THR"}, {"element": 9, "pin": "TRIG"}, {"element": 12, "pin": "a"}]},
    {"pins": [{"element": 9, "pin": "OUT"}, {"element": 13, "pin": "b"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 4, "pin": "b"}, {"element": 1, "pin": "GND"}, {"element": 6, "pin": "e"}, {"element": 9, "pin": "GND"}, {"element": 12, "pin": "b"}]}
  ]
}
`,y=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"SignalGenerator": {"waveform": "Sine", "volts": 9.0, "hertz": 1.0}}, "x": 120.0, "y": 220.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 240.0, "y": 120.0, "turns": 0},
    {"kind": {"Zener": {"part": "V5_1"}}, "x": 360.0, "y": 220.0, "turns": 3},
    {"kind": "Voltmeter", "x": 460.0, "y": 220.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "cathode"}, {"element": 3, "pin": "+"}]},
    {"pins": [{"element": 2, "pin": "anode"}, {"element": 0, "pin": "-"}, {"element": 3, "pin": "-"}]}
  ]
}
`,b=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 120.0, "y": 220.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 240.0, "y": 120.0, "turns": 0},
    {"kind": {"Zener": {"part": "V5_1"}}, "x": 360.0, "y": 220.0, "turns": 3},
    {"kind": "Voltmeter", "x": 460.0, "y": 220.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "cathode"}, {"element": 3, "pin": "+"}]},
    {"pins": [{"element": 2, "pin": "anode"}, {"element": 0, "pin": "-"}, {"element": 3, "pin": "-"}]}
  ]
}
`,x=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 100.0, "y": 160.0, "turns": 1},
    {"kind": {"SignalGenerator": {"waveform": "Square", "volts": 5.0, "hertz": 1.0}}, "x": 100.0, "y": 340.0, "turns": 1},
    {"kind": "Ammeter", "x": 240.0, "y": 280.0, "turns": 0},
    {"kind": {"Mosfet": {"channel": "N"}}, "x": 380.0, "y": 280.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 380.0, "y": 100.0, "turns": 1},
    {"kind": {"Led": {"colour": "Green"}}, "x": 380.0, "y": 200.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 5, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "cathode"}, {"element": 3, "pin": "d"}]},
    {"pins": [{"element": 1, "pin": "+"}, {"element": 2, "pin": "+"}]},
    {"pins": [{"element": 2, "pin": "-"}, {"element": 3, "pin": "g"}]},
    {"pins": [{"element": 3, "pin": "s"}, {"element": 1, "pin": "-"}, {"element": 0, "pin": "-"}]}
  ]
}
`,S=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 100.0, "y": 240.0, "turns": 1},
    {"kind": {"PushButton": {"closed": false}}, "x": 220.0, "y": 140.0, "turns": 1},
    {"kind": {"Capacitor": {"farads": 1e-06, "initial_volts": 0.0}}, "x": 220.0, "y": 340.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 4700000.0}}, "x": 300.0, "y": 340.0, "turns": 1},
    {"kind": {"Mosfet": {"channel": "N"}}, "x": 420.0, "y": 280.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 420.0, "y": 100.0, "turns": 1},
    {"kind": {"Led": {"colour": "Red"}}, "x": 420.0, "y": 200.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}, {"element": 5, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}, {"element": 3, "pin": "a"}, {"element": 4, "pin": "g"}]},
    {"pins": [{"element": 5, "pin": "b"}, {"element": 6, "pin": "anode"}]},
    {"pins": [{"element": 6, "pin": "cathode"}, {"element": 4, "pin": "d"}]},
    {"pins": [{"element": 4, "pin": "s"}, {"element": 2, "pin": "b"}, {"element": 3, "pin": "b"}, {"element": 0, "pin": "-"}]}
  ]
}
`,C=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 60.0, "y": 200.0, "turns": 1},
    {"kind": {"SignalGenerator": {"waveform": "Square", "volts": 5.0, "hertz": 1.0}}, "x": 60.0, "y": 380.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 4700.0}}, "x": 180.0, "y": 380.0, "turns": 0},
    {"kind": {"Bjt": {"polarity": "Npn"}}, "x": 300.0, "y": 380.0, "turns": 0},
    {"kind": {"Relay": {"part": "V9"}}, "x": 300.0, "y": 200.0, "turns": 0},
    {"kind": "Diode", "x": 200.0, "y": 200.0, "turns": 3},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 460.0, "y": 180.0, "turns": 0},
    {"kind": {"Led": {"colour": "Blue"}}, "x": 560.0, "y": 260.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 4, "pin": "coil+"}, {"element": 5, "pin": "cathode"}, {"element": 4, "pin": "com"}]},
    {"pins": [{"element": 4, "pin": "coil-"}, {"element": 5, "pin": "anode"}, {"element": 3, "pin": "c"}]},
    {"pins": [{"element": 1, "pin": "+"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "b"}]},
    {"pins": [{"element": 4, "pin": "no"}, {"element": 6, "pin": "a"}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 7, "pin": "anode"}]},
    {"pins": [{"element": 7, "pin": "cathode"}, {"element": 3, "pin": "e"}, {"element": 1, "pin": "-"}, {"element": 0, "pin": "-"}]}
  ]
}
`,w=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"SignalGenerator": {"waveform": "Square", "volts": 5.0, "hertz": 32.0}}, "x": 460.0, "y": 240.0, "turns": 3},
    {"kind": "LcdDigit", "x": 320.0, "y": 240.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "c"}, {"element": 1, "pin": "b"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "com"}]}
  ]
}
`,T=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 40.0, "y": 300.0, "turns": 1},
    {"kind": "Timer", "x": 200.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 100.0, "y": 190.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 120.0, "y": 250.0, "turns": 0},
    {"kind": {"Capacitor": {"farads": 4.7e-06, "initial_volts": 0.0}}, "x": 140.0, "y": 310.0, "turns": 1},
    {"kind": "Counter", "x": 360.0, "y": 420.0, "turns": 0},
    {"kind": "Counter", "x": 360.0, "y": 160.0, "turns": 0},
    {"kind": {"SevenSegment": {"common": "Cathode"}}, "x": 680.0, "y": 300.0, "turns": 0},
    {"kind": {"SevenSegment": {"common": "Cathode"}}, "x": 800.0, "y": 300.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 100.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 120.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 140.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 160.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 180.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 200.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 220.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 360.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 380.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 400.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 420.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 440.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 460.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 480.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 2, "pin": "a"}, {"element": 1, "pin": "VCC"}, {"element": 5, "pin": "VDD"}, {"element": 5, "pin": "DEI"}, {"element": 6, "pin": "VDD"}, {"element": 6, "pin": "DEI"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 1, "pin": "DIS"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 1, "pin": "TRIG"}, {"element": 1, "pin": "THR"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "OUT"}, {"element": 5, "pin": "CLK"}]},
    {"pins": [{"element": 5, "pin": "CO"}, {"element": 6, "pin": "CLK"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 4, "pin": "b"}, {"element": 1, "pin": "GND"}, {"element": 5, "pin": "VSS"}, {"element": 5, "pin": "RST"}, {"element": 5, "pin": "INH"}, {"element": 6, "pin": "VSS"}, {"element": 6, "pin": "RST"}, {"element": 6, "pin": "INH"}, {"element": 7, "pin": "com"}, {"element": 8, "pin": "com"}]},
    {"pins": [{"element": 6, "pin": "a"}, {"element": 9, "pin": "a"}]},
    {"pins": [{"element": 9, "pin": "b"}, {"element": 7, "pin": "a"}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 10, "pin": "a"}]},
    {"pins": [{"element": 10, "pin": "b"}, {"element": 7, "pin": "b"}]},
    {"pins": [{"element": 6, "pin": "c"}, {"element": 11, "pin": "a"}]},
    {"pins": [{"element": 11, "pin": "b"}, {"element": 7, "pin": "c"}]},
    {"pins": [{"element": 6, "pin": "d"}, {"element": 12, "pin": "a"}]},
    {"pins": [{"element": 12, "pin": "b"}, {"element": 7, "pin": "d"}]},
    {"pins": [{"element": 6, "pin": "e"}, {"element": 13, "pin": "a"}]},
    {"pins": [{"element": 13, "pin": "b"}, {"element": 7, "pin": "e"}]},
    {"pins": [{"element": 6, "pin": "f"}, {"element": 14, "pin": "a"}]},
    {"pins": [{"element": 14, "pin": "b"}, {"element": 7, "pin": "f"}]},
    {"pins": [{"element": 6, "pin": "g"}, {"element": 15, "pin": "a"}]},
    {"pins": [{"element": 15, "pin": "b"}, {"element": 7, "pin": "g"}]},
    {"pins": [{"element": 5, "pin": "a"}, {"element": 16, "pin": "a"}]},
    {"pins": [{"element": 16, "pin": "b"}, {"element": 8, "pin": "a"}]},
    {"pins": [{"element": 5, "pin": "b"}, {"element": 17, "pin": "a"}]},
    {"pins": [{"element": 17, "pin": "b"}, {"element": 8, "pin": "b"}]},
    {"pins": [{"element": 5, "pin": "c"}, {"element": 18, "pin": "a"}]},
    {"pins": [{"element": 18, "pin": "b"}, {"element": 8, "pin": "c"}]},
    {"pins": [{"element": 5, "pin": "d"}, {"element": 19, "pin": "a"}]},
    {"pins": [{"element": 19, "pin": "b"}, {"element": 8, "pin": "d"}]},
    {"pins": [{"element": 5, "pin": "e"}, {"element": 20, "pin": "a"}]},
    {"pins": [{"element": 20, "pin": "b"}, {"element": 8, "pin": "e"}]},
    {"pins": [{"element": 5, "pin": "f"}, {"element": 21, "pin": "a"}]},
    {"pins": [{"element": 21, "pin": "b"}, {"element": 8, "pin": "f"}]},
    {"pins": [{"element": 5, "pin": "g"}, {"element": 22, "pin": "a"}]},
    {"pins": [{"element": 22, "pin": "b"}, {"element": 8, "pin": "g"}]}
  ]
}
`,E=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"SignalGenerator": {"waveform": "Square", "volts": 2.5, "hertz": 32.0}}, "x": 180.0, "y": 240.0, "turns": 1},
    {"kind": {"SignalGenerator": {"waveform": "Square", "volts": 2.5, "hertz": 32.0}}, "x": 460.0, "y": 240.0, "turns": 3},
    {"kind": "LcdDigit", "x": 320.0, "y": 240.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 2, "pin": "g"}, {"element": 2, "pin": "e"}, {"element": 2, "pin": "f"}, {"element": 2, "pin": "d"}, {"element": 2, "pin": "com"}]},
    {"pins": [{"element": 1, "pin": "-"}, {"element": 2, "pin": "b"}, {"element": 2, "pin": "a"}, {"element": 2, "pin": "c"}]},
    {"pins": [{"element": 1, "pin": "+"}, {"element": 0, "pin": "-"}]}
  ]
}
`,D=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"SignalGenerator": {"waveform": "Square", "volts": 4.0, "hertz": 32.0}}, "x": 300.0, "y": 240.0, "turns": 3},
    {"kind": "LcdDigit", "x": 200.0, "y": 240.0, "turns": 0},
    {"kind": {"SignalGenerator": {"waveform": "Sine", "volts": 4.0, "hertz": 32.0}}, "x": 520.0, "y": 240.0, "turns": 3},
    {"kind": "LcdDigit", "x": 420.0, "y": 240.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "c"}, {"element": 1, "pin": "b"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "com"}]},
    {"pins": [{"element": 2, "pin": "+"}, {"element": 3, "pin": "c"}, {"element": 3, "pin": "b"}]},
    {"pins": [{"element": 2, "pin": "-"}, {"element": 3, "pin": "com"}]}
  ]
}
`,O=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 60.0, "y": 200.0, "turns": 1},
    {"kind": {"SignalGenerator": {"waveform": "Square", "volts": 5.0, "hertz": 1.0}}, "x": 60.0, "y": 380.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 4700.0}}, "x": 180.0, "y": 380.0, "turns": 0},
    {"kind": {"Bjt": {"polarity": "Npn"}}, "x": 300.0, "y": 380.0, "turns": 0},
    {"kind": {"Relay": {"part": "V9"}}, "x": 300.0, "y": 200.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 460.0, "y": 180.0, "turns": 0},
    {"kind": {"Led": {"colour": "Blue"}}, "x": 560.0, "y": 260.0, "turns": 1},
    {"kind": "Voltmeter", "x": 420.0, "y": 400.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 4, "pin": "coil+"}, {"element": 4, "pin": "com"}]},
    {"pins": [{"element": 4, "pin": "coil-"}, {"element": 3, "pin": "c"}, {"element": 7, "pin": "+"}]},
    {"pins": [{"element": 1, "pin": "+"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "b"}]},
    {"pins": [{"element": 4, "pin": "no"}, {"element": 5, "pin": "a"}]},
    {"pins": [{"element": 5, "pin": "b"}, {"element": 6, "pin": "anode"}]},
    {"pins": [{"element": 6, "pin": "cathode"}, {"element": 3, "pin": "e"}, {"element": 1, "pin": "-"}, {"element": 0, "pin": "-"}, {"element": 7, "pin": "-"}]}
  ]
}
`,k=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 60.0, "y": 200.0, "turns": 1},
    {"kind": {"SignalGenerator": {"waveform": "Square", "volts": 5.0, "hertz": 1.0}}, "x": 60.0, "y": 380.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 4700.0}}, "x": 180.0, "y": 380.0, "turns": 0},
    {"kind": {"Bjt": {"polarity": "Npn"}}, "x": 300.0, "y": 380.0, "turns": 0},
    {"kind": {"Relay": {"part": "V9"}}, "x": 300.0, "y": 200.0, "turns": 0},
    {"kind": "Diode", "x": 200.0, "y": 200.0, "turns": 3},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 460.0, "y": 180.0, "turns": 0},
    {"kind": {"Led": {"colour": "Blue"}}, "x": 560.0, "y": 260.0, "turns": 1},
    {"kind": "Voltmeter", "x": 420.0, "y": 400.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 4, "pin": "coil+"}, {"element": 5, "pin": "cathode"}, {"element": 4, "pin": "com"}]},
    {"pins": [{"element": 4, "pin": "coil-"}, {"element": 5, "pin": "anode"}, {"element": 3, "pin": "c"}, {"element": 8, "pin": "+"}]},
    {"pins": [{"element": 1, "pin": "+"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "b"}]},
    {"pins": [{"element": 4, "pin": "no"}, {"element": 6, "pin": "a"}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 7, "pin": "anode"}]},
    {"pins": [{"element": 7, "pin": "cathode"}, {"element": 3, "pin": "e"}, {"element": 1, "pin": "-"}, {"element": 0, "pin": "-"}, {"element": 8, "pin": "-"}]}
  ]
}
`,A=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 80.0, "y": 260.0, "turns": 1},
    {"kind": {"PushButton": {"closed": false}}, "x": 200.0, "y": 140.0, "turns": 0},
    {"kind": {"Relay": {"part": "V9"}}, "x": 320.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 460.0, "y": 140.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 560.0, "y": 200.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 460.0, "y": 280.0, "turns": 0},
    {"kind": {"Led": {"colour": "Green"}}, "x": 560.0, "y": 340.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}, {"element": 2, "pin": "com"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "coil+"}]},
    {"pins": [{"element": 2, "pin": "no"}, {"element": 3, "pin": "a"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 4, "pin": "anode"}]},
    {"pins": [{"element": 2, "pin": "nc"}, {"element": 5, "pin": "a"}]},
    {"pins": [{"element": 5, "pin": "b"}, {"element": 6, "pin": "anode"}]},
    {"pins": [{"element": 2, "pin": "coil-"}, {"element": 4, "pin": "cathode"}, {"element": 6, "pin": "cathode"}, {"element": 0, "pin": "-"}]}
  ]
}
`,j=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 120.0, "y": 240.0, "turns": 0},
    {"kind": {"SevenSegment": {"common": "Cathode"}}, "x": 400.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 470.0}}, "x": 400.0, "y": 160.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 470.0}}, "x": 460.0, "y": 160.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 470.0}}, "x": 500.0, "y": 250.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 2, "pin": "a"}, {"element": 3, "pin": "a"}, {"element": 4, "pin": "b"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 1, "pin": "b"}]},
    {"pins": [{"element": 4, "pin": "a"}, {"element": 1, "pin": "c"}]},
    {"pins": [{"element": 1, "pin": "com"}, {"element": 0, "pin": "-"}]}
  ]
}
`,M=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 20.0, "y": 250.0, "turns": 1},
    {"kind": "Decoder", "x": 300.0, "y": 240.0, "turns": 0},
    {"kind": {"SevenSegment": {"common": "Cathode"}}, "x": 600.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 440.0, "y": 150.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 440.0, "y": 170.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 440.0, "y": 190.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 440.0, "y": 210.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 440.0, "y": 230.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 440.0, "y": 250.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 440.0, "y": 270.0, "turns": 0},
    {"kind": {"Switch": {"closed": true}}, "x": 220.0, "y": 120.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 220.0, "y": 370.0, "turns": 1},
    {"kind": {"Switch": {"closed": false}}, "x": 180.0, "y": 120.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 180.0, "y": 370.0, "turns": 1},
    {"kind": {"Switch": {"closed": false}}, "x": 140.0, "y": 120.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 140.0, "y": 370.0, "turns": 1},
    {"kind": {"Switch": {"closed": false}}, "x": 100.0, "y": 120.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 100.0, "y": 370.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "VDD"}, {"element": 1, "pin": "LT"}, {"element": 1, "pin": "BI"}, {"element": 10, "pin": "a"}, {"element": 12, "pin": "a"}, {"element": 14, "pin": "a"}, {"element": 16, "pin": "a"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "VSS"}, {"element": 1, "pin": "LE"}, {"element": 2, "pin": "com"}, {"element": 11, "pin": "b"}, {"element": 13, "pin": "b"}, {"element": 15, "pin": "b"}, {"element": 17, "pin": "b"}]},
    {"pins": [{"element": 10, "pin": "b"}, {"element": 1, "pin": "A"}, {"element": 11, "pin": "a"}]},
    {"pins": [{"element": 12, "pin": "b"}, {"element": 1, "pin": "B"}, {"element": 13, "pin": "a"}]},
    {"pins": [{"element": 14, "pin": "b"}, {"element": 1, "pin": "C"}, {"element": 15, "pin": "a"}]},
    {"pins": [{"element": 16, "pin": "b"}, {"element": 1, "pin": "D"}, {"element": 17, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "a"}, {"element": 3, "pin": "a"}], "waypoints": [{"x": 350.0, "y": 210.0}, {"x": 350.0, "y": 150.0}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 4, "pin": "a"}], "waypoints": [{"x": 360.0, "y": 220.0}, {"x": 360.0, "y": 170.0}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 2, "pin": "b"}]},
    {"pins": [{"element": 1, "pin": "c"}, {"element": 5, "pin": "a"}], "waypoints": [{"x": 370.0, "y": 230.0}, {"x": 370.0, "y": 190.0}]},
    {"pins": [{"element": 5, "pin": "b"}, {"element": 2, "pin": "c"}]},
    {"pins": [{"element": 1, "pin": "d"}, {"element": 6, "pin": "a"}], "waypoints": [{"x": 380.0, "y": 240.0}, {"x": 380.0, "y": 210.0}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 2, "pin": "d"}]},
    {"pins": [{"element": 1, "pin": "e"}, {"element": 7, "pin": "a"}], "waypoints": [{"x": 390.0, "y": 250.0}, {"x": 390.0, "y": 230.0}]},
    {"pins": [{"element": 7, "pin": "b"}, {"element": 2, "pin": "e"}]},
    {"pins": [{"element": 1, "pin": "f"}, {"element": 8, "pin": "a"}], "waypoints": [{"x": 400.0, "y": 260.0}, {"x": 400.0, "y": 250.0}]},
    {"pins": [{"element": 8, "pin": "b"}, {"element": 2, "pin": "f"}]},
    {"pins": [{"element": 1, "pin": "g"}, {"element": 9, "pin": "a"}]},
    {"pins": [{"element": 9, "pin": "b"}, {"element": 2, "pin": "g"}]}
  ]
}
`,N=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 40.0, "y": 300.0, "turns": 1},
    {"kind": "Timer", "x": 200.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 100.0, "y": 190.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 120.0, "y": 250.0, "turns": 0},
    {"kind": {"Capacitor": {"farads": 4.7e-05, "initial_volts": 0.0}}, "x": 140.0, "y": 310.0, "turns": 1},
    {"kind": "Counter", "x": 360.0, "y": 240.0, "turns": 0},
    {"kind": {"SevenSegment": {"common": "Cathode"}}, "x": 660.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 150.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 170.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 190.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 210.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 230.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 250.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 270.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 2, "pin": "a"}, {"element": 1, "pin": "VCC"}, {"element": 5, "pin": "VDD"}, {"element": 5, "pin": "DEI"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 1, "pin": "DIS"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 1, "pin": "TRIG"}, {"element": 1, "pin": "THR"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "OUT"}, {"element": 5, "pin": "CLK"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 4, "pin": "b"}, {"element": 1, "pin": "GND"}, {"element": 5, "pin": "VSS"}, {"element": 5, "pin": "RST"}, {"element": 5, "pin": "INH"}, {"element": 6, "pin": "com"}]},
    {"pins": [{"element": 5, "pin": "a"}, {"element": 7, "pin": "a"}], "waypoints": [{"x": 410.0, "y": 210.0}, {"x": 410.0, "y": 150.0}]},
    {"pins": [{"element": 7, "pin": "b"}, {"element": 6, "pin": "a"}]},
    {"pins": [{"element": 5, "pin": "b"}, {"element": 8, "pin": "a"}], "waypoints": [{"x": 420.0, "y": 220.0}, {"x": 420.0, "y": 170.0}]},
    {"pins": [{"element": 8, "pin": "b"}, {"element": 6, "pin": "b"}]},
    {"pins": [{"element": 5, "pin": "c"}, {"element": 9, "pin": "a"}], "waypoints": [{"x": 430.0, "y": 230.0}, {"x": 430.0, "y": 190.0}]},
    {"pins": [{"element": 9, "pin": "b"}, {"element": 6, "pin": "c"}]},
    {"pins": [{"element": 5, "pin": "d"}, {"element": 10, "pin": "a"}], "waypoints": [{"x": 440.0, "y": 240.0}, {"x": 440.0, "y": 210.0}]},
    {"pins": [{"element": 10, "pin": "b"}, {"element": 6, "pin": "d"}]},
    {"pins": [{"element": 5, "pin": "e"}, {"element": 11, "pin": "a"}], "waypoints": [{"x": 450.0, "y": 250.0}, {"x": 450.0, "y": 230.0}]},
    {"pins": [{"element": 11, "pin": "b"}, {"element": 6, "pin": "e"}]},
    {"pins": [{"element": 5, "pin": "f"}, {"element": 12, "pin": "a"}], "waypoints": [{"x": 460.0, "y": 260.0}, {"x": 460.0, "y": 250.0}]},
    {"pins": [{"element": 12, "pin": "b"}, {"element": 6, "pin": "f"}]},
    {"pins": [{"element": 5, "pin": "g"}, {"element": 13, "pin": "a"}]},
    {"pins": [{"element": 13, "pin": "b"}, {"element": 6, "pin": "g"}]}
  ]
}
`,P=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 120.0, "y": 240.0, "turns": 0},
    {"kind": {"PushButton": {"closed": false}}, "x": 260.0, "y": 140.0, "turns": 0},
    {"kind": {"SevenSegment": {"common": "Cathode"}}, "x": 400.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 460.0, "y": 160.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 250.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 4, "pin": "b"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 2, "pin": "b"}]},
    {"pins": [{"element": 4, "pin": "a"}, {"element": 2, "pin": "c"}]},
    {"pins": [{"element": 2, "pin": "com"}, {"element": 0, "pin": "-"}]}
  ]
}
`,F=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 80.0, "y": 240.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 47000.0}}, "x": 220.0, "y": 140.0, "turns": 1},
    {"kind": {"Ldr": {"light": 0.8}}, "x": 220.0, "y": 340.0, "turns": 1},
    {"kind": {"Bjt": {"polarity": "Npn"}}, "x": 340.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 360.0, "y": 100.0, "turns": 1},
    {"kind": {"Led": {"colour": "White"}}, "x": 360.0, "y": 180.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}, {"element": 3, "pin": "b"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 5, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "cathode"}, {"element": 3, "pin": "c"}]},
    {"pins": [{"element": 3, "pin": "e"}, {"element": 2, "pin": "b"}, {"element": 0, "pin": "-"}]}
  ]
}
`,I=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 80.0, "y": 240.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 100000.0}}, "x": 220.0, "y": 140.0, "turns": 1},
    {"kind": {"Phototransistor": {"light": 0.8}}, "x": 220.0, "y": 340.0, "turns": 0},
    {"kind": {"Bjt": {"polarity": "Npn"}}, "x": 340.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 360.0, "y": 100.0, "turns": 1},
    {"kind": {"Led": {"colour": "White"}}, "x": 360.0, "y": 180.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "c"}, {"element": 3, "pin": "b"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 5, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "cathode"}, {"element": 3, "pin": "c"}]},
    {"pins": [{"element": 3, "pin": "e"}, {"element": 2, "pin": "e"}, {"element": 0, "pin": "-"}]}
  ]
}
`,L=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 100.0, "y": 240.0, "turns": 1},
    {"kind": {"Phototransistor": {"light": 0.6}}, "x": 240.0, "y": 180.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 240.0, "y": 300.0, "turns": 1},
    {"kind": "Voltmeter", "x": 340.0, "y": 300.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "c"}]},
    {"pins": [{"element": 1, "pin": "e"}, {"element": 2, "pin": "a"}, {"element": 3, "pin": "+"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 0, "pin": "-"}, {"element": 3, "pin": "-"}]}
  ]
}
`,R=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"Cell": {"part": "NineVolt", "charge": 1.0}}, "x": 60.0, "y": 220.0, "turns": 1},
    {"kind": "Motor", "x": 280.0, "y": 220.0, "turns": 1},
    {"kind": "Voltmeter", "x": 360.0, "y": 220.0, "turns": 1},
    {"kind": {"Cell": {"part": "FourAa", "charge": 1.0}}, "x": 500.0, "y": 220.0, "turns": 1},
    {"kind": "Motor", "x": 720.0, "y": 220.0, "turns": 1},
    {"kind": "Voltmeter", "x": 800.0, "y": 220.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}, {"element": 2, "pin": "+"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "b"}, {"element": 2, "pin": "-"}]},
    {"pins": [{"element": 3, "pin": "+"}, {"element": 4, "pin": "a"}, {"element": 5, "pin": "+"}]},
    {"pins": [{"element": 3, "pin": "-"}, {"element": 4, "pin": "b"}, {"element": 5, "pin": "-"}]}
  ]
}
`,z=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"Cell": {"part": "Coin", "charge": 1.0}}, "x": 80.0, "y": 220.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 150.0}}, "x": 200.0, "y": 140.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 300.0, "y": 220.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "anode"}]},
    {"pins": [{"element": 2, "pin": "cathode"}, {"element": 0, "pin": "-"}]}
  ]
}
`,B=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 60.0, "y": 240.0, "turns": 1},
    {"kind": {"Fuse": {"amps": 0.5, "part": "Fast", "blown": false}}, "x": 160.0, "y": 140.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 470.0}}, "x": 300.0, "y": 140.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 400.0, "y": 240.0, "turns": 1},
    {"kind": {"Switch": {"closed": false}}, "x": 240.0, "y": 240.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "anode"}]},
    {"pins": [{"element": 3, "pin": "cathode"}, {"element": 0, "pin": "-"}, {"element": 4, "pin": "b"}]}
  ]
}
`,V=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 60.0, "y": 240.0, "turns": 1},
    {"kind": {"Fuse": {"amps": 0.5, "part": "Polyfuse", "blown": false}}, "x": 160.0, "y": 140.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 470.0}}, "x": 300.0, "y": 140.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 400.0, "y": 240.0, "turns": 1},
    {"kind": {"Switch": {"closed": false}}, "x": 240.0, "y": 240.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "anode"}]},
    {"pins": [{"element": 3, "pin": "cathode"}, {"element": 0, "pin": "-"}, {"element": 4, "pin": "b"}]}
  ]
}
`,H=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 40.0, "y": 300.0, "turns": 1},
    {"kind": "Lm3914", "x": 340.0, "y": 300.0, "turns": 0},
    {"kind": "Bargraph", "x": 500.0, "y": 300.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 1200.0}}, "x": 220.0, "y": 420.0, "turns": 1},
    {"kind": {"Ldr": {"light": 0.6}}, "x": 160.0, "y": 220.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 160.0, "y": 360.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "V+"}, {"element": 1, "pin": "MODE"}, {"element": 2, "pin": "a1"}, {"element": 2, "pin": "a2"}, {"element": 2, "pin": "a3"}, {"element": 2, "pin": "a4"}, {"element": 2, "pin": "a5"}, {"element": 2, "pin": "a6"}, {"element": 2, "pin": "a7"}, {"element": 2, "pin": "a8"}, {"element": 2, "pin": "a9"}, {"element": 2, "pin": "a10"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "V-"}, {"element": 1, "pin": "RLO"}, {"element": 1, "pin": "REFADJ"}, {"element": 3, "pin": "b"}, {"element": 5, "pin": "b"}]},
    {"pins": [{"element": 1, "pin": "REFOUT"}, {"element": 1, "pin": "RHI"}, {"element": 3, "pin": "a"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 5, "pin": "a"}, {"element": 1, "pin": "SIG"}]},
    {"pins": [{"element": 2, "pin": "k1"}, {"element": 1, "pin": "L1"}]},
    {"pins": [{"element": 2, "pin": "k2"}, {"element": 1, "pin": "L2"}]},
    {"pins": [{"element": 2, "pin": "k3"}, {"element": 1, "pin": "L3"}]},
    {"pins": [{"element": 2, "pin": "k4"}, {"element": 1, "pin": "L4"}]},
    {"pins": [{"element": 2, "pin": "k5"}, {"element": 1, "pin": "L5"}]},
    {"pins": [{"element": 2, "pin": "k6"}, {"element": 1, "pin": "L6"}]},
    {"pins": [{"element": 2, "pin": "k7"}, {"element": 1, "pin": "L7"}]},
    {"pins": [{"element": 2, "pin": "k8"}, {"element": 1, "pin": "L8"}]},
    {"pins": [{"element": 2, "pin": "k9"}, {"element": 1, "pin": "L9"}]},
    {"pins": [{"element": 2, "pin": "k10"}, {"element": 1, "pin": "L10"}]}
  ]
}
`,U=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 40.0, "y": 300.0, "turns": 1},
    {"kind": "Lm3914", "x": 340.0, "y": 300.0, "turns": 0},
    {"kind": "Bargraph", "x": 500.0, "y": 300.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 2200.0}}, "x": 220.0, "y": 420.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 5600.0}}, "x": 220.0, "y": 500.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 8200.0}}, "x": 140.0, "y": 180.0, "turns": 1},
    {"kind": {"Potentiometer": {"ohms": 10000.0, "wiper": 0.75}}, "x": 140.0, "y": 300.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "V+"}, {"element": 1, "pin": "MODE"}, {"element": 5, "pin": "a"}, {"element": 2, "pin": "a1"}, {"element": 2, "pin": "a2"}, {"element": 2, "pin": "a3"}, {"element": 2, "pin": "a4"}, {"element": 2, "pin": "a5"}, {"element": 2, "pin": "a6"}, {"element": 2, "pin": "a7"}, {"element": 2, "pin": "a8"}, {"element": 2, "pin": "a9"}, {"element": 2, "pin": "a10"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "V-"}, {"element": 1, "pin": "RLO"}, {"element": 4, "pin": "b"}, {"element": 6, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "REFOUT"}, {"element": 1, "pin": "RHI"}, {"element": 3, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "REFADJ"}, {"element": 3, "pin": "b"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 5, "pin": "b"}, {"element": 6, "pin": "b"}]},
    {"pins": [{"element": 6, "pin": "wiper"}, {"element": 1, "pin": "SIG"}]},
    {"pins": [{"element": 2, "pin": "k1"}, {"element": 1, "pin": "L1"}]},
    {"pins": [{"element": 2, "pin": "k2"}, {"element": 1, "pin": "L2"}]},
    {"pins": [{"element": 2, "pin": "k3"}, {"element": 1, "pin": "L3"}]},
    {"pins": [{"element": 2, "pin": "k4"}, {"element": 1, "pin": "L4"}]},
    {"pins": [{"element": 2, "pin": "k5"}, {"element": 1, "pin": "L5"}]},
    {"pins": [{"element": 2, "pin": "k6"}, {"element": 1, "pin": "L6"}]},
    {"pins": [{"element": 2, "pin": "k7"}, {"element": 1, "pin": "L7"}]},
    {"pins": [{"element": 2, "pin": "k8"}, {"element": 1, "pin": "L8"}]},
    {"pins": [{"element": 2, "pin": "k9"}, {"element": 1, "pin": "L9"}]},
    {"pins": [{"element": 2, "pin": "k10"}, {"element": 1, "pin": "L10"}]}
  ]
}
`,W=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"SolarCell": {"sun": 1.0}}, "x": 100.0, "y": 240.0, "turns": 1},
    {"kind": "Ammeter", "x": 240.0, "y": 140.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 40.0}}, "x": 360.0, "y": 240.0, "turns": 1},
    {"kind": "Voltmeter", "x": 460.0, "y": 240.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "+"}]},
    {"pins": [{"element": 1, "pin": "-"}, {"element": 2, "pin": "a"}, {"element": 3, "pin": "+"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 0, "pin": "-"}, {"element": 3, "pin": "-"}]}
  ]
}
`,G=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"SolarCell": {"sun": 0.1}}, "x": 100.0, "y": 240.0, "turns": 1},
    {"kind": "Diode", "x": 240.0, "y": 140.0, "turns": 0},
    {"kind": {"Electrolytic": {"farads": 0.0047, "initial_volts": 0.0, "rated": "V16"}}, "x": 360.0, "y": 240.0, "turns": 1},
    {"kind": "Voltmeter", "x": 460.0, "y": 240.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "anode"}]},
    {"pins": [{"element": 1, "pin": "cathode"}, {"element": 2, "pin": "+"}, {"element": 3, "pin": "+"}]},
    {"pins": [{"element": 2, "pin": "-"}, {"element": 0, "pin": "-"}, {"element": 3, "pin": "-"}]}
  ]
}
`,K=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 100.0, "y": 240.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 240.0, "y": 140.0, "turns": 1},
    {"kind": {"Thermistor": {"celsius": 25.0}}, "x": 240.0, "y": 340.0, "turns": 1},
    {"kind": "Voltmeter", "x": 340.0, "y": 340.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}, {"element": 3, "pin": "+"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "-"}, {"element": 0, "pin": "-"}]}
  ]
}
`,q=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 120.0, "y": 240.0, "turns": 1},
    {"kind": {"Tmp36": {"celsius": 25.0}}, "x": 260.0, "y": 240.0, "turns": 0},
    {"kind": "Voltmeter", "x": 380.0, "y": 280.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "+Vs"}]},
    {"pins": [{"element": 1, "pin": "Vout"}, {"element": 2, "pin": "+"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "GND"}, {"element": 2, "pin": "-"}]}
  ]
}
`,J=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 120.0, "y": 220.0, "turns": 0},
    {"kind": {"PushButton": {"closed": false}}, "x": 240.0, "y": 120.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 150.0}}, "x": 360.0, "y": 120.0, "turns": 0},
    {"kind": "Buzzer", "x": 460.0, "y": 220.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "+"}]},
    {"pins": [{"element": 3, "pin": "-"}, {"element": 0, "pin": "-"}]}
  ]
}
`,Y=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 60.0, "y": 260.0, "turns": 1},
    {"kind": {"Thermistor": {"celsius": 25.0}}, "x": 180.0, "y": 160.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 2200.0}}, "x": 180.0, "y": 360.0, "turns": 1},
    {"kind": {"Mosfet": {"channel": "N"}}, "x": 300.0, "y": 260.0, "turns": 0},
    {"kind": {"Relay": {"part": "V9"}}, "x": 420.0, "y": 160.0, "turns": 0},
    {"kind": "Diode", "x": 300.0, "y": 120.0, "turns": 3},
    {"kind": "Motor", "x": 560.0, "y": 260.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}, {"element": 4, "pin": "coil+"}, {"element": 5, "pin": "cathode"}, {"element": 4, "pin": "com"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}, {"element": 3, "pin": "g"}]},
    {"pins": [{"element": 4, "pin": "coil-"}, {"element": 5, "pin": "anode"}, {"element": 3, "pin": "d"}]},
    {"pins": [{"element": 4, "pin": "no"}, {"element": 6, "pin": "a"}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 3, "pin": "s"}, {"element": 2, "pin": "b"}, {"element": 0, "pin": "-"}]}
  ]
}
`,X=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"SignalGenerator": {"waveform": "Square", "volts": 5.0, "hertz": 1.0}}, "x": 120.0, "y": 200.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 240.0, "y": 120.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 360.0, "y": 200.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "anode"}]},
    {"pins": [{"element": 2, "pin": "cathode"}, {"element": 0, "pin": "-"}]}
  ]
}
`,Z=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 120.0, "y": 280.0, "turns": 0},
    {"kind": {"SignalGenerator": {"waveform": "Square", "volts": 5.0, "hertz": 1.0}}, "x": 120.0, "y": 160.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 280.0, "y": 100.0, "turns": 0},
    {"kind": {"Led": {"colour": "Blue"}}, "x": 400.0, "y": 220.0, "turns": 1},
    {"kind": "Voltmeter", "x": 40.0, "y": 220.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "-"}]},
    {"pins": [{"element": 1, "pin": "+"}, {"element": 2, "pin": "a"}, {"element": 4, "pin": "+"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "anode"}]},
    {"pins": [{"element": 3, "pin": "cathode"}, {"element": 0, "pin": "-"}, {"element": 4, "pin": "-"}]}
  ]
}
`,Q=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"SignalGenerator": {"waveform": "Sine", "volts": 5.0, "hertz": 1.0}}, "x": 120.0, "y": 220.0, "turns": 0},
    {"kind": "Diode", "x": 240.0, "y": 100.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 360.0, "y": 220.0, "turns": 1},
    {"kind": "Voltmeter", "x": 460.0, "y": 220.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "anode"}]},
    {"pins": [{"element": 1, "pin": "cathode"}, {"element": 2, "pin": "a"}, {"element": 3, "pin": "+"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 0, "pin": "-"}, {"element": 3, "pin": "-"}]}
  ]
}
`,ne=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"SignalGenerator": {"waveform": "Square", "volts": 5.0, "hertz": 1.0}}, "x": 120.0, "y": 220.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 240.0, "y": 100.0, "turns": 0},
    {"kind": {"Capacitor": {"farads": 0.0001, "initial_volts": 0.0}}, "x": 360.0, "y": 220.0, "turns": 1},
    {"kind": "Voltmeter", "x": 460.0, "y": 220.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}, {"element": 3, "pin": "+"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 0, "pin": "-"}, {"element": 3, "pin": "-"}]}
  ]
}
`,re=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"SignalGenerator": {"waveform": "Square", "volts": 5.0, "hertz": 1.0}}, "x": 120.0, "y": 220.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 240.0, "y": 100.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 360.0, "y": 220.0, "turns": 1},
    {"kind": {"Led": {"colour": "Green"}}, "x": 460.0, "y": 220.0, "turns": 3}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "anode"}, {"element": 3, "pin": "cathode"}]},
    {"pins": [{"element": 2, "pin": "cathode"}, {"element": 3, "pin": "anode"}, {"element": 0, "pin": "-"}]}
  ]
}
`,ie=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    { "kind": { "DcSource": { "volts": 9.0, "series_ohms": 0.0 } }, "x": 120.0, "y": 200.0, "turns": 0 },
    { "kind": { "Regulator": { "part": "L7805" } }, "x": 240.0, "y": 120.0, "turns": 0 },
    { "kind": "Voltmeter", "x": 360.0, "y": 200.0, "turns": 1 },
    { "kind": { "Resistor": { "ohms": 150.0 } }, "x": 460.0, "y": 120.0, "turns": 0 },
    { "kind": { "Led": { "colour": "Green" } }, "x": 560.0, "y": 200.0, "turns": 1 }
  ],
  "nets": [
    { "pins": [{ "element": 0, "pin": "+" }, { "element": 1, "pin": "in" }] },
    { "pins": [{ "element": 1, "pin": "out" }, { "element": 2, "pin": "+" }, { "element": 3, "pin": "a" }] },
    { "pins": [{ "element": 3, "pin": "b" }, { "element": 4, "pin": "anode" }] },
    { "pins": [{ "element": 4, "pin": "cathode" }, { "element": 2, "pin": "-" }, { "element": 1, "pin": "gnd" }, { "element": 0, "pin": "-" }] }
  ]
}
`,ae=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"Capacitor": {"farads": 0.01, "initial_volts": 5.0}}, "x": 120.0, "y": 220.0, "turns": 1},
    {"kind": {"Inductor": {"henries": 2.5, "initial_amps": 0.0}}, "x": 360.0, "y": 220.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 1.0}}, "x": 240.0, "y": 100.0, "turns": 0},
    {"kind": "Voltmeter", "x": 40.0, "y": 220.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "a"}, {"element": 2, "pin": "a"}, {"element": 3, "pin": "+"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 0, "pin": "b"}, {"element": 3, "pin": "-"}]}
  ]
}
`,oe=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 120.0, "y": 200.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 240.0, "y": 120.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 360.0, "y": 120.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 480.0, "y": 200.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "anode"}]},
    {"pins": [{"element": 2, "pin": "cathode"}, {"element": 3, "pin": "anode"}]},
    {"pins": [{"element": 3, "pin": "cathode"}, {"element": 0, "pin": "-"}]}
  ]
}
`,$=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 160.0, "y": 300.0, "turns": 1},
    {"kind": "Timer", "x": 460.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 340.0, "y": 140.0, "turns": 1},
    {"kind": {"Potentiometer": {"ohms": 10000.0, "wiper": 0.5}}, "x": 340.0, "y": 210.0, "turns": 1},
    {"kind": "Diode", "x": 380.0, "y": 280.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 270000.0}}, "x": 380.0, "y": 320.0, "turns": 0},
    {"kind": {"Capacitor": {"farads": 1e-07, "initial_volts": 0.0}}, "x": 420.0, "y": 380.0, "turns": 1},
    {"kind": "Servo", "x": 600.0, "y": 240.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 2, "pin": "a"}, {"element": 1, "pin": "VCC"}, {"element": 7, "pin": "+5V"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}]},
    {"pins": [{"element": 3, "pin": "wiper"}, {"element": 1, "pin": "DIS"}, {"element": 4, "pin": "anode"}, {"element": 5, "pin": "a"}]},
    {"pins": [{"element": 4, "pin": "cathode"}, {"element": 5, "pin": "b"}, {"element": 1, "pin": "THR"}, {"element": 1, "pin": "TRIG"}, {"element": 6, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "OUT"}, {"element": 7, "pin": "SIG"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "GND"}, {"element": 6, "pin": "b"}, {"element": 7, "pin": "GND"}]}
  ]
}
`,se=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 6.0, "series_ohms": 0.0}}, "x": 60.0, "y": 120.0, "turns": 1},
    {"kind": "MotorDriver", "x": 300.0, "y": 240.0, "turns": 0},
    {"kind": "Motor", "x": 400.0, "y": 150.0, "turns": 0},
    {"kind": {"Switch": {"closed": true}}, "x": 220.0, "y": 120.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 220.0, "y": 370.0, "turns": 1},
    {"kind": {"Switch": {"closed": false}}, "x": 180.0, "y": 120.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 180.0, "y": 370.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "VCC1"}, {"element": 1, "pin": "VCC2"}, {"element": 1, "pin": "1,2EN"}, {"element": 3, "pin": "a"}, {"element": 5, "pin": "a"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "GND"}, {"element": 4, "pin": "b"}, {"element": 6, "pin": "b"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 1, "pin": "1A"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 5, "pin": "b"}, {"element": 1, "pin": "2A"}, {"element": 6, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "1Y"}, {"element": 2, "pin": "a"}], "waypoints": [{"x": 360.0, "y": 220.0}, {"x": 360.0, "y": 150.0}]},
    {"pins": [{"element": 1, "pin": "2Y"}, {"element": 2, "pin": "b"}], "waypoints": [{"x": 440.0, "y": 230.0}, {"x": 440.0, "y": 150.0}]}
  ]
}
`,ce=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 40.0, "y": 200.0, "turns": 1},
    {"kind": {"PushButton": {"closed": false}}, "x": 100.0, "y": 140.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 180.0, "y": 140.0, "turns": 0},
    {"kind": "Optocoupler", "x": 260.0, "y": 170.0, "turns": 0},
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 480.0, "y": 200.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 420.0, "y": 120.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 340.0, "y": 120.0, "turns": 2}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "A"}]},
    {"pins": [{"element": 3, "pin": "K"}, {"element": 0, "pin": "-"}]},
    {"pins": [{"element": 4, "pin": "+"}, {"element": 5, "pin": "b"}]},
    {"pins": [{"element": 5, "pin": "a"}, {"element": 6, "pin": "anode"}]},
    {"pins": [{"element": 6, "pin": "cathode"}, {"element": 3, "pin": "C"}]},
    {"pins": [{"element": 3, "pin": "E"}, {"element": 4, "pin": "-"}]}
  ]
}
`,le=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 40.0, "y": 300.0, "turns": 1},
    {"kind": "ShiftRegister", "x": 400.0, "y": 300.0, "turns": 0},
    {"kind": {"PushButton": {"closed": false}}, "x": 200.0, "y": 120.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 200.0, "y": 470.0, "turns": 1},
    {"kind": {"PushButton": {"closed": false}}, "x": 240.0, "y": 120.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 240.0, "y": 470.0, "turns": 1},
    {"kind": {"PushButton": {"closed": false}}, "x": 280.0, "y": 120.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 280.0, "y": 470.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 160.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 180.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 200.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 220.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 260.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 280.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 300.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 160.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 180.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 200.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 220.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 240.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 260.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 280.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 300.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "VCC"}, {"element": 1, "pin": "SRCLR"}, {"element": 2, "pin": "a"}, {"element": 4, "pin": "a"}, {"element": 6, "pin": "a"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "GND"}, {"element": 1, "pin": "OE"}, {"element": 3, "pin": "b"}, {"element": 5, "pin": "b"}, {"element": 7, "pin": "b"}, {"element": 16, "pin": "cathode"}, {"element": 17, "pin": "cathode"}, {"element": 18, "pin": "cathode"}, {"element": 19, "pin": "cathode"}, {"element": 20, "pin": "cathode"}, {"element": 21, "pin": "cathode"}, {"element": 22, "pin": "cathode"}, {"element": 23, "pin": "cathode"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 1, "pin": "SER"}, {"element": 3, "pin": "a"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 1, "pin": "SRCLK"}, {"element": 5, "pin": "a"}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 1, "pin": "RCLK"}, {"element": 7, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "QA"}, {"element": 8, "pin": "a"}], "waypoints": [{"x": 460.0, "y": 260.0}, {"x": 460.0, "y": 160.0}]},
    {"pins": [{"element": 8, "pin": "b"}, {"element": 16, "pin": "anode"}]},
    {"pins": [{"element": 1, "pin": "QB"}, {"element": 9, "pin": "a"}], "waypoints": [{"x": 470.0, "y": 270.0}, {"x": 470.0, "y": 180.0}]},
    {"pins": [{"element": 9, "pin": "b"}, {"element": 17, "pin": "anode"}]},
    {"pins": [{"element": 1, "pin": "QC"}, {"element": 10, "pin": "a"}], "waypoints": [{"x": 480.0, "y": 280.0}, {"x": 480.0, "y": 200.0}]},
    {"pins": [{"element": 10, "pin": "b"}, {"element": 18, "pin": "anode"}]},
    {"pins": [{"element": 1, "pin": "QD"}, {"element": 11, "pin": "a"}], "waypoints": [{"x": 490.0, "y": 290.0}, {"x": 490.0, "y": 220.0}]},
    {"pins": [{"element": 11, "pin": "b"}, {"element": 19, "pin": "anode"}]},
    {"pins": [{"element": 1, "pin": "QE"}, {"element": 12, "pin": "a"}], "waypoints": [{"x": 500.0, "y": 300.0}, {"x": 500.0, "y": 240.0}]},
    {"pins": [{"element": 12, "pin": "b"}, {"element": 20, "pin": "anode"}]},
    {"pins": [{"element": 1, "pin": "QF"}, {"element": 13, "pin": "a"}], "waypoints": [{"x": 510.0, "y": 310.0}, {"x": 510.0, "y": 260.0}]},
    {"pins": [{"element": 13, "pin": "b"}, {"element": 21, "pin": "anode"}]},
    {"pins": [{"element": 1, "pin": "QG"}, {"element": 14, "pin": "a"}], "waypoints": [{"x": 520.0, "y": 320.0}, {"x": 520.0, "y": 280.0}]},
    {"pins": [{"element": 14, "pin": "b"}, {"element": 22, "pin": "anode"}]},
    {"pins": [{"element": 1, "pin": "QH"}, {"element": 15, "pin": "a"}], "waypoints": [{"x": 530.0, "y": 330.0}, {"x": 530.0, "y": 300.0}]},
    {"pins": [{"element": 15, "pin": "b"}, {"element": 23, "pin": "anode"}]}
  ]
}
`,ue=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 60.0, "y": 240.0, "turns": 1},
    {"kind": {"Gate": {"logic": "Nand"}}, "x": 300.0, "y": 240.0, "turns": 0},
    {"kind": {"Switch": {"closed": false}}, "x": 160.0, "y": 140.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 160.0, "y": 340.0, "turns": 1},
    {"kind": {"Switch": {"closed": false}}, "x": 200.0, "y": 140.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 200.0, "y": 340.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 420.0, "y": 200.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 500.0, "y": 200.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "VCC"}, {"element": 2, "pin": "a"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "GND"}, {"element": 3, "pin": "b"}, {"element": 5, "pin": "b"}, {"element": 7, "pin": "cathode"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 1, "pin": "1A"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 5, "pin": "a"}, {"element": 1, "pin": "1B"}]},
    {"pins": [{"element": 1, "pin": "1Y"}, {"element": 6, "pin": "a"}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 7, "pin": "anode"}]}
  ]
}
`,de=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 60.0, "y": 240.0, "turns": 1},
    {"kind": {"Gate": {"logic": "And"}}, "x": 300.0, "y": 240.0, "turns": 0},
    {"kind": {"Switch": {"closed": false}}, "x": 160.0, "y": 140.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 160.0, "y": 340.0, "turns": 1},
    {"kind": {"Switch": {"closed": false}}, "x": 200.0, "y": 140.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 200.0, "y": 340.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 420.0, "y": 200.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 500.0, "y": 200.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "VCC"}, {"element": 2, "pin": "a"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "GND"}, {"element": 3, "pin": "b"}, {"element": 5, "pin": "b"}, {"element": 7, "pin": "cathode"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 1, "pin": "1A"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 5, "pin": "a"}, {"element": 1, "pin": "1B"}]},
    {"pins": [{"element": 1, "pin": "1Y"}, {"element": 6, "pin": "a"}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 7, "pin": "anode"}]}
  ]
}
`,fe=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 60.0, "y": 240.0, "turns": 1},
    {"kind": {"Gate": {"logic": "Or"}}, "x": 300.0, "y": 240.0, "turns": 0},
    {"kind": {"Switch": {"closed": false}}, "x": 160.0, "y": 140.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 160.0, "y": 340.0, "turns": 1},
    {"kind": {"Switch": {"closed": false}}, "x": 200.0, "y": 140.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 200.0, "y": 340.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 420.0, "y": 200.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 500.0, "y": 200.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "VCC"}, {"element": 2, "pin": "a"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "GND"}, {"element": 3, "pin": "b"}, {"element": 5, "pin": "b"}, {"element": 7, "pin": "cathode"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 1, "pin": "1A"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 5, "pin": "a"}, {"element": 1, "pin": "1B"}]},
    {"pins": [{"element": 1, "pin": "1Y"}, {"element": 6, "pin": "a"}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 7, "pin": "anode"}]}
  ]
}
`,pe=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 60.0, "y": 240.0, "turns": 1},
    {"kind": {"Gate": {"logic": "Xor"}}, "x": 300.0, "y": 240.0, "turns": 0},
    {"kind": {"Switch": {"closed": false}}, "x": 160.0, "y": 140.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 160.0, "y": 340.0, "turns": 1},
    {"kind": {"Switch": {"closed": false}}, "x": 200.0, "y": 140.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 200.0, "y": 340.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 420.0, "y": 200.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 500.0, "y": 200.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "VCC"}, {"element": 2, "pin": "a"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "GND"}, {"element": 3, "pin": "b"}, {"element": 5, "pin": "b"}, {"element": 7, "pin": "cathode"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 1, "pin": "1A"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 5, "pin": "a"}, {"element": 1, "pin": "1B"}]},
    {"pins": [{"element": 1, "pin": "1Y"}, {"element": 6, "pin": "a"}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 7, "pin": "anode"}]}
  ]
}
`,me=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 60.0, "y": 240.0, "turns": 1},
    {"kind": "Inverter", "x": 300.0, "y": 240.0, "turns": 0},
    {"kind": {"Switch": {"closed": false}}, "x": 180.0, "y": 140.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 180.0, "y": 340.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 420.0, "y": 210.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 500.0, "y": 210.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "VCC"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "GND"}, {"element": 3, "pin": "b"}, {"element": 5, "pin": "cathode"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 1, "pin": "1A"}]},
    {"pins": [{"element": 1, "pin": "1Y"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 5, "pin": "anode"}]}
  ]
}
`,he=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 60.0, "y": 240.0, "turns": 1},
    {"kind": {"Gate": {"logic": "Nand"}}, "x": 300.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 160.0, "y": 140.0, "turns": 1},
    {"kind": {"PushButton": {"closed": false}}, "x": 160.0, "y": 340.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 200.0, "y": 140.0, "turns": 1},
    {"kind": {"PushButton": {"closed": false}}, "x": 200.0, "y": 340.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 420.0, "y": 200.0, "turns": 0},
    {"kind": {"Led": {"colour": "Green"}}, "x": 500.0, "y": 200.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 420.0, "y": 260.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 500.0, "y": 260.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "VCC"}, {"element": 2, "pin": "a"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "GND"}, {"element": 3, "pin": "b"}, {"element": 5, "pin": "b"}, {"element": 7, "pin": "cathode"}, {"element": 9, "pin": "cathode"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 1, "pin": "1A"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 5, "pin": "a"}, {"element": 1, "pin": "2B"}]},
    {"pins": [{"element": 1, "pin": "1Y"}, {"element": 1, "pin": "2A"}, {"element": 6, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "2Y"}, {"element": 1, "pin": "1B"}, {"element": 8, "pin": "a"}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 7, "pin": "anode"}]},
    {"pins": [{"element": 8, "pin": "b"}, {"element": 9, "pin": "anode"}]}
  ]
}
`,ge=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 60.0, "y": 320.0, "turns": 1},
    {"kind": {"Switch": {"closed": false}}, "x": 160.0, "y": 140.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 160.0, "y": 520.0, "turns": 1},
    {"kind": {"Switch": {"closed": false}}, "x": 200.0, "y": 140.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 200.0, "y": 520.0, "turns": 1},
    {"kind": {"Gate": {"logic": "Nand"}}, "x": 340.0, "y": 240.0, "turns": 0},
    {"kind": {"Gate": {"logic": "Nand"}}, "x": 340.0, "y": 420.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 460.0, "y": 200.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 540.0, "y": 200.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 460.0, "y": 250.0, "turns": 0},
    {"kind": {"Led": {"colour": "Green"}}, "x": 540.0, "y": 250.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 100.0}}, "x": 460.0, "y": 450.0, "turns": 0},
    {"kind": {"Led": {"colour": "Blue"}}, "x": 540.0, "y": 450.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 5, "pin": "VCC"}, {"element": 6, "pin": "VCC"}, {"element": 1, "pin": "a"}, {"element": 3, "pin": "a"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 5, "pin": "GND"}, {"element": 6, "pin": "GND"}, {"element": 2, "pin": "b"}, {"element": 4, "pin": "b"}, {"element": 8, "pin": "cathode"}, {"element": 10, "pin": "cathode"}, {"element": 12, "pin": "cathode"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}, {"element": 5, "pin": "1A"}, {"element": 5, "pin": "1B"}, {"element": 5, "pin": "2A"}, {"element": 6, "pin": "1A"}, {"element": 6, "pin": "1B"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 4, "pin": "a"}, {"element": 5, "pin": "2B"}, {"element": 6, "pin": "2A"}, {"element": 6, "pin": "2B"}]},
    {"pins": [{"element": 5, "pin": "1Y"}, {"element": 7, "pin": "a"}]},
    {"pins": [{"element": 7, "pin": "b"}, {"element": 8, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "2Y"}, {"element": 5, "pin": "3A"}, {"element": 5, "pin": "3B"}]},
    {"pins": [{"element": 5, "pin": "3Y"}, {"element": 9, "pin": "a"}]},
    {"pins": [{"element": 9, "pin": "b"}, {"element": 10, "pin": "anode"}]},
    {"pins": [{"element": 6, "pin": "1Y"}, {"element": 6, "pin": "3A"}]},
    {"pins": [{"element": 6, "pin": "2Y"}, {"element": 6, "pin": "3B"}]},
    {"pins": [{"element": 6, "pin": "3Y"}, {"element": 11, "pin": "a"}]},
    {"pins": [{"element": 11, "pin": "b"}, {"element": 12, "pin": "anode"}]}
  ]
}
`,_e=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 60.0, "y": 240.0, "turns": 1},
    {"kind": "Schmitt", "x": 300.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 1000000.0}}, "x": 300.0, "y": 140.0, "turns": 0},
    {"kind": {"Capacitor": {"farads": 1e-06, "initial_volts": 0.0}}, "x": 200.0, "y": 300.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 420.0, "y": 210.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 500.0, "y": 210.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "VCC"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "GND"}, {"element": 3, "pin": "b"}, {"element": 5, "pin": "cathode"}]},
    {"pins": [{"element": 1, "pin": "1Y"}, {"element": 2, "pin": "b"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "a"}, {"element": 1, "pin": "1A"}, {"element": 3, "pin": "a"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 5, "pin": "anode"}]}
  ]
}
`,ve=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 40.0, "y": 300.0, "turns": 1},
    {"kind": {"PushButton": {"closed": false, "bounces": true}}, "x": 160.0, "y": 160.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 160.0, "y": 340.0, "turns": 1},
    {"kind": "Counter", "x": 360.0, "y": 240.0, "turns": 0},
    {"kind": {"SevenSegment": {"common": "Cathode"}}, "x": 660.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 150.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 170.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 190.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 210.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 230.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 250.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 500.0, "y": 270.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}, {"element": 3, "pin": "VDD"}, {"element": 3, "pin": "DEI"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}, {"element": 3, "pin": "CLK"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 2, "pin": "b"}, {"element": 3, "pin": "VSS"}, {"element": 3, "pin": "RST"}, {"element": 3, "pin": "INH"}, {"element": 4, "pin": "com"}]},
    {"pins": [{"element": 3, "pin": "a"}, {"element": 5, "pin": "a"}], "waypoints": [{"x": 410.0, "y": 210.0}, {"x": 410.0, "y": 150.0}]},
    {"pins": [{"element": 5, "pin": "b"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 6, "pin": "a"}], "waypoints": [{"x": 420.0, "y": 220.0}, {"x": 420.0, "y": 170.0}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 4, "pin": "b"}]},
    {"pins": [{"element": 3, "pin": "c"}, {"element": 7, "pin": "a"}], "waypoints": [{"x": 430.0, "y": 230.0}, {"x": 430.0, "y": 190.0}]},
    {"pins": [{"element": 7, "pin": "b"}, {"element": 4, "pin": "c"}]},
    {"pins": [{"element": 3, "pin": "d"}, {"element": 8, "pin": "a"}], "waypoints": [{"x": 440.0, "y": 240.0}, {"x": 440.0, "y": 210.0}]},
    {"pins": [{"element": 8, "pin": "b"}, {"element": 4, "pin": "d"}]},
    {"pins": [{"element": 3, "pin": "e"}, {"element": 9, "pin": "a"}], "waypoints": [{"x": 450.0, "y": 250.0}, {"x": 450.0, "y": 230.0}]},
    {"pins": [{"element": 9, "pin": "b"}, {"element": 4, "pin": "e"}]},
    {"pins": [{"element": 3, "pin": "f"}, {"element": 10, "pin": "a"}], "waypoints": [{"x": 460.0, "y": 260.0}, {"x": 460.0, "y": 250.0}]},
    {"pins": [{"element": 10, "pin": "b"}, {"element": 4, "pin": "f"}]},
    {"pins": [{"element": 3, "pin": "g"}, {"element": 11, "pin": "a"}]},
    {"pins": [{"element": 11, "pin": "b"}, {"element": 4, "pin": "g"}]}
  ]
}
`,ye=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 40.0, "y": 300.0, "turns": 1},
    {"kind": {"PushButton": {"closed": false, "bounces": true}}, "x": 120.0, "y": 160.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 180.0, "y": 180.0, "turns": 0},
    {"kind": {"Capacitor": {"farads": 1e-06, "initial_volts": 0.0}}, "x": 220.0, "y": 300.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 100000.0}}, "x": 260.0, "y": 300.0, "turns": 1},
    {"kind": "Schmitt", "x": 360.0, "y": 240.0, "turns": 0},
    {"kind": "Counter", "x": 520.0, "y": 240.0, "turns": 0},
    {"kind": {"SevenSegment": {"common": "Cathode"}}, "x": 820.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 660.0, "y": 150.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 660.0, "y": 170.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 660.0, "y": 190.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 660.0, "y": 210.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 660.0, "y": 230.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 660.0, "y": 250.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 660.0, "y": 270.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}, {"element": 5, "pin": "VCC"}, {"element": 6, "pin": "VDD"}, {"element": 6, "pin": "DEI"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 4, "pin": "a"}, {"element": 5, "pin": "1A"}]},
    {"pins": [{"element": 5, "pin": "1Y"}, {"element": 5, "pin": "2A"}], "waypoints": [{"x": 410.0, "y": 210.0}, {"x": 410.0, "y": 310.0}, {"x": 310.0, "y": 310.0}, {"x": 310.0, "y": 220.0}]},
    {"pins": [{"element": 5, "pin": "2Y"}, {"element": 6, "pin": "CLK"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 3, "pin": "b"}, {"element": 4, "pin": "b"}, {"element": 5, "pin": "GND"}, {"element": 6, "pin": "VSS"}, {"element": 6, "pin": "RST"}, {"element": 6, "pin": "INH"}, {"element": 7, "pin": "com"}]},
    {"pins": [{"element": 6, "pin": "a"}, {"element": 8, "pin": "a"}], "waypoints": [{"x": 570.0, "y": 210.0}, {"x": 570.0, "y": 150.0}]},
    {"pins": [{"element": 8, "pin": "b"}, {"element": 7, "pin": "a"}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 9, "pin": "a"}], "waypoints": [{"x": 580.0, "y": 220.0}, {"x": 580.0, "y": 170.0}]},
    {"pins": [{"element": 9, "pin": "b"}, {"element": 7, "pin": "b"}]},
    {"pins": [{"element": 6, "pin": "c"}, {"element": 10, "pin": "a"}], "waypoints": [{"x": 590.0, "y": 230.0}, {"x": 590.0, "y": 190.0}]},
    {"pins": [{"element": 10, "pin": "b"}, {"element": 7, "pin": "c"}]},
    {"pins": [{"element": 6, "pin": "d"}, {"element": 11, "pin": "a"}], "waypoints": [{"x": 600.0, "y": 240.0}, {"x": 600.0, "y": 210.0}]},
    {"pins": [{"element": 11, "pin": "b"}, {"element": 7, "pin": "d"}]},
    {"pins": [{"element": 6, "pin": "e"}, {"element": 12, "pin": "a"}], "waypoints": [{"x": 610.0, "y": 250.0}, {"x": 610.0, "y": 230.0}]},
    {"pins": [{"element": 12, "pin": "b"}, {"element": 7, "pin": "e"}]},
    {"pins": [{"element": 6, "pin": "f"}, {"element": 13, "pin": "a"}], "waypoints": [{"x": 620.0, "y": 260.0}, {"x": 620.0, "y": 250.0}]},
    {"pins": [{"element": 13, "pin": "b"}, {"element": 7, "pin": "f"}]},
    {"pins": [{"element": 6, "pin": "g"}, {"element": 14, "pin": "a"}]},
    {"pins": [{"element": 14, "pin": "b"}, {"element": 7, "pin": "g"}]}
  ]
}
`,be=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 40.0, "y": 300.0, "turns": 1},
    {"kind": "Timer", "x": 200.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 100.0, "y": 190.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 120.0, "y": 250.0, "turns": 0},
    {"kind": {"Capacitor": {"farads": 1e-05, "initial_volts": 0.0}}, "x": 140.0, "y": 310.0, "turns": 1},
    {"kind": "Decade", "x": 380.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 140.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 140.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 160.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 160.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 180.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 180.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 200.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 200.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 220.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 220.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 240.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 260.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 260.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 280.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 280.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 300.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 300.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 320.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 320.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 2, "pin": "a"}, {"element": 1, "pin": "VCC"}, {"element": 5, "pin": "VDD"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 1, "pin": "DIS"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 1, "pin": "TRIG"}, {"element": 1, "pin": "THR"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "OUT"}, {"element": 5, "pin": "CLK"}]},
    {"pins": [{"element": 5, "pin": "Q0"}, {"element": 6, "pin": "a"}], "waypoints": [{"x": 450.0, "y": 190.0}, {"x": 450.0, "y": 140.0}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 7, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "Q1"}, {"element": 8, "pin": "a"}], "waypoints": [{"x": 460.0, "y": 200.0}, {"x": 460.0, "y": 160.0}]},
    {"pins": [{"element": 8, "pin": "b"}, {"element": 9, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "Q2"}, {"element": 10, "pin": "a"}], "waypoints": [{"x": 470.0, "y": 210.0}, {"x": 470.0, "y": 180.0}]},
    {"pins": [{"element": 10, "pin": "b"}, {"element": 11, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "Q3"}, {"element": 12, "pin": "a"}], "waypoints": [{"x": 480.0, "y": 220.0}, {"x": 480.0, "y": 200.0}]},
    {"pins": [{"element": 12, "pin": "b"}, {"element": 13, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "Q4"}, {"element": 14, "pin": "a"}], "waypoints": [{"x": 490.0, "y": 230.0}, {"x": 490.0, "y": 220.0}]},
    {"pins": [{"element": 14, "pin": "b"}, {"element": 15, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "Q5"}, {"element": 16, "pin": "a"}]},
    {"pins": [{"element": 16, "pin": "b"}, {"element": 17, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "Q6"}, {"element": 18, "pin": "a"}], "waypoints": [{"x": 480.0, "y": 250.0}, {"x": 480.0, "y": 260.0}]},
    {"pins": [{"element": 18, "pin": "b"}, {"element": 19, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "Q7"}, {"element": 20, "pin": "a"}], "waypoints": [{"x": 470.0, "y": 260.0}, {"x": 470.0, "y": 280.0}]},
    {"pins": [{"element": 20, "pin": "b"}, {"element": 21, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "Q8"}, {"element": 22, "pin": "a"}], "waypoints": [{"x": 460.0, "y": 270.0}, {"x": 460.0, "y": 300.0}]},
    {"pins": [{"element": 22, "pin": "b"}, {"element": 23, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "Q9"}, {"element": 24, "pin": "a"}], "waypoints": [{"x": 450.0, "y": 280.0}, {"x": 450.0, "y": 320.0}]},
    {"pins": [{"element": 24, "pin": "b"}, {"element": 25, "pin": "anode"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 4, "pin": "b"}, {"element": 1, "pin": "GND"}, {"element": 5, "pin": "VSS"}, {"element": 5, "pin": "INH"}, {"element": 5, "pin": "RST"}, {"element": 7, "pin": "cathode"}, {"element": 9, "pin": "cathode"}, {"element": 11, "pin": "cathode"}, {"element": 13, "pin": "cathode"}, {"element": 15, "pin": "cathode"}, {"element": 17, "pin": "cathode"}, {"element": 19, "pin": "cathode"}, {"element": 21, "pin": "cathode"}, {"element": 23, "pin": "cathode"}, {"element": 25, "pin": "cathode"}]}
  ]
}
`,xe=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 40.0, "y": 300.0, "turns": 1},
    {"kind": "Timer", "x": 200.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 100.0, "y": 190.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 120.0, "y": 250.0, "turns": 0},
    {"kind": {"Capacitor": {"farads": 1e-05, "initial_volts": 0.0}}, "x": 140.0, "y": 310.0, "turns": 1},
    {"kind": "Decade", "x": 380.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 140.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 140.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 160.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 160.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 180.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 180.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 200.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 200.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 2, "pin": "a"}, {"element": 1, "pin": "VCC"}, {"element": 5, "pin": "VDD"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 1, "pin": "DIS"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 1, "pin": "TRIG"}, {"element": 1, "pin": "THR"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "OUT"}, {"element": 5, "pin": "CLK"}]},
    {"pins": [{"element": 5, "pin": "Q4"}, {"element": 5, "pin": "RST"}]},
    {"pins": [{"element": 5, "pin": "Q0"}, {"element": 6, "pin": "a"}], "waypoints": [{"x": 450.0, "y": 190.0}, {"x": 450.0, "y": 140.0}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 7, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "Q1"}, {"element": 8, "pin": "a"}], "waypoints": [{"x": 460.0, "y": 200.0}, {"x": 460.0, "y": 160.0}]},
    {"pins": [{"element": 8, "pin": "b"}, {"element": 9, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "Q2"}, {"element": 10, "pin": "a"}], "waypoints": [{"x": 470.0, "y": 210.0}, {"x": 470.0, "y": 180.0}]},
    {"pins": [{"element": 10, "pin": "b"}, {"element": 11, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "Q3"}, {"element": 12, "pin": "a"}], "waypoints": [{"x": 480.0, "y": 220.0}, {"x": 480.0, "y": 200.0}]},
    {"pins": [{"element": 12, "pin": "b"}, {"element": 13, "pin": "anode"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 4, "pin": "b"}, {"element": 1, "pin": "GND"}, {"element": 5, "pin": "VSS"}, {"element": 5, "pin": "INH"}, {"element": 7, "pin": "cathode"}, {"element": 9, "pin": "cathode"}, {"element": 11, "pin": "cathode"}, {"element": 13, "pin": "cathode"}]}
  ]
}
`,Se=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 40.0, "y": 300.0, "turns": 1},
    {"kind": {"PushButton": {"closed": false, "bounces": true}}, "x": 120.0, "y": 160.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 180.0, "y": 180.0, "turns": 0},
    {"kind": {"Capacitor": {"farads": 1e-06, "initial_volts": 0.0}}, "x": 220.0, "y": 300.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 100000.0}}, "x": 260.0, "y": 300.0, "turns": 1},
    {"kind": "Schmitt", "x": 360.0, "y": 240.0, "turns": 0},
    {"kind": "FlipFlop", "x": 520.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 100.0}}, "x": 640.0, "y": 200.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 720.0, "y": 200.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}, {"element": 5, "pin": "VCC"}, {"element": 6, "pin": "VDD"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 4, "pin": "a"}, {"element": 5, "pin": "1A"}]},
    {"pins": [{"element": 5, "pin": "1Y"}, {"element": 5, "pin": "2A"}], "waypoints": [{"x": 410.0, "y": 210.0}, {"x": 410.0, "y": 310.0}, {"x": 310.0, "y": 310.0}, {"x": 310.0, "y": 220.0}]},
    {"pins": [{"element": 5, "pin": "2Y"}, {"element": 6, "pin": "CLK1"}]},
    {"pins": [{"element": 6, "pin": "/Q1"}, {"element": 6, "pin": "D1"}], "waypoints": [{"x": 580.0, "y": 230.0}, {"x": 580.0, "y": 180.0}, {"x": 460.0, "y": 180.0}, {"x": 460.0, "y": 200.0}]},
    {"pins": [{"element": 6, "pin": "Q1"}, {"element": 7, "pin": "a"}]},
    {"pins": [{"element": 7, "pin": "b"}, {"element": 8, "pin": "anode"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 3, "pin": "b"}, {"element": 4, "pin": "b"}, {"element": 5, "pin": "GND"}, {"element": 6, "pin": "VSS"}, {"element": 6, "pin": "SET1"}, {"element": 6, "pin": "RST1"}, {"element": 8, "pin": "cathode"}]}
  ]
}
`,Ce=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 40.0, "y": 300.0, "turns": 1},
    {"kind": "Timer", "x": 200.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 100.0, "y": 190.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 120.0, "y": 250.0, "turns": 0},
    {"kind": {"Capacitor": {"farads": 2.2e-05, "initial_volts": 0.0}}, "x": 140.0, "y": 310.0, "turns": 1},
    {"kind": "FlipFlop", "x": 420.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 120.0, "turns": 0},
    {"kind": {"Led": {"colour": "Red"}}, "x": 640.0, "y": 120.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 200.0, "turns": 0},
    {"kind": {"Led": {"colour": "Green"}}, "x": 640.0, "y": 200.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 560.0, "y": 250.0, "turns": 0},
    {"kind": {"Led": {"colour": "Blue"}}, "x": 640.0, "y": 250.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 2, "pin": "a"}, {"element": 1, "pin": "VCC"}, {"element": 5, "pin": "VDD"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 1, "pin": "DIS"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 1, "pin": "TRIG"}, {"element": 1, "pin": "THR"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "OUT"}, {"element": 5, "pin": "CLK1"}, {"element": 6, "pin": "a"}]},
    {"pins": [{"element": 5, "pin": "/Q1"}, {"element": 5, "pin": "D1"}], "waypoints": [{"x": 480.0, "y": 230.0}, {"x": 480.0, "y": 180.0}, {"x": 360.0, "y": 180.0}, {"x": 360.0, "y": 200.0}]},
    {"pins": [{"element": 5, "pin": "Q1"}, {"element": 5, "pin": "CLK2"}, {"element": 8, "pin": "a"}], "waypoints": [{"x": 490.0, "y": 200.0}, {"x": 490.0, "y": 300.0}, {"x": 350.0, "y": 300.0}, {"x": 350.0, "y": 260.0}]},
    {"pins": [{"element": 5, "pin": "/Q2"}, {"element": 5, "pin": "D2"}], "waypoints": [{"x": 480.0, "y": 280.0}, {"x": 480.0, "y": 310.0}, {"x": 360.0, "y": 310.0}, {"x": 360.0, "y": 250.0}]},
    {"pins": [{"element": 5, "pin": "Q2"}, {"element": 10, "pin": "a"}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 7, "pin": "anode"}]},
    {"pins": [{"element": 8, "pin": "b"}, {"element": 9, "pin": "anode"}]},
    {"pins": [{"element": 10, "pin": "b"}, {"element": 11, "pin": "anode"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 4, "pin": "b"}, {"element": 1, "pin": "GND"}, {"element": 5, "pin": "VSS"}, {"element": 5, "pin": "SET1"}, {"element": 5, "pin": "RST1"}, {"element": 5, "pin": "SET2"}, {"element": 5, "pin": "RST2"}, {"element": 7, "pin": "cathode"}, {"element": 9, "pin": "cathode"}, {"element": 11, "pin": "cathode"}]}
  ]
}
`,we=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 6.0, "series_ohms": 0.0}}, "x": 40.0, "y": 200.0, "turns": 1},
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 120.0, "y": 400.0, "turns": 1},
    {"kind": {"Switch": {"closed": false}}, "x": 200.0, "y": 300.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 200.0, "y": 420.0, "turns": 1},
    {"kind": "Motor", "x": 360.0, "y": 140.0, "turns": 1},
    {"kind": "Diode", "x": 420.0, "y": 140.0, "turns": 3},
    {"kind": {"Mosfet": {"channel": "Power"}}, "x": 360.0, "y": 280.0, "turns": 0},
    {"kind": "Motor", "x": 560.0, "y": 140.0, "turns": 1},
    {"kind": "Diode", "x": 620.0, "y": 140.0, "turns": 3},
    {"kind": {"Mosfet": {"channel": "N"}}, "x": 560.0, "y": 280.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 4, "pin": "a"}, {"element": 5, "pin": "cathode"}, {"element": 7, "pin": "a"}, {"element": 8, "pin": "cathode"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 5, "pin": "anode"}, {"element": 6, "pin": "d"}]},
    {"pins": [{"element": 7, "pin": "b"}, {"element": 8, "pin": "anode"}, {"element": 9, "pin": "d"}]},
    {"pins": [{"element": 1, "pin": "+"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 6, "pin": "g"}, {"element": 9, "pin": "g"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "-"}, {"element": 3, "pin": "b"}, {"element": 6, "pin": "s"}, {"element": 9, "pin": "s"}]}
  ]
}
`,Te=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 40.0, "y": 260.0, "turns": 1},
    {"kind": {"PushButton": {"closed": false}}, "x": 140.0, "y": 200.0, "turns": 1},
    {"kind": "Ammeter", "x": 240.0, "y": 280.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 160.0, "y": 280.0, "turns": 0},
    {"kind": {"Bjt": {"polarity": "Darlington"}}, "x": 360.0, "y": 280.0, "turns": 0},
    {"kind": "Ammeter", "x": 360.0, "y": 200.0, "turns": 1},
    {"kind": "Motor", "x": 360.0, "y": 100.0, "turns": 1},
    {"kind": "Diode", "x": 420.0, "y": 100.0, "turns": 3}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}, {"element": 6, "pin": "a"}, {"element": 7, "pin": "cathode"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 3, "pin": "a"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 2, "pin": "+"}]},
    {"pins": [{"element": 2, "pin": "-"}, {"element": 4, "pin": "b"}]},
    {"pins": [{"element": 6, "pin": "b"}, {"element": 7, "pin": "anode"}, {"element": 5, "pin": "+"}]},
    {"pins": [{"element": 5, "pin": "-"}, {"element": 4, "pin": "c"}]},
    {"pins": [{"element": 4, "pin": "e"}, {"element": 0, "pin": "-"}]}
  ]
}
`,Ee=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 12.0, "series_ohms": 0.0}}, "x": 40.0, "y": 240.0, "turns": 1},
    {"kind": "Adjustable", "x": 200.0, "y": 160.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 240.0}}, "x": 260.0, "y": 220.0, "turns": 1},
    {"kind": {"Potentiometer": {"ohms": 1500.0, "wiper": 0.5}}, "x": 200.0, "y": 320.0, "turns": 0},
    {"kind": "Voltmeter", "x": 360.0, "y": 240.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 440.0, "y": 240.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "in"}]},
    {"pins": [{"element": 1, "pin": "out"}, {"element": 2, "pin": "a"}, {"element": 4, "pin": "+"}, {"element": 5, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 1, "pin": "adj"}, {"element": 3, "pin": "a"}]},
    {"pins": [{"element": 3, "pin": "wiper"}, {"element": 3, "pin": "b"}, {"element": 0, "pin": "-"}, {"element": 4, "pin": "-"}, {"element": 5, "pin": "b"}]}
  ]
}
`,De=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 40.0, "y": 240.0, "turns": 1},
    {"kind": "Adjustable", "x": 200.0, "y": 160.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 62.0}}, "x": 260.0, "y": 220.0, "turns": 1},
    {"kind": "Ammeter", "x": 320.0, "y": 280.0, "turns": 1},
    {"kind": {"Led": {"colour": "Red"}}, "x": 320.0, "y": 360.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "in"}]},
    {"pins": [{"element": 1, "pin": "out"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 1, "pin": "adj"}, {"element": 3, "pin": "+"}]},
    {"pins": [{"element": 3, "pin": "-"}, {"element": 4, "pin": "anode"}]},
    {"pins": [{"element": 4, "pin": "cathode"}, {"element": 0, "pin": "-"}]}
  ]
}
`,Oe=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 40.0, "y": 240.0, "turns": 1},
    {"kind": "Diode", "x": 160.0, "y": 160.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 470.0}}, "x": 240.0, "y": 240.0, "turns": 1},
    {"kind": "Voltmeter", "x": 300.0, "y": 240.0, "turns": 1},
    {"kind": "Schottky", "x": 400.0, "y": 160.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 470.0}}, "x": 480.0, "y": 240.0, "turns": 1},
    {"kind": "Voltmeter", "x": 540.0, "y": 240.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "anode"}, {"element": 4, "pin": "anode"}]},
    {"pins": [{"element": 1, "pin": "cathode"}, {"element": 2, "pin": "a"}, {"element": 3, "pin": "+"}]},
    {"pins": [{"element": 4, "pin": "cathode"}, {"element": 5, "pin": "a"}, {"element": 6, "pin": "+"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 2, "pin": "b"}, {"element": 3, "pin": "-"}, {"element": 5, "pin": "b"}, {"element": 6, "pin": "-"}]}
  ]
}
`,ke=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 40.0, "y": 240.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 200.0, "y": 160.0, "turns": 1},
    {"kind": {"Reed": {"magnet": 0.8333}}, "x": 200.0, "y": 320.0, "turns": 1},
    {"kind": {"Mosfet": {"channel": "N"}}, "x": 320.0, "y": 240.0, "turns": 0},
    {"kind": "Buzzer", "x": 320.0, "y": 140.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}, {"element": 4, "pin": "+"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}, {"element": 3, "pin": "g"}]},
    {"pins": [{"element": 4, "pin": "-"}, {"element": 3, "pin": "d"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 2, "pin": "b"}, {"element": 3, "pin": "s"}]}
  ]
}
`,Ae=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 40.0, "y": 240.0, "turns": 1},
    {"kind": {"Hall": {"magnet": 0.0}}, "x": 200.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 300.0, "y": 140.0, "turns": 1},
    {"kind": {"Led": {"colour": "Red"}}, "x": 300.0, "y": 200.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "VCC"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "anode"}]},
    {"pins": [{"element": 3, "pin": "cathode"}, {"element": 1, "pin": "OUT"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "GND"}]}
  ]
}
`,je=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 5.0, "series_ohms": 0.0}}, "x": 40.0, "y": 240.0, "turns": 1},
    {"kind": {"Pir": {"moving": false, "hold": 5.0, "retriggers": true}}, "x": 200.0, "y": 240.0, "turns": 0},
    {"kind": {"Mosfet": {"channel": "N"}}, "x": 320.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 320.0, "y": 100.0, "turns": 1},
    {"kind": {"Led": {"colour": "White"}}, "x": 320.0, "y": 180.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "VCC"}, {"element": 3, "pin": "a"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 4, "pin": "anode"}]},
    {"pins": [{"element": 4, "pin": "cathode"}, {"element": 2, "pin": "d"}]},
    {"pins": [{"element": 1, "pin": "OUT"}, {"element": 2, "pin": "g"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "GND"}, {"element": 2, "pin": "s"}]}
  ]
}
`,Me=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"SignalGenerator": {"waveform": "Sine", "volts": 5.0, "hertz": 50.0}}, "x": 120.0, "y": 220.0, "turns": 0},
    {"kind": "Diode", "x": 240.0, "y": 100.0, "turns": 0},
    {"kind": {"Electrolytic": {"farads": 0.00047, "initial_volts": 0.0, "rated": "V16"}}, "x": 360.0, "y": 220.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 460.0, "y": 220.0, "turns": 1},
    {"kind": "Voltmeter", "x": 560.0, "y": 220.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "anode"}]},
    {"pins": [{"element": 1, "pin": "cathode"}, {"element": 2, "pin": "+"}, {"element": 3, "pin": "a"}, {"element": 4, "pin": "+"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 2, "pin": "-"}, {"element": 0, "pin": "-"}, {"element": 4, "pin": "-"}]}
  ]
}
`,Ne=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from. The sine stands in for the mains: never wire a real circuit to the mains.",
  "elements": [
    {"kind": {"SignalGenerator": {"waveform": "Sine", "volts": 24.0, "hertz": 50.0}}, "x": 60.0, "y": 240.0, "turns": 1},
    {"kind": {"Transformer": {"ratio": 2.0}}, "x": 180.0, "y": 240.0, "turns": 0},
    {"kind": "Bridge", "x": 300.0, "y": 320.0, "turns": 0},
    {"kind": {"Electrolytic": {"farads": 0.001, "initial_volts": 0.0, "rated": "V16"}}, "x": 400.0, "y": 240.0, "turns": 1},
    {"kind": {"Regulator": {"part": "L7805"}}, "x": 500.0, "y": 160.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 220.0}}, "x": 620.0, "y": 220.0, "turns": 1},
    {"kind": {"Led": {"colour": "Red"}}, "x": 620.0, "y": 300.0, "turns": 1},
    {"kind": "Voltmeter", "x": 460.0, "y": 320.0, "turns": 1},
    {"kind": "Voltmeter", "x": 720.0, "y": 260.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "p1"}]},
    {"pins": [{"element": 1, "pin": "p2"}, {"element": 0, "pin": "-"}]},
    {"pins": [{"element": 1, "pin": "s1"}, {"element": 2, "pin": "ac1"}]},
    {"pins": [{"element": 1, "pin": "s2"}, {"element": 2, "pin": "ac2"}]},
    {"pins": [{"element": 2, "pin": "+"}, {"element": 3, "pin": "+"}, {"element": 4, "pin": "in"}, {"element": 7, "pin": "+"}]},
    {"pins": [{"element": 4, "pin": "out"}, {"element": 5, "pin": "a"}, {"element": 8, "pin": "+"}]},
    {"pins": [{"element": 5, "pin": "b"}, {"element": 6, "pin": "anode"}]},
    {"pins": [{"element": 2, "pin": "-"}, {"element": 3, "pin": "-"}, {"element": 4, "pin": "gnd"}, {"element": 6, "pin": "cathode"}, {"element": 7, "pin": "-"}, {"element": 8, "pin": "-"}]}
  ]
}
`,Pe=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 100.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 1000.0}}, "x": 240.0, "y": 120.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 2000.0}}, "x": 400.0, "y": 120.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 3000.0}}, "x": 520.0, "y": 240.0, "turns": 1},
    {"kind": "Voltmeter", "x": 240.0, "y": 40.0, "turns": 0},
    {"kind": "Voltmeter", "x": 400.0, "y": 40.0, "turns": 0},
    {"kind": "Voltmeter", "x": 600.0, "y": 240.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}, {"element": 4, "pin": "+"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}, {"element": 4, "pin": "-"}, {"element": 5, "pin": "+"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "a"}, {"element": 5, "pin": "-"}, {"element": 6, "pin": "+"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 0, "pin": "-"}, {"element": 6, "pin": "-"}]}
  ]
}
`,Fe=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    { "kind": { "DcSource": { "volts": 9.0, "series_ohms": 0.0 } }, "x": 120.0, "y": 200.0, "turns": 0 },
    { "kind": { "Switch": { "closed": true } }, "x": 240.0, "y": 120.0, "turns": 0 },
    { "kind": { "Resistor": { "ohms": 470.0 } }, "x": 360.0, "y": 120.0, "turns": 0 },
    { "kind": { "Led": { "colour": "Green" } }, "x": 480.0, "y": 200.0, "turns": 0 }
  ],
  "nets": [
    { "pins": [{ "element": 0, "pin": "+" }, { "element": 1, "pin": "a" }] },
    { "pins": [{ "element": 1, "pin": "b" }, { "element": 2, "pin": "a" }] },
    { "pins": [{ "element": 2, "pin": "b" }, { "element": 3, "pin": "anode" }] },
    { "pins": [{ "element": 3, "pin": "cathode" }, { "element": 0, "pin": "-" }] }
  ]
}
`,Ie=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 120.0, "y": 200.0, "turns": 0},
    {"kind": {"Regulator": {"part": "Lm1117"}}, "x": 240.0, "y": 120.0, "turns": 0},
    {"kind": "Voltmeter", "x": 360.0, "y": 200.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 100.0}}, "x": 460.0, "y": 120.0, "turns": 0},
    {"kind": {"Led": {"colour": "Blue"}}, "x": 560.0, "y": 200.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "in"}]},
    {"pins": [{"element": 1, "pin": "out"}, {"element": 2, "pin": "+"}, {"element": 3, "pin": "a"}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 4, "pin": "anode"}]},
    {"pins": [{"element": 4, "pin": "cathode"}, {"element": 2, "pin": "-"}, {"element": 1, "pin": "gnd"}, {"element": 0, "pin": "-"}]}
  ]
}
`,Le=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    { "kind": { "DcSource": { "volts": 9.0, "series_ohms": 0.0 } }, "x": 100.0, "y": 260.0, "turns": 0 },
    { "kind": { "Potentiometer": { "ohms": 10000.0, "wiper": 0.3 } }, "x": 220.0, "y": 300.0, "turns": 1 },
    { "kind": { "Resistor": { "ohms": 10000.0 } }, "x": 300.0, "y": 160.0, "turns": 1 },
    { "kind": { "Resistor": { "ohms": 12000.0 } }, "x": 300.0, "y": 380.0, "turns": 1 },
    { "kind": "Comparator", "x": 400.0, "y": 260.0, "turns": 0 },
    { "kind": { "Resistor": { "ohms": 1000.0 } }, "x": 500.0, "y": 140.0, "turns": 1 },
    { "kind": { "Led": { "colour": "Red" } }, "x": 500.0, "y": 220.0, "turns": 1 }
  ],
  "nets": [
    { "pins": [{ "element": 0, "pin": "+" }, { "element": 1, "pin": "a" }, { "element": 2, "pin": "a" }, { "element": 5, "pin": "a" }, { "element": 4, "pin": "V+" }] },
    { "pins": [{ "element": 1, "pin": "wiper" }, { "element": 4, "pin": "in+" }] },
    { "pins": [{ "element": 2, "pin": "b" }, { "element": 3, "pin": "a" }, { "element": 4, "pin": "in-" }] },
    { "pins": [{ "element": 5, "pin": "b" }, { "element": 6, "pin": "anode" }] },
    { "pins": [{ "element": 6, "pin": "cathode" }, { "element": 4, "pin": "out" }] },
    { "pins": [{ "element": 0, "pin": "-" }, { "element": 1, "pin": "b" }, { "element": 3, "pin": "b" }, { "element": 4, "pin": "V-" }] }
  ]
}
`,Re=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 100.0, "y": 360.0, "turns": 1},
    {"kind": {"TiltSwitch": {"closed": true}}, "x": 240.0, "y": 380.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 220.0, "y": 180.0, "turns": 1},
    {"kind": {"Bjt": {"polarity": "Npn"}}, "x": 340.0, "y": 260.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 150.0}}, "x": 340.0, "y": 100.0, "turns": 1},
    {"kind": "Buzzer", "x": 340.0, "y": 180.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 2, "pin": "a"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 1, "pin": "a"}, {"element": 3, "pin": "b"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 5, "pin": "+"}]},
    {"pins": [{"element": 5, "pin": "-"}, {"element": 3, "pin": "c"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "b"}, {"element": 3, "pin": "e"}]}
  ]
}
`,ze=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 120.0, "y": 200.0, "turns": 0},
    {"kind": {"TiltSwitch": {"closed": true}}, "x": 240.0, "y": 120.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 360.0, "y": 120.0, "turns": 0},
    {"kind": {"Led": {"colour": "Green"}}, "x": 480.0, "y": 200.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "anode"}]},
    {"pins": [{"element": 3, "pin": "cathode"}, {"element": 0, "pin": "-"}]}
  ]
}
`,Be=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 100.0, "y": 240.0, "turns": 0},
    {"kind": {"Switch": {"closed": true}}, "x": 200.0, "y": 340.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 22000.0}}, "x": 300.0, "y": 340.0, "turns": 0},
    {"kind": {"Bjt": {"polarity": "Npn"}}, "x": 420.0, "y": 340.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 420.0, "y": 120.0, "turns": 1},
    {"kind": {"Led": {"colour": "Blue"}}, "x": 420.0, "y": 220.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 1, "pin": "b"}, {"element": 2, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 3, "pin": "b"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 5, "pin": "anode"}]},
    {"pins": [{"element": 5, "pin": "cathode"}, {"element": 3, "pin": "c"}]},
    {"pins": [{"element": 3, "pin": "e"}, {"element": 0, "pin": "-"}]}
  ]
}
`,Ve=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 80.0, "y": 260.0, "turns": 0},
    {"kind": {"Potentiometer": {"ohms": 10000.0, "wiper": 0.78}}, "x": 200.0, "y": 260.0, "turns": 1},
    {"kind": "OpAmp", "x": 380.0, "y": 260.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 320.0, "y": 400.0, "turns": 1},
    {"kind": {"Resistor": {"ohms": 10000.0}}, "x": 440.0, "y": 160.0, "turns": 0},
    {"kind": "Voltmeter", "x": 280.0, "y": 340.0, "turns": 1},
    {"kind": "Voltmeter", "x": 520.0, "y": 340.0, "turns": 1}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 1, "pin": "a"}, {"element": 2, "pin": "V+"}]},
    {"pins": [{"element": 1, "pin": "wiper"}, {"element": 2, "pin": "in+"}, {"element": 5, "pin": "+"}]},
    {"pins": [{"element": 2, "pin": "in-"}, {"element": 3, "pin": "a"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 2, "pin": "out"}, {"element": 6, "pin": "+"}]},
    {"pins": [{"element": 0, "pin": "-"}, {"element": 1, "pin": "b"}, {"element": 3, "pin": "b"}, {"element": 2, "pin": "V-"}, {"element": 5, "pin": "-"}, {"element": 6, "pin": "-"}]}
  ]
}
`,He=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "elements": [
    {"kind": {"DcSource": {"volts": 9.0, "series_ohms": 0.0}}, "x": 160.0, "y": 240.0, "turns": 1},
    {"kind": {"RgbLed": {"common": "Cathode"}}, "x": 420.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 390.0}}, "x": 280.0, "y": 160.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 280.0, "y": 240.0, "turns": 0},
    {"kind": {"Resistor": {"ohms": 330.0}}, "x": 280.0, "y": 320.0, "turns": 0}
  ],
  "nets": [
    {"pins": [{"element": 0, "pin": "+"}, {"element": 2, "pin": "a"}, {"element": 3, "pin": "a"}, {"element": 4, "pin": "a"}]},
    {"pins": [{"element": 2, "pin": "b"}, {"element": 1, "pin": "R"}], "waypoints": [{"x": 360.0, "y": 160.0}, {"x": 360.0, "y": 230.0}]},
    {"pins": [{"element": 3, "pin": "b"}, {"element": 1, "pin": "G"}]},
    {"pins": [{"element": 4, "pin": "b"}, {"element": 1, "pin": "B"}], "waypoints": [{"x": 360.0, "y": 320.0}, {"x": 360.0, "y": 250.0}]},
    {"pins": [{"element": 1, "pin": "com"}, {"element": 0, "pin": "-"}], "waypoints": [{"x": 460.0, "y": 240.0}, {"x": 460.0, "y": 380.0}, {"x": 160.0, "y": 380.0}]}
  ]
}
`,Ue=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "description": "A pot sets the blink\\nanalogRead gives the wiper's voltage as 0 to 1023; D13 waits that many milliseconds between changes.",
  "elements": [
    {
      "kind": "Uno",
      "x": 200,
      "y": 300,
      "turns": 0,
      "program": {
        "name": "AnalogInput",
        "hex": ":100000000C945C000C946E000C946E000C946E00CA\\r\\n:100010000C946E000C946E000C946E000C946E00A8\\r\\n:100020000C946E000C946E000C946E000C946E0098\\r\\n:100030000C946E000C946E000C946E000C946E0088\\r\\n:100040000C942C010C946E000C946E000C946E00B9\\r\\n:100050000C946E000C946E000C946E000C946E0068\\r\\n:100060000C946E000C946E00000000002400270029\\r\\n:100070002A0000000000250028002B0004040404CE\\r\\n:100080000404040402020202020203030303030342\\r\\n:10009000010204081020408001020408102001021F\\r\\n:1000A00004081020000000080002010000030407FB\\r\\n:1000B000000000000000000011241FBECFEFD8E0B8\\r\\n:1000C000DEBFCDBF21E0A0E0B1E001C01D92A930AC\\r\\n:1000D000B207E1F70E9474010C9406020C94000030\\r\\n:1000E000E1EBF0E02491EDE9F0E09491E9E8F0E053\\r\\n:1000F000E491EE23D9F0222349F0243031F1B8F411\\r\\n:10010000223059F12330D9F0213019F1F0E0EE0F0F\\r\\n:10011000FF1FEE58FF4FA591B4912FB7F894EC91C3\\r\\n:10012000811124C090959E239C932FBF0895273062\\r\\n:10013000B9F0283059F72091B0002F7D14C02091DC\\r\\n:1001400080002F7720938000E1CF209180002F7DC9\\r\\n:10015000F9CF24B52F7724BDD9CF24B52F7DFBCF80\\r\\n:100160002091B0002F772093B000D0CF9E2BDCCF12\\r\\n:10017000CF93DF9300D000D0CDB7DEB73FB7F89470\\r\\n:100180008091050190910601A0910701B0910801AD\\r\\n:1001900026B5A89B05C02F3F19F00196A11DB11DE2\\r\\n:1001A0003FBFBA2FA92F982F8827820F911DA11D1D\\r\\n:1001B000B11D9C01AD0162E0220F331F441F551F8A\\r\\n:1001C0006A95D1F7CA01B9010F900F900F900F9067\\r\\n:1001D000DF91CF9108958F929F92AF92BF92CF926D\\r\\n:1001E000DF92EF92FF92CF93DF9300D000D0CDB794\\r\\n:1001F000DEB76B017C010E94B8004B015C01C114A9\\r\\n:10020000D104E104F104C9F00E94B800DC01CB0183\\r\\n:1002100088199909AA09BB09883E9340A105B1052F\\r\\n:1002200070F321E0C21AD108E108F10888EE880EC7\\r\\n:1002300083E0981EA11CB11CE2CF0F900F900F908D\\r\\n:100240000F90DF91CF91FF90EF90DF90CF90BF9014\\r\\n:10025000AF909F908F9008951F921FB61F92112408\\r\\n:100260002F933F938F939F93AF93BF9380910101FF\\r\\n:1002700090910201A0910301B0910401309100011D\\r\\n:1002800023E0230F2D3750F50196A11DB11D2093BA\\r\\n:1002900000018093010190930201A0930301B093A8\\r\\n:1002A00004018091050190910601A0910701B09190\\r\\n:1002B00008010196A11DB11D8093050190930601CF\\r\\n:1002C000A0930701B0930801BF91AF919F918F91C7\\r\\n:1002D0003F912F911F901FBE1F90189526E8230F66\\r\\n:1002E0000296A11DB11DD3CF00D000D0CDB7DEB78F\\r\\n:1002F000789484B5826084BD84B5816084BD85B501\\r\\n:10030000826085BD85B5816085BD80916E0081600C\\r\\n:1003100080936E00109281008091810082608093B2\\r\\n:1003200081008091810081608093810080918000B4\\r\\n:100330008160809380008091B10084608093B100DF\\r\\n:100340008091B00081608093B00080917A008460D9\\r\\n:1003500080937A0080917A00826080937A00809105\\r\\n:100360007A00816080937A0080917A00806880931F\\r\\n:100370007A001092C100EDE9F0E02491E9E8F0E0A4\\r\\n:100380008491882399F090E0880F991FFC01E85927\\r\\n:10039000FF4FA591B491FC01EE58FF4F85919491C8\\r\\n:1003A0008FB7F894EC91E22BEC938FBF00E010E054\\r\\n:1003B00080E480937C0080917A00806480937A004E\\r\\n:1003C00080917A0086FDFCCFE0907800F090790073\\r\\n:1003D00081E00E947000C701FF0CAA0BBB0B898350\\r\\n:1003E0009A83AB83BC83BC01CD010E94EB0080E00B\\r\\n:1003F0000E94700069817A818B819C810E94EB0050\\r\\n:1004000001151105A9F20E940000D2CFF894FFCF88\\r\\n:00000001FF\\r\\n"
      }
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 220.0
        }
      },
      "x": 370,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "Led": {
          "colour": "Red"
        }
      },
      "x": 460,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "Potentiometer": {
          "ohms": 10000.0,
          "wiper": 0.2
        }
      },
      "x": 370,
      "y": 440,
      "turns": 0
    }
  ],
  "nets": [
    {
      "pins": [
        {
          "element": 0,
          "pin": "D13"
        },
        {
          "element": 1,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 1,
          "pin": "b"
        },
        {
          "element": 2,
          "pin": "anode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "GND"
        },
        {
          "element": 2,
          "pin": "cathode"
        },
        {
          "element": 3,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "5V"
        },
        {
          "element": 3,
          "pin": "b"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "A0"
        },
        {
          "element": 3,
          "pin": "wiper"
        }
      ]
    }
  ]
}
`,We=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "description": "AnalogReadSerial\\nA0's reading printed to the serial monitor ten times a second. Turn the pot and the numbers follow.",
  "elements": [
    {
      "kind": "Uno",
      "x": 200,
      "y": 300,
      "turns": 0,
      "program": {
        "name": "AnalogReadSerial",
        "hex": ":100000000C9435000C945D000C945D000C945D0024\\r\\n:100010000C945D000C945D000C945D000C945D00EC\\r\\n:100020000C945D000C945D000C945D000C945D00DC\\r\\n:100030000C945D000C945D000C945D000C945D00CC\\r\\n:100040000C94AC010C945D000C941A020C94F40115\\r\\n:100050000C945D000C945D000C945D000C945D00AC\\r\\n:100060000C945D000C945D00700311241FBECFEF53\\r\\n:10007000D8E0DEBFCDBF11E0A0E0B1E0EEE8F7E0F0\\r\\n:1000800002C005900D92A631B107D9F721E0A6E193\\r\\n:10009000B1E001C01D92AC3BB207E1F710E0C5E34F\\r\\n:1000A000D0E004C02197FE010E94BF03C433D107F2\\r\\n:1000B000C9F70E9447020C94C5030C940000CF932B\\r\\n:1000C000DF9300D000D0CDB7DEB73FB7F894809172\\r\\n:1000D0001B0190911C01A0911D01B0911E0126B53C\\r\\n:1000E000A89B05C02F3F19F00196A11DB11D3FBF70\\r\\n:1000F000BA2FA92F982F8827820F911DA11DB11DFE\\r\\n:100100009C01AD0162E0220F331F441F551F6A9509\\r\\n:10011000D1F7CA01B9010F900F900F900F90DF91A6\\r\\n:10012000CF910895CF92DF92EF92FF920F931F939A\\r\\n:10013000CF93DF937C016A0110E000E00C151D05F0\\r\\n:1001400081F0EB01D701ED91FC910190F081E02D60\\r\\n:100150006991C7010995892B21F00F5F1F4FBE01DF\\r\\n:10016000EDCFC801DF91CF911F910F91FF90EF90DC\\r\\n:10017000DF90CF900895FC01538D448D252F30E002\\r\\n:10018000842F90E0821B930B541710F0CF960895A4\\r\\n:1001900001970895FC01918D828D981761F0A28DD1\\r\\n:1001A000AE0FBF2FB11D5D968C91928D9F5F9F7397\\r\\n:1001B000928F90E008958FEF9FEF0895FC01918D4D\\r\\n:1001C000828D981731F0828DE80FF11D858D90E0BA\\r\\n:1001D00008958FEF9FEF0895FC01818D228D90E0AF\\r\\n:1001E000805C9F4F821B91098F73992708958FE13F\\r\\n:1001F00091E00E94EC009C0181E0232B09F480E057\\r\\n:10020000089580E090E0892B29F00E94F700811189\\r\\n:100210000C9400000895FC01A48DA80FB92FB11D06\\r\\n:10022000A35ABF4F2C91848D90E001968F7399272C\\r\\n:10023000848FA689B7892C93A089B1898C9183700A\\r\\n:1002400080648C93938D848D981306C00288F38903\\r\\n:10025000E02D80818F7D80830895EF92FF920F9330\\r\\n:100260001F93CF93DF93EC01F62E81E0888F9B8D57\\r\\n:100270008C8D98131AC0E889F989808185FF15C093\\r\\n:100280009FB7F894EE89FF896083E889F9898081B6\\r\\n:100290008370806480839FBF81E090E0DF91CF9185\\r\\n:1002A0001F910F91FF90EF9008950B8D10E00F5F5D\\r\\n:1002B0001F4F0F731127E02E8C8D8E110CC00FB6BF\\r\\n:1002C00007FCFACFE889F989808185FFF5CFCE0157\\r\\n:1002D0000E940B01F1CFEB8DEC0FFD2FF11DE35AC6\\r\\n:1002E000FF4FF0829FB7F8940B8FEA89FB898081DA\\r\\n:1002F0008062D0CFCF93DF93EC01888D8823B9F053\\r\\n:10030000AA89BB89E889F9898C9185FD03C0808120\\r\\n:1003100086FD0DC00FB607FCF7CF8C9185FFF2CF9D\\r\\n:10032000808185FFEDCFCE010E940B01E9CFDF91E7\\r\\n:10033000CF910895BC01009761F0FC01019000206D\\r\\n:10034000E9F73197AF01481B590B8FE191E00C940D\\r\\n:10035000920090E080E008951F921FB61F92112432\\r\\n:100360002F933F938F939F93AF93BF9380911701E8\\r\\n:1003700090911801A0911901B0911A0130911601C4\\r\\n:1003800023E0230F2D3750F50196A11DB11D2093B9\\r\\n:1003900016018093170190931801A0931901B0934F\\r\\n:1003A0001A0180911B0190911C01A0911D01B09137\\r\\n:1003B0001E010196A11DB11D80931B0190931C018C\\r\\n:1003C000A0931D01B0931E01BF91AF919F918F919A\\r\\n:1003D0003F912F911F901FBE1F90189526E8230F65\\r\\n:1003E0000296A11DB11DD3CF1F920F920FB60F928F\\r\\n:1003F00011242F933F934F935F936F937F938F932A\\r\\n:100400009F93AF93BF93EF93FF938FE191E00E948F\\r\\n:100410000B01FF91EF91BF91AF919F918F917F91D0\\r\\n:100420006F915F914F913F912F910F900FBE0F9061\\r\\n:100430001F9018952F932FB72F938F939F93EF9320\\r\\n:10044000FF93E0912F01F09130018081E09135011F\\r\\n:10045000F091360182FD19C09081809138018F5F43\\r\\n:100460008F7320913901821741F0E0913801F0E05B\\r\\n:10047000E15EFE4F958F80933801FF91EF919F9140\\r\\n:100480008F912F912FBF2F9118958081F6CFCDB7E7\\r\\n:10049000DEB7A5970FB6F894DEBF0FBECDBF789438\\r\\n:1004A00084B5826084BD84B5816084BD85B5826079\\r\\n:1004B00085BD85B5816085BD80916E00816080932A\\r\\n:1004C0006E00109281008091810082608093810093\\r\\n:1004D00080918100816080938100809180008160A3\\r\\n:1004E000809380008091B10084608093B1008091FE\\r\\n:1004F000B00081608093B00080917A008460809326\\r\\n:100500007A0080917A00826080937A0080917A00EC\\r\\n:10051000816080937A0080917A00806880937A006D\\r\\n:100520001092C100E0912F01F091300182E08083B0\\r\\n:10053000E0912B01F0912C011082E0912D01F091BE\\r\\n:100540002E018FEC808310923701E0913301F091FE\\r\\n:10055000340186E08083E0913101F09132018081A5\\r\\n:1005600080618083E0913101F09132018081886067\\r\\n:100570008083E0913101F091320180818068808335\\r\\n:10058000E0913101F091320180818F7D808390E490\\r\\n:10059000392E2AE0822E912CA12CB12C00E010E003\\r\\n:1005A00030927C0080917A00806480937A00809100\\r\\n:1005B0007A0086FDFCCF80917800909179006C01E3\\r\\n:1005C000092E000CEE08FF0897FF6DC06DE28FE169\\r\\n:1005D00091E00E942D0188279927DC018C199D0943\\r\\n:1005E000AE09BF098AA39BA3ACA3BDA319A2DE01D8\\r\\n:1005F00091967D016AA17BA18CA19DA1A501940189\\r\\n:100600000E949D038AA19BA1ACA1BDA12AA33BA3EB\\r\\n:100610004CA35DA3605DF70162937F010A97A1057A\\r\\n:10062000B10540F7C7010E949A0142E050E062E143\\r\\n:1006300071E08FE191E00E9492000E945F006B01E7\\r\\n:100640007C0184E6482E512C612C712C0E945F00A5\\r\\n:100650006AA37BA38CA39DA3DC01CB018C199D090C\\r\\n:10066000AE09BF09883E9340A105B10578F3F1E0DA\\r\\n:100670004F1A51086108710828EEC20E23E0D21EFD\\r\\n:10068000E11CF11C4114510461047104F9F60115D7\\r\\n:10069000110509F485CF0E94F700882309F480CF63\\r\\n:1006A0000E9400007DCF26016E2CDF2C19A2CE0106\\r\\n:1006B00081967C01B201862D9D2DA50194010E9499\\r\\n:1006C0009D03C201A62DBD2D2901642ED52E605D8E\\r\\n:1006D000F70162937F010A97A105B10558F7A2CFF0\\r\\n:1006E000EFE1F1E01382128288EE93E0A0E0B0E047\\r\\n:1006F00084839583A683B78384E091E0918380838C\\r\\n:1007000085EC90E09587848784EC90E097878687D6\\r\\n:1007100080EC90E0918B808B81EC90E0938B828BCE\\r\\n:1007200082EC90E0958B848B86EC90E0978B868BA7\\r\\n:10073000118E128E138E148E0895A1E21A2EAA1B0A\\r\\n:10074000BB1BFD010DC0AA1FBB1FEE1FFF1FA21781\\r\\n:10075000B307E407F50720F0A21BB30BE40BF50B7E\\r\\n:10076000661F771F881F991F1A9469F76095709507\\r\\n:10077000809590959B01AC01BD01CF010895EE0FCE\\r\\n:0E078000FF1F0590F491E02D0994F894FFCF2F\\r\\n:10078E00000000002D019200BB007A01EC00CA00AF\\r\\n:06079E00DE000D0A000060\\r\\n:00000001FF\\r\\n"
      }
    },
    {
      "kind": {
        "Potentiometer": {
          "ohms": 10000.0,
          "wiper": 0.5
        }
      },
      "x": 370,
      "y": 440,
      "turns": 0
    }
  ],
  "nets": [
    {
      "pins": [
        {
          "element": 0,
          "pin": "GND"
        },
        {
          "element": 1,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "5V"
        },
        {
          "element": 1,
          "pin": "b"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "A0"
        },
        {
          "element": 1,
          "pin": "wiper"
        }
      ]
    }
  ]
}
`,Ge=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "description": "Blink\\nAn Arduino Uno running Blink: D13 high for a second, low for a second.",
  "elements": [
    {
      "kind": "Uno",
      "x": 200,
      "y": 300,
      "turns": 0,
      "program": {
        "name": "Blink",
        "hex": ":100000000C945C000C946E000C946E000C946E00CA\\r\\n:100010000C946E000C946E000C946E000C946E00A8\\r\\n:100020000C946E000C946E000C946E000C946E0098\\r\\n:100030000C946E000C946E000C946E000C946E0088\\r\\n:100040000C942F010C946E000C946E000C946E00B6\\r\\n:100050000C946E000C946E000C946E000C946E0068\\r\\n:100060000C946E000C946E00000000002400270029\\r\\n:100070002A0000000000250028002B0004040404CE\\r\\n:100080000404040402020202020203030303030342\\r\\n:10009000010204081020408001020408102001021F\\r\\n:1000A00004081020000000080002010000030407FB\\r\\n:1000B000000000000000000011241FBECFEFD8E0B8\\r\\n:1000C000DEBFCDBF21E0A0E0B1E001C01D92A930AC\\r\\n:1000D000B207E1F70E9477010C94E6010C9400004E\\r\\n:1000E000E1EBF0E02491EDE9F0E09491E9E8F0E053\\r\\n:1000F000E491EE23D9F0222349F0243031F1B8F411\\r\\n:10010000223059F12330D9F0213019F1F0E0EE0F0F\\r\\n:10011000FF1FEE58FF4FA591B4912FB7F894EC91C3\\r\\n:10012000811124C090959E239C932FBF0895273062\\r\\n:10013000B9F0283059F72091B0002F7D14C02091DC\\r\\n:1001400080002F7720938000E1CF209180002F7DC9\\r\\n:10015000F9CF24B52F7724BDD9CF24B52F7DFBCF80\\r\\n:100160002091B0002F772093B000D0CF9E2BDCCF12\\r\\n:10017000CF93DF9300D000D0CDB7DEB73FB7F89470\\r\\n:100180008091050190910601A0910701B0910801AD\\r\\n:1001900026B5A89B05C02F3F19F00196A11DB11DE2\\r\\n:1001A0003FBFBA2FA92F982F8827820F911DA11D1D\\r\\n:1001B000B11D9C01AD0162E0220F331F441F551F8A\\r\\n:1001C0006A95D1F7CA01B9010F900F900F900F9067\\r\\n:1001D000DF91CF9108958F929F92AF92BF92CF926D\\r\\n:1001E000DF92EF92FF92CF93DF9300D000D0CDB794\\r\\n:1001F000DEB70E94B8006B017C0188EE882E83E098\\r\\n:10020000982EA12CB12C0E94B800DC01CB018C19D6\\r\\n:100210009D09AE09BF09883E9340A105B10598F339\\r\\n:1002200021E0821A9108A108B10888EEC80E83E087\\r\\n:10023000D81EE11CF11C81149104A104B10419F72A\\r\\n:100240000F900F900F900F90DF91CF91FF90EF9054\\r\\n:10025000DF90CF90BF90AF909F908F9008951F92A6\\r\\n:100260001FB61F9211242F933F938F939F93AF93A9\\r\\n:10027000BF938091010190910201A0910301B0917F\\r\\n:1002800004013091000123E0230F2D3750F5019632\\r\\n:10029000A11DB11D209300018093010190930201E3\\r\\n:1002A000A0930301B0930401809105019091060190\\r\\n:1002B000A0910701B09108010196A11DB11D809385\\r\\n:1002C000050190930601A0930701B0930801BF9127\\r\\n:1002D000AF919F918F913F912F911F901FBE1F90C3\\r\\n:1002E000189526E8230F0296A11DB11DD3CF78944F\\r\\n:1002F00084B5826084BD84B5816084BD85B582602B\\r\\n:1003000085BD85B5816085BD80916E0081608093DB\\r\\n:100310006E00109281008091810082608093810044\\r\\n:100320008091810081608093810080918000816054\\r\\n:10033000809380008091B10084608093B1008091AF\\r\\n:10034000B00081608093B00080917A0084608093D7\\r\\n:100350007A0080917A00826080937A0080917A009E\\r\\n:10036000816080937A0080917A00806880937A001F\\r\\n:100370001092C100EDE9F0E02491E9E8F0E0849109\\r\\n:10038000882399F090E0880F991FFC01E859FF4FEE\\r\\n:10039000A591B491FC01EE58FF4F859194918FB7D0\\r\\n:1003A000F894EC91E22BEC938FBFC0E0D0E081E0B9\\r\\n:1003B0000E9470000E94EB0080E00E9470000E948A\\r\\n:1003C000EB002097A1F30E940000F1CFF894FFCF3B\\r\\n:00000001FF\\r\\n"
      }
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 220.0
        }
      },
      "x": 370,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "Led": {
          "colour": "Red"
        }
      },
      "x": 460,
      "y": 140,
      "turns": 0
    }
  ],
  "nets": [
    {
      "pins": [
        {
          "element": 0,
          "pin": "D13"
        },
        {
          "element": 1,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 1,
          "pin": "b"
        },
        {
          "element": 2,
          "pin": "anode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "GND"
        },
        {
          "element": 2,
          "pin": "cathode"
        }
      ]
    }
  ]
}
`,Ke=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "description": "Button\\nD2 reads the button through its own pull-up; the program lights D13 while it is held.",
  "elements": [
    {
      "kind": "Uno",
      "x": 200,
      "y": 300,
      "turns": 0,
      "program": {
        "name": "Button",
        "hex": ":100000000C9461000C9473000C9473000C947300B6\\r\\n:100010000C9473000C9473000C9473000C94730094\\r\\n:100020000C9473000C9473000C9473000C94730084\\r\\n:100030000C9473000C9473000C9473000C94730074\\r\\n:100040000C949F000C9473000C9473000C94730038\\r\\n:100050000C9473000C9473000C9473000C94730054\\r\\n:100060000C9473000C94730000000000240027001F\\r\\n:100070002A000000000023002600290000000000E4\\r\\n:10008000250028002B0004040404040404040202D4\\r\\n:100090000202020203030303030301020408102007\\r\\n:1000A0004080010204081020010204081020000012\\r\\n:1000B0000008000201000003040700000000000027\\r\\n:1000C000000011241FBECFEFD8E0DEBFCDBF21E07E\\r\\n:1000D000A0E0B1E001C01D92A930B207E1F70E9493\\r\\n:1000E000E7000C94BB010C940000843099F038F4C4\\r\\n:1000F0008230C1F0833041F0813081F00895873043\\r\\n:10010000A1F08830C1F00895809180008F778093AE\\r\\n:1001100080000895809180008F7DF9CF84B58F771E\\r\\n:1001200084BD089584B58F7DFBCF8091B0008F771B\\r\\n:100130008093B00008958091B0008F7DF9CF1F9219\\r\\n:100140001FB61F9211242F933F938F939F93AF93CA\\r\\n:10015000BF938091050190910601A0910701B09194\\r\\n:1001600008013091040123E0230F2D3750F501964B\\r\\n:10017000A11DB11D209304018093050190930601F8\\r\\n:10018000A0930701B09308018091000190910101B3\\r\\n:10019000A0910201B09103010196A11DB11D8093B0\\r\\n:1001A000000190930101A0930201B0930301BF915C\\r\\n:1001B000AF919F918F913F912F911F901FBE1F90E4\\r\\n:1001C000189526E8230F0296A11DB11DD3CF789470\\r\\n:1001D00084B5826084BD84B5816084BD85B582604C\\r\\n:1001E00085BD85B5816085BD80916E0081608093FD\\r\\n:1001F0006E00109281008091810082608093810066\\r\\n:100200008091810081608093810080918000816075\\r\\n:10021000809380008091B10084608093B1008091D0\\r\\n:10022000B00081608093B00080917A0084608093F8\\r\\n:100230007A0080917A00826080937A0080917A00BF\\r\\n:10024000816080937A0080917A00806880937A0040\\r\\n:100250001092C1007CE9E72E70E0F72EF70124919F\\r\\n:10026000E8E8CE2EE0E0DE2EF60184918823C1F08E\\r\\n:1002700090E0880F991FFC01E859FF4FA591B491B8\\r\\n:10028000FC01E458FF4FC591D4919FB7F8943C917D\\r\\n:10029000822F809583238C938881282B28839FBF6E\\r\\n:1002A000C7EAD0E0FE01249103E910E0F80184914F\\r\\n:1002B000882399F090E0880F991FFC01E859FF4FBF\\r\\n:1002C000A591B491FC01E458FF4F859194918FB7AB\\r\\n:1002D000F8949C91292B2C938FBF20EB622E20E069\\r\\n:1002E000722E3BEBA32E30E0B32E40E0842E40E094\\r\\n:1002F000942EF3018491F7015490F6014490442028\\r\\n:1003000009F434C081110E947500E42DF0E0EE0F75\\r\\n:10031000FF1FEE58FF4FA591B4918C9158224424B1\\r\\n:1003200043945110412CF5018491FE015490F80141\\r\\n:1003300034903320A1F081110E947500E32DF0E08C\\r\\n:10034000EE0FFF1FE458FF4FA591B4918FB7F894BB\\r\\n:100350009C9141100EC0509459225C928FBF811421\\r\\n:10036000910409F4C6CF0E940000C3CF44244394F3\\r\\n:0A037000DACF592AF2CFF894FFCF3C\\r\\n:00000001FF\\r\\n"
      }
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 220.0
        }
      },
      "x": 370,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "Led": {
          "colour": "Red"
        }
      },
      "x": 460,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "PushButton": {
          "closed": false
        }
      },
      "x": 370,
      "y": 440,
      "turns": 0
    }
  ],
  "nets": [
    {
      "pins": [
        {
          "element": 0,
          "pin": "D13"
        },
        {
          "element": 1,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 1,
          "pin": "b"
        },
        {
          "element": 2,
          "pin": "anode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "GND"
        },
        {
          "element": 2,
          "pin": "cathode"
        },
        {
          "element": 3,
          "pin": "b"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "D2"
        },
        {
          "element": 3,
          "pin": "a"
        }
      ]
    }
  ]
}
`,qe=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "description": "Dimmer\\nA pot on A0 sets an LED on D9: analogRead's 0 to 1023, divided by four, is analogWrite's 0 to 255.",
  "elements": [
    {
      "kind": "Uno",
      "x": 200,
      "y": 300,
      "turns": 0,
      "program": {
        "name": "Dimmer",
        "hex": ":100000000C945C000C946E000C946E000C946E00CA\\r\\n:100010000C946E000C946E000C946E000C946E00A8\\r\\n:100020000C946E000C946E000C946E000C946E0098\\r\\n:100030000C946E000C946E000C946E000C946E0088\\r\\n:100040000C94D4000C946E000C946E000C946E0012\\r\\n:100050000C946E000C946E000C946E000C946E0068\\r\\n:100060000C946E000C946E00000000080002010069\\r\\n:100070000003040700000000000000000000000072\\r\\n:10008000250028002B0000000000240027002A0083\\r\\n:10009000040404040404040402020202020203032E\\r\\n:1000A0000303030301020408102040800102040836\\r\\n:1000B000102001020408102011241FBECFEFD8E049\\r\\n:1000C000DEBFCDBF21E0A0E0B1E001C01D92A930AC\\r\\n:1000D000B207E1F70E941C010C94D4010C940000BB\\r\\n:1000E000EDEAF0E02491E9E9F0E08491882399F0C9\\r\\n:1000F00090E0880F991FFC01EA57FF4FA591B4913A\\r\\n:10010000FC01E458FF4F859194918FB7F894EC91DE\\r\\n:10011000E22BEC938FBF0895E1E7F0E02491EDEA44\\r\\n:10012000F0E09491E9E9F0E0E491EE23D9F02223A4\\r\\n:1001300049F0243031F1B8F4223059F12330D9F0AC\\r\\n:10014000213019F1F0E0EE0FFF1FE458FF4FA591A9\\r\\n:10015000B4912FB7F894EC91811124C090959E230F\\r\\n:100160009C932FBF08952730B9F0283059F720917C\\r\\n:10017000B0002F7D14C0209180002F772093800045\\r\\n:10018000E1CF209180002F7DF9CF24B52F7724BDBA\\r\\n:10019000D9CF24B52F7DFBCF2091B0002F772093AE\\r\\n:1001A000B000D0CF9E2BDCCF1F921FB61F92112420\\r\\n:1001B0002F933F938F939F93AF93BF9380910501AC\\r\\n:1001C00090910601A0910701B091080130910401BE\\r\\n:1001D00023E0230F2D3750F50196A11DB11D20936B\\r\\n:1001E00004018093050190930601A0930701B09349\\r\\n:1001F00008018091000190910101A0910201B0914C\\r\\n:1002000003010196A11DB11D80930001909301018E\\r\\n:10021000A0930201B0930301BF91AF919F918F9181\\r\\n:100220003F912F911F901FBE1F90189526E8230F16\\r\\n:100230000296A11DB11DD3CF789484B5826084BD90\\r\\n:1002400084B5816084BD85B5826085BD85B58160DA\\r\\n:1002500085BD80916E00816080936E001092810058\\r\\n:100260008091810082608093810080918100816013\\r\\n:1002700080938100809180008160809380008091D4\\r\\n:10028000B10084608093B1008091B0008160809360\\r\\n:10029000B00080917A00846080937A0080917A0027\\r\\n:1002A000826080937A0080917A00816080937A00E6\\r\\n:1002B00080917A00806880937A001092C1000E9439\\r\\n:1002C000700090E4B92E21E7E22E20E0F22E30E01B\\r\\n:1002D000C32E30E0D32EB0927C0080917A008064EF\\r\\n:1002E00080937A0080917A0086FDFCCF009178009F\\r\\n:1002F00010917900E80117FD2396D595C795D595FE\\r\\n:10030000C7950E947000209749F480E00E948C00FD\\r\\n:10031000C114D10401F30E940000DDCFCF3FD1050D\\r\\n:1003200011F481E0F3CFF7018491843059F150F456\\r\\n:100330008230C9F08330E1F0813081F00115124044\\r\\n:1003400024F3EFCF873041F18830C1F78091B000BE\\r\\n:1003500080628093B000C093B400DACF84B5806827\\r\\n:1003600084BDC7BDD5CF84B5806284BDC8BDD0CFA4\\r\\n:1003700080918000806880938000D0938900C09332\\r\\n:100380008800C6CF80918000806280938000D093E7\\r\\n:100390008B00C0938A00BCCF8091B00080688093AE\\r\\n:0C03A000B000C093B300B4CFF894FFCFBE\\r\\n:00000001FF\\r\\n"
      }
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 220.0
        }
      },
      "x": 370,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "Led": {
          "colour": "Green"
        }
      },
      "x": 460,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "Potentiometer": {
          "ohms": 10000.0,
          "wiper": 0.25
        }
      },
      "x": 370,
      "y": 440,
      "turns": 0
    }
  ],
  "nets": [
    {
      "pins": [
        {
          "element": 0,
          "pin": "D9"
        },
        {
          "element": 1,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 1,
          "pin": "b"
        },
        {
          "element": 2,
          "pin": "anode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "GND"
        },
        {
          "element": 2,
          "pin": "cathode"
        },
        {
          "element": 3,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "5V"
        },
        {
          "element": 3,
          "pin": "b"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "A0"
        },
        {
          "element": 3,
          "pin": "wiper"
        }
      ]
    }
  ]
}
`,Je=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "description": "Fade\\nanalogWrite on D9 and D10: two LEDs fading into each other.",
  "elements": [
    {
      "kind": "Uno",
      "x": 200,
      "y": 300,
      "turns": 0,
      "program": {
        "name": "Fade",
        "hex": ":100000000C945C000C946E000C946E000C946E00CA\\r\\n:100010000C946E000C946E000C946E000C946E00A8\\r\\n:100020000C946E000C946E000C946E000C946E0098\\r\\n:100030000C946E000C946E000C946E000C946E0088\\r\\n:100040000C94AD010C946E000C946E000C946E0038\\r\\n:100050000C946E000C946E000C946E000C946E0068\\r\\n:100060000C946E000C946E00000000080002010069\\r\\n:100070000003040700000000000000000000000072\\r\\n:10008000250028002B0000000000240027002A0083\\r\\n:10009000040404040404040402020202020203032E\\r\\n:1000A0000303030301020408102040800102040836\\r\\n:1000B000102001020408102011241FBECFEFD8E049\\r\\n:1000C000DEBFCDBF21E0A0E0B1E001C01D92A930AC\\r\\n:1000D000B207E1F70E94F5010C9470020C94000045\\r\\n:1000E00090E0FC01EC55FF4F249180579F4FFC019D\\r\\n:1000F0008491882399F090E0880F991FFC01EA57BA\\r\\n:10010000FF4FA591B491FC01E458FF4F8591949164\\r\\n:100110008FB7F894EC91E22BEC938FBF089590E0A9\\r\\n:10012000FC01E859FF4F2491FC01EC55FF4F34913D\\r\\n:10013000FC01E057FF4FE491EE23D9F0222349F070\\r\\n:10014000243031F1B8F4223059F12330D9F0213084\\r\\n:1001500019F1F0E0EE0FFF1FE458FF4FA591B491A5\\r\\n:100160008FB7F894EC91611124C030953E233C93F5\\r\\n:100170008FBF08952730B9F0283059F78091B0002B\\r\\n:100180008F7D14C0809180008F7780938000E1CFB5\\r\\n:10019000809180008F7DF9CF84B58F7784BDD9CFD2\\r\\n:1001A00084B58F7DFBCF8091B0008F778093B000B6\\r\\n:1001B000D0CF3E2BDCCF1F93CF93DF93182FEB01D3\\r\\n:1001C0000E947000209739F460E0812FDF91CF9179\\r\\n:1001D0001F910C948F00CF3FD10511F461E0F5CF52\\r\\n:1001E000E12FF0E0E859FF4FE491E43071F150F471\\r\\n:1001F000E230E1F0E330F9F0E13081F0C038D105D0\\r\\n:100200001CF3ECCFE73059F1E830C1F78091B00032\\r\\n:1002100080628093B000C093B40004C084B580684D\\r\\n:1002200084BDC7BDDF91CF911F91089584B58062D1\\r\\n:1002300084BDC8BDF7CF8091800080688093800026\\r\\n:10024000D0938900C0938800EDCF809180008062B8\\r\\n:1002500080938000D0938B00C0938A00E3CF80917D\\r\\n:10026000B00080688093B000C093B300DBCFCF9321\\r\\n:10027000DF9300D000D0CDB7DEB73FB7F8948091C0\\r\\n:10028000050190910601A0910701B091080126B5E2\\r\\n:10029000A89B05C02F3F19F00196A11DB11D3FBFBE\\r\\n:1002A000BA2FA92F982F8827820F911DA11DB11D4C\\r\\n:1002B0009C01AD0162E0220F331F441F551F6A9558\\r\\n:1002C000D1F7CA01B9010F900F900F900F90DF91F5\\r\\n:1002D000CF9108958F929F92AF92BF92CF92DF926B\\r\\n:1002E000EF92FF92CF93DF9300D000D0CDB7DEB76F\\r\\n:1002F0000E9437016B017C0184E1882E912CA12C96\\r\\n:10030000B12C0E943701DC01CB018C199D09AE098B\\r\\n:10031000BF09883E9340A105B10598F321E0821AF8\\r\\n:100320009108A108B10888EEC80E83E0D81EE11C30\\r\\n:10033000F11C81149104A104B10419F70F900F90DE\\r\\n:100340000F900F90DF91CF91FF90EF90DF90CF90C3\\r\\n:10035000BF90AF909F908F9008951F921FB61F92ED\\r\\n:1003600011242F933F938F939F93AF93BF938091CB\\r\\n:10037000010190910201A0910301B091040130911B\\r\\n:10038000000123E0230F2D3750F50196A11DB11D6B\\r\\n:10039000209300018093010190930201A093030137\\r\\n:1003A000B09304018091050190910601A09107018D\\r\\n:1003B000B09108010196A11DB11D80930501909394\\r\\n:1003C0000601A0930701B0930801BF91AF919F91DF\\r\\n:1003D0008F913F912F911F901FBE1F90189526E877\\r\\n:1003E000230F0296A11DB11DD3CF789484B58260EE\\r\\n:1003F00084BD84B5816084BD85B5826085BD85B5C9\\r\\n:10040000816085BD80916E00816080936E00109246\\r\\n:1004100081008091810082608093810080918100C1\\r\\n:100420008160809381008091800081608093800052\\r\\n:100430008091B10084608093B1008091B0008160B0\\r\\n:100440008093B00080917A00846080937A008091DC\\r\\n:100450007A00826080937A0080917A008160809334\\r\\n:100460007A0080917A00806880937A001092C100AF\\r\\n:1004700089E00E9470008AE00E9470000FEF10E097\\r\\n:1004800080E0E82E80E0F82ED0E0C0E0BE0189E0F8\\r\\n:100490000E94DB00B8016C1B7D0B8AE00E94DB0030\\r\\n:1004A0000E946A012596C43081E0D80779F7CFEF22\\r\\n:1004B000D0E0BE0189E00E94DB00B8016C1B7D0B1F\\r\\n:1004C0008AE00E94DB000E946A012597CB3F8FEFF4\\r\\n:1004D000D80779F7E114F104B9F20E940000D4CFF3\\r\\n:0404E000F894FFCFBE\\r\\n:00000001FF\\r\\n"
      }
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 220.0
        }
      },
      "x": 370,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "Led": {
          "colour": "Red"
        }
      },
      "x": 460,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 220.0
        }
      },
      "x": 370,
      "y": 200,
      "turns": 0
    },
    {
      "kind": {
        "Led": {
          "colour": "Green"
        }
      },
      "x": 460,
      "y": 200,
      "turns": 0
    }
  ],
  "nets": [
    {
      "pins": [
        {
          "element": 0,
          "pin": "D9"
        },
        {
          "element": 1,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 1,
          "pin": "b"
        },
        {
          "element": 2,
          "pin": "anode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "D10"
        },
        {
          "element": 3,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 3,
          "pin": "b"
        },
        {
          "element": 4,
          "pin": "anode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "GND"
        },
        {
          "element": 2,
          "pin": "cathode"
        },
        {
          "element": 4,
          "pin": "cathode"
        }
      ]
    }
  ]
}
`,Ye=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "description": "Night light\\nAn LDR under 10 kΩ on A0: the darker, the higher it reads. D13 lights past 600 and goes out below 500.",
  "elements": [
    {
      "kind": "Uno",
      "x": 200,
      "y": 300,
      "turns": 0,
      "program": {
        "name": "NightLight",
        "hex": ":100000000C945C000C946E000C946E000C946E00CA\\r\\n:100010000C946E000C946E000C946E000C946E00A8\\r\\n:100020000C946E000C946E000C946E000C946E0098\\r\\n:100030000C946E000C946E000C946E000C946E0088\\r\\n:100040000C94A3000C946E000C946E000C946E0043\\r\\n:100050000C946E000C946E000C946E000C946E0068\\r\\n:100060000C946E000C946E00000000002400270029\\r\\n:100070002A0000000000250028002B0004040404CE\\r\\n:100080000404040402020202020203030303030342\\r\\n:10009000010204081020408001020408102001021F\\r\\n:1000A00004081020000000080002010000030407FB\\r\\n:1000B000000000000000000011241FBECFEFD8E0B8\\r\\n:1000C000DEBFCDBF21E0A0E0B1E001C01D92AA30AB\\r\\n:1000D000B207E1F70E94EB000C94F7010C940000CA\\r\\n:1000E000CF93DF9300D000D0CDB7DEB73FB7F89401\\r\\n:1000F0008091060190910701A0910801B09109013A\\r\\n:1001000026B5A89B05C02F3F19F00196A11DB11D72\\r\\n:100110003FBFBA2FA92F982F8827820F911DA11DAD\\r\\n:10012000B11D9C01AD0162E0220F331F441F551F1A\\r\\n:100130006A95D1F7CA01B9010F900F900F900F90F7\\r\\n:10014000DF91CF9108951F921FB61F9211242F9314\\r\\n:100150003F938F939F93AF93BF93809102019091B0\\r\\n:100160000301A0910401B09105013091010123E048\\r\\n:10017000230F2D3750F50196A11DB11D20930101CC\\r\\n:100180008093020190930301A0930401B0930501B1\\r\\n:100190008091060190910701A0910801B091090199\\r\\n:1001A0000196A11DB11D8093060190930701A093B4\\r\\n:1001B0000801B0930901BF91AF919F918F913F9139\\r\\n:1001C0002F911F901FBE1F90189526E8230F0296AF\\r\\n:1001D000A11DB11DD3CF00D000D0CDB7DEB778942C\\r\\n:1001E00084B5826084BD84B5816084BD85B582603C\\r\\n:1001F00085BD85B5816085BD80916E0081608093ED\\r\\n:100200006E00109281008091810082608093810055\\r\\n:100210008091810081608093810080918000816065\\r\\n:10022000809380008091B10084608093B1008091C0\\r\\n:10023000B00081608093B00080917A0084608093E8\\r\\n:100240007A0080917A00826080937A0080917A00AF\\r\\n:10025000816080937A0080917A00806880937A0030\\r\\n:100260001092C100EDE9F0E0249109E810E0F801F6\\r\\n:100270008491882399F090E0880F991FFC01E85938\\r\\n:10028000FF4FA591B491FC01EE58FF4F85919491D9\\r\\n:100290008FB7F8949C91292B2C938FBF20E4322E9A\\r\\n:1002A00031EB832E30E0932E40E0A42E40E0B42EBC\\r\\n:1002B0002224239430927C0080917A008064809381\\r\\n:1002C0007A0080917A0086FDFCCF80917800909131\\r\\n:1002D00079008935F2E09F070CF45FC0209200019D\\r\\n:1002E000F4019491EDE9F0E08491F8013491332325\\r\\n:1002F00019F120910001992371F0943009F463C041\\r\\n:1003000008F052C0923009F466C0933009F455C029\\r\\n:10031000913009F45CC0E32FF0E0EE0FFF1FEE58C0\\r\\n:10032000FF4FA591B4919FB7F89421115DC02C9116\\r\\n:10033000809582238C939FBF0E9470006B017C018B\\r\\n:1003400082E3482E512C612C712C0E94700069832D\\r\\n:100350007A838B839C83DC01CB018C199D09AE09C8\\r\\n:10036000BF09883E9340A105B10578F3F1E04F1A2B\\r\\n:1003700051086108710828EEC20E23E0D21EE11C6C\\r\\n:10038000F11C4114510461047104F9F6A114B10483\\r\\n:1003900009F490CF0E9400008DCF843F91400CF073\\r\\n:1003A0009FCF109200019CCF9730C1F0983009F098\\r\\n:1003B000B2CF9091B0009F7D14C0909180009F7744\\r\\n:1003C00090938000A8CF909180009F7DF9CF94B545\\r\\n:1003D0009F7794BDA0CF94B59F7DFBCF9091B00047\\r\\n:1003E0009F779093B00097CF2C91822BA3CFF89456\\r\\n:0203F000FFCF3D\\r\\n:00000001FF\\r\\n"
      }
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 220.0
        }
      },
      "x": 370,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "Led": {
          "colour": "White"
        }
      },
      "x": 460,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 10000.0
        }
      },
      "x": 370,
      "y": 420,
      "turns": 1
    },
    {
      "kind": {
        "Ldr": {
          "light": 0.6
        }
      },
      "x": 370,
      "y": 520,
      "turns": 1
    }
  ],
  "nets": [
    {
      "pins": [
        {
          "element": 0,
          "pin": "D13"
        },
        {
          "element": 1,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 1,
          "pin": "b"
        },
        {
          "element": 2,
          "pin": "anode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "GND"
        },
        {
          "element": 2,
          "pin": "cathode"
        },
        {
          "element": 4,
          "pin": "b"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "5V"
        },
        {
          "element": 3,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "A0"
        },
        {
          "element": 3,
          "pin": "b"
        },
        {
          "element": 4,
          "pin": "a"
        }
      ]
    }
  ]
}
`,Xe=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "description": "D13 by serial\\nType on or off in the serial monitor and press Send: the program switches D13 and says so.",
  "elements": [
    {
      "kind": "Uno",
      "x": 200,
      "y": 300,
      "turns": 0,
      "program": {
        "name": "SerialLed",
        "hex": ":100000000C945E000C9486000C9486000C94860080\\r\\n:100010000C9486000C9486000C9486000C94860048\\r\\n:100020000C9486000C9486000C9486000C94860038\\r\\n:100030000C9486000C9486000C9486000C94860028\\r\\n:100040000C9420020C9486000C948E020C9468028E\\r\\n:100050000C9486000C9486000C9486000C94860008\\r\\n:100060000C9486000C9486000000000024002700F9\\r\\n:100070002A0000000000250028002B0004040404CE\\r\\n:100080000404040402020202020203030303030342\\r\\n:10009000010204081020408001020408102001021F\\r\\n:1000A00004081020000000080002010000030407FB\\r\\n:1000B0000000000000000000BB02AA0411241FBEC3\\r\\n:1000C000CFEFD8E0DEBFCDBF11E0A0E0B1E0E4E2C9\\r\\n:1000D000FEE002C005900D92AC35B107D9F722E0E1\\r\\n:1000E000ACE5B1E001C01D92AC30B207E1F710E021\\r\\n:1000F000CDE5D0E004C02197FE010E94B304CC35C9\\r\\n:10010000D107C9F70E942D030C9405070C94000039\\r\\n:10011000E1EBF0E02491EDE9F0E09491E9E8F0E022\\r\\n:10012000E491EE23D9F0222349F0243031F1B8F4E0\\r\\n:10013000223059F12330D9F0213019F1F0E0EE0FDF\\r\\n:10014000FF1FEE58FF4FA591B4912FB7F894EC9193\\r\\n:10015000811124C090959E239C932FBF0895273032\\r\\n:10016000B9F0283059F72091B0002F7D14C02091AC\\r\\n:1001700080002F7720938000E1CF209180002F7D99\\r\\n:10018000F9CF24B52F7724BDD9CF24B52F7DFBCF50\\r\\n:100190002091B0002F772093B000D0CF9E2BDCCFE2\\r\\n:1001A000CF92DF92EF92FF920F931F93CF93DF9343\\r\\n:1001B0007C016A0110E000E00C151D0581F0EB01E7\\r\\n:1001C000D701ED91FC910190F081E02D6991C7017B\\r\\n:1001D0000995892B21F00F5F1F4FBE01EDCFC8019C\\r\\n:1001E000DF91CF911F910F91FF90EF90DF90CF9013\\r\\n:1001F0000895FC01538D448D252F30E0842F90E02D\\r\\n:10020000821B930B541710F0CF9608950197089511\\r\\n:10021000FC01918D828D981761F0A28DAE0FBF2FDA\\r\\n:10022000B11D5D968C91928D9F5F9F73928F90E030\\r\\n:1002300008958FEF9FEF0895FC01918D828D98179F\\r\\n:1002400031F0828DE80FF11D858D90E008958FEFDC\\r\\n:100250009FEF0895FC01818D228D90E0805C9F4F7F\\r\\n:10026000821B91098F739927089585E691E00E947A\\r\\n:100270002A019C0181E0232B09F480E0089580E0AD\\r\\n:1002800090E0892B29F00E94350181110C94000027\\r\\n:100290000895FC01A48DA80FB92FB11DA35ABF4F1B\\r\\n:1002A0002C91848D90E001968F739927848FA68975\\r\\n:1002B000B7892C93A089B1898C91837080648C93C9\\r\\n:1002C000938D848D981306C00288F389E02D808178\\r\\n:1002D0008F7D80830895EF92FF920F931F93CF93AA\\r\\n:1002E000DF93EC01F62E81E0888F9B8D8C8D981327\\r\\n:1002F0001AC0E889F989808185FF15C09FB7F894F5\\r\\n:10030000EE89FF896083E889F98980818370806440\\r\\n:1003100080839FBF81E090E0DF91CF911F910F918B\\r\\n:10032000FF90EF9008950B8D10E00F5F1F4F0F733C\\r\\n:100330001127E02E8C8D8E110CC00FB607FCFACF62\\r\\n:10034000E889F989808185FFF5CFCE010E944901B6\\r\\n:10035000F1CFEB8DEC0FFD2FF11DE35AFF4FF08233\\r\\n:100360009FB7F8940B8FEA89FB8980818062D0CF98\\r\\n:10037000CF93DF93EC01888D8823B9F0AA89BB89DC\\r\\n:10038000E889F9898C9185FD03C0808186FD0DC0C7\\r\\n:100390000FB607FCF7CF8C9185FFF2CF808185FFE8\\r\\n:1003A000EDCFCE010E944901E9CFDF91CF910895B1\\r\\n:1003B000BC01FC0101900020E9F73197AF01481B17\\r\\n:1003C000590B85E691E00C94D000CF93DF930E9407\\r\\n:1003D000D801EC0188E191E00E94D8018C0F9D1FAB\\r\\n:1003E000DF91CF910895CF93DF93EC0180910202CA\\r\\n:1003F00090910302009759F0209104023091050278\\r\\n:100400002C173D0720F081E0DF91CF910895BE01C8\\r\\n:100410006F5F7F4F0E94ED05FC0180E03097A1F3F4\\r\\n:10042000F0930302E0930202D0930502C09304020A\\r\\n:100430008091060290910702892B29F71082E3CF61\\r\\n:100440001F921FB61F9211242F933F938F939F9358\\r\\n:10045000AF93BF938091610190916201A09163017C\\r\\n:10046000B09164013091600123E0230F2D3750F5E6\\r\\n:100470000196A11DB11D20936001809361019093AD\\r\\n:100480006201A0936301B093640180915C0190913B\\r\\n:100490005D01A0915E01B0915F010196A11DB11DAA\\r\\n:1004A00080935C0190935D01A0935E01B0935F0126\\r\\n:1004B000BF91AF919F918F913F912F911F901FBE40\\r\\n:1004C0001F90189526E8230F0296A11DB11DD3CFCA\\r\\n:1004D0001F920F920FB60F9211242F933F934F93B9\\r\\n:1004E0005F936F937F938F939F93AF93BF93EF939C\\r\\n:1004F000FF9385E691E00E944901FF91EF91BF9142\\r\\n:10050000AF919F918F917F916F915F914F913F91AB\\r\\n:100510002F910F900FBE0F901F9018952F932FB70C\\r\\n:100520002F938F939F93EF93FF93E0917501F09139\\r\\n:1005300076018081E0917B01F0917C0182FD19C000\\r\\n:10054000908180917E018F5F8F7320917F01821750\\r\\n:1005500041F0E0917E01F0E0EB59FE4F958F8093E2\\r\\n:100560007E01FF91EF919F918F912F912FBF2F913E\\r\\n:1005700018958081F6CF109268011092670188EE7D\\r\\n:1005800093E0A0E0B0E08093690190936A01A093AA\\r\\n:100590006B01B0936C018AE091E0909366018093C7\\r\\n:1005A000650185EC90E0909372018093710184EC79\\r\\n:1005B00090E0909374018093730180EC90E09093AD\\r\\n:1005C00076018093750181EC90E09093780180939F\\r\\n:1005D000770182EC90E090937A018093790186EC28\\r\\n:1005E00090E090937C0180937B0110927E011092A9\\r\\n:1005F0007F011092800110928101109203021092EB\\r\\n:100600000202109205021092040210920702109248\\r\\n:10061000060290E080E00E94F301282F8091020200\\r\\n:1006200090910302211111C0009711F00E945B0507\\r\\n:100630001092030210920202109207021092060218\\r\\n:10064000109205021092040208951092070210926F\\r\\n:1006500006026AE171E00C94C70600D0CDB7DEB7A0\\r\\n:10066000789484B5826084BD84B5816084BD85B58D\\r\\n:10067000826085BD85B5816085BD80916E00816099\\r\\n:1006800080936E001092810080918100826080933F\\r\\n:100690008100809181008160809381008091800041\\r\\n:1006A0008160809380008091B10084608093B1006C\\r\\n:1006B0008091B00081608093B00080917A00846066\\r\\n:1006C00080937A0080917A00826080937A00809192\\r\\n:1006D0007A00816080937A0080917A0080688093AC\\r\\n:1006E0007A001092C100EDE9F0E02491E9E8F0E031\\r\\n:1006F0008491882399F090E0880F991FFC01E859B4\\r\\n:10070000FF4FA591B491FC01EE58FF4F8591949154\\r\\n:100710008FB7F894EC91E22BEC938FBFE0917501C9\\r\\n:10072000F091760182E08083E0917101F091720195\\r\\n:100730001082E0917301F09174018FEC808310922C\\r\\n:100740007D01E0917901F0917A0186E08083E0916A\\r\\n:100750007701F0917801808180618083E091770159\\r\\n:10076000F0917801808188608083E0917701F09139\\r\\n:100770007801808180688083E0917701F091780131\\r\\n:1007800080818F7D80838CE491E00E94E50180E090\\r\\n:10079000A82E80E0B82E85E691E00E942A01181666\\r\\n:1007A00019060CF0CBC085E691E00E9408018D305F\\r\\n:1007B00091F300910602109107028A30E9F08983D3\\r\\n:1007C0001A820F5F1F4FC8010E94F301882319F39B\\r\\n:1007D000209102023091030280910602909107025B\\r\\n:1007E000BE016F5F7F4F820F931F0E94C706109359\\r\\n:1007F000070200930602CFCFE0900202F0900302BE\\r\\n:10080000E114F104F1F10115110509F47BC0470170\\r\\n:100810006401F40181914F01082E000C990B0E9494\\r\\n:10082000B904892BA9F7015011090E0D1F1DF801FC\\r\\n:100830008081082E000C990B0E94B904892B31F09D\\r\\n:100840000C151D0518F001501109F1CFA8014F5FDB\\r\\n:100850005F4F4C195D095093070240930602EC1458\\r\\n:10086000FD0420F4B601C7010E94B506E091020222\\r\\n:10087000F09103028091060290910702E80FF91FA0\\r\\n:1008800010828091060290910702892BD9F1009184\\r\\n:100890000202109103026BE171E0C8010E94BE06E2\\r\\n:1008A000892B19F581E00E9488008EE191E00E9479\\r\\n:1008B000E50190E080E00E94F301282F8091020280\\r\\n:1008C00090910302211131C0009711F00E945B0545\\r\\n:1008D0001092030210920202109207021092060276\\r\\n:1008E000109205021092040256CF69E271E0C8012D\\r\\n:1008F0000E94BE06892B31F480E00E9488008DE2C0\\r\\n:1009000091E0D5CF89E391E00E94D80140910602A1\\r\\n:1009100050910702609102027091030285E691E016\\r\\n:100920000E94D00089E491E0C2CF10920702109299\\r\\n:1009300006026AE171E00E94C7062DCFA114B1043E\\r\\n:1009400009F429CF0E943501882309F424CF0E949D\\r\\n:10095000000021CF8091020290910302009711F0D4\\r\\n:100960000C945B050895EE0FFF1F0590F491E02DA8\\r\\n:10097000099491110C94C204803219F08950855069\\r\\n:10098000C8F708959927882708950F931F93CF9349\\r\\n:10099000DF939C01029710F422E030E0E0910A021C\\r\\n:1009A000F0910B0290E080E0B0E0A0E0309791F48D\\r\\n:1009B000009709F445C0821B930B84309105B8F56C\\r\\n:1009C0008A819B816115710569F1FB0193838283A3\\r\\n:1009D000FE0111C0408151810281138142175307EA\\r\\n:1009E000F0F04217530799F4109761F012960C93A8\\r\\n:1009F000129713961C933296CF01DF91CF911F91DE\\r\\n:100A00000F91089500930A0210930B02F4CF009700\\r\\n:100A100019F04817590718F4CA01BD01EF01DF01A9\\r\\n:100A2000F801C4CF90930B0280930A02D1CFFE014C\\r\\n:100A3000E80FF91F21933193029799838883DCCFC4\\r\\n:100A40008091080290910902892B41F48091020162\\r\\n:100A50009091030190930902809308028091000114\\r\\n:100A600090910101009741F48DB79EB74091040128\\r\\n:100A700050910501841B950B409108025091090289\\r\\n:100A80004817590708F0B8CF841B950B82179307B6\\r\\n:100A900008F4B2CFB9016E5F7F4F8617970708F44D\\r\\n:100AA000ABCF640F751F7093090260930802FA01BF\\r\\n:100AB00021933193A1CFCF93DF939C010097E9F06D\\r\\n:100AC000FC01329713821282C0910A02D0910B026C\\r\\n:100AD000DE0190E080E02097A1F480819181280FD1\\r\\n:100AE000391F80910802909109028217930709F03B\\r\\n:100AF00067C0F0930902E0930802DF91CF91089557\\r\\n:100B0000DA01AE17BF0708F041C012964D915C9113\\r\\n:100B10001397CD014115510599F7DC011396FC930C\\r\\n:100B2000EE9312974D915D91A40FB51FEA17FB0745\\r\\n:100B300081F4208131812E5F3F4F240F351FDC016E\\r\\n:100B400011963C932E932281338113963C932E93DE\\r\\n:100B50001297B0E0A0E08A819B810097B1F5FE0179\\r\\n:100B600081919191E80FF91F8091080290910902FB\\r\\n:100B70008E179F0711F6109759F510920B021092DD\\r\\n:100B80000A02D0930902C0930802B7CFB383A283AD\\r\\n:100B900040815181240F351FA217B30771F44E5FB6\\r\\n:100BA0005F4F2D913C911197420F531F518340830A\\r\\n:100BB00012962D913C9133832283009709F0ADCF9B\\r\\n:100BC000F0930B02E0930A0298CFDE01EC01C3CF51\\r\\n:100BD00013961C921E921297D4CFB0E0A0E0E3EFE0\\r\\n:100BE000F5E00C94D2068C01CB010115110549F4F6\\r\\n:100BF0000E94C5048C01C801CDB7DEB7EEE00C94AD\\r\\n:100C0000EE06F801E60FF71F980122503109E217AE\\r\\n:100C1000F30708F4A7C0D901CD91DC911197C6174D\\r\\n:100C2000D707A0F0C530D10530F3AE0144505109CB\\r\\n:100C30004617570700F3C61BD70B2297C193D193D2\\r\\n:100C40006D937C93CF010E945B05D5CF4B018C1A2D\\r\\n:100C50009D0A5801AC0EBD1EA0910A02B0910B0274\\r\\n:100C6000712C612CF12CE12C109739F54091080280\\r\\n:100C7000509109024A155B0509F066C068167906AD\\r\\n:100C800008F062C0409100015091010141155105E9\\r\\n:100C900041F44DB75EB76091040170910501461BA8\\r\\n:100CA000570BE417F50708F05DC0F0930902E093D5\\r\\n:100CB0000802F901918380839ECF4D915C91119739\\r\\n:100CC000AA15BB05B9F56A0162E0C60ED11CC814AD\\r\\n:100CD000D90480F1BE01681B790B640F751F129651\\r\\n:100CE000DC9012971396AC915B01B2E0AB0EB11C95\\r\\n:100CF000B5E0AB16B10480F0D282A3837183608328\\r\\n:100D0000D9018D939C93E114F10479F0D7011396E6\\r\\n:100D1000FC93EE9312976FCF22964C0F5D1FF90153\\r\\n:100D200051834083ED2DFA2FEECFF0930B02E09329\\r\\n:100D30000A0261CF6416750608F43A017D01129625\\r\\n:100D40000D90BC91A02D90CF0E94C5047C0100970E\\r\\n:100D500049F0AE01B8010E94B506C8010E945B05CA\\r\\n:100D6000870149CF10E000E046CFFB01DC0102C063\\r\\n:100D700001900D9241505040D8F70895FB01DC01DD\\r\\n:100D80008D91019080190110D9F3990B0895FB0101\\r\\n:100D9000DC0101900D920020E1F708952F923F921F\\r\\n:100DA0004F925F926F927F928F929F92AF92BF927B\\r\\n:100DB000CF92DF92EF92FF920F931F93CF93DF9327\\r\\n:100DC000CDB7DEB7CA1BDB0B0FB6F894DEBF0FBE84\\r\\n:100DD000CDBF09942A88398848885F846E847D84D1\\r\\n:100DE0008C849B84AA84B984C884DF80EE80FD80D3\\r\\n:100DF0000C811B81AA81B981CE0FD11D0FB6F89449\\r\\n:100E0000DEBF0FBECDBFED01089510E0CDE5D0E00F\\r\\n:100E100004C0FE010E94B3042196CE35D107C9F764\\r\\n:040E2000F894FFCF74\\r\\n:100E240000000C022000000000006B01D000F9005B\\r\\n:100E3400B8012A0108011C010D0A006F6E0044313B\\r\\n:100E440033206973206F6E2E006F66660044313361\\r\\n:100E5400206973206F66662E004920646F206E6FD0\\r\\n:100E640074206B6E6F77202200222E2054797065D7\\r\\n:0C0E7400206F6E206F72206F66662E00EB\\r\\n:00000001FF\\r\\n"
      }
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 220.0
        }
      },
      "x": 370,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "Led": {
          "colour": "Red"
        }
      },
      "x": 460,
      "y": 140,
      "turns": 0
    }
  ],
  "nets": [
    {
      "pins": [
        {
          "element": 0,
          "pin": "D13"
        },
        {
          "element": 1,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 1,
          "pin": "b"
        },
        {
          "element": 2,
          "pin": "anode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "GND"
        },
        {
          "element": 2,
          "pin": "cathode"
        }
      ]
    }
  ]
}
`,Ze=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "description": "Serial at 115200\\nA TMP36 printed in °C at 115200 baud. The monitor starts at 9600: garbage until it is set to match.",
  "elements": [
    {
      "kind": "Uno",
      "x": 200,
      "y": 300,
      "turns": 0,
      "program": {
        "name": "SerialThermometer",
        "hex": ":100000000C9435000C945D000C945D000C945D0024\\r\\n:100010000C945D000C945D000C945D000C945D00EC\\r\\n:100020000C945D000C945D000C945D000C945D00DC\\r\\n:100030000C945D000C945D000C945D000C945D00CC\\r\\n:100040000C94AC010C945D000C941A020C94F40115\\r\\n:100050000C945D000C945D000C945D000C945D00AC\\r\\n:100060000C945D000C945D005F0411241FBECFEF63\\r\\n:10007000D8E0DEBFCDBF11E0A0E0B1E0E4EEFDE0EE\\r\\n:1000800002C005900D92A832B107D9F721E0A8E28D\\r\\n:10009000B1E001C01D92AE3CB207E1F710E0C5E34C\\r\\n:1000A000D0E004C02197FE010E94AE04C433D10702\\r\\n:1000B000C9F70E9447020C94F0060C940000CF93FD\\r\\n:1000C000DF9300D000D0CDB7DEB73FB7F894809172\\r\\n:1000D0002D0190912E01A0912F01B091300126B5F4\\r\\n:1000E000A89B05C02F3F19F00196A11DB11D3FBF70\\r\\n:1000F000BA2FA92F982F8827820F911DA11DB11DFE\\r\\n:100100009C01AD0162E0220F331F441F551F6A9509\\r\\n:10011000D1F7CA01B9010F900F900F900F90DF91A6\\r\\n:10012000CF910895CF92DF92EF92FF920F931F939A\\r\\n:10013000CF93DF937C016A0110E000E00C151D05F0\\r\\n:1001400081F0EB01D701ED91FC910190F081E02D60\\r\\n:100150006991C7010995892B21F00F5F1F4FBE01DF\\r\\n:10016000EDCFC801DF91CF911F910F91FF90EF90DC\\r\\n:10017000DF90CF900895FC01538D448D252F30E002\\r\\n:10018000842F90E0821B930B541710F0CF960895A4\\r\\n:1001900001970895FC01918D828D981761F0A28DD1\\r\\n:1001A000AE0FBF2FB11D5D968C91928D9F5F9F7397\\r\\n:1001B000928F90E008958FEF9FEF0895FC01918D4D\\r\\n:1001C000828D981731F0828DE80FF11D858D90E0BA\\r\\n:1001D00008958FEF9FEF0895FC01818D228D90E0AF\\r\\n:1001E000805C9F4F821B91098F739927089581E34B\\r\\n:1001F00091E00E94EC009C0181E0232B09F480E057\\r\\n:10020000089580E090E0892B29F00E94F700811189\\r\\n:100210000C9400000895FC01A48DA80FB92FB11D06\\r\\n:10022000A35ABF4F2C91848D90E001968F7399272C\\r\\n:10023000848FA689B7892C93A089B1898C9183700A\\r\\n:1002400080648C93938D848D981306C00288F38903\\r\\n:10025000E02D80818F7D80830895EF92FF920F9330\\r\\n:100260001F93CF93DF93EC01F62E81E0888F9B8D57\\r\\n:100270008C8D98131AC0E889F989808185FF15C093\\r\\n:100280009FB7F894EE89FF896083E889F9898081B6\\r\\n:100290008370806480839FBF81E090E0DF91CF9185\\r\\n:1002A0001F910F91FF90EF9008950B8D10E00F5F5D\\r\\n:1002B0001F4F0F731127E02E8C8D8E110CC00FB6BF\\r\\n:1002C00007FCFACFE889F989808185FFF5CFCE0157\\r\\n:1002D0000E940B01F1CFEB8DEC0FFD2FF11DE35AC6\\r\\n:1002E000FF4FF0829FB7F8940B8FEA89FB898081DA\\r\\n:1002F0008062D0CFCF93DF93EC01888D8823B9F053\\r\\n:10030000AA89BB89E889F9898C9185FD03C0808120\\r\\n:1003100086FD0DC00FB607FCF7CF8C9185FFF2CF9D\\r\\n:10032000808185FFEDCFCE010E940B01E9CFDF91E7\\r\\n:10033000CF910895BC01009761F0FC01019000206D\\r\\n:10034000E9F73197AF01481B590B81E391E00C9419\\r\\n:10035000920090E080E008951F921FB61F92112432\\r\\n:100360002F933F938F939F93AF93BF9380912901D6\\r\\n:1003700090912A01A0912B01B0912C01309128017C\\r\\n:1003800023E0230F2D3750F50196A11DB11D2093B9\\r\\n:1003900028018093290190932A01A0932B01B09307\\r\\n:1003A0002C0180912D0190912E01A0912F01B091EF\\r\\n:1003B00030010196A11DB11D80932D0190932E0156\\r\\n:1003C000A0932F01B0933001BF91AF919F918F9176\\r\\n:1003D0003F912F911F901FBE1F90189526E8230F65\\r\\n:1003E0000296A11DB11DD3CF1F920F920FB60F928F\\r\\n:1003F00011242F933F934F935F936F937F938F932A\\r\\n:100400009F93AF93BF93EF93FF9381E391E00E949B\\r\\n:100410000B01FF91EF91BF91AF919F918F917F91D0\\r\\n:100420006F915F914F913F912F910F900FBE0F9061\\r\\n:100430001F9018952F932FB72F938F939F93EF9320\\r\\n:10044000FF93E0914101F09142018081E0914701E9\\r\\n:10045000F091480182FD19C0908180914A018F5F1F\\r\\n:100460008F7320914B01821741F0E0914A01F0E037\\r\\n:10047000EF5CFE4F958F80934A01FF91EF919F9122\\r\\n:100480008F912F912FBF2F9118958081F6CFCDB7E7\\r\\n:10049000DEB7AD970FB6F894DEBF0FBECDBF789430\\r\\n:1004A00084B5826084BD84B5816084BD85B5826079\\r\\n:1004B00085BD85B5816085BD80916E00816080932A\\r\\n:1004C0006E00109281008091810082608093810093\\r\\n:1004D00080918100816080938100809180008160A3\\r\\n:1004E000809380008091B10084608093B1008091FE\\r\\n:1004F000B00081608093B00080917A008460809326\\r\\n:100500007A0080917A00826080937A0080917A00EC\\r\\n:10051000816080937A0080917A00806880937A006D\\r\\n:100520001092C10083E080930001E0914101F091BD\\r\\n:10053000420182E08083E0913D01F0913E01108212\\r\\n:10054000E0913F01F091400180E1808310924901E8\\r\\n:10055000E0914501F091460186E08083E0914301FE\\r\\n:10056000F0914401808180618083E0914301F091AA\\r\\n:100570004401808188608083E0914301F0914401CF\\r\\n:10058000808180688083E0914301F0914401808103\\r\\n:100590008F7D80839AE0292E312C412C512C809123\\r\\n:1005A000000120E4829FC001112480937C0080918F\\r\\n:1005B0007A00806480937A0080917A0086FDFCCF77\\r\\n:1005C00080917800909179009C01990F440B550B14\\r\\n:1005D0002AA33BA34CA35DA3CA01B9010E94C9058C\\r\\n:1005E00020E030E849E854E46AA37BA38CA39DA3F0\\r\\n:1005F0006AA17BA18CA19DA10E947E0620E030E033\\r\\n:1006000040E85AE36AA37BA38CA39DA36AA17BA1C4\\r\\n:100610008CA19DA10E947E0620E030E04AEF53E4C9\\r\\n:100620006AA37BA38CA39DA36AA17BA18CA19DA19E\\r\\n:100630000E94B40420E030E040E251E46AA37BA3CE\\r\\n:100640008CA39DA36AA17BA18CA19DA10E942605DC\\r\\n:10065000462F572FBC014AA35BA36CA37DA3DC01EB\\r\\n:10066000CA018EA39FA3A8A7B9A79C01AB01CB0188\\r\\n:10067000B9010E94EB06882309F444C083E191E0AC\\r\\n:100680000E949A018FE191E00E949A0184E291E038\\r\\n:100690000E949A010E945F006B017C0184EF882E0A\\r\\n:1006A00099249394A12CB12C0E945F006AA37BA390\\r\\n:1006B0008CA39DA3DC01CB018C199D09AE09BF0958\\r\\n:1006C000883E9340A105B10578F3F1E08F1A9108B7\\r\\n:1006D000A108B10828EEC20E23E0D21EE11CF11CD5\\r\\n:1006E00081149104A104B104F9F640E050E0452BD7\\r\\n:1006F00009F455CF0E94F700882309F450CF0E94D7\\r\\n:1007000000004DCF2AA13BA14CA15DA15F772AA794\\r\\n:100710003BA74CA75DA72FEF3FEF4FE75FE76AA529\\r\\n:100720007BA58CA59DA50E94EB0681110FC02FEF24\\r\\n:100730003FEF4FE75FE76AA57BA58CA59DA50E94CB\\r\\n:10074000210518161CF487E191E09ACF2FEF3FEFB7\\r\\n:100750004FE75FE46AA17BA18CA19DA10E9479066D\\r\\n:1007600018161CF48BE191E08BCF2FEF3FEF4FE792\\r\\n:100770005FEC6AA17BA18CA19DA10E94210587FD50\\r\\n:10078000F1CF20E030E0A9016AA17BA18CA19DA15D\\r\\n:100790000E94210587FF0EC06DE281E391E00E9477\\r\\n:1007A0002D018AA19BA1ACA1BDA1B0588EA39FA38E\\r\\n:1007B000A8A7B9A72DEC3CEC4CE45DE36EA17FA1AA\\r\\n:1007C00088A599A50E94B5045B016C016AA3BBA230\\r\\n:1007D0008CA3DDA26AA17BA18CA19DA10E9498059A\\r\\n:1007E0007B018C010E94C7059B01AC016AA17BA122\\r\\n:1007F0008CA19DA10E94B4043B015C0119A26E0171\\r\\n:1008000091E2C90ED11C4601B701C801A2019101B4\\r\\n:100810000E948C04C701D80179018A01605DF6014C\\r\\n:1008200062936F010A97A105B10570F7CF010E948D\\r\\n:100830009A016EE281E391E00E942D0120E030E018\\r\\n:1008400040E251E46AA27BA2ACA2BDA26AA17BA154\\r\\n:100850008CA19DA10E947E066AA37BA38CA39DA36D\\r\\n:100860006AA17BA18CA19DA10E9498056AA37BA38C\\r\\n:100870008CA39DA34AA15BA19A0150E040E02AA36A\\r\\n:100880003BA34CA35DA319A284016AA17BA18CA107\\r\\n:100890009DA1A20191010E948C048AA19BA1ACA1FF\\r\\n:1008A000BDA12AA33BA34CA35DA3605DF8016293A5\\r\\n:1008B0008F010A97A105B10540F7CF01E1CEE1E331\\r\\n:1008C000F1E01382128288EE93E0A0E0B0E084832E\\r\\n:1008D0009583A683B78385E091E09183808385EC3F\\r\\n:1008E00090E09587848784EC90E09787868780ECFA\\r\\n:1008F00090E0918B808B81EC90E0938B828B82ECEB\\r\\n:1009000090E0958B848B86EC90E0978B868B118E94\\r\\n:10091000128E138E148E0895A1E21A2EAA1BBB1BF1\\r\\n:10092000FD010DC0AA1FBB1FEE1FFF1FA217B307BB\\r\\n:10093000E407F50720F0A21BB30BE40BF50B661FD1\\r\\n:10094000771F881F991F1A9469F760957095809595\\r\\n:1009500090959B01AC01BD01CF010895EE0FFF1FE3\\r\\n:100960000590F491E02D09945058BB27AA270E94C6\\r\\n:10097000CC040C943F060E94310638F00E943806E1\\r\\n:1009800020F039F49F3F19F426F40C942E060EF44F\\r\\n:10099000E095E7FB0C942806E92F0E94500658F3D7\\r\\n:1009A000BA17620773078407950720F079F4A6F554\\r\\n:1009B0000C9472060EF4E0950B2EBA2FA02D0B01AD\\r\\n:1009C000B90190010C01CA01A0011124FF27591B94\\r\\n:1009D00099F0593F50F4503E68F11A16F040A22F9A\\r\\n:1009E000232F342F4427585FF3CF4695379527950B\\r\\n:1009F000A795F0405395C9F77EF41F16BA0B620B0A\\r\\n:100A0000730B840BBAF09150A1F0FF0FBB1F661F50\\r\\n:100A1000771F881FC2F70EC0BA0F621F731F841F93\\r\\n:100A200048F4879577956795B795F7959E3F08F0B9\\r\\n:100A3000B0CF9395880F08F09927EE0F979587957B\\r\\n:100A400008950E94040608F481E008950E943A0582\\r\\n:100A50000C943F060E94380658F00E94310640F080\\r\\n:100A600029F45F3F29F00C94280651110C94730669\\r\\n:100A70000C942E060E94500668F39923B1F3552377\\r\\n:100A800091F3951B550BBB27AA27621773078407A1\\r\\n:100A900038F09F5F5F4F220F331F441FAA1FA9F337\\r\\n:100AA00035D00E2E3AF0E0E832D091505040E69525\\r\\n:100AB000001CCAF72BD0FE2F29D0660F771F881F86\\r\\n:100AC000BB1F261737074807AB07B0E809F0BB0B79\\r\\n:100AD000802DBF01FF2793585F4F3AF09E3F51058D\\r\\n:100AE00078F00C9428060C9473065F3FE4F3983E6C\\r\\n:100AF000D4F3869577956795B795F7959F5FC9F776\\r\\n:100B0000880F911D9695879597F90895E1E0660FF6\\r\\n:100B1000771F881FBB1F621773078407BA0720F06F\\r\\n:100B2000621B730B840BBA0BEE1F88F7E0950895D8\\r\\n:100B30000E94580688F09F5798F0B92F9927B7510F\\r\\n:100B4000B0F0E1F0660F771F881F991F1AF0BA9571\\r\\n:100B5000C9F714C0B13091F00E947206B1E0089557\\r\\n:100B60000C947206672F782F8827B85F39F0B93F49\\r\\n:100B7000CCF3869577956795B395D9F73EF4909524\\r\\n:100B80008095709561957F4F8F4F9F4F0895E894A2\\r\\n:100B900009C097FB3EF490958095709561957F4FC5\\r\\n:100BA0008F4F9F4F9923A9F0F92F96E9BB27939573\\r\\n:100BB000F695879577956795B795F111F8CFFAF483\\r\\n:100BC000BB0F11F460FF1BC06F5F7F4F8F4F9F4FB4\\r\\n:100BD00016C0882311F096E911C0772321F09EE812\\r\\n:100BE000872F762F05C0662371F096E8862F70E078\\r\\n:100BF00060E02AF09A95660F771F881FDAF7880F52\\r\\n:100C00009695879597F90895990F0008550FAA0BA7\\r\\n:100C1000E0E8FEEF16161706E807F907C0F012160F\\r\\n:100C20001306E407F50798F0621B730B840B950B12\\r\\n:100C300039F40A2661F0232B242B252B21F4089567\\r\\n:100C40000A2609F4A140A6958FEF811D811D089504\\r\\n:100C500097F99F6780E870E060E008959FEF80EC6F\\r\\n:100C6000089500240A941616170618060906089512\\r\\n:100C700000240A9412161306140605060895092E78\\r\\n:100C80000394000C11F4882352F0BB0F40F4BF2BE7\\r\\n:100C900011F460FF04C06F5F7F4F8F4F9F4F089527\\r\\n:100CA00057FD9058440F551F59F05F3F71F047951D\\r\\n:100CB000880F97FB991F61F09F3F79F08795089502\\r\\n:100CC000121613061406551FF2CF4695F1DF08C021\\r\\n:100CD000161617061806991FF1CF8695710561053E\\r\\n:100CE00008940895E894BB2766277727CB0197F9E6\\r\\n:100CF00008950E94040608F48FEF08950E9491065B\\r\\n:100D00000C943F060E94310638F00E94380620F00D\\r\\n:100D1000952311F00C9428060C942E0611240C94A3\\r\\n:100D200073060E94500670F3959FC1F3950F50E033\\r\\n:100D3000551F629FF001729FBB27F00DB11D639F8D\\r\\n:100D4000AA27F00DB11DAA1F649F6627B00DA11D33\\r\\n:100D5000661F829F2227B00DA11D621F739FB00DD9\\r\\n:100D6000A11D621F839FA00D611D221F749F332749\\r\\n:100D7000A00D611D231F849F600D211D822F762FE2\\r\\n:100D80006A2F11249F5750409AF0F1F088234AF0BF\\r\\n:100D9000EE0FFF1FBB1F661F771F881F915050402B\\r\\n:100DA000A9F79E3F510580F00C9428060C94730619\\r\\n:100DB0005F3FE4F3983ED4F3869577956795B795B2\\r\\n:100DC000F795E7959F5FC1F7FE2B880F911D9695CC\\r\\n:100DD000879597F908950E940406880B990B08954A\\r\\n:040DE000F894FFCFB5\\r\\n:100DE40001000000002D019200BB007A01EC00CA52\\r\\n:100DF40000DE006E616E00696E66006F766600202C\\r\\n:080E0400C2B043000D0A00001A\\r\\n:00000001FF\\r\\n"
      }
    },
    {
      "kind": {
        "Tmp36": {
          "celsius": 22.0
        }
      },
      "x": 370,
      "y": 460,
      "turns": 0
    }
  ],
  "nets": [
    {
      "pins": [
        {
          "element": 0,
          "pin": "GND"
        },
        {
          "element": 1,
          "pin": "GND"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "5V"
        },
        {
          "element": 1,
          "pin": "+Vs"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "A0"
        },
        {
          "element": 1,
          "pin": "Vout"
        }
      ]
    }
  ]
}
`,Qe=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "description": "PWM, smoothed\\nD9 on a quarter of the time, through 10 kΩ into 1 µF: about 1.25 V, and a ripple.",
  "elements": [
    {
      "kind": "Uno",
      "x": 200,
      "y": 300,
      "turns": 0,
      "program": {
        "name": "Smoothed",
        "hex": ":100000000C945C000C946E000C946E000C946E00CA\\r\\n:100010000C946E000C946E000C946E000C946E00A8\\r\\n:100020000C946E000C946E000C946E000C946E0098\\r\\n:100030000C946E000C946E000C946E000C946E0088\\r\\n:100040000C948C000C946E000C946E000C946E005A\\r\\n:100050000C946E000C946E000C946E000C946E0068\\r\\n:100060000C946E000C946E00000000080002010069\\r\\n:100070000003040700000000000000000000000072\\r\\n:10008000250028002B0000000000240027002A0083\\r\\n:10009000040404040404040402020202020203032E\\r\\n:1000A0000303030301020408102040800102040836\\r\\n:1000B000102001020408102011241FBECFEFD8E049\\r\\n:1000C000DEBFCDBF21E0A0E0B1E001C01D92A930AC\\r\\n:1000D000B207E1F70E94D4000C94B4010C94000024\\r\\n:1000E000EDEAF0E02491E9E9F0E08491882399F0C9\\r\\n:1000F00090E0880F991FFC01EA57FF4FA591B4913A\\r\\n:10010000FC01E458FF4F859194918FB7F894EC91DE\\r\\n:10011000E22BEC938FBF08951F921FB61F921124FC\\r\\n:100120002F933F938F939F93AF93BF93809105013C\\r\\n:1001300090910601A0910701B0910801309104014E\\r\\n:1001400023E0230F2D3750F50196A11DB11D2093FB\\r\\n:1001500004018093050190930601A0930701B093D9\\r\\n:1001600008018091000190910101A0910201B091DC\\r\\n:1001700003010196A11DB11D80930001909301011F\\r\\n:10018000A0930201B0930301BF91AF919F918F9112\\r\\n:100190003F912F911F901FBE1F90189526E8230FA7\\r\\n:1001A0000296A11DB11DD3CF789484B5826084BD21\\r\\n:1001B00084B5816084BD85B5826085BD85B581606B\\r\\n:1001C00085BD80916E00816080936E0010928100E9\\r\\n:1001D00080918100826080938100809181008160A4\\r\\n:1001E0008093810080918000816080938000809165\\r\\n:1001F000B10084608093B1008091B00081608093F1\\r\\n:10020000B00080917A00846080937A0080917A00B7\\r\\n:10021000826080937A0080917A00816080937A0076\\r\\n:1002200080917A00806880937A001092C1000E94C9\\r\\n:1002300070000E947000E1E7F0E08491843009F4DE\\r\\n:100240005EC008F031C0823009F447C0833009F441\\r\\n:100250004AC0813009F435C08491EDEAF0E0949110\\r\\n:10026000E9E9F0E0E491EE2389F1882371F084302C\\r\\n:1002700009F469C008F058C0823009F46CC08330BA\\r\\n:1002800009F45BC0813009F462C0F0E0EE0FFF1F9B\\r\\n:10029000E458FF4FA591B4918FB7F894EC919095E5\\r\\n:1002A0009E239C938FBF12C08730A9F18830A1F69E\\r\\n:1002B0008091B00080628093B00080E48093B400AD\\r\\n:1002C00005C084B5806884BD80E487BDC0E0D0E00F\\r\\n:1002D0002097F1F30E940000FBCF84B5806284BDBB\\r\\n:1002E00080E488BDF3CF8091800080688093800097\\r\\n:1002F00080E490E09093890080938800E7CF80911C\\r\\n:10030000800080628093800080E490E090938B0076\\r\\n:1003100080938A00DBCF8091B00080688093B0002A\\r\\n:1003200080E48093B300D2CF8730C1F0883009F0E9\\r\\n:10033000ACCF8091B0008F7D14C0809180008F770A\\r\\n:1003400080938000A2CF809180008F7DF9CF84B50B\\r\\n:100350008F7784BD9ACF84B58F7DFBCF8091B0001D\\r\\n:0C0360008F778093B00091CFF894FFCF0E\\r\\n:00000001FF\\r\\n"
      }
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 10000.0
        }
      },
      "x": 370,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "Capacitor": {
          "farads": 1e-06,
          "initial_volts": 0.0
        }
      },
      "x": 460,
      "y": 190,
      "turns": 1
    },
    {
      "kind": "Voltmeter",
      "x": 540,
      "y": 190,
      "turns": 1
    }
  ],
  "nets": [
    {
      "pins": [
        {
          "element": 0,
          "pin": "D9"
        },
        {
          "element": 1,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 1,
          "pin": "b"
        },
        {
          "element": 2,
          "pin": "a"
        },
        {
          "element": 3,
          "pin": "+"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 2,
          "pin": "b"
        },
        {
          "element": 3,
          "pin": "-"
        },
        {
          "element": 0,
          "pin": "GND"
        }
      ]
    }
  ]
}
`,$e=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "description": "Traffic light\\nThree pins, three LEDs and delay(): green, then the middle one, then red.",
  "elements": [
    {
      "kind": "Uno",
      "x": 200,
      "y": 300,
      "turns": 0,
      "program": {
        "name": "TrafficLight",
        "hex": ":100000000C945C000C946E000C946E000C946E00CA\\r\\n:100010000C946E000C946E000C946E000C946E00A8\\r\\n:100020000C946E000C946E000C946E000C946E0098\\r\\n:100030000C946E000C946E000C946E000C946E0088\\r\\n:100040000C9468010C946E000C946E000C946E007D\\r\\n:100050000C946E000C946E000C946E000C946E0068\\r\\n:100060000C946E000C946E00000000080002010069\\r\\n:100070000003040700000000000000000000000072\\r\\n:10008000250028002B0000000000240027002A0083\\r\\n:10009000040404040404040402020202020203032E\\r\\n:1000A0000303030301020408102040800102040836\\r\\n:1000B000102001020408102011241FBECFEFD8E049\\r\\n:1000C000DEBFCDBF21E0A0E0B1E001C01D92A930AC\\r\\n:1000D000B207E1F70E94B0010C941B020C940000DF\\r\\n:1000E00090E0FC01EC55FF4F249180579F4FFC019D\\r\\n:1000F0008491882399F090E0880F991FFC01EA57BA\\r\\n:10010000FF4FA591B491FC01E458FF4F8591949164\\r\\n:100110008FB7F894EC91E22BEC938FBF089590E0A9\\r\\n:10012000FC01E859FF4F2491FC01EC55FF4F34913D\\r\\n:10013000FC01E057FF4FE491EE23D9F0222349F070\\r\\n:10014000243031F1B8F4223059F12330D9F0213084\\r\\n:1001500019F1F0E0EE0FFF1FE458FF4FA591B491A5\\r\\n:100160008FB7F894EC91611124C030953E233C93F5\\r\\n:100170008FBF08952730B9F0283059F78091B0002B\\r\\n:100180008F7D14C0809180008F7780938000E1CFB5\\r\\n:10019000809180008F7DF9CF84B58F7784BDD9CFD2\\r\\n:1001A00084B58F7DFBCF8091B0008F778093B000B6\\r\\n:1001B000D0CF3E2BDCCFCF93DF9300D000D0CDB794\\r\\n:1001C000DEB73FB7F8948091050190910601A091A8\\r\\n:1001D0000701B091080126B5A89B05C02F3F19F073\\r\\n:1001E0000196A11DB11D3FBFBA2FA92F982F8827B7\\r\\n:1001F000820F911DA11DB11D9C01AD0162E0220F76\\r\\n:10020000331F441F551F6A95D1F7CA01B9010F90DA\\r\\n:100210000F900F900F90DF91CF9108958F929F9242\\r\\n:10022000AF92BF92CF92DF92EF92FF92CF93DF9384\\r\\n:1002300000D000D0CDB7DEB77C014A015B0161E0A0\\r\\n:100240000C9709F060E08CE00E948F0061E08BE089\\r\\n:10025000E816F10409F060E08BE00E948F0061E095\\r\\n:100260002AE0E216F10409F060E08AE00E948F00C3\\r\\n:100270000E94DB006B017C010E94DB00DC01CB01F2\\r\\n:100280008C199D09AE09BF09883E9340A105B105AF\\r\\n:1002900098F381E0881A9108A108B10828EEC20EEF\\r\\n:1002A00023E0D21EE11CF11C81149104A104B104CD\\r\\n:1002B00019F70F900F900F900F90DF91CF91FF9053\\r\\n:1002C000EF90DF90CF90BF90AF909F908F90089568\\r\\n:1002D0001F921FB61F9211242F933F938F939F93CA\\r\\n:1002E000AF93BF938091010190910201A09103010E\\r\\n:1002F000B09104013091000123E0230F2D3750F518\\r\\n:100300000196A11DB11D20930001809301019093DE\\r\\n:100310000201A0930301B093040180910501909123\\r\\n:100320000601A0910701B09108010196A11DB11D20\\r\\n:100330008093050190930601A0930701B0930801F3\\r\\n:10034000BF91AF919F918F913F912F911F901FBEB1\\r\\n:100350001F90189526E8230F0296A11DB11DD3CF3B\\r\\n:10036000789484B5826084BD84B5816084BD85B590\\r\\n:10037000826085BD85B5816085BD80916E0081609C\\r\\n:1003800080936E0010928100809181008260809342\\r\\n:100390008100809181008160809381008091800044\\r\\n:1003A0008160809380008091B10084608093B1006F\\r\\n:1003B0008091B00081608093B00080917A00846069\\r\\n:1003C00080937A0080917A00826080937A00809195\\r\\n:1003D0007A00816080937A0080917A0080688093AF\\r\\n:1003E0007A001092C1008CE00E9470008BE00E94A5\\r\\n:1003F00070008AE00E947000C0E0D0E040ED57E05D\\r\\n:1004000060E070E08AE090E00E940E0144EF51E06D\\r\\n:1004100060E070E08BE090E00E940E0140ED57E05C\\r\\n:1004200060E070E08CE090E00E940E01209731F3D4\\r\\n:0A0430000E940000E3CFF894FFCF14\\r\\n:00000001FF\\r\\n"
      }
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 220.0
        }
      },
      "x": 370,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "Led": {
          "colour": "Red"
        }
      },
      "x": 460,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 220.0
        }
      },
      "x": 370,
      "y": 200,
      "turns": 0
    },
    {
      "kind": {
        "Led": {
          "colour": "White"
        }
      },
      "x": 460,
      "y": 200,
      "turns": 0
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 220.0
        }
      },
      "x": 370,
      "y": 260,
      "turns": 0
    },
    {
      "kind": {
        "Led": {
          "colour": "Green"
        }
      },
      "x": 460,
      "y": 260,
      "turns": 0
    }
  ],
  "nets": [
    {
      "pins": [
        {
          "element": 0,
          "pin": "D12"
        },
        {
          "element": 1,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 1,
          "pin": "b"
        },
        {
          "element": 2,
          "pin": "anode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "D11"
        },
        {
          "element": 3,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 3,
          "pin": "b"
        },
        {
          "element": 4,
          "pin": "anode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "D10"
        },
        {
          "element": 5,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 5,
          "pin": "b"
        },
        {
          "element": 6,
          "pin": "anode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "GND"
        },
        {
          "element": 2,
          "pin": "cathode"
        },
        {
          "element": 4,
          "pin": "cathode"
        },
        {
          "element": 6,
          "pin": "cathode"
        }
      ]
    }
  ]
}
`,et=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "description": "Sweep\\nThe Servo library on D9: a pulse every 20 ms, its length the angle.",
  "elements": [
    {
      "kind": "Uno",
      "x": 200,
      "y": 300,
      "turns": 0,
      "program": {
        "name": "Sweep",
        "hex": ":100000000C945D000C9485000C9485000C94850084\\r\\n:100010000C9485000C9485000C9485000C9485004C\\r\\n:100020000C9485000C9485000C9485000C941602A9\\r\\n:100030000C9485000C9485000C9485000C9485002C\\r\\n:100040000C94CE010C9485000C9485000C948500D2\\r\\n:100050000C9485000C9485000C9485000C9485000C\\r\\n:100060000C9485000C9485000000000024002700FB\\r\\n:100070002A0000000000250028002B0004040404CE\\r\\n:100080000404040402020202020203030303030342\\r\\n:10009000010204081020408001020408102001021F\\r\\n:1000A00004081020000000080002010000030407FB\\r\\n:1000B00000000000000000009A0311241FBECFEFD3\\r\\n:1000C000D8E0DEBFCDBF21E0A0E0B1E001C01D92CD\\r\\n:1000D000A233B207E1F711E0A0E0B1E0E8E5F8E013\\r\\n:1000E00002C005900D92A030B107D9F710E0CDE520\\r\\n:1000F000D0E004C02197FE010E94C103CC35D10796\\r\\n:10010000C9F70E94CD020C942A040C9400008F922F\\r\\n:100110009F92AF92BF92FF920F931F93CF93DF9363\\r\\n:1001200000D000D000D0CDB7DEB7F0902F012BE08B\\r\\n:100130002F1508F461C02091310108E512E0021B7F\\r\\n:10014000110927FD1395000F111F000F111F20919A\\r\\n:10015000300148E850E0421B510927FD5395440FF8\\r\\n:10016000551F440F551F5E834D834A01550FAA0842\\r\\n:10017000BB08DC01853B910514F0A4EBB0E0B7FFB0\\r\\n:1001800002C0B0E0A0E09801012E000C440B550B1A\\r\\n:10019000281939094A095B090E94F10369837A83A6\\r\\n:1001A0008B839C8324EB30E040E050E00E94C70347\\r\\n:1001B000820E931EA41EB51E2D813E8182169306CB\\r\\n:1001C0003CF01E830D838016910614F49E828D826E\\r\\n:1001D0004D815E8142505109440F551F2FB7F8944D\\r\\n:1001E0008F2D90E0FC01EE0FFF1FE80FF91FE55F78\\r\\n:1001F000FE4F528341832FBF26960FB6F894DEBF81\\r\\n:100200000FBECDBFDF91CF911F910F91FF90BF9097\\r\\n:10021000AF909F908F90089590E0FC01EC55FF4FB8\\r\\n:100220002491FC01E057FF4F3491FC01E458FF4F4B\\r\\n:10023000E491EE23D9F0222349F0243031F1B8F4CF\\r\\n:10024000223059F12330D9F0213019F1F0E0EE0FCE\\r\\n:10025000FF1FEE58FF4FA591B4918FB7F894EC9122\\r\\n:10026000611124C030953E233C938FBF0895273001\\r\\n:10027000B9F0283059F78091B0008F7D14C080917B\\r\\n:1002800080008F7780938000E1CF809180008F7D08\\r\\n:10029000F9CF84B58F7784BDD9CF84B58F7DFBCF5F\\r\\n:1002A0008091B0008F778093B000D0CF3E2BDCCF11\\r\\n:1002B000CF93DF9300D000D0CDB7DEB73FB7F8942F\\r\\n:1002C0008091070190910801A0910901B0910A0164\\r\\n:1002D00026B5A89B05C02F3F19F00196A11DB11DA1\\r\\n:1002E0003FBFBA2FA92F982F8827820F911DA11DDC\\r\\n:1002F000B11D9C01AD0162E0220F331F441F551F49\\r\\n:100300006A95D1F7CA01B9010F900F900F900F9025\\r\\n:10031000DF91CF9108958F929F92AF92BF92CF922B\\r\\n:10032000DF92EF92FF92CF93DF9300D000D0CDB752\\r\\n:10033000DEB70E9458016B017C018AE0882E912C67\\r\\n:10034000A12CB12C0E945801DC01CB018C199D0914\\r\\n:10035000AE09BF09883E9340A105B10598F321E09D\\r\\n:10036000821A9108A108B10888EEC80E83E0D81E51\\r\\n:10037000E11CF11C81149104A104B10419F70F9040\\r\\n:100380000F900F900F90DF91CF91FF90EF90DF9043\\r\\n:10039000CF90BF90AF909F908F9008951F921FB6FF\\r\\n:1003A0001F9211242F933F938F939F93AF93BF93EB\\r\\n:1003B0008091030190910401A0910501B091060183\\r\\n:1003C0003091020123E0230F2D3750F50196A11D36\\r\\n:1003D000B11D209302018093030190930401A09327\\r\\n:1003E0000501B09306018091070190910801A09149\\r\\n:1003F0000901B0910A010196A11DB11D8093070169\\r\\n:1004000090930801A0930901B0930A01BF91AF91A5\\r\\n:100410009F918F913F912F911F901FBE1F90189514\\r\\n:1004200026E8230F0296A11DB11DD3CF1F920F9274\\r\\n:100430000FB60F9211242F933F934F935F936F93B7\\r\\n:100440007F938F939F93AF93BF93CF93DF93EF935C\\r\\n:10045000FF9380910101C0910001D0E087FF24C08B\\r\\n:100460001092850010928400809101018F5F80932B\\r\\n:10047000010180910101082E000C990B8C179D073A\\r\\n:1004800024F4809101018C30B4F18091840090912A\\r\\n:100490008500049680349C4908F475C080918400DE\\r\\n:1004A00090918500049671C080910101082E000C86\\r\\n:1004B000990B8C179D07C4F680910101E82FEE0F70\\r\\n:1004C000FF0BE80FF11D87FDFA95E55FFE4F808178\\r\\n:1004D00086FFCACF80910101E82FEE0FFF0BE80FD6\\r\\n:1004E000F11D87FDFA95E55FFE4F808160E08F7317\\r\\n:1004F0000E940C01B9CF80918400909185002091D9\\r\\n:100500000101E22FEE0FFF0BE20FF11D27FDFA951F\\r\\n:10051000E55FFE4F21813281820F931F9093890006\\r\\n:100520008093880080910101E82FEE0FFF0BE80F08\\r\\n:10053000F11D87FDFA95E55FFE4F808186FF10C0B3\\r\\n:1005400080910101E82FEE0FFF0BE80FF11D87FDF1\\r\\n:10055000FA95E55FFE4F808161E08F730E940C0188\\r\\n:10056000FF91EF91DF91CF91BF91AF919F918F91CB\\r\\n:100570007F916F915F914F913F912F910F900FBE9F\\r\\n:100580000F901F90189580E49CE9909389008093C8\\r\\n:1005900088008FEF80930101E3CF789484B5826067\\r\\n:1005A00084BD84B5816084BD85B5826085BD85B517\\r\\n:1005B000816085BD80916E00816080936E00109295\\r\\n:1005C0008100809181008260809381008091810010\\r\\n:1005D00081608093810080918000816080938000A1\\r\\n:1005E0008091B10084608093B1008091B0008160FF\\r\\n:1005F0008093B00080917A00846080937A0080912B\\r\\n:100600007A00826080937A0080917A008160809382\\r\\n:100610007A0080917A00806880937A001092C100FD\\r\\n:1006200080912F018C3008F069C0E9E9F0E0249155\\r\\n:10063000E5E8F0E08491882399F090E0880F991F15\\r\\n:10064000FC01E859FF4FA591B491FC01EE58FF4F12\\r\\n:10065000859194918FB7F894EC91E22BEC938FBF36\\r\\n:1006600040912F01842F90E0FC01EE0FFF1FE80F57\\r\\n:10067000F91FE55FFE4F8081807C89608083109246\\r\\n:100680003001109231012CE0842F622F0E94B503BB\\r\\n:10069000829FC001112430E020E0BC01620F731F73\\r\\n:1006A000FB01EE0FFF1FE60FF71FE55FFE4F5081C6\\r\\n:1006B00056FD16C02F5F3F4F2C30310571F74C307F\\r\\n:1006C00078F41092800082E080938100109285007F\\r\\n:1006D00010928400B19A80916F00826080936F00C5\\r\\n:1006E000E0912F018E2F90E0FC01EE0FFF1FE80F2D\\r\\n:1006F000F91FE55FFE4F80818064808300E010E099\\r\\n:10070000D0E0C0E0CE010E9487000E948B012196BC\\r\\n:10071000C53BD105B9F7C4EBD0E0CE010E948700FC\\r\\n:100720000E948B012197C8F70115110549F30E941A\\r\\n:100730000000E6CF809100018C3098F491E0980F92\\r\\n:100740009093000180932F0190E0FC01EE0FFF1FBA\\r\\n:10075000E80FF91FE55FFE4F88EB9BE092838183F2\\r\\n:1007600008958FEF80932F010895991B79E004C0BD\\r\\n:10077000991F961708F0961B881F7A95C9F78095E0\\r\\n:100780000895EE0FFF1F0590F491E02D0994052EBA\\r\\n:1007900097FB1EF400940E94DE0357FD07D00E94D1\\r\\n:1007A000F90307FC03D04EF40C94DE0350954095FA\\r\\n:1007B000309521953F4F4F4F5F4F0895909580950D\\r\\n:1007C000709561957F4F8F4F9F4F08950E941B0436\\r\\n:1007D000A59F900DB49F900DA49F800D911D112495\\r\\n:1007E0000895B7FF0C94E6030E94E603821B930B67\\r\\n:1007F0000895A1E21A2EAA1BBB1BFD010DC0AA1F62\\r\\n:10080000BB1FEE1FFF1FA217B307E407F50720F079\\r\\n:10081000A21BB30BE40BF50B661F771F881F991FF4\\r\\n:100820001A9469F760957095809590959B01AC013D\\r\\n:10083000BD01CF010895A29FB001B39FC001A39F46\\r\\n:10084000700D811D1124911DB29F700D811D112409\\r\\n:08085000911D0895F894FFCFFB\\r\\n:00000001FF\\r\\n"
      }
    },
    {
      "kind": "Servo",
      "x": 400,
      "y": 150,
      "turns": 0
    }
  ],
  "nets": [
    {
      "pins": [
        {
          "element": 0,
          "pin": "D9"
        },
        {
          "element": 1,
          "pin": "SIG"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "5V"
        },
        {
          "element": 1,
          "pin": "+5V"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 1,
          "pin": "GND"
        },
        {
          "element": 0,
          "pin": "GND"
        }
      ]
    }
  ]
}
`,tt=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "description": "Thermometer\\nA TMP36 on A0, read against the chip's own 1.1 V: one more LED for every 5 °C from 20 °C.",
  "elements": [
    {
      "kind": "Uno",
      "x": 200,
      "y": 300,
      "turns": 0,
      "program": {
        "name": "Thermometer",
        "hex": ":100000000C945C000C9479000C9479000C947900A9\\r\\n:100010000C9479000C9479000C9479000C9479007C\\r\\n:100020000C9479000C9479000C9479000C9479006C\\r\\n:100030000C9479000C9479000C9479000C9479005C\\r\\n:100040000C94AE000C9479000C9479000C94790017\\r\\n:100050000C9479000C9479000C9479000C9479003C\\r\\n:100060000C9479000C947900000000002400270013\\r\\n:100070002A0000000000250028002B0004040404CE\\r\\n:100080000404040402020202020203030303030342\\r\\n:10009000010204081020408001020408102001021F\\r\\n:1000A00004081020000000080002010000030407FB\\r\\n:1000B000000000000000000011241FBECFEFD8E0B8\\r\\n:1000C000DEBFCDBF11E0A0E0B1E0E0E1F5E002C0AD\\r\\n:1000D00005900D92A230B107D9F721E0A2E0B1E07E\\r\\n:1000E00001C01D92AB30B207E1F70E94F6000C94FC\\r\\n:1000F00086020C940000CF93DF9300D000D0CDB7E0\\r\\n:10010000DEB73FB7F8948091070190910801A09164\\r\\n:100110000901B0910A0126B5A89B05C02F3F19F02F\\r\\n:100120000196A11DB11D3FBFBA2FA92F982F882777\\r\\n:10013000820F911DA11DB11D9C01AD0162E0220F36\\r\\n:10014000331F441F551F6A95D1F7CA01B9010F909B\\r\\n:100150000F900F900F90DF91CF9108951F921FB6CF\\r\\n:100160001F9211242F933F938F939F93AF93BF932D\\r\\n:100170008091030190910401A0910501B0910601C5\\r\\n:100180003091020123E0230F2D3750F50196A11D78\\r\\n:10019000B11D209302018093030190930401A09369\\r\\n:1001A0000501B09306018091070190910801A0918B\\r\\n:1001B0000901B0910A010196A11DB11D80930701AB\\r\\n:1001C00090930801A0930901B0930A01BF91AF91E8\\r\\n:1001D0009F918F913F912F911F901FBE1F90189557\\r\\n:1001E00026E8230F0296A11DB11DD3CF00D000D069\\r\\n:1001F000CDB7DEB7789484B5826084BD84B5816064\\r\\n:1002000084BD85B5826085BD85B5816085BD8091E1\\r\\n:100210006E00816080936E00109281008091810059\\r\\n:100220008260809381008091810081608093810051\\r\\n:10023000809180008160809380008091B100846013\\r\\n:100240008093B1008091B00081608093B000809174\\r\\n:100250007A00846080937A0080917A008260809333\\r\\n:100260007A0080917A00816080937A0080917A0090\\r\\n:10027000806880937A001092C10083E0809300012F\\r\\n:1002800022E030E0F901E057FF4F4491F901E458D2\\r\\n:10029000FF4F8491882399F090E0880F991FFC010B\\r\\n:1002A000E859FF4FA591B491FC01EE58FF4F85919D\\r\\n:1002B00094918FB7F894EC91E42BEC938FBF2F5F60\\r\\n:1002C0003F4F27303105F1F6FAE04F2E512C612CCB\\r\\n:1002D000712C8091000120E4829FC0011124809341\\r\\n:1002E0007C0080917A00806480937A0080917A000B\\r\\n:1002F00086FDFCCFA0917800B09179002CE434E029\\r\\n:100300000E944E0220E034E040E050E00E942F02C4\\r\\n:10031000DA01C901845F9140A109B10989839A83F7\\r\\n:10032000AB83BC83BC01CD01A30192010E942F02CB\\r\\n:1003300082E090E0E4E1CE2ED12CE12CF12CFC0106\\r\\n:10034000EC55FF4F7491FC01E057FF4F6491FC01A5\\r\\n:10035000E458FF4FE491EE2321F1772371F07430DC\\r\\n:1003600009F46AC008F059C0723009F46DC07330E6\\r\\n:1003700009F45CC0713009F463C0F0E0EE0FFF1FB8\\r\\n:10038000EE58FF4FA591B4917FB7F894EC912C15DE\\r\\n:100390003D054E055F050CF060C060956E236C93C3\\r\\n:1003A0007FBFA5E0CA0ED11CE11CF11C019687306D\\r\\n:1003B000910509F0C4CF0E947B006B017C0184E6AB\\r\\n:1003C000882E912CA12CB12C0E947B0069837A830A\\r\\n:1003D0008B839C83DC01CB018C199D09AE09BF097D\\r\\n:1003E000883E9340A105B10578F3B1E08B1A9108DE\\r\\n:1003F000A108B10828EEC20E23E0D21EE11CF11CB8\\r\\n:1004000081149104A104B104F9F680E090E0892BF5\\r\\n:1004100009F45FCF0E9400005CCF7730C1F07830E4\\r\\n:1004200009F0ABCF7091B0007F7D14C07091800057\\r\\n:100430007F7770938000A1CF709180007F7DF9CF8E\\r\\n:1004400074B57F7774BD99CF74B57F7DFBCF709104\\r\\n:10045000B0007F777093B00090CF6E2BA0CF052EA9\\r\\n:1004600097FB1EF400940E94460257FD07D00E949D\\r\\n:10047000550207FC03D04EF40C944602509540956B\\r\\n:10048000309521953F4F4F4F5F4F08959095809540\\r\\n:10049000709561957F4F8F4F9F4F08950E9477020F\\r\\n:1004A000B7FF0895821B930B0895A1E21A2EAA1B91\\r\\n:1004B000BB1BFD010DC0AA1FBB1FEE1FFF1FA21714\\r\\n:1004C000B307E407F50720F0A21BB30BE40BF50B11\\r\\n:1004D000661F771F881F991F1A9469F7609570959A\\r\\n:1004E000809590959B01AC01BD01CF010895A29F1D\\r\\n:1004F000B001B39FC001A39F700D811D1124911DF8\\r\\n:10050000B29F700D811D1124911D0895F894FFCFA5\\r\\n:020510000100E8\\r\\n:00000001FF\\r\\n"
      }
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 220.0
        }
      },
      "x": 370,
      "y": 80,
      "turns": 0
    },
    {
      "kind": {
        "Led": {
          "colour": "Green"
        }
      },
      "x": 460,
      "y": 80,
      "turns": 0
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 220.0
        }
      },
      "x": 370,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "Led": {
          "colour": "Green"
        }
      },
      "x": 460,
      "y": 140,
      "turns": 0
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 220.0
        }
      },
      "x": 370,
      "y": 200,
      "turns": 0
    },
    {
      "kind": {
        "Led": {
          "colour": "White"
        }
      },
      "x": 460,
      "y": 200,
      "turns": 0
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 220.0
        }
      },
      "x": 370,
      "y": 260,
      "turns": 0
    },
    {
      "kind": {
        "Led": {
          "colour": "Red"
        }
      },
      "x": 460,
      "y": 260,
      "turns": 0
    },
    {
      "kind": {
        "Resistor": {
          "ohms": 220.0
        }
      },
      "x": 370,
      "y": 320,
      "turns": 0
    },
    {
      "kind": {
        "Led": {
          "colour": "Red"
        }
      },
      "x": 460,
      "y": 320,
      "turns": 0
    },
    {
      "kind": {
        "Tmp36": {
          "celsius": 27.0
        }
      },
      "x": 370,
      "y": 460,
      "turns": 0
    }
  ],
  "nets": [
    {
      "pins": [
        {
          "element": 0,
          "pin": "D2"
        },
        {
          "element": 1,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 1,
          "pin": "b"
        },
        {
          "element": 2,
          "pin": "anode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "D3"
        },
        {
          "element": 3,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 3,
          "pin": "b"
        },
        {
          "element": 4,
          "pin": "anode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "D4"
        },
        {
          "element": 5,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 5,
          "pin": "b"
        },
        {
          "element": 6,
          "pin": "anode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "D5"
        },
        {
          "element": 7,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 7,
          "pin": "b"
        },
        {
          "element": 8,
          "pin": "anode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "D6"
        },
        {
          "element": 9,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 9,
          "pin": "b"
        },
        {
          "element": 10,
          "pin": "anode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "GND"
        },
        {
          "element": 11,
          "pin": "GND"
        },
        {
          "element": 2,
          "pin": "cathode"
        },
        {
          "element": 4,
          "pin": "cathode"
        },
        {
          "element": 6,
          "pin": "cathode"
        },
        {
          "element": 8,
          "pin": "cathode"
        },
        {
          "element": 10,
          "pin": "cathode"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "5V"
        },
        {
          "element": 11,
          "pin": "+Vs"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "A0"
        },
        {
          "element": 11,
          "pin": "Vout"
        }
      ]
    }
  ]
}
`,nt=`{
  "schema": 1,
  "note": "Educational simulation — not accurate enough to design from",
  "description": "Two pots\\nA0 and A1 read one after the other and printed side by side: one converter, switched between them.",
  "elements": [
    {
      "kind": "Uno",
      "x": 200,
      "y": 300,
      "turns": 0,
      "program": {
        "name": "TwoPots",
        "hex": ":100000000C9435000C945D000C945D000C945D0024\\r\\n:100010000C945D000C945D000C945D000C945D00EC\\r\\n:100020000C945D000C945D000C945D000C945D00DC\\r\\n:100030000C945D000C945D000C945D000C945D00CC\\r\\n:100040000C940E020C945D000C947C020C945602ED\\r\\n:100050000C945D000C945D000C945D000C945D00AC\\r\\n:100060000C945D000C945D00B40311241FBECFEF0F\\r\\n:10007000D8E0DEBFCDBF11E0A0E0B1E0E6E1F8E0FE\\r\\n:1000800002C005900D92A032B107D9F721E0A0E29D\\r\\n:10009000B1E001C01D92A63CB207E1F710E0C5E354\\r\\n:1000A000D0E004C02197FE010E940304C433D107AD\\r\\n:1000B000C9F70E94A9020C9409040C9400008E5008\\r\\n:1000C000806480937C0080917A00806480937A00C1\\r\\n:1000D00080917A0086FDFCCF809178009091790024\\r\\n:1000E0000895CF93DF9300D000D0CDB7DEB73FB7F0\\r\\n:1000F000F8948091250190912601A0912701B0915B\\r\\n:10010000280126B5A89B05C02F3F19F00196A11D17\\r\\n:10011000B11D3FBFBA2FA92F982F8827820F911D9D\\r\\n:10012000A11DB11D9C01AD0162E0220F331F441FD0\\r\\n:10013000551F6A95D1F7CA01B9010F900F900F9022\\r\\n:100140000F90DF91CF910895CF92DF92EF92FF92BF\\r\\n:100150000F931F93CF93DF937C016A0110E000E0BF\\r\\n:100160000C151D0581F0EB01D701ED91FC9101907B\\r\\n:10017000F081E02D6991C7010995892B21F00F5F6E\\r\\n:100180001F4FBE01EDCFC801DF91CF911F910F919D\\r\\n:10019000FF90EF90DF90CF900895FC01538D448D38\\r\\n:1001A000252F30E0842F90E0821B930B541710F022\\r\\n:1001B000CF96089501970895FC01918D828D98172F\\r\\n:1001C00061F0A28DAE0FBF2FB11D5D968C91928D07\\r\\n:1001D0009F5F9F73928F90E008958FEF9FEF089538\\r\\n:1001E000FC01918D828D981731F0828DE80FF11D01\\r\\n:1001F000858D90E008958FEF9FEF0895FC01818D2C\\r\\n:10020000228D90E0805C9F4F821B91098F7399270C\\r\\n:10021000089589E291E00E94FE009C0181E0232B79\\r\\n:1002200009F480E0089580E090E0892B29F00E9495\\r\\n:10023000090181110C9400000895FC01A48DA80F00\\r\\n:10024000B92FB11DA35ABF4F2C91848D90E0019618\\r\\n:100250008F739927848FA689B7892C93A089B18938\\r\\n:100260008C91837080648C93938D848D981306C0D9\\r\\n:100270000288F389E02D80818F7D80830895EF923D\\r\\n:10028000FF920F931F93CF93DF93EC01F62E81E043\\r\\n:10029000888F9B8D8C8D98131AC0E889F98980818D\\r\\n:1002A00085FF15C09FB7F894EE89FF896083E889C0\\r\\n:1002B000F98980818370806480839FBF81E090E0B2\\r\\n:1002C000DF91CF911F910F91FF90EF9008950B8DCB\\r\\n:1002D00010E00F5F1F4F0F731127E02E8C8D8E11D2\\r\\n:1002E0000CC00FB607FCFACFE889F989808185FF39\\r\\n:1002F000F5CFCE010E941D01F1CFEB8DEC0FFD2F4C\\r\\n:10030000F11DE35AFF4FF0829FB7F8940B8FEA89F3\\r\\n:10031000FB8980818062D0CFCF93DF93EC01888D01\\r\\n:100320008823B9F0AA89BB89E889F9898C9185FD70\\r\\n:1003300003C0808186FD0DC00FB607FCF7CF8C91FE\\r\\n:1003400085FFF2CF808185FFEDCFCE010E941D0198\\r\\n:10035000E9CFDF91CF910895BC01009761F0FC01D6\\r\\n:1003600001900020E9F73197AF01481B590B89E252\\r\\n:1003700091E00C94A40090E080E008954F925F9289\\r\\n:100380006F927F92AF92BF92CF92DF92EF92FF92E5\\r\\n:100390000F931F93CF93DF93CDB7DEB7A1970FB61F\\r\\n:1003A000F894DEBF0FBECDBF6B017C0119A25E01C8\\r\\n:1003B00081E2A80EB11C8AE0482E512C612C712CD0\\r\\n:1003C000B601C701A30192010E94E103922F832F7E\\r\\n:1003D00086019701C92ED82E7A01605DF5016293DE\\r\\n:1003E0005F010A3011052105310550F7CF010E9448\\r\\n:1003F000AC01A1960FB6F894DEBF0FBECDBFDF9162\\r\\n:10040000CF911F910F91FF90EF90DF90CF90BF9011\\r\\n:10041000AF907F906F905F904F9008951F921FB69E\\r\\n:100420001F9211242F933F938F939F93AF93BF936A\\r\\n:100430008091210190912201A0912301B09124018A\\r\\n:100440003091200123E0230F2D3750F50196A11D97\\r\\n:10045000B11D209320018093210190932201A0934C\\r\\n:100460002301B09324018091250190912601A09150\\r\\n:100470002701B09128010196A11DB11D809325018E\\r\\n:1004800090932601A0932701B0932801BF91AF91CB\\r\\n:100490009F918F913F912F911F901FBE1F90189594\\r\\n:1004A00026E8230F0296A11DB11DD3CF1F920F92F4\\r\\n:1004B0000FB60F9211242F933F934F935F936F9337\\r\\n:1004C0007F938F939F93AF93BF93EF93FF9389E2B3\\r\\n:1004D00091E00E941D01FF91EF91BF91AF919F911B\\r\\n:1004E0008F917F916F915F914F913F912F910F90DD\\r\\n:1004F0000FBE0F901F9018952F932FB72F938F93A8\\r\\n:100500009F93EF93FF93E0913901F0913A0180813D\\r\\n:10051000E0913F01F091400182FD19C090818091EE\\r\\n:1005200042018F5F8F7320914301821741F0E09168\\r\\n:100530004201F0E0E75DFE4F958F80934201FF910D\\r\\n:10054000EF919F918F912F912FBF2F9118958081BF\\r\\n:10055000F6CF00D000D0CDB7DEB7789484B58260F6\\r\\n:1005600084BD84B5816084BD85B5826085BD85B557\\r\\n:10057000816085BD80916E00816080936E001092D5\\r\\n:100580008100809181008260809381008091810050\\r\\n:1005900081608093810080918000816080938000E1\\r\\n:1005A0008091B10084608093B1008091B00081603F\\r\\n:1005B0008093B00080917A00846080937A0080916B\\r\\n:1005C0007A00826080937A0080917A0081608093C3\\r\\n:1005D0007A0080917A00806880937A001092C1003E\\r\\n:1005E000E0913901F0913A0182E08083E091350198\\r\\n:1005F000F09136011082E0913701F09138018FECD3\\r\\n:10060000808310924101E0913D01F0913E0186E02E\\r\\n:100610008083E0913B01F0913C0180818061808387\\r\\n:10062000E0913B01F0913C01808188608083E09102\\r\\n:100630003B01F0913C01808180688083E0913B0127\\r\\n:10064000F0913C0180818F7D80838EE00E945F006D\\r\\n:100650006C018FE00E945F007C0182E191E00E94CA\\r\\n:10066000AC01C6010D2C000CAA0BBB0B89839A832D\\r\\n:10067000AB83BC83D7FE6EC06DE289E291E00E943D\\r\\n:100680003F0189819A81AB81BC81B095A0959095FD\\r\\n:1006900081959F4FAF4FBF4FBC01CD010E94BE015E\\r\\n:1006A00086E191E00E94AC01C7010F2C000CAA0B5F\\r\\n:1006B000BB0B89839A83AB83BC83F7FE50C06DE28A\\r\\n:1006C00089E291E00E943F0189819A81AB81BC81DE\\r\\n:1006D000B095A095909581959F4FAF4FBF4FBC01AE\\r\\n:1006E000CD010E94BE018DE191E00E94AC010E940B\\r\\n:1006F00071006B017C0188EC882E912CA12CB12C0F\\r\\n:100700000E947100DC01CB018C199D09AE09BF0963\\r\\n:10071000883E9340A105B10598F391E0891A9108AC\\r\\n:10072000A108B108A8EECA0EA3E0DA1EE11CF11C74\\r\\n:1007300081149104A104B10419F780E090E0892BA1\\r\\n:1007400009F483CF0E940901882309F47ECF0E9417\\r\\n:1007500000007BCF69817A818B819C819FCF6981E9\\r\\n:100760007A818B819C81BDCFE9E2F1E01382128214\\r\\n:1007700088EE93E0A0E0B0E084839583A683B783FE\\r\\n:1007800084E091E09183808385EC90E09587848775\\r\\n:1007900084EC90E09787868780EC90E0918B808B4B\\r\\n:1007A00081EC90E0938B828B82EC90E0958B848B34\\r\\n:1007B00086EC90E0978B868B118E128E138E148EA2\\r\\n:1007C0000895A1E21A2EAA1BBB1BFD010DC0AA1F92\\r\\n:1007D000BB1FEE1FFF1FA217B307E407F50720F0AA\\r\\n:1007E000A21BB30BE40BF50B661F771F881F991F25\\r\\n:1007F0001A9469F760957095809590959B01AC016E\\r\\n:10080000BD01CF010895EE0FFF1F0590F491E02D7B\\r\\n:060810000994F894FFCFEB\\r\\n:10081600000000003F01A400CD008C01FE00DC00BA\\r\\n:10082600F00041302000202020413120000D0A0038\\r\\n:00000001FF\\r\\n"
      }
    },
    {
      "kind": {
        "Potentiometer": {
          "ohms": 10000.0,
          "wiper": 0.3
        }
      },
      "x": 330,
      "y": 440,
      "turns": 0
    },
    {
      "kind": {
        "Potentiometer": {
          "ohms": 10000.0,
          "wiper": 0.8
        }
      },
      "x": 470,
      "y": 440,
      "turns": 0
    }
  ],
  "nets": [
    {
      "pins": [
        {
          "element": 0,
          "pin": "GND"
        },
        {
          "element": 1,
          "pin": "a"
        },
        {
          "element": 2,
          "pin": "a"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "5V"
        },
        {
          "element": 1,
          "pin": "b"
        },
        {
          "element": 2,
          "pin": "b"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "A0"
        },
        {
          "element": 1,
          "pin": "wiper"
        }
      ]
    },
    {
      "pins": [
        {
          "element": 0,
          "pin": "A1"
        },
        {
          "element": 2,
          "pin": "wiper"
        }
      ]
    }
  ]
}
`,rt=[`First circuits`,`Measuring`,`Charge and time`,`Signals`,`Diodes and transistors`,`Chips`,`Logic`,`Arduino`],it=new Set([`DcSource`,`Cell`,`Resistor`,`Ground`,`Voltmeter`,`Ammeter`]);function at(e){let{elements:t=[]}=JSON.parse(e.document),n=t.map(e=>typeof e.kind==`string`?e.kind:Object.keys(e.kind)[0]);return n.find(e=>!it.has(e))??n.find(e=>e!==`DcSource`&&e!==`Cell`&&e!==`Ground`)??n[0]??`Resistor`}var ot=[{id:`milestone`,group:`First circuits`,title:`A lit LED`,look:`The smallest circuit that does something. Change the resistor and watch it dim.`,document:d},{id:`switched`,group:`First circuits`,title:`A switch`,look:`Click the switch to break the circuit. Everything stops, everywhere at once.`,document:Fe},{id:`held`,group:`First circuits`,title:`Hold to light`,look:`The LED is lit only while you hold the button down. Let go and it goes out at once.`,document:c},{id:`series`,group:`First circuits`,title:`Two in a row`,look:`Two LEDs in one line share one current, so both glow exactly as brightly as each other.`,document:oe},{id:`parallel`,group:`First circuits`,title:`Side by side`,look:`Each LED has a path of its own. Delete one and the other stays lit.`,document:m},{id:`dimmer`,group:`First circuits`,title:`A dimmer`,look:`Turn the knob and the LED dims and brightens: the knob adds resistance to the loop.`,document:ee},{id:`white`,group:`First circuits`,title:`White from three`,look:`Red, green and blue lit together add up to white. The red gets a larger resistor: it needs less voltage.`,document:He},{id:`mixing`,group:`First circuits`,title:`Mixing a colour`,look:`Three knobs, one lamp: turn each to add more or less red, green or blue, and the colour mixes.`,document:f},{id:`relay`,group:`First circuits`,title:`A switch that throws`,look:`Hold the button: the relay’s coil pulls its contact over, and the green LED hands over to the red.`,document:A},{id:`seven`,group:`First circuits`,title:`A seven`,look:`A digit is seven LEDs: feed bars a, b and c, each through its own resistor, and it reads 7.`,document:j},{id:`one`,group:`First circuits`,title:`A one, held`,look:`Hold the button: bars b and c light, and the digit reads 1 for as long as you hold it.`,document:P},{id:`buzzer`,group:`First circuits`,title:`A doorbell`,look:`Hold the button and the buzzer sounds; the resistor keeps a 5 V buzzer safe on 9 V.`,document:J},{id:`upright`,group:`First circuits`,title:`Which way up`,look:`A tilt switch is shut while it stands up. Click it to tip it over and the LED goes out.`,document:ze},{id:`tilted`,group:`First circuits`,title:`A tilt alarm`,look:`Click the tilt switch to tip it over: the ball rolls off its legs and the buzzer sounds.`,document:Re},{id:`hot`,group:`First circuits`,title:`A resistor running hot`,look:`Nine volts across 150 Ω is half a watt in a quarter-watt part. Select it and watch it heat.`,document:l},{id:`fused`,group:`First circuits`,title:`The fuse that saves it`,look:`Shut the switch to short the LED: the fuse blows. Open it again and the LED stays dark until a new fuse.`,document:B},{id:`resetting`,group:`First circuits`,title:`A fuse that resets`,look:`A polyfuse in its place: the short trips it hot. Open the switch and it cools, and conducts again by itself.`,document:V},{id:`thermometer36`,group:`Measuring`,title:`A thermometer chip`,look:`Turn the sensor’s dial: the meter moves 10 mV a degree, 0.75 V at 25 °C and 1 V at 50 °C.`,document:q},{id:`divider`,group:`Measuring`,title:`Two meters`,look:`One circuit read two ways: volts across the lower resistor, amps through the loop.`,document:te},{id:`loop`,group:`Measuring`,title:`None used up`,look:`Three meters around one loop all read the same current: none of it is used up on the way.`,document:u},{id:`sum`,group:`Measuring`,title:`It adds up`,look:`Three meters across three resistors. Add their readings and you get the battery’s 9 V.`,document:Pe},{id:`thermometer`,group:`Measuring`,title:`A thermometer`,look:`Warm the thermistor with its slider and watch the meter fall: 4.5 V at 25 °C, less when hot.`,document:K},{id:`nightlight`,group:`Measuring`,title:`A night light`,look:`Slide the light down on the sensor: in the dark the transistor turns the LED on by itself.`,document:F},{id:`dooralarm`,group:`Measuring`,title:`A door alarm`,look:`The magnet is on the door, 5 mm from the reed. Slide it away — open the door — and the buzzer sounds.`,document:ke},{id:`hallmagnet`,group:`Measuring`,title:`A magnet, felt`,look:`Bring the magnet within 10 mm: the LED lights. It stays lit until the magnet is back past 12 mm.`,document:Ae},{id:`passing`,group:`Measuring`,title:`A light for passers-by`,look:`It settles for a minute first. Then hold the sensor — someone walking by — and the light stays on 5 s after.`,document:je},{id:`nightphoto`,group:`Measuring`,title:`A night light, faster`,look:`The night light with a phototransistor for the LDR: dim the room past about 40 lux and the LED comes on.`,document:I},{id:`luxmeter`,group:`Measuring`,title:`Light on a meter`,look:`A phototransistor passes 2 µA a lux: over 1 kΩ the meter reads 2 mV a lux, in a straight line.`,document:L},{id:`solarmeter`,group:`Measuring`,title:`A solar cell on a meter`,look:`In full sun 40 Ω takes 4 V and 100 mA, the panel’s best. Halve the sun: half the current.`,document:W},{id:`ninemotor`,group:`Measuring`,title:`A motor on a 9 V`,look:`The 9 V sags to 5.6 V under the motor and has under an hour in it; four AAs give 5.7 V for hours.`,document:R},{id:`lasting`,group:`Measuring`,title:`How long will it last?`,look:`A coin cell lights an LED for a day and a half — its label says so. Turn its charge down to see less.`,document:z},{id:`gauge`,group:`Measuring`,title:`A battery gauge`,look:`The knob stands for a battery from flat to 5 V. Each LED is half a volt more: full lights all ten.`,document:U},{id:`lightbar`,group:`Measuring`,title:`A light meter`,look:`An LDR’s divider into an LM3914: slide the light up and the bar climbs, one LED for each step.`,document:H},{id:`charging`,group:`Charge and time`,title:`A capacitor filling`,look:`The scope draws it filling up — fast at first, then slower, and never quite full.`,document:t},{id:`sunlight`,group:`Charge and time`,title:`Sunlight in a capacitor`,look:`A dull day fills 4 700 µF through a diode, as a garden lamp does. At night the diode keeps it full.`,document:G},{id:`fading`,group:`Charge and time`,title:`A fading glow`,look:`No battery: a charged capacitor lights the LED on its own, and the glow fades as it empties.`,document:i},{id:`leaking`,group:`Charge and time`,title:`Holding a charge`,look:`Hold the button to fill it, then let go: the meter slowly drops as the charge leaks away.`,document:a},{id:`coil`,group:`Charge and time`,title:`A slow coil`,look:`Current through a coil cannot jump. The ammeter’s trace climbs, quickly and then slowly.`,document:n},{id:`ringing`,group:`Charge and time`,title:`Back and forth`,look:`Charge sloshes between a capacitor and a coil: the trace swings up and down, dying away.`,document:ae},{id:`blink`,group:`Signals`,title:`A blinking LED`,look:`A square wave pushes one way, then the other. The LED lights only for the forward half.`,document:X},{id:`turns`,group:`Signals`,title:`Taking turns`,look:`Two LEDs facing opposite ways on one signal: one lights on each half of the swing.`,document:re},{id:`lifted`,group:`Signals`,title:`Lifted by a battery`,look:`A battery in series lifts the signal: it swings from 0 to 10 V instead of −5 to +5.`,document:Z},{id:`rounded`,group:`Signals`,title:`Rounded edges`,look:`A capacitor cannot follow a sudden jump: its trace curves where the signal is square.`,document:ne},{id:`rectified`,group:`Signals`,title:`Only the top half`,look:`A diode lets through only the top of each wave: the meter never shows the bottom half.`,document:Q},{id:`smoothed`,group:`Signals`,title:`A smoothed wave`,look:`A big capacitor after the diode fills on each peak and feeds the gaps: the meter barely moves.`,document:Me},{id:`oldsupply`,group:`Signals`,title:`The old power supply`,look:`A sine halved by a transformer, a bridge, 1 000 µF with its ripple on the scope, a 7805: a steady 5 V.`,document:Ne},{id:`clipped`,group:`Signals`,title:`Clipped at the top`,look:`A ±9 V wave through a Zener: the top is cut off flat at 5.1 V, the bottom at a diode drop.`,document:y},{id:`glass`,group:`Signals`,title:`A watch’s digit`,look:`An LCD needs AC: a 32 Hz swing between bars b and c and COM turns them dark — a 1.`,document:w},{id:`antiphase`,group:`Signals`,title:`In step, out of step`,look:`Bars fed COM’s own signal stay clear; bars fed its opposite turn dark. It reads 7.`,document:E},{id:`rms`,group:`Signals`,title:`Same height, paler`,look:`A sine and a square of the same height: the glass answers the RMS, so the sine’s 1 is paler.`,document:D},{id:`oneway`,group:`Diodes and transistors`,title:`One way only`,look:`Two diodes, one each way round. Only the LED behind the one facing forwards lights.`,document:p},{id:`cheaper`,group:`Diodes and transistors`,title:`A diode that costs less`,look:`Either diode blocks a battery put in backwards. The Schottky, right, leaves the load 8.7 V; the 1N4148 8.2.`,document:Oe},{id:`zener`,group:`Diodes and transistors`,title:`A chip-free 5 V`,look:`A resistor and a reversed Zener hold the meter just under 5.1 V, with no regulator at all.`,document:b},{id:`transistor`,group:`Diodes and transistors`,title:`A small current`,look:`A tiny current into the transistor’s base lets a far bigger one through the LED. Flip the switch.`,document:Be},{id:`gate`,group:`Diodes and transistors`,title:`Switched by a voltage`,look:`A square wave on a MOSFET’s gate blinks the LED, and the meter on the gate reads a few picoamps.`,document:x},{id:`remembers`,group:`Diodes and transistors`,title:`A gate that remembers`,look:`Press once and let go: the gate keeps its charge, so the LED stays lit for seconds.`,document:S},{id:`toohot`,group:`Diodes and transistors`,title:`Too much for a 2N7000`,look:`Switch on: both motors run, but the 2N7000 on the right heads past 150 °C. The IRLZ44N stays cool.`,document:we},{id:`darlington`,group:`Diodes and transistors`,title:`A Darlington on a motor`,look:`Hold the button: under a milliamp into the TIP120’s base runs the motor’s half an amp.`,document:Te},{id:`fan`,group:`Diodes and transistors`,title:`A fan for the heat`,look:`Warm the thermistor past 50 °C: a MOSFET closes a relay, and the relay starts the motor.`,document:Y},{id:`clicking`,group:`Diodes and transistors`,title:`A relay that clicks`,look:`A transistor lets a weak signal drive a relay coil it could never drive on its own.`,document:C},{id:`kick`,group:`Diodes and transistors`,title:`A kick from the coil`,look:`Each time the transistor lets go, the trace spikes far past 9 V: the coil’s current has nowhere to go.`,document:O},{id:`caught`,group:`Diodes and transistors`,title:`A diode that catches it`,look:`The same relay with a diode across its coil: the trace stops at 9 V, where the kick used to be.`,document:k},{id:`regulated`,group:`Chips`,title:`A steady 5 V`,look:`Change the battery and the meter does not move — until the battery is too low to feed it.`,document:ie},{id:`dropout`,group:`Chips`,title:`Not enough volts`,look:`A 12 V regulator on a 9 V battery gives 7 V: it always needs two volts more than it makes.`,document:r},{id:`knob317`,group:`Chips`,title:`Any voltage from a knob`,look:`Turn the knob: an LM317 holds its output anywhere from 1.25 V to about 9 V, whatever the load.`,document:Ee},{id:`constant`,group:`Chips`,title:`An LED held at 20 mA`,look:`Make the battery 12 V: the meter stays at 20 mA. One 62 Ω resistor sets it: 1.25 V ÷ 62 Ω.`,document:De},{id:`threevolts`,group:`Chips`,title:`A 3.3 V supply`,look:`The same job at 3.3 V, the voltage most modern chips run on — from the same 9 V battery.`,document:Ie},{id:`heatsink`,group:`Chips`,title:`A regulator too hot`,look:`Dropping 7 V to run a motor, a 7805 with no heatsink heads past 125 °C. Select it and watch.`,document:o},{id:`threshold`,group:`Chips`,title:`A knob that decides`,look:`Turn the knob slowly. The LED does not dim: it snaps on at one exact point.`,document:Le},{id:`hotfan`,group:`Chips`,title:`A fan when it’s hot`,look:`Warm the sensor past 30 °C: the comparator lets go, the MOSFET pulls the relay in and the fan starts.`,document:s},{id:`decoded`,group:`Chips`,title:`Switches to a number`,look:`Four switches make a number in binary, and a decoder draws it on the digit. Past nine, it goes blank.`,document:M},{id:`counting`,group:`Chips`,title:`A digit that counts`,look:`A 555 ticks about one and a half times a second, and a counter adds one to the digit on every tick.`,document:N},{id:`hundred`,group:`Chips`,title:`Counting to 99`,look:`A second counter, clocked by the first one’s carry, counts the tens: 00 to 99, then round again.`,document:T},{id:`astable`,group:`Chips`,title:`A 555 that blinks`,look:`Two resistors and a capacitor set the pace: the LED blinks about one and a half times a second.`,document:h},{id:`servo`,group:`Chips`,title:`A servo on a knob`,look:`A 555 sends a pulse every 20 ms, 1 to 2 ms long: turn the knob and the horn follows.`,document:$},{id:`twoway`,group:`Chips`,title:`A motor either way`,look:`Two switches set the motor’s two ends: one up, it turns; the other, it turns back; both alike, it stops.`,document:se},{id:`isolated`,group:`Chips`,title:`Across a gap`,look:`Hold the button: the 5 V side lights a hidden LED, and the 9 V side’s LED comes on. No wire joins them.`,document:ce},{id:`byhand`,group:`Chips`,title:`Eight LEDs by hand`,look:`Left to right: data, shift, latch. Hold data, tap shift for a 1; latch shows all eight. These never bounce.`,document:le},{id:`organ`,group:`Chips`,title:`A one-note organ`,look:`A 555 plays an A, 440 Hz, on a passive buzzer. Turn the knob and the note slides down.`,document:_},{id:`siren`,group:`Chips`,title:`A 555 siren`,look:`A slow 555 nudges a fast one, so the buzzer swaps between two notes, high and low, like a siren.`,document:v},{id:`oneshot`,group:`Chips`,title:`A timed light`,look:`Press the button once: the LED stays lit for about a second, then goes out on its own.`,document:g},{id:`twice`,group:`Chips`,title:`Twice as much`,look:`Turn the knob: the right meter reads twice the left, until the op-amp runs out of battery.`,document:Ve},{id:`buffer`,group:`Chips`,title:`A divider that holds`,look:`Two dividers, the same load on each. Loaded directly it sags; through an op-amp it holds.`,document:e},{id:`and`,group:`Logic`,title:`AND: both at once`,look:`The LED lights only while both switches are on — one alone does nothing.`,document:de},{id:`or`,group:`Logic`,title:`OR: either will do`,look:`Either switch lights the LED, and so do both. Only both off puts it out.`,document:fe},{id:`xor`,group:`Logic`,title:`XOR: one or the other`,look:`One switch on lights the LED; turn the second on as well and it goes out.`,document:pe},{id:`not`,group:`Logic`,title:`NOT: the opposite`,look:`The LED is lit while the switch is off, and goes out when it is on.`,document:me},{id:`nand`,group:`Logic`,title:`NAND: anything but both`,look:`Lit for every pair of switches except both on — an AND with its answer turned round.`,document:ue},{id:`universal`,group:`Logic`,title:`A NAND makes everything`,look:`Only NANDs: red is NOT left, green is AND, blue is OR — on less resistance, as blue needs 3 V.`,document:ge},{id:`latch`,group:`Logic`,title:`Two gates that remember`,look:`Red is lit. Press the left button: green comes on and stays when you let go. The right one swaps back.`,document:he},{id:`blinker`,group:`Logic`,title:`A gate that blinks`,look:`One Schmitt inverter, a resistor and a capacitor: the LED blinks by itself, about once a second.`,document:_e},{id:`bounces`,group:`Logic`,title:`A button that bounces`,look:`Press once: the count jumps by three, and by two more as you let go. That is the button bouncing.`,document:ve},{id:`debounced`,group:`Logic`,title:`A button that does not`,look:`The same bouncy button, through a capacitor and a Schmitt inverter: each press counts one.`,document:ye},{id:`chaser`,group:`Logic`,title:`A running light`,look:`A 555 clocks a decade counter: ten LEDs light one after another, top to bottom, and round.`,document:be},{id:`countfour`,group:`Logic`,title:`Counting to four`,look:`Q4 is wired to reset, so the moment it would light the count starts again: four LEDs, never five.`,document:xe},{id:`toggle`,group:`Logic`,title:`One button, on and off`,look:`Press once and the LED stays on; press again and it goes off. A flip-flop remembers which.`,document:Se},{id:`halving`,group:`Logic`,title:`Halve a frequency`,look:`Red blinks with the 555; green, on one flip-flop, half as often; blue, on the next, half again.`,document:Ce},{id:`uno-blink`,group:`Arduino`,title:`Blink`,look:`A real Arduino program: D13 high for a second, low for a second, and the LED with it.`,document:Ge},{id:`uno-button`,group:`Arduino`,title:`A button on D2`,look:`Hold the button: D2 reads low through its own pull-up, and the program lights D13.`,document:Ke},{id:`uno-traffic`,group:`Arduino`,title:`Traffic light`,look:`Green for two seconds, the middle light for half a second, red for two: three pins and delay().`,document:$e},{id:`uno-fade`,group:`Arduino`,title:`Fade`,look:`analogWrite switches D9 and D10 on and off 490 times a second; one LED fades into the other.`,document:Je},{id:`uno-smoothed`,group:`Arduino`,title:`PWM, smoothed`,look:`D9 is on a quarter of the time; 10 kΩ and 1 µF smooth it to about 1.25 V. The scope shows the ripple.`,document:Qe},{id:`uno-sweep`,group:`Arduino`,title:`A servo sweeping`,look:`The Servo library sends D9 a pulse every 20 ms; its length is the angle, and the horn sweeps to 180°.`,document:et},{id:`uno-analog-input`,group:`Arduino`,title:`A pot sets the blink`,look:`analogRead turns the wiper’s 0–5 V into 0–1023; D13 waits that many milliseconds. Turn it.`,document:Ue},{id:`uno-dimmer`,group:`Arduino`,title:`Dimmer`,look:`The pot on A0 read as 0–1023, divided by four, is the LED’s analogWrite on D9.`,document:qe},{id:`uno-night-light`,group:`Arduino`,title:`Night light`,look:`Turn the light down: the LDR on A0 reads past 600 and D13 lights; back up past 500 and it goes out.`,document:Ye},{id:`uno-thermometer`,group:`Arduino`,title:`Thermometer`,look:`A TMP36 read against the chip’s 1.1 V, four times finer than 5 V: an LED for every 5 °C from 20.`,document:tt},{id:`uno-analog-read-serial`,group:`Arduino`,title:`AnalogReadSerial`,look:`A0’s reading in the serial monitor under the board, ten times a second. Turn the pot.`,document:We},{id:`uno-serial-led`,group:`Arduino`,title:`D13 by serial`,look:`Type on or off in the serial monitor and press Send: the program switches D13 and answers.`,document:Xe},{id:`uno-serial-thermometer`,group:`Arduino`,title:`Serial at 115200`,look:`The temperature at 115200 baud; the monitor starts at 9600, so it reads garbage until set to match.`,document:Ze},{id:`uno-two-pots`,group:`Arduino`,title:`Two pots`,look:`A0 and A1 printed side by side: one converter, switched from one pin to the other between readings.`,document:nt}];export{ot as EXAMPLES,rt as EXAMPLE_GROUPS,at as heroOf};