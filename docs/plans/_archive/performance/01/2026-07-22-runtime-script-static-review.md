---
related_code:
  - zircon_runtime/src/script
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
  - docs/plans/zircon_plugins/08-zr-vm.md
reference_sources:
  - dev/godot/core/object/script_language.cpp
  - dev/godot/modules/gdscript/gdscript.cpp
  - dev/Fyrox/fyrox-impl/src/script/mod.rs
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/GarbageCollection.cpp
tests:
  - zircon_runtime/src/script/vm/behavior_bridge.rs::tests::successful_callback_only_rewrites_cache_when_the_handle_changes
  - zircon_runtime/src/script/vm/gameplay_host/components.rs::performance_contract_tests::entity_exists_uses_the_world_entity_index
  - zircon_runtime/src/script/vm/host/host_registry.rs::tests::validity_check_does_not_clone_the_capability_record
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs::tests::callback_and_system_dispatch_avoid_wide_record_clones
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/gc.rs::gc_pending_queue_deduplicates_without_linear_queue_search
  - current-source Windows Cargo and script-scale product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime script逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/script/**`当前源 **96/96** 个Rust文件、**15,555** 行、**129** 条测试已逐文件阅读。覆盖VM backend、behavior bridge、gameplay host、GC bridge、host exports/registry/call table、四类host interface、module/plugin discovery与management policy、reflection catalog/schema、hot reload/slot manager、scene hook/runtime context及全部测试。现有测试对生命周期、反射正确性、GC FIFO与能力门控较强，但缺1/100/10k slots/bindings/types/worlds、1M host calls、真实wall-time GC、I/O预算和产品F0/F2/F4 trace。

## 关键瓶颈

- **PERF-MVP-442 / Runtime13**：scene hook的Fixed与Update各自全World `node_records()`，clone并反序列化每个`script.bindings` JSON，再按binding格式化identity、扫描package slot并同步调用export；稳定世界没有generation命中。
- **PERF-MVP-443 / Runtime13/Runtime07/09/12**：每个gameplay host call clone完整runtime context/LevelSystem，read-only String仍clone；input clone全snapshot，component/nearest/count扫全部nodes，navigation/dynamic component多次JSON往返，复合调用可能重复进入World锁。ScriptCallSite的owned module/function/capability问题继续复用PERF-MVP-331。
- **PERF-MVP-444 / Plugins08**：每stage system projection先由`list_slots()`深clonebackend/source/manifest/management，再重建generation map、clone/sort registrations；package call按name全slot collect/sort。callback generation wide record clone和owned system二次clone本轮已止损，generation-owned active table仍缺失。
- **PERF-MVP-445 / Plugins08/Runtime11**：GC每frame全slot scan/sort，预算信任backend自报值，现有测试允许10µs预算得到12µs报告；memory soft/hard policy没有coordinator执行闭环。FIFO membership的O(P²)已改为HashSet，但真实deadline、next-due索引与RSS动作仍缺失。
- **PERF-MVP-446 / Plugins08/Runtime09**：reflection prepare clone state/build registry/验证全部managed Worlds，commit再次clone并重复`validate_candidate`；同revision snapshot/apply又重建和clone registrations。
- **PERF-MVP-447 / Plugins08/Runtime11**：package discovery在caller同步无界递归`read_dir`，每候选立即读取完整manifest/project/bytecode；没有depth/file/bytes/symlink/ignore/cancel预算或worker backpressure。

## 本轮直接止损

1. behavior callback仅在hot reload改变handle generation时写回cache，稳定成功调用不再锁cache、clone key并重复insert。
2. `entity_exists`使用World entity index的`contains_entity`，删除为一个布尔查询物化全部node records。
3. `HostRegistry::is_valid`直接检查slot generation/occupancy，删除完整capability record与label String clone。
4. callback只读取Copy generation，不再构造wide slot record；registered systems消费owned Vec，不再`.iter().cloned()`复制registration。
5. GC pending queue用`HashSet<PluginSlotId>`维护membership，enqueue/requeue摊销O(1)，pop/unload同步维护queue/set，FIFO和panic重试语义不变。

五项均先得到源码契约RED，再完成GREEN、`rustfmt --edition 2021`与scoped `git diff --check`。两次Cargo CPU lane申请分别被`plugins01-host-context-registration-replay-r2-20260722`与`editor-layout15-native-keyboard-return-r3-20260722`预约，因此没有绕过协调器运行raw Cargo。

## 参考约束与动态验收

Godot把script profiling显式gate在debug/profiling开关内，并在reload时先短锁快照scripts、锁外按依赖顺序重载；这支持把稳定帧观察者成本关掉、把reload宽工作移出registry锁。Fyrox用稳定node handle和typed ScriptTrait/context/message形状，支持以typed handle/projection代替每call JSON与全World枚举。Unreal GC明确使用每帧time limit、检查granularity、incremental cursor与部分parallel reachability，而不是只信任collector自报耗时。

动态验收需要覆盖calls/nodes/slots/bindings/types/worlds 1/100/10k、host calls 1M、GC budget 50/500/5000µs、package tree 1/100/10k与bytecode 0/1/256MiB，记录clone bytes、alloc、node/slot/world visits、registry/World lock、JSON/I/O bytes、queue age/depth、host/backend duration、RSS与main-thread p95。current-source Cargo、规模counter和F0/F2/F4产品trace通过前，本目录继续留在`pending.md`，不得进入`review.md`。
