---
title: Editor57 Exact Table Row Capacity
category: zircon_editor
report_id: Editor57-exact-table-row-capacity-2026-08-25
date: 2026-08-25
session_id: root-editor57-name-line-split-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor57 Exact Table Row Capacity

## Scope

This slice reserves the asset-table node vector for exactly the rows missing after stale rows are
removed. It preserves list/thumbnail mode behavior, prototype discovery, existing row order and state,
hole filling order, stale-row removal, generated IDs, and materialized-item limits. It does not change
logical asset ordering, selection, table-cell projection, virtualization, or unrelated changes already
present in the touched file.

## Implementation

The existing row bitmap already identifies every row that survives synchronization. The retired path
then appended missing row clones without reserving destination capacity, repeatedly moving the large
`ViewTemplateNodeData` entries as the vector grew. The optimized path counts false bitmap entries once
and reserves that exact missing-row count before the existing append loop.

The regression compares retired and optimized nodes with an unrelated node, an existing hole, and a
stale row. A source contract requires the exact reserve to precede all missing-row pushes.

## Performance Contract

| Evidence per 2,048-row cold synchronization | Retired path | Optimized gate |
| --- | ---: | ---: |
| Missing row clones | 2,047 | 2,047 |
| Dynamic capacity growths | multiple | 1 exact reserve |
| Alternating release benchmark | 11 samples x 32 syncs | optimized P95 <= 90% of retired P95 |

The benchmark emits `EDITOR57_EXACT_TABLE_ROW_CAPACITY_BENCH_V1` with both P95 timings, reduction
basis points, sample/iteration/row counts, and measured retired/optimized capacity growths.

## Validation

The scoped TDD source probe first observed the missing row reserve, then observed exact reservation
before the append loop. Rust 1.94.1 formatting and scoped static checks passed before batching. One
managed Editor57 batch covers this slice together with exact logical-paint chunk capacity, including
equivalence, source contracts, and both ignored release benchmarks. Dynamic P95 evidence, integration
SHA, automatic commit, and automatic WeCom performance delivery remain coordinator-owned and pending.
