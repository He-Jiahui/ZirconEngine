---
related_code:
  - zircon_graphics/src/backend/render_backend/read_buffer_u32s.rs
  - zircon_graphics/src/runtime/hybrid_gi/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/update_stats/mod.rs
  - zircon_scene/src/components.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_hybrid_gi.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/types/mod.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
implementation_files:
  - zircon_graphics/src/backend/render_backend/read_buffer_u32s.rs
  - zircon_graphics/src/runtime/hybrid_gi/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/update_stats/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_hybrid_gi.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render_with_pipeline/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/types/mod.rs
plan_sources:
  - user: 2026-04-17 Hybrid GI next step should enter tracing/update completion source before radiance-cache lighting resolve
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m5-hybrid-gi-feedback-streaming.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_reports_completed_probe_updates_and_traces --locked
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_changes_when_probe_or_trace_scene_data_changes --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_applies_gpu_completed_updates_and_trace_schedule --locked
  - cargo test -p zircon_graphics --offline render_server_bridge
  - cargo test -p zircon_graphics --offline hybrid_gi_runtime
  - cargo test -p zircon_graphics --offline hybrid_gi_gpu
  - cargo test -p zircon_graphics hybrid_gi --locked
  - cargo test -p zircon_graphics --lib --locked
  - cargo test -p zircon_graphics render_server_bridge --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Hybrid GI GPU Completion Source

**Goal:** 把 `Hybrid GI` 从纯 CPU feedback-driven runtime host 推进到真实 GPU completion source：probe update completion 与 trace-region completion 都来自 renderer-local `wgpu` pass/readback，而不是只靠 `VisibilityHybridGiFeedback` 充当最终完成信号。

**Non-Goal:** 本轮不实现 radiance cache lighting resolve、screen-probe gather、surface cache/card atlas、hardware RT lighting 或任何最终 GI 着色输出。

## Delivered Slice

- `HybridGiRuntimeState` 新增两层能力：
  - `build_prepare_frame()`：把 resident probe cache、pending update、trace schedule、evictable probe 列表编成 renderer 可直接消费的 frame-local snapshot
  - `complete_gpu_updates(...)`：接受 GPU readback 的 completed probe ids、completed trace region ids 和 `probe_irradiance_rgb`，并在 runtime host 内推进 probe cache/residency/trace schedule/irradiance cache
- `SceneRenderer` 新增 renderer-local `hybrid_gi/` 子模块：
  - resident probe cache 上传到 storage buffer
  - pending probe updates 上传到 storage buffer
  - scheduled trace region ids 上传到 storage buffer
  - 从 `RenderHybridGiExtract` 读取 probe `position/radius` 与 trace region `bounds_center/bounds_radius/screen_coverage`，编成 renderer-local scene seed
  - compute pass 按 `probe_budget / tracing_budget / evictable_probe_count` 产出三类 readback：
    - `completed_probe_ids`
    - `completed_trace_region_ids`
    - `probe_irradiance_rgb`
- 当前 `hybrid_gi/gpu_resources/execute_prepare/execute/` 已继续拆成 `collect_inputs / create_buffers / create_bind_group / dispatch / copy_readbacks / execute` 等 helper 子模块，根入口只保留结构 wiring。
- `WgpuRenderServer::submit_frame_extract(...)` 现在和 `Virtual Geometry` 一样，优先消费 renderer GPU readback，再回退到旧 CPU feedback path。

## Contract Shape

- `Hybrid GI` GPU path仍然保持 renderer-local，不向 `zircon_render_server` 泄漏 backend 私有结构。
- `HybridGiPrepareFrame` 只描述 renderer 当前需要的 host snapshot：
  - `resident_probes`
  - `pending_updates`
  - `scheduled_trace_region_ids`
  - `evictable_probe_ids`
- `HybridGiGpuReadback` 当前会把：
  - `cache_entries`
  - `completed_probe_ids`
  - `completed_trace_region_ids`
  - `probe_irradiance_rgb`
  一起回传给 runtime host
- compute shader 仍然只负责生成本帧 completion / irradiance 更新结果；它不直接写 runtime host，也不自己执行最终 lighting resolve。
- 当前 irradiance kernel 已经不再只依赖 `probe_id / slot / ray_budget / trace count` 这类占位量，而是显式上传 probe `position/radius` 与 trace region `center/radius/coverage`，按两者空间关系生成 trace-region-localized radiance contribution。

## Runtime Behavior

- GPU 完成的 probe update 仍受 runtime host budget 约束：
  - 如果 probe cache 已满，则只能驱逐 prepare frame 给出的 evictable probe
  - 没有可回收 slot 时，probe update 仍留在 pending queue
- GPU 回传的 `probe_irradiance_rgb` 会被 runtime host 缓存下来，并在下一次 `build_prepare_frame()` 时重新导出给 renderer。
- `submit_frame_extract/update_stats/` 现在也已经拆成 base/hybrid-gi/quality/update helper 子模块，让 façade stats 汇总继续留在 render-server runtime 边界里，但不再由一个聚合脚本承担所有分支逻辑。
- GPU 完成的 trace region 列表会直接成为新的 runtime trace schedule，从而把“tracing completion source”从 visibility feedback 迁到 renderer readback。
- 同一 probe 或 trace region 只要场景空间元数据变化，GPU 回传的 `probe_irradiance_rgb` 也会变化；这保证后续 resolve 消费的是 extract 驱动的结果，而不是纯 id/count placeholder。
- 这让后续 radiance-cache lighting resolve 可以直接消费一个已经存在的“真实 GPU completion source + irradiance update source”边界，而不是重新拆 render-server façade。

## Validation Summary

- `hybrid_gi_gpu_completion_readback_reports_completed_probe_updates_and_traces`
  - 证明 GPU pass 会在 probe budget 与 tracing budget 下分别回传 completed probe、completed trace region，以及精确的 `probe_irradiance_rgb`
- `hybrid_gi_gpu_completion_readback_respects_probe_budget_without_evictable_slots`
  - 证明 probe cache 无 evictable slot 时 update completion 会为空，但 trace completion 仍可独立推进
- `hybrid_gi_gpu_completion_readback_changes_when_probe_or_trace_scene_data_changes`
  - 证明 probe 位置/半径或 trace region 空间信息变化时，GPU completion/readback 的 `probe_irradiance_rgb` 会随之变化
- `hybrid_gi_runtime_state_builds_prepare_frame_with_resident_pending_and_trace_schedule`
  - 证明 runtime host 现在能稳定导出 renderer-local prepare snapshot
- `hybrid_gi_runtime_state_applies_gpu_completed_updates_and_trace_schedule`
  - 证明 runtime host 会正确消费 GPU probe completion、trace completion 与 irradiance 更新，并在下一帧 prepare snapshot 中复用这些结果

## Remaining Route

- 真实 traced radiance-cache update kernel，而不是当前基于 quantized probe/trace 空间关系的 heuristic contribution
- screen probe / probe gather / temporal reuse
- RT hybrid lighting / BVH-AS coupling
- scene representation / surface cache / card atlas
- 与 history resolve、post process、Virtual Geometry scene representation 的联合路径
