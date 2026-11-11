# 14. The gate stays on this machine, now that there is a remote

- **Status:** accepted — revisits ADR 0005, whose premise has changed
- **Date:** 2026-10-01
- **Ticket:** C-106
- **Deciders:** Denis Bourge

## Context and problem statement

ADR 0005 put the checks in a `Justfile` and ran no hosted CI, because there
was no remote: a workflow would have been YAML that never ran. It said GitHub
Actions was "the right answer the moment a second person or a remote appears".

A remote has appeared — `origin` is `github.com/Srynetix/poc-elec-sim`, and
work is pushed to it. A second person has not. So the question ADR 0005 left
open is asked: does the gate move to a hosted runner?

What has changed since ADR 0005 is not only the remote. `just ci` now holds a
benchmark gate (C-15) that measures the solver's cost a frame against a
baseline recorded on this Mac, and that bites both ways — slower fails, and so
does much faster. It is calibrated to one machine, and on that machine only on
mains power: on battery in low-power mode it reads 40 % slow on unchanged code.

## Decision

The gate stays local. `just ci`, run on this Mac on mains power before a
commit, is what says a change is good. The remote is for keeping and sharing
the work, not for checking it. No workflow file is added.

## Options considered

### GitHub Actions running `just ci` — rejected for now

Every push checked, by a machine nobody has to remember to use. Rejected
because the gate's most particular check cannot run there honestly: a hosted
runner is a different machine on every run, so C-15's baseline would either
fail every time or have to be loosened until it no longer catches anything.

### GitHub Actions running everything but C-15 — rejected for now

Keeps the benchmark local and checks the rest on every push. Rejected while
there is one contributor: it is a second definition of the gate to keep in step
with the first, and the failure it guards against — a push nobody checked — is
the same person forgetting either way. It becomes worth it when a push can come
from someone who does not have this machine.

### Keep it local, and say why — chosen

## Consequences

1. **Nothing enforces the gate on a push,** as ADR 0005 already accepted. A
   commit message that says C-15 was not run, and why, is the convention when it
   cannot be (the board records the same).

2. **Revisit when a second person pushes.** Then the second option above is the
   starting point: everything but C-15 on every push, C-15 on the machine it is
   calibrated to.

3. **ADR 0005 stays as written,** with a pointer here: its decision stands, and
   it is the reason that changed.
