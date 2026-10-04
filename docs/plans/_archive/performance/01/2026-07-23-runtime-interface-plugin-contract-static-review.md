---
related_code:
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/plugin_diagnostics.rs
  - zircon_runtime_interface/src/plugin_events.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/bevy/crates/bevy_ecs/src/system/function_system.rs
  - dev/bevy/crates/bevy_ecs/src/system/system_param.rs
  - dev/godot/core/extension/gdextension.cpp
tests:
  - zircon_runtime_interface/src/tests/plugin_api_contracts.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - current-source Windows plugin ABI tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface plugin 合同性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/{plugin_api.rs,plugin_diagnostics.rs,plugin_events.rs}`当前源 **3/3** 个 Rust 文件、**369** 行已逐文件阅读；同时读取 `plugin_api_contracts.rs`、`contracts.rs`内 plugin callback合同，以及 runtime native host adapter和 sound dynamic-event ABI生产调用。该切片覆盖 MVP native plugin注册、host function table、event callback和结构化注册诊断。

## 性能结论

- `ZrSystemRegistrationV1`只表达 stage/order、set/before/after、invoke和 user data，没有 component/resource read/write集合或 main/worker affinity。current-source host adapter因此使用 `NativeDynamicAccess::init_state -> add_conservative_world_access()`，把每个 ABI native system标成全 World writer；不相交 native systems与普通 ECS systems也会冲突，调度器无法把它们分到 worker并行。这是既有 **PERF-MVP-041** 的 interface 根因，不新建重复编号。
- ABI v3的 `ZrSystemRegistrationV1` size/offset已由 contract tests固定；不能在 v3尾部静默追加访问字段。Plugins01必须通过版本化 v4 descriptor或与 system id绑定且同代验证的 immutable access table传递 exact access/affinity，并在注册期解析为 `SystemParamAccess`；缺失、未知或越权声明只能显式拒绝或记录为 conservative fallback，不能误报可并行。
- `ZrPluginEventCallbackRequestV1`为 Copy POD，sound adapter从已有 delivery借用 plugin/handler/event/source/schema/payload byte slices，成功 callback路径没有 DTO String/Vec分配；错误路径才构造 status detail。当前未发现新的 interface event-copy瓶颈，但必须保持 callback同步借用生命周期，不能让 foreign code跨调用保存 slice。
- `ZrHostApiV3`的 asset request、event emit/drain和 spawn command在 current-source adapter仍返回 `UnsupportedVersion`，因此没有可验收的产品吞吐；未来实现必须复用 Plugins01已有 bounded event cursor/page与 compiled command slot，禁止在 ABI边界引入无界 drain或每调用 serde/heap truth。
- `RegistrationDiagnostic::missing_capability`在失败控制面多 clone一次 plugin id；它不在帧循环或成功注册路径，单独改动收益不足，不升级为瓶颈。其余 API tables、handle/status与 result均为 pointer-dense/Copy数据。

## 优化设计

1. 新 ABI generation携带 stable component/resource/event access ids、read/write mode和 thread affinity；host在 registration/finalize阶段一次 validate、intern并编译冲突集，frame run只读取 dense compiled access。
2. scheduler对 exact disjoint access进入 worker batch，对冲突写确定串行，对 main-thread-only固定主线程；dynamic/unknown access保守但必须有 counter与诊断，不能无声吞掉并行能力。
3. event/asset/command host calls消费 generation-owned bounded transport和 caller-provided output buffer；成功热路径不分配、不做 String resolve或 schema parse，reload以 generation/quiescence回收旧 callback。

该设计已经由 PERF-MVP-041及 Plugins01 M5拥有。本轮只补 current-source interface/adapter证据，不修改 ABI，也不创建第二套 access authority。

## 参考引擎对照

Bevy在 system初始化时由 `SystemParam::init_access`产生 `FilteredAccessSet`，全 World读写显式成为保守访问，普通 typed参数则保留精确冲突信息。Godot GDExtension按 initialization level调用版本化 C entry，说明兼容函数表应在初始化慢路解析，而不是把字符串/反射解析放进帧回调。Zircon需要保留自己的 stable plugin ABI和 reload generation，但 exact access同样必须在执行前编译。

## 动态验收

1. current-source `zircon_runtime_interface` ABI layout/callback contracts和 runtime native registration tests；v3 size/offset保持不变，新版 descriptor未知版本/字段拒绝语义确定。
2. native systems 1/100/1k，World 1/8，worker 1/2/16：不相交 read/write系统可并行，真实冲突确定串行，main-thread affinity固定，unknown fallback有计数；记录parallel overlap、batch数、main queue wait与frame p95。
3. callback 1/1M，payload 0/1KiB/1MiB，threads 1/16：成功 ABI request heap alloc=0、String/schema parse=0；bounded event backlog、age/drop/overflow和 reload/unload quiescence门通过。
4. F0/F2/F4最小 native plugin产品 trace证明注册解析只发生一次，stable frame access rebuild=0，foreign callback不持有 world/plugin全局锁。

动态调度、current-source Cargo和产品 trace未完成，因此该切片继续保留在 `pending.md`，不进入 `review.md`。
