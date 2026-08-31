---
title: Editor23 Theme Refactor Label Cache
category: zircon_editor
report_id: Editor23-theme-refactor-label-cache-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Theme Refactor Label Cache

## Scope

This slice removes per-action imported-token map reconstruction from theme-refactor presentation.
Action ordering, inherited-value labels, duplicate rule labels, redundant-import labels, and the
public action projection remain unchanged.

## Change

- Return the imported-token map with the refactor actions from one internal projection.
- Borrow that shared map while formatting every duplicate-token label instead of rebuilding and
  cloning the full imported token cascade per action.
- Keep the public `theme_refactor_actions` result contract by discarding the retained map for
  non-presentation callers.

## Deterministic Performance Evidence

| 1,024 duplicate local/imported tokens, one pane build per sample | Before | After |
|---|---:|---:|
| Imported-token map builds in action projection | 1 | 1 |
| Imported-token map builds in label projection | 1,024 | 0 |
| Total imported-token map builds | 1,025 | 1 |
| Label-stage imported entries cloned | 1,048,576 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR23_THEME_REFACTOR_LABEL_CACHE_BENCH_V1`. Acceptance requires shared-map label presentation
P95 to be at least 50% below per-action map reconstruction. Exact Windows timings remain pending
the coordinator run.

## Acceptance

- `optimization_batch_20260826as_theme_refactor_labels_preserve_imported_values` covers ordered
  duplicate-token labels and inherited TOML values.
- `optimization_batch_20260826as_theme_refactor_labels_reuse_imported_token_map` requires one map
  build in the projection and rejects any map build in `label`.
- `optimization_batch_20260826as_theme_refactor_label_cache_p95` reports paired P50/P95 samples and
  enforces the 50% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns schema-backed preview data, typed diagnostics, incremental validation, preview
fidelity, bindings, transactions, cook artifacts, and large-asset gates. This slice only converges
theme-refactor presentation labels.
