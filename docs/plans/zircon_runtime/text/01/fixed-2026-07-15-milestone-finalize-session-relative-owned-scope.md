---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
resolved_at: 2026-07-15
summary_slug: milestone-finalize-session-relative-owned-scope
origin_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/text/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/tests/test_git_finalize.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_git_finalize.GitFinalizeTests.test_milestone_commit_keeps_attributed_tracked_change_after_global_baseline_absorbs_hash -v
---


# Tooling01: milestone finalize 丢失 Session 相对归属变更

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 来源执行切片：Text01 FR-M3 accepted milestone commit
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：最低共享原因位于协调器 `GitFinalizeService._require_owned_scope`，不是 Text runtime 代码或产品验证。

## 失败现象与复现证据

Text01 FR-M3 的精确 25 文件 manifest、current-hash attribution、managed validation 与独立 review 均已 accepted。标准 `milestone commit` action `d91068c6cdab41d4b78161350c9f8b74` 仍返回 `finalize_path_unchanged`，因为这些 tracked 文件的当前 hash 曾被后续全局 baseline 捕获，但文件仍相对当前 HEAD 修改。

预期正常路径是：attribution 证明当前内容属于该 Session；当前工作树与当前 HEAD 的差异证明 manifest 具有真实提交内容。全局 baseline 只负责共享工作树健康状态，不能抹除已经归属的 Session 相对变更。

## 最低共享层根因

`GitFinalizeService._require_owned_scope` 仅从 `BaselineService.diff()` 构造 change set。全局 baseline 合法推进或历史捕获 dirty hash 后，Session 已归属的 tracked 内容可能与 baseline 相同、但仍与 HEAD 不同，于是被错误归类为 unchanged。

## 架构修复验收

- focused lower-layer test 必须先复现 `finalize_path_unchanged`，修复后通过并提交精确 tracked 内容。
- `test_git_finalize` 全模块通过，现有 omitted-path、unattributed-path、ignored-path 与 deletion 门不得弱化。
- Text01 原标准 M3 coordinator action 成功生成真实 commit、accepted milestone attempt 与一次 WeCom 通知。

## 禁止临时方案

- 不得修改 Text01 文件制造内容 churn。
- 不得直接 Git commit、maintenance finalize 或接受外部 dirty baseline 来绕过 gate。
- 不得添加 alias、兼容 shim、silent fallback、重复 truth、测试专用 bypass 或调用点特例。

## 修复结果与回传

- 根因：Finalize owned-scope used mutable global baseline diff and lost attributed tracked deltas that still differed from HEAD.
- 架构修复：Compare each current-hash-attributed Session path against the current HEAD checkout while preserving all attribution, lease, omitted-path, staged-blob, Failure, and secret gates.
- 验证：Focused red-green 1/1; test_git_finalize 26/26; test_workflow_commit 11/11; compileall exit 0.
- 回传：Text01 FR-M3 may retry its unchanged exact manifest through the standard milestone commit after coordinator reload.
