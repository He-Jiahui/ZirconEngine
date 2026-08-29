---
handoff_kind: fixed
status: fixed
created_at: 2026-08-29
summary_slug: server-slow-workspace-observation-test-teardown
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/tests/test_server.py
tests:
  - python -u -B -m unittest -q tools.session_coordinator.tests.test_server
  - python -u -B -m unittest -v tools.session_coordinator.tests.test_server.ServerTests.test_foreground_mutation_is_not_blocked_by_slow_workspace_observation
resolved_at: 2026-08-29
---

# Coordinator01 slow workspace observation test teardown

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：unoccupied failure-chain sweep of the current Coordinator server suite
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：失败位于 Coordinator01 自有的 Windows server regression teardown，且不需要修改产品代码或其他计划的路径。

## 失败现象与复现证据

完整 `test_server` 套件运行 74 项后唯一失败为
`test_foreground_mutation_is_not_blocked_by_slow_workspace_observation`。业务断言已经通过，
但 `TemporaryDirectory` 清理 `repo` 时返回 Windows `WinError 32`，表明仍有进程使用该目录。
同一测试连续独立运行八次均通过，证明这是完整套件负载下的 teardown 时序缺口。

## 最低共享层根因

测试放行被阻塞的 `apply_scan` 后，只等待 maintenance worker 一秒，随后立即离开临时目录。
放行后的真实 workspace scan 会启动 Git 子进程；完整套件负载或 Defender I/O 下它可能超过一秒。
此时 daemon worker 仍在运行，Git 仍以 fixture repository 为工作目录，Windows 因而拒绝删除目录。
生产 `RunningCoordinator.stop()` 已等待最多三十秒并拒绝静默遗留 maintenance worker，缺口仅在该测试手写的 teardown。

## 架构修复验收

- teardown 必须观测真实 `apply_scan` 已完成，再删除临时 repository。
- foreground 与 maintenance worker 均必须在退出 fixture 前被确认终止。
- 保留原有核心断言：慢 workspace observation 不得占用前台 `session.register` mutation lane。
- focused regression 与完整 74 项 server suite 均通过，且不出现临时目录占用错误。

## 禁止临时方案

- 不得忽略 `TemporaryDirectory` 清理错误或增加全局删除重试。
- 不得跳过真实 `apply_scan`、mock 掉 Git workspace observation 或削弱前台并发断言。
- 不得修改生产 shutdown timeout 来掩盖测试自身未等待 worker 的问题。

## 修复结果与回传

- 根因：The suite-specific teardown released the blocked workspace scan but waited only one second before TemporaryDirectory cleanup, so the real Git scan could still hold the Windows fixture repository under load.
- 架构修复：The regression now records real apply_scan completion and requires the foreground registration thread, workspace scan, and maintenance worker to terminate before the temporary repository leaves scope; production shutdown behavior is unchanged.
- 验证：Source commit e451a465ef7f80d8b816677ee056143d7a80b5a2; focused regression passed 1/1 in 10.863s; complete tools.session_coordinator.tests.test_server passed 74/74 in 506.591s; exact diff check passed.
- 回传：Coordinator01 server teardown now waits for the actual workspace scan and both worker threads, preserving the nonblocking foreground mutation contract without Windows fixture-directory leakage.
