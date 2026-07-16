# Editor07 SelectionModel consumer hard-cut output record

Plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
Milestone: Editor05 inbound failure return / SelectionModel consumer hard cut
Status: in_progress
Files: ["docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md", "docs/plans/zircon_editor/editor/07/failure-2026-07-16-viewport-selection-model-consumer-hard-cut.md", "docs/plans/zircon_editor/editor/07/2026-07-16-selection-model-consumer-hard-cut-output-records.md", "docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md", "docs/plans/zircon_editor/editor/05/2026-07-16-m1-selection-mode-stack-output-records.md", "docs/zircon_editor/scene/modes.md", "docs/zircon_editor/core/commands.md", "zircon_editor/src/core/editing/history.rs", "zircon_editor/src/scene/selection/selection_model.rs", "zircon_editor/src/scene/selection/tests.rs", "zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs", "zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_edit_mode_projection.rs", "zircon_editor/src/scene/viewport/controller/scene_viewport_controller_frame_selection.rs", "zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_interaction.rs", "zircon_editor/src/scene/viewport/controller/scene_viewport_controller_reset_from_scene.rs", "zircon_editor/src/scene/viewport/controller/scene_viewport_controller_selection.rs", "zircon_editor/src/ui/binding_dispatch/inspector/apply.rs", "zircon_editor/src/ui/binding_dispatch/inspector/subject_path.rs", "zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs", "zircon_editor/src/ui/workbench/startup/editor_state_construction.rs", "zircon_editor/src/ui/workbench/startup/editor_state_project.rs", "zircon_editor/src/ui/workbench/state/editor_state.rs", "zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs", "zircon_editor/src/ui/workbench/state/editor_state_field_updates.rs", "zircon_editor/src/ui/workbench/state/editor_state_play_mode.rs", "zircon_editor/src/ui/workbench/state/editor_state_selection.rs", "zircon_editor/src/ui/workbench/state/editor_state_viewport.rs", "zircon_editor/src/tests/editing/editor_projection.rs", "zircon_editor/src/tests/editing/state.rs", "zircon_editor/src/tests/editing/viewport.rs", "zircon_editor/src/tests/host/binding_dispatch.rs"]

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `IMPLEMENTED / COORDINATOR BLOCKER FIXED / MANAGED VALIDATION PENDING` | 2026-07-16 | 已将 28 处 Workbench/binding 生产 consumer 与 controller/test 调用整体硬切到 `SelectionModel` active-domain API，并删除 `SceneViewportController::selected_node` / `set_selected_node`。非选择编辑保持有序多选与 primary；删除过滤失效实体并保留存活集合；PIE 进入时把 Edit 集合复制到独立 Play 域，退出恢复完整双域快照；history 仅为 create/delete/import 保存有序 before/after selection snapshot，undo/redo 恢复完整集合。新增行为测试和静态旧 API 守卫。全源码扫描为 0（排除不同类型 widget reflector），`rustfmt` 与 `git diff --check` 通过。Coordinator01 已回传 `fixed-2026-07-16-stale-session-pending-cpu-reservation-starvation.md`，schema 41 生产回放证明 canonical payload、stale atomic cleanup、orphan handoff 与 FIFO 前进；当前仅剩 Editor07 current-source managed Cargo 测试尚未执行，本记录不宣称 Rust 验证已通过。 |
| `INDEPENDENT REVIEW P0/P1/P2 = 0/0/0` | 2026-07-16 | 独立复审按最终共享树检查 controller、selection、history、Workbench、PIE、tests 与计划状态；最初发现的 PIE 重复同域断言和默认选择双层 `Option` 编译错误均已修复，最终未发现 Critical/Important/Minor 问题。审查明确未把未执行的 managed Cargo gate 计为通过。 |

## 实现边界

- 选择唯一事实源仍是 Editor05 的 `SelectionModel`；Editor07 不新增 host
  cache、单值 facade 或兼容 wrapper。
- binding、snapshot、startup、inspector、intent、viewport 与 PIE 直接使用
  active-domain API。单选操作通过 `select_only_active` 明确表达覆盖语义。
- `HistorySelectionSnapshot` 只由 create/delete/import 保存完整有序集合；
  update、reflection、batch 和 gizmo 命令不再借 `Option<NodeId>` 修改多选。
- 本记录不提升 Editor07 图编辑 M1，也不宣称 Editor05 M1.1 完成；内建模式、
  command-eval host 投影、overlay provider 与混合 `Cargo.lock` 仍是独立门禁。

## 验证证据

- 静态旧调用扫描：相关 viewport `selected_node` / `set_selected_node` 为 0；
  同名 widget-reflector 是不同类型与功能，不在本迁移范围。
- `rustfmt --edition 2021 <exact Rust paths>`：通过。
- `git diff --check -- <exact support scope>`：通过。
- managed Cargo 先前被 reservation `39d9c5788f09464fb20ea4c761164db4`
  拒绝；Coordinator01 已以 schema 41 修复并回传
  `docs/plans/zircon_editor/editor/07/fixed-2026-07-16-stale-session-pending-cpu-reservation-starvation.md`。
  生产回放通过，Editor07 current-source Cargo 本体仍待执行；未绕过为本地 Cargo。

## 待完成

- 获得 Windows managed CPU lane 后编译 current source，并执行
  `scene::selection`、`tests::editing`、`tests::host::binding_dispatch`。
- fresh independent review 必须为 P0/P1/P2 0，随后通过 lifecycle key 返回
  viewport fixed artifact；review 已为 0/0/0，仍须等待 managed Cargo，
  在此之前 viewport failure 维持 open。
