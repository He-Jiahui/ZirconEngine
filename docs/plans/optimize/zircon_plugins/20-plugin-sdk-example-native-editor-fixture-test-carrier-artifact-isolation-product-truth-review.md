---
title: Plugin SDK Example、Native Dynamic Fixture、Editor Contribution Fixture、Test Carrier、Artifact Isolation 与 Product Truth 工程化差距
category: zircon_plugins
report_id: Plugins20
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_plugins/plugin_sdk_examples
  - zircon_plugins/native_dynamic_fixture
  - zircon_plugins/editor_contribution_fixture
  - zircon_plugins/Cargo.toml
  - zircon_plugins/README.md
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_editor/build.rs
  - zircon_editor/src/core/plugin
  - zircon_editor/src/ui/host/editor_manager_plugins_export
  - zircon_runtime/src/plugin/package_manifest
  - zircon_runtime/src/plugin/native_plugin_loader
  - zircon_app/src/entry/first_party_editor_plugins.rs
tests:
  - zircon_plugins/plugin_sdk_examples/editor/src/tests.rs
  - zircon_plugins/plugin_sdk_examples/dist/src/lib.rs
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - zircon_editor/src/tests/editor_plugin_catalog_consistency.rs
  - zircon_editor/src/tests/editor_plugin_sdk.rs
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader/real_fixture.rs
  - zircon_runtime/src/tests/plugin_extensions/plugin_workspace_shape.rs
  - tools/tests/test_editor12_editor_contribution_fixture_contract.py
  - .github/workflows/ci.yml
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/03-desktop-export-native-window-source-dist-provider-integration-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/PluginDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ModuleDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/PluginReferenceDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ModuleDescriptor.cpp
  - dev/UnrealEngine/Engine/Plugins/Tests/TestSamples/TestSamples.uplugin
  - dev/UnrealEngine/Engine/Plugins/Tests/TestFramework/TestFramework.uplugin
  - dev/UnrealEngine/Engine/Plugins/Tests/ModularTestFrameworkTests/ModularTestFrameworkTests.uplugin
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/examples/app/plugin.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/godot/core/extension/gdextension.h
  - dev/godot/core/extension/gdextension.cpp
  - dev/godot/tests/compatibility_test/godot/compatibility_test.gdextension
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/Unity.RenderPipelines.Core.Editor.Tests.asmdef
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/package.json
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 20 · Plugin SDK Example、Native Dynamic Fixture、Editor Contribution Fixture、Test Carrier、Artifact Isolation 与 Product Truth 工程化差距

## 1. 结论

`plugin_sdk_examples`、`native_dynamic_fixture` 和 `editor_contribution_fixture` 不是三个无害的测试目录。它们都位于正常第一方插件 workspace，使用正常 `plugin.toml` 和正常 package identity；其中 SDK 示例与 Native Fixture 进入 standalone dist CI，三份 manifest 又都声明 editor module。`zircon_editor/build.rs` 会无条件扫描 `zircon_plugins/*/plugin.toml`，只要看到 editor module 就生成 builtin descriptor；它不读取 `maturity`，也没有 fixture、sample、developer-only、hidden、enabled-by-default 或 shipping policy。`EditorPluginManager::builtin` 随后将这些 descriptor 推进 Default loading phase，产品 Plugin Manager 的 status projection也会把它们作为 builtin package行展示。

这条路径把“物理目录存在”错误地等同于“产品插件存在”。三个载体在 manifest 中都因缺省值被解析为 `PluginPackageKind::Standard`，而 `Experimental` 只是一段没有进入 editor catalog、status、selection或export admission的描述性 metadata。结果不是单纯的 UI 噪声：测试插件、样例插件和可交付产品在 package schema、catalog generation、artifact build、project selection和shipping gate中无法区分。

SDK 示例的产品真值尤其断裂。source editor crate确实注册了两个 view、三个 menu item、四个 command、一个 glTF/GLB importer、一个 Model toolkit、两个 UI template、一个 creation template和一个 inspector customization；但产品 first-party editor provider catalog只链接 Navigation 与 Neural，并不链接这个 crate。它声明的三个资源文件及 `assets/`、`examples/` 两个根目录全部不存在，四个 operation在包外没有 handler/factory。其 `native_dynamic` dist又只发布一个 `extensions: []` 的 registration manifest，诊断文字明确承认行为仍由 source editor module承载。换言之，Source/Library形态没有产品 provider，Native形态没有业务贡献，没有任何一个标准交付形态能兑现 manifest宣称的能力。

两个 fixture也不仅是 metadata。Native Fixture暴露 runtime/editor entry、一个 `write:world` Update system、一个 importer、四个 command、state save/restore、panic capture、host callback和故障变体；但是 system bridge只是返回成功，事件从不发出，editor entry没有任何贡献。Editor Contribution Fixture会物化 view、drawer、menu、command、asset type和settings page，却同时声明空 command manifest，`invoke_command` 对所有 slot返回 DENIED；Editor侧也不存在把 materialized operation路由到 native behavior callback的产品 adapter。因此它最显眼的菜单命令按合同必然不可执行，现有 Python gate只解析 Rust源码中的 JSON 字符串，没有构建或加载 DLL。

Plugins01 已拥有 native ABI soundness、selection前加载、trust/signature、V2 debt、SDK API版本和39个 metadata shell等共享问题；Plugins06 已拥有第一方 source/dist/catalog closure；Editor06 已拥有通用 Plugin Manager与serialized contribution消费缺口。本篇不重复这些计数。本轮新增 owner 是 fixture/sample/product role、catalog allowlist、shipping exclusion、carrier parity和测试载体本身的真实性，登记 **3项P0、40项P1、10项P2**，并给出32项资格门。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes | 冻结事实 |
|---|---:|---|
| 三个 carrier root | 18 / 1,708 / 65,063 | 10个 SDK example文件、5个 native fixture文件、3个 editor fixture文件；审查时均为clean |
| package/catalog/product consumer | 20 / 5,508 / 208,569 | plugin workspace、Editor build-generated catalog、status/native materialization、manifest schema、native loader与App provider projection |
| focused tests/CI | 9 / 4,074 / 147,588 | Editor catalog/SDK、real DLL fixture、workspace shape、static Python contract和standalone CI |
| 合计 | 47 / 11,290 / 421,220 | fingerprint `16f49498897d75469a036d8e7ad9db7569b12dd85d18a21b7843d80dd02c2f7a` |
| carrier-only fingerprint | 18 / 1,708 / 65,063 | `effe6671fe2215fc18bfe3d3fcd0a2eb92162497baffc3ea9bad5ce475d67317` |
| reference engines | 18 / 8,289 / 322,642 | fingerprint `0e44deb7f9232f3fd4eaef433639954608a864fe52ddf0e310a1aca90b2e0041` |

冻结 revision 为 `bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator baseline epoch为335。三个 carrier root在审查时没有本地修改；47文件组合范围中 `native_registration/manager.rs`、两个 Editor test和 `plugin_workspace_shape.rs` 共4文件处于其他 Session或用户修改状态，因此标记 `source_recheck_required: true`。本文没有修改 production、tests、CI或tooling，也不把组合 fingerprint解释为clean build证据。

### 2.2 证据等级

本轮逐文件阅读三个 carrier root，沿 package declaration追到 generated `plugin.toml`、plugin workspace、Editor build script、builtin descriptor、manager Default phase、Plugin Manager status、first-party source provider、native loader、serialized contribution materializer、real DLL tests和standalone CI。参考侧读取 Unreal Projects descriptor/manager与三份测试插件、Bevy plugin lifecycle和example、Fyrox static/dynamic plugin边界、Godot GDExtension initialization/reload与compatibility fixture、Unity Graphics package sample/test assembly隔离。结论属于E3静态调用链、artifact和产品装配审查。

没有运行 Cargo、CI、真实 Editor、DLL加载、export、跨平台、sanitizer、fuzzer或性能测试。`real_fixture.rs` 的8个真实DLL测试是可保留证据，但不代表本轮 current source已经执行通过；Python源码字符串测试、manifest `.contains()`和build matrix存在也不能替代产品资格。

### 2.3 与既有报告的边界

- Plugins01继续拥有 native ABI内存/线程/卸载安全、panic跨ABI、V2、trust、signature、selection前加载和通用 dist shell；Plugins20只负责 fixture/sample是否应进入这些生产路径。
- Plugins06继续拥有39个第一方package的provider/catalog/profile/capability closure；Plugins20只负责三个 carrier的role、allowlist、shipping policy和形态等价性。
- Plugins08继续拥有通用Editor command/document/template/toolkit合同；Plugins20只证明SDK example的具体资源、operation和importer无法兑现。
- Editor06继续拥有 Plugin Manager status、enablement、native materialization和serialized contribution通用消费；Plugins20只要求 fixture能真实验证这些通路且不会污染产品目录。
- Runtime07、Runtime Interface01和Runtime04继续拥有native isolation、FFI ownership和artifact store；Plugins20不得另建第二套loader、ABI或cache。

## 3. 应保留的真实基础

1. `declare_plugin!` 能从一个 declaration生成 package/module/capability/native registration常量，三个载体都在使用这条主路径。
2. Native Fixture真实构建 cdylib，并覆盖descriptor、runtime/editor entry、capability negotiation、V3/V2、missing export、state、unload、host callback、output sink和panic capture。
3. `real_fixture.rs` 使用隔离临时workspace与 `--offline` 构建真实动态库，而不是只对 source字符串做断言。
4. V4 command table使用dense slot和host-owned bounded output sink；panic command通过 `catch_native_callback_panic` 转成状态，不直接越过C ABI。
5. Runtime registration manifest已经能表达system、access、thread affinity、event、extension和bridge method。
6. Editor contribution batch使用版本化schema、deny unknown fields、按kind/id排序并拒绝duplicate或错误kind schema。
7. Editor materializer以candidate registry原子应用batch，失败时不会发布部分贡献；package_id不匹配会fault整个registration。
8. Native editor registration会保留selected但不可用的package诊断，而不是静默从catalog消失。
9. SDK source示例覆盖view、menu、command、asset importer、toolkit、creation template和inspector customization，适合作为完整SDK conformance sample的起点。
10. CI已有standalone manifest validate与dist matrix入口，可以升级为role-aware artifact qualification，而不必重新发明执行平台。

## 4. 参考引擎给出的最低约束

### 4.1 Unreal Engine

Unreal的 `FPluginDescriptor` 将 `EnabledByDefault`、`bInstalled`、`bIsHidden`、`bExplicitlyLoaded`、experimental/beta、supported target/platform/program和module列表分开；`FModuleDescriptor` 进一步按 Runtime、DeveloperTool、Editor、Program、Server/Client、platform architecture、target type和build configuration决定是否编译、是否加载。`FPluginManager` 不把“扫描到descriptor”等同于“启用并挂载”。

仓内测试插件也不是靠命名约定隔离。`TestSamples` 和 `RuntimeTests` 显式 `EnabledByDefault: false`；`TestFramework` 使用 `DeveloperTool`；`ModularTestFrameworkTests` 同时声明 `TestPlugin: true`、`ExplicitlyLoaded: true` 和 `TargetConfigurationDenyList: [Shipping]`。Zircon不必复制UE字段名，但必须提供同等级的package role、default enablement、visibility、explicit load和shipping/configuration gate。

### 4.2 Unity Graphics

Unity Graphics package把可安装样例放在 `Samples~`，由 `package.json.samples` 显式列出并按需导入；测试程序集位于 `Tests`，`autoReferenced: false`，由 `UNITY_INCLUDE_TESTS` define约束，Editor测试还用 `includePlatforms: [Editor]`。这让sample/test artifact不因物理存在而自动成为产品runtime assembly。

Zircon当前把SDK example、ABI fixture和正式package放在同一workspace member列表、同一manifest扫描层、同一dist matrix和同一catalog schema中，缺少对应的结构隔离与构建条件。

### 4.3 Godot

Godot GDExtension将library path、entry symbol、`compatibility_minimum`、initialization level、initialize/deinitialize和reloadability作为真实加载合同；compatibility fixture拥有独立 `.gdextension` 和初始化/反初始化函数。被标为reloadable的扩展若注册不支持reload的class，Godot会诊断并撤销reloadability，而不是继续发布虚假的可重载状态。

Zircon fixture应同样验证load、initialize、behavior、deinitialize和reload的完整闭环；只验证JSON payload shape或entry report非空不足以证明插件合同。

### 4.4 Fyrox

Fyrox明确把 `PluginContainer::Static` 定义为最终构建的性能路径，把 `Dynamic` 定义为开发期快速迭代路径；`DyLibDynamicPlugin` 文档直接警告Rust dylib不适合production。动态reload会卸载旧实例、复制新library、重新register并恢复状态，Plugin trait又有register/init/on_loaded/on_deinit等完整阶段。

Zircon可以继续使用稳定C ABI作为正式NativeDynamic路径，但不能因此取消“开发fixture”和“可发布product artifact”的role区别；fixture必须在artifact和catalog admission阶段被硬隔离。

### 4.5 Bevy

Bevy的示例插件位于 `examples/app/plugin.rs`，只有应用显式 `add_plugins` 才参与构建；Plugin生命周期包含build、ready、finish和cleanup，默认拒绝重复plugin。它没有动态artifact catalog，因此只作为“示例必须显式组合、插件行为必须真实执行”的下限参考，不作为Zircon native ABI目标。

## 5. P0：必须先修复的产品与隔离阻断

### P0-01 · 物理目录扫描把fixture/sample自动提升为builtin产品插件

`zircon_editor/build.rs` 枚举 `zircon_plugins` 下每个一级目录，只要 `plugin.toml` 的任一module为editor就写入 generated catalog。三个carrier都满足条件；build script不读取package role、maturity、visibility、default enablement或shipping policy。生成descriptor又只有id/display/crate/category/capability，随后 `EditorPluginManager::builtin` 将全部条目推进Default phase。

因此测试载体会出现在核心catalog和产品Plugin Manager行中，并以空 `EditorPluginDescriptor` 被标记为已加载/激活。必须改为由canonical ProductPluginCatalog显式选择production package；TestFixture、Sample和DeveloperTool默认不得进入产品catalog、默认phase、shipping build或用户enablement面。

### P0-02 · SDK example没有任何一个交付形态能兑现声明能力

source crate包含实际extension registration，但普通App/Editor只通过 `zircon_first_party_editor_catalog` 链接selected source provider；该catalog只有Navigation与Neural，没有SDK example。generated builtin descriptor不持有source plugin object，所以它只注册空扩展。Native dist的registration manifest又是 `extensions: []`，无command、bridge或callback，并公开诊断“行为仍由source editor module承载”。

同时 `plugin.toml` 宣称 `assets`、`examples`、两个ZUI和一个settings文档，物理文件均不存在；四个operation没有产品handler。必须在一个milestone中选择并闭合真实交付合同：要么把完整source provider接入产品并让Native形态序列化/桥接等价贡献，要么把它移出product catalog成为显式导入sample。禁止继续让catalog capability、packaging strategy和真实行为分裂。

### P0-03 · 可破坏性native fixture可以通过正常dist/export路径成为shipping artifact

Native Fixture被当作普通workspace package和standalone dist matrix成员，manifest缺省为Standard，支持Client/Server/Editor并声明NativeDynamic。它暴露 `write:world` system、panic command、JSON importer、state callbacks和多个故障feature；现有schema没有TestFixture role、Shipping deny或artifact variant identity，export测试还直接以该package验证真实打包。

这不表示默认项目一定加载该DLL，但表示发布准入无法证明shipping artifact不含测试命令、故障variant或fixture package。必须在package role、resolved build graph和artifact receipt三层拒绝TestFixture进入Shipping，并对每个feature-built native artifact记录variant、source digest、ABI、capability与allowed target/configuration。release资格必须扫描最终artifact set，而不是依赖目录命名或调用者自律。

## 6. P1：工程化闭环差距

### 6.1 Package role、catalog与shipping policy

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| P1-01 | `PluginPackageKind` 只有Standard与FeatureExtension，三个carrier全部缺省为Standard | 增加Production、Sample、TestFixture、DeveloperTool等互斥role，或建立等价的强类型ArtifactRole；禁止用category/maturity代替role |
| P1-02 | `PluginMaturity::Experimental` 不进入Editor build-generated descriptor、status、enablement或shipping admission | maturity作为展示/稳定性维度，role作为加载/交付维度；两者都进入resolved package decision |
| P1-03 | catalog authority是一级目录 `plugin.toml` 枚举，新增目录会静默改变产品 | 使用显式、版本化、可review的ProductPluginCatalog；目录扫描只生成候选inventory，不直接发布产品 |
| P1-04 | generated editor descriptor丢弃version、description、maturity、platform、packaging、distribution、package kind和content roots | descriptor引用完整immutable package manifest或其source-bound projection，禁止再造窄metadata镜像 |
| P1-05 | builtin manager对所有generated descriptor执行Default phase，空descriptor也显示Active | activation必须依赖resolved selection、provider presence和behavior readiness；metadata-only row不能进入Active |
| P1-06 | schema没有hidden、enabled-by-default、explicitly-loaded、installed/test-only语义 | 建立PluginVisibility、DefaultEnablement、LoadPolicy与InstallOrigin，并在catalog/UI/loader/export共同消费 |
| P1-07 | module只有target mode/platform，没有build configuration、architecture、program/tool role allow/deny | 增加target/config/arch/product-role矩阵，fixture默认deny Shipping和普通产品角色 |
| P1-08 | `category = "sdk"` 同时承担领域分类和测试身份 | category只用于产品浏览；fixture/sample身份必须是不可混淆的类型字段 |
| P1-09 | workspace member、standalone validation、dist matrix和product package集合没有分层 | 建立product、sample、fixture三份resolved build set；CI可同时构建，但receipt必须证明不会互相进入artifact closure |
| P1-10 | 没有canonical inventory记录package为何存在、由谁消费、允许进入哪些artifact | 建立PluginPackageInventory与ArtifactEligibilityDecision，包含owner、role、consumer、target、config、provider和exclusion reason |

### 6.2 Plugin SDK Examples 的源码、资源、operation与carrier parity

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| P1-11 | manifest声明 `asset_roots=["assets"]`、`content_roots=["examples"]`，两个目录都不存在 | validate/package/build阶段要求每个root存在，或显式声明generated root及generator receipt |
| P1-12 | 两个ZUI和 `model_import_settings.toml` 均不存在 | 提供可解析、可实例化、可截图/交互的真实资源，并把URI resolution加入sample资格 |
| P1-13 | source importer输出Model，checked-in `plugin.toml` 写 `output_kind="Data"` | importer contract从一个typed declaration生成；source/file/native projection必须逐字段等价 |
| P1-14 | source注册完整扩展，native registration manifest却为零extension | Native carrier必须序列化全部host-safe贡献，并为非host-safe行为提供versioned bridge；否则移除NativeDynamic声明 |
| P1-15 | first-party editor source provider catalog不链接SDK example | 若它是产品插件则显式链接并由selection装配；若它是sample则不得出现在builtin product catalog |
| P1-16 | dist crate依赖source editor crate，而source editor crate依赖完整 `zircon_editor`/runtime contracts | native dist只依赖ABI-safe SDK/projection crate；共享常量与serialized manifest提取为无Editor依赖的declaration crate |
| P1-17 | 四个 `sdk.examples.*` operation在包外没有factory、event handler或command executor | 每个可见command必须绑定可执行operation，并验证success/failure/undo/cancel语义 |
| P1-18 |所谓Model importer只注册Editor descriptor，没有真实read/parse/import/publish实现 | sample必须执行至少一个GLTF/GLB happy path和一个failure path，产出typed Model artifact与diagnostics |
| P1-19 | source测试把window/asset拆成 `sdk_example_window`、`sdk_example_asset` 两个synthetic package | 测试真实 `plugin_sdk_examples` package selection、capability、lifecycle和owner撤销，避免绕开package级冲突 |
| P1-20 | tests只断言registry metadata，不检查resource存在、operation执行或document roundtrip | 增加resource load、template instantiate、command dispatch、import、toolkit open、save/reopen的产品测试 |
| P1-21 | README宣称sample“无需runtime linkage”并贡献完整authoring，当前product provider和dist均不成立 | 文档从verified capability/parity receipt生成或由gate校验，禁止叙述超前于可执行产品事实 |
| P1-22 | 没有SourceTemplate、LibraryEmbed、NativeDynamic三形态差分测试 | 对同一golden workflow比较capability、extension IDs、commands、assets、diagnostics、state与unload结果 |

### 6.3 Native Dynamic Fixture 的协议、预算与行为真实性

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| P1-23 | registration宣称 `write:world` Update system，`tick` bridge只返回OK且没有可观察副作用 | fixture写入host-owned test resource/counter并验证schedule、affinity、access和generation；否则不得宣称write access |
| P1-24 | manifest声明 `native_dynamic_fixture.echoed` event，代码没有任何emit路径 | 通过host event API真实发送并验证schema、ordering、backpressure与unload后拒绝 |
| P1-25 | editor capability和entry存在，但registration为零extension、零command | 要么提供最小真实editor contribution和lifecycle测试，要么删除editor target/capability，避免空成功 |
| P1-26 | invalid ABI/export/capability feature共用同一package id和dist crate，artifact缺variant identity | ArtifactReceipt记录feature set与expected outcome；故障artifact只能进入negative-test store，不能覆盖正常dist路径 |
| P1-27 | importer将攻击者提供的u64转usize，再执行未checked的 `metadata_len_end + metadata_len` | 使用checked conversion/checked_add并在切片前返回稳定MalformedLength错误；覆盖32/64位边界 |
| P1-28 | import payload没有metadata/source的显式输入预算 | host在调用前强制最大payload，fixture在解析时分配独立metadata/source budget并产生budget diagnostics |
| P1-29 | response先构造完整serde tree、复制source text和canonical JSON，再由1MiB output sink拒绝 | 预估/流式编码host-owned output，避免在sink admission前按输入规模放大内存 |
| P1-30 | 极端长度会通过panic capture变成通用panic诊断，而不是协议级结构错误 | fuzz corpus要求所有malformed request均返回typed protocol error，panic计数必须为0 |
| P1-31 | fixture importer priority为100并支持普通json，若误入产品会优先于低priority data importer | fixture importer使用fixture-only extension或测试source scheme；shipping/catalog gate双重拒绝 |
| P1-32 | package同时支持Client、Server、Editor，却没有按target验证runtime/editor behavior差异 | 建立三target矩阵：Client/Server system+importer，Editor runtime+editor entry，unsupported capability必须fail closed |

### 6.4 Editor Contribution Fixture 的可执行性与lifecycle

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| P1-33 | contribution batch发布command，V4 command manifest却为空 | serialized command必须对应dense command slot、payload/output schema和callback；cross-manifest validator拒绝悬空command |
| P1-34 | Editor materializer把command变成普通operation path，但没有native invoke adapter | 建立owner/generation-qualified NativeEditorOperationAdapter，将dispatch路由到loaded behavior并处理unload race |
| P1-35 | menu指向必然被DENIED的command，UI会发布死入口 | menu/toolbar在operation binding readiness之前不得可见；失败时显示结构化不可用原因 |
| P1-36 | view和drawer只有descriptor，没有view factory、pane data source或render contract | serialized schema表达host-safe factory/data contract，或只测试能真实materialize的贡献kind |
| P1-37 | settings page只有label/category，没有setting definition、scope、read/write和restart policy | fixture注册至少一个typed setting并验证project/user scope、persistence和卸载撤销 |
| P1-38 | asset type没有serializer、importer、document editor、thumbnail或creation路径 | 要么补最小完整asset workflow，要么从fixture声明中移除会造成产品假象的asset type |
| P1-39 | Python gate只正则提取Rust字符串，不构建/加载cdylib；crate自身没有行为测试 | CI构建真实DLL，走product loader/materializer/manager/operation dispatcher验证全部贡献kind |
| P1-40 | 没有disable、unload、hot reload、collision、capability denial和generation replacement验证 | 建立端到端lifecycle suite，证明旧operation/view/settings/asset row全部撤销且stale callback不可调用 |

## 7. P2：质量、可维护性与诊断债务

| ID | 当前差距 | 改进要求 |
|---|---|---|
| P2-01 | `sdk.example.*`、`plugin_sdk_examples.*` 和 `editor.contribution_fixture.*` 使用三套namespace | 建立package-owned ID规则与显式external operation namespace声明 |
| P2-02 | fixture/sample位于正式插件根顶层，目录结构不表达role | 迁入 `tests/fixtures`、`examples` 或独立workspace，并由catalog inventory显式引用 |
| P2-03 | package没有面向开发者的README、expected workflow、failure variant说明 | 为sample和fixture提供source-bound开发文档，但不得替代机器准入字段 |
| P2-04 | native callback diagnostics均是自由文本 | 使用stable diagnostic code、stage、package、variant和generation字段 |
| P2-05 | event manifest仍是 `event=...;payload=...` 的ad-hoc字符串 | 使用与registration/command一致的版本化typed serialization |
| P2-06 | importer把本机 `source_path` 拼入migration summary和diagnostics | 按privacy policy输出logical URI或redacted display path |
| P2-07 | 1MiB、4 bytes、schema version和state blob等fixture policy散落在单文件 | 收敛为具名fixture contract constants，并由host/fixture test共同引用 |
| P2-08 | 多个test依赖 `include_str!`、`.contains()` 或source regex | 结构检查保留少量，核心资格迁移到typed parse、real build/load和behavior assertions |
| P2-09 | standalone dist matrix只在Ubuntu构建这些carrier | fixture qualification增加Windows/Linux/macOS与至少x86_64/arm64 ABI布局矩阵，按平台能力允许明确skip |
| P2-10 | 没有生成source/native/package差分报告供Plugin Manager或CI展示 | 产出CarrierParityReceipt，列出declared/materialized/executable/packaged差异并绑定source digest |

## 8. 目标架构

### 8.1 Package、role与artifact eligibility

建立单一 `PluginPackageDefinition`，至少包含：

- `PackageRole = Production | Sample | TestFixture | DeveloperTool`；
- `Visibility = Public | Hidden | Internal`；
- `DefaultEnablement = Enabled | Disabled | Unspecified`；
- `LoadPolicy = Automatic(phase) | Explicit`；
- `TargetPolicy`：product role、target mode、platform、architecture、configuration allow/deny；
- `ShippingPolicy`：是否允许进入Shipping、是否允许依赖、是否允许用户安装；
- 完整version、SDK API、ABI、engine compatibility、capability、resource和distribution metadata。

`maturity` 继续表达Core/Stable/Beta/Experimental/Stub等完成度，不能再承担安全或artifact role。`category` 只表达浏览分类。三个维度必须在类型和UI上分开。

### 8.2 Catalog authority

构建期目录扫描只生成 `PluginPackageInventory`，不得直接生成builtin catalog。产品catalog由显式 `ProductPluginCatalog` 选择Production/允许的DeveloperTool，并输出每个候选的included/excluded reason。Editor core catalog、Plugin Manager status、App source provider catalog、native discovery和export必须消费同一resolved catalog generation。

Sample通过显式“导入样例”进入临时project或sample project，不成为builtin plugin。TestFixture只由test harness以test capability加载，最终shipping artifact scan必须证明其package id、crate name、entry symbol和diagnostic marker均不存在。

### 8.3 Carrier parity

为每个declared packaging form生成 `CarrierCapabilityManifest`：

- package、module、capability、extension、command、event、system、resource、state schema；
- source provider identity与native entry identity；
- executable binding，而不是仅有descriptor；
- unload/reload/state migration合同；
- artifact digest、feature set、target/config和qualification result。

SourceTemplate、LibraryEmbed和NativeDynamic可有不同实现机制，但同一对外能力必须在golden workflow中等价。无法投影的能力必须从该form的manifest删除，并在selection前明确报告unsupported，不能加载后才以空registration成功。

### 8.4 Fixture harness

Native Fixture拆成positive、negative artifact variants。positive variant真实修改host-owned test state、发event、执行import、保存/恢复、卸载；negative variants各自拥有不可发布的variant ID和expected failure stage。Editor Fixture必须经过真实DLL加载、contribution materialization、command dispatch、view/data/settings/asset最小行为、disable/unload/reload和stale generation拒绝。

fixture不拥有第二套ABI或loader；它只使用production ABI/loader/manager，以test capability注入可观察的host state和fault hooks。

## 9. 分阶段重构路线

### M0 · 先切断产品污染

1. 为三个package声明role；Native/Editor Fixture设TestFixture，SDK example设Sample。
2. Product catalog改为显式allowlist，停止由目录自动发布。
3. Shipping build/export加入fixture/sample hard deny和最终artifact scan。
4. Plugin Manager只显示resolved product catalog；开发模式可在独立filter下查看Sample/Fixture。

### M1 · 收敛package schema和catalog generation

1. 增加visibility/default enablement/load/shipping/target configuration字段。
2. generated descriptor保留完整manifest projection和source digest。
3. 合并Editor core catalog、status catalog、first-party source provider与native candidate的generation identity。
4. 对每个excluded candidate输出stable decision code和owner。

### M2 · 修复SDK example或将其彻底sample化

1. 补齐ZUI/settings文档与真实Model import workflow。
2. 为四个operation实现handler、failure、undo/cancel边界。
3. 选择source provider产品装配或显式sample project装配。
4. 使NativeDynamic贡献等价，或从sample manifest移除不支持的form。

### M3 · 升级native fixture

1. 拆分positive/negative artifact variant identity。
2. 使用checked framing、input/output budget和fuzz corpus。
3. 让system、event、import、state和unload产生可观察host结果。
4. 建立Client/Server/Editor与跨平台ABI矩阵。

### M4 · 升级editor contribution fixture

1. command contribution与V4 command table/slot一致。
2. 接入owner/generation-qualified native operation adapter。
3. 为view/drawer/settings/asset type提供最小真实host-safe behavior。
4. 验证disable/unload/hot reload/collision/capability denial。

### M5 · 资格与长期治理

1. 生成PluginPackageInventory、ArtifactEligibilityDecision和CarrierParityReceipt。
2. CI对每个carrier执行real build/load/behavior/product exclusion。
3. release artifact scan绑定source、resolved graph、target/config和final package set。
4. 文档与示例从通过资格的manifest/workflow生成或校验。

## 10. 32项验收门

### Role、catalog与shipping

1. G01：每个plugin package都有互斥PackageRole，缺失role时production catalog fail closed。
2. G02：TestFixture和Sample默认不进入builtin Editor catalog。
3. G03：TestFixture和Sample默认不进入Plugin Manager普通产品列表。
4. G04：TestFixture在Shipping resolved graph中出现时构建失败。
5. G05：最终artifact scan证明fixture package id、crate、entry symbol和marker均不存在。
6. G06：catalog不再由一级目录扫描直接发布，新增manifest不会改变产品集合。
7. G07：maturity、role、visibility、default enablement和load policy在status/UI中保持区分。
8. G08：target mode/platform/architecture/configuration/product role决策有source-bound receipt。

### SDK example

9. G09：所有asset/content root和URI在package validate、build和runtime resolution中存在。
10. G10：四个visible command都有可执行handler和稳定failure code。
11. G11：GLTF/GLB sample import产出typed Model，并完成save/reopen。
12. G12：source/file/native importer output type和capability逐字段一致。
13. G13：source provider由显式sample/product composition装配，不依赖空descriptor。
14. G14：dist crate不依赖完整Editor实现crate，只依赖ABI-safe projection。
15. G15：SourceTemplate、LibraryEmbed、NativeDynamic运行同一golden workflow并通过parity diff。
16. G16：不支持某form时selection前明确拒绝，不能加载后空成功。

### Native fixture

17. G17：positive system真实产生host-owned state change，并验证stage/access/affinity。
18. G18：event真实发出并验证schema、ordering、backpressure和unload后拒绝。
19. G19：import framing对整数边界使用checked conversion/checked_add，fuzz下零panic。
20. G20：metadata、source、response和temporary allocation都有硬预算及diagnostic。
21. G21：negative ABI/export/capability variant有独立ArtifactVariant ID和expected failure stage。
22. G22：negative artifact无法被product/export catalog选择。
23. G23：Client、Server、Editor三target行为矩阵均有真实DLL测试。
24. G24：Windows、Linux、macOS和适用architecture的ABI/loader测试有明确结果或policy skip。

### Editor contribution fixture

25. G25：contributed command在V4 table中有dense slot并能经Editor UI dispatch成功调用native callback。
26. G26：menu只有在command binding ready时发布，failure时不可点击且有结构化原因。
27. G27：view/drawer有真实factory/data source或从支持列表移除。
28. G28：settings page完成typed read/write、scope和persistence测试。
29. G29：asset type完成最小serializer/importer/document/thumbnail链或从fixture移除。
30. G30：disable/unload会原子撤销全部contribution，stale operation无法调用旧DLL。
31. G31：hot reload在新generation重建贡献，collision/capability denial不会留下部分状态。
32. G32：CI必须build/load真实fixture DLL；source regex与manifest contains只能作辅助gate。

## 11. 风险、依赖与实施边界

- M0必须先于新增Marketplace、第三方插件或更多SDK sample；否则每新增一个目录都会扩大产品catalog污染。
- Shipping exclusion必须在resolved graph和最终artifact两端执行，不能只在Editor UI隐藏。
- Native operation adapter涉及DLL generation与unload race，必须消费Plugins01/Runtime07/Runtime Interface01定义的lifetime和call guard，不允许裸callback逃逸。
- Source/native parity不得以“两个manifest文本相同”验收；必须比较materialized和executable behavior。
- Fixture可以包含故障注入和panic测试，但必须在test capability、child process和negative artifact store中运行，不能混入普通dist identity。
- 当前四个consumer/test文件处于共享修改状态；实施前必须重取source freeze并重跑所有32项gate。

## 12. 本轮未做事项

- 未修改Rust、TOML、CI、Python tooling或参考引擎代码。
- 未运行Cargo、Editor、DLL、export、fuzz、sanitizer、跨平台或性能测试。
- 未修复Plugins01/06、Editor06已经拥有的通用ABI、catalog、trust或Plugin Manager问题。
- 未把tooling迁移到Rust；只把现有build/export路径作为carrier能否进入产品artifact的消费证据。
- 未宣称整个 `zircon_plugins` review完成；Marketplace、安装/update、第三方隔离和剩余artifact孤岛仍需后续专项。
