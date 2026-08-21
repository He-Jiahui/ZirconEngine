---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-19
summary_slug: validation-stream-inherited-pipe-hang
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/tests/test_workspace_copy_terminal_status.py
  - tools/session_coordinator/workspace_copy_terminal.py
---

# validation-stream-inherited-pipe-hang: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Validation-copy terminal stream collector
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Validation-copy terminal stream collector` — Use a validation root process that exits while a descendant keeps inherited stdout or stderr open; ValidationCopyTerminalLifecycle.collect blocks forever in reader.join().

## 最低共享层根因

Terminal collection waits for unbounded pipe EOF after the root process exits and has no reader deadline or typed truncated-capture terminal path.

## 架构修复验收

- Terminal output is drained without waiting for inherited pipe EOF after root exit and `after_root_exit` cleanup.
- Windows blocking-I/O cancellation is a bounded fallback; closing a buffered `TextIOWrapper` from another thread must not reintroduce an unbounded lock wait.
- Normal large dual-stream and invalid-UTF8 terminal evidence remains bounded and durable.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- RED r1：可控 `InheritedPipe.read()` 证明原始无界 `reader.join()` 可永久占住 lane；r1 ticket `d08e03b75d434a88a50015e99fb11a28` 的 8/8 + 61/61 虽通过，但独立真实进程复现显示 reader 超时后的 `TextIOWrapper.close()` 会等待缓冲读锁，collector 在 8 秒后仍存活，因此该 ticket 不可作为提交证据。
- RED r2：真实 root process 启动一个继承 stdout/stderr 且持续存活的 descendant 后立即退出。r1 collector 不能在 8 秒内返回；直接从另一线程 `os.close()` 底层 fd 同样阻塞，证实必须避免阻塞式 EOF/close 路径。
- 修复 r2：reader 对真实 pipe fd 使用非阻塞 `os.read()`，root 运行期间持续排空，root exit 与 `after_root_exit` 后排空当前内核缓冲即结束，不等待 descendant EOF。异常平台 fallback 共享 5 秒 deadline，并在 Windows 用 `CancelSynchronousIo` 中断 reader 自己的同步 I/O，再给 1 秒收敛期；主线程不再跨线程关闭 buffered stream。
- 异常优先级：若 `after_root_exit` 的 Job/tree cleanup 自身失败，collector 仍发出 stop/cancel，但保留 cleanup 原异常为主错误；只有 cleanup 成功时才可能报告 typed stream timeout。
- GREEN r2：`python -m unittest tools.session_coordinator.tests.test_workspace_copy_terminal_status -v` 为 8/8；真实 descendant 在 3 秒门限内收口，large dual streams 保留 universal-newline 与 65,536 字符尾部契约，invalid UTF-8、同步/异步 terminal evidence、foreign session 与 malformed durable command 均通过。
- 更广 GREEN r2：`python -m unittest tools.session_coordinator.tests.test_workspace_copy -v` 为 61/61（271.419 秒），覆盖 run/start/cancel、restart recovery、benchmark Job Object、cleanup reservation、missing-stream fallback 与 completion hook。
- 尚待受管收口：immutable validation ticket、candidate/commit/rollover 完成前保持 `status: open`。
