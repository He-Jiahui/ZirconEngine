---
related_code:
  - zircon_graphics/src/types/hybrid_gi_resolve_runtime.rs
  - zircon_graphics/src/types/editor_or_runtime_frame.rs
  - zircon_graphics/src/types/editor_or_runtime_frame_with_hybrid_gi_resolve_runtime.rs
  - zircon_graphics/src/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_graphics/src/runtime/hybrid_gi/declarations/hybrid_gi_runtime_state.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_irradiance/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/mod.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
implementation_files:
  - zircon_graphics/src/types/hybrid_gi_resolve_runtime.rs
  - zircon_graphics/src/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_irradiance/mod.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting/mod.rs
plan_sources:
  - user: 2026-04-18 Hybrid GI 把 hierarchy-aware resolve / probe gather / RT hybrid lighting 从 encode-side 向 runtime/GPU source 收拢
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - cargo test -p zircon_graphics --offline hybrid_gi_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline hybrid_gi_resolve_render -- --nocapture
  - cargo test -p zircon_graphics --offline hybrid_gi_gpu_hierarchy -- --nocapture
  - cargo test -p zircon_graphics --offline render_framework_bridge -- --nocapture
doc_type: milestone-detail
---

# M5 Hybrid GI Runtime Resolve Source Closure

## Goal

把 `Hybrid GI` 的 hierarchy-aware resolve 真值继续从 post-process encode-side helper 收进 runtime/GPU host source，让 `EditorOrRuntimeFrame.hybrid_gi_resolve_runtime` 不再只承载直接 `probe_rt_lighting_rgb`，而是开始稳定携带：

- hierarchy resolve weight
- farther-resident-ancestor irradiance continuation
- ancestor-derived RT-lighting continuation

这样 `runtime/render_framework/submit_frame_extract/*` 在把 runtime host state 组装进 renderer frame 时，就不再只把 hierarchy continuity 留给单帧 encode 扫描。

## Delivered Slice

- `HybridGiResolveRuntime` 新增三类 runtime-carried inputs：
  - `probe_hierarchy_resolve_weight_q8`
  - `probe_hierarchy_irradiance_rgb_and_weight`
  - `probe_hierarchy_rt_lighting_rgb_and_weight`
- `HybridGiRuntimeState::build_resolve_runtime()` 现在会直接从 runtime host 的 resident probe lineage、ray budget、GPU-produced `probe_irradiance_rgb`、GPU-produced `probe_rt_lighting_rgb` 生成这些 resolve inputs。
- post-process probe encode 现在按以下顺序取值：
  1. 优先消费 `frame.hybrid_gi_resolve_runtime`
  2. 只有 runtime source 缺失时，才回退到旧的 encode-side hierarchy scan
- `hybrid_gi_hierarchy_rt_lighting(...)` 现在还会把 runtime direct RT history 与 runtime hierarchy RT continuation 合并，而不是要求当前帧必须重新带上 trace schedule 才能得到 ancestor tint。

## Why This Matters

在这次切片之前，M5 Hybrid GI 已经具备了更深 ancestor continuation，但真正的 resolve 入口仍有一个明显断层：

- runtime host / GPU readback 已经知道更多 hierarchy truth
- post-process encode 仍然每帧重新扫描 ancestor chain
- hierarchy resolve weight / farther-ancestor irradiance / ancestor-derived RT tint 主要还是 encode-time 推导，而不是 runtime-carried truth

这会让以下三件事继续耦合在单帧 encode 侧：

- hierarchy continuity 是否存在
- hierarchy continuity 有多强
- hierarchy continuity 是直接 probe history 还是 ancestor lineage continuation

本切片把这三项继续收回 runtime/GPU host source 后，下一轮更完整的 scene-driven screen-probe hierarchy / RT hybrid lighting continuation 就可以继续围绕 runtime frame contract 演进，而不是反复加厚 encode helper。

## Validation

- `hybrid_gi_runtime_state_builds_hierarchy_resolve_runtime_from_resident_lineage_history`
  - 证明 runtime host 已经能从 resident lineage 直接构建 hierarchy resolve weight / irradiance continuation / RT continuation。
- `hybrid_gi_resolve_uses_runtime_hierarchy_irradiance_and_weight_without_current_ancestor_prepare`
  - 证明当前帧即使不再携带 ancestor prepare data，resolve 仍会吃到 runtime hierarchy irradiance + weight。
- `hybrid_gi_resolve_uses_runtime_hierarchy_rt_lighting_without_current_trace_schedule`
  - 证明当前帧没有 scheduled trace work 时，resolve 仍会消费 runtime-carried hierarchy RT-lighting continuation。
- `render_framework_hybrid_gi_second_frame_resolve_reuses_gpu_completed_hierarchy_history`
  - 证明 `runtime/render_framework/*` 的第二帧桥接路径仍然能复用前一帧 GPU-completed hierarchy history。

## Remaining Route

- 继续把更完整的 screen-probe hierarchy gather / request scoring 也向 runtime/GPU host source 收拢，减少 encode-side 对 extract ancestry 的即时扫描。
- 继续把更复杂 trace-region 组合下的 RT hybrid lighting continuation 做成 scene-driven runtime truth，而不只是 current-frame encode fallback。
- 如果后续引入 screen-probe hierarchy cache / probe gather state，本次 `HybridGiResolveRuntime` 扩展就是它的直接承载边界。
