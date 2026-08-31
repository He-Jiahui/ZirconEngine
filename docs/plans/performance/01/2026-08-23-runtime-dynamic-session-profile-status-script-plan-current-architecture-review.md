---
related_code:
  - zircon_runtime/src/dynamic_api/session/profile.rs
  - zircon_runtime/src/dynamic_api/session/status.rs
  - zircon_runtime/src/dynamic_api/session/script_systems.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/apply_to_world.rs
  - zircon_runtime/src/scene/runtime_extension/mod.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/buffer.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-14-app-runtime-startup-teardown-current-review.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
tests:
  - current profile/status/script-system 3 of 3 Rust files and 4 inline tests reviewed
  - construction, fixed-step caller, registry clone/plan builder, system-set identity and ABI consumers reviewed
  - M0 static performance contract 3 of 3 passed after RED
  - full linked override behavior test added but current-source Cargo not executed
  - focused rustfmt 1.94.1 plus scoped diff check passed
  - current-source Cargo, startup scale, error ownership/RSS, WPR and power pending
doc_type: implementation-evidence
status: m0_static_complete_dynamic_blocked
---

# Runtime dynamic session profile、status与script plan复审（2026-08-23）

## 范围与当前性

已逐行复读`dynamic_api/session/{profile,status,script_systems}.rs`当前**3/3**个Rust文件，合计
实施前**368行、12,508 B、4 tests**，manifest SHA256为
`37eb5d9bc79408244fba5dc8db45632aec4f069a31cff058c8bdb2a2f12a8624`；M0后为
**416行、14,171 B、5 tests**，manifest SHA256为
`d2a9a26e4f2686dec6ed8b2cfee04af3a9afa88bce3deaaad8c52917ef26ec46`。同时沿调用链复核
construction、`state::tick_frame`、registry clone、world plan builder、system-set ID、interface status carrier及
App/runtime-host立即复制consumer。三个主文件实施前均干净，本轮M0没有覆盖外来修改。

## 当前源码判定

### Profile本身是O(1)策略，不是热点owner

profile解析是对6个静态byte slice的匹配；render bridge、pipelined submission、target mode与diagnostic log
schedule都是标量分支。所有profile当前共享`max_fixed_steps_per_frame=8`，`state::tick_frame`只调用一次
`tick_time`并把plan交给World driver。8步上限能防止长stall后无限追帧，但profile-specific defer/drop策略和
requested/executed/capped/remaining动态证据仍归Runtime03/07；不能在这个小文件凭经验修改常量。

### Status已经止住永久泄漏，但ABI寿命仍未冻结

旧`PERF-MVP-429`所述每次动态错误`Box::leak`已不是current事实。当前每OS thread复用固定4 KiB TLS array，
UTF-8边界截断，无error-count增长的runtime-owned heap；App与runtime-host当前都在FFI返回后立即校验并复制
diagnostics。稳态success/static-error也不走动态格式化。

但是`ZrStatus/ZrByteSlice`仍没有声明“仅到同线程下一次runtime call有效”的borrowed lifetime，复制行为只是
当前consumer约定，不能保护第三方plugin/host保存指针，也不能证明并发、reentrant callback或DLL unload安全。
因此PERF-MVP-429应更新为“leak止损已完成，TLS lifetime协议仍open”，最终仍由Runtime10提供caller-owned或
显式free的versioned diagnostics。不能把TLS现状标为动态验收完成。

### Script plan重复构建并深clone错误的聚合owner

construction先对linked registry执行一次`world_runtime_extension_plan()`；随后把结果传给
`merge_builtin_script_scene_systems`。后者完全忽略该plan，深clone整个`RuntimeExtensionRegistry`，对fixed/update
两个ID分别从头扫描runtime systems，最多两次intern同一system set，最后再次生成完整world plan。

registry的clone覆盖plugin/module/system-set、system/resource/event/interface/manager/module、render families、
component/UI/options/catalog/importer等聚合owner；world plan又逐项clone component/resource/event/system
registration与closure。以R个world registrations、S个runtime systems、M个缺失builtin phases计，当前每session
plan registration materialization为**2R+M**，system scan最多**2S**，registry aggregate clone为1，set intern
最多2。这是`PERF-MVP-629`平行catalog/generation问题在session startup的直接实例。

M0不改变`SystemSetId`。该ID是registry-local dense u32；用全新小registry构造builtin contribution会在linked
registry已有其他set时错配排序身份，因此拒绝这种表面“轻量”修补。安全止损是：一次scan得到两个presence，
完整override时直接从原registry构造唯一plan；有缺失时才clone原registry，owner/set各intern一次，补齐后只构造
一次最终plan。目标plan materialization **`2R+M -> R+M`**、system scan **`2S -> S`**、set intern
**`M -> min(1,M)`**；aggregate clone仍为0或1，最终硬切由PERF-MVP-629承担。

## Unreal源码依据

Unreal `PluginManager.cpp:2034-2085`只在`PluginsToConfigure`非空时建立一次discovery context，处理并mount后
清空pending；`2884-2978`先确保configure完成，再按单调loading phase加载enabled modules并发布phase完成事件。
可转移原则是同一accepted plugin generation只configure/materialize一次、阶段缺失才补工作；它不支持Zircon在
同一session startup先构造plan再丢弃，也不提供Zircon的最终规模阈值。

## 本轮M0与动态验收

本轮删除construction的预建plan，把唯一plan materialization交给script merge；一次遍历同时得到fixed/update
presence，完整linked override直接复用原registry构造plan，缺失阶段才clone registry，owner/set各intern一次后
补齐。由此每session实际执行的plan builder从**2降为1**，registration materialization从
**`2R+M`降为`R+M`**，runtime-system visits从最多**`2S`降为`S`**，set intern从最多**2降为1**；
full override的aggregate registry clone从**1降为0**。wire、system ID、stage、order、set identity和plugin override
优先级不变。

`tools/tests/test_runtime_session_script_plan_m0_performance_contract.py`先得到**0/3 RED**，实施后
**3/3 GREEN**；测试45行、1,946 B、SHA256
`c183b3bfc212f4c6936d66a9acdc128328a0ed2c90584f8ba6124ee5bb6c1523`。新增full-linked-override Rust
行为测试锁定两个阶段与registration count；focused `rustfmt +1.94.1 --edition 2021 --check`和scoped
diff check通过。current-source Cargo不可执行，新增及既有5条Rust tests没有运行；上述复杂度是源码调用/
所有权计数，不冒充wall time、RSS或功耗数据。

动态矩阵按world registrations/runtime systems/plugin families 0/1/100/1K/10K、missing phases 0/1/2、session
create 1/1K记录registry clones、plan builds/registration clones、system visits、String/Vec/Arc clone bytes、startup
wall/RSS/energy；M0要求plan build=1/session、visits=S、full override registry clone=0、missing override clone=1。
结构终态要求同accepted generation compiled plan build<=1且stable session build=0。

status按dynamic errors 1/1K/1M、threads 1/16、nested/reentrant call、immediate/deferred copy、reload/unload记录
TLS/owned bytes、RSS、pointer lifetime、invalid read与free；当前只可声明leaked heap=0的源码事实，最终合同要求
所有consumer可证明有效或exactly-once free。WPR/allocator负责startup/error CPU、alloc、RSS和power；RenderDoc与
该切片无直接GPU归因，不作为必需工具。current Cargo和动态数据完成前仍留在`pending.md`。
