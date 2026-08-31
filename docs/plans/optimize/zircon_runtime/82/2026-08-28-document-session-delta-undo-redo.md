# Runtime82 document-session delta undo and redo

Date: 2026-08-28

Status: `current_source_and_unreal_review_complete / architecture_selected /
delta_history_not_full_document_snapshots / implementation_complete_unvalidated /
focused_history_3_of_3_passed / document_harness_54_of_54_passed /
managed_runtime_profile_power_wgpu_pending`

## Problem

Text08 has exact committed edit intents, a retained document revision chain, and an atomic
document/Surface transaction, but no operation history or Ctrl/Cmd+Z and redo route. Implementing
history as full `UiEditableTextState` or full document snapshots would restore `O(N)` text copying
per input operation and multiply secret/plaintext retention. That conflicts with the measured
retained-document work that removed full-line and repeated source copying.

History also cannot live in clone/serde `UiSurface`: document identity is session-owned, and a
serialized transient undo stack would publish stale source/revision relationships. It cannot be a
process-global manager because node IDs and tree IDs are not unique product-surface identities.

## Unreal reference review

`FSlateEditableTextLayout` owns nested `BeginEditTransaction` / `EndEditTransaction`, snapshots state
before the outer transaction, pushes history only when text changes, removes redo states after a new
edit, and caps `UndoStates` at `EditableTextDefs::MaxUndoLevels`. Ctrl+Z is caught by an editable
control even with an empty stack so a focused search field cannot trigger an editor/world undo.
Undo/redo are disabled while read-only or composing. IME `BeginComposition` opens one transaction;
individual `SetTextInRange` updates do not add history, and `EndComposition` closes the unit.

The Unreal source stores full text in `FUndoState` and contains its own note that saving/restoring the
whole document is not ideal. Zircon adopts the scoped-owner, one-composition-unit, redo invalidation,
bounded-depth, and focused-command-capture behavior. It does not adopt full-document history.

Reference locations:

- `SlateEditableTextLayout.cpp:3300-3440`: transaction and bounded stack owner;
- `SlateEditableTextLayout.cpp:3443-3547`: focused undo/redo behavior;
- `SlateEditableTextLayout.cpp:4316-4469`: IME updates excluded from individual history and one
  composition transaction;
- `SlateEditableTextLayout.h:812-825`: retained history ownership.

## Selected architecture

History belongs beside each `UiTextDocumentBinding` in `UiTextDocumentSession`. One committed entry
stores only:

- exact old and new document byte ranges;
- removed bytes copied from the retained document's old range;
- inserted bytes copied from the already-final new-state range;
- before and after caret/selection state.

Undo replaces the entry's new range with removed bytes and restores the before interaction state.
Redo replaces the old range with inserted bytes and restores the after state. Both produce a normal
`CommittedTextEditIntent` with typed `Undo` or `Redo` kind, then use the existing document admission,
public receipt projection, Surface property preflight, and infallible dual commit. History stacks move
only after that transaction succeeds.

The existing `TextDocument::snapshot_range` piece walk is the source for removed bytes. Expose it
through the store with document/revision validation; do not flatten the complete document. Inserted
bytes are borrowed from the final state and copied once into the retained history entry.

## Policy and complexity

- maximum 100 entries per document binding, matching Unreal's bounded-level model without assuming
  unbounded editor history;
- maximum 1 MiB combined removed/inserted delta bytes per binding;
- an edit larger than the retained history budget still commits, but clears prior undo/redo and acts
  as a history barrier;
- trimming removes oldest undo entries until both limits hold; redo is cleared by every new edit;
- secure editable text never stores another plaintext copy in history and acts as a barrier;
- external source epoch rebind, Surface identity change, detached owner, or tree switch clears its
  history with the document binding;
- preedit remains state-only; one composition commit records one entry; history commands are caught
  but do not run while composition is active or text is read-only;
- normal commit work is `O(removed bytes + inserted bytes)` for history plus existing local document
  preparation, not `O(document bytes)`. Stack push/pop is amortized `O(1)`; oldest-entry trim is
  bounded by 100 entries.

The 100-entry/1-MiB limits are explicit MVP containment policy, not product-load or power evidence.
They may only change after a managed edit/undo matrix records retained history bytes, allocation,
RSS, and p50/p95/p99 latency for small sequential edits and large replace/delete operations.

## Acceptance

- insert/delete/replace undo and redo preserve one document UUID and adjacent revisions;
- Ctrl/Cmd+Z and Ctrl+Y/Ctrl+Shift+Z are handled by a focused editable owner even when unavailable;
- new edits after undo clear redo;
- preedit creates no entry and composition commit creates one entry;
- secure, external source rebind, detached owner, and Surface identity switch clear history;
- rejected property/document preflight or dropped prepared transaction does not move stacks;
- content-free public receipts identify `Undo`/`Redo` and contain no removed/inserted text;
- direct source tests and static checks precede managed Runtime, performance, power, and WGPU gates.

## 2026-08-28 implementation status

Completed in source:

- `TextDocumentStore::source_range` validates document UUID and exact revision, then walks only the
  requested piece range. The direct current-source document harness passes `54/54`, including the
  new cross-piece and stale-revision test.
- `UiTextDocumentSession` now owns a bounded delta history per binding. Successful changed commits
  are the only operation that records or moves a stack. External source rebind, detached owner,
  tree/Surface identity switch, and secure synchronization discard the corresponding history.
- normal edit preparation checks the delta budget before copying removed source bytes. Oversized and
  secure edits remain committable but become barriers; neither path retains another plaintext copy.
- undo and redo build typed exact-range intents and reuse document admission/public projection,
  Surface property preflight, and the existing dual commit. UUID continuity and revision advancement
  remain document-owned.
- focused edit owners recognize Ctrl/Cmd+Z, Ctrl+Y, and Ctrl/Cmd+Shift+Z before clipboard/payload
  routing. Unavailable, read-only, and composing commands remain handled locally.
- product regressions cover UUID/revision continuity, redo invalidation, IME one-unit history, and
  secure barriers. They are present but are not reported as passing without a managed Runtime result.
- an E-drive direct history harness passes `3/3` for delta round-trip, composition interaction
  restoration, bounded depth, and redo invalidation. Scoped Rustfmt and whitespace checks pass, and
  the production history path adds no `panic`/`unwrap`/`expect`/`unreachable`.

Still open:

- managed Runtime compile and product regression execution;
- the planned allocation/RSS/latency matrix for retained history limits;
- Windows power capture and genuine WGPU screenshot acceptance. No screenshot was produced by this
  non-rendering slice.
