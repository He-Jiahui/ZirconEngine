---
title: Runtime Scene Event Mirror、Registration、Subscription、Cursor、Backlog、Overflow、Reclaim、ABI、Consumer 与 Product Integration Current Source Review
category: zircon_runtime
report_id: Runtime119
review_date: 2026-08-23
baseline_head: 1354e50da53db3dad1dc25a6c9e375942ba04d35
baseline_epoch: 367
supersedes:
  - docs/plans/optimize/zircon_runtime/54-runtime-scene-event-mirror-registration-subscription-cursor-backlog-overflow-reclaim-abi-consumer-product-integration-review.md
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
  - zircon_runtime/src/scene/event_mirror/subscription/reclaim_queue_tests.rs
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
  - docs/plans/mvp/index.md
  - docs/plans/optimize/zircon_runtime/54-runtime-scene-event-mirror-registration-subscription-cursor-backlog-overflow-reclaim-abi-consumer-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_editor/47-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md
  - docs/plans/zircon_runtime/runtime/10/failure-2026-07-19-plugin-event-bounded-delivery.md
  - docs/plans/zircon_plugins/01/failure-2026-07-22-plugin-event-drain-frame-budget.md
  - docs/plans/zircon_plugins/12/failure-2026-07-22-runtime-event-mirror-drop-lifecycle.md
  - docs/plans/zircon_editor/editor/02/failure-2026-07-17-runtime-event-consumer-unbounded-pump-lock.md
reference_engines:
  - dev/bevy/crates/bevy_ecs/src/message
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging
  - dev/godot/core/object
  - dev/godot/tests/core/object/test_object.cpp
  - dev/Fyrox/fyrox-resource/src/event.rs
  - dev/Fyrox/fyrox-resource/src/manager.rs
  - dev/Fyrox/fyrox-core/src/pool
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Debugging/DebugWindow.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99t · Runtime Scene Event Mirror Current Source Review

## 1. 结论

当前 Scene Event Mirror 已有一组必须保留的真实底座：每订阅队列固定为 `16K events / 64 MiB`，单页最多 `64 events / 128 KiB payload`，ABI wire ceiling 为 `256 KiB`；队列在发送边界拒绝超大、过深或超时的 JSON，Session 以 pending page 隔开 scene dequeue 与 foreign allocation，空页不分配；subscription 已改成 World-owned 代际 slot，Drop 只提交去重 reclaim intent，显式 unsubscribe、Drop、WorldDriver tick 和 Session teardown 共享回收状态；Editor host 具备单页 pending、round-robin、全局/per-consumer/time budget、锁外 callback、页 bytes/remaining/oldest-age 与 drain/decode 时间观测。Runtime54 之后，回收队列还从 `VecDeque::retain` 的 O(N²) 批量删除改为 handle-indexed linked FIFO，Navigation producer 也会在 `NavigationDebugCapture.enabled == false` 时跳过 overlay 构造和发送。

但它仍不是工程级事件流。三个 P0 都未修复：AI Editor 发布三个 Runtime consumer，而 AI Runtime 仍只用 SDK `event()` 注册普通 ECS event；Runtime 在 foreign allocation 成功时就永久提交页，Editor decode、sequence 校验和 typed callback 都发生在提交之后，panic 测试还明确证明当前 delivery 会丢失；每个 subscriber 都安装独立 ECS observer、在 producer 线程重复 JSON encode 并持有独立 64 MiB queue，使 CPU 与内存上界按完整 payload 线性放大。固定页只限制一次 drain 的形状，不提供 ack、credit、gap、dropped range、provider generation、checkpoint 或 resync。

当前账本为 **3 P0 Open、60 P1 Open、15 P2 Open、1 P2 Partial、39 Gate Fail、1 Gate Partial**。唯一 P2 Partial 是 `SEMR-P2-006`：索引回收源码、行为测试和 ignored release benchmark 已存在，但本轮没有受管动态资格。唯一 Gate Partial 是 `SEMR-G04`：Navigation 源码已跳过无 reader 的 overlay 构造，测试只覆盖 reader-count 开关和一次正向 delivery，尚未证明“下一帧构造/发送/CPU/RSS 为零”。四份相关 failure 仍是 `open`。

目标不是继续向 per-subscription queue 堆字段，而是硬切为 `PluginEventExposureContract -> SceneEventBroker -> SharedEncodedStream -> AcknowledgedConsumerCursor -> ABI PageLease -> Editor ConsumerTransaction`。同一 publication 只编码一次；订阅者只拥有 cursor、credit、ack 与 lag；overflow 产生精确丢失区间和 typed recovery disposition；allocation、decode、apply 与 handled 分阶段记录；AI 与 Navigation 从同一个产品 contract 生成 Runtime exposure、catalog 和 Editor admission。

本轮只做 current-source 静态审查与文档记录，没有修改 production、tests、Cargo、ABI 或参考源码，没有运行 Cargo、真实 Editor/Runtime、1K/10K ignored benchmark、multi-subscriber storm、reload、fault injection、RSS 或 profiler。按用户范围，本篇不展开 tooling 优化，也不宣称当前性能或表现达到、超过 Unreal。

## 2. 审查边界与物理冻结

| 范围 | 文件 / 行 / bytes / tests / dirty | fingerprint |
|---|---:|---|
| Runtime core、World、Session owner | 10 / 4,104 / 149,922 / 30 / 3 | `21130781b4eec32a719c3692330055c411c0f10f51011fa92770cd3bf93907dd` |
| Interface、Gateway、Editor consumer | 10 / 1,833 / 64,023 / 2 / 0 | `d8bcad6c9787a58826fc1c86ce0f001dd636c543b86872543c2278a487fbd9e7` |
| SDK、AI、Navigation 产品链 | 5 / 1,767 / 66,016 / 3 / 3 | `eb68459c0cc84b1f8a1baee5c009ded21b1590ebc2fe9c103aaf3a2bdce2ebba` |
| focused direct tests | 12 / 5,509 / 195,479 / 102 / 0 | `98bcaff575ac44ca73667da4f7032ebfa35c02d6c4309ed5c683c1b47c244d29`；3 ignored |
| 五引擎参考实现与测试 | 36 / 16,251 / 546,486 / 95 / 0 | `8a4600e4781b454992bde9e302ed18a74996f676f576606baee995b679e4815a` |

fingerprint 算法：仓库相对路径转 `/`、排序去重，以 `path|lowercase per-file SHA-256` 逐行编码，LF 连接且末尾无 LF，再计算 UTF-8 SHA-256。它冻结本轮实际读取集合，不是 stream、provider、schema、World、Session 或 BuildSet generation。

coordinator 在 `d739bebb65dec1bef87c3be64820fe75ba0ba9b7` / epoch 367 注册；共享 main 在审查期间前进到本表 `baseline_head`，因此 working-tree fingerprint 是本轮源码事实的最终权威。六个 production 文件已有其他会话/用户改动：`plugin_sdk/src/registration.rs`、AI/Navigation runtime registration、Session `ffi.rs`、mirror `subscription.rs`、`world_driver.rs`。本文读取并审查其结果，但不覆盖、不归属这些改动。实施前必须按 fingerprint 重新读取，所以 `source_recheck_required` 为 true。

MVP `00` 仍为 `in_progress`，F0-F5 按严格依赖保持 blocked。本篇是高级子系统审查，不把任何 source-local progress 写成 MVP milestone accepted。

## 3. 当前真实链路

```text
RuntimeExtensionRegistry::register_mirrored_event<E>()
  -> World::register_runtime_event_mirror()
     -> subscribe: one typed ECS observer + one JSON queue per subscriber
        -> World::send_event(E)
           -> observers serialize E independently under queue mutexes
           -> RuntimeDynamicSession::prepare_plugin_event_output()
              -> pop scene queue -> Session pending page -> JSON ABI page
              -> foreign allocation success => commit page + local sequence
                 -> Editor SessionGateway decode/free
                    -> EditorRuntimeEventConsumerHost pending page
                       -> monotonic-only validation -> typed callback
```

`send_event` 把 event 写入普通 ECS store，并把所有 observer 的 bool 结果聚合。一个 subscriber overflow、另一个成功时会发生部分接受；失败 cursor 没有 sequence 或 dropped range。Dynamic Session 的 sequence 在 drain 时才分配，foreign allocation 成功即从 Runtime authority 删除 page；Gateway 解码和 Editor apply 都发生在提交后。Host 只拒绝 `sequence <= last`，不验证连续性；callback 前先 pop，Err/panic 都永久消费当前 delivery，只恢复尾部与最后成功 sequence。现有 panic 回归随后因 stale sequence 再丢一条，正是协议缺口证据。

AI 产品链在 exposure 入口前断开。SDK `RuntimePluginRegistrationBuilder::event<E>()` 只调用普通 `register_event`；AI Runtime 对 `BtNodeResultEvent`、`AiBehaviorDebugSnapshot` 使用该路径并每 tick 发送，AI Editor 却声明 debug snapshot、node result 和 snapshot-prune 三个 Runtime consumer。AI Editor 测试只覆盖 manifest/state，没有真实 Runtime ABI。Navigation overlay 仍是唯一直接使用低层 `register_mirrored_event` 的 first-party 产品事件。

## 4. Runtime54 后的 current-source 裁决

| 项目 | 当前证据 | 状态裁决 |
|---|---|---|
| fixed page / bounded queue | 10K backlog 跨 `World::update_events` 分页保持顺序；overflow 显式且保留已接受事件；idle 返回 empty buffer | 保留底座；不等于 ack、global budget 或 resync |
| Drop lifecycle | World-owned generational record、deduplicated reclaim、callback rollback、shutdown retry 与 Session closing owner 已进入源码和测试 | failure 仍 open；paused World wake、provider revoke、最终动态资格缺失 |
| reclaim queue complexity | `HashMap` 邻接索引替代 `pending.retain`，行为测试证明 survivor 顺序，ignored benchmark 对比 4,096/2,048 handles | `SEMR-P2-006 Partial`，不是 accepted performance closure |
| Navigation producer idle | `NavigationDebugCapture.enabled == false` 时不构造、不发送 overlay；reader-count 测试验证 0→1→0 | `SEMR-G04 Partial`；缺 next-frame zero-work trace、RSS/CPU 与 reload 交错 |
| Editor bounded pump | 单页 pending、公平/time/count budget、锁外 callback、backlog/bytes/timing report 存在 | 保留底座；当前 delivery 在 Err/panic 时仍丢失，真实 ABI 1K/10K 仍 ignored |
| P0 产品/协议/fanout | AI exposure 断链、allocation-before-decode commit、per-subscriber encode/queue 均仍存在 | 3 项全部 Open |

## 5. 五套参考实现裁决

| 参考 | 已读取的直接事实 | 对 Zircon 的约束 |
|---|---|---|
| Bevy `Messages` / `MessageCursor` / registry | typed message 在双 buffer 中只存一次；每 reader 只有 `last_message_count`；iterator 只在实际 `next/nth/count` 时推进；`missed_messages()` 可查询 retention hole；registry 缓存 component id 并在有变化或需要清空第二 buffer 时更新 | Zircon 可采用更强的跨 DLL retention，但必须使用 shared stream + independent cursor，并把 missed range 显式放进协议；不能每订阅复制 payload 或在 drain 前推进权威游标 |
| Unreal Messaging | `FMessageRouter` 用 MPSC command queue 串行订阅/路由变更；一个 shared `IMessageContext` 携 sender、recipients、scope、flags、time、expiration、annotations、attachment 和 forwarding identity；subscription 有 scope/enabled/weak receiver；按 recipient thread 直调或 TaskGraph dispatch；tracer 区分 sent/routed/dispatched/handled、endpoint、latency 与 pending | contract 必须有 provider/scope/QoS/dispatch identity；shared envelope 不能按 consumer 复制；allocation 不是 handled；生命周期、interceptor、routing 和处理延迟必须可观测。Unreal 此模块本身也不提供 Zircon 所需的持久 ack log，不能盲目复制 |
| Godot Object/MessageQueue | signal 双向登记 source slot 与 target incoming connection；析构同时清理 outgoing/incoming；emit 在锁内复制 callable/flags 快照、锁外执行，one-shot 先断开防递归；deferred queue 用 4 KiB page、可配置 max pages、超限显式 OOM，flush 在 callback 前预推进并释放锁以支持重入 | Zircon 的 callback snapshot、reentrancy、automatic disconnect 与 hard byte budget 必须成为状态机。Godot deferred queue 同样是 at-most-once callback，不是跨 DLL reliable stream，不能为当前无 ack 语义背书 |
| Fyrox broadcaster / generational pool | broadcaster 用 `Pool<Sender>` 返回 index+generation handle，显式 remove，send 失败时 retain 清理 dead receiver；资源 manager 在 added/loaded/reloaded/removed 的真实 owner 点广播；事件仍按 subscriber clone | 代际 handle 与 owner-point publication 可保留；逐 subscriber clone 只适合小型进程内通知，是 Zircon 高吞吐镜像不得停留的反例 |
| Unity Graphics Rendering Debugger | `DebugManager` 统一 panel/data register/unregister、dirty/refresh/reset；UI 打开后才 lazy 初始化；window close 解除 callback；状态跨 domain reload 保存；scheduler tracker 暂停 inactive panel/foldout；runtime UI 可完全禁用以避免 debug object/init overhead；测试覆盖 window state callback | first-party debug producer 必须由真实 UI/consumer demand 激活，关闭后停掉 scheduler/producer；registration、state persistence 与 reload 要幂等。Unity UI lifecycle 不是传输可靠性协议，仍需 Zircon cursor/ack/resync |

共同下限是：payload/context 形成一次权威表示，subscription 有 owner、scope 和生命周期，callback 不在注册容器锁内执行，队列有 hard budget，route 与 handled 分阶段观测，丢失或过期可检测。要超过这些参考，必须在同场景同硬件下证明 shared fanout、ack/resync、跨 DLL schema generation 和 global memory envelope，而不是只减少几次 clone。

## 6. Owner 边界

| Owner | 继续拥有 | Runtime119 登记的纵向边界 |
|---|---|---|
| Runtime02 / Runtime05 | 通用 ECS event/message/observer、World schedule 与生命周期 | typed event 到跨 DLL mirror 的 broker/cursor/recovery，不复制所有 ECS 问题 |
| Runtime43 / Interface01/07 | Dynamic Session、foreign allocation、API table、ABI 认证 | event page lease、ack/resync/request budget、schema/provider identity 与 commit 语义 |
| Editor47 / Editor02 | Gateway decode、consumer host、公平泵、Editor reconnect | typed apply 成功后的提交、失败重放、shared Editor fanout |
| Plugins01 / Plugins12 | plugin exposure contract、bounded delivery、drop/reload lifecycle | SDK mirrored builder、provider generation、AI/Navigation 产品接线 |
| Plugins14 / Plugins15 | Navigation 与 AI 业务状态、debug snapshot、consumer view | 真实产品 ABI 闭环，不复制 Nav/AI 算法差距 |
| Runtime119 | registration、publication、cursor、backlog、overflow、reclaim、ABI、Editor apply 的完整纵切面 | 本报告 3/60/16 账本与 40 个 gate |

## 7. P0 阻断

| ID | 状态 | 差距与硬切目标 |
|---|---|---|
| SEMR-P0-001 | Open | AI Editor 发布三个不可订阅的 Runtime consumer。建立唯一 `PluginEventExposureContract`，由 Runtime registration、package catalog 与 Editor manifest 共同生成；真实 dynamic session + gateway 测试证明 subscribe/deliver/disable/unload。 |
| SEMR-P0-002 | Open | Runtime commit 早于 decode/apply，gap 不可检测，Err/panic 消费当前 delivery。建立 acknowledged cursor：wire 携 stream/provider generation、contiguous range、dropped range/checkpoint；decode+typed apply 后 ack，失败 retry/quarantine/resync。 |
| SEMR-P0-003 | Open | 每 subscriber 独立 observer、JSON encode、mutex queue 和 64 MiB budget，成本为 `O(subscribers * payload)`。改为一次 encode 的 shared immutable segment log、cursor-only subscriber、global/stream/provider/consumer credit 与 admission。 |

## 8. P1 Contract、Registration 与 Exposure

| ID | 状态 | 需要重构的内容 |
|---|---|---|
| SEMR-P1-001 | Open | descriptor 从 `event_id + payload_schema` 扩为 stable contract id、provider/build generation、schema id/version/digest、scope 与 delivery class。 |
| SEMR-P1-002 | Open | 自由 schema 字符串替换为可查询 artifact、兼容规则、migration 与 resync 能力。 |
| SEMR-P1-003 | Open | event namespace 绑定 plugin/provider owner，卸载后不得被另一 provider 静默接管。 |
| SEMR-P1-004 | Open | contract 显式声明 lossless/latest/keyed-coalesced/bounded、retention、overflow 与 recovery。 |
| SEMR-P1-005 | Open | subscription 限定 RuntimeSession、World/Level 和可选 entity/domain scope。 |
| SEMR-P1-006 | Open | SDK 提供唯一 typed mirrored exposure builder，禁止插件绕过 owner/schema/policy。 |
| SEMR-P1-007 | Open | package catalog 表达 exposure、target、producer capability、availability 与 provider generation。 |
| SEMR-P1-008 | Open | plugin activation preflight 原子验证 TypeId/catalog id/World id/schema/provider，禁止部分发布。 |
| SEMR-P1-009 | Open | World registration 增加 unregister/revoke/drain，reload 终结或迁移旧 cursor。 |
| SEMR-P1-010 | Open | 删除 registry Clone 重置 live state与恒 true `PartialEq`，定义 staging/snapshot 差异。 |

## 9. P1 Publication、Fanout 与 Backpressure

| ID | 状态 | 需要重构的内容 |
|---|---|---|
| SEMR-P1-011 | Open | 每 event contract 只安装一个 broker adapter，subscriber 只注册 cursor/filter。 |
| SEMR-P1-012 | Open | 同一 publication 只编码一次 shared envelope。 |
| SEMR-P1-013 | Open | segment/page 以 lease/Arc 等价共享，cursor 只保存 offset/range。 |
| SEMR-P1-014 | Open | 增加 World、Session、provider、stream、consumer 多层 entry/byte hard budget。 |
| SEMR-P1-015 | Open | producer fast path 不跨 subscriber 锁或执行 S 次 serializer。 |
| SEMR-P1-016 | Open | serialization deadline 属于一次 publication，并纳入 frame admission。 |
| SEMR-P1-017 | Open | failure 从 first-error slot 升级为 typed counters、range、stage 与 retryability。 |
| SEMR-P1-018 | Open | `send_event` 聚合 bool 改为 publish receipt 与逐 cursor disposition。 |
| SEMR-P1-019 | Open | snapshot/debug stream 使用 latest/keyed coalesce，edge stream 保序且独立 retention。 |
| SEMR-P1-020 | Open | budget 从 validated profile/contract 产生，并返回 effective policy generation。 |

## 10. P1 Sequence、Commit、Retry 与 Resync

| ID | 状态 | 需要重构的内容 |
|---|---|---|
| SEMR-P1-021 | Open | sequence 在 publish admission 时由 producer stream 分配，并与 provider/stream generation 组成 identity。 |
| SEMR-P1-022 | Open | rejected attempt 有显式 disposition；不得无 sequence、无 gap 地消失。 |
| SEMR-P1-023 | Open | Editor 验证 `first == acked + 1`、页内连续与 range header。 |
| SEMR-P1-024 | Open | shared log 在 required cursor ack 或 retention retirement 前保留 authority。 |
| SEMR-P1-025 | Open | Session commit 拆为 wire lease、decode、apply、consumer ack 多阶段。 |
| SEMR-P1-026 | Open | 新 API table 提供 ack/nack/checkpoint 与幂等 retry；旧 drain hard cut。 |
| SEMR-P1-027 | Open | typed callback Err 保留当前 delivery，按 policy retry/dead-letter/disable/resync。 |
| SEMR-P1-028 | Open | panic boundary 保留当前 identity并将 consumer 置 faulted。 |
| SEMR-P1-029 | Open | dead-letter 记录 digest、contract/provider、attempt、failure、operator disposition、retention。 |
| SEMR-P1-030 | Open | state/snapshot class 注册 authoritative snapshot generation 与 rebuild API。 |

## 11. P1 Subscription、Reclaim 与 Lifecycle

| ID | 状态 | 需要重构的内容 |
|---|---|---|
| SEMR-P1-031 | Open | handle 绑定 provider/stream generation，revoke 返回 typed terminal state。 |
| SEMR-P1-032 | Open | ABI u64 变为 session-qualified opaque generation handle，拒绝跨 session/reload stale。 |
| SEMR-P1-033 | Open | Session handle-space exhaustion 有 rotation/renewal/diagnostic，而非只报错。 |
| SEMR-P1-034 | Open | slot generation exhaustion typed retirement 并计数，禁止静默损失容量。 |
| SEMR-P1-035 | Open | Drop intent 触发 bounded wake；paused/non-ticking World 也及时关闭按需 producer。 |
| SEMR-P1-036 | Open | `World::drop` 不得忽略 callback failure；teardown 有 quiesce 与 forced disposition。 |
| SEMR-P1-037 | Open | reader-count callback 收敛为幂等 broker activation lease，声明重入/rollback/shutdown。 |
| SEMR-P1-038 | Open | per-handle lifecycle 保存 state/retry/error/age，并聚合到 stream/provider/session。 |
| SEMR-P1-039 | Open | registration 缺失不再 `expect`，使用 revoking/tombstone/corruption state。 |
| SEMR-P1-040 | Open | 建立 subscribe/activate/drop/disconnect/ack/revoke/resync/terminal journal。 |

## 12. P1 ABI、Gateway 与 Editor Consumer

| ID | 状态 | 需要重构的内容 |
|---|---|---|
| SEMR-P1-041 | Open | page header 持稳定 descriptor/range，record 只存 sequence delta、payload slice 与必要 metadata。 |
| SEMR-P1-042 | Open | 协商 binary/raw JSON codec、compression、zero-copy lease；JSON 不再是唯一热路径。 |
| SEMR-P1-043 | Open | drain request 携 count/bytes/deadline/credit，并在 Runtime 编码前生效。 |
| SEMR-P1-044 | Open | wire 分离 RuntimeSessionId、PlayInstanceId、WorldId 与 provider generation。 |
| SEMR-P1-045 | Open | Gateway decode 失败前不得 ack；协议失败保留 lease 或进入 typed resync。 |
| SEMR-P1-046 | Open | Gateway 在大分配/Host 前验证 descriptor generation、range、continuity、bytes、resync。 |
| SEMR-P1-047 | Open | compatible Editor consumer 共享 Runtime cursor，再在 Editor 内有界 fanout。 |
| SEMR-P1-048 | Open | begin/end callback 返回 typed outcome，panic 隔离，生命周期原子 rollback/retry。 |
| SEMR-P1-049 | Open | capability reconcile prepared-set 原子切换，新 set 未 ready 前保留 last-known-good。 |
| SEMR-P1-050 | Open | Editor consumer generation checked allocation，exhaustion 后 session renewal，禁止 `u64::MAX` 碰撞。 |

## 13. P1 Diagnostics、测试与资格

| ID | 状态 | 需要重构的内容 |
|---|---|---|
| SEMR-P1-051 | Open | 持久指标覆盖 publish/encode/enqueue/ack/drop/resync、depth/bytes/lag/oldest、lock wait、CPU。 |
| SEMR-P1-052 | Open | ABI 报告 shared/consumer retained bytes、credit、high-water、observation generation。 |
| SEMR-P1-053 | Open | p95/p99 使用跨帧 bounded histogram/window，绑定 sample count 与 reset reason。 |
| SEMR-P1-054 | Open | envelope 加 frame/time/World/provider/operation trace context。 |
| SEMR-P1-055 | Open | 快慢 multi-subscriber 部分 overflow 测试证明 receipt、gap、ack、resync。 |
| SEMR-P1-056 | Open | AI 使用真实 registration、dynamic session、gateway、typed consumer，不准 manifest-only。 |
| SEMR-P1-057 | Open | 两项 1K/10K 真实 ABI storm 从 ignored 进入 required Windows lane。 |
| SEMR-P1-058 | Open | 1/64/1K subscriber + 64 MiB 压力矩阵记录 CPU/allocation/shared bytes/lag/RSS。 |
| SEMR-P1-059 | Open | invalid JSON/schema、callback Err/panic、allocation/free、ack loss/duplicate fault injection。 |
| SEMR-P1-060 | Open | provider reload/crash/reconnect/paused World/multi-World soak。 |

## 14. P2 可维护性与证据质量

| ID | 状态 | 后续处理 |
|---|---|---|
| SEMR-P2-001 | Open | 修正“producer boundary 只序列化一次”的失真注释；shared broker 后再声明 once-per-publication。 |
| SEMR-P2-002 | Open | Runtime/Interface 分散常量收敛为 versioned policy manifest 与编译期一致性检查。 |
| SEMR-P2-003 | Open | descriptor String intern 并由 generation handle 引用。 |
| SEMR-P2-004 | Open | admission 后使用 typed StreamId，hot path 不做 String BTreeMap。 |
| SEMR-P2-005 | Open | shutdown 枚举改为 bounded intrusive/work queue 或分块迭代。 |
| SEMR-P2-006 | Partial | handle-indexed linked FIFO 已消除 retire `retain` 扫描并保持 survivor 顺序；需运行 ignored release benchmark、受管 Windows qualification 与真实 lifecycle workload。 |
| SEMR-P2-007 | Open | scene-facing `drain()` 的 Value decode 限定 debug/legacy，产品迁移 raw/typed page。 |
| SEMR-P2-008 | Open | idle drain 提供无状态 cursor observation，减少稳态写锁/状态翻转。 |
| SEMR-P2-009 | Open | lifecycle `assert/expect` 转 typed corruption/revoke/forced teardown。 |
| SEMR-P2-010 | Open | queue/failure/page 提供 sanitized telemetry snapshot，不打印完整 payload。 |
| SEMR-P2-011 | Open | FFI 保留 stable error code、stage、generation、retryability，不退化为字符串。 |
| SEMR-P2-012 | Open | 文档公开 total memory formula、shared/private bytes 与 scale envelope。 |
| SEMR-P2-013 | Open | `include_str!` 源码形状守卫迁移为 behavior/fault/ABI compatibility tests。 |
| SEMR-P2-014 | Open | 721 行 Session event owner 与 785 行 Editor host 按 protocol/lease/transaction/diagnostic 拆分。 |
| SEMR-P2-015 | Open | public Scene API 收敛为 descriptor/policy/cursor/page lease/ack/outcome。 |
| SEMR-P2-016 | Open | benchmark artifact 绑定 BuildSet、HEAD、fingerprint、hardware、profile、workload、threshold。 |

## 15. 目标架构

```text
PluginEventExposureContract
  {ContractId, ProviderGeneration, SchemaDigest, Scope, DeliveryClass, Budgets}
        |
        v
SceneEventBroker (one typed adapter per contract)
  publish -> encode once -> SharedEncodedStream
                         {StreamGeneration, SequenceRange, Segments, GlobalBudget}
                                      |
                   +------------------+------------------+
                   v                                     v
          ConsumerCursor A                         ConsumerCursor B
          {ack, credit, lag}                       {ack, credit, lag}
                   |                                     |
                   +---------- ABI PageLease ------------+
                                      |
                         Editor ConsumerTransaction
                    validate -> decode -> apply -> ACK
                                      |
                       retry / quarantine / resync snapshot
```

`DeliveryClass` 至少区分 lossless edge、bounded telemetry、latest snapshot 与 keyed coalesced state。只有注册 authoritative snapshot provider 的 class 才能在 gap 后自动 resync；不可恢复的 lossless edge 突破 retention 必须 fail closed。segment retirement 由 required cursor ack、retention、provider revoke 与 global budget 共同决定，不能由某次 foreign allocation 单独决定。

## 16. 重构里程碑

### M0 · Characterization 与产品 Gate

- 添加 AI 真实 Runtime subscribe RED、multi-subscriber partial overflow/gap RED、Gateway decode failure RED、callback Err/panic current-delivery RED。
- 冻结 Navigation 正向 delivery、Drop reclaim、fixed page、allocation rollback 与索引 FIFO 行为。
- 四份 open failure 绑定同一 source fingerprint，不复制 owner 或伪造 fixed return。

### M1 · Exposure Contract 与产品接线

- 建立 provider/schema/scope/delivery-class contract，统一 Runtime catalog、SDK builder、World apply、Editor admission。
- AI 与 Navigation 迁移到同一 exposure API；加入 provider unload/reload generation。
- 删除普通 event 与 mirrored event 可形成 catalog/World 双 truth 的路径。

### M2 · Shared Stream、Sequence 与 Global Budget

- 每 contract 一个 typed adapter，一次 encode 进入 shared segment log。
- 建立 stream sequence、global/stream/consumer budget、credit、overflow range、coalesce policy。
- producer fast path不跨 subscriber mutex，取得 1/64/1K subscriber CPU/RSS/allocation 证据。

### M3 · Acknowledged ABI 与 Editor Transaction

- 新 API table 提供 budgeted drain lease、range metadata、ack/nack/checkpoint、resync token。
- Gateway validate/decode 与 typed apply 成功后才 ack；Err/panic 保留当前 delivery。
- compatible Editor consumer 共享 Runtime cursor，并保持各自 callback fault policy。

### M4 · Lifecycle、Reload 与 Recovery

- provider/subscription/session/World 统一 revoking、draining、faulted、resyncing、terminal 状态机。
- paused World Drop、plugin reload、Session destroy retry、Editor reconnect 有 quiescence 与 generation fence。
- reader-count side effect 迁移为幂等 broker activation lease。

### M5 · Scale、Fault 与产品资格

- required Windows lane 执行 AI/Navigation 真产品、1K/10K ABI、1/64/1K subscriber、60s slow consumer。
- 注入 allocation/decode/schema/callback/ack/unload/crash 故障，证明无静默 loss、duplicate、cross-generation apply。
- 同硬件同 workload 记录 Zircon 与适用 Unreal/Bevy 路径 CPU、RSS、latency 与语义；证据前禁止“超过 Unreal”。

## 17. 验收门禁

| Gate | 当前 | 验收条件 |
|---|---|---|
| SEMR-G01 | Fail | AI 三个 Editor consumer 在真实 Runtime ABI 中均可订阅并收到正确 schema payload。 |
| SEMR-G02 | Fail | catalog exposure 与 World provider registration 由同一 contract 原子生成。 |
| SEMR-G03 | Fail | descriptor 含 provider/build/schema/scope/delivery generation，可跨 reload 判 stale。 |
| SEMR-G04 | Partial | Navigation 无 subscriber 时下一帧 debug capture、overlay 构造/发送、CPU/RSS 为零。 |
| SEMR-G05 | Fail | provider unload 拒绝新订阅并给旧 cursor typed terminal/rebind disposition。 |
| SEMR-G06 | Fail | 多 World、多 Session 同 event id 不串流。 |
| SEMR-G07 | Fail | 每次 publication 最多执行一次 payload encode。 |
| SEMR-G08 | Fail | 1/64/1K subscribers 的 private retained bytes 不按完整 payload 线性复制。 |
| SEMR-G09 | Fail | World/Session/provider/stream/consumer 均有 entry+byte hard budget。 |
| SEMR-G10 | Fail | publish 不持 subscriber queue 锁执行 serializer/foreign callback。 |
| SEMR-G11 | Fail | lossless/latest/coalesced/bounded 各有独立 overflow 测试。 |
| SEMR-G12 | Fail | partial pressure 产生逐 cursor disposition，不再只有聚合 bool。 |
| SEMR-G13 | Fail | 每个 admitted event 有 stream generation 与唯一 sequence。 |
| SEMR-G14 | Fail | Editor 拒绝非连续 range，并在 apply 前 resync/fault。 |
| SEMR-G15 | Fail | Runtime authority 在 ack 前保留 delivery；allocation 不等于 handled。 |
| SEMR-G16 | Fail | duplicate ack/nack/drain retry 均幂等。 |
| SEMR-G17 | Fail | callback Err 不永久消费当前 delivery，并有 retry/quarantine/resync 终态。 |
| SEMR-G18 | Fail | callback panic 被隔离，当前 identity 与 consumer fault state 可查询。 |
| SEMR-G19 | Fail | overflow 报告 dropped range/count/bytes/reason/recovery token。 |
| SEMR-G20 | Fail | snapshot class 在 gap 后以新 snapshot generation 恢复一致状态。 |
| SEMR-G21 | Fail | subscriber Drop 在 paused World 也 bounded wake 并收敛 activation。 |
| SEMR-G22 | Fail | World drop callback failure 有 forced disposition，外部 side effect 最终收敛。 |
| SEMR-G23 | Fail | explicit unsubscribe、Drop、Session destroy、provider revoke 恰一次 retire。 |
| SEMR-G24 | Fail | stale slot/ABI/provider generation 永不作用于复用后的订阅。 |
| SEMR-G25 | Fail | reader activation 幂等、不可重入破坏 registry、失败可 rollback。 |
| SEMR-G26 | Fail | page header 不逐 delivery 复制稳定 descriptor。 |
| SEMR-G27 | Fail | count/bytes/deadline/credit 在 Runtime 编码前生效。 |
| SEMR-G28 | Fail | Gateway protocol/decode failure 不丢失未 ack range。 |
| SEMR-G29 | Fail | PlayInstance、RuntimeSession、World、provider identity 在 wire 分域。 |
| SEMR-G30 | Fail | shared Editor cursor 保持每 consumer 独立 callback fault/ack policy。 |
| SEMR-G31 | Fail | capability reconcile prepared-set 原子切换，失败保留旧 set。 |
| SEMR-G32 | Fail | idle consumer encode/decode/alloc 为零或满足明确稳态预算。 |
| SEMR-G33 | Fail | 指标可重建 publish-to-handle latency、lag、bytes、drop/resync。 |
| SEMR-G34 | Fail | p95/p99 绑定 sample/window/BuildSet/profile/workload。 |
| SEMR-G35 | Fail | required lane 运行真实 1K/10K ABI，不以 ignored 为唯一证据。 |
| SEMR-G36 | Fail | 1/64/1K storm 记录并通过 CPU/RSS/allocation/lock-wait/lag 阈值。 |
| SEMR-G37 | Fail | 128 KiB payload、64 MiB pressure、60s slow consumer 不突破 global budget。 |
| SEMR-G38 | Fail | plugin reload、Session destroy retry、Editor reconnect、process crash 矩阵通过。 |
| SEMR-G39 | Fail | failure artifact 绑定 fingerprint、hardware、profile 与 terminal receipt。 |
| SEMR-G40 | Fail | 同场景同硬件对比前，文档/UI/发布材料不宣称达到或超过 Unreal。 |

## 18. 开放 failure 与实施入口

- Runtime10 `plugin-event-bounded-delivery` 仍 open：fixed page 已存在，但 ack/resync、global memory、request credit 与当前 source-bound 资格未闭合。
- Plugins01 `plugin-event-drain-frame-budget` 仍 open：旧“全量无界 Vec”描述已过时；当前剩余问题是固定页没有 request deadline/credit、真实 ABI 1K/10K 仍 ignored、稳态空载与 shared payload 资格缺失。
- Plugins12 `runtime-event-mirror-drop-lifecycle` 仍 open：代际 record 和索引 reclaim 已进入源码，但 paused World wake、provider revoke、Navigation producer-idle trace、RSS/p95、受管验证与 fixed return 缺失。
- Editor02 `runtime-event-consumer-unbounded-pump-lock` 仍 open：单页 pending、公平预算和锁外 callback 已进入源码，但当前 delivery 在 Err/panic 时丢失，prepared reconcile、ack/retry 与完整动态矩阵未闭合。

首个实现切片应从 M0 的四个 RED repro 开始，随后先修 `SEMR-P0-001` 的 exposure truth 与 `SEMR-P0-002` 的 authority/ack 边界；不得先写第二套 Editor queue、扩大 64 MiB 常量、在 drain 后截断、吞 callback failure，或以 ignored benchmark 代替产品资格。
