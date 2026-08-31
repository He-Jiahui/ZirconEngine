---
title: Runtime Scene Event Mirror、Registration、Subscription、Cursor、Backlog、Overflow、Reclaim、ABI、Consumer 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime54
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/scene/event_mirror
  - zircon_runtime/src/scene/world/event_mirror.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/plugin/extension_registry/register/event_registration.rs
  - zircon_runtime_interface/src/runtime_api/session/plugin_event_mirror.rs
  - zircon_editor/src/core/gateway/session/plugin_events.rs
  - zircon_editor/src/core/runtime_event_consumer
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_plugins/ai/editor/src/runtime_mirror.rs
  - zircon_plugins/navigation/runtime/src/plugin.rs
  - zircon_plugins/navigation/editor/src/runtime_mirror.rs
tests:
  - zircon_runtime/src/scene/tests/ecs_event_mirror.rs
  - zircon_runtime/src/dynamic_api/tests/linked_plugins.rs
  - zircon_runtime/src/dynamic_api/session/registry/tests.rs
  - zircon_editor/src/tests/runtime_event_consumer.rs
  - zircon_editor/src/tests/runtime_event_consumer_bounded_pump.rs
  - zircon_editor/src/tests/runtime_event_consumer_bounded_pump/real_runtime_abi.rs
  - zircon_editor/src/tests/gateway/session/plugin_operations.rs
  - zircon_plugins/navigation/runtime/src/tests/runtime_mirror.rs
  - zircon_plugins/navigation/editor/src/tests.rs
  - zircon_plugins/ai/editor/src/tests.rs
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_editor/47-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md
  - docs/plans/performance/01/2026-08-14-runtime-ecs-observer-event-messaging-current-review.md
  - docs/plans/performance/01/2026-08-15-editor-runtime-event-consumer-semantic-routing-current-architecture-review.md
  - docs/plans/zircon_runtime/runtime/10/failure-2026-07-19-plugin-event-bounded-delivery.md
  - docs/plans/zircon_plugins/01/failure-2026-07-22-plugin-event-drain-frame-budget.md
  - docs/plans/zircon_plugins/12/failure-2026-07-22-runtime-event-mirror-drop-lifecycle.md
  - docs/plans/zircon_editor/editor/02/failure-2026-07-17-runtime-event-consumer-unbounded-pump-lock.md
reference_engines:
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
  - dev/bevy/crates/bevy_ecs/src/message/message_cursor.rs
  - dev/bevy/crates/bevy_ecs/src/message/iterators.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageSubscription.h
  - dev/godot/core/object/object.cpp
  - dev/Fyrox/fyrox-resource/src/event.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Debugging/DebugWindow.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 54 · Runtime Scene Event Mirror、Registration、Subscription、Cursor、Backlog、Overflow、Reclaim、ABI、Consumer 与 Product Integration 工程化差距

## 1. 结论

Scene Event Mirror并非旧报告描述的无界全量JSON drain，也不再是token直接Drop后永久遗留World reader的原始实现。当前底座已经具有固定`64 events / 128 KiB payload`页、`256 KiB` ABI wire ceiling、每订阅`16K events / 64 MiB`队列、序列化深度/时间/字节限制、Session内`prepare/commit/rollback`页事务、代际subscription slot、去重回收意图、reader-count rollback、WorldDriver回收、Session teardown重试条件，以及Editor侧单页pending、全局/per-consumer/time预算、round-robin和锁外typed callback。这些进展必须保留。

但当前实现仍不是工程级事件流。第一，AI Editor声明了三个Runtime消费者，而AI Runtime只通过SDK `event()`注册普通ECS event；SDK没有mirrored event builder，真实Runtime订阅会得到unknown event，现有AI测试只验证manifest文字。第二，Runtime在foreign allocation注册成功时即提交页，Editor decode、协议校验和typed callback都发生在提交之后；overflow/oversized事件不获得stream sequence，Editor又只拒绝`sequence <= last`而不检查连续性，callback error/panic还明确消费当前delivery，因此丢失可以被后续连续编号掩盖，系统没有ack、dropped range、snapshot generation或`ResyncRequired`。第三，每个subscriber创建一个ECS observer、在producer线程重新JSON序列化并持有一份独立队列；订阅数没有总预算，单一事件的成本为`O(subscribers * payload)`，内存上界是`subscribers * 64 MiB`而不是stream级硬上限，10ms序列化deadline也会逐订阅累加。

目标不是继续给per-subscription queue补字段，而是建立runtime-owned `SceneEventBroker + SharedEncodedStream + AcknowledgedConsumerCursor`：event contract显式声明provider generation、scope、schema digest、delivery class和resync provider；同一事件只编码一次并进入有界共享segment log；订阅者只持cursor、credit、ack和lag state；overflow产生精确丢失区间和typed resync disposition；ABI提交点延后到Editor语义消费成功，或以幂等ack/retry明确转移authority。AI与Navigation必须通过同一product registration contract取得真实Runtime/Editor闭环。

本轮登记 **3项P0、60项P1、16项P2和40项验收门禁**。只做静态review与文档总账；没有修改production、tests、Cargo或reference source，没有运行Cargo、真实Editor/Runtime、1K/10K managed ABI、multi-subscriber storm、plugin reload、fault injection、RSS或benchmark。四份相关failure仍为`open`，本报告不把source implementation或ignored benchmark写成accepted milestone。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / tests | fingerprint / 说明 |
|---|---:|---|
| Runtime core/World/Session owner | 10 / 4,110 / 150,274 / 19 | SHA-256 `4a31cd849a6db4d9be983fc8cd8eb14d47d52aac8efd92bda4e909b58b36cf44` |
| Interface、Gateway 与 Editor consumer | 10 / 1,833 / 64,023 / 2 | SHA-256 `d8bcad6c9787a58826fc1c86ce0f001dd636c543b86872543c2278a487fbd9e7` |
| SDK、AI 与 Navigation 产品链 | 5 / 1,710 / 63,689 / 2 | SHA-256 `281e178f6baa503775fdc5b2260b742f300b80e8d69d6e58388d1a892d0ceb3f` |
| focused direct tests | 11 / 5,330 / 189,428 / 100 | SHA-256 `174d77a4072cb23bf77ac685eb0431c208dc3feb6548c69504be3cfaf72235c9`；2 ignored |
| reference corpus | 8 / 4,535 / 142,845 / 6 | SHA-256 `c772f5e7c4c98667976aeb59f3f0196dd26b494aeaefa9b27fcecaacfc984e58` |

fingerprint算法与Runtime53一致：相对路径转`/`、排序去重，以`path|lowercase per-file SHA-256`编码，LF连接且末尾不追加LF，再计算UTF-8 SHA-256。它只冻结本轮读取集合，不是event stream、provider build、schema或ABI generation identity。

核心mirror、ABI和Editor consumer文件没有本轮写入。产品集合中的AI runtime registration已有其他会话的world-scoped snapshot查询修改，Navigation runtime plugin已有按reader count关闭debug capture的修改；两项均纳入当前指纹且没有被本报告覆盖。实施前必须重新读取这两份文件、Session state及direct tests，所以`source_recheck_required`为true。基线HEAD为`bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator epoch为335。

### 2.2 当前真实产品调用链

```text
RuntimeExtensionRegistry::register_mirrored_event<E>()
  -> World::register_runtime_event_mirror()
     -> subscribe: one typed ECS observer + one JSON queue per subscriber
        -> World::send_event(E)
           -> observers serialize E independently under queue mutexes
           -> RuntimeDynamicSession::prepare_plugin_event_output()
              -> scene queue pop -> Session pending_page -> JSON ABI page
              -> FFI foreign allocation success => commit sequence/page
                 -> Editor SessionGateway decode/free
                    -> EditorRuntimeEventConsumerHost pending page
                       -> validate monotonic metadata -> typed callback
```

`send_event`无论observer是否接受都会把event写入普通ECS store；返回值只是所有observer bool的按位聚合。多订阅者中一个queue overflow、另一个成功时，producer只得到`false`，已成功的订阅仍收到事件，失败订阅的event既无stream sequence也无dropped range。这不是可恢复的broadcast contract。

AI产品链在第一步之前断开：`RuntimePluginRegistrationBuilder::event()`只调用`register_event`，而三个AI Editor consumer期待`ai.events.behavior_debug_snapshot`与`ai.events.bt_node_result`可订阅；AI runtime没有调用`register_mirrored_event`。Navigation overlay是当前唯一直接的first-party mirrored product event，并以reader-count callback控制debug capture。

### 2.3 应保留的真实底座

1. payload在写入queue时即受128 KiB、nesting和10ms序列化边界约束，ABI页另有256 KiB ceiling。
2. queue拒绝超限新event但保留此前已接受payload；drain固定最多64条并暴露remaining count与oldest age。
3. Session pending page把scene dequeue与foreign allocation提交分开，allocation失败可rollback并重试同页。
4. ABI encoder直接拼接validated raw JSON payload，避免先解成`Value`再重复编码；idle返回empty owned buffer。
5. subscription使用slot+generation，Drop只提交有界去重reclaim intent；显式unsubscribe、Drop reclaim和Session shutdown共享World record owner。
6. reader-count callback失败会回滚connect/disconnect和count，Navigation可以在最后订阅者退出后关闭昂贵debug producer。
7. Editor pending非空时不会再次跨ABI drain；typed callback与gateway调用在active map锁外，且有round-robin、全局/per-consumer/time预算。
8. Editor report已经记录drained bytes、runtime/decode time、pending上界和最后一次runtime backlog observation；它是后续工程化telemetry的底座，不是最终资格证据。

## 3. 参考实现裁决

| 参考 | 直接源码事实 | 对Zircon的约束 |
|---|---|---|
| Bevy `Messages` / `MessageCursor` | 同一typed message只存于双buffer一次；每个reader保存`last_message_count`，iterator只在实际`next/nth/count`时推进；`missed_messages()`显式报告因retention错过的数量 | Zircon跨DLL retention可以比Bevy两帧更强，但必须采用shared stream + independent cursor并显式报告gap，不能每订阅复制payload后用局部sequence掩盖丢失 |
| Unreal `FMessageRouter` / `FMessageSubscription` | router命令串行化订阅变更；单个shared message context路由给多个recipient；subscription有scope、enabled与weak receiver；dispatch可按receiver thread投递并由tracer记录sent/routed/dispatched/handled | descriptor必须有scope、dispatch/QoS和provider identity；payload/context应共享；route与handle是两个可观测阶段，不能把ABI allocation当作业务handled |
| Godot `Object::emit_signalp` | 发射前在锁内复制callable/flags快照，锁外执行；one-shot先断开以防递归；支持deferred、persistent、reference-counted连接并在target失效时安全跳过 | callback snapshot、disconnect、reentrancy和lifetime必须有明确状态机；Godot signal不是持久跨DLL log，不能拿它为无ack transport背书 |
| Fyrox `ResourceEventBroadcaster` | broadcaster用generational pool管理sender并在send失败时回收；但broadcast仍逐subscriber clone事件 | generational removal可借鉴，逐subscriber clone只适合小型进程内通知，是本报告明确拒绝作为高吞吐产品stream目标的反例 |
| Unity Graphics `DebugWindow` | `OnEnable`注册callback，`OnDestroy`显式解除；注释处理window替换时新实例先enable、旧实例后destroy的交错 | plugin/editor reload必须有幂等register/unregister和generation fence；Unity callback lifecycle不是reliable stream，不能替代cursor/ack/resync |

共同约束是：producer payload/context只形成一次权威表示，subscription有scope和lifecycle，delivery与handled分阶段观测，过期/丢失可检测，外部callback不在注册容器锁内执行。Zircon要超过这些引擎，应在此基础上增加typed schema generation、global byte budget、ack/resync和跨DLL资格，而不是以复制更少字段或静默丢事件换取表面速度。

## 4. Owner边界与不得重复登记

| Owner | 继续拥有 | Runtime54只登记 |
|---|---|---|
| Runtime02 / Runtime05 | 通用ECS events/messages/observer、World schedule与生命周期 | mirrored stream从typed event到跨DLL consumer的纵向contract，不复制所有ECS event问题 |
| Runtime43 / Interface01/07 | Dynamic Session action owner、foreign allocation、API table、ABI认证 | event page的ack/resync/budget/schema字段和commit语义 |
| Editor47 / Editor02 failure | Gateway decode、consumer host、公平泵、Editor session reconnect | Runtime delivery如何在typed callback成功后提交及失败恢复 |
| Plugins01 / Plugins12 failure | plugin contract、bounded delivery、drop/reload lifecycle | SDK mirrored registration、provider generation和产品接线 |
| Plugins14 / Plugins15 | Navigation与AI业务事件、debug snapshot、consumer state | 真实产品订阅闭环，不复制Nav/AI算法差距 |
| Runtime54 | registration到shared publication、cursor、overflow、reclaim、ABI、Editor apply的完整纵切面 | 本报告3项P0和60/16项纵向差距 |

## 5. P0：当前产品正确性与可用性阻断

| ID | 差距 | 当前证据与硬切目标 |
|---|---|---|
| SEMR-P0-001 | AI Editor发布三个不可订阅的Runtime consumer | AI runtime以SDK `event()`注册`BtNodeResultEvent`与`AiBehaviorDebugSnapshot`，该路径只创建普通ECS event；SDK无mirrored builder，AI Editor却发布两个event id上的三个consumer。建立单一`PluginEventExposureContract`并由Runtime/Editor manifest共同消费；AI真实Runtime ABI test必须证明capability enable后subscribe、deliver、disable、unload全链成功。 |
| SEMR-P0-002 | Runtime commit早于Editor语义成功，gap又不可检测/恢复 | FFI在foreign allocation成功后commit；Gateway之后才decode，Host之后才校验和callback。overflow/oversized event无sequence；Host只检查`<= last`，callback error/panic消费当前delivery且只恢复尾部。硬切为acknowledged cursor：wire携带stream generation、contiguous range、dropped range/resync token；decode+typed apply成功后ack，失败可retry/quarantine或请求authoritative snapshot。 |
| SEMR-P0-003 | fanout成本与内存上界随订阅数无界放大 | 每订阅创建observer和`VecDeque<Vec<u8>>`，每次send在producer线程逐订阅JSON encode并锁queue；单订阅64 MiB而subscription count无session/world/global上限，10ms deadline逐订阅累加。改为event type级一次编码、共享immutable segment log、cursor-only subscriber、global/stream/consumer credit和admission；规模门证明1/64/1K subscribers下publish cost与RSS受控。 |

## 6. P1：Contract、Registration 与 Product Exposure

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| SEMR-P1-001 | descriptor只有`event_id + payload_schema` | 增加stable contract id、provider/plugin/build generation、schema id/version/digest、scope与delivery class。 |
| SEMR-P1-002 | payload schema是自由字符串 | 绑定可查询schema artifact、compatibility rule和migration/resync能力，禁止仅字符串相等冒充兼容。 |
| SEMR-P1-003 | event id namespace未绑定owner | catalog validation与World registration必须证明namespace owner，避免插件碰撞或卸载后被另一provider接管。 |
| SEMR-P1-004 | contract没有lossless/latest/coalesced/bounded策略 | 每个event声明delivery semantics、retention、overflow和snapshot recovery，不再统一套64 MiB FIFO。 |
| SEMR-P1-005 | registration没有project/world/session scope | subscription request必须限定RuntimeSession、World/Level和可选entity/domain scope，拒绝跨World误路由。 |
| SEMR-P1-006 | Plugin SDK没有mirrored event builder | 提供唯一typed exposure builder或统一普通/mirrored registration contract，禁止插件绕过owner/schema/policy。 |
| SEMR-P1-007 | package event catalog不表达是否跨DLL可订阅 | catalog增加exposure、target mode、producer capability和availability，Editor consumer admission先校验真实provider。 |
| SEMR-P1-008 | TypeId、catalog id和World event id分阶段检查 | 在plugin activation preflight原子验证type/id/schema/provider，禁止部分catalog已发布而World apply失败。 |
| SEMR-P1-009 | World registration没有unregister/revoke | plugin unload/reload必须撤销provider generation、拒绝新订阅并终结或迁移旧cursor。 |
| SEMR-P1-010 | registry `Clone`重置live state且`PartialEq`恒true | 删除伪相等，定义snapshot/staging语义；registration contract差异必须可见且不可被World比较吞掉。 |

## 7. P1：Publication、Fanout、Queue 与 Backpressure

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| SEMR-P1-011 | 一个subscriber对应一个ECS observer | 每event contract只安装一个broker adapter，subscriber只注册cursor/filter。 |
| SEMR-P1-012 | 同一event按subscriber重复JSON序列化 | producer边界一次编码为shared immutable envelope，失败只产生一个typed publication outcome。 |
| SEMR-P1-013 | payload bytes按subscriber完整复制 | segment/page由Arc或等价lease共享，cursor只保存offset/range；跨DLL复制另计且受wire budget。 |
| SEMR-P1-014 | 只有per-subscription 64 MiB预算 | 增加World、Session、provider、stream和consumer多层entry/byte budget及reserve/commit。 |
| SEMR-P1-015 | producer线程持queue mutex并执行serializer | publication fast path不得跨subscriber锁或执行S次serializer；重编码/压缩进入有界stage。 |
| SEMR-P1-016 | 10ms payload deadline会按subscriber累加 | deadline属于一次publication；记录serialization elapsed并在全局frame budget内admit。 |
| SEMR-P1-017 | queue只保存第一个failure | 累计typed failure counters和精确sequence/range，不能让后续overflow、depth/time失败消失。 |
| SEMR-P1-018 | `send_event` bool混合所有observer并允许部分接受 | 返回结构化publish receipt；lossless contract要么全体authority接受，要么明确每cursor disposition。 |
| SEMR-P1-019 | 所有event强制FIFO且不能coalesce | snapshot/debug frame采用latest/keyed coalesce，edge/lossless stream保持顺序并有独立retention策略。 |
| SEMR-P1-020 | page/queue常量不可由validated profile协商 | policy由contract和runtime profile生成effective budget，wire返回实际值与policy generation。 |

## 8. P1：Sequence、Commit、Retry 与 Resync

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| SEMR-P1-021 | sequence在Dynamic Session subscription局部生成 | producer stream在publish admission时分配全局单调sequence，并与stream/provider generation组成identity。 |
| SEMR-P1-022 | rejected event不消耗sequence | 所有publication attempt要么获得可见sequence/disposition，要么在进入stream前原子拒绝且producer知情。 |
| SEMR-P1-023 | Editor只检查单调，不检查连续 | 验证`first == acked + 1`、页内连续、range header一致；发现hole立即停止apply并resync。 |
| SEMR-P1-024 | Scene queue在Session prepare前已经pop | shared log保留authority直到所有required cursor ack或retention policy明确退休。 |
| SEMR-P1-025 | Session commit只证明foreign allocation成功 | commit拆为wire lease与consumer ack；allocation/decode/apply分别有状态和metrics。 |
| SEMR-P1-026 | ABI没有ack/nack/checkpoint | 新API table提供ack range、nack reason、checkpoint generation与idempotent retry；旧V7硬切而非双truth。 |
| SEMR-P1-027 | typed callback返回Err后当前delivery永久丢失 | error保留未ack delivery，按policy retry、dead-letter、disable consumer或resync。 |
| SEMR-P1-028 | callback panic只恢复尾部 | panic boundary保留当前envelope identity并把consumer置faulted，禁止继续假装sequence连续。 |
| SEMR-P1-029 | 没有poison event隔离与终态 | dead-letter记录payload digest、contract/provider、failure、attempt、operator disposition与retention。 |
| SEMR-P1-030 | 没有authoritative snapshot/resync provider | latest/state mirror必须注册snapshot generation和rebuild API；edge-only stream明确不可恢复并fail closed。 |

## 9. P1：Subscription、Reclaim、Unload 与 Session Lifecycle

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| SEMR-P1-031 | subscription不绑定provider unload generation | handle包含stream/provider generation；provider revoke使旧handle得到typed terminal状态。 |
| SEMR-P1-032 | ABI handle只有进程局部u64 | 使用session-qualified opaque generation handle，拒绝reload后的stale handle和跨session误用。 |
| SEMR-P1-033 | Session next handle溢出只报错且无rotation | 定义handle-space exhaustion、session renewal与diagnostic，不等待理论极限触发不可恢复状态。 |
| SEMR-P1-034 | internal slot generation溢出会遗弃free slot | generation exhaustion转typed retirement并计数，禁止静默减少可用slot。 |
| SEMR-P1-035 | token Drop要等下一WorldDriver tick回收 | Drop intent触发bounded wake；无后续tick的paused World也必须及时关闭按需producer。 |
| SEMR-P1-036 | `World::drop`只尝试一次并忽略callback failure | teardown owner必须quiesce、记录forced disposition并证明外部reader-count side effect收敛。 |
| SEMR-P1-037 | reader-count callback可任意mutate World且无幂等contract | 收敛为broker activation edge或service lease，声明reentrancy、failure、rollback和shutdown语义。 |
| SEMR-P1-038 | reclaim report只有本轮计数 | 每handle保留state/retry/last error/age，聚合到stream/provider/session lifecycle report。 |
| SEMR-P1-039 | live record假设registration永远存在并`expect` | unload/reload采用显式revoking/tombstone state，不以panic维持不变量。 |
| SEMR-P1-040 | 没有跨World/session/plugin lifecycle journal | 记录subscribe、activate、drop intent、disconnect、ack、revoke、resync和terminal disposition。 |

## 10. P1：ABI、Gateway 与 Editor Consumer

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| SEMR-P1-041 | 每delivery重复event id/schema/subscription/session字段 | page header保存稳定descriptor与range，record只保存sequence delta、payload slice和必要metadata。 |
| SEMR-P1-042 | wire只有owned JSON | 以schema协商binary/raw JSON codec、compression和zero-copy lease；JSON保留为debug/compat而非唯一热路径。 |
| SEMR-P1-043 | drain请求不能携带count/bytes/deadline/credit | V8请求显式提供预算，Runtime按有效policy返回，不让固定页单次调用突破Editor frame budget。 |
| SEMR-P1-044 | wire `playSessionId`实际写入Runtime session handle | 分开RuntimeSessionId、PlayInstanceId、WorldId与provider generation，字段名与语义一致。 |
| SEMR-P1-045 | Gateway decode failure发生在Runtime commit后 | decode验证完成前不得ack；协议失败保留wire lease或触发typed resync，不丢authority。 |
| SEMR-P1-046 | Gateway只预检ABI/count/subscription | 在分配大对象/进入Host前验证descriptor generation、range、sequence continuity、bytes和resync state。 |
| SEMR-P1-047 | 同一event的多个Editor consumer建立多个Runtime订阅 | Host按compatible filter共享一个runtime cursor，再在Editor内fanout immutable envelope或显式独立QoS。 |
| SEMR-P1-048 | begin/end callback无Result与panic boundary | lifecycle callback返回typed outcome，panic隔离，失败时订阅和state原子rollback或保持retry owner。 |
| SEMR-P1-049 | capability reconcile先删除再添加，添加失败可留下部分新状态 | 以prepared subscription set原子切换；旧set在新set全部ready前保持last-known-good。 |
| SEMR-P1-050 | Editor consumer generation饱和后重复`u64::MAX` | checked allocation、exhaustion terminal和session renewal，禁止generation collision。 |

## 11. P1：Diagnostics、测试、性能与资格

| ID | 差距 | 需要重构的内容 |
|---|---|---|
| SEMR-P1-051 | Runtime只暴露queue count/bytes/oldest age的错误快照 | 持久指标覆盖publish/encode/enqueue/ack/drop/resync、depth/bytes/lag/oldest、lock wait和CPU。 |
| SEMR-P1-052 | ABI不报告stream/consumer retained bytes与credit | page header返回precise retained/shared/consumer bytes、credit、high-water与observation generation。 |
| SEMR-P1-053 | Editor p95只对单pump样本排序 | 使用跨帧bounded histogram/window并标注sample count、generation、workload和reset reason。 |
| SEMR-P1-054 | event无frame/time/world/provider/trace context | envelope加入必要因果metadata，允许定位producer frame、World、operation和handled latency。 |
| SEMR-P1-055 | 没有多subscriber部分overflow行为测试 | 构造快慢subscriber并证明receipt、gap、ack和resync，不只断言单queue返回一次Err。 |
| SEMR-P1-056 | AI没有真实Runtime/Editor ABI产品测试 | 使用真实plugin registration、dynamic session、gateway和typed mirror，禁止manifest-only fixture冒充闭环。 |
| SEMR-P1-057 | 两项真实ABI storm仍为ignored | 纳入受管Windows required lane，记录1K/10K count/bytes/encode/decode/apply/p95/RSS。 |
| SEMR-P1-058 | 没有1/64/1K subscriber fanout与64 MiB压力矩阵 | 证明publish CPU、allocation、shared bytes、lag和disconnect随subscriber规模符合预算。 |
| SEMR-P1-059 | 没有decode/callback failure的no-loss或resync证据 | fault injection覆盖invalid JSON/schema、consumer Err/panic、allocation/free失败、ack丢失和重复。 |
| SEMR-P1-060 | 没有reload/crash/reconnect/multi-World soak | 覆盖provider unload/reload、Session destroy retry、paused World、Editor reconnect和跨World隔离。 |

## 12. P2：可维护性、API与证据质量

| ID | 差距 | 后续处理 |
|---|---|---|
| SEMR-P2-001 | queue注释声称payload在producer boundary只序列化一次 | 改为与事实一致；shared broker落地后再声明once-per-publication。 |
| SEMR-P2-002 | page/queue/writer常量分散于Runtime与Interface | 收敛为versioned policy manifest和编译期一致性检查。 |
| SEMR-P2-003 | descriptor String在registration/subscription/page重复持有 | intern stable descriptor并以generation handle引用。 |
| SEMR-P2-004 | hot path以String BTreeMap查event id | admission后解析为typed StreamId，热路径不做字符串树查找。 |
| SEMR-P2-005 | shutdown枚举全部live handle到新Vec | 使用bounded intrusive/work queue或分块迭代，避免teardown峰值分配。 |
| SEMR-P2-006 | retire时`pending.retain`扫描回收队列 | 用handle-indexed queue/tombstone获得摊销常数删除。 |
| SEMR-P2-007 | scene-facing`drain()`仍把raw JSON逐条解成Value | 明确只用于debug/legacy并迁移调用者到typed/raw page contract。 |
| SEMR-P2-008 | idle drain仍创建一次output in-flight状态 | 提供无状态idle observation或cursor status，减少稳态写锁/状态翻转。 |
| SEMR-P2-009 | lifecycle大量`assert/expect`承担production invariant | 转为typed corruption/revoke/forced teardown并记录诊断。 |
| SEMR-P2-010 | queue payload/failure/page缺少统一Debug/telemetry view | 建立sanitized snapshot，禁止日志打印完整业务payload。 |
| SEMR-P2-011 | Runtime error跨FFI退化为字符串 | 保留stable error code、stage、stream/subscription generation和retryability。 |
| SEMR-P2-012 | 文档没有总内存公式和subscriber成本模型 | 公开effective budget、shared-vs-private bytes与scale envelope。 |
| SEMR-P2-013 | 多个test依赖`include_str!`源码形状 | 用行为、fault和ABI compatibility测试替代脆弱文本锚点。 |
| SEMR-P2-014 | 721行Session encoder与785行Editor host继续聚合责任 | 按broker protocol、wire lease、consumer transaction、diagnostics拆分模块。 |
| SEMR-P2-015 | public Scene mirror API暴露queue机制而非stream contract | 收敛为descriptor、subscription policy、cursor、page lease、ack与outcome。 |
| SEMR-P2-016 | benchmark JSON缺BuildSet/reference/workload identity | 所有性能artifact绑定HEAD、source fingerprint、hardware、profile、sample和threshold。 |

## 13. 目标架构

```text
PluginEventExposureContract
  { StreamContractId, ProviderGeneration, SchemaDigest, Scope, DeliveryClass, Budgets }
        |
        v
SceneEventBroker (one typed adapter per event contract)
  publish -> encode once -> SharedEncodedStream
                         { StreamGeneration, SequenceRange, Segments, GlobalBudget }
                                      |
                   +------------------+------------------+
                   v                                     v
          ConsumerCursor A                         ConsumerCursor B
          {ack, credit, lag}                       {ack, credit, lag}
                   |                                     |
                   +---------- ABI page lease -----------+
                                      |
                         Editor Consumer Transaction
                    validate -> decode -> apply -> ACK
                                      |
                       retry / quarantine / resync snapshot
```

`DeliveryClass`至少区分lossless edge、bounded telemetry、latest snapshot和keyed coalesced state。只有具备authoritative snapshot provider的class才能在gap后自动resync；不可恢复lossless edge遇到retention breach必须停止consumer并给出typed terminal failure。共享segment退休由required cursor ack、retention和provider revoke共同决定，不能由某次ABI allocation单独决定。

## 14. 重构里程碑

### M0 · Characterization 与 Product Gate

- 添加AI真实Runtime订阅RED、multi-subscriber overflow/gap RED、Gateway decode failure RED和callback Err/panic RED。
- 冻结当前Navigation正向行为、drop reclaim、fixed page和Session allocation rollback，防止重构倒退。
- 对现有四份open failure建立同一source snapshot与验收路由，不重复创建实现owner。

### M1 · Event Exposure Contract 与产品接线

- 建立provider/schema/scope/delivery-class contract，统一Runtime catalog、SDK builder和Editor consumer admission。
- 将AI与Navigation迁移到单一exposure API；补齐provider unload/reload generation。
- 删除普通event与mirrored event可产生manifest/World双truth的路径。

### M2 · Shared Stream、Sequence 与 Global Budget

- 每event contract只安装一个typed adapter，一次编码进入共享segment log。
- 建立stream sequence、global/stream/consumer budget、credit、overflow range和coalesce policy。
- producer fast path不跨subscriber锁，取得1/64/1K fanout CPU/RSS证据。

### M3 · Acknowledged ABI 与 Editor Transaction

- 新API table提供budgeted drain lease、range metadata、ack/nack/checkpoint和resync token。
- Gateway验证与typed apply成功后才ack；error/panic保留delivery或进入typed quarantine/resync。
- 同event compatible Editor consumer共享Runtime cursor并在Editor内有界fanout。

### M4 · Lifecycle、Reload 与 Recovery

- subscription/provider/session/world统一revoking、draining、faulted、resyncing和terminal状态机。
- paused World drop、plugin reload、Session destroy retry、Editor reconnect均有quiescence和generation fence。
- reader-count副作用迁移为幂等broker activation lease。

### M5 · Scale、Fault 与产品资格

- required Windows lane执行AI/Navigation真实产品、1K/10K delivery、1/64/1K subscriber及60s slow consumer。
- 注入allocation/decode/schema/callback/ack/unload/crash故障，证明无静默loss、duplicate或cross-generation apply。
- 与同场景Unreal/Bevy适用路径记录CPU、RSS、latency和功能语义；未同条件测量前不得宣称超过。

## 15. 验收门禁

| Gate | 验收条件 |
|---|---|
| SEMR-G01 | AI三个Editor consumer在真实Runtime ABI中均可订阅并收到正确schema payload。 |
| SEMR-G02 | event catalog exposure与World provider registration由同一contract生成，不能单边成功。 |
| SEMR-G03 | descriptor含provider/build/schema/scope/delivery generation且可跨reload判stale。 |
| SEMR-G04 | Navigation无subscriber时下一帧debug capture和overlay构造为零。 |
| SEMR-G05 | provider unload拒绝新订阅并使旧cursor得到typed terminal/rebind disposition。 |
| SEMR-G06 | 多World、多Session同event id不串流。 |
| SEMR-G07 | 同一typed event每次publication最多执行一次payload encode。 |
| SEMR-G08 | 1/64/1K subscribers的private retained bytes不按完整payload线性复制。 |
| SEMR-G09 | World/Session/provider/stream/consumer所有层级均有entry+byte hard budget。 |
| SEMR-G10 | producer publish不持subscriber queue锁执行serializer或foreign callback。 |
| SEMR-G11 | lossless、latest、coalesced、bounded策略各有独立overflow行为测试。 |
| SEMR-G12 | partial subscriber pressure产生逐cursor disposition，不再只有聚合bool。 |
| SEMR-G13 | 每个admitted event拥有stream generation与唯一sequence。 |
| SEMR-G14 | Editor拒绝任何非连续range并在apply前进入resync/fault。 |
| SEMR-G15 | Runtime authority在ack前保留delivery，allocation成功不等于handled。 |
| SEMR-G16 | duplicate ack/nack/drain重试均幂等。 |
| SEMR-G17 | callback Err不会永久消费当前delivery且留下retry/quarantine/resync终态。 |
| SEMR-G18 | callback panic被隔离，当前delivery identity与consumer fault state可查询。 |
| SEMR-G19 | overflow报告first/last dropped sequence、count、bytes、reason和recovery token。 |
| SEMR-G20 | snapshot class在gap后以新snapshot generation恢复一致状态。 |
| SEMR-G21 | subscriber Drop在paused World也能bounded wake并收敛reader activation。 |
| SEMR-G22 | World drop callback failure有forced disposition且外部side effect最终收敛。 |
| SEMR-G23 | explicit unsubscribe、Drop、Session destroy和plugin revoke恰一次retire。 |
| SEMR-G24 | stale slot/ABI/provider generation永不作用于复用后的订阅。 |
| SEMR-G25 | reader activation callback幂等、不可重入破坏registry且失败可回滚。 |
| SEMR-G26 | ABI page header不逐delivery复制稳定descriptor。 |
| SEMR-G27 | drain请求的count/bytes/deadline/credit在Runtime编码前生效。 |
| SEMR-G28 | Gateway protocol/decode失败不会丢失未ack range。 |
| SEMR-G29 | PlayInstance、RuntimeSession、World和provider identity在wire中分域。 |
| SEMR-G30 | compatible Editor consumers共享cursor时仍保持各自callback fault和ack policy。 |
| SEMR-G31 | capability reconcile以prepared set原子切换，失败保留旧可用set。 |
| SEMR-G32 | idle consumer每tickencode/decode/alloc为零或达到明确稳态预算。 |
| SEMR-G33 | runtime/editor指标可重建publish-to-handle latency、lag、bytes和drop/resync。 |
| SEMR-G34 | p95/p99指标绑定sample count、window、BuildSet、profile和workload。 |
| SEMR-G35 | required lane运行真实1K/10K ABI测试，不保留ignored作为唯一证据。 |
| SEMR-G36 | 1/64/1K subscriber storm记录CPU、RSS、allocation、lock wait和lag并通过阈值。 |
| SEMR-G37 | 128 KiB payload、64 MiB pressure和60s slow consumer均不突破global budget。 |
| SEMR-G38 | plugin reload、Session destroy retry、Editor reconnect和process crash矩阵通过。 |
| SEMR-G39 | 所有failure artifact绑定source fingerprint、hardware、profile和terminal receipt。 |
| SEMR-G40 | 同场景同硬件对比完成前，文档、UI和发布材料不得宣称达到或超过Unreal。 |

## 16. 状态与开放记录

- `review_status: review_complete`只表示本轮current-source静态审查和参考对照完成；3/60/16项均未实施。
- `implementation_status: pending`；MVP F0-F5仍blocked，本轮按规则不修改production、不运行Cargo。
- Runtime10 bounded delivery failure仍为`open`：fixed page已有实现，但ack/resync、global memory和受管真实ABI证据未闭合。
- Plugins01 frame-budget failure仍为`open`：真实ABI 1K/10K测试仍ignored，单次Gateway call也没有request-aware deadline/credit。
- Plugins12 drop lifecycle failure仍为`open`：generational reclaim已进入source，但World drop、provider reload和最终受管证据未闭合。
- Editor02 pump failure仍为`open`：单页pending和锁外callback已进入source，但callback commit/recovery与完整动态矩阵未闭合。
- AI产品断链是本轮current-source新确认的纵向P0；不能用manifest测试或fake gateway把它写成已完成。
