---
related_code:
  - zircon_graphics/src/types.rs
  - zircon_graphics/src/runtime/hybrid_gi.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/hybrid_gi_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/post_process_params.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/scene_post_process_resources.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
implementation_files:
  - zircon_graphics/src/types.rs
  - zircon_graphics/src/runtime/hybrid_gi.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/hybrid_gi_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/scene_post_process_resources.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
plan_sources:
  - user: 2026-04-17 Hybrid GI next step should enter radiance-cache lighting resolve
  - user: 2026-04-17 remaining work should connect real shader/resource/runtime data paths into the fixed pass and capability boundaries
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-gpu-completion-source.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - cargo test -p zircon_graphics hybrid_gi_resolve_adds_radiance_cache_indirect_light_when_feature_enabled --locked
  - cargo test -p zircon_graphics hybrid_gi_resolve_uses_prepare_probe_irradiance_colors --locked
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
  - `HybridGiRuntimeState::build_prepare_frame()` 现在会为 runtime-generated resident probe 填充稳定的默认 irradiance。
  - 这为后续接入真正 GPU-produced probe radiance 预留了明确的数据槽位，而不需要改 façade。
- `post_process.wgsl` 现在会消费 `HybridGiProbe` 的 `irradiance_and_weight`，把 resident probe 的 irradiance 与 trace schedule 组合成最终间接光贡献。
  - `GlobalIllumination` feature 开启且有 resident probe 时，最终帧会被真正 brighten。
  - 不同 probe 的 `irradiance_rgb` 会把输出拉向不同颜色通道，而不再只是固定冷色常量。

## Runtime And Shader Contract

- host/runtime 侧：
  - `submit_frame_extract(...)` 继续负责把 viewport runtime host 生成的 `HybridGiPrepareFrame` 挂到 `EditorOrRuntimeFrame`
  - `HybridGiPrepareProbe` 现在同时携带 `probe_id / slot / ray_budget / irradiance_rgb`
- renderer 侧：
  - post-process 资源层把 `resident_probes` 编成 `GpuHybridGiProbe`
  - 每个 probe 都会带 `slot_and_budget` 与 `irradiance_and_weight`
- shader 侧：
  - GI resolve 会遍历 resident probe buffer
  - 每个 probe 的 irradiance 先按 ray-budget 权重缩放，再叠加 trace-region boost
  - 最终贡献在 post-process composite 阶段写入 `final_color`

这条数据路径现在已经具备“runtime 数据改变 -> GPU buffer 改变 -> final frame 改变”的闭环，后续只需要把 placeholder irradiance 源替换成更真实的 radiance cache 输出。

## Validation Summary

- `hybrid_gi_resolve_adds_radiance_cache_indirect_light_when_feature_enabled`
  - 证明 resident probe 存在时，GI resolve 确实会让最终帧变亮
- `hybrid_gi_resolve_uses_prepare_probe_irradiance_colors`
  - 证明不同 `irradiance_rgb` 会把最终帧推向不同颜色通道，GI resolve 不再只靠常量色
- `hybrid_gi_runtime_state_builds_prepare_frame_with_resident_pending_and_trace_schedule`
  - 证明 runtime host 会稳定导出带 `irradiance_rgb` 的 prepare snapshot
- `cargo test -p zircon_graphics hybrid_gi --locked`
  - 证明 GPU completion、runtime host、visibility plan 与新的 resolve 数据槽位兼容
- `cargo test -p zircon_graphics --lib --locked`
  - 证明新增 GI resolve 数据路径没有破坏其他 renderer behavior layer
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_graphics`
  - 证明 `zircon_graphics` 包级构建与测试矩阵维持绿色

## Remaining Route

- 让 `irradiance_rgb` 从真实 GPU radiance-cache/update pass 产出，而不是当前 runtime 默认值或测试注入值
- scene representation / screen probe gather / temporal reuse
- RT hybrid lighting / BVH-AS coupling
- 与 `Virtual Geometry` scene representation / page residency 的联合路径
