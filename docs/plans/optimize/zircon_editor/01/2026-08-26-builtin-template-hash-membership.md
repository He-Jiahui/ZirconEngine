---
title: Editor01 Builtin Template Hash Membership
category: zircon_editor
report_id: Editor01-builtin-template-hash-membership-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Builtin Template Hash Membership

## Scope

This slice reduces membership and duplicate-allocation cost while filtering builtin template
documents and traversing recursive widget/style imports. It preserves registration and final
`BTreeMap` output order but does not change synchronous file loading or compile-cache authority.

## Change

- Collect requested builtin document IDs into a borrowed `HashSet<&str>`.
- Track recursive widget/style imports in a `HashSet<String>`.
- Check a borrowed import reference before cloning it into the visited set.
- Preserve builtin document iteration, recursive first admission, and ordered import maps.

## Deterministic Performance Evidence

| 65,536 import admissions / 8,192 unique references | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered-set insert attempts | 65,536 | 0 | 100% removed |
| Hash membership probes | 0 | 65,536 | average O(1) |
| Owned reference allocations | 65,536 | 8,192 | 87.5% removed |
| Final import-map order | ascending key | ascending key | unchanged |

The ignored release gate alternates 17 ordered-admission and borrowed-hash-admission samples. It
emits `EDITOR01_BUILTIN_TEMPLATE_HASH_MEMBERSHIP_BENCH_V1`; acceptance requires hash-membership P95
to be at most 60% of ordered-membership P95. Exact Windows timings remain pending the coordinator
run.

## Acceptance

- `optimization_batch_20260826r_editor01_hash_membership_preserves_first_import_admission` covers
  first admission and duplicate rejection through the product helper.
- `optimization_batch_20260826r_editor01_builtin_templates_use_hash_membership` requires both
  document-filter and import-visited hash boundaries and rejects a production tree set.
- `optimization_batch_20260826r_editor01_builtin_template_hash_membership_performance_evidence`
  emits workload/allocation counts and both P95 values, then enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Editor01 still performs synchronous builtin/import file access and compilation on cache misses.
Generation-bound async loading, cancellation, last-good publication, retained-tree patching,
startup-scale qualification, and complete retained UI performance evidence remain open.
