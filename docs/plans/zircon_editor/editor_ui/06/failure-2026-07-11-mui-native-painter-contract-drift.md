---
handoff_kind: failure
status: open
created_at: 2026-07-11
summary_slug: mui-native-painter-contract-drift
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/06
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/tests/host/retained_window/native_material_painter_mui_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes
  - zircon_editor/src/ui/retained_host/ui
plan_sources:
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - cargo test -p zircon_editor --lib --locked native_material_painter -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked component_showcase -- --test-threads=1
---

# Editor UI 06：MUI/native painter 当前合同漂移失败交接

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：Editor M1 当前源码完整单线程门禁；Editor 15 M1 追加 retained UI 精确分片
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`
- 交接原因：失败集中在 MUI 组件 token、状态优先级、几何与 native painter parity，最低共享原因不属于 Editor 内核或导出流水线。

## 失败现象与复现证据

Editor M1 当前源码 08:31 binary 的完整门禁中，MUI/component/native-painter 聚类仍为 73 项失败：其中 `native_material_painter_mui_primitives` 22 项、基础 painter 8 项，其余集中于 alerts/buttons/list rows/metrics/showcase。独立 circular-progress exact 为 0/1（0.00s）：实际 RGBA `[42, 166, 184, 255]`，旧断言为 `[53, 199, 208, 255]`。

该聚类归 Editor UI 06 的组件 token、state priority、geometry 与 native painter parity。后续必须先确认 design token 当前单源，再区分产品绘制回归和旧 palette 快照漂移；禁止复制旧颜色常量、恢复旧 painter 路径或按测试名称特判。

## 最低共享层根因

最低已证实边界是 Editor UI 06 的共享 design token、component state projection 与 native painter
消费合同存在整体漂移；component showcase 的结构测试还要求退役的源码形状。功能 owner 必须先裁决当前 typed
组件 DTO 和中央 token，再同步生产实现与测试，不能从上层 retained host 添加兼容结构。

## 架构修复验收

- 共享 token/state/geometry focused tests 与 component showcase 精确组全绿。
- `native_material_painter` 与 `component_showcase` 原始复现命令全绿。
- 重跑 Editor M1 分区与完整门禁，确认没有恢复旧 painter 或旧 DTO。

## 禁止临时方案

- 禁止恢复旧 painter 路径、旧 DTO、局部 palette 或按测试名特例。
- 禁止批量改像素断言、忽略失败或降低组件合同覆盖。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Editor UI 06 / Editor M1 | MUI/native painter/component showcase parity | `未通过-73项待功能owner处理` | 2026-07-11 | 完整门禁按功能归类 73 项；circular-progress exact 0/1，当前/旧期望 RGBA 分别为 `[42,166,184,255]` 与 `[53,199,208,255]`。先从共享 design token/palette 最低层向上复验。 |
| Editor UI 06 / Editor M1 | 当前源码完整门禁复核 | `未通过-失败集合未变化` | 2026-07-11 | 08:31 当前源码 binary 完整执行 2930 项为 2763/133/34（2258.13s）；与 06:17 门禁逐项比较，133 个失败名 added=0、removed=0，本计划 73 项归属不变。同一 binary circular-progress exact 0/1，当前/旧 RGBA 仍为 `[42,166,184,255]` / `[53,199,208,255]`。 |
| Editor UI 06 / Editor03+08 M1 | 当前全量门 component/native painter 回归复现 | `未通过-继续由功能owner处理` | 2026-07-12 | 受管 job `520d85713df249afae31661a7697ad07` 再次复现 MUI primitive、alert/paper/button/list/tree/status、component-showcase state 与 reference-well 投影失败；代表项包括 `native_template_painter_draws_mui_circular_progress_ring`、`workbench_toast_paints_status_mark_action_and_close`、`component_showcase_option_and_action_callbacks_are_rust_wired`。原始失败列表见 `D:/cargo-targets/editor08-m1-rerun4-20260712.log`；必须从共享 token/state/geometry owner 自底向上修复，不得复制旧色值或增加按测试名特例。 |
| Editor UI 06 / Editor15 M1 | 当前 editor binary retained showcase 精确分片 | `未通过-6项组件合同待owner处理` | 2026-07-12 | `ui::retained_host::ui::` 为 94/102，8 项失败；其中 6 项归 component showcase/reference/structure owner：`reference_component_tests` 1、`structure_component_tests` 2、`component_showcase` 3。代表 panic 为测试仍要求 `TemplatePaneActionData` 源码声明，owner 应按当前 typed component DTO 硬切断言，不得恢复退役结构。另 2 项 Build/Export pane 投影已单独写入 Editor15 子计划。 |
| Editor UI 06 / Editor09 M1 | 当前源码完整门停滞前复现 | `未通过-继续由功能owner处理` | 2026-07-13 | job `e81ed19d256f40c28ddb2437e9a18460` 在外部停滞前再次记录基础 native painter 8 项、MUI primitives 22 项以及 alert/paper/notification/material-lab 等组件失败；日志 `.codex/tmp/editor09-m1-full-lib-test-r2-20260713.log`。该轮只证明聚类仍 open，不用部分执行数替代完整门。 |

## 修复结果与回传

- 状态：`open / 待修复`；先跑 native painter 与 component showcase focused groups，再向上重跑 Editor M1。
