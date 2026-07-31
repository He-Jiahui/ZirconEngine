---
plan: zircon-editor-07
failure: sample-grid-command-amplification
status: implemented-validation-pending-failure-open
session: editor07-domain-performance-failure-repairs-r3-20260718
related_code:
  - zircon_editor/src/ui/sample_grid
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes/sample_grid.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/sample_grid.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sample_grid
tests:
  - tools/tests/test_editor07_sample_grid_generation_contract.py
  - zircon_editor/src/ui/sample_grid/tests.rs
---

# Sample Grid Typed Generation Hard Cut

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与未完成项 |
|---|---|---|---|
| 2026-07-18 18:11 +08:00 | `implemented-validation-pending-failure-open` | 新增 immutable `SampleGridGeneration`、preformatted `SampleGridTick` 和 typed `SampleGridPoint`；固定 FNV-1a content generation 将 axis/range/ticks 与 points/selection 分为 static/dynamic identity。template projection 成为唯一构造点；`TemplatePaneSampleGridData` 删除旧 axis/range/`ModelRc` ticks/points 平行字段。surface/text/points painter、projection tests、paint fixture 与 Blend Space 视觉集成断言全部硬切到 typed slices，旧字段引用为 0。 | TDD RED 为 4/5（owner/projection/painter/host data 缺失），实现后 Python 合同 5/5；精确 rustfmt 与 scoped diff check 通过。Rust tests 已写：tick 预格式化、selection/drag 仅变 dynamic token、axis/tick 改变 static token；Cargo 因共享 Performance job 与 Coordinator source-bound gate未运行。Failure 不关闭：host dashed/diamond 仍展开多个 quad，EditorUI08 generation cache、Render13 bounded batch、规模/CPU p95/像素等价证据仍待下游完成。 |
| 2026-07-18 19:08 +08:00 | `implemented-validation-pending-failure-open` | 修正 generation dependency：point 位置由 x/y range 归一化，故 range 现在同时进入 static 与 dynamic token；axis label/tick-only 仍只改变 static，selection/drag 仍只改变 dynamic。该规则防止后续 dynamic point cache 在 range 改变时复用旧坐标。 | 新增静态合同的 range-input 与行为测试两项先 RED，完成后完整 Python 合同 5/5；精确 rustfmt 与 scoped diff check 通过。Rust 新增 `range_changes_update_static_and_dynamic_generation` 并强化 axis/tick 不改变 dynamic 的断言；共享 Runtime12 Cargo 仍运行，本行不声明 Rust test 已执行。Failure 继续 open。 |
