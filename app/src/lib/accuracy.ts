// SPDX-License-Identifier: MIT OR Apache-2.0

// What the app says this simulation is, and where it says it.
//
// C-26. Half of this shipped with M-13, which put the sentence in the status
// bar and left the placement to be decided; this is the decision.
//
// **The status bar, always.** It is ambient help in §14's sense: true the whole
// time, about the whole app, costing nothing to read and nothing to dismiss. It
// sits beside the other things that are always true — whether the core loaded,
// where the cursor is — which is where somebody looking for the caveat would
// think to look. It survives at a phone's width, where it wraps rather than
// being the first thing dropped.
//
// **The saved file, every time.** A file outlives the session and gets read
// where there is no status bar: mailed to somebody, opened in an editor, fed to
// the runner. The core writes the same sentence into the document, so the claim
// travels with the numbers instead of staying on the screen they were made on.
//
// **Nowhere else, and that is the decision.** Not a first-run dialog: §10 is
// thirty seconds to a lit LED, and greeting somebody with a caveat before they
// have done anything spends the one thing the milestone is made of. Not beside
// every reading: on-demand help is asked for rather than pushed (§14), and a
// sentence repeated against every number is a sentence people learn to skip —
// which is the worst thing that could happen to it before Phase 2 starts
// telling them what to buy (B-11) and how to wire it (B-15). Not on the
// runner's output either: that is data, read immediately by the person who
// typed the command, rather than something kept and passed on.
//
// **What is owed later.** The rule is that anything leaving the app carries it,
// so the BOM and the KiCad export inherit this when they land. And §14 says
// M-22's beginner sessions settle placement rather than taste: two author
// sessions produced no finding against the status bar, which is weak evidence
// in its favour and not evidence against.

/**
 * The claim, in the words M-13 first wrote it in.
 *
 * The core owns this sentence — it is a statement about the solver, not about a
 * screen (design doc §3), and `ACCURACY` in `crates/core/src/lib.rs` is where it
 * lives. This is the app's copy, held to the core's by a test, the way ADR 0008
 * holds every other fact the two of them share.
 *
 * A copy rather than a question asked at run time, for one reason: the sentence
 * that must always be on screen cannot be the one that waits for a wasm bundle
 * to load, and it has to still be there in the case where the core failed to
 * load at all — which is exactly when the app is least entitled to be believed.
 */
export const ACCURACY = 'Educational simulation — not accurate enough to design from'
