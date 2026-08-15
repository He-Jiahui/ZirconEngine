---
related_code:
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/runtime/events
  - zircon_runtime/src/core/runtime/tasks
  - zircon_runtime/src/core/framework/events.rs
  - zircon_runtime/src/core/framework/tasks
  - zircon_runtime/src/core/runtime/tests/events
  - zircon_runtime/src/tests/tasks.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_store.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/asset/pipeline/worker_pool
  - zircon_runtime/src/operation/maintenance.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
reference_engines:
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
  - dev/bevy/crates/bevy_ecs/src/message/message_cursor.rs
  - dev/godot/core/object/worker_thread_pool.cpp
  - dev/godot/core/object/message_queue.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Async/Fundamental/Scheduler.cpp
---

# 02 · Core Runtime Event 与 Task Execution 工程化差距

## 1. 结论

当前实现不是简单的 demo：event bus 已有明确的 lossless/drop-oldest/latest policy、同 topic 发布序、共享 payload、取消订阅和队列诊断；task 层也有三类 pool、依赖 continuation、worker 内协助等待、panic 记录、bounded keyed I/O 的容量/截止期/取消/停机保护，以及数量可观的并发行为测试。

但这些局部能力没有收敛为“动态 runtime library 可以安全卸载”的统一执行所有权。`CoreRuntime::new` 必然取得 `OnceLock` 中的进程级 Rayon pools，而 dynamic session destroy 没有停止 admission、排空任务或 join worker；process timer 也被 `OnceLock` 永久持有。对 `cdylib` 来说，这意味着 host 收到 destroy 成功并卸载代码后，旧 worker 仍可能等待或执行库内函数。该项为 P0，并直接扩展 `01` 的产品 shutdown 缺口。

本轮确认 1 项 P0、6 项 P1 和 4 项 P2。event 与 task 必须共同审查，是因为跨线程事件接收、deadline timer、asset/operation maintenance 和 module shutdown 都依赖同一个 cancellation/quiescence 边界；分别修局部 API 会继续制造无法证明停机的后台执行路径。

## 2. 当前实现闭环

### 2.1 Event bus 已有基础

- topic registry 使用 `Mutex<HashMap<String, Arc<EventTopic>>>`；每个 topic 维护 copy-on-write subscriber snapshot 与独立 delivery mutex（`events/topic.rs:12-18,120-148`）。
- 发布在同一 topic 的 delivery lock 内遍历 subscriber snapshot，payload 只构造一个 `Arc<EngineEvent>`，断开的 subscriber 会在本次发布后移除（`events/publish.rs:18-55`）。
- subscriber 队列支持不设容量的 `Lossless`、固定容量丢最旧和只保留最新；`recv/try_recv/recv_timeout` 与 subscription drop 均有行为实现（`events/subscriber.rs:69-95,110-239`）。
- 行为测试覆盖 fan-out 指针共享、顺序、容量峰值、丢弃、queue age、总线 drop、超时溢出、同/不同 topic contention 和订阅竞态。三组 ignored benchmark 输出 publish p50/p95、diagnostics ratio 与 bounded-pressure RSS 证据。

### 2.2 Task 层已有基础

- `TaskPools` 分为 I/O、async compute、compute；`CoreRuntime` 的 scheduler 复用 compute pool（`tasks/pools.rs:17-92`、`runtime.rs:37-57`）。
- `JobScheduler::schedule_after` 使用 terminal continuation，不占用 worker 阻塞等待依赖；`JobHandle::wait` 在 pool worker 上会先协助执行 Rayon 工作（`tasks/job_scheduler.rs:85-160`、`tasks/job_handle.rs:146-176`）。
- scheduled task 的 panic 被收集到 handle，wait 时传播；scheduler diagnostics 记录 queued/active/completed/panicked 和等待耗时。
- bounded keyed I/O 已具备 entry/byte budget、key coalescing、fence、deadline、取消 authority、shutdown guard 和压力行为测试。这一层应作为统一 execution service 的成熟输入，不应在重构中退化。

### 2.3 生产停机调用链

`zircon_runtime` 同时产出 `rlib` 和 `cdylib`（`zircon_runtime/Cargo.toml:8-9`）。每个 `CoreRuntime::new` 都调用 `TaskPools::default`，后者 clone `PROCESS_TASK_POOLS: OnceLock<TaskPools>` 中的三个 pool（`runtime.rs:37-50`、`tasks/pools.rs:25-33,95-98`）。

dynamic destroy 会阻止新 session action、等待 wake callback，然后只调用 `RuntimeDynamicSession::shutdown_before_library_unload`；该函数只关闭 plugin event subscriptions、project watchers 和 process log，成功后直接 drop session（`dynamic_api/session/registry/session_store.rs:91-131`、`dynamic_api/session/state.rs:141-162`）。task pools、scheduler 与 process timer 不在成功条件内。

timer 并非只存在于测试：bounded I/O deadline、asset worker completion 和 operation maintenance 都会懒加载 `TaskTimer::process_default`。`PROCESS_TIMER` 保存一个永久强 owner，因而 `TaskTimer::drop` 中仅最后 owner 才执行的 closing + join 无法在 session teardown 到达（`tasks/timer.rs:18-24,63-71,170-201`）。

配置持久化又提供了同类反例：`ConfigPersistenceWorker` 直接创建 `zr-config-persist` OS thread；其 Drop 最多等待两秒，超时后从 mutex 中 `take()` 出 `JoinHandle`，但仅在 `worker_exited == true` 时 join。超时分支让 handle 被直接 drop，worker 仍可在后台执行 snapshot、JSON 序列化、原子写入与 Zircon tracing 代码（`foundation/runtime/config_manager/worker.rs:56-66,151-211`）。commit fence 只能阻止迟到提交覆盖新 generation，不能证明线程已经离开 DLL 代码。

## 3. 差距清单

### P0-1：进程级 task/timer worker 越过 dynamic session 和 DLL unload 边界

**证据**

- `CoreRuntime::new` 无条件初始化 process-default task pools；`OnceLock<TaskPools>` 在库映像存活期永久持有 Rayon pools。
- `TaskPool` 只有 `spawn/install/join`，没有 close-admission、cancel/drain 或显式 worker join（`tasks/pool.rs:21-83`）。drop 最后一个 runtime clone 也不会 drop 静态 owner。
- process timer 虽然实现最后 owner drop 时 join，但 `OnceLock<Result<TaskTimer, String>>` 自身就是永不释放的 owner；callback 在专用线程上调用库内闭包（`tasks/timer.rs:180-201,251-262`）。
- config persistence worker 的 timeout 分支取消 commit fence 后丢弃未退出线程的 `JoinHandle`；这保护部分文件提交顺序，却不保护动态库代码寿命。
- dynamic destroy 的 `true` 不检查 pool/timer/queued callback。host 因而可能在仍有线程以 Zircon DLL 代码为入口或返回地址时执行 `FreeLibrary`。

**后果**

这是代码卸载安全问题，不只是线程泄漏。最轻表现为多次 session 创建后线程/任务跨 session 污染；严重时 worker 醒来进入已卸载代码、旧 callback 访问已释放 service，造成进程崩溃或未定义行为。静态执行器在单一进程二进制中可以是可接受取舍，在可热卸载 `cdylib` 中则必须由 host 生命周期覆盖或在卸载前显式停止并 join。

**目标契约**

由 host 或唯一 `RuntimeOwner` 持有 `ExecutionRuntime`。shutdown 必须按固定事务执行：关闭新提交 → 取消 `CancelOnShutdown` scope → 等待/处理 `FinishOnShutdown` → 清空 timer registration → join timer 与全部 worker → 证明没有 DLL callback/instruction pointer → 才允许 session destroy 返回可卸载。任何超时、panic 或未认领 task 都返回 teardown-incomplete，host 不得卸载。

### P1-1：detached 与 tracked task 使用冲突的 panic/结果契约

`JobScheduler::spawn`/`TaskPool::spawn` 返回 `()`；detached closure 不经 `catch_unwind`，真实 Rayon worker panic 走进程终止默认路径。测试甚至通过子进程明确断言“real Rayon detached panic must retain its process-terminating default”（`tasks/job_scheduler.rs:244-250,306-379`）。同一 scheduler 的 `schedule` 却 catch panic、把字符串存入 handle，再由 `wait()` panic（`:253-264`、`tasks/job_handle.rs:146-157`）。

调用者很难从 API 名称判断一个后台错误会杀死整个编辑器、延迟到 wait 才 panic，还是无人 wait 而静默滞留。目标是统一的 typed terminal result 与 error policy：默认 `spawn<T>` 返回 `Task<T>`；显式 `detach` 必须声明 owner、panic sink 和 shutdown policy。真正 fatal 的任务使用单独 API/descriptor，不靠 Rayon 默认行为偶然实现。

### P1-2：framework task contract 与实际 scheduler 未形成同一状态机

framework 已声明 `AsyncTaskDescriptor { handle, pool, label, cancellation_policy }`、`CancelOnDrop/DetachOnDrop/FinishOnShutdown`、`AsyncTaskStatus` 与 `TaskPollBudget`。但 generic `JobScheduler/JobHandle` 不接收 descriptor，不执行 cancellation policy，也没有 label/owner/typed output。`TaskPollBudget` 在 production 搜索没有调用点；scene spawn/asset reload 各自用 atomics 和 mutex 手工实现部分 cancellation/status。

这会让 API 表面看似具备工程化任务契约，实际每个子系统重新解释 drop、cancel、failure 和 shutdown。目标是一个 canonical `Task<T>`/`TaskScope` 状态机拥有 descriptor、result、cancellation token、deadline 与 provenance；scene/asset/plugin/graphics 只使用它，不再维护平行的 task status。

### P1-3：generic scheduler 没有 admission、容量、owner 和 shutdown，而成熟约束停留在 bounded I/O 孤岛

`JobScheduler::schedule/spawn` 直接把闭包交给 Rayon，没有队列容量、subsystem quota、priority、deadline、owner、取消或拒绝结果。相反，bounded keyed I/O 已实现 entry/byte capacity、coalescing、fence、deadline、cancel authority 与 shutdown guard。

这种分裂会使新增任务自然绕过更严格的入口，并让 runtime shutdown 无法枚举“谁仍持有工作”。目标不是删除 bounded I/O，而是让它成为 `ExecutionRuntime` 的受约束 lane；所有工作都必须属于 `TaskScope/SubsystemId`，提交返回 accepted/rejected reason，shutdown 能按 scope 关闭 admission、统计和 drain。

### P1-4：线程预算不是全局事实，公共构造器还能创建完整独立池

thread assignment 把每个 pool 的最小值 clamp 为 1，即使 remaining 已为 0。测试明确接受 `total_threads == 2` 但三个 pool 各 1 个 worker（`tasks/thread_assignment.rs:8-21`、`src/tests/tasks.rs:29-37`）；report 的 total 因而不是实际 worker 总数。

此外 `JobScheduler::default` 创建一个按全部 CPU 数配置的新 compute pool（`tasks/job_scheduler.rs:29-32`），graphics 的公开 render-framework constructor 也直接创建完整 compute pool（`graphics/.../construct.rs:169-223`），asset/project 等路径还创建独立单线程 pool。多个 subsystem 组合后线程数、affinity、stack 和优先级不再由 runtime 统一治理。

目标是一个可观测的 process/session worker budget：实际 worker 总和必须等于 report；subsystem 请求的是 lane/并发配额，不是随意 new OS pool。需要专用线程的设备、I/O 或实时域必须通过 descriptor 声明原因、affinity/priority/stack 和 shutdown owner。

### P1-5：event `Lossless` 是无界队列，publisher 无法知道交付或背压结果

`Lossless` 映射到 `capacity = None`，每个慢 subscriber 都可无限积累 `Arc<EngineEvent>`；总线没有 per-topic/per-subscriber/global byte budget。`publish` 返回 `()`，无 subscriber、发生 drop、subscriber disconnected 或资源压力对 producer 都不可见（`events/subscriber.rs:69-95`、`events/publish.rs:8-55`）。

在高频资产变化、编辑器遥测、文件监控或网络桥接中，单个暂停 consumer 即可造成长期内存增长。目标的跨线程 control bus 必须使用显式 bounded policy，返回 delivery receipt（delivered/dropped/no-subscriber/backpressured/closed），同时记录 event count 与 retained bytes。真正 lossless 流必须有 producer throttle、spill-to-disk 或有限事务 owner，不能等同于无界内存。

### P1-6：string + JSON 总线混合了帧内数据流、控制消息与 ABI 镜像

`EngineEvent` 只有 `topic: String` 和 `serde_json::Value`（`core/framework/events.rs:11-15`）。这一格式适合调试/FFI 边界，但若作为通用 engine event，会把类型校验、schema/version、分配和序列化成本带入帧内热路径；它也没有 sequence、source/session、timestamp、trace 或 schema identity。阻塞 `Condvar` subscription 又没有 scheduler waker/cancellation token，无法自然加入 async task scope 和 shutdown deadline。

目标必须拆成三种边界：world/frame-local 强类型 `Messages<T>`；跨线程 bounded `ControlBus`；跨 ABI/plugin 的 versioned serialized envelope。三者可以通过显式 adapter 连接，但不能继续由同一个 string/JSON API 承担不同一致性和性能目标。

### P2-1：timer 单线程串行执行任意 callback，panic 被静默吞掉

timer 容量固定 512，所有到期 callback 由一个线程串行运行；callback panic 被 `catch_unwind` 后直接丢弃（`tasks/timer.rs:15-18,116-124,251-260`）。一个慢 callback 会推迟所有后续 deadline，panic 也没有 task identity、diagnostic 或 owner failure policy。

目标 timer 只负责 deadline ordering 和 wake，把非微小 callback 投递到受控 lane；每项有 owner/id/deadline/late-by/cancel/result，panic 进入统一 task terminal report。控制面若保留 inline callback，必须有严格类型、执行预算和超限诊断。

### P2-2：event diagnostics 默认启用，但 managed benchmark 没有回归门槛

`EventBusDiagnosticsMode` 默认 `Enabled`（`core/framework/events.rs:24-29`）；publish 和每个 subscriber delivery 会读取时间并更新多个原子统计。现有三项 benchmark 全部 `#[ignore]`，主要打印 p50/p95、ratio 和 RSS；没有平台基线、最大允许 overhead、噪声处理或 CI 回归判断。

不能据此宣称性能不足，也不能宣称已优于参考引擎。M0 应先建立可重复 harness，区分 diagnostics on/off、1/2/5/100 fanout、64B/4KiB/256KiB payload、contention 和 paused consumer，记录 allocation、p50/p95/p99、throughput 与 retained bytes，再决定默认采样策略。

### P2-3：poison recovery 一律继续使用可能已破坏的不变量

event topic、subscriber queue、timer state 和 scheduler task lock 多处使用 `poisoned.into_inner()`。现有 poison tests 多在持锁后立即 panic，未先部分修改核心 invariant，因此只证明“锁还能打开”，不证明中途 panic 后状态仍合法。

目标按数据结构区分策略：纯缓存可重建；transactional state 要校验/回滚 journal；无法证明一致性的 scheduler/event owner 进入 `Poisoned` 并关闭 admission。测试必须在修改 queue counters、subscriber snapshot、timer maps 和 pending task 后注入 panic，验证 conservation 或 fail-closed。

### P2-4：pool 构造失败用 `expect`，执行资源无法参与产品错误恢复

`TaskPool::new` 对 Rayon pool build 使用 `expect("zircon task pool")`（`tasks/pool.rs:21-37`）。线程创建失败、stack/OS 限制或部分 pool 初始化失败都会 panic，而不是返回可关联具体 pool descriptor 的 typed error，也没有对已创建 pool 的 rollback 证明。

目标为 `ExecutionRuntime::try_new(config) -> Result<ExecutionRuntime, ExecutionInitError>`，记录 requested/created workers 与 OS error；多 pool 初始化失败要停止并 join 已创建 worker。host 决定降级、重试或 fail-fast，底层构造器不擅自 panic。

## 4. 参考引擎证据与适用边界

| 参考 | 已核对机制 | Zircon 应吸收 | 不应误读 |
|---|---|---|---|
| Bevy task pool | pool 直接拥有 worker `JoinHandle` 与 shutdown channel（`task_pool.rs:135-142,160-221`）；`spawn<T>` 返回可 cancel/detach 的 `Task<T>`（`:551-581`）；Drop 关闭 channel 并逐线程 join（`:609-620`）；scoped task drop 取消未完成任务（`:690-700`） | typed result、显式 detach/cancel、worker ownership、scope cleanup、joinable shutdown | Bevy 的全局 pool 面向常驻 App；不能直接证明 Zircon DLL 热卸载或跨 session 静态池正确 |
| Bevy ECS messages | `Messages<M>` 强类型双缓冲、单调 message id、cursor；每次 update 交换并清理旧 buffer（`messages.rs:95-146,175-217`） | frame-local typed stream、有限 retention、reader cursor、批量写入 | 它不是跨线程可靠消息队列；不能替代 bounded control bus |
| Godot | WorkerThreadPool `finish` 切到 exit runlevel、等待所有线程、清任务，析构再次保证 finish（`worker_thread_pool.cpp:857-928`）；CallQueue 以 4KiB page 和 `max_pages` 设内存上限，push 返回 `Error`（`message_queue.h:41-47,73-76,106-147`） | 明确 stop/join、未认领任务诊断、bounded queued-call memory、producer error | Godot 的 singleton 和 Variant/Callable 消息格式不是 Zircon hot path 类型目标 |
| Unreal | scheduler stop 由 game-thread owner 执行，先启动 waiting queue shutdown，再 join 全部 workers，最后可 drain global queue；restart 显式 stop/start，并配置 foreground/background priority 与 affinity（`Scheduler.cpp:462-516,545-556`） | 停机 owner、admission/shutdown 顺序、join barrier、drain policy、priority/affinity 治理 | Unreal 静态全局调度器服务于常驻进程；Zircon 仍需额外的动态库代码寿命证明 |

共同原则是 execution owner 必须知道每个 worker、task 和 callback 的归属，并在释放代码/服务前建立 join/drain barrier；消息系统则按数据局部性、一致性和序列化边界分层，而不是追求单一万能总线。

## 5. 目标架构

### 5.1 Execution ownership

- `ExecutionRuntime`：唯一创建/拥有 worker 与 timer thread；构造可失败，shutdown 可等待并报告 blocker。
- `WorkerDomain`：`Compute/Io/Background/Control` 等稳定 domain；配置实际 worker count、priority、affinity、stack 与 queue budget。
- `TaskScope`：绑定 runtime session/module/subsystem owner，拥有 admission gate、cancellation token、deadline、task census 和 drop/shutdown policy。
- `Task<T>`：typed terminal result；drop 默认 cancel，detach 必须显式指定 sink/owner；panic 转为 typed failure，fatal policy 由上层声明。
- `TaskDescriptor`：统一现有 async descriptor 与 scheduler metadata，至少含 id、label、owner、domain、priority、deadline、cancellation/shutdown policy 和 trace provenance。

动态 runtime 可选两种实现，但只能选择一个 canonical owner：host 创建 execution service 并通过 versioned ABI capability 传入 DLL；或每个 runtime-library instance 自己拥有可显式 stop/join 的 execution runtime。禁止在可卸载 DLL 内保留 process `OnceLock` worker。

### 5.2 Event boundaries

- `Messages<T>`：world/schedule/frame owner 内的强类型、批量、有限帧 retention；不做 JSON，不提供阻塞 recv。
- `ControlBus<T>` 或 erased typed envelope：跨线程、明确 bounded policy 和 producer receipt；支持 sequence/source/session/trace、shutdown cancellation 和 async wake。
- `PluginEventEnvelopeVn`：跨 ABI/动态插件使用 version/schema id、长度/所有权明确的序列化 payload；与内部类型通过 adapter 转换。
- diagnostics 作为采样 observer，不改变交付语义；hot path 默认成本必须由 benchmark gate 决定。

### 5.3 Shutdown transaction

```text
Running
  -> CloseAdmission (all task scopes, control buses, timers)
  -> Cancel (drop/cancel policies; no new callbacks)
  -> Drain (finish policy, in-flight event/task consumers, deadline)
  -> Join (timer and every worker; verify thread census)
  -> ReleaseServicesAndModules
  -> DestroySession
  -> UnloadLibrary
```

该顺序由 `01` 的 `RuntimeOwner/LifecycleCoordinator` 触发。module cleanup 不能先于其 scope quiescence；DLL unload 不能只凭 Rust object drop 推断 worker 已退出。

### 5.4 Hard cut

- 不保留 process-static pools/timer 与 session-owned `ExecutionRuntime` 两条生产路径。
- 不保留 `JobScheduler::spawn -> ()`；调用方迁移到 typed task 或显式 fatal/detached API 后删除旧入口。
- 不让 `AsyncTaskDescriptor/Status` 继续作为仅 scene 使用的 DTO；合并进 canonical task core 后删除各子系统手写状态机。
- 不把 string/JSON `EngineEvent` 继续扩展成所有事件的兼容中心；按三类边界迁移并删除万能内部入口。
- 不允许 subsystem public constructor 隐式创建 full-size pool；必须注入 execution capability。

## 6. 测试先行重构里程碑

| 里程碑 | 先写的失败证据 | 实现范围 | 晋级条件 |
|---|---|---|---|
| M0 | 真 DLL load/create/destroy/unload/reload、worker/timer/config-persistence census、卸载后 callback sentinel、detached panic、配置 writer 超时 | 只建动态库与故障注入 harness | 当前静态与 detached worker 风险稳定复现；destroy success 与可卸载分开断言 |
| M1 | close admission、queued/running task、cancel/drop/detach、finish deadline、partial pool init | `ExecutionRuntime`/domain/scope/typed task | 所有 worker 可枚举并 join；typed terminal conservation 成立 |
| M2 | bounded I/O 与 generic task 的 quota、priority、deadline、owner diagnostics | 把 bounded I/O 收敛为 execution lane | 所有生产提交通过 canonical admission，无旁路 scheduler |
| M3 | timer slow/panic/cancel/teardown、callback late-by | timer 只做 deadline/wake，接入 scope | timer 退出可证明；慢 callback 不阻塞其他 deadline |
| M4 | lossless paused consumer、publisher receipt、shutdown wake、poison-after-mutation | bounded control bus + failure policy | 无无界 production queue；所有 wait 可取消并随 owner 关闭 |
| M5 | typed frame messages 的 order/cursor/retention/batch/perf | world-local `Messages<T>` 与 adapter | 高频内部事件无 string/JSON/Condvar；retention 明确 |
| M6 | ABI schema/version/ownership、plugin mirror teardown、session recreate | versioned plugin event envelope | DLL 边界无 Rust object/closure 泄漏，旧 schema 有 typed negotiation failure |
| M7 | workspace consumers、thread budget、long-run pressure、performance baseline | 删除旧 pool/scheduler/event API | 无双轨/shim；产品停机和 profile gate 全通过 |

M0-M3 与 `01` 的 shutdown M0-M4 属于同一个最低层修复序列；实施计划必须合并依赖，不能分别引入两个 runtime owner。

## 7. 验收矩阵

### 7.1 动态库与停机

- Windows host 循环 1K 次 load/create/work/destroy/free/reload；每轮 worker/timer/callback census 回到基线。
- destroy 与 queued/running/detached/finish-on-shutdown task 竞争；成功时所有 join 完成，超时时返回 teardown-incomplete 且不卸载。
- task、timer、event consumer 在 DLL 边界前后记录代码 generation；旧 generation 在 unload 后零执行。
- worker panic、timer panic、pool init partial failure 和 shutdown callback reentrancy 均有确定结果。

### 7.2 Task correctness 与规模

- accepted/rejected/completed/failed/cancelled 总量守恒；typed result 只消费一次，等待和 observer 不丢终态。
- scope drop、explicit cancel、deadline、parent shutdown 与 explicit detach 的优先级矩阵明确。
- 1/2/8/32 worker、深 dependency chain、宽 fan-out、nested wait、mixed I/O/compute 压力无死锁或饥饿。
- report 的 configured/created/live/busy worker 与 OS 线程 census 一致；小主机不再出现 `total=2` 但实际 3 workers。

### 7.3 Event correctness 与压力

- 0/1/5/100 subscriber 的 order、fan-out、unsubscribe race、shutdown wake 与 publisher receipt。
- 每种 bounded policy 同时限制 event count 和 retained bytes；暂停 consumer 长时压力内存保持上限。
- frame-local typed messages 验证 cursor、两帧 retention、batch、reader lag 与 world reset。
- control bus poison-after-mutation 要么恢复 conservation，要么关闭 admission；不得盲目继续。

### 7.4 性能

- task submit/start/complete latency、work stealing、nested wait、cancel storm：p50/p95/p99、throughput、allocation 和 CPU utilization。
- typed messages 对比当前 string/JSON bus：64B/4KiB payload、批量 1/32/1K、1/5/100 readers。
- control bus diagnostics on/off、contention、paused consumer 和 byte budget；基线含噪声区间与回归阈值。
- profile 必须按 Debug/Release、Windows/Linux 和 1/8/32 logical cores 分层；无数据时不宣称优于 Unreal 或其他参考。

## 8. 既有计划纠正

1. `runtime/07-runtime-performance-hotpath.md`：其 event ordering、bounded policy 和 diagnostics 行为可作为已完成局部证据；“performance baseline”若只依赖 ignored print benchmark，不应表示有持续回归门槛。需要重开 lossless budget、typed/frame boundary、publisher receipt 与 shutdown cancellation。
2. `frameworks/02-module-kernel-and-lifecycle-unification.md`：task/timer join 是 module cleanup 与 DLL unload 的前置条件；不能把线程 teardown 留给各 service 的手工 cleanup。
3. 后续 diagnostics `03` 必须统一 scheduler/event/timer/bounded-I/O 的 owner/label/trace 与采样策略，不应继续增加彼此独立的 snapshot。
4. Graphics、asset、scene、operation 专篇需要登记自己的 pool/task 调用点，但 canonical worker ownership 只由本篇拥有，禁止各域另建 execution core。

## 9. 工作区复核标记

本轮 tasks、diagnostics 和部分 graphics 文件存在其他会话修改。P0 证据落在 `runtime.rs`、`tasks/pools.rs`、`tasks/timer.rs` 与 dynamic session teardown 的 current source；开始 M0 前必须重新读取这些文件，并检查其他会话是否已加入 execution shutdown。若加入，只能以真实 DLL unload/thread census 测试把 finding 降级，不能仅凭新 API 名称关闭。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
