---
related_code:
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/core/runtime/tasks/timer.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system/inventory.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system/source_helpers.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system/split_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_worker_policy.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_worker_policy/worker_pool.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_worker_policy/split_layout.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_markdown.py
  - tools/tests/test_runtime_job_system_audit.py
  - tests/acceptance/runtime-job-system-audit-owner-sync.md
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/Task.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/Pipe.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/TaskConcurrencyLimiter.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/Async.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/LocalWorkQueue.h
  - dev/bevy/crates/bevy_tasks/src
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
status: in_progress
last_refined: 2026-08-01
---

# 11 多线程 JobSystem 任务模型与调度

2026-07-28 Runtime 11 当前 guard-owner 同步：`job_system_boundary` 报告 `expected_module_count = 10`、`expected_guard_file_count = 2`、`missing_guard_files = []`、`direct_rayon_paths = 2`、`schedule_parallel_executor_direct_rayon = []`、`diagnostic_anchor_count = 11`、`behavior_test_anchor_count = 27`、`missing_behavior_test_anchors = []`、`oversized_modules = []`、`mirror_docs_guard_present = true` 与 `risks = []`。新增 `tasks/timer.rs` 是进程级、容量受限的一次性 deadline 服务，供 Runtime11 生命周期维护复用，不能由 asset worker 私建维护线程。2 个 guard owner 为 route parent `job_system.rs` 与真实 folder-backed owner `job_system/mirror_docs.rs`；`runtime_11_job_system_mirror_docs_match_structure_audit_counts` 保持计划、runtime index、JobSystem 模块文档、M0 review 与 interface convergence 一致。当前诊断面已覆盖 dependency waiting、ready queue、active、queue wait、explicit wait、panic、cancellation 与通用 terminal observer；重叠 writer 以 acquire/release 终态 handoff 建立完整可见性，continuation unwind 逐项 containment 后仍释放后续 barrier callback 与 observer。named `tasks/ecs_schedule/worker_pool/rayon` filters 的最终 current-source 证据仍按编号记录推进。

把 runtime 的并行执行底座从"三池 + 三原语 + 多处旁路"升级为带**依赖图、句柄、同步点、数据并行原语**的统一 JobSystem——任务模型对照 Unity C# Job System（JobHandle / 依赖链 / Complete 同步点 / IJobParallelFor），调度实现对照 UE5 Tasks System（`Tasks::FTask` 前置依赖、`FPipe` 串行管道、`FTaskConcurrencyLimiter`、worker 本地队列 + 窃取）。**证据优先原则继承 07：每一步结构升级必须有消费方需求或计数证据，不做投机调度器**。

## 现状与证据（2026-06-12 实仓盘点）

- **三池底座已对齐 bevy_tasks**：`core/runtime/tasks/pools.rs` 的 `TaskPools { compute, async_compute, io }`（:17，访问器 :61/:65/:69）+ `TaskPoolThreadCounts`（:9）+ `from_options_with_available_parallelism`（:29）；线程配额经 `thread_assignment.rs` 的 `TaskPoolThreadAssignmentPolicy`（:2，`thread_count(remaining, total)` :9）与 `TaskPoolOptions`（:25）按比例/上下限切分——Bevy `TaskPoolOptions` 同形。`report.rs` 已有池报告（`TaskPoolReport`）。
- **执行原语过薄**：`job_scheduler.rs`（53 行）是 compute 池的薄 facade，仅三原语——`spawn`（fire-and-forget，:31）、`install`（阻塞执行，:35）、`join`（二路 fork-join，:39）。**没有**：任务句柄（无法等待单个任务完成）、任务间依赖声明、批量数据并行（ParallelFor）、取消、优先级。`TaskPool` 本体 rayon 背书（pool.rs）。
- **旁路实测（统一底座的反证）**：
  1. `graphics/visibility/culling/parallel_frustum.rs` 直接 `use rayon`——绕过 JobScheduler/TaskPools 做剔除并行；
  2. `asset/pipeline/worker_pool.rs` 自建线程（`spawn_named_thread`，`zircon-asset-{i}`）——IO 解码不走 `TaskPools::io`（04 计划 M2 改造 options 时是接入窗口）；
  3. `scene/ecs/schedule_parallel_executor.rs` 经 JobScheduler 跑批次（合规），但 batch 间无依赖表达——同 stage 串行批次靠顺序执行而非依赖图。
- **ECS 集成现状**（03-M3 盘点继承）：conflict graph 产出保守并行批次，executor 有失败上报与批次顺序语义（`schedule_parallel_executor_reports_task_failure_by_batch_order` 等 11 测试）；03-M3 将加开关与诊断计数。
- 参考锚点（每点一行）：
  - UE5 Tasks：`FTask` 前置依赖（Prerequisites）+ 嵌套任务 — `dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/Task.h`
  - UE `FPipe` 串行管道（同管道任务串行、跨管道并行）— `dev/.../Tasks/Pipe.h`
  - UE `FTaskConcurrencyLimiter`（并发上限闸）— `dev/.../Tasks/TaskConcurrencyLimiter.h`
  - UE worker 本地队列/窃取 — `dev/.../Async/LocalWorkQueue.h`；`Async.h`（EAsyncExecution 分层）
  - UE ParallelFor — `dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/ParallelFor.h`（2026-06-13 实测存在）
  - Bevy 三池 + scope API + 并行切片/迭代器 — `dev/bevy/crates/bevy_tasks/src/{task_pool.rs,usages.rs,slice.rs,iter/}`
  - Godot WorkerThreadPool（组任务/优先级/yield 的 C++ 工程实现，worker 内等待的死锁规避对照——M1.1 必读）— `dev/godot/core/object/worker_thread_pool.{h,cpp}`
  - Unity C# Job System（源码不在 dev/，按公开语义对照）：`JobHandle` + `JobHandle.CombineDependencies` 依赖链、`Schedule(dependsOn)`、主线程 `Complete()` 同步点、`IJobParallelFor` 批量切分、安全系统禁止 worker 内 Schedule——作为任务模型语义锚而非实现锚。

## 目标

1. **任务模型定稿**：`JobHandle`（可等待/可组合）+ 依赖声明（schedule-with-deps）+ 主线程同步点（complete/wait_all）+ 数据并行原语（parallel_for over 切片/范围）——四件套语义文档化并落 API。
2. **执行旁路清零**：rayon 直连（graphics 剔除）与自建线程（asset worker）收编到统一底座或显式白名单（带理由），CPU 配额单点治理（三池 + rayon 全局池的线程数不再各自为政）。
3. **ECS 调度消费升级**：ScheduleParallelExecutor 的批次经依赖图表达（batch N+1 depends-on batch N），为跨 stage 重叠（03 未来项）留好模型位。
4. 可观测：任务计数/等待耗时/窃取统计走 `core::diagnostics`（与 03-M3、07-M1 同通道）。

## 非目标

- 不引入新依赖（async runtime/tokio、第三方 job 库）；底座仍是 rayon + 自研层（"不新增 crate"硬约束下 rayon 已在树内）。
- 不做 GPU/渲染线程模型（render 计划与 RHI 会话地盘）；不动 `AssetWorkerPool` 的去重/背压语义（04-M2 地盘，本计划只管它的线程来源）。
- 不投机实现 work stealing 自研队列——rayon 已自带窃取；自研仅当 M0 证据表明 rayon 语义不满足（如同步点饥饿）。
- 不做 Unity 式安全系统（borrow 检查由 Rust 类型系统承担，无需运行时安全层）。

### 全局硬约束（继承总计划 §4，违反即返工）

- 不新增 crate；硬切换不留兼容层；渲染骨架归 render 计划 01-08；非网络语义 server 命名是 blocker（"JobSystem/scheduler/pool/pipe" 词汇合法）。

## 执行前检查清单

1. 前置依赖确认：02-M2 已完成（tasks 族落位 `core/runtime/tasks/`，2026-06-12 实测）；03-M3（executor 开关与计数）排期对齐——本计划 M2 与其同文件（`schedule_parallel_executor.rs`），错峰执行。
2. 活动会话对齐：`git status --porcelain -- zircon_runtime/src/core/runtime/tasks/ zircon_runtime/src/scene/ecs/ zircon_runtime/src/graphics/visibility/`；10fps 会话改动禁止回退。
3. 事实重核：
   - `ls zircon_runtime/src/core/runtime/tasks/`（核 5 文件清单）
   - `grep -rln "use rayon\|rayon::" zircon_runtime/src --include=*.rs`（旁路基线，2026-06-12 为 4 文件）
   - `grep -n "pub fn" zircon_runtime/src/core/runtime/tasks/job_scheduler.rs`（核三原语仍是全部公共面）
4. 基线记录：`.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter tasks` 与 `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter ecs_schedule` 通过数记入状态节。

## 里程碑

### M0 任务模型设计定稿（先语义后实现）

#### 切片 0.1 消费方需求清单与模型选型

- 目标文件：`docs/zircon_runtime/core/job_system.md`（新建；挂 `docs/zircon_runtime/` 索引）。
- 改动形态：纯文档。两部分：
  - **消费方需求矩阵**（实测五消费方逐行）：ECS 并行批次（需要：批次间依赖、批内 fork-join、失败传播）、graphics 剔除 parallel_frustum（需要：parallel_for、每帧低开销）、asset 解码（需要：IO 池、长任务不占 compute、完成通知——已有 channel 形态）、animation/navigation 等模块系统（执行时盘点：Grep `JobScheduler|TaskPool`，path `zircon_runtime/src`，列实际用法）、未来物理（01-M3 决策后的 fixed-step 内并行，预留行）。
  - **模型选型判词**（对照表三列：Unity 语义 / UE 实现 / 本仓决策）：
    | 维度 | Unity | UE5 Tasks | 本仓决策（候选，M0 定稿） |
    |---|---|---|---|
    | 句柄 | `JobHandle`（值类型，可组合） | `FTask`（引用计数，`Wait/IsCompleted`） | `JobHandle`（轻量克隆，内部 Arc 完成态） |
    | 依赖 | `Schedule(dependsOn)` + `CombineDependencies` | Prerequisites 数组 + `Launch(..., Prerequisites(...))` | `schedule_after(&[JobHandle])` 形态 |
    | 同步点 | 主线程 `Complete()`（强制求值点） | `Wait()`/`BusyWait()` | `JobHandle::wait()` + 帧末 `wait_all` 闸（挂接 03 帧循环的位置写明） |
    | 串行域 | 无（靠依赖链表达） | `FPipe`（命名串行管道） | 是否需要 Pipe 由消费方矩阵裁决（asset 顺序解码是候选用户） |
    | 数据并行 | `IJobParallelFor`（批量切分 + 窃取） | `ParallelFor`（分块 + 负载均衡） | `parallel_for(range, chunk, fn)` 包 rayon `par_chunks`，剔除/ECS 迭代消费 |
    | 并发上限 | 无显式 | `FTaskConcurrencyLimiter` | 仅当 04 背压证据需要时加 |
- 调用方迁移：无。
- 验收：需求矩阵每行有"需要的原语"列；选型表无"待定"，每行判词带消费方依据。
- DoD：`job_system.md` 落地；不被任何消费方需要的原语显式标注"不实现（YAGNI）"。

#### 切片 0.2 线程预算单点治理方案

- 目标文件：同 0.1 文档（"线程预算"节）。
- 改动形态：决策记录——现状三方分头拿线程：`TaskPools` 按 `TaskPoolOptions` 切分、rayon 全局池默认 = 逻辑核数（parallel_frustum 直连即用它）、`AssetWorkerPool(default_worker_count)` 自建。定稿单点：`TaskPoolOptions` 为唯一预算 owner，rayon 全局池线程数由其显式初始化（或全部 rayon 使用走 `TaskPool::install` 进指定池），asset worker 线程计入 io 池配额（与 04-M2 的 `AssetWorkerPoolOptions` 对齐：io 池借线程 vs 仅记账，二选一判词）。
- 验收：预算流向图（谁声明/谁消费/谁记账）+ 判词。
- DoD：方案与 04-M2、03-M3 的参数 owner 口径互引一致。

#### M0 测试阶段（milestone-first）

- 纯审计：`git status --porcelain` 仅 docs 变更。

### M1 JobHandle 与依赖调度落地

#### 切片 1.1 句柄与完成态

- 目标文件：`core/runtime/tasks/`（新文件 `job_handle.rs`（新建）+ `job_scheduler.rs` 扩展；`mod.rs` 加声明）。
- 改动形态（签名草案，执行时定稿）：

  ```rust
  pub struct JobHandle { /* Arc<JobState>：完成标志 + 等待原语（Condvar 或 rayon yield 循环，按 M0 判词） */ }
  impl JobHandle {
      pub fn is_complete(&self) -> bool;
      pub fn wait(&self);                      // 主线程同步点；worker 内调用的语义按 M0 判词（禁止或 work-assist）
      pub fn combine(handles: &[JobHandle]) -> JobHandle;   // 对照 Unity CombineDependencies
  }
  impl JobScheduler {
      pub fn schedule(&self, task: impl FnOnce() + Send + 'static) -> JobHandle;
      pub fn schedule_after(&self, deps: &[JobHandle], task: impl FnOnce() + Send + 'static) -> JobHandle;
  }
  ```

  既有三原语保留（spawn 即"不要句柄的 schedule"，文档标注分工）；worker 内 `wait()` 的死锁规避策略必须在实现前定稿（候选：worker 内 wait 转 work-assist——对照 UE `BusyWait`，或 debug 断言禁止——对照 Unity 安全规则）。
- 调用方迁移：无强制（新增 API；既有 spawn/install/join 调用方不动）。
- 验收（测试名草案，归属 `core/runtime/tasks/` 同级测试树或 `tests/tasks.rs` 既有位）：
  - `job_handle_wait_blocks_until_task_completes`
  - `schedule_after_runs_task_only_after_all_dependencies`
  - `combined_handle_completes_when_all_children_complete`
  - `combined_handle_waits_for_all_children_before_propagating_panic`
  - `worker_thread_wait_does_not_deadlock_scheduler`（按死锁策略定稿改名/改断言）
- DoD：五测试绿；`job_system.md` API 节与实现一致。

#### 切片 1.2 parallel_for 原语

- 目标文件：`core/runtime/tasks/`（新文件 `parallel_for.rs`（新建））。
- 改动形态（签名草案）：`pub fn parallel_for<T: Send>(pool: &TaskPool, items: &mut [T], chunk: usize, f: impl Fn(&mut [T]) + Send + Sync)`（rayon `par_chunks_mut` 包装，chunk 语义对照 Unity `innerloopBatchCount`/UE ParallelFor 分块）；返回形态（阻塞 vs JobHandle）按 M0 消费方矩阵定。
- 调用方迁移：无强制（M2 收编旁路时迁移）。
- 验收：`parallel_for_visits_every_item_exactly_once`、`parallel_for_chunk_size_bounds_task_granularity`。
- DoD：原语测试绿且文档含"何时用 parallel_for vs schedule"判据。

#### 切片 1.3 通用终态观察器

- 目标文件：`core/runtime/tasks/job_handle.rs`、`src/tests/tasks.rs`、Runtime 11 source inventory 与本计划/模块文档。
- 改动形态：新增 `JobHandle::on_terminal(...)` one-shot observer；注册可发生在终态前后，多个 observer 各执行一次。终态转换在锁内只发布状态并取走队列，先保持既有 dependency continuation 次序，再在锁外运行 observer。observer panic 被 containment，计入 handle-local `terminal_observer_panic_count()`，不得改写任务 panic、依赖取消或 scheduler-wide diagnostics。
- 边界：Runtime 11 不引入 winit、dynamic API、application cadence policy 或 scheduler-wide wake；具体 frame-visible consumer 由后续 Runtime 10/03 owner 选择性绑定。
- 验收：before/after terminal、exactly once、multiple observers、panic containment、dependency continuation、reentrant handle access，以及 dependency continuation unwind 仍投递 observer 的七组 focused tests。
- DoD：focused Cargo 与 Runtime 11 diagnostics parity 通过、独立 review `Critical 0 / Important 0` 后方可接受；静态实现不能替代 Cargo 证据。

#### M1 测试阶段（milestone-first）

- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipTest`（切片期）
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter tasks`
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter job`
- 验收证据：句柄/依赖/数据并行测试族 + `job_system.md` 定稿。

### M2 旁路收编与 ECS 集成升级

#### 切片 2.1 graphics 剔除旁路收编

- 目标文件：`graphics/visibility/culling/parallel_frustum.rs`（rayon 直连 → `parallel_for` 或 compute 池 `install`）。
- 改动形态：剔除并行改走统一原语；行为零变化（输出一致性测试先行锁定）。**前置**：与 render 计划/10fps 会话确认该文件无在飞改动。
- 调用方迁移：仅该文件内部。
- 验收：`parallel_frustum_culling_matches_serial_reference_output`（一致性锚，改造前先落）；改造后 Grep `use rayon` 该文件 0 命中。
- DoD：旁路基线 4 文件 → 3（executor 与 pool.rs 是合法底座使用，结构测试白名单化）。

#### 切片 2.2 rayon 使用面结构守卫

- 目标文件：`zircon_runtime/src/tests/runtime_absorption/`（新守卫，复用 05/02 已落的源扫描 helper——公约 §7.8）。
- 改动形态：`rayon_is_only_reachable_through_core_task_primitives`——断言 `use rayon` 仅出现在白名单（`core/runtime/tasks/pool.rs`、`parallel_for.rs`）；负例自检。
- 调用方迁移：无。
- 验收：守卫 + 负例。
- DoD：`.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter rayon` 绿（过滤词按测试名定）。

#### 切片 2.3 ECS 批次依赖化

- 目标文件：`scene/ecs/schedule_parallel_executor.rs`（与 03-M3 错峰；若 03-M3 已落开关/计数，在其上叠加）。
- 改动形态：批次提交从"顺序 await 每批"改为 `schedule_after` 链（batch N+1 deps=[batch N handle]），主线程在 stage 末 `wait()` 尾批句柄——执行语义不变（保守串行批次链），但模型位就绪，为未来跨 stage 重叠（03 backlog）与 fixed-step 内物理并行（01-M3 后）留接口。失败传播语义保持既有测试约束（`...reports_task_failure_by_batch_order`）。
- 调用方迁移：executor 内部；公共面不变。
- 验收：既有 11 个 conflict_graph/executor 测试无回归 + `executor_batches_are_chained_through_job_dependencies`（结构/行为锚）。
- DoD：`.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter ecs_schedule` 全绿。

#### 切片 2.4 asset worker 线程来源裁决执行（按 M0 0.2 判词）

- 目标文件：`asset/pipeline/worker_pool.rs`（仅线程来源段；去重/背压归 04-M2）。
- 改动形态：按 0.2 判词二选一——(a) 解码任务改投 `TaskPools::io`（worker_pool 退化为请求编排层）；(b) 保留自建线程但线程数经统一预算记账。与 04-M2 的 `AssetWorkerPoolOptions` 改造同切片窗口执行，避免两次动同一构造面。
- 调用方迁移：`AssetWorkerPool::new` 2 处（04 已实测全列）。
- 验收：(a) 路线：既有 worker_pool 测试族无回归 + io 池报告含 asset 任务计数；(b) 路线：预算记账测试。
- DoD：判词执行完毕，`asset.worker.*` 诊断（04-M2.3）与任务底座计数口径一致。

#### M2 测试阶段（milestone-first）

- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter ecs_schedule`
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter worker_pool`；`.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter tasks`
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests`（横切收编后全量）
- 验收证据：旁路清零（白名单守卫）+ 一致性锚测试 + 全量无回归。

### M3 可观测与压测验收

#### 切片 3.1 调度诊断计数

- 目标文件：`core/runtime/tasks/`（计数登记走 `core::diagnostics`；`report.rs` 扩展）。
- 改动形态：`tasks.scheduled` / `tasks.completed` 保留累计终态口径；新增当前 `tasks.dependency_waiting` / `tasks.queued` / `tasks.active`、累计 `tasks.queue_wait_ms` + `tasks.queue_wait_samples`、`tasks.panicked` / `tasks.cancelled`，依赖释放继续单列 `tasks.dependency_wait_ms`。四个 lifecycle gauge 守恒 `scheduled = completed + dependency_waiting + queued + active`。旧 `tasks.main_thread_wait_ms` 无法证明 caller identity，硬切为语义准确的 `tasks.explicit_wait_ms`，不保留别名或双计数；真正主线程 stall 由 WPR/帧 trace 与该显式同步指标关联判断。
- 验收：`job_diagnostics_track_schedule_complete_and_wait_times`、`task_diagnostics_track_ready_queue_active_and_queue_wait`、`task_diagnostics_queue_pressure_matrix_drains_without_gauge_leaks`（1/2/4 workers）、`worker_side_wait_is_reported_as_explicit_wait`、`task_diagnostics_distinguish_panics_from_dependency_cancellation`、`detached_spawn_counts_panicked_tasks_as_completed`。
- DoD：dependency-waiting/ready queue/active 为实时 gauge；enqueue-to-start 与 dependency wait 不混算；panic task 与未启动 dependent cancellation 分项准确；重叠 writer 通过 acquire/release retirement chain 发布同一稳定快照；全部 hotpath 只用原子计数，不增加每任务诊断锁。

#### 切片 3.2 行为压测锚

- 目标文件：`core/runtime/tasks/` 测试位（聚焦验收测试，非 benchmark——遵守"不引入 criterion"）。
- 改动形态：两类语义压测：依赖链深度 N（如 64）正确完成且无栈溢出/死锁；宽扇出（N 任务 combine 等待）正确聚合。`deep_dependency_chain_completes_in_order`、`wide_fanout_combine_waits_for_all`。
- 验收：两测试在 `--test-threads=1` 与默认并发下均稳定。
- DoD：进常驻测试树。

#### M3 测试阶段（milestone-first）

- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter tasks`
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests`（收尾全量）
- `runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass` 保持 `tasks/ecs_schedule/worker_pool/rayon` 验证闸门可见，直到上述过滤测试和 render-owned `parallel_frustum` cutover 均有证据。
- 验收证据：诊断计数 + 压测锚；`job_system.md` 增"可观测"节；07 的帧分解（M0.3）可引用 `tasks.explicit_wait_ms` 定位显式同步开销，并结合线程 trace 判断是否属于主线程。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前状态：`in_progress`。统一任务模型、source-cubemap direct-Rayon 收编与任务诊断切片仍在 managed acceptance 链上；最高优先级仍是有界偏好持久化 failure，其 current-source 二审、受管 lock/编译/测试 receipt 与 fixed return 未完成，不得标记 accepted。

- 迁入记录：[既有 2026-07-09 产出记录](11/2026-07-09-job-system-task-model-output-records.md)
- 迁入产出记录：[2026-08-01 产出与性能交接归档](11/2026-08-01-plan-output-and-performance-handoffs.md)
- coordinator-open failure：[editor-full-harness-runtime-thread-budget](11/failure-2026-07-13-editor-full-harness-runtime-thread-budget.md)
- coordinator-open failure：[task-diagnostics-accuracy](11/failure-2026-07-17-task-diagnostics-accuracy.md)
- coordinator-open failure：[operation-service-synchronous-unbounded](11/failure-2026-07-19-operation-service-synchronous-unbounded.md)
- coordinator-open failure：[asset-worker-shared-completion-backpressure](11/failure-2026-07-22-asset-worker-shared-completion-backpressure.md)
- coordinator-open failure：[dynamic-scene-session-bounded-async-io](11/failure-2026-07-22-dynamic-scene-session-bounded-async-io.md)
- coordinator-open failure：[native-plugin-discovery-bounded-refresh-publication](11/failure-2026-07-27-native-plugin-discovery-bounded-refresh-publication.md)
- coordinator-open failure：[dynamic-runtime-animation-module-duplication](11/failure-2026-07-29-dynamic-runtime-animation-module-duplication.md)
- coordinator-open failure：[preference-storage-bounded-persistence-lane](11/failure-2026-07-31-preference-storage-bounded-persistence-lane.md)
