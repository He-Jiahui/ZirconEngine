---
title: Editor06 Runtime Capabilities Unstable Sort
category: zircon_editor
report_id: Editor06-runtime-capabilities-unstable-sort-2026-08-26
date: 2026-08-26
session_id: root-editor06-plugin-admission-borrows-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor06 Runtime Capabilities Unstable Sort

## Scope

Runtime capability projection normalized core capability strings and plugin summary entries with
stable sorting before deduplication. Both comparison keys fully cover their deduplication values,
so stable tie preservation is not observable.

## Implementation

Core capabilities now use `sort_unstable`, and plugin summaries use the existing comparison tuple
with `sort_unstable_by`; deduplication and projection order are unchanged. Reverse-ordered,
duplicate-heavy fixtures compare the optimized projection with the legacy stable behavior.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Targeted stable sorts | 2 | 0 |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `EDITOR06_RUNTIME_CAPABILITIES_UNSTABLE_SORT_BENCH_V1` with capability
and plugin legacy/optimized p95, sample/iteration/cardinality counts, and `stable_sorts=2->0`.

## Validation

Scoped rustfmt, diff checks, source contracts, and projection equivalence tests are prepared. The
ignored benchmark runs in one Editor crate release command; commit integration, terminal p95 values,
and WeCom delivery remain coordinator-owned.
