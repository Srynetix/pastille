# Contributing to Pastille

Thank you for wanting to help. Pastille is a small project kept by one person, so a short
issue first saves you work: say what you would like to change and why, and we can agree on
the shape before you write it.

## Reporting a bug

Open an issue with:

- what you did, what you expected, and what happened instead;
- **the circuit**, saved with *Save* (a `.json` file): most simulator bugs are about one
  circuit, and with the file the bug can be seen in a minute;
- where you ran it: the web app (which browser) or the desktop app (which system and which
  release).

A result that differs from a real circuit is worth reporting too, but read the note first:
Pastille is an educational simulation, not accurate enough to design from. A difference that
changes what a beginner would learn is a bug; a few percent off often is not.

A security problem goes through [SECURITY.md](SECURITY.md), not an issue.

## Ideas

Ideas are welcome as issues. What fits best: a part found in beginners' kits, an example that
teaches one thing, a check that catches a mistake people make on a real bench. Pastille stays
aimed at someone building their first circuits, which is why some good ideas will not be
taken.

## Changing the code

### Setting up

Install what the README's [*Build it*](README.md#build-it) section lists, then:

```bash
just doctor   # what this machine is missing
just install
just dev      # the app on http://localhost:5173
```

### Before you open a pull request

**Run `just ci`, and say in the pull request that it passed.** There is no hosted pipeline
([ADR 0014](docs/adrs/0014-the-gate-stays-local.md)): nothing checks a pull request but this
command, which runs formatting, lints, licence checks, every test in the core and in a real
browser, a cold load and an offline run. It also runs benchmarks, so run it on mains power: a
laptop on battery may fail them while the code is fine.

`just check-commits` checks your commit subjects alone, and
`ln -s ../../scripts/check-commits.sh .git/hooks/commit-msg` checks each one as it is made.
`just fmt` fixes formatting; `just test` runs the tests alone while you work.

### Conventions

- **Commits** follow [Conventional Commits](https://www.conventionalcommits.org):
  `feat(core): …`, `fix(app): …`, `docs: …`. The release notes are built from them, so the
  subject is written for someone reading what changed.
- **Every source file** starts with `SPDX-License-Identifier: MIT OR Apache-2.0` in its first
  three lines; the gate checks it.
- **Tests come with the change.** A fixed bug gets a test that fails without the fix.
- **Dependencies** are few on purpose, and only under permissive licences: no copyleft code
  in this repository ([ADR 0001](docs/adrs/0001-permissive-dual-licence.md)). `cargo deny` and
  the licence checks enforce it; ask before adding one.

### Where things are decided

Every technical decision is recorded in [`docs/adrs`](docs/adrs/README.md), with what was
considered and why it lost. Much of what looks odd here was chosen: read the record behind a
part of the code before proposing to change it. A change that goes against a record comes
with a new one.

Some common changes:

- **A new part:** its electrical facts in `crates/core`, its symbol and words in `app`, as
  [ADR 0008](docs/adrs/0008-component-registry.md) describes. Add an example that uses it.
- **A new example:** a circuit saved from the app into `app/src/examples/`, and an entry in
  `app/src/lib/examples.ts` that says what to look at.

## Licence

Unless you state otherwise, any contribution you submit for inclusion is dual-licensed under
MIT or Apache 2.0, as the project is, with no additional terms.
