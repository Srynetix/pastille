# 23. Pastille follows the P suite's conventions

- **Status:** accepted
- **Date:** 2026-11-11
- **Ticket:** —
- **Deciders:** Denis Bourge

## Context and problem statement

Pastille is one of the apps of the [P suite](https://github.com/Srynetix/p-suite), with Prism
and Pavé: apps built almost entirely by AI coding agents, which share the way they are made
rather than their code. The suite wrote its conventions down in `SUITE.md` and checks an app
against them with `just audit`. Much of them came from Pastille — the ADRs, the local gate,
`just doctor`, the SPDX line — but the audit found Pastille departing on eight points:

- no `AGENTS.md`, so the rules an agent works under lived only in its local memory;
- `Justfile` where the suite writes `justfile`, and no `check` recipe;
- the commit subjects the release notes depend on were a convention, not a check;
- the release files were published with a `.sha256` only, where the suite signs every file
  with minisign ([suite ADR 0003](https://github.com/Srynetix/p-suite/blob/main/docs/adrs/0003-every-release-signed.md));
- *How it was made* worded differently from the other apps, and one stylesheet without its
  SPDX line, which `just spdx` did not look at.

Which of these does Pastille take?

## Decision

All of them.

1. **`AGENTS.md`** is the contract an agent works under, committed; `CLAUDE.md` imports it.
   It carries what had been learned in sessions and kept outside the repository: the bench
   gate needs mains power, every new part comes with an example.
2. **`justfile`**, and a **`check`** recipe — format, lints, types — that `ci` starts from.
3. **`just check-commits`** (`scripts/check-commits.sh`, the suite's) checks that every commit
   since the last `pastille-v*` tag is Conventional. `just ci` runs it first, and
   `scripts/release.sh` runs it even with `--skip-checks`.
4. **Every file a release publishes is signed with minisign**, with the key
   `~/.minisign/pastille.key` (`PASTILLE_SIGN_KEY`), whose public half is `update-key.pub` at the
   root. `release.sh` refuses to publish without it, and checks every signature it makes.
5. **`just spdx` checks stylesheets and shell scripts too**, as the suite's audit does.

## Options considered

### Take the suite's conventions — chosen

The cost is small, most of it a rename and two scripts, and in return an agent that has
worked on Prism or Pavé finds Pastille's commands, gate and release where it expects them.

### Keep Pastille's ways where they differ — rejected

Pastille's ways were the suite's model, so the departures are few; keeping them would make
the audit report them on every run, or require one ADR per departure, for no gain.

## Consequences

1. **The first release after this one is the first signed.** `pastille-v0.1.0` stays as it was
   published, with its `.sha256` only.
2. **The key has to be made once**, before that release:
   `minisign -G -p update-key.pub -s ~/.minisign/pastille.key`, then `update-key.pub`
   committed. Losing the key means a new one, announced in the release notes.
3. **`just ci` no longer runs `test` as such:** it runs `test-core`, and the app's tests against
   the optimised bundle with the benchmark, which is the stricter form of `test-app`. Running
   `test` too would run the app's tests twice.
4. **ADR 0005 keeps saying `Justfile`:** a record is not rewritten; the file is now spelled
   `justfile`.
5. **ADR 0013's "unsigned"** is about Apple's and Microsoft's signing, which this record does not
   change: the apps are still not notarized.
