---
title: Runtime navigation fallback path deduplication in place
category: zircon_runtime
report_id: Runtime861-navigation-path-dedup-in-place-2026-09-20
date: 2026-09-20
session_id: root-runtime-editor-async-optimization-20260920
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime861 Navigation Fallback Path Deduplication In Place

## Finding

The baked-mesh fallback query already built its path points in a caller-owned
vector, but `deduplicate_path_points` copied every retained point into a new
zero-capacity result vector. Dense paths therefore paid a second allocation
growth sequence even though the input vector had exactly the required storage.

## Optimization

`deduplicate_path_points` now uses `Vec::dedup_by` on the input vector and
returns that vector after in-place compaction. The distance predicate remains
the same XZ threshold (`0.05`), so adjacent duplicate removal keeps the first
point, preserves path order, and retains the input allocation. No navigation
authority, fallback ordering, or off-mesh metadata semantics change.

## TDD and deterministic evidence

The Python source/model contract was intentionally run RED before the
production change, then GREEN at `4/4`. The lower Rust tests cover capacity
retention for `4,096` unique points and the existing adjacent-threshold
semantics; the ignored managed marker is
`RUNTIME861_NAVIGATION_PATH_DEDUP_IN_PLACE_BENCH_V1`. Its simple geometric
model changes the legacy zero-capacity output collector from `11` modeled
growth events to `0` in-place output growth events.

## Local validation boundary

- Exact-file Rustfmt and Python compilation pass for the changed Rust and
  source-contract files.
- The focused navigation source/model batch runs as one process and passes
  `31/31` tests in `0.033s`, with zero failures, errors, or skips; its receipt
  is recorded in the linked Astra feature and async admission log.
- The refreshed one-process explicit performance-or-contract loader covers
  `969` non-tooling files and passes `4100/4100` tests in `225.799s`, with zero
  load errors, failures, errors, or skips. Two shader-prewarm Cargo command
  lines printed by fixture tests are local fixture output, not managed Windows
  Release/Cargo acceptance.
- After Runtime862 extracted the shared vertex-projection helper and the
  Runtime08d lower contract was adapted to that owner, the expanded navigation
  batch passes `38/38` in `0.139s`; the refreshed broad loader covers `970`
  files and passes `4104/4104` in `228.769s`, again with zero failures, errors,
  load errors, or skips.
- Local source/model receipts do not establish Rust compilation, allocator
  behavior, or product navigation latency percentiles.
- Tooling production remains deferred for the later Rust migration.

On 2026-09-21, the same one-process non-tooling loader was rerun after the
record-integrity and document-structure audits and again passed `4104/4104`
across `970` files in `274.515s`, with zero failures, errors, load errors, or
skips. Fixture-emitted Cargo command lines remain non-acceptance output.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/navigation/runtime/baked_mesh.rs` | `0ECB4B874765D5CE51C88283126A54A497B81D63D423DBD5213F6D680493E558` |
| `tools/tests/test_runtime861_navigation_path_dedup_in_place_performance_contract.py` | `3B490E77A530CCA5C5DF96B6E1A8366C6538A1A8ED265605F55E2646035A1B5F` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed managed Windows Release lane compiles the current tree,
executes the lower regression and ignored marker, and supplies allocator plus
navigation fallback product p50/p95/p99 evidence. Do not infer product
acceptance from the local source/model receipts.
