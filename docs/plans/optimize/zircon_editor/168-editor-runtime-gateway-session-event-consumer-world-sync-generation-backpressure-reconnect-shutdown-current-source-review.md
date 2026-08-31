---
title: Editor Runtime Gateway、Session、Event Consumer、World Sync、Generation、Backpressure、Reconnect 与 Shutdown 当前源码复核
category: zircon_editor
report_id: Editor168
review_date: 2026-08-27
baseline_head: 7fea65a3ae9cb836ad85adfdcece01ae7a6b7df1
production_baseline: 982baa1ba87bc8c25fe44312507a4af15027e058
canonical_owner: Editor47
refreshes:
  - docs/plans/optimize/zircon_editor/47-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/120-editor-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-current-source-review.md
related_code:
  - zircon_editor/src/core/gateway
  - zircon_editor/src/core/runtime_event_consumer
  - zircon_editor/src/core/sync
  - zircon_editor/src/core/play
  - zircon_editor/src/ui/host
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/editor
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_library/runtime_session
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/runtime_api/session/session_identity.rs
  - zircon_runtime_interface/src/runtime_api/session/plugin_event_mirror.rs
  - zircon_runtime_interface/src/world_sync
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
  - zircon_runtime/src/dynamic_api/session/world_sync.rs
tests:
  - zircon_editor/src/core/gateway/session/tests.rs
  - zircon_editor/src/core/sync/pump/tests.rs
  - zircon_editor/src/core/sync/watch_map/tests.rs
  - zircon_editor/src/tests/gateway
  - zircon_editor/src/tests/runtime_event_consumer.rs
  - zircon_editor/src/tests/runtime_event_consumer_bounded_pump.rs
  - zircon_editor/src/tests/runtime_event_consumer_bounded_pump
plan_sources:
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/01/failure-2026-07-17-gateway-stable-call-lock-and-clone.md
  - docs/plans/zircon_editor/editor/02/failure-2026-07-17-runtime-event-consumer-unbounded-pump-lock.md
  - docs/plans/zircon_editor/editor/02/failure-2026-07-22-world-sync-subscription-invalidation-scaling.md
  - docs/plans/zircon_editor/editor/02/failure-2026-08-01-plugin-registration-runtime-consumer-atomicity.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_runtime_interface/04-profiling-plugin-event-script-diagnostic-manifest-crate-ownership-consolidation-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/SessionServices/Public/ISessionManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/SessionServices/Private/SessionManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/SessionServices/Private/SessionManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SessionServices/Private/SessionInfo.h
  - dev/UnrealEngine/Engine/Source/Runtime/SessionServices/Private/SessionInfo.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bridge/MessageBridge.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bridge/MessageBridge.cpp
  - dev/godot/editor/debugger/editor_debugger_node.h
  - dev/godot/editor/debugger/editor_debugger_node.cpp
  - dev/godot/editor/debugger/script_editor_debugger.h
  - dev/godot/editor/debugger/script_editor_debugger.cpp
  - dev/Fyrox/editor/src/message.rs
  - dev/Fyrox/editor/src/plugin.rs
  - dev/bevy/crates/bevy_remote/src/lib.rs
  - dev/bevy/crates/bevy_remote/src/builtin_methods.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/RenderGraph/RenderGraphEditorLocalDebugSession.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/RenderGraph/RenderGraphEditorRemoteDebugSession.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/Debug/DebugMessageHandler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/Debug/RenderGraphDebugSession.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 168 · Editor Runtime Gateway / Session / Event Consumer / World Sync 工程化复核

## 1. 最终结论

Editor120 的“opaque token 可跨替换误伤新 session”主风险已经被当前源码实质封口，但 canonical P0 仍只能判为 **Partial**，不能关闭。当前 `GatewaySessionIdentity` 已包含 runtime instance、runtime session、gateway generation、transport epoch、project 与 play instance；`GatewayLease`/`GatewayOrigin` 固定到不可变 `GatewayGeneration`；world watch 与 plugin subscription 都保存创建时完整 identity，replacement 后的 stale drain/unwatch/unsubscribe 不再改道到新 endpoint。`ArcSwap` replacement 不让短操作持有全局 replacement lock，这些都是可保留的工程底座。

未闭合的是跨步骤协议。replacement 仍然只是发布新 generation；world pump 发现 identity 变化后清空本地 watch，event consumer 则在本帧退休旧 consumer、下一帧重新订阅。两条链都没有 `quiesce -> publish -> rebind -> baseline/resync -> commit -> retire` 状态机、gap 证明或 replacement receipt，因此新 session 在重新绑定前产生的数据可以永久丢失。world invalidation 仍是 `Vec<InvalidationBatch>`，没有 cursor、page sequence、remaining、final、oldest age和world revision；pump 逐 fact 调 `bus.publish`，忽略 dispatch disposition，却立即推进 published count 与 generation watermark。这里是当前最严重的数据一致性缺口。

Runtime event consumer 的局部质量已明显高于旧报告：begin/consume/end 全部有 `catch_unwind`，slow/poison consumer 可隔离，round-robin 与时间/事件预算成立，pending tail 和 fault payload 共用总字节预算，backlog 能表达 lower bound、unknown consumer、oldest age和observation age，fault receipt journal 有界。仍然没有真正的 retry/backoff/dead-letter workflow；consume panic 会隔离 consumer但丢弃剩余 tail；deferred remote cleanup 只按 `consumer_id` 存一条，重复失败重绑可能覆盖/漏记多个 subscription，且失败 cleanup 只再尝试一次。

Host/App 已有 typed shutdown receipt 和 consumer -> watch -> play -> gateway -> backend 的显式顺序，但 active tick 的 frame tick、consumer pump、world pump和hierarchy/Inspector query分别取得 lease；plugin registration 的 UI install 与下一帧 consumer activation也不是同一个提交。Retained Host 只保存 shutdown receipt，没有在一般产品退出路径上证明失败会阻止 session owner 释放；Drop 分散在 Host controller 与 RuntimeSession，后者 teardown 失败会 `abort`，所以统一 terminal coordinator 仍未完成。

本轮不新增 finding，继续由 Editor47 唯一拥有 1 个 P0、44 个 P1、12 个 P2。当前重判为：P0 **0 Open / 1 Partial / 0 Closed**；P1 **11 Open / 27 Partial / 6 Closed**；P2 **12 Open**；36 个 canonical gate 为 **9 Fail / 27 Partial / 0 Pass**。局部 Closed 不表示系统达到 Unreal 或其他参考引擎水平；没有同场景、同硬件、同构建配置的可靠性与性能证据，禁止作“优于 Unreal”的结论。

## 2. 审查边界与 currentness

### 2.1 唯一 owner 与去重

1. Editor168 只刷新 Editor47/Editor120，不重复登记 Runtime43 的动态 session/ABI 实现、Interface02/04/05 的 DTO/foreign output owner、Editor07 的 PIE 产品状态、Editor09 的 job infrastructure 或 Editor25 的 observation authority。
2. Editor 负责 gateway lease 消费、world/event projection、callback fault domain、Host lifecycle 与产品诊断接入；Runtime/Interface 负责 producer、ABI、page DTO 与 foreign output 基础合同。
3. Tooling 按用户要求排除；本轮没有查询、轮询、等待或实时跟踪协调器。

### 2.2 冻结点

| 项目 | 当前值 |
|---|---|
| 当前磁盘冻结时间 | `2026-08-27T14:30:18.6772158+08:00` |
| Git HEAD | `7fea65a3ae9cb836ad85adfdcece01ae7a6b7df1`，`2026-08-27T13:12:45+08:00` |
| 最近 production baseline | `982baa1ba87bc8c25fe44312507a4af15027e058`，`2026-08-27T12:53:32+08:00` |
| 审查方式 | 当前磁盘静态源码、测试 inventory、合同追踪、本地五套参考源码；不运行 Cargo 或动态产品测试 |

### 2.3 可复算 selected set

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | Fingerprint |
|---|---:|---|
| Zircon gateway/event/sync/play/Host/App/Interface/Runtime/tests | **978 / 120,277 / 110,435 / 4,331,065 / 1,042 / 37** | `ea865468417d20196fbfb2f54aba92cd9310b5ccdeb4900c1977372fbe8bd84f` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference | **19 / 10,464 / 8,934 / 367,062 / 11 / 0** | `1a6f3e93725bdd9f03cc9bfd28b2591a4eabdc15b21b36852ab7cf91e076d6b6` |
| 全部选择集 | **997 / 130,744 / 119,372 / 4,698,127 / 1,053 / 37** | `3a59918a8c52563256135e6a80b380aabe57995baac9b1e497d57a2c6244998f` |

Fingerprint 算法为：路径转 workspace-relative、小写并统一 `/`，逐文件 SHA-256，再对 `path + NUL + hash + LF` 清单做 SHA-256。Zircon scope 递归展开 frontmatter 中的 gateway、runtime event consumer、sync、play、Host、retained App、App entry/runtime session、Interface session/world、Runtime dynamic session 与 focused tests，统计的是当前磁盘物理文件而非旧 commit 快照。

## 3. 当前生产链事实

### 3.1 Identity 与 lease 已经是真实底座

`GatewaySessionIdentity` 不再只是 raw session handle；它携带 runtime instance/session、transport epoch、gateway generation、project 与 play instance。`EditorRuntimeGatewayHandle` 通过 `ArcSwap<GatewayGeneration>` 发布不可变 endpoint；`GatewayLease` 是单个有界调用链的短 lease，`GatewayOrigin` 是 watch/subscription 的长期 cleanup origin。`replace_for_play` 串行化 generation 发布并对 `u64` 溢出 fail closed。

局限是 handle 上仍保留 `watch_world`、`unwatch_world`、`drain_world_invalidations`、`subscribe_plugin_event`、`unsubscribe_plugin_event`、frame/event/output 等“每次重新取 lease”的便利方法。单个方法内部安全，不代表多个方法组成的逻辑事务同代。Host active tick 先从 play handle tick frame，再由 consumer host 自己取 lease；world、hierarchy、Inspector也各自取 lease，所以 P1-25 保持 Open。

### 3.2 World watch 已限定 origin，但没有 page commit

`QualifiedWatchToken` 保存 raw token 与完整 identity。注册用同一 lease发 watch，随后复查 current identity；若 replacement 插入中间，会通过旧 origin补偿 unwatch。stale unwatch不触碰新 endpoint；shutdown先清本地 token，再尝试远端 cleanup，并返回逐 watch typed receipt。pump也使用同一 lease drain，完成后复查 identity，stale result被丢弃。

但是 producer合同仍返回无 envelope 的 batch vector。没有 cursor/page sequence/remaining/final/oldest age/world revision，consumer只能拒绝 generation回退，识别不了同 generation的duplicate、gap或缺尾页。更严重的是 `WorldSyncPump` 对每个 fact 调 `EditorMessageBus::publish` 后不读取 dispatch disposition，即使 subscriber拒绝、背压或fault，也继续统计published并推进watermark。`synchronize_gateway_identity` 在replacement后清空watch和generation，没有自动rebind、baseline、resync或commit receipt。

### 3.3 Hierarchy 与 Inspector 只做到 identity-qualified query

Play hierarchy/Inspector在查询前取得identity，并通过 `query_world_at_identity` 做前后identity复查，projection也按identity分区。这阻止旧查询结果落入新session，是正确修复。它们与 `ensure_hierarchy_world_watch`、world invalidation pump并不共享一个 `QualifiedWorldSource`、world revision或commit receipt；hierarchy row与Inspector字段仍可能来自相同identity下不同world revision，因此 P1-26 只能 Partial。

### 3.4 Runtime event consumer 有 fault domain，但没有可恢复 delivery 协议

`ActiveConsumer` 保存 registration、`GatewayOrigin`、qualified subscription、local generation、last sequence、pending queue/bytes、runtime backlog observation与callback health。begin/consume/end都经过 `catch_unwind(AssertUnwindSafe)`；begin panic不会把active entry安装进去，end panic不阻断本地退休，consume panic生成有界fault receipt并隔离consumer。round-robin、每consumer事件上限、总时间预算、slow callback quarantine和共享retained-byte budget都是真实实现。

剩余断点：

1. sequence validation只拒绝 `<= last_sequence`，不识别 forward gap，也没有producer/session/schema之外的page cursor/final commit。
2. `Retryable` disposition存在于类型和测试容器，但执行路径没有backoff、max attempts、retry schedule或durable dead letter。
3. ordinary budget exit会恢复pending tail；callback panic则把current标为Poison并丢弃剩余tail，不满足P1-18。
4. deferred cleanup map只以`consumer_id`为key并使用`or_insert`；同consumer跨多次失败replacement可能同时留下多个remote subscription，但本地只能记一条。flush失败后不建立持续重试journal。
5. capability reconcile每tick clone registration/quarantine/disabled集合并做全量diff；没有registration/capability generation不变的O(1) fast path。

### 3.5 Replacement 是隐式退休/重订阅，不是状态机

consumer pump在drain前后比较current identity与active origin；不匹配时本地退休旧consumer，并通过旧origin尝试unsubscribe。下一帧reconcile发现desired consumer缺失后，才在新gateway重新subscribe。这个设计修复了raw handle复用污染，却留下replacement窗口：新endpoint发布到re-subscribe之前的event没有baseline或gap proof，会静默丢失。

World sync更直接：identity变化时清空本地watch，等待上层以后重新注册。两条链都没有统一 replacement ID、old/new identity、quiesce barrier、rebind集合、initial sync revision、retire结果和terminal receipt。P0因此是Partial而非Closed。

### 3.6 Host/App shutdown 有顺序，没有统一 terminal authority

`EditorRuntimeSessionShutdownReceipt` 已表达runtime consumer、play/edit world watch、play session、gateway detach与backend retirement的disposition；产品关停顺序也明确为consumer -> watch -> play -> detach -> retire。Project Close会检查play shutdown是否允许close，Retained Host只执行一次full shutdown并保存receipt。

仍有三个缺口：一般窗口退出路径保存receipt后继续释放owner，未证明receipt中的失败会阻止unsafe release；Host controller Drop只注销bus subscriber，而RuntimeSession Drop在destroy失败时直接abort，终态责任分裂；active pump与shutdown使用busy gate而非deadline/cancel/join协议，正在执行的callback不会被一个统一coordinator有界收敛。

### 3.7 Plugin registration 没有跨UI/runtime原子激活

当前链先prepare consumer registry、注册UI extension、再install prepared registry。prepare/install本身比旧实现可靠，但若已有active runtime，consumer真正subscribe发生在后续reconcile；activation失败不会回滚已可见UI contribution，也没有UI revision、consumer activation、capability与runtime identity的联合receipt。

### 3.8 Diagnostics 停在core API

consumer host可返回 `last_pump_report`、fault receipts、quarantined count/reason和backlog observation；world pump report也含drain identity/generation与stale drain计数。这些字段没有进入Retained Host的Runtime Diagnostics pane；该pane当前主要投影UI debug reflector/template数据。没有session/page/cursor/resync/shutdown timeline，也没有接入Editor25 observation authority，所以P1-37 Partial、P1-39 Open。

## 4. 参考引擎差异

### 4.1 Unreal：session/transport分离与显式bridge lifecycle

本轮选取的 `SessionManager`/`SessionInfo` 明确区分session GUID、instance GUID、owner、sender address、selection与last-update expiry；`MessageBridge`有显式Enable/Disable、transport Start/Stop、address-book removal、expired message drop、bus shutdown callback和析构清理。这证明Zircon应把logical session identity、transport node/address、selection与stale-peer retirement分开建模。所选Unreal文件不提供Zircon所需的world page commit协议，不能据此虚构Unreal已有同名cursor/receipt。

### 4.2 Godot：disconnect必须先成为本地终态

Godot debugger node区分最多四个debug session，连接可admit/refuse，stop时清理remote selection/history并发出stopped信号；script debugger析构时close/unref peer。它提供“远端不可用时本地仍必须完成stop”的产品下限，但不是generation-qualified lease或现代page/backpressure模型。

### 4.3 Fyrox：显式plugin lifecycle只是下限

Fyrox EditorPlugin有start/exit/sync/mode/scene/suspend/resume/update/message回调，container在回调期间take plugin slot，避免直接reentrant mutable aliasing；message通道仍是简单mpsc，callback无typed panic receipt、deadline、quarantine或page commit。Zircon当前fault boundary已超过这组局部下限，但跨session replacement协议仍未闭合。

### 4.4 Bevy Remote：bounded request channel与change cursor可借鉴

Bevy Remote使用JSON-RPC request ID/typed errors/custom method registry，异步request channel固定容量16；watch在RemoteLast执行、`try_send`并清理closed watcher，component watch复用change ticks/removal cursor。其event observer仍是 `Arc<Mutex<Vec<Value>>>`，没有完整session/generation/page identity。可借鉴bounded admission与cursor，不应复制unqualified buffer。

### 4.5 Unity Graphics：协议版本与Dispose是必要但不足的基线

RenderGraph remote debug有显式callback注册、Activate handshake、Dispose时unregister/destroy handler，并以固定protocol version拒绝不兼容payload。它仍是singleton/unqualified callback/unbounded dictionary，callback invoke无panic boundary，也没有generation、sequence、page、backpressure或reconnect receipt。Zircon应保留typed version/fail-visible语义，不应把这组debug helper当成工程终点。

## 5. Editor47 finding 重判

### 5.1 汇总

| 级别 | Open | Partial | Closed | 合计 |
|---|---:|---:|---:|---:|
| P0 | 0 | 1 | 0 | 1 |
| P1 | 11 | 27 | 6 | 44 |
| P2 | 12 | 0 | 0 | 12 |

### 5.2 P0

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| P0-01 A session/generation创建的watch/subscription不得被B drain/unsubscribe | Partial | qualified identity/origin、stale drain/unwatch/unsubscribe和raw值复用测试已封口直接污染；replacement仍无rebind/resync/gap proof/old retirement receipt，数据可在切换窗口丢失。 |

### 5.3 P1

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| P1-01 Gateway/Runtime/World/Subscription identity | Partial | Gateway identity完整；独立WorldIdentity、SubscriptionOrigin公共合同与world revision仍缺。 |
| P1-02 generation-qualified GatewayLease | Partial | 短lease与长期origin已存在；Host多步tick和跨projection事务仍未绑定同一lease。 |
| P1-03 token origin namespace | Partial | world/plugin本地wrapper已携完整identity；ABI opaque handle本身仍无namespace，旧便利API仍可误用。 |
| P1-04 owner/generation/request/sequence/receipt传播 | Partial | identity、subscription、sequence与部分receipt存在；world request/page/commit及deferred cleanup多subscription记录不完整。 |
| P1-05 World page cursor/sequence/remaining/final/age | Open | Interface仍返回无page envelope的`Vec<InvalidationBatch>`。 |
| P1-06 plugin page producer/session/schema | Partial | bounded page、sequence、remaining、oldest age、event/schema存在；producer/session identity未进入每页可验证envelope。 |
| P1-07 duplicate/gap/generation/stale识别 | Partial | stale identity与non-increasing sequence可拒绝；forward gap、missing final和same-generation page gap不可识别。 |
| P1-08 final page加bus commit后才推进watermark | Open | world pump逐fact发布后无条件推进；没有final/commit概念。 |
| P1-09 bus rejection/backpressure进入retry | Open | dispatch disposition未参与world sync状态；无page retry queue。 |
| P1-10 baseline/resync与increment绑定同一revision | Open | replacement清watch/重订阅，没有world baseline revision合同。 |
| P1-11 replacement quiesce/publish/rebind/resync/retire | Partial | ArcSwap publish与old-origin retirement存在；quiesce/rebind/resync/commit状态机缺失。 |
| P1-12 replacement receipt | Open | 无覆盖old/new identity、watch/subscription/output和顺序的统一receipt。 |
| P1-13 ActiveConsumer保存origin generation/session | Closed | `GatewayOrigin`与qualified subscription完整保存创建endpoint；按设计不长期保存短`GatewayLease`。 |
| P1-14 reconcile O(1) fast path | Open | 每tick仍clone并diff registrations/quarantine/disabled集合。 |
| P1-15 capability downgrade affected-only receipt | Partial | 可退休不再desired的consumer；仍全量扫描且无compatibility receipt。 |
| P1-16 callback panic boundary/classification | Closed | begin/consume/end均catch_unwind，phase/disposition/fault digest可记录。 |
| P1-17 begin panic不泄漏remote subscription | Partial | 立即cleanup、deferred cleanup和不安装active entry已实现；cleanup持续失败无durable重试保证。 |
| P1-18 consume panic隔离、保留tail、dead-letter | Partial | 隔离和有界fault receipt成立；remaining tail会被discard，非可恢复dead letter。 |
| P1-19 end panic仍完成本地retirement | Closed | active entry先移除，end panic被局部捕获并记录。 |
| P1-20 retryable/permanent/poison不同policy | Partial | poison/quarantine有policy；Retryable未进入实际调度/backoff/max-attempt路径。 |
| P1-21 slow consumer quarantine与公平性 | Closed | typed slow threshold、quarantine与round-robin/budget测试存在。 |
| P1-22 pending/dead-letter总字节预算 | Closed | pending tail与retained fault payload共享retained byte budget并有聚焦测试。 |
| P1-23 backlog lower-bound/unknown/age/sample | Closed | report精确表达known lower bound、sampled/unknown consumer、oldest和observation age。 |
| P1-24 deadline/cancel/shutdown typed | Partial | pump时间预算与shutdown disposition存在；callback无cancel token/deadline outcome/join receipt。 |
| P1-25 Host active tick单一lease | Open | frame tick、consumer pump、world pump、hierarchy和Inspector分别load当前generation。 |
| P1-26 hierarchy/world sync同qualified source | Partial | query前后identity校验成立；watch/query无同一world revision与commit source。 |
| P1-27 registration install/activation原子 | Partial | registry prepare/install可回滚局部状态；UI已可见与runtime subscribe没有联合commit。 |
| P1-28 transport loss本地Degraded/Stopped | Partial | terminal backend会本地退休并恢复Editor state；一般remote transport loss没有统一degraded state/receipt。 |
| P1-29 reconnect initial sync/gap/retry/backoff | Open | 只有下一帧隐式重订阅；无initial sync、gap proof、backoff或reconnect state。 |
| P1-30 shutdown coordinator顺序 | Partial | typed顺序与receipt已存在；failure gating、in-flight callback收敛和跨owner唯一authority未闭合。 |
| P1-31 Drop只做nonblocking fallback | Partial | Host Drop局部nonblocking；RuntimeSession Drop仍执行destroy并在失败时abort。 |
| P1-32 startup逆序回滚 | Partial | App product shutdown/部分attach失败路径有typed rollback；Host/UI/consumer/gateway全部提交阶段没有统一rollback journal。 |
| P1-33 play/project/PIE switch不误伤新session | Partial | reused raw session/subscription与replacement测试已覆盖核心identity；完整project/PIE/reconnect矩阵缺失。 |
| P1-34 output/frame先于destroy或fuse receipt | Partial | RuntimeFrame RAII、surface release、foreign output fuse存在；统一Editor shutdown receipt未证明所有borrowed output drain。 |
| P1-35 ABI/capability/page schema typed unavailable | Partial | V7 shape、capability、payload schema有typed validation；world page version与mixed ABI window不完整。 |
| P1-36 bus/projection dirty commit receipt | Open | publish结果不控制world watermark，projection无统一commit receipt。 |
| P1-37 diagnostics identities | Partial | core report/fault/quarantine/shutdown字段存在；未投影到产品Runtime Diagnostics。 |
| P1-38 Editor09 jobs承载reconcile/resync/shutdown | Open | gateway/event/world lifecycle仍在Host tick同步执行，未接job admission/cancel/progress。 |
| P1-39 接入Editor25 observation | Open | 没有canonical observation adapter；Retained diagnostics是独立UI reflector投影。 |
| P1-40 ABA/replacement/detach/panic并发测试 | Partial | raw值复用、mid-drain replacement、panic与busy gate有测试；完整ABA+detach+resync矩阵缺失。 |
| P1-41 multi-page/duplicate/gap/bus reject测试 | Partial | stale/duplicate sequence与generation regression有覆盖；world无page/final/gap/bus reject合同可测。 |
| P1-42 1K/10K规模与性能测试 | Partial | 1K/10K managed ABI tests存在但ignored，缺同构阈值、world scale和replacement长期预算。 |
| P1-43 correctness/performance/real-runtime lanes | Partial | focused correctness与ignored managed/real-runtime测试已分开；尚无统一qualification artifact和CI policy。 |
| P1-44 currentness recheck | Partial | 本报告已重查当前磁盘与五套参考；旧ABI/failure文档和产品exit/reconnect lane尚未全部绑定source fingerprint。 |

### 5.4 P2

| Finding | 状态 | 说明 |
|---|---|---|
| P2-01 multi-session archive/replay browser | Open | 先完成单session commit/resync。 |
| P2-02 world compression/dedup/CAS | Open | 先冻结page envelope和revision。 |
| P2-03 remote view interest management | Open | 先建立qualified source。 |
| P2-04 QoS/adaptive page | Open | 先有可观测backpressure。 |
| P2-05 durable audit/privacy | Open | 先建立低敏typed receipt。 |
| P2-06 mixed-version ABI window | Open | 当前只有固定V7 admission。 |
| P2-07 deterministic replay/time travel | Open | 先有完整commit journal。 |
| P2-08 GPU/IO zero-copy handoff | Open | 由Interface05/RHI owner先闭合安全合同。 |
| P2-09 multi-runtime fanout/relay | Open | 单runtime identity尚未全闭环。 |
| P2-10 long-session soak | Open | 无qualified soak artifact。 |
| P2-11 lifecycle visualization | Open | 产品diagnostics投影尚未接入。 |
| P2-12 超越参考引擎benchmark | Open | 无公平、可复现的同场景证据。 |

## 6. Canonical 资格门

| Gate | 状态 | 当前判定 |
|---|---|---|
| G01 完整session/generation identity | Partial | Gateway完整，World revision/source不完整。 |
| G02 opaque token必须origin-qualified | Partial | Editor wrappers已限定，ABI raw handle仍可能被错误便利API消费。 |
| G03 replacement无长锁且old/new隔离 | Partial | ArcSwap/short lease成立，跨步骤事务未隔离。 |
| G04 quiesce/publish/rebind/resync/retire receipt | Fail | 状态机与receipt不存在。 |
| G05 A-B-A stale资源不得误伤新session | Partial | 核心复用测试通过设计封口，完整并发矩阵未运行。 |
| G06 replacement/detach并发无泄漏 | Partial | 有局部测试，deferred cleanup仍可能漏记多个subscription。 |
| G07 world page envelope完整 | Fail | cursor/final/remaining/revision不存在。 |
| G08 duplicate/gap/stale识别 | Partial | stale与non-increasing成立，forward gap缺失。 |
| G09 final page加bus commit推进watermark | Fail | 当前无条件推进。 |
| G10 bus reject/backpressure可恢复 | Fail | disposition被忽略。 |
| G11 baseline与increment同revision | Fail | baseline/resync合同不存在。 |
| G12 hierarchy/Inspector/world sync同source | Partial | identity一致，revision/commit不一致。 |
| G13 begin panic零active leak | Partial | 本地成立，remote持续cleanup失败无保证。 |
| G14 consume panic保留tail/dead-letter | Partial | quarantine成立，tail保留不成立。 |
| G15 end panic本地terminal | Partial | 静态合同与测试存在，本轮未动态执行。 |
| G16 retry/permanent/poison矩阵 | Partial | poison成立，retry调度缺失。 |
| G17 slow consumer公平隔离 | Partial | 实现与测试存在，本轮未执行规模资格门。 |
| G18 deadline/cancel/backlog矩阵 | Partial | backlog完整，callback cancel/deadline缺失。 |
| G19 capability downgrade只影响目标consumer | Partial | 语义成立但仍全量扫描且无receipt。 |
| G20 transport loss本地terminal | Partial | backend terminal路径成立，一般transport loss不完整。 |
| G21 reconnect initial sync/gap/backoff | Fail | 不存在。 |
| G22 project/PIE switch identity安全 | Partial | 有核心测试，产品矩阵不完整。 |
| G23 output/frame release证明 | Partial | RAII/fuse存在，统一receipt不完整。 |
| G24 local terminal不等待remote ack | Partial | world/consumer局部成立，Host产品状态不统一。 |
| G25 explicit shutdown顺序 | Partial | 顺序和typed receipt存在，失败gating不完整。 |
| G26 startup逆序rollback | Partial | 局部rollback存在，无统一commit journal。 |
| G27 Drop仅nonblocking fallback | Partial | RuntimeSession Drop仍destroy/abort。 |
| G28 ABI/capability/schema mismatch typed | Partial | V7/plugin schema成立，world/mixed-version缺失。 |
| G29 diagnostics/jobs产品接入 | Fail | core数据未进入Editor25/Retained diagnostics/jobs。 |
| G30 ABA/detach/panic concurrency matrix | Partial | 只有focused子集。 |
| G31 1K/10K预算 | Partial | ignored tests有形状，无required qualification。 |
| G32 required/managed/real-runtime lanes | Partial | 文件分层存在，CI admission artifact缺失。 |
| G33 Windows ABI qualification | Partial | 现有ignored real-runtime lane不足以关闭。 |
| G34 产品exit/reconnect E2E | Fail | reconnect协议本身不存在。 |
| G35 source currentness/fingerprint | Partial | 本轮有当前磁盘fingerprint，旧依赖报告仍需持续重查。 |
| G36 同场景超过参考引擎证据 | Fail | 无同硬件/构建/负载的性能与可靠性证据。 |

## 7. 目标架构与 Hard Cutover

```text
App RuntimeSessionOwner
  -> GatewaySessionIdentity + GatewayGeneration
  -> QualifiedGatewayLease(tick/replacement/shutdown deadline)
  -> prepare watch/subscription set
  -> initial baseline(WorldRevision)
  -> paged incremental(cursor/sequence/final/remaining/age)
  -> validate -> projection/bus commit receipt -> watermark
  -> diagnostics/observation journal

replacement:
  quiesce old -> publish new -> rebind -> baseline/resync -> commit -> retire old

shutdown:
  cancel/join callbacks -> end consumers -> unwatch -> release output/frame
  -> detach gateway -> destroy session -> terminal receipt
```

Hard cutover要求：

1. 删除或收窄跨步骤可误用的raw convenience API；long-lived资源只保存`GatewayOrigin`，逻辑事务只通过显式`QualifiedGatewayLease`。
2. `WorldInvalidationPageV1`与plugin event page统一owner/session/generation/schema/request/cursor/sequence/remaining/final/oldest-age envelope。
3. watermark只消费projection/bus terminal commit receipt；任何reject/backpressure/fault都保留page并进入有界retry/dead-letter。
4. replacement、reconnect与shutdown共享一个session coordinator和durable-enough operation journal，不再由world/consumer/Host各自推断终态。
5. diagnostics只向Editor25 canonical observation发布低敏snapshot；Retained UI只投影provider snapshot，不直接读取内部Mutex和拼装第二authority。

## 8. 分层重构计划

| Milestone | 内容 | 退出条件 |
|---|---|---|
| M0 | 安全封口测试 | A-B-A、raw值复用、replacement中detach、多个deferred cleanup、bus reject先写RED tests。 |
| M1 | Identity/lease收敛 | 冻结WorldIdentity、QualifiedWorldSource、SubscriptionOrigin；移除跨步raw API。 |
| M2 | Page/commit协议 | World/plugin page envelope、cursor/final/gap、bus/projection commit和watermark完成。 |
| M3 | Callback delivery状态机 | retry/backoff/max-attempt/dead-letter、tail保留、deadline/cancel/join完成。 |
| M4 | Replacement/reconnect | quiesce/publish/rebind/baseline/resync/retire及完整receipt完成。 |
| M5 | Host/App terminal coordinator | tick单lease、registration activation、shutdown/rollback/Drop边界统一。 |
| M6 | Diagnostics/jobs | 接Editor25 observation与Editor09 jobs，Retained diagnostics显示真实session/page/backlog/quarantine/resync。 |
| M7 | 规模与故障资格 | required correctness、1K/10K、real-runtime、Windows ABI、exit/reconnect E2E进入明确lane。 |
| M8 | 超越性验证 | 固定硬件/构建/场景与参考实现对比延迟、吞吐、丢失率、恢复时间、内存和长会话稳定性。 |

## 9. 逐 owner 检查台账

| Owner/文件簇 | 已检查的真实实现 | 仍需重构 |
|---|---|---|
| `zircon_runtime_interface/runtime_api/session` | 完整Gateway identity、plugin subscription/page DTO、payload schema | WorldIdentity、producer identity、统一page/commit envelope |
| `zircon_runtime_interface/world_sync` | watch registration/token/query/batch合同 | cursor/final/gap/revision/baseline/resync/version |
| `zircon_editor/core/gateway` | ArcSwap generation、short lease、origin、identity-checked world query、replacement tests | 收窄raw convenience API、qualified tick transaction、replacement receipt |
| `zircon_editor/core/sync` | qualified watch、mid-registration compensation、stale drain discard、indexed watch map、shutdown receipt | page validator、bus disposition、watermark commit、rebind/resync |
| `zircon_editor/core/runtime_event_consumer` | lifecycle gate、panic boundary、round-robin、budgets、fault journal、quarantine、backlog | gap、retry/dead-letter、tail preservation、cleanup multimap/journal、O(1) reconcile |
| `zircon_editor/core/play` | play identity/backend/controller与session切换基础 | replacement/reconnect coordinator和terminal product state |
| `zircon_editor/ui/host` | identity-qualified hierarchy/Inspector、typed terminal detach/shutdown paths | 同tick单lease、同revision projection、callback join、registration activation receipt |
| `zircon_editor/ui/retained_host/app` | project-close guard、once-only runtime shutdown receipt、startup assembly | 一般exit failure gate、逆序rollback journal、真实diagnostics projection |
| `zircon_app/entry`与`runtime_library/runtime_session` | App-owned session、surface/output release、try_destroy/fuse、product shutdown receipt | 与Editor唯一terminal authority、Drop职责、所有output completion证明 |
| `zircon_runtime/dynamic_api/session` | bounded plugin event producer、world watch/query bridge | page identity/revision、baseline/resync producer与commit-aware drain |
| focused tests | token/subscription复用、mid-drain replacement、panic/slow/budget/round-robin、ignored 1K/10K | same-generation gap、bus reject、multi-cleanup、full ABA、reconnect、exit E2E、soak |
| 五套reference | Unreal session/bridge、Godot disconnect、Fyrox lifecycle、Bevy bounded watch、Unity version/dispose | 只提取边界与验证方法，不复制其较弱的unqualified buffer/singleton设计 |

## 10. 完成定义与本轮 closeout

本轮只完成review与重构计划，不修改生产代码。没有运行Cargo、Editor、Runtime DLL、PIE、replacement/reconnect、callback fault注入、bus rejection、Windows ABI、1K/10K、soak或跨引擎benchmark，因此所有依赖动态证据的gate最多为Partial。Editor47只有在P0关闭、44项P1逐项有实现/测试/产品接入证据、36门全部Pass且同场景基准可复现后，才允许宣称该链达到工程级；“优于Unreal”还必须额外证明性能、表现、稳定性与恢复能力，而不是由架构文档推断。

下一实现顺序必须从M0-M2开始：先把world page和commit语义做成不可绕过的合同，再做replacement/reconnect；如果先补UI或增加更多watch/event功能，只会扩大当前不可恢复的数据窗口。
