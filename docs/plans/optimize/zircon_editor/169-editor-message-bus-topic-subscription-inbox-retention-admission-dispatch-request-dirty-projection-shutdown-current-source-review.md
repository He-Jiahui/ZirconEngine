---
title: Editor Message Bus、Topic、Subscription、Inbox、Retention、Admission、Dispatch、Request、Dirty Projection 与 Shutdown 当前源码复核
category: zircon_editor
report_id: Editor169
review_date: 2026-08-27
baseline_head: 7fea65a3ae9cb836ad85adfdcece01ae7a6b7df1
production_baseline: 982baa1ba87bc8c25fe44312507a4af15027e058
canonical_owner: Editor48
refreshes:
  - docs/plans/optimize/zircon_editor/48-editor-message-bus-topic-subscription-inbox-retention-admission-dispatch-request-dirty-projection-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/121-editor-message-bus-topic-subscription-inbox-retention-admission-dispatch-request-dirty-projection-shutdown-current-source-review.md
related_code:
  - zircon_editor/src/core/editor_message
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/context/builder
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/context/tool_scheduler.rs
  - zircon_editor/src/core/editing/engine/events.rs
  - zircon_editor/src/core/editing/engine/transaction/lifecycle.rs
  - zircon_editor/src/core/i18n/service.rs
  - zircon_editor/src/core/jobs/event.rs
  - zircon_editor/src/core/jobs/pump.rs
  - zircon_editor/src/core/jobs/event_journal
  - zircon_editor/src/core/logging/service.rs
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/core/plugin/lifecycle_message_bridge.rs
  - zircon_editor/src/core/plugin/isolation.rs
  - zircon_editor/src/core/plugin/manager.rs
  - zircon_editor/src/core/plugin/registration.rs
  - zircon_editor/src/core/sync/pump.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/host/scene_inspection_publication.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/scene_hierarchy_refresh.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
tests:
  - zircon_editor/src/tests/editor_message
plan_sources:
  - docs/zircon_editor/core/editor_message.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/02/2026-07-22-message-inbox-backpressure-and-fanout.md
  - docs/plans/zircon_editor/editor/02/failure-2026-07-17-message-inbox-backpressure-and-fanout.md
  - docs/plans/zircon_editor/editor/14/failure-2026-07-17-job-pump-budget-and-pending-scan.md
  - docs/plans/zircon_editor/editor/14/2026-08-10-job-event-delivery-reservation-analysis.md
  - docs/plans/zircon_editor/editor/14/failure-2026-07-22-message-subscriber-result-consumer-drift.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/33-localization-string-table-culture-translation-import-export-fallback-pseudo-localization-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/47-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageBus.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageBus.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageSubscription.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Public/IMessageBus.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Public/IMessageContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Public/IMessageReceiver.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Public/IMessageSubscription.h
  - dev/bevy/crates/bevy_ecs/src/event/mod.rs
  - dev/bevy/crates/bevy_ecs/src/observer/mod.rs
  - dev/bevy/crates/bevy_ecs/src/observer/centralized_storage.rs
  - dev/godot/core/object/message_queue.h
  - dev/godot/core/object/message_queue.cpp
  - dev/Fyrox/editor/src/message.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Util/MessageManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/FixedBufferStringQueueTests.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 169 · Editor Message Bus / Dirty Projection / Subscriber Lifecycle 工程化复核

## 1. 最终结论

Editor Message Bus 已经有一组必须保留的工程基础：每次publication只构造一个immutable `Arc` payload；subscriber各有独立inbox；Lossless/Latest/Bounded retention、count和logical-byte预算、checked subscriber/delivery sequence、lossless fanout全有或全无预检、latest key索引和surviving global sequence顺序均存在。request callback在bus lock外执行；当前inbox eviction planner还从临时sequence `Vec`收敛为count-only plan，zero-route publish也不再浪费delivery sequence。这些实现不是临时占位。

但是Editor48唯一P0仍是 **Open**。`publish_view_invalidation()`只有在`EditorTopic::parse("view.invalidated")`失败时才直接`mark_view_dirty`；该topic合法，而且全生产树只有这一个producer、零subscriber。bus对zero-route返回无error空report，并且只有`delivered`非空才合并message dirty mark。因此真实refresh路径依然可以得到空dirty set。当前focused test声明`refresh_view`应得到dirty，但本轮没有运行测试，源码装配也没有隐藏subscriber，不能把测试期望写成修复事实。

Job链有重要新进展：事件先进入有界journal，具备journal sequence、progress coalescing、entry/byte/age预算、gap与backpressure/error `restore_front`；escaped job context也不能在terminal后继续发progress。但bus仍把zero-route当成功，所以journal record仍会永久pop。Log/I18n也新增了有界dispatch queue、authoritative store/catalog和resync；其resync adapter能识别零delivery为`NotConfigured`，普通record/locale/transaction adapter却仍把零delivery判为`Delivered`，形成同一authority内部的不一致成功语义。

Plugin lifecycle callback已有统一`catch_unwind`边界和faulted plugin state，这是可关闭的局部合同；bridge仍先把bounded bus inbox整箱drain进无界`VecDeque`，callback/manager busy时只把当前消息放回队首，下一tick继续追加新消息。Scene Inspection有previous/current generation、compact delta和resync，但producer先推进`previous` observation再忽略publish report；若最后一次publication丢失且没有后续变化，consumer永远得不到触发resync的下一条消息。

本轮不新增finding，继续由Editor48拥有1个P0、52个P1、15个P2。当前状态为：P0 **1 Open**；P1 **23 Open / 26 Partial / 3 Closed**；P2 **15 Open**；40个canonical gate为 **15 Fail / 25 Partial / 0 Pass**。没有dynamic race/shutdown/scale结果，也没有同场景跨引擎benchmark，禁止宣称这条消息控制面达到或超过Unreal。

## 2. 审查边界与 currentness

### 2.1 owner与去重

1. Editor169只刷新Editor48/Editor121，不重复登记Editor02的document/transaction authority、Editor09/14的Job scheduler、Editor11的Log store、Editor33的I18n product、Editor47/168的world watermark或Editor01的Retained UI owner。
2. 本报告拥有bus topic、subscription、inbox、dispatch disposition、delivery page、request、dirty commit和shutdown边界；领域owner负责使用这些合同完成产品闭环。
3. Tooling按用户要求排除；没有查询、轮询、等待或实时跟踪协调器。

### 2.2 冻结点

| 项目 | 当前值 |
|---|---|
| 当前磁盘冻结时间 | `2026-08-27T14:45:13.8320990+08:00` |
| Git HEAD | `7fea65a3ae9cb836ad85adfdcece01ae7a6b7df1` |
| production baseline | `982baa1ba87bc8c25fe44312507a4af15027e058` |
| selected working tree | 40条modified/untracked状态；本文以当前磁盘而非HEAD内容为裁决对象 |
| 动态证据 | 本轮未运行Cargo、Editor、race、shutdown或性能lane |

### 2.3 可复算selected set

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | Fingerprint |
|---|---:|---|
| Zircon bus与直接producer/consumer/tests | **80 / 14,884 / 13,540 / 536,430 / 113 / 6** | `719a2987bc31049835237bd5f0b53c631f1aca4a2120e702a527e9f6076583b9` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics | **17 / 6,515 / 5,537 / 219,669 / 74 / 0** | `b29f176fbc608761cf52a11e52a2658f48fc3c78037fcc534f3bd4fd35ee56b1` |
| 全部选择集 | **97 / 21,399 / 19,077 / 756,099 / 187 / 6** | `76e00b9dace2909630c10578d25a99b68f0a59b13dd2ddfa45a2a9804d120510` |

Fingerprint算法为workspace-relative小写`/`路径与逐文件SHA-256组成`path + NUL + hash + LF`清单，再对清单做SHA-256。Zircon scope递归展开frontmatter owner，统计当前物理文件；references是17个明确列出的本地文件。

## 3. 当前生产链事实

### 3.1 Bus核心不是无界Vec，但report仍不表达协议终态

`EditorMessageBus`以BTree索引维护subscriber/topic/inbox，delivery共享`Arc<EditorMessageDeliveryPayload>`。Lossless预先锁定按subscriber ID排序的所有inbox，任一无法准入则全组不enqueue；Latest与Bounded各有索引和容量/字节策略。2 MiB单delivery、16 MiB单inbox retained bytes与4096/256/256 entry默认预算仍是全bus固定值。

`EditorMessageDispatchReport`只有delivered/coalesced/dropped/backpressured subscriber数组和可选sequence-exhausted error。零target、route消失、subscriber关闭、atomic skip、required/optional target都不可区分。`prepare_publish`在zero-route时会把原message返回给内部caller，但公共`publish`立即丢弃它；因此“non-consuming admission”只存在于私有形状，producer无法重试。

### 3.2 P0控制流没有被当前优化修复

P0证据链仍完整成立：

1. `VIEW_INVALIDATED_TOPIC`只在Host定义并发布一次，全生产树无subscriber。
2. `view.invalidated`符合topic语法，`parse`成功，fallback `mark_view_dirty`永远不执行。
3. `prepare_publish`对zero-route返回空成功且不分配sequence。
4. `finish_dispatch`仅当delivered非空时`mark_message_dirty`。
5. `refresh_view`随后立即drain，得到空dirty；TREE_STRUCTURE与full reflection fallback均可能不触发。

zero-route不分配sequence是性能改进，不是正确性修复。目标必须把authoritative invalidation从optional message side effect中拆开，不能注册假subscriber或重新改成所有message无条件dirty。

### 3.3 subscriber unregister仍可能销毁或接收孤儿delivery

`register_subscriber`接受只有topics的descriptor，返回裸`u64` ID；没有owner、scope、generation、execution domain、criticality、debug label或lease。`unregister_subscriber`删除topic索引和inbox后只返回bool，未读Lossless/Bounded/Latest内容静默丢失。

Shared bus在metadata lock内snapshot `Arc<Mutex<Inbox>>`，解锁后dispatch。另一个线程可unregister并返回，但旧dispatch plan仍持有Arc并向orphan inbox enqueue；外部已经无法再按subscriber ID drain它。需要route generation与in-flight fence，而不是恢复全局长锁。

### 3.4 drain不是delivery page

`drain_deliveries`对inbox整箱`mem::take`，业务callback前就删除所有内容；返回值没有cursor、remaining、oldest age、ack/nack或tail restore token。inbox stats只有depth、lane depths、retained bytes、累计drained/coalesced/dropped/backpressured和以message sequence估算的age，不足以恢复部分处理失败。delivery global sequence是crate内部字段，serde和`PartialEq`明确忽略sequence，反序列化后归零，因此不能作为replay/wire合同。

### 3.5 request只有lock split，没有deadline与fault domain

request在bus lock外调用handler，避免了旧版重入死锁；完成response前会重新验证target是否仍注册，已有测试覆盖handler内unregister后返回`UnknownSubscriber`。但request已经enqueue给target，handler却由caller直接同步调用；没有correlation ID、deadline、cancel、panic boundary、executor/thread语义或target generation lease，callback panic仍可越过bus边界。

### 3.6 Plugin bridge把bounded inbox变成无界shadow queue

Plugin manager的lifecycle callback经`run_editor_plugin_boundary`隔离panic/错误，失败plugin进入Faulted且不阻断后续active plugin，这是Closed能力。bridge自身仍持`Mutex<VecDeque<EditorMessageDelivery>>`，每tick先完整drain bus再`pending.extend`。manager mutation busy时当前lossless消息被push_front并重试，顺序得以保留；pending没有count/bytes/age上限、deadline、quarantine或shutdown receipt，bus也看不到真实backlog。

### 3.7 Job journal前进明显，zero-route仍破坏终态语义

Job event journal有checked sequence、latest progress index、entry/byte/oldest-age预算、gap合并、高水位、sequence exhaustion和`restore_front`。pump对report error或backpressure会恢复record并停止；Job event source阻止terminal后的progress，focused tests覆盖progress burst与late context。

但零subscriber时report没有error/backpressure，pump计数后永久删除record；Started/Completed/Failed/Cancelled仍没有subscriber ack或consumer resync receipt。Job progress barrier局部可Closed，P1-27整体仍Partial。

### 3.8 Log/I18n有authoritative resync，但adapter成功语义分裂

Log service先写authoritative store，可选rolling file，然后用有界count/bytes dispatch queue通知sink；失败会折叠为`through_sequence` resync。I18n以catalog/settings generation为authority，有32条/64-byte事件队列并保留latest locale resync。bus只接收Log sequence或locale tag，不复制敏感日志正文，P1-44可关闭。

适配器问题仍在：resync明确检查`delivered().is_empty()`并返回NotConfigured；普通Log record与locale change只检查error/drop/backpressure，因此zero-route返回Delivered。Transaction adapter同样把zero-route判为Delivered，transaction engine只在Backpressured/Rejected时写warning，没有receipt或retry。

### 3.9 Scene Inspection具备generation/resync，却先推进producer observation

Scene Inspection保存previous artifact/focus/selection/fields，生成stable-entity anchor delta或显式resync；Retained consumer发现artifact generation不匹配时会从authoritative artifact重建。这是强底座。

`observe()`在publish前已经以`previous.replace(current)`推进producer observation，publish report完全忽略。若Latest/Bounded admission失败，后续消息可能检测gap并resync；若失败的是最后一次变化，就没有后续消息触发检测，UI长期停留旧状态。它需要prepare/commit/rollback observation或producer watermark只在consumer commit receipt后推进。

### 3.10 shutdown与diagnostics仍不是bus owner合同

bus没有Open/Closing/Closed状态。Host Drop只unregister Scene Inspection subscriber；Plugin bridge subscriber没有lease/Drop终态；producer、consumer、pending callback、dirty/UI delta和inbox没有统一drain/discard策略。inbox stats提供局部depth/bytes/age计数，但没有owner/topic/generation、oldest delivery sequence、route health、orphan delivery、resync和shutdown timeline。

## 4. 参考实现对照

### 4.1 Unreal Messaging是生命周期主参考，不是可靠队列完成证明

Unreal MessageBus区分receiver address、subscription object、message context、scope、sender/recipient、expiration、authorizer、tracer与router thread；subscription可Enable/Disable，recipient用weak ownership，bus有显式Shutdown delegate并join/kill router thread。Zircon缺少这些owner、authorization、expiration、trace与shutdown边界。所选Unreal实现并不提供Zircon所需的per-subscriber durable ack/page/byte budget，因此不能照抄其异步router后宣称exact delivery。

### 4.2 Godot MessageQueue提供显式有界queue/flush下限

Godot以page/size上限、mutex、flush guard和clear建立call queue边界，overflow返回明确错误。它主要是deferred call queue，不是topic/subscription control plane；可借鉴bounded admission与reentrant flush防护，不能替代subscriber lease或delivery receipt。

### 4.3 Bevy Event/Observer提供lifetime cleanup语义

Bevy Events/Observer将reader cursor、event update/drain与observer entity lifecycle分开，observer随entity/despawn清理，centralized storage负责注册索引。它没有Zircon跨plugin/Host的lossless inbox和shutdown receipt，但证明consumer lifetime不应只是裸数字ID和手写unregister。

### 4.4 Fyrox mpsc只是低基线

Fyrox Editor `MessageSender`封装标准mpsc Sender，message enum覆盖command/scene/mode/UI同步，send失败只交给Log verify；channel没有bus级topic、budget、subscriber generation、ack或replay。这不是Zircon应追随的工程上限。

### 4.5 Unity Graphics只提供provider scope与TryPush语义

ShaderGraph MessageManager按provider与node存储/清除诊断；FixedBufferStringQueue测试明确`TryPush`在容量不足时返回false且不破坏队列。它们支持owner-scoped state和可见admission failure，但不是完整Unity Editor message bus，也不提供generation/page/shutdown。

## 5. Editor48 finding重判

### 5.1 汇总

| 级别 | Open | Partial | Closed | 合计 |
|---|---:|---:|---:|---:|
| P0 | 1 | 0 | 0 | 1 |
| P1 | 23 | 26 | 3 | 52 |
| P2 | 15 | 0 | 0 | 15 |

### 5.2 P0

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| P0-01 zero-route `view.invalidated`仍提交authoritative dirty | Open | 唯一producer、零subscriber；合法topic绕过fallback，bus只在delivered非空时mark dirty。必须拆分authority并动态覆盖zero-route/oversize/backpressure。 |

### 5.3 P1

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| P1-01 typed dispatch disposition | Open | report仍是四组subscriber ID和单一sequence error；无NoRoute/Closed/Partial/AtomicSkip。 |
| P1-02 dirty authority与inbox acceptance分离 | Open | dirty仍是message side effect且无generation。 |
| P1-03 Custom topic owner/schema/capability/revision | Open | topic仅校验ASCII segment；schema只是任意String，无registry。 |
| P1-04 subscription owner/scope/generation/lease | Open | 只有裸ID与topic set。 |
| P1-05 global/topic/request sequence与correlation | Partial | global sequence存在但仅crate内部；topic sequence/correlation缺失。 |
| P1-06 Lossless reject保留原message | Partial | 私有prepare返回message；公共publish丢弃，producer不可用。 |
| P1-07 unregister pending disposition | Open | 只返回bool并销毁inbox。 |
| P1-08 publish/unregister route fence | Open | old Arc plan可在unregister返回后投递orphan inbox。 |
| P1-09 subscription lease revoke/Drop | Open | lease类型不存在。 |
| P1-10 bus Closing/Closed | Open | 无bus lifecycle状态。 |
| P1-11 delivery page cursor/remaining/age/ack | Partial | stats有depth和message-age；drain仍是无cursor/ack整箱Vec。 |
| P1-12 业务失败保留tail | Open | drain后无restore token；Plugin自行shadow pending。 |
| P1-13 Latest eviction key/resync | Open | report只有subscriber dropped；无evicted key/range或canonical resync request。 |
| P1-14 Bounded drop sequence/range/age | Partial | dropped count和age-in-messages存在；缺sequence range与reason。 |
| P1-15 sequence进入serde/replay DTO | Open | serde/PartialEq忽略sequence，decode重置为0。 |
| P1-16 budget按subscriber policy | Partial | count/bytes基础完整；所有subscriber共享固定limits。 |
| P1-17 actual memory与logical estimate分离 | Partial | ignored Windows RSS lane存在；生产diagnostics仍只有estimate。 |
| P1-18 request terminal result | Open | 无timeout/cancel/panic/retire disposition。 |
| P1-19 request lease revalidation/deadline | Open | 完成时只查裸ID是否仍存在；无generation/deadline。 |
| P1-20 request reentry与capability | Partial | lock外callback和unregister re-entry测试存在；无capability/admission。 |
| P1-21 plugin callback panic isolation | Closed | registration/lifecycle统一catch_unwind并转typed boundary failure，失败plugin可Faulted。 |
| P1-22 poison dead-letter/quarantine receipt | Partial | plugin fault state/diagnostic存在；delivery没有dead-letter identity或receipt。 |
| P1-23 slow subscriber公平隔离 | Partial | per-inbox并发与Latest/Bounded局部隔离存在；Lossless optional slow target仍阻断原子组。 |
| P1-24 plugin pending总预算 | Open | shadow VecDeque无count/bytes/age上限。 |
| P1-25 禁止无界shadow queue | Open | bridge仍先整箱drain再pending.extend。 |
| P1-26 retry/permanent/poison policy | Partial | manager busy会重试、callback failure使plugin faulted；无统一delivery disposition。 |
| P1-27 Job lifecycle reservation/ack/retry | Partial | journal sequence/gap/budget/restore-front成立；zero-route无ack仍丢terminal record。 |
| P1-28 Job progress不跨terminal barrier | Closed | latest-progress journal、terminal state gate与late-context测试形成精确合同。 |
| P1-29 Transaction failure进入receipt/diagnostic | Partial | failure会warning；transaction receipt和retry缺失。 |
| P1-30 zero-consumer不误称Delivered | Open | Log/Locale普通事件、Transaction和Job仍把zero-route当成功。 |
| P1-31 Scene dirty/world watermark原子commit | Open | producer observation先推进，report被忽略。 |
| P1-32 structure/field/resync payload区分 | Partial | SceneInspection有generation、hierarchy/fields/selection与resync；缺bus commit/page。 |
| P1-33 World Sync reject不推进watermark | Open | Editor168已确认report被忽略。 |
| P1-34 Document/Play generation过滤 | Open | typed payload有doc/play state但无producer generation/source lease。 |
| P1-35 topic adoption matrix | Partial | Document/Mode/Scene、Log/I18n有真实adapter；Tool/Focus等仍无产品consumer。 |
| P1-36 schema/namespace/oversize/depth fail-close | Partial | topic语法与byte admission存在；schema/capability/depth/ID length无registry policy。 |
| P1-37 Custom capability registry | Open | arbitrary parsed topic/schema仍可旁路。 |
| P1-38 跨document/world/PIE coalesce隔离 | Partial | SelectionDomain/WorldDomain/DocumentId/JobId进入部分key；SceneInspection仍单全局key。 |
| P1-39 callback失败保留last-known-good | Partial | Log/I18n/Scene有authoritative snapshot/resync；一般subscriber无ack与resync。 |
| P1-40 startup typed degraded | Open | Host构造register failure仍`expect` panic。 |
| P1-41 explicit shutdown | Open | 无producer-stop/drain-or-discard/revoke/close receipt。 |
| P1-42 diagnostics owner/topic/generation/backlog | Partial | inbox depth/bytes/age/counters存在；owner/topic/gen/orphan/resync/shutdown缺失。 |
| P1-43 Editor09 jobs承载resync/cancel/progress/shutdown | Partial | Job journal/progress/cancel/gap真实；bus lifecycle和产品resync job未接。 |
| P1-44 Log低敏payload与correlation | Closed | bus只发record sequence或through-sequence resync，不复制日志正文，authoritative store可回读。 |
| P1-45 retention/sequence/bytes/request deterministic race | Partial | 大量focused tests与request re-entry存在；dispatch/unregister fence race缺失。 |
| P1-46 publish/unregister、request/retire、shutdown barrier tests | Partial | request内unregister有覆盖；publish race与shutdown无合同。 |
| P1-47 zero-route dirty/reject/resync integration | Partial | zero-route sequence、backpressure与领域resync子集存在；P0 zero-route dirty未修。 |
| P1-48 plugin panic/slow/poison/backlog tests | Partial | panic和manager mutation busy有覆盖；slow deadline、pending budget和dead-letter缺失。 |
| P1-49 1/5/100/10K与100K规模 | Partial | ignored 1/5/100 fanout和100K zero-route证据存在；10K subscriber/100K delta缺失。 |
| P1-50 correctness/performance/real-runtime lanes | Partial | ignored managed lane已分离；required CI和real-product artifact不完整。 |
| P1-51 docs/manifest/runbook/currentness | Partial | owner docs/failure/current review存在；capability manifest与shutdown runbook未闭合。 |
| P1-52 删除假subscriber/无界shadow/false Delivered/raw topic | Open | 后三类坏模式仍在；本轮没有通过假subscriber修P0。 |

### 5.4 P2

| Finding | 状态 | 说明 |
|---|---|---|
| P2-01 durable journal/replay/checkpoint | Open | 通用bus无durable journal。 |
| P2-02 remote transport/version negotiation | Open | local serde不稳定。 |
| P2-03 QoS/priority/adaptive page | Open | page合同未建立。 |
| P2-04 payload dedup/zero-copy | Open | Arc共享只覆盖进程内。 |
| P2-05 health dashboard/operator controls | Open | 产品诊断未接。 |
| P2-06 plugin topic certification/revocation | Open | capability registry不存在。 |
| P2-07 million-message query/archive | Open | 无archive owner。 |
| P2-08 deterministic time-travel | Open | sequence不进入replay DTO。 |
| P2-09 distributed subscriber/relay | Open | 单进程lifecycle尚未闭合。 |
| P2-10 per-field privacy/redaction | Open | schema registry不存在。 |
| P2-11 virtualized message inspector | Open | 先完成canonical diagnostics。 |
| P2-12 adaptive frame-budget backpressure | Open | 当前固定limits。 |
| P2-13 chaos/crash/soak | Open | 无qualification artifact。 |
| P2-14 cross-platform benchmark | Open | 无公平可复现对照。 |
| P2-15 unified provenance browser | Open | Gateway/Snapshot/Collaboration journal尚未统一。 |

## 6. Canonical资格门

| Gate | 状态 | 当前判定 |
|---|---|---|
| G01 zero-route dirty正确性 | Fail | P0控制流仍成立。 |
| G02 oversized/backpressured dirty正确性 | Fail | authoritative dirty与message admission未分离。 |
| G03 sequence exhaustion fail-close | Partial | checked exhaustion存在，产品receipt未接。 |
| G04 NoRoute/Accepted/Rejected/Closed | Fail | typed aggregate不存在。 |
| G05 stale scope拒绝 | Partial | 部分coalescing key有domain，subscription无scope generation。 |
| G06 Lossless retry | Partial | 私有message return和Job restore存在，公共bus admission缺失。 |
| G07 Latest eviction/resync | Partial | coalescing/index和领域resync存在，report不精确。 |
| G08 Bounded drop证据 | Partial | count/age有，sequence range无。 |
| G09 unregister race fence | Fail | orphan enqueue仍可能发生。 |
| G10 lease revoke | Fail | lease不存在。 |
| G11 page cursor/ack/remaining | Fail | drain整箱删除。 |
| G12 serde sequence一致 | Fail | sequence被忽略并重置。 |
| G13 request timeout/panic | Fail | 无deadline/catch_unwind。 |
| G14 request re-entry | Partial | lock split和unregister test存在，本轮未动态运行。 |
| G15 topic/schema admission | Partial | topic语法有，schema/capability无。 |
| G16 plugin panic isolation | Partial | 源码与tests存在，本轮未执行qualification。 |
| G17 slow subscriber公平 | Partial | 局部per-inbox成立，required/optional group缺失。 |
| G18 poison/dead-letter | Partial | Faulted plugin有，delivery dead-letter无。 |
| G19 plugin pending预算 | Fail | unbounded VecDeque。 |
| G20 Job terminal delivery | Partial | journal/restore有，zero-route ack无。 |
| G21 Transaction receipt | Partial | warning存在，receipt/retry缺失。 |
| G22 Scene watermark commit | Fail | observation先推进且忽略report。 |
| G23 Document/Play generation | Fail | message未source-qualified。 |
| G24 Custom capability | Fail | raw parsed topic/schema旁路存在。 |
| G25 last-known-good/resync | Partial | Log/I18n/Scene局部成立。 |
| G26 startup degraded | Fail | subscriber注册失败panic。 |
| G27 shutdown order | Fail | bus lifecycle不存在。 |
| G28 diagnostics correlation | Partial | sequence/depth/bytes/age局部存在。 |
| G29 Editor09/11 integration | Partial | Job journal与低敏Log adapter真实，统一receipt缺失。 |
| G30 malformed/cross-scope/barrier | Partial | 部分syntax/domain/barrier tests存在。 |
| G31 1 subscriber profile | Partial | ignored managed evidence，未运行。 |
| G32 5 subscriber profile | Partial | ignored managed evidence，未运行。 |
| G33 100 subscriber profile | Partial | ignored managed evidence，未运行。 |
| G34 10K subscriber profile | Fail | 不存在。 |
| G35 100K delta/memory/backpressure | Partial | zero-route 100K和RSS局部证据，非delta产品矩阵。 |
| G36 required/managed/real-runtime lanes | Partial | managed分离，required/real-runtime不完整。 |
| G37 platform metrics | Partial | Windows RSS条件代码存在，未形成跨平台artifact。 |
| G38 docs/manifest/runbook | Partial | docs/current review有，manifest/runbook缺。 |
| G39 Editor02/47 owner acceptance | Partial | boundary已记录，commit合同未实现。 |
| G40 path/link/fingerprint/static quality | Partial | 本轮完成静态复核；动态与实现尚未完成。 |

## 7. 目标架构与 Hard Cutover

```text
Typed Producer
  -> RouteDescriptor(owner/schema/capability/scope/generation)
  -> prepare admission(message ownership retained)
  -> RouteLease + per-target disposition
  -> DeliveryPage(cursor/remaining/oldest/sequence)
  -> callback fault domain
  -> Ack / Nack / Retry / DeadLetter / Resync
  -> domain commit receipt -> dirty/watermark/projection commit

shutdown:
  stop producers -> close admission -> drain/discard by policy
  -> revoke leases -> join callbacks -> close bus -> terminal receipt
```

Hard cutover要求：

1. `view.invalidated`不再通过optional subscriber决定dirty authority；删除这条伪message或改为真正typed invalidation transaction。
2. 公共publish返回未消费message或reservation，report必须有NoRoute/Closed和逐target终态。
3. raw subscriber ID迁移为owner-qualified lease；unregister必须返回pending/discard/fence receipt。
4. `Vec<Delivery>` drain迁移为page lease，业务成功后ack；失败保留tail，不允许领域bridge再建无界shadow queue。
5. Scene/World/Job/Transaction/Log/I18n producer只在commit receipt后推进watermark；authoritative snapshot负责resync。
6. local delivery sequence不得冒充wire/replay ID；跨进程前另立versioned envelope与compatibility negotiation。

## 8. 分层重构计划

| Milestone | 内容 | 退出条件 |
|---|---|---|
| M0 | P0封口 | zero-route/oversize/backpressure都提交authoritative dirty，focused test动态通过。 |
| M1 | Route与disposition | owner/schema/capability/scope/generation、NoRoute/Closed、non-consuming admission冻结。 |
| M2 | Subscription lifecycle | lease、route fence、unregister pending策略、Closing/Closed与terminal receipt完成。 |
| M3 | Delivery page | cursor/remaining/age/ack/nack/tail retry/dead-letter和总预算完成。 |
| M4 | 领域接入 | Plugin pending删除、Job zero-route修复、Transaction receipt、Scene/World commit完成。 |
| M5 | Diagnostics与shutdown | owner/topic/generation/backlog/resync/orphan timeline和显式shutdown完成。 |
| M6 | 资格验证 | race/fault、1/5/100/10K、100K delta、Windows/跨平台、real product E2E完成。 |
| M7 | 超越性基准 | 同硬件/构建/负载比较latency、throughput、memory、drop、recovery和soak。 |

## 9. 逐owner检查台账

| Owner/文件簇 | 已检查的真实实现 | 仍需重构 |
|---|---|---|
| `core/editor_message/bus.rs`、`shared.rs` | route snapshot、lock split、request re-entry、global sequence、zero-route fast path | typed disposition、message return、route fence、bus close |
| `inbox.rs`、`retention.rs` | 三lane、count/bytes、indexes、count-only eviction、stats | policy descriptor、page/ack、eviction range、actual memory |
| `message/*`、`topic.rs` | typed payload、syntax、Scene generation/delta/resync | stable envelope、correlation、schema/capability registry、serde sequence |
| `editor_ui_delta.rs`、`view_dirty_set.rs` | path coalescing、barrier、mask merge | generation/page/budget/commit receipt |
| Host reflection | refresh/drain/full fallback结构 | zero-route P0，authoritative invalidation transaction |
| Scene Inspection | artifact generation、anchor/field/selection delta、consumer resync | prepare/commit/rollback producer observation与bus ack |
| Plugin lifecycle | callback panic boundary、Faulted state、retry manager busy | 删除无界shadow pending，deadline/quarantine/dead-letter/shutdown |
| Job journal/pump | sequence、gap、budget、age、progress coalesce、restore-front | zero-route ack、consumer receipt与durable resume边界 |
| Log/I18n adapters | authoritative store/catalog、有界queue、low-sensitive event、resync | 普通event zero-route语义、统一bus receipt |
| Transaction sink | typed lifecycle payload与warning | delivery影响transaction receipt并可retry/resync |
| focused tests | retention/bytes/order/request/refresh/100-subscriber/zero-route performance | P0动态证据、unregister race、shutdown、10K、product E2E |
| five references | Unreal lifecycle、Godot bounds、Bevy cleanup、Unity scope/TryPush、Fyrox低基线 | 只吸收边界，不复制其弱ack/queue语义 |

## 10. 完成定义与本轮closeout

本轮只写review和重构计划，不修改生产代码。没有运行Cargo、Editor、thread race、callback panic、backpressure、shutdown、1/5/100/10K、100K delta、RSS、soak或跨引擎benchmark，因此任何依赖动态证据的gate最多为Partial。

Editor48只有在P0关闭、52项P1逐项有实现与产品消费证据、40门全部Pass后才可完成。下一实现顺序必须从M0开始：直接拆开dirty authority与message delivery；禁止通过注册永不消费的`view.invalidated` subscriber、扩大queue或给producer做永久deep clone来掩盖协议缺口。
