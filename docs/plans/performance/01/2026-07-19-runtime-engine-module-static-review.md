---
related_code:
  - zircon_runtime/src/engine_module
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
tests:
  - zircon_runtime/src/engine_module/tests.rs
  - current 8 of 8 Rust files, 509 lines, 8 tests inventoried
  - deterministic current manifest SHA256 91b75828fe6dbd2cbe8f30ddbd9ce1be8d93dbdcb2b868aa88e81607907784e6
  - rustfmt plus 1.94.1 edition 2021 check passed for current 8 of 8 files
  - current-source Windows Cargo and F0 activation trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime engine_module逐文件性能静态审查（2026-07-19）

`zircon_runtime/src/engine_module/**`已于2026-08-23按当前源重新逐文件阅读 **8/8** 个Rust文件、**509** 行、**14,323 B**、**8** 条测试；清单SHA256为`91b75828fe6dbd2cbe8f30ddbd9ce1be8d93dbdcb2b868aa88e81607907784e6`。该层只重导出core module/service契约，提供context、qualified name/dependency、Arc factory及descriptor→service contract helper；不拥有registry、activation、shutdown、mutex或循环，因此不是帧热路径owner。

全仓当前源调用图确认：`driver_contract`只有1个测试caller，`manager_contract`/`plugin_contract`无caller；三者会clone `RegistryName`和dependency `Vec`，但不进入产品启动或帧热路径。`module_context`仅在本测试使用，`plugin_context`的生产caller是VM plugin descriptor构造；`qualified_name`/`dependency_on`/`factory`/`plugin_factory`是运行时、编辑器和首方插件的启动/control-plane声明工具。不为未调用contract helper做推测性微优化；其公开表面的删除应跟唯一catalog硬切同里程完成，不留兼容re-export。

端到端启动追踪表明，真正的成本不在这些叶helper：`ResolvedPluginGroup`同时长期保留module与owned descriptor，`BuiltinEngineEntry::module_selection_report`为报告再深拷贝一次descriptor，`bootstrap`又对每个descriptor clone后注册。`EngineModule::descriptor()`返回owned `ModuleDescriptor`，普通entry、builtin profile筛选与dynamic session又可分别重建它。这是F0启动的descriptor/String/dependency/factory `Arc`多owner问题，已归入PERF-MVP-322与PERF-MVP-628：由唯一catalog generation构建一次immutable descriptor/compiled graph，报告使用借用snapshot或索引，注册消费同一owner。在allocator/clone-byte数据和catalog owner硬切前，不在此声明层增加缓存或局部adapter。

Unreal依据与结构目标一致：`dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp:992-1023`先查并直接返回已加载module，`1968-1985`只在pending initializer非空时提交且提交后清空。Zircon应继承“stable generation不重建声明”的行为，不照搬C++ singleton/container。完整对照和实施门见`02/2026-08-15-runtime-module-service-lifecycle-current-architecture-review.md`。

需用current-source Cargo锁定helper/contract语义，并在F0 module activation trace对0/1/5/100/1000 modules/services/edges记录descriptor builds/clones、String/Arc clone bytes、alloc bytes与catalog generation。stable generation重复activation 1,000次必须为descriptor build/clone=0；完成前留在`pending.md`，不进入`review.md`。
