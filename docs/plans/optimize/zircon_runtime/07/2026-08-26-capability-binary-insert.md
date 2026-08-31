---
title: Runtime07 Capability Binary Insert
category: zircon_runtime
report_id: Runtime07-capability-binary-insert-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Capability Binary Insert

## Scope

This slice reduces repeated sorting while constructing script host function and module capability
descriptors. It preserves public vectors as ascending unique strings and repairs externally
populated unsorted/duplicate vectors before returning. It does not change capability identity,
authorization, descriptor publication, or plugin resolution.

## Change

- Share one sorted-unique insertion helper across function and module descriptor builders.
- Use binary search and indexed insertion when the existing vector is strictly sorted and unique.
- Retain a sort/dedup compatibility fallback for public or deserialized malformed vectors.
- Preserve exact case-sensitive `String` ordering and duplicate behavior.

## Deterministic Performance Evidence

| Representative 16,384 admissions / 1,024 unique capabilities | Before | After |
|---|---:|---:|
| Full-vector sort calls on normal builder path | 16,384 | 0 |
| Full-vector dedup calls on normal builder path | 16,384 | 0 |
| Strict sorted/unique invariant scans | 0 | 16,384 |
| Insertion-point search | full sort | binary search |
| Compatibility repair for malformed public vectors | sort + dedup | sort + dedup, unchanged |
| Published capability order | ascending unique | ascending unique |

The normal path still performs an O(n) strict-invariant scan and may shift `Vec` elements on a new
insertion; it is not claimed as a pure O(log n) builder. The ignored release gate alternates 17
old and new builder samples and emits `RUNTIME07_CAPABILITY_BINARY_INSERT_BENCH_V1`. Acceptance
requires optimized P95 to be at most 60% of legacy P95. Exact Windows timings remain pending.

## Acceptance

- `optimization_batch_20260826u_runtime07_capability_builders_remain_sorted_and_unique` covers
  both builders, duplicates, ordering, and malformed-vector compatibility repair.
- `optimization_batch_20260826u_runtime07_capability_builders_use_binary_insertion` requires two
  product callers, binary search, the invariant guard, and a single compatibility fallback.
- `optimization_batch_20260826u_runtime07_capability_binary_insert_performance_evidence` verifies
  identical output, emits both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Runtime07 still needs canonical capability/interface identity, immutable catalog generations,
version/source/trust resolution, backend lifecycle leases, script budgets and isolation, and
product-scale descriptor/call profiling.
