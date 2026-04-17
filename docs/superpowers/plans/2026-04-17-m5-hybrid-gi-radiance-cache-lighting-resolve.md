---
related_code:
  - zircon_graphics/src/types/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/scene/scene_renderer/post_process/gpu_data/hybrid_gi_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/gpu_data/hybrid_gi_trace_region_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/params/post_process_params.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/scene_post_process_resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
implementation_files:
  - zircon_graphics/src/types/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/scene/scene_renderer/post_process/gpu_data/hybrid_gi_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/gpu_data/hybrid_gi_trace_region_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/scene_post_process_resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl
plan_sources:
  - user: 2026-04-17 Hybrid GI next step should enter radiance-cache lighting resolve
  - user: 2026-04-17 remaining work should connect real shader/resource/runtime data paths into the fixed pass and capability boundaries
  - user: 2026-04-17 Hybrid GI should replace runtime default/test injected irradiance_rgb with real GPU radiance-cache update output
  - user: 2026-04-17 Hybrid GI next slice should add trace-region-localized resolve instead of only a scheduled-count multiplier
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-gpu-completion-source.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_reports_completed_probe_updates_and_traces --locked
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_changes_when_probe_or_trace_scene_data_changes --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_applies_gpu_completed_updates_and_trace_schedule --locked
  - cargo test -p zircon_graphics hybrid_gi_resolve_adds_radiance_cache_indirect_light_when_feature_enabled --locked
  - cargo test -p zircon_graphics hybrid_gi_resolve_localizes_indirect_light_by_probe_screen_position --locked
  - cargo test -p zircon_graphics hybrid_gi_resolve_localizes_trace_region_boost_by_screen_position --locked
  - cargo test -p zircon_graphics hybrid_gi_resolve_uses_prepare_probe_irradiance_colors --locked
  - cargo test -p zircon_graphics hybrid_gi_resolve_render --locked
  - cargo test -p zircon_graphics hybrid_gi --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Hybrid GI Radiance Cache Lighting Resolve

**Goal:** 把 `Hybrid GI` 从“只有 runtime host + GPU completion source”的状态推进到真正会影响最终成像的 `radiance-cache lighting resolve` 基线，并保证这条路径遵守既有的 `RenderFeature -> runtime prepare -> renderer resource -> post-process shader` 边界。

**Non-Goal:** 本轮不实现 Lumen 风格的 scene representation、surface cache/card atlas、screen probe gather、hardware RT lighting 或 probe temporal reuse。

## Delivered Slice

- `SceneRendererCore::render_compiled_scene(...)` 现在把完整 `EditorOrRuntimeFrame` 交给 post-process，而不是只传 `RenderFrameExtract`。
  - 这让 post-process 能直接访问 `hybrid_gi_prepare`，不需要绕回 renderer 外部状态。
- `ScenePostProcessResources` 新增 `hybrid_gi_probe_buffer` 绑定，并在 `execute_post_process(...)` 里把 resident probe prepare snapshot 编码成 GPU storage buffer。
- `HybridGiPrepareProbe` 新增 `irradiance_rgb`。
  - `hybrid_gi/update_completion.wgsl` 与 `HybridGiGpuResources` 现在会为 resident probe 和本帧完成的 pending probe update 产出真实 GPU `probe_irradiance_rgb` readback。
  - 当前 GPU update 不再只看 id/count/budget 占位量，而是显式折叠 `RenderHybridGiExtract` 的 probe 空间信息与 trace region 场景信息，因此 resolve 已经开始消费 scene-driven GPU readback。
  - `submit_frame_extract(...)` 会把这些 GPU irradiance 更新回写到 `HybridGiRuntimeState::complete_gpu_updates(...)`，下一次 `build_prepare_frame()` 导出的就不再只是 runtime 默认值或测试注入值。
- `post_process.wgsl` 现在会消费 `HybridGiProbe` 的 `irradiance_and_weight`，把 resident probe 的 irradiance 与 trace schedule 组合成最终间接光贡献。
  - `execute_post_process(...)` 现在还会把 `RenderHybridGiExtract.probes` 的 `position/radius` 投影成 `screen_uv_and_radius`，并和 prepare snapshot 里的 `irradiance_rgb` 一起编码进 `GpuHybridGiProbe`。
  - `execute_post_process(...)` 现在还会把 `RenderHybridGiExtract.trace_regions` 里被 `scheduled_trace_region_ids` 命中的 region 投影成 `GpuHybridGiTraceRegion` buffer，并按 `bounds_center / bounds_radius / screen_coverage / tracing_budget` 编码局部 boost。
  - `GlobalIllumination` feature 开启且有 resident probe 时，最终帧会在 probe 覆盖区域被真正 brighten，而不是再走全屏平均提亮；trace region 也不再只是统一倍率，而会把 boost 压到命中的屏幕局部区域。
  - 不同 probe 的 `irradiance_rgb` 会把各自影响区域拉向不同颜色通道，而不再只是固定冷色常量。

## Runtime And Shader Contract

- host/runtime 侧：
  - `submit_frame_extract(...)` 负责把 viewport runtime host 生成的 `HybridGiPrepareFrame` 挂到 `EditorOrRuntimeFrame`
  - render 完成后，它还会把 GPU readback 的 `probe_irradiance_rgb` 合并回 runtime host，因此下一帧 `HybridGiPrepareProbe` 会稳定携带 `probe_id / slot / ray_budget / irradiance_rgb`
- renderer 侧：
  - completion compute pass 会上传 resident probe、pending probe update 与 scheduled trace region，并结合 extract 提供的 probe/trace 场景元数据为 resident / completed probe 生成 packed RGB irradiance
  - post-process 资源层把 `resident_probes` 编成 `GpuHybridGiProbe`
  - 每个 probe 现在都会带 `screen_uv_and_radius` 与 `irradiance_and_intensity`
- shader 侧：
  - GI resolve 会遍历 resident probe buffer
  - GI resolve 现在还会遍历 `GpuHybridGiTraceRegion` buffer，把 scheduled trace region 的屏幕空间半径、coverage 与 tracing-budget 强度转成局部 `trace_boost`
  - 每个 probe 的 irradiance 会先按 ray-budget 权重和屏幕空间半径衰减，再叠加 trace-region-localized boost
  - 最终贡献在 post-process composite 阶段写入 `final_color`

这条数据路径现在已经具备“GPU update pass 输出 irradiance -> runtime host 缓存 -> 下一帧 prepare -> GPU probe/trace-region screen projection + resource encode -> shader resolve -> final frame 局部改变”的闭环。

## Validation Summary

- `hybrid_gi_gpu_completion_readback_reports_completed_probe_updates_and_traces`
  - 证明 GPU update pass 会回传确定性的 `probe_irradiance_rgb`，而不是只回传 completed probe/trace id
- `hybrid_gi_runtime_state_applies_gpu_completed_updates_and_trace_schedule`
  - 证明 runtime host 会把 GPU-produced irradiance 写回 resident probe，并在下一帧 prepare snapshot 中继续导出
- `hybrid_gi_gpu_completion_readback_changes_when_probe_or_trace_scene_data_changes`
  - 证明 resolve 所消费的 `probe_irradiance_rgb` 已经会随 probe/trace 场景元数据变化，而不是只依赖测试注入色或固定 kernel
- `hybrid_gi_resolve_adds_radiance_cache_indirect_light_when_feature_enabled`
  - 证明 resident probe 存在时，GI resolve 确实会让 probe 影响区域变亮
- `hybrid_gi_resolve_localizes_indirect_light_by_probe_screen_position`
  - 证明 probe 的屏幕位置会改变哪一侧区域获得更多间接光，GI resolve 已经不再是全屏平均贡献
- `hybrid_gi_resolve_localizes_trace_region_boost_by_screen_position`
  - 证明 scheduled trace region 的屏幕位置会改变哪一侧区域获得更强的 GI boost，resolve 已经不再把 trace schedule 当成统一全屏倍率
- `hybrid_gi_resolve_uses_prepare_probe_irradiance_colors`
  - 证明不同 `irradiance_rgb` 会把 probe 影响区域推向不同颜色通道，GI resolve 不再只靠常量色
- `hybrid_gi_runtime_state_builds_prepare_frame_with_resident_pending_and_trace_schedule`
  - 证明 runtime host 会稳定导出带 `irradiance_rgb` 的 prepare snapshot
- `cargo test -p zircon_graphics hybrid_gi --locked`
  - 证明 GPU completion、runtime host、visibility plan 与新的 resolve 数据槽位兼容
- `cargo test -p zircon_graphics --lib --locked`
  - 证明新增 GI resolve 数据路径没有破坏其他 renderer behavior layer
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_graphics`
  - 证明 `zircon_graphics` 包级构建与测试矩阵维持绿色

## Remaining Route

- 用真实 traced radiance-cache update / tracing 结果替换当前基于场景 metadata 的 deterministic GPU seeded irradiance kernel
- scene representation / screen probe gather / temporal reuse
- RT hybrid lighting / BVH-AS coupling
- 与 `Virtual Geometry` scene representation / page residency 的联合路径
