---
title: Runtime04 Project Root Hash Validation
category: zircon_runtime
report_id: Runtime04-project-root-hash-validation-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Project Root Hash Validation

## Scope

This slice removes ordered-tree membership from project asset-root and UI-root uniqueness
validation. The validator still walks roots in manifest order, reports the first duplicate, and
performs the existing ordered pair overlap checks. The membership sets are private and not
published.

## Change

- Replace borrowed asset-root `BTreeSet<&str>` with `HashSet<&str>`.
- Replace UI-root `BTreeSet<String>` with `HashSet<String>`.
- Preserve scheme, label, empty-root, overlap, and first-error validation order.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique root IDs | Before | After |
|---|---:|---:|
| Membership class | O(log n) | average O(1) |
| Borrowed asset-root key allocations | 0 | 0 |
| First duplicate error | manifest ordered | unchanged |

The ignored release gate runs 17 alternating samples and emits
`RUNTIME04_PROJECT_ROOT_HASH_VALIDATION_BENCH_V1`. Acceptance requires hash validation P95 to be
at most 60% of ordered validation P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826ae_runtime04_hash_root_validation_preserves_first_duplicate_error`
  exercises a real project manifest with a repeated asset root.
- `optimization_batch_20260826ae_runtime04_project_root_validation_uses_hash_membership` requires
  both hash sets and rejects ordered membership.
- `optimization_batch_20260826ae_runtime04_project_root_hash_validation_performance_evidence`
  checks admission equivalence, reports both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Runtime04 still needs authoritative cook graph integration, typed artifact manifests, range-backed
mounting, bounded streaming decompression, signed package admission, and product-scale project
qualification. This slice only improves project root uniqueness validation.
