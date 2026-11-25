# 24. A sketch is compiled on the desktop, by the machine's own arduino-cli

- **Status:** accepted
- **Date:** 2026-11-22
- **Ticket:** A-08
- **Deciders:** Denis Bourge

## Context and problem statement

The Uno runs real programs (ADR 0017), but until now only as an Intel HEX: one of the bundled
builds, or a `.hex` exported from the Arduino IDE. A beginner who wants to change Blink has to
leave Pastille, compile somewhere else and come back. ADR 0017 left compiling open.

What the choice has to fit:

- **The compiler is GCC**, which is GPL-3.0. ADR 0001 keeps copyleft code out of the
  repository and out of what it ships; the HEX it makes carries Arduino's core, LGPL-2.1, as
  the bundled programs already do (`programs/NOTICE.md`).
- **The browser app works offline** (ADR 0009) and sends nothing anywhere.
- **The circuit file carries the program** (`SavedProgram`): changing what it holds needs the
  author's agreement, and an older file has to open unchanged.
- **Arduino's own avr-gcc and ctags are Intel-only on macOS.** On Apple silicon without
  Rosetta, which is the author's machine, a plain `arduino-cli compile` fails with
  `bad CPU type in executable`; `programs/build.sh` already works around it with Homebrew's
  avr-gcc and no ctags.

## Decision

**The desktop shell compiles, with the `arduino-cli` the user installed; the browser does not
compile.** A `compile_sketch` command writes the sketch to a folder of its own under the
system's temporary folder, runs `arduino-cli compile --fqbn arduino:avr:uno`, and answers the
`.ino.hex` it leaves, or what the compiler said, with the temporary folder taken out of its
paths. GCC is never shipped and never linked: it is another program on the user's machine.
`arduino-cli` is looked for on the `PATH` and where Homebrew puts it, since a desktop app on
macOS is not given the terminal's `PATH`.

**When Arduino's tools cannot start**, the same compilation runs again with two build
properties: `compiler.path` set to the folder of an avr-gcc found on the machine, and
`tools.ctags.path` set to a ctags that finds nothing. That is `build.sh`'s way, and it has
its limit: `arduino-cli` writes no prototypes, so a function has to be declared before it is
called.

**The sketch is the program's, beside its HEX.** `Program` and `SavedProgram` gain an optional
`sketch`; a file without one opens as before, and a program loaded as a HEX saves none. The
HEX is still what runs: the core never compiles, and a circuit opened on a machine without
`arduino-cli`, or in the browser, runs the program it was saved with. `setCompiledProgram`
puts a program and its sketch on an Uno in one step of undo; the element view carries the
sketch back to the inspector, so the app keeps no copy of it.

**The editor is the inspector's**, in the desktop app only: a text area starting from what the
Uno runs (its own sketch, the bundled program's, or an empty `setup` and `loop`), *Compile and
run*, *Open a .ino…*, and the compiler's words under it when it refuses. The browser says
where compiling is.

## Options considered

### A build service the browser and the desktop both call — rejected

It would compile everywhere. Rejected because the user's sketch leaves their machine, the
browser app stops working offline for this, and there would be a server to run and pay for.

### A compiler in WebAssembly — rejected for now

clang's AVR backend is Apache-2.0 with the LLVM exception, so licence-wise it could ship. It
would be tens of megabytes, a toolchain to build and keep up with, and the Arduino core to
bundle as source; a project of its own, against a feature the desktop gets in a few hundred
lines. Revisit if compiling in the browser becomes the point.

### Only the HEX in the file — rejected

It leaves the file format alone, but a reopened circuit could no longer be changed: the
sketch would have to be found again on disk.

### Calling avr-gcc directly, as `build.sh` does — rejected as the default

It would not need `arduino-cli`, but it would mean finding Arduino's core ourselves and
writing no prototypes for every user, where `arduino-cli` writes them wherever its tools run.
It is kept as the fallback above, through `arduino-cli`'s own build properties.

## Consequences

1. **Compiling needs `arduino-cli` and `arduino-cli core install arduino:avr`.** The shell
   says so when it finds neither; nothing is installed for the user.
2. **The circuit file gains an optional `sketch`** under an Uno's `program`; the schema number
   does not change, since every file that opened before still opens the same.
3. **`just desktop-check` runs the shell's tests.** They stand a script in for `arduino-cli`,
   and compile the bundled Blink with the real one where it is installed.
4. **On Apple silicon without Rosetta**, a sketch must declare a function before calling it,
   as the bundled sketches already do.
