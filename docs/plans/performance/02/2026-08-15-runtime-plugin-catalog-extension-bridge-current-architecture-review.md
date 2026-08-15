---
related_code:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog
  - zircon_runtime/src/plugin/extension_registry
  - zircon_runtime/src/plugin/bridge.rs
  - zircon_runtime/src/plugin/bridge
  - zircon_runtime/src/plugin/runtime_profile.rs
  - zircon_runtime/src/plugin/runtime_profile
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/feature_reports.rs
  - zircon_runtime/src/dynamic_api/session/linked_plugins.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
tests:
  - plugin catalog extension bridge and runtime profile scope 135 of 135 Rust files reviewed
  - 12342 lines and 46 inline tests inventoried
  - deterministic 135-file source manifest SHA256 99d4592f8741f425aa0c7277613451ac430fc851a19d953e4d8e11d05065851c
  - rustfmt edition 2021 check passed for 135 of 135 files
  - current-source Cargo product WPR allocator and power acceptance blocked
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Runtime 插件目录、扩展注册表与桥接当前架构审查（2026-08-15）

## 范围与结论

本轮逐文件复读 `runtime_plugin_catalog` 90/90、`extension_registry` 32/32、`bridge` 5/5、`runtime_profile` 8/8，共 135 个 Rust 文件、12,342 行、46 条内联测试。按 `git ls-files` 稳定顺序拼接路径与当前内容得到 SHA256 `99d4592f8741f425aa0c7277613451ac430fc851a19d953e4d8e11d05065851c`；135/135 通过 `rustfmt --edition 2021 --check`。范围内有 84 个 `HashMap` 类型词、262 个 `String` 类型词和 209 个 `.clone()` 调用点；这些只用于定位所有权密度，不冒充动态分配次数。

结论不是“再给目录加一层 cache”。当前 `RuntimePluginCatalog`、`RuntimeExtensionRegistry`、`FrozenBridgeTable`、`RuntimePluginBridgeLifecycleState`、runtime profile availability projection 和 Core module registry 分别持有目录、代际、owner 或生命周期状态；同一批 registration 在 app bootstrap 和 dynamic session 至少构建两次 catalog projection。M1/M5 必须硬切为 `EngineRuntime` 内唯一 package/module/capability/extension catalog generation，profile、project plan、module lifecycle 和 native bridge 都只消费该代际的 handle/view。

当前源码仍没有可运行的 current product binary；managed app build、focused runtime test 均被仓库其他编译错误阻断。因此下述复杂度和调用次数为 current-source 静态证据，不是 WPR、allocator、wall-clock、功耗或 bridge throughput 实测，本范围继续留在 `pending.md`。

## 应保留的行为合同

- catalog update 先构造候选 projection，验证成功后才发布 generation；失败保留 last-good。
- feature resolution 已使用 capability waiter index 和分层 ordered-ready bitset，工作以访问的 feature/edge 为主，不应退回全量 fixed-point 扫描。
- frozen extension table 以 slot 直接取值为 `O(1)`，key lookup 使用排序索引；bridge entry 把 provider 与 enable generation 作为一个 `ArcSwap` 状态发布。
- native discovery 已有单一 authority、异步 I/O lane、coalesced ticket 和 last-good snapshot；后续 native-loader 审查会单独验证，本文不把它冒充为 135 文件范围。
- foreign lifecycle callback 的最终目标仍是锁外执行；现有 listener 虽在 `&mut registry` mutation 中调用，但不能被新 owner 扩大为 registry/global lock 内调用。

这些是迁移合同，不是保留多套 catalog、无代际 slot、全 registry thaw 或跨动态库 Rust trait object 的理由。

## P0：产品启动和 dynamic session 重复构建同一目录

`builtin/runtime_modules/assembly/feature_reports.rs:42-52` 为 module assembly 从 registrations 构建一次 `RuntimePluginCatalog` 并解析 feature；紧接着 `zircon_app/src/entry/builtin_modules.rs:192-218` 又 clone 同一批 registrations/feature registrations，构建第二个 catalog、project plan、extension report 和 bridge lifecycle state。`dynamic_api/session/linked_plugins.rs:39-53` 也先调用 runtime module assembly，再从同一 registrations 新建 catalog 和 extension report。一次 bootstrap/session 因 owner 分裂至少支付两次 projection build，且两实例的 plan cache 互不共享。

`RuntimePluginCatalog::clone` 又深 clone registrations、feature registrations、diagnostics，并清空 `project_plans` 与 build counter，只共享 derived projection `Arc`。因此 clone 不是 cheap immutable handle，consumer 边界会丢失已编译 project plan。`RuntimePluginBridgeLifecycleState` 同时拥有整个 catalog、extension report 和 bridge table，再以 module-name String 映射 provider package；它是 Core module lifecycle 旁边的第二生命周期权威。

这与 Unreal 的 owner 方向相反：`PluginManager.cpp:2034-2080` 只在 `PluginsToConfigure` 非空时由一个 manager 完成查找、标记、处理、挂载并清空待配置集合；`ModuleManager.cpp:992-1061` 对已知 module 先走 fast check，并在初始化前发布同一个 not-ready module info。Zircon 目标应让一次 catalog mutation 生成一个共享 plan，而不是由 assembly、app、session 和 bridge observer 各自重放 registration。

## P0：project-plan cache 的键和锁范围会造成串行抖动

`runtime_plugin_catalog/project.rs:96-140` 的 cache 只以 `RuntimeTargetMode -> u8` 为键，因此最多保留三个 entry，并且每个 target 只能保留最后一个 manifest。两个 editor project/session 使用同一 target 交替查询时会互相驱逐，反复完成 manifest、feature report 和完整 extension report。命中前虽比较 catalog generation、fingerprint 和完整 manifest，但 cache mutex 从查找到 `runtime_extension_report_for_project` 完成一直持有；不同 target、不同 manifest 的无冲突构建也被全局串行。

目标 cache key 为 `{catalog_generation, project_selection_generation, target}`，值为 immutable `Arc<CompiledPluginPlan>`。同 key 用 per-key single-flight；昂贵 build 在 publication lock 外运行，不同 key 可由 TaskGraph 并行。catalog clone 不得清空 plan owner；project/session 只持 plan handle。

## P0：extension unload 与重新冻结按全 registry 规模增长

`RuntimeExtensionRegistry` 在一个对象中维护最多 20 个独立 `TypedExtensionPoint`，另有 shader source、asset importer、plugin interner、system-set、bridge table 和 revocation listener。`ownership_for` 为查询一个 owner 扫描每个 extension family；`revoke_owner_registrations:219-285` 先扫描/解绑 imports、线性扫描全部 bridge entry、通知全部 listener，再对每个 family 调用 `remove_owned_by`。

`TypedExtensionPoint::remove_owned_by:191-221` 会 thaw frozen table、take 全部 dense arrays、清空 index、扫描所有 row 并重建 survivor index；`staging_mut:302-316` 的 thaw 本身也从全部 key 重建 HashMap。随后 `finalize_bridge_imports` 又 clone 全部 exports 重建 bridge table并绑定全部 imports。故移除只贡献 `k` 个 row 的 owner，工作上界仍为 `O(sum(N_family) + bridge_exports + bridge_imports + listeners + shader/assets)`，而不是 `O(k + affected_edges)`。

此外 `ExtensionSlot(u32)` 没有 catalog/registry generation；registry clone、rebuild 或新代从 0 重新分配后，裸 slot 无法区分旧代和新代。目标使用 catalog-owned owner-to-slot ranges/adjacency 与 copy-on-write generation publication；卸载只访问 owner 贡献和受影响依赖，handle 至少携带 `{catalog_generation, slot, entry_generation, capability}`。

## P0：bridge 有原子发布基础，但仍不是统一生命周期 fast path

`FrozenBridgeTable` 的 interface-id 首次解析为 slot 后可避免重复 String lookup，这是正向基础；但 `set_owner_enabled_slots` 与 `deactivate_owner_slots` 每次都扫描全部 bridge entries。`BridgeImport::call` 加 `WeakBridge::call` 的稳定路径仍读取 binding、entry generation 和 cached provider，升级 Weak，并在 debug build 对共享 `AtomicU64` 做一次 RMW；cache miss 再读取 provider state。`resolve_strong` 则返回脱离后续 enable generation 的强 Rust trait-object handle，可能把 provider 生命周期延伸到 module quiesce 之外。

M5 的 native bridge 应是同一 module slot/capability table 的受限 ABI adapter：slot resolution、entry generation、callback lease、affinity、deadline 和 dynamic-library owner 由唯一 lifecycle coordinator 管理。owner enable/disable 使用 owner slot range，不扫描全表；诊断使用 thread-local/batched counter 或采样，不让 debug 每次调用争用共享 cache line；跨动态库不共享 Rust `Any`、trait object 或锁。

## P1：runtime profile 仍在目录外重建 availability 权威

`runtime_profile/availability.rs` 的多个 report API 每次构造新的 `RuntimePluginAvailabilityProjection`；即使 `from_catalog_with_provider_membership` 借用 catalog，也另建 descriptor HashMap 和 provider membership index。`RuntimeProfileDescriptor::for_id` 和 `project_manifest` 还会重新物化 String/Vec selection。已有 borrowed generation/report 边界是正确方向，但 profile availability 仍是 catalog 外的第二索引 owner。

这属于 bootstrap/editor control-plane 容量问题，未证明为逐帧热点。硬切后 profile 只是 `CompiledPluginPlan` 上的 borrowed/filter view；owned diagnostic rows 只在导出边界按需物化，不再有独立 availability generation。

## Unreal 对照后的唯一 owner

唯一 owner 为 `zircon_runtime::core::runtime::EngineRuntime` 内部的 plugin/module catalog，而不是新增 crate 或兼容 facade：

1. `PluginCatalogGeneration`：一次 intern package/module/capability/interface/extension ID，编译 dependency/loading-phase/owner adjacency，并与 ModuleCatalog 共用 generation。
2. `CompiledPluginPlan`：按 catalog/project-selection/target generation 构建一次，包含 enabled package/module slots、profile availability view、extension ranges 和 phase plan。
3. `PluginLifecycleCoordinator`：复用 M1 module slot 与 TaskGraph，执行 configure/load/active/quiesce/migrate/publish/retire；bridge 不是 observer 旁路。
4. `ExtensionGeneration`：immutable dense tables、owner ranges、affected-edge index 与 RCU/COW publication；读路径只持 generation handle。
5. `NativeHostAdapter`：versioned bytes/value ABI、generational handle、capability、callback lease、affinity/deadline 和 state migration；不暴露 Rust object graph。

Unreal `PluginManager.cpp:2884-2978` 先配置一次 enabled plugins，再按严格不倒退、不跳过的 loading phase 交给 ModuleManager；`ModuleManager.cpp:1316-1405` 在同一 module info 上先标 not-ready、shutdown/reset、按条件卸载代码，成功后才广播变更。Zircon 继承单 owner、phase 和 fast-path原则，不照搬 C++ singleton 或其线性 AllPlugins phase scan。

## 实施前 RED 门与动态验收

- 0/1/100/1000 packages、features、capabilities、extensions 和 dependency edges：记录 catalog/plan/extension builds、row/edge visits、String/descriptor/registry clone bytes、allocations、lock wait/hold、RSS 和 wall time。一次成功 structural transaction 只发布 1 generation、构建 1 projection/plan；失败发布 0。
- 同一 registration batch 的 app bootstrap、dynamic session 和 editor status 中 catalog authority=1；module assembly 后第二 catalog build=0。stable generation 重复查询 1,000 次的 projection/plan/registry build=0。
- 两个同 target 不同 project manifest 和三个不同 target 并发：无互相驱逐；同 key build=1；不同 key 的 build 不被一个全局 mutex 串行。cache memory 有 count/bytes/age/eviction budget。
- owner 贡献 `k`、全 registry 为 `N` 的 unload/reload：访问量与 `k + affected_edges` 成比例；unrelated family thaw/scan/sort=0，bridge owner full scan=0，callback-in-registry-lock/mutation=0。
- 1/16 threads x 1M bridge calls：记录 slot/generation loads、shared atomic RMW、global/name lookup、callback lease、p50/p95/p99；稳定调用无 global lock/String hash，debug shared RMW 不是每 call 固定成本。stale handle、disable/reload-in-flight、panic/timeout/migration rollback 和 strong-handle quiesce 均通过。
- M0 恢复 current-source F0/F4 后，runtime/editor 各至少 3 次采集 WPR/xperf CPU sample、context switch/wait、idle wakeup、线程峰值、RSS、I/O、wall time 和功耗；没有这些数据前不宣称接近 Unreal。

本轮不修改 Rust 实现：问题同时跨 app assembly、Core module lifecycle、project/session owner、extension ABI、native bridge 和 editor status；局部扩大 cache、增加 owner HashMap 或保留 observer adapter 会延长双系统寿命，与 Plan02 M1/M5 hard cut 冲突。`PERF-MVP-537/538` 继续承担 transaction/projection 与 extension materialization，新增 `PERF-MVP-629` 只承担本报告识别的单一权威、cache/slot/owner-index硬切。
