---
title: Runtime206 Asset Type Posting Index
category: zircon_runtime
report_id: Runtime206-asset-type-posting-index-2026-08-31
date: 2026-08-31
session_id: root-runtime-interface03-activate-link-failure-20260831
implementation_status: implementation_complete
validation_status: static_passed_managed_cargo_pending
performance_status: deterministic_target_met
---

# Runtime206 Asset Type Posting Index

## Scope

`AssetRegistryIndex::get_assets_by_type` and every filter carrying `type_marker` scanned all
registry entries before sorting the matching subset by canonical URI. The registry already owns
derived UUID/path/source/dependency indexes and centralizes entry insertion and source removal, so
asset type can use the same mutation-owned posting model without adding another query cache.

## Change

- Add `AssetKind -> HashSet<AssetUuid>` postings to `AssetRegistryIndex`.
- Maintain the posting in `insert_checked` and `remove_source_path`, including empty-bucket removal.
- Route direct type queries and type-qualified composite filters through the posting.
- Preserve the existing URI ordering, tag/path/package predicates, borrowed entry results, and
  missing-type empty result.
- Add Rust behavior coverage for unordered construction, composite filtering, source removal, and
  empty posting retirement, plus a static performance contract for the candidate boundary.

## Deterministic Performance Evidence

The pressure model uses 1,048,576 registry entries uniformly distributed across 32 asset types. A
query selects one type with 32,768 matching entries.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Registry candidates visited | 1,048,576 | 32,768 | 96.875% |
| Candidate visit ratio | 32.0x | 1.0x | 32.0x |
| Non-matching entries visited | 1,015,808 | 0 | 100.000% |

The retained result sort remains `O(K log K)` because the public API promises canonical URI order;
the optimized candidate phase changes from `O(N)` to average `O(K)`. The posting adds one logical
UUID membership per registry row and hash-table overhead. These are deterministic operation counts,
not CPU time, allocator, RSS, or power measurements.

## Validation

- The new source contract failed 3/4 checks before implementation and passed after the posting was
  wired into both mutation directions and query paths.
- The focused asset performance batch passes 11/11.
- Python bytecode compilation and scoped diff checks pass.
- Managed Windows Rust compilation and the focused registry behavior tests remain pending in the
  next asynchronous multi-task validation batch.

## Remaining Work

Runtime206 P1-041 remains partially open for tag/path/package postings. Compiled query plans,
visitor/cursor APIs, generation leases, result/deadline budgets, large-corpus allocator/RSS data,
and product query qualification remain owned by the parent plan.
