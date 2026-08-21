---
related_code:
  - zircon_editor/src/core/editor_event
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_execution/dispatch.rs
  - zircon_editor/src/ui/host/editor_event_execution/execution_outcome.rs
  - zircon_editor/src/ui/host/editor_event_execution/undo_policy.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/event_dispatch.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport/pointer_dispatch.rs
  - zircon_editor/src/ui/retained_host/app/automation.rs
  - zircon_app/src/entry/entry_runner/editor/composition.rs
  - zircon_app/src/entry/entry_runner/editor/project_automation.rs
tests:
  - zircon_editor/src/tests/editor_event
  - zircon_editor/src/ui/retained_host/app/tests/retained_host_automation.rs
  - zircon_app/tests/editor_mvp_authoring.rs
plan_sources:
  - docs/zircon_editor/core/editor_event.md
  - docs/plans/zircon_editor/editor/02/failure-2026-07-17-editor-event-journal-listener-unbounded-retention.md
  - docs/plans/zircon_editor/editor/02/2026-07-18-editor-event-retention-and-lock-split.md
  - docs/plans/performance/01/2026-08-15-editor-event-retention-routing-current-architecture-review.md
  - docs/plans/performance/01/2026-08-16-editor-core-editor-event-input-transaction-audit-current-architecture-review.md
  - docs/plans/mvp/06-f5-acceptance-wave.md
  - docs/plans/optimize/zircon_app/07-renderable-empty-project-template-create-import-render-export-evidence-product-integration-review.md
  - docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/48-editor-message-bus-topic-subscription-inbox-retention-admission-dispatch-request-dirty-projection-shutdown-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Delegates/MulticastDelegateBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/ScopedTransaction.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorTransaction.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
  - dev/bevy/crates/bevy_ecs/src/message/message_cursor.rs
  - dev/godot/core/object/object.cpp
  - dev/godot/core/object/message_queue.h
  - dev/godot/core/object/message_queue.cpp
  - dev/godot/core/object/undo_redo.cpp
  - dev/Fyrox/editor/src/message.rs
  - dev/Fyrox/editor/src/command/mod.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Util/MessageManager.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Tests/Editor/UnitTests/MessageManagerTests.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 49 · Editor Event Runtime / Envelope / Listener Registry / Journal / Replay / Snapshot / Dirty / Lifecycle 产品集成工程化差距

## 1. 结论

`core::editor_event`已经从早期的无界`Vec`和全局锁fanout演进出一批应当保留的基础：事件ID、到达sequence与revision分开；journal和每个listener inbox共享一个immutable `Arc<SharedEditorEventRecord>`；DurableReplay、FrameLocal、LatestState有独立count/encoded-byte/age预算；latest replacement有key index；listener page使用独立delivery cursor；registry只在短锁内复制immutable route snapshot，filter与per-inbox enqueue位于锁外；drop/coalesce/lag有诊断；focused测试覆盖1,000 listener x 1,000 event、10,000 paused deliveries、byte/age eviction、out-of-order arrival与cursor continuation。旧failure中“仍然无界、每listener深拷贝、registry全局锁跨enqueue”的描述已经过时，本轮不恢复这些结论。

但这套实现仍不是工程级Editor事件、审计、监听和重放基础设施。最严重的新发现位于当前MVP F5产品路径：retained-host automation在每个binding前后各深拷贝一次完整全局journal，用前一次`records().len()`作为后一次slice下标，并把slice中的所有记录都归给当前binding。retention淘汰或latest coalesce可使新snapshot缩短并触发越界panic；并发或refresh产生的无关记录会被误归因；记录被sequence重排时，长度差也不是因果receipt。之后`normalize_cli_action_records()`还克隆这些真实由RetainedHost callback产生的记录，覆盖`source = Cli`和`binding_path = 当前binding`。F5测试反过来要求这份伪造后的provenance，导致错误的证据合同被测试固化。

既有性能审查确认的三个P0同样仍存在：`EditorEventReplay`把journal所有类别和失败记录重新执行，包含pointer、transient、save/import/close等外部副作用；pointer move在零listener时仍支付command reverse lookup、shell锁、effect/result分配、完整record clone、JSON长度编码、journal索引和fanout；`begin_event()`又在执行前推进全局revision，使失败、no-op和每次实时输入都伪装为authoring commit。当前listener控制面则没有任何production consumer，delivery DTO甚至不携带`event/effects/binding/transaction/save/revision/undo`，因此测试证明的是一个可轮询容器，不是可被插件、远程控制或恢复系统安全采用的产品协议。

本报告登记 **5项P0、60项P1、15项P2和40个资格门**。Editor02继续拥有document/transaction/save authority，Editor08拥有command/capability/remote admission，Editor14拥有animation authoring语义，Editor48拥有通用message bus；App07与Tooling15拥有模板和F5 EvidenceSet。Editor49只拥有`EditorExecutionReceipt -> AuditEnvelope / ObservationDelivery / CommittedOperationLog / ActionInvocationReceipt`之间的event-specific contract，不重复建立第二个transaction、message bus或evidence service。

## 2. 审查边界、currentness 与证据等级

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | 证据等级 | 本轮检查重点 |
|---|---:|---|---|
| `core/editor_event`完整生产模块 | 36 / 2,667 / 85,420 | E3 | 36/36逐文件检查event schema、stamp、journal、retention、listener registry/route/page/control与replay |
| `tests/editor_event`聚焦测试 | 30 / 8,109 / 293,454 | E3 | 138个test attributes及support；逐test inventory并深读retention/listener/registry/replay/retained/project/trace主链 |
| 直接产品caller与F5消费链 | 13 / 3,970 / 148,082 | E3 | pointer callback、host dispatch/effect/undo、Context owner、retained automation、App composition/report与MVP assertion |
| 模块文档、failure、性能与owner计划 | 13 / 3,466 / 350,278 | E2/E3 | 既有forward repair、开放验证、F5边界及跨报告唯一owner |
| Unreal、Bevy、Godot、Fyrox、Unity Graphics参考 | 15 / 16,847 / 566,660 | E2/E3 | committed transaction、delegate/signal lifetime、reader cursor、queue admission、command/message分层与provider scope |
| 去重冻结合计 | 107 / 35,059 / 1,443,894 | E2/E3 | 当前工作树fingerprint `af663ee796aa7f191f48c880135e3af3c2d6ab9f9384c29c9b966fd2934692fd` |

指纹按107个selected path去重排序，对每个文件取lowercase SHA-256，再以`forward/slash/path<TAB>hash`和LF连接、无末尾LF后取总SHA-256。冻结日期为2026-08-19，基线提交为`25e09a23178000f2e783ce2143cf70a8b118d404`。

### 2.2 在途文件与动态边界

1. Coordinator当前仍由MVP00 session持有`listener/registry.rs`、三个`tests/editor_event`文件和`zircon_app/tests/editor_mvp_authoring.rs`。台账落笔前再次重读current diff：registry仍仅有两处test helper构造调用调整；其余在途变化分别是F4场景加载、plugin snapshot借用、authoring world API及retained test imports，均未改变F5 journal切片/provenance断言或本文生产控制流结论。二次重算后的fingerprint为`af663ee796aa7f191f48c880135e3af3c2d6ab9f9384c29c9b966fd2934692fd`；实施前仍必须再算。
2. 本轮为review-only，没有运行Cargo、产品F5或性能捕获。当前树由其他session持续收敛，不能把历史117/138 tests、旧WPR或source guard当作本轮动态通过。
3. P0-01/02由当前确定性控制流成立；未运行不等于未发现。实施时应先写小预算/并发/retention RED测试，再改产品路径。
4. 当前`out_of_order_fanout...`测试要求journal snapshot为到达顺序`[4,3,1]`，而`records()`按`retained_by_event_sequence`三路merge会产出sequence顺序；这是静态合同矛盾，本文不冒充已执行失败结果。
5. 全仓production adoption反查只找到retained automation消费`journal()`；listener control与`EditorEventReplay::replay`只在测试中调用。公开method存在不等于产品采用。

### 2.3 检查方法

按`raw input/binding -> normalized event -> stamp allocation -> command metadata -> shell execution -> transaction/save trace -> effects/result -> UI refresh/log -> journal retention -> listener route -> page/ack -> replay/F5 evidence -> unregister/shutdown`顺序逐段阅读；再反向搜索所有production consumer。对每个失败分支判断：authoritative mutation是否发生、revision是否推进、receipt能否返回、journal是否接受、listener是否见到、caller能否重试、provenance能否被证明。

## 3. 必须保留的工程基础

1. 保留sequence/revision、journal、listener三类锁owner，禁止恢复单个service大锁。
2. 保留每条record一个immutable shared owner，fanout只clone `Arc`。
3. 保留Durable/FrameLocal/Latest三类预算结构，但重命名和重分类必须反映真实durability/replay disposition。
4. 保留count/byte/age eviction与drop/coalesce/lag诊断，扩展原因和resync receipt。
5. 保留latest key index与旧sequence不能覆盖新state的保护。
6. 保留listener-local arrival cursor；不要用event sequence替代continuation。
7. 保留page上限256作为count硬门，再增加bytes/deadline而不是删除count门。
8. 保留immutable route snapshot与registry guard外filter/enqueue，补generation/fence而不是恢复持锁回调。
9. 保留journal/listener共享payload，DTO只在真实ABI边界投影一次。
10. 保留poison recovery的进程存活能力，但必须同时发布degraded state与fault receipt。
11. 保留typed `EditorEvent`作为本地语义输入，不把所有行为退回字符串command。
12. 保留operation ID/group、binding、transaction/save generation字段的审计价值，但使其qualified、validated和不可事后改写。
13. 保留failed operation进入audit的能力，但失败audit绝不能成为可执行replay entry。
14. 保留authoring transaction作为undo authority，event journal不得变成第二个undo stack。
15. 保留retained-host callback产品路径；自动化必须通过真实callback，但应返回真实action receipt而不是从global journal猜测。

## 4. 当前实现与产品断路

```text
Pointer / Binding / Operation
  -> begin_event(): event_id++, sequence++, revision++ (before execution)
  -> reverse command metadata lookup
  -> lock whole WorkbenchShell -> execute
  -> build effects + JSON result + transaction/save trace
  -> refresh/log
  -> record.clone()
  -> SharedEditorEventRecord::new
       serde_json::to_vec(full record) -> len -> discard bytes
  -> journal push
  -> snapshot listener routes
  -> sequential filter + inbox lock + enqueue

Replay
  -> journal.records() across Durable + FrameLocal + Latest
  -> clone raw event
  -> dispatch as Replay (mapped to UiBinding operation source)
  -> execute side effects again and append new journal rows

F5 retained automation, per binding
  -> journal_start = deep_clone_global_journal.records.len
  -> invoke real retained callback + refresh
  -> action_journal = deep_clone_global_journal
  -> action_journal.records[journal_start..]
  -> clone every delta row
  -> overwrite source=Cli and binding_path=current hardcoded path
  -> App serializes rows as product evidence
```

## 5. P0：当前正确性、证据与热路径断路

### E-EVT-P0-01 · F5用global journal长度差伪造单binding receipt，可越界panic或误归因

`automation.rs:139`在callback前深拷贝完整journal并保存`records().len()`；`155-156`在refresh后再次深拷贝并直接slice。journal有count/bytes/age eviction和latest coalescing，当前长度可小于起始长度，slice因此panic。即使不panic，global journal delta可混入refresh或并发事件；长度没有ActionId、causal parent或commit fence，无法证明哪条record由当前binding产生。journal snapshot又按event sequence组织，而listener page按arrival cursor，长度差更不构成稳定continuation。

目标：每次产品callback入口创建qualified `ActionInvocationId`，dispatch返回一个不可变`EditorExecutionReceipt`或明确的0/1/N child receipt set；receipt包含initiator、executor path、operation、transaction/save generation、effects、terminal disposition和causal parent。F5只消费这个返回值，不读取global journal。测试覆盖coalesce、eviction、refresh副事件、并发producer、callback失败和一动作多receipt。

### E-EVT-P0-02 · F5事后改写真实source与binding，当前测试要求伪造provenance

`normalize_cli_action_records()`克隆真实`RetainedHost`记录并覆盖`source = Cli`与硬编码`binding_path`。`editor_mvp_authoring.rs:171-188`明确要求六条记录都宣称`Cli`和canonical path；retained-host单测也把这种重写命名为“CLI evidence”。这混淆了initiator、transport、executor和observed callback：CLI请求可以发起RetainedHost callback，但执行记录的真实source不能被事后改写；global delta中的无关记录还会一起被重标。

目标：hard-cut为`initiator = Cli`、`transport = ProductAutomation`、`executor = RetainedHostCallback`、`binding = actual normalized binding`四个不可变维度；audit owner在dispatch时签发，不允许evidence adapter mutate record。旧字段迁移必须fail-close，历史测试改为验证双来源链和record hash不变。

### E-EVT-P0-03 · journal所有记录均可执行replay，包含实时输入、失败与外部副作用

`EditorEventReplay::replay()`逐条clone `record.event`并再次dispatch，只比较是否发生相同错误。它不读取`retention class`、`undo_policy`、transaction、revision、effects或side-effect disposition。默认catch-all又把Open/Save/Close Project、ImportModel、layout、selection、draft、search、console与OpenCommandPalette归为`DurableReplay`。重放失败记录时，若当前版本已经成功，副作用先发生，然后replay才报告“expected failure”。

目标：删除`replay(&[EditorEventRecord])`执行面；仅允许versioned `CommittedOperationEntry`或transaction delta进入replay，显式声明precondition、target identity、schema、idempotency、external-effect policy与rollback/checkpoint。raw input、presentation、failure和external request只能进入audit，不能执行。

### E-EVT-P0-04 · 每个pointer move仍同步支付完整command/audit/listener路径

retained pointer callback把move立即转换为`EditorViewportEvent::PointerMoved`；正常dispatch在零listener下仍推进revision、反查command descriptor、锁整个shell、创建effect Vec与JSON result、深clone成功record、完整serde到临时Vec、更新journal三个索引。存在listener时再逐route过滤和锁inbox。LatestState只限制最终队列长度，不限制每次125/500/1,000 Hz输入的工作量。

目标：RealtimeInput直接进入interaction owner并在frame boundary coalesce；press/release/cancel保序。只有semantic command生成execution receipt；audit observation按显式策略和预算派生，零observer时route/serialization/JSON为0。用同机WPR/xperf和stage counters比较input-to-damage p50/p95/p99、alloc bytes、lock wait/hold和CPU/package power。

### E-EVT-P0-05 · revision在执行前推进，当前值是尝试计数而非成功authoring commit

`begin_event()`无条件`allocate_stamp(true)`，在command lookup和execution前用`saturating_add`推进revision。失败、no-op、hover、pointer、resize和纯presentation都获得更高`after_revision`；唯一test还明确断言OpenCommandPalette和Hover各推进一次。缓存、dirty、CAS或evidence若把它解释为authoring generation，会得到假依赖和假提交。

目标：拆分`EventOrder`、`PresentationGeneration`、`DocumentRevision`与`TransactionGeneration`。EventOrder在admission时分配；document revision只在successful changed commit后推进一次；失败/no-op/realtime input推进0次。receipt必须携带commit disposition，禁止通过`before != after`猜测mutation。

## 6. P1：Event、Receipt 与 Revision 合同

| ID | 当前差距 | 目标重构 |
|---|---|---|
| E-EVT-P1-01 | 一个`EditorEvent`同时装raw input、presentation state、semantic command、external request和audit observation。 | 拆`RealtimeInput`、`EditorCommandIntent`、`PresentationDelta`、`AuditEnvelope`和`CommittedOperationEntry`，用receipt连接而非继承同一enum。 |
| E-EVT-P1-02 | `EditorEventEnvelope`只有source+event，没有Action/Request/Project/Document/Window/Viewport/Session identity。 | envelope携qualified scope、correlation、causal parent、deadline与schema。 |
| E-EVT-P1-03 | `EditorEventSource`由caller直接传入，不能证明principal、transport、executor或plugin generation。 | 由trusted dispatch boundary构造不可伪造`InvocationProvenance`。 |
| E-EVT-P1-04 | record和enum没有schema/version/migration/unknown-field策略。 | 本地typed owner与持久/wire DTO分离；后者有SchemaId、version、reader/writer matrix和fail-closed migration。 |
| E-EVT-P1-05 | serde默认enum variant/field名称直接成为持久形状，Rust rename即协议变更。 | 生成稳定tag/field ID与compatibility tests，不让源码标识符充当ABI。 |
| E-EVT-P1-06 | EventId/Sequence可Default为0且只有裸`u64`，无process/session generation。 | nonzero qualified ID包含EventService generation；0只作为明确invalid sentinel且不反序列化为合法ID。 |
| E-EVT-P1-07 | event ID、sequence、revision和delivery cursor使用`saturating_add`，到MAX后静默重复。 | checked exhaustion进入terminal degraded state，拒绝新admission并给operator receipt。 |
| E-EVT-P1-08 | revision是全Editor全局值，无法表达哪个project/document/world发生提交。 | DocumentRevision按authority scope分配并与Project/Document generation绑定。 |
| E-EVT-P1-09 | `EditorEventResult`允许value/error同时为空或同时存在，success Value完全无typed schema。 | `Result<TypedOutcome, TypedEditorError>`加disposition、retry和user action。 |
| E-EVT-P1-10 | effects是可重复、无scope/generation的`Vec`，caller需线性探测。 | 固定bit/domain mask + scoped typed external requests，去重并绑定publication generation。 |
| E-EVT-P1-11 | viewport全部标为DelegatedToTransactionEngine；layout/save/open/import又标FutureInverseEvent，但当前没有inverse。 | 每command descriptor声明真实undo/replay/external disposition，未实现能力不得用Future命名冒充。 |
| E-EVT-P1-12 | failure path用通用effects猜测dirty，不能区分零mutation、partial mutation或compensation。 | execution返回`NoMutation/Committed/PartiallyCommitted/Compensated/Unknown`，effects只来自真实commit receipt。 |
| E-EVT-P1-13 | operation args、binding path、subject path、query、error和JSON长期进入journal，无redaction或secret policy。 | AuditPolicy按field分类allow/redact/hash/drop，并在admission前限制depth/items/bytes。 |
| E-EVT-P1-14 | transaction_id/save_generation是裸`u64`，缺history/project/document generation和authority。 | 使用qualified `TransactionRef`、`SaveReceiptRef`，跨reopen/PIE不可ABA。 |
| E-EVT-P1-15 | failure记录入journal后API只返回`Err(String)`，caller拿不到event ID、effects、partial state或audit receipt。 | dispatch始终返回typed terminal receipt；业务error作为receipt disposition，不丢诊断identity。 |

## 7. P1：Journal、Retention 与 Publication

| ID | 当前差距 | 目标重构 |
|---|---|---|
| E-EVT-P1-16 | 成功dispatch为“返回给caller + 保留”深clone完整record。 | receipt与audit共享一个immutable owner，caller按引用/Arc消费。 |
| E-EVT-P1-17 | `record()`返回`()`, journal因oversize/age/count drop或listener lag时caller完全未知。 | 返回`EventPublicationReceipt`，列journal和每observer的Accepted/Coalesced/Dropped/Closed。 |
| E-EVT-P1-18 | 先push journal再取得route snapshot；并发register可能收到订阅前事件，unregister也可能收到终态后事件。 | publication在一个generation-stamped plan冻结membership；register/unregister返回明确first/last visible cursor。 |
| E-EVT-P1-19 | matching listener按route顺序逐个锁inbox，单个contended listener可串行阻塞后续owner。 | bounded observer admission、affinity与per-owner wait telemetry；必要时用共享log/cursor，但需测量选择。 |
| E-EVT-P1-20 | mutex poison统一`into_inner()`，没有degraded flag、diagnostic或consistency check。 | 恢复后标记owner degraded，执行index invariant check并禁止错误地宣称healthy。 |
| E-EVT-P1-21 | `DurableReplay`只在内存保留最多24小时，age使用`Instant`，进程退出即消失。 | 重命名为`AuditWindow`；真正durable log另有crash-safe storage、wall/monotonic双时钟和recovery。 |
| E-EVT-P1-22 | retention match的`_`全部进入DurableReplay，search/draft/console/layout/selection等语义被混放。 | 每event descriptor显式声明Audit/Frame/Latest/CommittedReplay/DoNotRetain，新增variant缺声明即编译失败。 |
| E-EVT-P1-23 | pointer、viewport、timeline latest key是进程全局，缺window/viewport/document/editor instance scope。 | key包含qualified owner scope和generation。 |
| E-EVT-P1-24 | HoverNode/PressNode payload带node path，但latest key只有全局类型；A-clear可被B-set替换。 | 使用node-qualified key或发布完整InteractionState generation，保证旧节点清除。 |
| E-EVT-P1-25 | 每条record先`serde_json::to_vec`完整分配/编码，只读len后丢弃。 | typed owned-heap charge在构造时增量计算；只在真实wire/persistence boundary编码一次。 |
| E-EVT-P1-26 | serde失败fallback到`size_of_val(record)`，严重漏计String/Vec/Value heap。 | serialization failure是typed rejection；禁止用stack size冒充resident charge。 |
| E-EVT-P1-27 | 没有单record pre-admission；64 MiB以上payload可先分配/clone/serialize再被push后淘汰。 | 在event/args构造前按items/depth/bytes预算拒绝，single-item hard cap独立于queue cap。 |
| E-EVT-P1-28 | encoded JSON bytes不包含capacity、Arc、maps/index、allocator overhead，不能证明RSS预算。 | 定义calibrated logical charge并用heap/RSS profiler验证系数和pressure behavior。 |
| E-EVT-P1-29 | retention budget硬编码为全service默认，所有listener共享同一策略，无法表达criticality和topic。 | policy registry按event class、subscriber role、profile与memory pressure解析，配置有provenance。 |
| E-EVT-P1-30 | `journal()`每次merge并深clone全部retained record；没有cursor/bytes/deadline page。 | 产品只用bounded shared page/export stream，full snapshot限debug且有显式预算。 |
| E-EVT-P1-31 | journal宣称sequence order，现有out-of-order test却要求arrival order；listener page又使用arrival cursor。 | 定义allocation/execute/commit/publication四种order，journal/replay选择唯一commit order并做并发model test。 |

## 8. P1：Listener Registry、Page、Ack 与 Lifecycle

| ID | 当前差距 | 目标重构 |
|---|---|---|
| E-EVT-P1-32 | listener control在production只有method definition，无真实注册、轮询或ack consumer。 | 先接入一个owner-qualified产品consumer并通过shutdown/lag/resync，再宣称该能力available。 |
| E-EVT-P1-33 | listener ID/display name接受任意空白、长度、字符和case；filter raw Vec同样无预算。 | validated namespaced ID、display length和filter item/bytes cap。 |
| E-EVT-P1-34 | descriptor没有owner plugin/session/principal/capability/affinity/criticality。 | `EditorObservationSubscriptionDescriptor`携完整owner与execution policy。 |
| E-EVT-P1-35 | unregister后同名可立即重注册，旧Arc route/inbox仍存活，存在ABA和不可见orphan delivery。 | SubscriptionId包含generation；旧route只能投到旧generation并有terminal discard/drain receipt。 |
| E-EVT-P1-36 | unregister无条件销毁registry可达的pending记录，不报告count/bytes/oldest/critical drops。 | `Drain/RejectIfPending/DiscardWithReceipt`显式策略。 |
| E-EVT-P1-37 | inline test明确允许unregister后旧route继续enqueue detached inbox。 | unregister fence等待in-flight plan或把旧plan标Stale，不允许终态返回后静默写orphan。 |
| E-EVT-P1-38 | disable/filter update同样只影响新snapshot；旧snapshot继续按旧filter投递且没有last-visible cursor。 | mutation receipt包含effective generation和boundary cursor。 |
| E-EVT-P1-39 | 没有RAII lease、explicit shutdown、drain deadline、leak census或Drop terminal contract。 | Host/plugin/session退出按逆依赖序revoke -> stop admission -> drain/discard -> census。 |
| E-EVT-P1-40 | filter只能按operation prefix/group/source/success，无法按event kind/effect/project/document/window/transaction筛选。 | typed topic/predicate compiler，scope先索引再执行有限predicate。 |
| E-EVT-P1-41 | prefix/group/source列表不dedupe且无最大项数；prefix lower-case而operation ID匹配依赖外部canonical保证。 | 注册时parse canonical operation path、dedupe、编译filter并返回cost/coverage receipt。 |
| E-EVT-P1-42 | delivery DTO不包含实际event、effects、binding、transaction/save、undo和revisions，consumer无法语义处理或resync。 | in-process shared typed delivery；wire DTO按subscription projection声明字段和schema。 |
| E-EVT-P1-43 | page只有count cap，256条宽JSON可突破合理bytes和wall budget。 | count+bytes+deadline三门，返回remaining、projection bytes和truncated reason。 |
| E-EVT-P1-44 | page没有gap/resync_required/baseline_generation/final/closed，空页含义不明确。 | page contract携gap range、snapshot token、terminal state与next cursor。 |
| E-EVT-P1-45 | lag信息只在另一次status查询，page/status间可继续drop，consumer无法原子决定resync。 | 每page包含同一lock snapshot的lag/resync state。 |
| E-EVT-P1-46 | ack接受任意cursor并删除所有更早记录，不校验listener generation、delivered high-water或stale cursor。 | ack携SubscriptionId+page token，拒绝future/stale/foreign cursor并保持幂等。 |
| E-EVT-P1-47 | status首尾只报告event sequence，不报告pending delivery cursor bounds；并发out-of-order时含义混乱。 | 同时公开event commit range与delivery cursor range，名称不得混用。 |
| E-EVT-P1-48 | page先clone owned delivery中的String/JSON/result，再`listener_deliveries()`二次JSON投影。 | 只在最终ABI boundary一次编码；in-process consumer保留shared payload。 |
| E-EVT-P1-49 | control response是`Value + Option<String>`，没有request/schema/principal/deadline/cancel/audit identity。 | 复用typed operation contract，所有admin mutation和query返回versioned receipt。 |

## 9. P1：Replay、产品采用与 F5 Adapter

| ID | 当前差距 | 目标重构 |
|---|---|---|
| E-EVT-P1-50 | replay输入没有Project/Document/World/BuildSet/schema identity或precondition。 | CommittedOperationLog绑定authoritative target generation与compatible build/schema。 |
| E-EVT-P1-51 | replay忽略`undo_policy`和retention class，无法区分audit与executable row。 | compile-time exhaustive replay disposition；未知/legacy默认Reject。 |
| E-EVT-P1-52 | replay不比较event_id/revision/effects/transaction/save/result value/changed final state。 | 每step验证pre/post hash、commit revision、transaction和typed outcome。 |
| E-EVT-P1-53 | expected-failure row若现在成功，副作用已发生后才返回错误。 | failure audit永不执行；preflight必须在apply前确认operation eligibility。 |
| E-EVT-P1-54 | replay用exact error String比较，文案、路径或版本变化即伪失败。 | stable error code + structured fields；localized/detail text不参与语义相等。 |
| E-EVT-P1-55 | batch中途失败没有checkpoint、rollback、compensation、idempotency或cancel。 | staged replay transaction，支持dry-run、checkpoint、abort/compensate与unknown-outcome reconciliation。 |
| E-EVT-P1-56 | Replay source映射为UiBinding operation source，并把重放结果再次写journal。 | 独立Replay capability/sandbox；audit派生不递归成为默认replay输入。 |
| E-EVT-P1-57 | replay、listener和journal export都无production caller，测试无法证明产品生命周期。 | 每个能力先定义availability truth和owner；无consumer时保持Internal/Experimental。 |
| E-EVT-P1-58 | F5 canonical binding path由hardcoded match生成；`SelectCube`可标记任意node ID。 | 使用实际normalized binding/semantic selector，label不得伪造目标身份。 |
| E-EVT-P1-59 | 一个binding可生成0/N global records，adapter没有expected event、transaction/save、child receipt或causal closure。 | Scenario step声明expected receipt graph和state transition，缺失/额外记录均按typed policy处理。 |
| E-EVT-P1-60 | 每个binding两次full journal snapshot，复杂度随保留窗口和payload增长；没有page bytes/deadline。 | Action receipt O(step output)，F5严禁读取global journal；EvidenceSet只引用receipt digest和必要projection。 |

## 10. P2：长期工程能力

| ID | 能力 | 目标 |
|---|---|---|
| E-EVT-P2-01 | Durable audit segment | 可选append-only segment、checksum、rotation、crash recovery与support export。 |
| E-EVT-P2-02 | Event schema registry | 自动生成topic/disposition/redaction/retention/replay declaration和compat matrix。 |
| E-EVT-P2-03 | Event timeline UI | 按commit/dispatch/delivery/ack展示因果、lag、drop和resync。 |
| E-EVT-P2-04 | Compiled subscription predicates | 在可证明cost预算内支持typed field predicate，不执行任意插件代码。 |
| E-EVT-P2-05 | Adaptive retention | memory pressure下按policy降采样/缩窗并生成可解释receipt。 |
| E-EVT-P2-06 | Compact wire encoding | ABI边界支持versioned binary/delta/compression，不影响本地typed owner。 |
| E-EVT-P2-07 | Observation sampling | 高频input观测支持deterministic sampling和sample weight。 |
| E-EVT-P2-08 | Subscription inspector | 查看owner、generation、filter cost、backlog、last ack与shutdown state。 |
| E-EVT-P2-09 | Replay dry-run/diff | 在clone/staging document上输出预计change set和冲突，不触发external side effect。 |
| E-EVT-P2-10 | Replay compatibility lab | 多版本BuildSet与schema corpus验证upgrade/downgrade/reject行为。 |
| E-EVT-P2-11 | Audit privacy profile | project/team/CI/support不同export profile与consent/retention policy。 |
| E-EVT-P2-12 | Localized operation presentation | stable operation identity与locale display/detail分离。 |
| E-EVT-P2-13 | Consumer SDK | cursor/page/ack/resync/lease的typed Rust与跨进程SDK。 |
| E-EVT-P2-14 | Model checking/fuzz | 并发publish/register/unregister/drop/ack与replay failure state-machine测试。 |
| E-EVT-P2-15 | Multi-instance event isolation | 同进程多Editor/Project/PIE窗口互不覆盖latest、revision和subscription。 |

## 11. 参考引擎对照与适用边界

| 参考 | 可复用事实 | 对Zircon的约束 | 不照搬 |
|---|---|---|---|
| Unreal Slate/Transaction | pointer move在Slate input route处理；`FScopedTransaction`只在明确请求时begin/end/cancel；transaction保存custom change或object state。 | realtime input、committed transaction与audit必须分owner；只重放已提交semantic change。 | 不复制UObject序列化、Slate widget path或全局GEditor。 |
| Unreal Messaging/Delegates | router先冻结recipient，再按declared thread affinity dispatch；multicast delegate返回`FDelegateHandle`并支持Remove/RemoveAll，广播期间有明确snapshot/compaction语义。 | subscription需要handle、owner、generation、affinity和unregister fence。 | 不新增私有无限router thread，不把Messaging用于本地每次pointer move。 |
| Bevy Messages | 每reader拥有cursor，可得到pending/missed；frame message双buffer并明确update/drop contract。 | 消费进度属于consumer identity；FrameLocal必须有frame清理和miss诊断。 | Bevy默认两帧静默drop不适合Editor audit/transaction。 |
| Godot Signal/CallQueue/UndoRedo | signal在锁内复制slot，one-shot先disconnect；target维护反向connection；CallQueue拒绝超单页消息并统计buffer；UndoRedo独立拥有action history。 | callback lifetime、single-item admission、queue telemetry和undo owner必须明确。 | 不复制Variant调用或Object单体模型。 |
| Fyrox Editor | `MessageSender`只发送Editor message；`CommandStack`独立有max capacity、execute/revert/finalize。 | message transport不能冒充undo/replay；command retirement需要finalize。 | 其mpsc sender和简单capacity不是Zircon可靠observer协议。 |
| Unity Graphics | ShaderGraph `MessageManager`按provider/node保存并可清除provider；changed flag只表达diagnostic projection。 | diagnostic/audit owner要能按producer generation撤销，scope不能只靠display name。 | 本地Graphics镜像不含完整Unity Editor event bus，不能用来证明跨进程能力。 |

## 12. 目标架构

```text
RealtimeInput
  -> InteractionOwner
  -> frame-coalesced PresentationDelta
  -> no command lookup / transaction / audit encoding by default

EditorCommandIntent + InvocationProvenance
  -> Command Admission (capability, target generation, precondition)
  -> Transaction/External Effect Executor
  -> EditorExecutionReceipt
       event_order
       commit disposition + scoped document revision
       transaction/save references
       typed effects / external requests
       initiator + transport + executor + causal parent
       terminal error/action
       |
       +-> UI projection/invalidation
       +-> AuditEnvelope policy -> bounded ObservationLog
       +-> CommittedOperationEntry policy -> durable replay owner
       +-> ActionInvocationReceipt -> F5 EvidenceSet

ObservationBroker
  -> SubscriptionLease(owner, generation, scope, predicate, affinity, budget)
  -> shared page(count + bytes + deadline + gap/resync)
  -> ack(page token, cursor)
  -> revoke/drain/discard terminal receipt
```

`EditorExecutionReceipt`不是另一个万能event。它只描述一次已admit执行的真实结果；audit、UI和replay分别投影所需字段。`CommittedOperationLog`归Editor02/03 transaction authority，Editor49只定义从receipt到entry的资格边界。F5的`ActionInvocationReceipt`由真实callback调用栈返回，Tooling15只负责把它和BuildSet/Product/Run/Evidence identity组合，不允许二次改写业务provenance。

## 13. 依赖顺序与重构里程碑

### M0 · Truth Freeze 与 RED Contract

- 冻结当前event variant、retention disposition、production consumer和F5 record来源矩阵。
- 为journal shrink/coalesce、concurrent unrelated record、out-of-order publication、source immutability和one-action receipt写RED。
- 为revision failure/no-op/input、replay external side effect和unregister terminal boundary写RED。

### M1 · F5 Action Receipt P0 Hard Cut

- callback dispatch返回`ActionInvocationReceipt`，删除`journal_start`/slice和`normalize_cli_action_records` mutation。
- App report只投影真实receipt；测试验证initiator/transport/executor三者而非伪`source=Cli`。
- Tooling15 EvidenceSet引用receipt digest、ProductReceipt和ScenarioStepId。

### M2 · Realtime Input、Commit Revision 与 Shared Receipt

- pointer/resize/transient从command/audit热路径拆出，保留edge order和typed UI delta。
- 成功/no-op/failure产生单一shared receipt；document revision只在successful changed commit推进。
- 删除成功record深clone、per-input JSON result和discarded size encoding。

### M3 · Replay Hard Cut

- 定义versioned `CommittedOperationEntry`、precondition、target generation、side-effect policy和checkpoint。
- 迁移合法transaction producer与测试；raw `EditorEventRecord` replay API直接删除，不保留shim。
- failure/input/presentation/save/import/close默认Reject，legacy ambiguity fail-close。

### M4 · Observation Subscription Lifecycle

- 建立owner-qualified generation lease、compiled filter、count/bytes/deadline page、gap/resync和token ack。
- unregister/disable/filter mutation有effective cursor和terminal fence。
- 接入一个真实plugin/diagnostic consumer，完成callback fault、lag、reconnect和shutdown。

### M5 · Audit Storage、Policy 与 Product Adoption

- `DurableReplay`重命名为真实AuditWindow；需要durability的owner使用append-only segment和recovery。
- redaction、single-item admission、heap charge、drop reason与pressure policy闭合。
- journal debug/export通过bounded page，不再full snapshot。

### M6 · 性能、故障与 Competitive Qualification

- 0/1/1K/10K observer、64B/2MiB/64MiB、1/16 producer、125/500/1,000Hz input矩阵。
- WPR/xperf测input-to-damage、alloc、lock、CPU/context switch/package power；产品F5执行coalesce/eviction/concurrency fault注入。
- 只有correctness、failure、memory、latency、shutdown和evidence provenance同时通过，才允许与Unreal等引擎比较。

## 14. 验收门

- [ ] 01. F5不再读取global journal长度或slice。
- [ ] 02. retention shrink/coalesce期间每binding receipt不panic、不丢、不误归因。
- [ ] 03. concurrent unrelated event不进入当前ScenarioStep receipt。
- [ ] 04. stored audit record在evidence projection前后hash不变。
- [ ] 05. receipt同时证明initiator、transport、executor和actual binding。
- [ ] 06. arbitrary selected node不再被标记为`SelectCube`。
- [ ] 07. realtime pointer move不访问command registry、transaction、journal或listener registry。
- [ ] 08. 125/500/1,000Hz下ordered edges不丢，latest state按viewport scope coalesce。
- [ ] 09. failure/no-op/realtime input的DocumentRevision advance为0。
- [ ] 10. successful changed commit恰好推进一次scoped revision。
- [ ] 11. ID/cursor exhaustion显式拒绝，不重复或饱和复用。
- [ ] 12. dispatch失败仍返回typed terminal receipt和audit identity。
- [ ] 13. raw input、presentation、failure和external request不可进入executable replay。
- [ ] 14. legacy replay ambiguity fail-close且零mutation。
- [ ] 15. replay precondition失败在apply前被拒绝。
- [ ] 16. replay batch中途失败可rollback/compensate或明确Unknown并可reconcile。
- [ ] 17. replay不会递归生成默认可重放记录。
- [ ] 18. replay final state/hash、revision、transaction和typed outcome可验证。
- [ ] 19. event/audit/wire schema有version、migration和unknown policy。
- [ ] 20. event/source/transaction/save identity全部绑定owner generation。
- [ ] 21. 单record items/depth/bytes在clone/serialize前admit。
- [ ] 22. 每event不再产生只为len而丢弃的完整JSON buffer。
- [ ] 23. logical charge经heap/RSS矩阵校准且pressure行为可解释。
- [ ] 24. retention disposition对每variant exhaustive，新增variant缺声明编译失败。
- [ ] 25. Hover/Press跨node不会因latest coalesce遗留stale state。
- [ ] 26. journal明确commit order，concurrent publication model test稳定。
- [ ] 27. journal产品访问只有count+bytes+deadline page，没有full snapshot。
- [ ] 28. publication receipt报告journal和每observer disposition。
- [ ] 29. listener注册拒绝非法/超长ID、display、filter和scope。
- [ ] 30. subscription具有owner、principal、capability、generation、affinity和budget。
- [ ] 31. unregister返回后旧route不能向orphan inbox静默投递。
- [ ] 32. pending unregister必须Drain/Reject/DiscardWithReceipt三选一。
- [ ] 33. disable/filter update返回effective generation和last-visible cursor。
- [ ] 34. page同时限制count/bytes/deadline并携gap/resync/baseline/final。
- [ ] 35. ack拒绝foreign/stale/future cursor且幂等。
- [ ] 36. page只在最终ABI边界编码一次，无owned DTO + JSON双clone。
- [ ] 37. 至少一个真实production consumer完成register/page/ack/resync/revoke全生命周期。
- [ ] 38. Host/plugin/session shutdown完成stop admission、drain/discard、terminal receipt和leak census。
- [ ] 39. F5 clean product run验证真实ActionReceipt与persisted state，不能由自报record来源自证。
- [ ] 40. 同机WPR、fault、soak、memory和source-bound Cargo全部通过后才更新implementation status。

## 15. 与其他报告的唯一 Owner 边界

| Owner | 本篇依赖 | 本篇不重复计数 |
|---|---|---|
| Editor01 Retained UI | input route、dirty/delta和present性能 | retained tree/layout/paint/host整体架构 |
| Editor02/03 | document revision、transaction、save与committed change | undo stack、autosave/recovery、scene mutation本体 |
| Editor08 | command identity、when/capability、remote admission | command registry/keymap/menu/palette与remote bypass |
| Editor14 | animation event的真实authoring/compiler语义 | sequence/graph/state-machine完整toolkit |
| Editor48 | 通用EditorMessageBus topic/inbox/request/dirty | message bus zero-subscriber、plugin shadow queue与通用subscription P0 |
| App07 | template/create/import/render/export产品闭环 | template schema、provider/BuildSet和export资格 |
| Tooling15 | BuildSet/ProductReceipt/Run/EvidenceSet/promotion | F5脚本、process supervisor、artifact archive和qualification service |
| Editor49 | event execution receipt、audit/listener/replay和ActionReceipt边界 | 不建立第二bus、transaction、scheduler或evidence service |

## 16. 状态与产出记录

本篇是review-and-refactor plan，不是已接受implementation milestone。当前状态为`review_complete / implementation_pending / dynamic_validation_pending`；没有写入子计划accepted milestone记录，也没有把静态阅读冒充Cargo、F5或性能通过。

本轮只新增本报告并同步`docs/plans/optimize`索引、coverage和跨报告owner总账；未修改Rust、tests、Cargo、ABI、资源、workflow或产品配置。实施时必须先重算五个在途selected path和107文件fingerprint，再按M0-M6执行。
