---
title: Editor23 Theme Source Lightweight Selection
category: zircon_editor
report_id: Editor23-theme-source-lightweight-selection-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Theme Source Lightweight Selection

## Scope

This slice removes full theme-entry projection from key reconciliation and index selection in the
UI asset editor. Local-first ordering, authored import order, duplicate references, missing-import
selection, summary labels, and fallback behavior remain unchanged.

## Change

- Reconcile a current key directly against local-theme availability and the authored style-import
  slice; clone only the selected/fallback key.
- Resolve an index by applying the optional local-theme offset and reading the import slice
  directly.
- Build display entries once in `build_theme_summary` and locate the selected key in that existing
  projection instead of calling reconciliation that rebuilt all entries.

## Deterministic Performance Evidence

| 8,192 authored theme imports, selected last | Before | After |
|---|---:|---:|
| Theme entry projections | 8,192 | 0 |
| Label formatting operations | 8,192 | 0 |
| Imported token/rule statistics | up to 8,192 documents | 0 |
| Selected key clones | 1 | 1 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR23_THEME_SOURCE_LIGHTWEIGHT_SELECTION_BENCH_V1`. Acceptance requires lightweight selection
P95 to be at least 90% below entry-projection P95. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826an_theme_source_lightweight_selection_preserves_keys` covers local,
  missing, loaded, duplicate, fallback, index, and summary-selection behavior.
- `optimization_batch_20260826an_theme_source_selection_avoids_entry_projection` requires direct
  slice lookup and rejects entry reconstruction in select, reconcile, and summary selection.
- `optimization_batch_20260826an_theme_source_lightweight_selection_p95` reports paired P50/P95
  samples and enforces the 90% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns complete UI asset authoring, previews, bindings, themes, accessibility, menus,
fonts, and runtime-product parity. This slice only converges theme-source selection projection.
