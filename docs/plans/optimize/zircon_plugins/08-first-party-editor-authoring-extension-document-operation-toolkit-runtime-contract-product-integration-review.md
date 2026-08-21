---
related_code:
  - zircon_plugins/material_editor
  - zircon_plugins/animation_graph
  - zircon_plugins/timeline_sequence
  - zircon_plugins/ui_asset_authoring
  - zircon_plugins/prefab_tools
  - zircon_plugins/tilemap_2d
  - zircon_plugins/terrain
  - zircon_plugins/texture
  - zircon_plugins/runtime_diagnostics
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_plugins/first_party_editor_catalog/src/lib.rs
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_editor/build.rs
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/core/plugin/catalog_store.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration/manager.rs
  - zircon_editor/src/ui/template_runtime/runtime/plugin_documents.rs
tests:
  - zircon_plugins/material_editor/editor/src/tests.rs
  - zircon_plugins/animation_graph/editor/src/tests.rs
  - zircon_plugins/timeline_sequence/editor/src/tests.rs
  - zircon_plugins/ui_asset_authoring/editor/src/tests.rs
  - zircon_plugins/prefab_tools/editor/src/tests.rs
  - zircon_plugins/prefab_tools/runtime/src/tests.rs
  - zircon_plugins/tilemap_2d/editor/src/tests.rs
  - zircon_plugins/tilemap_2d/runtime/src/tests.rs
  - zircon_plugins/terrain/editor/src/tests.rs
  - zircon_plugins/terrain/runtime/src/tests.rs
  - zircon_plugins/texture/editor/src/tests.rs
  - zircon_plugins/texture/runtime/src/tests.rs
  - zircon_plugins/runtime_diagnostics/editor/src/tests.rs
  - zircon_plugins/first_party_editor_catalog/src/tests.rs
  - zircon_editor/src/core/plugin/manager/tests.rs
  - zircon_editor/src/core/plugin/manager/tests/project_registration.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_editor/34-sprite-atlas-tileset-tilemap-canvas-2d-animation-collision-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/35-texture-image-cubemap-render-target-sampler-compression-streaming-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/44-archetype-class-defaults-instance-override-property-propagation-reset-to-default-authoring-review.md
  - docs/plans/optimize/zircon_editor/45-cinematic-sequencer-shot-track-binding-take-recorder-movie-render-queue-authoring-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/29-terrain-landscape-heightfield-quadtree-lod-material-layer-foliage-world-partition-physics-navigation-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/39-prefab-archetype-prototype-class-default-instance-override-runtime-instantiation-propagation-hot-reload-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/Toolkits/AssetEditorToolkit.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Toolkits/AssetEditorToolkit.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/WorkflowOrientedApp/WorkflowCentricApplication.h
  - dev/UnrealEngine/Engine/Source/Editor/LandscapeEditor/Private/LandscapeEditorModule.cpp
  - dev/UnrealEngine/Engine/Plugins/2D/Paper2D/Source/Paper2DEditor/Private/TileMapEditing/EdModeTileMap.cpp
  - dev/godot/editor/plugins/editor_plugin.h
  - dev/godot/editor/plugins/editor_plugin.cpp
  - dev/godot/editor/editor_data.h
  - dev/godot/editor/editor_data.cpp
  - dev/godot/editor/scene/texture/texture_editor_plugin.cpp
  - dev/Fyrox/editor/src/plugins/mod.rs
  - dev/Fyrox/editor/src/plugins/material/mod.rs
  - dev/Fyrox/editor/src/scene/commands/material.rs
  - dev/Fyrox/editor/src/plugins/animation/command/mod.rs
  - dev/Fyrox/editor/src/plugins/tilemap/commands.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_asset/src/loader.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Importers/ShaderGraphImporter.cs
  - dev/Graphics/Packages/com.unity.shadergraph/package.json
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 08 · First-Party Editor Authoring Extension、Document、Operation、Toolkit、Runtime Contract 与 Product Integration 工程化差距

## 1. 结论

本轮逐文件覆盖九个第一方编辑扩展：Material Editor、Animation Graph、Timeline Sequence、UI Asset Authoring、Prefab Tools、Tilemap 2D、Terrain、Texture 与 Runtime Diagnostics，并向下追到共享 `editor_support`、first-party editor/runtime catalog、App feature 组合、Editor extension/command/native registration、plugin document resolver及相关测试。九个包冻结112个tracked文件、7,171物理行、266,582 bytes，包含77个 `#[test]`；共享调用侧另冻结30个文件、8,346行、318,723 bytes。参考侧冻结Unreal、Godot、Fyrox、Bevy与Unity Graphics共19个文件、13,115行、476,470 bytes。

这些包不是完全没有价值。Material、Animation Graph、Timeline、Prefab、Tilemap与Terrain已有typed asset/helper、若干局部validator以及可枚举的操作描述；共享层能统一生成surface/view/menu/open command；Editor host也已有真正的operation factory入口、transaction/undo查询和plugin document resolver。问题在于这些基础停在“声明和孤立helper”层：九包共声明42个命令，除9个通用打开surface命令外，30个领域操作只有descriptor、没有event也没有operation factory；UI Asset的3个“创建”命令实际只发送OpenView。九包没有一个调用 `register_operation_command`，20条 `plugins://` 文档URI对应的包内资源全部缺失。

产品闭环更弱。first-party editor catalog只实际链接Navigation与Neural，不链接本轮九个editor provider；runtime catalog在四个带runtime包中只链接Texture，Prefab、Tilemap与Terrain缺席。五个editor-entry dist和四个runtime-entry dist都导出空bridge、空command callback、无state/ready/unload行为。native serialized contribution materializer又只能重建View/Drawer/Menu/Command/AssetType/SettingsPage，无法重建UI template、toolkit、scene mode、operation factory、compiler、preview或document provider。因此“manifest里存在包”“builtin catalog里出现descriptor”“孤立registry测试通过”均不等于真实Editor可挂载、命令可执行、资产可保存或Runtime可消费。

UI Asset Authoring和Runtime Diagnostics还会与内建 `editor.ui_asset`、`editor.runtime_diagnostics` view authority冲突；UI Asset open operation也重复。真实host会在view/command准入阶段拒绝，而包内测试使用空registry，恰好绕过冲突。Material compiler只折叠base color，Animation Graph compiler只返回output source字符串，Timeline move会先原地修改/排序再验证且失败不回滚，Prefab apply/revert/break只改局部DTO，Tilemap/Terrain helper没有事务、文档、持久化或Runtime系统，Texture“complete”runtime能力只计算宽高和单层texel数量，Diagnostics没有任何数据源。

上述硬阻塞已分别由Plugins01/06、Editor14/15/16/23/25/34/35/44/45/50及Runtime29/39/42拥有。本报告不复制同一问题的最高优先级owner，登记 **0项新增P0、72项P1和18项P2**；它拥有的是九包从source/native registration到document、operation、toolkit、compiler/preview、runtime consumer及产品catalog的纵向闭环。只写review与重构计划，不修改production/tests。

## 2. 范围、currentness 与动态证据边界

### 2.1 Zircon冻结范围

| 范围 | 文件 / 行 / bytes | 当前事实 |
|---|---:|---|
| `material_editor` | 10 / 1,043 / 39,280 | experimental；6个领域操作descriptor；base-color-only compiler |
| `animation_graph` | 10 / 1,015 / 38,720 | experimental；4个领域操作descriptor；palette与runtime node schema不一致 |
| `timeline_sequence` | 10 / 864 / 32,492 | experimental；5个领域操作descriptor；失败后可残留mutation |
| `ui_asset_authoring` | 9 / 524 / 20,099 | experimental；3个create operation只OpenView；与内建authority冲突 |
| `prefab_tools` | 16 / 916 / 34,332 | beta/partial；5个操作descriptor；runtime只有component/diagnostic importer |
| `tilemap_2d` | 16 / 903 / 32,946 | beta/partial；5个操作descriptor；runtime没有renderer/system |
| `terrain` | 16 / 856 / 31,660 | beta/partial；5个操作descriptor；runtime没有terrain service/system |
| `texture` | 16 / 614 / 20,925 | stable/complete声明；实际runtime只做尺寸摘要 |
| `runtime_diagnostics` | 9 / 436 / 16,128 | experimental；只有通用surface，没有数据provider |
| 九包合计 | 112 / 7,171 / 266,582 | SHA-256 manifest `3bc7523221ac52beba9f102f581d38cedf6e357e5d7d93c3b1103400f1a7478e` |
| 共享catalog/host/caller补充 | 30 / 8,346 / 318,723 | SHA-256 manifest `23e0189dec161ab7ca0b18a8097dd6f250a201f82950058f94cb7d7f3a286af2` |

fingerprint按相对路径不区分大小写排序，对文件计算SHA-256，再对 `path|hash` 的LF连接串计算SHA-256。九个package root在冻结时没有tracked working-tree改动；共享范围中command registry、catalog store、plugin manager tests、native registration manager与plugin document resolver已有其他会话或用户改动，所以本文标记 `source_recheck_required: true`，实施前必须重新绑定基线。

### 2.2 命令、文档与测试实数

| 证据 | 数量 | 解释 |
|---|---:|---|
| package tests | 77 | 无ignored/should_panic；主要覆盖descriptor、helper与isolated registration |
| package command descriptors | 42 | 9个surface open；30个无event/factory的领域操作；3个UI“创建”只OpenView |
| `register_operation_command` / factory registration | 0 | host已有执行入口，但九包没有绑定执行体 |
| `plugins://` URI | 20 | 20/20对应package-relative文件不存在 |
| editor catalog实际链接本轮provider | 0 / 9 | generated metadata catalog不能替代source provider调用 |
| runtime catalog实际链接四个runtime-backed包 | 1 / 4 | 只有Texture；Prefab/Tilemap/Terrain缺席 |
| native dist bridge methods / invoke callback | 0 / 9 | manifest存在不等于行为可重建 |

本轮未运行Cargo、Editor GUI、NativeDynamic加载、真实资产save/reopen/cook或Runtime play。静态源码足以证明URI文件不存在、factory注册为零、catalog feature closure缺失以及若干helper语义，但不把这些判断写成“构建已失败”或“运行时基准已验证”。

### 2.3 参考冻结范围

参考19文件fingerprint为 `7b378fa7ded4fea7a9ec7ce010db950ba6c3997bbda83c8a78eb05bfb810adf1`。比较只提取工程合同，不照搬对象模型：

| 参考 | 可验证的工程机制 | 对Zircon的约束 |
|---|---|---|
| Unreal AssetEditorToolkit/Workflow | 持有真实editing object、tab/layout/command注册、save/save-as、close与生命周期对称性 | toolkit必须绑定document identity、dirty/save/close、command context及mount receipt |
| Unreal Landscape/Paper2D | module startup/shutdown、真实edit mode、viewport input与transaction | Terrain/Tilemap不能以纯DTO helper冒充完整authoring mode |
| Godot EditorPlugin/EditorData | handles/edit/make_visible、state/save/apply、unsaved status、undo-redo、add/remove扩展点 | plugin admission必须有可撤销mount和真实对象/状态authority |
| Fyrox editor plugins/commands | plugin生命周期与Material/Animation/Tilemap大量execute/revert命令 | descriptor与helper必须落到可逆command/transaction，不应直接改数组后补验证 |
| Bevy Plugin/AssetLoader | executable build/ready/finish/cleanup；typed Asset/Settings/Error和LoadContext | capability发布必须来自已执行provider和typed contract，不来自manifest字符串 |
| Unity ShaderGraph importer | 反序列化/校验graph，生成真实shader/material/compute subasset，登记dependency | graph“compile”必须产出可追踪artifact与产品consumer，不能只是摘要DTO |

## 3. 当前真实数据流与断点

```text
package declaration / build-generated metadata
  -> EditorPluginDescriptor::builtin_catalog()        (metadata only)
  -> first_party_editor_catalog selected providers    (本轮 0/9)
  -> App editor host registration vector
  -> register View/Menu/Command descriptors
       -> plugins:// document needs bound plugin root + real file
       -> operation command needs factory or event
       -> duplicate built-in view/command is rejected
  -> native dist serialized contribution materializer
       -> only View/Drawer/Menu/Command/AssetType/SettingsPage
       -> no template/toolkit/factory/compiler/preview/document provider
  -> product document/save/cook/runtime consumer       (本轮没有闭环)
```

`EditorPluginRegistrationReport::from_plugin`把plugin注册到新的空registry，能证明单包descriptor内部没有自冲突，但不能证明与内建Editor或其他插件共存。source registration也不绑定plugin root；只有native discovery manager调用template root绑定，而包内文件仍不存在。operation invocation先处理event，缺event时查factory，所以30个领域操作会以MissingFactory结束，UI三个create则只打开view。

## 4. 可保留基础

1. 九包的manifest、source/editor/runtime/dist分层已提供统一审计入口。
2. `EditorAuthoringContributionBatch`能聚合view/menu/command/asset type/palette/template等descriptor。
3. `register_authoring_surface`统一surface ID、菜单和OpenView事件，适合保留为最小shell构建器。
4. Editor command registry已有operation factory、transaction outcome、undoable查询入口。
5. plugin document resolver已经要求owner-scoped URI和绑定root，安全方向正确。
6. Material/Animation Graph/Timeline的typed DTO与局部validator可作为后续document model输入。
7. Prefab override、Tilemap cell、Terrain heightfield与Texture descriptor已有最小领域词汇。
8. package tests能快速验证声明投影，适合作为更深产品测试的底层单元层。
9. manifest生成和native projection已有结构化协议基础，不需要退回手写动态符号散点。
10. first-party catalog把“可构建依赖”与“启动时选择”集中在少数模块，适合硬切唯一产品authority。

## 5. P0归属：本文不新增最高优先级finding

| 现象 | canonical owner | 本文边界 |
|---|---|---|
| first-party selection/profile/provider缺失和complete状态失真 | Plugins06 `FP-CATALOG-P0-001..005` | 只记录九包的具体纵向证据与低层合同 |
| native ABI/load/admission/source-native parity父问题 | Plugins01 | 只定义本轮贡献类型的parity验收 |
| Animation Graph/State Machine产品、toolkit、transaction、compiler/preview | Editor14 `P0-1..5` | 只审package provider如何接入该产品authority |
| Material graph资源、factory、compiler、schema、document/preview | Editor15 `P0-1..5` | 只审插件包内的降级和接线 |
| Terrain workbench/runtime/world authority | Editor16 `P0-1..5`；Runtime29 | 只审Terrain package contribution闭环 |
| UI Asset创建命令和plugin resource/factory/catalog | Editor23 `P0-5` | 只审重复authority和伪create行为 |
| Diagnostics真实数据/远程/时钟/perf/telemetry | Editor25 `P0-1..6` | 只审runtime_diagnostics包的无provider状态 |
| Tilemap持久化/runtime/plugin/schema | Editor34 `P0-1..5` | 只审package helper和runtime catalog |
| Texture产品/importer/runtime能力失真 | Editor35 `P0-1..5`；Plugins06 `FP-CATALOG-P0-005` | 只审stable/complete包的证据资格 |
| Prefab静态workbench、工厂/backend/resource、override authority | Editor44 `E-DEFAULT-P0-01..05`；Runtime39 | 只审prefab_tools包内 destructive helper与接线 |
| Sequencer假产品、资源/factory/bridge、move原子性、marker能力 | Editor45 `P0-01..05` | 只审timeline_sequence的包级实现 |
| extension mount/revoke/capability/callback/toolkit lock | Editor50 `E-EXT-P0-01..05` | 复用其宿主生命周期，不另建第二套host |

## 6. P1：共享Package、Catalog、Admission 与Resource Contract

| ID | 当前差距 | 需要重构 |
|---|---|---|
| EAP-P1-001 | 九包没有统一声明source/native支持的贡献种类 | 建立 `EditorExtensionPackageContract`，枚举surface、document、operation、toolkit、compiler、preview、runtime dependency与native parity |
| EAP-P1-002 | build-generated builtin catalog只生成metadata descriptor | 将metadata与可执行provider分离为明确类型，catalog不得把descriptor row当作已加载extension |
| EAP-P1-003 | editor catalog对九包0链接，却没有缺失选择的可解释报告 | 输出requested/resolved/linked/admitted/activated逐阶段receipt和拒绝原因 |
| EAP-P1-004 | runtime catalog只链接Texture，Prefab/Tilemap/Terrain无闭环 | 每个editor package声明required runtime provider，catalog解析依赖闭包并fail-close |
| EAP-P1-005 | App `target-editor-host` feature只组合Navigation/Neural | 由版本化产品profile生成first-party extension closure，禁止Cargo feature与运行时选择双重漂移 |
| EAP-P1-006 | source registration不绑定plugin root | source/native都由同一mount transaction提供canonical resource root与只读resource broker |
| EAP-P1-007 | 20个URI对应文件全部缺失，manifest不校验资源存在 | build/package阶段生成资源manifest，校验路径、hash、media type、schema、locale与size budget |
| EAP-P1-008 | document URI只在运行时首次打开时暴露失败 | admission先解析并编译required documents；optional文档也必须显式降级而非空白view |
| EAP-P1-009 | resource owner来自URI字符串，未绑定package identity/version | URI解析为 `PackageId + ResourceId + Version`，禁止owner伪造和跨包相对跳转 |
| EAP-P1-010 | UI template、asset template、component template混用URI但无typed用途 | 为每类资源定义schema和consumer，admission按用途验证，不接受“文件存在即有效” |
| EAP-P1-011 | isolated registration测试绕过内建host冲突 | 增加完整默认Editor registry上的admission测试，覆盖view/command/menu/resource/capability冲突 |
| EAP-P1-012 | UI Asset和Diagnostics与内建view authority重复 | 建立唯一surface authority表；包选择replace/decorate/extend之一并由host原子仲裁 |
| EAP-P1-013 | duplicate command/view报错缺少双方来源上下文 | conflict diagnostic包含现有/候选package、版本、contribution ID、profile与解决建议 |
| EAP-P1-014 | contribution batch只有descriptor vectors | 扩展为typed provider registrations，descriptor必须引用同批次可执行provider或明确readonly |
| EAP-P1-015 | package capability status与实际可执行贡献无绑定 | capability只在resource、factory、toolkit和runtime dependency全部admitted后发布 |
| EAP-P1-016 | admission没有对必需/可选贡献做原子性区分 | 定义required contribution set；任一必需项失败则整包零发布，可选项产生degraded receipt |
| EAP-P1-017 | package卸载没有可审计的反向顺序 | mount返回 `ExtensionMountReceipt`，按依赖逆序撤销resource/factory/command/menu/view/runtime lease |
| EAP-P1-018 | 包升级/重载没有document和operation generation隔离 | contribution handle携带generation；旧document完成迁移/关闭后才能释放旧provider |

## 7. P1：Operation、Document、Toolkit、Native Parity 与Maturity

| ID | 当前差距 | 需要重构 |
|---|---|---|
| EAP-P1-019 | 30个领域命令没有event或factory | descriptor必须绑定 `ExecutableOperationBinding`；缺绑定的命令不进入可用菜单/palette |
| EAP-P1-020 | UI三个create命令错误地绑定OpenView | create操作接收目标目录、命名、模板与冲突policy，执行资产写事务并返回新document identity |
| EAP-P1-021 | operation list把无factory命令显示为非undoable而非unavailable | 暴露availability、reason、undo policy、selection predicate与required capability |
| EAP-P1-022 | helper直接接收数组index/DTO，未绑定当前document revision | operation input使用stable object ID、document ID、base revision和selection snapshot |
| EAP-P1-023 | factory错误没有统一映射到用户可恢复状态 | 定义typed rejection：Unavailable/Stale/Conflict/Invalid/Cancelled/BackendFailed，并保留诊断receipt |
| EAP-P1-024 | package没有统一dirty/save/save-as/close协议 | 建立 `AuthoringDocumentProvider`，绑定source URI、schema、revision、dirty reason、save transaction与close veto |
| EAP-P1-025 | surface等同于view，没有真实editing object | toolkit实例必须持有document lease、selection context、command context、preview session与tab layout |
| EAP-P1-026 | 多document/多window状态没有package所有权 | document identity与toolkit instance分离，支持同asset单authority或显式多view共享session |
| EAP-P1-027 | package没有crash/reload恢复快照 | 以schema化session state保存open docs、layout、selection、unsaved operation journal并版本迁移 |
| EAP-P1-028 | 五个editor native entry和四个runtime native entry行为均为空 | native export必须承载与source等价的provider callback表或明确拒绝不支持的贡献类型 |
| EAP-P1-029 | serialized materializer只能重建六类metadata贡献 | 扩展ABI以注册resource/toolkit/factory/compiler/preview/document provider，并带size/version/capability negotiation |
| EAP-P1-030 | native `invoke_command: None` 却可发布命令descriptor | admission禁止无callback的可执行命令；纯导航event也必须由host能力明确实现 |
| EAP-P1-031 | `bridge_methods: []` 与manifest capability并存 | 每项跨边界能力绑定bridge method/table entry与probe receipt，不能依赖字符串声明 |
| EAP-P1-032 | save/restore/unload/on_host_ready全部为空且不降级 | lifecycle支持矩阵进入manifest；需要状态的package缺callback时拒绝激活 |
| EAP-P1-033 | source/native parity没有相同fixture对比 | 同一package在SourceLinked/NativeStatic/NativeDynamic产生规范化相同的贡献图、资源hash与行为结果 |
| EAP-P1-034 | experimental/beta/stable/complete未绑定门槛 | maturity policy绑定真实host mount、operation、save/reopen、cook/play、native parity、failure和性能证据 |
| EAP-P1-035 | package tests只证明局部helper/descriptor | 建立分层矩阵：unit、default-host admission、document lifecycle、operation undo、native parity、runtime consumer |
| EAP-P1-036 | 没有从package到产品能力的可观测链 | 生成 `PackageActivationReceipt`，记录provider、resource、document、command factory、runtime module和用户可见surface |

## 8. P1：Material Editor

| ID | 当前差距 | 需要重构 |
|---|---|---|
| EAP-P1-037 | graph ZUI和default material graph资源缺失 | 资源随包发布并经过schema编译、hash、版本和默认资产save/reopen验收 |
| EAP-P1-038 | 六个create/connect/disconnect/delete/compile操作只有descriptor | 绑定可逆graph command factory，按stable node/pin ID执行并产生transaction receipt |
| EAP-P1-039 | validator只检查少数ID/output/base-color连通性 | 校验typed pin、方向、cardinality、cycle、required input、parameter identity、stage/domain和asset reference |
| EAP-P1-040 | palette的Add/Multiply pin声明float，compiler按四维color求值 | 建立唯一typed IR与显式cast规则，palette、serializer、validator、compiler共享schema |
| EAP-P1-041 | compiler只折叠base color，纹理参与Add/Multiply即拒绝 | graph lower到真实shader/material IR，覆盖normal、metallic、roughness、AO、emissive、alpha和domain/options |
| EAP-P1-042 | compile结果没有source/dependency/toolchain/artifact identity | 产出版本化artifact key、dependency graph、diagnostic spans、backend binary/reflection与install receipt |

## 9. P1：Animation Graph

| ID | 当前差距 | 需要重构 |
|---|---|---|
| EAP-P1-043 | authoring、graph player与state-machine player三份ZUI缺失 | 发布并编译资源，分别绑定document、preview player与debug overlay provider |
| EAP-P1-044 | 四个graph/state operation只有descriptor | 以stable node/transition/state ID绑定可逆factory和selection predicate |
| EAP-P1-045 | palette声明BlendSpace1D/2D，runtime node asset没有对应variant | 由单一node schema生成palette、serialization、runtime evaluator和migration |
| EAP-P1-046 | graph compile只返回output source字符串 | 产出拓扑排序、typed ports、clip/mask/parameter bindings和可执行evaluation program |
| EAP-P1-047 | state machine compile只返回entry与计数 | 编译transition条件、priority、duration、interrupt、exit time、blend profile和parameter layout |
| EAP-P1-048 | validator缺cycle/拓扑/参数兼容和运行时能力检查 | 对非法反馈、不可达state、歧义transition、missing clip/mask和backend capability fail-close |
| EAP-P1-049 | 没有preview与runtime evaluator一致性证据 | editor preview和runtime使用同一compiled artifact；fixture逐帧比较pose/event/root motion |

## 10. P1：Timeline Sequence

| ID | 当前差距 | 需要重构 |
|---|---|---|
| EAP-P1-050 | authoring ZUI缺失且五个操作无factory | 发布真实Sequencer document/toolkit，所有track/key/binding操作走transaction |
| EAP-P1-051 | move以binding/track/key数组index寻址 | 改用稳定GUID和document revision，排序不得改变操作对象身份 |
| EAP-P1-052 | move先修改并排序，整体validation失败时不回滚 | 在working copy验证后原子commit，任何失败保持byte-for-byte零变化 |
| EAP-P1-053 | sorted helper只返回路径报告，不写回asset | 明确区分query与normalization command；需要规范化时生成可撤销patch并持久化 |
| EAP-P1-054 | local `TimelineEventMarker`不进入AnimationSequenceAsset | 定义版本化marker/event/payload schema并贯通serializer、editor、runtime dispatch与cook |
| EAP-P1-055 | track descriptor没有property binding解析/重绑定合同 | 绑定stable object/component/property identity，处理rename、missing target、type migration和multi-object |
| EAP-P1-056 | 没有evaluation/capture/render consumer | 由Editor45 Sequencer authority提供确定性evaluation、scrub、play、event、capture和artifact receipt |

## 11. P1：UI Asset Authoring 与 Runtime Diagnostics

| ID | 当前差距 | 需要重构 |
|---|---|---|
| EAP-P1-057 | UI Asset四份ZUI缺失 | 将layout/widget/style authoring作为真实typed document资源发布并验证引用闭包 |
| EAP-P1-058 | UI Asset view/command与内建authority重复 | 在Editor23唯一authority下选择插件扩展点或硬切替换，禁止平行surface ID |
| EAP-P1-059 | create layout/widget/style只OpenView | 写入模板实例、建立AssetId、索引、dirty/save/reopen并打开新document |
| EAP-P1-060 | UI Asset没有canvas/inspector/hierarchy/binding provider | package只提供可组合贡献，实际编辑状态归Editor23 document/toolkit authority |
| EAP-P1-061 | Diagnostics ZUI缺失且surface与内建重复 | 合并到Editor25唯一Diagnostics product，插件仅扩展pane/data provider |
| EAP-P1-062 | Diagnostics没有runtime/interface数据源 | 注册typed diagnostic stream provider，声明transport、clock、sampling、backpressure、privacy与lifetime |
| EAP-P1-063 | Diagnostics native entry没有bridge/command | source/native均需通过同一query/subscription ABI取得数据，并验证disconnect/reconnect/unload |

## 12. P1：Prefab、Tilemap、Terrain 与 Texture

| ID | 当前差距 | 需要重构 |
|---|---|---|
| EAP-P1-064 | Prefab三份资源缺失，五个操作无factory | 资源随包发布；create/apply/revert/break绑定Editor44 transaction和Runtime39 authority |
| EAP-P1-065 | Prefab同一路径override被BTreeMap静默覆盖 | admission拒绝重复或保留ordered conflict，typed property path携带owner/type/schema |
| EAP-P1-066 | Prefab apply清空instance overrides但不写source prefab | 两端锁定后生成source+instances原子事务，失败补偿且保留override |
| EAP-P1-067 | revert/break只清vector或返回DTO | revert按inheritance重算实例；break实际物化subtree/identity/reference并可撤销 |
| EAP-P1-068 | Prefab runtime只有component和DiagnosticOnly importer | 建立instantiate/propagate/hot-reload/save/network consumer后才发布partial runtime capability |
| EAP-P1-069 | Tilemap资源缺失，paint直接按数组index修改 | 采用stable layer/cell identity、stroke transaction、undo merge、dirty/save与selection snapshot |
| EAP-P1-070 | Tilemap runtime只有component和广泛`.json` diagnostic matcher，projection又只有supports布尔值 | 缩窄格式probe，提供typed importer、renderer、collision/navigation/streaming consumer，并实现正交/等距/六边形坐标、picking、bounds与render order |
| EAP-P1-071 | Terrain资源缺失、五个操作无factory，LayerStack复用heightfield sample plan且runtime无terrain system | 建立有预算且区分height/layer/channel/format/endian语义的importer，并接入document transaction、render/physics/nav/world service |
| EAP-P1-072 | Texture以stable/complete发布，但runtime只计算宽高、`max(1)` mip数和base-level texel count | 以真实decode、mip、format/compress、memory/residency、stream artifact与Editor preview/reimport替换摘要，并撤销无资格证据的complete状态 |

## 13. P2：产品化与维护质量

| ID | 改进项 | 目标 |
|---|---|---|
| EAP-P2-001 | UI Asset、Texture、Diagnostics缺README | 每包说明authority、能力边界、资源、依赖、失败模式与最小产品例程 |
| EAP-P2-002 | display/menu/category字符串散落 | 统一localization key与术语表，descriptor只引用稳定文本identity |
| EAP-P2-003 | icon与菜单分类没有跨包规范 | 建立领域icon、menu placement、command discoverability与冲突策略 |
| EAP-P2-004 | extension UI缺焦点/键盘/accessibility基线 | 对toolkit、palette、graph、timeline、canvas建立focus order、screen reader和contrast门 |
| EAP-P2-005 | 缺官方sample assets | 为九包提供小型、真实、版本化、许可明确的成功与失败样本 |
| EAP-P2-006 | 缺截图和视觉回归 | 对每个可见surface建立desktop/DPI/theme/empty/error/large-state golden |
| EAP-P2-007 | graph/timeline/prefab payload schema未文档化 | 生成字段、版本、migration、unknown data与roundtrip说明 |
| EAP-P2-008 | 第三方扩展作者没有同类范例 | 提供source/native等价的document+operation+runtime dependency参考包 |
| EAP-P2-009 | 没有extension使用/失败/perf telemetry | 只采集有consent的聚合指标，区分admission、activation、operation和backend失败 |
| EAP-P2-010 | 包重命名/废弃缺用户迁移UX | 提供profile/asset/command/resource ID remap和可预览迁移报告 |
| EAP-P2-011 | 未保存文档崩溃恢复UX缺失 | 展示恢复来源、revision、operation journal、冲突与丢弃选择 |
| EAP-P2-012 | 多窗口/多document交互没有一致规范 | 统一tab restore、focus、selection sharing、close prompts和duplicate open行为 |
| EAP-P2-013 | 多人协作时缺asset ownership提示 | document显示lease/lock/revision/conflict状态，不暗示无冲突保存 |
| EAP-P2-014 | 包资源没有体积与去重治理 | 统计模板/icon/sample/compiled cache大小，按hash去重并设profile预算 |
| EAP-P2-015 | authoring helper没有profile可见成本 | 暴露compile/validation/preview/paint/apply阶段耗时、allocation和取消延迟 |
| EAP-P2-016 | 快捷键不可按领域/上下文定制 | command identity与input chord解耦，支持context-aware remap及冲突检查 |
| EAP-P2-017 | preview质量/设备档位缺统一入口 | graph/material/texture/terrain preview使用产品quality profile和明确降级提示 |
| EAP-P2-018 | package metadata缺兼容性展示 | 显示engine/schema/ABI/platform/profile/dependency支持窗和最近资格receipt |

## 14. 目标合同

### 14.1 Package与mount

```rust
pub struct EditorExtensionPackageContract {
    pub package: PackageIdentity,
    pub required_runtime: Vec<CapabilityRequirement>,
    pub required_resources: Vec<ResourceRequirement>,
    pub contributions: Vec<TypedContributionRequirement>,
    pub lifecycle: LifecycleSupport,
    pub maturity: EvidenceBackedMaturity,
}

pub struct ExtensionMountReceipt {
    pub package: PackageIdentity,
    pub generation: u64,
    pub mounted: Vec<ContributionHandle>,
    pub degraded: Vec<DegradationReason>,
    pub runtime_leases: Vec<CapabilityLease>,
}
```

mount必须是一笔事务：先解析依赖和资源、验证所有required contribution、建立provider与document root，再原子发布命令/menu/view/capability。失败时不得留下半个surface；unmount按receipt逆序撤销。

### 14.2 Operation与document

```rust
pub struct ExecutableOperationBinding {
    pub command: CommandId,
    pub factory: OperationFactoryHandle,
    pub availability: OperationAvailabilityProvider,
    pub undo_policy: UndoPolicy,
    pub required_capabilities: Vec<CapabilityRequirement>,
}

pub trait AuthoringDocumentProvider {
    fn open(&self, asset: AssetId) -> Result<DocumentSession, DocumentOpenError>;
    fn save(&self, request: SaveRequest) -> Result<SaveReceipt, SaveError>;
    fn close(&self, request: CloseRequest) -> Result<CloseReceipt, CloseError>;
}
```

所有mutation携带DocumentId、base revision和stable object ID。factory在working copy上验证并输出patch/undo payload；commit后才改变dirty revision。MissingFactory不是可发布产品状态。

### 14.3 Compiler、preview与runtime consumer

```text
Authoring document
  -> validate(schema + dependencies + backend capabilities)
  -> compile(versioned IR + diagnostics + dependency graph)
  -> artifact(source/settings/toolchain/platform/schema key)
  -> preview install receipt
  -> cook/install receipt
  -> runtime consumer capability + behavior evidence
```

Material、Animation Graph、Timeline、Terrain与Texture必须沿用对应Editor/Runtime canonical product authority；插件负责提供provider和资源，不再建立平行假workbench。Prefab、Tilemap同理，任何partial capability必须明确列出缺失阶段。

### 14.4 Source/native parity

同一contract生成source registrar和native ABI projection。parity不是descriptor JSON相同，而是以下规范化结果相同：贡献图、required resource hash、operation availability、成功/失败结果、document roundtrip、runtime dependency、lifecycle撤销和capability receipt。native ABI无法表达的贡献必须在打包时失败，不能静默丢弃。

## 15. 依赖有序重构里程碑

| Milestone | 内容 | 依赖 | 退出条件 |
|---|---|---|---|
| M0 · Baseline recheck | 重扫共享dirty文件、冻结九包/source/native/catalog产品路径 | 无 | fingerprint、caller graph、冲突表和owner表更新 |
| M1 · Contract | 实现package/resource/operation/document/mount receipt合同 | M0；Editor50/Plugins01 | 默认host能原子admit/reject/unmount测试包 |
| M2 · Catalog closure | 统一metadata/provider/profile/runtime dependency解析 | M1；Plugins06/Runtime42 | requested到activated全链receipt，九包不再隐式缺席 |
| M3 · Resource/native parity | 发布20项required资源或删除无效引用；扩展native provider ABI | M1-M2 | source/static/dynamic贡献图与resource hash一致 |
| M4 · Document/operation shell | 真实document lifecycle、factory、undo、dirty/save/close | M1-M3 | 所有可见领域命令可执行或明确不可用，无MissingFactory |
| M5 · Domain convergence A | Material、Animation Graph、Timeline接入Editor14/15/45 | M4及对应canonical P0 | 真实artifact/preview/runtime fixture闭环 |
| M6 · Domain convergence B | UI、Diagnostics、Prefab、Tilemap、Terrain、Texture接入唯一产品authority | M4及Editor16/23/25/34/35/44、Runtime29/39 | save/reopen/cook/play和failure测试闭环 |
| M7 · Qualification | maturity重算、兼容/性能/恢复/卸载/长期测试 | M5-M6 | stable/complete只来自版本化evidence bundle |

不得先批量补20个空ZUI来“消除missing”，也不得先给30个命令绑定总是成功的空factory。资源、document、operation和runtime consumer必须按同一vertical slice验收。

## 16. 验收门

| Gate | 必须提供的证据 |
|---|---|
| EAP-G01 | 九包及共享caller重新冻结，所有在途文件绑定新的source fingerprint |
| EAP-G02 | 每个package有唯一metadata descriptor和唯一可执行provider映射 |
| EAP-G03 | product profile解析requested/resolved/linked/admitted/activated全阶段receipt |
| EAP-G04 | 九包required editor/runtime dependency closure无隐式缺项 |
| EAP-G05 | 20个保留URI均有manifest/hash/schema/owner并在admission编译成功 |
| EAP-G06 | 删除的URI没有悬空descriptor、测试或打包记录 |
| EAP-G07 | default Editor host无view/command/menu/resource authority冲突 |
| EAP-G08 | conflict diagnostic同时报告现有与候选贡献来源 |
| EAP-G09 | required contribution任一失败时package零可见发布 |
| EAP-G10 | optional contribution降级有显式reason且不虚报capability |
| EAP-G11 | 所有领域命令都有factory/event或被标记unavailable并从可用入口移除 |
| EAP-G12 | UI三个create命令创建真实资产并save/reopen，而不是只OpenView |
| EAP-G13 | mutation使用stable ID和base revision，不以排序后数组index作为身份 |
| EAP-G14 | 每个失败operation保持document byte-for-byte零变化 |
| EAP-G15 | execute/undo/redo/reopen在同一fixture上得到规范化相同资产 |
| EAP-G16 | dirty/save-as/close-veto/crash recovery覆盖单文档和多文档 |
| EAP-G17 | unmount/reload按receipt逆序撤销且无悬空callback/resource/runtime lease |
| EAP-G18 | SourceLinked/NativeStatic/NativeDynamic贡献图和行为fixture等价 |
| EAP-G19 | native缺callback/bridge时admission fail-close，不能只发布descriptor |
| EAP-G20 | Material typed IR覆盖palette声明节点和主要material channels |
| EAP-G21 | Material artifact绑定source/dependency/toolchain/platform/schema key并被真实preview消费 |
| EAP-G22 | Animation palette/serialized schema/evaluator node集合完全一致 |
| EAP-G23 | Animation Graph/State Machine compiled artifact由preview和runtime共同消费 |
| EAP-G24 | Timeline move/normalize/event marker通过failure atomicity与runtime dispatch测试 |
| EAP-G25 | UI Asset使用Editor23唯一document/toolkit authority，无平行假surface |
| EAP-G26 | Diagnostics使用Editor25真实数据authority，验证时钟/backpressure/disconnect/unload |
| EAP-G27 | Prefab apply/revert/break在source、instance、undo和runtime instantiate上闭环 |
| EAP-G28 | Tilemap paint/projection/import/render/collision或明确支持矩阵通过golden测试 |
| EAP-G29 | Terrain height/layer edit到render/physics/nav/world consumer具备预算与一致性证据 |
| EAP-G30 | Texture真实decode/mip/compress/stream/artifact替代摘要helper，并重算complete状态 |
| EAP-G31 | 77个既有单测保留，新增default-host、document、operation、native parity和runtime集成层 |
| EAP-G32 | experimental/beta/stable/complete均能追溯版本化资格bundle，过期证据自动降级 |

## 17. Owner边界与禁止事项

- Plugins01拥有native ABI、load-before-admission和通用source/native生命周期；本文只消费并扩展到authoring provider类型。
- Plugins06拥有first-party catalog/profile/capability真相；本文不另建九包专用catalog。
- Editor50拥有extension contribution store、mount/revoke/reload与toolkit provider父合同。
- Editor14/15/16/23/25/34/35/44/45及Runtime29/39/42拥有领域产品和Runtime authority；本文不把package helper提升为第二套产品后端。
- 禁止以“manifest含capability”“builtin catalog含descriptor”“isolated test通过”“view能打开”“helper返回Ok”作为完成证据。
- 禁止让native dist静默丢弃source贡献，禁止无factory命令进入可用菜单，禁止缺资源时渲染空白surface。
- 禁止通过复制内建view ID建立替代authority；替换必须是显式、原子、可回滚的profile决定。
- 禁止把单个尺寸摘要、validator计数或输出字符串命名为compile/process/complete。

## 18. 当前状态

本报告静态review完成，implementation仍为pending。九个直接package root在冻结时clean；共享Editor/caller范围存在在途改动，因此所有实现必须先执行M0重检。本轮没有修改production或tests，没有运行Cargo，也没有声称真实Editor挂载、NativeDynamic、save/cook/play、性能或稳定性通过。

在同场景、同资产、同画质、同平台、同硬件、同采样、同失败条件的可复现实验完成前，这些扩展及整个引擎都不能据此宣称性能或表现达到、超过当前Unreal。本文的作用是把“包存在”推进到可实施、可验收的工程闭环，而不是提高完成度标签。
