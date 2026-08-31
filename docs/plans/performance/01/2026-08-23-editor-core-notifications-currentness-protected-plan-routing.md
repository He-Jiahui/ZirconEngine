---
source_report:
  - docs/plans/performance/01/2026-08-23-editor-core-notifications-currentness-revalidation.md
protected_ledgers:
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
doc_type: plan-routing
status: proposed_protected_files_untouched
---

# Editor core notifications性能计划路由（2026-08-23）

本记录通知既有owner吸收当前性重验，不修改受保护`review.md`、`pending.md`、Performance01主计划或
编号owner计划，也不新增与`PERF-MVP-596`重复的任务。

| owner | 必须吸收 |
|---|---|
| Performance01 / PERF-MVP-596 | 当前冻结更新为25/25、3,471行、110,931字节、38 tests、SHA256 `2b855a73...e3a8`；继续P0 pending。 |
| Editor17 | 一个immutable typed `ActivityNotificationProjection`，revision tuple包含Decision/Toast/Progress/locale；拥有unread/overflow与`next_toast_expiry`；stable返回`NotModified`。 |
| Editor04 | 用ticket/notification/selection direct index删除`A*D` nested matching；按Decision或locale generation一次构造localized Play choices。 |
| Editor14 | progress authority发布generation/shared rows，删除notification surface每tick两次map materialization；与PERF-MVP-017共享source generation。 |
| EditorUI08 | tick读取compact revision；dispatch与toast publish只mark dirty；每帧最多apply一次，empty generation显式clear。 |
| PERF-MVP-269 | pipe-string codec只保留在临时ABI边界并按changed generation编码一次；typed unread/id/kind不再反向parse。 |

## 账本建议

- `zircon_editor/src/core/notifications/**`继续保持pending：静态currentness完成，但generation/wake cutover、
  current Cargo、规模/锁/分配计数、F4 WPR、RSS/功耗和RenderDoc paint parity均未验收。
- 本轮修复的是陈旧Python source-path contract，不能作为Rust行为或性能完成证据。
- 动态里程碑完成前不得把模块加入`review.md`，不得提交性能完成commit或发送企微完成通知。

