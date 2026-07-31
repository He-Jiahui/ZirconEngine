---
related_code:
  - zircon_editor/src/core/jobs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/host/export_cargo_process.rs
  - zircon_editor/src/ui/host/export_process_support
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue
  - zircon_editor/src/ui/retained_host/viewport
reference_sources:
  - dev/bevy/crates/bevy_tasks/src/usages.rs
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphInterfaces.h
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
implementation_files:
  - zircon_editor/src/core/jobs/mod.rs
  - zircon_editor/src/core/jobs/event_sink.rs
  - zircon_editor/src/core/jobs/progress.rs
  - zircon_editor/src/core/jobs/pump.rs
  - zircon_editor/src/core/jobs/system/mod.rs
  - zircon_editor/src/core/jobs/system/pending.rs
  - zircon_editor/src/core/jobs/system/state.rs
  - zircon_editor/src/core/jobs/test_support.rs
  - zircon_editor/src/core/jobs/cancellation_token.rs
  - zircon_editor/src/core/jobs/ticket.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/job.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/completion.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/poll.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/host/export_cargo_process.rs
  - zircon_editor/src/ui/host/export_process_support/output_capture.rs
  - zircon_editor/src/ui/host/export_process_support/process_tree.rs
  - zircon_editor/src/ui/host/export_process_support/child_guard.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/state.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/start.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/updates.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/job_queue/worker.rs
  - zircon_editor/src/ui/retained_host/viewport/bind_jobs.rs
  - zircon_editor/src/ui/retained_host/viewport/render_framework_resolve_job.rs
  - zircon_editor/src/ui/retained_host/viewport/viewport_state.rs
  - zircon_editor/src/ui/retained_host/viewport/viewport_state_drop.rs
tests:
  - zircon_editor/src/core/jobs/tests/scheduling_contract.rs
  - zircon_editor/src/core/jobs/tests/pump_contract.rs
  - zircon_editor/src/core/jobs/tests/background_storm_contract.rs
  - zircon_editor/src/core/jobs/tests/thread_ownership_contract.rs
  - zircon_editor/src/ui/host/export_cargo_process.rs
  - zircon_editor/src/ui/host/export_process_support/output_capture.rs
  - zircon_editor/src/ui/host/export_process_support/process_tree.rs
  - zircon_editor/src/ui/host/export_process_support/child_guard.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/job.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/panel_session.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/tests.rs
  - zircon_editor/src/ui/retained_host/viewport/viewport_state_job_tests.rs
doc_type: implementation-plan
status: in_progress
---

# 14 多线程调度管理

## 参照证据（dev/）

**bevy 三池分类**（`dev/bevy/crates/bevy_tasks/src/usages.rs:52-76`）：

```rust
// ComputeTaskPool  — CPU 密集、须在下一帧前完成（帧内并行）
// AsyncComputeTaskPool — CPU 密集、可跨多帧（后台计算）
// IoTaskPool — I/O 密集（读盘/网络）
```

`TaskPool::{spawn, install, join}`（`task_pool.rs:21-70`）。要点：**按完成期限与资源特征分池**，而非按业务域分池——编辑器 job 类别是池之上的逻辑层。

**UE 命名线程与优先级**（`TaskGraphInterfaces.h:54-108`）：`ENamedThreads` 三命名线程（GameThread/ActualRenderingThread/RHIThread）+ `AnyThread` + 队列（MainQueue/LocalQueue）+ 任务/线程双优先级（Normal/High task × Normal/High/Background thread）——**回主线程是显式寻址**（`AnyThread` 完成后 dispatch 到 GameThread），不是约定。

## 现状与证据（zircon）

**runtime 任务内核完备**（`core/runtime/tasks/` 实测 13 文件：`diagnostics / job_handle / job_scheduler / mod / parallel_for / pool / pools / report / thread_assignment`）：

```rust
// pool.rs:21-69
TaskPool::{ spawn(task), install(task) -> R, join(a, b) -> (RA, RB) }
// TaskPoolDescriptor { worker_threads: Option<usize>, thread_name: String, kind: TaskPoolKind }
// job_scheduler.rs:34-100 —— 依赖调度已内建！
JobScheduler::{ spawn(task), schedule(task) -> JobHandle,
                schedule_after(dependencies, task) -> JobHandle }
```

另有 `parallel_for`（数据并行）、`report/diagnostics`（任务观测）、`thread_assignment`（`TaskPoolThreadAssignmentPolicy` + `TaskPoolOptions`，:2-25——优先级映射的会签对象即此二型）、资产侧 `pipeline/worker_pool.rs` 专池（`request/completion_receiver` 通道，09 已核）。签名复核（2026-07-05）：`spawn/install/join` 在 `pool.rs:52-60`，`schedule/schedule_after` 在 `job_scheduler.rs:51-65`，`JobScheduler` 亦自带 `install/join`（:121-125）。

**编辑器侧自管线程当前源码复核（2026-07-11）**：最初取证时 `std::thread::spawn` 仅命中 `export_build/wizard/controller.rs`；M2 开始前复核发现 wizard process stdout/stderr reader、retained build-export queue worker 与 `export_cargo_process.rs` pipe reader，共 4 个 `thread::spawn` owner。M2.2 零命中守卫评审又识别出 `retained_host/viewport/viewport_state.rs` 通过 `std::thread::Builder::spawn` 隐藏的第 5 个 owner。五处全部归 M2 散点收编，不再沿用“仅 1 处”或只扫描直接 spawn 的过期结论：

```rust
// ExportWizardJobController (controller.rs:27-78)
pub struct ExportWizardJobController {
    handle: ExportWizardJobHandle,               // Arc<AtomicBool> 取消信号
    events: Receiver<ExportWizardJobEvent>,      // mpsc 事件流
    worker: JoinHandle<ExportWizardJobSnapshot>, // std::thread
}
// spawn() / handle() / request_cancel() / events() / finish() -> Result<Snapshot>
```

——这些都是**手工实现的 job/reader 协议**（取消信号/事件流/结果快照/JoinHandle），形状正确但绕开了 runtime `JobScheduler`。M2 必须把可调度工作迁入 `EditorJobSystem`；子进程管道并发读取若仍需专用阻塞 reader，也必须由门面提交并以 typed ticket 收口，不保留裸 `thread::spawn`。

**缺口**：编辑器无统一 job 门面（导出向导的取消/事件/快照协议是孤例，导入、缩略图、编译、registry 扫描各计划将各造一套）；无类别/互斥/优先级层（`JobScheduler` 有依赖无类别）；无主线程回流约定（UE GameThread 寻址的对应物）；无进度中心数据源；关停无收尾协议。

## 目标

1. **`EditorJobSystem` 门面**（包装 runtime `JobScheduler/TaskPool`，编辑器**零自建线程池**）：

```rust
pub struct EditorJobSpec {
    pub label: String,
    pub category: JobCategory,        // Import/Compile/Thumbnail/Export/Index/Play/Misc
    pub priority: JobPriority,        // Interactive/Normal/Background（Editor 准入顺序）
    pub mutex_group: Option<MutexGroup>,   // 如 script_artifacts（13）、同 path 导入（09）
    pub cancel: CancellationToken,    // ExportWizard 的 Arc<AtomicBool> 协议泛化
    pub after: Vec<JobId>,            // 直通 JobScheduler::schedule_after 既有能力
}
pub trait EditorJob: Send + 'static {
    type Output: Send + 'static;
    fn run(self, ctx: JobCtx) -> Result<Self::Output, JobError>;  // ctx: 进度上报 + 取消检查点
}
impl EditorJobSystem {
    pub fn submit<J: EditorJob>(&self, spec: EditorJobSpec, job: J) -> JobTicket<J::Output>;
}
```

2. **主线程回流约定**（UE 显式寻址直译）：job 完成/失败/进度一律折算 `EditorMessagePayload::Job(JobEvent)` 入 01 bus，主循环 drain 应用——`JobCtx` 不提供任何 UI/EditorContext 访问（类型层：job 闭包只捕获 `Send` 数据）；`JobTicket` 双态取结果：完成消息通知（推）或 `try_take()`（拉），一源两用。
3. **类别配额与互斥**：类别→并发上限表（Thumbnail≤2、Import≤worker_pool 宽度、Export=1…，设置化 17）；`MutexGroup` 内串行（`schedule_after` 链式实现）；`Interactive/Normal/Background` 是 Editor 准入队列的逻辑优先级，满额类别释放许可后按该顺序选择下一任务，防后台队列持续插队。当前 runtime `thread_assignment` 只有线程数 min/max/percent，没有任务/线程优先级枚举，M1 不伪造不存在的直接映射；是否申请独立 Background pool 由 M3 压测与 runtime/03 会签决定。
4. **散点收编**：`ExportWizardJobController` 迁为 `EditorJobSystem` 首个客户（其取消/事件/快照协议即门面协议的验证原型，迁移后删除手工线程）；09 导入、10 registry 扫描、13 编译、缩略图、04 Play 子进程监视全部经门面（Play 监视为 `JobCategory::Play` 的长驻 job）。
5. **进度中心与收尾**：活跃 job 数据源 `{label, category, progress: Option<(u32,String)>, cancellable}`（状态栏/任务面板消费）；关停协议：`shutdown(deadline)` → 停收新 job → 广播取消 → 等待至 deadline → 记录未竟 job 清单（17 崩溃恢复衔接）。

## 非目标

- 不改 runtime tasks 内核（类别/配额是编辑器层；内核需求走 runtime/03 提案）；不引 async 运行时（`TaskPool::spawn` 同步任务模型够用，`install/join` 保留给数据并行场景）；进程外任务的进程管理（04/15 自持 Child，仅其**监视**入门面）。

## 架构设计

### 模块布局

```
zircon_editor/src/core/jobs/
  mod.rs
  system/          # EditorJobSystem：submit/准入状态/待执行任务
  spec.rs          # EditorJobSpec
  category.rs      # JobCategory/JobPriority
  cancellation_token.rs
  ticket.rs        # JobTicket 推拉双态
  pump.rs          # bus 回流泵（JobEvent 折算）
  event.rs         # 可序列化 JobEvent 消息族
  progress.rs      # M3 进度中心数据源
```

`EditorContext`（01）持 `jobs: EditorJobSystem` 服务位。

### 与 runtime 内核的映射

| 门面概念 | runtime 既有物 | 映射方式 |
| --- | --- | --- |
| submit | `JobScheduler::schedule` | 直通 |
| after 依赖 | `schedule_after(dependencies, task)` | 直通（既有能力，零新建） |
| MutexGroup | 同上 | 组内前 job 的 handle 作后 job 依赖（链式） |
| 类别配额 | 无 | 门面层许可计数（信号量语义），满则排队 |
| 优先级 | runtime 当前无任务优先级；`thread_assignment` 仅分配线程数 | Editor 类别准入队列按 Interactive→Normal→Background 选取；不宣称 OS/worker priority |
| 取消 | 无内核支持 | `CancellationToken`（AtomicBool）+ job 内检查点协作式取消 |

### 深度测试

夹具 job 族（可编程时长/失败点/取消检查密度/进度序列）覆盖：并发上限（提交 N>上限 断言在途≤上限）、互斥组串行序、after 依赖序、取消及时性（检查点粒度内）、失败传播、关停 deadline 三路径（全完/取消/超时记录）——全部不依赖真实业务 job。

## 里程碑

### M1 门面与回流泵

- 切片 1.1：`core/jobs/` 文件夹模块；submit/配额/互斥/after 映射 `JobScheduler`；`CancellationToken` 协议。
- 切片 1.2：`pump.rs` 回流泵接 01 bus（`JobEvent` 消息族）；`JobTicket` 推拉双态；Send 约束的类型层验证（编译失败测试：trybuild 或 doc-test 断言非 Send 捕获不过编译）。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`（夹具矩阵全绿）+ `cargo test -p zircon_runtime --lib --locked`（tasks 内核消费不回归）。更新 `docs/zircon_editor/core/jobs.md`。

### M2 散点收编

- 切片 2.1：`ExportWizardJobController` 迁门面（协议对齐：`ExportWizardJobEvent`→进度序列、`request_cancel`→token、`finish`→ticket），删除 `controller.rs:49` 手工 spawn；导出既有测试迁移。
- 切片 2.2：迁移 M2 当前源码复核发现的其余四类线程 owner（wizard/Cargo pipe readers、retained export queue worker、viewport render-framework resolver），随后落裸线程守卫测试（zircon_editor 全 crate 的 direct/Builder/import alias 零命中、白名单空，守卫防复发）。
- 测试阶段：导出向导既有流程测试全过 + 守卫测试落地；手验导出向导取消/进度 UI 无回归。

### M3 进度中心与收尾协议

- 切片 3.1：`progress.rs` 数据源 + 状态栏/任务面板接线（面板外观 editor_layout）。
- 切片 3.2：`shutdown(deadline)` 协议 + 未竟清单；类别配额设置化（17）。
- 切片 3.3：后台风暴压测夹具：1000 缩略图 job 下主循环帧时基线（与 runtime/03 预算口径对账）。
- 测试阶段：收尾三路径矩阵；压测基线记状态节；证据记状态节。

## 风险与开放问题

- 与 runtime 帧调度共享 `TaskPool` 的干扰：若压测显示后台类别侵蚀帧预算，为 Background 优先级申请独立 `TaskPoolDescriptor{ kind }` 池（pools.rs 多池机制既有，属配置非内核改动）——证据驱动，决策记状态节。
- 协作式取消对不可中断步骤（外部进程等待、单次大文件读）的语义：取消=尽力 + 结果丢弃 + job 标记 `CancelledLate`，契约文档明示。
- 2026-07-11 前置复核确认 `TaskPoolThreadAssignmentPolicy` 只有 `min_threads/max_threads/percent`，不存在三级优先级映射；M1 采用编辑器准入顺序。若 M3 压测证明不足，按上一条风险向 runtime/03 提案，不在 Editor 复制线程池或伪造优先级。

## 产出记录与时间

请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前状态：M1 局部门禁已通过；M2 线程 owner 硬切、guard 范围和 Editor scheduler 所有权修复已实现；M3.1 唯一进度事实源与 retained UI 只读入口、M3.2 关停 deadline/未竟清单、M3.3 1000 个后台缩略图任务基线夹具均已完成，M3 受管 Windows 局部门禁 36/36 通过，风暴精确命令连续两次各 1/1 通过且 release wall-clock 均值相对偏差 6.607579%。最新 full harness 的 5547-thread 最低根因已进一步证实为 Runtime02 service registry 与 `EditorUiHost.core` 的强 `CoreHandle` 自拥有环；Runtime11 的三池 + asset worker 双预算改为等待生命周期环修复后的独立复测项。本计划不以 test-only 小池、共享业务 Runtime 或分区替代全量门。

- 产出归档：[2026-07-12-thread-ownership-and-resource-gates.md](14/2026-07-12-thread-ownership-and-resource-gates.md)
- fixed 已修复：[editor-full-gate-thread-exhaustion](08/fixed-2026-07-14-editor-full-gate-thread-exhaustion.md)
- 最低共享层交接（`open / Runtime11 任务资源与 asset worker 预算`）：[editor-full-harness-runtime-thread-budget](../../zircon_runtime/runtime/11/failure-2026-07-13-editor-full-harness-runtime-thread-budget.md)
- fixed 已修复：[service-corehandle-retention-cycle](14/fixed-2026-07-14-service-corehandle-retention-cycle.md)
- fixed 已修复：[componentized-workspace-test-export](14/fixed-2026-07-12-componentized-workspace-test-export.md)
- fixed 已修复：[component-registry-typed-contract-test](14/fixed-2026-07-12-component-registry-typed-contract-test.md)
- fixed 已修复：[export-build-string-error-boundary](14/fixed-2026-07-12-export-build-string-error-boundary.md)
- fixed 已修复：[animation-state-machine-infallible-conversion](14/fixed-2026-07-11-animation-state-machine-infallible-conversion.md)
- 2026-07-30 core jobs当前源复核：`core/jobs/**` 27/27确认priority/category/dependency selection固定为每pass最多21 bucket probes，pump已有64 events/1ms且Progress按JobId latest coalesce，旧全scan/Vec移位和drain-to-empty结论已过时。剩余P0：Compile/Index/Play/Misc默认无限，`promote`持state mutex循环`Runtime schedule_after`并可整批转移ready work；Started/terminal lane无entry/bytes/age界；emit跨lifecycle/progress/queue三锁并clone稳定label、progress message双份常驻；pinned terminal history超256后锁内线性find+中段remove。Editor14按PERF-MVP-018/020做全类别entry+estimated-bytes+age reservation、bounded dispatch batch、锁外schedule和generation-safe install/rollback，label/spec/message共享owner，history用dependency refcount+evictable ordered index；accepted terminal不得丢。
- 2026-07-30 progress/ticket补充：`primary_snapshot`已把状态栏从clone全部active jobs降为1项，但retained tick仍每帧clone label/message、format task/detail并无条件presentation refresh；按PERF-MVP-017发布primary generation，stable tick投影/刷新为0。`JobTicket::wait`/`join`保持worker/tool/shutdown-only；当前产品UI均poll，wizard `finish_job`阻塞入口无生产caller，接入前补main/retained-thread source+affinity guard。
- 2026-07-31 close-prompt save接入：当前retained native close在UI callback逐view同步执行serialize、文件写入、asset import、workspace refresh/hydrate/sync，partial failure retry还会重复已成功I/O。Editor14按PERF-MVP-602为Editor09 canonical save batch提供显式有界interactive lane、entry/estimated-bytes/age reservation、per-resource mutex group、cooperative cancel/进度与有界completion apply；不得落入当前默认无限`Misc`整批promote，也不得把`UiHostWindow`、retained host borrow或session mutex搬到worker。UI线程的fs/import=0，completion按dirty generation一次提交，shutdown沿M3.2 deadline返回明确未完成清单。
- 2026-07-22 Play process output补充：live poll已止损为64 lines/poll且terminal finish不持active mutex；两个reader仍为per-session手建thread，`read_until`允许单行无界，queue无bytes/time/age预算。按PERF-MVP-552与[open failure](14/failure-2026-07-22-play-process-output-byte-budget.md)接Runtime11 blocking-I/O owner，不能以1024 entries冒充内存有界。
- 2026-07-22 asset import准入补充：Editor09同URI mutex group只串行、不合并等价generation请求；
  watch/digest/manual storm会把重复job推入现有无entry/byte/age预算的submission/lifecycle队列。Editor14提供
  typed single-flight admission与queue budget指标，Editor09拥有UUID/source-generation/reason语义，Runtime04
  保持唯一import执行owner；见Editor09
  [open failure](09/failure-2026-07-22-asset-import-duplicate-admission-backpressure.md)与PERF-MVP-555。
- 2026-07-22 script build准入补充：Editor13已把watch unique path常驻限制为20+full-rebuild sentinel，
  但Command/Play仍向无界request FIFO逐项提交，持续watch也可无限后推deadline。Editor14提供
  generation-keyed shared ticket、entry/bytes/age预算、cancel/supersede与latest Play resume intent；见
  Editor13 [open failure](13/failure-2026-07-22-script-build-debounce-admission-backpressure.md)和PERF-MVP-557。
- 2026-07-22 Welcome probe准入补充：PERF-MVP-559要求draft input debounce同时有max feedback latency，queued stale generation在I/O前supersede，同target共享ticket；Editor14把probe计入submit entry+draft bytes+oldest-age预算，禁止取消token后仍无界排队。语义owner见Editor10 [open failure](10/failure-2026-07-22-welcome-project-probe-admission-storm.md)。

## Code Review 建议 (2026-07-31)

### 与代码现状不符，需修订

- 「架构设计 §模块布局」列出 `core/jobs/` 为 `mod.rs / system/ / spec.rs / category.rs / cancellation_token.rs / ticket.rs / pump.rs / event.rs / progress.rs` 九项。当前实读已扩展到更细的拆分：除计划所列外还有 `context.rs`（`JobCtx` owner）、`error.rs`（`JobError`）、`id.rs`（`JobId`）、`job.rs`（`EditorJob` trait）、`limits.rs`（类别配额表，对应目标 3）、`mutex_group.rs`（`MutexGroup`，对应目标 3）、`shutdown.rs`（对应目标 5 `shutdown(deadline)`）、`event_sink.rs`、`test_support.rs`，且 `tests/` 已含 `admission_scaling_contract.rs`、`progress_contract.rs`（front-matter tests 字段只列到 `scheduling/pump/background_storm/thread_ownership` 四项）。模块布局图与 front-matter tests 清单的正文引用宜同步为当前拆分，避免读者按九项结构核对时误判 `limits.rs`/`mutex_group.rs`/`shutdown.rs` 为缺失。这些新增文件恰好证明目标 3（类别配额/互斥）与目标 5（收尾协议）已落地为独立 owner，与产出记录「M3.2 关停 deadline/未竟清单已完成」一致。（注：front-matter 本身不改，仅提示正文与之对齐。）

- 2026-07-31 retained progress adapter补证：`primary_snapshot()`已把全active job clone降为一项，但stable tick仍clone label/message、format task id/detail并调用setter；setter为比较又读取owned current status。Editor14按PERF-MVP-017发布immutable primary snapshot+generation，EditorUI08只在generation变化时投影，100K stable ticks的snapshot/String/format/setter/invalidation均为0。证据见`../../performance/01/2026-07-31-editor-retained-tick-projection-adapters-current-review.md`。
