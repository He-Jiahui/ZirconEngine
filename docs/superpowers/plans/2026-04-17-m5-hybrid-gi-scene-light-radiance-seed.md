---
related_code:
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/execute_runtime_prepare_passes.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/hybrid_gi_completion_params.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/scene_light_seed.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/execute_runtime_prepare_passes.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/hybrid_gi_completion_params.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/scene_light_seed.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
plan_sources:
  - user: 2026-04-17 continue the next M5 slice and keep going without waiting for confirmation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-gpu-completion-source.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-radiance-cache-lighting-resolve.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-normalized-multi-region-gather.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_changes_when_directional_light_color_changes --locked
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_changes_when_directional_light_intensity_changes --locked
  - cargo test -p zircon_graphics hybrid_gi_gpu --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Hybrid GI Scene-Light Radiance Seed

**Goal:** 把 `Hybrid GI` 的 GPU completion 再往 scene-driven 方向推进一层，让 traced radiance source 不再只由 probe/trace region 的量化空间关系决定，而是显式受当前帧 scene lighting 的颜色与强度驱动。

**Non-Goal:** 本轮仍然不实现完整 screen-probe gather、surface cache、RT hybrid lighting 或更高阶 Lumen-like scene representation。

## Delivered Slice

- `execute_runtime_prepare_passes.rs` 现在会把 `RenderFrameExtract.lighting.directional_lights` 一起传给 `HybridGiGpuResources::execute_prepare(...)`。
- `execute_prepare/scene_light_seed.rs` 新增 scene-light 聚合：
  - 对当前帧方向光做 `color * intensity` 累积
  - 归一化为 `rgb8` light seed
  - 若场景没有有效方向光，则回退到 neutral white `255/255/255`，保持旧 baseline 的无光测试与非 scene-driven 路径不退化
- `HybridGiCompletionParams` 新增 `scene_light_seed_rgb` uniform 字段，CPU 会在每帧 GPU completion 前写入。
- `HybridGiCompletionParams` 现在还会携带 `scene_light_strength_q`，用于把方向光总强度映射进 GPU completion。
- `update_completion.wgsl` 新增 `apply_scene_light_seed(...)`，把 trace region 原有 base RGB 按 scene-light seed 做通道调制，并再按 `scene_light_strength_q` 缩放整体 radiance 强度。
  - 这意味着 trace region 仍然提供空间分布和局部差异
  - 但最终 radiance hue 和 energy 已开始受真实 scene light tint / intensity 影响
- 新增红转绿测试 `hybrid_gi_gpu_completion_readback_changes_when_directional_light_color_changes`，证明同一 probe/trace 布局下，仅改变方向光颜色就会改变 GPU-produced `probe_irradiance_rgb`。
- 新增红转绿测试 `hybrid_gi_gpu_completion_readback_changes_when_directional_light_intensity_changes`，证明保持同色但提高方向光强度也会提升 GPU-produced `probe_irradiance_rgb` 的亮度。

## Why This Slice Exists

- 之前 `Hybrid GI` 已经有：
  - runtime host
  - GPU completion source
  - temporal radiance-cache update
  - trace-region-localized gather
  - normalized multi-region blend
- 但 traced radiance source 仍然是“几何关系 + region id 量化色”的 heuristic，尚未真正连接 scene lighting。
- 把方向光颜色和强度都接进 completion shader 后，当前链路已经从“只会跟 trace region 变”推进到“也会跟 scene light tint / intensity 变”，这是走向 scene-driven radiance cache 的第一段真实输入闭环。

## Validation Summary

- `hybrid_gi_gpu_completion_readback_changes_when_directional_light_color_changes`
  - 证明 warm / cool 方向光会把同一 pending probe 的 readback 分别推向更红 / 更蓝的结果
- `hybrid_gi_gpu_completion_readback_changes_when_directional_light_intensity_changes`
  - 证明同色但更强的方向光会让同一 pending probe 的 radiance-cache readback 更亮，而不是继续被 seed 归一化抹掉能量差异
- `hybrid_gi_gpu`
  - 证明新的 light tint + intensity seed 没有破坏 probe budget、history-sensitive resident update、no-trace preserve、localized gather 与 normalized multi-region blend
- `cargo test -p zircon_graphics --lib --locked`
- `./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics`

## Remaining Route

- `Hybrid GI`
  - 把 scene-driven 输入从“方向光 tint seed”继续推进到更完整的 radiance cache / screen-probe / scene representation
  - capability-gated RT hybrid lighting
- `Virtual Geometry`
  - GPU-generated indirect args
  - visibility-owned indirect compaction
  - deeper cluster raster / split-merge hierarchy
