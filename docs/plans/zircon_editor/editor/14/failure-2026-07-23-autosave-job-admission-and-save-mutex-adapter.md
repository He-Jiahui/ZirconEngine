---
handoff_kind: failure
status: open
created_at: 2026-07-23
updated_at: 2026-08-13
summary_slug: autosave-job-admission-and-save-mutex-adapter
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/14
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/jobs/mod.rs
  - zircon_editor/src/core/jobs/system/mod.rs
  - zircon_editor/src/core/recovery/autosave.rs
  - zircon_editor/src/core/recovery/autosave_adapter.rs
  - zircon_editor/src/core/recovery/tests/autosave_adapter.rs
tests:
  - cargo test -p zircon_editor --lib --locked core::jobs::tests -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked core::recovery::tests -- --test-threads=1
---

# Editor 14：缺少 autosave 的唯一 job admission 与保存互斥适配层

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行者：`editor17-recovery-autosave-core-r1-20260723`
- 来源执行切片：Editor17 M2.1 自动保存调度、快照存储与 dirty 投影基础层
- 修复责任计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 交接原因：Editor17 只提供不可变 `AutosavePlan` 与 `AutosaveJobPolicy`；按计划，实际后台执行和同保存互斥必须由 Editor14 的唯一 `EditorJobSystem` 拥有。

## 失败现象与复现证据

`AutosaveScheduler::plan` 在首次到期脏文档时进入 single-flight，`mark_submission_failed` 与 `mark_finished` 分别对应“job 未获准入”和“已获准入 job 的成功或失败终态”。当前没有生产 adapter 将 `AutosavePlan` 的每个文档提交给 `EditorJobSystem`，没有将真实 save mutex group 传给 `AutosaveJobPolicy::build_job_spec`，也没有把 submit/terminal 结果回写 scheduler。因此基础层不会自行运行，且若由 UI 或 recovery 层直接 spawn/写入，将绕过 Editor14 的优先级、类别限额、取消、生命周期和关停协议。

现有 `EditorJobSpec` 已支持 `JobCategory::Misc`、`JobPriority::Background` 和 `MutexGroup`；缺失的是将这些现有事实连接到 autosave 的单一适配器，不是新增线程池、队列或保存 owner。

## 最低共享层根因

Editor14 的 job 门面是通用的，尚未为 Editor17 autosave 定义“计划文档 -> snapshot serialization -> ticket terminal -> scheduler release”的 bridge。恢复层不能安全猜测 `EditorJobSystem` 生命周期、保存互斥命名、shutdown 时序或 terminal delivery，因此不能在 `autosave.rs` 自行提交 runtime task。

## 架构修复验收

- 新建的 adapter 必须通过 `EditorContext` 中唯一 `EditorJobSystem` 提交；不得在 recovery/UI/commandlet 建立 worker、thread 或第二个 job system。
- 每个 autosave 文档使用 `AutosaveJobPolicy::build_job_spec`，即 `JobCategory::Misc`、`JobPriority::Background`，并传入该文档前台保存使用的同一个 `MutexGroup`。
- submit 未获准入或同步前置失败时调用 `AutosaveScheduler::mark_submission_failed`；一旦 job 已提交，成功、取消和任何其他终态失败均在整批 terminal 后调用 `mark_finished(completed_at)`，不得永久保留 single-flight。
- job 执行只接收从唯一文档/事务 owner 获取的快照 payload，调用 `AutosaveStore::write_snapshot`；不得调用源文件保存、`mark_saved_if_unchanged`、修改 Editor03 save token 或复制源路径。
- `shutdown` 与 Editor14 M3 协议协作：停止新 autosave admission、等待/取消既有 ticket，并保证 mutex tail 与类别许可收敛。Editor17 的最后一次 autosave 必须使用此 adapter，不得绕过 job 关停顺序。
- 合同测试覆盖：同 save mutex 的前台保存与 autosave 不重叠；Background/Misc spec 保留；submit 拒绝可立即重试；成功与写入失败均在下一个 interval 恢复；shutdown 后不接受新 autosave。

## 禁止临时方案

- 禁止在 `autosave.rs`、UI tick、恢复对话或 commandlet 中直接 `thread::spawn`、创建 runtime task pool 或手写 channel 队列。
- 禁止为了 autosave 绕过 save mutex、提升为 Interactive/Normal、或改写 Editor14 的类别配额。
- 禁止将 autosave 成功当作源文件保存，或用 autosave 修改 Editor03 的 dirty/save-token 事实。

## 修复结果与回传

Open state: `bounded_admission_and_completion_budget_source_repair_complete_pending_managed_validation`。`core/recovery/autosave_adapter.rs` 已通过唯一 `EditorJobSystem::reserve_batch_admission` 预留选中窗口，再以 `reservation.commit` 接收 immutable autosave plan；worker 真正开始后才向 document authority 捕获 snapshot，随后仅写入 autosave snapshot。atomic admission 拒绝会释放 scheduler single-flight；完成回流按显式 ticket budget 轮转并累计批次终态，全部 ticket terminal 后恰好一次推进下个 interval；shutdown 停止新 admission 并请求已属 ticket 协作取消。

2026-08-08 current-source performance review 证明旧 adapter 尚不能 return：`AutosaveScheduler::plan` 先为全部 dirty document 建立 `AutosavePlan::documents`，`AutosaveJobAdapter::schedule` 随后为整批请求构建 `BTreeMap`、`Vec<(EditorJobSpec, AutosaveWriteJob)>` 和 `submit_batch` 的 sender/task 容器，之后才由 `ensure_batch_pending_admissible` 拒绝 over-budget admission。故 snapshot bytes 虽为零，但 `1/100/10k` dirty 输入能够在 queue budget 决定前使 transient task/intention 数量随全量 dirty set 增长；这不满足 PERF-MVP-592 的 entry/bytes/age bounded admission window。

2026-08-10 forward repair 已将预算前移到任何 request/job/channel 物化之前：`EditorJobSystem::pending_admission_window` 在同一 pending queue owner 内复用 entry、estimated bytes 与 oldest age 语义；`AutosaveScheduler::plan_window` 只保留受 entry 预算限制的排序窗口，并通过 adapter cursor 跨 interval 轮转，避免窗口外 dirty document 饥饿；adapter 先调用纯 estimated-bytes 投影选定 byte window，再只为选中文档调用 request source 和 `submit_batch`。首个 intent 自身超过剩余 byte budget 时直接返回与 queue 相同的 `AdmissionByteLimitExceeded`，不会构造 request。snapshot capture 仍只发生在 admitted worker 内。

2026-08-10 independent review forward fixes further closed combined byte-window fairness, exact caller-supplied snapshot sequence semantics, preflight admission before document identity/source resolution, and restart sequence allocation across both persisted snapshots and crash-left `.{sequence}.autosave-reservation` markers. `write_snapshot` remains an exact-sequence API; only the admitted worker calls `next_sequence`, so a rolled-back wall clock cannot repeatedly collide with a stale marker.

二次审查已发现并前向修复 byte-full 产品 preflight 回归：`pending_admission_window` 可正确报告零剩余 bytes，adapter 必须将该状态视为不进入 dirty document projection，不能把通用 `Ok(window)` 直接解释为可排程。真实 document serialization 和 M2.2 startup recovery admission 仍分别受既有 document/Editor16 owner 约束；focused/broad managed validation、PERF-MVP-592 规模矩阵与独立复审完成前，本 failure 保持 open，不得 return fixed。本轮未运行 Cargo、未执行 coordinator `failure return`，也未把静态证据写成产品验收。

2026-08-11 forward repair closes the remaining materialization race: the advisory window now limits only
the pure document/byte selection. Before calling `request_for` or building `AutosaveWriteJob`, the adapter
reserves the exact selected `Misc + Background` entry/byte set through `EditorJobSystem::reserve_batch_admission`.
The reservation commit verifies the final specs; request or commit failure calls `mark_submission_failed` and
releases the claim through RAII. The focused race regression fills the last shared admission entry inside the
estimate hook and proves the request source is never called. This is source/static evidence only; managed Cargo,
the PERF-MVP-592 matrix, and Editor17 upward acceptance remain required.

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-07-23 18:10 +08:00 | `失败已交接-等待Editor14唯一job适配` | 将 autosave 的实际 admission、同保存互斥与 terminal callback 责任交接给 Editor14；Editor17 不创建第二 job system。 | `AutosaveJobPolicy` 静态规定 `Misc + Background + save mutex`；`AutosaveScheduler` 已区分 submit failure 与 admitted terminal；`core/jobs` 现物未包含 autosave adapter。 |
| 2026-08-08 | `partial_implementation_static_green / bounded_admission_repair_required` | 当前源已建立 `AutosaveJobAdapter`，以 `submit_batch` 原子准入 `AutosavePlan` 的 document requests；`AutosaveWriteJob` 只在 admitted worker 中捕获 snapshot，复用 policy 的 `Misc + Background + save mutex` spec，且不触碰 authoritative source save/dirty facts。 | `core/recovery/tests/autosave_adapter.rs` 已有 admitted-mutex 前零 capture、atomic rejection release、write failure advance 与 shutdown rejection 三条 focused regression；但静态追踪证明 all-dirty plan/request/task 容器会在 queue budget 之前物化，需先加入 1/100/10k bounded-window 回归和 production repair。Editor17 recovery ownership contract 静态守卫 3/3 通过；未运行 Cargo，未写 fixed return。 |
| 2026-08-10 | `bounded_admission_source_repair_complete_pending_managed_validation_and_independent_review` | queue owner 新增 entry/bytes/age admission window；scheduler 以 bounded ordered sets 和 round-robin cursor 选择计划；adapter 以 estimate/request 两阶段仅物化选中 intent。 | 新增 10k dirty / 3-entry 两轮公平窗口、2-byte request materialization、zero-age backlog 前置拒绝回归；`rustfmt --edition 2021 --check` 与静态 wiring/contracts 通过，未运行 Cargo。 |
| 2026-08-10 | `implementation_complete_validation_pending` | 独立审查的四项 Important 已逐项前向修复：组合 byte-window 不再让永久超大文档隐藏临时跳过项；精确 sequence 写入合同恢复；产品层在解析 dirty document identity/path 前执行 admission preflight；重启 floor 同时跨过 snapshot 与 reservation marker。 | 新增组合公平、重启单调序号和 stale marker floor 回归；最后一轮 scoped `git diff --check` 通过。二次审查与受管 Windows Cargo 仍待 coordinator acceptance，未提前 return fixed。 |
| 2026-08-10 | `open / second-review-forward-fix` | 二次审查发现 byte-full 时通用 admission window 的 `Ok` 会让 UI 继续收集全量 dirty identity；adapter 现将零 remaining entries 或 bytes 明确返回 `false`，使产品调用方在任意 document projection 前返回。 | 新增 `autosave_preflight_returns_false_when_pending_byte_capacity_is_exhausted` 回归；待受管 Windows Cargo 与独立复审确认，不宣称验收通过。 |
| 2026-08-10 | `open / second-review-clean` | 对 byte-full 前向修复完成独立复审：`Ok(true)` 仅在 entry 与 byte capacity 同时存在时成立，UI 仅于 `Ok(true)` 后访问 dirty document 和 identity projection。 | 独立复审 `Critical/Important/Minor = 0/0/0`；`rustfmt --edition 2021 --check` 与 scoped `git diff --check` 通过。受管 Cargo 票据 `f13a6c60f5824a02a89d0e8e6a8c5b43` 尚未形成 terminal evidence，failure 保持 open。 |
| 2026-08-11 | `implementation_complete_static_validation_pending` | 将 Autosave 从 advisory-window 后 `submit_batch` 改为 selected entry/bytes 的 atomic reservation：request source 与 job payload 仅在预留成功后物化，request/commit 失败均释放 claim 并清理 scheduler single-flight。 | 新增 `autosave_adapter_reserves_capacity_before_materializing_requests` 竞争回归；snapshot `1609`，recovery static contract `3/3`、scoped `rustfmt --check`/`git diff --check` 通过，独立复审 `Critical/Important/Minor = 0/0/0`。受管 Cargo、PERF-MVP-592 与 Editor17 上行验收仍待，failure 保持 open。 |
| 2026-08-13 | `completion-pump-budget-forward-fixed / static-revalidation-pending` | 当前源码审计发现 admission window 已有界，但 `pump_completed` 每 tick `take` 全部 tickets 并重建 pending `Vec`，大批次仍让主线程工作量随总票数增长。本轮硬切为 `VecDeque` 轮转与显式 ticket budget，默认 64；跨 tick 累积 succeeded/failed，队列清空后一次推进 scheduler 并归零批次累计。 | 新增 100-ticket / 8-inspection 行为回归，要求首次 pump 只检查 8 张且最终仍返回 100 个成功结果；未复制线程、队列 owner 或改变 snapshot/save mutex 合同。等待静态复验、独立二审和受管产品 gate，failure 保持 `open`。 |
| 2026-08-13 | `second-review-minors-forward-fixed / final-re-review-pending` | 独立二审 `Critical/Important/Minor = 0/0/2`；本轮补齐零预算、blocked-head/ready-tail 轮转、跨 tick 成功/失败累计、terminal 后全零复位与下一 interval 重排程回归，并导出默认预算、同步 recovery 文档累计消费语义。 | 同步修正顶部 Open state 的旧 `submit_batch` 表述和 `updated_at`。等待静态复验与最终独立 re-review；受管 Cargo、PERF-MVP-592 与 Editor17 上行验收仍待，failure 保持 `open`。 |
| 2026-08-13 | `final-review-test-race-forward-fixed / re-review-pending` | 最终复审 `Critical/Important/Minor = 0/1/1` 指出 snapshot capture 计数早于 ticket channel terminal，直接断言下一次 budget-1 pump 会形成时序竞态。本轮改为有 deadline 的 budget-1 状态循环，每次断言 inspection 不超过 1，再等待目标累计状态；同时明确已提交后的取消属于 terminal `mark_finished`，不属于 `mark_submission_failed`。 | 中间混合终态只断言一个 terminal/一个 pending，不假定 success/failure 顺序；生产 VecDeque/累计状态机未发现行为缺陷。等待修复后静态复验与最终 clean re-review；受管 Cargo、PERF-MVP-592 与 Editor17 上行验收仍待，failure 保持 `open`。 |
| 2026-08-13 | `implementation-complete / final-second-review-clean / managed-acceptance-pending` | 最终独立复审 `Critical/Important/Minor = 0/0/0`：budget-1 状态循环不会跳过目标中间态，累计计数、terminal exactly-once、复位和下一 interval 重排程合同均闭合。 | 五路径 scoped `rustfmt --check`、结构合同与 `git diff --check` 通过；未直接运行 Cargo。受管 Cargo、PERF-MVP-592 和 Editor17 上行验收仍只延迟 accepted closeout，failure 保持 `open`，未提前 return fixed。 |

## 2026-07-30 Performance01 性能验收补充

- `PERF-MVP-592`要求adapter以Editor14现有queue的entry/bytes/age预算形成bounded admission window；不得遍历`AutosavePlan`时先为全部dirty documents构建完整序列化payload。
- 每个ticket获准执行后才从唯一document generation取得snapshot并交给Runtime11共享bounded streaming/atomic persistence；queue中只保留轻量document/generation intent，取消或supersede在payload构建前生效。
- 规模门覆盖dirty docs `1/100/10k`、payload `1KiB/1GiB`、writers `1/16`与stall `0/10ms/2s`；记录pre-admission serialized docs、queue entries/bytes/age、payload owners/RSS、save-mutex wait和terminal latency。必须满足queued payload=0、内存硬有界、同document save/autosave不重叠、shutdown不遗留payload/job。
