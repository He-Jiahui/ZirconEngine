---
related_code:
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/editor_message/message/payload.rs
  - zircon_editor/src/core/editor_message/topics.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/completion.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/job.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/poll.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/view_model.rs
  - zircon_editor/src/ui/host/export_cargo_process.rs
  - zircon_editor/src/ui/host/export_process_support/output_capture.rs
  - zircon_editor/src/ui/host/export_process_support/process_tree.rs
  - zircon_editor/src/ui/host/export_process_support/child_guard.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/state.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/start.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/worker.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/updates.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/session_state.rs
  - zircon_editor/src/ui/retained_host/app/job_progress.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state/construction/assembly.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/ui/retained_host/viewport/viewport_state.rs
  - zircon_editor/src/ui/retained_host/viewport/bind_jobs.rs
  - zircon_editor/src/ui/retained_host/viewport/render_framework_resolve_job.rs
  - zircon_editor/src/ui/retained_host/viewport/viewport_state_drop.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
implementation_files:
  - zircon_editor/src/core/jobs/mod.rs
  - zircon_editor/src/core/jobs/progress.rs
  - zircon_editor/src/core/jobs/event_sink.rs
  - zircon_editor/src/core/jobs/system/mod.rs
  - zircon_editor/src/core/jobs/test_support.rs
  - zircon_editor/src/core/jobs/system/pending.rs
  - zircon_editor/src/core/jobs/system/state.rs
  - zircon_editor/src/core/jobs/shutdown.rs
  - zircon_editor/src/core/jobs/spec.rs
  - zircon_editor/src/core/jobs/ticket.rs
  - zircon_editor/src/core/jobs/pump.rs
  - zircon_editor/src/core/jobs/event.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/completion.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/job.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/poll.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/view_model.rs
  - zircon_editor/src/ui/host/export_cargo_process.rs
  - zircon_editor/src/ui/host/export_process_support/output_capture.rs
  - zircon_editor/src/ui/host/export_process_support/process_tree.rs
  - zircon_editor/src/ui/host/export_process_support/child_guard.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/state.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/start.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/worker.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/updates.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/session_state.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/retained_host/app/job_progress.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state/construction/assembly.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/ui/retained_host/viewport/viewport_state.rs
  - zircon_editor/src/ui/retained_host/viewport/bind_jobs.rs
  - zircon_editor/src/ui/retained_host/viewport/render_framework_resolve_job.rs
  - zircon_editor/src/ui/retained_host/viewport/viewport_state_drop.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/01/2026-07-17-editor-jobs-messaging-static-review.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - user: 2026-07-11 Plan 14 M2.1 export wizard controller hard cutover
  - user: 2026-07-11 Plan 14 M2.2 export worker and pipe-reader hard cutover
tests:
  - zircon_editor/src/core/jobs/progress.rs::tests::primary_snapshot_clones_only_the_smallest_visible_job
  - zircon_editor/src/core/jobs/test_support.rs
  - zircon_editor/src/core/jobs/tests/scheduling_contract.rs
  - zircon_editor/src/core/jobs/tests/pump_contract.rs
  - zircon_editor/src/core/jobs/tests/progress_contract.rs
  - zircon_editor/src/core/jobs/tests/background_storm_contract.rs
  - zircon_editor/src/core/jobs/tests/thread_ownership_contract.rs
  - zircon_editor/src/ui/host/export_cargo_process.rs
  - zircon_editor/src/ui/host/export_process_support/output_capture.rs
  - zircon_editor/src/ui/host/export_process_support/process_tree.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/job.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/panel_session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/cancellation_tests.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session/tests.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/tests/host/render_framework_boundary/mod.rs
  - zircon_editor/src/ui/retained_host/viewport/viewport_state_job_tests.rs
doc_type: module-detail
---

# Editor Jobs

`core/jobs` 是 Editor 后台工作的唯一调度门面。它复用 Runtime `JobScheduler`，不创建 Editor 私有线程池，也不暴露旧任务架构的兼容入口。

## 所有权与边界

`EditorContext` 持有一个 `EditorJobSystem`。`EditorContextBuilder` 强制接收外部 `JobScheduler`，`EditorManager` 从 `CoreHandle::scheduler()` 克隆 Runtime 已有调度器后再构造 context；Builder 同时把同一个 `SharedEditorMessageBus` 交给事件服务和 job 回流泵。因此生产 context 不会隐藏创建第二个线程池，job 完成事实也不会绕过 01 消息内核。

`EditorJobSystem` 只保留 `with_scheduler` 与 `with_scheduler_and_bus` 两个显式构造入口。旧的 `Default`、`with_limits`、`with_bus` 已硬删除，调用方无法再通过看似普通的默认构造隐式创建 `JobScheduler`。`#[cfg(test)]` 下的 `core/jobs/test_support.rs` 使用进程级 `OnceLock<JobScheduler>` 为测试夹具共享一份调度器；每个夹具仍获得独立的 admission state、event channel、bus 和配额，但并行库测试不会按夹具倍增 Rayon worker pool。

业务任务实现 `EditorJob`。trait 强制任务及输出为 `Send + 'static`，`JobContext` 只提供协作式取消与进度上报，不提供 UI、窗口或 `EditorContext` 访问。这一类型边界阻止 worker 直接改写主线程状态。

## 调度合同

- `after` 依赖直接转换为 `JobScheduler::schedule_after` 的 `JobHandle` 依赖。
- `MutexGroup` 保存组尾 `JobHandle`，后续任务把组尾追加为依赖，形成 Runtime 原生依赖链。
- 类别配额在 Editor 准入层计数；默认 Thumbnail 为 2、Export 为 1、Import 为 Runtime worker parallelism，其余类别不额外限流。
- 同一类别出现排队时按 Interactive、Normal、Background，再按 `JobId` 选择。该顺序是 Editor 逻辑准入顺序，不伪装成 Runtime/OS 线程优先级。
- `CancellationToken` 是协作式取消。启动前已取消的任务不执行；运行中的任务通过 `check_cancelled` 设置检查点。
- `EditorJobSystem::join` 原样透传 Runtime `JobScheduler::join`，允许一个已准入 job 在同一 runtime pool 内组织有借用的结构化并发。它不创建 Editor worker、不分配新的 `JobId`，也不占第二份类别许可。
- `EditorJobSystem::cancel(JobId)` 对 pending job 在 state lock 内原子摘除并取消共享 token，锁外同步产生 typed `Cancelled` ticket 与通用事件，再收敛 terminal record 并继续 admission promotion；该路径不提交 Runtime worker、不占类别许可、不等待 dependency 或 mutex tail。已经 scheduled/running 的任务通过唯一进度生命周期表取得并取消同一 token，返回 `true` 表示协作取消请求已送达；未知或已进入 terminal 可见状态的 ID 返回 `false`。

内部锁均在 poison 后恢复 inner state。任务 panic 被捕获并转换为 `JobError::Panicked`；完成 guard 即使在任务包装层异常退出也会释放类别许可，避免队列永久停滞。

## 主线程回流

worker 把 `Started/Progress/Completed/Failed/Cancelled` 写入 MPSC，不直接调用消息 bus。主循环调用 `EditorJobSystem::pump_events()`，再把事件作为 `EditorMessagePayload::Job` 发布到 `TOPIC_JOB`。这使主线程回流是显式动作，并避免 worker 执行订阅者逻辑。

`JobTicket` 提供两种读取方式：`wait()` 阻塞取结果，`try_take()` 非阻塞拉取。结果只有一个所有者，成功或通道关闭后再次 `try_take()` 返回 `None`。

业务失败由 `JobError::Failed(JobFailure)` 保存为 `Arc<dyn Error + Send + Sync>`，不再把错误压成 `String`。`JobError::failed(error)` 接受具体错误类型，`JobError::downcast_ref::<E>()` 可在 ticket/session 边界恢复原始类型；clone 共享同一 source 身份，错误相等性不依赖文本。`JobFailure` 与 `JobError::Failed` 都参与标准 `Error::source()` 链，便于诊断器继续遍历底层 IO、进程或业务错误。只有 `JobEventKind::Failed`、状态栏和面板 diagnostics 等展示投影可以调用 `to_string()`；`JobError::Panicked(String)` 仍只表示 Rust panic payload，不冒充业务 typed error。

## 统一进度中心

`EditorJobProgressSource` 是 active job 元数据、取消 token 与 UI 进度的唯一事实源；调度 state 不再保存第二份 `active_jobs`。`submit` 在共享 state lock 内完成合法性检查、JobId 注册和进度表注册，`snapshot()` 以 `BTreeMap<JobId, ...>` 给出稳定升序的只读 `EditorJobProgressSnapshot`。`JobContext::report_progress(completed, total, message)` 只更新该事实源并继续产生通用 `Progress` 事件，不允许业务队列维护第二套任务面板状态。

terminal 事件先把条目标记为 UI 不可见，因此状态栏和任务面板不会展示已完成、失败或取消的任务；条目仍保留到 Runtime handle、类别许可、mutex tail 与 scheduler record 真正清理完成，随后 `complete(JobId)` 才物理移除。这个“终态可见性”与“生命周期完成”分离保证 `shutdown(deadline)` 不会因为 terminal event 先于资源收尾而提前返回。

`EditorHostEventController::job_progress_snapshot()` 保留完整只读任务列表入口。状态栏每帧只需要最小 `JobId` 的 active job，因此走 `primary_job_progress_snapshot()` / `EditorJobProgressSource::primary_snapshot()`，直接从有序事实源找到首个非 terminal entry 并只克隆这一项；消息为空时回退到类别名，百分比使用 `u64` 中间值计算、总量为 0 时保持不确定态并钳制到 100。旧 Export queue 的 `sync_desktop_export_status_task` / `desktop_export_status_task_from_queue` 事实源及文件已硬删除，Export 只通过 `JobContext::report_progress` 进入统一中心；任务面板外观继续由 Editor Layout 负责。

## 关停协议

`EditorJobSystem::shutdown(deadline: Instant)` 是 Context 所有后台工作的统一收尾边界。第一次调用在共享 `Arc` 状态内停止接收新任务；所有 clone 随即共享关闭事实，后续 `submit` 返回 typed `JobSubmitError::ShuttingDown`，不存在重新开启或旧提交入口。

关停会原子取消全部仍活跃任务的 `CancellationToken`，同时从 Editor 准入队列整体摘除尚未交给 Runtime 的任务。pending ticket 在锁外立即收敛为 `JobError::Cancelled` 并产生通用 `Cancelled` 事件，不等待类别许可、依赖或互斥组尾。已经 scheduled/running 的任务沿既有协作式检查点退出；不可中断任务允许运行到 deadline。

等待使用 condition variable，并在等待 worker 完成期间释放 state mutex。deadline 到达后，返回按 `JobId` 稳定升序排列的 `Vec<UnfinishedEditorJob>`；每项包含 `id/label/category`，供 17 崩溃恢复与诊断记录消费。重复或并发调用 `shutdown` 是幂等的：它们共享关闭位和 active metadata，不创建额外 worker，也不会产生第二次 pending terminal 结果。

## M3.3 后台风暴基线

`thumbnail_storm_preserves_quota_and_records_main_thread_pump_baseline` 构造 1000 个 `Background/Thumbnail` job，在 gate 释放前固定验证 `scheduled=2`、`pending=998`，且不创建 Editor 线程、async runtime 或私有线程池。释放后逐 JobId 验证严格 `Started -> Progress(1/1, "thumbnail ready") -> Completed`、ticket 单次成功消费和最终 pending/running/scheduled/mutex-tail 全归零，同时记录 submit、主线程 pump+delivery tick 与非空 batch 的 P50/P95/max。

当前 runtime/03 没有 Editor job pump 的数值帧时预算，因此测试只输出机器可读 `EDITOR_JOB_STORM_BASELINE ... numeric_budget=undefined` 观察值，不把墙钟样本伪装成 SLA。runtime/07 的同命令两次样本均值相对偏差 `<20%` 仅用于 Windows 测试阶段的基线可重复性检查；它不是 scheduler 性能通过阈值。测试中的 1ms 诊断节拍位于计时区间外，并把 60 秒 watchdog 下的样本向量限制在约 60000 项。

## 首个业务客户：Export Wizard

`ExportWizardJobController` 是 `EditorJobSystem` 的首个生产业务客户。controller 只保留业务细粒度 `ExportWizardJobEvent` 通道、Context-owned `EditorJobSystem` clone、共享 `CancellationToken` 和 `JobTicket<ExportWizardJobSnapshot>`；旧的私有线程、`JoinHandle`、`AtomicBool` handle 与 `spawn` 入口已删除。取消同时设置 token 并调用 `EditorJobSystem::cancel(ticket.id())`，因此 queued job 立即终结，scheduled/running job 继续协作取消。窄 `controller/job.rs` owner 实现 `EditorJob`，使用 `JobCategory::Export` 提交，并把 `JobContext` token 直接适配为现有 `ExportWizardCancelSignal`，所以阶段前、阶段内与阶段后的取消检查继续走同一业务执行路径。

每个业务事件同步上报通用 progress，计数只使用“已完成 stage 数/计划 stage 总数”，消息包含业务 event kind 与当前 stage，不推导或伪造百分比。业务 owner 将 `Finished` snapshot 返回为成功，将只有明确 cancellation error 的 `Cancelled` 映射为 `JobError::Cancelled`。runner 返回真实 typed error 时，stage snapshot 以 `JobFailure` 保留该 source，同时另存字符串 diagnostics 供 UI 展示；controller 把同一 `JobFailure` 原样交给 ticket。只有非零退出码、缺失输入等没有底层 `Error` 的业务失败才构造结构化 `EditorExportBuildError::WizardStageFailed`，意外非终态使用 `WizardNonTerminal`；不保留字符串包装错误。因此通用 `Started/Progress/Completed/Failed/Cancelled` 仍只由 `EditorJobSystem` 产生，ticket 可 downcast 回 `EditorExportBuildError` 并继续遍历 process/native/IO source，terminal kind 与业务终态一致。业务事件顺序和流式 stdout/stderr snapshot 投影保持独立。

`ExportWizardPanelSession` 显式持有注入的 `EditorJobSystem` clone。生产 retained host 从 `EditorManager.context().jobs()` 注入同一服务，不创建第二套默认 pool/bus；测试通过 test-only helper 构造逻辑隔离、调度器共享的 fixture。提交错误保持为 `JobSubmitError`，ticket 完成错误保持为 `JobError`，并在 session 边界分别映射为 `ExportWizardPanelSessionError::JobSubmit` 与 `ExportWizardPanelSessionError::Job`，不再退化为 join/panic 字符串。

controller 的窄 poll 状态把 `JobTicket::try_take()` 映射为 pending，或携带剩余业务 events 的 completed snapshot/typed `JobError`。Panel session 先做常规业务 drain；ticket ready 后 controller 立即再次 drain 自己的 receiver，把跨通道竞态窗口内到达的 terminal events 与结果作为一个 poll outcome 返回。Session 先应用这些 events，再以 ticket snapshot 收束成功状态或映射错误并清除 controller，所以 `StageFinished/Cancelled/Failed` 的最终 stage/output/diagnostics 不会丢失。类别配额内排队、owner 启动前取消以及 panic 等没有业务 terminal event 的路径也不会永久 active。阻塞 `finish()` 硬切为单一 `ExportWizardJobCompletion`，等待 ticket 后一次性收集剩余业务事件与 typed result；session 同样先应用全部业务事件再处理 result，不存在旧 `Result` 兼容入口或 ticket 重复消费。

retained host 的每帧主线程 `tick()` 是通用 job 事件的生产 pump owner。它在业务 job/session polling 前调用同一个 `EditorManager.context().jobs().pump_events()`；worker 仍只写 MPSC，消息 bus 发布只发生在该 retained tick 边界。局部静态合同只扫描 `#[cfg(test)]` 前的生产源码，固定单一 pump 调用及其先于 export wizard polling 的顺序；该合同证明 owner/顺序，不声称验证 OS thread identity。

## M2.2 导出 worker 与阻塞管道

`ProcessCommandRunner` 已从 unit/`Copy` runner 硬切为持有注入 `EditorJobSystem` 的结构体，并且只能通过 `ProcessCommandRunner::new(jobs)` 构造。`ExportWizardPanelSession` 的默认启动路径和 retained wizard session 都把自身已有的同一 jobs clone 注入 runner，不创建生产默认 scheduler。向导把 stdout/stderr 重定向到每次调用唯一的临时 capture 文件；monitor 循环以嵌套 `jobs.join` 并列执行两个“最多读取 64 KiB 已落盘字节”的有界 reader 与一次 `try_wait`，随后在同一 job 上做逐行增量解码和 UI emit。regular-file EOF 读取与 `try_wait` 都不会阻塞 worker，因此 Runtime parallelism=1 时也不会因 reader 占满 worker；退出后 final drain 循环读到双流 EOF，保留最后一批输出和无换行尾行。child RAII guard 在 panic/unwind 时终止并 wait 进程树，正常完成在 final drain 后显式 disarm；capture RAII guard 后析构并清理文件。reader 不作为新的 Export ticket 提交。

Cargo 导出进程复用同一 capture/join/final-drain 结构，返回后一次性构造 `EditorExportCargoInvocation`。Wizard 与 Cargo 也复用同一个进程树 helper：Unix 在独立 process group 启动并终止整组，Windows 使用 `taskkill /T /F`，平台路径失败时才尝试 direct-child fallback。整个 native preparation、generated export build 与 retained queue 链路只接受 `CancellationToken`；旧 `AtomicBool`、`Arc<AtomicBool>` 和可选 atomic 引用已删除。未取消的直接导出入口会为本次同步调用创建独立 token，但不存在旧 atomic 重载。

`DesktopExportJobQueue` 由 retained startup 从 `EditorManager.context().jobs().clone()` 显式构造。pending entry 持 token；active entry 持 `JobTicket<DesktopExportJobResult>`、同一 token 和最新 progress。`start_next` 以 `JobCategory::Export` 提交一个 `DesktopExportEditorJob`。worker 只发送 progress DTO；成功返回 typed result，取消返回 `JobError::Cancelled`，业务失败返回 `JobError::Failed`，因此通用 Job bus 的 terminal kind 与 retained UI 终态一致。主线程 `poll_updates()` 通过 `ticket.try_take()` 统一折算 exported/failed/cancelled summary。active 取消同时调用 `token.cancel()` 和 `jobs.cancel(ticket.id())`，使仍在准入队列的任务立即终结，而运行中的任务沿共享 token 在 native preparation/Cargo polling 检查点退出；pending 取消从 `VecDeque` 删除后取消 token。旧 worker thread 与 `Finished` 消息 owner 均不存在。

零裸线程 guard 递归扫描 `zircon_editor/src` 中的生产 Rust 源码，明确排除名为 `tests` 的测试目录以及 `tests.rs`、`*_test.rs`、`*_tests.rs` 测试模块文件。生产范围内拒绝直接限定名、短限定名、单项/brace import 的 spawn，并拒绝 `std::thread::Builder`/导入 Builder 后的 `.spawn(...)`。检测 pattern 在测试运行时拼接，guard 本身不会因内嵌禁用字面量自命中；不设置白名单或逐文件忽略，合法的并发测试可以继续使用线程验证锁序，`sleep`/`yield_now` 不属于线程所有权创建，继续允许。范围回归测试逐类断言规范测试目录和测试文件会被排除，同时生产 owner 文件不会被误排除。

Viewport 的 lazy `RenderFramework` resolve 同样由 Context jobs 所有。startup assembly 在取得 `editor_jobs` 后显式调用 `viewport.bind_jobs(editor_jobs.clone())`；state 以 `JobTicket<Arc<dyn RenderFramework>>` 保存异步结果，`try_take()` 完成主线程收口，并以 `JobCategory::Misc` 提交提取后的 typed `RenderFrameworkResolveJob`。该 job 在调用 `CoreHandle::resolve_render_framework()` 前执行协作取消检查。旧 Builder/JoinHandle owner 已删除，state drop 会尝试取消仍在 admission queue 的 ticket；结构守卫跟随当前提取 owner，不再要求把 resolver 内联回 state 文件。

## 验证状态

2026-07-11 的 M1 局部验证覆盖 typed output、启动前取消、after 顺序（含业务失败依赖后续不悬挂）、类别并发上限、跨类别互斥组、准入优先级、typed panic、非法 dependency、MutexGroup 反序列化校验、ticket 单次拉取，以及 worker→pump→01 bus 和 Context 同 bus 接线。focused 合同 12/12、Runtime tasks 回归 3/3 已通过。M2.1/M2.2 已补充 export wizard、retained queue、join passthrough、CancellationToken、child/process-tree RAII、viewport typed ticket/drop cancel 与零裸线程 guard 契约源码。

2026-07-12 的 M3 受管 Windows 门禁在修正 `PendingJob.cancel_task` trait-object 字段调用语法后复跑为 36 passed / 0 failed / 3035 filtered out；统一进度源、UI 只读入口、关停并发合同、零裸线程守卫和 1000-job 风暴均在同一过滤器内通过。风暴精确命令随后连续两次各 1/1 通过，`release_elapsed_ns` 为 278322000 / 260519800，均值相对偏差 6.607579%；`submit_total_ns` 为 104629400 / 126985600，均值相对偏差 19.304622%。两者仅作为可重复性基线；`numeric_budget=undefined` 仍表明没有伪造 runtime/03 数值 SLA。日志位于 coordinator 管理的 `D:/cargo-targets/zircon-engine/pool/525d696af7ff7754af95fa549668e728f5cc05bc36657acf1040f23e37a9cd34/`。既有 Editor UI/Layout 与 Export typed error 边界失败继续由对应功能计划处理，本模块不增加兼容层代修。

2026-07-14，当前 `render_framework_boundary/mod.rs` 由独立 `rustc --test` harness 直接编译执行，3 项 RenderFramework boundary 测试通过；与 Editor03 守卫合并结果为 6 passed / 0 failed、9.72 秒，日志为 `.codex/tmp/editor03-render01-guard-standalone-20260714.log`。该证据只验收本次 source guard hard cut；共享完整 Cargo lib gate 仍由其他功能失败阻塞，未在此宣称通过。
