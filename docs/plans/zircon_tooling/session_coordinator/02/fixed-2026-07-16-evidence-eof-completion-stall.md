---
handoff_kind: fixed
status: fixed
created_at: 2026-07-16
resolved_at: 2026-07-16
summary_slug: evidence-eof-completion-stall
origin_plan: docs/plans/zircon_tooling/session_coordinator/02-codex-session-hook-sync.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/02
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/codex_sync/history.py
  - tools/session_coordinator/codex_sync/evidence.py
  - tools/session_coordinator/tests/test_codex_evidence_projection.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_codex_evidence_projection
---


# Coordinator02 → Coordinator01: evidence source reaches EOF but never becomes complete

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/02-codex-session-hook-sync.md`
- 来源执行切片：H3 single-flight evidence reconciliation
- 交接原因：Codex 历史证据的受限回填到达 EOF 后未持久化完成态，根因位于 Coordinator01 的共享历史扫描实现。
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`

## 失败现象与复现证据

一个历史 rollout 的已提交扫描游标已等于当前文件长度，且没有 pending call；
`scan_complete` 仍为 `0`。后续受管同步不会更新该行，导致实时证据覆盖率永久少一项。

当前大来源仍按受限预算前进，说明问题仅限于“已到 EOF、尚未写入完成态”的收束分支，
不得把大文件回填改成无预算全量扫描。

## 最低共享层根因

收集器在来源大小未变且 `scan_offset >= source_size` 时提前返回，
没有让尚未完成的来源执行一次零字节 EOF 读取。因此 EOF 事实未被持久化为
`scan_complete=1`。

## 架构修复验收

- 未完成来源即使 `scan_offset == source_size`，也必须执行一次安全 EOF 收束并持久化完成态。
- 有未换行尾部或正在增长的来源仍保持未完成和重试语义；不得吞掉半条 JSONL 事件。
- 已完成来源继续零扫描返回，不重复解析或重复写入 evidence records。
- 增加 focused 回归：EOF 收束、未换行尾部、已完成来源幂等，以及大来源预算不退化。
- 修复后通过受管 `codex.sessions.reconcile` 使当前覆盖率从 `154/156` 至少推进该 EOF 来源。

## 禁止临时方案

- 禁止直接修改 SQLite 的 `scan_complete`。
- 禁止关闭或删除未完成来源。
- 禁止为完成该来源执行无预算全量历史扫描。
- 禁止在实时证据中写入 rollout 原始路径、命令、提示词或日志正文。

## 修复结果与回传

- 根因：Unfinished EOF cursor with scan_offset equal to source_size returned before scan_complete was persisted.
- 架构修复：Only scan_complete sources skip unchanged scans; unfinished EOF cursors receive a zero-byte completion probe while partial JSONL tails retain retry semantics and bounded budgets.
- 验证：20/20 evidence projection tests, py_compile, diff check, and real evidence progress from 168/170 to 169/170.
- 回传：Coordinator02 H3 may resume bounded evidence synchronization with the EOF source recorded complete.
