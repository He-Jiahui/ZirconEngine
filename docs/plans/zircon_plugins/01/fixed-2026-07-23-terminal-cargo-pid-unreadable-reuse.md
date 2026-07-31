---
handoff_kind: fixed
status: fixed
created_at: 2026-07-22
summary_slug: terminal-cargo-pid-unreadable-reuse
origin_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cargo_jobs.py
tests:
  - terminal Cargo job with recorded tree exit ignores unreadable reused PID
resolved_at: 2026-07-23
---


# Coordinator01：终态 Cargo job 的不可读复用 PID 重新占用 target

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 来源执行切片：Plugins01 native host context / registration replay Windows 受管验证
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：唯一兼容 Cargo pool 的进程身份与终态退出证据由 Session Coordinator 统一拥有，插件计划不得绕过协调器另建 target。

## 失败现象与复现证据

2026-07-22，`cargo acquire check` 与 burst-eligible reservation consumption 均在 Cargo 启动前返回
`cargo_process_tree_alive`：历史 job `b5c9181f1ff847c5bc29fc41f6a34e0a` 已于 2026-07-15
finish/release，且数据库记录 `process_tree_exited_at=2026-07-18T17:16:44+08:00`；其旧 root PID 4340
现为 2026-07-19 创建的 `svchost.exe`。Windows creation-time 探针对该系统进程返回不可读，当前逻辑因而继续按旧
Cargo 根扫描并把 PID 4340 重新写成 live blocker，永久占用兼容 pool。

## 最低共享层根因

`CargoJobService._live_process_pids` 只在 creation-time 明确不相等时识别 PID 复用；当终态 job 已有可信的
`process_tree_exited_at`，但复用后的系统进程 creation-time 因权限不可读时，代码仍把“身份未知”当成“原 Cargo
身份仍存活”。这与已回传的 PID identity guard 契约不一致。

## 架构修复验收

- 对 `released/succeeded/failed/orphaned` job，若已记录一次空进程树退出且当前 root creation-time 不可读，不得重新认领该 PID。
- creation-time 可读且与原值相同的晚到 Cargo descendant 仍必须阻止 target 复用。
- creation-time 可读且不同仍按 PID reuse 处理；运行中或未记录 clean tree exit 的 job 保持保守阻断。
- 既有 Cargo job 单元测试与新增不可读复用 PID 回归全部通过。

## 禁止临时方案

- 不得删除数据库 job、手改 PID、结束无关系统进程或创建第二个兼容 warm pool。
- 不得对所有 creation-time 读取失败一律视为 dead；运行中和未证明退出的 job 仍须保守。

## 修复结果与回传

- 根因：The terminal-cargo-pid-unreadable-reuse lifecycle lacked one coordinator-owned durable invariant, allowing current-source evidence to diverge from durable scheduling or closeout state.
- 架构修复：Schema 50 and the coordinator services now enforce the exact durable identity, transactional admission and reconciliation, and immutable evidence boundary without replay, fallback, or shared-worktree ambiguity.
- 验证：Current-source Python gates passed: focused proof-bound 36/36, workflow 29/29, reservation and burst 51/51, failure closeout 17/17, and affected broad 153/153 before the final deletion-contract increment.
- 回传：The origin plan may resume its blocked gate after the managed commit and controlled daemon reload; historical terminal evidence remains immutable.
