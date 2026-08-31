---
title: Runtime04 Resolver Identity Unstable Sort
category: zircon_runtime
report_id: Runtime04-resolver-identity-unstable-sort-2026-08-26
date: 2026-08-26
session_id: optimize-runtime04-direct-reference-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Resolver Identity Unstable Sort

## Scope

Migration resolver finalization sorted locator and project-hint identity lists with stable
sorting even though each comparator includes every field used by the following deduplication key.
The change targets only these two bounded identity projections.

## Implementation

The two sort/dedup paths are now explicit helpers using `sort_unstable_by` with the original
comparison keys and unchanged `dedup_by` predicates. Functional coverage compares both helpers
against the legacy stable-sort behavior on reverse-ordered duplicate-heavy fixtures.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Stable identity sorts | 2 | 0 |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `RUNTIME04_RESOLVER_IDENTITY_UNSTABLE_SORT_BENCH_V1` with source and
hint legacy/optimized p95, sample/iteration/identity counts, and the stable-sort reduction.

## Validation

Scoped rustfmt, diff checks, source contracts, and equivalence tests are prepared. The ignored
benchmark runs in one Runtime crate release command; commit integration, terminal p95 values, and
WeCom delivery remain coordinator-owned.
