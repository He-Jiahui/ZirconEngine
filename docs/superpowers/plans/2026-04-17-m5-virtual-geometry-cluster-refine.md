---
related_code:
  - zircon_scene/src/components.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/mod.rs
  - zircon_graphics/src/tests/visibility.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
implementation_files:
  - zircon_scene/src/components.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/mod.rs
plan_sources:
  - user: 2026-04-17 continue next M5 slice after Virtual Geometry GPU uploader/readback
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-preprocess.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-gpu-uploader-readback.md
tests:
  - zircon_graphics/src/tests/visibility.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics visibility --locked
  - cargo test -p zircon_graphics virtual_geometry --locked
  - cargo test -p zircon_scene --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Cluster Refine

**Goal:** 在已有 `Virtual Geometry` preprocess、runtime host、prepare、GPU uploader/readback 之上，补一层最小可用的 cluster hierarchy/refine 规则，让可见 cluster 选择不再只是平铺排序截断，而是能在预算允许时用子 cluster 替换父 cluster。

**Non-Goal:** 本轮不实现完整的 Nanite-like cluster tree、split-merge hysteresis、screen-space threshold tuning、page streaming ownership 或 indirect raster。

## Delivered Slice

- `RenderVirtualGeometryCluster` 新增 `parent_cluster_id`，把 hierarchy 明确放进 extract 合同，而不是继续依赖隐式命名或外部约定。
- `build_virtual_geometry_plan(...)` 新增 budget-aware refine frontier：
  - 先找到所有当前可见的 hierarchy frontier root
  - 按 `screen_space_error -> lod_level -> cluster_id` 稳定排序
  - 对当前 frontier 中最高优先级且具备可见子 cluster 的节点，只有在 `frontier_len - 1 + child_count <= cluster_budget` 时才执行 `parent -> visible children` 替换
  - 循环直到没有更多可执行 refinement
- 这让 `Virtual Geometry` 第一次具备显式 parent-child refine 语义，同时仍然保持 deterministic、budget-gated、只发生在统一 visibility 计划层。

## Behavior Rules

- 子 cluster 可见并不意味着一定替换父 cluster；必须满足 refinement 后仍不超过 `cluster_budget`。
- 如果 children 过多导致预算无法容纳，parent 会继续保留在 visible frontier 里，作为 coarse fallback。
- refinement 只依赖当前帧可见 hierarchy，不引入跨帧 hysteresis 或 merge state。
- page request 计划始终跟随最终 refined frontier，因此：
  - children 替换父 cluster 时，请求 children 的 page，并把不再可见的 resident parent page 视为 evictable
  - budget 不够保留 parent 时，继续请求 parent page，不提前抬升 children page

## Validation Summary

- `visibility_context_refines_virtual_geometry_parent_cluster_into_visible_children_when_budget_allows`
  - 证明当前 budget 足够时，visible children 会替换 parent，并把 page request 转向 refined frontier
- `visibility_context_keeps_parent_virtual_geometry_cluster_when_children_exceed_budget`
  - 证明 children 超出预算时，系统会保留 parent，而不是产生一个被截断的半成品 refined frontier
- `cargo test -p zircon_graphics visibility --locked`
  - 证明整个 visibility 规划面没有被 hierarchy refinement 破坏
- `cargo test -p zircon_graphics virtual_geometry --locked`
  - 证明 refine 行为和已有 prepare/runtime/GPU uploader 基线兼容
- `cargo test -p zircon_scene --locked`
  - 证明新的 `RenderVirtualGeometryCluster.parent_cluster_id` 没有破坏 scene crate 的 extract/asset/world 合同

## Remaining Route

- hierarchy 的更深层 split-merge / hysteresis / SSE threshold policy
- cluster frontier 和 page residency manager 的联动
- refined frontier 对 indirect draw、occlusion、cluster raster 的真正消费；当前仍只有 prepare-driven fallback `draw_index_count + tint` consumption baseline
