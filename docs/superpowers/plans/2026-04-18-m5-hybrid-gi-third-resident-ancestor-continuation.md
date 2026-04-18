---
related_code:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_pending_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/probe_quantization.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu_hierarchy.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_pending_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/probe_quantization.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu_hierarchy.rs
plan_sources:
  - user: 2026-04-18 Virtual Geometry 的 deeper unified indirect / cluster raster / residency-manager cascade，以及 Hybrid GI 的更完整 scene-driven screen-probe hierarchy / RT hybrid lighting continuation
  - user: 2026-04-18 剩余更值得继续推进的主链是更完整的 scene-driven screen-probe hierarchy / probe gather / RT hybrid lighting
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-multi-ancestor-completion-continuation.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu_hierarchy.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_completion_readback_inherits_third_resident_ancestor_trace_rt_lighting
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_completion_readback_gathers_third_resident_ancestor_radiance
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_hierarchy
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Third Resident Ancestor Continuation

**Goal:** 把 `Hybrid GI` 的 GPU completion 从“两层 resident ancestor continuation”推进到“三层 resident ancestor continuation”，让更深 screen-probe hierarchy 的 radiance gather 与 RT trace inheritance 真正进入 scene-driven radiance-cache update。

**Non-Goal:** 本轮仍不实现无上限 ancestor 链上传、screen-probe page table、surface cache 或完整 hardware RT scene representation。

## Delivered Slice

- `GpuPendingProbeInput` 现在新增：
  - `resident_tertiary_ancestor_probe_id`
  - `resident_tertiary_ancestor_depth`
- `probe_quantization.rs` 的 `probe_resident_ancestors(...)` 现在继续沿 `parent_probe_id` 链提取前三层 resident ancestor，而不再在 secondary ancestor 停止。
- `pending_probe_inputs.rs` 现在会把第三层 resident ancestor 一并编码进 GPU pending update 输入。
- `update_completion.wgsl` 的 pending probe path 现在同时把 tertiary ancestor 接进两条 continuation：
  - `gathered_lineage_resident_rgb(...)` 的更深层 radiance lineage gather
  - `traced_contribution_rgb_with_resident_ancestors(...)` 的更深层 RT trace inheritance
- 这意味着 pending probe 即使经过 `resident -> resident -> resident -> nonresident -> pending` 的更深 hierarchy，也不会再在第二层 resident ancestor 截断。

## Why This Slice Exists

- 之前的 multi-ancestor continuation 已经把 GPU completion 从“只认最近 resident ancestor”推进到“primary + secondary resident ancestor”。
- 但 post-process resolve helper 已经沿完整 ancestor 链遍历，而 GPU completion 仍然只能落两层 resident lineage，导致 runtime/GPU source 仍比 resolve 侧更浅一层。
- 这个缺口会直接体现在两类场景：
  - 第三层 resident ancestor 才持有明显暖色 radiance history
  - 第三层 resident ancestor 才覆盖 warm RT trace region，而 nearer resident ancestors 只看到 local neutral trace

## Validation Summary

- `hybrid_gi_gpu_completion_readback_inherits_third_resident_ancestor_trace_rt_lighting`
  - 证明 pending probe 现在会继续继承第三层 resident ancestor 的 warm RT tint，而不再只在 primary / secondary ancestor 处截断。
- `hybrid_gi_gpu_completion_readback_gathers_third_resident_ancestor_radiance`
  - 证明 pending probe 现在会继续 gather 第三层 resident ancestor 的 radiance history，即使 nearer resident ancestors 保持中性。
- `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_hierarchy`
  - 证明第三层 continuation 没有破坏已有的 hierarchy gap、farther ancestor、primary-lineage 等 GPU hierarchy regression。
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明新的 Rust/WGSL struct 对齐与 prepare path 编码保持 compile closure。

## Remaining Route

- 把当前“三层 resident ancestor continuation”继续推进到更完整的 scene-driven hierarchy source，例如可扩展 ancestor ring、显式 lineage budgeting、screen-probe hierarchy cache。
- 继续把同一条 hierarchy truth 接到 resolve 权重、runtime host cache policy 和更完整的 RT hybrid lighting continuation 上。
