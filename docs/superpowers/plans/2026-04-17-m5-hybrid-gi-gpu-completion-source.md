---
related_code:
  - zircon_graphics/src/backend/render_backend/read_buffer_u32s.rs
  - zircon_graphics/src/runtime/hybrid_gi.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_hybrid_gi.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/types.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
implementation_files:
  - zircon_graphics/src/backend/render_backend/read_buffer_u32s.rs
  - zircon_graphics/src/runtime/hybrid_gi.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_hybrid_gi.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/types.rs
plan_sources:
  - user: 2026-04-17 Hybrid GI next step should enter tracing/update completion source before radiance-cache lighting resolve
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m5-hybrid-gi-feedback-streaming.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - cargo test -p zircon_graphics hybrid_gi --locked
  - cargo test -p zircon_graphics render_server_bridge --locked
doc_type: milestone-detail
---

# M5 Hybrid GI GPU Completion Source

**Goal:** 把 `Hybrid GI` 从纯 CPU feedback-driven runtime host 推进到真实 GPU completion source：probe update completion 与 trace-region completion 都来自 renderer-local `wgpu` pass/readback，而不是只靠 `VisibilityHybridGiFeedback` 充当最终完成信号。

**Non-Goal:** 本轮不实现 radiance cache lighting resolve、screen-probe gather、surface cache/card atlas、hardware RT lighting 或任何最终 GI 着色输出。

## Delivered Slice

- `HybridGiRuntimeState` 新增两层能力：
  - `build_prepare_frame()`：把 resident probe cache、pending update、trace schedule、evictable probe 列表编成 renderer 可直接消费的 frame-local snapshot
  - `complete_gpu_updates(...)`：接受 GPU readback 的 completed probe ids 与 completed trace region ids，并在 runtime host 内推进 probe cache/residency/trace schedule
- `SceneRenderer` 新增 renderer-local `hybrid_gi/` 子模块：
  - resident probe cache 上传到 storage buffer
  - pending probe updates 上传到 storage buffer
  - scheduled trace region ids 上传到 storage buffer
  - compute pass 按 `probe_budget / tracing_budget / evictable_probe_count` 产出两类 completion readback：
    - `completed_probe_ids`
    - `completed_trace_region_ids`
- `WgpuRenderServer::submit_frame_extract(...)` 现在和 `Virtual Geometry` 一样，优先消费 renderer GPU readback，再回退到旧 CPU feedback path。

## Contract Shape

- `Hybrid GI` GPU path仍然保持 renderer-local，不向 `zircon_render_server` 泄漏 backend 私有结构。
- `HybridGiPrepareFrame` 只描述 renderer 当前需要的 host snapshot：
  - `resident_probes`
  - `pending_updates`
  - `scheduled_trace_region_ids`
  - `evictable_probe_ids`
- compute shader 只负责决定“本帧哪些 probe update / trace region 算完成”，不直接写 runtime host，也不负责最终 lighting resolve。

## Runtime Behavior

- GPU 完成的 probe update 仍受 runtime host budget 约束：
  - 如果 probe cache 已满，则只能驱逐 prepare frame 给出的 evictable probe
  - 没有可回收 slot 时，probe update 仍留在 pending queue
- GPU 完成的 trace region 列表会直接成为新的 runtime trace schedule，从而把“tracing completion source”从 visibility feedback 迁到 renderer readback。
- 这让后续 radiance-cache lighting resolve 可以直接消费一个已经存在的“真实 GPU completion source”边界，而不是重新拆 render-server façade。

## Validation Summary

- `hybrid_gi_gpu_completion_readback_reports_completed_probe_updates_and_traces`
  - 证明 GPU pass 会在 probe budget 与 tracing budget 下分别回传 completed probe 与 completed trace region
- `hybrid_gi_gpu_completion_readback_respects_probe_budget_without_evictable_slots`
  - 证明 probe cache 无 evictable slot 时 update completion 会为空，但 trace completion 仍可独立推进
- `hybrid_gi_runtime_state_builds_prepare_frame_with_resident_pending_and_trace_schedule`
  - 证明 runtime host 现在能稳定导出 renderer-local prepare snapshot
- `hybrid_gi_runtime_state_applies_gpu_completed_updates_and_trace_schedule`
  - 证明 runtime host 会正确消费 GPU probe completion 与 trace completion

## Remaining Route

- radiance cache lighting resolve
- screen probe / probe gather / temporal reuse
- RT hybrid lighting / BVH-AS coupling
- scene representation / surface cache / card atlas
- 与 history resolve、post process、Virtual Geometry scene representation 的联合路径
