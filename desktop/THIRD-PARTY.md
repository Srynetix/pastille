# Third-party code in the desktop build

Pastille is dual-licensed MIT OR Apache-2.0. The desktop build links, unmodified,
a few libraries that the Tauri framework depends on and that are under the
Mozilla Public License 2.0 instead (ADR 0013). Their source is available, unchanged,
from crates.io and from the repositories below; the licence text is at
https://mozilla.org/MPL/2.0/.

| Crate | Version | Source |
|---|---|---|
| cssparser | 0.37.0 | https://crates.io/crates/cssparser/0.37.0 · https://github.com/servo/rust-cssparser |
| cssparser-macros | 0.7.1 | https://crates.io/crates/cssparser-macros/0.7.1 · https://github.com/servo/rust-cssparser |
| selectors | 0.38.0 | https://crates.io/crates/selectors/0.38.0 · https://github.com/servo/stylo |
| dtoa-short | 0.3.5 | https://crates.io/crates/dtoa-short/0.3.5 · https://github.com/upsuper/dtoa-short |
| option-ext | 0.2.0 | https://crates.io/crates/option-ext/0.2.0 · https://github.com/soc/option-ext |

Every other library in the build is under MIT, Apache-2.0, BSD, ISC, Zlib or
Unicode-3.0, which `cargo deny check` in this directory enforces.

## Arduino's AVR core, in the example programs

The example circuits with an Arduino Uno carry compiled programs, and each one
contains Arduino's AVR core (ArduinoCore-avr 1.8.8), statically linked as every
Arduino program is; `Sweep` also contains the Servo library 1.3.0. Both are under
the GNU Lesser General Public License 2.1 or later. The build ships their notice
and the licence text in `programs/` beside the app, and the repository's
`programs/` holds the sketches and the script that rebuilds them, against that
core or a modified one. See `programs/NOTICE.md`.

## System libraries, in the Linux AppImage

The AppImage is the one package that carries libraries of the system it was built
on, so that it runs on distributions that lack them: GTK 3, WebKitGTK 4.1, GLib and
the libraries they need, as Ubuntu 22.04 ships them, unmodified and dynamically
linked. Most are under the GNU Lesser General Public License, version 2 or 2.1 or
later; each one's licence and source are those of its Ubuntu 22.04 package, at
https://packages.ubuntu.com/jammy/ under the library's name. They can be replaced
by running the app against the system's own instead: the `.deb` and the Arch package do
exactly that, and carry none of them.

The Windows installer carries none either: the web view is Microsoft's WebView2,
which the installer downloads from Microsoft if Windows lacks it.
