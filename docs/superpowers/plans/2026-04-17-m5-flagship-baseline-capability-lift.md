---
related_code:
  - zircon_graphics/src/runtime/server/capability_summary/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/update_stats/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/mod.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_render_server/src/types.rs
  - zircon_render_server/src/tests.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
  - docs/assets-and-rendering/index.md
  - docs/superpowers/plans/2026-04-16-m5-flagship-capability-slots.md
implementation_files:
  - zircon_graphics/src/runtime/server/capability_summary/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/update_stats/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/mod.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_render_server/src/types.rs
  - zircon_render_server/src/tests.rs
plan_sources:
  - user: 2026-04-17 continue next task after M5 Virtual Geometry indirect raster and Hybrid GI resolve baselines
  - user: 2026-04-17 continue missing capability/runtime boundary work without waiting for confirmation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m5-flagship-capability-slots.md
tests:
  - zircon_render_server/src/tests.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - cargo test -p zircon_render_server --lib --locked
  - cargo test -p zircon_graphics render_server_bridge --locked
  - cargo test -p zircon_graphics --lib --locked
  - cargo test -p zircon_entry runtime_sources_route_preview_through_render_server_without_wgpu_surface_bindings --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Flagship Baseline Capability Lift

**Goal:** 在 `Virtual Geometry` 与当前 `Hybrid GI` 已经具备真实 runtime/resource/shader 数据路径之后，把 `RenderServer` façade 的 capability 结果和真实可运行基线重新对齐，不再让 pure `wgpu` headless baseline 继续把这两条已落地功能误报为不可用。

**Non-Goal:** 本轮不打开任何 RT/AS 路径，不把 `hybrid_global_illumination_supported` 解释成 hardware RT GI 可用，也不把 `virtual_geometry_supported` 解释成 Nanite-like cluster raster/full streaming manager 已经完成。

## Delivered Slice

- `capability_summary(...)` 现在把当前 M5 baseline 的支持条件收敛为：
  - backend 具备 `offscreen` 渲染能力
  - backend 具备 `Graphics` queue
- 在当前 `wgpu` RHI 基线下，这意味着：
  - `virtual_geometry_supported = true`
  - `hybrid_global_illumination_supported = true`
  - `acceleration_structures_supported = false`
  - `inline_ray_query = false`
  - `ray_tracing_pipeline = false`
- `compile_options_for_profile(...)` 不需要额外改接口；它继续只在 profile 显式 opt-in 且 capability 为真时打开 `VirtualGeometry` / `GlobalIllumination`。
- `RenderStats` 新增 `last_virtual_geometry_indirect_draw_count`，并由 `WgpuRenderServer` 在 submit 后回填 renderer 实际提交的 VG indirect raster 数量。

## Why This Lift Was Needed

- 更早的 capability-slot 切片把两条旗舰 feature 保守地绑在：
  - `supports_async_compute && supports_pipeline_cache`
  - `acceleration_structures_supported && (inline_ray_query || ray_tracing_pipeline)`
- 但后续 M5 实现已经证明当前 baseline 并不依赖这些条件：
  - `Virtual Geometry` 的 uploader/readback 与 indirect raster 都能在 graphics queue 上工作
  - `Hybrid GI` 的 GPU completion 与 radiance-cache lighting resolve 也是 graphics/offscreen baseline，可在无 RT/AS 条件下运行
- 如果继续维持旧 gate，`RenderServer` façade 就会和 renderer 真实现状发生漂移：上层看到“不支持”，而底层同一 backend 实际已经能运行并通过回归。

## Runtime Contract

- capability flag 现在描述的是“当前实现的 baseline 是否可运行”，不是“最终旗舰形态是否完整”。
- RT/AS 仍然通过独立 capability 字段表达，后续 Lumen-like RT hybrid lighting、hardware RT、Nanite-like cluster raster 仍必须继续走更高 capability tier。
- `last_virtual_geometry_indirect_draw_count` 仍然只暴露 façade 级计数，不把 renderer-local indirect args buffer 或 `wgpu` 原生对象泄漏到 `zircon_render_server` 外部。

## Validation Summary

- `cargo test -p zircon_render_server --lib --locked`
  - 证明 `RenderStats` 公共类型扩展保持默认值兼容。
- `cargo test -p zircon_graphics render_server_bridge --locked`
  - 证明 headless `wgpu` 在显式旗舰 profile 和带 payload 的 extract 下，会把 `virtual_geometry` 与 `global_illumination` 写入 `last_effective_features`，并产出非零 VG/GI runtime 统计。
- `cargo test -p zircon_graphics --lib --locked`
  - 证明 capability lift 与新增 façade stats 没有破坏现有 M4/M5 renderer 回归。
- `cargo test -p zircon_entry runtime_sources_route_preview_through_render_server_without_wgpu_surface_bindings --locked`
  - 证明 entry/runtime 侧继续只通过 `RenderServer` 消费渲染输出。
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_graphics`
  - 证明 `zircon_graphics` 包级 build/test 维持绿色。

## Remaining Route

- `Virtual Geometry`
  - unified indirect args ownership / GPU-driven indirect compaction
  - deeper cluster streaming / page residency manager
  - Nanite-like cluster raster
- `Hybrid GI`
  - traced radiance-cache update kernel instead of current deterministic scene-seeded irradiance source
  - screen probe gather / temporal reuse
  - RT hybrid lighting tier gated by AS/RT capability
