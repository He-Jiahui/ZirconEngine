---
related_code:
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/session/linked_plugins.rs
  - zircon_runtime/src/dynamic_api/session/linked_session.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_store.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/scene/event_mirror
  - zircon_runtime/src/scene/world/event_mirror.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/01/2026-08-14-runtime-ecs-observer-event-messaging-current-review.md
  - docs/plans/performance/01/2026-08-23-runtime-host-intent-outbox-transaction-architecture-review.md
  - docs/plans/performance/01/2026-07-22-runtime-plugin-catalog-registration-static-review.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
tests:
  - current dynamic session plugin-event/host-request/linked-plugin/session 4 of 4 Rust files and 11 inline tests reviewed
  - supporting session lock, scene mirror producer, input outbox and plugin catalog owners reviewed
  - M0 static performance contract 3 of 3 passed after RED
  - focused rustfmt 1.94.1 plus scoped diff check passed
  - current-source Cargo, event/plugin scale, WPR, allocator and power traces pending
doc_type: implementation-evidence
status: m0_static_complete_dynamic_blocked
---

# Runtime dynamic session插件事件、linked plan与host request复审（2026-08-23）

## 范围与当前性

已逐行复读`dynamic_api/session/{event_mirror,host_requests,linked_plugins,linked_session}.rs`当前
**4/4**个Rust文件。实施前合计**1,058行、40,926 B、11 tests**；M0后为
**1,076行、41,663 B、11 tests**，按`path|lines|file-hash`生成的manifest SHA256为
`aa78efddf5d7ab10cf35dba65567de97e19d794b17fb8d4fabfeec29e2c6207c`。同时沿调用链复核
session store锁、scene event mirror producer/queue、input/runtime UI host outbox和plugin catalog generation。
`host_requests.rs`、`linked_session.rs`及若干支持owner已有其他Session改动，本轮只读并保留。

## 当前源码判定

### Plugin event已有局部硬界，但聚合复杂度仍是E乘S

单subscription当前有64 events/128 KiB page、16,384 events/64 MiB queue、payload depth/bytes/time
限制、remaining/oldest-age、sequence headroom、output prepare/commit/rollback和空页零字节。这些已推翻旧的
“完全无界、typed Value再转ABI JSON”结论：producer JSON bytes当前直接写进ABI输出。

结构性P0仍成立。每个subscription都为同一typed event注册独立同步observer，observer callback逐个取得
自己的queue mutex并在锁内执行`serde_json::to_writer`。E个events、S个subscriptions因此产生E*S次
callback/downcast/mutex/serialization，并在S个私有队列最多各保留64 MiB payload。subscription数量本身无
session级count/aggregate-bytes门；1,000个停滞consumer的合同上界不是64 MiB，而是约64 GiB加容器开销。
单payload processing deadline也只限制一次subscription编码，不能限制一个event在S个同步observer中的总
wall time。

scene queue drain把最多64个owned payload移入session `pending_page`，直至ABI allocation成功才commit；
allocation finalizer在session锁外，但payload page选择与JSON encode发生在session mutex内。若整页超出wire
限制，编码器先尝试全页，再尝试1条，再二分prefix，64条page最多可执行约8次编码；deadline只能停止后续
attempt，不能收回已发生的payload copy。本轮增加attempt counter，但在动态数据前不重写prefix算法。

### Linked plugin启动重复物化同一project plan

`LinkedRuntimePluginPlan::prepare`先clone整个可选`ProjectPluginManifest`，逐registration线性检查并clone缺失
selection；之后分别构造runtime module report、clone registration reports构造临时
`RuntimePluginCatalog`、clone enabled package id形成`BTreeSet`、再构造一份project extension report，最后又
clone selected package ids。fatal diagnostics还从module/extension两边重新汇总、sort/dedup/join。

这是启动/激活成本而非逐帧热点，但所有权仍错误：session需要的是同一compiled project plugin generation中
的module plan、extension handles和package membership，不应从registration DTO与manifest为每次linked session
重建catalog/project projection。`contains_package`只在construction中查询navigation/animation两次；把小Vec
改成HashSet不能解决宽clone/build，且可能增加小集合常数成本，故本轮不改。

`linked_session.rs`仅负责profile解析、project root解析、构造和registry insert，不包含独立热算法。其性能由
project/plugin candidate build与session registry publication决定，不应旁挂第三套linked-session cache。

### Host request旧事实已有修复，剩余问题仍归单一outbox

current host路径已有空batch零编码/零buffer、32 KiB UTF-8安全IME context窗口、256 rows/256 KiB/10 ms
page、borrowed prefix encode、prepare/commit/rollback和producer/page/attempt/bytes counters。当前M0/M1/M1a的
完整证据已在host-intent专门报告中记录，本轮不覆盖其他Session改动。

剩余`PERF-MVP-425`仍成立：core IME/cursor/rumble与每个runtime UI surface各持`Vec` outbox，session先分别
`mem::take`并合成一个完整pending batch，分页发生在全部producer rows已驻留之后；manager存在多次锁事务，
latest-value state尚未按语义合并，257+ rows仍缺少不强制tick/redraw的continuation receipt。页上限不能被误报
为producer/outbox aggregate硬界。

## Unreal源码依据与统一结构

Unreal `MessageRouter.cpp:53-63,256-287`由router runnable/command queue处理route；`118-181`把一个
thread-safe `IMessageContext`共享给全部recipient，并按recipient thread直接调用或提交task。
`MessageContext.h:37-151`只让context拥有一份payload/attachment和转发时的original shared context。这支持
Zircon建立“一次producer encode的shared segment + subscription cursor/lag”，而不是复制UE线程数或为每个
plugin建私有worker/queue。

Unreal `PluginManager.cpp:2034-2085`只在`PluginsToConfigure`非空时构造一次discovery context，完成enabled
plugin处理后清空pending set；`2884-2988`先确保配置完成，再按显式loading phase遍历enabled plugin，并维护
单调`LastCompletedLoadingPhase`及phase trace/event。可转移原则是一份enabled generation、一次candidate
configure和显式阶段，不是让session consumer重新clone manifest/catalog。

目标结构为：

1. Runtime06/Plugins01发布唯一`PluginCatalogGeneration -> CompiledProjectPluginPlan`；linked session只持
   module/extension/package stable handles，startup/reload按generation变更重建一次。
2. Runtime10把每个event type接到一次编码的shared segmented broadcast log；subscription只持cursor、sequence、
   lag/overflow状态和ABI lease。低于全部cursor的segment统一回收，慢consumer按明确策略隔离。
3. Runtime11只为声明了非main affinity或超出实测frame budget的工作提供shared bounded lane；不得把E*S同步
   放大简单搬到无界后台队列。
4. Runtime10/12维持一个typed `HostIntentOutbox`，按lossless edge/latest state/bounded command分类，一次
   transaction drain，并用host-work continuation在不触发simulation tick/present时续作。

## 本轮M0

旧plugin-event ABI encoder在每个delivery内分别调用`serde_json::to_writer(event_id)`与
`serde_json::to_writer(payload_schema)`。descriptor在subscription生命周期稳定，因此一张N-row page的每次
encode attempt重复执行`2*N`次字符串转义扫描；oversize prefix搜索还会重复这些扫描。

本轮在subscribe时把两段descriptor各编码一次并以`Box<[u8]>`保存在subscription state；页编码只写已验证
JSON bytes，并把payload+重复descriptor bytes纳入有界capacity hint。成功64-row page的descriptor热路转义
调用从**128降为0**，subscription生命周期总计固定为**2**；wire字段、字段顺序、sequence、page、队列与
commit/rollback语义不变。新增`plugin_event.page_encode_attempt`计数，为全页+prefix search动态归因提供入口。

`tools/tests/test_runtime_plugin_event_descriptor_json_m0_performance_contract.py`先得到**0/3 RED**，实施后
**3/3 GREEN**；测试42行、1,621 B、SHA256
`0594509340e4dbf4354b5397ac6da7db920ec8d2112e4bd213644ebec4cfa2dc`。focused
`rustfmt +1.94.1 --edition 2021 --check`和scoped diff check通过。current-source Cargo不可执行，现有
11条Rust tests没有运行；静态调用数不冒充wall time、RSS或功耗数据。

## 动态验收矩阵

| owner | matrix | 必须采集与验收 |
|---|---|---|
| event mirror | events 0/1/1K/100K；subscriptions 0/1/8/1K；payload 0/32B/1KiB/128KiB；fast/slow/drop | producer serialize calls、observer callbacks、queue locks、payload owners/retained bytes、page attempts/bytes、session lock、lag/drop、p50/p95/p99/RSS/energy；目标serialize=E而非E*S，payload近O(E+S cursor)，aggregate retention硬有界 |
| descriptor M0 | rows 1/64；escaping 0/128 B；normal/oversize；1/1M pages | descriptor encode/scan calls、attempts、alloc/copy bytes和wall；subscription encode=2，page热路encode=0，wire bytes/sequence/order完全等价 |
| linked plugin | plugins/features/modules/extensions 0/1/100/1K/10K；cold/warm/stable/1% reload | catalog/project-plan builds、registration/manifest/package clone bytes、rows/edges/sorts、publication count、startup p50/p95/RSS；每accepted generation build/publish=1，stable session build=0 |
| host intent | rows 1/256/257/1K/10K；surfaces 1/4；idle/continuous；state/edge mixed | producer/pending/page rows+bytes+age、manager locks、attempts、continuation wake、OS calls、tick/redraw/present、main p95；aggregate有界、257+不滞留且续作不增加tick/present |

同一硬件、电源计划、foreground、frame cap与fixture至少运行三次并报告median/range/profiler overhead。
WPR/ETW负责CPU/thread/wake/lock/context switch/power，allocator负责owner/RSS。RenderDoc仅在F2/F4验证plugin
event或host continuation变更未增加render/present/draw/upload且像素一致，不作为CPU结论。current-source binary
尚不可得，本切片继续留在`pending.md`，不提交milestone、不发送完成企微。

