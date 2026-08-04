---
handoff_kind: failure
status: open
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

Open state: `deletion_tombstone_implemented_managed_acceptance_pending`.

- `ValidationTicket.source_manifest` 及 submit 入口现在接受 `str | None`；`None` 以 canonical JSON
  `null` 参与 immutable manifest hash 和 dedupe，非空值仍必须是 SHA-256。
- worker 在 queue claim 和 materialized-copy 两阶段都把删除路径重现判为
  `snapshot_stale`；copy overlay 期间的归属散列变化也统一收敛到该终态，其他材料化错误仍为
  `failed`。
- 真实 `WorkspaceCopyService` 回归证明 baseline archive 中的旧文件会被删除墓碑移除，并拒绝
  attribution 后、overlay 前重新出现的路径。
- focused Python batch 16/16 通过，Python compileall 与 scoped `git diff --check` 通过；首轮独立
  复审的 `C0/I2/M1` 已逐项修复，最终独立复审为 `C0/I0/M0`。
- immutable source manifest `d7e44074e0c10dd09e26c1f11943ce258cdfc5baa977dc74445fbb7f205661e9`
  已提交 managed focused ticket `00dfbdb27a9e4e85935a5f85a7c4f462`；仅有 queued receipt，尚无
  terminal GREEN。Runtime15 的 1,865 路径 manifest（1,858 个删除墓碑、7 个存活验证输入，SHA-256 `856cec84...`）还依赖 Coordinator01 的
  [large-manifest CLI transport](../../zircon_tooling/session_coordinator/01/failure-2026-08-03-validation-ticket-large-manifest-cli-transport.md)；
  在 tombstone 协议被 coordinator wakeup 加载且 stdin transport fixed return 前无法提交完整
  lib-test ticket，因此 failure 保持 `open`，不宣称 fixed/accepted。
- 2026-08-03 二次审查发现 `ValidationTicketWorker::_manifest_drift` 将目录与不存在路径同样
  视为 `None`：删除墓碑在 materialized copy 中被目录重新占用时会错误进入运行阶段。前向修复必须
  令 tombstone 只匹配不存在的路径，并以 focused regression 断言该目录重现与文件重现同样终结为
  `snapshot_stale`；在该修复及其受管验证完成前，本 failure 继续保持 `open`。
- 该目录回归已由 `python -m unittest tools.session_coordinator.tests.test_validation_ticket_deletions.ValidationTicketDeletionTests.test_reappearing_directory_in_materialized_copy_is_snapshot_stale`
  以 exit 0 终结（1/1，1.359 秒）。合并的 23-case Python batch 已输出 `OK`，但外层执行器未在
  60 秒内终结；该非终态批次不作为 accepted evidence，仍须在 source-bound managed gate 中重跑。
