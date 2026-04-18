---
related_code:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/build.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu_hierarchy.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/build.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu_hierarchy.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
plan_sources:
  - user: 2026-04-18 更完整的 screen-probe hierarchy / RT hybrid lighting 或 Virtual Geometry unified indirect / residency-manager cascade
  - user: 2026-04-18 后续任务需要列出所有 tasks 写入 todo 然后执行
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-scene-driven-probe-gather.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-gpu-hierarchy-completion-continuation.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-descendant-request-frontier.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu_hierarchy.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_hierarchy
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_visibility
  - cargo test -p zircon_graphics --offline --locked visibility
  - cargo test -p zircon_graphics --offline --locked hybrid_gi
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Primary-Lineage Gather And Lineage Budgeting

**Goal:** 把 `Hybrid GI` 再往更完整的 scene-driven screen-probe hierarchy 推一层，补上两条之前仍然缺失的 hierarchy 行为：

- pending probe 在只有 hierarchy lineage、没有直接 spatial overlap 时，仍然能继承 primary resident ancestor 的 radiance
- request budgeting 在多个 active lineages 竞争有限 `probe_budget` 时，先给每条 lineage 一个首轮 descendant request，再进入同一 lineage 的第二轮 refine

## Delivered Slice

- `update_completion.wgsl` 现在不再把 primary resident ancestor 的 lineage-only gather 权重固定成空值。
- 当 pending probe 通过 nonresident hierarchy gap 才连到 resident ancestor，而当前帧又没有足够的 spatial overlap 可直接 gather 时，GPU completion 现在仍会把 primary resident ancestor 的 `previous_irradiance_rgb` 混进 pending probe update。
- 这条主 lineage 仍然不会压死更远 ancestor continuation：
  - primary resident ancestor 现在只提供一条较轻的 lineage continuation
  - farther resident ancestor 仍然可以通过 secondary lineage continuation 把自己的 radiance 拉回 pending probe
- `build_hybrid_gi_plan(...)` 的 request 预算分发也不再只是简单的全局 probe 排序。
- 现在会先按 active frontier 拆成多组 request candidates：
  - resident active probe 对应整条 visible nonresident descendant lineage
  - nonresident active probe 对应它自己的 request group
- 每组内部仍然按现有 scene-driven 规则排序：
  - local trace support
  - ancestor trace-lineage support
  - hierarchy depth specificity
  - ray budget
  - stable probe id
- 但真正进入 `requested_probe_ids` 之前，会先做 “按 lineage 分轮次” 的 interleave：
  - 第一轮先拿每条 active lineage 的最佳 candidate
  - 第二轮才开始拿各 lineage 的第二个 candidate
  - 每轮内部仍继续按 request sort key 排序

## Why This Slice Exists

- 之前的 hierarchy GPU completion 已经支持：
  - nearest resident ancestor continuation
  - farther resident ancestor continuation
  - ancestor-driven RT-lighting continuation
- 但 primary resident ancestor 的 lineage-only radiance gather 仍然有一个明显空洞：
  - 只要 spatial gather 覆盖不到它
  - 且 secondary ancestor 也不存在
  - pending probe 就会直接退回局部 neutral trace 结果
- 同时 request 层虽然已经支持 descendant frontier 和 ancestor-lineage scoring，但 budget 裁剪还是全局排序：
  - 一条强 lineage 可以连续吃掉多个 request 槽位
  - 另一条 active lineage 即使也有有价值的 descendant，也可能一整帧拿不到首轮 refine 机会
- 这两条空洞叠在一起，会让 “screen-probe hierarchy” 虽然在 score 上存在，却还没有真正形成更稳定的 hierarchy-aware request/update 主链。

## Validation Summary

- `hybrid_gi_gpu_completion_readback_inherits_primary_resident_ancestor_radiance_without_spatial_overlap`
  - 证明当 pending probe 只能靠 hierarchy lineage 连到 resident ancestor 时，GPU completion 不会再退化成纯 neutral trace
- `hybrid_gi_gpu_completion_readback_blends_farther_resident_ancestor_radiance_beyond_nearest_resident_parent`
  - 证明 primary lineage continuation 加回后，没有把既有的 farther ancestor continuation 压坏
- `hybrid_gi_gpu_hierarchy`
  - 证明 primary lineage gather、nearest/farther ancestor continuation、ancestor RT-lighting continuation 仍然一起成立
- `visibility_context_spreads_hybrid_gi_probe_budget_across_active_lineages_before_second_descendant_request`
  - 证明 request budgeting 现在会先把预算铺到不同 active lineages，而不是让单条 lineage 连续拿走多个 descendant request
- `hybrid_gi_visibility`
  - 证明新的 lineage budgeting 没有破坏 descendant frontier、ancestor-trace-lineage scoring、newly resident hysteresis 与 merge-back child hysteresis
- `visibility`
  - 证明 Hybrid GI 这轮 request 预算调整没有破坏统一 visibility 主链，也没有影响 Virtual Geometry 现有 visibility coverage
- `hybrid_gi`
  - 证明 visibility request、runtime host、GPU completion、resolve 仍然保持闭环

## Remaining Route

- `Hybrid GI` 仍然还没完成更完整的 hierarchy-aware resolve / probe gather / RT hybrid lighting 主链，尤其是更深 screen-probe hierarchy、更多 lineage continuation 层次，以及更 scene-driven 的 trace coupling。
- `Virtual Geometry` 仍然保留更值得继续推进的主链：
  - unified indirect ownership 的进一步下沉
  - deeper cluster raster consumption
  - residency-manager cascade / split-merge continuation
