---
related_code:
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_plugins/README.md
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_plugins/plugin_sdk
  - zircon_plugins/ai
  - zircon_plugins/animation
  - zircon_plugins/asset_importers
  - zircon_plugins/physics
  - zircon_plugins/rendering
  - zircon_plugins/sound
  - zircon_plugins/terrain
  - zircon_plugins/ui_asset_authoring
  - zircon_plugins/virtual_geometry
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/product_host_config/
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs
  - tools/audit_plugin_structure.py
tests:
  - zircon_plugins/first_party_runtime_catalog/src/tests.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests/provider_snapshot.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests/runtime_projection.rs
  - zircon_plugins/first_party_editor_catalog/src/tests.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_app/src/entry/tests/source_assertions.rs
  - zircon_app/src/entry/entry_runner/editor/tests/gui_startup.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/05-shader-wgsl-family-importer-compiler-artifact-native-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/PluginDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ModuleDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/Interfaces/IPluginManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/godot/core/extension/gdextension_library_loader.cpp
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/plugin_group.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Unity.RenderPipelines.Core.Runtime.asmdef
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 06 · First-Party Plugin Source、Editor、Runtime、Dist Catalog、Profile 与 Capability Closure 工程化差距

## 1. 结论

`zircon_plugins`当前是一个真实而且规模已经很大的独立Cargo workspace：`cargo metadata --no-deps`成功解析139个package、139个workspace member和162个target，其中98个lib、41个cdylib、20个integration test、2个custom build与1个bin。39份生成式`plugin.toml`合计3,230行、104,490 bytes，声明54个runtime module row、42个editor module row与39个native module row；30个package至少有一个runtime module，25个至少有一个editor module，37个至少有一个native module。结构审计脚本当前返回`classified-and-clear`，确认39/39 manifest存在、29个runtime descriptor root、41个dist build matrix entry、0 schema/capability/source-owner/skeleton violation。这些基础必须保留。

但“manifest完整”和“产品可materialize”是两件不同的事。通用first-party runtime catalog只依赖并路由14个package：AI、Animation、glTF、Hybrid GI、Navigation、Neural、Net、Particles、Rendering、Solari、Sound、Texture、Virtual Geometry与ZrVM Language。另有16个声明runtime source module的package不在catalog：五个`asset_importer.*`、Audio/OBJ/Opus/Shader WGSL/Texture/UI Document importer、Physics、Prefab Tools、Terrain、Tilemap2D与native fixture。它们有真实`plugin_registration()`、importer/component/system或manager代码，也广告`source_template`/`library_embed`，但通用App source provider path无法返回registration。

Editor差额更大。25个package声明editor module，仓内有40个`editor/src/plugin.rs`入口、4,413行源码；AI、Physics、Terrain、Sound、Rendering等入口确实注册toolkit、command、view、overlay、event consumer或asset contribution。first-party editor catalog却只编译Navigation和Neural两个provider，另外23个package无法通过通用Editor source catalog进入产品。大量Editor专项报告中看到的“descriptor/Workbench存在但默认产品不可达”，在这里可以归结为同一条物理catalog closure，而不是23个互不相关的小遗漏。

profile与build feature还存在确定性矛盾。`runtime-feature-presets.toml`把Client2D/Client3D/Editor/Dev的Sound和Rendering列为required plugin，并要求`runtime.plugin.sound`与`runtime.plugin.rendering`；但Client2D/3D的`app_features`没有`first-party-runtime-plugins`，Editor/Dev只启用advanced-render、Navigation和两个Editor provider，也没有base runtime catalog。`zircon_app`的`target-client`、`target-server`、`target-editor-host`同样不包含base catalog。相关bootstrap测试用`#[cfg(feature = "first-party-runtime-plugins")]`保护，只证明手工加feature后的路径，不证明对应标准profile的feature closure自洽。

selection admission又会掩盖上述问题。两个catalog都遍历`manifest.enabled_for_target()`，无法解析ID或找不到compiled provider时直接`continue`，返回类型只是`Vec<RegistrationReport>`；`required`标记没有进入结果，调用者也收不到`requested/resolved/missing/unsupported`逐项receipt。Editor startup再从返回的registration推导`RuntimeCapabilities`，所以缺provider的required/optional selection会从可观察事实中消失，而不是阻止Ready或显示Unavailable。

生成式export runtime路径是可保留的独立基础：export build plan根据`linked_runtime_crates`生成`ExportRuntimePluginRegistrationProvider::new(crate::plugin_registration)`，因此并非所有SourceTemplate/LibraryEmbed export都依赖first-party catalog。本篇不把通用App/Editor catalog缺口外推成“所有export完全不可用”。但这也证明当前至少有三条provider composition authority：手写first-party catalog、生成式export providers、native discovery/load；三者没有统一的selection resolution receipt和逐包装形态parity。

Plugins01继续拥有SDK/native ABI、load-before-select、签名/trust与39个dist行为空壳；各Runtime/Editor专项拥有算法、authoring与产品语义；Plugins04/05拥有render feature与shader importer专链。本文只拥有39-package source/editor/runtime/dist provider矩阵、built-in profile/build-feature closure、selection resolution、catalog generation与跨包装capability truth。本轮登记 **5项P0、72项P1和16项P2**。

## 2. 物理范围与可复核事实

### 2.1 Workspace 与 Declaration Inventory

| 集合 | 当前事实 | 结论 |
|---|---:|---|
| plugin workspace | 139 package / 162 target | 独立workspace可由metadata完整解析 |
| target kinds | 98 lib / 41 cdylib / 20 test / 2 custom-build / 1 bin | 不是39个简单目录壳，而是大型产品子图 |
| generated package manifests | 39 / 3,230行 / 104,490 bytes | schema/source-owner结构审计通过 |
| module rows | 54 runtime / 42 editor / 39 native | 含Net/Rendering/Sound等optional feature rows |
| package shapes | 30 runtime / 25 editor / 37 native | package级形态并不与module row一一相等 |
| runtime entry files | 53 / 4,822行 / 177,985 bytes | root与optional feature provider入口 |
| editor entry files | 40 / 4,413行 / 166,644 bytes | 多数有真实contribution builder/registration逻辑 |
| dist `lib.rs` | 39 / 4,248行 / 162,941 bytes / 79 tests | ABI/行为真实性归Plugins01，本篇只路由形态 |

39个package maturity为8 stable、7 beta、24 experimental；capability status rows中35项partial、3项complete。三个complete分别是AI Perception、Rendering和Texture，但Runtime08F、Plugins04及render/asset报告已经证明“声明complete”不能代表默认产品可达、真实backend或性能证据。结构审计只验证声明投影一致，不能把status升级成证据状态。

### 2.2 Static Catalog Coverage

| Catalog | 声明package | 已编译/路由 | 缺失 | 覆盖率 |
|---|---:|---:|---:|---:|
| runtime package | 30 | 14 | 16 | 46.7% |
| editor package | 25 | 2 | 23 | 8.0% |

runtime缺失列表是：`asset_importer.audio/data/model/shader/texture`、`audio_importer`、`native_dynamic_fixture`、`obj_importer`、`opus_importer`、`physics`、`prefab_tools`、`shader_wgsl_importer`、`terrain`、`texture_importer`、`tilemap_2d`、`ui_document_importer`。

editor缺失列表是：AI、Animation、Animation Graph、Desktop Export、Editor Contribution Fixture、Hybrid GI、Material Editor、Native Fixture、Native Window Hosting、Net、Particles、Physics、SDK Examples、Prefab Tools、Rendering、Runtime Diagnostics、Sound、Terrain、Texture、Tilemap2D、Timeline Sequence、UI Asset Authoring与Virtual Geometry。

“缺catalog”不等于这些源文件没有实现。例如Physics runtime导出`PhysicsQueryInterface`并注册fixed systems，Terrain注册component和diagnostic-only heightfield importer，UI Document Importer注册真实function importer；AI/Physics Editor都构造真实asset/toolkit/command/view贡献。问题是product composer没有把它们纳入可选择的compiled provider set。

### 2.3 Profile、Feature 与 Product Composition

| Product/profile | 声明要求 | App build feature事实 | 结果 |
|---|---|---|---|
| Client2D/3D | Sound、Rendering required；Texture optional | `target-client`不启用base first-party catalog | 标准feature closure不能提供required source registration |
| Editor | Sound、Rendering required；Texture/Animation/Nav/Particles/Net optional | 只链接advanced render、Nav runtime、Nav/Neural editor | base runtime与绝大多数editor source provider缺席 |
| Dev | 同Editor并额外default Net | 同`target-editor-host` | Net source provider不因profile默认选择自动链接 |
| Server | builtin core；未来project可选择server plugin | `target-server`不链接任何first-party catalog | project source selection需要额外手工feature或generated provider |
| Generated export | selection生成linked provider function pointers | 不依赖hand-written catalog覆盖 | 路径可保留，但必须与其余形态共享resolution receipt |

2026-08-27后`EntryConfig`只记录产品请求，`resolve()`一次生成`ResolvedProductHostConfig`并合并runtime-profile manifest、显式project selection与render overlay；Editor provider投影和module composition复用同一结果。`EntryConfig::new(Editor)`仍不等同于显式`RuntimeProfileId::Editor`，因此Editor角色baseline与profile catalog的最终统一继续由本计划负责，不能因App配置权威收敛而提前关闭。

### 2.4 Selection Resolution 实际语义

```text
ProjectPluginManifest selections
  -> enabled_for_target
  -> RuntimePluginId::parse_key
  -> hand-written cfg branch lookup
  -> Some(registration): push
  -> None/invalid: continue
  -> Vec<RegistrationReport>
  -> derive RuntimeCapabilities only from returned registrations
```

这条链丢弃selection index、required、packaging、requested version、expected crate/artifact与missing reason。调用者无法区分disabled、target mismatch、catalog没编译、provider代码没生成、unsupported packaging、registration失败或恶意ID。Editor catalog的App adapter还只以`first-party-navigation-editor-plugin`作为整个实现分支cfg：理论上的Neural-only feature组合会走fallback空Vec，即使`first-party-neural-editor-plugin`已启用。

### 2.5 动态证据边界

本轮实际运行两条只读结构命令：

1. `python tools/audit_plugin_structure.py --json`成功，返回39 manifests、29 runtime descriptor roots、41 dist matrix、0结构违规；脚本不比较30/25声明package与14/2 product catalog coverage。
2. `cargo metadata --manifest-path zircon_plugins/Cargo.toml --no-deps --format-version 1`成功，解析139 packages/162 targets；`zircon_plugins/Cargo.lock`前后均未修改。

本轮不运行Cargo compile/tests。Editor依赖lane仍受既有239个compile errors阻断；重跑同一未变化lane不会增加证据。metadata、manifest check、source-shape test和cfg下test数量都不能记为product startup或provider parity pass。

## 3. 参考引擎约束

- Unreal `FPluginManager::ConfigureEnabledPlugins()`先解析target/project/required dependency并维护`bHaveAllRequiredPlugins`，`LoadModulesForEnabledPlugins()`只有在configure成功后才按`ELoadingPhase`加载；`AreRequiredPluginsAvailable()`直接复用configure结果。Zircon无需复制UBT，但required selection不能在provider lookup中静默消失。
- Godot extension manager把load/reload与initialize/deinitialize level放在显式manager中，loader错误会形成可返回结果。Zircon的source/native/export provider也需要进入一个resolution/lifecycle coordinator，而不是三条路径各自返回不同形状。
- Bevy `PluginGroupBuilder`通过add/set/disable/finish形成确定的plugin group，重复唯一plugin会报错；它是静态Rust组合参考，不是DLL ABI答案。可借鉴的是build-time group必须与runtime installed set同一真值并可枚举。
- Fyrox将静态plugin与dynamic plugin明确分开，dynamic owner保留library并在reload时重新fill/register；源码还明确警告Rust dylib不适合作为稳定production ABI。Zircon必须保持Source/Library/Native形态差异，但用同一个selection outcome与行为资格矩阵约束它们。
- Unity Graphics的`package.json`列版本与dependencies，runtime `.asmdef`再定义可编译assembly边界。仓内镜像不包含Unity完整Package Manager，本文只借鉴“package metadata与compiled assembly closure必须同时成立”，不外推安装/信任行为。

## 4. 可保留基础

1. 39个manifest由Rust declaration生成且结构审计无漂移。
2. plugin workspace能独立解析139个package与所有target，不污染根workspace成员。
3. RuntimePluginId支持动态ID并规范化常用alias，不把生态永久锁成enum。
4. runtime/editor provider catalog隔离了`zircon_app`对具体plugin crate的直接fan-out方向。
5. catalog按target过滤并用HashSet去重，避免同一manifest重复注册。
6. provider crate入口普遍有typed registration report，而非直接把全局registry暴露给App。
7. generated export按linked runtime crate生成function pointer，证明catalog可以被生成式closure取代。
8. plugin manifest包含target/platform/capability/maturity/packaging/module/dependency与部分status信息。
9. Runtime/Editor专项已经逐域审过大多数算法与authoring surface，可作为provider readiness的下游证据输入。
10. audit script已有machine-readable JSON，适合扩展而不是另造不可组合脚本。

## 5. P0：Catalog、Profile 与 Capability Truth 硬阻断

### FP-CATALOG-P0-001 · Required selection在provider lookup中可无声消失

两个catalog遇到invalid ID或未编译provider都直接`continue`，返回Vec不携unresolved rows；required与optional使用同一路径。产品无法证明“所有required plugin已经解析”，也不能把missing provider与disabled/target mismatch区分。任何Ready/capability/Editor visible projection都可能建立在不完整selection set上。

改为`PluginResolutionPlan -> PluginResolutionReceipt`：每个selection保留identity、required、target、packaging、source、requested provider与outcome；required的Missing/Unsupported/Failed必须阻止composition commit，optional才允许typed Degraded。所有product composer只消费成功plan，不直接消费裸registration Vec。

### FP-CATALOG-P0-002 · Built-in profile required plugin与App feature closure确定性冲突

Client2D/3D、Editor/Dev把Sound/Rendering设为required并声明required capability，但对应`app_features`与`zircon_app` target feature没有base first-party catalog。只有显式额外启用`first-party-runtime-plugins`的cfg tests能得到这三个provider。标准profile source配置无法自证其required closure。

用同一generated `ProductProviderBuildMatrix`生成runtime profile、App Cargo feature、catalog feature和tests；build.rs在任何required plugin无compiled provider时失败。每个标准profile至少要有clean feature build与startup receipt，不能依靠README附加隐藏feature修正profile。

### FP-CATALOG-P0-003 · 16个runtime source provider不在通用catalog，却广告SourceTemplate/LibraryEmbed

30个runtime package只有14个可由通用App catalog解析。Physics、Terrain、Prefab、Tilemap与十个importer等真实registration入口存在，manifest仍把source/library作为包装能力；普通project选择却只会被静默跳过。静态/source形态的可用性声明和compiled product closure不一致。

catalog不得继续手写逐ID分支。由39-package declaration与product profile生成provider table；每个SourceTemplate/LibraryEmbed package都必须有crate、feature、registration symbol、target/platform和dependency closure，或从该产品的available packaging中移除。generated export与dev App消费相同矩阵。

### FP-CATALOG-P0-004 · 23个Editor source provider没有产品catalog入口

25个editor package只有Navigation/Neural进入first-party editor catalog。AI、Physics、Sound、Rendering、Terrain、UI Asset等真实source registration无法通过普通Editor project manifest materialize；但manifest、Plugin Manager与Workbench仍能展示它们的capability或描述符。产品看见能力和compiled provider之间没有fail-closed关系。

建立generated `EditorProviderCatalog`，按package/target/feature链接全部获准source provider，并在startup前生成resolution receipt。未编译provider的package必须显示Unavailable且不注册command/view/template；不能用core Workbench或descriptor fallback维持“可见但不可执行”。

### FP-CATALOG-P0-005 · `complete`与结构绿灯不绑定provider、artifact、runtime generation或证据

39份manifest中AI Perception、Rendering、Texture有`complete` status；audit的`classified-and-clear`又只表示结构一致。status不消费provider resolution、artifact、product caller、runtime generation、测试BuildSet或性能证据。现有专项报告已经静态推翻部分complete语义，但声明仍可继续进入catalog/UX。

把capability状态拆为Declared/Compiled/Packaged/Installed/Registered/Ready/Qualified，并绑定owner、BuildSet、artifact、target、generation与EvidenceSet。`complete`只能是满足版本化qualification policy后的派生状态；source declaration不得手写终局完成度。

## 6. P1：Inventory、Schema 与 Generated Matrix

| ID | 当前差距 | 需要重构 |
|---|---|---|
| FP-CATALOG-P1-001 | 39 package、139 crate、162 target没有统一machine-readable关系表 | 生成Package→Module→Crate→Target→Artifact→Provider图 |
| FP-CATALOG-P1-002 | module row与package shape统计语义不同 | schema区分root module、optional feature module、fixture与dist projection |
| FP-CATALOG-P1-003 | Cargo target kind不进入plugin declaration | build matrix记录lib/cdylib/build/test/bin及required target |
| FP-CATALOG-P1-004 | manifest只保存crate name，不保存package/version/source | 绑定Cargo package ID、source digest、lock与workspace generation |
| FP-CATALOG-P1-005 | catalog依赖手写Cargo feature组 | declaration生成feature group与provider dependency，不双写列表 |
| FP-CATALOG-P1-006 | runtime/editor catalog覆盖率未被audit | audit新增30/25 declared与compiled/route set差分 |
| FP-CATALOG-P1-007 | 41 dist entry与39 native module row关系不透明 | 显式标注base/optional-feature/fixture projection与owner package |
| FP-CATALOG-P1-008 | maturity/status缺source revision | 每条声明带policy version、owner与last-qualified evidence identity |
| FP-CATALOG-P1-009 | dependency只验证schema形状 | 解析required/optional/interface/capability到实际provider closure |
| FP-CATALOG-P1-010 | package alias与canonical ID未进artifact | manifest、selection、catalog、loader统一canonical normalized ID |

## 7. P1：Selection、Resolution 与 Admission

| ID | 当前差距 | 需要重构 |
|---|---|---|
| FP-CATALOG-P1-011 | catalog返回裸Vec | 返回逐selection resolution rows与全局terminal outcome |
| FP-CATALOG-P1-012 | `required`未被provider lookup读取 | required missing/failed阻止commit，optional产生Degraded reason |
| FP-CATALOG-P1-013 | invalid ID被静默跳过 | 保留selection index/raw ID并返回InvalidIdentity |
| FP-CATALOG-P1-014 | target mismatch与provider missing不可区分 | stable DisabledForTarget/NotCompiled/UnsupportedPlatform等状态 |
| FP-CATALOG-P1-015 | packaging策略不参与lookup | Source/Library/Native分别验证provider/artifact与允许组合 |
| FP-CATALOG-P1-016 | selection version/SDK/ABI未贯穿 | resolution绑定package/SDK/ABI/schema兼容结果 |
| FP-CATALOG-P1-017 | duplicate只按normalized ID丢弃 | duplicate相同配置可合并，冲突配置必须报Conflict |
| FP-CATALOG-P1-018 | provider registration失败与缺失混同 | Prepared/Registered/Failed阶段保留error chain与rollback |
| FP-CATALOG-P1-019 | capability从成功rows反推，遗漏不可见 | capability projection同时消费requested与resolution outcome |
| FP-CATALOG-P1-020 | 无resolution generation | project/profile/catalog/provider变化使旧receipt失效 |
| FP-CATALOG-P1-021 | 无dependency chain诊断 | missing required interface报告完整package→provider链 |
| FP-CATALOG-P1-022 | resolution没有budget | 限制selection/dependency/module/capability数量与解析work |

## 8. P1：Runtime Source Provider Closure

| ID | 当前差距 | 需要重构 |
|---|---|---|
| FP-CATALOG-P1-023 | 16 runtime package不在catalog | 自动生成完整provider table或明确产品排除矩阵 |
| FP-CATALOG-P1-024 | base/advanced/nav/zr feature组人工维护 | 由profile与target policy生成可组合feature bundles |
| FP-CATALOG-P1-025 | target-client未启用base providers | 标准Client profile构建闭合required Sound/Rendering/Texture policy |
| FP-CATALOG-P1-026 | target-server无first-party provider path | server选择AI/Physics/Net等时有显式source或native provider策略 |
| FP-CATALOG-P1-027 | target-editor只链接局部runtime provider | Editor simulation/preview按project selection使用同一runtime set |
| FP-CATALOG-P1-028 | Physics有真实registration但静态不可选 | provider matrix链接Physics或声明仅external/native并fail-close |
| FP-CATALOG-P1-029 | Terrain/Tilemap/Prefab存在registration孤岛 | 逐包补product caller与compiled provider资格 |
| FP-CATALOG-P1-030 | 十类importer source provider缺catalog | importer selection从同一asset pipeline provider registry解析 |
| FP-CATALOG-P1-031 | plugin root与optional feature注册混合 | base package/feature provider分别产生typed registration kind |
| FP-CATALOG-P1-032 | render overlay只补三项且可能重复authority | overlay输出selection intent，统一resolution负责去重/冲突 |
| FP-CATALOG-P1-033 | `RuntimePluginId`alias可改变输入字符串 | receipt保留raw与canonical ID，hash只使用canonical qualified ID |
| FP-CATALOG-P1-034 | no-feature fallback返回空Vec | 返回CatalogNotCompiled，required selection不能被吞掉 |

## 9. P1：Editor Source Contribution Closure

| ID | 当前差距 | 需要重构 |
|---|---|---|
| FP-CATALOG-P1-035 | 23 editor package未链接 | 生成完整EditorProviderCatalog与product feature closure |
| FP-CATALOG-P1-036 | App adapter只以Navigation feature做总cfg | cfg条件覆盖任一editor catalog feature，或只保留单generated feature |
| FP-CATALOG-P1-037 | Neural-only组合会走空fallback | feature powerset compile test验证每个单包与组合 |
| FP-CATALOG-P1-038 | AI真实toolkit/command无法materialize | package resolution成功后才注册AI contributions与runtime mirror |
| FP-CATALOG-P1-039 | Physics/Navigation等debug view路径不统一 | source/native/editor注册共享同一contribution identity与generation |
| FP-CATALOG-P1-040 | Editor-only包没有统一provider组 | Material/Timeline/UI/Diagnostics等按authoring profile生成链接 |
| FP-CATALOG-P1-041 | Editor plugin依赖runtime mirror未预检 | resolution验证mirror package/version/event schema先于UI注册 |
| FP-CATALOG-P1-042 | missing provider仍可能保留core Workbench | 无provider时view/command/template全部不可注册并显示typed reason |
| FP-CATALOG-P1-043 | contribution注册缺原子commit | 在staging registry校验全部ID/dependency后一次发布 |
| FP-CATALOG-P1-044 | 多plugin冲突没有package级receipt | command/view/asset/template collision回到具体provider并rollback |
| FP-CATALOG-P1-045 | Editor unload未绑定catalog generation | 注销所有contribution/operation/mirror后才释放provider |
| FP-CATALOG-P1-046 | Editor capability数来自声明 | visible/enabled/ready分别绑定registration、surface与operation factory |

## 10. P1：Profile、Build Feature 与 Product Identity

| ID | 当前差距 | 需要重构 |
|---|---|---|
| FP-CATALOG-P1-047 | preset与App feature双写 | 从单一profile schema生成Runtime/App/catalog features |
| FP-CATALOG-P1-048 | required plugin未映射required Cargo dep | build-time closure验证每个profile的compiled providers |
| FP-CATALOG-P1-049 | `EntryConfig::new`不应用profile descriptor | 明确raw profile与builtin runtime profile，不允许同名不同manifest |
| FP-CATALOG-P1-050 | Editor项目manifest覆盖内建default集合 | 定义merge/override策略并保留required baseline |
| FP-CATALOG-P1-051 | default project无manifest时走空selection | profile默认selection必须显式进入resolution |
| FP-CATALOG-P1-052 | profile tests受额外cfg保护 | 每个shipping target用其真实feature set运行required closure test |
| FP-CATALOG-P1-053 | README手工追加feature补产品 | 生成受支持命令/profile，未知组合在启动前拒绝 |
| FP-CATALOG-P1-054 | build feature不进入BuildSet | receipt记录Cargo features、provider packages与target artifacts |
| FP-CATALOG-P1-055 | Editor/Dev共享Cargo feature但selection不同 | 每个profile单独生成/验证provider resolution golden |
| FP-CATALOG-P1-056 | optional provider缺失没有SLO/UX policy | profile声明optional degradation、替代provider与operator提示 |

## 11. P1：Packaging、Export、Native 与 Parity

| ID | 当前差距 | 需要重构 |
|---|---|---|
| FP-CATALOG-P1-057 | 三条composition authority分离 | source/export/native都实现统一ProviderResolver接口与receipt |
| FP-CATALOG-P1-058 | generated export provider不回写catalog identity | generated source携package/module/schema/artifact fingerprint |
| FP-CATALOG-P1-059 | SourceTemplate与LibraryEmbed未区分build semantics | 分别记录source closure、compiled library、link flags和license |
| FP-CATALOG-P1-060 | NativeDynamic readiness沿用manifest能力 | loader成功、behavior registration、state/lifecycle通过后才Ready |
| FP-CATALOG-P1-061 | 39 dist壳行为问题与catalog状态分离 | 消费Plugins01 parity result，失败时包装形态从available移除 |
| FP-CATALOG-P1-062 | optional feature dist与owner package关系弱 | provider_package_id、owner、dependency与feature generation同代 |
| FP-CATALOG-P1-063 | packaging fallback未定义 | requested形态缺失时fail或经明确policy选替代，不能静默换形态 |
| FP-CATALOG-P1-064 | editor provider没有generated export类路径 | Editor build生成source provider catalog，不手写逐包features |
| FP-CATALOG-P1-065 | 安装artifact与compiled source可重复注册 | resolution按qualified provider identity检测跨形态冲突 |
| FP-CATALOG-P1-066 | static/native行为差分不进release gate | 每包至少有registration/capability/lifecycle/state golden parity |

## 12. P1：验证、可观测性与维护

| ID | 当前差距 | 需要重构 |
|---|---|---|
| FP-CATALOG-P1-067 | audit不查catalog覆盖率 | 输出declared/compiled/routed/packaged/qualified五集合差分 |
| FP-CATALOG-P1-068 | metadata成功被误作compile | 结果明确MetadataOnly，required crate build另设lane |
| FP-CATALOG-P1-069 | catalog tests主要source-shape/cfg | 生成39包target/strategy矩阵的真实resolution tests |
| FP-CATALOG-P1-070 | 无standard profile clean-build矩阵 | Client2D/3D/Editor/Dev/Server按真实features build+startup |
| FP-CATALOG-P1-071 | 无product resolution telemetry | 记录requested/resolved/missing/degraded、耗时、dependency与generation |
| FP-CATALOG-P1-072 | source drift不自动失效完成状态 | manifest/catalog/profile/provider fingerprint变化触发recheck与降级 |

## 13. P2：长期能力

| ID | 能力 | 目标 |
|---|---|---|
| FP-CATALOG-P2-001 | Generated provider registry | 无手写逐ID分支，支持第三方扩展表 |
| FP-CATALOG-P2-002 | Profile solver | 按capability、target、quality、trust与budget选择provider |
| FP-CATALOG-P2-003 | Multi-provider variants | 同feature可选择CPU/GPU/vendor实现并报告决策 |
| FP-CATALOG-P2-004 | Package graph viewer | Editor展示selection、dependency、artifact与missing chain |
| FP-CATALOG-P2-005 | Feature powerset testing | 自动选择高风险组合而非穷举全部2^N |
| FP-CATALOG-P2-006 | Hermetic plugin SDK matrix | 旧/当前/next SDK与多toolchain consumer验证 |
| FP-CATALOG-P2-007 | Cross-platform package qualification | Windows/Linux/macOS/mobile/web/headless独立receipt |
| FP-CATALOG-P2-008 | Incremental provider link cache | 按完整BuildSet复用source/library编译产物 |
| FP-CATALOG-P2-009 | Capability fallback policy | 明确quality降级、替代provider与用户确认 |
| FP-CATALOG-P2-010 | Hot provider replacement | quiesce、state migration、双代切换与rollback |
| FP-CATALOG-P2-011 | Plugin sandbox profile | 低信任provider进程隔离与最小capability broker |
| FP-CATALOG-P2-012 | Remote package registry | signed metadata、content-addressed artifact与revocation |
| FP-CATALOG-P2-013 | License/SBOM closure | package/provider/artifact依赖生成可发布SBOM |
| FP-CATALOG-P2-014 | Usage/deprecation telemetry | 先证明旧package/member无consumer再retire |
| FP-CATALOG-P2-015 | Behavior differential suite | source/library/native对相同fixture输出与lifecycle差分 |
| FP-CATALOG-P2-016 | Ecosystem compatibility dashboard | SDK/engine/profile/package/target资格矩阵可查询 |

## 14. 目标架构

```text
39 Plugin Declarations + Cargo Metadata + Product Profiles
                         |
                         v
             ProviderBuildMatrixCompiler
                         |
       +-----------------+------------------+
       |                 |                  |
       v                 v                  v
Source/Library Catalog  Editor Catalog   Native Artifact Catalog
       |                 |                  |
       +-----------------+------------------+
                         v
              ProductPluginResolver
                         |
       Requested -> Resolved/Missing/Failed receipt
                         |
                         v
            Transactional Registration Plan
                         |
       Runtime Registry + Editor Registry + Capabilities
                         |
                         v
       BuildSet/Generation/Evidence-bound Ready state
```

核心记录建议：

- `ProviderBuildMatrixRow`：package/module/crate/target/strategy/feature/artifact/registration symbol。
- `PluginSelectionRequest`：canonical/raw ID、required、target、packaging、version、feature与principal/policy。
- `PluginResolutionRow`：Resolved、Disabled、TargetMismatch、NotCompiled、ArtifactMissing、Incompatible、Failed。
- `PluginResolutionReceipt`：全selection覆盖、dependency closure、BuildSet、catalog generation与terminal status。
- `PluginRegistrationTransaction`：staging、conflict validation、commit、rollback、generation与unload receipt。
- `CapabilityQualification`：Declared到Qualified的分阶段状态与EvidenceSet。

## 15. 分层里程碑

### M0 · Inventory Truth Freeze

把39 package、139 crate、162 target、54/42/39 module rows与14/2 catalog coverage生成成单一矩阵；audit对missing route和orphan route fail。现有结构绿灯改名为SchemaConformance，禁止解释为product readiness。

### M1 · Resolution Receipt 与 Required Fail-Closed

替换两个裸Vec catalog API。所有selection产生outcome；required missing阻止product composition，optional missing进入typed degraded。App/Editor/Export/Native共享schema。

### M2 · Profile/Feature Closure Generation

从runtime profile schema生成App Cargo features与provider groups，修复Client/Editor required Sound/Rendering closure；标准profile按真实features clean build/startup。

### M3 · Runtime Source Catalog Full Matrix

为30个runtime package逐个决定Link/External/Unsupported，补齐16个当前缺失owner。importer、system、interface和feature provider按target/dependency有序注册。

### M4 · Editor Source Catalog Full Matrix

链接25个Editor provider或明确排除；23个当前缺失包先保持Unavailable，直到surface、operation factory、runtime mirror与artifact资格通过。

### M5 · Cross-Packaging Parity

消费Plugins01的ABI/trust/dist修复，将source/library/generated export/native统一到一个provider resolver与registration transaction；逐包建立行为差分。

### M6 · Qualification 与 Release

capability状态绑定BuildSet、target、artifact、runtime generation与EvidenceSet；profile/package/strategy矩阵进入release gate与生态dashboard。

## 16. 必须通过的验收门

1. 39个package、全部module row、139 crate与162 target均在ProviderBuildMatrix中且0 orphan。
2. 30个runtime与25个editor package各自有Link/External/Unsupported明确决策。
3. catalog declared/compiled/routed集合差异由CI machine-readable报告并阻止required缺口。
4. 每个selection产生唯一resolution row，invalid/missing/target mismatch不再静默丢弃。
5. required missing/failed阻止Ready，optional missing产生稳定Degraded reason。
6. Client2D/3D、Editor、Dev的required Sound/Rendering feature closure可由标准命令构建。
7. Server project选择受支持first-party plugin时无需隐藏feature或得到明确Unsupported。
8. runtime source catalog当前16个缺口全部materialize或移除对应产品包装声明。
9. editor source catalog当前23个缺口全部materialize或在UI fail-close。
10. Neural-only、Navigation-only及任意单Editor provider feature组合不会走空fallback。
11. project manifest与profile default有明确merge/override规则且required baseline不丢失。
12. generated export provider与App catalog使用同一package/module/schema identity。
13. SourceTemplate、LibraryEmbed、NativeDynamic分别验证其完整artifact/provider closure。
14. 同package跨形态重复provider被检测，不能双注册system/interface/command/view。
15. Runtime/Editor registration在staging registry全量校验后原子commit，失败可rollback。
16. unload/reload按逆依赖序注销contribution并使旧generation handle失效。
17. `complete`状态必须有BuildSet/target/artifact/product caller/test/perf EvidenceSet。
18. audit结构绿灯与product qualification状态分栏，不再使用同一个“clear”概念。
19. 39包至少完成registration/capability/lifecycle的source/library/native差分矩阵。
20. clean profile build、startup、package、install与resolution receipt进入release required lane。

## 17. 验证状态与禁止误判

本轮只新增review文档，没有修改production、tests、Cargo manifest、lockfile、generated plugin manifest或dist artifact。结构audit与Cargo metadata是本轮新鲜结果；没有运行compile/test/product startup，也没有重复既有Editor失败lane。

以下结果不能关闭本报告：

- 39/39 `plugin.toml`存在，不代表39个package进入产品catalog。
- 139 package metadata成功，不代表任一provider能编译或启动。
- 41个cdylib target存在，不代表dist行为与source等价。
- `classified-and-clear`不代表14/2 catalog覆盖完整。
- `plugin_registration()`源码存在，不代表App feature实际链接它。
- profile声明required不代表Cargo feature closure提供provider。
- Editor Workbench或capability descriptor可见，不代表Editor provider已注册。
- generated export路径存在，不代表通用App、Editor和native路径自动同语义。

关闭P0必须先完成M0-M4；整篇完成还要求M5-M6逐包装形态parity与资格证据。内部算法finding仍由各Runtime/Editor专项关闭，不能用catalog连通替代功能正确性、性能或表现验收。
