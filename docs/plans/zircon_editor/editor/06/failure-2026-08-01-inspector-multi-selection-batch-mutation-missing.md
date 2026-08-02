---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: inspector-multi-selection-batch-mutation-missing
origin_plan: docs/plans/zircon_editor/editor/06-ui-extension-framework.md
origin_workflow_node: M2
fixing_plan: docs/plans/zircon_editor/editor/06-ui-extension-framework.md
origin_child_dir: docs/plans/zircon_editor/editor/06
fixing_child_dir: docs/plans/zircon_editor/editor/06
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/tests/editing/reflected_command.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_selection.rs
tests:
  - reflected_inspector_batch_mutates_all_selected_nodes_in_one_history_record
  - cargo test -p zircon_editor --lib --locked
---

# Editor06: Inspector multi-selection batch mutation missing

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
- 来源执行切片：M2 Inspector 双层定制，`多选批量改值` 验收项
- 修复责任计划：`docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
- 交接原因：该缺口属于 Editor06 内部 Inspector adapter 的本地修复，不转移到其他编号计划。
- 当前执行会话：`editor06-document-toolkit-hardcut-r1-20260801`

## 失败现象与复现证据

`EditorState::apply_inspector_changes` 读取
`viewport_controller.selection().active_primary()`，仅为主选中实体生成 reflected commands。现有
`reflected_edit_preserves_active_multi_selection` 仅断言编辑、撤销和重做后选集不丢失；它没有断言同一
Inspector 修改会写入每一个 active item，也没有断言它们共享一个历史条目。

因此当 Edit domain 中存在多个选中实体时，Inspector 的批量修改实际只变更 primary，和 Editor06 M2 的
“多选批量改值，撤销为单条历史”合同不符。

## 最低共享层根因

Editor05 的 `SelectionModel` 已持有有序 active set，Editor03 的
`execute_scene_commands(..., MergeMode::Disable)` 已在一个 transaction scope 内提交命令。缺口仅在
Editor06 Inspector adapter 没有枚举现有 active set；不能通过重建 SelectionModel 或新增 history stack 修复。

## 架构修复验收

- Inspector 批量路径按 active Edit-domain selection 的稳定顺序为每个实体准备 reflected node 与 dynamic
  component updates；任意 preparation failure 在 transaction 之前返回，不能发布部分世界修改。
- 全部命令仍只调用一次既有 `execute_scene_commands`，产生一条 Global transaction history record；undo/redo
  必须对所有目标共同往返，selection snapshot 保持不变。
- 动态字段继续按每个目标的实际 reflected schema/value 解析，不能从 primary 复制缓存的 typed value。
- focused regression 与受管 `cargo test -p zircon_editor --lib --locked` 成功后，再经独立复审回填 fixed record。

## 禁止临时方案

- 禁止在 Editor06 复制或缓存 Editor05 选集，或退回 primary-only fallback。
- 禁止按实体分别启动 transaction、合成多个 history records，或新建第二套 undo stack。
- 禁止在缺少某一目标的字段/组件时跳过该目标并声明批量成功。

## 修复结果与回传

Open state: 待修复; no pass is claimed. Forward fix in progress: current session owns the Inspector adapter and focused regression. Editor05 selection
and Editor03 transaction owners remain authoritative and are not modified by this repair.

## 产出记录与时间

- 2026-08-01：状态 `fixing`。已复现 primary-only mutation 根因，确认 SelectionModel 与 transaction scope
  已提供所需基础能力；已创建本计划 failure record，开始以多目标 mutation、单 history record、undo/redo
  往返回归驱动前向修复。
- 2026-08-02：状态仍为 `fixing`。Inspector adapter 已按稳定 active selection 顺序在同一 transaction 前
  准备每个目标的 reflected updates，并只调用一次既有 `execute_scene_commands`；回归
  `reflected_inspector_batch_mutates_all_selected_nodes_in_one_history_record` 覆盖全选集变更、单条 Global
  history、undo/redo 与 selection 保持。该实现随当前 Editor06 受管 broad gate 冻结，尚待独立复审和
  terminal validation 后才可 fixed return。
