---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: cargo-pid-reuse-identity-guard
origin_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor_layout/15
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/processes.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/tests/test_cargo_jobs.py
  - tools/session_coordinator/tests/test_database.py
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
tests:
  - python -m unittest tools.session_coordinator.tests.test_cargo_jobs
  - python -m unittest tools.session_coordinator.tests.test_database
  - python -m unittest tools.session_coordinator.tests.test_server
  - Invoke-Pester -Script .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1 -TestName 'Coordinator supervisor role'
  - .\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1 -Package zircon_editor -VerboseOutput
resolved_at: 2026-07-17
---


# Tooling 01：Cargo PID 复用身份守卫

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 来源执行切片：S15.4/S15.5 Blend Space Preview toolbar 的当前源码合同与 ignored production-window 截图门禁。
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：受管 Cargo target 的进程存活判断是共享 coordinator 生命周期边界；编辑器切片不得通过改用其他 target、复用旧二进制或终止外部进程来伪造截图验证。

## 失败现象与复现证据

2026-07-14，`validate-matrix.ps1 -Package zircon_editor -VerboseOutput` 在启动 Cargo 前四次返回 `cargo_process_tree_alive`。孤儿作业 `b0821bf60f2644beaea1cd165ed9414d` 记录 PID `48464`，target 为 `D:\\cargo-targets\\zircon-engine\\pool\\841a130ffbd3fd2e938e76b488988119044b676acced751dae7166d95d7f1025`。Windows 当前 PID `48464` 实际为 2026-07-14 15:53 +08:00 启动的无关 `python -m renderdoc_mcp.server`，且 coordinator 没有运行中的受管 Cargo 作业。

期望行为：只有与登记 Cargo 根进程具有同一创建身份的根或其可证明的后代才能占用 target；PID 被系统复用后必须视为旧根已退出，不能阻断新作业。

## 最低共享层根因

`CargoJobService` 把持久化的裸 PID 传给 `process_is_alive` / `live_process_tree_pids`，但没有持久化并比对 PID 的进程创建时间。`processes.py` 已可读取创建时间，尚未成为 Cargo 作业观察契约的一部分，因此 PID 复用被误判为仍存活的 Cargo 进程树。

## 架构修复验收

- 新作业登记 root PID 时持久化其进程创建身份；观察时 root identity 不匹配必须视为原 root 已退出。
- `validate-matrix.ps1` 的长寿命 PowerShell root 作为 supervisor 记录：finish/release 忽略该包装器自身，但仍必须锁住活跃 Cargo/rustc 后代；直接登记的 Cargo root 继续严格锁定。
- 保留已有“父进程退出但真实 Cargo/rustc 子进程仍存活”保护；同一 root identity 下的后代仍不得释放或复用 target。
- focused coordinator 单测覆盖 PID 复用不阻塞 orphan reconciliation / target reuse，且同一身份或活后代仍阻塞。
- 重跑来源的 managed `zircon_editor` package 矩阵、focused Blend Space 合同和 ignored screenshot capture。

## 禁止临时方案

- 不得改用新 target、复用旧测试二进制、手工删库或终止 PID `48464`；daemon 只能在没有活跃受管 Cargo 作业后走 coordinator 的受控 lifecycle 重载。
- 不得仅按命令名猜测 Cargo、固定 sleep、忽略孤儿记录、添加调用点特例或测试专用绕过。
- 不得削弱现有活子进程保护或来源截图验收条件。

## 修复结果与回传

- 根因：CargoJobService formerly persisted and observed a raw root PID without a process-creation identity, so Windows PID reuse could make a terminal legacy job appear live and pin its target.
- 架构修复：Coordinator schemas 31/32 added root_process_creation_time and root_process_kind; Cargo lifecycle observation now validates root identity, preserves live Cargo/rustc descendants, treats supervisor wrappers separately, and keeps terminal legacy rows terminal. The authoritative daemon is now schema 44.
- 验证：Focused coordinator evidence: cargo lifecycle 38/38, database migration 12/12, server 23/23, validator supervisor Pester 1/1. Legacy b0821bf60f2644beaea1cd165ed9414d is terminal orphaned with live_process_pids=[] and no OS PID 48464; e9224849cae24daaa14a9606981f7a23 reused and released the same 841a target with live_process_pids=[] and no OS PID 23116. Fresh upward evidence: managed zircon_editor job 37b0965d5e7647bb8952c3adb523145d exit 0; current binary state_layer 11 passed/0 failed/1 ignored, selected+ripple 1/1, ignored capture 1/1, screenshot target scan 0.
- 回传：The Layout15 current-source Material state-layer acceptance gate can resume; the historical PID-reuse target pin is cleared without modifying the active Layout15 exact manifest.
