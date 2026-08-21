---
related_code:
  - zircon_editor/src/core/editor_message
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/context/tool_scheduler.rs
  - zircon_editor/src/core/editing/engine/transaction/lifecycle.rs
  - zircon_editor/src/core/i18n/service.rs
  - zircon_editor/src/core/jobs/event.rs
  - zircon_editor/src/core/jobs/pump.rs
  - zircon_editor/src/core/logging/service.rs
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/core/plugin/lifecycle_message_bridge.rs
  - zircon_editor/src/core/sync/pump.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
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

# 48 · Editor Message Bus / Topic / Subscription / Inbox / Retention / Admission / Dispatch / Request / Dirty Projection / Shutdown 工程化差距

## 1. 结论

`core::editor_message`已经有一批不能推翻的工程基础。每次publication只构造一个`Arc`共享payload；每个subscriber拥有独立inbox mutex；`Lossless / Latest / Bounded`保留策略、4096/256/256条目预算、2 MiB单消息与16 MiB retained logical bytes预算、checked subscriber/delivery sequence、lossless fanout全有或全无预检、latest key索引、全局surviving sequence顺序，以及request回调前释放bus锁均已真实落地。旧报告中“无界Vec、fanout深拷贝、request持总线锁回调、ID饱和复用”的结论已经过时，本轮不重复登记。

但当前消息基础设施仍不是可承担工程级Editor控制面的可靠协议。最直接的产品回归发生在dirty projection：`EditorHostEventController::publish_view_invalidation()`把失效包装成合法的`view.invalidated` custom消息；该topic在生产中没有subscriber，而bus只在至少一个inbox接受消息时调用`mark_message_dirty()`。topic解析成功使显式`mark_view_dirty()` fallback永远不执行。因此`refresh_view()`、`refresh_workbench()`以及资产、状态、扩展注册、组件和场景文档路径可以返回空dirty set，既不刷新完整reflection，也不发布structure-only场景增量。现有refresh测试正断言相反结果，但本轮review-only没有把受已知编译阻断的Cargo运行冒充动态通过。

更深层的问题是协议把“事件事实”“最新状态提示”“view invalidation”和“产品操作回执”混在一个只返回subscriber ID数组的dispatch report中。零target是空成功；lossless backpressure后原消息所有权已经被消费；unregister会无条件销毁未读lossless inbox；shared dispatch在metadata lock外执行，注销返回后仍可能向已移除的orphan inbox投递；drain又在业务处理前整箱删除，没有ack/nack/cursor/page。sequence不对consumer公开，serde和`PartialEq`还故意忽略sequence，因而这套结构无法直接扩展为重放、持久订阅、跨进程或exactly-once合同。

产品采用同样没有闭合。Document与Play消息被plugin lifecycle bridge消费，但producer不处理backpressure；bridge每tick先把bus inbox整箱搬到无界`pending`，callback失败时只把队首放回，后续tick继续把新消息追加到shadow queue，等于绕过bus预算。Transaction、Job、Tool、Focus、Log和I18n topic在生产中没有与其声称语义一致的真实subscriber；Job pump还先pop再忽略report。Scene Inspection虽然有唯一真实retained consumer和generation-resync基础，但publish失败后producer observation已推进；若最后一次大结构变更超过2 MiB而没有后续变化，UI可长期停在旧状态。

本报告新增 **1项P0、52项P1、15项P2和40个资格门**。P0只计dirty invalidation当前产品断路；Job终态投递归Editor09/14、Log零consumer归Editor11、I18n归Editor33、World Sync错误推进归Editor47、Document/Transaction权威归Editor02，它们在本文仅作为message owner的依赖和集成矩阵，不重复累计父报告P0。

## 2. 审查边界、语料与 currentness

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | 证据等级 | 本轮检查重点 |
|---|---:|---|---|
| `core/editor_message`完整模块 | 35 / 2,935 / 93,365 | E3 | bus、shared lock split、inbox、retention、delivery、topic、request、dirty与UI delta逐文件 |
| 聚焦测试 | 13 / 1,642 / 59,366 | E3 | 42个test attributes、1个managed ignored性能lane；保留策略、bytes、顺序、request与refresh合同 |
| 直接产品生产者/消费者 | 18 / 6,796 / 253,338 | E3 | Context sinks、transaction、job、play、tool、plugin bridge、world sync、scene publication、retained tick与shutdown |
| owner计划、failure与模块文档 | 8 / 1,408 / 123,947 | E2/E3 | 已落地修复、开放non-consuming admission、job reservation和旧结论currentness |
| Unreal、Bevy、Godot、Fyrox、Unity Graphics参考 | 17 / 6,515 / 219,669 | E2/E3 | subscription lifetime、receiver identity、shutdown/tracing、bounded call queue、observer cleanup与provider-scoped state |
| selected combined scope | 91 / 19,296 / 749,685 | E2/E3 | 工作树fingerprint `3b7e94575b94bc84a45e0b99e3681b89eeafaddd371d2edac711e47c16c4a7da` |

指纹按91个selected path去重排序，对每个文件取lowercase SHA-256，再以`forward/slash/path<TAB>hash`和LF连接、无末尾LF后取总SHA-256。统计冻结的是2026-08-19当前工作树，基线提交为`25e09a23178000f2e783ce2143cf70a8b118d404`。本轮只新增review文档并更新索引/账本，不修改Rust、测试、Cargo、ABI、资源或产品配置。

### 2.2 检查方法

按`topic/payload construction -> retention classification -> sequence allocation -> target snapshot -> inbox admission -> report construction -> dirty merge -> consumer drain -> business callback -> retry/resync -> unregister -> host shutdown`顺序阅读，并反向搜索全部production注册、发布、drain、stats和report consumer。对每个built-in topic建立producer/consumer矩阵；对每个失败分支判断权威状态能否补读、原消息是否仍可重试、watermark是否推进、用户是否可见。

参考源码只用于验证owner和协议语义，不按类或功能数量打分。Unreal Messaging作为subscription/receiver/shutdown/tracing主参考；Godot CallQueue作为显式有界准入和flush guard参考；Bevy当前snapshot使用即时Observer，主要对照observer entity随despawn注销；Fyrox的`std::sync::mpsc`单consumer无界sender作为低基线反例；Unity Graphics本地镜像不是完整Unity Editor bus，只用于provider/node scoped diagnostic state和fixed-buffer `TryPush`语义。

### 2.3 动态证据边界

1. 本轮为review-only，没有运行Cargo。当前Editor library、Hub persist、WOC协议、npm计数和plugin locked metadata已有外部阻断，本轮没有重复运行已知失败lane。
2. P0由当前控制流和全仓production subscriber absence共同成立；`refresh_view_marks_view_dirty...`等测试应成为实施时的focused RED/GREEN证据，但未运行不能写成“测试已失败”或“修复已通过”。
3. message测试覆盖lane容量、logical bytes、fanout payload sharing、sequence exhaustion和request re-entry；没有覆盖共享dispatch与unregister交错、subscriber lease、shutdown、shadow pending queue或zero-target dirty产品链。
4. 1/5/100 subscriber性能门标记为ignored managed evidence；计划记录仍显示受管Cargo与真实数字pending，本文不把测试源码或阈值当作性能资格。
5. 当前`EditorMessageDelivery`的logical byte estimate不是heap profiler或RSS测量；2 MiB/16 MiB只是协议准入基础，不证明实际内存等于计数值。

### 2.4 已完成failure与旧结论修正

- `message-inbox-backpressure-and-fanout`已经落地shared payload、三类retention、lane indexes、checked identity和bytes预算；旧failure仍open是受管验证与更高层合同未关闭，不得恢复无界Vec或每subscriber深clone。
- `job-pump-budget-and-pending-scan`已经有64 events / 1 ms帧预算和progress coalescing；仍open的是lifecycle reservation、queue bytes/age以及bus失败后无损重试。本文不另建job queue workaround。
- Editor01的request重入修复真实存在：handler在bus lock外执行。本文新增的是target lifecycle race、deadline/panic和response receipt，不恢复持锁callback。
- Editor02早期“世界变化不进bus”已经source drift；World Sync现在真实publish并mark dirty。其bus report被忽略与generation推进已由Editor47拥有。
- Editor11已经证明log普通record零subscriber却返回`Delivered`。本文把它放入统一topic adoption矩阵，不重复计算该P0。

## 3. 必须保留的工程基础

1. 保留每publication单一immutable `Arc` payload，禁止退回fanout深拷贝。
2. 保留per-subscriber inbox mutex和metadata lock外enqueue，进一步增加route generation/fence而不是恢复全局大锁。
3. 保留Lossless全有或全无预检；新增required/optional subscriber group与明确per-target disposition。
4. 保留Latest语义key和bounded lane indexes；扩展scope、evicted key与resync disposition。
5. 保留2 MiB单delivery和16 MiB单inbox logical budget，增加按subscriber policy、实际内存验证和业务compact-resync。
6. 保留checked subscriber ID和delivery sequence exhaustion，不引入wrap、reuse或sentinel。
7. 保留surviving delivery的全局sequence顺序，不用lane-local drain破坏跨族顺序。
8. 保留request handler在bus lock外执行，增加lease revalidation、deadline、panic isolation和typed response。
9. 保留poison ownership recovery作为进程存活手段，但必须同时标记bus degraded并记录结构化诊断。
10. 保留Scene Inspection authoritative artifact和generation resync，不把完整hierarchy塞进每条消息。
11. 保留World Sync indexed watch projection和独立dirty set写入，修正commit/report而不是复制第二bus。
12. 保留plugin lifecycle callback失败时当前delivery位于后续消息之前的顺序要求，但把pending纳入同一预算与health state。
13. 保留typed Document/Transaction/Mode/Focus/Tool/Job/SceneInspection payload，不退回Text/Empty。
14. 保留`Custom`作为受治理扩展点，补namespace/schema/capability，不恢复任意debug text为正式内建协议。
15. 保留UI delta的path内latest-property coalescing和discrete barrier分段，增加page/budget/generation/ack。

## 4. 当前实现链与断路

```text
Producer
  -> SharedEditorMessageBus::publish(topic, EditorMessage)
       metadata lock: allocate sequence + snapshot Arc<Inbox> targets
       unlock metadata
       inbox locks: Lossless all-or-nothing / Latest / Bounded
       report { delivered, coalesced, dropped, backpressured, error }
       if delivered is not empty:
           metadata lock -> merge message.dirty into ViewDirtySet

View invalidation product path
  -> refresh_workbench / refresh_view
  -> publish valid topic "view.invalidated" with dirty mark
  -> production matched targets = 0
  -> empty success report; dirty is not merged
  -> drain_view_updates returns empty
  -> no reflection refresh / no scene-inspection publication

Plugin lifecycle path
  -> document/play producer ignores dispatch report
  -> plugin bridge drains bounded bus inbox completely
  -> appends into unbounded pending VecDeque
  -> callback error pushes current front back and returns
  -> next tick drains more before retry; bus sees healthy consumer

Job path
  -> JobEventQueue::pop removes lifecycle event
  -> bus.publish consumes EditorMessage
  -> report ignored
  -> backpressure/zero consumer cannot restore queue front
```

## 5. P0：产品正确性断路

### E-MSG-P0-01 · view invalidation绑定“至少一个subscriber成功投递”，生产refresh路径实际可成为空操作

证据链：

1. `EditorMessageBus::finish_dispatch()`和`SharedEditorMessageBus::finish_dispatch()`都只在`delivered`非空时调用`mark_message_dirty()`。
2. `EditorHostEventController::publish_view_invalidation()`构造带dirty mark的custom消息，并对常量`VIEW_INVALIDATED_TOPIC = "view.invalidated"`调用`EditorTopic::parse()`。
3. 该topic合法，所以失败分支中的直接`mark_view_dirty(view, mask)`不会执行。
4. 全仓production注册点只有plugin bridge的Document/Mode和controller的Scene Inspection；`view.invalidated`没有subscriber。
5. 因而publication返回无error、无drop、无backpressure、零delivered，dirty set保持不变。随后`refresh_view()`立即`drain_pending_view_refreshes()`，得到空dirty并跳过full snapshot/scene publication。
6. `refresh_workbench()`被asset access、component dispatch、status、extension registration和scene document等真实产品路径调用，所以影响不是孤立debug helper。

目标合同：dirty invalidation是view projection authority的输入，不能依赖消息listener是否存在。M0应硬切为显式`mark_view_dirty`或定义独立typed invalidation admission；若仍允许message携dirty，必须明确`NoSubscriber`时dirty是否提交，并禁止同一API同时表达“消息未投递”和“view invalidation成功”。focused RED必须覆盖零subscriber、subscriber backpressure、oversized消息、TREE_STRUCTURE、PRESENTATION_DATA、asset/status/extension真实caller；修复后再运行现有refresh集成测试。

## 6. P1：核心路由、订阅与请求合同

| ID | 当前差距 | 目标重构 |
|---|---|---|
| E-MSG-P1-01 | 零target返回无error的空report，没有`NoSubscriber/NoRoute`。 | report包含matched、required、accepted与`NoRoute`，领域adapter决定是否等于`NotConfigured`。 |
| E-MSG-P1-02 | `publish/broadcast`按值消费`EditorMessage`，backpressure、sequence exhaustion或零target后producer不能无clone取回原消息。 | Editor02提供non-consuming prepare/admit或`Err(UndeliveredMessage)`，支持Job lifecycle队首重试。 |
| E-MSG-P1-03 | Lossless fanout失败时只列出真正full的subscriber；其他因原子性被跳过的target既不在delivered也不在backpressured。 | 每target返回`Accepted/Full/SkippedAtomic/Closed/Stale`，aggregate不丢原因。 |
| E-MSG-P1-04 | 一个慢的可选subscriber会阻断所有lossless target，没有required/optional delivery group。 | subscription descriptor声明criticality和fanout group；required原子组与best-effort observer隔离。 |
| E-MSG-P1-05 | Shared bus在metadata锁内冻结`Arc<Inbox>`后解锁派发；并发unregister可先返回，旧plan仍向orphan inbox投递。 | route generation + in-flight lease/fence；unregister receipt定义最后可见sequence并等待或取消旧plan。 |
| E-MSG-P1-06 | `unregister_subscriber()`无条件移除inbox，未读lossless/bounded/latest delivery及stats静默销毁。 | `Drain/RejectIfPending/DiscardWithReceipt`显式策略，返回pending counts/bytes/oldest sequence与discard reason。 |
| E-MSG-P1-07 | subscriber只是公开`u64`，没有owner、generation、Drop注销或weak receiver；plugin bridge已出现裸注册。 | `EditorSubscriptionLease`持owner/topic/policy/generation，Drop只能做local revoke，显式shutdown返回terminal receipt。 |
| E-MSG-P1-08 | 注册允许空topic集合，且不记录debug name、thread/executor、required flag、handler或health owner。 | validated descriptor至少包含owner id、topics、delivery policy、budget、execution domain和diagnostic label。 |
| E-MSG-P1-09 | inbox limits是bus-wide固定值，Scene、plugin lifecycle、log custom和测试subscriber共享同一容量。 | policy registry按payload/topic/subscriber role解析count/bytes/age，默认值有来源与profile证据。 |
| E-MSG-P1-10 | `drain_deliveries()`整箱move并立即清空，最多可返回混合lane全部记录；处理失败没有ack/nack或remaining。 | count/bytes/deadline page、cursor、remaining、oldest age；ack后释放lossless reservation。 |
| E-MSG-P1-11 | delivery sequence对module外consumer不可见；Serialize/Deserialize省略sequence并重建为0，`PartialEq`也忽略sequence。 | 明确区分local envelope与wire DTO；需要重放的DTO携bus generation、sequence、schema version与correlation。 |
| E-MSG-P1-12 | stats只有message-count age和subscriber局部累计值；unregister后消失，生产无人读取。 | wall/monotonic age、peak、last progress、last pressure、aggregate health和product diagnostics snapshot。 |
| E-MSG-P1-13 | bus没有`Running/Closing/Closed`状态；任意clone可在host逻辑退出后继续注册或publish。 | 显式shutdown admission fence、subscriber revoke、pending disposition和closed error。 |
| E-MSG-P1-14 | request handler无deadline、cancel、panic隔离或execution-domain约束，可无限阻塞caller或unwind。 | request ticket/correlation/deadline/cancel，handler fault domain与typed terminal response。 |
| E-MSG-P1-15 | handler可在执行中注销target；业务副作用已发生，随后`complete_request()`返回UnknownSubscriber，caller只看到失败。 | request持有target lease/generation；定义commit point，response completion与target retirement互斥或返回明确`HandledAfterRetire`。 |
| E-MSG-P1-16 | request只验证target存在，不验证其订阅topic；同一topic descriptor对publish与request含义不同却未文档化为两套能力。 | targeted endpoint capability与pub/sub topic分域，handler registration决定可处理schema。 |
| E-MSG-P1-17 | topic集合注册后不能原子更新；只能注销并重建新ID，中间存在route gap。 | subscription revision支持prepare/replace/commit，旧revision在新revision可见后退休。 |
| E-MSG-P1-18 | bus metadata、subscriptions、dirty和UI delta仍由一个mutex owner；高fanout先分配target Vec，再逐inbox锁，最后再次取metadata锁。 | immutable route snapshot + interned topic + lock-free/read-mostly lookup；dirty/UI owner按generation批量提交。 |

## 7. P1：Retention、Payload、Dirty 与 UI Delta合同

| ID | 当前差距 | 目标重构 |
|---|---|---|
| E-MSG-P1-19 | Latest key不含topic；bus又允许任意payload/topic组合，同subscriber上不同topic的SceneMode/Selection等可互相coalesce。 | key包含validated protocol schema + scope，或硬性验证built-in payload只能走canonical topic。 |
| E-MSG-P1-20 | Selection key只有Scene/Asset，FocusObject、SceneInspection和SceneMode都是全局单key，没有document/world/PIE/project session。 | payload携authority scope/generation；coalescing key使用qualified owner identity。 |
| E-MSG-P1-21 | Latest在总bytes压力下可驱逐其他semantic key，但report只给subscriber ID，不说明被驱逐的key。 | disposition携evicted keys/sequences/bytes及每key resync action。 |
| E-MSG-P1-22 | `dropped`同时表示incoming拒绝、旧bounded eviction和其他Latest eviction；caller无法判断新状态是否真的保留。 | 分离`RejectedIncoming/ReplacedSameKey/EvictedOther/AcceptedAfterEviction`。 |
| E-MSG-P1-23 | SceneInspection coalesce只组合selection delta，hierarchy anchor与focused field delta直接由新消息替换。 | 只有连续generation才组合所有可组合delta；否则生成小型explicit resync marker。 |
| E-MSG-P1-24 | 超过2 MiB的SceneInspection被当Latest直接drop；没有compact resync marker，最后一次大变更可永久不触发后续恢复。 | oversize退化为带artifact generation的bounded resync notification，consumer按authority重取。 |
| E-MSG-P1-25 | logical byte estimate按`len/size_of_val`计算，不含Vec/String capacity、BTree节点、Arc/allocator metadata和subscriber metadata。 | 保留快速logical admission，同时用managed heap/RSS profile校准放大系数和hard process budget。 |
| E-MSG-P1-26 | Topic、schema_id、SceneModeId、property path、job/log文本等缺少统一长度/字符/深度准入；Custom namespace不校验owner。 | schema registry定义namespace、version、max fields/depth/strings/bytes与plugin capability。 |
| E-MSG-P1-27 | 所有Custom一律Bounded，无法表达durable edge、latest state或journal cursor；调用方可能误把重要插件事实放入可淘汰lane。 | Custom必须引用注册schema及retention contract；未知schema fail-close或进入明确telemetry lane。 |
| E-MSG-P1-28 | Tool全部Lossless，但没有production consumer；policy由payload enum硬编码，新增variant必须修改中央match。 | domain owner注册typed schema/policy，Tool authority提供query/resync；无consumer时不伪装可靠event。 |
| E-MSG-P1-29 | best-effort fanout只要一个subscriber接受就合并全局dirty，即使目标consumer被drop；P0又证明零subscriber时完全不dirty。 | dirty projection与subscriber delivery分离，以authority commit/generation驱动。 |
| E-MSG-P1-30 | `dirty`和`ui_deltas`共享bus但没有共同publication generation、frame id、ack或failed-apply retry。 | `EditorViewUpdateBatch`携generation/frame/cursor，apply成功后ack，失败保留last-good并请求resync。 |
| E-MSG-P1-31 | UI delta `entries`和`pending`无count/bytes/age预算；每个press/release/scroll/focus/geometry/commit barrier都可增长Vec。 | per-frame count/bytes budget、barrier-preserving page、overflow disposition和full reflection fallback。 |
| E-MSG-P1-32 | UI barrier sequence不验证单调、重复或gap；不同producer可插入不一致顺序。 | owner-issued monotonic sequence + batch generation；duplicate/gap触发diagnostic/resync。 |
| E-MSG-P1-33 | mutex poison统一`into_inner()`继续运行，没有记录哪个owner/invariant可能受损。 | poison恢复同时发布degraded health、invariant audit和一次性diagnostic；关键owner可fail-close。 |
| E-MSG-P1-34 | invalidation mask是可serde的裸`u16` newtype，未知bits可反序列化进入策略判断。 | versioned mask codec拒绝/保留未知bits并定义forward compatibility，不让未知bit静默变NONE。 |

## 8. P1：产品生产者、消费者、恢复与关闭

### 8.1 当前production adoption矩阵

| Topic / payload | 真实producer | 真实consumer | 当前结果 |
|---|---|---|---|
| `editor.document` | EditorManager project/document lifecycle | Plugin lifecycle bridge | 有consumer；producer忽略report，bridge失败队列无界 |
| `editor.mode` / PlayState | PlaySessionController | Plugin lifecycle bridge | 有consumer；producer忽略lossless backpressure |
| `editor.scene_inspection` | Host scene publication | Retained hierarchy consumer | 有consumer和artifact resync；oversize/drop与最后状态恢复未闭合 |
| `editor.transaction` | TransactionEventSink | 无production subscriber | 零target仍映射Delivered；history authority可查询但通知缺失 |
| `editor.job` | JobEventPump | 无production subscriber | lifecycle先pop后空投递；产品进度另读Job snapshot |
| `editor.tool` | ToolSchedulerService | 无production subscriber | lossless消息无人接收，scheduler query仍是authority |
| `editor.focus` | 未发现production typed producer | 无production subscriber | 类型/topic存在但产品链未采用 |
| `editor.log` | Log service sink | 无production subscriber | 普通record错误映射Delivered，Console另扫store |
| `editor.i18n` | I18n service sink | 无production subscriber | 普通locale change错误映射Delivered，resync才NotConfigured |
| `editor.world.fact` custom | WorldSyncPump | 无通用production subscriber | report忽略且published counter/watermark推进，归Editor47 |
| `view.invalidated` custom | Host refresh path | 无production subscriber | dirty不提交，形成E-MSG-P0-01 |

### 8.2 产品集成P1

| ID | 当前差距 | 目标重构 / owner |
|---|---|---|
| E-MSG-P1-35 | 11类production topic中只有Document、Mode、Scene Inspection有真实subscriber；“typed family存在”被误当产品采用。 | 每topic冻结authority/producer/consumer/resync/zero-route policy；无consumer删除空投递或显式Unavailable。 |
| E-MSG-P1-36 | Plugin bridge把bus inbox整箱追加到无界`pending VecDeque`，绕过4096/16 MiB budget。 | pending成为同一subscription的ack window，受count/bytes/age预算并把压力反馈producer。 |
| E-MSG-P1-37 | Plugin callback失败后队首永久阻塞；每tick继续drain新消息，没有backoff、quarantine、skip policy或operator action。 | per-plugin callback fault domain、bounded retry、quarantine和typed lifecycle health；不能让一个plugin阻塞全部。 |
| E-MSG-P1-38 | Plugin lifecycle bridge没有Drop/unregister；EditorManager Drop只释放project session guard。 | manager显式shutdown先停止producer、drain/reconcile callback，再revoke subscription并记录discard/terminal receipt。 |
| E-MSG-P1-39 | Document publish在authority mutation后忽略report；Plugin SceneChanged callback可能缺失且没有replay generation。 | document lifecycle publication携document/project revision；失败保留resync marker或让plugin按revision补读。 |
| E-MSG-P1-40 | Play mode transition在mode已经替换后忽略report；Entered/ExitedPlayMode plugin callback可缺失。 | Play transition receipt包含plugin notification disposition；plugin可按authoritative play generation reconcile。 |
| E-MSG-P1-41 | Transaction event delivery只warn，不重试、journal、计数或投影health；零subscriber还返回Delivered。 | Editor02拥有history cursor/resync；message只做可靠notification，失败进入structured diagnostic和lag state。 |
| E-MSG-P1-42 | Job pump先pop lifecycle再消费式publish并忽略report。 | 依赖Editor02 non-consuming admission；Editor14队首retry与lifecycle reservation直到ack释放。 |
| E-MSG-P1-43 | Job bus无产品consumer，retained UI另读progress snapshot；同一事件链支付序列化/路由成本却不驱动UI。 | 明确选择typed job journal consumer或删除无效bus hop；terminal/retry仍由Job authority保证。 |
| E-MSG-P1-44 | Tool lifecycle publish report全部丢弃且无consumer，所谓Lossless只代表inbox policy，不代表业务被观察。 | Tool panel/plugin consumer按scheduler generation查询；无consumer时report为NoRoute并有metric。 |
| E-MSG-P1-45 | Focus family既无production typed producer也无consumer，selection实际由其他authoritative路径传递。 | Editor03决定唯一selection/focus notification；删除死协议或接入qualified selection revision。 |
| E-MSG-P1-46 | Log普通record和I18n普通change在零subscriber时映射Delivered，resync分支却能返回NotConfigured。 | Editor11/33统一`NoConsumer/Accepted/Displayed/Resynced`状态机；不重复本文P0。 |
| E-MSG-P1-47 | WorldSync每fact忽略report，却增加`published_facts`并推进generation。 | Editor47拥有page commit/resync；bus disposition必须进入watermark commit。 |
| E-MSG-P1-48 | Scene publication先推进`SceneInspectionPublication`观察状态再publish；drop后没有立即退回resync pending。 | 保留last acknowledged generation；drop/oversize置`resync_required`，下一tick即发compact marker。 |
| E-MSG-P1-49 | `inbox_stats()`在production没有caller，drop/backpressure/age只存在于不可见API。 | Editor diagnostics authority周期采样bus/subscription health，Console/health pane可定位owner/topic。 |
| E-MSG-P1-50 | Controller构造对Scene Inspection注册失败直接`expect` panic；没有degraded host或typed startup error。 | construction返回typed error，或fail-close该surface并显示可恢复diagnostic。 |
| E-MSG-P1-51 | Controller Drop只注销Scene subscriber，Manager未注销Plugin subscriber；bus没有统一终止顺序。 | Host shutdown coordinator按producer stop -> consumer drain -> lease revoke -> bus close执行并返回receipt。 |
| E-MSG-P1-52 | 公开serde消息没有wire schema/version/sender principal/capability；任何拿到bus的crate consumer可构造built-in topic/payload组合。 | local typed bus与跨plugin/process wire protocol分离；wire envelope有schema、sender owner、capability、generation和budget。 |

## 9. P2：质量、性能与验证债务

| ID | 当前差距 | 目标 |
|---|---|---|
| E-MSG-P2-01 | built-in topic只有Document/Transaction/Log/I18n私有canonical constructor，Mode/Tool/Job/Scene仍重复parse和String allocation。 | 全部canonical interned TopicId；只在外部文本边界parse一次。 |
| E-MSG-P2-02 | zero-target仍分配sequence、delivery Arc和targets Vec。 | route lookup先确定matched count；NoRoute快路不物化payload wrapper，但保留必要producer metric。 |
| E-MSG-P2-03 | fanout report为多个`Vec<SubscriberId>`，高fanout每publish重复分配。 | small-vector/bitset或per-target compact disposition page，以profile决定布局。 |
| E-MSG-P2-04 | JSON logical byte estimate每publication遍历完整tree；大custom仍在确定route/policy前计量。 | 结构化payload在构造边界缓存validated byte cost；拒绝未知/oversize后不重复遍历。 |
| E-MSG-P2-05 | BTreeMap/BTreeSet适合确定性但高频topic/subscriber lookup和sequence queue节点开销未与workload profile绑定。 | interned key + appropriate hash/slab/ring结构；先以1/100/10K subscriber profile证明。 |
| E-MSG-P2-06 | `reflection_patches()`为每次refresh再次clone全部patch。 | batch提供借用迭代或ownership transfer，consumer按page apply。 |
| E-MSG-P2-07 | `EditorViewRefreshReport`使用serde default兼容旧deltas字段但没有schema version。 | versioned local snapshot DTO与migration test。 |
| E-MSG-P2-08 | Error/dispatch状态只有Display字符串和一个sequence exhausted code，没有稳定diagnostic code/stage/correlation。 | structured `MessageBusDiagnostic`与operation correlation。 |
| E-MSG-P2-09 | 文档声称protocol矩阵覆盖所有built-in payload，测试fixture只枚举Document/Transaction/Mode/Focus/SceneInspection，漏Tool/Job/Custom。 | exhaustive payload-family inventory由单一macro/schema生成测试矩阵。 |
| E-MSG-P2-10 | 没有zero-target dirty、unregister pending、in-flight unregister、shutdown publication测试。 | focused concurrency与lifecycle RED矩阵。 |
| E-MSG-P2-11 | 没有跨topic same-payload coalescing、qualified scope或evicted-key property test。 | table/property/fuzz测试覆盖topic/payload/policy组合。 |
| E-MSG-P2-12 | Plugin bridge测试覆盖callback failure顺序，但不证明连续失败下pending bytes有界、unsubscribe或quarantine。 | synthetic failure storm + leak census + terminal receipt。 |
| E-MSG-P2-13 | Scene测试覆盖10K sparse rename，但不覆盖>2 MiB最后一批、drop后无后续mutation和compact resync。 | large structural burst/last-change recovery E2E。 |
| E-MSG-P2-14 | Shared bus测试主要验证re-entry，没有确定性调度覆盖prepare/unregister/dispatch/complete交错。 | Loom/Shuttle或barrier-controlled race tests，不以sleep断言。 |
| E-MSG-P2-15 | managed fanout性能测试仍ignored且记录pending；当前没有真实Editor窗口pressure、slow consumer、shutdown和heap证据。 | Windows managed profile + product tick/frame/RSS/latency/oldest-age evidence。 |

## 10. 参考引擎对照与适用性

| 参考 | 可转移合同 | Zircon差距 | 不能照搬 |
|---|---|---|---|
| Unreal Messaging | `FMessageSubscription`保存weak receiver、scope range和enable state；router验证receiver有效性、按recipient thread dispatch并trace routed/dispatched/handled；bus有显式Shutdown delegate和router thread终止。 | Zircon需要subscription lease、receiver/owner identity、execution domain、route trace和shutdown fence。 | Unreal router并不自动给Zircon提供本报告要求的per-subscriber byte budget或lossless ack；不能因线程更重就复制线程模型。 |
| Bevy Observer | 当前Event默认即时trigger；observer是world entity，despawn或移除Observer会从global/entity/component caches注销，并有despawn后不得触发的测试。 | Zircon裸numeric subscriber与owner lifecycle脱钩；需要RAII/owner-bound retirement。 | 即时Observer会把callback成本放入trigger调用，不能替代Zircon异步有界inbox和跨线程产品消息。 |
| Godot CallQueue | page allocator有`max_pages`，满载返回`ERR_OUT_OF_MEMORY`并输出statistics；flush有re-entry guard并明确清理页。 | UI delta和plugin shadow pending缺少相同等级的显式容量、失败与flush状态。 | Godot单CallQueue不表达multi-subscriber lossless fanout、业务ack或per-topic resync，不可直接复制drop语义。 |
| Fyrox Editor Message | typed enum把Editor动作汇入单一main-loop sender，结构直观。 | 证明typed message只是最低起点；其`std::mpsc::channel`无界且`send`只由Log verify，不足以作为Zircon工程上限。 | 不把Fyrox单consumer队列替换成Zircon第二个中心大枚举或无界channel。 |
| Unity Graphics local mirror | ShaderGraph MessageManager按provider/node持有、清除和聚合诊断；FixedBufferStringQueue测试用`TryPush=false`表达容量失败。 | Custom schema需要provider/subject/generation和明确Try-admission；诊断不应是匿名append消息。 | 本地Graphics仓不含完整Unity Editor message bus、domain reload或Hub，不能据此声称Unity全局订阅/持久化方案。 |

共同规律是：订阅必须绑定owner lifecycle，路由结果必须能区分“无consumer、拒绝、已接收、已处理”，队列预算不能被下游shadow queue绕过，最新状态必须有qualified identity和resync来源，shutdown必须在owner消失前关闭新准入并给出终态。Zircon可以选择更低延迟的数据结构，但不能删除这些语义来换取表面性能。

## 11. 重构目标架构

```text
EditorMessageAuthority
  RouteCatalog
    TopicId + SchemaId + Version + Owner + Capability
    RetentionPolicy + ScopeKey + Count/Bytes/Age + DeliveryGroup

  SubscriptionLease
    SubscriberId + Generation + Owner + ExecutionDomain
    InboxPolicy + Health + LastAck + ClosingFence

  PreparedPublication<T>
    original message retained by producer until admission outcome
    matched routes + shared payload + logical bytes + correlation

  DispatchDisposition
    NoRoute
    Accepted { per_target, sequence }
    Rejected { reason, undelivered_message }
    PartialBestEffort { accepted, evicted_keys, rejected }

  DeliveryPage
    bus_generation + first/last_sequence + deliveries
    remaining + oldest_age + ack/nack/resync

  ViewProjectionAuthority
    dirty + ui delta batch + frame/generation
    independent from subscriber presence

  ShutdownReceipt
    admission_closed + producers_stopped + pages_acked/discarded
    subscriptions_revoked + diagnostics_flushed
```

不新建第二bus。`core::editor_message`继续拥有L1本地Editor消息合同；Document、Job、Play、Plugin、World Sync等owner通过注册policy和typed adapter接入。跨process/plugin ABI必须使用独立versioned wire DTO，不能把当前local `EditorMessageDelivery` serde形状直接认作稳定ABI。

## 12. 分阶段重构

### M0 · 修复dirty P0并冻结disposition语义

1. 将view invalidation从subscriber delivery解耦，保留零subscriber/背压/oversize正确刷新。
2. Dispatch report增加NoRoute、matched和明确incoming accepted/rejected。
3. 给现有refresh测试补真实production subscriber absence断言；运行focused RED/GREEN后再扩大范围。
4. 普通Log/I18n adapter不得再把零target映射Delivered。

### M1 · Subscription Lease、Non-consuming Admission 与 Ack

1. 引入owner-bound `SubscriptionLease`、route generation和in-flight fence。
2. unregister返回pending/discard/last sequence receipt，关闭注销后orphan dispatch窗口。
3. 发布先prepare/admit，失败返回原消息；Job lifecycle可以队首恢复。
4. drain升级为count/bytes/deadline page和ack/nack，保留全局顺序。
5. request绑定lease/correlation/deadline/panic boundary和typed terminal response。

### M2 · Retention Scope、Schema Registry 与 Compact Resync

1. built-in payload/topic强绑定；Latest key加入project/world/document/PIE scope。
2. report区分incoming rejection、same-key replace和other-key eviction并列出resync key。
3. Custom接入namespace/schema/version/capability/bytes/depth/retention registry。
4. Scene Inspection合并连续delta；gap/oversize立即转换compact resync marker。
5. UI delta建立frame/generation/page/budget与failed-apply resync。

### M3 · 产品Consumer与失败隔离

1. Plugin lifecycle shadow pending并回subscription ack window；callback按plugin quarantine，不阻塞全局队首。
2. Document/Play带authority generation并处理publication disposition。
3. Transaction/Job/Tool/Focus明确唯一consumer或删除无效bus hop。
4. World Sync按Editor47的page commit把bus disposition纳入watermark。
5. Scene retained consumer、Log、I18n和Console建立真实health/resync产品投影。

### M4 · Shutdown、Observability 与性能资格

1. Bus进入Closing后拒绝新注册/发布；按owner逆序drain、discard with receipt、revoke、Closed。
2. Product health显示topic/subscriber depth/bytes/age/last ack/drop/backpressure/degraded reason。
3. route trace绑定source/build/session/project/thread/correlation，但按预算采样。
4. 完成1/100/10K fanout、slow/stalled consumer、plugin failure storm、large scene、host shutdown和heap/RSS profile。

## 13. 验收门

1. 零`view.invalidated`subscriber时`refresh_view`仍返回目标dirty mask并执行正确snapshot/scene路径。
2. subscriber full、message oversize和sequence exhaustion不能阻止authoritative dirty提交，也不能伪报message delivered。
3. 每个dispatch outcome显式区分NoRoute、Accepted、Rejected、Partial与Closed。
4. Lossless失败把原消息无深clone返回producer，Job lifecycle可恢复到队首。
5. 一个optional stalled subscriber不能阻断required product consumer；required原子组仍保持全有或全无。
6. unregister与publish确定性交错后，注销receipt之后没有orphan delivery。
7. pending lossless unregister必须拒绝、drain或返回带counts/bytes/sequence的discard receipt。
8. SubscriptionLease owner Drop/显式shutdown后route catalog、inbox和health无泄漏。
9. request handler timeout/cancel/panic/target retire各有typed terminal result，副作用commit point明确。
10. request callback可重入publish/request且不死锁；新lease/fence不破坏已有能力。
11. delivery page返回first/last sequence、remaining、oldest wall age和ack；失败处理不自动释放lossless记录。
12. sequence在需要重放的DTO roundtrip后保持；local-only serde若删除则所有consumer硬切完成。
13. built-in payload发到错误topic被typed拒绝，不能跨topic coalesce。
14. Scene/Asset selection在不同document/world/PIE scope不互相coalesce。
15. Latest驱逐other key时consumer得到精确evicted key并按authority resync。
16. Custom未知namespace/schema、超深JSON、超长ID和超bytes均fail-close并有diagnostic。
17. Plugin声明Lossless/Latest/Bounded时必须经过capability和policy registry，不可私自字符串约定。
18. >2 MiB Scene Inspection大变更转compact resync，最后一次mutation也能恢复到当前artifact。
19. 连续Scene delta可组合；generation gap不尝试把不连续patch当成功。
20. dirty/UI delta batch携同一frame/generation；apply失败后last-good仍可用并触发resync。
21. 100K input delta/barrier storm受count/bytes/time预算约束且press/release/commit顺序不丢。
22. poison fault触发degraded health和invariant audit，不静默继续为健康。
23. Document open/save/close backpressure后plugin lifecycle按document revision补齐且只执行一次。
24. Play Enter/Exit callback按play generation补齐，不因bus压力缺失或重复。
25. Plugin callback连续失败时pending entries/bytes/age有界，故障plugin可quarantine，其他plugin继续推进。
26. Plugin bridge shutdown注销subscription并给出pending disposition，Manager Drop不留下裸subscriber。
27. Transaction notification失败保留history query/resync和structured diagnostic，零consumer不叫Delivered。
28. Job Started/terminal在stalled subscriber下不丢失，reservation只在ack后释放。
29. Job progress仍按JobId+owner generation latest，terminal顺序不被progress coalesce破坏。
30. Tool/Focus/Log/I18n/World topic各有唯一producer-consumer-resync矩阵；无consumer显式Unavailable/NoRoute。
31. World Sync只有在页面全部bus disposition可接受后推进watermark；拒绝触发同generation retry/resync。
32. Scene subscriber registration失败产生typed startup/degraded surface，不panic退出进程。
33. Bus Closing后所有clone的新publish/register返回Closed，不产生late product side effect。
34. shutdown顺序证明producer stop、consumer ack/discard、lease revoke、bus close和Host owner释放。
35. `inbox_stats`进入产品diagnostics，包含owner/topic/generation/depth/bytes/oldest age/last pressure。
36. report和diagnostic带稳定code/stage/correlation，UI文案不参与控制流。
37. protocol/payload矩阵覆盖Document/Transaction/Mode/Focus/Scene/Tool/Job/Custom及invalid组合。
38. barrier-controlled race覆盖publish/unregister、request/retire、shutdown/publish，无sleep-based假证据。
39. managed Windows profile报告1/5/100/10K subscriber的p50/p95/p99、allocations、RSS、queue age和frame impact。
40. scoped Rust tests、broader Editor integration、真实window tick、`git diff --check`、Markdown链接/LF/BOM/path/frontmatter/fingerprint全部通过后才可关闭。

## 14. 禁止的临时修法

- 不得通过给`view.invalidated`注册一个永不消费的假subscriber来修P0。
- 不得恢复“publish无条件mark dirty”，而不区分真正message side effect与view authority；应拆清合同。
- 不得只把4096、2 MiB或16 MiB调大来掩盖Job/Plugin/Scene恢复缺失。
- 不得在Job pump为失败消息额外deep clone作为长期重试协议。
- 不得让每个pane/plugin自建第二个无界pending队列或私有drop规则。
- 不得把所有消息改Lossless；unknown telemetry和latest state需要不同策略。
- 不得把所有subscriber改同步callback；慢callback不能进入producer关键路径。
- 不得用process退出证明subscriber、pending callback或bus shutdown无泄漏。
- 不得把当前local serde envelope直接宣布为稳定plugin/remote ABI。
- 不得以参考引擎类更多、线程更多或队列更大证明Zircon完成度/性能更高。

## 15. Owner与跨报告依赖

| 范围 | 唯一owner | 本报告依赖/不重复项 |
|---|---|---|
| bus、topic、subscription、inbox、retention、delivery page、request | Editor02 / `core::editor_message` | 本文拥有P0定位和目标合同；实现进入既有Editor02 owner，不建第二bus |
| view dirty、UI delta、retained refresh | Editor01 + Editor02 | E-MSG-P0-01优先；Editor01 retained UI性能P0不重复 |
| document/transaction/save/recovery | Editor02 | 依赖authority revision/history resync，不重复其save/transaction P0 |
| selection/focus/scene mode | Editor03 | 依赖qualified selection/world/document scope |
| plugin lifecycle bridge | Editor06 | 依赖plugin callback fault/quarantine和状态generation，不重复Plugin Manager P0 |
| Play transition | Editor07 | 依赖play generation/terminal state，不重复PIE/process P0 |
| Job lifecycle/progress | Editor09/14 | 依赖non-consuming bus admission和reservation，不重复Job P0 |
| Logging/Console | Editor11 | 零consumer/false Delivered已经由Editor11拥有 |
| I18n/locale | Editor12/33 | locale consumer/resync由其产品owner关闭 |
| World Sync/gateway | Editor47 | bus disposition进入page/watermark，不重复cross-session P0 |
| Local/wire ABI | Runtime Interface相关owner | 当前local envelope不是稳定ABI，若跨界需另立versioned DTO |

## 16. 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| 91文件/19,296行逐链审查 | review_complete | 2026-08-19 | fingerprint `3b7e94575b94bc84a45e0b99e3681b89eeafaddd371d2edac711e47c16c4a7da` |
| production topic adoption矩阵 | review_complete | 2026-08-19 | 11类topic/payload逐个反向搜索producer、subscriber、drain和report consumer |
| P0/P1/P2与重构门 | review_complete | 2026-08-19 | 1 P0 / 52 P1 / 15 P2 / 40 gates |
| Production实现与动态验证 | pending | - | 本轮未修改Rust/tests/Cargo，未运行Cargo；按M0-M4实施 |
