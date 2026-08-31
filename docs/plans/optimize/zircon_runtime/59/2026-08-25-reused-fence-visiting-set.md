---
title: Runtime59 Reused Fence Visiting Set
category: zircon_runtime
report_id: Runtime59-reused-fence-visiting-set-2026-08-25
date: 2026-08-25
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime59 Reused Fence Visiting Set

## Scope

This slice removes one temporary cycle-detection set allocation per root prerequisite from bounded
keyed IO fence failure scans. It preserves prerequisite order, cycle boundaries, superseded-ticket
resolution, failure propagation, and all public task/runtime contracts. It does not claim to close
Runtime59's remaining scheduler, shutdown, product-integration, or diagnostics work.

## Implementation

`fence_prerequisite_failure` previously constructed a new `HashSet` for every root prerequisite.
Recursive evaluation removes every inserted ticket ID before returning, so one set can safely be
created for the scan and cleared at each root boundary. The optimized path retains that explicit
clear while reusing the set's allocation and capacity.

The regression compares retired and optimized results for successful, failed, and superseded-chain
inputs. A source contract rejects reintroduction of a per-root fresh set and requires one creation
plus the root-boundary clear.

## Performance Contract

| Evidence | Retired path | Optimized gate |
| --- | ---: | ---: |
| Visiting-set constructions per 4,096-root successful scan | 4,096 | 1 |
| Alternating release benchmark | 11 samples x 64 scans | optimized P95 <= 75% of retired P95 |

The benchmark emits `RUNTIME59_REUSED_FENCE_VISITING_SET_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/prerequisite counts, and retired/optimized construction
counts.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped `git diff --check`, and the production source guards passed
before submission (apart from the repository's existing CRLF notice). One managed Runtime batch
covers retired/optimized behavioral equivalence, the single-set source contract, and the ignored
release benchmark. Dynamic P95 evidence, integration SHA, and automatic WeCom performance delivery
remain coordinator-owned and pending.
