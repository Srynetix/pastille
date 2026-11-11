# 5. No hosted CI; the checks live in a Justfile

- **Status:** accepted — **replaces the GitHub Actions workflow added under C-02**;
  its premise, no remote, has since changed, and ADR 0014 revisits it (the gate
  stays local, for a different reason)
- **Date:** 2026-09-18
- **Ticket:** C-02
- **Deciders:** Denis Bourge

## Context and problem statement

C-02 asked for the checks to "run on push and block merge on failure", and was first
built as a GitHub Actions workflow. That presumes a hosted forge, a remote, and a
pull-request flow — none of which this project has. It is a personal project with one
contributor and no remote configured.

The real requirement underneath the ticket is narrower: **there is one list of checks,
it is complete, and running it is a single command.** Where that command runs is a
separate question, and answering it early buys a dependency on a service before there is
anything to serve.

## Decision

The checks live in a `Justfile`. `just ci` runs all of them, ordered so the fastest thing
that can fail fails first. No hosted CI for now.

`just doctor` reports whether the toolchain can run the checks at all.

## Options considered

### GitHub Actions — rejected for now

What C-02 originally specified, and the right answer the moment a second person or a
remote appears. Rejected because it cannot run today: with no remote, the workflow is
YAML that has never executed, which is worse than no pipeline — it looks like coverage
and provides none.

### A shell script — rejected

Zero dependencies, and it would work. Rejected because a list of recipes with
descriptions is self-documenting in a way a script is not, and because `just ci` composing
existing recipes keeps one definition per check rather than two.

### `Justfile` — chosen

Recipes are individually runnable (`just clippy` while iterating), compose into the gate,
and `just --list` is the discovery surface. Task, Make and mise would all do; `just` has
the least ceremony.

## Consequences

1. **Nothing enforces the gate.** A hosted pipeline refuses a bad merge; a Justfile waits
   to be run. This is the cost of the decision and it is accepted knowingly: with one
   contributor the failure mode is forgetting, not disagreement. A pre-commit hook is the
   cheap mitigation if forgetting turns out to be common.

2. **The machine is the pipeline.** A missing `cargo-deny` means the licence check
   silently does not run, which is the quiet failure a hosted runner makes impossible.
   `just doctor` exists for that and should be run after any toolchain change.

3. **The checks are portable.** Every recipe is a plain command, so adopting a hosted
   runner later is a few lines calling `just ci`, not a rewrite. Nothing about this
   decision has to be undone — only added to.

4. **C-02's acceptance is rewritten on the board** to match: one complete local gate
   rather than a hosted pipeline blocking merges.
