---
related_code:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_pending_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/probe_quantization.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu_hierarchy.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/post_process/execute_post_process_stack.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/runtime_prepare/execute_runtime_prepare_passes.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/prepare_overlay_buffers.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/decode/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/decode/cache_entries.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/decode/completed_probe_ids.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/decode/completed_trace_region_ids.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/decode/probe_irradiance_rgb.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/collect.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/hybrid_gi_gpu_pending_readback.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback/decode/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback/decode/completed_page_assignments.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback/decode/page_table_entries.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback/pending_readback/collect.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback/pending_readback/virtual_geometry_gpu_pending_readback.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_pending_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/probe_quantization.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu_hierarchy.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/post_process/execute_post_process_stack.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/runtime_prepare/execute_runtime_prepare_passes.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/prepare_overlay_buffers.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/decode/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/decode/cache_entries.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/decode/completed_probe_ids.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/decode/completed_trace_region_ids.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/decode/probe_irradiance_rgb.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/collect.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/hybrid_gi_gpu_pending_readback.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback/decode/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback/decode/completed_page_assignments.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback/decode/page_table_entries.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback/pending_readback/collect.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback/pending_readback/virtual_geometry_gpu_pending_readback.rs
plan_sources:
  - user: 2026-04-18 剩余更值得继续推进的主链是更完整的 scene-driven screen-probe hierarchy / probe gather / RT hybrid lighting
  - user: 2026-04-18 Hybrid GI 的 hierarchy-aware resolve / deeper screen-probe hierarchy / RT hybrid lighting continuation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-gpu-hierarchy-completion-continuation.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-descendant-request-frontier.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu_hierarchy.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_completion_readback_blends_farther_resident_ancestor_radiance_beyond_nearest_resident_parent
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_completion_readback_inherits_farther_resident_ancestor_trace_rt_lighting_beyond_nearest_resident_parent
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_hierarchy
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu
  - cargo test -p zircon_graphics --offline --locked hybrid_gi
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Multi-Ancestor Completion Continuation

**Goal:** 把 `Hybrid GI` 的 GPU completion 从“pending probe 只认最近 resident ancestor”推进到“pending probe 还能继续消费更远 resident ancestor 的 radiance / RT lineage”，让 scene-driven screen-probe hierarchy 在 update 阶段继续向多层 ancestor chain 收敛。

**Non-Goal:** 本轮仍然不实现完整 probe relocation、surface cache、hardware RT scene representation，或真正的 screen-probe hierarchy page table。

## Delivered Slice

- `GpuPendingProbeInput` 现在不再只编码最近 resident ancestor，而是继续携带两级 resident ancestor continuation：
  - `resident_ancestor_probe_id / resident_ancestor_depth`
  - `resident_secondary_ancestor_probe_id / resident_secondary_ancestor_depth`
- `probe_quantization.rs` 会沿完整 `parent_probe_id` chain 继续向上扫描，稳定提取前两层 resident ancestor，而不是在遇到第一层 resident probe 时就截断。
- `update_completion.wgsl` 的 pending path 现在分成三段 hierarchy-aware gather：
  - 现有 spatial resident gather 仍然保留 direct parent / nearest resident ancestor 的本地权重。
  - 新增显式 lineage gather，只负责把更远 resident ancestor 的 `previous_irradiance_rgb` 作为补充贡献混进 pending probe 的 gathered result，而不重复放大最近 resident parent。
  - ancestor trace continuation 现在同样会顺序评估 primary / secondary resident ancestor，对跨多层 resident lineage 的 traced contribution 继续做 RT-lighting inheritance。
- 这意味着两类此前仍不完整的场景现在进入真实 GPU 路径：
  - 当最近 resident parent 偏冷、但更远 resident ancestor 保留更暖 radiance-cache history 时，pending probe 不再只被最近 parent 吞没。
  - 当最近 resident parent 只覆盖本地 neutral trace，而更远 resident ancestor 才真正覆盖 warm RT region 时，pending probe 仍会沿更深 lineage 继承这份 traced tint。

## Why This Slice Exists

- 上一轮已经把 hierarchy-aware completion 推到“最近 resident ancestor continuation”：
  - pending probe 能跨 nonresident hierarchy gap 找到第一层 resident ancestor
  - ancestor RT-lighting continuation 不再被 `active_trace_count()` 截断
- 但这仍然留下一个更深层的缺口：
  - ancestor chain 一旦跨过第一层 resident probe，后续 resident lineage 就再次丢失
  - 这会让 pending probe 在“冷 parent / 暖 grandparent”或“neutral parent / warm farther ancestor RT region”这类 screen-probe hierarchy 场景里退回近邻单层逻辑
- 把第二层 resident ancestor continuation 接上之后，当前主链进一步收敛成：
  - visibility request 能继续追更深 descendant
  - GPU completion 能继续沿两层 resident ancestor chain 延续 radiance 与 traced tint
  - resolve 继续保留 ancestor/descendant lineage 与 inherited RT baseline

## Support Fix

- 本轮还顺手补了当前 dirty worktree 下的 compile-closure 缺口：
  - `scene_renderer_core_render_compiled_scene/*`
  - `hybrid_gi/gpu_readback/*`
  - `virtual_geometry/gpu_readback/*`
- 这些改动只做模块可见性和 helper wiring 收口：
  - 让 folder-backed helper 子树继续能从 sibling helper 调用
  - 不扩大 `RenderServer`、runtime host 或 feature 对外 API

## Validation Summary

- `hybrid_gi_gpu_completion_readback_blends_farther_resident_ancestor_radiance_beyond_nearest_resident_parent`
  - 证明 pending probe 不再只吃最近 resident parent 的 radiance，而会继续把更远 resident ancestor 的 warm irradiance 混进 completion readback。
- `hybrid_gi_gpu_completion_readback_inherits_farther_resident_ancestor_trace_rt_lighting_beyond_nearest_resident_parent`
  - 证明当最近 resident parent 只覆盖 neutral local trace 时，pending probe 仍能沿更远 resident ancestor 继承 warmer RT-lighting continuation。
- `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_hierarchy`
  - 证明新的两级 resident ancestor radiance / RT continuation 红测都已经转绿。
- `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu`
  - 证明 direct parent gather、single-ancestor hierarchy gap continuation、scene-light tint、normalized multi-region gather 与 no-trace fallback 都没有回归。
- `cargo test -p zircon_graphics --offline --locked hybrid_gi`
  - 证明 visibility、runtime host、GPU completion、resolve 的整条 Hybrid GI 主链仍然闭环。
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明本轮补上的 compiled-scene / gpu-readback helper 可见性收口没有留下新的 crate 编译缺口。

## Remaining Route

- 把当前“两级 resident ancestor continuation”继续推进到更完整的 scene-driven screen-probe hierarchy，例如更多层 lineage budgeting、screen-probe hierarchy gather policy，或更显式的 scene-driven probe hierarchy cache。
- 把 `Hybrid GI` 的 RT continuation 从 ancestor-traced tint inheritance 继续推进到更完整的 RT hybrid lighting / scene representation。
- 如果切回 `Virtual Geometry`，下一条主链仍然是 visibility-owned unified indirect / deeper cluster raster / residency-manager cascade。
