# 20. The app is called Pastille, everywhere, with no migration

- **Status:** accepted
- **Date:** 2026-11-10
- **Ticket:** —
- **Deciders:** Denis Bourge

## Context and problem statement

The app went by two working names: `poc-elec-sim` in the window title, the manifest and the
desktop bundle, and `elec-sim` (`elecsim` in identifiers) in its header, crates, binaries,
storage keys and exports. Both say what the repository was when it started, a proof of
concept. The app is meant to be published as open source, so it needs a name of its own,
one a search can find and nobody else already holds.

## Decision

**The app is Pastille, a solder pad, the place where a part meets the board. The name
replaces both old ones everywhere, and nothing carries data across from them.**

- Shown as `Pastille`: the window title, the manifest's `name` and `short_name`, the
  desktop `productName` (so `Pastille.app`), the header (`pas·tille`, split as
  `elec·sim` was), the README, and the tool named in the KiCad and circuit exports.
- `pastille` in identifiers: the crates `pastille-core`, `pastille-run` and
  `pastille-desktop`; the wasm module `pastille_core`; the `pastille:` prefix of every
  localStorage key; the service worker's `pastille-` caches; `PASTILLE_SMOKE`; the KiCad
  library nickname; the bundle id `io.github.denisbourge.pastille`.
- The repository is `github.com/Srynetix/pastille`: `repository` in `Cargo.toml`, and the
  source the built app's `programs/README.txt` points to. ADRs before this one keep the
  names they were written with, ADR 0014 the remote's among them.
- The word "pinout", where the code means a part's pins, is not a name and stays.

## Options considered

- **Visible name only**: rename what a user sees and keep `elecsim` underneath, so
  autosaves and installs carry on. Cheaper, but an open-source project would ship with two
  names, one of them only in its code.
- **Everything, with migration**: also copy the old localStorage keys across on first load,
  and keep the old bundle id. Worth it with users. There are none yet besides the author.
- **Pinout**, as the name: chosen first, then dropped. pinout.xyz and many projects already
  use it, so a public project under that name would be hard to find.
- **Picot, Paillasse, Perfwire, Plaquette, Pontet**: other names from the bench. Pastille was
  the most distinctive one that still reads easily in English.

## Consequences

- A circuit autosaved under the old keys is not reopened, and the first-visit hints show
  again. The old service worker caches stay until the browser drops them.
- A new bundle id makes the desktop app a different app to macOS: the old one's data stays
  where it was, and `poc-elec-sim.app` has to be removed by hand.
- A KiCad schematic exported before the rename names its library `elec_sim`, and one
  exported after names it `pastille`.
- The GitHub repository is renamed to match, or those links lead nowhere.
- Before the name is published, check that it is free where the project will live: the
  GitHub repository, crates.io (`pastille-core`, `pastille-run`), npm, a domain. The search
  behind this decision was a web search, not a trademark search.
