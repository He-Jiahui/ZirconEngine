---
record_kind: implementation_slice
status: accepted
created_at: 2026-07-19
plan_source: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
milestone: M6.8
related_code:
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/workspace_copy_terminal.py
  - tools/session_coordinator/tests/test_workspace_copy.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_workspace_copy
  - python -m unittest tools.session_coordinator.tests.test_server.ServerTests.test_foreground_mutation_is_not_blocked_by_validation_copy_materialize tools.session_coordinator.tests.test_workflow_commit.WorkflowCommitTests.test_validation_copy_mutation_after_binding_is_rejected
  - python -m py_compile tools/session_coordinator/workspace_copy.py tools/session_coordinator/workspace_copy_terminal.py tools/session_coordinator/tests/test_workspace_copy.py
  - validation-copy run with a 24,597-path immutable workspace and the Runtime11 tasks command
---

# Coordinator01：Validation Copy 终态证据原子化

Plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
Milestone: M6.8
Status: accepted
Files: ["docs/plans/zircon_runtime/frameworks/01/fixed-2026-07-19-validation-copy-cargo-run-loses-terminal-evidence.md", "docs/plans/zircon_tooling/session_coordinator/01/2026-07-19-validation-copy-cargo-run-loses-terminal-evidence-return.md", "docs/plans/zircon_tooling/session_coordinator/01/2026-07-19-validation-copy-terminal-evidence-fix.md", "tools/session_coordinator/tests/test_workspace_copy.py", "tools/session_coordinator/workspace_copy.py", "tools/session_coordinator/workspace_copy_terminal.py"]

## 验收结论

`validation_copy.run` 现在严格按“子进程终态采集 -> durable run evidence -> workflow completion -> copy terminal cleanup”单向推进。可空 stdout/stderr、非零 exit、hook failure、evidence INSERT failure、restart recovery 与并发 cleanup 均不会再删除唯一诊断载体或留下无 run row 的 removed copy。

本切片关闭 `validation-copy-cargo-run-loses-terminal-evidence` 的 Coordinator01 责任。生产 replay 中原 Runtime11 命令返回 exit 101 与规范化空流，属于可审计业务 RED；它证明终态证据链已修复，但不替代 Frameworks01/Runtime11 后续业务修复与 GREEN 验收。

## Scope delivered

- `workspace_copy_terminal.py` 独立拥有 stream 规范化、run row 持久化、completion hook、typed hook failure 与 terminal finalize。
- sync/async run 在 DB 暴露 `running` 前先取得本地 reservation；PID 注册、finalize、recovery snapshot 都使用状态行数检查和同一并发边界。
- completion hook 成功前 copy 保持不可清理；hook/证据失败保留 source root，restart 只恢复非本地 active run，真实 running process 与 completion-pending copy 不被周期维护误删。
- 成功链只在 durable evidence 和 hook 都完成后清理；API 返回的 exit/stdout/stderr 与 SQLite run row 一致。

## Fresh testing evidence

- 冻结 snapshot `585` exact4 哈希：`workspace_copy.py=2ee4490c...`、`workspace_copy_terminal.py=313d6ea6...`、`test_workspace_copy.py=213759a8...`、failure record `0d04ed3d...`。
- 当前磁盘 fresh `test_workspace_copy` 32/32 GREEN，耗时 269.713 秒；Server/Workflow 交叉门 2/2、`py_compile`、scoped diff-check GREEN。
- 独立复审最终 `Critical/Important/Minor = 0/0/0`；一次空路径全仓 API 试验因 index、deletion attribution 与 I/O admission 三项 Important 被完整撤销，不进入产出。
- 受控 rollover action `beaad9df...` 加载 successor `b07627e81a7f43eb8172f236290fb1ee`，schema 49。
- production copy `93b7be8bfa664df7b3d4f945d336748e` 固定 HEAD `d9bae9df7ba1d22cb2e83d37fc3b6ab22671bc20`，manifest 24,597，exact4 overlay 校验一致。
- CPU reservation `0f126e47ee4e48e492b0b134f5672b39` 绑定 failure lifecycle、rank 0、source manifest fingerprint `6a81aad6...`；outer job `3677f65b...` / run `8d6758b7...` 通过 `cargo run-reserved` 执行 immutable wrapper，终态 exit 0、released、PID 空。
- inner validation run `b56ab61ce7234cadb36157f6cd54c753` 执行原 Runtime11 tasks 命令，durable exit 101、stdout/stderr 各 0 字符；copy 之后转 `removed`，job root 已删除。

## Review

- 当前 exact 代码与记录经独立复审，`Critical/Important/Minor = 0/0/0`。
- replay 的 exit 101 只证明非零终态与空 stream durable 入账、证据后 cleanup；不宣称 Runtime11 GREEN。
- canonical Frameworks01 `fixed-*` 与 fixing-side receipt 同属本次 failure return 证据，用于 immutable manifest 的 node-scoped failure selector；不纳入 foreign `server.py`、`workflows/milestones.py` 或其他工作树改动。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与边界 |
| --- | --- | --- | --- |
| 2026-07-19 08:10 +08:00 | `accepted / M6.8 support slice` | 完成 validation-copy 终态证据原子化、并发恢复防护、typed hook failure、successor reload 与 24,597 路径生产 replay。 | Local 32/32、cross 2/2、review 0/0/0；production inner exit 101 已持久化并在其后清理。Runtime11 业务 GREEN 仍由 Frameworks01 fresh gate 负责。 |
