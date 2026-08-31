---
title: Runtime Builtin Module / Catalog / Profile / Target / Feature / Extension Assembly 当前源码复审
category: zircon_runtime
report_id: Runtime136
review_date: 2026-08-24
baseline_head: 858350a5707a1b251eda626f78bce4e329c0da1a
baseline_epoch: 419
verification_head: f811b3bf474d70347199772a175422333dfb36f6
verification_epoch: 420
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
related_code:
  - zircon_runtime/src/builtin
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_runtime/build.rs
  - zircon_runtime/src/core/framework/platform/runtime_target_mode.rs
  - zircon_runtime/src/core/framework/project/runtime_profile_id.rs
  - zircon_runtime/src/core/framework/project/project_plugin_manifest
  - zircon_runtime/src/plugin/runtime_profile
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog
  - zircon_runtime/src/plugin/extension_registry
  - zircon_runtime/src/dynamic_api/session/linked_plugins.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/plugins/groups.rs
  - zircon_app/src/plugins/groups/resolution.rs
tests:
  - zircon_runtime/src/builtin/runtime_modules/tests
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/derived_projection/tests.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/update/tests.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry/tests.rs
  - zircon_runtime/src/plugin/extension_registry/typed_extension_point/tests.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/42/2026-08-24-effective-manifest-registration-filter.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/21-zr-language-parser-type-system-semir-bytecode-package-loader-vm-runtime-review.md
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99zj-runtime-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ModuleDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ModuleDescriptor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/plugin_group.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/godot/modules/register_module_types.h
  - dev/godot/core/extension/gdextension_manager.h
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/godot/core/extension/gdextension_library_loader.cpp
  - dev/godot/core/extension/gdextension.h
  - dev/godot/core/extension/gdextension.cpp
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Unity.RenderPipelines.Core.Runtime.asmdef
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Unity.RenderPipelines.HighDefinition.Runtime.asmdef
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime136 · Builtin Module / Catalog / Profile / Target / Feature / Extension Assembly 当前源码复审

## 1. 结论

Runtime42记录的方向仍然成立，但当前源码已经出现四项真实进展。第一，`active_plugin_registration_refs()`现在先计算effective manifest，把manifest中enabled、target匹配且可canonicalize的`RuntimePluginId`建成集合，再过滤registration；对应disabled、unselected、alias和selected-conflict行为测试已加入共享工作树。第二，App开始用同一份effective manifest选择first-party registration并调用builtin assembly，render profile overlay也集中到一个helper。第三，`RuntimePluginCatalog`已有catalog generation、immutable derived projection、按target缓存的project plan、source manifest fingerprint，以及candidate update的last-good发布语义。第四，availability已经有单一row generation、分类索引、missing-required索引视图和规模metrics。这些基础应保留。

但产品启动仍然没有一份工程级`RuntimeCompositionPlan/Receipt`。Runtime builtin只把Importer和9类graphics extension拍平成裸Vec，并未消费catalog的正式完整registry；App随后又从registration追加module descriptor，过滤条件仍只看registration自身selection，不看effective manifest。因此即使Runtime侧Importer/Render extension已被M0过滤，disabled或unselected registration的module仍能进入最终Core图。Feature module追加更严重：App和builtin都只按available feature ID匹配registration，没有执行catalog已有的provider选择；同feature多个provider时，正式extension report选一个，module路径却可追加多个。

Profile与BuildSet边界也没有闭合。6个Profile、12个builtin module仍由TOML生成，但Graphics/Script enum variant和Profile membership会被`#[cfg]`直接删除；`required_capabilities`仍无builtin composition consumer；无Profile的target入口继续构造synthetic permissive profile。Client/Editor/Server只是三值target和少量条件分支，不是产品角色、平台、artifact、loading phase与capability共同约束的BuildSet。

Runtime42的52项P1当前为 **44 Open、8 Partial、0 Closed**；14项P2仍全部Open；42项资格门为 **33 Fail、9 Partial、0 Pass**。本文不新增P0，也不替Plugins01/06、App01、Runtime01/07/21/46重复计数。M0 registration过滤只能判为Partial：源码和focused tests存在，但原实施会话仍为`waiting_validation`，受管Cargo、独立review和最终App module隔离尚未完成。

## 2. 审查边界、方法与currentness

### 2.1 冻结物理范围

统计口径为物理行、非空行、文件bytes；test declaration匹配Rust `#[test]`、常见C++ test宏和C# `[Test]`，ignored匹配`#[ignore`。fingerprint按normalized lowercase path排序，对每个文件拼接`path + NUL + lowercase(file SHA-256) + LF`后再取SHA-256。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| builtin、profile source/build generation、target/profile/manifest | **50 / 6,429 / 5,908 / 232,842 / 50 / 1** | `0668f32a34bc96a288ae073f1652f39d3911876a99c3248bbf4dc41e422765fd` |
| RuntimePluginCatalog与RuntimeExtensionRegistry完整目录 | **122 / 9,864 / 8,970 / 355,407 / 46 / 0** | `88b5443318ee26d3e007c50ebffe5bfd8e15a9f5a61515af046d9e374963c997` |
| App与dynamic session的6个生产consumer | **6 / 1,292 / 1,174 / 46,482 / 5 / 0** | `0a74a0df61a68216e0f4374bfdf879ad0691b75435c10a5433103079a2a5ca0e` |
| Zircon selected union | **178 / 17,585 / 16,052 / 634,731 / 101 / 1** | `746fca241e791b21293b134bf3b51a63f3aaaff5ce2d54c7d9194c3699f61716` |
| 旧Runtime42与M0实施记录 | **2 / 687 / 479 / 54,102 / 0 / 0** | `fe3655c0a4bba5b14b87f56688e85d9c042d7749cbffe8c0949573c9e3000cb6` |
| 五引擎参考选择集 | **22 / 16,511 / 14,303 / 613,477 / 35 / 0** | `b4414d59db7ceedcf0eb1f10918577b0a1cb4afe7646ca9bc139faf08a3be57e` |
| selected combined scope | **202 / 34,783 / 30,834 / 1,302,310 / 136 / 1** | `36f623ac096634a20bda803d95782ad4296eab0fce7e9c10e0e36c06c26d3622` |

参考revision：Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`。`dev/UnrealEngine`没有独立Git元数据，只由6个物理文件与参考集合fingerprint冻结。

### 2.2 检查方法

1. 沿`runtime-feature-presets.toml -> build.rs -> generated profile -> target/profile candidate -> effective manifest -> availability -> registration/feature filter -> extension inputs -> module materialization -> load report`读取完整路径。
2. 逐文件扫描`RuntimePluginCatalog`和`RuntimeExtensionRegistry`目录，重点反查generation、project plan cache、provider match、capability resolution、module dependency order、全部extension family、owner slot、freeze/revoke与bridge lifecycle。
3. 反向搜索全部非`dev/`生产consumer，核对App group预装配、registration/feature module追加、catalog重复构建、Core提交和dynamic session并行truth。
4. 对Runtime42的52项P1、14项P2和42项gate按原编号逐项重判；局部foundation存在但产品断路仍在时只允许Partial，不以测试数量或新类型名判Closed。
5. 对照Unreal module/plugin loading policy、Bevy single App plugin registry、Godot initialization level/restart状态、Fyrox plugin lifecycle/dylib reload、Unity package/assembly build closure。

### 2.3 currentness与动态证据边界

- 本轮Session基线为`858350a5707a1b251eda626f78bce4e329c0da1a` / epoch 419。共享工作树处于degraded baseline；`zircon_app`和Runtime42 M0相关文件有未提交改动，本文审查的是登记时与最终复测时的工作树bytes，不把它们视为已集成发布证据。
- 最终静态复测时仓库已由其他会话推进到`f811b3bf474d70347199772a175422333dfb36f6` / epoch 420；202个selected文件的七组统计与fingerprint均和首次冻结一致。
- Runtime42 M0记录的4个关键源码/测试hash与当前bytes一致，但原会话处于`waiting_validation`；其RED/green managed ticket都在编译前被外部validation-copy漂移阻断，不能作为行为通过证据。
- 本轮只修改本文和三个索引；不修改Runtime、App、Editor、Plugin、Cargo、旧Runtime42或M0记录。
- review-only不运行Cargo、产品启动、clean feature powerset、dynamic DLL、rollback、reload、shutdown soak或benchmark。静态调用图可以证明的断路不依赖这些动态测试；资格门仍按缺证据Fail/Partial处理。

## 3. 当前组合链与断路

```text
Profile TOML --build.rs--> RuntimeProfileDescriptor
      |                         |
      |                         +--> required_capabilities (metadata only)
      v
App effective manifest -------------------------------+
      |                                                |
      +--> first-party registration filtering          |
      +--> builtin assembly                             |
      |      +--> availability projection               |
      |      +--> manifest-filtered plugin registries   |
      |      +--> feature ID-filtered registries        |
      |      +--> partial extension Vec flatten         |
      |      +--> public RuntimeModuleLoadReport        |
      |                                                |
      +--> App registration module append -------------+-- no manifest filter
      +--> App feature module append -------------------+-- no provider filter
      +--> independent Default/Dev/Headless group
      +--> set/add + final group sort
      +--> Core register/activate

Parallel formal path:
registrations -> RuntimePluginCatalog generation/project-plan cache
              -> selected provider + full owner-scoped extension registry
              -> bridge lifecycle state
              (not the builtin module composition authority)
```

| 边界 | 可保留基础 | 仍然断开的合同 |
|---|---|---|
| Profile source | schema v2、deny unknown、6 Profile/12 module exact row、重复ID/target/maturity/feature gate检查 | expected表、enum、materializer仍多点硬编码；cfg删除稳定intent |
| effective manifest | App已集中baseline/profile/render overlay并复用到first-party registration | Runtime、App、dynamic session仍各自merge；无source/precedence/conflict receipt |
| registration filter | Runtime builtin已按canonical manifest membership过滤 | App module追加仍只看registration自身selection；M0未受管验证 |
| feature provider | catalog可按`feature@provider`解析并只merge一个registration | builtin/App module与partial extension路径只按feature ID，仍可多provider激活 |
| catalog | generation、projection metrics、project plan cache、candidate last-good publish | 只编译plugin catalog局部；不含builtin/Editor/App最终module graph、BuildSet或artifact closure |
| extension registry | typed owner slot、freeze/thaw、全family merge、revoke owner、bridge table | builtin只投影Importer+9 graphics family并丢owner/generation；无统一transaction receipt |
| availability | immutable row generation、primary category index、missing-required view、summary | 产品继续携带公开可变的owned Vec report；不是selection/provider/capability完整resolution row |
| App composition | fatal load diagnostic会在启动前转CoreError | group先独立装配后覆盖；Editor临时append；registration/feature module旁路；多次catalog/sort |
| lifecycle | bridge owner可activate/disable/deactivate并阻止strong dependent | state的reload把同一registry同时当current和replacement；无module/provider restart policy |
| diagnostics | Core/feature/importer已有部分typed variant | registration与catalog仍有自由字符串，App直接`eprintln!`，无code/stage/source/correlation |

## 4. 必须保留的工程基础

1. 保留TOML作为builtin Profile声明源及build-time严格schema检查，但生成稳定intent，不再让cfg删除wire variant。
2. 保留Profile candidate registry、内建依赖闭包和Core最终拓扑校验；把descriptor snapshot继续传入compiled graph，避免App重算。
3. 保留RuntimePluginCatalog的immutable projection、catalog generation、project plan cache、candidate last-good发布和线性feature resolution。
4. 保留RuntimeExtensionRegistry的typed owner、stable slot、完整family merge、freeze、owner revoke与bridge lifecycle fence。
5. 保留availability单row generation、互斥primary category、missing-required index view与metrics；owned report只应是只读导出投影。
6. 保留M0 manifest membership过滤和canonical ID集合，但把同一admission用于extension与module proposal。
7. 保留required/optional、target、maturity、packaging和Linked/NativeDynamic分类，收敛为一个selection resolution row。
8. 保留App在commit前拒绝fatal report的原则，但App只消费Runtime生成的frozen plan/receipt，不继续拥有选择逻辑。

## 5. P0边界

本文仍不新增P0。required selection/provider/catalog closure由Plugins06拥有；产品角色、单一composition receipt与startup/shutdown由App01拥有；module activation/rollback/reverse teardown由Runtime01/46拥有；native ABI与动态代码释放由Plugins01、Runtime07/21拥有。Runtime136只负责把这些输入收敛到Runtime侧唯一composition compiler，不建立平行owner。

## 6. P1当前源码重判

### 6.1 Selection、catalog与结果类型

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| P1-01 | Partial | Runtime builtin已按effective manifest canonical membership过滤extension输入，新增4组负例；App lines 70-95/145-170仍按registration自身selection追加module | 一个admission row同时控制module、extension、artifact和lifecycle；完成受管验证 |
| P1-02 | Open | formal catalog按provider匹配；builtin `active_feature_registration_refs`和App feature module追加仍只按feature ID | compiler输出唯一selected provider，所有下游只读provider row |
| P1-03 | Open | builtin继续调用`extension_inputs_from_extension_registries`，未消费project extension report | 删除partial flatten，builtin module直接消费同代frozen registry |
| P1-04 | Open | load report只加入feature和asset importer错误；registration diagnostics由App打印 | registration/catalog diagnostic进入统一fatal/degraded truth |
| P1-05 | Open | builtin feature route建catalog，App又建catalog/lifecycle state，dynamic session再建一份 | 一次catalog generation和一次project plan服务所有consumer |
| P1-06 | Source implemented / validation pending | App已删除Default/Dev/Headless预装配覆盖路径，直接消费Runtime冻结的module/descriptor order；Dynamic fallback也移入compiler input | App只提交product inputs并消费compiled graph；仍需Editor/Export统一receipt验收 |
| P1-07 | Partial | `RuntimeModuleCompositionCompiler`已成为冻结plan+host module纵切入口；public `runtime_modules_for_*`组合适配器仍未删除 | 一个compiler入口；legacy adapter只构造input，不拥有逻辑 |
| P1-08 | Open | linked ID membership与registration report仍表达不同provider信息量 | typed ProviderEvidence，统一package/module/artifact/packaging来源 |
| P1-09 | Source implemented / validation pending | public结果已分为`RuntimeModuleCompositionPlan`和不含module字段/accessor的`RuntimeModuleCompositionRejection`；mutable load report收回crate-private | 补受管编译、失败矩阵与外部API验收后关闭 |
| P1-10 | Source implemented / validation pending | `builtin_runtime_modules()`生产导出和调用已删除；所有target/profile adapter返回typed Result | 补受管编译和吸收守卫验收后关闭 |
| P1-11 | Partial | `CompiledProjectPluginPlan`保留source fingerprint；ready plan带catalog generation/target/profile和绑定最终logical module/service graph的BLAKE3 identity；legacy adapter不伪造generation | 仍缺BuildSet、selection/provider/capability全row identity和跨consumer receipt校验 |
| P1-12 | Partial | finalizer复用Core `FrozenModuleGraph`在Ready前验证module+service graph；catalog candidate仍可保留last-good | activation/extension commit、rollback census和receipt仍未实现 |

### 6.2 Profile、BuildSet与产品角色

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| P1-13 | Open | Profile `required_capabilities`仅生成/序列化，无builtin assembly consumer | capability solver逐条绑定module/provider/artifact |
| P1-14 | Open | Graphics/Script enum variant仍受cfg删除 | 稳定wire enum，缺实现为NotCompiled row |
| P1-15 | Open | generated Profile membership仍插入cfg attribute | Profile intent稳定；BuildSet availability另行裁决 |
| P1-16 | Open | build.rs只验证表形状、token和feature包含，不验证provider/artifact闭包 | build-time closure validator与product BuildSet receipt |
| P1-17 | Open | target默认UI/UI importer selection仍随compiled `ui` feature出现/消失 | intent固定，未编译required provider明确Rejected |
| P1-18 | Open | target入口组装candidate，不选择Profile | typed TargetCompositionPolicy或显式ProfileIntent |
| P1-19 | Open | target availability仍构造name为`target module selection`的synthetic profile | 无Profile产品必须提供真实policy和maturity/capability合同 |
| P1-20 | Open | RuntimeTargetMode仍只有Client/Server/Editor | ProductRole、platform、host type、build config、target分型 |
| P1-21 | Open | Server仅靠core candidate分支排Graphics/Script，仍保留Input/Asset/Scene | constraint解释每个module为什么included/excluded |
| P1-22 | Open | EditorModule仍由App条件append，不在Editor Profile图 | Editor proposal进入同一final graph |
| P1-23 | Open | 新Profile/module仍需改TOML、expected表、enum和materializer | versioned declarative registry/codegen，移除重复硬编码 |
| P1-24 | Partial | App集中effective manifest并复用first-party路径；Runtime target/profile/dynamic session仍各自merge | 一个有source、precedence、conflict的manifest compiler |

### 6.3 Identity、provenance与extension merge

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| P1-25 | Open | public `RuntimePluginId::from_static`仍不校验 | private unchecked constructor + const-validated builtin IDs |
| P1-26 | Open | public `RuntimePluginId::new`仍`expect` panic | `TryFrom/FromStr` typed error，所有数据入口fallible |
| P1-27 | Open | alias仍硬编码在`parse_key` match | versioned alias registry、migration和collision receipt |
| P1-28 | Open | runtime alias、project selection ID、package ID与provider ID继续互转String | RuntimeId/PackageId/ProviderId/ModuleId独立类型 |
| P1-29 | Open | ID无publisher、namespace和major contract | versioned global identity schema |
| P1-30 | Open | `label()`继续在ID类型硬编码显示名 | metadata/localization owner提供display label |
| P1-31 | Open | UnknownPlugin只由parse失败产生，合法但catalog未知另走Stub/MissingCatalog | ParseError、UnknownCatalogId、MissingProvider分型 |
| P1-32 | Open | builtin `runtime_plugin_descriptors()`每次调用`RuntimePluginDescriptor::builtin_catalog()` | 借用process-wide immutable catalog generation |
| P1-33 | Open | builtin plugin module loader仍只特殊识别UI | module proposals来自catalog/package descriptor，不按ID特判 |
| P1-34 | Open | builtin flatten返回裸Vec和AssetImporterRegistry，owner/provider/source丢失 | full registry中的owner-scoped contribution row |
| P1-35 | Open | builtin flatten顺序仍等于caller registry输入顺序 | module dependency order + canonical tie-break决定merge顺序 |
| P1-36 | Open | builtin层只有Importer重复产生typed error，其余Vec不裁决冲突 | 每个family统一duplicate/conflict/override policy |
| P1-37 | Open | formal registry会拒绝重复import path；builtin按owner/path/hash三元组去重，仍允许同path不同内容并存 | path identity先裁决，相同内容coalesce，不同内容fatal双owner诊断 |
| P1-38 | Open | builtin只投影Importer和9类graphics；正式registry还有module/manager/system/resource/event/interface/component/UI/metadata | 删除family白名单，传递完整frozen registry |

### 6.4 Final graph、lifecycle与诊断

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| P1-39 | Open | Linked/NativeDynamic module仍由App遍历registration.extensions.modules追加 | Runtime compiler统一materialize所有module proposal |
| P1-40 | Open | core candidate只有三值target gate | host/graphics/offscreen/compute/server VM等typed constraints |
| P1-41 | Open | load report图之后仍会追加Editor、registration和feature module，并经group覆盖/重排 | report直接携最终Core graph hash |
| P1-42 | Open | Zircon scope无loading phase或explicit-load policy | typed phase、host policy、跨phase依赖验证 |
| P1-43 | Open | bridge lifecycle有局部切换，但plan无Unloadable/Reloadable/NeedsRestart；reload state还复用同一registry | provider/module reload eligibility与replacement generation receipt |
| P1-44 | Open | load diagnostic混合typed error和自由字符串；catalog/bridge诊断仍format String | stable code/severity/stage/owner/source/correlation/remediation |
| P1-45 | Open | report只有modules、availability分类与diagnostics，无每个selection terminal row | SelectionResolutionRow覆盖disabled/mismatch/resolved/rejected |
| P1-46 | Partial | 新availability generation只存一份row并以索引投影分类；产品导出仍是public mutable Vec | plan内保留immutable generation，外部只读paged view |
| P1-47 | Open | formal runtime registration按module图排序；builtin/feature/diagnostic仍受输入顺序影响 | 全graph canonical order和determinism golden |
| P1-48 | Open | App继续`eprintln!`warning、registration和feature diagnostic | 统一diagnostic sink与startup receipt |
| P1-49 | Partial | catalog plan/report改用Arc并缓存，projection有generation metrics；faulty route仍clone registration并重复建catalog | 一次projection、一次plan、借用/Arc传递所有产物 |
| P1-50 | Partial | catalog update与availability已有线性work metrics和1/100/10,000测试；输入仍无hard capacity/cost admission | 全局/owner容量、checked arithmetic、typed BudgetExceeded |
| P1-51 | Partial | M0增加disabled/unselected/alias负例，catalog有generation/scale/last-good测试；multi-provider、cfg closure、rollback和产品矩阵仍缺 | 以gate 38-41为行为/产品测试矩阵 |
| P1-52 | Open | 没有真实BuildSet identity、clean Profile matrix、startup/teardown或fresh performance receipt | 发布资格绑定source、BuildSet、plan hash与SLO |

## 7. P2重判

14项P2仍全部Open。当前catalog的capability resolution、generation cache和owner bridge是底座，不等于以下工程终态。

| ID | 状态 | 目标 |
|---|---|---|
| P2-01 | Open | Constraint-based provider solver：capability、target、quality、trust、license、budget共同选择并解释provider |
| P2-02 | Open | Versioned Profile inheritance/composition与受约束delta |
| P2-03 | Open | Cook/export生成content-addressed precompiled composition artifact |
| P2-04 | Open | manifest变化产生增量add/remove/replace diff、影响面与rollback |
| P2-05 | Open | CPU/GPU/vendor/fallback多provider quality tier与downgrade receipt |
| P2-06 | Open | 不可信provider隔离域与capability broker |
| P2-07 | Open | 带版本、limit、resource class、permission和negotiated value的定量capability |
| P2-08 | Open | Editor selection/provider/module/phase/extension/artifact graph explorer |
| P2-09 | Open | crash/telemetry携composition hash与fleet provenance |
| P2-10 | Open | 保存input/decision log并确定性replay |
| P2-11 | Open | partition/on-demand module domain与owner lease |
| P2-12 | Open | 受依赖/资源约束的并行prepare，确定性commit/rollback |
| P2-13 | Open | receipt绑定signature、SBOM、license、SDK/ABI/schema和revocation |
| P2-14 | Open | selection/merge/rollback/hot replace property、fuzz与model checking |

## 8. 42道资格门重判

### 8.1 Identity、Schema、Selection与Profile（1-16）

| Gate | 状态 | 当前证据 |
|---:|---|---|
| 1 | Fail | cfg仍删除Graphics/Script wire variant |
| 2 | Fail | `new` panic且`from_static`公开unchecked |
| 3 | Fail | alias无version/migration/collision receipt |
| 4 | Fail | package/provider/module/capability仍可String误传 |
| 5 | Fail | 只有部分schema version，diagnostic/receipt缺失 |
| 6 | Fail | 无BuildSet identity |
| 7 | Fail | 无manifest/Profile migration和canonical hash策略 |
| 8 | Fail | 无每selection terminal row |
| 9 | Source pass / validation pending | required/fatal只能产生不携modules的RejectedComposition |
| 10 | Partial | Runtime extension输入已隔离disabled registration；App module仍泄漏 |
| 11 | Fail | builtin/App feature route仍可能激活多个provider |
| 12 | Fail | Profile required capability未解析 |
| 13 | Fail | Profile intent仍被BuildSet cfg静默裁剪 |
| 14 | Partial | App effective manifest局部统一，其他入口仍分裂 |
| 15 | Fail | target入口仍用synthetic permissive profile |
| 16 | Partial | 已有最终logical graph hash和source fingerprint；BuildSet/canonical selection rows仍缺 |

### 8.2 Extension、Module、Product与Lifecycle（17-31）

| Gate | 状态 | 当前证据 |
|---:|---|---|
| 17 | Partial | formal catalog可合并完整owner-scoped registry；builtin仍走partial flatten |
| 18 | Partial | typed registry能拒绝多数duplicate；无统一identical/conflict/override receipt |
| 19 | Partial | formal registry按import path拒绝重复，builtin仍允许同path不同内容 |
| 20 | Partial | formal runtime registration按module图排序，builtin/feature仍依赖input Vec |
| 21 | Partial | App Editor/Dev、Dynamic fallback和compiled provider modules进入最终图；export/phase graph仍未统一 |
| 22 | Fail | 无loading phase、phase rollback |
| 23 | Partial | RejectedComposition已无module Vec；activation rollback census仍缺 |
| 24 | Fail | plan不记录Unloadable/Reloadable/NeedsRestart |
| 25 | Source pass / validation pending | App product entry只编译一个catalog/plan/final graph，不再二次PluginGroup profile assembly/sort |
| 26 | Fail | App/dynamic/export不共享receipt/hash |
| 27 | Fail | Server Script/VM/Input/Graphics仍由临时gate决定 |
| 28 | Source pass / validation pending | EditorModule作为host input在final freeze前提交；缺feature时fail closed |
| 29 | Fail | 三种packaging无同schema outcome |
| 30 | Fail | 无composition admission close、owner revoke、reverse deactivate统一receipt |
| 31 | Fail | 无NeedsRestart decision |

### 8.3 Diagnostics、预算、测试与性能（32-42）

| Gate | 状态 | 当前证据 |
|---:|---|---|
| 32 | Fail | diagnostic无完整stable schema |
| 33 | Source pass / validation pending | composition warning进入Entry report diagnostics；Editor缺feature返回typed CoreError |
| 34 | Partial | immutable generation有单row/索引；public owned report仍可制造矛盾 |
| 35 | Fail | 无全局与单owner容量预算 |
| 36 | Partial | 有projection/update metrics和部分1/100/10,000测试；缺完整cardinality/alloc/clone/graph基准 |
| 37 | Partial | App/Dynamic各自一catalog/plan/final freeze；legacy adapter和candidate内部预排序仍待profile量化/收敛 |
| 38 | Partial | disabled/unselected/alias负例已加入；multi-provider、cfg module、registration diagnostic仍缺 |
| 39 | Fail | 五Profile无fresh clean Cargo/startup/shutdown receipt |
| 40 | Fail | 无source/native、conflict、rollback、reload、DLL unload CI矩阵 |
| 41 | Fail | 无manifest/ID/graph/conflict determinism property/fuzz |
| 42 | Partial | ready plan绑定source fingerprint和composition hash；BuildSet与发布receipt仍缺 |

## 9. 五套参考源码给出的工程边界

### 9.1 Unreal

`FModuleDescriptor`把host type、loading phase、platform/target/config allow/deny和build/load判断放在同一声明，phase load返回逐module failure；`FPluginManager`先配置enabled/required plugin再按phase装载；`FModuleManager`保留load failure reason、query status、pre-unload、shutdown和逆序卸载。Zircon不应复制全局singleton，但必须把build eligibility、runtime selection、phase activation、failure receipt和reverse teardown连接成一个可查询状态机。

### 9.2 Bevy

Bevy `Plugin`把build、ready、finish、cleanup和unique identity交给唯一App registry，`PluginGroupBuilder`集中处理add/set/enable/disable/order。其显式顺序不能替代Zircon需要的复杂依赖图，但证明产品调用者不应在registry之外维护第二份plugin truth。

### 9.3 Godot

Godot module与GDExtension共享Core/Servers/Scene/Editor initialization level，manager逐级initialize并逆序deinitialize；load结果区分AlreadyLoaded、NotLoaded、NeedsRestart和Failed，loader还校验entry symbol、platform library和minimum compatibility。Zircon当前bridge局部reload不能替代完整module/provider restart资格。

### 9.4 Fyrox

Fyrox plugin把register、init、on_loaded、on_deinit和graphics context created/destroyed放在同一contract，资源registry ready后才enable，dylib reload重新fill/register并维护明确状态。它的故障隔离不是Zircon上限，但生命周期连续性明显优于“Runtime拍平部分extension、App再安装observer”。

### 9.5 Unity Graphics

SRP Core/HDRP以精确package version dependency表达包闭包，asmdef另行声明assembly reference、platform inclusion、auto-reference和version define。这说明package依赖与代码BuildSet条件应分层表达、共同验证；不能只靠Cargo cfg把Profile intent删除。

## 10. 目标架构

```text
ProductRole + Platform + BuildSet + ProfileIntent + ProjectManifest
                              |
                              v
                 RuntimeCompositionCompiler
       Identity/Migration -> Selection/Provider/Capability Solver
                              |
                              v
              Frozen RuntimeCompositionPlan (generation/hash)
  selection rows / provider rows / capability rows / module graph+phase
  full extension registry / artifact closure / diagnostics / budgets
                              |
                              v
                 RuntimeCompositionTransaction
          prepare -> validate -> commit -> rollback on failure
                              |
                              v
                 RuntimeCompositionReceipt
             App / Core / Editor / DLL / Export consumers
```

核心owner必须位于`zircon_runtime`：Runtime编译组合计划，Core只验证并提交compiled graph，App只提供产品输入和host bindings。`zircon_app`中的effective manifest helper、registration module append、feature module append和PluginGroup预装配都应在hard cutover后删除，不保留兼容shim。

## 11. 分阶段重构计划

### M0 · 完成现有registration过滤验收

1. 原Runtime42会话完成受管focused Cargo与独立review；验证disabled/unselected/alias/selected-conflict。
2. 补App module泄漏RED：disabled/unselected registration即使自身enabled，其module不得进入最终group。
3. 补feature multi-provider RED：只有manifest选择的provider能贡献extension和module。
4. 在此之前P1-01保持Partial，禁止把M0标Closed。

### M1 · 建立单一Composition Compiler

以现有catalog projection/project-plan为底座，加入Profile/target/BuildSet/product input、builtin proposal和final module graph；输出immutable selection/provider/capability/module/extension rows。旧9个API只允许成为薄input adapter，随后hard cut删除。

### M2 · 稳定Identity、Profile intent与BuildSet

移除cfg-controlled wire variant；引入typed ProductRole/BuildSet和NotCompiled outcome；执行Profile required capabilities与artifact closure；统一baseline/profile/project/render overlay的source/precedence/conflict政策。

### M3 · 完整Extension与Module事务

删除builtin partial flatten；按module graph顺序合并一份full RuntimeExtensionRegistry，执行所有family冲突政策并保留owner/provider/generation。Builtin、Editor、source/native/generated module进入同一phase graph；prepare/commit失败返回rollback census。

### M4 · App/Dynamic/Export hard cutover

App删除独立PluginGroup Profile装配、registration/feature module旁路和第二catalog；dynamic session、Editor、export只消费同一receipt或校验同hash。接入Runtime01/46 reverse teardown和Plugins01 owner fence/NeedsRestart。

### M5 · 诊断、预算、性能与发布资格

统一structured diagnostic sink；加入全局/owner预算、1/100/1,000/10,000规模bench、clean Profile BuildSet matrix、provider conflict、rollback/reload/DLL unload、property/fuzz/soak。发布证据必须绑定source fingerprint、BuildSet和plan hash。

## 12. 首个可执行切片

首个切片不继续增加helper，而是完成一条可证明的vertical cut：

1. RED：在App最终`module_selection_report()`上复现disabled registration module泄漏和multi-provider feature module重复。
2. Runtime侧新增内部`CompiledRuntimePluginSelection`，由catalog completed manifest和provider lookup产生唯一active registration/feature registration集合。
3. builtin extension输入与module proposal都只消费该compiled selection；App删除本地两段registration/feature遍历。
4. report同时返回final module proposal snapshot和catalog generation；App不得再从registration重建module。
5. focused tests绿后再接入managed Windows validation与独立review；这只是M1入口，不宣称完整RuntimeCompositionPlan完成。

### 12.1 2026-08-27 M1 composition outcome / host graph 源码进展

本轮在受管Cargo通道仍阻塞时继续完成可落地的非验收项，未把验收队列作为唯一工作项：

1. 新增`RuntimeModuleCompositionCompiler`、`RuntimeModuleCompositionPlan`、`RuntimeModuleCompositionRejection`和`RuntimeModuleCompositionIdentity`。内部`RuntimeModuleLoadReport`收回`crate::builtin::runtime_modules`私有；Rejected类型没有module字段或访问器。
2. `CompiledProjectPluginPlan`正式保留catalog cache使用的source manifest fingerprint。Ready identity绑定catalog generation、source fingerprint、target/profile和最终logical descriptor graph；legacy adapter的provenance字段为`None`，不伪造generation。
3. finalizer复用Core `FrozenModuleGraph::freeze`，一次验证module duplicate/missing/cycle与service duplicate/missing/cycle，再按其activation order冻结module+descriptor pair。没有复制Core拓扑算法。
4. App把Dev `LogDiagnosticsModule`和`EditorModule`作为host input交给compiler；删除先构造Default/Dev/Headless group、再set/add、再`try_finish`的第二图。缺少`target-editor-host`时Editor请求返回typed failure，不再打印后继续。
5. Dynamic在compile前根据冻结provider package rows决定Navigation/Animation fallback；session construction只注册Ready plan中已经通过最终图验证的descriptor，不再append、自行查询fatal或重新调用`EngineModule::descriptor()`生成第二份声明。
6. 删除`builtin_runtime_modules()`和App `for_config_with_available_runtime_plugins`硬切入口，不保留丢诊断raw Vec或linked-ID产品旁路。

当前算法上界（源码推导，非实测）：设最终module数为`M`、module dependency边为`E_m`、service数为`S`、service dependency边为`E_s`、descriptor逻辑字节量为`B`，final freeze/ordering为`O(M + E_m + S + E_s)`加稳定有序容器开销，pair重排平均`O(M)`，identity digest为`O(B)`；内存为图索引和最终pair的`O(M + E_m + S + E_s)`。相比App旧路径，产品entry已从“两次profile/module graph materialization + overlay + 第二次sort”降为“一次Runtime final freeze + O(M) pair handoff”。candidate内部已有的早期builtin排序仍可能造成额外常数开销，P1-07/P1-37在profile数据确认前保持Partial。

本轮没有可发布性能数据。尚未运行的受管矩阵必须至少记录：`M/S/E_m/E_s`、catalog/plan cache hit、candidate build、final freeze、hash、总composition latency、alloc count/bytes、peak RSS、启动CPU time，以及固定机器/电源策略下的能耗采样。需覆盖1/100/1,000/10,000 module/service规模、五Profile、App/Dynamic/Editor/Export消费者，并与Unreal同类phase/plugin load经验范围按相同口径比较。当前不能声称瓶颈消失、功耗接近其它引擎或算法已达实测最优。

M4当前源码进度新增一项已完成子项：dynamic session保留`RuntimeModuleCompositionCompiler`产出的同一`RuntimeModuleCompositionIdentity`，通过既有可选`profile_control` JSON slot投影版本化`ZrRuntimeModuleCompositionReceiptV1`；App在session构造事务内对缺失capability、error response、漏收据、旧schema以及requested session profile/target不匹配fail closed，并把成功收据交给同代`SessionGateway`/`EditorRuntimeGatewayHandle` snapshot。该子项不重建第二catalog/module graph，不修改V7 C表或slot inventory；Export产品消费、reverse teardown、owner fence/NeedsRestart、受管Editor行为验证与M4产品资格仍未完成。

静态证据：本轮owned Rust文件的直接`rustfmt --check`先报告格式差异，定向format后通过语法解析；`git diff --check`通过（仅工作区既有LF/CRLF提示）。全工作区`cargo fmt --all -- --check`在34秒无输出后超时终止；对`core/runtime/mod.rs`的普通rustfmt递归遇到其它会话占用的profiling文件Windows映射锁，随后使用`skip_children=true`只处理owned文件。限域独立源码review复核profile/target映射、构造失败销毁、冻结identity provenance、Editor同代receipt和新增命令穷尽分支，Critical/Important/Minor均为0，结论仅为可进入受管Cargo验证。受管Cargo、行为测试、性能和功耗仍未完成，因此本记录状态保持`source_implemented_static_passed_managed_validation_pending`，不关闭Runtime42/Runtime136或任何failure handoff。

## 13. 明确禁止的继续实现方式

1. 禁止再增加`runtime_modules_for_*_with_*`组合入口。
2. 禁止App、Editor、DLL或export重新实现manifest/profile/provider选择。
3. 禁止用registration自身selection替代effective project selection。
4. 禁止把同feature ID解释为所有provider都激活。
5. 禁止从多个registry flat-map裸Vec并绕过full registry merge/finalize。
6. 禁止让cfg删除稳定wire enum或Profile intent。
7. 禁止required capability只停留在metadata/tests。
8. 禁止用String同时表示runtime alias、package、provider和module owner。
9. 禁止Rejected report继续暴露可提交module Vec。
10. 禁止用`eprintln!`、自由字符串或调用者自觉检查fatal维持产品正确性。
11. 禁止把源码`contains()`、catalog row数量或未受管test代码当发布证据。
12. 禁止在无generation、replacement registry、owner fence和rollback receipt时宣称hot reload。

## 14. 收口判定

Runtime42/Runtime136只有同时满足以下条件才能关闭：

1. 一个产品输入只生成一个不可变、可hash、可解释的composition plan。
2. 每个selection、provider、capability、module、extension和artifact均可追溯到owner与generation。
3. disabled/unselected/failed provider在commit前被隔离，任何module或extension旁路都不存在。
4. Profile intent、compiled BuildSet和runtime availability分层且闭合，required缺口fail closed。
5. Core、App、Editor、dynamic DLL和export共享同一最终module graph和extension registry truth。
6. activation失败可rollback，shutdown按receipt逆序排空，动态代码释放受owner fence保护。
7. clean Profile矩阵、包装形态、冲突、规模、性能与reload证据均为fresh并绑定plan hash。

当前实现应定义为“已有typed profile/catalog/registry底座，但产品组合authority仍分裂的过渡层”。它尚未达到Unreal级工程化module/plugin assembly，更没有可支持“性能和表现优于Unreal”的可复核证据。
