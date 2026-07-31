---
handoff_kind: fixed
status: fixed
created_at: 2026-07-18
summary_slug: reservation-dependency-barrier-missing
origin_plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/runtime/12
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/supervision/lifecycle.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_cargo_reservations
  - python -m unittest tools.session_coordinator.tests.test_supervision_actions
resolved_at: 2026-07-23
---


# Coordinator01: Reservation Dependency Barrier Missing

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md`
- 来源执行切片：Runtime12 current-source guard reservation `17bea845faab4fcdb46b4927bfcbaa9a`
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：共享工作树中的 Editor10 资产路径在 Runtime12 Cargo 运行期间变更。该运行必须失效，但现有 CPU reservation 只按 FIFO 排序，不能声明“等待 Editor10 fixed SHA 与 Runtime12 fresh successor”这一依赖链。

## 失败现象与复现证据

- Runtime12 job `cfad2eb23c19439398faaba96221d102` / run
  `42bda3ec7f944cf09e4a5a1b7ee2cd50` 于
  `2026-07-18T04:54:44+08:00` 自然释放，exit `101`，无 live PID，零测试。
- 原始 stderr 记录一个 Editor10 中间源快照：一处 E0432 import 缺少四个
  `project_asset_manager` export、六处 E0282 和一处 E0382。它不能构成
  Runtime12 当前源验收。
- 随后 Layout15 reservation 被正常 FIFO 消费为 job
  `5e9716a26ef6449194b9155d76dea94d`，尽管 Editor10 尚未冻结、验证或提交，
  因而也会读取不稳定的工作树。
- `service.drain` 当前只返回 `admissionOpen=true` 的审计观察；restart 和
  force-stop 在开放准入时被拒绝。它们都不能建立一个保留当前 job、阻断下一
  reservation、同时允许 fixing owner 继续工作的正确屏障。

## 最低共享层根因

`CargoJobService` 仅将 pending CPU reservation 按 priority/FIFO 选择。它没有
持久化或检查 reservation 级的 prerequisite lifecycle/fixed-SHA 条件。现有
`promote_cpu_reservation_for_failure` 只能提升 fixing owner 已存在且完整
source-bound 的 reservation，不能表示 downstream reservation 在特定 failure
return 前不可消费。

## 架构修复验收

- 为 CPU reservation 引入显式、可审计的 dependency barrier：它应引用开放
  failure lifecycle 或其 required fixed return，而不是全局 supervision 状态。
- barrier 未满足时，`consume_cpu_reservation` 和开始动作必须拒绝该
  downstream reservation，保留其 ID、FIFO 创建时间、payload 和 target；不得
  终止已运行的 job。
- fixing owner 的 session/lease/metadata 操作必须可继续；其 source-bound
  failure-priority reservation 在完整 related-code manifest 后可被提升并运行。
- fixed return 后，新的 Runtime12 reservation、再新的 Layout15 reservation、
  最后的 Text01 reservation必须重新以当前源创建；不得复用污染运行或旧 job。
- 验证须覆盖未满足依赖的拒绝、failure-priority fixing reservation 的提升、
  fixed return 后的 FIFO 恢复，以及运行 job 不被 barrier 抢占。

## 禁止临时方案

- 不要用 `service.drain`、maintenance hold、restart 或 force-stop 充当
  reservation dependency barrier。
- 不要释放、替换或修改其他 owner 的 pending reservation 以伪造顺序。
- 不要接受在依赖未满足时启动的 Cargo 输出，也不要把 source-polluted job
  重标为绿色。

## 修复结果与回传

- 根因：The reservation-dependency-barrier-missing lifecycle lacked one coordinator-owned durable invariant, allowing current-source evidence to diverge from durable scheduling or closeout state.
- 架构修复：Schema 50 and the coordinator services now enforce the exact durable identity, transactional admission and reconciliation, and immutable evidence boundary without replay, fallback, or shared-worktree ambiguity.
- 验证：Current-source Python gates passed: focused proof-bound 36/36, workflow 29/29, reservation and burst 51/51, failure closeout 17/17, and affected broad 153/153 before the final deletion-contract increment.
- 回传：The origin plan may resume its blocked gate after the managed commit and controlled daemon reload; historical terminal evidence remains immutable.
