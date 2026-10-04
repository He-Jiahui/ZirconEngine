---
title: Runtime82 Inactive Composition Metadata Encoding
category: zircon_runtime
report_id: Runtime82-inactive-composition-metadata-encoding-2026-09-27
date: 2026-09-27
implementation_status: implemented_pending_validation
validation_status: managed_validation_pending
performance_status: product_gates_open
---

# Runtime82 inactive composition metadata encoding

## Failure and owned repair

The editable property transaction previously encoded `composition = None`
as two caret offsets and empty text/restore/clauses. Both input materialization
and render resolution interpret nonnegative offsets with a string as an active
composition. After the first normal edit, the next Backspace could change the
visible body without a committed intent, document receipt, or undo entry.
Material text-input descriptors also injected the same ambiguous empty tuple
before the first edit.

`ui/editable_text_composition.rs` now owns the private shared metadata constant
`INACTIVE_COMPOSITION_OFFSET = -1`. The transaction emits it for both inactive
offsets; built-in descriptor defaults use the same value. Existing unsigned
offset readers reject a negative value, so input and render materialize `None`
without another state parser or a new public Rust API. Active composition keeps
its actual nonnegative offsets, including an empty preedit at a zero-length
range. Its restore text and clauses keep their existing projection.

Previously authored or serialized nonnegative empty tuples retain the existing
active interpretation: they cannot be distinguished losslessly from a real
empty preedit. There is no heuristic migration based on empty text or equal
range endpoints. New inactive snapshots serialize the negative offsets.

## Reference and affected contracts

Local Unreal Slate `SlateEditableTextLayout.cpp:4158-4168` initializes an
independent `bIsComposing` flag; its context methods at `4428-4472` manage
Begin/Update/End separately from the composition length. This is evidence for
retaining an explicit lifecycle distinction when length is zero. The negative
metadata representation is Zircon's local choice for its existing Int schema
and unsigned readers.

Inactive expectations were updated in keyboard Cancel/Commit/Escape, ordinary
property reset, focused bound model reset, and accessibility replacement or
selection cleanup. Existing activity assertions for real nonempty Preedit
remain unchanged. Preexisting foreign edits in these files were backed up and
preserved.

The new `runtime82_inactive_composition_regression` integration target uses
real Surface/InputManager dispatch, render extraction, descriptor compilation,
and clone/serde. Nine tests cover consecutive Backspace/Undo, empty preedit
over a selection, zero-range empty preedit Cancel/Commit, catalog initialization,
programmatic reset, and active/inactive snapshots. No test reads source text
or assumes the numeric value of the sentinel.

## Acceptance boundary

Managed compilation and execution are pending in the grouped Windows batch.
The empty selected-preedit Commit regression also exposed separate committed
source synchronization and metadata-only epoch defects; their repair is a
required follow-on in the same batch, not a reason to weaken this test.
Static review and formatting are not functional acceptance.

This is an editing correctness prerequisite. It adds no latency, allocation,
RSS, WGPU-present, or matched Unreal performance claim. Runtime82's complete
product performance gates remain open until the applicable Release measurements
and functional regressions pass on the sealed source snapshot.
