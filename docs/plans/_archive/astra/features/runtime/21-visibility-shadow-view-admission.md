---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/213-runtime-visibility-gpu-scene-culling-batching-instancing-hzb-virtual-geometry-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/09b/2026-09-09-disabled-directional-shadow-view-admission.md
---

# Visibility shadow-view admission

The visibility producer and shadow planner now share the same directional
shadow admission predicate and first-owner policy. Disabled directional lights
do not create a shadow view or reserve an extra-view slot; enabled lights retain
the existing cascade projection and layer-independent caster behavior, while
additional enabled directionals do not create views that the current planner
cannot consume.

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Runtime09B/VIS213-G08 | disabled directional light 过滤、首个 planner owner 对齐、容量估算与构造层回归 | `implemented_pending_validation` | `optimization_batch_runtime213_disabled_directional_shadow_emits_no_views`、`optimization_batch_runtime213_shadow_capacity_filters_disabled_directional_lights`、`optimization_batch_runtime213_directional_view_owner_matches_shadow_planner`、`visibility_context_skips_disabled_directional_shadow_views`、`visibility_context_matches_single_directional_shadow_planner_owner` 已写入对应 owner；Rust 解析、静态契约与 scoped diff-check 通过。受管 Windows Cargo、shadow product capture 与 release p50/p95/p99 仍待异步验证，外部 `E:/Git/zr_vm` dirty，故本轮不封存。 |
