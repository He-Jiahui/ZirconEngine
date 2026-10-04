---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-hzb-bind-group-hash-lru.md
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-instance-upload-hash-membership.md
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-frame-batching-hash-entity-sets.md
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-visible-spatial-query-hash-dedup.md
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-bvh-update-hash-index.md
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-virtual-geometry-page-priority-hash-aggregation.md
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-virtual-geometry-hash-membership.md
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-particle-upload-linear-difference.md
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-particle-history-hash-dedup.md
  - docs/plans/optimize/zircon_runtime/09b/2026-08-27-dirty-proportional-static-index-update.md
  - docs/plans/optimize/zircon_runtime/09b/2026-08-27-single-allocation-parallel-frustum.md
  - docs/plans/optimize/zircon_runtime/09b/2026-08-27-indexed-virtual-geometry-execution-projection.md
  - docs/plans/optimize/zircon_runtime/09b/2026-09-09-static-index-query-candidate-normalization.md
  - docs/plans/optimize/zircon_runtime/09b/2026-09-09-disabled-directional-shadow-view-admission.md
  - docs/plans/optimize/zircon_runtime/213-runtime-visibility-gpu-scene-culling-batching-instancing-hzb-virtual-geometry-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/625/2026-09-01-preallocated-frame-batching-collections.md
---

# Runtime09b visibility 优化计划完成列表

## 计划完成列表

| 批次 | 优化内容 | 状态 | 记录 |
| --- | --- | --- | --- |
| Runtime948 | HZB bind-group 命中路径合并为一次可变 HashMap 探测 | `implemented_pending_validation` | [Runtime948](948-runtime09b-hzb-bind-group-hash-lru.md) |
| Runtime949 | 实例上传动态/dirty stable-key HashSet membership | `implemented_pending_validation` | [Runtime949](949-runtime09b-instance-upload-hash-membership.md) |
| Runtime950 | 帧批处理 EntitySet 懒 Hash membership 与单调序列快路径 | `implemented_pending_validation` | [Runtime950](950-runtime09b-frame-batching-entity-set-fast-path.md) |
| Runtime951 | 可见空间查询候选保序去重与实体 ID 单次归一化 | `implemented_pending_validation` | [Runtime951](951-runtime09b-visible-spatial-query-fast-path.md) |
| Runtime952 | 有序 BVH snapshot 双指针增量合并 | `implemented_pending_validation` | [Runtime952](952-runtime09b-bvh-update-hash-index.md) |
| Runtime953 | 虚拟几何 page-priority dense/sparse aggregation | `implemented_pending_validation` | [Runtime953](953-runtime09b-virtual-geometry-page-priority-hash.md) |
| Runtime954 | 虚拟几何 requested/hot-resident HashSet membership | `implemented_pending_validation` | [Runtime954](954-runtime09b-virtual-geometry-hash-membership.md) |
| Runtime955 | 粒子上传 current/previous 双指针差分 | `implemented_pending_validation` | [Runtime955](955-runtime09b-particle-upload-linear-difference.md) |
| Runtime956 | 粒子历史大批量 Vec sort/dedup 快路径 | `implemented_pending_validation` | [Runtime956](956-runtime09b-particle-history-hash-dedup.md) |
| Runtime958 | changed-key HashMap 驱动的 dirty-proportional static-index update | `implemented_pending_validation` | [Runtime958](958-runtime09b-dirty-proportional-static-index-update.md) |
| Runtime959 | direct-output parallel frustum，移除中间 work-item projection | `implemented_pending_validation` | [Runtime959](959-runtime09b-direct-output-parallel-frustum.md) |
| Runtime960 | virtual-geometry execution/selection shared indexed projection | `implemented_pending_validation` | [Runtime960](960-runtime09b-indexed-virtual-geometry-execution-projection.md) |
| Runtime961 | prepared local bounds affine projection、fail-open 与 extraction capacity | `implemented_pending_validation` | [Runtime961](961-runtime09b-prepared-local-bounds-projection.md) |
| Runtime213/09B | static-index query Vec normalization 与 disabled directional shadow admission | `implemented_pending_validation` | [Runtime22](22-visibility-query-normalization.md)、[Runtime21](21-visibility-shadow-view-admission.md) |

本列表保留 `implemented_pending_validation`，因为当前源包级 managed Cargo、Release
P95、allocator 与 product gates 仍按集中日志异步等待；tooling 迁移按用户要求暂缓。
