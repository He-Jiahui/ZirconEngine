---
related_code:
  - zircon_runtime/src/core/runtime/tasks
  - zircon_runtime/src/core/framework/tasks
  - zircon_plugins/navigation/runtime/src
  - zircon_runtime/src/graphics/runtime/render_framework
  - zircon_runtime/src/platform/module.rs
  - zircon_runtime/src/platform/service_types/driver.rs
  - zircon_runtime/src/platform/preferences/persistence/adapter.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_runtime/src/text/render_state.rs
  - zircon_runtime/src/text/parallel/raster_pool.rs
  - zircon_runtime/src/ui/surface/render/text_prewarm.rs
  - zircon_editor/src/core/settings
  - zircon_editor/src/ui/retained_host
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphInterfaces.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Async/TaskGraph.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/AsyncWork.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Private/NavMesh/RecastNavMeshGenerator.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCacheCompositeFont.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateSdfGenerator.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/SlateRenderer.cpp
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
tests:
  - runtime task implementation 22 of 22 Rust files reviewed
  - framework task descriptors 10 of 10 Rust files reviewed
  - 5119 lines and 41 inline tests inventoried
  - deterministic 32-file manifest SHA256 d00976288fa949dba4f887008aa29c564411733c615410f66884fa90483702ae
  - rustfmt edition 2021 check passed for 32 of 32 files
  - 2026-08-27 WGPU private pool construction sites 3 to 0; JobSystem static audit 3 of 3 passed
  - 2026-08-27 Navigation production private pool construction sites 1 to 0; 55 test constructors migrated
  - 2026-08-27 Platform implicit owner-selection routes 3 to 0; direct process-default sites 2 to 0; 42 driver and 4 adapter test constructors migrated
  - 2026-08-27 offline Font SDF implicit process-owner routes 1 to 0; process pool sets 3 to 0; one explicit TaskGraph owner added
  - current-source Cargo, product trace and power acceptance blocked
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Runtime TaskGraph 当前架构审查（2026-08-15）

## 范围与结论

已逐文件复读 `zircon_runtime/src/core/runtime/tasks/**` 22/22 个 Rust 文件和 `zircon_runtime/src/core/framework/tasks/**` 10/10 个 Rust 文件，共 5,119 行、41 条内联测试。清单按规范化相对路径排序，对每个文件计算 SHA256，再以 `hash + two spaces + path` 和 LF 拼接后计算总 SHA256，得到 `d00976288fa949dba4f887008aa29c564411733c615410f66884fa90483702ae`。32/32 文件通过 `rustfmt --edition 2021 --check`。

直接 owner/caller 另复核 navigation lazy service、Wgpu framework 构造、editor settings shutdown 和 scene artifact I/O；这些文件不计入 32 文件覆盖率。当前结论是：现有实现有可保留的任务终态、bounded admission、panic containment 和 process-default 共享行为，但物理三线程池、可旁路的私有线程池、串行 keyed-I/O pump、无界 Drop 等待和未接入产品的诊断共同构成 M1 结构瓶颈。它们应由单一 Runtime TaskGraph 硬切，不应继续在现有三池 API 上叠加缓存或局部算术修补。

当前源码没有可运行产品 binary；managed `zircon_app` build 在 324.2 s 后因 6 个 foreign runtime 编译错误失败，focused runtime lib-test 在 843.4 s 后编译失败，均执行 0 tests。因此本报告的线程数和复杂度是源码模型，不是 WPR、wall-clock 或功耗实测，模块仍留在 `pending.md`。

## 应保留的行为合同

- `TaskPools::process_default()` 以 `OnceLock` 让默认 `CoreRuntime` 共享 execution owner；当前 runtime-builtin graphics 产品路径也从 `core.task_pools().compute()` 注入已有 pool。
- `BoundedKeyedIoLane` 已有 entry/retained-byte 上限、deadline、coalescing、fence、typed terminal、observer 锁外通知和 panic containment；这些语义应迁移，不保留其当前数据结构与单 pump 所有权。
- `JobHandle` 能传播 dependency terminal/panic，并在被 poison 后恢复状态锁。非 Rayon 线程等待时，`rayon::yield_now()` 返回 `None` 并进入 condvar，不会初始化另一个 Rayon global pool；只有当前 Rayon worker 会协助所在 pool。
- process timer 以单独线程管理 deadline，避免每个 deadline 长期占住一个 I/O worker；目标实现应保留集中定时控制面。

## P0：线程预算合同不真实，插件可把 worker 数翻倍

`thread_assignment.rs:9-20` 先把 desired 限制到 remaining，再执行至少 1 的 clamp；remaining 为 0 时仍返回 1。`pools.rs:119-147` 因而可以报告 `total_threads` 小于三个物理 Rayon pool 的 worker 总数。现有 `zircon_runtime/src/tests/tasks.rs:30-37` 明确把 2 个 total threads 分配成 I/O=1、async-compute=1、compute=1，当作正确合同。

默认源码模型如下。1/2 核约束环境都创建 3 个 worker；当前进程可见 16 个逻辑处理器时，process pools 为 4+4+8=16。`DefaultNavigationManager::new` 在 `zircon_plugins/navigation/runtime/src/manager.rs:44-50` 以未指定 worker 数的 descriptor 新建 async-compute pool，`TaskPool::new` 会取全部 16 个可用处理器；lazy service 在 `zircon_plugins/navigation/runtime/src/lib.rs:50-64` 首次解析时触发。因此导航激活后的静态上限是 16 process workers + 16 navigation workers = 32 个 Rayon workers，尚未计 main/render/timer/OS 线程。

| 可用逻辑线程 | process report total | I/O + async + compute 实际 worker | navigation 激活后 Rayon worker |
|---:|---:|---:|---:|
| 1 | 1 | 3 | 4 |
| 2 | 2 | 3 | 5 |
| 4 | 4 | 4 | 8 |
| 16 | 16 | 16 | 32 |

Wgpu 当前 module-host 产品路径正确注入共享 compute pool，但 `construct.rs:169-223` 仍公开两个会创建 full compute pool 的构造器；当前 Rust 调用图未找到产品 caller，属于 dormant bypass API。`JobScheduler::default()` 同样会创建一个 full compute pool，当前直接 caller 位于测试，但公开默认值使后续 consumer 很容易再次产生私有 owner。硬切验收必须删除这些旁路，而不是只修导航一处。

## P0：bounded keyed I/O 有界但全局串行，队列维护可退化

`LaneState` 只有一个 `pump_active` 和一个 `active`；pump 在一个 scheduler job 中循环执行所有 work closure，互不冲突的不同 key 也不能并行。队列算法还包括：

- `insert_ordered` 线性找位置并在 `VecDeque` 中间插入，为 `O(Q)`。
- `coalesce_queued_generation` 多次全队列扫描，再循环 middle-remove；同 key storm 可达到 `O(Q^2)` 元素移动。
- `front_is_runnable` 每次取下一项前扫描全部 suspended admissions，为 `O(S)`；处理 Q 项可达 `O(Q*S)`。
- deadline 和 terminal-observer 路径按 ticket position 扫描/移除队列；fence planning 多次扫描 suspended/active/queue 并复制 prerequisite。
- fence failure 对每个 prerequisite 再递归扫描 prerequisite slice；最坏为 `O(P^2)`，并为每条递归链维护 `HashSet`。

现有测试覆盖 100,000 suspended admissions 和 1,000 项 storm 的语义，但没有对 activation/coalescing/fence 的 visits、moves、lock hold 或 p95 设门，因此不能证明规模算法合格。目标不是放大现有单 pump 的 worker 数，而是 keyed ready index + generation barrier：同 key 串行，不同 key 在 lane quota 内并行，coalesce 平均 `O(1)` 查找，ready/fence 更新为 `O(log Q)` 或摊销常数，并以 counter 证明。

## P0：shutdown guard 的 Drop 可无限阻塞主线程

`BoundedKeyedIoShutdownGuard::Drop` 无条件调用无限期 `wait()`；`wait_until()` 超时后 guard 若离开作用域，Drop 仍再次无限等待。editor retained host 在 event loop 结束后执行 settings persistence finish，失败路径也会 drop guard；scene artifact I/O 的 owner Drop 亦通过 shutdown guard 等待。这使一个阻塞 I/O closure、插件 callback 或失效文件系统能够把 F0 编辑器退出永久卡死。

硬切合同必须是 `Running -> Quiescing(deadline) -> Cancelling -> Draining -> Stopped/Abandoned`。main thread 只能推进 phase 或等待明确预算；超时后返回 typed incomplete report，并把尚未终止的状态交给进程生命周期 owner。不能简单删除 Drop wait，否则会把当前持久化/fence 语义改成静默丢数据。

## P1：调度语义和观测不足以支撑产品剖析

- `TaskPoolKind` 只有 Compute/AsyncCompute/Io；descriptor 没有 main/render/RHI affinity、priority、deadline、budget class、plugin/domain quota 或 cancellation。
- Rayon spawn 是无界提交；`JobScheduler::spawn` detached 且没有 backpressure/cancel。dependency 以每条输入一个 boxed callback 表示，terminal observer 在发布完成的线程同步执行，慢 observer 会占住 worker。
- task diagnostics 按 `JobScheduler` 实例持有，默认关闭；直接 task scheduler 的 `.with_diagnostics()` 和 `record_diagnostics()` 当前只在其单元/集成测试中调用，产品没有 process-level queue truth。
- `TaskPool` 不暴露 queue depth/age、active、steal、park/wake、affinity、pool-wide wait；explicit wait 也没有 caller/thread identity，无法区分 main-thread stall 与 worker cooperative wait。
- timer thread 在 `timer.rs:251-260` 同步执行所有到期 callback；一个慢 callback 会推迟其他 deadline，且没有 callback duration/late/budget 指标。

## Unreal 对照与目标 owner

Unreal `TaskGraph.cpp:722-810` 让 named thread 的目标队列空闲时阻塞；`1114-1132` 让 worker 通过共享 scheduler 找任务并在无任务时 stall；`1374-1415` 的 enqueue 保留 target/current thread，区分 named thread 与 worker；`1786-1790` 对总 TaskGraph thread 数设统一上限。Zircon 当前把 work kind 放大成三个物理 pool，并允许 subsystem/plugin 再造 full pool，既没有统一 affinity，也没有真实全局预算。

M1 的唯一 owner 固定为 `zircon_runtime::core::runtime::tasks::EngineTaskGraph`：

1. 一个受全局 worker budget 约束的共享 worker set；main/render/RHI 是 named executor/affinity lane，不以每个 kind 新建一套满规模 worker。
2. task descriptor 至少包含 label/domain、affinity、priority、budget class、prerequisites、cancel token、deadline、retained bytes 和 panic policy；submission 同时受全局及 domain/plugin count/bytes 配额限制。
3. ready graph 的 steady 执行与 ready vertices + changed dependency edges 成比例；keyed I/O 使用 key index 和 generation barrier，不同 key 有界并行。
4. always-on 低成本 process counters 提供 submitted/queued/active/completed/cancelled/panicked、queue depth/peak/age、execution/wait、steal/park/wake、caller affinity、deadline late 和 shutdown phase；高成本 samples 显式启用。
5. timer 只维护 timer wheel/deadline heap并把到期任务提交到相应 lane，不在 timer control thread 执行 foreign callback。
6. hard cut 同里程碑迁移 ECS、asset、editor background、navigation bake 和允许调度的 plugin work，随后删除三物理池、public private-pool constructors、detached bypass 与旧测试合同；不保留 alias/re-export/forwarding wrapper。

## 实施前 RED 门与动态验收

先把当前错误合同写成会失败的行为/结构门，再实现硬切：

- 1/2/N worker 配置下，实际 shared workers 不超过统一预算；当前 16-thread 主机激活 navigation 不增加第二套 16-worker owner。
- 1/1k/100k independent/dependent/keyed tasks 记录 submit allocations、queue visits/moves、lock hold、queue p50/p95/p99、steal/wake/wait；同 key 有序，不同 key 确实并行。
- 0/1/100k suspended、same-key storm 和 fence chain 证明 coalesce/index 不作 `O(Q^2)` middle-remove，fence resolution 不作 `O(P^2)` repeated scan。
- 阻塞 work、panic、cancel、deadline、plugin unload 和 process exit 矩阵中，quiesce deadline 到期后主线程在预算内返回 typed report；Drop 不做无界等待。
- `git grep`/结构测试证明 production direct `TaskPool::new`, `JobScheduler::default`, old three-pool owner 和 timer-thread callback execution 为 0；测试 fixture 必须显式构建 isolated executor。
- M0 current-source 产品恢复后，对 F0/F2/F4 各至少 3 次采集 WPR/xperf CPU sampling、context switch/wait、线程峰值、idle wakeups、RSS、I/O、wall time 和功耗；同 run id 绑定 scheduler counters。没有这些数据前不声明耗时/功耗接近 Unreal。

本轮不修改 Rust 实现：上述 P0 都跨 scheduler owner、插件构造、I/O 语义和退出生命周期；对现有三池公式或单 pump 做局部修复会延长双系统寿命，并与 Plan02 的 hard-cut 门冲突。

## 2026-08-27 current-source follow-up

后续 Runtime11 source slice 已按本报告结论开始硬切，而非调整三池公式：
`tasks/task_graph/EngineTaskGraph` 取代 Core 三物理池 owner，每个 CoreRuntime 只创建
一个精确 1/2/N 预算的共享 worker set，旧 Core pool selector 与 execution public
类型删除；scheduler 和已迁移 runtime/app/editor consumer 共用该 owner。dynamic
session 的关闭顺序也收敛为 scope drain -> module cleanup -> TaskGraph close/join。

源码 worker 数模型由 1/2 配置下的 3/3 个实际 worker 收敛到 1/2，16 配置仍为 16，
但物理 worker set 从 3 个变为 1 个。该结果只证明结构和预算公式，不能覆盖原报告的
全部 P0：affinity/priority/quota、keyed-I/O 异 key 并行、timer/process/private owner、
managed Cargo、WPR/RSS/功耗与 1/1k/100k task 矩阵仍未完成。详细记录见
`docs/plans/optimize/zircon_runtime/11/2026-08-27-engine-task-graph-shared-worker-owner.md`；
在动态证据完成前，本报告 front matter 的 `static_complete_dynamic_blocked` 仍成立。

### Graphics private-owner hard cut

同日对 WGPU construction 全调用链复核后，原报告只标出的两个 public bypass 实际还被
`new`、plugin extension 和 startup-report convenience chain 间接使用；`construct.rs`
共有 3 个 `TaskPool::new(TaskPoolDescriptor::compute())` 站点。参考 Unreal Core 显式
`FTaskGraphInterface::Startup/Shutdown/Get`、Renderer 只消费该 owner，以及 Fyrox
`EngineInitParams { task_pool: Arc<TaskPool> }` 的注入合同后，当前源码已将全部 6 个
production-visible WGPU constructor 改为显式接收 `TaskPool`，并删除无调用的隐式 Solari
constructor。module-host、PBR viewer 和测试/plugin fixture 均从其保留的 CoreRuntime
TaskGraph 注入同一 worker owner。

静态结果为 Graphics private-pool allocation sites `3 -> 0`，tracked+untracked 旧单参数
`WgpuRenderFramework::new(...)` 调用 `0`，JobSystem audit `3/3`，focused rustfmt/diff check
通过。在该切片的顺序点，Navigation 仍有 1 个 production `TaskPool::new`；全仓原始文本
扫描还包含多个 inline `#[cfg(test)]` fixture 与未收敛的 timer/text/asset owner family，
因此不能宣称 private owner 全仓归零。该切片没有修改渲染算法，也没有运行 Cargo、WPR
或功耗采集，不构成耗时/功耗改善或接近 Unreal 的声明。

### Navigation private-owner hard cut

同日对 Navigation 模块描述符、lazy driver factory、manager state、同步/异步 bake、dirty bake、
tile dispatch 与全部直接构造测试复核后，确认唯一 production 私有 owner 位于
`DefaultNavigationManager::new()`。Unreal `FRecastNavMeshGenerator` 从
`FTaskGraphInterface::Get().GetNumWorkerThreads()` 推导 tile 并发上限，tile task 的
`StartBackgroundTask()` 默认进入 Core 的 `GThreadPool`；NavigationSystem 不创建第二套通用
worker owner。因此 Zircon 将 manager 硬切为 `new(bake_pool: TaskPool)`，删除 `Default`，
Navigation 模块声明 `TasksModule` 依赖并从 activation Core 注入 TaskGraph worker。

55 个 unit construction 迁移到显式、保活、两 worker 的 isolated TaskGraph fixture。
现有 Recast tile 算法、每 tile dispatch、Arc 共享 plan、panic containment 与相关 benchmark
均未修改。静态结果为 Navigation production private-pool sites `1 -> 0`；按本报告的 16
逻辑处理器源码模型，Navigation 激活的额外 worker 为 `16 -> 0`，组合 Rayon 上限为
`32 -> 16`。JobSystem audit `3/3`、focused rustfmt 与 diff check 通过。该结论只关闭
Navigation production owner；全仓仍有 inline test fixture 及 timer/text/asset 等独立 owner
需要按各自调用链继续复核。没有运行 Cargo、WPR、RSS 或功耗采集，不能据此宣称实际
耗时/功耗改善或接近 Unreal。

### Platform preference-executor owner hard cut

同日继续复核 `PlatformModule`、`PlatformDriver`、preference persistence、App 自定义 descriptor
factory、preference adapter 与直接测试构造入口。内置 factory 已从 activation Core 注入
TaskGraph owner，但 `PlatformDriver::default()` 仍选择 process default，且 App 安装
host-provided backend 的 factory 通过 `with_preference_storage_backend(backend)` 间接走回
该 owner；crate-private `PreferencePersistenceAdapter::new(...)` 还直接选择 process I/O pool，
合计形成三个 production-compiled implicit owner routes。

参考 Unreal Core 集中 `FTaskGraphInterface::Startup/Shutdown`、`Async(ThreadPool)` 与
`FAsyncTask::StartBackgroundTask()` 默认消费 `GThreadPool`，并确认 ApplicationCore 没有另建
通用 executor 后，Zircon 删除 `PlatformDriver::default()`，把 backend constructor 硬切为
`with_preference_storage_backend(pool, backend)`，并把 adapter 硬切为 `with_pool(...)`。内置与
App override factory 都注入当前 Core TaskGraph worker；42 个 driver/manager 测试入口与四个
adapter 入口改用显式保活的单 worker I/O fixture，且先 drop consumer、后 drop pool。

静态结果为 Platform implicit owner-selection routes `3 -> 0`、直接 process-default sites
`2 -> 0`、driver default test calls `42 -> 0`、adapter implicit test calls `4 -> 0`，
JobSystem audit `3/3`。
偏好存储、platform host、window state 与 event-loop 算法均未修改。没有运行 Cargo、WPR、
wall-clock、RSS、idle wakeup 或功耗采集，因此不能据此声明实际性能/功耗改善或接近 Unreal；
timer、text、asset standalone owner 与其他 private-owner family 仍需继续复核。

## 下一阶段结构优化准入（尚未实施）

### Callback continuation owner

当前普通 `JobScheduler::from_pool(...)` 已在目标 TaskPool 上构造 dispatcher；process dispatcher
只由无 owner 的 `JobHandle::completed/default` 与 process timer 进入。但 dependency callback
被保存在 source `JobHandle`，并由 source handle 的 dispatcher 发布，所以跨 Runtime 依赖会把
consumer launch continuation 送回 producer owner。删除 `OnceLock`、改成 inline callback 或只给
timer 换 pool 都无法修正这个语义，反而可能破坏异步 observer 顺序和 shutdown census。

实现前必须先把 continuation envelope 改造方向写成 RED 合同：envelope 显式携带 target
executor/scope authority；source terminal 只在释放 state lock 后请求 target dispatch；target 已关闭时
发布 typed prelaunch terminal，而不是回退到 source/process owner。`completed()` 不得物化 worker，
空依赖 sentinel 应由 consumer scheduler 创建或由 schedule API 内部表达。profile 矩阵覆盖
1/1k/100k chain/fan-out、同 owner/跨 owner、target 先关闭/source 先关闭、observer panic 与 1/2/N
workers，记录 callback queue visits、runner wake、lock hold、p50/p95/p99 terminal-to-launch、shutdown
drain 和 retained bytes。该证据完成前不修改 continuation 算法。

### Text execution owner

当前 `TextRenderState::new_with_process_raster_worker_budget()` 同时从 legacy process compute pool
构造 SDF scheduler，并按 process async-compute 线程数另建 `zircon-text-raster-*` 私有 worker；UI
shape prewarm 仍直接解析 process compute pool。按本报告 16 逻辑处理器模型，
运行中 Core TaskGraph 已有 16 workers 时，text 首次进入可再物化 `4+4+8=16` legacy workers 和
4 raster workers，静态组合上限为 `16 + 16 + 4 = 36`，即 text 路径额外 `20`。这是源码模型，
不是实测线程峰值。

2026-08-27 非算法 owner 切片已把离线 `.zsdf` bake 从该 process-default 路径中移除：库 API
要求调用方传入 `&TaskPool`，CLI 显式创建一个 `EngineTaskGraph` 并在写盘前取得有界 shutdown
receipt。离线工具的隐式 owner route 为 `1 -> 0`，物理 worker set 为 `3 -> 1`。在 16 逻辑
处理器模型下，旧三池配置 `4+4+8=16` 与新单图 `16` 的静态 worker 总数相同，但 generation
可用并行度从 Compute split 的 `4` 变为 `16`。这只证明 topology/lifetime 收敛；小 glyph batch
可能因并行开销或唤醒增加而回退，必须纳入下述 cold/warm、0/1/8/1k glyph Windows profile。

Unreal Slate 的 lazy font load 与 SDF generator 使用 `FAsyncTask::StartBackgroundTask()` 进入 Core
`GThreadPool`；SDF 通过有限 `FAsyncTask` 对象池返回 `BUSY`，update/flush 收割或等待任务，字体
cache 分离 Game/Render owner，完成后向 GameThread 发布失效。Zircon 的目标方向因此不是把
`TaskPools::process_default()` 替换成另一个全局，而是保留现有 request/completion byte budget、
face epoch、batching 与 backpressure 语义，将 raster/SDF/shape work 收编到 Runtime TaskGraph，
并把 cache publication 留在 UI/renderer owner thread。是否需要一个 serial raster lane、每 worker
rasterizer state 或 bounded task-object pool，必须由 profile 决定。

Windows profile 在实现前采集 cold/warm cache、bitmap/SDF、0/1/8/1k unique glyphs、1/16/64
surfaces 与 queue saturation；记录 worker peak、CPU sampling、context switches、idle wakeups、
request/completion bytes、queue p50/p95/p99、frame p50/p95/p99、Drop/join、RSS 与功耗。同机同负载
与 Unreal Slate 采样绑定 run id。没有这些数据，不宣称 text 算法、耗时或功耗达到 Unreal 经验值。
