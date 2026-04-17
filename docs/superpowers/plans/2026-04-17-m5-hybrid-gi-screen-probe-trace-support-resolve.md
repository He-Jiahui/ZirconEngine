---
related_code:
  - zircon_graphics/src/scene/scene_renderer/post_process/gpu_data/hybrid_gi_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/gpu_data/hybrid_gi_trace_region_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
plan_sources:
  - user: 2026-04-17 continue the remaining M5 route without waiting for confirmation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-radiance-cache-lighting-resolve.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-scene-light-radiance-seed.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - cargo test -p zircon_graphics hybrid_gi_resolve_prefers_screen_probe_irradiance_supported_by_scheduled_trace_regions --locked
  - cargo test -p zircon_graphics hybrid_gi_resolve_render --locked
  - cargo test -p zircon_graphics hybrid_gi --locked
  - cargo check -p zircon_graphics --lib --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Screen-Probe Trace Support Resolve

**Goal:** 把 `Hybrid GI` 的 post-process resolve 从“probe irradiance 求和后再乘一个 trace-region 全局 boost”推进到真正的 screen-probe trace support 路径，让 scheduled trace region 能在 resolve 阶段偏向附近 probe，而不是只整体放大所有 probe 的总和。

**Non-Goal:** 本轮仍然不实现真正的 screen-probe hierarchy、surface cache、hardware RT gather 或 Lumen-like scene representation。

## Delivered Slice

- `post_process.wgsl` 里的 Hybrid GI resolve 不再先把所有 resident probe 统一累加、再单独计算一份 `trace_boost`。
- shader 现在会在每条 probe contribution 内部计算 `trace_support`：
  - 像素到 trace region 的距离决定当前屏幕位置是否真的处在 active trace work 影响带里
  - probe center 到 trace region center 的距离决定这条 probe 是否被当前 trace region 支持
  - 两部分支持度再乘上 `boost` 与 `coverage`，直接调节该 probe 自己的 resolve weight
- 这意味着 scheduled trace region 现在不只是“让当前像素整体更亮”，而是会在多个 probe 同时覆盖同一区域时，优先提升与 active trace region 更匹配的 probe irradiance。
- `GpuHybridGiProbe` 和 `GpuHybridGiTraceRegion` 原有的 screen-space contract 没有变化：
  - probe 继续提供 `screen_uv_and_radius + irradiance_and_intensity`
  - trace region 继续提供 `screen_uv_and_radius + boost_and_coverage`
  - 新行为完全落在 post-process shader 内部，没有把 backend 私有细节泄漏到 `RenderServer` 或 runtime host

## Shader Contract

- 没有 scheduled trace region 时，probe resolve 仍然只受 probe 自己的屏幕投影、半径、budget 和 irradiance 驱动。
- 有 scheduled trace region 时，每条 probe 的 resolve 权重会被一个 trace-support 因子调节；这个因子只会增强当前 probe，不会直接修改其他 probe 的 irradiance cache 内容。
- 因为 trace support 已经内嵌进 probe 权重，原来那条单独的全局 `localized_trace_boost` 不再需要继续存在；这样 trace region 的影响从“全局乘法”收紧成“probe selection”。

## Why This Slice Exists

- 之前的 radiance-cache lighting resolve 已经把：
  - resident probe irradiance
  - scheduled trace region screen projection
  - localized GI brightening
  接进了同一个 post-process pass。
- 但 trace region 只作为全局 boost 存在时，多个 probe 同时覆盖同一像素时，active trace work 仍然无法决定“哪条 probe 更可信”。
- 这让 resolve 更像“trace region 亮度放大器”，而不是更接近 scene-driven screen-probe support 的行为。
- 把 trace support 直接并入 per-probe resolve weight 后，当前架构终于具备了最小可用的 “active trace work biases probe choice” 行为，为后续真正的 screen-probe gather / radiance cache scene representation 预埋了更可信的消费边界。

## Validation Summary

- `hybrid_gi_resolve_prefers_screen_probe_irradiance_supported_by_scheduled_trace_regions`
  - 证明同一中心区域同时被 warm/cool 两条 probe 覆盖时，左侧 trace region 会把结果往左侧 warm probe 拉，右侧 trace region 会把结果往右侧 cool probe 拉。
- `hybrid_gi_resolve_render --locked`
  - 证明新的 per-probe trace support 没有破坏既有的 probe-color、probe-position、trace-region-position 与 GI brighten 回归。
- `hybrid_gi --locked`
  - 证明 resolve 路径的新 probe weighting 与 runtime host、GPU completion、radiance-cache history、scene-light-seeded update 仍然一致。
- `cargo check -p zircon_graphics --lib --locked`
  - 证明 post-process shader 改动没有留下编译缺口。

## Remaining Route

- 把当前 screen-space trace support 继续推进到更完整的 screen-probe hierarchy / radiance cache gather，而不是继续只依赖 projected probe + trace region 启发式
- 把 scheduled trace work 和更高阶 scene representation、surface cache、RT hybrid lighting 连接起来
- 让 probe support 不只影响 resolve 权重，也能继续反馈回更深的 probe scheduling / update priority
