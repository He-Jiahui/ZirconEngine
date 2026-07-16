---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: cargo-release-retains-live-child-process-lock
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/tests/test_cargo_jobs.py
tests:
  - .\tools\zircon-session.ps1 -Json cargo list
  - cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
resolved_at: 2026-07-14
---


# Tooling 01：Cargo release 保留活跃子进程与 artifact 锁

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：Editor02 M1 测试阶段，`core-min scene` generation 修复回归
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：复用池是否可再次分配、作业 PID 树是否已经退出属于协调器 Cargo 生命周期与监督边界，Editor02 不得通过新建绕行 target 或手工清理其他 Session 进程伪造通过。

## 失败现象与复现证据

2026-07-14 12:40:54，Runtime04 作业 `c417278b23da4db291748bad1cf6d11d` 以退出码 `130` 完成，并于 12:40:56 被协调器标记 `released`。协调器随后允许 Editor02 作业 `4a421137a9ca48ab8b3a11558a742897` 复用相同 compatibility key `569e0d4b772933e1ab3d593a42ad81224230969e2a1138986c1f66f825584999` 与 target。

实际 Windows 进程树仍包含原作业的 `cargo.exe` PID `34940`、子 `cargo.exe` PID `3300` 与 `rustc.exe` PID `35876`，并继续使用同一 target。新作业因此只输出 `Blocking waiting for file lock on artifact directory`，没有开始 Editor02 测试。Editor02 已停止自己的等待进程并以 `130` 结束/释放作业；未终止 Runtime04 的进程树。

## 最低共享层根因

`cargo finish`/`cargo release` 只根据数据库作业状态开放复用池，没有在 Windows 上证明登记 PID 的 Cargo 子进程树已经退出。父包装进程提前结束或被中断时，仍活跃的 cargo/rustc 后代可以继续持有 artifact 锁，而 `acquire` 已把该 target 视为可复用。

## 架构修复验收

- 增加 Windows 进程树夹具：父包装进程退出但 Cargo 子进程仍存活时，target 不得被第二作业获取为可用复用池。
- `finish`/`release`/orphan cleanup 必须形成明确、可审计的进程树退出契约；若后代仍活跃，应返回 typed 状态或保持池占用，不能静默开放 target。
- 修复后重跑 Tooling01 Cargo job 单测与真实双作业复用夹具，再重跑 Editor02 原始 `cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked` 门禁。

## 禁止临时方案

- 不得让调用方手工选择新 target、轮询 artifact 锁文件或强杀其他 Session 的 cargo/rustc 进程。
- 不得仅增加固定 sleep、忽略退出码 `130`，或在数据库中把 released target 永久弃用来掩盖监督缺口。
- 不得添加兼容别名、静默 fallback、重复真相、测试专用绕过或调用点特例。
- 不得削弱测试或计划验收条件来隐藏失败。

## 修复结果与回传

- 根因：CargoJobService previously changed finish and release state without proving the registered Cargo process tree had exited; acquire considered only leased and running rows, so a live descendant could retain an artifact lock after the target was reused.
- 架构修复：Added Windows process-tree observation, persisted observed/live/exited state in schema 30, and enforced the same check in finish, release, orphan reconciliation, and target reuse. A live tree now retains ownership and returns cargo_process_tree_alive with audited PIDs.
- 验证：Focused Cargo lifecycle regressions passed: live root; exited root with live descendants; release/reuse rejection; orphan reconciliation; normal release/reuse. Live daemon Schema 30 probe rejected finish while a managed process was alive and accepted finish then release after tree exit with persisted empty livePids.
- 回传：Editor02 may rerun its original core-min scene gate through the managed Cargo lifecycle; no alternate target or foreign-process termination is required.
