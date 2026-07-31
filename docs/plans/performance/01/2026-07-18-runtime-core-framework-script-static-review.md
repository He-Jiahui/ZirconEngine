---
related_code:
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/src/core/framework/script/behavior_bridge.rs
  - zircon_runtime/src/script/vm/host/script_call_table.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/capability_set.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
tests:
  - two of two framework script Rust files reviewed
  - production call-context construction traced
  - current-source Cargo, allocation counters and script product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime core framework script逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读framework script当前Rust文件2/2，并追到HostExportRegistry、production ScriptCallSite、CapabilitySet与behavior bridge resolution。Descriptor/builders是启动注册数据；call context、value conversion与bridge invoke是每次VM→host调用路径。

## PERF-MVP-331：每个host call重建owned调用上下文

`ScriptCallSite::call`已有预解析callback/descriptor，但每次仍把稳定module/function各`to_string()`，并深clone`CapabilitySet.capabilities: Vec<String>`到`ScriptHostCallContext`；arguments拥有Vec，String/Bytes values拥有正文，`ScriptHostFromValue for String`又clone一次参数正文。HostExportRegistry的非生产直调路径还clone完整function descriptor后验证。高频transform/input/component API因此即使业务只读一个数值也支付多次allocator/字符复制。

反射注册侧`from_reflect_registration`先为每个projected field线性扫reflected fields，再为每个reflected field线性扫projection，复杂度O(F²)并克隆descriptor文本；capability builder每追加一次sort/dedup。它们属于F0启动/热重载，不与per-call根因混为一个阈值，但应共享compiled descriptor identity。

Runtime13/07应硬切为借用或arena调用帧：ScriptCallSite持interned ModuleId/FunctionId与Arc compiled descriptor，context借用`&[ScriptHostValue]`和共享CapabilitySet/bitset；string/bytes参数提供borrowed view，只有host明确取得所有权时clone。VM backend可复用per-thread argument scratch；reflection投影在registration generation建立field-name index并一次生成compiled ABI，reload按generation替换。不得保留同时构建owned旧context的兼容双路径。

## 验收要求

对calls 1/100/1M、args 0/1/16/256、string/bytes 0/16 B/1 KiB/1 MiB、capabilities 0/1/32/1k、types/fields 1/100/10k记录context/value/string clone bytes、alloc、capability probes、descriptor field comparisons/builds、callback throughput/p95/RSS：stable call module/function/capability clone=0，borrowed string clone=0，scratch bounded；registration近O(types+fields)，generation descriptor build≤1；error index/type/capability/serde/native/ZrVM parity、Cargo/F2 script产品trace通过前，本目录留在`pending.md`。
