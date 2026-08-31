---
title: Editor23 Theme Compare Borrowed Rule Index
category: zircon_editor
report_id: Editor23-theme-compare-borrowed-rule-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Theme Compare Borrowed Rule Index

## Scope

This slice removes selector, label, and declaration-block cloning from theme comparison rule
indexing. Import order, stylesheet order, duplicate selector last-write behavior, inline labels,
comparison ordering, declaration formatting, and generated comparison text remain unchanged.

## Change

- Store borrowed stylesheet labels, selectors, and declaration blocks in the comparison BTreeMap.
- Format rule labels only for entries that are emitted to the comparison model.
- Aggregate imported rules by moving borrowed references instead of cloned rule payloads.
- Reuse borrowed declaration blocks for equality checks and final formatting.

## Deterministic Performance Evidence

| 2,048 rules, 16 declarations per rule, four index builds per sample | Before | After |
|---|---:|---:|
| Selector string clones per sample | 8,192 | 0 |
| Declaration-block deep clones per sample | 8,192 | 0 |
| Nested declaration value clones per sample | 131,072 | 0 |
| Rule index entries per sample | 8,192 | 8,192 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR23_THEME_COMPARE_BORROWED_RULE_INDEX_BENCH_V1`. Acceptance requires borrowed rule indexing
P95 to be at least 50% below cloned rule indexing. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826ax_theme_compare_index_preserves_last_selector_and_output` covers
  duplicate selector last-write behavior, shadowed local output, and imported-only output.
- `optimization_batch_20260826ax_theme_compare_uses_borrowed_rule_index` requires borrowed keys and
  blocks and rejects selector or declaration-block clones in index construction.
- `optimization_batch_20260826ax_theme_compare_borrowed_rule_index_p95` reports paired P50/P95
  samples and enforces the 50% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns schema-backed preview data, typed diagnostics, incremental validation,
preview fidelity, bindings, transactions, cook artifacts, and large-asset gates. This slice only
converges theme comparison rule indexing.
