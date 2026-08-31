# Runtime82 committed edit range intent

Date: 2026-08-28

Status: `exact_committed_edit_intent_implemented / action_sequence_single_commit_guarded /
composition_preedit_state_only / surface_property_exclusive_prepare_implemented /
document_store_exclusive_prepare_implemented / dual_commit_coordinator_implemented_unwired /
product_document_binding_and_limits_open / managed_runtime_validation_blocked`

## Problem

The retained document store requires the exact old byte range and replacement slice before it can
admit and prepare an edit. The previous Surface input path published only the final
`UiEditableTextState`. Recovering an edit from two full strings would add an `O(N)` prefix/suffix
diff to every key, cut, paste, IME commit, and surrounding-text deletion. It would also make
composition and same-value edits ambiguous.

Unreal Slate avoids that reconstruction. `FSlateEditableTextLayout` opens a scoped edit transaction,
applies the known selection/range edit, then updates retained layout state
(`SlateEditableTextLayout.cpp:2138-2153`). `FTextLayout::InsertAt` and `RemoveAt` receive explicit
locations/ranges and repair retained runs and dirty state from those edits
(`TextLayout.cpp:2336-2399,2683-2774`). Zircon now preserves the same information at the edit owner.

## Implemented boundary

`apply_text_edit_action_with_intent` returns the final editable state plus an optional fixed-size
`CommittedTextEditIntent { old, new, kind }`. The replacement is borrowed from `state.text[new]`;
the intent does not allocate or retain another String. Insert, backspace, delete, selection replace,
IME composition commit, clipboard cut, paste/text input, and IME surrounding deletion preserve the
exact range. Caret movement, selection, preedit updates, composition cancel, and identical source
replacement remain state-only and do not request a document revision.

Keyboard word deletion is a state-only selection followed by one delete. The sequence reducer
retains at most one committed intent and returns typed `MultipleCommittedEdits` if a future action
mapping accidentally contains two document edits. The caller rejects that sequence without
publishing a partially reduced state.

The editable property transaction now validates and returns the intent in its internal receipt.
Public `UiTextEditReceipt` is deliberately not synthesized here: only a successful document-store
commit may sign the document UUID and consecutive revisions.

Both authorities now expose exclusive prepared owners. The Surface preparation retains an exclusive
`&mut UiSurface` and owns its ten values; the document preparation retains an exclusive
`&mut TextDocumentStore` after admission and public projection. Dropping either prepared owner makes
no mutation. `PreparedUiEditableTextDocumentTransaction` consumes both and commits document then
Surface without a business-error return. It is deliberately unwired until the product editing
session supplies explicit store limits and node-to-document binding.

## Complexity and containment

- intent creation is `O(1)` after the edit owner has already selected the source range;
- replacement access is a borrowed UTF-8 slice and creates no second payload allocation;
- no-op classification compares only the selected range with the requested replacement;
- the action sequence is currently one or two actions and uses no document-length scan;
- invalid new ranges fail property preflight before tree/style/component mutation.

This removes the need for a per-keystroke whole-string diff. It does not claim the piece-store or
layout pipeline is optimal; the storage residency profile and matched Unreal workload remain the
algorithm-selection gate.

## Verification

The direct current-source edit harness on E drive passes `12/12` tests, including combining-grapheme
delete, exact selection replacement, transient preedit, composition commit, sequence rejection, and
same-value no-op. Rustfmt and scoped `git diff --check` pass.

`cargo check -p zircon_runtime --lib --locked --no-default-features --features text` reaches the real
Runtime module graph and reports 95 workspace errors / 198 warnings. None names the new
prepared/coordinator/store files or the changed UI edit files. The blockers include concurrent text
shaping/layout changes and unrelated core/scene/platform visibility/type errors, so this is not a
managed Runtime green result.

## Required next boundary

The dual-prepare/commit mechanism now exists. The product gateway must wire it as one transaction:

1. prepare and validate the fixed ten-property editable mutation without writes;
2. resolve the session-owned document and prepare/admit the exact document edit;
3. validate public receipt projection before writes;
4. construct the prepared coordinator and enter its infallible commit section;
5. publish the signed receipt only after both commits succeed.

Do not commit the property transaction and then diff/repair the document. Do not commit the document
before a still-fallible property preflight. A rollback journal is required if either commit section
later gains a fallible operation.
