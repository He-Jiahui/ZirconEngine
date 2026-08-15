---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-14
summary_slug: rhi-wgpu-ui-surface-present-stats-non-exhaustive-construction
origin_plan: docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
fixing_plan: docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
origin_child_dir: docs/plans/zircon_runtime/text/09
fixing_child_dir: docs/plans/zircon_runtime/text/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/presentation.rs;zircon_runtime/crates/zr_rhi/src/ui_surface.rs
---

# rhi-wgpu-ui-surface-present-stats-non-exhaustive-construction: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md`
- 来源执行切片：runtime_text_static_label_profile_baseline_exports_complete_frame_matrix
- 修复责任计划：`docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`runtime_text_static_label_profile_baseline_exports_complete_frame_matrix` — validate-matrix profiling lib test exits 101 before Text09 test execution

## 最低共享层根因

zr_rhi_wgpu ui_surface/presentation.rs:383 constructs non-exhaustive zr_rhi::UiSurfacePresentStats with a struct expression

## 架构修复验收

- The zr_rhi_wgpu consumer compiles through the UiSurfacePresentStats boundary, then Text09 M0 begins its measured WGPU capture.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

Open state: `待修复`; the coordinator must keep the validation ticket and route this Plan to repair work.
