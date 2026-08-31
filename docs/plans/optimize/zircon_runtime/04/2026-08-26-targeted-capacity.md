---
title: Runtime04 Targeted Registry Capacity
category: zircon_runtime
report_id: Runtime04-targeted-registry-capacity-2026-08-26
date: 2026-08-26
session_id: optimize-runtime04-direct-reference-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Targeted Registry Capacity

## Scope

Targeted source removal/replacement materializes source-owned entries and dependency-path tuples.
The filtered source-entry iterator previously left capacity estimation to `collect`, while the
dependency tuple vector started empty despite a known entry count.

## Implementation

Source entry extraction now reserves the source UUID set length before filtering missing entries.
Dependency-path tuples reserve the exact root-plus-entry lower bound after a single root check.
Returned entries, tuple order, and duplicate/path semantics remain unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Source-entry reservation | filter collect | source UUID count |
| Dependency tuple reservation | 0 | root + entry count |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `RUNTIME04_TARGETED_CAPACITY_BENCH_V1` with source-entry and
dependency-path p95 pairs, sample/iteration/count data, and reservation reductions.

## Validation

Scoped rustfmt, diff checks, source contracts, and set/order equivalence tests are prepared. The
release benchmark is batched with registry projection capacity in one Runtime crate command;
commit integration, terminal p95 values, and WeCom delivery remain coordinator-owned.
