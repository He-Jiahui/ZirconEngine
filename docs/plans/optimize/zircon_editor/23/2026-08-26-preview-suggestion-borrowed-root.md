---
title: Editor23 Preview Suggestion Borrowed Root
category: zircon_editor
report_id: Editor23-preview-suggestion-borrowed-root-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Preview Suggestion Borrowed Root

## Scope

This slice removes full nested-value projection from preview-mock suggestion root selection.
Array and table display keys, dotted authored keys, exact selection, stale-descendant fallback,
root fallback, suggestion ordering, and applied suggestion values remain unchanged.

## Change

- Walk only container paths that can prefix the selected nested key and return a borrowed
  `&Value` instead of recursively cloning every nested value into an entry list.
- Return immediately when a generated container path exactly matches the selected key.
- Preserve the deepest existing container fallback for stale descendant selections; clone only
  the immediate suggestion values that the presentation contract returns.

## Deterministic Performance Evidence

| 4,096 payload branches plus one first-position target | Before | After |
|---|---:|---:|
| Nested values cloned during root selection | 8,194 | 0 |
| Full nested-entry sort | 1 | 0 |
| Exact target container reads | after full projection | direct borrowed read |
| Returned root value ownership | cloned | borrowed |

Each payload branch contains 256 text bytes. The ignored release gate runs 17 alternating sample
pairs and emits `EDITOR23_PREVIEW_SUGGESTION_BORROWED_ROOT_BENCH_V1`. Acceptance requires borrowed
root selection P95 to be at least 90% below full nested projection P95. Exact Windows timings
remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826ao_preview_suggestion_borrowed_root_preserves_selection` covers
  nested arrays, objects, stale descendants, dotted keys, root fallback, and scalar rejection.
- `optimization_batch_20260826ao_preview_suggestion_root_avoids_nested_tree_projection` rejects
  nested-entry projection and root-value cloning while requiring borrowed array/table traversal.
- `optimization_batch_20260826ao_preview_suggestion_borrowed_root_p95` reports paired P50/P95
  samples and enforces the 90% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns lossless V2 editing, revisioned transactions, incremental validation,
preview fidelity, bindings, themes, accessibility, fonts, cook artifacts, and large-asset gates.
This slice only converges preview suggestion root selection.
