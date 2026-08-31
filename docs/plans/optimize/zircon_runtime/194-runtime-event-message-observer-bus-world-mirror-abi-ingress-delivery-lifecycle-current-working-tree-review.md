---
title: Runtime Event、Message、Observer、Core Bus、World Mirror、ABI Ingress、Delivery 与 Lifecycle 当前源码复核
category: zircon_runtime
report_id: Runtime194
review_date: 2026-08-30
baseline_head: 399f2318150ae4fa0df3a2543133b03b80099288
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundation_product_incomplete
source_recheck_required: true
canonical_owners:
  - Runtime02
  - Runtime05
  - Runtime54
  - Runtime55
refreshes:
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/54-runtime-scene-event-mirror-registration-subscription-cursor-backlog-overflow-reclaim-abi-consumer-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/158-runtime-core-events-tasks-timer-event-bus-task-graph-current-source-review.md
related_code:
  - zircon_runtime/src/core/framework/events.rs
  - zircon_runtime/src/core/framework/foundation/event_manager.rs
  - zircon_runtime/src/core/runtime/events
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/foundation/runtime/event_manager.rs
  - zircon_runtime/src/scene/ecs/events
  - zircon_runtime/src/scene/ecs/messages
  - zircon_runtime/src/scene/ecs/observer
  - zircon_runtime/src/scene/ecs/system/events.rs
  - zircon_runtime/src/scene/ecs/system/messages.rs
  - zircon_runtime/src/scene/event_mirror
  - zircon_runtime/src/scene/world/events.rs
  - zircon_runtime/src/scene/world/messages.rs
  - zircon_runtime/src/scene/world/event_mirror.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/events
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime_interface/src/runtime_api/session/events.rs
  - zircon_runtime_interface/src/runtime_api/session/plugin_event_mirror.rs
  - zircon_app/src/entry/runtime_entry_app/event_dispatch.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_editor/src/core/gateway/session/plugin_events.rs
reference_engines:
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
  - dev/bevy/crates/bevy_ecs/src/message/message_cursor.rs
  - dev/bevy/crates/bevy_ecs/src/message/message_registry.rs
  - dev/bevy/crates/bevy_ecs/src/observer/mod.rs
  - dev/godot/core/object/message_queue.h
  - dev/godot/core/object/message_queue.cpp
  - dev/godot/core/object/object.h
  - dev/godot/core/object/object.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Public/IMessageBus.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Public/IMessageContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/MessagingCommon/Public/MessageEndpoint.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageBus.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Delegates/MulticastDelegateBase.h
  - dev/Fyrox/fyrox-ui/src/message.rs
  - dev/Fyrox/fyrox-ui/src/lib.rs
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Debugging/DebugWindow.cs
---

# Runtime194 当前源码审查

## 1. 结论

本轮把 Runtime 的事件链作为一条纵向产品合同重新逐文件扫描，而不是只复读 Core `EventBus`。当前源码实际存在四类并行机制：Core topic bus、World 双缓冲 typed events 与同步 observer、World retained messages、Dynamic ABI ingress 与 Runtime event mirror。分层本身合理，问题是没有一份编译后的 taxonomy、owner、schema、budget、receipt、sequence、shutdown contract 说明何时必须使用哪一类，也没有把它们接到同一 session/world/provider generation 与生命周期证据上。

当前实现有可保留底座：Core bus 对同 topic 保序，支持 `Lossless`/drop-oldest/latest、阻塞/超时读取和 sampled diagnostics；World events 有 active-channel worklist、reader lease、逐项 cursor commit 和容量回缩；Messages 有 entry/byte/age retention、drop metrics 和逐项 cursor commit；mirror 有 bounded JSON writer、16,384 events/64 MiB subscription queue、64-event/128 KiB page、foreign output commit/rollback；ABI 有 version 和 slice/JSON 上限。它们不是空壳。

但工程闭环仍失败。Core `EventManager` 在 production 调用图中没有解析者，retained `MessageReader/Writer` 同样没有产品消费者；普通 `Events<T>` 单帧无 entry/byte budget，slow reader 跨 generation 丢失不可见；observer 在 producer 栈同步执行，panic、递归和 bool 拒绝语义不受控；mirror 每订阅重复 observer、JSON encode、mutex 和 64 MiB queue，Runtime 又在 foreign allocation 后提前提交而没有 consumer ack/resync；V1 ABI 用一个宽平 struct 承载所有事件并在持有 session mutex 时直接修改 input/UI/camera/world/clock，多类浮点和 UTF-8 校验不一致，App 将任何事件错误升级为进程退出。

本轮不新增唯一 P0。Runtime55 `FND-P0-003` 与 Runtime54 `SEMR-P0-001..003` 共 **4 项继承 P0 均保持 Open**；新增当前源码细化账本为 **36 项 P1（30 Open / 6 Partial）**、**12 项 P2（10 Open / 2 Partial）**，资格门 **32 项（25 Fail / 6 Partial / 1 Pass）**。Runtime60 旧 `RECS-P1-62` 所述 MessageCursor 在创建 iterator 时确认全部尾部的结论已过时：当前 Event/Message cursor 都在 `Iterator::next` 时逐项推进，因此该局部问题重判 Closed，但不能代替 gap、retention 和生命周期合同。

本轮只写 review、索引和 coverage；没有修改 production Rust、tests、Cargo、ABI、ZUI 或 tooling，没有运行 Cargo、真实 App/Editor、动态 DLL、Miri、loom、fuzz、fault、scale、soak 或 benchmark。静态复核不能证明性能或表现已经达到、更不能证明优于 Unreal。

## 2. 审查边界与物理冻结

### 2.1 选择集

| 范围 | 文件 | 行 | bytes | test attributes | 说明 |
|---|---:|---:|---:|---:|---|
| Core event bus、Foundation manager 与 focused tests | 25 | 3,717 | 129,663 | 41 | topic、subscriber、delivery、prune、diagnostics、manager facade；另有 5 个 `#[ignore = ...]` 自报 benchmark |
| Scene events/messages/observers/mirror 与 focused tests | 39 | 8,083 | 273,315 | 96 | queue、cursor、lease、store、system param、World update、mirror reclaim |
| Dynamic ABI、App producer、Editor transport consumer | 48 | 12,712 | 460,961 | 68 | V1 ingress、session lock、plugin event page、foreign output、consumer pump |
| 第一方 typed event producer | 7 | 2,410 | 95,041 | 12 | AI、Animation、Navigation、Net、Physics 的实际 `World::send_event` 调用点 |
| 去重 Zircon 选择集 | **118** | **26,183** | **930,328** | **209** | fingerprint `e7f89751721e91e088e2f793f3d954815306a952a629d181b2a7a4e41673c39c` |
| 五引擎参考选择集 | **21** | **20,765** | **753,515** | **58** | fingerprint `e2bdccc7a6346b03d3b20aae331346b9d42456e5542246fc394ffa0327096f3d` |

fingerprint 算法为相对路径小写并转 `/`，每文件 SHA-256，以 `path|hash` 按路径排序并用 LF 连接，最后计算 UTF-8 SHA-256。它只冻结本轮读取集合，不是 event schema、stream generation、BuildSet 或发布身份。

### 2.2 已读与未宣称范围

本轮逐读 Core bus 的 DTO、topic map、copy-on-write subscriber snapshot、delivery mutex、queue/condvar、drop/prune、diagnostics 和行为/结构/benchmark 源码；逐读 Scene event/message queue、cursor、lease、observer、store、system param、World update 与 mirror registry/queue/reclaim；沿 dynamic FFI、session store、App dispatch、Editor gateway 追踪输入与跨 DLL 输出；搜索第一方 `send_event`、Core `publish_event/subscribe_events`、MessageReader/Writer 的 production adoption。

本轮没有展开 UI 自有 `UiEventManager`、Editor transaction events、Resource/Asset 自有 event stream、Sound dynamic events、network protocol 内部消息、logging/telemetry 或 tooling。它们各自有 owner，不能被 Runtime194 代替。Unity Graphics 只作为 domain-scoped callback lifecycle 的窄参考，不把 rendering debug callback 当成通用 event bus 标杆。

## 3. 当前实现闭环

### 3.1 Core topic bus

- `EngineEvent` 只有公开可变 `topic: String` 与 `payload: serde_json::Value`；`EventManager::publish` 和 `EventBus::publish` 都返回 `()`（`core/framework/events.rs:11-22`、`core/framework/foundation/event_manager.rs:5-11`、`core/runtime/events/publish.rs:41-55`）。
- topic map 是 `RwLock<HashMap<String, Arc<EventTopic>>>`；每 topic 有 subscriber snapshot mutex 和 delivery mutex。publish 在 delivery mutex 下依序对每 subscriber 入队，因此同 topic 保序，但 publisher 等待最慢 fanout 临界区（`core/runtime/events/topic.rs:12-18,118-129`、`publish.rs:60-93`）。
- `Lossless` 映射为 `capacity = None`；bounded/latest 只按 entry 数 drop，没有 charged bytes、per-owner/global credit 或 durable spill（`subscriber.rs:76-101`）。
- subscribe 直接接受任意 `String`，subscriber id 用 `AtomicU64::fetch_add`，subscriber snapshot 每次 add/remove 复制；没有 namespace/schema/provider/session/world 或 generation（`topic.rs:30-75,147-190`）。
- bus 只在最后一个 `Arc<EventBusState>` drop 时 deactivate subscriber；没有显式 close admission、deadline drain、owner census 或 shutdown receipt（`topic.rs:111-123`）。
- `DefaultEventManager` 的 `CoreWeak` 失效时 publish 静默丢弃，subscribe 返回伪造的 disconnected subscription（`foundation/runtime/event_manager.rs:26-60`）。production 搜索只找到 facade/registration，没有真实 Core EventManager consumer。

### 3.2 World typed events、observer 与 messages

- 所有 `'static + Send + Sync` 类型自动实现 `Event`，没有显式 declaration/schema/owner；`EventTypeId` 是裸 `u32`，注册时把 `channels.len()` 直接 cast（`scene/ecs/events/id.rs:1-18`、`events/store.rs:77-93`）。
- `Events<T>` 是 `current/next Vec<T>`，`send` 与 `send_batch` 无硬预算；generation 和 capacity metrics 用 saturation。`IndirectRecommended` 只根据 `size_of::<T>() > 128` 提示，并不改变存储（`events/queue.rs:8-63`、`events/metrics.rs:3-48`）。
- `EventCursor` 在 generation 不同后从当前 buffer 的 0 开始，无法报告期间错过了多少 generation/event。逐项 iterator commit 已正确，部分读取不会确认尾部（`events/cursor.rs:20-53,96-107`）。
- observer 在 `send` 栈内同步运行；单发聚合 bool 后无论 false 仍入队，batch 则完全丢弃 observer bool。callback panic 没有隔离，递归/耗时/side effect 没有 budget（`events/store.rs:219-230,242-269,346-354`）。
- observer 通过 `connect_untracked` 加入 reader count，retention reader 与 synchronous tap 被混在一个数字中。Observer ID exhaustion 返回 `None`，但没有 owner/generation/typed terminal diagnosis（`events/store.rs:116-145`）。
- `EventStore::clone` 返回空 default，`PartialEq` 恒 true；`MessageStore` 完全相同。这会让 World clone/equality 隐式丢弃 live queue/registration/cursor state（`events/store.rs:366-375`、`messages/store.rs:140-149`）。
- Messages 默认保留 1,024 entries、256 KiB、600 frames，并报告 budget/age drop；但 `Message::retained_byte_size` 默认只算栈内 size，heap owner 依赖每种消息自觉 override（`messages/id.rs:5-12`、`messages/queue.rs:16-46`）。
- write 先分配 ID、push、再 enforce budget，消息可能立即被驱逐但调用者仍只得到成功 ID；ID exhaustion panic，frame/generation/metrics saturation 无 terminal flag（`messages/queue.rs:82-102,142-219`、`messages/store.rs:87-102`）。
- `MessageCursor` 对 retention eviction 有累计 `dropped_count`，但 clear/generation mismatch 直接重置到现存窗口且不报告 loss。逐项 ack 已修复（`messages/cursor.rs:34-54,79-80`）。
- production 精确搜索没有 `MessageReaderParam`/`MessageWriterParam` 或 `write_message` consumer；这套 retained message 目前仍是 API/test 基础。

### 3.3 Runtime event mirror

- descriptor 只有 `event_id` 与 `payload_schema` 两个最大 128 bytes 的自由字符串；无 provider/plugin/BuildSet generation、schema digest、scope、delivery class 或 recovery contract（`scene/event_mirror/registration.rs:18-30,176-220`）。
- 每个订阅创建独立 typed observer 和 `Mutex<RuntimeEventMirrorQueue>`；每次 `send_event` 在 producer 栈逐订阅执行 bounded JSON serialization。单订阅上限 16,384 events/64 MiB，subscription count 本身无 World/session/global 上限（`event_mirror/subscription.rs:32-64,91-140`）。
- queue 的 payload/page size、depth、processing time 与 first-failure retention 是真实保护，但只能限制单订阅；S 个订阅仍是 S 次 encode、S 把锁、S 份 bytes 和 S 倍 deadline（`subscription.rs:100-199,218-324`）。
- reader-count callback 是可任意修改 `&mut World` 的闭包。subscribe/unsubscribe 尝试 rollback，但没有 panic boundary、reentrancy contract、idempotency key 或 external side-effect receipt（`registration.rs:80-114`、`world/event_mirror.rs:51-126`）。
- slot generation overflow 时跳过 free slot并继续增长 vector；registration、slot、subscription 和 retained bytes 没有整体 admission（`registration.rs:264-299`）。
- Drop intent 需要 World owner 再 reclaim；`World::drop` 只调用一次 shutdown 并丢弃 callback failure/retry report（`world/event_mirror.rs:165-254,297-300`）。
- Dynamic session sequence 是每 subscription 在 drain 时分配，而不是 publish 时的 source sequence；foreign allocation完成后 commit 就从 pending page 移除，Editor decode/typed apply 失败没有 ack/nack/resync。`zircon_app::RuntimeSession` 还把 backlog metadata 丢掉，只返回 deliveries（`dynamic_api/session/event_mirror.rs:119-180,336-356`、`zircon_app/.../runtime_session.rs:522-554`）。

### 3.4 V1 ABI ingress

- `ZrRuntimeEventV1` 是一个公开 `repr(C)` 宽平 tagged struct，所有 kind 共用 viewport/size/metrics/x/y/delta/button/state/pointer/key/scan/payload；没有 source sequence、timestamp/clock、device/window generation、user、trace、delivery class 或 consumed disposition（`zircon_runtime_interface/.../events.rs:10-24`）。
- FFI 校验 ABI version 后取得 session action并锁住整个 `RuntimeDynamicSession`，再由一个大 `match` 直接修改 input reducer、runtime UI、camera、World、clock、clipboard/accessibility（`dynamic_api/session/ffi.rs:168-178`、`events.rs:58-140`、`registry/session_store.rs:181-198`）。
- 只接受 `DEFAULT_VIEWPORT`。每个事件独立进入 FFI/lock，没有 batch、move/wheel coalesce、queue stage、frame boundary 或 source ordering receipt（`events.rs:58-61`）。
- pointer/motion/touch/gamepad 等多条路径未统一验证 finite/range；wheel 新分支检查 finite，但 legacy delta、pointer坐标、touch坐标、gamepad axis/button value 未检查（`events.rs:75-124,265-356,372-420`、`events/gamepad.rs:48-93`）。
- keyboard/gamepad UTF-8 失败静默变成 `None`；未知 keyboard action 返回 OK；repeat 永远 false。invalid payload 与 ignored input 因而不可区分（`events/keyboard_ime.rs:26-78`、`events/gamepad.rs:23-45`）。
- `event_payload` 把借用 FFI pointer 的 slice 暴露为 `&'static [u8]`。当前 caller 同步消费，尚未观察到 escape，但签名本身不成立并为未来缓存制造 unsound 入口（`events.rs:597-608`）。
- UI consumed 与 gameplay/camera mutation 顺序按 kind 分散，部分路径先写 input 再问 UI，部分 text 先问 UI；没有统一 capture/propagation contract。App 把任何 `handle_event` error 记录后直接 `event_loop.exit()`（`events.rs:75-140,265-356`、`zircon_app/.../event_dispatch.rs:8-27`）。

## 4. 继承 P0 当前状态

| ID | 状态 | 当前源码复核 |
|---|---|---|
| `FND-P0-003` | Open | Foundation 注册 EventManager 为 immediate service，但 production 无 resolver/consumer；仅 facade 与测试 roundtrip，能力仍可伪绿。 |
| `SEMR-P0-001` | Open | AI 等第一方 producer 继续使用普通 `World::send_event`；跨 DLL mirrored exposure 不是同一 registration authority，Editor consumer capability 仍可能不可订阅。 |
| `SEMR-P0-002` | Open | sequence 仍在 drain 时生成，foreign allocation 后 commit；decode、continuity、typed callback 在 commit 之后且没有 ack/nack/resync。 |
| `SEMR-P0-003` | Open | 每订阅 observer/encode/queue 仍存在，64 MiB 只是单订阅上限，subscription count 与 aggregate CPU/RSS 无界。 |

这些 P0 的 canonical owner 仍是 Runtime54/55；Runtime194 只提供 2026-08-30 current-source 复核，不把它们重复加入全局唯一总数。

## 5. P1 差距与重构内容

| ID | 状态 | 当前差距 | 需要重构的内容 |
|---|---|---|---|
| `REV-P1-01` | Open | 四套 event/message 路径没有编译后的 taxonomy 与选择规则 | 建立 `CompiledEventContract`，明确 frame event、retained stream、synchronous signal、cross-boundary stream、input command 的 owner/ordering/retention/receipt |
| `REV-P1-02` | Open | Core envelope 只有 String+JSON | 增加 stable contract id、provider generation、session/world/source、sequence/frame/time、trace、schema identity；进程内优先 typed payload |
| `REV-P1-03` | Open | topic 任意字符串且 publish 可隐式创建/静默找不到 | topic 只能来自 frozen registry；namespace、长度、owner、capability、schema 和 lifecycle 在 activation preflight 验证 |
| `REV-P1-04` | Open | Core publish 返回 `()` | 返回 `PublicationReceipt`：accepted/coalesced/dropped/backpressured/closed/no-subscriber，以及精确 stream range |
| `REV-P1-05` | Partial | 有 drop-oldest/latest，但 Lossless 无界且所有策略只按 entry | 统一 entry+charged-byte+owner/global credit；可靠流要 backpressure、bounded spill 或显式 fail，禁止无限内存 |
| `REV-P1-06` | Partial | same-topic delivery mutex能保序，但 publisher 同步 O(subscribers) fanout | 保留确定性顺序，改为 bounded broker log/cursor；publisher只做一次 admission/publish，不逐订阅持 queue lock |
| `REV-P1-07` | Open | subscriber snapshot mutation O(N)，ID 可 wrap/collide，无 subscription cap | slot+generation handle、checked exhaustion、owner quota、bulk revoke和quiescent unsubscribe |
| `REV-P1-08` | Open | subscription 只有 blocking recv/try/timeout | 增加 cancellation/deadline、async wake、bounded page read、ack 与 terminal status；不得让 worker shutdown靠永久 condvar |
| `REV-P1-09` | Open | EventBus 无显式 admission close/drain/report | runtime/module/session owner执行 close producers -> wake receivers -> drain/abandon policy -> terminal receipt |
| `REV-P1-10` | Open | bus、mirror、session registry把 poisoned lock恢复后继续成功 | poison 进入 typed failed/poisoned state并关联 owner/invariant；只允许显式 repair 或 fail-close |
| `REV-P1-11` | Partial | sampled aggregate diagnostics 存在，但无 per-topic/subscriber budget/gap/overflow truth | descriptor-backed metrics按 contract/provider/session 分组，计数有 overflow flag，snapshot带 generation/window/consistency |
| `REV-P1-12` | Open | Core manager失效时静默 publish、伪造 disconnected subscribe，且无产品 consumer | manager API返回 typed unavailable；未有真实 consumer、health 与 teardown 证据前不得 Ready，迁移后删除裸 Core facade |
| `REV-P1-13` | Open | World `Events<T>` 单帧 entry/bytes 无界 | contract定义 channel/producer/world/global budget和 overflow/coalesce policy；send返回 typed receipt |
| `REV-P1-14` | Open | EventCursor 跨 generation 只从当前 0 开始，silent loss | stream generation+monotonic range；cursor返回 missed range/gap，critical consumer必须 fail/resync |
| `REV-P1-15` | Open | public `update_events<T>/update_all_events` 与 frame maintenance可并行成为退休 authority，generation saturation隐藏终态 | 单一 frame/schedule owner推进；manual update需要 lease/phase proof；checked epoch rollover或channel retirement |
| `REV-P1-16` | Partial | payload profile能提示 >128 bytes，但不改变布局/预算 | declaration选择 inline/Arc/arena/encoded segment，charged bytes包含heap，compiler拒绝不匹配策略 |
| `REV-P1-17` | Open | observer bool、event queueing、validation/tap/delivery混为一体，panic/递归无隔离 | 拆 `Validator`、`Tap`、`DeliveryObserver`；callback写 deferred command，具 depth/work/time/panic policy和receipt |
| `REV-P1-18` | Open | batch send完全忽略 observer bool | batch预留、逐项/整批 disposition 和 atomicity明确；拒绝不能被 `usize written` 掩盖 |
| `REV-P1-19` | Open | observer计入 reader count，callback与retention lease生命周期混淆 | 分开 CursorLease、ObserverLease、MirrorBrokerLease；owner/provider generation、in-flight与unregister barrier独立统计 |
| `REV-P1-20` | Open | EventStore clone清空、equality恒真 | 删除欺骗性 trait；World snapshot/clone显式声明 queue、registration、cursor、observer 的 preserve/drop/unsupported policy和receipt |
| `REV-P1-21` | Open | EventTypeId cast u32，observer/readers exhaustion缺统一 typed policy | World-qualified slot+generation registry，checked retirement、stale error和provider unload revoke |
| `REV-P1-22` | Open | Message byte charge默认只算栈内 size | 使用 sealed `RetainedSize`/arena allocation receipt，heap/Arc共享按统一策略计费，debug下校验 undercharge |
| `REV-P1-23` | Open | Message write即使立刻被budget驱逐仍返回成功ID | reserve/commit后返回 retained/coalesced/dropped/backpressured receipt；ID只代表可观察的publication attempt |
| `REV-P1-24` | Open | MessageId exhaustion panic，frame/generation saturation不终结 | checked allocator和channel epoch；exhaustion关闭admission、retire channel并产生 owner diagnostic |
| `REV-P1-25` | Open | MessageStore clone/equality与EventStore同样丢状态 | 与 WorldProjectionPolicy 合并，不能以 trait convenience 隐式删除 retained message |
| `REV-P1-26` | Partial | retention eviction有 dropped_count、逐项ack已正确；clear/generation mismatch仍无loss disposition | cursor返回 `ReadPage { range, gap, terminal }`，clear/reconfigure/reload都发布 generation transition |
| `REV-P1-27` | Open | retained Message API没有第一方 production consumer | 为真实跨帧需求建立 adoption matrix；若无需求则删除重复系统，不保留只在测试里“完成”的产品能力 |
| `REV-P1-28` | Open | mirror descriptor是自由 event/schema 字符串 | stable exposure id、provider/BuildSet generation、schema version/digest、scope、delivery/recovery policy进入manifest与ABI |
| `REV-P1-29` | Open | reader-count callback可mutate World且Drop丢 teardown report | 收敛为 broker activation lease；panic/reentry/idempotency/rollback明确，World/session shutdown必须消费最终 receipt |
| `REV-P1-30` | Open | mirror registration/slot/subscription无aggregate admission，slot overflow静默放弃复用 | World/session/provider多层quota，checked retired-slot metric，subscribe返回effective budget和拒绝原因 |
| `REV-P1-31` | Open | source event无publish sequence，wire无consumer ack/resync | shared immutable log在publish时分配 `(stream_generation, sequence)`；wire lease与semantic ack分离，支持nack/checkpoint/snapshot |
| `REV-P1-32` | Open | V1 flat struct缺source/time/device/window/user/generation且字段按kind复用 | ABI使用小型固定header + versioned typed payload；输入身份、时钟、sequence、viewport/device generation不可丢 |
| `REV-P1-33` | Open | 一个大 match 在session lock内跨域直接mutate | `IngressRouter`先validate/normalize/admit，再按 Input/UI/Window/Clock/EditorCommand stage执行，顺序由compiled policy控制 |
| `REV-P1-34` | Open | finite/range/UTF-8/action校验不一致，`&'static [u8]`借用签名错误 | 每kind schema validator统一错误；borrow lifetime绑定FFI call，需保留则显式copy/foreign lease；未知值不得静默OK |
| `REV-P1-35` | Open | 每事件一次FFI+session mutex，无batch/coalesce，任一错误让App退出 | batch ingress、pointer/wheel keyed coalesce、per-event disposition；protocol/session-fatal与recoverable input error分级 |
| `REV-P1-36` | Partial | Editor gateway保留 runtime backlog，App `RuntimeSession`只返回delivery Vec；consumer truth分叉 | Runtime App/Editor共享一个 page DTO与continuity/ack contract，所有surface保留backlog/gap/policy generation |

## 6. P2 性能、诊断与维护差距

| ID | 状态 | 差距与目标 |
|---|---|---|
| `REV-P2-01` | Open | Core publish反复分配/hash String；contract编译为interned stable ID并保留debug label |
| `REV-P2-02` | Open | subscribe/unsubscribe复制 Arc slice且remove排序/搜索；以slot table+immutable epoch snapshot降低大fanout mutation成本 |
| `REV-P2-03` | Open | 每subscriber持Arc、Mutex、VecDeque、Condvar，cacheline与allocator密度未测；按delivery class采用compact cursor/segment布局 |
| `REV-P2-04` | Partial | mirror bounded writer避免超大/超深JSON，但仍按subscriber重复编码和App再decode；一次编码、共享bytes、必要时typed zero-copy view |
| `REV-P2-05` | Partial | queue/page常量有硬上限但不能由validated profile协商；编译effective budgets并发布generation |
| `REV-P2-06` | Open | hot atomics未做cacheline布局、contention或NUMA分析；用真实fanout profile决定shard/padding，不盲目加锁或原子 |
| `REV-P2-07` | Open | u64 counter可wrap，纳秒转f64毫秒丢长时精度；使用windowed integer histogram和overflow bit |
| `REV-P2-08` | Open | Core 5个性能测试是ignored、自打印、单机瞬时数据；纳入可复现Release harness、环境manifest与regression threshold |
| `REV-P2-09` | Open | 无 1/64/1K subscriber、1B/1KiB/128KiB payload、paused consumer、mixed topic fairness 的稳定p99/RSS矩阵 |
| `REV-P2-10` | Open | 无 loom/Miri/fuzz、lock poison、callback panic、reentrant subscribe/unsubscribe、generation exhaustion系统验证 |
| `REV-P2-11` | Open | 无100h editor/server soak与queue/reader/subscription conservation、RSS slope、late callback census |
| `REV-P2-12` | Open | Event/EventManager/Message/Observer命名过泛且跨子系统重复；公开docs和lint必须要求delivery class后缀与owner链接 |

## 7. 参考引擎证据与适用边界

| 参考 | 可直接借鉴 | 不能照搬 |
|---|---|---|
| Bevy Messages/Observer | `Messages` 双buffer和单调message count；cursor显式 `missed_messages`；MessageRegistry拥有统一update authority；Observer测试覆盖递归、顺序、传播停止、unregister、deferred apply | Bevy frame messages本身也可能增长且不是跨DLL可靠流；其unsafe World内部不能替代Zircon的provider/session generation与ABI ack |
| Godot Signal/CallQueue | CallQueue按page/max_pages硬限，单消息过大/queue OOM返回Error；flush显式拒绝重入且每项解锁执行；signal发射前复制slot snapshot，one-shot先disconnect，支持deferred/persistent/ref-counted连接 | Variant/Callable/ObjectID模型不适合作为Rust typed storage；Godot signal不是durable stream，不为无ack mirror背书 |
| Unreal Messaging/Delegate | IMessageContext包含sender/recipient/scope/flags/time/expiration/attachment/original context；MessageBus有router thread、register/subscribe和显式Shutdown；Endpoint有thread affinity、enable/disable、inbox与SafeRelease；delegate有lifetime tracker和broadcast期mutation规则 | UE message bus API不自动提供Zircon需要的byte credit/ack；UObject/TaskGraph/global subsystem lifetime不能替代DLL generation和Rust teardown receipt |
| Fyrox UI/Engine events | UiMessage显式destination、direction、routing strategy、delivery mode和handled；poll返回processed count；bubble/preview/direct route分层；OS event先更新engine input再调用plugin hook | mpsc与直接plugin callback不等于有界可靠bus；UI routing只适合域内消息，不应升级为全引擎万能事件模型 |
| Unity Graphics Debug callbacks | domain manager注册/注销callback成对，Editor window在enable/destroy边界显式解除；说明consumer lifecycle必须归owner | 这是Rendering Debugger局部callback，不具message context、budget、cursor、gap或跨DLL语义，只作为窄生命周期证据 |

关键结论不是选择某一个参考引擎的单一机制，而是像这些引擎一样先区分 synchronous signal、frame message、deferred call、endpoint messaging 和 domain callback。Zircon要超过它们，必须再增加统一的 typed contract compiler、global byte credit、provider/session/world generation、ack/resync和可验证shutdown，而不是把所有事件塞进 String+JSON 或一个万能queue。

## 8. 目标架构

```text
EventContractCompiler
  -> CompiledEventContract
       identity/schema/provider/scope
       delivery class/order/retention/recovery
       entry+byte budgets/overflow policy
       lifecycle owner/diagnostics contract

Producer
  -> validate + reserve
  -> publish once -> PublicationReceipt
       | frame event channel      (deterministic frame retirement + gap)
       | retained stream broker   (range cursor + ack + retention)
       | observer graph           (validator/tap/delivery + deferred commands)
       | mirror broker            (shared encoded segment + consumer ack/resync)

ABI Input
  -> versioned envelope validator
  -> bounded ingress queue/coalescer
  -> staged Input/UI/Window/Clock/EditorCommand routing
  -> EventIngressReceipt

LifecycleOwner
  -> close producer admission
  -> terminate/wake subscriptions
  -> drain or record abandoned ranges
  -> revoke provider generation
  -> EventShutdownReceipt + census
```

目标不是统一所有payload存储，而是统一它们的声明、身份、receipt、预算、诊断和生命周期。frame event可以保留双buffer，retained message可以保留deque/segment，observer可以保持同步validator的窄用途；但它们不能继续共享模糊的 `bool`/`usize`/`()` 成功表面。

## 9. 分层重构里程碑

### M0：真相与继承 P0

- 冻结 event taxonomy、owner matrix、真实producer/consumer和capability truth。
- 先关闭 Runtime54 三项 mirror P0 与 Runtime55 Foundation false-ready；没有真实consumer的服务降级或删除。
- 为现有四套路径建立 source fingerprint、adoption inventory 和删除清单。

### M1：Contract、identity 与 receipt kernel

- 引入 `EventContractId`、provider/session/world generation、schema identity、delivery/recovery policy。
- 建立 `PublicationReceipt`、range/gap、typed terminal status和分层entry/byte credit。
- Event/Message/Observer/subscription allocator统一checked slot+generation，不允许wrap/saturate/panic隐藏耗尽。

### M2：Core bus hard cutover

- 用compiled topic ID替代公开String topic；publish返回receipt。
- reliable flow改为bounded broker/backpressure；subscriber改range cursor，显式close/drain。
- 迁移唯一真实consumer后删除 `CoreHandle::publish_event/subscribe_events` 和 fake disconnected facade。

### M3：World events/messages/observer

- 普通Events增加budget、gap和唯一frame maintenance owner。
- Messages使用可信charged bytes、reserve/commit receipt与clear/reconfigure generation event。
- Validator/Tap/DeliveryObserver拆分，callback使用deferred lane、panic/depth/work policy；删除store伪Clone/PartialEq。

### M4：Mirror shared log、ack 与 resync

- 每contract只安装一个World broker adapter，producer一次序列化为shared immutable segment。
- subscription只持cursor/credit；sequence在publish时分配，wire lease与semantic ack分开。
- ABI增加ack/nack/checkpoint/dropped range/snapshot token；unload/reload按provider generation fail-close。

### M5：ABI ingress normalization

- 新版ABI使用固定header+versioned payload，完整携带source/time/sequence/device/window/viewport generation。
- 一次FFI提交bounded batch，先validate/normalize/admit，再按compiled stage执行；pointer/wheel/gamepad可声明coalesce。
- 消费、忽略、拒绝、recoverable error与session-fatal分级，App不再因一个坏input无条件退出。

### M6：第一方迁移与旧路径删除

- AI/Animation/Navigation/Net/Physics 按真实需求声明 frame/retained/mirrored contract。
- App与Editor共享page/backlog/gap/ack DTO；删除只返回Vec的旁路。
- 没有产品consumer的 retained Message 或 Foundation EventManager 不保留兼容壳；hard cutover后结构测试禁止旧API复活。

### M7：竞争性资格

- loom/Miri/fuzz/fault、callback panic/reentry、generation exhaustion、DLL reload与shutdown census。
- 1/64/1K subscriber，mixed payload/topic，paused/failed consumer，multi-session/multi-World，100h soak。
- Release benchmark固定CPU/OS/build/profile，报告p50/p95/p99、throughput、allocation、lock wait、RSS、gap与fairness；再与参考引擎同场景对比。

## 10. 资格门

| Gate | 状态 | 通过条件 |
|---|---|---|
| G01 | Fail | 所有production event/message均能映射到编译后的delivery class与canonical owner |
| G02 | Fail | contract具有stable identity、schema/provider/session/world generation且跨ABI可验证 |
| G03 | Fail | 每个Ready event service至少有真实producer、consumer、health、shutdown证据 |
| G04 | Fail | publish/write统一返回accepted/drop/backpressure/closed/gap receipt |
| G05 | Partial | 所有channel有entry+charged-byte+owner/global budget；目前仅Message和单subscription mirror局部有界 |
| G06 | Fail | 不存在以内存无限增长实现的Lossless |
| G07 | Fail | slow reader、retention、clear、reload、overflow都返回连续range或typed gap |
| G08 | Fail | observer panic/递归/耗时受policy限制，失败不破坏World commit |
| G09 | Fail | runtime/module/world/session teardown有close/drain/revoke receipt和late callback census |
| G10 | Fail | World clone/snapshot明确守恒event/message/observer状态或typed拒绝 |
| G11 | Fail | Event/Observer/Subscription/Message allocator exhaustion不wrap/saturate/panic |
| G12 | Partial | retained message byte charge覆盖heap/shared allocation；目前依赖type手写override |
| G13 | Fail | message立即evict时producer收到明确disposition而不是成功ID |
| G14 | Pass | Event/Message cursor部分迭代只确认已yield项，尾部保持未读并有行为测试 |
| G15 | Partial | Message retention loss有计数；frame Event与generation reset仍无range gap |
| G16 | Fail | mirror exposure来自manifest/compiled schema，不以两个自由字符串判兼容 |
| G17 | Fail | mirror同一source event只编码一次，subscriber不各自安装observer/复制payload |
| G18 | Fail | mirror aggregate CPU/RSS受World/session/provider/global credit约束 |
| G19 | Fail | source sequence在publish admission时分配并跨subscriber可关联 |
| G20 | Fail | Editor/App在semantic apply成功后ack，失败可retry/quarantine/resync |
| G21 | Fail | reader-count/provider teardown callback失败可重试且最终receipt不被Drop吞掉 |
| G22 | Partial | ABI version、slice、JSON/page有界；finite/range/UTF-8/action校验仍不完整 |
| G23 | Fail | 多viewport/window/device generation正确路由且stale event被拒绝 |
| G24 | Fail | ingress支持bounded batch、coalesce和source ordering，不是每event全session锁 |
| G25 | Fail | UI capture/propagation与gameplay/camera mutation顺序由单一policy决定并有receipt |
| G26 | Fail | recoverable bad input/consumer failure不会无条件终止App或永久吞delivery |
| G27 | Partial | diagnostics能显示aggregate queue/timing；仍缺contract维度、gap、budget、overflow truth |
| G28 | Partial | order/drop/cursor/retention/mirror bounds有单测；缺跨层fault、reload和product acceptance |
| G29 | Fail | loom/Miri/sanitizer证明publish/unsubscribe/reclaim/shutdown竞争安全 |
| G30 | Fail | 可复现Release benchmark证明目标场景p99、allocation、lock wait和RSS达标 |
| G31 | Fail | 100h多session/editor/server soak中range、lease、subscription、RSS conservation成立 |
| G32 | Fail | 第一方AI/Animation/Navigation/Net/Physics/App/Editor全部使用canonical contract且旧路径被结构守卫禁止 |

## 11. 实施前硬约束

1. Runtime194 是 review，不授权以兼容 facade、双写、静默 fallback 或新全局 singleton 延长旧路径。
2. 先关闭 Runtime54/55 继承 P0，再实施普通性能优化；不能用更快的重复JSON encode掩盖无ack与无global budget。
3. 不把所有机制合并成万能bus。frame、retained、observer、mirror、input的payload/storage可以不同，但contract/lifecycle/receipt必须统一。
4. 任何 ABI 变更与 Runtime54、Runtime43、Interface owner共同版本化；旧版在迁移完成后hard cutover删除，不长期双truth。
5. 实施前重取118文件选择集与相关producer/consumer fingerprint；当前共享工作树持续变化，本文的line evidence不是未来源码的替代品。
6. Tooling 按用户要求排除；本篇不以未来Rust tooling迁移阻塞Runtime事件架构review。
