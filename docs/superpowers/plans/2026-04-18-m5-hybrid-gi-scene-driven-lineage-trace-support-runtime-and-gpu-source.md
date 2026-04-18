---
related_code:
  - zircon_graphics/src/runtime/hybrid_gi/declarations/hybrid_gi_runtime_state.rs
  - zircon_graphics/src/runtime/hybrid_gi/declarations/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/extract_registration.rs
  - zircon_graphics/src/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_graphics/src/runtime/hybrid_gi/prepare_frame/collect_pending_updates.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_pending_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_resident_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/probe_quantization.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/resident_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu_hierarchy.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
implementation_files:
  - zircon_graphics/src/runtime/hybrid_gi/declarations/hybrid_gi_runtime_state.rs
  - zircon_graphics/src/runtime/hybrid_gi/extract_registration.rs
  - zircon_graphics/src/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_graphics/src/runtime/hybrid_gi/prepare_frame/collect_pending_updates.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_pending_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_resident_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/probe_quantization.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/resident_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
plan_sources:
  - user: 2026-04-18 Hybrid GI scene-driven screen-probe hierarchy gather / request / RT hybrid lighting continuation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu_hierarchy.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - cargo test -p zircon_graphics --offline hybrid_gi_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline hybrid_gi_gpu_hierarchy -- --nocapture
  - cargo test -p zircon_graphics --offline hybrid_gi_resolve_render -- --nocapture
  - cargo check -p zircon_graphics --lib --offline
doc_type: milestone-detail
---

# M5 Hybrid GI Scene-Driven Lineage Trace Support Runtime And GPU Source

## Goal

把 `Hybrid GI` 当前仍然分散在 visibility request 排序、GPU completion 和 runtime resolve 之间的 lineage trace support 收成一条连续数据路径，让更完整的 scene-driven screen-probe hierarchy 不再只靠单帧 helper 临时推导。

这次切片固定收拢三件事：

- runtime host 持有 probe / trace region 的 scene-driven quantized truth
- pending probe update 排序按 lineage trace support 而不只按 descendant 形状
- GPU completion 与 runtime resolve 都开始消费同一类 lineage trace support / lineage RT-lighting continuation

## Delivered Slice

- `HybridGiRuntimeState` 现在额外持有 quantized probe scene data 与 trace region scene data，并在 `register_extract(...)` 时随 scene 收缩一起裁掉 stale entries。
- `collect_pending_updates(...)` 新增 `lineage_trace_support_sort_key(...)`，会沿 probe -> ancestor chain 聚合当前 scheduled trace regions 的 scene-driven support，再决定 pending update 排序。
- `GpuResidentProbeInput` 与 `GpuPendingProbeInput` 现在新增：
  - `lineage_trace_support_q`
  - `lineage_trace_lighting_rgb`
- `probe_quantization.rs` 现在会沿完整 probe parent chain 聚合 scheduled trace-region support，并把 nonresident lineage 命中的 `rt_lighting_rgb` 编成 GPU-source continuation。
- `update_completion.wgsl` 现在会在 resident / pending 两条路径上：
  - 先把 lineage RT-lighting continuation 混进 traced contribution
  - 再按 lineage trace support 提高 traced + gathered 的合成权重
- `build_resolve_runtime()` 现在也会让 hierarchy resolve weight 与 hierarchy RT-lighting continuation 吃到当前 scheduled trace work 的 lineage support，而不是只认 resident lineage budget / cached RT tint。

## Why This Matters

在这次切片之前，M5 Hybrid GI 已经完成了多级 resident ancestor continuation，但还剩两个明显断层：

- runtime host 的 pending queue 仍主要按层级形状排序，没有稳定消费当前 scheduled trace work 的 lineage support
- GPU completion 虽然已经理解 resident ancestor continuation，但对 nonresident lineage 的 scene-driven trace tint 仍然缺少独立输入

这会导致 request、update、resolve 三段虽然都在讲 hierarchy continuity，却还没有共享一套真正的 scene-driven trace support 真值。

本切片之后：

- request 层会优先推进仍被当前 scheduled trace work 支持的 lineage
- GPU completion 会把 nonresident hierarchy 命中的 RT tint 真正做成 input，而不是要求必须等 resident ancestor 先落地
- runtime resolve weight 也开始直接反映当前 trace schedule 对 probe lineage 的支持强度

## Validation

- `hybrid_gi_runtime_state_prioritizes_pending_probe_with_stronger_lineage_trace_support`
  - 证明 runtime pending queue 会优先推进被 scheduled trace hierarchy 支持的 lineage。
- `hybrid_gi_runtime_state_strengthens_resolve_weight_when_trace_schedule_supports_lineage`
  - 证明 runtime resolve weight 现在开始直接消费当前 trace schedule 的 lineage support。
- `hybrid_gi_gpu_completion_readback_inherits_nonresident_lineage_trace_lighting_without_resident_ancestors`
  - 证明即使 resident ancestor 还没落地，nonresident hierarchy 也会把 scene-driven RT tint 真实带进 GPU trace-lighting readback。
- `hybrid_gi_gpu_hierarchy`
  - 证明现有多级 resident ancestor radiance / RT continuation 路径没有被这次扩展打穿。
- `hybrid_gi_resolve_render`
  - 证明 resolve 侧原有 hierarchy continuity 与 runtime resolve source closure 仍然保持稳定。

## Remaining Route

- 继续把更完整的 scene-driven screen-probe hierarchy gather / resolve host 闭环推进到 radiance-cache source，而不只体现在 trace-support weighting。
- 继续把 Hybrid GI 的 screen-probe hierarchy / RT hybrid lighting continuation 往更完整的 scene representation、probe gather hierarchy 与 hybrid lighting source 上推进。
- 等 Hybrid GI 这条主链再收一到两刀后，再切回 `Virtual Geometry` 的 unified indirect / residency-manager cascade / split-merge frontier policy。
