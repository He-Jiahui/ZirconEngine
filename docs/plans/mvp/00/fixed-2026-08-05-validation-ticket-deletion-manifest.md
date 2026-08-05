---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-03
summary_slug: validation-ticket-deletion-manifest
origin_plan: docs/plans/mvp/00-current-source-baseline-recovery.md
fixing_plan: docs/plans/mvp/00-current-source-baseline-recovery.md
origin_child_dir: docs/plans/mvp/00
fixing_child_dir: docs/plans/mvp/00
related_code:
  - tools/session_coordinator/validation_tickets.py
  - tools/session_coordinator/validation_ticket_worker.py
  - tools/session_coordinator/tests/test_validation_tickets.py
  - tools/session_coordinator/tests/test_validation_ticket_deletions.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_validation_tickets tools.session_coordinator.tests.test_validation_ticket_deletions
resolved_at: 2026-08-05
---


# MVP00: validation ticket deletion manifest

## 来源执行者

- 来源计划：`docs/plans/mvp/00-current-source-baseline-recovery.md`
- 来源执行切片：M0.1 Runtime15 current-source receipt hard cut
- 修复责任计划：`docs/plans/mvp/00-current-source-baseline-recovery.md`
- 交接原因：最低共享根因位于 MVP00 validation-ticket 控制面，而不是 Runtime15 业务源码。

## 失败现象与复现证据

Runtime15 已硬删除旧 `zircon_runtime/src/core/framework/error.rs`，但
`ValidationTicketService` 的 `source_manifest` 只接受 SHA-256 字符串。票据无法封存删除墓碑，
worker 因而只能在隔离副本叠加仍存在的文件，不能证明当前硬删除源码。

## 最低共享层根因

工作副本 overlay 已支持归属路径删除，但 validation-ticket 清单和值类型及 drift 判定仍把
“文件缺失”一律视为散列不匹配，控制面和材料化能力的契约没有收敛。

## 架构修复验收

- 票据清单用 JSON `null` 表达删除墓碑，并保留既有路径安全校验和确定性散列。
- 源树与隔离副本都缺失该路径时允许 validation 启动；任一阶段重新出现都判为 `snapshot_stale`。
- Runtime15 可提交包含硬删除路径的受管 Runtime lib-test 票据，不恢复旧文件或兼容导出。

## 禁止临时方案

- 不恢复旧 `core/framework/error.rs`，不增加 alias、shim、fallback 或测试绕过。
- 不从 Runtime15 清单中删掉硬删除路径来伪造 current-source receipt。

## 修复结果与回传

- 根因：Validation-ticket manifests could not represent deletion tombstones, and worker drift checks treated a missing path, directory, and dangling symlink as the same state.
- 架构修复：Represent deletion tombstones as canonical JSON null and require the path to remain truly absent at claim, overlay, and materialized-copy boundaries; any file, directory, or dangling-symlink reappearance becomes snapshot_stale.
- 验证：Local and immutable coordinator copies both passed the full two-module batch 30/30; managed job 5eab4d8afea440aa992dc22940d9782d run 7b4f08b2d5c347e68dfb0ed793d3eedc exited 0 in 25.518 seconds with input hash 97c6f8e510f1b06d4b266c7083604e86a20360333b819df7694200ee5a3bfb46.
- 回传：MVP00 deletion-tombstone validation is accepted; Runtime15 may submit hard-deletion manifests without restoring retired files or dropping tombstones.
