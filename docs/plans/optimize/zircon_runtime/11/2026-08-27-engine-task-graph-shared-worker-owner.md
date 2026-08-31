# EngineTaskGraph 单一 Worker Owner 结构硬切报告

- 日期：2026-08-27
- owner 计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 上位计划：`docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`
- 状态：`source_implemented_static_review_pending_managed_validation_and_product_profile`

## 结论

当前主要缺陷不是三池分配公式，而是把 `Compute`、`AsyncCompute`、`Io`
工作种类直接建模为三套物理 Rayon pool。这个模型无法表达统一 affinity、priority、
dependency、domain quota 和 shutdown owner，还允许 subsystem 再创建私有 worker set。

本切片把 Core 的运行时执行 owner 硬切为 folder-backed
`core/runtime/tasks/task_graph/EngineTaskGraph`：每个 `CoreRuntime` 只创建一个物理
worker set，`JobScheduler`、asset、platform、graphics、scene、VM discovery、dynamic
session archive 和 editor settings 均复用该 owner。旧 `ExecutionRuntime` 类型、
`tasks/execution/` 模块、Core `task_pools()/task_pool(kind)` selector 和兼容 alias 均不保留。

这只是 Plan02 M1 的第一层基础设施，不代表统一 TaskGraph 已完成。named-thread affinity、
priority、全局/domain/plugin 配额、keyed-I/O 异 key 并行、timer owner、process-default/private
worker 清零和 current-source 产品性能/功耗证据仍为开放项。

## 调研依据

实现前复读了：

- `docs/plans/performance/02/2026-08-15-runtime-taskgraph-current-architecture-review.md`；
- `dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Async/TaskGraph.cpp` 中 named thread、
  shared worker scheduler、stall/wake、target/current thread 和总线程上限行为；
- `docs/plans/optimize/zircon_runtime/11/2026-08-27-execution-worker-join-owner.md`
  中 Rayon custom spawn、弱 consumer handle、精确 join receipt 和重试式关闭结论；
- CoreRuntime、JobScheduler、TaskPool/TaskPools 以及 asset/platform/graphics/scene/editor
  的实际 pool 注入调用图。
- `dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/World.cpp` 的
  `UWorld::CreateWorld`：world 构造只建立 world/package/lifecycle 状态，不在 scene
  object 内隐式创建 worker owner；
- `dev/Fyrox/fyrox-impl/src/engine/mod.rs` 的 `EngineInitParams`：resource manager 与
  asynchronous task pool 都由 engine composition 显式注入，而不是由 scene manager
  default constructor 获取进程全局执行器。

选择保留底层 `TaskPool` 的弱句柄、admission gate、worker `JoinHandle` 和 Rayon work
stealing 后端，但不保留三物理池 owner。该选择复用已经验证过的 worker 生命周期机制，
同时把结构方向收敛到 Unreal 的单 scheduler/共享 worker 集。

## 源码基线与当前模型

原三池源码模型和当前单 worker-set 模型如下。数据是构造公式，不是 WPR 或功耗实测：

| 配置 worker budget | 原物理 worker set | 原实际 worker | 当前物理 worker set | 当前实际 worker |
|---:|---:|---:|---:|---:|
| 1 | 3 | 3 | 1 | 1 |
| 2 | 3 | 3 | 1 | 2 |
| 16 | 3 | 16 | 1 | 16 |

因此本切片已经消除 1/2 worker 配置下的预算超配，并把 Core 的物理 owner 数从 3
收敛为 1；它没有证明 wall-clock、RSS、idle wakeup 或功耗改善，也没有覆盖仍可能触发
`TaskPools::process_default()` 或专用线程的 process-lifetime/standalone 路径。

## 已实现

1. `EngineTaskGraphOptions` 直接表达一个精确全局 worker budget；默认值取当前可用并行度。
2. `EngineTaskGraph` 只持有一个 `zircon-taskgraph-worker` pool，scope 提交不再按
   `TaskPoolKind` 选择物理 pool；descriptor 中的 kind 暂时只保留为任务语义标签。
3. `TaskGraphWorkerInventory` 只报告 `worker_set_count`、`worker_count` 和 thread name；
   shutdown receipt 只报告这一 worker set 的 expected/exited/joined 守恒。
4. CoreRuntime 默认 scheduler 和所有已迁移产品 consumer 复用同一个 execution owner。
5. dynamic session 的退出顺序改为：停止 session scope admission，等待该 scope
   quiescent，关闭 runtime modules，最后关闭 TaskGraph 并 join worker。模块 cleanup
   不再发生在 scheduler 已停止之后。
6. 旧 public 类型与 selector 硬删除；结构审计切换到 `tasks/task_graph/`，没有 alias、
   forwarding wrapper 或双写模型。
7. scope 注册由其最后一个 live owner 持有 RAII registration；空 scope 销毁时立即从
   graph 的弱引用表移除，不再把历史创建次数累积为 shutdown 前的常驻状态。
8. 外部 admission 与内部 continuation 已分离：每个被接受的提交持有一个有界执行租约，
   dependency pending work 把同一租约带到实际 enqueue，终态/依赖 callback 只能从仍活跃
   的租约派生 continuation。关闭先拒绝新租约，再等待 scope 与租约归零，最后由外部
   owner 释放并 join worker；租约只持 weak pool + 独立 tracker，不会让最后一个 Rayon
   owner 在其 worker 自身析构。shutdown census 同时报告 `active_submission_count`。
9. scene manager 的内存 owner 与 artifact I/O owner 已分离：
   `DefaultLevelManager::default()` 不再取得 `TaskPools::process_default()`，纯内存 level
   构造不会物化或借用进程池；只有 `SceneModule` 的 `with_core` composition 注入
   TaskGraph worker handle。没有 runtime owner 的 artifact save 在序列化和文件系统工作前
   返回 typed `SceneProjectError::RuntimeUnavailable`，不建立 standalone fallback。

## 复杂度与热路径预算

设 `W` 为 worker 数、`S_live` 为 live scope 数、`T_live` 为当前已 admission 且未退休的
task 数：

- runtime 构造与持久 worker/join 状态：`O(W)`；
- scope 创建与最后 owner 释放会更新有序注册表：`O(log S_live)`；
- worker-owner 获取和正常任务 admission：摊销 `O(1)`；
- 持久状态：`O(W + S_live + T_live)`，不再包含已销毁 scope 的历史弱引用；
- scope shutdown/accounting：`O(S_live + T_live)`；
- worker close/join 与 receipt：`O(W)`；
- 每次外部提交、continuation 派生和租约释放各更新一次 pool-local atomic word
  （closed bit + active count）：`O(1)`；任务热路径不获取 shutdown mutex，关闭等待与最后
  一个租约释放才使用 `Condvar`，没有新增轮询；
- 本切片没有新增每任务线程、轮询循环、全局 scheduler 锁或额外队列复制。

上述规模结论只描述当前源码结构。ready graph、priority queue、keyed ready index 和
affinity lane 引入后必须重新量化，不能沿用本报告作为最终算法证明。

## 未完成与动态验收门

- 为 descriptor 增加 main/render/RHI/worker affinity、priority、budget class、deadline、
  dependency 和 domain/plugin quota，并在一个 scheduler 中执行。
- 把 bounded keyed I/O 从单 pump 和 `VecDeque` middle-remove 改为同 key 有序、异 key
  有界并行的 ready index/generation barrier。
- 审查并收敛 `TaskPools::process_default()`、`JobScheduler::default/process_io()`、timer、
  native discovery、text raster 和其它专用线程 owner；不能在没有 profile 时机械替换。
- 在 E 盘受管 target/profile 目录运行 1/2/N worker 和 1/1k/100k task 矩阵，记录线程峰值、
  queue p50/p95/p99、allocation、steal/wake/wait、shutdown p50/p95/max、RSS 和 CPU time。
- 对 F0/F2/F4 current-source 产品各至少 3 次采集 WPR/xperf 与功耗数据，并绑定 source
  fingerprint 和 scheduler counters。没有这些数据前不得宣称瓶颈消失或接近 Unreal。
- 完成 managed Windows Cargo、结构审计和独立 code review 后，才允许把 Plan02 M1 对应
  条目勾选、创建里程碑 commit 并发送企微量化摘要。

## 当前证据

- 本切片涉及的 Rust 文件已通过固定 `rustfmt 1.8.0-stable (Rust 1.94.1)` 解析和格式化；
- workspace tracked Rust 调用面中，Core `.task_pools()` / `.task_pool(kind)` consumer 为 0；
- 旧 `ExecutionRuntime`/`ExecutionScope` 等 public 类型调用为 0；
- `job_system_boundary` 报告 owner 15/15、行为锚点 67/67、direct-Rayon 白名单
  2/2、缺失 API/声明/模块 0、超 500 行 owner 0、runtime 到 editor 依赖 0、
  `risks = []`；
- managed Cargo、产品 profile、RSS 与功耗证据尚未执行，因此状态保持 source-only。
