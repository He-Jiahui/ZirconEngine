---
related_code:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_pending_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/probe_quantization.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu_hierarchy.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_new/layouts/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_new/scene_bind_group_bundle/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/new/bind_group_layout/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/new/params_buffer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/new/pipeline/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/buffer_helpers/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/gpu_pending_request_input/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/virtual_geometry_uploader_params/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/virtual_geometry_gpu_resources/virtual_geometry_gpu_resources.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_pending_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/probe_quantization.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu_hierarchy.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_new/layouts/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_new/scene_bind_group_bundle/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/new/bind_group_layout/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/new/params_buffer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/new/pipeline/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/buffer_helpers/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/gpu_pending_request_input/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/virtual_geometry_uploader_params/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/virtual_geometry_gpu_resources/virtual_geometry_gpu_resources.rs
plan_sources:
  - user: 2026-04-18 剩余更值得继续推进的主链是更完整的 scene-driven screen-probe hierarchy / probe gather / RT hybrid lighting
  - user: 2026-04-18 continue M5
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-rt-lighting-screen-probe-hierarchy.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-hierarchy-aware-radiance-gather.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-hierarchy-gap-resolve-and-rt-inheritance.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu_hierarchy.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_completion_readback_prefers_resident_ancestor_radiance_through_nonresident_hierarchy_gap
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_completion_readback_inherits_ancestor_trace_rt_lighting_through_nonresident_hierarchy_gap
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_hierarchy
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu
  - cargo test -p zircon_graphics --offline --locked hybrid_gi
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI GPU Hierarchy Completion Continuation

**Goal:** 把 `Hybrid GI` 的 hierarchy-aware 行为真正推进到 pending probe 的 GPU completion，而不是继续只停在 direct parent gather 或 resolve 侧 ancestor tint。

**Non-Goal:** 本轮仍然不实现完整的 screen-probe relocation、probe hierarchy page table、surface cache、hardware RT tracing，或真正的 Lumen-like scene representation。

## Delivered Slice

- `GpuPendingProbeInput` 现在除了 `parent_probe_id` 之外，还会携带：
  - `resident_ancestor_probe_id`
  - `resident_ancestor_depth`
- `probe_quantization.rs` 会根据 extract 里的完整 `parent_probe_id` chain 与当前 `HybridGiPrepareFrame.resident_probes` 集合，稳定算出 pending probe 最近的 resident ancestor；中间即使夹着 nonresident probe，也不会把 hierarchy 关系截断。
- `pending_probe_inputs(...)` 会把这份 resident-ancestor 元数据继续下沉到 GPU completion 的 storage buffer。
- `update_completion.wgsl` 的 pending path 现在新增两条 hierarchy-gap-aware continuation：
  - resident gather 不再只看 direct parent/child；当 pending probe 的最近 resident ancestor 跨过 nonresident gap 才连到当前 hierarchy 时，这个 ancestor 仍然会得到显式 gather boost。
  - ancestor RT-lighting continuation 不再只看 `active_trace_count()` 允许执行的本地 trace 数；pending probe 会沿最近 resident ancestor 重新评估整条 scheduled trace-region 集合里的 traced contribution，再按 hierarchy depth 衰减后并回自己的 traced contribution。
- 这意味着两类此前断裂的场景现在都进入了真实 GPU 路径：
  - `300 -> 250(nonresident) -> 200(resident)` 这类 hierarchy gap 不会让 pending probe 的 radiance gather 退化成 flat nearby-resident blend。
  - 当本地 tracing budget 只够执行部分 scheduled trace region 时，pending probe 仍然能继承 ancestor 覆盖到的 RT-lighting tint，而不是被本地 active trace 截断。

## Why This Slice Exists

- 之前已经完成了三层基线：
  - direct parent/child radiance gather
  - resolve 侧 hierarchy-gap lineage weight
  - ancestor-derived RT tint inheritance
- 但 pending probe 的 GPU completion 还留着一个明显空洞：
  - radiance-cache update 只知道 direct parent，不知道最近 resident ancestor
  - ancestor trace continuation 也会被 `active_trace_count()` 截断，只能看到本地 tracing budget 真正执行到的那几个 trace region
- 这会让当前 M5 路线在最关键的 request/update 闭环上仍然不完整：
  - visibility 已经会保留 hierarchy frontier
  - resolve 已经会穿过 nonresident gap
  - 但真正生成下一帧 probe history 的 GPU completion 还会在同样的 hierarchy gap 上退化
- 把这一层补齐后，当前主链已经收拢成：
  - hierarchy-aware visibility frontier
  - hierarchy-aware GPU radiance gather
  - hierarchy-gap-aware resolve
  - hierarchy-gap-aware pending completion RT continuation

## Support Fix

- 这轮还顺手修了一个 dirty worktree 下的编译闭包问题：
  - `scene_renderer_core_new`
  - `hybrid_gi::gpu_resources::new`
  - `virtual_geometry::gpu_resources`
    这些 folder-backed helper 子树里，有一批 nested module / helper item 的可见性只够父模块自己用，导致 sibling helper 在模块化拆分后无法再访问。
- 本轮没有把这些 helper 升成 crate public API，而是统一收口成 subtree-scoped `pub(in crate::scene::scene_renderer::...)` 可见性。
- 这样做的作用只有一个：保证 renderer 内部的 construct / execute helper 在继续模块化拆分后依然 compile-safe，不会反向污染 `RenderServer`、feature descriptor 或其它上层边界。

## Validation Summary

- `hybrid_gi_gpu_completion_readback_prefers_resident_ancestor_radiance_through_nonresident_hierarchy_gap`
  - 证明 pending probe 即使隔着 nonresident 中间层，也会更偏向最近 resident ancestor 的 radiance，而不再退化成 flat gather。
- `hybrid_gi_gpu_completion_readback_inherits_ancestor_trace_rt_lighting_through_nonresident_hierarchy_gap`
  - 证明当本地 tracing budget 只覆盖 neutral local region 时，pending probe 仍然能沿 resident ancestor 继承到 warmer RT-lighting tint。
- `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_hierarchy`
  - 证明两条新的 hierarchy-completion 红测都已经稳定转绿。
- `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu`
  - 证明 direct parent gather、scene-light tint、normalized multi-region gather、no-trace fallback 等既有 GPU completion 行为没有回归。
- `cargo test -p zircon_graphics --offline --locked hybrid_gi`
  - 证明 visibility、runtime host、GPU completion、resolve 的整条 Hybrid GI 主链仍然闭合。
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明新的 pending input contract、shader layout 与 helper-module 可见性收口没有留下 crate 编译缺口。

## Remaining Route

- 把当前“最近 resident ancestor”继续推进到更完整的 scene-driven screen-probe hierarchy，而不是只停在单条 ancestor chain continuation。
- 把 ancestor-derived RT continuation 从 trace-region tint inheritance 继续推进到更完整的 probe gather / scene representation / RT hybrid lighting。
- 如果切回 `Virtual Geometry`，下一条主链仍然是 visibility-owned unified indirect / deeper cluster raster / residency-manager cascade。
