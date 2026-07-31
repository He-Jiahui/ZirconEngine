---
handoff_kind: failure
status: open
created_at: 2026-07-23
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
- submit 未获准入、取消或同步前置失败时调用 `AutosaveScheduler::mark_submission_failed`；一旦 job 已提交，成功和任何终态失败均调用 `mark_finished(completed_at)`，不得永久保留 single-flight。
- job 执行只接收从唯一文档/事务 owner 获取的快照 payload，调用 `AutosaveStore::write_snapshot`；不得调用源文件保存、`mark_saved_if_unchanged`、修改 Editor03 save token 或复制源路径。
- `shutdown` 与 Editor14 M3 协议协作：停止新 autosave admission、等待/取消既有 ticket，并保证 mutex tail 与类别许可收敛。Editor17 的最后一次 autosave 必须使用此 adapter，不得绕过 job 关停顺序。
- 合同测试覆盖：同 save mutex 的前台保存与 autosave 不重叠；Background/Misc spec 保留；submit 拒绝可立即重试；成功与写入失败均在下一个 interval 恢复；shutdown 后不接受新 autosave。

## 禁止临时方案

- 禁止在 `autosave.rs`、UI tick、恢复对话或 commandlet 中直接 `thread::spawn`、创建 runtime task pool 或手写 channel 队列。
- 禁止为了 autosave 绕过 save mutex、提升为 Interactive/Normal、或改写 Editor14 的类别配额。
- 禁止将 autosave 成功当作源文件保存，或用 autosave 修改 Editor03 的 dirty/save-token 事实。

## 修复结果与回传

Open state: `autosave foundation 与 job policy 已就绪，但唯一 Editor14 admission/terminal adapter 尚不存在。Editor14 完成 adapter、focused job/recovery 合同、独立复审和受管提交后，按 lifecycle key 回传 fixed；Editor17 再将真实文档序列化与 M2.2 恢复流接到该 adapter。`

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-07-23 18:10 +08:00 | `失败已交接-等待Editor14唯一job适配` | 将 autosave 的实际 admission、同保存互斥与 terminal callback 责任交接给 Editor14；Editor17 不创建第二 job system。 | `AutosaveJobPolicy` 静态规定 `Misc + Background + save mutex`；`AutosaveScheduler` 已区分 submit failure 与 admitted terminal；`core/jobs` 现物未包含 autosave adapter。 |

## 2026-07-30 Performance01 性能验收补充

- `PERF-MVP-592`要求adapter以Editor14现有queue的entry/bytes/age预算形成bounded admission window；不得遍历`AutosavePlan`时先为全部dirty documents构建完整序列化payload。
- 每个ticket获准执行后才从唯一document generation取得snapshot并交给Runtime11共享bounded streaming/atomic persistence；queue中只保留轻量document/generation intent，取消或supersede在payload构建前生效。
- 规模门覆盖dirty docs `1/100/10k`、payload `1KiB/1GiB`、writers `1/16`与stall `0/10ms/2s`；记录pre-admission serialized docs、queue entries/bytes/age、payload owners/RSS、save-mutex wait和terminal latency。必须满足queued payload=0、内存硬有界、同document save/autosave不重叠、shutdown不遗留payload/job。
