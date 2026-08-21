---
related_code:
  - zircon_runtime/src/operation
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/registry
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/navigation/operation
  - zircon_runtime/src/navigation/runtime.rs
  - zircon_runtime/src/scene/navigation.rs
  - zircon_runtime_interface/src/runtime_api/operation.rs
  - zircon_runtime_interface/src/runtime_api/api_table.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_editor/src/core/gateway
  - zircon_plugins/navigation/editor/src/operation_command
  - zircon_plugins/navigation/runtime/src/manager.rs
tests:
  - zircon_runtime/src/operation/tests.rs
  - zircon_runtime/src/operation/tests/source_guards.rs
  - zircon_runtime/src/operation/service/completion.rs
  - zircon_plugins/navigation/editor/src/tests/operation_command.rs
  - zircon_plugins/navigation/runtime/src/tests/operation.rs
plan_sources:
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/19-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-authoring-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
  - docs/plans/optimize/zircon_tooling/37-transaction-atomicity-prepare-commit-publish-rollback-compensation-idempotency-crash-recovery-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/IAssetCompilingManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/AssetCompilingManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/AssetCompilingManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/AsyncWork.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/QueuedThreadPool.h
  - dev/godot/core/object/worker_thread_pool.h
  - dev/godot/core/object/worker_thread_pool.cpp
  - dev/bevy/crates/bevy_tasks/src/lib.rs
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/Fyrox/fyrox-core/src/task.rs
  - dev/Fyrox/fyrox-impl/src/engine/task.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/PathTracing/LightBakerWorkerProcessImporter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Water/AsyncTextureSynchronizer.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 41 · Operation Service / Registry / Admission / Prepare / Apply / Progress / Cancel / Shutdown 工程化差距

## 1. 结论

`RuntimeOperationService`不是临时空壳。当前实现已经有按session实例化的handler registry、task/retained-byte双重容量、raw JSON decode前预留、owner-thread snapshot、worker prepare、owner-thread apply、panic隔离、deadline、cancel、terminal TTL以及两阶段harvest。尤其是prepared command与terminal result在apply前共同预留、foreign allocation成功后才commit harvest，这两处体现了正确的“失败工作前移”和跨FFI所有权意识，应当保留。

但它仍只是一个导航命令专用的进程内异步桥，不是工程级Operation Control Plane。请求只有`operation_id + JSON payload`，handle只是session内单调`u64`；调度从`HashMap`任取任务，没有FIFO、优先级、公平性、资源冲突或owner配额；cancel/deadline只改变发布状态，已经运行的prepare仍继续消耗CPU和内存；公开进度永远是`0/1`或`1/1`；dynamic ABI只有submit/poll/harvest，连cancel、deadline和订阅都没有。更关键的是，Operation只在`tick_frame`推进，提交既不请求wake，session的`frame_demand()`也不观察Operation队列，reactive host可以让已接受任务永久停在Queued/ReadyToApply。

事务边界同样没有类型约束。`snapshot`文档声称只捕获immutable input，但拿到的`RuntimeOperationContext`公开`world_mut()`；`apply`也允许先修改World或外部driver再返回`Err`/panic，service随后只记录`Failed`，没有`NotApplied/Applied/Partial/Unknown` disposition、rollback、compensation或receipt。当前两个navigation runtime的snapshot不写World，clear/restore的最终replace当前又恰好无失败，因此静态证据尚不足以把这个通用合同漏洞升级为新的shipping P0；但继续允许新handler按该trait接入，会把“失败”变成可能已修改世界的错误陈述，必须在扩展前硬切。

当前唯一生产consumer还没有闭环：四个navigation operation中Bake Scene/Bake Surface的prepare固定返回“requires a pure prepare backend”，Editor command提交后只`yield_now + poll` 16次且不驱动runtime tick，in-process gateway则明确返回capability missing；plugin focused test仍调用已经不存在的`poll(context, handle)`签名并期待Bake成功。该产品阻断已经由Editor19 P0-1/P0-3和Runtime08D拥有，本篇不重复登记P0。进程级worker越过session/DLL unload由Runtime02 P0-1、ABI destroy/quiesce由Runtime Interface01 P0-05拥有；Runtime41负责给Operation增加owner task、close-admission、cancel/drain/fence的领域合同。

本报告新增 **0项P0、48项P1、12项P2和40个资格门**。实施必须先冻结descriptor/request/identity/disposition和只读snapshot合同，再建立确定性调度、合作式取消/进度/wake与owned task group，随后扩展ABI和真实异步consumer，最后以并发模型、停机、饥饿、超载、跨版本和长时规模测试收敛。不能继续在现有String/JSON/HashMap轮询器上逐个补按钮。

## 2. 审查边界、语料与 currentness

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | test属性或宏 / ignored | 结论 |
|---|---:|---:|---|
| `zircon_runtime::operation`完整目录 | 11 / 2,639 / 95,277 | 23 / 0 | E3逐文件检查registry、admission、task、completion、maintenance、harvest和测试 |
| ABI、dynamic session与Editor gateway | 16 / 4,310 / 151,672 | 8 / 0 | E3核对V7函数表、bounded JSON、session tick/wake/frame demand与三类gateway |
| navigation真实producer/consumer | 12 / 2,083 / 71,958 | 9 / 0 | E3核对唯一handler、runtime driver、Editor command和编译漂移测试 |
| 父计划与唯一owner | 10 / 4,809 / 459,343 | 3 / 2 | E2核对P0归属、task/ABI/identity/job/transaction依赖 |
| Unreal/Godot/Bevy/Fyrox/Unity Graphics | 13 / 4,988 / 173,842 | 7 / 0 | E2/E3核对任务取消、进度、scope、owner completion与资源释放边界 |
| selected combined scope | 62 / 18,829 / 952,092 | 50 / 2 | 工作树fingerprint `015542c0edfdf11b31978b61a8e0dde011c3e42f70949686a49fbf5481286eef` |

指纹按62个selected path去重排序，对每个文件取lowercase SHA-256，再以`forward/slash/path|hash`和LF连接、无末尾LF后取总SHA-256。测试数字是静态`#[test]`、C++/C#测试标记，不表示已经编译或通过。父计划中的2个ignored来自其自身证据，不属于Operation测试。

### 2.2 检查方法

本轮按`register -> submit -> reserve/decode -> queue select -> snapshot -> dispatch -> prepare -> completion reserve -> apply claim -> terminal publication -> poll/harvest -> TTL/eviction -> session shutdown`逐段阅读，并反向搜索全部非`dev/`消费者。每个阶段分别核对identity、owner、linearization point、容量、取消、deadline、错误、世界副作用、wake、retention与teardown；然后以本地五套参考源码校准能力边界，不从类名或注释推断未实现功能。

### 2.3 动态证据边界

1. 本轮是review-only，没有修改Runtime、Interface、Editor、Plugin、App生产代码或测试，也没有运行新的全工作区编译。
2. `zircon_plugins/navigation/runtime/src/tests/operation.rs`静态调用`poll(RuntimeOperationContext, handle)`，而当前public签名是`poll(handle)`；这足以证明source drift，但未单独运行该package，故不伪造动态失败日志。
3. 已知Editor、Hub、WOC、plugin metadata验证阻断保持原状；Runtime41不重复执行耗时且无关的失败lane。
4. 没有执行真实长Bake、worker拒绝、1000任务并发、session destroy race、DLL unload、timer失效、handle耗尽、进度订阅或多客户端压力，因此这些能力均保持未通过。
5. 当前工作树包含其他会话的大量Editor修改；报告实施前必须重取指纹并复核overlap，不能把本次快照当成稳定提交基线。

## 3. 必须保留的工程基础

1. 保留每个runtime session独立的`RuntimeOperationService`，不要退回进程全局裸registry。
2. 保留`RuntimeOperationHandler: Send + Sync`以及owner snapshot、worker prepare、owner apply三段方向，但重新收紧每段可见能力和commit语义。
3. 保留raw request在decode前预留count/bytes的设计，并让dynamic ABI真实走同一admission primitive。
4. 保留task count与retained bytes双预算、checked arithmetic和prepared command/result在apply前预留。
5. 保留prepare/apply panic捕获，但panic必须进入service fuse、typed diagnostic和effect disposition，而不只是字符串。
6. 保留deadline与terminal TTL的独立概念；前者限制执行，后者限制结果保留，不能合并成一个timeout。
7. 保留cancel在owner apply claim前关闭publication的线性化方向，并补合作式停止和明确`TooLate` receipt。
8. 保留`prepare_harvest -> foreign allocation -> commit/rollback_harvest`，这是当前最成熟的跨FFI提交段。
9. 保留navigation clear/restore的compare-before-apply思想，将其上升为generation/conflict contract。
10. 保留bounded JSON作为兼容transport，但descriptor与内部handler不能继续以无schema的JSON作为唯一类型系统。
11. 保留Runtime02的统一scheduler owner，不在Operation内部再造第二个线程池；Operation应通过owned task group消费它。
12. 保留Runtime Interface01/05的ABI与foreign allocation authority、Runtime24的handle identity authority和Editor09的UI job projection authority。

## 4. 当前代码事实与断路

| 链路 | 当前事实 | 工程后果 |
|---|---|---|
| Registry | `BTreeMap<String, Arc<dyn Handler>>`，只在session construction以`&mut self`注册navigation四项 | 没descriptor、schema、owner、permission、unregister或plugin drain |
| Request | V1只有ABI、operation ID和JSON payload | 无request identity、deadline、priority、idempotency、principal和resource claims |
| Handle | 从1开始checked-add的裸`u64` | 不携带session/epoch/generation，stale或foreign handle只能靠map miss |
| Admission | 默认1024 tasks、32 prepares、4 MiB retained；参数为crate-private常量 | 有局部容量，但无产品配置、owner quota、CPU预算或拒绝重试提示 |
| Dynamic decode | ABI先bounded decode成完整Value，再调用`submit`而非`submit_json` | 1 MiB上限存在，但service的raw reservation并未覆盖真实FFI decode生命周期 |
| Queue | `HashMap::iter().find_map`选择Queued和ReadyToApply | 顺序不确定，无FIFO/优先级/公平性/aging；queue depth只是入队时快照 |
| Snapshot | owner线程同步调用，context公开`world_mut()` | 任意耗时/分配可卡帧；失败或panic前可修改World且无法回滚 |
| Prepare | generic scheduler detached closure，只有owned JSON snapshot | 无task owner、cancel token、deadline、progress、budget或shutdown fence |
| Cancel/expiry | task转Cancelled/Expired并释放retained bytes | worker仍运行；取消表示“不再发布”而不是“计算已停止” |
| Completion | 每tick dispatch batch创建一个sync channel，receiver vector线性扫描 | 多batch轮询和channel生命周期不透明，没有completion wake |
| Apply | owner线程同步调用，最多8项；snapshot也独立最多8项 | 一帧可执行16个任意时长callback，count budget不等于frame time budget |
| Effect | apply返回`Result<()>`，error/panic统一写Failed | 不能说明未应用、已应用、部分应用、补偿中或状态未知 |
| Progress | task status固定nonterminal `0/1`、terminal `1/1` | ABI字段形似进度，实际没有工作量、phase detail或ETA |
| Wake | submit使用`with_session`，不请求frame；frame demand只看asset reload/animation | reactive session可接受任务但不继续tick，deadline/ready apply也无产品推进保证 |
| ABI | V7只暴露submit/poll/harvest | 外部调用方不能cancel、设deadline/priority、订阅、枚举或查询descriptor |
| Result | V1仅Succeeded JSON或Failed String | 无错误码、stage、retryability、disposition、receipt和diagnostic correlation |
| Output | 内部总预算4 MiB，FFI单result上限1 MiB | 合法内部结果可能永远无法通过ABI harvest，且没有分页/artifact fallback |
| Retention | Completed/Failed 60秒后转Expired；pressure只驱逐Cancelled/Expired/Harvested | 状态从Expired最终变Unknown，terminal task占task slots，策略不可由caller协商 |
| Maintenance | process-default timer + wrapping generation；refresh错误多处被忽略 | paused/headless无frame时deadline/TTL可能失去alarm，失败不可观察 |
| Shutdown | dynamic session先停plugin events/watchers/modules，Operation没有close/drain | preparing closure和handler lease不进入session teardown证据；依赖generic scheduler偶然收尾 |
| Consumer | 只有navigation；Bake prepare固定失败，Editor 16次poll，in-process缺能力 | framework测试不能证明任何真实长操作产品闭环 |
| Tests | 23个core tests主要覆盖容量、panic、TTL、cancel queued与harvest | 无公平性、并发race、in-flight cancel、shutdown、wake、ABI大结果、soak或model check |

## 5. 参考实现给出的边界

### 5.1 Unreal

本地`IAssetCompilingManager`公开remaining count、finish selected/all、best-effort cancellation和`Shutdown()`；其注释明确cancel只是一种提示，要确认活动结束仍必须finish，而shutdown要阻塞到可安全退出。`AsyncWork`区分queued retract/abandon与running completion，并明确反对持续spin `IsDone`，要求按帧检查或`EnsureCompletion`。Zircon应吸收的是“取消状态”和“安全排空”分离、按对象/类型owner管理以及受预算的主线程completion，不是复制其类层次。

### 5.2 Godot

`WorkerThreadPool`为单任务和group任务提供独立ID、优先级、description、group processed element count、completion查询与wait。它证明大批工作需要group级进度和显式等待边界；但它仍是执行器，不替Operation决定权限、幂等、世界commit或跨ABI receipt。Zircon应让Operation descriptor/resource policy位于通用scheduler之上。

### 5.3 Bevy

`bevy_tasks::TaskPoolBuilder`显式配置线程，`TaskPool::scope`保证scope内任务全部结束后才返回，Task的drop/detach语义也要求调用方明确是否保留执行。适用结论是owner lifetime必须由类型或scope证明；不适用结论是把所有长操作改成任意detached future。Zircon仍需session-owned task group、cancel/progress和owner-thread publication。

### 5.4 Fyrox

Fyrox core `TaskPool`给result分配UUID并通过channel回收，engine `TaskPoolHandler`把completion handler绑定plugin或scene node，再在拥有相应context时应用结果。这与Zircon的prepare/apply方向接近，说明owner completion routing值得保留；同时其简单pool没有提供本报告要求的完整admission、cancel、priority和teardown，因此只能作结构参考，不能作为目标上限。

### 5.5 Unity Graphics

本地Graphics镜像的Light Baker worker通过artifact异步启动独立process，显式连接progress channel、轮询cancellation、更新共享progress，结束时cancel reporter并`Join`线程；`AsyncTextureSynchronizer`在资源释放前等待尚未完成的GPU readback。这里可验证的基线是长任务必须有进度、取消传播和释放前fence。镜像不包含Unity通用Editor job authority，本报告不对缺失源码作推断。

### 5.6 Zircon的超越目标

Zircon不应只追平上述单个机制。目标合同还必须同时具备bounded admission、typed descriptor/schema、session/owner/generation identity、deterministic fairness、cooperative cancellation、explicit effect disposition、idempotency、event-driven wake、cross-ABI allocation commit以及可证明的shutdown census。这些能力组合起来，才是可承载Cook、Bake、Import、World build、server maintenance和plugin operation的共享内核。

## 6. 目标架构

```text
RuntimeOperationCatalog
  -> OperationDescriptor
     id / schema / owner / capability / effect-class / budgets / queue-policy
  -> submit(OperationRequest)
     request-id / idempotency-key / session-epoch / priority / deadline / payload
  -> AdmissionController
     global + owner + handler + resource quota / retry-after / overload receipt
  -> DeterministicScheduler
     FIFO within priority / aging / fairness / conflict keys / wake demand
  -> SnapshotPreflight (read-only owner context)
  -> PrepareTask (owned task group + cancel/deadline/progress context)
  -> Commit
     infallible publication OR typed transaction/compensation receipt
  -> ResultStore
     status stream / paged result or artifact / retention lease
  -> Shutdown
     close admission -> cancel queued -> signal running -> drain -> fence -> drop handlers
```

建议将handler合同拆为显式类型：

```text
OperationDescriptor
OperationRequest<Payload>
OperationSnapshot<OwnedInput>
PrepareContext { cancellation, deadline, progress, budget, diagnostics }
PreparedOperation<Command, ResultDraft>
CommitContext { owner generation, transaction coordinator }
OperationReceipt<Result> { disposition, effect scope, result, diagnostics }
```

`SnapshotContext`只暴露`&World`和只读service；任何需要owner侧预处理的mutation必须成为单独transaction并返回receipt。`commit`要么通过类型保证all-failable-work已经完成并成为不可失败publication，要么返回带`NotApplied/Applied/Partial/Compensated/RecoveryPending/Unknown`的明确disposition。`Err(String)`不再承担effect truth。

## 7. P0 唯一归属与依赖路由

本篇不新增P0，以下现有阻断必须作为Runtime41的前置或联合交付，不复制计数：

| Canonical owner | 现有阻断 | Runtime41责任 |
|---|---|---|
| Editor19 P0-1 | navigation Bake生产固定失败，focused test期待成功且签名漂移 | 提供可驱动真实prepare/commit的Operation合同；导航实现仍由Editor19/Runtime08D闭环 |
| Editor19 P0-3 | Bake panel、异步job、asset transaction与product host断开 | 暴露event-driven progress/cancel/result；UI/job projection由Editor09/19完成 |
| Runtime02 P0-1 | process task/timer worker越过dynamic session与DLL unload | Operation所有prepare必须加入session-owned task group并参与drain fence |
| Runtime Interface01 P0-05 | destroy/quiesce缺deadline/cancellation，action可永久阻塞 | Operation close/drain receipt接入session destroy，不新建第二套unload authority |
| App01 P0-3 | process-wide shutdown coordinator缺失 | Operation提供领域census/fence，App拥有最终跨service排序 |

## 8. P1：必须重构的共享服务差距

### P1-01：snapshot的“immutable”注释与`world_mut()`能力直接矛盾

`RuntimeOperationContext`同时提供`world()`和`world_mut()`，而trait文档把snapshot定义为immutable capture。类型系统允许handler在snapshot中修改World后返回error或panic，service却把任务标成Failed。应拆出只读`OperationSnapshotContext`，并用compile-fail/API测试证明snapshot拿不到任何mutation authority。

### P1-02：apply没有commit point与effect disposition

`apply`返回`Result<()>`，service无法判断error前是否已经写World、driver、文件、GPU或网络。目标是每个descriptor声明effect class和commit point：纯内存publication应不可失败，事务型apply返回receipt，不可回滚副作用必须报告Applied/Partial/Unknown并进入reconcile。

### P1-03：terminal result在prepare阶段预制，不能表达实际commit结果

`RuntimeOperationPrepared`同时携带command和最终result；apply成功后service原样发布prepare生成的result。它不能包含commit generation、实际affected entities、conflict resolution或外部receipt。应把prepare输出改为result draft，commit生成或确认最终receipt。

### P1-04：handler registry没有descriptor与schema catalog

registry只知道String ID和trait object，无法枚举payload/result version、display metadata、权限、队列、预算、effect class、是否可取消或是否支持progress。建立immutable `OperationDescriptor` catalog，并在session创建时验证重复ID、schema compatibility和owner capability。

### P1-05：operation ID没有命名空间、长度和canonical规则

当前仅`trim().is_empty()`，实际lookup仍使用原字符串；大小写、空白、Unicode、超长ID和owner前缀都未定义。使用validated `OperationTypeId`，冻结ASCII/Unicode策略、最大长度、namespace和版本规则，transport拒绝非canonical表示。

### P1-06：registry没有owner、generation、lease或unregister/drain

`register_handler(&mut self)`只适合construction-time builtin，既阻止受控动态扩展，也无法表达plugin reload/disable时哪些任务和handler必须排空。建立owner-scoped registration lease；retire先close owner admission，再cancel/drain其任务，最后释放handler generation。

### P1-07：请求没有稳定RequestId与幂等合同

每次submit都分配新handle，同一调用重试会重复执行；请求也没有canonical input fingerprint。引入stable request ID/idempotency key、attempt和dedup/result replay policy；same ID + different intent必须fail closed。

### P1-08：operation handle没有session、owner、epoch和generation

裸`u64`只在当前map中有意义，日志、跨gateway重连、session replacement和stale handle无法分类。按Runtime24建立opaque identity，至少绑定session epoch和generation；ABI lookup应区分invalid、foreign、stale、expired与unknown。

### P1-09：请求没有principal、capability与审计上下文

任何持有session handle的caller都能按字符串提交已注册操作，handler看不到调用者身份。请求需绑定principal/capability token、origin、audit correlation和redaction policy；descriptor声明所需权限，admission在decode后、snapshot前授权。

### P1-10：请求没有deadline、priority和服务等级

内部`submit_with_deadline(Instant)`不能跨ABI，公开V1也无priority、latency class或best-effort/interactive/background区分。新增版本化request policy，跨进程使用duration或host/runtime clock-domain明确的deadline，并给每项策略可验证范围。

### P1-11：Queued选择基于`HashMap`迭代，顺序不确定

`take_queued_snapshot_task`任取首个matching entry，提交顺序与执行顺序没有合同，不同seed/build可改变结果。使用单调sequence和明确的per-priority FIFO；相同输入与策略下，selection trace必须可重放。

### P1-12：ReadyToApply同样无序，可能扩大世界冲突

prepare完成顺序已经非确定，apply又从HashMap任取，相关操作可能以偶然顺序commit。descriptor需声明conflict key/ordering group，scheduler对同资源串行化或在commit做generation conflict，而无关资源才能并行。

### P1-13：queue depth只是提交瞬间的task count

`detail_value`在admission时写`tasks.len()+1`，包含terminal tombstone且之后不更新，不是排队位置。状态应分别报告queued ahead、active by class、estimated work或仅报告可证明的queue sequence，禁止展示伪精确值。

### P1-14：没有全局、owner、handler和resource分层配额

单一1024 task/4 MiB预算允许一个handler或caller占满全session，也不限制CPU、scratch、GPU、I/O和external process。建立hierarchical quota、resource class token和retry-after；重要交互任务要保留最小容量，background任务不得饿死产品控制面。

### P1-15：limits是private硬编码，无法按产品和平台配置

32 prepare、4 MiB、8 owner callback、60秒TTL没有来自profile/device/server policy的证据。迁移为validated `RuntimeOperationPolicy`，提供platform defaults、project override边界、运行时诊断和拒绝不安全组合的构造器。

### P1-16：同一count被误用为snapshot和apply预算

`max_owner_applies_per_tick`既循环snapshot又循环apply，一帧最多执行两倍命名预算。拆成snapshot/apply count与time budget，统一由frame budget controller扣账；命名、统计和实际执行必须一致。

### P1-17：snapshot没有时间、分配或访问预算

handler在owner线程可做任意同步工作，service只限制调用次数。增加cooperative owner budget、slow callback telemetry和descriptor最大snapshot成本；超预算要延迟后续任务并记录违规handler，不能卡住整帧。

### P1-18：apply同样没有frame-hitch保护

prepare再重也不能保证apply轻量；任意driver call可能阻塞。要求descriptor给出commit class，heavy apply拆成precomputed artifact + bounded publication；对不可避免的同步提交建立专用maintenance window和hitch gate。

### P1-19：prepare提交到generic scheduler后失去task ownership

service只保留completion sender，不持有task handle或owner group，不能查询queued/running、取消、join或在shutdown证明归零。Runtime02提供owned scheduler contract后，Operation必须保存task lease并维护精确census。

### P1-20：cancel只取消结果发布，不停止运行中的prepare

Preparing任务被标Cancelled并释放accounted bytes，但closure继续持有snapshot/handler并运行，completion回来后才被丢弃。API必须明确`CancelRequested`与`Cancelled`，prepare通过token定期checkpoint；只有worker确认停止或commit不可取消点后才能进入terminal。

### P1-21：deadline也不能中断运行中的prepare

deadline expiry与cancel走相同的“状态先结束、工作继续”模式，可能在超载时持续占用全部worker。deadline token进入prepare context，scheduler支持撤回尚未开始任务；running task若不合作必须被标记并纳入shutdown/health fuse。

### P1-22：prepare trait没有取消、进度、deadline和预算上下文

当前参数只有owned JSON snapshot，handler无法合法观察任何控制信号。引入`PrepareContext`，其中的cancel/deadline/progress/budget/diagnostic sink都是owner-scoped、线程安全且生命周期受task group控制。

### P1-23：没有pause、resume和reprioritize语义

大型Bake/Cook/Import通常需要在交互负载上升时降级或暂停，当前只有不可见的内部cancel。descriptor声明是否支持pause/checkpoint；scheduler级reprioritize与handler cooperative pause分开，不能假装所有任务可暂停。

### P1-24：没有dependency、batch、coalescing和supersede策略

同一asset/world连续修改会提交多个过期operation，service既不合并也不知道依赖。增加dependency handle、supersede key、coalescing policy与generation input；旧请求可在snapshot前被替代，commit前仍需generation验证。

### P1-25：dynamic ABI没有cancel入口

Rust service虽有`cancel`，V7 function table和Editor gateway contract只暴露submit/poll/harvest。新增尾部兼容的cancel API，返回Accepted/AlreadyTerminal/TooLate/Unknown/Stale等typed状态，并贯穿dynamic与in-process gateway。

### P1-26：dynamic ABI无法传deadline、priority或idempotency

内部能力不能由真实外部caller使用，导致测试面与产品面分裂。定义Operation Request V2及capability negotiation，旧V1映射到明确默认policy并记录legacy usage，不能静默猜测交互优先级。

### P1-27：公开progress字段是假进度

`completed_work`只按terminal设0/1，`total_work`恒为1。改为optional progress model：unknown total、units、completed、phase label key、updated sequence和timestamp；不支持进度的handler必须显式`Indeterminate`，不能伪装0%。

### P1-28：只有poll，没有completion/progress订阅

外部caller只能主动轮询，Editor因此出现固定16次yield loop。增加subscription或session event stream，使用bounded coalescing progress和lossless terminal receipt；poll保留为恢复/诊断路径，不再是产品主循环。

### P1-29：submit不请求runtime wake

`submit_operation`通过`with_session`写task后没有调用`RuntimeFrameActivity::request_frame`或wake callback。成功admission必须原子地请求Immediate frame；worker completion、earliest deadline和ready-to-apply也分别触发coalesced wake。

### P1-30：session frame demand完全忽略Operation状态

`frame_demand()`只组合asset reload与animation，Queued/ReadyToApply/near deadline均不可见。Operation提供`frame_demand()`：owner work为Immediate，worker running可OnDemand并依赖completion wake，future deadline为SleepUntil；session accumulator取最早需求。

### P1-31：状态没有排队、执行和提交时间证据

task只存deadline/terminal time，没有submitted、snapshot start/end、prepare start/end、apply start/end或wait原因。补齐monotonic timing与sequence，用于ETA、SLO、hitch、deadline miss和性能回归；跨ABI只暴露稳定duration/age而非进程Instant。

### P1-32：错误只有String，没有stage、code、retryability和effect truth

handler与result失败都压成文本，FFI又将多数service error映射为generic status。建立稳定error code/domain、stage、retry policy、diagnostic ID、safe message和effect disposition；内部source chain写受控诊断，不直接泄露敏感payload。

### P1-33：内部与ABI结果上限不一致会制造永久不可harvest结果

service允许command+result占4 MiB，dynamic output只允许1 MiB；apply已发生后encode失败会rollback harvest标记，却无法让结果变小。admission按transport capability预留可交付上限，大结果转typed artifact/page，不得产生永远取不出的成功。

### P1-34：JSON在多个阶段重复序列化、clone和计数

FFI decode为Value，service又serialize计算payload bytes；snapshot/prepare产生Value后再serialize计算command/result，result还会重新encode给ABI。内部改用schema-bound owned types或canonical bytes/artifact，计数与ownership一次建立；JSON只留兼容边界。

### P1-35：真实FFI路径绕过`submit_json`的raw reservation

`submit_json`注释承诺decode前预留，却只有测试使用；dynamic API自己bounded decode后调用`submit`。统一`admit_encoded_request`，在分配DOM前同时占用transport和service budget，完成typed decode后原子转换reservation。

### P1-36：结果只能一次性整包harvest

大日志、Bake report、Cook manifest和诊断列表无法分页或流式消费；失败后也没有artifact定位。ResultStore支持inline small result、paged result和content-addressed artifact，receipt记录size/digest/schema/retention，caller逐页ack。

### P1-37：terminal retention与task admission耦合

Completed/Failed在TTL前占用task slot，pressure却只驱逐Cancelled/Expired/Harvested；结果未取走时新任务可能被拒绝。分离active task capacity与result/tombstone store，分别配置bytes/count/TTL和pressure policy。

### P1-38：Expired最终变Unknown，无法稳定解释结果去向

TTL把Completed/Failed改成Expired，后续admission又可删除tombstone；同一handle随时间得到不同错误且没有expiry receipt。保留bounded generation-aware tombstone或返回signed expiry token，明确ResultExpired与HandleStale，诊断能追溯最终原因。

### P1-39：harvest虽有两阶段提交，却没有caller级重试身份

foreign allocation失败可rollback harvest，这是优点；但调用重连后没有request identity、allocation receipt或幂等read token。将result retrieval绑定request/result generation，重复harvest可返回同一artifact metadata或明确AlreadyAcknowledged，而不是只靠进程task map。

### P1-40：completion receiver按dispatch batch增长并线性扫描

每个有dispatch的tick创建channel，receiver vector每次从头`try_recv`并`swap_remove`。改为每service一个bounded MPSC completion queue或scheduler completion port，producer持有permit；completion到达同时触发wake，drain按frame budget处理。

### P1-41：completion send失败被静默忽略

worker closure使用`let _ = completion_sender.send(completion)`，service drop、channel错误和teardown无法形成task receipt。发送失败必须完成owner task lease、递减census并记录session-closing/receiver-lost原因；shutdown等待这些lease归零。

### P1-42：maintenance refresh失败在状态转换后被吞掉

`refresh_maintenance_after_transition`和timer callback递归refresh都忽略错误。timer不可用时应进入可观察degraded mode：请求frame wake或由host maintenance scheduler接管，暴露health diagnostic，不能让deadline/TTL无声失效。

### P1-43：maintenance generation允许wrap且没有alarm receipt

`wrapping_add`理论上允许旧callback ABA，state只存deadline/subscription而无scheduler task identity。使用nonzero generation/epoch并在耗尽时fuse；alarm注册、取消、触发与重臂都记录sequence，测试stale callback绝不修改新session。

### P1-44：poisoned mutex直接恢复数据但不验证不变量

所有state/refresh/completion lock都`into_inner()`继续，若panic发生在临界区，accounting与phase可能已部分更新。将mutation缩成no-panic段，poison后运行invariant validator并fuse service；受影响任务返回Unknown/RecoveryRequired，而非继续执行。

### P1-45：accounting大量`expect`会把局部错误升级为session panic

retained bytes、in-flight count和harvest invariant依赖多个`expect`。把accounting收敛到reservation/permit RAII和checked transition API；debug构建assert，shipping构建fuse并导出census/diagnostic，禁止继续修改可疑state。

### P1-46：Operation Service没有显式shutdown状态机

没有Open/Closing/Draining/Closed状态、close admission、cancel all、drain deadline、force disposition或final census。实现幂等shutdown：先拒绝新请求，取消queued/ready，通知running，等待owned tasks与harvest归零，超时返回未完成清单而不是静默Drop。

### P1-47：dynamic session teardown没有调用Operation drain

`shutdown_before_library_unload`停event mirror、watcher和modules，却不处理operations；顺序甚至可能先卸载handler依赖再等待prepare。Operation drain必须位于module/plugin teardown之前，最终session destroy receipt包含active/forced/unknown任务数并由Runtime02/Interface01共同验收。

### P1-48：测试矩阵没有证明并发、顺序、wake、停机和产品一致性

现有core测试覆盖有价值的容量、panic、cancel queued、deadline和harvest，但没有simultaneous submit/cancel/poll/harvest、in-flight cooperative cancel、fairness、completion wake、timer failure、result paging、shutdown race或long soak。navigation plugin测试还保留旧poll签名并期待当前固定失败的Bake成功；该编译/产品P0继续路由Editor19，本篇新增Operation conformance suite。

## 9. P2：质量、诊断与开发体验差距

### P2-01：缺少可枚举的Operation catalog

提供只读descriptor枚举、filter与schema导出，供Editor、CLI和自动化生成控制面；它不能绕过capability检查。

### P2-02：缺少统一Debug与诊断快照

service应导出不含payload的queue/phase/owner/bytes/timing快照，`Debug`只显示计数和policy，避免打印敏感JSON。

### P2-03：没有稳定的trace correlation

request、worker task、owner apply、ABI call和Editor job应共享trace/span ID，使一次操作能跨线程和DLL定位。

### P2-04：错误文本没有localization与安全显示分层

UI使用message key + safe arguments，内部日志保留source chain；handler原始字符串不能直接成为用户界面或远端响应。

### P2-05：没有operator级list/filter/pagination

诊断工具需要按owner、phase、age、priority和resource查看任务，并以generation-safe pagination避免锁住全表。

### P2-06：没有历史聚合指标

记录submit/reject/cancel/deadline/outcome、queue/prepare/apply latency、retained peak、deadline miss和shutdown duration，并按operation type限基数。

### P2-07：没有可注入clock与scheduler测试夹具

将Instant/timer/scheduler抽成窄接口，测试无需sleep即可推进deadline、TTL、wake和completion顺序。

### P2-08：source guard测试依赖`include_str().contains()`

这些测试易被格式变化满足或破坏。关键契约改为行为测试、compile-fail测试和API shape test，source guard只保留少量架构禁令。

### P2-09：缺少property/fuzz/model-check测试

对phase transition、reservation accounting、harvest rollback和cancel/complete race运行property test；核心锁状态机使用loom或等价模型验证。

### P2-10：缺少基准与scale envelope

建立1/32/1024任务、small/large payload、burst submit、completion storm和terminal retention benchmark，发布CPU、alloc、lock wait与owner frame成本。

### P2-11：缺少handler开发模板与conformance harness

SDK提供typed handler skeleton、descriptor builder、cancel/progress检查点、no-mutation snapshot测试和effect receipt示例，避免插件重复犯合同错误。

### P2-12：缺少状态机与运维文档

文档应图示phase、linearization、cancel/deadline、retention和shutdown，列出每个ABI版本与兼容窗口，并链接真实资格receipt。

## 10. 分阶段重构计划

### M0 · 冻结扩展并建立契约测试

1. 暂停新增String/JSON handler和新的同步poll consumer。
2. 将当前phase、capacity、harvest正例固化为black-box conformance，新增snapshot只读compile-fail测试。
3. 对navigation旧签名测试和固定失败Bake保留canonical owner链接，不在Operation专项内临时改成假成功。

### M1 · Descriptor、identity与receipt

1. 实现`OperationTypeId`、`OperationDescriptor`、schema/version、owner/capability和effect class。
2. 实现request ID/idempotency、session epoch/generation handle以及typed error/disposition/receipt。
3. V1 adapter继续可用但进入legacy telemetry，所有新handler必须使用V2 typed contract。

### M2 · 确定性admission与scheduler

1. 用sequence + per-priority FIFO替代HashMap选择，建立aging、owner/handler quota和resource conflict key。
2. 将task/result/tombstone容量拆分，配置policy并提供retry-after。
3. 统一encoded request reservation，消除FFI decode与service budget之间的窗口。

### M3 · 只读snapshot、可取消prepare与可信commit

1. snapshot context硬切为只读，所有可失败预处理在mutation前完成。
2. prepare接owned task group、cancel/deadline/progress/budget context。
3. apply改为infallible publication或transaction/compensation receipt，最终result在commit后形成。

### M4 · Wake、ABI与result delivery

1. submit/completion/deadline触发coalesced wake，Operation参与session frame demand。
2. ABI增加V2 submit、cancel、subscribe/unsubscribe、descriptor query和paged/artifact harvest。
3. dynamic与in-process gateway共享同一能力，不再存在“DLL可用、in-process缺失”的产品分叉。

### M5 · Shutdown与plugin lifecycle

1. 实现Open/Closing/Draining/Closed与owner-scoped close/drain。
2. dynamic session在module/plugin卸载前排空Operation owned tasks，超时输出census和Unknown receipts。
3. 与Runtime02 task、Interface01 destroy、App01 process coordinator跑统一teardown matrix。

### M6 · 真实产品和规模资格

1. navigation Bake改用真实pure prepare artifact、bounded progress、cancel和generation commit，Editor使用事件驱动job而非16次poll。
2. 再接入一个非navigation重型consumer，证明catalog不是单功能特化。
3. 完成并发model、超载、公平性、长时、shutdown、ABI大结果和跨版本测试后，才允许声明Operation Service工程化完成。

## 11. Required 资格门

| Gate | Required evidence |
|---|---|
| G01 | selected source recheck、frontmatter路径和fingerprint通过 |
| G02 | 所有operation有validated descriptor、owner、schema与effect class |
| G03 | snapshot context在类型层不能获得World mutation authority |
| G04 | apply commit point与receipt disposition一致 |
| G05 | pre-commit failure保持live state不变，测试覆盖panic与error |
| G06 | post-commit failure不会被报告为普通Failed/NotApplied |
| G07 | request ID + fingerprint支持幂等retry和result replay |
| G08 | handle绑定session epoch/generation，foreign/stale分类稳定 |
| G09 | principal/capability在snapshot前验证并留下audit correlation |
| G10 | V2 request可表达priority、deadline、idempotency和origin |
| G11 | 相同submission trace产生确定性FIFO/priority selection trace |
| G12 | aging/fairness测试证明background和interactive均不饥饿 |
| G13 | global/owner/handler/resource quota在并发提交下不超配 |
| G14 | policy配置非法组合fail closed并导出effective values |
| G15 | owner snapshot/apply受count与time budget共同限制 |
| G16 | slow handler被测量、隔离并形成diagnostic，不拖垮整帧 |
| G17 | every prepare持有session-owned task lease与精确census |
| G18 | queued cancel撤回任务，running cancel合作式停止并确认终态 |
| G19 | deadline传播到scheduler与handler，超时工作不会无限继续 |
| G20 | TooLate cancel返回commit/disposition证据而非模糊错误 |
| G21 | progress支持indeterminate/units/sequence，禁止伪0/1进度 |
| G22 | terminal event lossless，progress event bounded/coalesced |
| G23 | submit、completion、deadline和ready apply都触发正确wake |
| G24 | Operation frame demand在reactive host中完成无animation任务 |
| G25 | dynamic与in-process gateway能力和错误合同一致 |
| G26 | ABI cancel/deadline/progress在旧host上安全协商或拒绝 |
| G27 | error code/stage/retryability/disposition跨FFI稳定 |
| G28 | 最大合法内部结果始终可inline、分页或artifact交付 |
| G29 | encoded admission在DOM分配前占用count/bytes permit |
| G30 | active task、result和tombstone使用独立容量与retention policy |
| G31 | harvest foreign allocation失败可重试且不重复ack/result |
| G32 | completion queue有界、单authority且arrival触发wake |
| G33 | timer不可用进入可观察degraded fallback，deadline仍推进 |
| G34 | poison/accounting invariant失败fuse service并输出census |
| G35 | shutdown close admission并按owner cancel/drain全部task |
| G36 | session destroy在module/plugin unload前取得Operation fence |
| G37 | navigation真实Bake成功、可取消、有进度且Editor不spin poll |
| G38 | 第二个重型consumer通过同一descriptor/conformance合同 |
| G39 | race/model/soak/scale基准满足已发布CPU、内存和frame预算 |
| G40 | `git diff --check`、finding计数、索引/coverage/总账和0重复路径通过 |

## 12. Currentness 与完成定义

| Evidence lane | 状态 | 日期 | 证据 |
|---|---|---|---|
| operation core逐文件审查 | review_complete | 2026-08-16 | 11文件、2,639行；admission/service/completion/maintenance/tests全覆盖 |
| ABI/dynamic/gateway链路 | review_complete | 2026-08-16 | V7仅submit/poll/harvest；submit无wake；frame demand无Operation |
| 唯一产品consumer | review_complete | 2026-08-16 | navigation四handler；Bake固定失败；Editor 16次poll；in-process缺能力 |
| source/reference fingerprint | review_complete | 2026-08-16 | 62文件、18,829行、952,092 bytes；SHA-256 `015542c0edfdf11b31978b61a8e0dde011c3e42f70949686a49fbf5481286eef` |
| five-engine comparison | review_complete | 2026-08-16 | Unreal/Godot/Bevy/Fyrox/Unity Graphics本地源码适用性已区分 |
| target architecture | design_complete | 2026-08-16 | 本篇第6节；descriptor/identity/scheduler/receipt/shutdown尚未实现 |
| production refactor | pending | - | 本轮review-only，未修改源码、测试、Cargo或ABI |
| dynamic qualification | blocked_by_implementation | - | G01-G40均未完成，不以既有局部unit test替代 |

当前结论是`review_complete / implementation_pending`。现有service可以继续作为受控实验底座，但在M0-M6与G01-G40完成前，不得把“有task handle”“有deadline”“能cancel queued”“有progress字段”“catch了panic”“使用bounded channel”或“navigation注册了operation”表述为工程级异步Operation系统。真正的完成标准是：请求身份和权限明确、调度可预测、取消与deadline停止真实工作、进度可信、commit effect可证明、结果一定可交付、reactive host会被唤醒、session/plugin shutdown能排空，并由至少两个真实重型产品consumer与规模测试共同证明。
