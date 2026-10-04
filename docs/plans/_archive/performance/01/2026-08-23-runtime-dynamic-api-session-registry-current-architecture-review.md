---
related_code:
  - zircon_runtime/src/dynamic_api/session/registry
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/session/operation.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderingThread.h
  - dev/godot/core/templates/command_queue_mt.h
  - dev/bevy/crates/bevy_render/src/pipelined_rendering.rs
tests:
  - current session registry 9 of 9 Rust files and 18 tests reviewed
  - tools/tests/test_runtime_allocation_registry_storage_contract.py
  - current-source Cargo and contention traces blocked
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Runtime dynamic API session registry当前架构复审（2026-08-23）

## 范围与当前性

已逐行复读`zircon_runtime/src/dynamic_api/session/registry/**`当前 **9/9** 个Rust文件、**1,636行、57,450 B、18 tests**，manifest SHA256为`98a6df07a11d05be486ad74a854d1dbc6f70fe3e0b4e2e37b3662ba969cc6f10`。`mod.rs`和`session_store.rs`当前diff只有rustfmt import排序，本轮不覆盖；host-request事务改动涉及的commit/rollback语义已按current source复核。

生产调用链覆盖session lookup/action、frame demand/wake、capture/profile/host/plugin/world/operation owned allocation、destroy/retry及FFI finalize。测试覆盖action+wake quiescence、回调重入destroy拒绝、finalize期间destroy屏障、allocation单次释放/foreign session/retry和性能夹具，但current Cargo未执行。

## 已正确的边界

- global session registry锁只完成handle验证、HashMap lookup和`Arc<SessionSlot>` clone；不跨session action、GPU、JSON或session Drop。
- `with_session_result_finalized`在session mutex外执行ABI allocation registration，同时action guard一直保留到publication结束；`with_session_result_committed`在finalize失败后重新短锁rollback，成功后短锁commit。这是host/plugin/world分页事务的必要语义。
- allocation payload现在直接保留producer `Vec<u8>`，不再以`into_boxed_slice`强制收缩capacity；release从HashMap移除后在锁外drop大Vec。destroy以Condvar等待action和wake callback，不是busy-spin。
- closing/teardown-retry阻止新普通action但允许owned allocation release，outstanding allocation会阻止DLL/session卸载；wake callback同步重入destroy被显式拒绝。

这些行为必须在优化后保留，不能通过移除action lease、在global lock内finalize或强制释放foreign-owned buffer换取表面吞吐。

## 已落地M0：保留producer Vec storage

`register_runtime_allocation_in_action`原先对每个非空Frame/Accessibility/Profile/HostRequests/WorldSync/PluginEvents/Operation输出执行`Vec::into_boxed_slice`。bounded JSON writer从capacity 0增量扩容，CapturedFrame也不承诺`capacity == len`；转换因此包含一次不必要的capacity收缩请求，并可能触发realloc/搬迁。

当前`RuntimeAllocationRecord`直接拥有`Vec<u8>`，在进表前取得data pointer；移动Vec句柄不会移动其buffer。静态上，非空输出每次强制capacity收缩尝试从1降为0，ABI长度、opaque allocation ID、census、foreign-session拒绝、double-release和锁外drop语义不变。源码契约先RED命中`into_boxed_slice`，修改后1/1 GREEN；Rust行为测试和allocator trace仍待managed Cargo恢复。

## P0：唯一session mutex仍包围完整业务action

每个`with_session`/`with_session_activity` action都持有`Mutex<Option<RuntimeDynamicSession>>`。FFI调用中该范围包含tick、event dispatch、GPU capture/present、accessibility/profile snapshot、plugin subscribe/unsubscribe、world watch/unwatch及部分operation操作；同session的输入、控制、查询和呈现被最慢action完全串行。finalize锁外只移除了最后的owned allocation插入，不能消除前段GPU/JSON/World工作。

PERF-MVP-430的结构方向保持：slot短锁admission并生成typed command/ticket；runtime/world lane与render lane按语义有界、有序执行，完成后短锁publish generation或commit token。只读结果消费immutable generation-owned artifact。destroy先关闭admission，再cancel/drain有界lanes，最后执行当前quiescence与allocation ownership门。

Bevy `pipelined_rendering.rs:116-181`使用容量1双向channel让N帧render与N+1帧simulation重叠；Godot `command_queue_mt.h:230-264`明确区分普通`push`和同步`push_and_sync`；Unreal `RenderingThread.h:107,166-237,1034-1106`把render enqueue/pipe与显式`FlushRenderingCommands`分离。共同约束是昂贵工作进入owner queue、同步点显式，不是给每个FFI调用增加线程。

## P1：owned allocation仍是进程级单锁

所有session、所有Frame/Accessibility/Profile/HostRequests/WorldSync/PluginEvents/Operation输出共用`RUNTIME_ALLOCATIONS: Mutex<RuntimeAllocationRegistry>`。每次非空输出注册执行global ID、census HashMap和allocation HashMap写；每次release执行lookup/remove/census写。payload copy/drop已移出锁，但多session capture与事件消费者仍在同一进程锁上串行，单session slot分离在这里失效。

Runtime10应把allocation table和census下沉到`SessionSlot`，allocation id携带不可伪造的slot/generation或由slot-local opaque id解析；若ABI必须保持进程级id，则使用稳定shard/slot index直接路由，global表只管理session generation。destroy继续要求该slot outstanding=0，foreign session/double free返回语义不变。验收必须含1/8/64 sessions与frame 1/60/240 Hz，而不是只有当前单session 4 KiB循环。上述Vec M0只移除了潜在realloc，未关闭本项。

## P1：destroy/wake缺少延迟与阻塞观测

`destroy_session_slot`对active actions和wake callbacks无deadline地Condvar等待；只要GPU/JSON/action或宿主wake callback永久阻塞，destroy和library unload也永久阻塞。同步callback本身在producer线程执行，只有in-flight计数，没有callback wall time、owner thread、waiter age或stuck diagnostic。

不能简单加超时后卸载DLL，因为仍在执行的callback/action可能触及已卸载代码。Runtime10/11应提供cancel request、lane quiescence deadline与typed teardown-incomplete原因；超时必须保留slot/DLL owner并允许显式retry。记录active action kind/age、session mutex wait/hold、queue age/depth、wake callback count/time/thread和destroy各阶段wait。

## 验收

按action 0/1/10/1000 ms、GPU/JSON payload 0/1 MiB/256 MiB、producer capacity=`len`/1.5x/2x、callers 1/8/64、sessions 1/8/64、allocation 0/4 KiB/256 MiB与wake callback 0/1/1000 ms记录global/session/allocation lock wait/hold、realloc/copy bytes、queue depth/age/drop/cancel、action age、allocation ops/bytes和destroy phase time；目标是registration realloc/copy=0、global registry锁不跨业务、session mutex不跨GPU/I/O/JSON/大复制、不同session allocation无单锁串行、队列/bytes有界且现有order/rollback/release/retry/reentrant-destroy语义等价。current Cargo、WPR/ETW contention、RSS/allocator与产品F2/F4未通过前留在`pending.md`，不进入`review.md`。
