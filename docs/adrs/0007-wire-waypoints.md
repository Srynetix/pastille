# 7. A wire's route is derived, unless the user has bent it

- **Status:** accepted
- **Date:** 2026-09-18
- **Ticket:** C-43
- **Deciders:** Denis Bourge

## Context and problem statement

Where a wire runs is derived and never stored. `routeNet` takes the positions of a net's
pins and draws an L between them, and that is the only route a wire can have. Two
components whose L happens to cross a third have no recourse — the second finding of
the author's test, M-22, and the one a user hits the moment a circuit outgrows a straight line.

The difficulty is not drawing a bent wire. It is that a wire has two ends that belong to
something else. Components move (C-23) and turn (C-40), and a route the user placed by
hand has to survive both without either forgetting what they asked for or leaving a wire
hanging off the pin it is supposed to join.

There is also a rule the app has kept since M-17 and should not lose: wires are
orthogonal. A right angle is easy to follow with the eye; a diagonal crossing two others
is not, and for an audience that has never read a schematic that legibility is most of
the point.

## Decision

A net owns an ordered list of **waypoints**: interior points its wire passes through, in
world coordinates, stored in the layout half of the core beside the element positions
(ADR 0006).

The route is derived *through* them. A net with no waypoints routes exactly as it does
today, so nothing changes for a wire nobody has touched. A net with waypoints routes
orthogonally from its first pin to the first waypoint, waypoint to waypoint, and from the
last waypoint to its second pin.

**The ends are never stored.** Only the interior is the user's. That single asymmetry is
what answers the question this ticket was left open on: when a component moves or turns,
its waypoints stay exactly where they were put, and the runs between the pins and the
nearest waypoint are re-derived. The bend the user placed to dodge an obstacle stays
dodging it, and the wire stays attached and stays orthogonal.

A waypoint that ends up in line with its neighbours is dropped, so a wire straightened by
hand does not keep invisible bends in it.

## Options considered

### An offset remembered against the derived route — rejected

Keep `routeNet` in charge and let a segment be nudged, storing how far. Cheap, and it
degrades well when a component moves because the route simply re-derives. Rejected
because it is not what was asked for: it can move a segment the router already drew, and
it can never add a bend the router would not have chosen. A wire that has to go around
two obstacles is still impossible.

### Every vertex stored, including the ends — rejected

What KiCad does: the wire is a polyline the user owns outright, and nothing is derived.
Full control, and no asymmetry to explain. Rejected on what happens next: when a pin
moves, the stored end no longer sits on it, so either the wire detaches — which is a
broken drawing — or the end is dragged along and the segment next to it stops being
orthogonal. KiCad can afford that because its audience reads schematics. §4's audience
has never built a circuit.

### Interior waypoints, derived ends — chosen

The user owns the middle of the wire and the app owns the two runs that reach the pins.
Bends survive a component moving because they are not attached to it; the wire survives
because its ends were never stored in the first place; orthogonality survives because
every run is still drawn by the same router as before.

## Consequences

1. **Waypoints belong to the run between a net's first two pins.** A third pin joining a
   net still branches from the first pin the way it does today, derived. A net joining
   three pins therefore has one hand-placed run and the rest automatic. Per-branch
   waypoints are a later ticket if anybody wants them; nobody has asked yet.

2. **Removing a net must forget its waypoints**, exactly as removing an element forgets
   its position. There is a test.

3. **C-04 serializes them with everything else.** A saved circuit that reloads with its
   wires straightened is not a reload.

4. **The breadboard view (B-14) gets its own list.** Waypoints are schematic geometry, in
   the same way positions are, and the breadboard's routing is a different problem with
   different constraints.

5. **Undo covers bending a wire**, and one drag is one step, the same rule a move follows.
