---
title: Runtime Plugin Interface Bridge、Slot、Generation、Strong/Weak、Native/VM、Lifecycle、Diagnostics 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime58
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/core/framework/bridge
  - zircon_runtime/src/plugin/bridge.rs
  - zircon_runtime/src/plugin/bridge
  - zircon_runtime/src/plugin/extension_registry/register/bridge_registration.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/bridge_scope
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_host_handle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs
  - zircon_runtime/src/script/vm/host/bridge_host_module.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_editor/src/core/play/plugin_activation
  - zircon_editor/src/ui/workbench/snapshot/data/bridge_diagnostics_snapshot.rs
  - zircon_plugins/ai/runtime/src
  - zircon_plugins/physics/runtime/src/plugin.rs
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
tests:
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_bridge_dependencies.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/bridge_scope/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/bridge_bindings.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_publication.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/registration_replay.rs
  - zircon_runtime/src/script/vm/tests/bridge_host.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_editor/src/tests/editor_event/runtime/stack_play.rs
  - zircon_plugins/ai/runtime/src/tests/integration_tasks.rs
  - zircon_plugins/ai/runtime/src/tests/perception_runtime.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/12-first-party-physics-source-runtime-editor-dist-catalog-simulation-collision-joint-ragdoll-product-integration-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_tooling/35-ownership-graph-shared-weak-borrow-lease-callback-subscription-raii-cycle-detach-leak-isolation-review.md
  - docs/plans/optimize/zircon_tooling/36-type-erasure-dynamic-dispatch-any-downcast-trait-object-reflection-type-identity-vtable-generation-performance-review.md
  - docs/plans/optimize/zircon_tooling/37-transaction-atomicity-prepare-commit-publish-rollback-compensation-idempotency-crash-recovery-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Features/IModularFeatures.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Features/ModularFeatures.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/godot/core/extension/gdextension_manager.h
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/godot/core/extension/gdextension.h
  - dev/Fyrox/fyrox-dylib/src/lib.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/ContextContainer.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 58 · Runtime Plugin Interface Bridge、Slot、Generation、Strong/Weak、Native/VM、Lifecycle、Diagnostics 与 Product Integration 工程化差距

## 1. 结论

Zircon 的插件桥接并非空壳。它已经有冻结表、dense `InterfaceSlot`、奇偶 generation、弱缓存、owner 批量 enable/deactivate、required dependency blocker、native method binding、dynamic-library callback lease、registration replay cache、VM host adapter，以及 Editor bridge matrix。局部测试还覆盖了 concurrent generation publication、binding reinstall、native reload 顺序、worker access admission 和 1/100/1,000 system 的手工 benchmark。后续重构应保留 immutable generation、weak observation、library pin 和预编译 access plan 这些正确方向。

但当前实现仍不是工程级插件间接口系统。`InterfaceSlot` 只是公开的裸 `u32`，既不携带 table identity、interface identity，也不携带 generation；`FrozenBridgeTable` 用 `HashMap<String, InterfaceSlot>` 和 `Arc<dyn Any>` 保存合同，wrong-type downcast被压成`NotEnabled`。`WeakBridge::call`只在调用前读取一次奇偶 generation，随后拿到的`Arc<T>`可以跨 disable/deactivate 执行；`BridgeGuard`和`StrongBridge`更能无限期持有provider，却没有call lease、quiescence或retirement回执。生命周期名称包含`at_frame_boundary`，API却不要求frame token、owner thread或safe-point证据。

Native链存在更直接的产品断点。两项registration replay公开API除了HostHandle转发与测试外没有production caller；Editor Play只load DLL、apply bridge lifecycle并进入play snapshot，没有把registration manifest中的system安装进其RuntimeExtensionRegistry/World。即便外部调用replay，生成的World system闭包也会捕获当时的`NativeHostBridgeCallScope`；现有测试明确认证重新安装binding后旧World继续持有旧generation。成功热重载会让旧library owner保持transition closed，旧World因而静默拒绝调用，同时闭包继续强持旧DLL；system body又丢弃全部`ZrStatus`，无法向World、Editor或reload transaction报告停跑。Native live host还先发布loaded plugin和method binding，再单独apply bridge lifecycle，bridge失败无法回滚前一阶段。

本轮登记三项本地P0。第一，bridge handle没有operation lease，disable/deactivate不能证明调用已经停止。第二，native registration replay没有进入App/Editor产品装配，NativeDynamic system贡献不可达。第三，已replay的World不会在reload时替换/撤销system，旧scope静默停跑并阻止旧library retirement。Runtime07已拥有全局PluginCatalogGeneration和跨所有contribution的总事务，本篇不重复该父P0；本篇只拥有bridge slot/call/lifecycle/native-VM adapter和World binding的具体闭环。

本轮登记 **3项P0、64项P1、16项P2和40项验收门禁**。目标不是继续给`WeakBridge`追加布尔检查，而是建立`InterfaceContractCatalog + BridgeTableGeneration + TypedInterfaceHandle + BridgeCallLease + BridgeLifecycleTransaction + NativeBridgeGeneration + WorldBridgeBindingRegistry + VmBridgeAdapter + BridgeObservationStream`。所有consumer必须在同一generation上解析、调用、诊断和退休，任何stale table/slot/provider/method/world binding都要结构化fail-close。

本轮只做静态review与文档总账，没有修改production、tests、Cargo、ABI或参考源码；没有运行Cargo、动态库、真实Editor Play、hot reload、并发stress、soak、sanitizer或benchmark。静态结构不能证明性能已达到或超过Unreal。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / tests | fingerprint / 说明 |
|---|---:|---|
| core bridge、extension registration与runtime lifecycle | 14 / 2,325 / 77,145 / 8 | SHA-256 `2160e55ebeb01403d293fd595c28fd479aa500f412704a2aa8a68b73a57cb6ab` |
| native bridge、reload/replay与VM adapter | 11 / 4,846 / 177,623 / 2 | SHA-256 `f8686e23ff189231e05a55857d226e0daebccc3402039ebcbf8c84a409c0a460` |
| App、Editor与AI/Physics/ZrVM产品消费者 | 17 / 3,445 / 123,124 / 6 | SHA-256 `502e788b030ea0c56bb18bcf7d820b26e72ce02da3ada8734fb4c2198c8cfeaf` |
| focused direct/source-shape tests | 19 / 9,334 / 332,200 / 199 | SHA-256 `5e2dae85ccb0bacc7f85bde23685f59334cab52bf6fcb2fb70b2edc94d48651f` |
| reference corpus | 14 / 8,427 / 306,210 / 19 | SHA-256 `c8a37a108ee813b7c856893b935b2f63b5ff0111e98f0e1616ec0227c05477ee` |

fingerprint算法延续Runtime57：相对路径转`/`并排序去重，以`path|lowercase per-file SHA-256`组成LF连接且无末尾LF的UTF-8字节，再计算SHA-256。它只冻结本轮实际读取集合，不是bridge table或ABI的产品identity。

基线HEAD为`bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator baseline epoch为335。工作树存在大量其他会话/用户改动；本轮候选production文件保持只读，报告按当前working tree读取。共享代码与索引仍会变化，因此`source_recheck_required`保持true。

### 2.2 范围与去重

- Runtime07继续拥有resolver、统一PluginCatalogGeneration、跨native/static/script/content的prepare/publish/retire总事务和进程隔离。
- Runtime01/46拥有Core module生命周期和service装配；Runtime24拥有通用handle/generation规范；本篇只定义bridge具体identity和call lease。
- Plugins01拥有SDK、manifest、native ABI与distribution parity。`physics/plugin.toml`遗漏`physics.query.v1`而Rust descriptor提供它，是其manifest parity的具体输入，本篇不重复计P0。
- Plugins12/Runtime08A拥有Physics provider行为；AI行为树/感知算法由Runtime08F拥有。本篇只审其bridge消费语义。
- Tooling35拥有全仓ownership/callback lease，Tooling36拥有全仓type erasure，Tooling37拥有全仓transaction；本篇拥有这些控制面在plugin interface bridge的落地要求。
- Unreal `IModularFeatures`本身只是多provider/raw pointer注册器，不能作为hot-unload安全上限；本文只采用其注册/注销通知。卸载与retirement上限来自ModuleManager。
- Unity Graphics只用于`TypeId<T>`与container reset/reuse对照，不把Graphics ContextContainer错误外推为完整插件ABI。

## 3. 当前真实产品链

### 3.1 静态provider、import与冻结表

```text
RuntimePlugin registration
  -> RuntimeExtensionRegistry::export_interface<T>(owner, Arc<T>)
  -> RuntimeExtensionRegistry::import_interface<T>() -> BridgeImport<T>
  -> finalize/frozen_bridge_table()
       -> Vec<BridgeEntry>
       -> HashMap<String, InterfaceSlot(u32)>
       -> Arc<dyn Any> containing Arc<T>
       -> bind every InterfaceImport to a FrozenBridgeTable clone

BridgeImport::call
  -> ArcSwapOption<WeakBridge<T>>
  -> generation parity check
  -> cached Weak<T> upgrade or Any downcast refresh
  -> invoke consumer closure
```

表内generation与provider state使用单个`ArcSwap`快照发布，避免读到“新generation+旧provider”的撕裂，这是可保留基础。但slot无法证明来自哪张表；`from_exports`把`entries.len()`直接cast为`u32`；type mismatch、provider missing和disabled最后只剩`Absent/NotEnabled`两类错误。缓存命中后，lifecycle仍可在closure执行前关闭slot。

### 3.2 package lifecycle与dependency blocker

```text
ProjectPluginManifest required dependency
  -> RuntimePluginCatalog strong dependents
  -> RuntimePluginBridgeDisableBlocker

provider package event
  -> map package to all runtime modules
  -> enable/disable/deactivate/reload every owner slot
  -> BridgeOwnerTransitionReport
```

这里的“strong”来自manifest中`required=true`，不是当前live handle、task、callback或World system ownership。optional dependency完全不进入blocker；required与optional consumer拿到的又都是同一种`BridgeImport`。未知package或没有bridge export的package可以返回`Applied`，affected owner/slot均为零。`*_at_frame_boundary`只是命名，任何线程都可调用。

### 3.3 Native ABI与registration replay

```text
native entry report
  -> bridge method bindings: interface string + method string + fn pointer
  -> dense method directory
  -> NativeHostBridgeCallScope { bridge table, method directory, library owner }

registration manifest system
  -> resolve interface slot + method slot
  -> register_external_native_system
  -> World closure captures one NativeHostBridgeCallScope
  -> call(handle, interface_slot:u32, method_slot:u32, null input, empty output)
  -> discard ZrStatus
```

Host API call先读bridge status，再单独获取dynamic-library callback lease；两者不是一个原子admission。interface/method只用裸`u32`，没有table、ABI、schema、method signature或generation。registration replay能在单次build中把manifest、binding与loaded generation配对，这是好的局部实现；但产品启动链没有调用它，World也没有binding generation registry。

### 3.4 Editor Play与VM

Editor `NativePluginBridgeActivation::activate`会load project native runtime plugins、apply bridge lifecycle、enter play mode并复制diagnostics matrix；它没有拿到或更新RuntimeExtensionRegistry，也不replay native systems。App构建静态first-party bridge lifecycle state，但native load helpers同样没有把replay接入World。

VM的`register_bridge_host_module`能预解析interface slot并向脚本host注册method，不过production没有调用者。其真实dispatch是“先查询table status，再调用外部注入的Rust callback”；callback不是从bridge provider取得，method slot也由注册者自由选择。因此它验证的是availability gate，不是同一authoritative provider dispatch。

## 4. 可保留基础

- `BridgeEntry`把generation与provider放在单个immutable state内发布，避免明显的双atomic撕裂。
- frozen table和dense slot为热路径预解析提供了正确方向，不应退回每次字符串发现。
- `WeakBridge`允许consumer不永久强持provider，是可保留的默认observation语义。
- `BridgeImport`能在registry rebuild时rebind，已有从composition到consumer的更新入口。
- owner批量transition report与per-interface snapshot为后续transaction receipt提供了雏形。
- native `NativePluginStableLibrary`用transition bit和active callback count阻止活跃callback期间卸载。
- registration replay generation把manifest parse、access plan、method lookup与scope build缓存到plugin generation。
- native replay会验证system stage、set/order、component/resource access和worker capability。
- VM adapter预解析slot而不是每次脚本调用查字符串，方向正确。
- Editor已能消费统一matrix而不是分别查询每个插件，适合升级为revisioned observation stream。

这些基础证明局部generation publication和library pin存在，不证明bridge call与lifecycle同一事务、native system进入产品World、旧World可退休或diagnostics在shipping中可信。

## 5. P0 阻断项

| ID | 当前证据 | 工程后果 | 硬切目标 / owner |
|---|---|---|---|
| BRG-P0-001 | `WeakBridge::call`在generation检查后取得`Arc<T>`再执行任意closure；disable/deactivate可在两者之间发生。`BridgeGuard`/`StrongBridge`直接返回可长期保存的`Arc<T>`，没有call lease、admission close、in-flight drain或retire fence | lifecycle report显示Disabled/Deactivated时，旧provider仍可能执行并触及已退役module/state；依赖blocker只看manifest，不能证明实际调用停止。native status check与library lease也分属两次admission | 用`BridgeCallLease { table_generation, slot_generation, provider_generation, owner }`包围完整调用；disable/deactivate先close admission、drain lease到deadline，再publish/retire。长期pin必须是可撤销lease并进入holder census。Runtime58 + Runtime07/24/Tooling35 |
| BRG-P0-002 | `replay_runtime_registration_manifests_via_bridge`与单plugin版本除HostHandle转发和tests外无production caller；Editor Play只load、apply lifecycle、enter snapshot，App也只构建静态bridge state | native package即使声明runtime systems，普通App/Editor Play也不会把它们安装到RuntimeExtensionRegistry/World；“DLL loaded”和“runtime contribution active”被误当成同一产品事实 | 在统一PluginActivationTransaction的prepare阶段replay到staging extension generation，validate后一次publish到目标World/Session；缺replay、空system factory或apply失败必须使capability unavailable。Runtime58 + Runtime07/42 + Plugins01 |
| BRG-P0-003 | replay system closure捕获固定`NativeHostBridgeCallScope`；测试`native_registration_replay_keeps_old_binding_generation_alive_after_reinstall`认证旧World继续调用旧binding。成功native hot reload让旧library owner保持transition closed；closure仍强持owner，且system body丢弃所有`ZrStatus` | 已存在World不会切到新generation：旧system每帧静默no-op，旧DLL又因World closure强持而无法retire；reload report可成功但玩法逻辑已经停止 | 建立`WorldBridgeBindingRegistry`和generation-aware system trampoline；reload在safe point原子替换/撤销受影响system，旧generation只允许drain且每个失败返回typed execution receipt；所有World holder归零后才能卸载。Runtime58 + Runtime05/07/41 |

### 5.1 继承阻断，不重复计数

- Runtime07 P1-1/P1-2继续拥有catalog/extension/bridge/profile多authority和跨全部plugin contribution的非原子publication；本篇只登记bridge-specific split与World holder。
- Plugins01的SDK/manifest parity拥有`physics/plugin.toml`遗漏`[[provides_interfaces]] physics.query.v1`；Plugins12拥有Physics source/native provider parity。
- Tooling35/36/37分别拥有全仓ownership、type-erasure和transaction规则，本篇的验收必须消费它们而不复制全仓P0。

## 6. P1 工程化差距

### 6.1 Contract、Identity、Slot 与 Schema

| ID | 差距 | 目标 / owner |
|---|---|---|
| BRG-P1-001 | `InterfaceSlot`公开`from_raw/raw/index`且只有`u32` | 改为`InterfaceHandle { table_id, slot, generation }`，raw carrier仅留ABI内部 |
| BRG-P1-002 | interface identity只是一段自由字符串常量 | 由`InterfaceContractId(namespace,name,major)`和canonical registry生成 |
| BRG-P1-003 | trait没有method schema、argument/result、error或thread contract | 编译`InterfaceContractDescriptor`与method signatures、affinity、reentrancy、budget |
| BRG-P1-004 | export存`Arc<dyn Any>`内再包`Arc<T>`，刷新时downcast | 静态路径用typed slot factory；跨ABI路径用generated vtable，不以Any作为公共真值 |
| BRG-P1-005 | wrong type downcast映射为`BridgeError::NotEnabled` | 增加ContractMismatch、StaleHandle、ProviderRetired、MethodMismatch等稳定错误码 |
| BRG-P1-006 | `u32` generation以奇偶表达状态并`wrapping_add` | 使用非零宽代际、显式state和exhaustion policy；不得wrap后接受陈旧cache |
| BRG-P1-007 | `entries.len() as u32`无overflow admission | table build在超过slot policy时结构化拒绝并报告requested/limit |
| BRG-P1-008 | slot分配只在当前table构建顺序内有效，无table digest/serialization | generation携带稳定contract digest；外部不得持久化或跨generation复用slot |

### 6.2 Handle、Call、Pin 与 Outcome

| ID | 差距 | 目标 / owner |
|---|---|---|
| BRG-P1-009 | cached Weak升级成功后不二次确认generation/admission | lease acquisition与provider pin在同一state transition中完成 |
| BRG-P1-010 | `BridgeGuard`没有expiry、owner、generation或Drop回执 | guard变成tracked lease，Drop递减holder census并唤醒retirement |
| BRG-P1-011 | `StrongBridge`只是裸`Arc<T>`，没有允许场景或产品caller | 删除公共strong resolve，或仅允许声明为non-unloadable的engine-static provider |
| BRG-P1-012 | `is_enabled()`是瞬时检查，调用者可形成TOCTOU | API只提供`try_acquire_call`/`call`，health query明确标为observation非admission |
| BRG-P1-013 | Rust provider closure panic会越过bridge outcome accounting | 在host boundary隔离panic并按provider policy quarantine/fail-fast |
| BRG-P1-014 | enabled counter在callback完成前递增，不能区分成功/失败/panic | 记录attempt/success/error/panic/cancel/deadline及latency分布 |
| BRG-P1-015 | `BridgeImport::call`只返回provider closure结果，缺bridge execution context | 注入call id、session/world、deadline、cancel、trace和budget context |
| BRG-P1-016 | import rebind只有ArcSwap更新，没有revision/event/retry合同 | 发布`BridgeBindingChanged`流，consumer可观察old/new/reason并重建缓存 |

### 6.3 Table Authority、Freeze 与 Lifecycle

| ID | 差距 | 目标 / owner |
|---|---|---|
| BRG-P1-017 | `set_enabled`、`replace_provider`、`deactivate_owner`等直接公开 | mutator只由`BridgeLifecycleTransaction`持有，consumer得到只读generation |
| BRG-P1-018 | registry未finalize时`frozen_bridge_table()`可构造临时表却不发布binding | 未完成composition时返回NotFinalized，不产生平行table |
| BRG-P1-019 | registry revoke/rebuild只rebind已登记import；旧table clone仍可独立存活 | generation registry跟踪所有holder，旧table进入Retiring且不可新调用 |
| BRG-P1-020 | package映射到其全部runtime module并批量toggle，缺interface级计划 | transition plan精确列出package/module/interface/method及dependent closure |
| BRG-P1-021 | unknown package或零slot transition也可返回`Applied` | outcome区分MissingProvider、NoContribution、NoOp、Applied和PartiallyApplied |
| BRG-P1-022 | `*_at_frame_boundary`没有frame/safe-point token或线程检查 | 要求`RuntimeSafePointToken`、owner thread和phase proof |
| BRG-P1-023 | reload将同一registry同时作为current和replacement export源 | transaction显式持old/new generation，validate contract diff后交换 |
| BRG-P1-024 | lifecycle report无transition id、catalog/table revision和durable result | 生成revisioned `BridgeLifecycleReceipt`并接入session trace |

### 6.4 Dependency、Import Policy 与 Provider Selection

| ID | 差距 | 目标 / owner |
|---|---|---|
| BRG-P1-025 | disable blocker来自manifest `required=true`，不来自live holder | blocker合并declaration、resolved plan、call/task/world leases和retirement state |
| BRG-P1-026 | optional dependency不进diagnostics或disable影响面 | optional仍进入graph，availability change要通知并有degraded policy |
| BRG-P1-027 | required/optional consumer拿到相同`BridgeImport` | 类型化`RequiredImport`/`OptionalImport`，startup和runtime failure policy不同 |
| BRG-P1-028 | import只声明interface id，没有version/provider/feature/target约束 | import contract包含version range、provider selection、target与capability predicate |
| BRG-P1-029 | AI package把Physics/ZrVM列optional，registration却总是创建两项import | 根据resolved optional closure构建feature plan，并显式发布degraded reasons |
| BRG-P1-030 | AI perception对所有physics bridge错误调用`.ok()` | 区分Absent/Disabled/Stale/Fault并进入agent/world diagnostic policy |
| BRG-P1-031 | script behavior consumer把bridge错误压成Debug字符串 | 保留稳定code、provider/interface/generation和可重试性 |
| BRG-P1-032 | dependency report没有当前holder、last call、drain deadline或leak owner | 加入`BridgeHolderCensus`和阻断链，Editor可定位谁阻止reload/unload |

### 6.5 Native ABI、Method Directory 与 Replay

| ID | 差距 | 目标 / owner |
|---|---|---|
| BRG-P1-033 | native call先查bridge enabled，再独立acquire library callback | 单一native call lease同时pin table/provider/method/library generation |
| BRG-P1-034 | ABI只传interface slot与method slot两个`u32` | handle携带table/interface/method generation和ABI epoch校验 |
| BRG-P1-035 | bridge call层没有input/output/alloc/host-call预算 | method descriptor声明上限，host在进入DLL前admit并计量 |
| BRG-P1-036 | registration system发现`api.bridge.call=None`就静默return | 缺function pointer使system registration或execution fail-close |
| BRG-P1-037 | system调用后只计算一次未使用的status比较 | status进入World execution report、quarantine和reload health |
| BRG-P1-038 | method binding discovery错误降级成diagnostic + `Some(None)` | malformed replacement在prepare阶段拒绝，不发布“方法全部消失”的generation |
| BRG-P1-039 | live host先发布loaded plugin/binding，之后才apply bridge lifecycle | staging中联合验证并一次publish；bridge失败回滚replacement和binding |
| BRG-P1-040 | load report逐plugin apply activation，零owner也可成功 | activation验证expected contribution count和provider/interface closure |

### 6.6 Native Reload、World Binding 与 Retirement

| ID | 差距 | 目标 / owner |
|---|---|---|
| BRG-P1-041 | replay只作为手工API存在，调用者需自行选择registry/lifecycle | product activation拥有唯一staging registry和target session |
| BRG-P1-042 | World创建后没有记录它消费的plugin/binding generation | 每个World维护`WorldBridgeBindingSet`并参与reload impact analysis |
| BRG-P1-043 | replay closure强持library owner，World drop前旧DLL不能释放 | holder census可见World/system，reload可撤销system并等待确定性Drop |
| BRG-P1-044 | successful hot reload不重新开放旧owner，旧scope只会持续拒绝 | 旧generation进入明确Draining/Retired，system不得继续被scheduler调用 |
| BRG-P1-045 | `apply_to_world`是一次性复制，没有generation update协议 | schedule消费immutable generation handle并在safe point交换compiled schedule |
| BRG-P1-046 | extension registry缺external native system unregister/replace receipt | contribution有stable id、owner generation、replace/remove和rollback |
| BRG-P1-047 | batch hot update逐plugin发布，bridge report后补 | 依赖Runtime07总事务，一批provider与dependent作为一个publication unit |
| BRG-P1-048 | unload失败后bridge仅做activate rollback，不能证明exact state恢复 | rollback恢复old table generation、bindings、systems、counters与holder policy |

### 6.7 VM、Editor 与 Diagnostics

| ID | 差距 | 目标 / owner |
|---|---|---|
| BRG-P1-049 | VM bridge status来自table，执行callback却是另行注入的authority | VM method trampoline从同一contract/provider generation解析 |
| BRG-P1-050 | `register_bridge_host_module`没有production caller | language activation按resolved interface imports生成host module并记录receipt |
| BRG-P1-051 | VM注册者可为任意interface选择任意method slot | method slot由contract catalog生成并验证signature |
| BRG-P1-052 | VM在callback完成前记录enabled call，错误不进入bridge metrics | 统一typed outcome、fuel/time/bytes和script trap映射 |
| BRG-P1-053 | bridge counters仅`debug_assertions`启用，release恒定返回零 | shipping保留低开销采样和准确disabled/error计数，可按policy降采样 |
| BRG-P1-054 | matrix行包含raw slot/generation、Debug status和String diagnostics | 定义稳定DTO code/schema/table revision/provider artifact identity |
| BRG-P1-055 | Editor每次把matrix全量复制为String row，缺diff/pagination | revisioned delta stream、bounded page和retained history policy |
| BRG-P1-056 | diagnostics没有call/transition/world/session关联 | trace关联interface/method/provider/world/system/frame和reload transaction |

### 6.8 Product Reachability、Scale 与 Qualification

| ID | 差距 | 目标 / owner |
|---|---|---|
| BRG-P1-057 | production typed interface目前仅Physics query、Script behavior、AI node registry三项 | 建立contract catalog和成熟度矩阵；数量少不能证明体系已完成 |
| BRG-P1-058 | Physics Rust descriptor提供`physics.query.v1`，生成plugin.toml未声明 | Plugins01修复单一declaration生成链，并做source/file/embedded parity gate |
| BRG-P1-059 | table按interface id只保留单provider，缺多provider/priority/selection合同 | 明确single/multi/replaceable策略；需要多provider时提供stable enumeration与事件 |
| BRG-P1-060 | snapshot/matrix每次遍历全表并克隆interface id、rows和diagnostics | 提供immutable shared snapshot、delta和按owner/interface索引 |
| BRG-P1-061 | refresh路径仍做HashMap字符串查找、Any downcast和多次Arc clone | build期生成typed dispatch cell；用profile证明cache miss成本和内存局部性 |
| BRG-P1-062 | 没有ordinary App/Editor Play加载真实native system并跨World执行的E2E | 添加真实fixture DLL、产品启动、frame执行、reload、unload和restart证据 |
| BRG-P1-063 | 1/100/1,000 system benchmark全部`#[ignore]`且非持续基线 | 托管optimized-profile benchmark产出artifact metadata和回归阈值 |
| BRG-P1-064 | 缺active-call reload、stale slot、panic、hung call、1k provider、multi-world soak | 建立fault/stress/soak矩阵并绑定source/binary hashes与platform profile |

## 7. P2 质量与效率差距

| ID | 差距 | 改进方向 |
|---|---|---|
| BRG-P2-001 | interface id在entry、map、report和Editor row多次分配 | interned contract id与shared descriptor |
| BRG-P2-002 | diagnostics matrix每次全量排序/clone | revisioned immutable snapshot与delta cursor |
| BRG-P2-003 | owner transition为每个slot再构造完整snapshot | receipt引用generation snapshot和compact changed-slot set |
| BRG-P2-004 | WeakBridge每实例分配`Arc<ArcSwapOption<...>>` | import cell按contract consumer共享或内联存储，基准后决定 |
| BRG-P2-005 | cache miss同时升级Weak、provider downcast和Arc clone | typed dispatch cell保存validated vtable/provider lease factory |
| BRG-P2-006 | method directorypaged fanout没有规模/稀疏度策略说明 | 按method count选择small dense或paged布局并记录memory receipt |
| BRG-P2-007 | native empty input/output每帧仍穿完整ABI层 | generated zero-payload fast path但保留call lease与metrics |
| BRG-P2-008 | system closure每帧重复取得API/Option call | schedule compile时验证trampoline并保存不可变call plan |
| BRG-P2-009 | error diagnostics重复format/sort/dedup String | stable code + interned context，展示层最后格式化 |
| BRG-P2-010 | Editor status用`format!("{:?}")` | localized presentation映射稳定enum/code |
| BRG-P2-011 | release diagnostics全关导致性能和可观测性不可同时评估 | sampled counters、per-interface开关和开销基线 |
| BRG-P2-012 | no-op transition也分配report与diagnostic字符串 | compact typed outcome，按需materialize展示文本 |
| BRG-P2-013 | bridge table没有按owner的预编译slot range/index | build期建立owner->slot span/list，避免每次全表scan |
| BRG-P2-014 | holder census尚无compact call-site metadata策略 | stable call-site id与bounded top-N，不保留无界stack/string |
| BRG-P2-015 | benchmark只测replay build，不测steady call与reload hitch | 增加hit/miss/call/disable/drain/reload p50/p95/p99与alloc/RSS |
| BRG-P2-016 | reference比较没有同场景机器数据 | 同硬件、同provider/method/world规模比较Zircon旧/新路径，参考引擎只作结构基线 |

## 8. 参考引擎对照与适用边界

| 参考 | 可采用证据 | Zircon应吸收 | 不应错误复制 |
|---|---|---|---|
| Unreal ModularFeatures | 支持同feature多实现、注册/注销事件和查询 | provider selection policy与availability change event | raw pointer registry本身不是动态卸载安全上限 |
| Unreal ModuleManager | `PreUnloadCallback`、`SupportsDynamicReloading`、live object检查、GC等待、transaction commit后卸载、stale delegate检查、延迟/放弃卸载 | quiesce、holder census、retirement、rollback和安全地不卸载 | 不复制C++宏、全局singleton或module类层次 |
| Godot GDExtensionManager | reloadability/editor/recovery gate；prepare reload；反向deinit level；instance binding清理；重开库、重建class、恢复property/instance并通知 | instance/world holder追踪、prepare/deinit/rebind/restore/finish完整状态机 | 不把Godot initialization level直接映射为Zircon schedule stage |
| Fyrox DynamicPlugin | dylib复制隔离文件锁、tick末尾reload；scene/user data/plugin node/script状态迁移 | safe point、旧库文件generation、场景持有者迁移 | Fyrox明确标注unsafe/slow，不能作为最终性能和安全上限 |
| Bevy Plugin lifecycle | build -> ready -> finish -> cleanup和uniqueness | 启动阶段与ready gate分离 | Bevy核心Plugin不是hot-unload系统，不能据此证明reload安全 |
| Unity Graphics ContextContainer | `TypeId<T>` dense索引、reset清除stale reference并复用container storage | typed slot、generation reset和容器复用 | Graphics frame context不是跨DLL插件ABI，不能照搬其生命周期假设 |

Unreal的关键经验不是“所有module都必须卸载”。ModuleManager在shutdown场景会因析构/虚调用风险选择abandon code而不是强卸载；Zircon也应把Unloadable、RestartRequired、ProcessPinned作为明确policy，而不是让裸`Arc`偶然决定DLL何时释放。

## 9. 目标架构

```text
PluginResolutionPlan
  -> InterfaceContractCatalog
       contract id/version/schema/method/affinity/budget/error ABI
  -> BridgeGenerationBuilder
       providers + imports + dependency closure + native/VM adapters
  -> validate
       identity/type/schema/provider/method/world contribution
  -> BridgeLifecycleTransaction
       prepare -> close admission -> drain -> publish -> rebind -> retire
  -> BridgeTableGeneration
       immutable typed dispatch cells + table digest
  -> consumers
       RequiredImport / OptionalImport -> BridgeCallLease -> typed outcome
  -> WorldBridgeBindingRegistry
       compiled systems + generation holders + safe-point replacement
  -> BridgeObservationStream
       revisioned diagnostics + transition/call/holder receipts
```

### 9.1 Contract与handle

`InterfaceContractCatalog`是唯一schema source。Rust trait、native vtable/headers、VM host exports、manifest schema和Editor docs均由同一contract生成。`TypedInterfaceHandle`包含table、slot、contract和generation；slot只是当前generation内的紧凑索引，不再作为可序列化身份。

### 9.2 Call lease与lifecycle transaction

每次调用先原子取得call lease；lease同时pin provider、method adapter和native library。Disable/Reload/Unload关闭新admission，等待call/task/World holder到deadline；超时返回结构化blocker，不伪装Applied。新generation一次publish，consumer rebind和World schedule swap在同一safe point完成；旧generation只drain，不能继续调度。

### 9.3 Native与VM adapter

Native method由generated ABI descriptor定义signature、input/output ownership、budgets、panic/fault policy和ABI epoch。VM adapter从同一provider generation解析，不再把status table与任意callback拼接。无法安全卸载的backend明确标记ProcessPinned或RestartRequired。

### 9.4 Product与观测

App、Editor Play、source export和NativeDynamic都消费同一PluginActivationTransaction。Editor只展示receipt，不自己拼装真相。Diagnostics在shipping保留低开销计数，错误拥有稳定code；full strings、stack和高频样本按policy启用。

## 10. 分阶段重构计划

### M0：冻结合同、caller与capability truth

- 生成全部interface/provider/import/native/VM/product caller inventory。
- 修复Physics source/file/embedded manifest parity并标出NativeDynamic空贡献。
- 把registration replay未接产品和VM host未注册写入capability admission。

### M1：InterfaceContractCatalog与typed handle

- 定义contract/version/method/error/thread/budget schema。
- 生成Rust/native/VM绑定和table digest。
- 硬切裸slot持久化与公共`from_raw`调用。

### M2：BridgeCallLease与holder census

- 统一Weak/Strong/Guard/native callback lease。
- lifecycle close admission、drain、deadline、blocker chain。
- panic/error/cancel/deadline outcome进入稳定telemetry。

### M3：transactional generation publication

- current/replacement使用独立staging generation。
- provider、method binding、imports、diagnostics一次publish。
- Missing/NoOp/Applied/Blocked/Retired outcome可证明且可回滚。

### M4：World native replay与VM product integration

- registration replay进入App/Editor activation prepare阶段。
- 建立WorldBridgeBindingRegistry和safe-point schedule replacement。
- VM host modules按resolved imports自动生成并绑定同一generation。

### M5：diagnostics、scale与retirement

- revisioned observation stream、bounded Editor page和holder census。
- 旧table/provider/DLL最终归零；泄漏输出owner chain。
- steady-call、reload hitch、1k provider/1k system/multi-world基准达标。

### M6：产品资格与性能结论

- ordinary App、Editor Play、source/native export、restart完整通过。
- active-call reload、panic/hang/stale/fault/soak矩阵通过。
- 同硬件同workload数据达标后，才允许声称性能达到或超过Unreal。

## 11. 验收门禁

### 11.1 Contract与identity（G01-G08）

| Gate | 验收要求 |
|---|---|
| G01 | 所有interface来自唯一contract catalog；source/manifest/native/VM projection逐字段一致 |
| G02 | handle跨table、跨generation、wrong contract和exhausted generation均结构化拒绝 |
| G03 | method signature、affinity、reentrancy、budget和error ABI可机器校验 |
| G04 | table超过slot上限在build阶段拒绝，不发生usize到u32截断 |
| G05 | table digest和artifact/plugin generation进入activation receipt |
| G06 | wrong type不再映射为NotEnabled，稳定错误码覆盖所有失效原因 |
| G07 | single/multi/replaceable provider policy显式且有deterministic selection |
| G08 | 不存在可跨generation持久化的裸InterfaceSlot公共路径 |

### 11.2 Call lease与lifecycle（G09-G16）

| Gate | 验收要求 |
|---|---|
| G09 | 每次Rust/native/VM调用都持同一call lease覆盖完整执行窗口 |
| G10 | disable/deactivate关闭admission后无新调用进入，已有调用可drain或按deadline失败 |
| G11 | Strong/Guard只在明确non-unloadable policy下存在，其他pin进入holder census |
| G12 | active call、task、World system和Editor holder都能定位owner/call site/generation |
| G13 | lifecycle API要求safe-point token、owner thread与phase，不靠函数名约定 |
| G14 | Missing、NoContribution、NoOp、Applied、Blocked、Retired不混淆 |
| G15 | panic/error/cancel/deadline不会漏计、跨FFI unwind或留下半transition |
| G16 | generation exhaustion/ABA测试通过，旧cache永不重新变成有效 |

### 11.3 Native、VM与World（G17-G24）

| Gate | 验收要求 |
|---|---|
| G17 | ordinary App与Editor Play都能加载真实fixture DLL并执行manifest system |
| G18 | native interface/method handle验证table、contract、method、library和ABI generation |
| G19 | malformed/missing binding在prepare失败，不发布空method replacement |
| G20 | native status进入World/system health，任何silent return/no-op测试失败 |
| G21 | reload原子替换所有受影响World system，旧World不会继续调度旧scope |
| G22 | World drop/reload后旧scope、provider和DLL holder最终归零 |
| G23 | VM adapter从同一provider generation dispatch，production activation自动注册 |
| G24 | native/VM input/output/fuel/time/alloc/host-call预算在进入provider前执行 |

### 11.4 Transaction、dependency与diagnostics（G25-G32）

| Gate | 验收要求 |
|---|---|
| G25 | provider、bindings、imports、World schedule和diagnostics一次generation publish |
| G26 | 任一阶段失败只销毁staging或完整恢复old generation |
| G27 | required/optional依赖有不同startup/runtime policy和availability event |
| G28 | disable blocker同时包含resolved dependency和live holder，不只读manifest |
| G29 | optional provider消失时consumer按声明degrade，错误不被`.ok()`吞掉 |
| G30 | diagnostics携带稳定code、table/provider/method/world/session/transition revision |
| G31 | shipping counters非恒零，采样开销有上限与基准 |
| G32 | Editor通过bounded delta stream展示，不每次全表String clone |

### 11.5 Scale、fault与产品资格（G33-G40）

| Gate | 验收要求 |
|---|---|
| G33 | 1/100/1,000 provider和method的build、call hit/miss、memory基准持续运行 |
| G34 | 1/100/1,000 native system replay与safe-point swap有p50/p95/p99和hitch阈值 |
| G35 | 1/4/16 Worlds在active-call reload/unload下无停跑、泄漏或错generation |
| G36 | provider panic、hung call、poison、missing symbol、ABI mismatch和rollback fault均可恢复或隔离 |
| G37 | 10,000 reload与24小时soak后table/provider/DLL/World holder稳定 |
| G38 | App、Editor、source export、NativeDynamic和restart消费同一activation receipt |
| G39 | CI绑定source manifest、binary hash、target/profile、contract digest和benchmark artifact |
| G40 | 只有同硬件同场景CPU/RSS/p99/reload hitch数据达标，才关闭性能领先声明 |

## 12. Owner 路由与实施约束

| 范围 | canonical owner | Runtime58责任 |
|---|---|---|
| 全局plugin resolver/generation transaction | Runtime07 | 提供bridge-specific generation、call lease和World binding要求 |
| Core module lifecycle/service composition | Runtime01/46 | 消费safe-point、module owner和lifecycle observer，不另造module system |
| Scene/World schedule replacement | Runtime05 | 定义native bridge system generation与holder接口 |
| 通用handle/generation | Runtime24 | 复用exhaustion/stale规则并提供bridge复合identity |
| manifest/SDK/native ABI | Plugins01 + Interface01 | 提供contract schema和product reachability gate |
| Physics package parity | Plugins12 + Runtime08A | 路由`physics.query.v1`遗漏和source/native行为差异 |
| ownership/type erasure/transaction | Tooling35/36/37 | 在bridge域落地其控制面，不复制横向报告 |
| Editor bridge UI | zircon_editor后续owner | 只消费Runtime58的revisioned diagnostics/receipt |

实施时必须硬切旧路径。不能长期保留`InterfaceSlot::from_raw`公共兼容层、旧`StrongBridge`、双写新旧table、把typed错误再格式化回`String`作为内部协议，或让World同时运行旧/新system等待“最终一致”。若某native plugin不能满足安全卸载，明确标记RestartRequired/ProcessPinned，而不是以泄漏旧`Arc`模拟安全。

## 13. 复核与未执行项

本轮确认：

- core table/provider state局部原子发布和native library callback lease值得保留；
- `resolve_strong`与`WeakBridge::owned`目前只有测试caller，不能据此宣称产品强引用策略已验证；
- registration replay两项公开API没有production caller；
- VM bridge host注册没有production caller；
- native system closure丢弃callback status；
- Editor Play只load/apply lifecycle/enter snapshot，不replay systems；
- Physics source descriptor与生成manifest的provided-interface真值漂移；
- bridge release diagnostics返回零；
- 现有测试主动保留旧binding generation给旧World，但没有验证真实hot reload后的system replacement和DLL retirement。

未执行：Cargo check/test、fixture DLL build/load、Editor Play、App frame、sanitizer、Loom、Miri、Valgrind、soak和benchmark。报告是current-source静态review，不是实现完成或性能资格证书。

## 14. 当前状态

`review_complete / implementation_pending / product_qualification_blocked`

Runtime58首轮深审完成。下一步应先执行M0/M1：冻结全部interface/provider/import/native/VM/product caller inventory，建立唯一contract catalog，并把native replay/VM host不可达纳入capability truth；在call lease、World binding replacement和transactional publication完成前，不应继续扩展更多依赖该bridge的first-party功能。
