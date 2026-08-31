---
title: Runtime04 Migration Sidecar Capacity
category: zircon_runtime
report_id: Runtime04-migration-sidecar-capacity-2026-08-26
date: 2026-08-26
session_id: optimize-runtime04-direct-reference-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Migration Sidecar Capacity

## Scope

Sidecar preflight constructed document, pending-write, compound-binding, and registry-entry
vectors from zero even though inventory cardinalities and document entry counts bound every result.

## Implementation

Documents and pending outputs reserve the sidecar plus recognized-source bound; compound bindings
reserve sidecar candidates; registry entries reserve the exact sum of one root entry plus all
subentries. Parsing, retired-sidecar handling, duplicate detection, and registry semantics are
unchanged.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Preflight result start capacity | 0 | bounded by inventory cardinality |
| Registry entry start capacity | 0 | exact root + subentry count |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `RUNTIME04_MIGRATION_SIDECAR_CAPACITY_BENCH_V1` with legacy/optimized
p95, sample/iteration/document counts, and capacity reduction.

## Validation

Scoped rustfmt, diff checks, source contracts, capacity formula tests, and the ignored release
benchmark are prepared. Commit integration, terminal p95 values, and WeCom delivery remain
coordinator-owned.
