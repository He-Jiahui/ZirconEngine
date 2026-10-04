---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/169/2026-09-20-navigation-path-dedup-in-place.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/navigation/runtime/baked_mesh.rs
tests:
  - tools/tests/test_runtime861_navigation_path_dedup_in_place_performance_contract.py
---

# Runtime861 Navigation Fallback Path Deduplication In Place

## Completion entry

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime baked-mesh fallback path | Compact the existing `Vec<NavPathPoint>` with `dedup_by` instead of allocating a second output vector, preserving the `0.05` XZ threshold, first-point retention, order, and path metadata. | Intentional RED/GREEN source-model contract `4/4`; lower `4,096`-point capacity-retention and duplicate-semantics regressions plus ignored `RUNTIME861_NAVIGATION_PATH_DEDUP_IN_PLACE_BENCH_V1` marker are wired. The deterministic model changes modeled legacy output growth `11→0`; exact Rustfmt and Python compilation pass. The batched navigation source-contract set passes `31/31` in `0.033s`; the refreshed broad loader passes `4100/4100` across `969` files in `225.799s`, with zero load errors/failures/errors/skips. Managed Cargo/Release, allocator, and navigation fallback p50/p95/p99 evidence remain pending. | implemented_pending_validation |

The later Runtime862 vertex-projection helper required a compatibility refresh
of the existing Runtime08d borrowed-index contract. That expanded navigation
batch passes `38/38` in `0.139s`; the current broad source-contract loader
passes `4104/4104` across `970` files in `228.769s`. These are local
source/model receipts and do not replace managed Cargo/Release or product
percentile evidence.

The 2026-09-21 rerun of the same one-process non-tooling loader again passes
`4104/4104` across `970` files in `274.515s`, with zero failures, errors, load
errors, or skips. Fixture Cargo strings are not managed acceptance evidence.
The focused Runtime08d/861/862 navigation batch also passes `38/38` in
`0.021s`, with zero failures, errors, or skips.

## Scope boundary

This slice changes only fallback path deduplication's allocation shape. It does
not change navigation authority, path-query status, point threshold semantics,
off-mesh metadata, or tooling production.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/navigation/runtime/baked_mesh.rs` | `0ECB4B874765D5CE51C88283126A54A497B81D63D423DBD5213F6D680493E558` |
| `tools/tests/test_runtime861_navigation_path_dedup_in_place_performance_contract.py` | `3B490E77A530CCA5C5DF96B6E1A8366C6538A1A8ED265605F55E2646035A1B5F` |

## Managed gate

No managed Windows Cargo/Release command is started locally and coordinator
status is not polled. Keep this entry `implemented_pending_validation` until
the combined owner-attributed Windows Release lane proves current-source
compilation, lower-test reachability, allocator behavior, and navigation
fallback product p50/p95/p99 evidence.
