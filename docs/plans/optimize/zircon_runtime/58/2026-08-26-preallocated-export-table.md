---
title: Runtime58 Preallocated Export Table
category: zircon_runtime
report_id: Runtime58-preallocated-export-table-2026-08-26
date: 2026-08-26
session_id: root-runtime58-three-task-bridge-performance-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime58 Preallocated Export Table

## Scope

`FrozenBridgeTable::from_exports` receives the full export iterator at a registration generation
boundary. The common Vec/HashMap iterators expose a lower-bound count, so the table can reserve
storage before inserting entries without changing slot order, duplicate handling, or provider
ownership.

## Implementation

The constructor now consumes the iterator once, uses its `size_hint` lower bound for both the
entry slice and interface-slot map, and retains the existing insertion order and boxed immutable
table shape. Empty or non-exact iterators remain correct because the collections still grow as
needed.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Entry capacity reserve for 2,048 exports | 0 | 2,048 |
| Slot map capacity reserve for 2,048 exports | 0 | 2,048 |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `RUNTIME58_PREALLOCATED_EXPORT_TABLE_BENCH_V1` with both p95
durations, sample/iteration/export counts, and entry/map capacity reserves.

## Validation

Scoped rustfmt, diff checks, table slot regressions, and the source contract are prepared. The
managed `runtime58_batch_` release gate alternates legacy/optimized samples and covers all three
bridge optimizations in one Cargo invocation: 3 source contracts, 8 Rust tests, and 3 performance
rows. Commit integration, terminal P95 values, and WeCom delivery remain coordinator-owned.
