---
title: Runtime Plugin Interface Bridge、Slot、Generation、Strong/Weak、Native/VM、Lifecycle、Diagnostics 与 Product Integration 当前源码复核
category: zircon_runtime
report_id: Runtime115
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - zircon_runtime/src/core/framework/bridge
  - zircon_runtime/src/plugin/bridge.rs
  - zircon_runtime/src/plugin/bridge
  - zircon_runtime/src/plugin/extension_registry/register/bridge_registration.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_dependencies.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle_state.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/bridge_scope
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_host_handle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/loading.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs
  - zircon_runtime/src/script/vm/host/bridge_host_module.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_editor/src/core/play/plugin_activation/native.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/with_viewport/runtime_backend.rs
  - zircon_editor/src/ui/workbench/snapshot/data/bridge_diagnostics_snapshot.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_plugins/ai/runtime/src/perception/scan.rs
  - zircon_plugins/physics/runtime/src/plugin.rs
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
tests:
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge_performance_baseline.rs
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
  - docs/plans/optimize/zircon_runtime/58-runtime-plugin-interface-bridge-slot-generation-strong-weak-native-vm-lifecycle-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/72-runtime-core-lifecycle-registry-concurrency-shutdown-current-source-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Features/IModularFeatures.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Features/ModularFeatures.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/godot/core/extension/gdextension_manager.h
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/godot/core/extension/gdextension.h
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/ContextContainer.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99p · Runtime Plugin Interface Bridge 当前源码复核

## 1. 当前结论

Runtime58 的核心结论在当前源码上完整成立。Zircon 已经具有冻结 bridge table、dense slot、弱缓存、owner 级 activate/disable/deactivate/reload、required dependency blocker、native method directory、dynamic library callback lease、registration replay generation cache、VM host adapter 和 Editor diagnostics matrix。这些不是空壳，尤其 immutable table generation、预解析 slot、library pin、worker access plan 与并发 cache publication 都应保留。

但是这些局部能力没有形成一个可证明的工程级接口运行时。`WeakBridge::call`、native table status、library callback lease、World system closure、VM callback和module lifecycle分别拥有不同的admission时刻；disable/deactivate没有覆盖完整调用窗口，也没有holder census或drain。native registration replay仍只有测试caller，普通App与Editor Play不会把DLL声明的system安装到产品World。replay生成的system closure又强持固定旧call scope，reload后旧World继续调度已关闭generation并长期pin旧DLL。

产品接线断点比Runtime58时更明确：App确实构造并保存`RuntimePluginBridgeLifecycleState`，但retained Editor启动仍调用`NativePluginBridgeActivation::new`；唯一能携带lifecycle的`new_with_bridge_lifecycle`在production和tests之外没有任何caller。VM manager只注册builtin host modules，`register_bridge_host_module`也仍只有tests/source-shape fixture调用。Physics Rust descriptor继续声明`physics.query.v1`，生成`zircon_plugins/physics/plugin.toml`仍没有`[[provides_interfaces]]`。AI manifest把Physics与ZrVM列为optional，registration却无条件创建两项import，perception继续用`.ok()`吞掉全部bridge错误。

本轮账本保持 **3项本地P0 Open、64项P1 Open、16项P2 Open、40项Gate Fail**。Runtime07、Runtime24、Runtime72、Plugins01和Runtime Interface01的父问题继续由原报告唯一计数。本轮没有发现任何finding达到完整Partial或Closed条件；局部native callback lease、typed replay error、cache publication和AI/Physics算法改动不能替代统一call lease、activation transaction、World replacement与shipping diagnostics。

本轮只做review与计划记录，不修改production、tests、Cargo、ABI或参考源码，也没有运行Cargo、真实DLL产品链、active-call reload、multi-world soak、fault injection、sanitizer、profiler或同负载跨引擎benchmark。当前没有证据可以宣称bridge性能或稳定性达到、更不能宣称超过Unreal。用户要求暂缓tooling，本报告不规划现有脚本或工具链优化。

## 2. 当前源码冻结与可复现性

| 范围 | 文件 / 行 / 非空行 / bytes / `#[test]` / dirty | fingerprint / 选择规则 |
|---|---:|---|
| bridge core与catalog | **16 / 2,814 / 2,496 / 95,360 / 11 / 0** | `e306bb6833d7254f05a077e8a2303abbbf2981bd6a87742243e728ae64e15d7f`；core bridge、table/import/weak、registry registration、bridge dependency/lifecycle和interface validation |
| native bridge与registration replay | **14 / 6,177 / 5,674 / 228,105 / 21 / 0** | `14ae594d5a25d1852512c27fb60de02d46dfea50839df1d649eb17ce1caef344`；bridge scope、library owner、live host lifecycle/loading/reload/method/replay/report |
| VM、App、Editor与first-party consumer | **22 / 6,054 / 5,564 / 226,128 / 19 / 2** | `f016e0f20fb17cf71c61c7135c95036e446a334695a4ccdd32d5328a18b200f4`；VM host/manager、App entry、Editor Play/diagnostics、AI/Physics/ZrVM manifests与runtime adapters |
| focused tests | **17 / 8,959 / 8,147 / 318,089 / 192 / 0** | `446ff09f56f0a55bff116bf4b4ac8ab46a0dd0964b26aa45a22cab60c27b2ade`；bridge table/dependency/native scope/replay/VM/App/Editor/AI产品测试 |
| 五引擎显式参考 | **14 / 8,730 / 7,553 / 316,759 / 19 / 0** | `1a501f6a7e3199af01ad507f5da8054c57d73c3b8de252a3d437ea2eee5398fa`；Unreal modular/module、Godot extension、Fyrox dylib/plugin、Bevy app/plugin、Unity ContextContainer |

fingerprint算法为：仓库相对路径转`/`并排序去重；每个文件计算lowercase SHA-256；以`path|hash`按LF连接且末尾不追加LF，再对UTF-8 payload计算SHA-256。行数为物理行，非空行为trim后非空，test只统计`#[test]`与`#[tokio::test]`。focused tests中与native组重叠的bridge scope test按各组独立目的保留，数字不能相加冒充去重总数。

基线HEAD为`bee4c707b714738346b49bba15c59468b8bd9b39`，coordinator epoch为339。报告读取当前共享working tree；入选范围中的`zircon_plugins/ai/runtime/src/perception/scan.rs`与`plugin/registration.rs`含其他Session未提交修改，本轮只读取当前结果，不接管、不回退。`source_recheck_required`保持true。

## 3. Runtime58 后的真实变化

| 变化 | 当前证据 | 账本结论 |
|---|---|---|
| core bridge合同没有推进 | bridge core/catalog与native bridge/replay入选组均无working-tree改动；slot、generation、Strong/Weak、mutator和release diagnostics语义与Runtime58一致 | 3项本地P0及相关identity/call/lifecycle findings全部Open |
| App保存bridge lifecycle snapshot | `BuiltinEngineEntry`能返回`runtime_plugin_bridge_lifecycle_state()` | retained Editor启动不消费它，普通activation仍走无lifecycle构造；只能算局部底座，不能关闭产品接线finding |
| native replay cache与并发测试仍在 | manifest、binding、slot、call scope按revision缓存；interleaved reload测试约束cache一致性 | cache一致不等于World generation替换；测试还显式认证old World继续使用old binding |
| native callback lease仍是真实基础 | library owner以transition bit与active callback count阻止新foreign call | table/provider status与library lease分两次admit，长期World holder不计active callback，transition关闭后仍可无限pin DLL |
| first-party AI/Physics功能在推进 | AI perception与Physics query当前有其它Session算法改动 | interface contract、optional import、error policy、manifest parity、product replay均未改变 |
| benchmark仍非持续资格证据 | 1/100/1,000 native system/method六组benchmark全部`#[ignore]` | BRG-P1-063与scale gates保持Fail |

## 4. 当前真实产品链

```text
source plugin registration
  -> RuntimeExtensionRegistry export/import
  -> FrozenBridgeTable { String -> InterfaceSlot(u32) }
  -> Arc<dyn Any> provider + parity generation
  -> BridgeImport -> WeakBridge cache -> user closure

native plugin product
  Editor retained startup
    -> NativePluginBridgeActivation::new(no lifecycle)
    -> load DLL -> enter play snapshot
    -> no registration replay -> no native system in product World

  manual/test replay
    -> parse registration manifest
    -> cache fixed NativeHostBridgeCallScope + slots
    -> register external native system closure
    -> apply_to_world copies closure into World schedule
    -> reload publishes new loaded/binding generation
    -> old World still owns old closed library generation

VM product
  VmPluginManager -> register_builtin_host_modules
  register_bridge_host_module -> tests only
```

Rust weak call先读取slot generation，再升级cached provider并执行任意closure；disable可夹在generation读取和closure执行之间。native call先读取bridge status，再独立向dynamic library owner申请callback lease；两者不能证明同一个provider generation在整个调用窗口仍被admit。VM adapter只检查table status，实际业务callback来自调用者另行注入的`ScriptBridgeMethodFn`，因此status authority与执行authority也不是同一对象。

`RuntimePluginBridgeLifecycleState`的方法名带`at_frame_boundary`，但调用者不提供safe-point token、frame id、owner thread或schedule swap proof。reload还把同一个registry同时作为current与replacement export源。unknown package、没有runtime module或零slot的transition仍可形成Applied report；report没有transaction id、catalog/table revision或durable state。

## 5. 可保留底座

- 保留immutable table generation与预解析dense slot，但升级为qualified typed handle，不保留裸`u32`公共身份。
- 保留`BridgeImport`的共享binding cell与weak cache方向，但lease acquisition、generation check、provider pin和outcome accounting必须原子化。
- 保留native library callback transition bit、active callback count和generation owner；它应成为统一bridge call lease的一部分，而不是独立的第二道门。
- 保留registration manifest parse/access-plan/method lookup cache和并发publication测试；cache必须挂到activation generation，而不是永久挂到World closure。
- 保留required dependency graph、Editor matrix和typed native replay errors；它们分别升级为live holder graph、revisioned observation stream和稳定错误schema。
- 保留Bevy式build/ready/finish/cleanup分阶段、Unreal式safe abandon、Godot式holder迁移等结构经验，但不复制其语言ABI、全局singleton或对象模型。

## 6. Owner边界与继承阻断

- Runtime07继续唯一拥有跨catalog、extension registry、bridge、profile、native和VM的单一compiled generation及总publication transaction。本篇拥有bridge-specific call lease、table/provider generation、World binding replacement与adapter一致性。
- Runtime24拥有全仓identity/generation/exhaustion规则；本篇落地`InterfaceContractId`、`BridgeTableGeneration`、`ProviderGeneration`、`MethodHandle`和holder identity。
- Runtime72拥有Core module/service shutdown、owned runtime和真实active ledger；本篇要求bridge provider retirement消费其quiescence receipt，不重复登记静态App/Editor shutdown P0。
- Plugins01拥有SDK author contract、Physics file/source/embedded manifest parity、artifact trust与loader admission；本篇只把parity作为interface activation gate依赖。
- Runtime Interface01拥有通用FFI byte/status/handle/build identity；native bridge carrier必须复用它，不再发明另一套裸slot与status协议。

## 7. 本地P0阻断项

| ID | Status | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| BRG-P0-001 | Open | `WeakBridge::call`在generation检查后取得`Arc<T>`再执行closure；`BridgeGuard`/`StrongBridge`直接暴露长期`Arc<T>`。disable/deactivate无call admission close、in-flight drain、retire fence或holder census。native status与library lease也分两次取得。 | `BridgeCallLease { table_generation, slot_generation, provider_generation, owner }`覆盖完整Rust/native/VM调用；close admission后drain到deadline，所有长期holder可追踪、可撤销并阻断retire。 |
| BRG-P0-002 | Open | replay API除HostHandle转发和tests外无production caller。Editor Play只load/enter snapshot；retained启动又使用无lifecycle activation。 | `PluginActivationTransaction`在staging registry重放manifest、校验contribution并一次publish到目标session/world；缺replay或apply失败使capability不可用。 |
| BRG-P0-003 | Open | replay system closure捕获固定`Arc<NativeHostBridgeCallScope>`；测试认证old World在reinstall后继续调用old binding。真实old library transition关闭后，closure每帧被拒绝但仍强持DLL，system body还丢弃status。 | `WorldBridgeBindingRegistry + generation-aware trampoline + schedule safe-point swap`原子替换或撤销全部受影响system；旧generation只drain不再调度，typed execution receipt可见，holder归零后才卸载。 |

## 8. P1工程化差距总账

### 8.1 Contract、identity、slot与schema

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| BRG-P1-001 | Open | `InterfaceSlot`公开`from_raw/raw/index`且只有`u32`。 | `InterfaceHandle { table_id, slot, generation }`；raw carrier仅留ABI内部。 |
| BRG-P1-002 | Open | interface identity只是一段自由字符串常量。 | canonical `InterfaceContractId(namespace,name,major)`与registry。 |
| BRG-P1-003 | Open | trait无method schema、参数/结果、error、affinity或thread contract。 | 编译`InterfaceContractDescriptor`与generated signatures。 |
| BRG-P1-004 | Open | export以`Arc<dyn Any>`再包`Arc<T>`并在refresh downcast。 | source路径用typed dispatch cell，ABI路径用generated vtable。 |
| BRG-P1-005 | Open | wrong downcast映射为`NotEnabled`。 | 稳定区分ContractMismatch、StaleHandle、ProviderRetired、MethodMismatch。 |
| BRG-P1-006 | Open | `u32`奇偶generation并`wrapping_add`。 | 非零宽代际、显式state与checked exhaustion，禁止ABA。 |
| BRG-P1-007 | Open | `entries.len() as u32`无overflow admission。 | table build结构化拒绝requested/limit。 |
| BRG-P1-008 | Open | slot只对当前构建顺序有效，无table digest或serialization policy。 | activation receipt携contract digest；外部不得跨generation复用slot。 |

### 8.2 Handle、call、pin与outcome

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| BRG-P1-009 | Open | Weak cache升级后不二次确认generation/admission。 | lease acquisition与provider pin在同一state transition完成。 |
| BRG-P1-010 | Open | `BridgeGuard`无expiry、owner、generation或Drop receipt。 | tracked lease在Drop递减holder census并唤醒retirement。 |
| BRG-P1-011 | Open | `StrongBridge`只是裸Arc且无production caller。 | 删除公共strong resolve，或只允许non-unloadable engine-static policy。 |
| BRG-P1-012 | Open | `is_enabled()`是瞬时TOCTOU observation。 | admission只通过`try_acquire_call/call`；health query明确不可授权调用。 |
| BRG-P1-013 | Open | Rust provider panic可越过outcome accounting。 | host boundary隔离panic并按policy quarantine/fail-fast。 |
| BRG-P1-014 | Open | enabled counter在callback前递增，不能区分结果。 | attempt/success/error/panic/cancel/deadline与latency分布。 |
| BRG-P1-015 | Open | call无call id、session/world、deadline、cancel、trace或budget。 | 注入bounded `BridgeExecutionContext`。 |
| BRG-P1-016 | Open | import rebind只有ArcSwap更新，无revision/event/retry合同。 | 发布`BridgeBindingChanged` old/new/reason流。 |

### 8.3 Table authority、freeze与lifecycle

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| BRG-P1-017 | Open | `set_enabled/replace_provider/deactivate_owner`等直接公开。 | mutator只由`BridgeLifecycleTransaction`持有。 |
| BRG-P1-018 | Open | registry未finalize时可构造临时frozen table却不发布binding。 | composition未完成返回NotFinalized，不产生平行table。 |
| BRG-P1-019 | Open | rebuild只rebind登记import，旧table clone可独立存活。 | generation registry跟踪全部holder，旧table Retiring且拒绝新调用。 |
| BRG-P1-020 | Open | package映射全部runtime module后批量toggle，无interface/method计划。 | transition plan精确列出package/module/interface/method与dependent closure。 |
| BRG-P1-021 | Open | unknown package或零slot也可Applied。 | MissingProvider、NoContribution、NoOp、Applied、PartiallyApplied分离。 |
| BRG-P1-022 | Open | `at_frame_boundary`无safe-point token、frame或thread proof。 | 必须传`RuntimeSafePointToken`与phase/owner proof。 |
| BRG-P1-023 | Open | reload把同一registry当current与replacement。 | transaction显式持old/new generation并验证contract diff后交换。 |
| BRG-P1-024 | Open | lifecycle report无transition id、catalog/table revision与durable result。 | revisioned `BridgeLifecycleReceipt`进入session trace。 |

### 8.4 Dependency、import policy与provider selection

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| BRG-P1-025 | Open | disable blocker只来自manifest required边，不来自live holder。 | 合并declaration、resolved plan、call/task/world lease与retirement。 |
| BRG-P1-026 | Open | optional dependency不进disable影响面或availability event。 | optional graph有degraded policy和change notification。 |
| BRG-P1-027 | Open | required/optional consumer得到相同`BridgeImport`。 | `RequiredImport/OptionalImport`类型化不同failure policy。 |
| BRG-P1-028 | Open | import无version/provider/feature/target约束。 | contract含version range、selection、target和capability predicate。 |
| BRG-P1-029 | Open | AI manifest列Physics/ZrVM optional，registration仍无条件创建两项import。 | resolved optional closure生成feature plan与degraded reason。 |
| BRG-P1-030 | Open | AI sight对所有Physics bridge错误调用`.ok()`。 | 区分Absent/Disabled/Stale/Fault并进入agent/world diagnostics。 |
| BRG-P1-031 | Open | script behavior consumer把bridge错误压为Debug字符串。 | 保留stable code、provider/interface/generation和retryability。 |
| BRG-P1-032 | Open | dependency report无holder、last call、drain deadline或leak owner。 | `BridgeHolderCensus`供Editor定位unload blocker。 |

### 8.5 Native ABI、method directory与replay

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| BRG-P1-033 | Open | native call先查table status，再独立acquire library callback。 | 单一lease同时pin table/provider/method/library generation。 |
| BRG-P1-034 | Open | ABI只传interface slot与method slot两个`u32`。 | 验证table/contract/method/library/ABI generation。 |
| BRG-P1-035 | Open | bridge call层无input/output/alloc/host-call预算。 | method descriptor声明上限，进入DLL前admit与计量。 |
| BRG-P1-036 | Open | replay system发现`api.bridge.call=None`便静默return。 | registration或execution fail-close并产生typed terminal。 |
| BRG-P1-037 | Open | system调用后只计算未使用的status比较。 | status进入World execution health、quarantine和reload policy。 |
| BRG-P1-038 | Open | malformed discovered binding降级为diagnostic并发布`None`。 | replacement在prepare拒绝，不发布空method generation。 |
| BRG-P1-039 | Open | live host先发布loaded plugin/binding，再apply bridge lifecycle。 | staging联合验证后一次publish，失败回滚replacement和binding。 |
| BRG-P1-040 | Open | load report逐plugin activate，零owner也可成功。 | 验证expected contribution count与provider/interface closure。 |

### 8.6 Native reload、World binding与retirement

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| BRG-P1-041 | Open | replay仍是手工API，调用者自行选registry/lifecycle。 | product activation拥有唯一staging registry和target session。 |
| BRG-P1-042 | Open | World不记录消费的plugin/binding generation。 | 每个World维护`WorldBridgeBindingSet`。 |
| BRG-P1-043 | Open | replay closure强持library owner到World drop。 | census可见World/system，reload可撤销并等待确定性Drop。 |
| BRG-P1-044 | Open | successful reload不重开旧owner，旧scope持续拒绝。 | 旧generation明确Draining/Retired且不再被scheduler调用。 |
| BRG-P1-045 | Open | `apply_to_world`一次复制，无generation update协议。 | schedule消费immutable generation并在safe point交换compiled schedule。 |
| BRG-P1-046 | Open | external native system无unregister/replace receipt。 | contribution有stable id、owner generation、replace/remove/rollback。 |
| BRG-P1-047 | Open | batch hot update逐plugin发布，bridge report后补。 | provider与dependent作为Runtime07单一publication unit。 |
| BRG-P1-048 | Open | unload失败只activate rollback，不能证明exact state恢复。 | rollback恢复old table、bindings、systems、counters与holder policy。 |

### 8.7 VM、Editor与diagnostics

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| BRG-P1-049 | Open | VM status来自table，执行callback却来自另行注入authority。 | trampoline从同一contract/provider generation解析。 |
| BRG-P1-050 | Open | `register_bridge_host_module`仍无production caller。 | language activation按resolved imports生成host module并记录receipt。 |
| BRG-P1-051 | Open | VM注册者可为任意interface选择任意method slot。 | slot由contract catalog生成并验证signature。 |
| BRG-P1-052 | Open | VM callback前记录enabled，error不进bridge metrics。 | 统一typed outcome、fuel/time/bytes和trap映射。 |
| BRG-P1-053 | Open | bridge counter只在debug启用，release恒为零。 | shipping低开销采样与准确disabled/error计数。 |
| BRG-P1-054 | Open | matrix暴露raw slot/generation、Debug status和String。 | stable DTO code/schema/table revision/provider artifact identity。 |
| BRG-P1-055 | Open | Editor transition时全量复制matrix与String，无diff/page。 | revisioned bounded delta stream与history policy。 |
| BRG-P1-056 | Open | diagnostics无call/transition/world/session关联。 | trace关联method/provider/world/system/frame/reload transaction。 |

### 8.8 Product reachability、scale与qualification

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| BRG-P1-057 | Open | production typed interface仍只有Physics query、Script behavior、AI node registry三项。 | contract catalog和maturity matrix；数量少不等于架构完成。 |
| BRG-P1-058 | Open | Physics Rust descriptor提供`physics.query.v1`，生成plugin.toml仍未声明。 | Plugins01关闭source/file/embedded parity gate。 |
| BRG-P1-059 | Open | interface id只保留单provider，无multi/priority/selection合同。 | 显式single/multi/replaceable policy与stable enumeration/event。 |
| BRG-P1-060 | Open | snapshot/matrix每次遍历全表并clone id、row、diagnostic。 | immutable shared snapshot、delta与owner/interface index。 |
| BRG-P1-061 | Open | refresh仍做String HashMap、Any downcast与多次Arc clone。 | build期typed dispatch cell并profile locality/cost。 |
| BRG-P1-062 | Open | 无ordinary App/Editor Play真实DLL跨World E2E。 | fixture DLL、产品启动/frame/reload/unload/restart证据。 |
| BRG-P1-063 | Open | 六组1/100/1,000 replay benchmark仍全部ignore。 | optimized-profile托管benchmark、artifact metadata和threshold。 |
| BRG-P1-064 | Open | 无active-call reload、stale slot、panic/hang、1K provider、multi-world soak。 | fault/stress/soak矩阵绑定source/binary/platform profile。 |

## 9. P2一致性、效率与资格差距

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| BRG-P2-001 | Open | interface id在entry/map/report/Editor row多次分配。 | interned contract id与shared descriptor。 |
| BRG-P2-002 | Open | diagnostics matrix每次全量排序/clone。 | revisioned immutable snapshot与delta cursor。 |
| BRG-P2-003 | Open | owner transition为每个slot重建完整snapshot。 | receipt引用generation snapshot与compact changed-slot set。 |
| BRG-P2-004 | Open | 每个WeakBridge分配`Arc<ArcSwapOption<...>>`。 | 按consumer/contract共享或内联，基准后定。 |
| BRG-P2-005 | Open | cache miss同时Weak upgrade、downcast和Arc clone。 | typed dispatch cell保存validated lease factory。 |
| BRG-P2-006 | Open | paged method directory无small/dense/sparse规模策略。 | 按method count选择布局并记录memory receipt。 |
| BRG-P2-007 | Open | empty input/output每帧仍穿完整ABI层。 | generated zero-payload fast path仍保留lease和metrics。 |
| BRG-P2-008 | Open | replay closure每帧重复取API与Option call。 | schedule compile时验证并保存immutable call plan。 |
| BRG-P2-009 | Open | error重复format/sort/dedup String。 | stable code + interned context，presentation最后格式化。 |
| BRG-P2-010 | Open | Editor status使用`format!("{:?}")`。 | stable enum/code到localized presentation映射。 |
| BRG-P2-011 | Open | release diagnostics全关，性能与可观测性无法共评。 | sampled counter、per-interface policy与开销基线。 |
| BRG-P2-012 | Open | no-op transition仍分配report与diagnostic。 | compact typed outcome，按需materialize文本。 |
| BRG-P2-013 | Open | table无owner预编译slot range/index。 | build期owner到slot span/list索引。 |
| BRG-P2-014 | Open | holder census无compact call-site metadata策略。 | stable call-site id与bounded top-N。 |
| BRG-P2-015 | Open | benchmark只测replay build，不测steady call/reload hitch。 | hit/miss/call/drain/reload p50/p95/p99与alloc/RSS。 |
| BRG-P2-016 | Open | reference比较无同场景机器数据。 | 同硬件比较Zircon old/new；参考引擎只作结构基线。 |

## 10. 五参考引擎对照与适用边界

| 参考 | 当前源码证据 | Zircon应吸收 | 不应错误复制 |
|---|---|---|---|
| Unreal ModularFeatures | 同feature可注册多个实现，有register/unregister事件与implementation enumeration | provider multiplicity/selection policy与availability change event | raw pointer registry不是unload safety方案 |
| Unreal ModuleManager | `PreUnloadCallback`、dynamic reload policy、transaction commit后unload、GC后live-object检查、stale delegate检查；shutdown可选择abandon DLL code | quiesce、holder census、delayed retirement、rollback和ProcessPinned/RestartRequired policy | 不复制C++宏、global singleton或module class hierarchy |
| Godot GDExtensionManager | reload gate后执行prepare、反向deinit、清instance binding、重开library、finish reload，并跟踪reloadable instance binding | World/object holder inventory、prepare/deinit/rebind/restore/finish状态机 | initialization level不直接映射Zircon schedule stage |
| Fyrox DynamicPlugin | tick末尾reload、复制DLL避免文件锁、保持library owner、fill/register后发布；源码明确警告Rust dylib非生产安全上限 | safe point、per-generation library path和场景状态迁移 | 不把Rust trait-object ABI或全卸载策略当第三方SDK终态 |
| Bevy Plugin lifecycle | build、ready、finish、cleanup与uniqueness由App owner驱动 | activation阶段和ready gate分离，composition owner持生命周期 | Bevy core Plugin不是hot-unload参考 |
| Unity Graphics ContextContainer | `TypeId<T>` dense index，active index list，Dispose调用Reset并复用storage | typed dense slot、active-set retirement和reset复用 | frame context不是跨DLL plugin bridge，也没有provider reload合同 |

Unreal的关键经验不是“更积极地卸载DLL”，而是只有在对象、delegate、transaction和GC条件可证明时才卸载；不能证明时明确abandon。Zircon当前旧World closure既继续调度又永久pin关闭generation，既不是安全unload，也不是显式abandon policy。

## 11. 目标架构与硬切原则

```text
InterfaceContractCatalog
  -> compiled contract/method descriptors + digest
  -> BridgeTableGeneration
       -> TypedInterfaceHandle / MethodHandle
       -> ProviderGeneration + admission state
       -> RequiredImport / OptionalImport

PluginActivationTransaction
  prepare: package resolve + native/VM/source adapters + registration replay
  validate: contract/version/budget/dependency/contribution/world impact
  publish: provider + imports + World schedules + diagnostics as one generation
  retire: close admission -> drain call/holder leases -> remove systems -> unload/abandon

BridgeObservationStream
  -> stable typed receipts/deltas
  -> Runtime/App/Editor diagnostics consumers
```

必须硬切删除裸`InterfaceSlot`跨层传播、公共`StrongBridge`、直接table mutator、手工product replay和VM另行注入authority。不得通过compat re-export、shim guard或“新API包旧Arc”维持两套lifecycle。source、native和VM可以有不同adapter，但必须消费同一contract、activation generation、call lease和outcome schema。

## 12. 分阶段重构计划

### M0: 当前失败固化与owner冻结

- 为3项本地P0增加RED测试：call/disable race、ordinary Editor/App replay缺失、old World reload retirement。
- 冻结三个production contract与manifest parity；记录所有raw slot、strong resolve、direct mutator和manual replay caller。
- 与Runtime07/24/72、Plugins01、Interface01冻结owner和receipt接口，拒绝平行transaction。

### M1: Contract catalog与qualified identity

- 引入`InterfaceContractCatalog`、generated method descriptors、table digest和checked identity。
- 用typed dispatch cell替换public Any/downcast truth；wrong type和stale generation返回稳定错误。
- 实现single/multi/replaceable provider policy与required/optional import类型。

### M2: Call lease与lifecycle transaction

- 统一Rust/native/VM admission，lease覆盖完整执行窗口并关联owner/world/session/deadline。
- lifecycle只接受safe-point token；close admission、drain、retire和rollback产生durable receipt。
- 引入holder census，Strong/Guard只允许显式non-unloadable policy。

### M3: Native replay与World schedule generation

- activation prepare自动replay到staging registry，校验expected contribution。
- `WorldBridgeBindingRegistry`记录每个World/system generation并在safe point交换compiled schedule。
- replacement失败不发布；旧system停止调度、holder归零后unload或明确abandon。

### M4: VM adapter与diagnostics stream

- VM language activation按resolved imports自动生成adapter，method slot来自contract catalog。
- bridge outcome统一映射script trap、fuel/time/bytes/host-call budget。
- shipping sampled diagnostics、typed delta stream、pagination和trace correlation进入Editor。

### M5: Product、fault、scale与性能资格

- ordinary App、Editor Play、source/native/VM、export和restart消费同一activation receipt。
- active-call reload、panic/hang/malformed binding/stale slot/rollback/multi-world/soak全部托管运行。
- 只有同硬件同workload CPU/RSS/p99/reload hitch达到预冻结目标，才允许声称达到或超过Unreal。

## 13. 验收门禁

| Gate | Status | 验收条件 |
|---|---|---|
| BRG-G01 | Fail | 所有interface来自唯一contract catalog，source/manifest/native/VM逐字段一致。 |
| BRG-G02 | Fail | 跨table/generation/wrong contract/exhaustion handle均结构化拒绝。 |
| BRG-G03 | Fail | method signature、affinity、reentrancy、budget和error ABI可机器校验。 |
| BRG-G04 | Fail | slot超限在build拒绝，无usize到u32截断。 |
| BRG-G05 | Fail | table digest与artifact/plugin generation进入activation receipt。 |
| BRG-G06 | Fail | wrong type不再映射NotEnabled，稳定错误覆盖失效原因。 |
| BRG-G07 | Fail | provider multiplicity与selection显式且deterministic。 |
| BRG-G08 | Fail | 无可跨generation持久化的裸InterfaceSlot公共路径。 |
| BRG-G09 | Fail | 每次Rust/native/VM call都持同一lease覆盖完整执行。 |
| BRG-G10 | Fail | close admission后无新调用，已有调用可drain或deadline失败。 |
| BRG-G11 | Fail | Strong/Guard只用于non-unloadable policy，其他pin进census。 |
| BRG-G12 | Fail | active call/task/World/Editor holder可定位owner/call site/generation。 |
| BRG-G13 | Fail | lifecycle要求safe-point token、owner thread与phase。 |
| BRG-G14 | Fail | Missing/NoContribution/NoOp/Applied/Blocked/Retired不混淆。 |
| BRG-G15 | Fail | panic/error/cancel/deadline不漏计、不跨FFI unwind、不留半transition。 |
| BRG-G16 | Fail | generation exhaustion/ABA测试通过，旧cache永不复活。 |
| BRG-G17 | Fail | ordinary App与Editor Play执行真实fixture DLL manifest system。 |
| BRG-G18 | Fail | native handle验证table/contract/method/library/ABI generation。 |
| BRG-G19 | Fail | malformed/missing binding在prepare失败，不发布空replacement。 |
| BRG-G20 | Fail | native status进入World health，silent no-op测试失败。 |
| BRG-G21 | Fail | reload原子替换所有受影响World system。 |
| BRG-G22 | Fail | World drop/reload后旧scope/provider/DLL holder最终归零。 |
| BRG-G23 | Fail | VM从同provider generation dispatch且production自动注册。 |
| BRG-G24 | Fail | native/VM预算在进入provider前执行。 |
| BRG-G25 | Fail | provider/binding/import/World schedule/diagnostics一次publish。 |
| BRG-G26 | Fail | 任一阶段失败只销毁staging或完整恢复old generation。 |
| BRG-G27 | Fail | required/optional依赖有不同policy和availability event。 |
| BRG-G28 | Fail | blocker合并resolved dependency与live holder。 |
| BRG-G29 | Fail | optional provider消失按声明degrade，错误不被`.ok()`吞掉。 |
| BRG-G30 | Fail | diagnostics含stable code与table/provider/method/world/session/transition revision。 |
| BRG-G31 | Fail | shipping counter非恒零，采样开销有上限与基准。 |
| BRG-G32 | Fail | Editor通过bounded delta stream展示，不做全表String clone。 |
| BRG-G33 | Fail | 1/100/1,000 provider/method build、call、memory benchmark持续运行。 |
| BRG-G34 | Fail | 1/100/1,000 replay与safe-point swap有p50/p95/p99/hitch阈值。 |
| BRG-G35 | Fail | 1/4/16 Worlds active-call reload无停跑、泄漏或错generation。 |
| BRG-G36 | Fail | panic/hang/poison/missing symbol/ABI mismatch/rollback fault可恢复或隔离。 |
| BRG-G37 | Fail | 10,000 reload与24h soak后table/provider/DLL/World holder稳定。 |
| BRG-G38 | Fail | App、Editor、source export、NativeDynamic和restart共享receipt。 |
| BRG-G39 | Fail | CI绑定source manifest、binary hash、profile、contract digest和artifact。 |
| BRG-G40 | Fail | 同硬件同场景CPU/RSS/p99/reload hitch达标后才关闭性能声明。 |

## 14. Owner路由与实施约束

- Runtime115是Runtime58的current-source状态账本；Runtime58保留首次完整审查历史，后续实现进度写回本篇或新的里程碑记录，不改写历史证据。
- Runtime07实现single compiled generation和跨source/native/VM总activation transaction；Runtime115实现bridge table/call/World adapter具体合同。
- Runtime24提供identity/exhaustion公共规则；Runtime72提供module/service quiescence；Runtime05/41提供World schedule replacement和operation receipt依赖。
- Plugins01关闭Physics和其它package的declaration/file/embedded interface parity；Interface01提供generated FFI carrier和build identity。
- Editor只消费typed observation/transition receipt，不拥有第二套bridge state；retained host必须从App composition取得同一lifecycle owner。
- 不得先给旧`InterfaceSlot`加字段、给`StrongBridge`包一层guard、或在Editor临时手调replay。第一实现切片必须从M0 RED tests与M1 contract identity开始。
- tooling按用户要求排除；不得用新增Python/source-shape脚本冒充runtime contract闭合。

## 15. 复核与未执行项

本轮完成了当前working tree的静态源码复核、五参考引擎对照、物理指纹、Runtime58逐finding对账和40项门禁判定。未运行Cargo是有意的：本轮不改production或tests，且全工程MVP baseline仍未完成；历史或其他Session的Cargo结果不能冒充本轮证据。

进入实现前必须重新检查HEAD、coordinator epoch、AI/Physics当前dirty diff、Runtime07/72 lifecycle实现进度、Plugins01 manifest parity和App/Editor composition owner。任一实现若不删除旧manual replay、raw slot、direct mutator或untracked World holder旁路，不得计为本报告进度。
