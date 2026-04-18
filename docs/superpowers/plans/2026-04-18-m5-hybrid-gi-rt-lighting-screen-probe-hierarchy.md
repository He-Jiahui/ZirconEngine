---
related_code:
  - zircon_scene/src/components.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_trace_region_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/trace_region_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/build.rs
- zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/frontier/refine_visible_probe_frontier.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
implementation_files:
  - zircon_scene/src/components.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_trace_region_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/trace_region_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/build.rs
- zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/frontier/refine_visible_probe_frontier.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
plan_sources:
  - user: 2026-04-17 continue M5
  - user: 2026-04-17 Hybrid GI still needs scene-driven radiance cache / probe gather / RT hybrid lighting
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-scene-driven-probe-gather.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-cache-entry-residency-cascade.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_completion_readback_uses_trace_region_rt_lighting_when_present
  - cargo test -p zircon_graphics --offline --locked visibility_context_keeps_hybrid_gi_parent_probe_visible_while_requesting_nonresident_children
  - cargo test -p zircon_graphics --offline --locked hybrid_gi
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI RT Lighting And Screen-Probe Hierarchy

**Goal:** 把 `Hybrid GI` 再往 Lumen-like 方向推进一层，在不打开硬件 RT/AS 依赖的前提下，让 trace region 能携带显式的 RT-lighting tint，同时让 probe visibility/request 走最小可用的 parent-child hierarchy。

**Non-Goal:** 本轮仍然不实现完整的 screen-probe relocation、surface cache、hardware RT gather 或真正的 Lumen-like scene representation。

## Delivered Slice

- `RenderHybridGiProbe` 现在显式携带 `parent_probe_id`，`build_hybrid_gi_plan(...)` 会先走 `refine_visible_probe_frontier(...)`：
  - resident parent 会在 children 非 resident 时继续留在 active frontier
  - parent/children 都 resident 时，会自然 refine 到 child probes
  - `probe_budget` 仍然只截 request 集，不会错误截断已激活的 hierarchy frontier
- `RenderHybridGiTraceRegion` 现在显式携带 `rt_lighting_rgb`，并一路进入：
  - `gpu_trace_region_input.rs`
  - `trace_region_inputs.rs`
  - `update_completion.wgsl`
- `update_completion.wgsl` 现在优先消费 `rt_lighting_rgb`，只有没有显式 RT-lighting tint 时才回退到旧的 region-color 启发式。

## Why This Slice Exists

- 之前的 `Hybrid GI` 已经有 scene-driven request、resident gather、temporal cache update 与 screen-space resolve，但 trace side 仍然主要依赖 procedural region color。
- 同时 probe active frontier 还缺少最基础的 hierarchy 结构，parent/child probe 之间没有真正的前沿切换语义。
- 如果这两层边界不补齐，后续继续往 screen-probe hierarchy / RT hybrid lighting 前进时，就还得回头重拆 scene contract 和 visibility planning。
- 本轮先把 `rt_lighting_rgb` 与 `parent_probe_id` 接进现有 GPU/visibility 主链，让更高阶路径可以在当前 contract 上继续向下长。

## Validation Summary

- `hybrid_gi_gpu_completion_readback_uses_trace_region_rt_lighting_when_present`
  - 证明 trace region 显式 RT-lighting tint 会真实改变 GPU radiance completion 输出
- `visibility_context_keeps_hybrid_gi_parent_probe_visible_while_requesting_nonresident_children`
  - 证明 resident parent probe 会在 child probes 尚未 resident 时继续承担 active frontier，同时 request 会继续指向 children
- `cargo test -p zircon_graphics --offline --locked hybrid_gi`
  - 证明 RT-lighting tint、screen-probe hierarchy、runtime host、resolve 与 cache residency 主链仍然兼容
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明 scene extract / GPU input / visibility planning contract 扩展没有留下编译缺口

## Remaining Route

- 把当前 parent-child frontier 继续推进到更完整的 screen-probe hierarchy，而不是只停在一层 parent-child refine
- 把 RT-lighting tint 继续推进到更真实的 scene-driven radiance source / hybrid lighting integration，而不是只停在 trace-region-local override
- 继续朝 screen-probe gather、surface-cache scene representation 与 capability-gated RT hybrid lighting 迈进
