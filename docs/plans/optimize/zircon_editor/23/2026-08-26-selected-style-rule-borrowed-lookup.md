---
title: Editor23 Selected Style Rule Borrowed Lookup
category: zircon_editor
report_id: Editor23-selected-style-rule-borrowed-lookup-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Selected Style Rule Borrowed Lookup

## Scope

This slice removes full local-style metadata projection when the UI asset editor needs declaration
rows for one selected rule. Flattened stylesheet order, selected-index interpretation,
declaration order, and empty selection/out-of-range behavior remain unchanged.

## Change

- Traverse stylesheet rule slices with one borrowed `flat_map(...).nth(index)` lookup.
- Stop cloning every preceding rule ID and selector into `LocalStyleRuleEntry` values for a
  single-rule read.
- Keep the existing full projection for callers that actually display the rule list.

## Deterministic Performance Evidence

| 16,384 rules, selected last | Before | After |
|---|---:|---:|
| `LocalStyleRuleEntry` allocations | 16,384 | 0 |
| Rule ID clones | 16,384 | 0 |
| Selector clones | 16,384 | 0 |
| Rule visits | 16,384 | 16,384 borrowed |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR23_SELECTED_STYLE_RULE_BORROWED_LOOKUP_BENCH_V1`. Acceptance requires borrowed lookup P95
to be at least 80% below full metadata-projection P95. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826am_selected_style_rule_borrowed_lookup_preserves_flat_index` covers
  cross-stylesheet indexing, declaration projection, empty selection, and out-of-range behavior.
- `optimization_batch_20260826am_selected_style_rule_uses_borrowed_nth_lookup` requires borrowed
  `flat_map(...).nth(...)` and rejects full-list construction and cloning.
- `optimization_batch_20260826am_selected_style_rule_borrowed_lookup_p95` reports paired P50/P95
  samples and enforces the 80% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns complete UI asset authoring, previews, bindings, accessibility, menus, fonts,
and runtime-product parity. This slice only converges selected style-rule declaration projection.
