---
related_code:
  - zircon_runtime/src/scene/ecs/change_detection
  - zircon_runtime/src/scene/ecs/events
  - zircon_runtime/src/scene/ecs/messages
  - zircon_runtime/src/scene/ecs/observer
  - zircon_runtime/src/scene/ecs/lifecycle.rs
  - zircon_runtime/src/scene/ecs/removal.rs
  - zircon_runtime/src/scene/ecs/system/removed_components.rs
  - zircon_runtime/src/scene/event_mirror
  - zircon_runtime/src/scene/world/change_detection.rs
  - zircon_runtime/src/scene/world/events.rs
  - zircon_runtime/src/scene/world/event_mirror.rs
  - zircon_runtime/src/scene/world/observers.rs
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassObserverManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassCommandBuffer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp
  - dev/bevy/crates/bevy_ecs/src/lifecycle.rs
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
  - dev/bevy/crates/bevy_ecs/src/world/mod.rs
tests:
  - current observer event message mirror and change tracking slice 38/38 files and 12 inline tests statically reviewed
  - related behavior integration and structure tests 8/8 files and 90 tests statically reviewed
  - direct rustfmt 45/46 passed; one foreign-dirty dynamic API file has import-order drift
  - managed Windows zircon_runtime lib-test compile failed; focused tests and profiles did not run
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime ECS observer/event/messaging current-source结构性能复审（2026-08-14）

## 范围、快照与已生效改进

本轮完整复审observer、typed event、retained message、change wrapper、removed component、runtime event
mirror及plugin ABI直接桥接，共 **38/38个生产Rust文件、5,023行、4,458个非空行、12条内联测试**；
另复审 **8/8个直接相关测试文件、3,260行、90条测试**。生产与测试manifest指纹分别为
`CF2E6767D7D8E712A338C8741E01E9DA1B238CE2D89D4FBA61291F9ED3ECB4DE`和
`9FEFF96E75D34DFD2AD221983BD18B97D31935F7C31E418306709613BFB4E45C`。38个生产文件中14个为其它
Session修改，本轮只读取，不覆盖其源码。

当前源码已有三项必须保留的改进。observer已按`(operation, component)`、event type和
`(event type, entity)`直接建bucket，并用`observer_locations`反向定位删除，不再触发时扫描异构observer；
event/message store只维护dirty channel worklist；message已有entries、bytes、age三重retention，runtime mirror
也有64 events/128 KiB page与16,384 events/64 MiB subscription queue硬门。plugin raw drain复用producer侧JSON
bytes，不再先还原`Value`再编码ABI。这些修复改善了旧实现，但没有改变逐entity通知和逐subscription发布算法。

## P0：生命周期热路径在无observer时仍逐entity分配并同步回调

`World::trigger_component_lifecycle`先从descriptor取type name并构造`ComponentLifecycleEvent`，随后才查
observer bucket。事件构造把`String`内容重新`Arc::from`为`Arc<str>`，所以没有任何observer时每条Add、Insert、
Replace、Remove、Despawn仍有一次type-name heap allocation/copy。普通insert/spawn对每个component发布Add+Insert；
N个entity、C个component的批量提交静态上可产生`2*N*C`次事件构造，100,000 x 31即6,200,000次，尚未计
`staged_lifecycle_events`增长。该数字是调用结构上界，不是allocator实测值。

有observer时，每条事件再`Arc::clone`一个`BTreeMap` snapshot并逐callback同步持有`&mut World`执行。当前
keyed BTree删除比旧slice重建好，但dispatch从连续callback slice退成tree traversal；dispatch期间注册/删除还会
触发`Arc::make_mut`复制整个bucket。更关键的是callback可任意修改World，导致它不能透明批处理、并行或交给已有
compiled system access调度。deferred/bundle路径即使已经按transaction提交，最后仍逐entity/逐component发布。

Runtime08/03/11必须先拆合同：MVP内建热observer使用`operation + component + archetype/entity ranges`的
`LifecycleBatchView`和显式command buffer/compiled access；legacy `Fn(&mut World, &event)`保留确定顺序与可重入
语义，但作为有预算的兼容lane在batch边界展开。staged transaction只保存compact key/range，确认目标bucket存在后
才物化legacy event；descriptor type name必须复用注册期interned owner。不能只把BTree换HashMap而保留逐entity同步
发布。

## P0：plugin event fanout按subscription重复锁、序列化和保留payload

每个runtime mirror subscription都会向同一`EventChannel`注册一个`TypedEventObserver`。`send_event`对S个observer
逐个调用，callback各自获取subscription `Mutex`、执行`serde_json::to_vec(event)`并把独立`Vec<u8>`压入独立
`VecDeque`。因此E个event、S个subscription的核心工作是E*S次downcast/callback、Mutex和JSON serialization，
payload bytes也保留S份。当前单subscription 64 MiB门只限制局部队列；1,000个慢subscription的合同聚合上界仍为
62.5 GiB。这里同样是常量推导上界，不代表当前产品RSS。

EventStore还把queue reader和同步observer都折叠进`reader_count`，但`send_by_id`无论是否存在queue reader都会把
typed event再写入双buffer；mirror是唯一consumer时，事件已同步编码一次，原始T仍被额外保留。ABI drain又在每个
delivery重复写同一event id和schema，预留capacity却只统计payload bytes。当前10,000-event与10,000-subscription
测试只验顺序、hard limit和回收，没有serialization count、lock、copy、allocation或耗时门。

Runtime10/Plugins01/Runtime11应建立type-level fanout broker：每个event只编码一次并进入共享分段payload log，
subscription只持sequence/cursor/lag budget；低于所有cursor的segment统一回收，慢consumer按显式overflow或disconnect
合同隔离。ABI page应在page header携带一次descriptor，或由Runtime10明确保留v1重复字段的兼容成本。producer到
broker、broker到ABI page的线程边界必须由WPR/Tracy证明；可以借用worker job，但不能把主线程同步stall搬到无界后台
队列。

## P0：removed component记录没有frame生命周期，且每次读取重新分配

`RemovedComponentEvents`为每个TypeId持有一张只增不减的`Vec<RemovedComponentEvent>`和重复type-name String。
当前源码中没有`update`或`clear`调用；`World::clear_trackers`只复制change tick，正常frame driver也不维护removed
buffer。因此进程生命周期累计R次removal就永久保留R条记录，所有reader cursor也以该全历史Vec为基准。每次
`RemovedComponentReader::read`又把未读slice复制到新`Vec<EntityId>`，SystemParam再把它转回iterator。

Runtime03/08应让numeric ComponentId直接索引active removed channels，并采用两个frame window或带sequence的有界
ring；frame/standalone `clear_trackers`必须有唯一update authority。reader直接返回borrowed/range iterator，慢reader
丢失量需显式计数。Bevy `RemovedComponentMessages::update`每帧swap/clear旧buffer，`World::clear_trackers`同时推进
removed messages；Zircon需要同等生命周期，不必复制Bevy API。

## P1：event与message retention仍会把burst成本留在主线程

`Events<T>`有capacity shrink策略，但没有entry/byte硬门；`EventPayloadProfile::IndirectRecommended`只是标签，不改变
存储。单帧producer burst可无界增长，8个low-water frame后又在主线程为current/next各分配replacement并搬运。
应按event type声明lossless、coalescing、latest-only或bounded delivery，而不是统一静默drop；只存在sync sink时不应
同时保留queue copy，lossless lane必须把backpressure暴露给producer/schedule。

message基础队列已正确有界，但`write_batch_at_frame`先按iterator原始lower bound reserve，再逐message执行
`enforce_budget + refresh_retention_metrics`。一个1,000,000项batch即使`max_entries=1,024`也会先请求百万级capacity，
随后立即淘汰绝大部分且没有shrink合同。cursor read还在iterator消费前把`next_id`推进到queue末尾，部分消费会丢掉
tail；构造read iterator用循环跳过start，工作随保留prefix增长。Runtime03/11应使用budget-aware reserve、逐项硬门但
batch末尾一次metrics publish、range cursor和按消费提交的sequence，保持entries/bytes/age语义。

## Change detection与测试false-green

system `Mut<T>`已做到只在`DerefMut/as_mut/set_changed`后写changed tick，raw World mutable API则明确eager mark；这项
语义不应被性能修复暗改。Added/Changed仍扫描全部matching rows、diagnostic scan常驻成本和chunk binding已归入
PERF-MVP-605/613，本切片不重复建立第二套change authority。

现有测试大量通过`include_str`固定容器和函数正文。`event_and_message_batch_writers_preallocate_from_size_hint`仍要求
`ids.push(self.write(message))`，当前生产源码实际为`ids.push(self.write_at_frame(message, frame))`，静态即不满足；
observer测试还明确锁定`Arc<BTreeMap>`。这些守卫既不能量化成本，也会阻止结构收敛，应替换为行为、复杂度、
allocation与backpressure门。

本轮direct rustfmt为45/46通过；唯一失败是其它Session修改的
`dynamic_api/session/event_mirror.rs` import order，本轮不代改。managed Windows focused lib-test沿用D盘coordinator
结果：843.4秒后以361个编译错误、1,520条warning失败，0条本切片测试执行。当前没有可运行binary，WPR/xperf、
Tracy、allocator profile和RenderDoc均未运行；RenderDoc后续只验证draw/dispatch/readback无回归，不能证明CPU
observer/messaging瓶颈消失。

## Unreal主依据、补充依据与统一计划

UE Mass `MassObserverManager.cpp:266-278,431-460`直接接收archetype entity collections，先用observed element
bitset求交，再把多个collection交给observer pipeline；`467-506`保证同一processor每batch最多执行一次。
`MassCommandBuffer.cpp:170-217`在command flush期间持有observer/creation lock并到operation group边界统一释放；
`MassObserverManager.cpp:687-739`再消费合并后的buffered collection通知，而不是每次row move立即回调。

UE Messaging `MessageRouter.cpp:53-63,256-287`在router runnable/command queue上处理路由，`118-181`共享同一
thread-safe message context，并按receiver thread直接调用或派发task。该依据支持“共享payload + 明确线程lane”，不
表示Zircon应复制UE线程数或把所有ECS lifecycle异步化。Bevy仅作为Rust ECS补充：`lifecycle.rs:436-473`用
ComponentId sparse set持有removed message double buffers，`world/mod.rs:1706-1709`在clear trackers更新；
`message/messages.rs:149-163,189-200`让batch extend和frame buffer update具有单一owner。

| task / owner | 结构目标 | 必须证明的验收 |
|---|---|---|
| PERF-MVP-614 / Runtime03+08+11 | lifecycle用typed key和entity/archetype range批量发布；interned descriptor；legacy mutable callback为有预算兼容lane | no-observer event/name allocation=0；batch count随key/range而非N*C增长；final signature、顺序、reentrant add/remove、rollback等价；worker/command access合法 |
| PERF-MVP-615 / Runtime10+Plugins01+Runtime11 | type-level一次编码的共享分段broadcast log；subscription cursor/lag budget；ABI page descriptor收敛 | E events下serialize calls=E而非E*S；payload retained近O(E+S cursor)，event-path lock不随E*S；slow consumer隔离、sequence/order/error/v1 contract与shutdown通过 |
| PERF-MVP-616 / Runtime03+08 | removed component以ComponentId active channel和双window/ring维护；frame update单一owner；borrowed iterator | 长时removal retained受两window/预算约束；read allocation/copy=0；direct/deferred/despawn order与slow-reader dropped count正确 |
| PERF-MVP-617 / Runtime03+08+10 | event type声明queue demand和lossless/coalesce/latest/bounded policy；observer-only不保留typed queue副本；capacity回收不制造主线程尖峰 | no-reader/observer-only/queued矩阵的queue writes、alloc、drop/backpressure符合合同；burst steady p95/p99与retained bytes受门；无静默loss |
| PERF-MVP-618 / Runtime03+11 | message batch按retention预算reserve、逐项保持硬门且一次publish metrics；cursor按实际消费提交并直接定位range | 1M input/1,024 retention不保留百万capacity；batch O(M)、metrics publish O(1)/batch；partial read不丢tail，cursor seek不线性扫retained prefix |
| PERF-MVP-619 / Runtime03+08+10+11+Plugins01 | 删除容器源码形状门；增加batch/serialize/lock/copy/queue/lag/allocator与线程诊断 | disabled counter overhead=0；至少20 warm samples输出p50/p95/p99、RSS、CSwitch/ReadyThread/energy；WPR/Tracy/allocator/F2与RenderDoc回归证据齐全 |

动态矩阵覆盖entities/events 0/1/1k/100k、components 1/8/31、observer/subscription 0/1/8/1k、payload
0/32/1KiB/128KiB、burst/steady/idle、reader none/sync/queued/mirror、message retention 0/1,024/1M和slow consumer。
记录event construction/name allocation、bucket snapshot/tree ops、batch/range、serialize/lock/copy、typed/encoded retained
bytes、queue high-water/shrink/drop/age、cursor lag、main/worker time、p50/p95/p99、RSS、cache miss、CSwitch/
ReadyThread与energy。同一硬件同一场景前后数据完成前，不宣称达到UE经验值、功耗接近或算法最优。

本切片继续留在`pending.md`，不进入`review.md`；动态门与独立复核完成前，不提交性能里程碑，也不发送企微完成
消息。
