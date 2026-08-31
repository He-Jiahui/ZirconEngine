---
title: Editor23 Theme Cascade Borrowed Definitions
category: zircon_editor
report_id: Editor23-theme-cascade-borrowed-definitions-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Theme Cascade Borrowed Definitions

## Scope

This slice removes eager token and rule-definition ownership from theme cascade inspection. Layer
order, token active/shadowed order, rule occurrence order, duplicate selector arbitration, inline
stylesheet labels, declaration formatting, and final presentation strings remain unchanged.

## Change

- Index token names by borrowed `&str` and retain borrowed source and TOML value references.
- Index rule selectors by borrowed `&str` and retain borrowed source, stylesheet, selector, and
  declaration-block references.
- Format declaration blocks only for selectors with an active/shadowed summary.
- Avoid all intermediate definition string and declaration clones while preserving owned output.

## Deterministic Performance Evidence

| 2,048 unique rules, 16 declarations per rule, two builds per sample | Before | After |
|---|---:|---:|
| Selector clones for key plus definition per sample | 8,192 | 0 |
| Source string clones per sample | 4,096 | 0 |
| Stylesheet string clones per sample | 4,096 | 0 |
| Eager declaration leaf formats per sample | 65,536 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR23_THEME_CASCADE_BORROWED_DEFINITIONS_BENCH_V1`. Acceptance requires borrowed cascade
definition construction P95 to be at least 80% below eager declaration formatting. Exact Windows
timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826ay_theme_cascade_borrowing_preserves_active_shadowed_order` covers
  imported/local layer order plus token and rule active/shadowed presentation.
- `optimization_batch_20260826ay_theme_cascade_uses_borrowed_definitions` requires borrowed token
  and rule definitions and rejects eager declaration formatting.
- `optimization_batch_20260826ay_theme_cascade_borrowed_definitions_p95` reports paired P50/P95
  samples and enforces the 80% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns schema-backed preview data, typed diagnostics, incremental validation,
preview fidelity, bindings, transactions, cook artifacts, and large-asset gates. This slice only
converges theme cascade inspection projection.
