---
related_code:
  - Cargo.toml
  - Cargo.lock
  - deny.toml
  - zircon_app/Cargo.toml
  - zircon_editor/Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_runtime/crates/zr_rhi/Cargo.toml
  - zircon_runtime/crates/zr_rhi_wgpu/Cargo.toml
  - zircon_runtime/reflection_macros/Cargo.toml
  - zircon_reflect_derive/Cargo.toml
  - zircon_runtime_interface/Cargo.toml
  - zircon_runtime_host/Cargo.toml
  - zircon_hub/Cargo.toml
  - zircon_plugins/Cargo.toml
  - zircon_plugins/Cargo.lock
  - zircon_plugins/plugin_sdk/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/navigation/native/Cargo.toml
  - zircon_plugins/physics/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - examples/woc/native/Cargo.toml
  - examples/woc/native/Cargo.lock
  - examples/woc/native/apps/woc_client/Cargo.toml
  - tools/session_tray/Cargo.toml
  - tools/session_tray/Cargo.lock
  - .github/workflows/ci.yml
  - .github/workflows/profile-feature-contract.yml
  - .github/workflows/mvp-editor-windows.yml
  - tools/runtime-profile-feature-presets.py
tests:
  - tools/tests/test_frameworks_03_profile_feature_presets.py
  - tools/tests/test_frameworks_03_server_feature_boundary.py
  - tools/tests/test_frameworks_06_dependency_governance_contract.py
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_runtime/21-zr-language-parser-type-system-semir-bytecode-package-loader-vm-runtime-review.md
  - docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_tooling/15-mvp-build-staging-product-process-acceptance-evidence-resource-baseline-control-plane-review.md
  - docs/plans/optimize/zircon_tooling/17-repository-content-source-set-ignore-generated-vendor-license-distribution-review.md
  - docs/plans/optimize/zircon_tooling/18-executable-target-entrypoint-cli-process-receipt-product-qualification-review.md
reference_engines:
  - dev/bevy/Cargo.toml
  - dev/bevy/crates/bevy_internal/Cargo.toml
  - dev/Fyrox/Cargo.toml
  - dev/godot/SConstruct
  - dev/godot/methods.py
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Configuration/Rules/ModuleRules.cs
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Configuration/Rules/TargetRules.cs
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Configuration/Rules/RulesAssembly.cs
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/System/TargetReceipt.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Unity.RenderPipelines.Core.Runtime.asmdef
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: false
---

# 20 · Cargo Package、Workspace、Feature、Dependency、Target Graph 与 Build Receipt 工程化差距

## 1. 结论

ZirconEngine的Rust包图已经远超“小项目Cargo workspace”。当前Git tree有162份`Cargo.toml`、159个package和338个Cargo target object：18个binary、157个library target（含42个`cdylib`与2个proc macro）、156个integration test target及7个custom build target。四份tracked lockfile合计21,832行、524,791 bytes。根workspace在locked metadata中解析37个package；插件workspace静态包含139个package；WOC native workspace解析8个；Session Tray是独立1-package workspace。扣除根图与插件图重叠的26个package后，唯一package正好159个。

这些manifest不是普遍失控的文本堆。162份文件均能由标准TOML parser读取；四份lock均为format 4；638个直接dependency occurrence中342个使用workspace inheritance，当前没有Git dependency；67个optional dependency全部能从feature graph到达；83个直接registry dependency name中只有`proc-macro2`、`syn`和`tokio`存在多种手写version requirement。根、插件与WOC workspace都显式使用resolver 2，核心`zircon_runtime`和`zircon_plugin_sdk`在workspace dependency层默认关闭default feature。现有runtime profile TOML、feature projection测试、dependency governance、plugin structure audit和locked metadata也提供了可继续收敛的基础。

但当前Cargo图还不能表达唯一产品身份。最直接的确定性反例是Editor：`zircon_editor`依赖`zircon_runtime = { path = "../zircon_runtime" }`，没有`default-features = false`；而runtime的default feature是`target-client`。实际执行`cargo tree --locked -p zircon_app --no-default-features --features target-editor-host -e features -i zircon_runtime`后，resolved graph同时包含`zircon_runtime/target-editor-host`与`zircon_runtime/default -> target-client`，后者明确经`zircon_editor`进入。命令、MVP manifest和CI都声称这是纯`target-editor-host`构建，真实编译图却是Client与Editor角色的feature union。

现有守卫没有发现它。profile test只比较TOML中的手写`runtime_features/app_features`与两份manifest的直接feature数组；本地`reachable_feature_graph()`虽会追踪default feature，却只禁止desktop profile带入server/headless，没有禁止Editor带入client role。CI的profile lane只执行`cargo check`，没有保存或比较Cargo最终resolved feature/package graph。因此错误产品闭包既能通过字符串合同，也能成功编译，并继续进入BuildSet与MVP证据。

包图的其余维度也没有形成工程合同。159个package没有任何`package.metadata`来声明PackageId、role、owner、visibility、ABI/publication或product capability；53个package共声明208个feature，却把product role、平台、backend、测试、fault fixture、profiling和普通可加能力放进同一种Cargo feature。四个workspace分别拥有32/5/9/0个workspace dependency authority；四份lock之间的共同依赖版本集合大量不同，根与plugin有105个共同name的version set不一致，根与Session Tray有77个，根与WOC有13个。CI dependency governance只覆盖根和plugin两个manifest，WOC与Tray不在矩阵中。

Tooling01已经拥有plugin lock失配、双workspace归属、toolchain/MSRV、workspace lint、profile和CI总控；Tooling05拥有7个build script内部生成与target判断；Tooling10拥有test discovery/result；Tooling17拥有publish/license/SBOM；Tooling18拥有binary/CLI/process receipt；Plugins06拥有首方plugin provider/profile/capability closure。本篇不重复这些finding。本篇只拥有**完整Cargo Package Graph、feature语义与resolved closure、workspace/lock graph投影、package/target身份及ResolvedPackageGraphReceipt**。本轮新增 **1项P0、48项P1和12项P2**。

## 2. 审查边界与证据

### 2.1 物理范围

| 范围 | 数量 | 本轮深度 |
|---|---:|---|
| Cargo manifest | 162文件 / 2,914行 / 88,076 bytes | 标准TOML全量结构化解析，package/workspace/target/feature/dependency逐项聚合 |
| Cargo package | 159 unique | package identity、metadata、publication、default、feature和target inventory |
| Cargo target object | 338 | 18 bin、113普通lib、42 cdylib、2 proc macro、1额外rlib kind、156 test、7 custom-build |
| Lockfile | 4文件 / 21,832行 / 524,791 bytes | package/version/source集合、multi-version与跨lock差异 |
| Dependency declaration | 638 | 607 normal、24 dev、6 build、1 target-specific；342 workspace、150 path、146 registry |
| Feature declaration | 53 package / 208 feature | default、optional dependency、local/cross-package edge、product role与profile闭包 |
| Workspace | root/plugin/WOC/Session Tray | member、default member、resolver、target dir、lock与CI consumer |
| Product graph | Client/Server/Editor + plugin dist/WOC/Tray | `cargo metadata/tree`只读解析与静态consumer追踪 |

159个package中158个从所在workspace继承`version/edition/license`，Session Tray显式声明相同值；有效值全部是`0.1.0`、edition 2021、`MIT OR Apache-2.0`。全部159个package缺`rust-version`、`repository`和`readme`；34个缺description；只有`zr_rhi`与`zr_rhi_wgpu`显式`publish = false`；没有package声明`package.metadata`或`links`；没有package继承workspace lint。上述公共metadata/publish/lint事实已经由Tooling01/17拥有，本篇只把它们作为PackageGraph缺少分类输入的证据，不重复登记原finding。

### 2.2 Workspace与Lock快照

| 解析根 | 当前package/member | default member | lock/target目录 | 状态 |
|---|---:|---:|---|---|
| root `Cargo.toml` | 37 / 37 | 1：`zircon_runtime` | root lock / root `target` | locked metadata成功；11个显式member经path graph扩成37 |
| `zircon_plugins/Cargo.toml` | 139 static members | 全部 | plugin lock / plugin `target` | known locked metadata在编译前因lock漂移失败；本轮不重复同一阻断 |
| WOC native | 8 / 8 | 全部 | WOC lock / WOC `target` | locked metadata成功；当前production compile另有既知6错误 |
| Session Tray | 1 / 1 | 自身 | Tray lock / Tray `target` | locked metadata成功；1 bin + 1 lib + 1 build script |

根resolved graph中的26个plugin package同时属于139-member plugin workspace。它们包括两个first-party catalog、Plugin SDK、Navigation/Neural/Rendering/Sound/Physics/ZrVM等runtime/editor/native依赖。相同package source因此可按调用根进入不同lock、profile、target directory与feature union。本篇要求把这个事实投影进ResolvedPackageGraphReceipt；是否最终合并workspace或显式exclude仍由Tooling01的硬切决定。

四份lock的静态差异如下：

| Lock | package / unique name / multi-version name | 与根lock共同name但version set不同 |
|---|---:|---:|
| root | 868 / 772 / 65 | - |
| plugin | 793 / 722 / 49 | 105 |
| WOC | 40 / 40 / 0 | 13 |
| Session Tray | 421 / 378 / 33 | 77 |

版本不同本身不自动构成bug：独立产品可以有独立解算。但当前没有machine-readable policy说明哪些图必须同代、哪些允许独立升级、哪些dependency是跨DLL/serialization/GPU/native singleton。四个lock hash也没有共同进入一份产品BuildSet。

### 2.3 Feature Graph确定性复现

Editor manifest自身`default = []`并不能阻止dependency default。Cargo feature按package全图做并集；`zircon_editor -> zircon_runtime`没有关闭default，所以App的Editor入口最终得到：

```text
zircon_app/target-editor-host
  -> zircon_runtime/target-editor-host
  -> zircon_editor
       -> zircon_runtime/default
            -> zircon_runtime/target-client
```

该路径由当前locked graph实际输出，不是根据文件名猜测。对照的Client命令只解析出`target-client`，没有Editor依赖。当前代码没有`compile_error!`、graph admission或receipt拒绝多个product role同时激活。

仓库已有自写`reachable_feature_graph()`，会解析workspace inheritance、target dependency和default feature；它是正向基础。但相关断言只验证Client不带Editor、desktop不带Server，以及RHI依赖不泄漏；没有定义“每个产品恰好一个role feature”的不变量。profile preset test又只比较manifest直接数组，不能代表Cargo最终解算结果。

### 2.4 动态验证边界

本轮只运行不编译源码的locked `cargo metadata`与`cargo tree`。root、WOC和Tray metadata成功；Editor/Client resolved feature tree成功。没有重跑已知必然失败的plugin locked metadata、WOC native compile、Hub compile或完整Editor测试，也没有在当前其他Session修改的大量production源码上生成新的“全仓通过”声明。

本篇列出的manifest、lock、workflow和feature-test source scoped set当前均无Git worktree改动，因此`source_recheck_required: false`；这只表示本篇静态输入clean，不代表其finding已实现或其他dirty production范围已验收。

## 3. 已有工程基础

1. 所有manifest可由标准TOML parser处理，没有依赖正则或字符串拼接才能建立基础inventory。
2. 四个workspace都提交lockfile；根、plugin和WOC显式resolver 2。
3. 638个direct dependency中没有Git source，`deny.toml`拒绝unknown registry与unknown git。
4. 342个dependency occurrence已使用workspace inheritance，具备继续集中版本和default-feature政策的迁移点。
5. 67个optional dependency全部可从feature edge到达，没有发现完全悬空的optional dependency。
6. 核心runtime、RHI、Plugin SDK与catalog大量使用`default-features = false`，说明最小闭包意识已经存在。
7. runtime profile有schema-versioned TOML、strict parser、Rust生成投影和CI matrix生成器。
8. 自写feature resolver已有workspace merge、target table、default feature和局部path dependency traversal测试。
9. 18个binary target已有Tooling18清单，7个build script已有Tooling05逐文件审查，不需要重新发明入口inventory。
10. Plugin06已有139-package/162-target结构审计和provider catalog矩阵，可作为Cargo graph与产品capability graph的对账输入。

## 4. 已有Canonical Finding，不在本篇重复计数

| 既有问题 | Canonical owner | 本篇只新增的连接 |
|---|---|---|
| plugin lock与manifest失配 | Tooling01 `TOOL-CI-P0-001` | ResolvedPackageGraphReceipt必须先验证lock freshness |
| root/plugin双workspace归属 | Tooling01 `TOOL-WS-P1-001/002` | 收敛前必须显式记录同source的workspace/lock context |
| rust toolchain、lint、profile、package publish metadata | Tooling01 P1-003..008 | PackageGraph消费policy，不再复制字段结论 |
| build.rs host/target、输入、生成和原子性 | Tooling05 | custom-build target只登记node与receipt引用 |
| test discovery、Hub `test=false`、result | Tooling10 | test target只登记图，不重复测试可达性P0 |
| publication/license/SBOM | Tooling17 | PackageRole决定哪个publication gate适用 |
| executable target/CLI/process receipt | Tooling18 | binary target消费ResolvedPackageGraphReceipt |
| plugin provider/profile/capability closure | Plugins06 | Cargo feature/package图与PluginSelectionReceipt做双向对账 |
| external ZrVM path source | Runtime21 | graph只引用ZrToolchainSourceReceipt，不重复其P0 |

## 5. 参考实现约束

### 5.1 Unreal Build Tool

Unreal `TargetRules`把Game、Editor、Client、Server和Program建成`TargetType`，并将link type、build environment、architecture、compile-against-engine/editor、client/server code等作为target rule，而不是依赖全图可相加的一个布尔feature。`ModuleRules`分开Public/Private/Include-only/DynamicallyLoaded dependency，并登记RuntimeDependencies与ExternalDependencies。`RulesAssembly`从module/target name解析唯一rules source，`TargetReceipt`最终记录target type、platform、configuration、architecture、build environment、build products和runtime dependencies。

Zircon不需要复制UBT或放弃Cargo；需要在Cargo之上补同等的product-role exclusivity和resolved receipt。Cargo feature适合可加能力，不足以单独表示互斥产品身份。

### 5.2 Bevy与Fyrox

Bevy根manifest固定repository/documentation/rust-version、resolver 3、workspace exclude与workspace lint。`bevy_internal`对内部path dependency同时写version与`default-features = false`，用显式`dep:`和弱dependency feature组织能力，并使用target-specific dependency隔离Android与非WASM路径。重点不是Bevy feature数量少，而是内部package可以发布、依赖版本与feature传播可被Cargo完整描述。

Fyrox把Editor、Project Manager和CI建成不同Cargo profile，明确opt-level、strip、panic与LTO。它仍没有解决所有大型引擎package治理问题，但证明产品用途不应只靠调用者记住一串feature。

### 5.3 Godot

Godot SCons把platform、compiler minimum、production/dev、build profile和module dependency作为构建环境中的typed option。每个module先执行`can_build(platform)`，required dependency缺失时明确disable并记录，最终module list按dependency排序；Editor还追加自己的required module并fail-close。它提供的对照是“解析后的模块图必须能被检查和输出”，不是要求Zircon改用SCons。

### 5.4 Unity Graphics

Unity SRP Core package固定package name/version与dependency version；assembly definition分开references、platform include/exclude、unsafe、auto-reference、define constraints和version defines。它只代表Graphics package图，不代表Unity完整引擎；本篇仅借鉴package identity、dependency与platform/conditional compilation分层。

## 6. P0：产品角色闭包失真

### CARGO-GRAPH-P0-001 · `target-editor-host`实际同时激活`zircon_runtime/target-client`，命令与BuildSet产品身份不真实

`zircon_editor/Cargo.toml`对runtime依赖未关闭default，而runtime default是`target-client`。即使调用者使用`--no-default-features --features target-editor-host`，该开关只关闭root package的default，不关闭dependency default。Cargo tree已证明Editor path把client role重新带回。当前MVP、profile workflow、build summary和产品输入manifest仍只记录请求字符串`target-editor-host`，不记录resolved closure；它们能够为混合角色artifact生成错误标签。

第一步应把`zircon_editor -> zircon_runtime`改为`default-features = false`，但这不是完整修复。建立`ProductRoleId`与互斥规则：Client、Server、Editor、Tool、Test每个target必须恰好解析一个允许的role，普通capability feature不得隐式加入role；graph validator消费Cargo metadata/resolve结果并在编译前拒绝0个或多个role。`ResolvedPackageGraphReceipt`保存请求feature与最终package-feature closure，Tooling18/15、MVP、export和release只消费receipt内的ProductRole，不再相信命令字符串。

## 7. P1：Workspace、Lock 与 Graph Identity

| ID | 当前差距 | 需要重构 |
|---|---|---|
| CARGO-GRAPH-P1-001 | 162 manifest/159 package/338 target没有唯一机器清单 | 生成`CargoPackageCatalog`，为每个package/target记录stable ID、manifest digest、role、owner和workspace policy |
| CARGO-GRAPH-P1-002 | 四个workspace没有明确monorepo/product/tool/fixture分类 | `WorkspaceDescriptor`声明用途、source membership、default command、lock owner、target root和release关系 |
| CARGO-GRAPH-P1-003 | Tooling01的两lock BuildSet模型尚未纳入WOC与Tray | BuildSet按产品声明required workspace/lock集合，遗漏或额外lock均失败 |
| CARGO-GRAPH-P1-004 | lock之间版本差异没有intent/policy | 为共享ABI/schema/singleton dependency定义must-converge，其余差异记录approved rationale与expiry |
| CARGO-GRAPH-P1-005 | root `default-members`只有runtime | 无限定`cargo build/test`不得作为仓库成功；开发入口必须显示实际selected package set |
| CARGO-GRAPH-P1-006 | root 11个显式member解析成37个member | gate比较declared、resolved、implicit path member与approved nested workspace关系 |
| CARGO-GRAPH-P1-007 | 26个plugin package在root/plugin两种context编译 | 为每个共享package比较source digest、features、dependency versions、target/profile与artifact用途 |
| CARGO-GRAPH-P1-008 | invocation root决定lock、profile和target directory | command admission解析canonical WorkspaceId，不允许按当前目录隐式选择产品图 |
| CARGO-GRAPH-P1-009 | package graph没有内容身份 | 计算规范化manifest、lock、toolchain、target与resolved graph digest，进入BuildSetId |
| CARGO-GRAPH-P1-010 | nested workspace边界没有owner/exclude证明 | SourceSet gate验证每个Cargo package恰好属于批准的一个canonical workspace或显式shared projection |

## 8. P1：Feature、Profile 与 Product Composition

| ID | 当前差距 | 需要重构 |
|---|---|---|
| CARGO-GRAPH-P1-011 | product role使用可相加Cargo feature表达 | ProductRole在target descriptor层互斥；Cargo feature只作为生成后的低层projection |
| CARGO-GRAPH-P1-012 | profile测试只比较直接feature数组 | 从Cargo resolve结果比较完整transitive feature/package closure与expected ProductProfile |
| CARGO-GRAPH-P1-013 | 自写resolver未断言Editor不能含Client | 为Client/Server/Editor/Tool建立exact-one role与forbidden closure tests |
| CARGO-GRAPH-P1-014 | 208个feature没有语义分类 | schema区分role、capability、backend、platform、instrumentation、test和fault-injection feature |
| CARGO-GRAPH-P1-015 | `default`分别表示Client、runtime、dist或空集合 | 每个package声明default policy；产品package默认必须映射canonical profile或显式为空 |
| CARGO-GRAPH-P1-016 | `--all-features`混合互斥role/platform/backend/fault fixture | all-features只用于声明完整性；资格矩阵由合法组合schema生成 |
| CARGO-GRAPH-P1-017 | 159包仅一个target-specific dependency table | platform dependency由target predicate拥有，不能只靠feature名和下游crate内部cfg猜测 |
| CARGO-GRAPH-P1-018 | Android两种activity feature可在all-features同开 | 为互斥platform adapter定义cardinality与compile/graph rejection |
| CARGO-GRAPH-P1-019 | backend feature没有统一exactly-one/optional policy | 每个backend family声明zero/one/many规则、fallback和shipping要求 |
| CARGO-GRAPH-P1-020 | test-support、integration与fault ABI feature混入产品面 | test/fault feature只能由TestProfile激活，shipping graph检测到即失败 |
| CARGO-GRAPH-P1-021 | Cargo feature与Plugin capability/catalog无双向映射 | 生成`FeatureCapabilityProjection`并验证compiled provider、selection、artifact和capability一致 |
| CARGO-GRAPH-P1-022 | optional dependency激活只有Cargo edge，没有产品owner | optional edge附PackageId、reason、product roles、cost与qualification gate |
| CARGO-GRAPH-P1-023 | 跨package feature使用裸字符串 | 规范化为`PackageId/FeatureId`，alias/deprecation有版本和retirement gate |
| CARGO-GRAPH-P1-024 | artifact只记请求feature，不记resolved closure | receipt保存排序后的package-version-source-feature集合及其digest |
| CARGO-GRAPH-P1-025 | 没有每种公开role的minimal closure资格 | Client/Server/Editor/Tool分别从空default解算、编译、启动并验证absence/presence |
| CARGO-GRAPH-P1-026 | negative/pairwise/conflict矩阵不完整 | 从feature constraint schema生成required positive、forbidden和pairwise lanes |
| CARGO-GRAPH-P1-027 | 没有unused/unreachable feature审计 | 对manifest edge、`cfg(feature)`、consumer、profile与artifact做双向reachability |
| CARGO-GRAPH-P1-028 | feature没有compile-time/binary/package成本预算 | 按feature记录新增package、compile wall、artifact bytes、startup和runtime capability成本 |

## 9. P1：Dependency、Package 与 Target Graph

| ID | 当前差距 | 需要重构 |
|---|---|---|
| CARGO-GRAPH-P1-029 | 638个dependency occurrence分散在159包 | `DependencyPolicyCatalog`聚合name/source/version/default/features/role并生成manifest projection |
| CARGO-GRAPH-P1-030 | 146个registry declaration未走workspace authority | 共享依赖集中版本与feature baseline；局部override必须有结构化理由 |
| CARGO-GRAPH-P1-031 | `proc-macro2/syn/tokio`存在多种直接requirement | 明确是否兼容、收敛或隔离，并在graph diff显示直接约束变化 |
| CARGO-GRAPH-P1-032 | root/plugin/Tray各有大量multi-version name | 为wgpu/naga/winit/serde/tokio/native sys等singleton family设置fail gate，其余建立预算 |
| CARGO-GRAPH-P1-033 | root/plugin共同name有105组version-set差异 | shared 26-package parity receipt必须列出每项差异及是否影响ABI/schema/artifact |
| CARGO-GRAPH-P1-034 | WOC/Tray与根分别有13/77组version-set差异 | 产品边界若传递wire/file/FFI类型，必须验证schema/ABI而非假设同名crate等价 |
| CARGO-GRAPH-P1-035 | cargo-deny矩阵不含WOC与Tray | 所有shipping/product/tool workspace进入advisory/license/source/duplicate policy或显式豁免 |
| CARGO-GRAPH-P1-036 | source-leveldomain audit不约束manifest dependency | package role声明allowed dependency layer，Cargo edge越层在metadata阶段失败 |
| CARGO-GRAPH-P1-037 | platform边界大多不在Cargo target graph | Windows/Linux/macOS/Android/Web各自产生resolved graph并比较非法native dependency |
| CARGO-GRAPH-P1-038 | 没有unused direct dependency门 | 编译/源码/feature三方分析识别未消费dependency，删除前走owner review |
| CARGO-GRAPH-P1-039 | 没有minimum-version与dependency update兼容lane | 对公共SDK/format/ABI相关图运行最小支持、受控update和lock diff qualification |
| CARGO-GRAPH-P1-040 | 150个path dependency均无version且157包默认可publish | PackageRole生成`publish=false`或path+version发布合同，禁止偶然依赖Cargo报错保护 |
| CARGO-GRAPH-P1-041 | 159包没有`package.metadata`身份 | 声明PackageId、role、owner、visibility、stability、ABI/schema/publication与product consumers |
| CARGO-GRAPH-P1-042 | 全部package有效版本均为0.1.0 | internal同代可共享BuildSet；SDK/ABI/tool/plugin需独立compat version和release policy |
| CARGO-GRAPH-P1-043 | 338 target object没有统一TargetId/PackageId关系 | catalog显式投影lib/cdylib/proc/bin/test/build target、required feature、platform和consumer |
| CARGO-GRAPH-P1-044 | 41个plugin cdylib与`plugin.toml`/ABI artifact非同源 | 生成package-target-module-artifact bijection，漏项、重复或crate type不匹配均失败 |
| CARGO-GRAPH-P1-045 | runtime同一lib target同时产`rlib`和`cdylib` | 两种artifact各有ABI/feature/profile/consumer identity，不能只按package name归档 |
| CARGO-GRAPH-P1-046 | library/test target大量依赖Cargo自动发现 | inventory gate记录auto target并在新增/删除时要求owner、role与test-plan更新 |

## 10. P1：Resolved Receipt 与 Qualification

| ID | 当前差距 | 需要重构 |
|---|---|---|
| CARGO-GRAPH-P1-047 | `cargo metadata/tree`结果是临时终端文本 | 生成schema-versioned `ResolvedPackageGraphReceipt`，绑定source、manifest、lock、toolchain、target、profile和features |
| CARGO-GRAPH-P1-048 | PR/CI没有package/feature/dependency graph diff与预算 | required gate输出added/removed/changed node/edge、role污染、version drift、cost与owner decision |

## 11. P2：可维护性与开发体验

| ID | 当前差距 | 需要重构 |
|---|---|---|
| CARGO-GRAPH-P2-001 | 34个feature package缺description | 从PackageCatalog生成准确职责说明并检查与role一致 |
| CARGO-GRAPH-P2-002 | package name、plugin id、capability id与artifact name难互查 | 提供单命令按任一ID反查完整映射与consumer |
| CARGO-GRAPH-P2-003 | plugin workspace手写139个member | catalog生成或验证排序、存在、唯一和退役状态，避免列表静默漂移 |
| CARGO-GRAPH-P2-004 | feature没有owner、文档或deprecation字段 | 生成feature reference，标注public/internal、cost、incompatibility和replacement |
| CARGO-GRAPH-P2-005 | lock diff没有workspace/product摘要 | dependency update自动输出direct/transitive、license/advisory、singleton和artifact impact |
| CARGO-GRAPH-P2-006 | graph失败只暴露Cargo自由文本 | 转换为stable diagnostic code、PackageId、edge chain、requested role与修复owner |
| CARGO-GRAPH-P2-007 | target directory分散且不显示BuildSet identity | managed target lease显示workspace/lock/toolchain/target/profile/feature digest |
| CARGO-GRAPH-P2-008 | 没有package/feature/dependency规模趋势 | 记录node/edge/duplicate/feature/artifact size基线与增长原因 |
| CARGO-GRAPH-P2-009 | manifest字段顺序与声明风格不统一 | formatter/validator按package role输出canonical order，不做无意义全仓改写 |
| CARGO-GRAPH-P2-010 | dependency exception缺expiry/owner | 所有version/default/source/layer例外进入可到期结构化ledger |
| CARGO-GRAPH-P2-011 | resolved graph没有紧凑可视化投影 | 为开发者生成role/package/feature差异图，机器receipt仍是canonical |
| CARGO-GRAPH-P2-012 | graph inventory没有currentness入口 | `cargo zircon graph status`显示source/lock/receipt是否匹配及最早失效原因 |

## 12. 目标架构

```text
RepositoryContentManifest
          |
          v
CargoPackageCatalog ---- DependencyPolicyCatalog ---- FeatureConstraintSchema
          |                        |                         |
          +------------------------+-------------------------+
                                   v
                      ProductProfile / ProductRole
                                   |
                                   v
                       Canonical Cargo Invocation
                                   |
                         cargo metadata / resolve
                                   |
                                   v
                    ResolvedPackageGraphReceipt
                   /          |             \
          build script    target build     graph diff
          receipt ref       receipt          gate
                   \          |             /
                                   v
                              BuildSet
```

`CargoPackageCatalog`不是第二套Cargo parser。Cargo manifest仍是低层构建声明；catalog提供Cargo没有的engine role、owner、product/ABI/schema与publication语义。validator使用Cargo metadata作为最终resolve authority，并把catalog expectation与实际图比较。

核心不变量：

1. 每个package属于一个canonical workspace policy；shared projection必须显式列出所有context。
2. 每个product target恰好一个ProductRole；role不能由dependency default悄悄加入。
3. BuildSet保存请求和resolved两个feature集合；两者不同必须可解释，非法差异失败。
4. 每个Cargo target有PackageId、TargetId、role、platform、artifact/test/build owner与consumer。
5. 四个lock是否同代由ProductBuildSet声明，不由当前目录或最近一次Cargo命令决定。
6. Plugin descriptor、Cargo package/feature、compiled target、artifact和runtime provider形成可验证双射或显式一对多关系。
7. graph receipt只有绑定clean source、manifest/lock digest、toolchain、target和Cargo版本才可复用。

## 13. 重构里程碑

### M0 · Truth Freeze 与Editor Role止血

- 保存当前162/159/338 inventory与四lock digest，不更新依赖来掩盖差异。
- 修复Editor到Runtime的default leak，增加exact-one role resolver test。
- 将现有plugin locked metadata失败继续保留为Tooling01阻断。
- 禁止MVP/CI用请求feature字符串生成role qualification。

### M1 · Package、Workspace 与Dependency Catalog

- 为159包分类engine/runtime/editor/interface/plugin/tool/product/fixture/vendor。
- 明确四workspace及26个shared package的canonical关系。
- 定义dependency authority、singleton、source、version和exception schema。
- 为WOC与Tray接入dependency governance或批准的隔离policy。

### M2 · Feature Constraint 与Product Profile

- 分类208个feature并定义role/platform/backend/test/fault cardinality。
- 从runtime profile和plugin catalog生成Cargo projection，删除手写副本。
- 扩展resolver验证default、optional、weak edge、workspace merge与target predicate。
- 为Client/Server/Editor/Tool生成positive、negative和pairwise matrix。

### M3 · ResolvedPackageGraphReceipt

- 从locked Cargo metadata生成规范化package/version/source/target/feature graph。
- 绑定source、manifest、lock、Cargo/rustc、host/target、profile和environment allowlist。
- 输出graph digest、diff、role contamination和dependency policy diagnostic。
- Tooling18 target receipt与Tooling15 BuildSet引用graph receipt ID。

### M4 · Target、Plugin 与Artifact Closure

- 为338 target建立TargetId与owner，保留auto-discovery但禁止silent drift。
- 验证41个plugin cdylib、runtime dual crate type与plugin descriptor/artifact映射。
- Tooling05 build receipt作为7个custom-build node的输入，不复制其内部owner。
- TestPlan引用156个test target的graph generation。

### M5 · Cross-Platform、Update 与Publication Qualification

- 生成Windows/Linux/macOS/Android/Web resolved graphs并验证非法native edge。
- 运行minimum supported、controlled update、singleton convergence与public API/ABI lanes。
- 按PackageRole执行package/publish/license/SBOM门。
- 记录dependency/feature的compile time、artifact size和startup cost。

### M6 · Hard Cutover

- CI、MVP、Hub、export、plugin build和release只能引用current graph receipt。
- 无限定root Cargo命令明确标注partial，不生成engine qualification。
- 删除手写package/feature/target清单，只保留由catalog或receipt生成的projection。
- 旧receipt随任一manifest/lock/toolchain/profile变化自动失效。

## 14. 验收矩阵

| 资格面 | 必须证明 |
|---|---|
| inventory | 162 manifest对应159唯一package和338 target；新增/删除有owner decision |
| workspace | 每个package一个canonical workspace policy；26个shared projection逐项可解释 |
| lock | 每个产品声明完整lock set与digest；stale lock在编译前失败 |
| product role | Client/Server/Editor/Tool/Test每个target恰好一个role，Editor closure不含Client |
| feature | default/optional/workspace/target/weak dependency完整解算；非法组合fail-close |
| dependency | source/version/default/features/layer/singleton符合policy；四workspace均受治理 |
| target | lib/cdylib/proc/bin/test/build target均有TargetId、role、owner和consumer |
| plugin | package/feature/plugin manifest/module/artifact/provider闭包双向一致 |
| cross-platform | 各支持target graph不含不允许native/platform dependency |
| update | lock update、minimum version与shared dependency divergence有兼容结果和receipt |
| reproducibility | 两次clean resolve/build引用相同graph digest与BuildSet inputs |
| currentness | source/manifest/lock/toolchain/profile任一变化使旧receipt失效 |

## 15. 禁止旁路

1. 只给`zircon_editor`补`default-features = false`，却不建立role exclusivity与resolved receipt。
2. 用`cargo check`通过证明feature closure正确；编译成功允许多个role共存。
3. 用`--all-features`替代合法产品组合矩阵。
4. 把四个workspace强行合并，却不审查lock、profile、target directory与发布边界变化。
5. 仅比较Cargo.toml文本数组，不读取Cargo最终resolve graph。
6. 把package name或文件路径直接当稳定BuildSet/ABI/plugin identity。
7. 为了收敛版本无审查运行全量`cargo update`，删除可复核的旧lock差异。
8. 再写一份手工package/feature清单而没有manifest-to-catalog双向gate。

## 16. 本轮记录

本轮只新增review与索引，不修改Cargo manifest、lockfile、workflow、production、test或build script。只读metadata/tree没有编译源码或产生资格artifact。现有plugin lock、WOC compile、Hub compile与Editor aggregate阻断仍由原报告拥有，不能因本篇静态图审查完成而标记为通过。
