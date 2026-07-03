---
related_code:
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_markdown.py
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/Task.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/Pipe.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/TaskConcurrencyLimiter.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/Async.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/LocalWorkQueue.h
  - dev/bevy/crates/bevy_tasks/src
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
status: in_progress
last_refined: 2026-07-01
---

# 11 多线程 JobSystem 任务模型与调度

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
4. 基线记录：`cargo test -p zircon_runtime --lib tasks --locked` 与 `--lib ecs_schedule --locked` 通过数记入状态节。

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
  - `worker_thread_wait_does_not_deadlock_scheduler`（按死锁策略定稿改名/改断言）
- DoD：四测试绿；`job_system.md` API 节与实现一致。

#### 切片 1.2 parallel_for 原语

- 目标文件：`core/runtime/tasks/`（新文件 `parallel_for.rs`（新建））。
- 改动形态（签名草案）：`pub fn parallel_for<T: Send>(pool: &TaskPool, items: &mut [T], chunk: usize, f: impl Fn(&mut [T]) + Send + Sync)`（rayon `par_chunks_mut` 包装，chunk 语义对照 Unity `innerloopBatchCount`/UE ParallelFor 分块）；返回形态（阻塞 vs JobHandle）按 M0 消费方矩阵定。
- 调用方迁移：无强制（M2 收编旁路时迁移）。
- 验收：`parallel_for_visits_every_item_exactly_once`、`parallel_for_chunk_size_bounds_task_granularity`。
- DoD：原语测试绿且文档含"何时用 parallel_for vs schedule"判据。

#### M1 测试阶段（milestone-first）

- `cargo check -p zircon_runtime --lib --locked`（切片期）
- `cargo test -p zircon_runtime --lib tasks --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib job --locked -- --nocapture`
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
- DoD：`cargo test -p zircon_runtime --lib rayon --locked` 绿（过滤词按测试名定）。

#### 切片 2.3 ECS 批次依赖化

- 目标文件：`scene/ecs/schedule_parallel_executor.rs`（与 03-M3 错峰；若 03-M3 已落开关/计数，在其上叠加）。
- 改动形态：批次提交从"顺序 await 每批"改为 `schedule_after` 链（batch N+1 deps=[batch N handle]），主线程在 stage 末 `wait()` 尾批句柄——执行语义不变（保守串行批次链），但模型位就绪，为未来跨 stage 重叠（03 backlog）与 fixed-step 内物理并行（01-M3 后）留接口。失败传播语义保持既有测试约束（`...reports_task_failure_by_batch_order`）。
- 调用方迁移：executor 内部；公共面不变。
- 验收：既有 11 个 conflict_graph/executor 测试无回归 + `executor_batches_are_chained_through_job_dependencies`（结构/行为锚）。
- DoD：`cargo test -p zircon_runtime --lib ecs_schedule --locked` 全绿。

#### 切片 2.4 asset worker 线程来源裁决执行（按 M0 0.2 判词）

- 目标文件：`asset/pipeline/worker_pool.rs`（仅线程来源段；去重/背压归 04-M2）。
- 改动形态：按 0.2 判词二选一——(a) 解码任务改投 `TaskPools::io`（worker_pool 退化为请求编排层）；(b) 保留自建线程但线程数经统一预算记账。与 04-M2 的 `AssetWorkerPoolOptions` 改造同切片窗口执行，避免两次动同一构造面。
- 调用方迁移：`AssetWorkerPool::new` 2 处（04 已实测全列）。
- 验收：(a) 路线：既有 worker_pool 测试族无回归 + io 池报告含 asset 任务计数；(b) 路线：预算记账测试。
- DoD：判词执行完毕，`asset.worker.*` 诊断（04-M2.3）与任务底座计数口径一致。

#### M2 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib ecs_schedule --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib worker_pool --locked`；`cargo test -p zircon_runtime --lib tasks --locked`
- `cargo test -p zircon_runtime --lib --locked`（横切收编后全量）
- 验收证据：旁路清零（白名单守卫）+ 一致性锚测试 + 全量无回归。

### M3 可观测与压测验收

#### 切片 3.1 调度诊断计数

- 目标文件：`core/runtime/tasks/`（计数登记走 `core::diagnostics`；`report.rs` 扩展）。
- 改动形态：计数项（草案）：`tasks.scheduled`、`tasks.completed`、`tasks.dependency_wait_ms`（句柄等待累计）、`tasks.main_thread_wait_ms`（同步点阻塞——Unity 主线程 stall 的等价观测）；与 03-M3 的 `schedule.parallel_batches` 同 snapshot 可读。
- 验收：`job_diagnostics_track_schedule_complete_and_wait_times`。
- DoD：四计数在 vampire 场景非零且稳定（07-M0 基线采集时一并记录）。

#### 切片 3.2 行为压测锚

- 目标文件：`core/runtime/tasks/` 测试位（聚焦验收测试，非 benchmark——遵守"不引入 criterion"）。
- 改动形态：两类语义压测：依赖链深度 N（如 64）正确完成且无栈溢出/死锁；宽扇出（N 任务 combine 等待）正确聚合。`deep_dependency_chain_completes_in_order`、`wide_fanout_combine_waits_for_all`。
- 验收：两测试在 `--test-threads=1` 与默认并发下均稳定。
- DoD：进常驻测试树。

#### M3 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib tasks --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib --locked`（收尾全量）
- `runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass` 保持 `tasks/ecs_schedule/worker_pool/rayon` 验证闸门可见，直到上述过滤测试和 render-owned `parallel_frustum` cutover 均有证据。
- 验收证据：诊断计数 + 压测锚；`job_system.md` 增"可观测"节；07 的帧分解（M0.3）可引用 `tasks.main_thread_wait_ms` 定位同步点开销。

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M0 | 0.1 模型选型 | completed_static_passed | 2026-06-13 | `docs/zircon_runtime/core/job_system.md` 消费方矩阵 + Unity/UE/Bevy/Godot 判词；已读锚点:`Task.h`、`Pipe.h`、`TaskConcurrencyLimiter.h`、`ParallelFor.h`、`bevy_tasks/{task_pool.rs,usages.rs,slice.rs}`、`godot/core/object/worker_thread_pool.{h,cpp}` |
| M0 | 0.2 线程预算单点 | completed_static_passed | 2026-06-13 | `docs/zircon_runtime/core/job_system.md` 线程预算节：`TaskPoolOptions` 为唯一预算 owner；direct rayon 收敛到 tasks 原语；asset worker 后续已在 11-M2.4 按自建线程显式记账路线执行 |
| M1 | 1.1 句柄与依赖 | code_static_pending_cargo | 2026-06-13 | 新增 `core/runtime/tasks/job_handle.rs`；`JobScheduler::schedule` / `schedule_after`；测试锚:`job_handle_wait_blocks_until_task_completes`、`schedule_after_runs_task_only_after_all_dependencies`、`combined_handle_completes_when_all_children_complete`、`schedule_after_does_not_consume_worker_while_waiting_on_dependencies`。`rustfmt --edition 2021 --check` 通过；anchor scan、冲突标记与尾随空白扫描为空；`git diff --check` 仅 index/runtime task docs/job_scheduler/mod.rs/tests LF/CRLF 提示。2026-06-13 `cargo test -p zircon_runtime --lib tasks --locked -- --nocapture` 未进入 tasks 测试，先被 plugin native-loader 测试漏导入 `PluginInterfaceManifest` 阻断；漏导入已修，重跑待 cargo/rustc 通道空闲。 |
| M1 | 1.3 scheduler wait_all 同步点 | wait_all_static_passed_cargo_pending | 2026-06-17 | `JobScheduler::wait_all(&[JobHandle])` 已作为调度器拥有的多句柄主线程同步点落地，内部通过 `JobHandle::combine_with_scheduler_diagnostics(...)` 复用既有完成回调/组合句柄语义，并把显式等待时间记入当前 scheduler 的 `tasks.main_thread_wait_ms`；新增行为锚 `scheduler_wait_all_waits_for_all_handles_and_records_sync_time`。同步 `docs/zircon_runtime/core/{job_system.md,tasks.md,runtime/tasks.md}`、`job_system_boundary.py`、`runtime_absorption::job_system::runtime_11_job_system_mirror_docs_match_structure_audit_counts` 与 Runtime 11 状态表，当前 `behavior_test_anchor_count = 12`、`missing_behavior_test_anchors = []`。验证：rustfmt check、Python py_compile、direct `job_system_boundary_audit` (`behavior_test_anchor_count = 12`, `risks = []`)、standalone job_system 1/1、standalone plan_status 32/32 通过；Cargo `tasks/ecs_schedule/worker_pool/rayon` gates 仍 pending。 |
| M1 | 1.4 panic-safe handle completion | panic_safe_completion_static_passed_cargo_deferred | 2026-06-20 | 状态锚 `panic_safe_completion_static_passed_cargo_deferred`；`JobHandle` 现在保存 panic terminal state，`JobScheduler::schedule` / `schedule_after` 在任务闭包周围使用 `catch_unwind`，保证任务 panic 时仍 `record_completed()`、标记 handle 终止并唤醒等待方；`JobHandle::wait()` 在调用线程报告 `job task panicked: ...`，`schedule_after` 与 `JobHandle::combine` 会把依赖 panic 传播到返回 handle 且不运行 dependent task。新增行为锚 `job_handle_wait_reports_task_panic_without_leaking_completion` 与 `schedule_after_propagates_dependency_panic_without_running_dependent_task`；`job_system_boundary` 当前同步 `behavior_test_anchor_count = 12`、`missing_behavior_test_anchors = []`。验证：rustfmt/static/standalone guard 见本轮状态；Cargo `tasks/ecs_schedule/worker_pool/rayon` gates 仍按“先实现功能”方向 deferred。 |
| M1 | 1.2 parallel_for | code_static_pending_cargo | 2026-06-13 | 新增 `core/runtime/tasks/parallel_for.rs` 与 public re-export；测试锚:`parallel_for_visits_every_item_exactly_once`、`parallel_for_chunk_size_bounds_task_granularity`。`rustfmt --edition 2021 --check` 通过；anchor scan、冲突标记与尾随空白扫描为空；`git diff --check` 仅 index/runtime task docs/job_scheduler/mod.rs/tests LF/CRLF 提示。2026-06-13 `cargo test -p zircon_runtime --lib tasks --locked -- --nocapture` 未进入 tasks 测试，先被 plugin native-loader 测试漏导入 `PluginInterfaceManifest` 阻断；漏导入已修，重跑待 cargo/rustc 通道空闲。 |
| M2 | 2.1 剔除旁路收编 | pre_m2_1_rayon_render_exception_guard_static_passed_pending_render_owner | 2026-06-13 | 扩展 `runtime_absorption::rayon_boundary`，在不编辑 render-owned `graphics/visibility/culling/parallel_frustum.rs` 的前提下先锁定当前 production direct-rayon 分布：只允许 `core/runtime/tasks/{pool,parallel_for}.rs` 两个 task primitive owner 与 `parallel_frustum.rs` 这个 `render-owner-pending-runtime-11-m2-1-cutover` 例外。actual graphics cutover not executed；真正把 `parallel_frustum.rs` 改走 `parallel_for` / compute pool `install` 仍等待 render owner 窗口。本轮静态验证通过：direct-rayon production source scan 仅命中 `core/runtime/tasks/parallel_for.rs`、`core/runtime/tasks/pool.rs`、`graphics/visibility/culling/parallel_frustum.rs`；`rustfmt --edition 2021 --check` 通过；conflict/trailing/anchor scans 通过；scoped `git diff --check` 仅 LF-to-CRLF warnings。standalone rustc 与 Cargo 未启动，因为 active cargo/rustc lanes 仍在运行。 |
| M2 | 2.1 剔除旁路收编 | runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending | 2026-06-16 | `graphics/visibility/culling/parallel_frustum.rs` 已删除 direct rayon，改为构建工作项后经 `core::runtime::parallel_for(...)` 在 runtime compute `TaskPool` 上执行；`WgpuRenderFramework` 新增 `compute_task_pool`，runtime graphics module construction 传入 `core.task_pools().compute().clone()`，`build_frame_submission_context` 通过 `VisibilityContext::from_extract_with_history_static_index_and_task_pool(...)` 把池传到主视图与 shadow view culling。`runtime_absorption::rayon_boundary` / `job_system_boundary` 当前锁定 `direct_rayon_paths = 2`，只剩 `core/runtime/tasks/{pool,parallel_for}.rs` 两个 task primitive owner；`parallel_frustum.rs` cutover source guard 锚为 `runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending`，Cargo gate 仍待 `tasks/ecs_schedule/worker_pool/rayon` 干净窗口。 |
| M2 | 2.2 rayon 守卫 | code_static_pending_cargo | 2026-06-13 | 非 graphics 侧已执行：`ScheduleParallelExecutor` 去掉 `use rayon` / `rayon::join` / `into_par_iter`，固定 2-6 系统批次改走 `JobScheduler::join(...)`，泛化批次改走 balanced `run_parallel_tasks(...)`；结构锚 `schedule_parallel_executor_does_not_call_rayon_directly` 已补。`runtime_absorption::rayon_boundary` 生产源码守卫现在只允许 `core/runtime/tasks/{pool,parallel_for}.rs` 两个 task primitive owner，当前 `direct_rayon_paths = 2`，负例 `rayon_boundary_guard_rejects_unclassified_runtime_source` 固定 `schedule_parallel_executor.rs` 与 `graphics/visibility/culling/parallel_frustum.rs` 均不可作为例外。2026-06-13 07:05 +08 尝试 `cargo test -p zircon_runtime --lib rayon --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-11-validation-0613 --message-format short --color never -- --nocapture`，20 分钟超时且未返回测试结果；本轮孤立的 rayon cargo/rustc 进程已停止。M2.2 Cargo 仍待干净编译窗口重跑。 |
| M2 | 2.3 ECS 批次依赖化 | code_static_pending_cargo | 2026-06-13 | `ScheduleParallelExecutor::run_batches_with_report(...)` 已改为用 `JobHandle::completed()` 起点和 `JobScheduler::schedule_after(...)` 逐批串接，主线程只等待尾批 handle；`ScheduleParallelTaskRegistry` 内部改为 `Arc` 任务，允许批次闭包提交到 scheduler；失败批次设置 abort flag，后续批次 no-op 完成并按原批次顺序回放首个错误。M2.2 追加收编后，批次内部并行也改为 `JobScheduler::join(...)` / `run_parallel_tasks(...)`，不再 direct rayon。新增行为锚 `executor_batches_are_chained_through_job_dependencies`，新增结构锚 `schedule_parallel_batches_chain_through_job_handles` 与 `schedule_parallel_executor_does_not_call_rayon_directly`，并同步 `docs/zircon_runtime/scene/ecs/schedule_parallel_executor.md` 与 `docs/zircon_runtime/core/job_system.md`。`rustfmt --edition 2021 --check` 通过；锚点扫描、冲突标记扫描、尾随空白扫描通过；scoped `git diff --check` 仅 LF/CRLF 提示。Cargo 待 active lanes 清空后运行 `ecs_schedule` 与 Runtime 11 M2 测试阶段 |
| M2 | 2.4 asset 线程裁决 | code_static_pending_cargo | 2026-06-13 | 按 M0.2 路线 (b) 执行：`AssetWorkerPoolOptions::from_task_pool_options(...)` 从 `TaskPoolOptions::resolve_thread_counts(...).io_threads` 推导 worker 数；新增 `AssetWorkerThreadBudgetSource::{Explicit,TaskPoolIo}`；`ProjectAssetManager::default()` 与 asset module factory 走 runtime IO 预算，`ProjectAssetManager::new(count)` 保持显式 override；`AssetWorkerPoolDiagnostics` 新增 `thread_budget_source` / `budgeted_threads` 并发布 `asset.worker.budgeted_threads`；新增/更新测试锚 `worker_pool_options_can_derive_threads_from_runtime_io_budget`、`project_asset_manager_default_workers_use_runtime_io_budget_source`、`worker_pool_diagnostics_track_in_flight_and_failure_counts`；追加 `runtime_absorption::asset_worker_policy::asset_worker_pool_matches_runtime_04_and_11_decisions` 锁定 Runtime 04/11 跨计划 worker 预算、背压、去重、诊断口径；同步 `docs/zircon_runtime/asset/worker_pool.md` 与 `docs/zircon_runtime/core/job_system.md`。`rustfmt --edition 2021 --check` 通过；冲突标记扫描、尾随空白扫描、预算锚点扫描通过；scoped `git diff --check` 仅 LF/CRLF 提示。Cargo 待 active cargo/rustc lanes 清空 |
| M3 | 3.1 调度诊断 | code_static_pending_cargo | 2026-06-13 | 新增 `core/runtime/tasks/diagnostics.rs`、`JobSchedulerReport`、`JobScheduler::diagnostic_report()` 与 `JobScheduler::record_diagnostics(...)`；`spawn`/`schedule`/`schedule_after` 记录 `tasks.scheduled` / `tasks.completed`，依赖释放记录 `tasks.dependency_wait_ms`，`JobHandle::wait()` 记录 `tasks.main_thread_wait_ms`；测试锚:`job_diagnostics_track_schedule_complete_and_wait_times`；`docs/zircon_runtime/core/{job_system.md,tasks.md,runtime/tasks.md}` 已同步。`rustfmt --edition 2021 --check`、锚点扫描、冲突标记扫描、尾随空白扫描与 scoped `git diff --check` 通过（仅 LF/CRLF 提示），Cargo 重跑待 active lanes 清空。 |
| M3 | 3.2 压测锚 | code_static_pending_cargo | 2026-06-13 | 新增测试锚:`deep_dependency_chain_completes_in_order`、`wide_fanout_combine_waits_for_all`，覆盖 64 层依赖链顺序完成与 128 宽扇出 combine 等齐子任务；`docs/zircon_runtime/core/{job_system.md,tasks.md,runtime/tasks.md}` 已同步。`rustfmt --edition 2021 --check`、锚点扫描、冲突标记扫描、尾随空白扫描与 scoped `git diff --check` 通过（仅 LF/CRLF 提示）。2026-06-13 `cargo test -p zircon_runtime --lib tasks --locked -- --nocapture` 未进入 tasks 测试，先被 plugin native-loader 测试漏导入 `PluginInterfaceManifest` 阻断；漏导入已修，重跑待 cargo/rustc 通道空闲。 |
| 横切 | Cargo pending gate | code_static_pending_cargo | 2026-06-13 | 新增 `runtime_absorption::plan_status::cargo_gates::runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass`，锁定 Runtime 11 在 `tasks/ecs_schedule/worker_pool/rayon` 过滤验证与 broader lib gate 通过前保持 `in_progress`；`parallel_frustum.rs` 真实 cutover 已在 2026-06-16 静态完成，但 gate 继续同步 Runtime 11、本 runtime index P14/子计划行、Runtime 05 M3.2、`docs/zircon_runtime/core/job_system.md` 与 M0 评审。 |
| 横切 | JobSystem 结构审计 owner | structure_audit_static_passed_cargo_pending | 2026-06-13 | 新增并接入 `runtime_structure_audits/job_system_boundary.py`，静态复核 `core/runtime/tasks` 9 个 folder-backed owner 模块、`JobHandle` / `JobScheduler::schedule_after` / `JobScheduler::wait_all` / `parallel_for` / `JobSchedulerReport` / `tasks.*` 诊断锚、`ScheduleParallelExecutor` 的 dependency scheduling + `JobScheduler::join(...)` 路径、Runtime 11 M1/M3 行为测试锚，以及 Runtime 11 direct-Rayon 白名单。当前 targeted audit: `expected_module_count = 9`, `direct_rayon_paths = 2`, `schedule_parallel_executor_direct_rayon = []`, `diagnostic_anchor_count = 4`, `behavior_test_anchor_count = 12`, `missing_behavior_test_anchors = []`, `oversized_modules = []`, `mirror_docs_guard_present = true`, `risks = []`。Cargo 仍等待 active lanes 清空。 |
| 横切 | JobSystem 镜像文档守卫 | mirror_docs_static_passed_cargo_pending | 2026-06-14 | 新增 `runtime_absorption::job_system::runtime_11_job_system_mirror_docs_match_structure_audit_counts`，锁定 `job_system_boundary` 的 `expected_module_count = 9`、`direct_rayon_paths = 2`、`schedule_parallel_executor_direct_rayon = []`、`diagnostic_anchor_count = 4`、`behavior_test_anchor_count = 12`、`missing_behavior_test_anchors = []`、`oversized_modules = []`、`mirror_docs_guard_present = true` 与 `risks = []` 必须同步到 JobSystem 模块文档、Runtime 11、runtime index、M0 review 和 runtime-interface convergence。Cargo/rustc 仍待 active lanes 清空。 |
| 横切 | JobSystem 总索引状态表闭环 | mirror_docs_static_passed_cargo_pending | 2026-06-14 | 本轮把 `Runtime 11 JobSystem 镜像文档守卫` 写入 runtime 总索引 `## 状态与产出记录`，并扩展 `runtime_absorption::plan_status::status_output_tables::runtime_index_status_output_records_recent_cross_plan_slices`，要求总索引记录 `runtime_11_job_system_mirror_docs_match_structure_audit_counts`、`job_system_boundary`、standalone rustc 1/1 与 `tasks/ecs_schedule/worker_pool/rayon Cargo gates pending`。验证：`rustfmt --edition 2021 --check` 通过；`runtime_11_job_system_mirror_docs_match_structure_audit_counts` standalone rustc 1/1 通过；状态表 harness 1/1 通过；Python direct `job_system_boundary_audit` 与 aggregate Runtime 11 assertions 通过；conflict/trailing scans 通过。 |
| 横切 | JobSystem 行为测试锚审计同步 | mirror_docs_static_passed_cargo_pending | 2026-06-17 | `job_system_boundary` 与 `runtime_11_job_system_mirror_docs_match_structure_audit_counts` 现在锁定 Runtime 11 M1/M3 的 12 个 `zircon_runtime/src/tests/tasks.rs` 行为锚，新增 `scheduler_wait_all_waits_for_all_handles_and_records_sync_time`；当前 `behavior_test_anchor_count = 12`、`missing_behavior_test_anchors = []`；同步 JobSystem 模块文档、Runtime 11、本 runtime index、M0 review、runtime-interface convergence 与状态输出表守卫。验证：rustfmt check、Python py_compile、direct `job_system_boundary_audit` (`behavior_test_anchor_count = 12`, `risks = []`)、standalone job_system 1/1、standalone plan_status 32/32 通过；tasks/ecs_schedule/worker_pool/rayon Cargo gates pending，`parallel_frustum.rs` cutover 已在 2026-06-16 静态完成。 |
| 横切 | JobSystem 2026-06-20 验证窗口探测 | cargo_recheck_timeout_static_guards_passed | 2026-06-20 | 空闲窗口运行 `cargo test -p zircon_runtime --lib tasks --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-11-validation-0620 --message-format short --color never -- --test-threads=1 --nocapture`，1200s 后仍停留在 `zircon_runtime` lib-test 编译且未产出测试二进制；追加等待 650s 后仍有同一 Cargo/rustc 进程，已停止本轮残留进程，未声明 Cargo 通过。轻量守卫补证通过：standalone `job_system.rs` 1/1、standalone `rayon_boundary.rs` 3/3、standalone `asset_worker_policy.rs` 1/1；其中 `asset_worker_policy` 先暴露并修正误报，将退休 `AssetWorkerPool::new(worker_count)` 检查收窄到 `impl AssetWorkerPool`，保留 `AssetWorkerPoolOptions` 为 worker-count 配置 owner。`rustfmt --edition 2021 --check zircon_runtime\src\tests\runtime_absorption\asset_worker_policy.rs` 通过；Runtime 11 仍为 `in_progress`，`tasks/ecs_schedule/worker_pool/rayon` Cargo gates pending。 |
| 横切 | JobSystem core-min 验证窗口探测 | core_min_cargo_recheck_timeout_static_guards_passed | 2026-06-20 | 更窄窗口运行 `cargo test -p zircon_runtime --lib tasks --no-default-features --features core-min --locked --jobs 1 --target-dir E:\Git\ZirconEngine\target\codex-runtime11-coremin-0620 --message-format short --color never -- --test-threads=1 --nocapture`，1200s 后仍停留在 `zircon_runtime` lib-test 编译，`target\codex-runtime11-coremin-0620\debug\deps` 无 `zircon_runtime*.exe` 测试二进制或测试结果；残留 Cargo/rustc command line 已确认匹配该 target-dir 并停止，本行不声明 Cargo 通过。轻量守卫复核通过：`job_system_boundary.py` py_compile；direct `job_system_boundary_audit` 报 `expected_module_count = 9`、`direct_rayon_paths = 2`、`behavior_test_anchor_count = 12`、`missing_behavior_test_anchors = []`、`risks = []`；standalone `job_system.rs` 1/1、standalone `rayon_boundary.rs` 3/3 通过；`tasks/ecs_schedule/worker_pool/rayon` Cargo gates 继续 pending。 |
| 横切 | JobSystem current audit recheck | job_system_current_audit_static_passed_cargo_pending | 2026-06-20 | 状态锚 `job_system_current_audit_static_passed_cargo_pending`；本轮只复核 Runtime 11 当前 JobSystem/TaskPool/调度消费边界事实，生产代码未改：`job_system_boundary_audit` 报告 task owner modules 9/9、direct Rayon paths 2/2、`schedule_parallel_executor_direct_rayon = []`、diagnostic anchors 4/4、behavior-test anchors 12/12、`oversized_modules = []`、`mirror_docs_guard_present = true`、`risks = []`。验证：Python py_compile、direct `job_system_boundary_audit` risks=[]、standalone `job_system.rs` 1/1、standalone `rayon_boundary.rs` 3/3、standalone `plan_status.rs` 32/32；tasks/ecs_schedule/worker_pool/rayon Cargo gates 仍 pending。 |
| 横切 | JobSystem 2026-07-01 current audit recheck | job_system_20260701_current_audit_static_passed_cargo_deferred | 2026-07-01 | 状态锚 `job_system_20260701_current_audit_static_passed_cargo_deferred`；复核当前 Runtime 11 JobSystem/TaskPool/调度消费边界，生产代码未改：`job_system_boundary_audit` 报告 `expected_module_count = 9`、task owner modules 9/9、`direct_rayon_paths = 2`、`schedule_parallel_executor_direct_rayon = []`、diagnostic anchors 4/4、behavior-test anchors 13/13、`oversized_modules = []`、`mirror_docs_guard_present = true`、`risks = []`。同轮 full `audit_runtime_structure.py --json` 风险汇总为 `{}`；standalone `plan_status.rs` 41/41 通过。`tasks/ecs_schedule/worker_pool/rayon` package Cargo gates 仍 deferred，因为外部 Cargo/rustc 通道 active。 |
| 横切 | JobSystem inventory split | job_system_inventory_split_static_passed_cargo_deferred_tests_deferred | 2026-06-21 | 状态锚 `job_system_inventory_split_static_passed_cargo_deferred_tests_deferred`；新增 `job_system_source_inventory.py` 作为 Runtime 11 task owner 模块清单、500 行预算与 direct-Rayon 白名单扫描所有者，新增 `job_system_anchor_inventory.py` 作为 mod/public/API/schedule executor/behavior-test/mirror-doc 锚点所有者；`job_system_boundary.py` 现在只保留审计读取、缺失项计算与风险聚合，当前 193 行；`job_system_markdown.py` 承接 Markdown 渲染并为 64 行。direct `job_system_boundary_audit` 报告 task owner modules 9/9、direct Rayon paths 2/2、`schedule_parallel_executor_direct_rayon = []`、diagnostic anchors 4/4、behavior-test anchors 12/12、`oversized_modules = []`、`mirror_docs_guard_present = true`、`risks = []`。验证：Python py_compile、direct `job_system_boundary_audit` risks=[]、standalone `job_system.rs` 1/1、standalone `rayon_boundary.rs` 3/3、standalone `plan_status.rs` 33/33；tasks/ecs_schedule/worker_pool/rayon Cargo gates 仍 pending，未提升 Runtime 11 包级 gate。 |
| 横切 | JobSystem Markdown renderer split | job_system_markdown_split_static_passed_cargo_deferred_tests_deferred | 2026-06-21 | 状态锚 `job_system_markdown_split_static_passed_cargo_deferred_tests_deferred`；`job_system_markdown.py` now owns `render_job_system_boundary_markdown`, and `audit_runtime_structure.py` imports the renderer from that Markdown owner instead of `job_system_boundary.py`; `job_system_boundary.py` now owns audit read, missing-anchor calculation, and risk aggregation at 193 lines, while the Markdown owner is 64 lines. Direct audit reports task owner modules 9/9, direct Rayon paths 2/2, `schedule_parallel_executor_direct_rayon = []`, diagnostic anchors 4/4, behavior-test anchors 12/12, `oversized_modules = []`, `mirror_docs_guard_present = true`, and `risks = []`. Validation: Python py_compile and direct `job_system_boundary_audit`; standalone `job_system.rs` 1/1, standalone `rayon_boundary.rs` 3/3, and standalone `plan_status.rs` 33/33; broader tasks/ecs_schedule/worker_pool/rayon Cargo gates remain deferred while external compile lanes are active. |
| M1 | 1.1 worker wait-assist | worker_wait_assist_static_passed_cargo_deferred | 2026-06-21 | 状态锚 `worker_wait_assist_static_passed_cargo_deferred`；`JobHandle::wait()` 现在在等待循环中释放状态锁并调用 task-pool-owned `assist_current_thread_once(...)`，当前 Rayon worker 可先执行同池 pending task，空闲时只短暂 `WORKER_WAIT_IDLE_PARK`，避免单 worker 调度器内 worker task 等待刚提交 child handle 时自锁；direct Rayon 仍只在 `core/runtime/tasks/{pool,parallel_for}.rs` 两个 owner 中，`job_handle.rs` 不新增 direct-Rayon path。新增行为锚 `worker_thread_wait_does_not_deadlock_scheduler`，`job_system_boundary_audit` 当前 `behavior_test_anchor_count = 13`、`missing_behavior_test_anchors = []`、`direct_rayon_paths = 2`、`risks = []`。验证：rustfmt check、Python py_compile、direct `job_system_boundary_audit`、standalone `job_system.rs` 1/1、standalone `rayon_boundary.rs` 3/3、standalone `plan_status.rs` 33/33；tasks/ecs_schedule/worker_pool/rayon Cargo gates 仍 deferred，未声明包级 Cargo 通过。 |
| 横切 | worker wait-assist core-min 验证窗口探测 | worker_wait_assist_core_min_cargo_timeout_no_result_residual_stopped | 2026-06-21 | 状态锚 `worker_wait_assist_core_min_cargo_timeout_no_result_residual_stopped`；运行 `cargo test -p zircon_runtime --lib worker_thread_wait_does_not_deadlock_scheduler --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime11-workerwait-0621 --message-format short --color never -- --test-threads=1 --nocapture`，1800s 工具窗口超时，未产出测试结果；`target\codex-runtime11-workerwait-0621\debug\deps` 无 `zircon_runtime*.exe` 测试二进制，仅有 `zircon_runtime-c339c28ec98a5de7.d` 依赖文件；匹配该 target-dir 的 2 个 cargo 与 1 个 rustc 残留进程已停止。并行 render-owned `render_product_directional_shadow_pcf_quality_changes_receiver_edge_capture` 编译通道仍活动；Runtime 11 `tasks/ecs_schedule/worker_pool/rayon` Cargo gates 继续 deferred，未声明包级 Cargo 通过。 |
| 横切 | worker wait-assist core-min test binary 验证 | worker_wait_assist_core_min_test_binary_passed_cargo_gate_pending | 2026-06-21 | 状态锚 `worker_wait_assist_core_min_test_binary_passed_cargo_gate_pending`；复用并行 render-owned core-min Cargo 产出的 `target\codex-runtime-shadow-spot-0621\debug\deps\zircon_runtime-c339c28ec98a5de7.exe`，复制为 `target\codex-runtime11-workerwait-0621\debug\deps\zircon_runtime-coremin-workerwait.exe` 后运行 `worker_thread_wait_does_not_deadlock_scheduler --test-threads=1 --nocapture`，结果 `1 passed; 0 failed; 4687 filtered out`。该证据覆盖 worker wait-assist 行为在当前 core-min test binary 中可运行通过，但不是 `cargo test -p zircon_runtime --lib tasks/ecs_schedule/worker_pool/rayon` 完整 gate；Runtime 11 包级 Cargo gates 仍 pending，未声明 completed。 |
| 横切 | core-min test binary task/guard batch | runtime_11_core_min_test_binary_task_guard_batch_passed_cargo_gate_pending | 2026-06-21 | 状态锚 `runtime_11_core_min_test_binary_task_guard_batch_passed_cargo_gate_pending`；继续使用 `target\codex-runtime11-workerwait-0621\debug\deps\zircon_runtime-coremin-workerwait.exe` 直跑 Runtime 11 相关过滤项：`tests::tasks::` 为 `18 passed; 0 failed; 4670 filtered out`（其中 panic-propagation 用例按预期打印 panic 后最终通过），`worker_pool` 为 `10 passed; 0 failed; 4678 filtered out`，`rayon` 为 `4 passed; 0 failed; 4684 filtered out`，`runtime_absorption::job_system` 为 `1 passed; 0 failed; 4687 filtered out`，`runtime_absorption::rayon_boundary` 为 `3 passed; 0 failed; 4685 filtered out`，`runtime_absorption::asset_worker_policy` 为 `1 passed; 0 failed; 4687 filtered out`。该批次覆盖 core-min test binary 中的任务模型、worker pool、Rayon 边界、JobSystem 镜像和 Runtime 04/11 asset worker policy 守卫；完整 `cargo test -p zircon_runtime --lib tasks/ecs_schedule/worker_pool/rayon` gate 仍 pending，未声明 Runtime 11 completed。 |
| 横切 | ecs_schedule source-guard lifetime anchor repair | runtime_11_ecs_schedule_lifetime_guard_anchor_static_passed_rebuild_pending | 2026-06-21 | 状态锚 `runtime_11_ecs_schedule_lifetime_guard_anchor_static_passed_rebuild_pending`；复用 core-min test binary 直跑 `ecs_schedule` 得到 `74 passed; 1 failed; 4613 filtered out`，唯一失败 `world_driver_reuses_tick_schedule_snapshots_for_stage_runs` 是源码守卫仍匹配旧锚 `native_steps: &[Self]`；当前实现 `scheduled_scene_step.rs` 已是 `native_steps: &'a [Self]`，仍保持 `SortedScheduledSceneSteps<'a>` 直接借用切片且无 `native_steps: Vec<Self>`。已将守卫更新为 `native_steps: &'a [Self]` 并通过 `rustfmt --edition 2021 --check zircon_runtime\src\scene\tests\ecs_scheduled_native_systems.rs` 与源码锚扫描；由于当前 render-owned Cargo/rustc lane 活动，本修复仍需重建 test binary 后复跑 `ecs_schedule`，不提升 Runtime 11 Cargo gate。 |
| 横切 | ecs_schedule core-min Cargo 复验 | runtime_11_core_min_ecs_schedule_cargo_passed_remaining_gates_pending | 2026-06-21 | 状态锚 `runtime_11_core_min_ecs_schedule_cargo_passed_remaining_gates_pending`；复验先暴露 `world_driver_reuses_tick_schedule_snapshots_for_stage_runs` 后续源码护栏漂移：`SceneScheduleStagePlan::from_registry` 恢复显式 `internal_system_counts` / `native_step_counts` 临时变量，`Schedule` taken-id helper 锚从 native-only 同步到 runtime-aware `remove_taken_system_id(...)`，`SceneSystemRegistry` step/conflict count 锚同步到 `runtime_systems`，`insert_native_system_sorted(...)` 计数同步为两个 native 注册入口 + restore 共 3 次。最终运行 `cargo test -p zircon_runtime --lib ecs_schedule --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --test-threads=1 --nocapture`，日志 `target\codex-runtime11-logs\ecs_schedule_coremin_20260621-142346.{out,err}.log` 记录 `75 passed; 0 failed; 4616 filtered out`（lib-test 仍有既存 54 warnings）。该证据关闭 core-min `ecs_schedule` 复验窗口，但 Runtime 11 的 `tasks`/`worker_pool`/`rayon` Cargo gates 与更宽配置仍 pending，未声明 Runtime 11 completed。 |
| 横切 | tasks core-min Cargo 复验 | runtime_11_core_min_tasks_cargo_passed_remaining_gates_pending | 2026-06-21 | 状态锚 `runtime_11_core_min_tasks_cargo_passed_remaining_gates_pending`；运行 `cargo test -p zircon_runtime --lib tasks --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --test-threads=1 --nocapture`，日志 `target\codex-runtime11-logs\tasks_coremin_20260621-145417.{out,err}.log` 记录 `19 passed; 0 failed; 4673 filtered out`（panic-propagation 用例按预期打印 panic 后最终通过，lib-test 仍有既存 54 warnings）。该证据关闭 core-min `tasks` Cargo 复验窗口；完整守卫锚 `tasks/ecs_schedule/worker_pool/rayon` 继续可见，剩余 `worker_pool`/`rayon` Cargo gates 与更宽配置仍 pending，未声明 Runtime 11 completed。 |
| 横切 | worker_pool core-min Cargo 复验 | runtime_11_core_min_worker_pool_cargo_passed_remaining_gates_pending | 2026-06-21 | 状态锚 `runtime_11_core_min_worker_pool_cargo_passed_remaining_gates_pending`；运行 `cargo test -p zircon_runtime --lib worker_pool --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --test-threads=1 --nocapture`，日志 `target\codex-runtime11-logs\worker_pool_coremin_20260621-153418.{out,err}.log` 记录 `10 passed; 0 failed; 4682 filtered out`（lib-test 仍有既存 54 warnings）。该证据关闭 core-min `worker_pool` Cargo 复验窗口；完整守卫锚 `tasks/ecs_schedule/worker_pool/rayon` 继续可见，剩余 `rayon` Cargo gate 与更宽配置仍 pending，未声明 Runtime 11 completed。 |
| 横切 | rayon core-min Cargo 复验 | runtime_11_core_min_rayon_cargo_passed_broader_gates_pending | 2026-06-21 | 状态锚 `runtime_11_core_min_rayon_cargo_passed_broader_gates_pending`；运行 `cargo test -p zircon_runtime --lib rayon --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-shadow-spot-0621 --message-format short --color never -- --test-threads=1 --nocapture`，日志 `target\codex-runtime11-logs\rayon_coremin_20260621-155517.{out,err}.log` 记录 `4 passed; 0 failed; 4688 filtered out`（lib-test 仍有既存 54 warnings）。至此 core-min `tasks/ecs_schedule/worker_pool/rayon` 四个过滤项均通过；默认/更宽配置 Cargo gate 仍 pending，未声明 Runtime 11 completed。 |
| 横切 | tasks default Cargo 复验 | runtime_11_default_tasks_cargo_passed_remaining_default_gates_pending | 2026-06-21 | 状态锚 `runtime_11_default_tasks_cargo_passed_remaining_default_gates_pending`；运行 `cargo test -p zircon_runtime --lib tasks --locked --jobs 1 --target-dir target\codex-runtime11-default-tasks-0621 --message-format short --color never -- --test-threads=1 --nocapture`，日志 `target\codex-runtime11-logs\tasks_default_20260621-161421.{out,err}.log` 记录默认配置 test binary 编译通过并运行 `19 passed; 0 failed; 4673 filtered out`（panic-propagation 用例按预期打印 panic 后最终通过，lib-test 仍有既存 53 warnings）。该证据关闭默认配置 `tasks` 复验窗口；默认/更宽配置 `worker_pool`/`rayon`/`ecs_schedule` gates 仍 pending，未声明 Runtime 11 completed。 |
| 横切 | worker_pool default Cargo 复验 | runtime_11_default_worker_pool_cargo_passed_remaining_default_gates_pending | 2026-06-21 | 状态锚 `runtime_11_default_worker_pool_cargo_passed_remaining_default_gates_pending`；运行 `cargo test -p zircon_runtime --lib worker_pool --locked --jobs 1 --target-dir target\codex-runtime11-default-tasks-0621 --message-format short --color never -- --test-threads=1 --nocapture`，日志 `target\codex-runtime11-logs\worker_pool_default_20260621-165855.{out,err}.log` 记录默认配置 test binary 编译通过并运行 `10 passed; 0 failed; 4683 filtered out`（lib-test 仍有既存 53 warnings）。该证据关闭默认配置 `worker_pool` 复验窗口；默认/更宽配置 `rayon`/`ecs_schedule` gates 仍 pending，未声明 Runtime 11 completed。 |
| 横切 | rayon default Cargo 复验 | runtime_11_default_rayon_cargo_passed_full_lib_gate_pending | 2026-06-21 | 状态锚 `runtime_11_default_rayon_cargo_passed_full_lib_gate_pending`；运行 `cargo test -p zircon_runtime --lib rayon --locked --jobs 1 --target-dir target\codex-runtime11-default-tasks-0621 --message-format short --color never -- --test-threads=1 --nocapture`，日志 `target\codex-runtime11-logs\rayon_default_20260621-171236.{out,err}.log` 记录默认配置 test binary 编译通过并运行 `4 passed; 0 failed; 4690 filtered out`（lib-test 仍有既存 53 warnings）。该证据关闭默认配置 `rayon` 复验窗口；默认配置 `ecs_schedule` 与收尾全量 `cargo test -p zircon_runtime --lib --locked` gate 仍 pending，未声明 Runtime 11 completed。 |
| 横切 | ecs_schedule default Cargo 复验 | runtime_11_default_ecs_schedule_cargo_passed_full_lib_gate_pending | 2026-06-21 | 状态锚 `runtime_11_default_ecs_schedule_cargo_passed_full_lib_gate_pending`；运行 `cargo test -p zircon_runtime --lib ecs_schedule --locked --jobs 1 --target-dir target\codex-runtime11-default-tasks-0621 --message-format short --color never -- --test-threads=1 --nocapture`，日志 `target\codex-runtime11-logs\ecs_schedule_default_20260621-172513.{out,err}.log` 记录默认配置 test binary 编译通过并运行 `75 passed; 0 failed; 4619 filtered out`（lib-test 仍有既存 53 warnings）。至此 core-min 与默认配置 `tasks/ecs_schedule/worker_pool/rayon` 过滤项均已通过；收尾全量 `cargo test -p zircon_runtime --lib --locked` gate 仍 pending，未声明 Runtime 11 completed。 |
| 横切 | full-lib default Cargo closeout attempt | runtime_11_full_lib_cargo_timeout_with_broader_failures_observed | 2026-06-21 | 状态锚 `runtime_11_full_lib_cargo_timeout_with_broader_failures_observed`；运行 `cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir target\codex-runtime11-default-tasks-0621 --message-format short --color never -- --test-threads=1 --nocapture`，1200s 工具窗口超时，日志 `target\codex-runtime11-logs\full_lib_default_20260621-173046.{out,err}.log` 未出现最终 `test result:`，最后一行停在 `test graphi`；超时前已观察到 58 条 `... FAILED`，跨 asset、core runtime structure、dynamic_api session、graphics/post-process/materialization/project-render 等非 Runtime 11 专属过滤项；匹配该 target-dir 的 cargo/rustc/test 残留进程为 0。该证据阻塞收尾全量 gate，Runtime 11 仍保持 in_progress，不声明 completed。 |
| 横切 | core runtime full-lib triage recheck | runtime_11_core_runtime_tests_passed_full_lib_gate_broader_failures_pending | 2026-06-21 | 状态锚 `runtime_11_core_runtime_tests_passed_full_lib_gate_broader_failures_pending`；针对上轮 full-lib broader failure 里的 core runtime 低层窗口，修复 activation empty startup/shutdown fast path 与 unload-order typed slice 传递，并刷新 event bus prune、registration、registry-name、resolution、blocked-unload/source-list 等 stale source guards。验证：`rustfmt --edition 2021 --check` 覆盖相关 core runtime 测试文件；`cargo test -p zircon_runtime --lib registration_source_preserves_hot_path_structure --locked --jobs 1 --target-dir target\codex-runtime11-default-tasks-0621 --message-format short --color never -- --test-threads=1 --nocapture` 日志 `target\codex-runtime11-logs\registration_structure_20260621-195458.log` 为 `1 passed; 0 failed; 4694 filtered out`；`cargo test -p zircon_runtime --lib core::runtime::tests:: --locked --jobs 1 --target-dir target\codex-runtime11-default-tasks-0621 --message-format short --color never -- --test-threads=1 --nocapture` 日志 `target\codex-runtime11-logs\core_runtime_all_20260621-200647.log` 为 `82 passed; 0 failed; 4613 filtered out`（invalid segment `catch_unwind` 用例按预期打印 panic，既存 53 warnings）。该证据关闭 full-lib 中的 core runtime broader failure slice；asset/dynamic_api/graphics 等 broader gates 仍 pending，未声明 Runtime 11 completed。 |
| 横切 | asset broader failure triage core-min 复验 | runtime_11_asset_tests_passed_full_lib_gate_dynamic_graphics_pending | 2026-06-21 | 状态锚 `runtime_11_asset_tests_passed_full_lib_gate_dynamic_graphics_pending`；继续按 support-first 从上轮 full-lib broader failure 中修复 asset 低层窗口：`SceneMeshInstanceAsset`/`SceneMeshLodLevelAsset` 对非 human-readable serializer 写完整字段序列，UI v1/v2 document artifact cache 改为规范化 TOML 文本边界，stacked array 纹理 fixture 改为断言 RGBA8 ready upload plan，vampire 示例 WGSL 恢复 `zr_gpu_scene_shadow_params` realistic material/light marker。验证：`rustfmt --edition 2021 --check zircon_runtime\src\asset\tests\assets\texture_importer.rs` 通过；`cargo test -p zircon_runtime --lib importer_texture_fixture_reinterprets_stacked_array_layout --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime11-coremin-tasks-0621 --message-format short --color never -- --test-threads=1 --nocapture` 日志 `target\codex-runtime11-logs\asset_texture_stacked_array_coremin_20260621.log` 为 `1 passed; 0 failed; 4696 filtered out`；直跑 `target\codex-runtime11-coremin-tasks-0621\debug\deps\zircon_runtime-c339c28ec98a5de7.exe vampire_example_manifest_scene_and_scripts_are_importable --test-threads=1 --nocapture` 日志 `target\codex-runtime11-logs\asset_vampire_manifest_coremin_direct_20260621.log` 为 `1 passed; 0 failed; 4696 filtered out`；同一 test binary 直跑 `asset::tests::` 日志 `target\codex-runtime11-logs\asset_namespace_coremin_direct_20260621.log` 为 `363 passed; 0 failed; 4334 filtered out`。该证据关闭当前 asset broader failure slice；dynamic_api/graphics 与默认 full-lib gate 仍 pending，未声明 Runtime 11 completed。 |
| 横切 | full-lib default after asset triage recheck | runtime_11_full_lib_after_asset_recheck_blocked_graphics_compile_timeout | 2026-06-21 | 状态锚 `runtime_11_full_lib_after_asset_recheck_blocked_graphics_compile_timeout`；资产低层窗口关闭后再次运行 `cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir target\codex-runtime11-default-tasks-0621 --message-format short --color never -- --test-threads=1 --nocapture`，日志 `target\codex-runtime11-logs\full_lib_default_after_asset_20260621.log` 在编译阶段被 graphics execution-record 测试模块阻断：`render_graph_execution_record.rs:871:22` 报 `RenderExposureReadbackReport` 未解析；随后当前源码已显示该类型 import 存在，说明同一图形侧文件正在漂移/被其他会话推进。为复核该阻断点，运行 `execution_record_preserves_exposure_readback_report` 默认配置窄过滤，日志 `target\codex-runtime11-logs\graphics_execution_record_exposure_default_20260621.log` 在 904s 工具窗口内未产出最终测试结果；确认匹配该 target-dir/filter 的 2 个 cargo 与 1 个 rustc 残留进程后已停止。该证据只记录 full-lib/default gate 仍被 graphics 编译/长验证链路阻塞；core runtime 与 asset 已关闭的低层切片不回退，dynamic_api/graphics 与默认 full-lib gate 仍 pending，Runtime 11 不提升 completed。 |
| 横切 | full-lib default after graphics exposure retry | runtime_11_full_lib_after_graphics_exposure_retry_timeout_104_broader_failures | 2026-06-21 | 状态锚 `runtime_11_full_lib_after_graphics_exposure_retry_timeout_104_broader_failures`；复跑默认配置窄过滤 `execution_record_preserves_exposure_readback_report`，日志 `target\codex-runtime11-logs\graphics_execution_record_exposure_default_retry_20260621.log` 记录编译通过并运行 `1 passed; 0 failed; 4702 filtered out`，说明上一行的 exposure-readback import 编译阻断已由当前源码收束。随后再次运行默认全库 `cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir target\codex-runtime11-default-tasks-0621 --message-format short --color never -- --test-threads=1 --nocapture`，日志 `target\codex-runtime11-logs\full_lib_default_after_graphics_exposure_retry_20260621.log` 在 1800s 工具窗口超时，已进入 `running 4704 tests` 且观察到 104 条 `... FAILED`，失败簇跨 post-process/render graph、dynamic_api session、graphics product/render-framework、input boundary、native/plugin export、scene dynamic/session、extension/graphics-surface guards；无匹配本 target-dir 的 cargo/rustc 残留进程，另有外部 `target\codex-runtime-postprocess-0621` Cargo lane 未触碰。该证据说明 Runtime 11 的 core-min/default `tasks/ecs_schedule/worker_pool/rayon` 与本轮 core runtime、asset 低层切片仍保持关闭，但默认 full-lib gate 仍被跨计划 broader failures 阻塞，Runtime 11 不提升 completed。 |

基线数值（开工首日记录）：

- `JobScheduler` 公共原语基线：3（spawn/install/join，job_scheduler.rs:31-47）；2026-06-13 M1 静态扩展后为 5（spawn/install/join/schedule/schedule_after）；2026-06-17 M1.3 后为 6（新增 `wait_all`）
- rayon 使用文件基线：production 当前 2 文件（`core/runtime/tasks/pool.rs`、`core/runtime/tasks/parallel_for.rs`）；`schedule_parallel_executor.rs` 已在 11-M2.2 非 graphics 收编中移出 direct rayon，`graphics/visibility/culling/parallel_frustum.rs` 已在 2026-06-16 M2.1 收编到 runtime `parallel_for(...)` / compute `TaskPool`。
- 三池线程切分基线：`TaskPoolOptions` 默认值 io 25% cap 4、async_compute 25% cap 4、compute 100% of remaining（all min 1；`min_total_threads=1`，`max_total_threads=usize::MAX`）
- `cargo test -p zircon_runtime --lib tasks --locked` 通过数基线：2026-06-20 空闲窗口复跑 1200s + 650s 仍停留在 lib-test 编译且无测试二进制/测试结果，本轮 Cargo/rustc 残留已停止；2026-06-21 targeted `worker_thread_wait_does_not_deadlock_scheduler` core-min filter 1800s 后仍未产出测试二进制/测试结果，匹配 target-dir 的残留已停止；同日复用 render-owned core-min test binary 直跑 `worker_thread_wait_does_not_deadlock_scheduler` 得到 `1 passed; 0 failed; 4687 filtered out`，并直跑 `tests::tasks::` 得到 `18 passed; 0 failed; 4670 filtered out`；随后 core-min Cargo `tasks` gate 通过，结果 `19 passed; 0 failed; 4673 filtered out`；默认配置 Cargo `tasks` gate 通过，结果 `19 passed; 0 failed; 4673 filtered out`
- `cargo test -p zircon_runtime --lib worker_pool --locked` 通过数基线：2026-06-21 core-min Cargo `worker_pool` gate 通过，结果 `10 passed; 0 failed; 4682 filtered out`；默认配置 Cargo `worker_pool` gate 通过，结果 `10 passed; 0 failed; 4683 filtered out`
- `cargo test -p zircon_runtime --lib rayon --locked` 通过数基线：M2.2 当前静态收编 ECS direct-rayon，且 M2.1 已收编 render-owned `parallel_frustum.rs` 例外，当前 direct_rayon_paths = 2；2026-06-21 core-min Cargo `rayon` gate 通过，结果 `4 passed; 0 failed; 4688 filtered out`；默认配置 Cargo `rayon` gate 通过，结果 `4 passed; 0 failed; 4690 filtered out`
- `cargo test -p zircon_runtime --lib ecs_schedule --locked` 通过数基线：2026-06-21 core-min Cargo `ecs_schedule` gate 通过，结果 `75 passed; 0 failed; 4616 filtered out`；默认配置 Cargo `ecs_schedule` gate 通过，结果 `75 passed; 0 failed; 4619 filtered out`

## 风险与协调

- **同文件三计划交汇**：`schedule_parallel_executor.rs` 同时是 03-M3（开关/计数）、07-M1（计数消费）、本计划 M2.3 的目标——执行序定为 03-M3 → 11-M2.3 → 07 采集，先开工者状态节登记占用；`worker_pool.rs` 与 04-M2 同窗口执行（2.4 已写明）。
- `parallel_frustum.rs` 在 graphics 区，受 10fps 会话与 render 计划影响：M2.1 已在 2026-06-16 收编到 runtime compute `TaskPool` + `parallel_for(...)`。后续 render 计划若重写剔除（GPU 驱动剔除），新 CPU 兜底路径仍必须直接使用统一任务原语，不得恢复 direct rayon。
- worker 内 `wait()` 死锁策略已由 `worker_wait_assist_static_passed_cargo_deferred` 收束为 task-pool-owned `assist_current_thread_once(...)` + `worker_thread_wait_does_not_deadlock_scheduler` 行为锚；2026-06-21 core-min focused Cargo 探测 `worker_wait_assist_core_min_cargo_timeout_no_result_residual_stopped` 未生成测试二进制或结果，随后复用 render-owned core-min test binary 取得 `worker_wait_assist_core_min_test_binary_passed_cargo_gate_pending` 与 `runtime_11_core_min_test_binary_task_guard_batch_passed_cargo_gate_pending`；`ecs_schedule` direct binary filter 暴露的 stale source guard 已以 `runtime_11_ecs_schedule_lifetime_guard_anchor_static_passed_rebuild_pending` 静态修正，并在 `runtime_11_core_min_ecs_schedule_cargo_passed_remaining_gates_pending` 中通过 core-min Cargo 复验（75/75 passed）；`tasks` 已在 `runtime_11_core_min_tasks_cargo_passed_remaining_gates_pending` 中通过 core-min Cargo 复验（19/19 passed），并在 `runtime_11_default_tasks_cargo_passed_remaining_default_gates_pending` 中通过默认配置 Cargo 复验（19/19 passed）；`worker_pool` 已在 `runtime_11_core_min_worker_pool_cargo_passed_remaining_gates_pending` 中通过 core-min Cargo 复验（10/10 passed），并在 `runtime_11_default_worker_pool_cargo_passed_remaining_default_gates_pending` 中通过默认配置 Cargo 复验（10/10 passed）；`rayon` 已在 `runtime_11_core_min_rayon_cargo_passed_broader_gates_pending` 中通过 core-min Cargo 复验（4/4 passed），并在 `runtime_11_default_rayon_cargo_passed_full_lib_gate_pending` 中通过默认配置 Cargo 复验（4/4 passed）；`ecs_schedule` 已在 `runtime_11_default_ecs_schedule_cargo_passed_full_lib_gate_pending` 中通过默认配置 Cargo 复验（75/75 passed）。core-min 与默认配置 `tasks/ecs_schedule/worker_pool/rayon` 过滤闸门已闭合；收尾全量 `cargo test -p zircon_runtime --lib --locked` 在 `runtime_11_full_lib_cargo_timeout_with_broader_failures_observed` 中 1200s 超时且观察到 broader non-11 failures，仍是最终验证闸门，未通过前不得把 Runtime 11 置为 completed。
- 主线程同步点的帧位（帧末 wait_all 闸放在 03 帧循环的哪个 stage 之后）与 03-M1/M2 的帧序定稿联动——`job_system.md` 与 `frame_schedule.md` 互引，避免两文档各说一套。
- 物理（01-M3 决策）是 JobSystem 的最大潜在客户：jolt/rapier 自带线程池 vs 接本系统的裁决，在物理选型文档与本计划 M0 消费方矩阵中双向引用。
