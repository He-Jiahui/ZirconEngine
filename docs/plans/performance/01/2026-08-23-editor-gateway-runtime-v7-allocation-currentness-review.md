---
related_code:
  - zircon_editor/src/core/gateway
  - zircon_runtime/src/dynamic_api/session/registry/allocation_registry.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime_host/src/foreign_output
  - zircon_runtime_interface/src
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/UnrealMemory.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/HAL/UnrealMemory.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/HAL/MallocBinned2.cpp
tests:
  - tools/tests/test_runtime_allocation_registry_storage_contract.py
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Editor Gateway / Runtime V7 allocation currentness复审（2026-08-23）

## 范围、漂移与结论

以2026-08-15 Gateway报告为基线，当前`zircon_editor/src/core/gateway/**`为 **21/21 Rust文件、2,934行、95,550 B、11 tests**，path+raw fingerprint仍为`a1c665faf9ca94b8c405304594389019dba81b4441e7354b2f078e1f25ac2079`。V7提交`c6ba29949bbca40505b352d5f2bc642b5f5b6c17`改变其中 **9文件、当前1,477行、50,883 B、10 tests**；这9个文件已逐行复读，其余12个文件沿用前次逐文件结论。

同时复核V7 allocation owner相关12个当前文件（runtime registry/action/session/FFI/encoder、runtime_host validation/state、runtime_interface result/id/API表），合计2,761行、95,783 B。结论是：V7解决了跨DLL Rust allocator callback和裸capacity重建风险，Gateway的exactly-once释放与DLL owner存活顺序正确；但当前runtime以一个进程级allocation mutex串行所有session和所有输出类型，现有性能测试不能证明并发可扩展性。该结构问题保持open。

## 当前调用与锁成本

非空输出共有9处registration：Frame、Accessibility、Profile、HostRequests、PluginEvents、World query/invalidation两处及Operation result。一次典型runtime生产+host释放至少经过：session registry lookup、action lifecycle begin、session mutex、allocation registry insert、action lifecycle finish，再在release侧经过session registry lookup、release lifecycle begin、allocation registry remove和lifecycle finish，即静态约 **9次mutex acquisition**；JSON成功接收还会进入host acceptance gate。数量是源码边界计数，不是动态耗时。

空payload不会进入allocation table，因此空host request和空plugin event page不会承担该全局表成本。非空事件页、WorldSync JSON、profile/accessibility/operation结果及未来产品可达的captured frame仍共享同一mutex。当前单session 2,000次/4 KiB顺序测试没有threads、sessions、outstanding depth、payload/capacity矩阵；提交标题中的p99/吞吐不得作为本轮current-machine证据。

## 已落地M0

`RuntimeAllocationRecord.bytes`从`Box<[u8]>`改为`Vec<u8>`，删除registration内`into_boxed_slice`。producer buffer指针在进表前取得，表只移动Vec句柄；释放仍从表移除后在锁外drop。静态量化：每个非空ABI输出的强制capacity收缩尝试 **1 -> 0**，额外payload copy的可能入口随之移除；ownership、长度、opaque id、census与exactly-once合同不变。

`tools/tests/test_runtime_allocation_registry_storage_contract.py`先RED后GREEN，1/1通过；`rustfmt --edition 2021 --check`和scoped `git diff --check`通过。由于managed Cargo session已归档，本轮没有执行Rust测试，也不声称动态realloc、lock p99或功耗改善。

## 结构性问题与目标架构

### P0：UI帧内同步FFI/JSON仍未改变

V7只替换owned-result释放协议，没有改变Gateway的同步调用模型。Active Play tick和每个无pending page的event consumer仍由retained UI frame直接调用foreign runtime并解码；一次慢provider或最大页仍可吞掉整个UI frame。继续由`PERF-MVP-597`/Runtime11实现session-owned、单飞、有界、generation-tagged ordered lane，UI只提交ticket和应用immutable completion。

### P1：全局allocation mutex破坏session隔离

`RUNTIME_ALLOCATIONS`同时拥有global ID、allocation map和第二张census map。Runtime10应把allocation/census放入`SessionSlot`，或让opaque ID携带slot generation并直接路由到稳定shard；不同session不得先争抢一个allocation锁。不得直接照搬Unreal的共享C++ allocator，因为独立Rust DLL allocator provenance不同；Zircon仍需runtime-owned release入口和outstanding unload barrier。

### P1：release附带重复session查找和lifecycle锁

host已经持有validated session handle与runtime owner，但每个release仍重新经过global session registry、slot lifecycle和global allocation registry。目标是generation-qualified slot lease直接完成local allocation removal，同时使destroy关闭admission、等待outstanding=0并保留retry语义。foreign-session、double-release、release-after-close和DLL unload必须继续fail closed。

## 变更文件逐项结论

| Gateway文件 | current-source结论 |
|---|---|
| `session/frame.rs` | V2 frame持有opaque output；显式release和Drop都保证先释放allocation、后释放runtime owner。capture仍同步；当前产品调用仅见接口/测试，不能用该事实替代未来帧门。 |
| `session/gateway.rs` | 构造时固定V7 release函数和session；每次输出不重复解析API表。同步执行和host acceptance gate仍存在。 |
| `session/operations.rs` | harvest切到V2 result和显式ABI版本；当前无产品caller，保留P1。 |
| `session/output.rs` | validation/release单owner、错误时融合cleanup failure、Drop兜底；未发现double release。 |
| `session/plugin_events.rs` | 空页不分配；非空页decode/release仍在UI caller，既有P0未关闭。 |
| `session/profile.rs` | V2 result迁移；交互profile仍同步encode/call/decode。 |
| `session/protocol.rs` | expected ABI显式传入，O(1)，无新增热点。 |
| `session/tests.rs` | fake API补release入口，但没有多thread/session contention和slow release测试。 |
| `session/world_sync.rs` | V2 result迁移；serialized drain/query仍有同步JSON与global allocation路径。 |

## Unreal主参考约束

- `UnrealMemory.h:201-224`将Malloc/Realloc/Free导向统一Core allocator，并显式提供TLS cache lifecycle；`UnrealMemory.cpp:523-553`最终直接调用`GMalloc->Malloc/Free`，没有为每个跨模块结果建立全局opaque HashMap记录。
- `MallocBinned2.cpp:571-617`的小对象分配先走per-thread free lists，miss后才锁对应size-class pool；`866-938`释放也先走TLS bundle，large allocation才使用external mutex。其结构原则是把常见分配/释放从单一全局临界区拆开，而不是用一次单线程p99证明全局锁足够。
- Zircon应借鉴分片、thread/session locality和显式慢路径，而不是复制Unreal的pointer ABI。Rust DLL边界仍以opaque ID和runtime release为安全前提。

## 动态验收矩阵

1. allocation micro：threads 1/2/8/32，sessions 1/8/64，outstanding 1/64/2,000，payload 0/64 B/4 KiB/1 MiB/frame，capacity len/1.5x/2x；记录register/release p50/p95/p99、throughput、wait/hold、realloc/copy bytes、RSS和census high-water。
2. product F4：active Play idle/continuous、0/1/64 event consumers、WorldSync idle/storm、start/stop/reload；WPR/ETW记录UI foreign/decode wall、allocation/session lock contention、wakeups、CPU和package power。
3. correctness：double/foreign/stale generation release、destroy-with-outstanding retry、concurrent release、provider error+cleanup error、drop frame before/after gateway replacement；现有status和exactly-once语义必须等价。
4. engine parity：同机、同场景、分辨率、帧限和power plan比较Zircon/可用Unreal editor的30秒idle/continuous CPU、RSS、wakeups、package power和frame p95。RenderDoc只在current-source viewport可执行文件存在后检查present/readback/draw/GPU，不用于判定本CPU锁问题。

在current Cargo、并发矩阵、WPR/allocator/RSS/power和F4通过前，本模块保持`pending`；本轮不提交milestone、不发送企微完成通知。
