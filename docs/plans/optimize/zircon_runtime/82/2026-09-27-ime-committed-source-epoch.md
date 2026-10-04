---
title: Runtime82 IME Committed Source Epoch
category: zircon_runtime
report_id: Runtime82-ime-committed-source-epoch-2026-09-27
date: 2026-09-27
implementation_status: implemented_pending_validation
validation_status: managed_validation_pending
performance_status: product_gate_pending
---

# Runtime82 IME committed source epoch

## Production failure and repair

The real `UiInputManager` IME path opens a retained text document before
publishing Preedit. The previous property commit advanced the source epoch
when a provisional value changed. On the next event, source synchronization
therefore discarded that document and opened the visible preedit text. Empty
Preedit replacing selection `1..3` in `abcd` produced visible `ad`; its subsequent
empty Commit still described deletion of original bytes `1..3`, which cannot
be read from the incorrectly reopened two-byte document. Nonempty Preedit could
also replace the retained original and corrupt the text saved for Undo.

The Surface source epoch now identifies committed source changes independently
of layout/text revisions. Provisional composition updates preserve that epoch.
When a no-intent edit ends an active composition, the mutation owner compares
the final body with the bound committed document across retained pieces. An equal
source preserves the epoch; a different source keeps the normal reset behavior.
This handles IME Cancel, keyboard Escape, a matched-source Commit, and Backspace
that removes only provisional text. Known focus/input-method-loss cancellation
also preserves the epoch. If the document binding is unavailable, explicit IME
Cancel and keyboard Escape (logical key or key code 27) still restore the source;
restoration does not depend on document admission succeeding. Other no-intent
finishing operations retain a diagnostic and use source reset when their equality
cannot be established. Layout invalidation, visible value updates, binding
reports, and component state projection still run. Other property source changes
retain the normal reset behavior. A committed edit intent advances the epoch
even when the visible text was already published during Preedit; the property
writer also handles a metadata batch with no changes. These writer rules are a
coordinated change in the root-owned `property_transaction.rs`.

`synchronize_editable_source` first reuses an existing binding at the same source
epoch. When a new manager opens an active Surface snapshot, it restores the
source from visible prefix, composition `restore_text`, and visible suffix.
Equal replacement/restore bytes and non-composition states borrow the visible
body. A malformed UTF-8 range records `InvalidEditIntent` and does not open the
visible projection. Full source reconstruction occurs only when a binding must
be opened, rather than on every active preedit event.

The comparison uses `TextDocumentStore::source_equals`, which checks document
identity/revision and the existing `TextDocument::range_equals` authority. It
does not create a flattened snapshot or temporary body. It runs only when an
active composition ends without a committed edit intent. A length mismatch
returns immediately; a same-length body can require a full byte comparison.
The million-character cost of this rare finishing step remains unmeasured and
its product budget remains open. Ordinary committed edits and ongoing Preedit
avoid that scan. Without a document manager, only explicit IME Cancel/keyboard
Escape and the existing cancellation helper use source preservation.

The existing negative inactive-composition offsets are a separate representation
repair. Nonnegative active empty Preedit tuples remain active, including after
Surface clone or JSON restore. No serialized public type or public API is added.

## Behavior evidence

The new private session tests cover selected empty preedit source restoration,
successive visible preedit updates retaining one document, a zero-range empty
preedit, UTF-8 source restoration, programmatic epoch reset, and malformed
snapshot rejection. The integration target
`runtime82_inactive_composition_regression` uses actual public manager dispatch,
property mutation, compiler defaults, render extraction, and JSON snapshots.
Its added cases verify that nonempty Preedit/Cancel preserves the preceding
document revision and Undo history; Escape, matched-source Commit, and removal
of only provisional text also retain the prior Undo entry. Partial provisional
deletion leaving a different source still rebinds before the next committed edit.
An active selected empty Preedit snapshot can be committed and undone by a new
manager. Its original selected
empty Commit expectation remains unchanged.

The private session tests additionally lower the Store document limit to two
bytes and dispatch real Preedit followed by Cancel, physical-key-code Escape,
and no-intent Commit. They verify restored visible text, inactive composition,
source epoch preservation for cancellation, owner clearing for IME Cancel, and
diagnosed source reset for an unbound Commit. Another real dispatch case covers
physical-key-code Escape without a document manager.

Existing retained-document IME, focus-loss, programmatic reset, accessibility,
and bound-model tests remain part of the managed behavior batch. Bound-model
state-only updates during active composition have a separate source-epoch
failure, tracked by the root-owned owner-path repair; this slice does not claim
that path as validated.

## Ownership and acceptance boundary

Exact lease request `5aa60a070f7243d4a21c7cbba19c185f` acquired the three source
paths, new private tests, and two records without conflicts under Session
`astra-optimize-20260926-batch-a`. Preexisting bytes and Git diffs were saved in
`.codex/state/session-coordinator/async-validation-batches/2026-09-27-astra-optimize-batch-m-runtime82-ime-preexisting-diffs.json`.
Store source comparison and its new tests were acquired without conflicts by
exact lease request `967e39066abc4e65810b3b801196ad9b`; the original Store bytes
and diff were appended to the same backup. The integration target retains its
earlier exact lease. The root owns the
property writer and inactive offset/default changes; this slice does not edit
those paths.

Static Rustfmt, scoped diff, and API review are the available evidence. Compilation
and behavior remain pending grouped managed Windows validation. This correctness
repair does not contain a performance comparator: the old path loses source
semantics, so a timing ratio against it would not be a fair product acceptance
measure. Runtime82 `RTE-GATE-016` and `RTE-GATE-047`, million-character edit-to-present
latency, allocation/RSS, and matched Unreal evidence remain open. No full product
performance pass is claimed.
