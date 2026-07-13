---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: editor-manager-weak-runtime-caller-lifetime
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_runtime/runtime/02
related_code:
  - zircon_runtime/src/core/runtime/weak.rs
  - zircon_runtime/src/core/runtime/state/service_entry.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/tests/host/manager/ui_asset_workspace_watcher.rs
tests:
  - .codex/tmp/zircon_editor-editor09-m1-4-source-authority-r6-20260713.exe tests::host::manager:: --nocapture --test-threads=1
resolved_at: 2026-07-13
---


# Runtime 02：弱 Runtime service 合同下调用方提前释放 root

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：ProjectAuthority manager fixture hard cut 的 83-test 回归
- 修复责任计划：`docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md`
- 交接原因：失败发生在 Runtime02 把 registry-owned `EditorManager -> EditorUiHost` 强 `CoreHandle` 硬切为 `CoreWeak` 后的调用方所有权合同；Editor09 不应恢复 service 强根，或在 asset watcher 测试里给 manager 增加特判。

## 失败现象与复现证据

Editor09 已把 16 处旧 Manager 工程夹具统一迁到 `ProjectAuthority` 并修正 renderable template
shader `.zmeta` schema。当前 Windows binary 的 ProjectAuthority template 测试通过，随后执行完整
Manager 窄分区：

` .codex/tmp/zircon_editor-editor09-m1-4-source-authority-r6-20260713.exe tests::host::manager:: --nocapture --test-threads=1`

结果为 82 passed、1 failed。唯一失败位于
`ui_asset_workspace_watcher::editor_manager_marks_and_recovers_stale_imports_from_watched_changes`：

- `manager_for` 在函数内创建 `CoreRuntime`、解析 `Arc<EditorManager>` 后只返回 manager；
- Runtime02 当前 hard cut 规定 registry-owned manager 仅保留 `CoreWeak`；
- 临时 Runtime 在 `manager_for` 返回前被释放；
- 随后 `manager.open_project(...)` 在 `EditorUiHost::runtime_core()` 返回 typed
  `CoreError::RuntimeUnavailable`。

日志：`.codex/tmp/editor09-project-authority-manager-suite-r6-20260713.log`。

## 最低共享层根因

最低边界不是 ProjectAuthority 或 watcher 行为，而是 Runtime02 新弱所有权合同尚未对调用方生命周期
写清：需要执行 Runtime-backed 操作时，谁持有外部 `CoreRuntime` root，以及 service handle 跨 root
生命周期后应如何失败。当前失败恰好证明 weak cut 生效，也证明测试 caller 仍依赖旧的 manager 强根
隐式续命。

## 架构修复验收

- 保持 `EditorUiHost` 只存 `CoreWeak`，不得恢复 `CoreHandle` 字段或强/弱双路径。
- Runtime-backed manager fixture 显式持有 `CoreRuntime` 到最后一次 manager 操作之后；生命周期负向测试
  继续证明 drop Runtime 后 manager 返回 `RuntimeUnavailable` 且不复活 root。
- Manager 83-test 窄分区自然产生 83 passed、0 failed summary。
- 原 `service-corehandle-retention-cycle` 的 128 次 fixture、failure/unwind 与产品启动验收保持独立有效。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止在 watcher test 重新构造第二 Runtime，禁止让 manager 或 watcher 强持有 Runtime root。

## 修复结果与回传

- 根因：The watcher test helper returned only Arc<EditorManager>, so the local CoreRuntime root was dropped before the one Runtime-backed open_project call; the weak-service contract correctly returned RuntimeUnavailable.
- 架构修复：The fixture now returns (CoreRuntime, Arc<EditorManager>) and each caller keeps the explicit external root alive for the manager scope. EditorUiHost remains CoreWeak-only and lifecycle negative tests remain unchanged.
- 验证：Scoped rustfmt and git diff --check pass. The prior r6 Manager suite proved the sole failure was RuntimeUnavailable at this helper. Current-source no-run retry is temporarily blocked earlier by the separately routed Frameworks05 RuntimeProfileId consumer cutover.
- 回传：Weak service caller lifetime is explicit in the test fixture without restoring a service strong root; final Manager 83/83 awaits current-source recompilation after Frameworks05 unblocks.
