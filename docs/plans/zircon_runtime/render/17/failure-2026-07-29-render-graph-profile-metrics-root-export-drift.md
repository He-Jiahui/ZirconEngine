---
handoff_kind: failure
status: source_complete_dynamic_validation_pending
created_at: 2026-07-29
summary_slug: render-graph-profile-metrics-root-export-drift
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
origin_workflow_node: M2
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_runtime/render/17
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/backend_types/graph_reports.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
tests:
  - cargo test -p zircon_editor --lib gateway:: --locked --jobs 1 --color never -- --test-threads=1
---

# Render17: RenderGraphPassProfileMetrics root export drift

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：Editor01 M2 runtime-frame demand hard cut，执行者 `editor01-runtime-frame-demand-hardcut-r1-20260729`
- 来源受管运行：job `640dc354cc38475daa1bd25e7217baf6` / run `bb226267623f4322839092a6f7365c15`
- 修复责任计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 交接原因：render graph pass profiling DTO 的定义、crate-private framework projection 和两处 graph execution consumer 同属 Render17；Editor01 不得改写运行时的模块可见性或导入边界。

## 失败现象与复现证据

2026-07-29 11:05 CST，source-manifest fingerprint `d3510f7e2cd5a4ade996a4887d77103aee3b1c710e29b2714427666e9503eed5` 的受管 focused gate 自然终态为 `exit 101`，尚未执行 `gateway::` 测试。`zircon_runtime` 编译报告两个相同的 E0432：

```text
unresolved import `crate::core::framework::render::RenderGraphPassProfileMetrics`
  --> .../core/.../render/execute_graph_stage.rs:5:5
  --> .../graph_execution/render_graph_execution_record.rs:6:39
note: `core::framework::render::backend_types::RenderGraphPassProfileMetrics` exists
but is inaccessible
```

原始 stderr：`.codex/state/session-coordinator/cargo-runs/640dc354cc38475daa1bd25e7217baf6/bb226267623f4322839092a6f7365c15/stderr.log`。同一运行中的 Layout21 `BatchDrawPlanStats` E0063 已由独立工单接收。

## 最低共享层根因

`RenderGraphPassProfileMetrics` 已定义于 `backend_types/graph_reports.rs`，并由 `backend_types.rs` 重导出，但 `core::framework::render::mod.rs` 的既有契约根没有同步重导出。图执行 consumer 按约定从该契约根导入，因中间层停留造成两个编译点同步失联。该问题是 Render17 PF-M1 新增 pass profile metrics 的 projection 迁移不完整，不是 consumer 的局部类型推断问题。

## 架构修复验收

- 在既有 `core::framework::render` 契约根完成 `RenderGraphPassProfileMetrics` 的唯一 crate-private 投影，保持定义仍由 `backend_types::graph_reports` 所有。
- `execute_graph_stage` 和 `render_graph_execution_record` 保持从契约根消费，二者与 producer 在同一 managed current-source compile 中通过。
- 为根 re-export 增加或更新聚焦编译/契约测试，防止新增 profile DTO 再次只停在中间模块。
- 在稳定完整输入 attestation 下通过 Render17 的受管 Rust 1.94.1 runtime lib 验证；之后重新创建 Editor01 source-bound reservation，确认原 gateway 命令能越过两个 E0432 后才继续上层验收。

## 禁止临时方案

- 不得让 consumer 直接导入 `backend_types` 私有子路径，或创建平行 alias/兼容模块。
- 不得把 `RenderGraphPassProfileMetrics` 删除、内联成裸字段，或用调用点局部结构体替代共享 DTO。
- 不得将本次源快照上的 `exit 101` 归类为 Editor01 gateway 失败或作为 Render17 fixed evidence。

## 修复结果与回传

Open state: `core::framework::render`契约根已唯一crate-private投影`RenderGraphPassProfileMetrics`，producer/consumer继续通过该根消费，且根导出聚焦契约测试已落地；待受管current-source编译、独立复审及Editor01新immutable manifest upward gate后以`fixed-*`回传，不能复用失败job `640dc354cc38475daa1bd25e7217baf6`。

## 产出记录与时间

| 时间 | 范围 | 状态 | 完成项与后续门禁 |
| --- | --- | --- | --- |
| 2026-07-29 11:05 CST | Render17 PF-M1 pass profile DTO projection | failure open | 已从 Editor01 受管终态提取两个 E0432，并定位为 `graph_reports -> backend_types -> framework render root` 的最后一层投影遗漏。等待 Render17 修复、受管验证、独立复审和 fixed return。 |
