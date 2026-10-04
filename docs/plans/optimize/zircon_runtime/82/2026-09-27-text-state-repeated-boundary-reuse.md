---
title: Runtime82 Text State Repeated Boundary Reuse
category: zircon_runtime
report_id: Runtime82-text-state-repeated-boundary-reuse-2026-09-27
date: 2026-09-27
implementation_status: implemented_pending_validation
validation_status: managed_validation_pending
performance_status: local_release_gate_pending
---

# Runtime82 text state repeated boundary reuse

## Production path and bounded change

Keyboard, text, IME, and pointer editing materialize an owned
`UiEditableTextState` from Surface metadata before applying an edit.
`editable_text_state_for_node` separately clamps selection anchor/focus,
composition start/end, and caret: up to five calls against the same body.
The current shared grapheme helper has constant-time ASCII adjacency handling;
non-ASCII intermediate offsets still traverse graphemes from the beginning.
The editable property transaction publishes the same caret offset into both
selection fields when selection is absent. A subsequent materialization can
therefore repeat the same long Unicode scan three times. Active composition
can add two more offsets, which may equal those earlier offsets.

The materializer now retains up to five `(raw offset, clamped offset)` pairs in
a stack array scoped to that single invocation. A repeated raw offset reuses
its result. A distinct offset continues through the existing
`clamp_grapheme_boundary` authority; there is no second Unicode implementation,
new shared cache, or revision/owner inference. Capacity exhaustion would fall
back to the existing helper rather than change boundary semantics.

The change preserves metadata coercion, property fallback, affinity,
selection/composition presence, preedit clauses, read-only flags, and the owned
body. It removes repeated boundary work only; one full-body materialization
copy and one scan per distinct non-ASCII offset remain. A later Runtime82
contract repair encodes inactive composition with negative offsets; the
unsigned reader therefore omits those fields. The committed-surface regression
now expects no active composition after a normal edit.

## Behavior and local Release fixture

The regression module checks real Surface state materialization with absent,
partial, repeated, and distinct metadata offsets; combining/prepend Unicode,
an interior UTF-8 offset, CRLF, out-of-range offsets, plain ASCII, and missing
or non-editable nodes. It checks results against a test-local copy of the HEAD
materializer. A repeated offset after the retained body changes is clamped
again, proving that reuse does not survive into another call. A small real
property prepare/commit fixture confirms the ten-field projection produces
the same complete materialized state.

The ignored `runtime82_text_state_repeated_boundary_release_profile` creates
1K, 10K, and 1M Unicode-character documents (`é`: one scalar and one grapheme,
two UTF-8 bytes per character). Each Surface is primed through the real
property transaction with a legal near-tail caret before either path is timed.
The original HEAD materializer and new materializer then receive that same
held Surface; five warmups and 31 measured pairs alternate old/new call order.
Fixture construction and equality assertions occur outside timing. Both
timed calls include their real owned-body clone and full state construction.
Every pair compares the entire optional state, body byte length, and caret.

`RUNTIME82_TEXT_STATE_REPEATED_BOUNDARY_V1` prints OS, architecture, crate
version, processor identity, character/byte counts, offset, raw nanosecond
samples, and nearest-rank p50/p95/p99. The local 1M-character materialization
p95 must be at most 80% of the old path. This gate covers the repeated Unicode
offset case, not distinct-offset throughput or an ASCII improvement claim.

## Acceptance boundary

Only static Rustfmt, scoped diff, and source-structure checks have been run.
Managed Windows Runtime behavior tests and the ignored Release comparison are
pending in the grouped asynchronous batch. This measures state materialization
only; document mutation, commit projection, render extraction, WGPU present,
allocation/RSS, and matched Unreal measurements are outside its timed interval.
Runtime82 `RTE-GATE-016` and `RTE-GATE-047` remain open; no complete product
latency or memory pass is claimed.
