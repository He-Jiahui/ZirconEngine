---
title: Runtime04 Migration Run and Transaction Capacity
category: zircon_runtime
report_id: Runtime04-migration-run-transaction-capacity-2026-08-26
date: 2026-08-26
session_id: optimize-runtime04-direct-reference-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Migration Run and Transaction Capacity

## Scope

Migration orchestration copied known root paths and migration roots into zero-capacity vectors;
transaction application also collected prepared writes without reserving the pending document count.

## Implementation

Root path and migration-root projections reserve `roots.len()`. Transaction write preparation
reserves `pending.len()` before preserving the existing order and retirement mapping. Recovery,
fault injection, and durable transaction sequencing remain unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Root projection start capacity | 0 | roots count |
| Prepared write start capacity | 0 | pending count |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `RUNTIME04_MIGRATION_RUN_CAPACITY_BENCH_V1` and
`RUNTIME04_MIGRATION_TRANSACTION_CAPACITY_BENCH_V1` with legacy/optimized p95 and cardinalities.

## Validation

Scoped rustfmt, diff checks, source contracts, order equivalence tests, and both ignored release
benchmarks are prepared. Commit integration, terminal p95 values, and WeCom delivery remain
coordinator-owned.
