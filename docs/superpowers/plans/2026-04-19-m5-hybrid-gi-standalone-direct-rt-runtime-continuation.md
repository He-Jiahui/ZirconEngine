---
related_code:
  - zircon_graphics/src/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/runtime_trace_source.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
  - docs/assets-and-rendering/render-framework-architecture.md
implementation_files:
  - zircon_graphics/src/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/runtime_trace_source.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
plan_sources:
  - user: 2026-04-19 继续完成全部的虚拟几何体任务，不要中途确认，然后完善 Hybrid GI
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-requested-lineage-rt-runtime-source.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-runtime-resolve-gpu-prepare-rt-lighting-continuation.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_pending_probe_gpu_trace_lighting_uses_runtime_direct_rt_history_when_hierarchy_weight_is_flat -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_runtime_source -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Standalone Direct RT Runtime Continuation

## Goal

补上 `Hybrid GI` runtime host 在 “probe 没有 hierarchy parent，但自己已经持有 GPU-produced RT lighting history” 这条路径上的 continuation 闭环，避免 `build_resolve_runtime() -> runtime_trace_source() -> GPU prepare` 在 hierarchy resolve weight 仍为 flat baseline 时直接掉回黑值。

## Delivered Slice

`build_resolve_runtime()` 现在对 standalone nonresident probe 新增了一条更窄的 direct-RT fallback：

- 只在 probe 当前不在 `resident_slots` 且没有 `parent_probe_id` 时启用
- 直接消费 probe 自己已经缓存的 `probe_rt_lighting_rgb`
- 用 probe 自身 `ray_budget` 与 RT intensity 量化成 lightweight hierarchy continuation
- 仍然沿现有 `hierarchy_rt_lighting(probe_id) -> runtime_trace_source(...) -> GPU prepare` 主链进入 renderer

这样 runtime/GPU source 的 direct RT continuation 被补齐，但不会把已有 lineage 的 requested-vs-flat hierarchy 边界一并抹掉。

## Why This Slice Matters

此前 runtime-side RT continuation 主要围绕 requested-lineage / ancestor lineage 建模。

这会留下一个空洞：

- standalone pending probe 明明已经有 probe-local RT history
- 但因为没有 lineage support，`direct_lineage_rt_lighting_fallback(...)` 会直接返回 `None`
- `runtime_trace_source(...)` 因而拿不到任何 hierarchy RT payload，GPU prepare 只能回退黑值

这条 slice 让 “scene-driven lineage” 和 “standalone direct runtime history” 两类 continuation 都能复用同一条 runtime-to-GPU contract，而不需要再为无 parent probe 单独开 encode-side 旁路。

## Validation Summary

- 红:
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_pending_probe_gpu_trace_lighting_uses_runtime_direct_rt_history_when_hierarchy_weight_is_flat -- --nocapture`
- 绿:
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_pending_probe_gpu_trace_lighting_uses_runtime_direct_rt_history_when_hierarchy_weight_is_flat -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_runtime_source -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining M5 Todo After This Slice

- `Hybrid GI`: 继续把 scene-driven screen-probe hierarchy / gather / RT hybrid-lighting continuation 从 runtime-host 推进到更深的 GPU/runtime source 闭环。
- `Virtual Geometry`: 继续推进 deeper unified indirect / cluster-raster submission ownership。
- `Virtual Geometry`: 继续收口更深的 residency-manager cascade / split-merge frontier policy。
