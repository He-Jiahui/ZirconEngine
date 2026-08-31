---
title: Runtime99i Contiguous Archetype Transition Validation
category: zircon_runtime
report_id: Runtime99i-contiguous-transition-validation-2026-08-27
date: 2026-08-27
session_id: root-runtime99i-contiguous-transition-validation-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime99i Contiguous Archetype Transition Validation

## Scope

This slice reduces allocation and ordered-tree work in dense archetype row transition preflight.
It advances Runtime99i M2 structural-delta work without changing component storage, row
publication, sparse values, public signatures, or transition error precedence.

## Change

- Replace the temporary `BTreeSet<ComponentId>` with one normalized contiguous component-ID
  buffer.
- Validate inserted values before mutation and reserve only component IDs that are genuinely new.
- Apply the ordered delta with binary search while preserving idempotent insert/remove behavior.
- Compare the final and target sorted schemas linearly, retaining the prior rule that an
  unexpected component is reported before a missing target component.
- Preserve arbitrary source iterator order and duplicate normalization at the existing generic
  API boundary.

## Deterministic Performance Evidence

Independent optimized Rust 1.94.1 model, 8,192 source component IDs, 128 ordered changes, 96
allocation repetitions, 31 alternating timing samples, and 8 operations per timing sample:

| Metric | `BTreeSet` transition | Contiguous transition | Reduction |
|---|---:|---:|---:|
| allocations | 72,960 | 192 | 99.74% |
| allocated bytes | 14,105,856 | 9,437,184 | 33.10% |
| P50 | 287,812 ns | 62,387 ns | 78.32% |
| P95 | 343,125 ns | 113,575 ns | 66.90% |

The executable model requires at least 99% fewer allocations, at least 30% fewer allocated
bytes, and at least 40% lower P95. The stable checksum is `372548482738675712`.

## Acceptance

- Rust regressions cover arbitrary source order, duplicate normalization, idempotent membership
  updates, and unexpected-before-missing error precedence.
- The ignored release benchmark emits
  `RUNTIME99I_CONTIGUOUS_TRANSITION_VALIDATION_BENCH_V1` and requires optimized P95 to remain at
  or below 60% of the tree implementation.
- The Python source contract rejects tree materialization and requires real-insertion capacity,
  ordered delta application, linear schema checks, and the recorded performance evidence.
- Exact-file Rust 1.94.1 formatting, focused ECS library tests, source contracts, the independent
  model, and scoped diff checks are submitted as one coordinator validation batch.

## Remaining Parent-Plan Work

This slice does not close Runtime99i's ECS kernel qualification. World/schema identity, query
borrowing, real World-backed parallel execution, bounded lifecycle channels, transaction
admission, Miri/sanitizer coverage, product-scene benchmarks, and long-running mutation soak
remain open under the parent plan.
