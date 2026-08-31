---
related_code:
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/jobs/system
  - zircon_editor/src/core/jobs/event_journal
  - zircon_editor/src/core/jobs/progress
  - zircon_editor/src/core/asset/dirty/save_job_adapter
  - zircon_editor/src/core/asset/import_flow
  - zircon_editor/src/core/project/scene_load_job.rs
  - zircon_editor/src/core/recovery/autosave_adapter
  - zircon_editor/src/core/recovery/autosave_service.rs
  - zircon_editor/src/core/recovery/autosave_shutdown.rs
  - zircon_editor/src/core/export/stages/compile_host.rs
  - zircon_editor/src/core/play/process_backend/output.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/ui/retained_host/app/job_progress.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/project_probe
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/development_watch.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_runtime/src/core/runtime/tasks
  - zircon_runtime/src/core/runtime/runtime.rs
plan_sources:
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/14/failure-2026-07-17-job-pump-budget-and-pending-scan.md
  - docs/plans/zircon_editor/editor/14/failure-2026-07-22-play-process-output-byte-budget.md
  - docs/plans/zircon_editor/editor/14/failure-2026-07-23-autosave-job-admission-and-save-mutex-adapter.md
  - docs/plans/zircon_editor/editor/14/failure-2026-08-12-thread-ownership-guard-test-scope.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/131-editor-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/192-runtime-task-execution-job-scheduler-task-graph-worker-domain-scope-cancellation-deadline-shutdown-diagnostics-product-adoption-current-working-tree-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/IAssetCompilingManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/AssetCompilingManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/AsyncTaskNotification.h
  - dev/godot/core/object/worker_thread_pool.h
  - dev/godot/core/object/worker_thread_pool.cpp
  - dev/godot/editor/gui/progress_dialog.h
  - dev/godot/tests/core/threads/test_worker_thread_pool.cpp
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/bevy/crates/bevy_app/src/task_pool_plugin.rs
  - dev/Fyrox/fyrox-core/src/task.rs
  - dev/Fyrox/fyrox-impl/src/engine/task.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Batching/CPUDrawInstanceData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Batching/InstanceCullingBatcher.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
doc_type: review-and-refactor-plan
refreshes: docs/plans/optimize/zircon_editor/131-editor-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-current-source-review.md
review_status: current_working_tree_review_complete
implementation_status: pending
tooling_scope: excluded_by_user
source_recheck_required: true
---

# 257 - Editor Background Jobs、Admission、Scheduling、Cancellation、Progress、Shutdown 与产品接入当前工作树审查

## 1. 结论

当前工作树已经有一个真实的 Editor job core：`EditorJobSystem` 统一 pending admission、category quota、priority slots、mutex group、dependency edge、typed ticket、panic containment、progress source、bounded event journal 和 observer resync。`EditorJobBatchAdmissionReservation` 先做原子 reservation 再 materialize，dirty save 也已使用该路径；这些底座不能被后续重构丢掉。

但它仍不是工程级后台作业平台。Editor system 直接包裹 Runtime `JobScheduler`，没有进入 Runtime `TaskGraphScope` 的 owner/census/shutdown 合同；`shutdown(deadline)`只等待 Editor progress map 归零，不能 join/reap Runtime task、child process、reader 或 watcher，也不能禁止 late publish。这个差异仍是 P0。Compile host 与 Play output 仍自行创建阻塞 reader 线程，Autosave shutdown 仍以 `thread::yield_now()` 忙等，Autosave 还和普通 Misc 任务共享类别配额。

本轮沿用 Editor09/131 的稳定编号，当前重判为：P0 `1 Open / 2 Closed`；P1 `42 Open / 4 Partial / 2 Closed`；P2 `8 Open / 2 Closed`。24 个产品验收门全部 `FAIL`，因为本轮只做源码和参考实现审查，未运行 Cargo、真实宿主、故障注入、压力或跨平台关闭矩阵。`Closed` 只表示源码级局部差异已修复，不表示整个 Editor job platform 已达标。

这份报告只覆盖 Runtime-backed Editor background jobs、其 product adoption、线程/进程边界和关闭合同。按用户要求，tooling 不在本轮范围内；tooling 后续迁移 Rust 时仍必须接入同一 job authority，而不是继续扩张本地线程。

## 2. 审查范围与证据

### 2.1 当前工作树物理范围

| 区域 | 物理证据 | 结论 |
|---|---:|---|
| `zircon_editor/src/core/jobs` | 58 个 Rust 文件，约 10,070 行，147 个 test marker | admission、pending、scheduling、state、progress、journal、observer、ticket、shutdown 全部逐文件检查 |
| Editor product consumers | save/import/scene load/autosave/welcome probe/export/Play/plugin/profile/host lifecycle | 逐项检查提交、取消、轮询、Drop、close/reload路径 |
| `zircon_runtime/src/core/runtime/tasks` | `TaskPool`、`JobScheduler`、`EngineTaskGraph`、`TaskGraphScope`、bounded stream lane | 对照实际 owner、scope census、worker join 和 cancellation policy |
| local references | Unreal、Godot、Bevy、Fyrox、Unity Graphics | 至少两类实现与测试合同交叉验证；不是把任何一个引擎整套复制 |

### 2.2 关键源码事实

1. `zircon_editor/src/core/jobs/system/scheduling.rs` 在 promotion 时先增加 Editor running accounting，再把 closure 交给 Runtime scheduler；closure 真正开始时才发 Started event，因此 executor queued 与 running 仍混在一起。
2. `zircon_editor/src/core/jobs/system/mod.rs` 暴露 `with_scheduler*` 构造器和 `join` 透传；多个 system 可以共享一个 Runtime scheduler，却拥有各自 quota、ID、progress、journal 和 shutdown。
3. `zircon_editor/src/core/jobs/system/lifecycle.rs` 的 `shutdown` 只等待 `progress.has_active()`；它没有调用 `CoreRuntime::shutdown_task_graph`，也没有处理外部 process/reader/watcher census。
4. `zircon_editor/src/core/jobs/cancellation_token.rs` 是 `Arc<AtomicBool>`；没有 reason、requester、deadline、acknowledgement、phase 或 parent/child token。
5. `zircon_editor/src/core/jobs/spec.rs` 只有 category/priority/mutex/after/admission key/estimated bytes/max pending age，没有 owner scope、resource vector、attempt、retry 或 execution deadline。
6. `zircon_editor/src/core/jobs/event.rs` 的 Failed 和 Progress payload 以 `String` 表示消息；`JobFailure` 的 typed source 在 ticket 错误路径保留，但发布到 event journal 时被降格为字符串。
7. `zircon_editor/src/core/jobs/progress.rs` 对外的 snapshot 把 active job 固定标成 cancellable；progress 值没有拒绝 `completed > total`、倒退、total 变更或超长 message。
8. `zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs` 是生产中唯一明确的 `jobs().pump_events()` 调用点，默认每次最多 64 个 event 或 1 ms；没有 headless/CLI 替代 driver 或退出时 catch-up drain。
9. `zircon_editor/src/ui/retained_host/app/job_progress.rs` 只把最小 JobId 的 primary snapshot投影到状态栏；完整列表通过另一条只读接口取得，activity 没有基于 focus、recent activity、phase 或 pin 的选择策略。
10. `zircon_editor/src/core/export/stages/compile_host.rs` 为 stdout/stderr 各 spawn 一个 `std::thread`，同步 `child.wait()`，没有 Editor job injection、cancel、deadline 或 Runtime bounded stream ticket。
11. `zircon_editor/src/core/play/process_backend/output.rs` 虽有 64 KiB line、1024 条/4 MiB queue、64 行/256 KiB/2 ms drain 和计数器，reader 仍是私有 `JoinHandle`，取消依赖外部关闭 pipe 或进程退出。
12. `zircon_editor/src/core/recovery/autosave_shutdown.rs` 在最终保存 drain 中重复 `thread::yield_now()`，没有 condvar/host wake；global jobs shutdown 的 condvar 不能抵消这条忙等路径。
13. `zircon_editor/src/core/recovery/autosave_adapter/adapter.rs` 的 Autosave policy 使用 `JobCategory::Misc`，因此 autosave 与普通 misc 工作竞争相同类别入口。
14. welcome probe 使用有界 250 ms feedback、keyed latest-wins 和 cancel token，但每个 UI state 使用唯一 key，无法跨窗口或项目 owner 去重；request replacement 不立即调用 `jobs.cancel`。
15. development watcher 当前已移除私有 worker，host tick 合并一次 timestamp 后提交 Compile ticket；notify 错误仍通过 `eprintln!`，且 managed Cargo、产品 closeout 和 watcher scope receipt 未验收。

### 2.3 当前静态改进，不能误判为完成

- admission key 256 bytes、mutex group 128 bytes 与字符校验已经存在。
- `JobFailure` 保留 `Arc<dyn Error>` 并支持 downcast；Save adapter 可以读取 typed error。
- event journal 有 entries/bytes/oldest-age 上限、progress coalesce、sequence gap、high-water 与 dropped/coalesced metrics。
- labels/messages 使用 `Arc<str>`；primary projection 有 generation fast path；observer backlog 达到上限时折叠为 resync。
- batch reservation 在 task materialization 前提交，completion slot 和 autosave completion poll 有界。

## 3. P0

### E-JOB-P0-01 - Closed: keyed merge 已刷新当前取消 authority

`merge_pending_admission` 成功后，当前代码重新注册同一个 JobId 的 progress entry，替换 latest cancellation endpoint、label、priority 和 presentation。相关 regression contract 验证旧 token 不会取消 latest token。未来增加 owner/generation 时必须保持这个原子 replace 语义。

### E-JOB-P0-02 - Open: Editor shutdown deadline 不是 quiescent barrier

`EditorJobSystem::shutdown` 关闭 admission、取消 pending、请求 active token 后，在 condvar 上等待 progress active map；deadline 到达就返回 unfinished list。Runtime 已有更强的基础：`TaskGraphScope::close_admission`、`wait_until_quiescent`、`TaskCancellationPolicy::{CancelOnDrop,DetachOnDrop,FinishOnShutdown}`，以及 `TaskPool::close_and_join` 的 submission census 和 worker join。但 Editor job 没有通过 scope `submit/schedule_after`，所以 Editor shutdown 无法证明 Runtime closure、timer、bounded stream、process reader、plugin watcher 已停止。

这会产生三个危险窗口：任务在 deadline 后继续写文件或发布状态；`autosave_service` 清空 active/retired adapter 后仍有 late completion；settings/project/plugin/runtime 在检查 unfinished 前被拆掉。正确合同必须是 `close admission -> request cancel -> pump terminal -> scope quiescence -> join/reap all owned workers/processes -> forbid late commit -> release resources`。deadline 失败必须进入 quarantine/fatal closeout，而不能当成普通 close 成功。

### E-JOB-P0-03 - Closed: lifecycle journal 已有硬边界和 gap

当前 journal 已限制 entry、retained bytes 和 oldest age，progress 可合并，lifecycle 超窗以 gap 表达并暴露 high-water/dropped/coalesced/sequence exhaustion。旧的无界 VecDeque/OOM 差异已关闭；生产仍缺明确的 gap consumer、snapshot resync 和 shutdown drain，这些属于 P1-37/38/41。

## 4. P1 finding ledger

状态说明：`Open` 表示当前实现仍缺合同；`Partial` 表示只有局部源码底座；`Closed` 表示本 finding 的局部根因已修复。

### 4.1 Authority、scope、executor ownership

| ID | 状态 | 当前差距 | 重构目标 |
|---|---|---|---|
| E-JOB-P1-01 | Open | Editor spec 没有 app/project/document/plugin owner、generation 或 scope lease，未进入 Runtime scope。 | 复用 `TaskGraphScope`，引入 `JobScopeId + OwnerLease + Generation`，支持按 scope query/cancel/drain/revoke。 |
| E-JOB-P1-02 | Open | `with_scheduler*` 可创建多个相互隔离的 system，共享物理 Runtime worker 但不共享 quota/ID/shutdown。 | 每个 editor instance 一个 `EditorJobAuthority`，普通 domain 只拿 capability-bound client。 |
| E-JOB-P1-03 | Open | `EditorJobSystem` clone 都能调用 global shutdown。 | 唯一 top-level shutdown capability；clone 只能操作自己的 scope。 |
| E-JOB-P1-04 | Open | 只有 active progress、pending snapshot 和 256 项 terminal dependency history，没有权威状态/owner/executor registry。 | bounded registry 保存完整状态转移、owner、resource、terminal reason、结果摘要并可分页。 |
| E-JOB-P1-05 | Open | Autosave service 间接拥有 global jobs shutdown 权力。 | `EditorShutdownCoordinator` 统一阶段；autosave 只关闭自己的 scope。 |
| E-JOB-P1-06 | Open | Compile、Play、watcher 私有 thread/process owner 不在 supervisor graph。 | `WorkerSupervisor` 统一 stop/join/reap/deadline/diagnostic，但允许专用 blocking executor。 |
| E-JOB-P1-07 | Open | `EditorJobSystem::join` 透传 Runtime pool，绕过 admission、progress、cancel、shutdown。 | 受限 `JobExecutionContext::parallel_join`，计入父 job resource claim。 |
| E-JOB-P1-08 | Open | `JobTicket` Drop 隐式 detach，丢票后没有责任/结果政策。 | await/cancel/detach 显式化，detached 仍绑定 scope 和 retention，记录 orphan。 |

### 4.2 Admission、资源和调度

| ID | 状态 | 当前差距 | 重构目标 |
|---|---|---|---|
| E-JOB-P1-09 | Open | pending bytes 是 caller 自报，默认 4 KiB，没有 actual working-set reconciliation。 | typed input/working/output/process/GPU claim，终态记录 estimated/actual 偏差。 |
| E-JOB-P1-10 | Open | promotion 后立即释放 bytes，executor queued/running 期间不占资源预算。 | lease 覆盖 Queued/Running/Publishing，cleanup 完成后释放。 |
| E-JOB-P1-11 | Open | category quota 可远大于 Runtime worker width，没有全局 executor capacity 状态。 | global executor capacity 叠加 category/scope/resource-class quota，快照分离 admitted/queued/running。 |
| E-JOB-P1-12 | Partial | Started event 在 closure 内发送已修复；`mark_started` 和 category running 仍早于实际 executor start。 | Admitted/Ready/ExecutorQueued/Running/Publishing/Terminal 状态机。 |
| E-JOB-P1-13 | Open | priority 只影响 pending selection，Runtime scheduler 不接收 priority/deadline，不能 retract/reschedule。 | priority/aging/deadline 进入 executor，支持合法 queued retract 与 Interactive SLO。 |
| E-JOB-P1-14 | Open | blocking FS、child wait、compile、preview 共用 compute scheduler，可能耗尽 compute worker。 | Compute/BlockingIO/Process/GPU/MainThreadCommit 分离 executor class。 |
| E-JOB-P1-15 | Open | settings 只开放 4 类 1..64，重启生效，没有 per-project/plugin fairness 或热调度。 | hardware/executor/scope validated policy，记录配置来源和安全上下限。 |
| E-JOB-P1-16 | Open | oldest age 是全局值，跨 category/owner 产生错误反压。 | scope/category age budget，返回具体阻塞 owner；global emergency gate 独立。 |
| E-JOB-P1-17 | Open | key 256 bytes、mutex 128 bytes 已校验；label、namespace、owner identity 仍无严格合同。 | 稳定 `JobKey/ResourceLockId/LocalizedJobLabel`，绑定 namespace 和 byte/count budget。 |
| E-JOB-P1-18 | Open | 只有 max pending age，没有 execution/publish deadline、timeout action、retry/backoff。 | queue expiry、execution timeout、publish timeout 分离，retry 声明幂等和 cleanup。 |
| E-JOB-P1-19 | Open | 没有 CPU weight、IO bandwidth、open files、process slots、GPU memory/device queue budget。 | admission broker 原子授予多资源 vector，输出饱和原因，防止多资源死锁。 |
| E-JOB-P1-20 | Open | promotion 固定 64 项，pending scan 固定 48 probes，没有 frame hitch/queue depth feedback。 | time/work budget + backpressure，记录 scan/promotion/queue latency 分位数。 |
| E-JOB-P1-21 | Open | reservation 只有 Drop/release 回收，没有 owner expiry/renewal/stale reclaim。 | 带 expiry 的 reservation lease，自动回收并诊断 abandoned materialization。 |

### 4.3 Dependency、取消、结果与票据

| ID | 状态 | 当前差距 | 重构目标 |
|---|---|---|---|
| E-JOB-P1-22 | Open | `.after(JobId)` 没有失败传播 policy，dependent 可在 prerequisite failure 后照常执行。 | `RequireSuccess/Always/OnFailure/Finally` edge policy。 |
| E-JOB-P1-23 | Open | terminal history 固定 256，晚到依赖变成 ExpiredDependency，缺 build/session graph lifetime。 | scope-owned DAG 和 retention，不依赖全局最近 N 个 terminal。 |
| E-JOB-P1-24 | Open | dependency 只有 JobId，没有 typed result/artifact/fan-in/main-thread commit；生产 `.after` 使用很少。 | typed `JobNode<I,O>`、artifact handle 和 admission-time graph validation。 |
| E-JOB-P1-25 | Open | cancellation token 只有 bool，没有 reason/requester/deadline/phase/generation。 | structured cancellation request + audit state machine，token 仅作 executor 快速缓存。 |
| E-JOB-P1-26 | Open | `cancel(id)` 的 true 只代表写 bool，不代表 observed/ack/side-effect stop；Runtime acknowledgement 未接入 Editor event/ticket/UI。 | Requested/Acknowledged/Quiescent 分离，UI 和 shutdown 只以 Quiescent 为安全门。 |
| E-JOB-P1-27 | Open | progress snapshot 永远 `cancellable=true`，即使已进入不可中断 commit。 | descriptor/phase 动态声明 cancel capability。 |
| E-JOB-P1-28 | Closed | keyed merge 已原子刷新 current cancel/label/priority/presentation。 | 保留原子 replace，扩展 owner/generation 时不得回退。 |
| E-JOB-P1-29 | Open | admitted job 不能 pause/resume/reprioritize/retract，交互请求可能被队列反转。 | handle 支持合法 priority change、queued retract、checkpoint/pause policy。 |
| E-JOB-P1-30 | Open | `JobTicket::wait` 可无限阻塞，没有 UI thread guard；部分 controller 直接 wait。 | nonblocking poll/async await/deadline wait，UI blocking wait 由 lint/runtime assertion 禁止。 |
| E-JOB-P1-31 | Open | ticket 丢失后没有 JobId typed result query，只有 progress。 | bounded typed result store，支持 late observer、artifact receipt 和 restart-safe query。 |
| E-JOB-P1-32 | Open | `JobFailure` 本身保留 typed source；`JobEventKind::Failed` 仍只发布 String。 | versioned failure envelope，stage/code/retryability/cause/attachment/redaction。 |
| E-JOB-P1-33 | Open | partial artifact cleanup、retry、resume/checkpoint/crash recovery 由 adapter 各自实现。 | descriptor 声明 attempt directory、cleanup owner、resume token、幂等策略。 |

### 4.4 Progress、journal、observer 和 telemetry

| ID | 状态 | 当前差距 | 重构目标 |
|---|---|---|---|
| E-JOB-P1-34 | Open | progress 接受 total=0、completed>total、倒退、total 变化和无界 message，没有 phase/subtask。 | validated indeterminate/phase/nested model，显式 reset，message byte cap。 |
| E-JOB-P1-35 | Partial | primary 已按 priority+JobId，不再纯按最小 ID；同优先级没有 focus/recent/pin/phase policy。 | presentation policy 综合 visibility、priority、recent activity、focus、pin。 |
| E-JOB-P1-36 | Open | terminal 时 active progress 立即移除，没有结果、retry、artifact 入口。 | terminal outcome 按 severity/policy 保留，通知消费结构化 result。 |
| E-JOB-P1-37 | Open | production 没有通用 TOPIC_JOB subscriber，journal publication 主要是未消费旁路。 | 唯一 journal/subscription API，定义 consumer、cursor、backpressure。 |
| E-JOB-P1-38 | Open | 只有 retained host tick pump，headless/CLI/退出没有 canonical driver。 | host-neutral driver，GUI/CLI/test 显式注册消费，shutdown 自带 drain。 |
| E-JOB-P1-39 | Closed | journal snapshot 已暴露 depth/bytes/age/high-water/gap/drop/coalesce/sequence metrics。 | 保留并接入 profile/status/performance gate，不把 metrics 当作 consumer。 |
| E-JOB-P1-40 | Open | observer 在 submit/finish producer 线程同步执行，slow observer 会阻塞 producer；只支持一个 observer。 | bounded async fan-out 或 cursor subscription，多 observer slow-consumer policy。 |
| E-JOB-P1-41 | Partial | checked sequence + gap 已存在；没有 timestamp/scope/owner generation/attempt/executor/correlation。 | versioned envelope，consumer 可去重、gap detect、拒绝 stale generation。 |
| E-JOB-P1-42 | Partial | Arc 文本、primary generation 和 `snapshot_for_ids` 已有；通用 snapshot 仍全量 clone，无 paged/delta cursor。 | immutable generation page/delta view，共享 Arc 数据。 |
| E-JOB-P1-43 | Open | 没有 queue/execution/publish/cancel latency、owner terminal rate、deadline miss 的 bounded telemetry。 | JobId stage timing，按 executor/category/scope 输出分位数并关联 trace。 |

### 4.5 产品接入和治理

| ID | 状态 | 当前差距 | 重构目标 |
|---|---|---|---|
| E-JOB-P1-44 | Open | `save_document_toolkit` 路径会在提交 ForegroundDocumentSaveJob 后 `ticket.wait()`，可阻塞 UI。 | 返回 operation handle，UI pump progress/cancel/close gate，最终 commit 仍归 dirty/save authority。 |
| E-JOB-P1-45 | Open | save/import/autosave/preview/export/welcome/profile 各自维护 ticket、generation、cancel 和 stale-result 规则。 | scope-aware adapter/combinator 统一 latest-wins、replacement、terminal publish、teardown。 |
| E-JOB-P1-46 | Open | compile host、Play output 仍直接创建线程；watcher虽已移除 worker，仍未有 supervisor receipt。 | worker type capability + supervisor registry + join/reap contract。 |
| E-JOB-P1-47 | Open | 762 行手写 source scanner 依赖源码形状，不能可靠覆盖 macro/generated/间接 spawn。 | compiler/clippy capability lint + link inventory；scanner 仅补充。 |
| E-JOB-P1-48 | Open | Runtime dynamic scene scope、Editor jobs、Play output、compile capture、watcher 不在同一 lifecycle/quota/progress/status registry。 | adoption matrix 覆盖所有长任务、线程、process、watcher，统一 scope/supervisor/close authority。 |

## 5. P2

| ID | 状态 | 当前差距 | 重构建议 |
|---|---|---|---|
| E-JOB-P2-01 | Open | JobId `saturating_add` 可在极限后重复并覆盖 record。 | checked terminal error 或 scoped 128-bit identity。 |
| E-JOB-P2-02 | Open | reservation ID/terminal order 使用饱和 arithmetic，极限时不再唯一。 | epoch + checked sequence，禁止回绕。 |
| E-JOB-P2-03 | Open | primary generation overflow 使用 `expect` panic，和其他 counter 策略不一致。 | typed overflow terminal，不在产品进程 panic。 |
| E-JOB-P2-04 | Open | 多个独立 system 都从 JobId 1 开始，event/progress 没有 authority ID。 | `JobAuthorityId + JobSequence`。 |
| E-JOB-P2-05 | Open | 封闭 `JobCategory` 迫使 plugin/domain 使用 Misc。 | stable built-ins + validated extension category/resource tags。 |
| E-JOB-P2-06 | Open | fairness slots、retention 256、pump 64/1 ms、observer 1024 是硬编码。 | 记录预算来源、暴露 diagnostics，仅开放 validated config。 |
| E-JOB-P2-07 | Closed | global jobs shutdown 已用 condvar，不再 busy-spin。 | scope barrier 中保留事件驱动等待；Autosave final drain 的 yield 另由 P1-48/P1-46 收敛。 |
| E-JOB-P2-08 | Closed | label/progress message/event 使用 Arc<str> 共享。 | 保留不可变共享文本，message cap/localization 由 P1-34/P2-10 处理。 |
| E-JOB-P2-09 | Open | progress source/snapshot constructor 可公开伪造，不一定来自 authority。 | 收窄构造权，测试使用 fixture builder。 |
| E-JOB-P2-10 | Open | priority/category/debug 文本没有稳定 wire/localization/schema ID。 | stable ID、localized key、schema version。 |

## 6. 产品 adoption 矩阵

| 产品链 | 当前 owner | 现状 | 必须补齐 |
|---|---|---|---|
| Dirty save / Save All | `save_job_adapter` + source write authority | atomic reservation、mutex、bounded completion 是真实底座；部分 UI 路径仍同步 wait | scope lease、nonblocking operation、typed result、close barrier |
| Model import / scene load | Import/Index job + ticket | 任务进入统一 system；部分 caller 丢 ticket 或只保留 domain flight | result store、owner generation、import artifact transaction |
| Autosave / recovery | `EditorAutosaveService` + adapter | batch admission 与 retired project fence；category 为 Misc；shutdown final drain 忙等 | dedicated scope/category、condvar wake、quiescent receipt、deadline quarantine |
| Welcome project probe | retained welcome state | keyed latest-wins、250 ms max feedback、mutex/cancel | cross-window owner key、immediate cancel acknowledgement、result receipt |
| Export compile host | `SystemZirconBuildCommandRunner` | 两个私有 reader thread，同步 child wait，未注入 jobs | Runtime bounded stream lane、Process executor、stop/join/reap/deadline |
| Play process output | `PlayOutputPump` | 本地 queue/byte/line/drain budgets，reader JoinHandle 私有 | 迁移 Runtime bounded stream，process scope、pipe close、terminal census |
| Native plugin watcher | `DevelopmentPluginWatch` | watcher callback 仅记 timestamp，host tick 提交 Compile；错误用 eprintln | scope-bound watch owner、supervisor receipt、structured diagnostic |
| Profile artifact / viewport refresh | host contract and adapters | owner drop 会 cancel，产品结果依赖 ticket polling | scope/generation result store、late publish rejection |
| Host event consumption | retained host tick | 唯一 pump，64 events/1 ms；状态栏只选一个 primary | host-neutral cursor/ACK、multi-consumer resync、exit drain |
| Project/window close | app/editor shutdown paths | 各 owner cancel，但没有统一所有 worker/process close proof | top-level coordinator、ordered barriers、unfinished quarantine |

## 7. 参考引擎对照

| 参考 | 已核验的工程原则 | Zircon 差距 | 可复用边界 |
|---|---|---|---|
| Unreal `IAssetCompilingManager` / `AssetCompilingManager` | manager 注册/注销、dependent type name、topological dependency、对象级 Finish/Cancel/Shutdown、每帧执行时间限额、post-compile event、memory-bound foreground/background pool | 没有 central manager registry、依赖拓扑、对象级 finish、动态 cancelability 或 memory/foreground policy | 借鉴 authority/manager contract；不复制 UObject singleton 和历史兼容层 |
| Unreal `AsyncTaskNotification` | Pending/Success/Failure/Prompt 状态、动态 progress/cancel、headless/unattended、keep-open 和 completion-before-destroy | Zircon 只有 active progress + `cancellable=true`，terminal 立即消失，无 prompt/keep-open/finalization contract | presentation 只消费 job state，不能成为 execution authority |
| Godot `WorkerThreadPool` | normal/low priority 队列、TaskID/GroupID、completion semaphore、collaborative wait/yield、退出 runlevel、group progress | 没有 group task/fence、executor priority、runlevel shutdown 或 worker collaboration | 复用 pool/runlevel/group 语义；保持 Zircon scope owner 隔离 |
| Godot `EditorProgress` / `ProgressDialog` | RAII progress、step/end、cancel button、deferred UI update、background progress host | progress 没有 RAII terminal guarantee、phase/step policy；observer 可同步阻塞 producer | 引入 phase/terminal presentation，不让 UI 直接持有 worker |
| Bevy `TaskPool` / `TaskPoolPlugin` | pool builder 负责 worker lifecycle，IO/async compute/compute 分池，scope 返回前完成，Task future 显式 cancel/detach，Drop close/join，main thread tick | Editor ticket Drop 隐式 detach，所有 job 混合 scheduler，Editor shutdown 不 join | 采用显式 detach/scope/drop join；Editor domain 仍需 owner/admission/result |
| Fyrox `TaskPoolHandler` | UUID task result、plugin/node owner map、completion closure 在下一 game-loop apply，提醒 closure 必须轻量 | ticket 丢失即失去 result，adapter 手工 stale rules，缺统一 owner map | 采用 owner-bound completion 与 next-frame apply；不照搬 Fyrox unbounded channel |
| Unity Graphics `JobHandle` / `RenderGraph` | typed dependency handle、Read/Write/Discard resource access、explicit disposal、validation、ProfilerMarker、debug session | `.after` 只有 JobId，没资源读写 vector、artifact fence、dispose/validation/debug cursor | 借鉴 typed DAG/resource lease/validation；Graphics job 不是 Editor shutdown authority |

## 8. 目标架构

```text
EditorShutdownCoordinator (唯一 global shutdown capability)
        |
        v
EditorJobAuthority -- bounded registry / journal / telemetry
        |
        +-- JobScopeLease(App / Project / Document / Plugin / Tool)
        |       `-- descriptor(identity, generation, resources, priority,
        |                         deadlines, cancel policy, presentation)
        +-- AdmissionBroker
        |       +-- global + executor + scope + category quotas
        |       `-- resource-vector lease + expiring reservation
        +-- TypedJobGraph
        |       +-- success/failure/finally edges
        |       `-- artifact/result/main-thread commit nodes
        +-- WorkerSupervisor
        |       +-- Runtime compute/IO scope
        |       +-- process and bounded stream owner
        |       +-- GPU/device owner
        |       `-- long-lived watcher/thread owner
        `-- PresentationDriver
                +-- progress/cancel/terminal notification
                +-- GUI/CLI/shutdown cursors
                `-- resync/diagnostic projection
```

不可违反的合同：

- Editor job descriptor 必须携带 owner scope、generation、resource claim、priority、queue/execution/publish deadline 和 cancellation policy。
- `cancel requested`、`cancel acknowledged`、`scope quiescent` 是三个不同状态；后者才允许释放 owner 资源。
- Runtime `TaskGraphScope` 是物理 worker owner 的唯一下层边界；Editor 不再另造平行 scheduler 或 scope。
- resource lease 覆盖 executor queue、running、publishing 和 cleanup；估算值与实际峰值都进入 telemetry。
- 一个 bounded journal 可以有多个 cursor，但 event envelope 必须携带 authority/scope/owner/generation/attempt/executor/timestamp/correlation。
- blocking reader、child process、watcher 可以使用专用实现，但必须有 supervisor registration、stop signal、join/reap 和 deadline receipt。

## 9. 分阶段重构路线

### M0 - 封闭 shutdown P0

- 将 Editor authority 接到 `EngineTaskGraph::create_scope`，禁止直接用裸 Runtime scheduler 提交产品 job。
- 先关闭 admission，再按 scope request cancellation，持续 pump terminal，等待 scope census，最后调用 Runtime task graph shutdown 和 worker join。
- 把 process reader、Play output、watcher、timer 的未完成 owner 变成显式 unfinished/quarantine 记录；deadline 失败不得继续普通 closeout。

### M1 - 唯一 authority 与 scope lease

- 收窄 `with_scheduler*`、`Clone` 和 `join`；测试使用 fixture capability。
- 引入 `JobAuthorityId/JobScopeId/OwnerLease/OwnerGeneration`，建立 bounded registry。
- 将 autosave、welcome、preview、profile、document、plugin 各自绑定 scope，按 scope 取消/查询/排空。

### M2 - 资源感知 admission 与多 executor

- 定义 Compute、BlockingIO、Process、GPU、MainThreadCommit resource classes。
- 让 resource lease 跨 Queued/Running/Publishing，加入 actual bytes、process slots、IO 和 GPU budget。
- 将 priority/deadline 传给 Runtime executor，替换固定 promotion-only 选择，建立 Interactive queue p95 gate。

### M3 - Typed DAG、取消 acknowledgement、result

- `.after` 升级为带 `RequireSuccess/Always/OnFailure/Finally` 的 typed DAG，支持 artifact fan-in 和 commit node。
- 复用 Runtime cancellation policy，向 Editor event/ticket/UI 暴露 Requested/Acknowledged/Quiescent。
- 引入 result store、failure envelope、attempt directory、retry/checkpoint/cleanup receipt。

### M4 - Progress、journal、notification、telemetry

- progress 增加 phase/subtask/indeterminate/validated reset 和 message budget。
- 为 journal 增加 cursor/ACK/resync、多 consumer、terminal retention 与 shutdown drain。
- 统一 stage latency、queue depth、cancel latency、deadline miss、resource wait 和 owner terminal rate。

### M5 - 产品迁移与线程治理

- 迁移 save/autosave/import/scene load/welcome/preview/export/profile。
- 将 compile host、Play output 迁移 Runtime bounded stream/process lane；watcher 注册 supervisor。
- Autosave final drain 改为 condvar/host wake；删除 public scheduler escape 和粗糙 thread scanner 作为唯一约束。

### M6 - 规模、故障、跨平台验收

- 1/1K/100K short jobs，1K/10K progress writers，slow/paused consumer，memory pressure 和 priority inversion。
- Windows/Linux/macOS child hang、pipe close、file lock、disk full、plugin unload、project switch、device loss。
- 固定硬件上记录 queue-start/terminal/cancel p95/p99、RSS、power 和 worker join census；没有这些数据不能宣称优于 Unreal。

## 10. 验收门

以下 24 门本轮均为 `FAIL`，因为没有动态执行；它们是后续实现不可跳过的证据要求。

1. keyed merge 在 pending/promotion/started/shutdown 竞态下只取消 latest generation。
2. cancel 状态可分别观察 Requested、Acknowledged、Quiescent。
3. paused consumer 下 journal entry/bytes/oldest-age 仍受硬上限约束。
4. gap 后 consumer 能通过 authority snapshot resync，且终态不重复。
5. app/project/document/plugin/tool 都有 scope lease，revoke 后 stale completion 不得写当前状态。
6. 非 top-level client 无法关闭 global authority。
7. shutdown 顺序包含 admission close、cancel、pump、scope wait、join/reap、late commit rejection、resource release。
8. non-cooperative work 到 deadline 时进入 quarantine/fatal，不继续普通 close。
9. 所有受管 thread、watcher、child process 都 join/reap，OS handle 和临时目录无泄漏。
10. reservation expiry 能在 owner 丢失时自动回收并记录诊断。
11. global/executor/scope/category/memory/IO/process/GPU 任一饱和都返回 typed reason。
12. resource claim 直到 publish/cleanup 完成才释放。
13. background saturation 下 Interactive queue-start p95/p99 达到固定硬件门槛。
14. blocking IO/child wait 不耗尽 compute pool，各 executor 有 starvation/backpressure 测试。
15. dependency edge 的 success/failure/finally 传播在 fan-in 图中可验证。
16. typed artifact/result 在 main-thread commit 前拒绝 stale generation。
17. progress 拒绝非法范围、超长 message，phase aggregate 可重现。
18. non-cancelable commit 阶段不显示虚假 cancel。
19. terminal failure 保留 structured stage/code/retryability/cause，UI/CLI 不解析任意字符串。
20. ticket 必须显式 await/cancel/detach；UI blocking wait 有 lint/runtime failure。
21. save/autosave/import/preview/export/welcome/viewport/profile/Play/plugin 全部出现在 registry/adoption matrix。
22. GUI、headless、CLI 和 shutdown driver 都能消费 event/result，不依赖 retained tick。
23. 10 万短任务、慢 observer、暂停 consumer、长时间 progress 压力满足内存、tick、sequence、terminal 基线。
24. 跨平台 fault matrix 覆盖 worker panic、child hang、file lock、disk full、plugin unload、project switch、device loss，并保证无 late product commit。

## 11. 交叉边界与禁止捷径

- Runtime task pool、`TaskGraphScope`、JobHandle、bounded stream 和 worker census 由 Runtime192/Runtime canonical plans 拥有；Editor 只定义上层 product owner、admission、presentation 和 adoption。
- dirty/save、import/asset、plugin、Play、notification 各专项继续拥有业务语义；本报告只要求它们进入同一 lifecycle authority。
- 不得把所有 blocking IO/process/watcher 强塞到 compute pool来满足 scanner；必须保留正确 executor class并注册 supervisor。
- 不得把 bool cancellation、progress disappearance 或 event publication 当作 quiescence。
- 不得以扩大 64/1024/256 常量掩盖缺少 consumer、result store、scope 或 shutdown barrier。
- 不得以绿色局部单测或 source scanner 结论宣称工程级性能；所有性能、故障、关闭和跨平台门必须有可复算证据。

## 12. 当前审查状态

- `review_status: current_working_tree_review_complete`：Editor job core、Runtime scope/worker contract、产品提交/取消/关闭链和五类参考实现已经逐文件复核。
- `implementation_status: pending`：本轮没有修改 Editor/Runtime/plugin/Cargo/ABI/ZUI/测试实现，只准备 optimize review 文档。
- `tooling_scope: excluded_by_user`：tooling 不在本轮实现或验收范围，后续迁移仍必须遵守 P1-48 adoption matrix。
- `source_recheck_required: true`：实现前必须重新读取当前 dirty worktree、失败交接、Cargo 可达性和线程/进程 owner 变化。
- 动态验证缺口：本轮未运行 Cargo、真实 Editor host、GUI/CLI/headless pump、process/pipe fault、GPU、scale/soak、sanitizer 或跨平台 benchmark。
