---
title: Runtime Engine Module / Service Contract / Context / Factory / Descriptor Snapshot / Composition / Lifecycle / Product Integration 当前源码复审
category: zircon_runtime
report_id: Runtime135
review_date: 2026-08-24
baseline_head: 16122ac757cf3b2e60e43477bda6c5fa94c63ddb
baseline_epoch: 396
verification_head: 080fefe6acd449beded4497dee4a474b9e1f7383
verification_epoch: 402
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
related_code:
  - zircon_runtime/src/engine_module
  - zircon_runtime/src/core/runtime/descriptors
  - zircon_runtime/src/core/runtime/contexts
  - zircon_runtime/src/core/runtime/handle/registration
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/handle/activation
  - zircon_runtime/src/core/runtime/state
  - zircon_runtime/src/core/runtime/diagnostics/devtools.rs
  - zircon_runtime/src/builtin/runtime_modules
  - zircon_runtime/src/script/vm/module
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/script/vm/host/vm_plugin_host_context.rs
  - zircon_app/src/plugins
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_plugins
tests:
  - zircon_runtime/src/engine_module/tests.rs
  - zircon_runtime/src/builtin/runtime_modules/tests
  - zircon_runtime/src/core/runtime/tests/registration
  - zircon_runtime/src/core/runtime/tests/resolution
  - zircon_runtime/src/core/runtime/tests/activation
  - zircon_app/src/plugins/tests.rs
  - zircon_app/src/entry/tests
plan_sources:
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/46/2026-08-23-factory-panic-containment.md
  - docs/plans/optimize/zircon_runtime/99x-runtime-manager-resolver-named-service-handle-generation-lifecycle-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99s-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/zircon_runtime/runtime/02/failure-2026-07-17-module-descriptor-regeneration.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManifest.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ModuleDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ModuleDescriptor.cpp
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/plugin_group.rs
  - dev/godot/modules/register_module_types.h
  - dev/godot/core/extension/gdextension_manager.h
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/godot/core/extension/gdextension.h
  - dev/godot/main/main.cpp
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderPipeline/RenderPipelineGlobalSettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderPipeline/RenderPipelineGlobalSettingsUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/ResourceReloader.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime135 · Engine Module / Service Contract / Context / Factory / Descriptor Snapshot / Composition / Lifecycle / Product Integration 当前源码复审

## 1. 结论

Zircon当前模块底座不是空壳。Core已经能注册module/service descriptor，校验module与service依赖，冻结激活/反向卸载顺序，执行`build -> ready -> finish -> cleanup`，隔离module lifecycle panic，并维护service index、generation、admission和devtools snapshot。当前生产树共有23个文件、24个`EngineModule`实现；本轮逐个核对后，24/24的trait name、description与其descriptor当前文本一致。旧Runtime46记录的`AssetModule`说明漂移已经通过共享`ASSET_MODULE_DESCRIPTION`修复，动态`DescriptorBackedEngineModule`也不再`Box::leak`，`ResolvedPluginGroup`已有enabled/disabled/nested descriptor call-count测试。这些都是真实进展。

但模块系统仍没有形成工程级单一事实源。`EngineModule`继续公开`module_name()`、`module_description()`和可执行的`descriptor()`三个authority；`ModuleDescriptor`及三类service descriptor继续公开所有字段和opaque closure。Runtime profile选择时生成并缓存descriptor，却在`RuntimeModuleLoadReport`中只返回公开可改写的module bag；App又构造一次默认group，用Runtime module执行`set/add`，其中`set`清空descriptor，最终`try_finish()`再次调用author code。`ResolvedPluginGroup`仍是平行`modules/descriptors`向量，未校验每对identity；若两个module返回互换descriptor name，sorter会按module name重排并静默保存错配。Core还会在freeze前按module name重新排序descriptor，使无依赖同level模块的App report顺序和真实激活顺序可不同。

`EngineService`接口族仍只有定义和本目录测试消费者。Core registration、resolver、devtools与diagnostics全部绕过它直接读取descriptor/state；公开constructor允许`owner_module`与registry owner矛盾，开放marker trait允许kind语义由实现者自报。它目前不是compiled contract，而是一套已经进入prelude、却没有生产authority的平行元数据表面。

Runtime46的M0工厂panic修复已出现在共享工作树：`ServiceInitializationClaim`用Drop复位slot，统一`invoke_service_factory()`用`catch_unwind`把panic转成`ServiceFactoryPanicked`，新增Immediate、Lazy manager、Lazy plugin和单waiter重试测试。方向正确，但改动仍属于状态为`registered`的`optimize-runtime46-factory-panic-containment-r2-20260823`会话，源码和4个测试尚无受管验证或独立验收，因此原P0只能从Open重判为 **Partial**。M0也没有解决更深的发布错误：factory返回的`Arc<dyn Any>`先写入slot并设为`Running`，调用者之后才`Arc::downcast`；错误类型会成为永久Running实例。普通factory error仍被`to_string()`压成`Initialization(String, String)`，panic payload、binding、stage、generation和cause chain均丢失。

产品生命周期也不完整。默认App bootstrap使用零`ready_timeout`，任何第一次`ready=false`的模块立即失败；显式timeout路径每1 ms阻塞线程轮询。`ModuleContext`与`PluginContext`字段全公开，无composition/runtime generation、phase、transaction、deadline、cancel或grants；VM manager继续克隆基础context并直接修改三个PathBuf，公开`with_vm_owner`还可复制context后改写slot/generation。`Debug`会输出包含roots的plugin context。Core plugin factory得到完整`CoreWeak`，可解析未声明服务，dependency graph不是capability boundary。

Runtime46的1项P0本轮为 **0 Open、1 Partial、0 Closed**；48项P1为 **30 Open、18 Partial、0 Closed**；12项P2全部Open；36项资格门为 **22 Fail、12 Partial、2 Pass**。本文不新增finding，也不替Runtime01/03/07/24/42/45、App01或Plugins01重复计数。`module-descriptor-regeneration` failure继续保持open；没有managed validation、reclamation和report/activation parity证据，不能改名fixed，更不能声称模块系统性能或表现优于Unreal。

## 2. 审查边界、方法与currentness

### 2.1 冻结物理范围

统计口径：物理行、非空行、文件bytes；test declaration匹配`#[test]`，ignored匹配`#[ignore`；fingerprint为normalized lowercase path排序后，对每个文件拼接`path + NUL + lowercase(file SHA-256) + LF`再做SHA-256。三个Zircon组按路径去重后互斥；记录与参考另计。`zircon_plugins/plugin_sdk/src/registration.rs`已逐行审查，但在收尾期间被其他registered会话反复改写，无法形成原子current snapshot，因此从冻结统计与fingerprint中剔除；正文裁决不依赖该文件的瞬时版本。受其影响的原composition、Zircon union与combined aggregate fingerprint一并撤回，不把瞬时哈希包装成可复核证据。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| `engine_module`门面、24个生产实现及其descriptor owner | **34 / 2,882 / 2,569 / 100,080 / 10 / 0** | `2e27d5df161525b67a321b0bc75dc4e61963df8ffd710da9dea2b124e9fc08e5` |
| Core descriptor/context/registration/resolution/lifecycle与focused tests | **99 / 17,234 / 16,034 / 676,927 / 136 / 2** | `51ac3ac000b32487e722685d97f8cbdcc4bacb0bee9d9b84775ef9ef91474c0a` |
| Runtime/App composition、VM context、plugin declaration与产品测试（剔除并发`registration.rs`） | **85 / 11,406 / 10,491 / 430,350 / 133 / 1** | withdrawn：原aggregate受并发写入污染 |
| Zircon selected union | **218 / 31,522 / 29,094 / 1,207,357 / 279 / 3** | withdrawn：composition aggregate未接受 |
| 旧报告、M0记录与开放failure | **3 / 624 / 501 / 59,839 / 0 / 0** | `3537c0f8a923e105347840b260858fe6fe2cdf728abe9a82bfafc091c4deb1eb` |
| 五引擎参考选择集 | **19 / 15,850 / 13,659 / 588,187 / 35 / 0** | `d83e47ccbcec01ea67e330c5e3b7ed585d8f64ccf7bdd053254e46e52a218374` |
| selected combined scope | **240 / 47,996 / 43,254 / 1,855,383 / 314 / 3** | withdrawn：Zircon union aggregate未接受 |

参考revision：Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`。`dev/UnrealEngine`没有独立Git元数据，`git -C`会向上解析到Zircon仓库；因此Unreal只由所列6个物理文件和参考集合fingerprint冻结，不伪造revision。

### 2.2 检查方法

1. 逐个读取23个生产owner文件中的24个`EngineModule`实现及其分离descriptor builder，核对name、description、dependency、startup mode、factory capture和动态文本所有权。
2. 逐文件读取`engine_module`门面、Core descriptor/context、module graph、registration、resolution、activation、state、diagnostics及相应registration/resolution/activation tests。
3. 沿`Runtime profile -> descriptor cache -> load report -> App default group -> set/add -> try_finish -> EngineEntry report/bootstrap -> Platform binding patch -> Core register/freeze/activate`追踪每次物化、排序、clone和authority变化。
4. 反向搜索`EngineService`、三个contract helper、`module_context()`与`plugin_context()`的全产品caller；前两类仍为零生产consumer，`plugin_context()`只有VM module生产caller。
5. 读取VM package context派生、host context、slot owner和capability路径；区分Core plugin service、VM package、native plugin边界，不把相邻Runtime07问题重复记账。
6. 对Runtime46的1项P0、48项P1、12项P2和36项gate原编号逐项重判，并检查M0共享工作树diff和4个新增测试；未接管或修改该会话拥有的源码。
7. 对照Unreal module manager/descriptor、Bevy plugin lifecycle/group、Godot module/GDExtension phases与reload、Fyrox registration/runtime context与dynamic reload、Unity Graphics global settings/resource reload。

### 2.3 currentness与共享工作树

- 本报告注册基线为`16122ac757cf3b2e60e43477bda6c5fa94c63ddb` / epoch 396；最终verification为`080fefe6acd449beded4497dee4a474b9e1f7383` / epoch 402。两个HEAD之间240个冻结selected path没有committed变化；中间提交只改变本文范围外的Render failure记录。共享工作树的`zircon_plugins/plugin_sdk/src/registration.rs`在最终重扫前累计增加21 bytes，且同一次closeout观察中发生多次重写；它因此不属于上述240个冻结path。当前实现去除scene-system factory的一层dyn/Arc间接并按stage选择默认clock domain，不改变本文模块contract裁决。
- 协调器基线状态为degraded：注册时记录3,442个未accept workspace changes，closeout最终记录3,426个；审查期间另有3,481个和`git status --porcelain` 2,849条两个观察值。这些是不同观察时点，不把共享工作树漂移吸收到baseline。
- Runtime46 M0的5个Rust路径和1个新增测试仍是未集成改动；其会话状态为`registered`，本文只读审计，不覆盖、不归因、不宣称验证通过。
- 本轮只修改本报告和三个索引，不修改Rust、Cargo、旧Runtime46计划、M0记录或failure；不运行Cargo、App、Editor、fresh process、panic/fault soak或benchmark。
- 最终交付前必须重算可原子冻结的子集fingerprint并核对HEAD/epoch；任何并发改写的集合只保留明确排除记录，不发布aggregate fingerprint。共享路径若变化，只更新currentness证据，不回退其他会话改动。

## 3. 当前拓扑、可达性与断路

```text
EngineModule owner
  -> module_name()                 authority A
  -> module_description()          authority B
  -> descriptor()                 authority C + opaque closures

Runtime profile selection
  -> materialize descriptor generation 1
  -> dependency closure + local sort cache
  -> RuntimeModuleLoadReport { public modules, no descriptors/generation/hash }

App BuiltinEngineEntry
  -> independently build Default/Dev/Headless group (more descriptors)
  -> set(Runtime module) clears cached descriptor
  -> try_finish() materializes again
  -> ResolvedPluginGroup { parallel modules, descriptors }
  -> report reads stored descriptors
  -> bootstrap clones descriptors
       -> string-match PlatformModule/PlatformDriver and replace factory
       -> Core register raw public descriptor

Core
  -> clone registered descriptors from HashMap
  -> sort modules alphabetically
  -> freeze topology-only graph
  -> activate/build/ready/finish
  -> factory returns Arc<dyn Any>
  -> publish Running
  -> caller downcasts afterwards                         [wrong-type late failure]
```

| 边界 | 当前可保留事实 | 仍然断开的工程合同 |
|---|---|---|
| author surface | 24个生产实现当前name/description parity成立 | parity靠作者约定，API仍能表达三份互相矛盾的真值 |
| profile selection | dependency closure内descriptor只求值一次 | cache被load report丢弃，App不能消费同一snapshot |
| App group | enabled/disabled/nested局部call-count测试存在 | base group二次解析，replacement清cache，pairing不校验，generic entry仍重算 |
| Core graph | module/service依赖、kind方向、反向卸载顺序会校验 | 只保留拓扑，无graph id/hash/build/source/binding；重新排序可与report不同 |
| service resolution | generation/index、wait graph、M0 RAII panic reset底座存在 | expected type在publish后才检查；无deadline/cancel/binding/capability/cause |
| lifecycle | build/ready/finish/cleanup及panic rollback存在 | 默认ready budget为0，显式budget用1 ms阻塞轮询，context无phase/generation |
| diagnostics | devtools列出module/service name、count、state | 无composition、binding、provider、package、source、last transition可关联receipt |
| plugin/VM | VM host context有CapabilitySet和private主constructor | 内嵌可伪造PluginContext、raw roots/full Core、public owner改写与Debug路径泄露 |

## 4. 必须保留的工程基础

1. 保留Core的module/service依赖校验、init-level约束、确定性拓扑意图和逆序卸载计划，但让它们消费compiled graph而不是再编译公开field bag。
2. 保留`ModuleLifecycle::build/ready/finish/cleanup`分阶段语义、panic rollback和deactivation call drain；ready必须改为wake/ticket协议而不是阻塞轮询。
3. 保留service index/generation、initialization owner、wait dependency graph、call admission和stale handle拒绝；Runtime24/50继续拥有identity与调用期guard父边界。
4. 保留M0的RAII initialization claim与Core内no-unwind trampoline，补齐受管验证、并发矩阵、typed cause和所有terminal。
5. 保留`RegistryName::new`的fallible serde入口与缓存offset，但统一ModuleId/ServiceId规范并删除公开panic构造路径。
6. 保留profile typed enum、manifest availability与完整依赖闭包；最终输出必须是可共享compiled receipt，不是module bag。
7. 保留`ResolvedPluginGroup`局部一次descriptor snapshot和dynamic wrapper owner-borrowed文本；把module+descriptor收敛为单个validated entry。
8. 保留devtools从真实Core registry投影状态的方向；声明与运行态通过graph generation + ServiceId关联，不复制第二registry。
9. 保留VM host context已有的CapabilitySet、host export registry和private主constructor；删除内嵌通用PluginContext和public owner伪造面。
10. 保留模块作者的轻量Rust trait体验，但trait只提交一个proposal/registration object，不继续维护name/description/descriptor三authority。

## 5. P0当前源码裁决

| ID | 状态 | 当前源码证据 | 未完成的验收 |
|---|---|---|---|
| MOD-P0-001 | Partial | 共享工作树已有`ServiceInitializationClaim`、factory `catch_unwind`、typed `ServiceFactoryPanicked`及Immediate/Lazy manager/Lazy plugin/单waiter 4个测试；panic/error路径会按index+generation+owner复位并notify | 变更尚未集成或受管验证；缺8/64 waiter、dependency panic、hung/cancel/deadline、runtime shutdown/fresh process和独立review。panic payload/stage/binding/generation未保留，不能Closed |

P0修复不得移到App外层。slot已经在Core内变为`Initializing`，只有Core持有完整index/generation/owner并能正确reset/notify。当前M0方向应保留，但原会话必须先完成自己的验证与closeout。

## 6. P1当前源码重判

### 6.1 Module authority、snapshot与composition连续性

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| MOD-P1-01 | Open | trait name与descriptor name仍是两份身份；Runtime/App只在后续sort时偶然暴露部分mismatch | 单一`ModuleProposal/ModuleId` authority，所有key由validated id派生 |
| MOD-P1-02 | Partial | Asset说明已改为共享常量，24/24实现当前文本一致；独立trait description仍存在 | 删除第二description authority，diagnostic/display只读proposal |
| MOD-P1-03 | Partial | Runtime profile与ResolvedPluginGroup各有局部一次求值测试；端到端仍多代materialization | compiler每个enabled owner只调用一次author code，之后只传Arc snapshot |
| MOD-P1-04 | Partial | profile选择持有`descriptors_by_name`并复用于本地sort；`RuntimeModuleLoadReport`仍只返回modules | report返回`Arc<CompiledModuleGraph>`或同代proposal+diagnostic receipt |
| MOD-P1-05 | Open | App先build Default/Dev/Headless group，再用Runtime report modules set/add | Runtime composition compiler唯一执行profile/target/plugin selection；App只提交输入 |
| MOD-P1-06 | Partial | nested group会传递已解析descriptor；普通`set()`仍清空descriptor并重新调用owner | replacement接收validated entry，不退回trait重算 |
| MOD-P1-07 | Partial | BuiltinEngineEntry覆盖方法并返回group snapshot；generic EngineEntry默认实现每次重算 | EngineEntry只返回共享compiled receipt，删除漂移默认实现 |
| MOD-P1-08 | Partial | finish前`PluginEntry`短暂配对module/descriptor，但随后unzip为平行Vec且不校验identity；双module交换descriptor可静默错配 | `ResolvedModuleEntry { owner, compiled }`单对象并在构造时typed校验 |
| MOD-P1-09 | Open | Module/Driver/Manager/Plugin descriptor字段全部public，bootstrap可直接改factory | author proposal可构造，compiled对象字段private且只读Arc slice |
| MOD-P1-10 | Open | report读默认Platform factory，bootstrap注册字符串patch后的factory | report/Core共享同一`CompiledBindingSet`与binding hash |
| MOD-P1-11 | Open | 稳定metadata、lifecycle trait object和opaque factory closure仍混在可clone descriptor | 分离proposal、factory/lifecycle binding key与compiled declaration |
| MOD-P1-12 | Open | report/graph无composition generation、BuildSet、source/package/provider或digest | compiled receipt携generation、hash、provenance和diagnostic digest |

### 6.2 EngineService contract收敛

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| MOD-P1-13 | Open | `EngineService`及三marker除定义/re-export/test外无生产consumer | Core registration、devtools、diagnostics消费统一compiled service view |
| MOD-P1-14 | Open | Driver/Manager/Plugin Contract继续clone registry name和dependency Vec | contract成为compiled declaration的borrowed/Arc view |
| MOD-P1-15 | Open | public helper任意接收`owner_module`，可与RegistryName owner矛盾 | owner从ModuleId+ServiceId派生，矛盾状态不可构造 |
| MOD-P1-16 | Open | wrapper hardcode kind，RegistryName又缓存kind | typed declaration只编码一次kind，字符串仅作projection |
| MOD-P1-17 | Open | marker trait未sealed，外部实现可自报不一致kind | sealed typed contract/associated type，加入compile-fail test |
| MOD-P1-18 | Open | contract和descriptor均无expected service type identity | declaration记录stable ServiceTypeKey及进程内TypeId校验策略 |
| MOD-P1-19 | Open | 无factory binding、thread affinity、panic和startup preparation policy | `FactoryBindingKey + ExecutionPolicy`进入compiled graph |
| MOD-P1-20 | Partial | Core state已有index/generation/lifecycle/admission；EngineService没有关联这些状态的runtime view | immutable declaration与generation-bound state通过同一ServiceId join |
| MOD-P1-21 | Open | Core公开`register_module(ModuleDescriptor)`并直接读取field bag | 产品Core只接受validated compiled graph/entry |
| MOD-P1-22 | Open | 未生效contract/helper已经进入runtime prelude | 降低实验面可见性，最终只导出有owner/consumer/version政策的API |

### 6.3 Identity、声明完整性与诊断

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| MOD-P1-23 | Partial | 有`RegistryName`与builtin typed module enum；通用module/dependency仍是String | 统一ModuleId/ServiceId fallible/const构造与serde/fuzz规范 |
| MOD-P1-24 | Partial | `RegistryName::new`对数据返回typed error；`from_parts/qualified_name`仍assert | data-driven路径全部fallible，静态路径使用const验证 |
| MOD-P1-25 | Partial | Core freeze会校验完整service graph；App composition只校验module order，错误到commit后才发现 | compile transaction在Core mutation前汇总module+service diagnostics |
| MOD-P1-26 | Partial | 五个InitLevel提供基础阶段；无host/target/explicit-load/restart/reload政策 | typed host/load/reload policy进入proposal并由compiler裁决 |
| MOD-P1-27 | Open | declaration缺version、required/optional、conflict、capability和provider provenance | typed requirement/provision/conflict/provider求解 |
| MOD-P1-28 | Partial | devtools已有name/description/count/state；无graph/binding/source/generation join key | 投影graph id、provider、binding、runtime generation和last transition |

### 6.4 Context authenticity、factory binding与类型边界

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| MOD-P1-29 | Open | public`module_context()`可用任意String/CoreWeak伪造；生产caller为0 | constructor仅Core transaction可见，字段private、只读accessor |
| MOD-P1-30 | Open | ModuleContext仍只有String和CoreWeak，每个callback重新分配String | 加module/graph/runtime generation、phase、transaction、deadline/cancel/grants |
| MOD-P1-31 | Partial | 已有独立VmPluginHostContext，但它仍内嵌/暴露通用PluginContext | Core factory与VM package context物理分域，只共享最小identity value |
| MOD-P1-32 | Open | Core plugin context把完整service registry name放进`plugin_name` | ServiceId、PackageId、PluginSlotId与display name分型 |
| MOD-P1-33 | Open | Core plugin roots固定None，内建service仍使用package-shaped context | 内建Core plugin与verified package factory使用不同context |
| MOD-P1-34 | Open | VmPluginManager clone基础context后直接改三个public root字段；detached模式制造立即失效的CoreWeak | 从verified package source一次构造immutable generation-bound context |
| MOD-P1-35 | Open | raw PathBuf可clone、可Debug，未表达canonical scope和权限 | scoped root capability handle，防escape/symlink/reload越权 |
| MOD-P1-36 | Open | ServiceFactory只收CoreWeak，不知道service/owner/generation/binding | sealed ServiceFactoryContext + declared-dependency resolver |
| MOD-P1-37 | Open | factory可通过完整CoreWeak解析任何服务，未声明edge不构成拒绝 | compiled grants限制resolver，越权稳定Denied并记录correlation |
| MOD-P1-38 | Partial | VM host context已有CapabilitySet；Core PluginContext仍暴露完整Core和raw roots，`with_vm_owner`可改slot/generation | owner-issued不可伪造package capability table与generation token |
| MOD-P1-39 | Open | factory instance先写slot/Running，typed resolve之后才downcast；wrong type污染slot | commit前验证expected type，错误实例永不publish |
| MOD-P1-40 | Open | factory只有同步返回；默认module ready timeout=0，非零路径1 ms sleep轮询 | bounded prepare/ready ticket、wake、cancel/deadline与统一terminal |
| MOD-P1-41 | Partial | panic有typed variant；普通factory error仍to_string，panic payload与stage/binding/generation丢失 | 保留typed cause chain、retryability、stage、binding、generation、correlation |
| MOD-P1-42 | Partial | M0建立统一panic trampoline；closure仍无binding id/source hash/generation/thread policy | FactoryBindingRecord +受控execution cell，report/Core/crash证据可join |

### 6.5 Host binding、reload与资格证据

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| MOD-P1-43 | Open | App按module/service字符串查找PlatformDriver并替换factory | compiler前提交typed host binding并验证唯一consumer |
| MOD-P1-44 | Open | module declaration无dynamic reload/automatic shutdown/restart/state migration policy | 声明reload tier；不支持时明确RestartRequired |
| MOD-P1-45 | Partial | Asset drift已修，24/24本轮人工parity；自动测试仍只覆盖fixture而非全部生产owner | 迁移期间建立全owner invariant suite，最终由单authority消灭比较点 |
| MOD-P1-46 | Open | EngineService测试仍只检查driver metadata复制，无Core/devtools矛盾输入 | driver/manager/plugin/type/binding/Core consumer conformance suite |
| MOD-P1-47 | Partial | group call-count与dynamic borrow源码测试存在；无Runtime->App->Core总数、activation parity、reclamation或managed validation | 完成开放failure的1/100/1,000 create/query/bootstrap/drop/reload证据 |
| MOD-P1-48 | Open | 无1/100/1k/10k graph的物化、clone、RSS、fault和竞争benchmark | source-bound calls/allocs/bytes/p50/p95/p99矩阵，正确性先行 |

P1统计：**30 Open、18 Partial、0 Closed**。Partial只代表局部底座或未验收源码存在，不代表产品合同通过。

## 7. P2当前源码重判

| ID | 状态 | 当前源码证据与后续方向 |
|---|---|---|
| MOD-P2-01 | Open | engine_module、core与prelude重复导出CoreRuntime/CoreHandle/CoreWeak和descriptor/lifecycle类型；收敛curated author facade |
| MOD-P2-02 | Open | `EngineModule: Debug`仍不能提供结构化identity/state；正式诊断只读compiled receipt |
| MOD-P2-03 | Open | description在descriptor/query/devtools继续String clone；冻结后使用Arc string table并先测量 |
| MOD-P2-04 | Open | descriptor Debug隐藏closure/lifecycle且无binding identity；输出安全binding key/policy摘要 |
| MOD-P2-05 | Open | 三个EngineService impl机械重复；用sealed typed declaration共享实现 |
| MOD-P2-06 | Open | contract constructor clone完整dependency Vec；compiled Arc slice共享并统计clone bytes |
| MOD-P2-07 | Open | ResolvedPluginGroup `module_keys()`每次分配Vec；提供borrowed iterator/Arc slice |
| MOD-P2-08 | Open | Runtime/App重复String HashMap排序，Core又按name重排；compiler只建立一次stable index |
| MOD-P2-09 | Open | RuntimeModuleLoadReport Clone + public module Vec鼓励二次装配；receipt只clone共享Arc |
| MOD-P2-10 | Open | lifecycle每次构造String context，VM clone多个String/PathBuf；使用qualified id/scoped handle |
| MOD-P2-11 | Open | declaration层测试依赖`include_str!`文本包含/排除；改为API、compile-fail和behavior tests |
| MOD-P2-12 | Open | ModuleId仍混用`AssetModule`、`DiagnosticsCoreModule`、`animation.runtime`等命名；制定hard-cut migration表 |

## 8. 五引擎参考结论

| 参考 | 物理源码事实 | 对Zircon的约束 |
|---|---|---|
| Unreal | `IModuleInterface`只承担startup/pre-unload/post-load/shutdown及reload/automatic-shutdown policy；name、filename、loaded state、load order和query由`FModuleManager`拥有。`FModuleDescriptor`把host type、loading phase、platform/architecture/target/config allow/deny和additional dependency结构化，phase load返回typed result，unsupported unload明确拒绝 | identity/metadata与implementation lifecycle分层；publish/query/unload/reload、artifact provenance和build/host policy必须由manager/compiled descriptor统一拥有 |
| Bevy | Plugin有`build -> ready -> finish -> cleanup`及`PluginsState`；App按plugin name检查唯一性，group按TypeId保存单个plugin entry和顺序。其panic语义仍偏同进程配置系统 | 保留简单author trait和分阶段readiness，但Zircon必须增加no-unwind terminal、generation、卸载、binding和ABI/capability合同 |
| Godot | module按Core/Servers/Scene/Editor逐级初始化并逆序反初始化；GDExtension manager持有加载map、level、signals、instance binding，reload涉及prepare、deinitialize、close/open、finish，无法安全切换时返回NeedsRestart | phase、instance binding cleanup、reload eligibility和RestartRequired必须是协议，不是布尔宣传或App旁路 |
| Fyrox | `PluginRegistrationContext`与运行`PluginContext`分开；DynamicPlugin reload先prepare，engine保存状态/解绑对象，再load并重新register。上下文明确暴露scene/resource/UI/graphics/time/task/input能力 | registration、runtime、factory、VM package context必须分域；Zircon应比Fyrox更进一步使用capability-limited handle而非大范围可变环境 |
| Unity Graphics | 该仓没有通用engine module host；可比的窄证据是GlobalSettings以pipeline type绑定唯一asset，ensure失败会注销/报告，ResourceReloader按attribute、package location与AssetDatabase readiness修复资源并显式标记dirty | 不把Unity Graphics伪装成模块管理器；它只证明metadata与runtime binding、package root、reload readiness和持久asset identity需要显式owner与失败结果 |

共同规律不是增加trait方法，而是implementation不拥有第二份identity，声明与runtime binding分层，manager拥有状态和query，publish前完成验证，失败有typed terminal，reload有prepare/commit/rollback或诚实RestartRequired。当前没有同硬件、同build、同模块图、同服务语义的跨引擎benchmark，禁止声称Zircon已经优于Unreal。

## 9. 目标架构

```text
EngineModule author
  -> &ModuleProposal
       ModuleId / metadata / host-load-reload policy
       ServiceProposal[] / requirements / lifecycle binding key
  -> RuntimeCompositionCompiler
       profile + target + BuildSet + package catalog generation
       full module/service/type/capability validation
  +  HostBindingSet
       FactoryBindingKey -> execution cell / provenance / policy
  -> Arc<CompiledModuleGraph>
       generation + content hash + ordered CompiledModuleEntry[]
       diagnostics + capability decisions + binding receipt
  -> CoreRuntime::register_compiled_graph(Arc<...>)
  -> generation-bound module/service runtime state
  -> App/Editor/DLL/devtools read the same receipt
```

核心合同至少包含：

- `ModuleId/ServiceId/ServiceTypeKey/FactoryBindingKey`，全部有fallible data path和稳定编码。
- `CompiledModuleEntry { proposal, services, lifecycle_binding, source/provider/build provenance }`，字段private且冻结后不可改。
- `ServiceFactoryContext`只暴露declared dependency handles、qualified identity、binding/runtime generation、deadline/cancel和diagnostic span。
- `FactoryInvocationClaim`让success/error/panic/cancel/deadline只走一个terminal commit；expected type验证必须先于publish。
- `CompiledModuleGraphReceipt`是report、sort、registration、activation、devtools和动态ABI的唯一代际事实。
- `ModuleLifecycleContext`不可外部构造，包含phase/transaction/grants；ready由wake/ticket驱动。
- Core plugin service、native package、VM package/slot使用不同context与identity，scoped roots不会通过Debug泄露。

## 10. 硬切范围与禁止方案

1. 删除`EngineModule::module_name/module_description/descriptor`三authority，迁移为单一proposal入口；不保留兼容默认方法。
2. 删除public `module_context()`、`plugin_context()`和可写context字段；Core/VM transaction owner使用私有constructor。
3. 删除平行clone式Driver/Manager/Plugin Contract，名称如保留必须指向真实compiled typed view。
4. 删除公开数据路径上的`RegistryName::from_parts/qualified_name` panic；静态const与动态fallible路径分开。
5. 删除App descriptor字符串patch、默认group二次解析和Core内再次排序；所有输入在freeze前提交。
6. 产品Core只接受compiled graph；raw ModuleDescriptor builder仅可作为测试/author proposal输入，不成为旁路。
7. 不使用process-global descriptor cache、永久interner、`Box::leak`、name-only once-cell或reload次数限制掩盖所有权问题。
8. 不靠“descriptor应为纯函数”注释、debug assert或全owner parity test替代API单一authority。
9. 不只在App或module lifecycle外层catch factory panic；Core slot必须先复位并notify。
10. 不把所有行为对象强制实现EngineService；metadata declaration与业务接口继续分层。
11. 不把Core plugin、native ABI plugin和VM package压入一个巨型context。
12. 不用source-string test、未运行测试、空服务或较少语义的benchmark证明工程完成或性能领先。

## 11. 测试先行的重构里程碑

| 里程碑 | 先写RED证据 | 实施边界 | 退出条件 |
|---|---|---|---|
| M0 Factory terminal闭合 | 当前4测受管执行；补Immediate plugin/driver、dependency panic、2/8/64 waiter、drop/shutdown、cancel/deadline | 完成既有RAII claim，不展开descriptor大改 | 所有terminal bounded、no unwind、reset/notify且原会话独立验收 |
| M1 Identity与单一proposal | name/description swap、mutable author、invalid Unicode/control/dot/trim、24-owner inventory | ModuleId/ServiceId与单一proposal hard cut | API不能表达identity/description双真值，数据输入不panic |
| M2 Compiled service contract | owner/kind/type/binding mismatch、wrong Any publish、undeclared resolve | full graph compiler、expected type、EngineService真实view | wrong type永不Running，Core/devtools/diagnostics三生产consumer |
| M3 Context与binding分域 | forged module/plugin/VM owner、stale generation、root escape/symlink、Debug leak | sealed contexts、scoped roots、typed host binding | 外部不能构造/改owner，Platform patch删除，权限fail closed |
| M4 Runtime->App->Core single receipt | total descriptor calls、A/B swap、report/Core sibling order、binding parity | Runtime返回Arc compiled graph，App只传输入，Core不再重编译 | enabled=1 call、disabled=0，report/query/activation同generation/hash/order |
| M5 Reload与运行态投影 | unsupported unload、prepare/save/load/restore/commit各阶段失败 | reload tier、RestartRequired、rollback、devtools join | 旧generation可回滚/退休，所有query可关联binding/source/runtime state |
| M6 Scale与竞争证据 | 1/100/1k/10k module，0/1/10/100 service，稀疏/稠密图与fault矩阵 | bounded workspace、allocation/profile artifact | raw calls/bytes/RSS/p50/p95/p99绑定source/build/test inventory |

## 12. 资格门

| Gate | 状态 | 当前证据 / 通过要求 |
|---|---|---|
| G01 | Partial | 24/24当前parity，但仍有三个公开authority |
| G02 | Fail | Runtime->App->Core enabled module端到端materialization不保证1次 |
| G03 | Partial | ResolvedPluginGroup disabled fixture为0；完整profile/catalog/App路径无统一计数 |
| G04 | Fail | report/query/Core/devtools没有共同graph generation/hash |
| G05 | Fail | A/B descriptor交换没有typed mismatch gate，App可静默错配 |
| G06 | Partial | Asset drift已修；全owner自动suite和单authority未完成 |
| G07 | Partial | RegistryName serde/new为fallible；ModuleId规范与control/Unicode/fuzz未完成 |
| G08 | Fail | qualified_name/from_parts动态非法输入仍panic |
| G09 | Partial | Core最终校验owner/kind/dependency；composition precommit gate缺失 |
| G10 | Fail | EngineService没有三个生产consumer |
| G11 | Fail | custom marker实现仍可自报错误kind |
| G12 | Fail | wrong ServiceObject type在publish Running后才downcast失败 |
| G13 | Fail | factory可解析未声明dependency，无Denied capability boundary |
| G14 | Fail | factory error仍to_string，typed cause chain丢失 |
| G15 | Partial | M0源码覆盖Immediate/Lazy/Plugin panic；未集成、未受管验证 |
| G16 | Partial | panic/error可source-level reset；cancel/deadline和完整terminal矩阵缺失 |
| G17 | Partial | 1个owner+1个waiter测试存在；8/64 waiter和managed evidence缺失 |
| G18 | Fail | ModuleContext仍可外部构造且无generation |
| G19 | Fail | Core plugin/package/VM slot identity仍可混用/改写 |
| G20 | Fail | roots不绑定verified package generation，旧handle不失效 |
| G21 | Fail | root escape、symlink swap和授权写无矩阵 |
| G22 | Fail | Platform backend仍靠App字符串patch |
| G23 | Fail | binding没有identity，无法在Core注册前检查missing/duplicate/target |
| G24 | Partial | profile/target选择存在；host/load schema仍受cfg与App分支影响 |
| G25 | Fail | module声明无RestartRequired/reload tier |
| G26 | Fail | module reload无prepare/save/restore/commit/rollback协议 |
| G27 | Fail | report/devtools不能关联BuildSet/source/provider/binding/runtime generation |
| G28 | Fail | VmPluginHostContext Debug包含PluginContext，raw root路径可进入diagnostic |
| G29 | Partial | dynamic wrapper无Box::leak且有1/100/1,000 borrow test；无drop/reclamation/RSS证据 |
| G30 | Fail | 无1/100/1k/10k calls/alloc/clone/RSS/latency矩阵 |
| G31 | Fail | Core按name重排，同level sibling输入顺序可改变App/Core parity；无receipt hash |
| G32 | Fail | compiled graph前的公开descriptor字段可任意改name/dependency/factory/lifecycle |
| G33 | Partial | failure已有局部source repairs；call-count/parity/reclamation/managed validation未齐 |
| G34 | Pass | 本文保留Runtime01/03/07/24/42/45、App01、Plugins01父owner，不重复关闭 |
| G35 | Partial | 稳定子集绑定当前fingerprint，并发composition路径有明确排除记录；benchmark仍无完整source/build/test inventory |
| G36 | Pass | frontmatter、链接、计数、LF/BOM/trailing-space与scoped diff通过后保持Pass |

Gate统计：**22 Fail、12 Partial、2 Pass**。两个Pass只证明owner纪律与文档卫生，不代表模块系统具备产品资格。

## 13. Owner、依赖顺序与开放failure

| Owner | 保留职责 | Runtime135只要求的接口 |
|---|---|---|
| Runtime01 | module/service完整lifecycle、shutdown、drain、teardown、rollback | generation-bound lifecycle coordinator与terminal receipt |
| Runtime03 | diagnostics、error/profiling/config projection | graph/binding/runtime state可join的bounded snapshot |
| Runtime07 / Plugins01 | VM/native package、ABI、capability、hot reload和安全 | verified package/slot generation与scoped grants |
| Runtime24 / Runtime50 | stable identity、generation、typed handle与call admission | ModuleId/ServiceId/ServiceTypeKey及运行态identity view |
| Runtime42 | profile/target/provider/extension选择 | 单次composition inputs与compiled graph output |
| Runtime45 | preference storage backend语义 | typed host binding，不再让App改descriptor closure |
| App01 | product host、bootstrap、main loop和shutdown | 只消费compiled receipt并提交host/product inputs |
| Runtime46 / Runtime135 | module/service author contract、single materialization、factory/context边界 | 本文M0-M6与36门禁 |

保持开放：`docs/plans/zircon_runtime/runtime/02/failure-2026-07-17-module-descriptor-regeneration.md`。当前局部修复包括ResolvedPluginGroup snapshot、动态文本owner borrow和无Box::leak；仍缺Runtime->App->Core总调用数、report/registered/activated descriptor与binding等价、同level真实激活顺序、repeated entry/drop/reload reclamation、fresh source-bound managed validation。满足这些条件前不得改名`fixed-*`。

## 14. 验证边界与首个实施切片

本轮执行的是静态current-source review、caller反查、共享diff检查、物理统计、五引擎本地源码对照和文档索引更新。没有运行Cargo、M0 focused tests、App/Editor、真实DLL/VM package、8/64 waiter、hung factory、cancel/deadline、fresh process、reload rollback、root security、reclamation、soak/profile或跨引擎同语义benchmark。

首个实施切片仍是M0，不应先扩充module enum、增加compat wrapper或继续手写更多1..5 cardinality fast path。既有Runtime46会话应先完成factory panic源码的受管验证与独立review，再补8/64 waiter和所有terminal；随后立即把wrong-type-before-publish作为M2首个RED测试。性能工作必须建立在同功能、同模块图、同服务类型/依赖和同失败语义上，不能通过省略type、capability、reload或diagnostic合同声称优于Unreal。
