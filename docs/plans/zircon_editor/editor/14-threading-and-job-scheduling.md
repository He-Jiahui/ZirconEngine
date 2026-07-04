---
related_code:
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller.rs
reference_sources:
  - dev/bevy/crates/bevy_tasks/src/usages.rs
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphInterfaces.h
plan_sources:
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
status: planned
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

**编辑器侧几乎无自管线程**（好于 v1 计划假设）：Grep `std::thread::spawn` 于 `zircon_editor/src` **仅 1 处命中**——`export_build/wizard/controller.rs:49`：

```rust
// ExportWizardJobController (controller.rs:27-78)
pub struct ExportWizardJobController {
    handle: ExportWizardJobHandle,               // Arc<AtomicBool> 取消信号
    events: Receiver<ExportWizardJobEvent>,      // mpsc 事件流
    worker: JoinHandle<ExportWizardJobSnapshot>, // std::thread
}
// spawn() / handle() / request_cancel() / events() / finish() -> Result<Snapshot>
```

——这是一套**手工实现的 job 协议**（取消信号/事件流/结果快照），形状正确但绕开了 runtime `JobScheduler`。

**缺口**：编辑器无统一 job 门面（导出向导的取消/事件/快照协议是孤例，导入、缩略图、编译、registry 扫描各计划将各造一套）；无类别/互斥/优先级层（`JobScheduler` 有依赖无类别）；无主线程回流约定（UE GameThread 寻址的对应物）；无进度中心数据源；关停无收尾协议。

## 目标

1. **`EditorJobSystem` 门面**（包装 runtime `JobScheduler/TaskPool`，编辑器**零自建线程池**）：

```rust
pub struct EditorJobSpec {
    pub label: String,
    pub category: JobCategory,        // Import/Compile/Thumbnail/Export/Index/Play/Misc
    pub priority: JobPriority,        // Interactive/Normal/Background（映射 thread_assignment）
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
3. **类别配额与互斥**：类别→并发上限表（Thumbnail≤2、Import≤worker_pool 宽度、Export=1…，设置化 17）；`MutexGroup` 内串行（`schedule_after` 链式实现）；`Interactive` 优先级经 `thread_assignment` 既有策略映射，防后台风暴挤压帧循环（与 runtime/03 帧预算口径对齐）。
4. **散点收编**：`ExportWizardJobController` 迁为 `EditorJobSystem` 首个客户（其取消/事件/快照协议即门面协议的验证原型，迁移后删除手工线程）；09 导入、10 registry 扫描、13 编译、缩略图、04 Play 子进程监视全部经门面（Play 监视为 `JobCategory::Play` 的长驻 job）。
5. **进度中心与收尾**：活跃 job 数据源 `{label, category, progress: Option<(u32,String)>, cancellable}`（状态栏/任务面板消费）；关停协议：`shutdown(deadline)` → 停收新 job → 广播取消 → 等待至 deadline → 记录未竟 job 清单（17 崩溃恢复衔接）。

## 非目标

- 不改 runtime tasks 内核（类别/配额是编辑器层；内核需求走 runtime/03 提案）；不引 async 运行时（`TaskPool::spawn` 同步任务模型够用，`install/join` 保留给数据并行场景）；进程外任务的进程管理（04/15 自持 Child，仅其**监视**入门面）。

## 架构设计

### 模块布局

```
zircon_editor/src/core/jobs/
  mod.rs
  system.rs        # EditorJobSystem：submit/配额/互斥/关停
  spec.rs          # EditorJobSpec/JobCategory/JobPriority/CancellationToken
  ticket.rs        # JobTicket 推拉双态
  pump.rs          # bus 回流泵（JobEvent 折算）
  progress.rs      # 进度中心数据源
```

`EditorContext`（01）持 `jobs: EditorJobSystem` 服务位。

### 与 runtime 内核的映射

| 门面概念 | runtime 既有物 | 映射方式 |
| --- | --- | --- |
| submit | `JobScheduler::schedule` | 直通 |
| after 依赖 | `schedule_after(dependencies, task)` | 直通（既有能力，零新建） |
| MutexGroup | 同上 | 组内前 job 的 handle 作后 job 依赖（链式） |
| 类别配额 | 无 | 门面层许可计数（信号量语义），满则排队 |
| 优先级 | `thread_assignment` 策略 | descriptor 映射，会签确认现策略枚举 |
| 取消 | 无内核支持 | `CancellationToken`（AtomicBool）+ job 内检查点协作式取消 |

### 深度测试

夹具 job 族（可编程时长/失败点/取消检查密度/进度序列）覆盖：并发上限（提交 N>上限 断言在途≤上限）、互斥组串行序、after 依赖序、取消及时性（检查点粒度内）、失败传播、关停 deadline 三路径（全完/取消/超时记录）——全部不依赖真实业务 job。

## 里程碑

### M1 门面与回流泵

- 切片 1.1：`core/jobs/` 五文件；submit/配额/互斥/after 映射 `JobScheduler`；`CancellationToken` 协议。
- 切片 1.2：`pump.rs` 回流泵接 01 bus（`JobEvent` 消息族）；`JobTicket` 推拉双态；Send 约束的类型层验证（编译失败测试：trybuild 或 doc-test 断言非 Send 捕获不过编译）。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`（夹具矩阵全绿）+ `cargo test -p zircon_runtime --lib --locked`（tasks 内核消费不回归）。更新 `docs/zircon_editor/core/jobs.md`。

### M2 散点收编

- 切片 2.1：`ExportWizardJobController` 迁门面（协议对齐：`ExportWizardJobEvent`→进度序列、`request_cancel`→token、`finish`→ticket），删除 `controller.rs:49` 手工 spawn；导出既有测试迁移。
- 切片 2.2：`std::thread::spawn` 守卫测试（zircon_editor 全 crate Grep 断言零命中，白名单空——当前仅 1 处即将删除，守卫防复发）。
- 测试阶段：导出向导既有流程测试全过 + 守卫测试落地；手验导出向导取消/进度 UI 无回归。

### M3 进度中心与收尾协议

- 切片 3.1：`progress.rs` 数据源 + 状态栏/任务面板接线（面板外观 editor_layout）。
- 切片 3.2：`shutdown(deadline)` 协议 + 未竟清单；类别配额设置化（17）。
- 切片 3.3：后台风暴压测夹具：1000 缩略图 job 下主循环帧时基线（与 runtime/03 预算口径对账）。
- 测试阶段：收尾三路径矩阵；压测基线记状态节；证据记状态节。

## 风险与开放问题

- 与 runtime 帧调度共享 `TaskPool` 的干扰：若压测显示后台类别侵蚀帧预算，为 Background 优先级申请独立 `TaskPoolDescriptor{ kind }` 池（pools.rs 多池机制既有，属配置非内核改动）——证据驱动，决策记状态节。
- 协作式取消对不可中断步骤（外部进程等待、单次大文件读）的语义：取消=尽力 + 结果丢弃 + job 标记 `CancelledLate`，契约文档明示。
- `thread_assignment` 现策略枚举与三级优先级的映射需 runtime tasks owner 会签（M1 前置确认项）。
