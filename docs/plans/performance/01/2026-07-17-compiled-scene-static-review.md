---
title: Compiled scene static performance review
date: 2026-07-17
status: static-reviewed-code-fixed-dynamic-pending
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/irradiance_volume
plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
---

# Compiled scene 静态性能审查

`render_compiled_scene` 在每帧准备 irradiance volume 时，原先不论 volume 列表是否为空，都会先遍历所有 camera-layer mesh 并构造 sample-position `Vec`。Irradiance volumes 是可选高级功能；默认/MVP 无 volume 时这段 mesh 扫描和分配没有消费者。

已直接增加 lazy iterator guard：只有 extract 至少包含一个 irradiance volume 时才消费 mesh-position iterator。聚焦测试锁定两点：空 volume 列表不访问 iterator；存在 volume 时每个候选位置只收集一次。camera-layer filter、`select_irradiance_volume_for_view` 与 texture lookup 语义不变。

状态：`PERF-MVP-025` code fixed、Cargo/WGPU pending。Rustfmt 已通过；共享 CPU lane 被 Sound reservation 占用，所以不声明测试通过，`graphics` 目录仍留在 `pending.md`。

## Compiled pipeline 派生状态重复计算

同一帧在进入 compiled scene 前，`runtime_features_from_pipeline` 为每个 flag 分别线性扫描 `enabled_features`；screen-space reflection、HZB、exposure、volumetric history 又分别调用 `pipeline_writes_resource`，重复遍历 live pass/resource。进入 `render_compiled_scene` 后，`validate_compiled_pipeline` 再遍历所有 live pass，为每个 executor id clone `String`、构造 typed id 并查一次 registry，实际 execute 阶段还会再次查 executor。

这些数据只随 compiled pipeline 或 plugin executor registry generation 变化，不随普通 frame extract 改变。修复需要 Render01 冻结 compiled derived metadata 与 registry generation invalidation，不能简单删除 validation：热重载后缺失 executor 仍必须在提交前报告。已作为 `PERF-MVP-026` 移交 `docs/plans/zircon_runtime/render/01/failure-2026-07-17-compiled-pipeline-frame-derived-recomputation.md`。
