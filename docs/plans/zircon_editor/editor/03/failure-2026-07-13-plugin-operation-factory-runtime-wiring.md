---
handoff_kind: failure
status: open
created_at: 2026-07-13
summary_slug: plugin-operation-factory-runtime-wiring
origin_plan: docs/plans/zircon_plugins/05-navigation.md
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/zircon_plugins/05
fixing_child_dir: docs/plans/zircon_editor/editor/03
related_code:
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_plugins/navigation/editor/src/plugin/registration/operations.rs
  - zircon_plugins/navigation/editor/src/bake_panel.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_navigation_editor -TargetDir E:/cargo-targets/zircon-navigation-m6-editor -SkipBuild
---

# Editor 03：插件 operation factory 与 Runtime 执行接线缺失

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/05-navigation.md`
- 来源执行切片：M6-T1 Navigation Bake/Clear editor contract
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：最低共享原因是 Editor 03 已计划但尚未落地的 operation factory、事务和 inverse 接线；Navigation 内另造执行旁路会复制事实源。

## 失败现象与复现证据

Navigation Bake Scene、Bake Selected 与 Clear 已注册为带 payload schema 的 pending edit operations，并有 framework `NavMeshBakeRequest` 映射，但 `EditorHostEventController::invoke_operation` 对没有 `EditorEvent` 的描述符固定返回 `EditCommandFactoryPending`。因此 `.zui` 按钮无法到达 controller/runtime；`UndoableEditorOperation` 仍仅为 display-name metadata，不能恢复生成资产。

## 最低共享层根因

Editor 03 计划 M3.2 的 `OperationCommandFactory`、operation-group→transaction 路由与真实 inverse 尚未实现；当前源码和计划记录都明确把 pending failure 当作过渡行为。

## 架构修复验收

- SDK/Editor registry 可为插件 operation 注册真实 command factory；host invocation 不再对已安装 factory 返回 pending。
- Bake/Clear factory 经类型化 runtime client 提交、轮询/订阅 progress 与 harvest/clear completion；不得让 editor 持有 Runtime `World`。
- Bake/Clear transaction 保存前后生成资产状态并通过统一 Editor transaction engine 完成 undo/redo。
- Navigation editor host 集成测试从 retained command route 调用到 runtime adapter，并重跑 M6 package gate。

## 禁止临时方案

- 禁止用 `with_event(OpenView)` 覆盖 edit operation、在 Navigation 内另造私有 undo 栈、仅测试 fake handler、直接持有 Runtime World，或把 pending failure 改成静默成功。
- 禁止 aliases、compatibility shims、silent fallback、duplicated truth、test-only bypasses 或 call-site exceptions。
- 禁止削弱测试或 M6 验收标准以隐藏失败。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
