---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-13
summary_slug: unmanaged-artifact-readonly-cleanup-wedge
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/artifact_governance.py
  - tools/session_coordinator/windows_tree_delete.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/tests/test_artifact_governance.py
  - tools/session_coordinator/tests/test_windows_tree_delete.py
tests:
  - python -X dev -W error::ResourceWarning -m unittest tools.session_coordinator.tests.test_artifact_governance -v
  - python -X dev -W error::ResourceWarning -m unittest tools.session_coordinator.tests.test_windows_tree_delete -v
  - powershell -File tools/zircon-session.ps1 artifact cleanup -Json
resolved_at: 2026-08-14
---


# Coordinator01: unmanaged artifact readonly cleanup wedge

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：snapshot 1707 combined Failure closeout managed validation gate
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 独占 unmanaged artifact 扫描、删除与 managed work admission。

## 失败现象与复现证据

`artifact audit` 只报告
`E:\ZirconBuilds\runtime15-lock-poison-production-view-review-20260813-r1`；无注册
Session、Cargo job、validation copy、cleanup reservation 或持有进程。服务端单项
`artifact cleanup` 返回该目录于 `failed` 和 `remaining`，随后 `cargo acquire test
--ephemeral` 持续被 `unmanaged_artifacts_detected` 拒绝。该副本的 Git object 文件带
Windows `ReadOnly` 属性，普通 `shutil.rmtree` 无法删除。

## 最低共享层根因

`ArtifactGovernanceService.cleanup()` 对已重验的 unmanaged 目录直接调用
`shutil.rmtree`，只记录笼统 OSError 失败；没有对 Windows 只读文件做局部清除属性后
重试。因此一个已确认无归属、可删除的 Git 副本可永久阻断所有 managed Cargo 和
validation-copy admission。

## 架构修复验收

- 只在已解析、已重验的 candidate 子树内对删除失败路径清除只读位并重试原操作。
- symlink/reparse point、managed path 与 candidate 边界门禁保持不变。
- Windows 只读 Git object 回归必须得到 started -> deleted，目录实际消失。
- 原始 `artifact cleanup` 删除该单一候选，随后 `artifact audit` 为空且 managed Cargo admission 恢复。

## 禁止临时方案

- 不得手工强删、全盘 chmod、跳过 artifact gate 或将失败目录加入 managed 白名单。
- 不得吞掉删除异常、伪造 deleted 事件或放宽路径重验。

## 修复结果与回传

- 根因：ArtifactGovernanceService used plain shutil.rmtree without Windows read-only retry, durable singleflight reservation, handle-bound containment, reparse rejection, or filesystem identity checks, allowing one unowned Git copy to wedge managed admission.
- 架构修复：Artifact cleanup now reserves the exact target durably, revalidates all managed overlaps and filesystem identity, deletes through a Windows handle-bound tree remover that clears read-only only inside the bound tree, rejects reparse/hardlink escape, and reconciles interrupted reservations fail-closed.
- 验证：artifact+Windows deletion tests 24/24 passed with ResourceWarning fatal, including a read-only Git object, concurrent singleflight, reservation overlap, junction replacement, hardlink attribute isolation, and restart recovery. Production schema-60 rollout aed57dc... cleaned the original candidate and restored managed Cargo admission; successor dc0144... remains healthy.
- 回传：Unmanaged read-only Git artifacts are now removed by a durable handle-bound deletion transaction without weakening containment or managed-work admission.
