---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: welcome-project-probe-admission-budget
origin_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
fixing_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
origin_child_dir: docs/plans/zircon_editor/editor/10
fixing_child_dir: docs/plans/zircon_editor/editor/14
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/jobs/admission.rs
  - zircon_editor/src/core/jobs/spec.rs
  - zircon_editor/src/core/jobs/limits.rs
  - zircon_editor/src/core/jobs/progress.rs
  - zircon_editor/src/core/jobs/system/mod.rs
  - zircon_editor/src/core/jobs/system/pending.rs
  - zircon_editor/src/core/jobs/system/state.rs
  - zircon_editor/src/core/jobs/tests/admission_scaling_contract.rs
  - zircon_editor/src/core/jobs/tests/background_storm_contract.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/project_probe.rs
tests:
  - EditorJobSystem typed admission accepted/merged/backpressured contract
  - 1/1000/1000000 request entry/bytes/oldest-age/RSS admission storm
  - cargo test -p zircon_editor --lib core::jobs::tests::admission_scaling_contract --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib welcome_project_probe --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked --jobs 1 -- --test-threads=1
---

# Editor14：Welcome project probe 准入预算交接

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 来源执行切片：`failure-2026-07-22-welcome-project-probe-admission-storm`
- 修复责任计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 交接原因：Editor10 拥有草稿语义与 `ProjectAuthority` 调用边界；队列的 entry/bytes/oldest-age 预算、合并结果和调度指标属于 Editor14 的 `EditorJobSystem` 统一准入契约。
- 生命周期键：`welcome-project-probe-admission-budget`

## 失败现象与复现证据

当前 `WelcomeProjectProbeState` 已能在 host 内按草稿 generation 进行防抖、到期提交、同目标复用与取消；但提交使用普通 `EditorJobSpec { category: Index, priority: Background, cancel }`。`EditorJobSpec` 没有 admission key、payload-byte 或 deadline 信息，`EditorJobLimits` 对 `Index` 默认无上限。快速输入因此仍可被其他调用路径在调度层无界排队，且没有 accepted/merged/backpressured、队列 bytes、oldest age 与 RSS 的统一证据。

该结论来自 Editor10 独立源码复审：Welcome 层不得自己建立线程池、私有预算或第二个 job truth；这样会使 asset import、script build 与 welcome probe 的背压语义再次分裂。

## 最低共享层根因

`EditorJobSystem` 只表达类别、优先级、互斥、依赖和取消，没有让业务调用者声明 typed coalescing key、请求 payload 成本与最大可等待年龄的准入协议，也没有返回可审计的 admission outcome。既有 `pending` 索引只能选择已入队任务，不能对无界提交本身实施 entry/byte/age 背压。

## 架构修复验收

- Editor14 在 `EditorJobSpec`/submission 边界发布 typed admission request 与 result：调用者声明语义 key、payload bytes 与最大 age；结果显式为 accepted、merged 或 backpressured，且不丢弃已开始或终态事件。
- 预算由 `EditorJobSystem` 作为唯一 owner 按类别/全局统计 entry、queued bytes、oldest age、merged、cancelled 与 started；上层读取同一份观测，不建立 Welcome 私有队列或计数器事实源。
- 同 key 的未开始请求在 I/O 前合并为 latest generation；正在执行请求保持协作式取消，并由 job 内检查点在下一段 I/O 前停止。Editor10 仅提供 draft generation/key 与 `ProjectAuthority` probe 实现。
- 背压 contract 覆盖 1、1,000、1,000,000 请求以及 32B/4KiB payload、1ms/1s probe；固定 entry/bytes/oldest-age/RSS 与 UI pump p95 门，并保留 missing/linked/invalid/current generation/submit failure/shutdown 语义。
- 先通过 Editor14 focused scheduling/background-storm tests，再向上复跑 Editor10 Welcome probe tests、current-source Cargo 与 F0 产品 trace。

## 禁止临时方案

- Do not add a Welcome-private worker pool, queue, timer thread, budget counter, or separate scheduler truth.
- Do not satisfy bytes/age budgets with a fixed debounce, category concurrency cap, token cancellation alone, or a test-only smaller pool.
- Do not drop terminal/cancel/error events, relax the 1/1k/1M evidence, or use aliases, compatibility shims, silent fallbacks, duplicated truth, or call-site exceptions.

## 修复结果与回传

Open state: `implementation_complete_validation_pending`；Editor14 源码已接入共享 typed admission，但 Editor10 在受管 focused/current-source 与 1/1k/1M、RSS、UI pump p95 产品证据完成前仍不得把 `welcome-project-probe-admission-storm` 回传为 fixed。

- 已完成：Welcome probe 通过 `submit_admitted` 声明 owner-scoped key、draft estimated bytes、250ms maximum pending age 与同 owner mutex group；accepted/merged/backpressured 保持 typed，不建立 Welcome 私有线程、队列或计数器。
- 已完成：同 key pending merge 原位替换 latest task，并同步刷新 worker `JobContext` 与 progress cancellation token；原 ticket 保持唯一，合并后启动再取消仍命中最新 token。
- 已完成：`clear`/`Drop` 使用原 ticket ID 通过 `EditorJobSystem::cancel` 权威终结 pending reservation，entry/bytes/key 立即释放；oldest-age 拒绝终结过期 ticket、保留 latest draft 并轮换 key 以允许下一 tick 重试；running supersede 继续协作式取消，并由 mutex terminal 顺序约束 latest probe。
- 已完成：共享 admission snapshot 同时提供全局与 `JobCategory` 视图，entry、queued bytes、oldest age、merged、cancelled、started 仍来自唯一 `PendingJobQueue` 真源；32B/4KiB 混合类别回归覆盖聚合一致性。
- 已完成：category admission snapshot 改为由 `PendingJobQueue` 在 insert/remove/latest-merge 时维护的类别 entry/bytes/oldest JobId 索引读取，不再在每次类别观测时扫描全部 pending jobs；交错类别、merge 与移除最老项回归锁定同一真源语义。
- 待验证：受管 Windows focused Welcome/admission suites、全 `zircon_editor --lib` current-source，以及 1/1k/1M、1ms/1s、RSS 与 UI pump p95 产品门；在这些证据完成前保持 `status: open`，不执行 failure return。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-08-10 | `implementation_complete_validation_pending` | 完成 Welcome shared admission 接入、latest-wins token/唯一 ticket、clear/drop 与 oldest-age reservation 终结、running mutex terminal 顺序及类别/全局统一观测。 | scoped `rustfmt --check`、`git diff --check`，独立二次审查 `Critical/Important/Minor = 0/0/0`；受管 Cargo 与产品性能证据待 coordinator receipt，未声明 fixed/accepted。 |
| 2026-08-10 | `open / category-snapshot-index-forward-fix` | 将类别 admission 观测从 pending-wide projection 收敛为 `PendingJobQueue` 的按类别 ID/byte 索引；覆盖交错类别、latest merge byte 更新和最老项移除后的 oldest-age。 | 独立复审 `Critical/Important/Minor = 0/0/0`；`pending.rs`、`admission_scaling_contract.rs` 和 `project_probe.rs` scoped rustfmt/diff-check 通过。source snapshot `1600`（manifest `32cea9cf` / `82f4755a` / `5f0f00e3` / `46219b04`）只作同步；完整 Cargo 闭包仍需 Settings owner 的已登记当前源码，failure 保持 `open`。 |
| 2026-08-11 | `open / shared-pending-m4-successor-stacked` | Welcome 继续只消费 Editor14 的唯一 admission truth；共享 `PendingJobQueue` 已随 M4 采用 folder-backed admission/fairness 契约、确定性 fair admission 与 terminal-before-handle mutex-tail 防护，未引入 Welcome 私有队列、计数器或兼容分支。 | M4 immutable snapshot `1639`（23 paths）和最终独立复审 `0/0/0`；该记录仅更新共享层 current-source 归属。Welcome focused/full Cargo、1/1k/1M/RSS/UI pump p95 与上游 fixed return 仍未取得，因此 failure 保持 `open`。 |
| 2026-08-11 | `open / shared-pending-source-closure-forward-fixed` | source-closure 审计确认旧 M4 23-path candidate 未包含 `system/mod.rs` 的全部 M1 leaf owner，已由完整 M1/M3/M4 successor 取代。Welcome 仍只读取唯一 `PendingJobQueue` admission truth，未增加私有队列、计数器、兼容分支或额外调度器。 | [`2026-08-11-m1-m3-m4-current-source-manifest.md`](2026-08-11-m1-m3-m4-current-source-manifest.md) 冻结 39 个输入加自身共 40 paths，覆盖全部 10 个 `system` leaf owner。此前 23-path snapshot 不再可作为 current-source 验证依据；focused/full Cargo、1/1k/1M/RSS/UI pump p95 与上游 fixed return 仍未取得，failure 保持 `open`。 |
