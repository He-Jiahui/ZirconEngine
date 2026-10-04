---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/04/2026-09-20-asset-refresh-visual-path-capacity.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/app/assets/refresh.rs
  - zircon_editor/src/ui/retained_host/app/assets/refresh/capacity_tests.rs
tests:
  - tools/tests/test_editor_asset_refresh_visual_path_capacity_performance_contract.py
---

# Editor844 · asset refresh visual-path capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor retained asset refresh | Compute a saturating event upper bound and lazily reserve the visual locator vector on first real visual input; preserve zero-capacity non-visual batches, sprite-atlas/reconcile branches, sorting, deduplication, and `None` behavior. | TDD source/model contract `4/4`; lower lazy/empty/order regression and ignored `EDITOR844_ASSET_REFRESH_VISUAL_PATH_CAPACITY_BENCH_V1` marker are wired; deterministic `(8,7,6)` model removes `6→0` growth events. The focused Runtime/Editor loader passes `82/82` across `21` modules in `0.089s`; the broad non-tooling loader passes `2370/2370` across `648` modules in `4.907s`, with zero load errors/failures/errors/skips. Managed Cargo/Release, allocator, and asset-refresh product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Complexity boundary

Only the temporary visual-locator buffer allocation shape changes. Asset
refresh ownership, event ordering, cache invalidation policy, and tooling
production remain outside this slice.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/app/assets/refresh.rs` | `4084F51E10703D8BDF898F4EF62AA0005DCA9A67ED287F16BB7989181B7FA3FB` |
| `zircon_editor/src/ui/retained_host/app/assets/refresh/capacity_tests.rs` | `5783FC8D8287118250F28FF64579EC1D12D310CBCAD588172B5CFD0EE272957E` |
| `tools/tests/test_editor_asset_refresh_visual_path_capacity_performance_contract.py` | `7B72F9C641A5415B746678E35F689AB022A48CB2834F9C7886652881F52AED26` |

## Managed gate

No standalone Cargo process is started locally and coordinator status is not
polled. Keep this entry `implemented_pending_validation` until the combined
owner-attributed Windows Release lane proves current-source compilation,
lower-test reachability, allocator behavior, and asset-refresh product
p50/p95/p99 evidence.
