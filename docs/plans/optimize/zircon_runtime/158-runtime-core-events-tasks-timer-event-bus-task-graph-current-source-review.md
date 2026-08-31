---
title: Runtime Core Events、Tasks、Timer、Event Bus 与 Task Graph 当前源码复核
category: zircon_runtime
report_id: Runtime158
review_date: 2026-08-29
baseline_head: b2e76ff33cc298ad76f7b801a1d06d1e2faa046d
canonical_owner: Runtime02
refreshes:
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
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
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/src/manager.rs
  - zircon_runtime/src/platform/module.rs
  - zircon_runtime/src/platform/service_types/driver.rs
  - zircon_runtime/src/platform/preferences/persistence/adapter.rs
  - zircon_app/src/entry/engine_entry.rs
reference_engines:
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
  - dev/bevy/crates/bevy_ecs/src/message/message_cursor.rs
  - dev/godot/core/object/worker_thread_pool.cpp
  - dev/godot/core/object/message_queue.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphInterfaces.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Async/Fundamental/Scheduler.cpp
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundation_product_incomplete
source_recheck_required: true
---

# Runtime158 当前源码审查

## 1. 结论

本轮逐文件复核 Runtime02 的事件、任务、计时器、回调分发、bounded I/O、动态 session teardown 以及已登记的 graphics、Navigation、Platform 和 asset/operation consumers。选择集为 **111 个文件、22,709 行、783,043 bytes、219 个测试属性、22 个 ignored**；当前工作树在该选择集内有 **34 个 tracked 修改、55 个 untracked**。选择集指纹为 `e1911720c3b472a2ac7d4fc5fbada54274a334d23ee893af48210fde90f63361`。

参考树选择 8 个文件、8,223 行、306,737 bytes，指纹为 `d9527429640b8e54f4ab338a14e4723f56b854a4edcf9f8164eb87124dfcb6b7`。本轮没有新增唯一 finding；这是对旧 Runtime02 的 current-source refresh 和状态重判。

当前源码已经不再是“只有一个 Rayon 全局池”的早期形状：`EngineTaskGraph` 可独占 worker pool，`TaskGraphScope` 有 admission、取消策略、任务 census 和 quiescent wait，`TaskDescriptor`/`TaskStatus` 已存在，bounded keyed/stream I/O 有容量、deadline、fence 和 shutdown guard，graphics、Navigation、Platform 的若干生产构造器已改为注入 runtime-owned pool，event bus 具备明确的 delivery policy、subscriber 生命周期和诊断采样，timer 有注册容量、取消与周期 tick 合并。

这些是可以保留的底座，但没有形成统一的 engine execution contract：

- `TaskPools::process_default`、`TaskTimer::process_default` 和 `TaskCallbackDispatcher::process_default` 仍通过 `OnceLock` 形成进程级 owner；`EngineTaskGraph::worker_inventory` 还明确排除这些 worker。
- `JobScheduler::spawn` 与 `TaskPool::spawn/install` 仍是 panic-on-close、无 typed result 的旁路；只有 `schedule`/TaskGraph scope 路径产生可等待终态。
- event `Lossless` 仍把 `VecDeque` 容量设为 `None`，`EventBus::publish` 返回 `()`，没有 accepted/dropped/backpressured/closed receipt，也没有 byte budget。
- `EngineEvent` 仍是 `String + serde_json::Value`，阻塞式 `Condvar` subscription 没有 sequence、schema、source/session、trace 或 scope cancellation；它同时承担 frame、control 和 FFI 语义。
- timer callback 在 dispatcher 中 `catch_unwind` 后丢弃 panic；timer/callback 没有统一 task identity、failure sink、late-by 或 teardown receipt，且显式 `TaskTimer::new` 默认仍接入进程级 callback dispatcher。

因此旧 Runtime02 的核心 P0（可卸载动态库中的后台执行仍可能越过 session 边界）仍 **Open**。本轮重判为：P0 **1 Open**；P1 **2 Open / 4 Partial**；P2 **2 Open / 2 Partial**。这不是性能优于 Bevy、Fyrox、Godot、Unreal 或 Unity Graphics 的证明；当前没有可接受的跨平台 Release benchmark、内存上限和 DLL unload 证据。

## 2. 当前实现闭环

### 2.1 Event bus

- `EventBus` 以 `Arc<EventBusState>` 持有 topic registry；topic 使用读写锁、copy-on-write subscriber snapshot 和独立 delivery mutex，订阅竞态与 disconnected prune 有测试覆盖（`core/runtime/events/topic.rs:13-110`）。
- `EngineEventDeliveryPolicy` 具备 `Lossless`、`BoundedDropOldest { capacity }`、`Latest` 三种语义；subscriber 使用 `VecDeque`、`Condvar`、`recv/try_recv/recv_timeout`，并记录 queued depth、queue age、dropped/disconnected 等诊断（`core/framework/events.rs:11-79`、`events/subscriber.rs:30-218`）。
- diagnostics 默认已从旧报告的全量 enabled 改为 `Sampled { every: 64 }`，也支持 Enabled/Disabled；这降低了常态计时成本，但 ignored benchmark 仍只打印结果，不是回归 gate。
- 发布过程仍同步取得 topic delivery lock，fan-out 时共享一个 `Arc<EngineEvent>`；没有发布结果对象、producer 等待协议、全局 bytes 预算或可取消的异步唤醒。

### 2.2 Task graph、scheduler 与 bounded lanes

- `EngineTaskGraph::try_new` 创建唯一 runtime-owned compute worker set，并使用同一 pool 创建 callback dispatcher；`create_scope` 在 Running/Closing/Stopped 三态上执行 admission（`tasks/task_graph/engine_task_graph.rs:25-87`）。
- `TaskGraphScope::submit/schedule/schedule_after` 收到 `TaskDescriptor`，将 `CancelOnDrop/DetachOnDrop/FinishOnShutdown` 记录到 `TaskRecord`，以 `TaskHandle` 暴露 descriptor、状态和 terminal wait；依赖失败可在 prelaunch 阶段转为 cancelled/failed（`tasks/task_graph/scope.rs:45-193`）。
- `EngineTaskGraph::shutdown` 关闭所有 scope admission，等待 scope quiescence，再 close/join worker pool；超时保留 Closing 状态供重试。`Drop` 只关闭 admission，注释明确要求 host 在 unload 前显式调用 shutdown（`engine_task_graph.rs:106-151`）。
- `JobScheduler::from_pool` 可以复用已有 owner，`schedule`/`schedule_after` 对 closure `catch_unwind` 并通过 `JobHandle` 传播完成或 panic；`spawn` 仍返回 `()`，走 `submission_or_panic`，且 detached panic 只进入 diagnostics/默认 Rayon 行为的混合路径（`tasks/job_scheduler.rs:32-84,255-270`）。
- bounded keyed/stream I/O 已使用 TaskGraph scope、descriptor、容量 permit、deadline 和 cancellation token，是最接近 canonical execution lane 的生产实现；但它与普通 scheduler 并存，不能代表所有任务都受相同 admission 管理。

### 2.3 Timer、callback 与动态 teardown

- `TaskTimer` 为注册数量设置 512 默认容量，支持一次性/周期 deadline、取消和 `delivery_pending` tick coalescing；最后 owner drop 会通知并 join timer thread（`tasks/timer.rs:15-20,65-178,191-214`）。
- timer worker 只负责 deadline ordering，实际 callback 交给 `TaskCallbackDispatcher`；dispatcher 限制每次 runner 的 callback 数和并发 runner 数，并在执行时 `catch_unwind`，但 panic 没有进入统一 `TaskStatus` 或 failure journal（`tasks/timer.rs:262-334`、`tasks/callback_dispatcher.rs:142-236,319-330`）。
- `TaskTimer::process_default` 静态保存 `Result<TaskTimer, String>`；`TaskCallbackDispatcher::process_default` 又静态保存并引用 `TaskPools::process_default().async_compute()`。显式 `TaskTimer::new` 也默认选择该 dispatcher，因此拥有 session TaskGraph 不等于拥有 timer/callback。
- dynamic session destroy 当前会调用 module shutdown drain，再调用 TaskGraph shutdown 并关闭 process log；这是相对旧报告的重要进展，但没有同一 receipt 覆盖 process static pools、timer、callback queue、config persistence、asset private worker、OS event-loop 或所有 external producer。成功 destroy 仍不能直接推出可安全 `FreeLibrary`。

## 3. 差距清单与重分类

### P0-1：进程级 execution owner 越过 dynamic session / DLL unload 边界（Open）

**证据**

- `TaskPools::process_default` 从 `PROCESS_TASK_POOLS: OnceLock<TaskPools>` 返回长期存在的三类 pool；`Default` 仍指向它（`tasks/pools.rs:18-48,141-145`）。
- `TaskTimer::process_default` 和 `TaskCallbackDispatcher::process_default` 同样由 `OnceLock` 持有；timer callback 可在 static owner 的 worker 上继续执行 DLL 内闭包（`tasks/timer.rs:18-74`、`tasks/callback_dispatcher.rs:18-68`）。
- `EngineTaskGraph::worker_inventory` 只报告自己创建的一个 worker set，并在文档中声明排除 process-default、timer 和 dedicated worker owners（`engine_task_graph.rs:92-103`）。
- dynamic destroy 虽然加入 TaskGraph shutdown，但没有把上述静态 owner、callback queue、配置持久化线程和 asset/operation worker 统一纳入 shutdown census；任何单项成功都不能作为 library unload barrier。

**后果**

多次 session 会共享旧 generation 的任务与 callback；host 在收到 destroy success 后卸载 `cdylib`，静态 worker 仍可能进入已卸载函数或访问失效 service。该风险是代码寿命和未定义行为问题，不是“线程数略多”的性能问题。

**必须重构为**

建立唯一 `ExecutionRuntime`/`RuntimeOwner`：worker domain、timer、callback dispatcher、private worker 和 external producer 都必须注册 owner、代码 generation 与 shutdown blocker。固定顺序为 close admission -> cancel -> drain -> join -> release services/modules -> destroy -> unload；超时、panic、unaccounted task 或 callback queue 非空都必须返回 teardown-incomplete。

### P1-1：detached/tracked task 的终态与 panic 契约不一致（Open）

`JobScheduler::schedule` 有 typed `JobHandle`，可等待完成/失败；`JobScheduler::spawn` 和 `TaskPool::spawn` 却返回 `()`，在 pool closing 时直接 panic。detached closure 的 panic 与 tracked closure 的 `catch_unwind`、diagnostics、wait-time propagation 不是同一契约，调用者无法仅凭 API 判断失败是否可观察、是否会终止进程、是否必须 wait。

Canonical API 应为 `Task<T>`/`TaskResult`，默认每个任务有 owner、descriptor、取消 token、终态 receipt；`detach` 必须显式声明 sink、生命周期和 shutdown policy，fatal task 使用单独 API，不能依赖 Rayon 默认 panic 行为。

### P1-2：descriptor/scope 已存在，但 generic scheduler 与生产 consumers 仍有旁路（Partial）

`TaskDescriptor` 目前只有 id、logical pool kind、label、cancellation policy；`TaskGraphScope` 能执行 admission 和 drain，但 `JobScheduler::spawn/schedule` public API 不要求 descriptor、owner、priority、deadline 或 scope。scene、asset、platform、timer 等历史路径仍可通过 scheduler/pool 直接提交。应把 descriptor 扩展为完整 provenance，并将 generic scheduler 收敛到 scope admission；保留 bounded I/O 的 permit/fence 作为 lane 实现，而非第二套任务状态机。

### P1-3：generic scheduler 的容量、配额、优先级和 shutdown census 仍不完整（Partial）

TaskGraph 现在能关闭 admission、等待 scope、join其 worker，bounded I/O 也有 entry/byte limits；但普通 `JobScheduler` 仍可无 descriptor、无 subsystem quota、无 queue byte budget、无 deadline/priority 地提交。`TaskGraphWorkerInventory` 也不统计 timer、callback、process pool 和 dedicated owners。新增功能仍很容易绕过严谨 lane。目标是统一 `WorkerDomain + TaskScope + TaskDescriptor`，提交明确返回 accepted/rejected reason，shutdown report 能列出每个 owner 的 queued/running/terminal 数量。

### P1-4：线程预算与实际 OS worker census 仍非全局事实（Partial）

graphics、Navigation、Platform 的若干生产路径已改为 pool 注入，减少了临时完整 pool；但 `TaskPools::process_default` 仍存在，public `TaskPool::new`/`JobScheduler` 仍可创建或复用未注册 owner，timer/callback/dedicated worker 不纳入 graph inventory。配置中的 pool thread minimum 仍可能使实际总 worker 超过“remaining”预算。应由 runtime owner 分配 domain quota，并将 configured/created/live/busy/retired 与 OS thread census 对账。

### P1-5：Lossless 队列仍无界，publish 没有 backpressure receipt（Partial）

当前 bounded drop-oldest/latest 和 queue diagnostics 是真实进展，但 `Lossless` 仍在 `subscriber.rs:80-83` 映射为 `capacity = None`，慢 consumer 可以无限积累 `Arc<EngineEvent>`；`EventBus::publish` 在 `publish.rs:41-43` 返回 `()`。没有 per-subscriber/global retained-bytes 上限，producer 看不到 drop、disconnected、closed 或 pressure。应将可靠流建模为有 owner/预算的 bounded control channel，返回 delivery receipt；真正 lossless 必须节流、持久 spill 或有界事务，不能以无限内存伪装。

### P1-6：String + JSON 总线仍混合 frame、control 与 ABI 语义（Open）

`EngineEvent` 只有 `topic: String` 与 `serde_json::Value`（`core/framework/events.rs:11-15`），没有 schema/version、sequence、source/session、timestamp、trace 或 bytes accounting。同步 `Condvar` subscription 也没有 scheduler waker、scope cancellation 或 shutdown wake contract。应硬切为 frame-local typed `Messages<T>`、跨线程 bounded `ControlBus<T>`、跨插件 versioned envelope，使用显式 adapter 连接，而不是继续扩大万能 event API。

### P2-1：timer 串行 callback 的可观测失败与执行预算不完整（Open）

timer 已有 512 registration limit 和周期 tick coalescing，但一个 callback 仍在 dispatcher 的单个 envelope 中串行执行；`catch_unwind` 结果被丢弃，没有 task id、owner、late-by、duration、retry/fatal policy。慢 callback 会占据 callback budget，panic 既不使 scope fail 也不进入统一 shutdown receipt。timer 应只负责排序/wake，把 callback 作为带 descriptor 的 Task 投递到受控 lane。

### P2-2：diagnostics 默认采样已有改进，性能 gate 仍缺失（Partial）

`EventBusDiagnosticsMode::default` 现为每 64 次采样，而非旧报告所述的全量 enabled；这是状态降级的依据。但并发 topic lookup、bulk prune、diagnostics ratio 和 pressure benchmark 仍主要是 `#[ignore]` 测试打印，没有 Debug/Release、Windows/Linux、CPU 核数、fan-out、payload size、paused consumer 的稳定基线，也没有 p99、allocation、retained bytes 和回归阈值。因此只能标 Partial，不能宣称性能优于参考引擎。

### P2-3：poison lock 一律恢复，不能证明 transaction invariant（Open）

event topic/subscriber/timer/dispatcher 多处用 `unwrap_or_else(|poisoned| poisoned.into_inner())`；当前测试证明锁 poison 后仍能继续读写，但没有覆盖“更新 queue/counter/map 一半后 panic”的 conservation、journal rollback 或 fail-closed admission。执行 owner 的不可证明状态应进入 `Poisoned` 并关闭新提交，不能静默当健康状态继续。

### P2-4：fallible pool construction 已有，但 panic wrapper 仍是公共入口（Partial）

`TaskPool::try_new`、`TaskPools::try_from_options` 和 `EngineTaskGraph::try_new` 已能返回初始化错误，多 pool 构造也按顺序 rollback；但 `TaskPool::new`、`TaskPools::from_options`、`from_options_with_available_parallelism` 和 `submission_or_panic` 仍会 panic。应将 product-facing construction 全部迁移到 typed `ExecutionInitError`，保留 panic wrapper 只在明确的 test/tool boundary，并在错误中记录 requested/created workers、pool kind 和 OS cause。

## 4. 参考引擎对照

| 参考 | 当前源码吸收点 | Zircon 仍需补齐 | 边界 |
|---|---|---|---|
| Bevy task pool / ECS messages | worker `JoinHandle`、`Task<T>`、scope cancel/drop、typed `Messages<T>`、message id/cursor 和双缓冲 | Zircon 需要把 typed task 与 frame message 接入真实 runtime consumers，并为 dynamic library 增加 owner/generation/unload receipt | Bevy 的常驻 App 全局资源不能直接证明 Zircon cdylib 热卸载正确 |
| Godot WorkerThreadPool / MessageQueue | 明确 finish/join，queued call 有 page/max-pages 内存上限并向 producer 返回错误 | Zircon timer/callback、control bus 要有可见的关闭、容量和 producer error；不能让 Lossless 无限增长 | Godot Variant/Callable 是通用边界格式，不应直接成为 Zircon frame hot path |
| Unreal TaskGraph / Scheduler | owner 驱动 stop、waiting queue shutdown、worker join、global queue drain、priority/affinity 配置 | Zircon 要把所有 worker domain、timer、callback、I/O lane 纳入单一 census 和 shutdown transaction | Unreal 静态调度器服务常驻进程；Zircon 还需处理每 session 代码 generation |
| Fyrox engine/plugin lifecycle | 主循环统一驱动 plugin/engine 更新和资源生命周期 | Zircon event/task API 尚未把 editor/runtime/plugin producer 接到统一 frame/control boundary | 参考只用于生命周期形状，不等于 Fyrox 的单线程 loop 可替代 Zircon 并发图 |

共同原则是：执行资源必须有唯一 owner、可枚举 worker、可取消 admission 和可证明 join；消息必须按 frame locality、跨线程 backpressure 和 ABI/schema 边界分层。当前 Zircon 的局部类型和测试可以作为迁移输入，但不能以 API 名称存在就关闭 finding。

## 5. 目标架构与重构顺序

### 5.1 Canonical execution contract

1. `ExecutionRuntime` 唯一创建并拥有 compute/io/background/control worker、timer thread 和 callback dispatcher；禁止可卸载 runtime library 内的 process-static worker。
2. `WorkerDomain` 由配置决定实际 worker、priority、affinity、stack、queue/bytes budget；所有 dedicated thread 必须注册 owner、代码 generation、stop/join callback。
3. `TaskScope` 绑定 session/module/subsystem，拥有 admission gate、parent cancellation、deadline、descriptor census 和 shutdown policy。
4. `Task<T>` 统一 completed/failed/cancelled/detached/fatal 终态；panic 转 typed failure 并进入 diagnostics/failure sink，只有显式 fatal API 才允许升级进程级故障。
5. `TaskDescriptor` 至少包含 stable id、owner、domain、label、priority、deadline、cancellation/shutdown policy、trace provenance 和 ABI generation。现有 bounded I/O descriptor 应直接扩展/复用。

### 5.2 Event boundary hard cut

- frame/world 内使用强类型、批量、有限 retention 的 `Messages<T>`，以 schedule/frame owner 驱动，不阻塞 `recv`。
- 跨线程使用 `ControlBus<T>` 或 erased typed envelope，按事件数与 retained bytes 双限额；publish 返回 `DeliveryReceipt`（delivered/dropped/rejected/disconnected/closed/backpressured）。
- plugin/FFI 使用 versioned envelope，明确 schema id、length、allocator/owner、sequence、source/session 和 generation；内部 event 通过 adapter 转换。
- diagnostics 只能采样观察，不改变 delivery 语义；默认成本必须由正式 benchmark gate 决定。

### 5.3 Shutdown transaction

```text
Running
  -> CloseAdmission (task scopes, control buses, timers, producers)
  -> Cancel (CancelOnDrop/Shutdown; publish no new callback)
  -> Drain (FinishOnShutdown, in-flight calls, subscribers, I/O fences)
  -> Join (timer, dispatcher, every worker and dedicated thread)
  -> ReleaseServicesAndModules
  -> DestroySession
  -> UnloadLibrary
```

每一步都必须产出不可变 receipt；任何 timeout、panic、poison、remaining queue、unaccounted worker 或 generation mismatch 都阻止 unload。`EngineTaskGraph::Drop` 只能作为 fail-closed fallback，不能替代显式 shutdown。

### 5.4 测试先行里程碑

| 阶段 | 先写的失败证据 | 重构范围 | 晋级条件 |
|---|---|---|---|
| M0 | 真 DLL load/create/destroy/free/reload；worker/timer/callback/config census；detached panic 与 timeout | 建立 execution ownership 和 unload harness | destroy success 与 unload-safe 明确分离，残留线程可复现 |
| M1 | queued/running/cancel/detach/finish、panic、partial pool init、scope drop | `ExecutionRuntime`、domain、scope、`Task<T>` | accepted/rejected/terminal 守恒，所有 worker 可 join |
| M2 | generic scheduler 与 bounded I/O 的 quota/deadline/owner 压力 | 所有 lane 走 canonical admission | production submit 无 scheduler/pool 旁路 |
| M3 | timer slow/panic/cancel/late-by/shutdown | timer 只排序/wake，callback 是 typed task | timer/dispatcher 可独立 drain/join，慢 callback 不阻塞 deadline |
| M4 | paused lossless consumer、bytes pressure、receipt、shutdown wake、poison-after-mutation | bounded `ControlBus` 与 fail-closed policy | 无无界 production queue，所有 wait 可取消 |
| M5 | typed message cursor、retention、batch、world reset、性能 | `Messages<T>` 与 adapter | 高频内部事件不再依赖 String/JSON/Condvar |
| M6 | ABI schema/version/allocator/generation、session recreate | plugin envelope 与 unload receipt | 动态库边界无 Rust closure/object 泄漏 |
| M7 | 1/2/8/32 worker、long-run pressure、Debug/Release baseline | 删除旧 process default、panic shim、万能 event API | 无双轨，产品 teardown 和性能 gate 全通过 |

## 6. 验收 gates

| Gate | 当前状态 | 必须证明 |
|---|---|---|
| G01 execution owner/session isolation | Fail | 每个 session 有唯一 execution owner，禁止 process-static worker 跨 generation |
| G02 TaskGraph scope admission | Partial | 所有生产任务带 descriptor/owner，关闭后返回 typed rejection |
| G03 all worker/timer/callback join | Fail | shutdown census 覆盖 graph、pools、timer、dispatcher、dedicated thread |
| G04 typed task terminal/panic | Fail | spawn/schedule/detach 的终态、panic 和 fatal policy 可观察且守恒 |
| G05 generic scheduler descriptor integration | Partial | scheduler 不再绕过 TaskScope、quota、deadline 和 cancellation |
| G06 global worker budget | Partial | configured/created/live/busy 与 OS thread census 一致，禁止隐式 full pool |
| G07 bounded event delivery | Fail | count/bytes 双上限、publish receipt、paused consumer 不增长越界 |
| G08 frame/control/ABI event split | Fail | Messages、ControlBus、plugin envelope 具有独立类型和 adapter |
| G09 timer callback fault/late policy | Fail | callback 有 identity、owner、duration、late-by、failure sink 和 shutdown receipt |
| G10 poison fail-closed | Fail | transaction state 中途 panic 后 rollback 或关闭 admission，不盲目恢复 |
| G11 fallible initialization | Partial | product construction 不 panic，partial init 会停止并 join 已建 worker |
| G12 dynamic unload barrier | Fail | destroy 只有在 producer quiesced、queue drained、所有 join 完成时才允许 unload |

## 7. 既有计划纠正

1. `docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md` 的旧 P0 描述应保留为历史背景；本报告是 current-source 状态，不能因 `EngineTaskGraph` 新增就关闭静态 timer/callback/pool 风险。
2. `runtime/07-runtime-performance-hotpath` 的 event ordering、drop-oldest/latest 和 diagnostics sampling 可作为局部行为证据；ignored print benchmark 不能作为性能回归门槛。
3. lifecycle/module 计划必须把 task/timer/event quiescence 放在 service cleanup 和 library unload 之前；模块清理不能替 execution owner 的 join barrier。
4. graphics、asset、scene、operation、platform 专篇可以登记自己的 consumers，但 canonical worker ownership 和 shutdown receipt 只能由 ExecutionRuntime 统一提供。

## 8. Review-only 结论

本轮只完成源码审查、参考引擎对照、重分类和重构计划，没有修改 Zircon 生产 Rust、Cargo、构建脚本或测试实现。后续实现应从 M0 的真实 DLL unload/worker census 开始，并在每个里程碑完成后重新计算选择集指纹和 gates；在此之前不能宣称“性能和表现优于虚幻引擎”。
