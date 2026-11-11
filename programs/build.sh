#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Builds Arduino sketches for the Uno with avr-gcc and Arduino's AVR core,
# with the flags the Arduino IDE uses, without arduino-cli's ctags.
set -euo pipefail
# Usage, from this directory: ./build.sh Blink Button … — each NAME.ino here
# becomes NAME.hex. Needs Arduino's AVR core 1.8.8 (arduino-cli core install
# arduino:avr) and an avr-gcc on the PATH: Arduino's own is Intel-only, so on
# Apple silicon without Rosetta, Homebrew's (brew install osx-cross/avr/avr-gcc).
CORE=~/Library/Arduino15/packages/arduino/hardware/avr/1.8.8/cores/arduino
VAR=~/Library/Arduino15/packages/arduino/hardware/avr/1.8.8/variants/standard
FLAGS=(-mmcu=atmega328p -DF_CPU=16000000L -DARDUINO=10819 -DARDUINO_AVR_UNO -DARDUINO_ARCH_AVR -Os -ffunction-sections -fdata-sections -flto "-I$CORE" "-I$VAR")
CXX=(-std=gnu++11 -fpermissive -fno-exceptions -fno-threadsafe-statics)
mkdir -p build/core
for f in "$CORE"/*.c; do avr-gcc "${FLAGS[@]}" -std=gnu11 -fno-fat-lto-objects -c "$f" -o "build/core/$(basename "$f").o"; done
for f in "$CORE"/*.cpp; do avr-g++ "${FLAGS[@]}" "${CXX[@]}" -fno-fat-lto-objects -c "$f" -o "build/core/$(basename "$f").o"; done
for f in "$CORE"/*.S; do avr-gcc "${FLAGS[@]}" -x assembler-with-cpp -c "$f" -o "build/core/$(basename "$f").o"; done
rm -f build/core.a; avr-gcc-ar rcs build/core.a build/core/*.o
# A library a sketch includes is built with it, from where arduino-cli puts
# one (arduino-cli lib install Servo): its src, and src/avr for this chip.
LIBRARIES=~/Documents/Arduino/libraries
for s in "$@"; do
  { echo '#include <Arduino.h>'; cat "$s.ino"; } > "build/$s.cpp"
  objects=("build/$s.o")
  includes=()
  for library in $(grep -o '#include <[A-Za-z0-9_]*\.h>' "$s.ino" | sed 's/#include <\(.*\)\.h>/\1/'); do
    [ -d "$LIBRARIES/$library/src" ] || continue
    includes+=("-I$LIBRARIES/$library/src")
    for f in "$LIBRARIES/$library"/src/*.cpp "$LIBRARIES/$library"/src/avr/*.cpp; do
      [ -e "$f" ] || continue
      avr-g++ "${FLAGS[@]}" "${CXX[@]}" "-I$LIBRARIES/$library/src" -c "$f" -o "build/$library-$(basename "$f").o"
      objects+=("build/$library-$(basename "$f").o")
    done
  done
  avr-g++ "${FLAGS[@]}" "${CXX[@]}" "${includes[@]+"${includes[@]}"}" -c "build/$s.cpp" -o "build/$s.o"
  avr-gcc -mmcu=atmega328p -Os -flto -fuse-linker-plugin -Wl,--gc-sections "${objects[@]}" build/core.a -lm -o "build/$s.elf"
  avr-objcopy -O ihex -R .eeprom "build/$s.elf" "$s.hex"
  avr-size "build/$s.elf" | tail -1
done
