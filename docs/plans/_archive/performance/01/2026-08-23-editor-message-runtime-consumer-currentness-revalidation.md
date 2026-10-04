---
related_code:
  - zircon_editor/src/core/editor_message
  - zircon_editor/src/core/runtime_event_consumer
base_reports:
  - docs/plans/performance/01/2026-08-15-editor-message-bus-ui-delta-current-architecture-review.md
  - docs/plans/performance/01/2026-08-15-editor-runtime-event-consumer-semantic-routing-current-architecture-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Public/IMessageBus.h
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageBus.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MessagingCommon/Public/MessageEndpoint.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
doc_type: implementation-evidence
status: static_current_structural_cutover_required_dynamic_blocked
---

# Editor Message / Runtime Consumer currentness复验（2026-08-23）

## 指纹结论

| 模块 | current Rust | 行 / bytes / tests | path+raw SHA256 | currentness |
|---|---:|---:|---|---|
| `core/editor_message/**` | 35/35 | 2,935 / 93,365 / 10 | `f67e0c600b7e8352d91e21034906d20c07a3f546898ad6bc5b7bddc6b248652e` | 与8月15日35/35逐文件审查完全一致 |
| `core/runtime_event_consumer/**` | 8/8 | 1,526 / 53,205 / 2 | `10231d12f9ebb68b2b9a5d493058c9357cd6912948cfc908713d9d077d931c46` | 与8月15日8/8逐文件审查完全一致 |

`7a20f921b`只是把当时已审工作区内容提交到历史；当前两棵生产树无工作区diff，字节指纹与base report相同。因此不重复逐文件结论，也不把已完成的payload `Arc`共享、lossless原子fanout、one-page editor pending、batch commit和round-robin误列为新问题。

## 仍开放的结构瓶颈

### Message / UI delta

- generic bus本身已不在global mutex内逐subscriber enqueue；主要P0在其上层`EditorUiDeltaQueue`无entry/byte/generation/age/deadline上限，barrier仍在bus mutex内flush/materialize完整pending map。
- retained host仍可drain全部UI batch、clone全部reflection patches；apply失败后可能full reflection rebuild并再次apply同一patch vector。
- plugin lifecycle bridge仍完整drain inbox到第二个无界`VecDeque`，并在bridge/manager owner域内执行任意callback，无每帧count+bytes+deadline预算。
- generic inbox `drain`仍一次移动全部accepted delivery，缺少paged count+bytes+deadline与wall-age结果；零target publish仍可能构建delivery和遍历payload size。
- message retained-byte estimate仍只是估计，不是allocator/RSS校准的hard resident bound；scene hierarchy generation gap仍可能把稀疏变化扩大为完整reflow。

### Runtime event consumer

- manifest/ABI仍不能声明`Lossless`、`Latest { key }`或`Bounded`语义及affinity；AI和Navigation每帧完整snapshot仍按lossless FIFO积压，producer端已经序列化后再在Editor丢弃为时过晚。
- transport subscription仍按consumer而非稳定event route创建；同event/schema的多个endpoint会重复runtime queue、JSON/ABI page和decode。
- typed consumer仍在retained UI owner同步decode、锁plugin state并调用`consume`；4 ms循环预算不能抢占单个16 ms/10 s callback。
- stable Play tick仍重建capability/registration desired maps；每pump仍clone active metadata、分配/排序p95 vectors并扫描queue diagnostics，即使生产没有读取详细report。
- empty active consumer仍每帧进入ABI/session/queue drain，idle下限为`active consumers * frame rate`，缺少runtime route-ready generation/wake bit。

## 参考引擎约束

- Unreal `IMessageBus.h`、`MessageBus.cpp`和`MessageRouter.cpp`让多个recipient共享一个immutable message context，并把recipient affinity作为路由合同；Zircon应共享route/page/decoded artifact并锁外callback。
- Unreal `MessageEndpoint.h`支持异步arrival或同步poll inbox，同时明确AnyThread handler必须快速且线程安全。Zircon不应因此新增私有无界线程，而应使用Runtime11有界ticket和main-thread小提交。
- Unreal messaging的expiration/interception/resequencing说明delivery semantics属于message/route contract。Zircon的`Lossless/Latest/Bounded`是由当前AI/Navigation产品数据形态推导的本地hard cut。
- Unreal `SlateInvalidationRoot.cpp`以唯一dirty结构、paint order和分阶段profile处理UI invalidation。Zircon应把patch retention移给retained owner，而不是继续把UI patch journal放在generic bus锁内。

## 结构优化顺序

1. Editor02先为UI delta和inbox加入count+owned-bytes+deadline page、remaining/oldest age与锁/高水位计数；UI patch retention迁到retained owner，bus只保留compact dirty summary。
2. Runtime10/Plugins01把consumer manifest硬切到显式delivery policy、stable key和affinity；producer queue实现Latest replacement及Bounded overflow，不能在ABI后补救。
3. 构建唯一`RuntimeEventRouteGeneration(event_id, schema, policy, key)`；同route endpoints共享一次subscription、serialization、page和typed decode。
4. Runtime11执行声明为thread-safe的decode/prepare，发布generation-tagged immutable projection；Editor只做有界current-generation commit，stale completion apply=0。
5. Editor12以capability generation和route generation驱动reconcile；stable Play tick clone/build/subscribe=0。runtime发布ready generation，Editor只drain ready routes。
6. detailed p95/route diagnostics迁到有界中央profiler并按profile开关启用；stable empty frame不分配/sort sample vectors。

## 静态验证与动态门

- 当前生产源码`rustfmt --edition 2021 --check`：Message 35/35、Runtime Consumer 8/8通过；无生产diff需要修补。
- 两个focused Python模块执行14 tests：13通过、1 error。错误是`test_editor02_message_backpressure_contract.py`仍读取已删除的聚合路径`src/tests/editor_message/bus/backpressure.rs`；当前测试已拆到`backpressure/{behavior,fixture,performance}.rs`。这是验证owner漂移，不是产品断言失败，本切片不恢复旧聚合文件。
- 未运行Rust/Cargo和两个ignored scale benchmark。managed validator session已归档；无current-source可执行文件，F4 WPR/xperf、allocator/RSS、CPU/wakeups、package power均未执行。

两个模块继续留在`pending`。接受矩阵保持：subscribers 0/1/100/10k、UI patches/nodes 0/1/1k/100k、barriers 0/1/1k、routes 1/100/10k、snapshot producer 1/60/240 Hz、stall 0/1/60 s、callback 0/1/4/16 ms/10 s、idle 30/60/120 Hz。最终要求stable materialization/reconcile/empty poll/sort为0，latest stale apply为0，共享payload/route只serialize/decode一次，callback-in-owner-lock为0。RenderDoc不适用于该CPU控制面；只有UI invalidation cutover改变可见输出时才做draw/pixel parity。
