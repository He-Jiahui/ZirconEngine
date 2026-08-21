---
related_code:
  - zircon_editor/src/ui/host/builtin_views
  - zircon_editor/src/ui/host/view_registry.rs
  - zircon_editor/src/ui/host/editor_capabilities.rs
  - zircon_editor/src/ui/host/editor_subsystems.rs
  - zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing.rs
  - zircon_editor/src/ui/host/startup/welcome_view.rs
  - zircon_editor/src/ui/workbench/view
  - zircon_editor/src/ui/workbench/snapshot/workbench
  - zircon_editor/src/ui/workbench/reflection/activity_descriptors.rs
  - zircon_editor/src/ui/workbench/preset/design_stack.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/core/editor_extension/view_descriptor.rs
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/ui/workbench/event/menu_item_binding.rs
  - zircon_editor/assets/ui/editor
tests:
  - zircon_editor/src/tests/host/builtin_window_descriptors.rs
  - zircon_editor/src/tests/host/pane_template_descriptor.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup/window_topology.rs
  - zircon_editor/src/tests/workbench/registry/instance_policy.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/performance/01/2026-08-19-editor-ui-workbench-view-descriptor-instance-generation-plugin-lifecycle-architecture-review.md
  - docs/plans/performance/01/2026-08-19-editor-ui-binding-compiled-intent-generation-architecture-review.md
  - docs/plans/performance/01/2026-08-19-editor-ui-binding-dispatch-single-domain-request-architecture-review.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/WorkflowOrientedApp/WorkflowTabFactory.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/WorkflowOrientedApp/WorkflowTabFactory.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking/TabManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp
  - dev/godot/editor/docks/editor_dock_manager.h
  - dev/godot/editor/docks/editor_dock_manager.cpp
  - dev/godot/editor/plugins/editor_plugin.h
  - dev/godot/editor/plugins/editor_plugin.cpp
  - dev/Fyrox/editor/src/plugin.rs
  - dev/Fyrox/editor/src/plugins/material/mod.rs
  - dev/Fyrox/editor/src/plugins/animation/mod.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeComponentProvider.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Unity.RenderPipelines.Core.Editor.asmdef
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 52 · Editor Builtin View / Window Descriptor Catalog / Content Provider / Capability / Template / Localization 产品集成工程化差距

## 1. 结论

Zircon Editor已经有一套可保留的Workbench描述符骨架：40个内建描述符统一进入`ViewRegistry`，能够表达Activity View/Window、单例或多实例、dock policy、默认slot、document kind、约束、pane/window template与required capability；14个描述符声明pane template，8个声明activity-window template，目录引用的16份ZUI资源当前全部存在。UI Asset与Animation也证明真实session/toolkit可以建立在这套壳层之上，不能把现状误写成“完全没有视图系统”。

但目录当前不是产品能力目录，而是一个允许空内容通过的展示元数据清单。`ViewDescriptor`没有spawn/content provider，`open_descriptor()`只创建`ViewInstance`并立即返回成功；内容类型随后由另一份raw-ID `match`推断。40个内建描述符中有19个落入`ViewContentKind::Placeholder`，其中10个所谓functional panel和7个所谓functional window没有内容映射。前10个在production中只有默认布局引用，`editor.scene_game_window`没有任何目录外production引用，另外5个window只有command/menu入口。最终snapshot却把这些实例标记为`placeholder: false`，retained pane再显示“Missing View”。`editor.prefab`虽被映射为PrefabEditor，产品正文仍明确写着asset-specific tooling是placeholder。

这不是普通缺功能，而是feature admission失真：注册、菜单、默认布局、open success和测试都能向上游证明“功能存在”，真实内容却没有provider、toolkit、document session或可验证模板合同。本报告因此登记 **1项P0、40项P1、12项P2与32个资格门**。Editor52唯一拥有builtin catalog的产品真实性、typed content binding、capability closure、template/localization/icon编译与全目录资格；Editor50继续拥有extension contribution/reconcile/revoke，Editor13拥有布局恢复与迁移，Editor03/14/15/23/25分别拥有Prefab、Animation、Material、UI Asset与Diagnostics工具本体，既有性能报告拥有registry/session clone与生成代际成本。

## 2. 审查边界、currentness 与证据等级

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | 测试 / 资源 | 证据等级 | 本轮检查重点 |
|---|---:|---:|---|---|
| `ui/host/builtin_views/**` | 29 / 838 / 33,867 | 38个目录内描述符 | E3 | 逐文件读取全部Activity View/Window声明、能力映射和聚合入口 |
| 外部内建描述符 | 2个定义点 | UI Asset、Welcome | E3 | 40项完整catalog闭包 |
| 支撑产品路径 | 13个owner文件/目录 | registry、snapshot、pane projection、layout、menu、reflection | E3 | 从register/open到实际pane body反查内容和能力真实性 |
| 聚焦测试 | 5 / 1,305 / 47,857 | 25个`#[test]`、0 ignore | E2/E3 | descriptor metadata、host placement、capability toggle与instance policy |
| 唯一ZUI资源 | 16 / 2,771 / 135,439 | 16/16存在 | E2 | 只证明路径存在；不等同root/schema/payload/route或工具闭环 |
| 参考源码 | 14 / 11,421 / 415,202 | Unreal/Godot/Fyrox/Bevy/Unity Graphics | E2/E3 | spawn contract、插件生命周期、真实Control/tool owner、feature-conditioned provider |

目录源码按normalized relative path排序，写入`path + NUL + raw bytes + NUL`后取SHA-256，fingerprint为`5b1d206bd6fb5be9b3767672942eaa9a93db6fb585c4d62a7baf377e22fe003e`。五份聚焦测试fingerprint为`ee746a44ca702216b106d28b93255153c88cc20e67d8c581e5a566886073d26b`；16份资源fingerprint为`bf408f1cf43a67cf8a928eb81ceefefb9bbb2cc88868fd060dc5885bbe45b468`；14份参考源码fingerprint为`241e317875ff522ce4b188634a4fd25a220b156c24ad8b8b363a66deab1d7e05`。冻结基线为`bea1acf91b909525ab1759e2c800858b0eda6528`。

29个目录源码文件当前clean。`zircon_editor/src/ui/workbench/preset/design_stack.rs`与`zircon_editor/src/tests/host/manager/bootstrap_and_startup/window_topology.rs`存在非本轮产生的在途修改，所以本报告保守设置`source_recheck_required: true`；实施前必须重取这两个caller/test及40项catalog矩阵，不得回退共享工作树。

### 2.2 目录闭包

| 集合 | 数量 | ID / 当前事实 |
|---|---:|---|
| Activity View | 22 | 12个直接声明，加10个`functional_panel_view_descriptors()` |
| Activity Window | 18 | 9个直接声明、7个functional window，加外部UI Asset与Welcome |
| 有明确非Placeholder content映射 | 21 | 包含Project、Hierarchy、Inspector、Scene、Game、Assets、Console、Prefab、Asset Browser、UI Asset、Animation、Diagnostics等 |
| 未映射functional panel | 10 | `editor.prefab.viewport`、`editor.prefab.inspector`、`editor.material.graph`、`editor.material.preview`、`editor.ui.designer`、`editor.ui.source`、`editor.animation.timeline`、`editor.animation.graph`、`editor.asset_preview`、`editor.asset_metadata` |
| 未映射functional window | 7 | `editor.scene_game_window`、`editor.prefab_editor_window`、`editor.material_editor_window`、`editor.ui_asset_editor_window`、`editor.animation_editor_window`、`editor.asset_browser_window`、`editor.diagnostics_window` |
| 未映射special window | 2 | `editor.debug_observatory`、`editor.workbench_window` |
| 显式业务placeholder | 1 | `editor.prefab`被映射为PrefabEditor，但pane正文明确声明tooling仍是placeholder |

19个未映射ID会被`descriptor_content_kind()`的默认分支统一降为`Placeholder`。这19项中，17项名字和测试语义都把自己表述为functional editor panel/window；另外两项虽然有特殊shell/template路径，仍没有在snapshot类型系统中表达该特殊性。

### 2.3 产品调用证据

| ID集合 | 排除tests与`builtin_views`后的production引用 | 结论 |
|---|---|---|
| 10个functional panel | 每项仅在`ui/workbench/preset/design_stack.rs`出现2行 | 只是layout instance/placement，不是内容owner |
| `editor.scene_game_window` | 0行 / 0文件 | catalog外无产品caller |
| Prefab/Material/Animation/Diagnostics functional window | 每项2行 / 2文件 | 仅command defaults与menu binding |
| `editor.ui_asset_editor_window` | 3行 / 3文件 | command、Welcome action与menu；真实UI Asset session使用另一ID `editor.ui_asset` |
| `editor.prefab` | 有document与layout路径 | 最终pane正文仍声明业务placeholder |
| `editor.animation_sequence`、`editor.animation_graph`、`editor.ui_asset` | 有实际session/payload owner | 证明目标模式可行，也证明functional alias不是必要抽象 |

本轮按`catalog declaration -> capability filter -> registry admission -> open instance -> snapshot content resolution -> retained pane/window projection -> domain session/provider`正向阅读，再从默认layout、menu/command、Welcome、tool session、resource文档和tests反向验证。静态证据足以证明空provider也会被承认为open success；本轮没有运行Cargo、真实Editor窗口、ZUI编译、恢复布局、插件reload或性能捕获。

## 3. 必须保留的工程基础

1. 保留集中聚合40个内建描述符的入口，但把它升级为编译后的immutable catalog，而不是继续追加raw builder。
2. 保留`ViewDescriptorId`作为稳定外部身份的方向，补namespace、owner、schema、alias与retirement，不改回显示标题当identity。
3. 保留View/Window、multi-instance、dock policy、slot、document kind与constraints这些布局元数据，但让非法组合在catalog compile时失败。
4. 保留`EditorCapabilitySnapshot`的fail-closed过滤入口，改为由定义自身声明完整capability集合并绑定provider generation。
5. 保留PaneTemplate的document/payload/route/interaction四元组，升级为可编译、可链接、可版本化的typed resource contract。
6. 保留UI Asset与Animation现有真实session owner，作为首批provider迁移样板。
7. 保留布局恢复时显示Missing View的降级产品，但只用于已卸载插件、未知future ID或历史布局，不得作为已注册feature的默认内容。
8. 保留现有metadata/placement/capability测试，增加实际spawn、render、session、reload与restore资格，而不是删除已有覆盖。

## 4. 当前断路

```text
builtin descriptor builders
        |
        +--> raw-ID capability side table
        |          |
        |          +--> capability-filtered ViewRegistry
        |                         |
        |                         +--> open_descriptor() creates metadata-only ViewInstance -> Ok
        |                                                        |
        +--------------------------------------------------------+
                                                                 v
                                                raw-ID content-kind side table
                                                                 |
                                           unknown ID -> Placeholder, but placeholder=false
                                                                 |
                                      pane projection -> "Missing View"
```

目录声明、capability映射、content-kind映射、builtin shell instances、default design stack和domain session是彼此独立的事实源。新增一个ID时，编译器不会要求作者同时提供内容factory、能力闭包、资源合同、持久化迁移、localization/icon或产品测试；因此“忘记接线”仍能通过注册、菜单、布局和open测试。

## 5. P0：先关闭假功能准入

### P0-01 · Builtin catalog把无内容provider的描述符发布为可打开产品功能

`ViewDescriptor`没有content provider或spawn factory，`ViewRegistry::open_descriptor()`只创建带title/payload/host的实例就返回`Ok`。40项中19项进入Placeholder，17项又被命名和测试为functional panel/window；10个panel只有layout caller，Scene/Game Window完全无caller，Prefab/Material/Animation/Diagnostics四个window只有menu/command caller，UI Asset与Asset Browser window也只多一个Welcome入口。`resolve_view_tab()`仍把它们标成`placeholder: false`，retained pane最终显示Missing View；Prefab则直接显示“tooling is still placeholder”。

目标合同是：任何被catalog标为`Available`并允许open的定义，必须在同一compiled definition内绑定一个可验证`ViewContentProvider`/`DocumentToolkitProvider`/typed template provider，且provider必须能在当前BuildSet、capability snapshot与owner generation下创建真实内容。没有provider的项目只能是`Prototype`/`Unavailable`，不能进入默认layout、命令菜单、功能统计或产品资格。Missing View只接收历史恢复和已卸载owner，不接收当前builtin admission失败。

## 6. P1：Content、Provider 与产品真实性

### P1-01 · Descriptor本身没有内容factory/provider

`ViewDescriptor`只包含字符串和布局元数据，无法回答谁创建body、谁拥有session、失败如何回滚、何时可以close或reload。必须绑定typed provider key与owner lease。

### P1-02 · `open_descriptor()`把实例metadata创建等同于打开成功

当前成功点早于template解析、provider resolve、document/session创建和首个可呈现frame。目标API必须返回`OpenViewReceipt`，区分Rejected、Resolving、Opened、Presented、Failed与Recovered。

### P1-03 · Content kind由目录外raw-ID match二次推断

声明时传入的`ViewContentKind`只用于计算constraints，随后被丢弃；snapshot再按字符串重新猜测。Content binding必须是定义的一部分，禁止第二份match authority。

### P1-04 · 未知已注册ID默认降为Placeholder

默认分支掩盖catalog编译遗漏。已注册定义缺content binding必须在admission/compile阶段失败；只有restore resolver可显式生成`UnavailableRestoredView`。

### P1-05 · `Placeholder`内容仍发布`placeholder: false`

snapshot把内容分类和可用性标志写成互相矛盾的事实。目标snapshot必须携带typed availability、reason、owner generation和恢复来源。

### P1-06 · 十个functional panel只把“内容类型”折叠成默认尺寸

Prefab、Material、UI与Animation panel的content kind只影响constraints，没有provider、template或session binding。尺寸相似不能证明内容同语义。

### P1-07 · 十个functional panel只有默认布局consumer

Design stack能放置实例，但没有任何生产controller或toolkit消费这些ID。默认布局必须由qualified catalog生成，禁止反过来把布局引用当成功能证明。

### P1-08 · `editor.scene_game_window`是零caller孤岛

它声明DocumentKind、slot和icon，却没有目录外production引用。要么绑定真实Scene/Game toolkit，要么从shipping catalog硬删除。

### P1-09 · 四个feature window只有command/menu入口

Prefab、Material、Animation、Diagnostics window的产品链终止在`open_view`。命令可达性测试必须继续追踪到provider、body、session和domain action，不以窗口拓扑终止。

### P1-10 · UI Asset/Asset Browser window与真实tool ID并行存在

`editor.ui_asset_editor_window`并不拥有`editor.ui_asset`的session，`editor.asset_browser_window`又与`editor.asset_browser`/`editor.assets`并行。必须选择canonical definition，并以alias/migration处理旧layout ID。

### P1-11 · Prefab内容明确仍是placeholder

`editor.prefab`拥有DocumentKind、多实例与产品标题，但正文承认asset-specific tooling未实现。Editor03/44负责Prefab工具本体；Editor52负责在其资格完成前不把该ID发布为Available。

### P1-12 · Debug Observatory只是Runtime Diagnostics的别名投影

它复用相同ZUI、payload kind和route namespace，却以独立产品名出现且自身content kind仍未映射。若它是saved query/perspective，应声明typed variant；若不是，应合并ID。

### P1-13 · Material Demo与Component Lab复用UI showcase payload

Material命名没有对应Material document/compiler/preview provider，三者都投影`UiComponentShowcaseV1`。Editor15负责Material产品；本目录应把demo明确标成internal sample而非Material authoring入口。

### P1-14 · 仓内存在两套同名`ViewDescriptor`

`core/editor_extension::ViewDescriptor`与`ui/workbench::view::ViewDescriptor`字段、身份和模板语义不同。Editor50拥有extension收敛；Editor52要求builtin catalog最终消费同一compiled contribution schema，不能继续私有复制。

## 7. P1：Capability、Lifecycle 与 Reflection

### P1-15 · Required capability由ID side table补写

能力不在定义现场，新增ID会静默获得空集合。定义必须自带capability expression并在catalog compile中校验owner/provider闭包。

### P1-16 · Side table一次只能返回一个capability并覆盖原集合

`with_required_capabilities([capability])`替换vector，不能表达Animation + Native Window、UI Asset + Native Window等交集，也会覆盖未来descriptor-local声明。

### P1-17 · Capability表保留不存在的`editor.animation_timeline`

实际目录ID是`editor.animation.timeline`。这证明raw字符串side table已经漂移；compiled ID和exhaustive match必须让此类旧分支在构建期失败。

### P1-18 · Material Component Lab缺UI Asset/UI Component能力门

它加载UI template并使用UiComponentShowcase payload，却未被UI Asset capability过滤。Capability必须从provider/resource依赖闭包派生，不能靠人工标题判断。

### P1-19 · Floating window定义没有完整复合能力

UI Asset与Animation Editor Window只要求domain capability，不要求Native Window Hosting；Prefab/Material Window又只要求Native Window Hosting。placement adapter和domain provider必须分别声明并求交。

### P1-20 · Reflection把所有Activity Window声明为可floating

`activity_descriptors_from_views()`无条件`with_supports_floating_window(true)`，不读取native hosting能力或definition policy。Reflection只能投影compiled effective capability。

### P1-21 · Catalog没有owner generation与reconciliation identity

注册结果无法表达definition来自哪个builtin/plugin generation，也无法证明open期间owner仍可用。Add-only/revoke/quiesce实现继续由Editor50与性能报告负责，本报告要求catalog输出owner-qualified definition。

### P1-22 · Capability过滤只回答enabled，不回答ready/degraded/faulted

Subsystem report的bool能力无法表达provider正在加载、资源编译失败、需要重启或owner fault。目录需要typed availability state与用户可见disabled reason。

## 8. P1：Identity、Persistence 与多Authority

### P1-23 · `ViewDescriptorId::new`接受任意字符串

没有namespace、长度、字符集、owner或reserved builtin规则。应引入validated ID和compile-time builtin constants，跨插件输入走checked parser。

### P1-24 · ID没有alias、redirect与retirement contract

目录改名会直接变成Missing View。目标catalog必须发布versioned alias/redirect/tombstone，并由Editor13执行layout migration。

### P1-25 · 同一概念存在多套ID vocabulary

Assets/Asset Browser、Prefab、Animation Timeline/Graph、UI Asset和window后缀各自分叉，点号与下划线混用。需要canonical identity matrix，而不是继续增加match arm。

### P1-26 · Persistence key默认等于raw descriptor ID

没有schema version、instance payload version、migrator或owner generation。持久化key必须与显示ID解耦，并声明reader/writer支持窗口。

### P1-27 · Builtin shell instances复制descriptor metadata

instance ID、descriptor ID、title、payload和host在另一文件手写；默认layout与catalog可独立漂移。实例模板应从compiled definition和preset schema生成。

### P1-28 · Assets标题已发生可见漂移

descriptor标题是`Assets`，builtin shell instance标题是`Asset Browser`。Localized title、instance title policy和document-derived title必须各有明确owner。

### P1-29 · Activity reflection再次复制标题、icon与host能力

Reflection生成另一套ActivityView/Window DTO并自行推断document/floating支持。它应只携带compiled catalog generation上的只读句柄或稳定投影。

### P1-30 · Builder允许非法字段组合静默成立

任意kind可配任意slot、document kind、pane/window template、multi-instance与dock policy；缺字段还会获得看似合理的默认值。Catalog compiler必须实施kind-specific schema和互斥/必需约束。

## 9. P1：Template、Localization、Icon 与资格

### P1-31 · Template引用是未验证的raw `res://`字符串

本轮16/16路径存在，但不存在并不等于唯一document root、正确schema或runtime可编译。Definition必须引用typed asset identity及其artifact/schema generation。

### P1-32 · 没有全目录template link/compile门

现有测试检查selected metadata或document ID非空，没有对40项逐个验证资源存在、parse、root、binding、payload、route和interaction compatibility。

### P1-33 · Payload kind与route namespace是中心化closed enum

新增领域工具必须修改Workbench中心枚举和pane projection，插件无法提供自有typed payload adapter。应通过versioned provider schema注册，核心只保留稳定envelope。

### P1-34 · Template、payload、route与data source没有同代receipt

资源更新、provider reload和layout restore可分别来自不同generation。一次open/present必须绑定CatalogGeneration、TemplateArtifactId、ProviderGeneration和PayloadSchemaVersion。

### P1-35 · 所有标题都是Rust或ZUI中的硬编码英文

没有LocalizationKey、fallback culture、locale revision或运行时刷新合同。Editor12/33拥有i18n平台；Editor52要求catalog只发布localized handle而非复制String。

### P1-36 · Icon key是未验证字符串

没有icon catalog resolve、fallback policy、theme/scale/high-contrast variant或missing诊断。Editor23拥有icon产品；catalog compile必须验证引用并保留typed handle。

### P1-37 · Definition没有产品成熟度与disabled reason

Demo、prototype、internal diagnostic和shipping authoring tool在同一列表中。必须声明visibility channel、maturity、feature flag、unsupported reason和release qualification。

### P1-38 · Catalog没有immutable generation/digest/BuildSet identity

调用方无法证明menu、layout、snapshot和provider看到同一目录。目标输出是原子发布的`CompiledViewCatalogSnapshot`，包含definition digest、BuildSet、owner generations与diagnostics。

### P1-39 · 现有测试主要证明metadata和host拓扑

25个聚焦测试能证明template字段、slot、多实例和capability toggle，但functional window测试在floating/exclusive host创建后即结束，没有证明实际body或工具命令。

### P1-40 · 没有40项exhaustive产品资格矩阵

缺少唯一ID、content provider、capability closure、resource compile、icon/i18n、open-present-close、save/restore、disable/reload和negative admission的全量参数化测试。

## 10. P2：一致性与长期维护

### P2-01 · ID命名混用点号、下划线与`_window`后缀

命名本身不致命，但会放大alias、menu route和持久化错误；迁移后应冻结一套namespace规范。

### P2-02 · 每个小描述符独立文件却没有生成catalog索引

文件拆分清晰，但重复builder boilerplate无法自动证明目录闭包。保留模块边界，同时由definition source生成索引与测试向量。

### P2-03 · 新描述符默认进入Authoring preset

遗漏preset声明也会进入默认产品面。Shipping visibility应显式声明，默认应是internal/unavailable。

### P2-04 · 默认slot与dock policy掩盖遗漏

`DocumentCenter`和`DrawerOrDocument`让未完成定义看起来可用。Kind-specific constructor不应提供会改变产品语义的宽松默认值。

### P2-05 · Registry列表顺序来自HashMap values

`list_descriptors()`没有稳定排序，菜单/reflection顺序可随进程变化。Catalog snapshot应按显式category/order/stable ID排序。

### P2-06 · Descriptor/list投影反复拥有String与clone

这会增加大catalog的分配与复制；具体性能修复由既有Workbench性能报告拥有，目标catalog应提供Arc-backed immutable handles。

### P2-07 · Definition没有category、search keyword与menu order

大型工程编辑器需要可扩展的Workspace Menu结构，不能长期依赖调用方按title或ID组织。

### P2-08 · Definition没有键盘、无障碍与focus入口元数据

Open后首焦点、accessible name/description和命令shortcut没有统一合同。Runtime UI负责语义执行，catalog负责声明与验证。

### P2-09 · Definition没有diagnostic source jump

Missing provider/resource/icon时，operator只能看到泛化Missing View。诊断应携带definition owner、source locator、generation和failure code。

### P2-10 · 没有per-definition open/failure/present telemetry

无法区分目录不可达、provider失败、模板失败或首帧失败，也无法建立功能使用和性能基线。

### P2-11 · 没有deprecation与移除预算

旧ID只能永久保留match arm或突然变Missing View。需要deprecated-since、replacement、support window和tombstone expiry。

### P2-12 · 同一ZUI可被多个产品名复用但没有资源owner声明

复用本身合理，但必须标明shared template contract、允许的payload variants和变更影响面，避免Debug/Material/UI命名掩盖真实owner。

## 11. 参考引擎差异与适用边界

| 参考 | 可直接采用的工程合同 | Zircon当前差距 | 不应误抄的部分 |
|---|---|---|---|
| Unreal `FWorkflowTabFactory` / `FTabManager` | Stable tab ID、localized `FText` label/tooltip、icon、singleton、`CanSpawnTab`、真实spawn delegate、register/unregister和workspace group在同一工厂链闭合 | Zircon descriptor没有factory/can-spawn，label/icon/capability/content分散在side table | 不复制Slate宏与shared-pointer形态；采用provider-bound spawn/admission语义 |
| Godot `EditorDockManager` / `EditorPlugin` | Plugin直接提交真实`EditorDock`/`Control`，可add/remove、enable/disable、open/close、floating restore，并有state/save hooks | Zircon可只提交字符串descriptor并打开空instance，owner state/remove合同在目录中缺失 | 不复制Node/Object继承模型；采用真实内容对象和layout key/state owner |
| Fyrox Material/Animation plugins | 插件按需创建并持有真实editor，处理start/sync/UI message/close/destroy和layout恢复 | Zircon已有Animation/UI Asset局部类似，但functional alias没有owner | Fyrox成熟度较低，只取tool owner与MVC/lifecycle证据，不作性能上限 |
| Bevy `Plugin` | `build/ready/finish/cleanup`、stable name和uniqueness把声明与生命周期状态分开 | Zircon capability bool不能表达provider ready/finish/cleanup | Bevy不是Editor docking参考，只取composition/lifecycle contract |
| Unity Graphics `VolumeComponentProvider`与Editor asmdef | 按当前render pipeline筛选typed component，排除已存在项，并以editor-only/version/feature define限定可用集合 | Zircon目录没有typed feature-conditioned provider closure | 本地Graphics仓不是Unity完整Editor shell，不推断闭源窗口系统 |

参考源码说明成熟引擎的“目录”不是标题列表：Unreal的spawner必须绑定spawn/can-spawn，Godot插件提交真实Control并能移除，Fyrox插件拥有实际editor对象。Zircon目标也不能停在更复杂的descriptor DTO；必须把definition、provider、owner lifecycle和产品资格闭合。

## 12. 目标架构

### 12.1 Canonical definition

```rust
pub struct BuiltinViewDefinition {
    pub id: QualifiedViewId,
    pub owner: ViewOwnerId,
    pub kind: ViewDefinitionKind,
    pub localization: LocalizedViewMetadata,
    pub icon: IconAssetRef,
    pub provider: ViewContentProviderRef,
    pub required_capabilities: CapabilityExpression,
    pub placement: ViewPlacementPolicy,
    pub instance_policy: ViewInstancePolicy,
    pub document: Option<DocumentViewContract>,
    pub template: Option<ViewTemplateContract>,
    pub persistence: ViewPersistenceContract,
    pub aliases: Vec<ViewIdAlias>,
    pub visibility: ProductVisibility,
    pub qualification: ViewQualificationPolicy,
}
```

这只是合同形状，不要求用一个巨型struct堆积所有optional字段。实现应采用kind-specific definition enum/typed builders，使Activity View、Document Toolkit、Exclusive Page、Native Window和Restored Unavailable View各自只有合法字段。

### 12.2 Catalog compiler

1. 收集builtin与extension definition proposal，绑定owner/provider generation。
2. 验证ID唯一、namespace、alias DAG、retirement、document kind与instance/persistence policy。
3. 解析capability expression，验证domain provider、native host、template/runtime adapter依赖闭包。
4. resolve并编译template，检查唯一root、schema、payload、route、interaction、icon和localization key。
5. 拒绝shipping definition中的placeholder/no-op/fallback-only provider。
6. 生成稳定排序的immutable `CompiledViewCatalogSnapshot`与diagnostics，原子发布generation/digest/BuildSet。
7. 从同一snapshot生成menu、activity reflection、default preset、restore resolver和全量测试向量。

### 12.3 Open transaction

`OpenViewRequest`必须携带catalog generation、qualified descriptor ID、project/document/session identity、host preference和principal/context。Resolver先验证availability与capability，再取得provider lease、创建tool/document session、resolve template artifact、创建实例并提交layout；任何失败在可见发布前回滚。成功返回的receipt至少绑定instance ID、provider generation、template artifact、layout revision和first-present generation。

### 12.4 Placeholder边界

Placeholder只允许两类来源：历史layout中的未知/retired ID，以及owner在恢复期间不可用但仍保留用户布局位置的definition。它必须显示typed reason和恢复动作，不得共享`Available`状态，也不能被menu/default preset主动创建。

### 12.5 Domain切分

Prefab、Material、Animation、UI Asset、Diagnostics与Asset Browser各自由对应domain report/toolkit实现内容provider。Builtin catalog只编排稳定definition和provider binding，不吸收domain compiler、transaction、preview或runtime逻辑；反过来domain也不能再通过新建raw window ID绕过catalog资格。

## 13. Owner边界

| Owner | 唯一负责 | Editor52只消费/约束 |
|---|---|---|
| Editor03 / 44 | Prefab source/instance/override/toolkit | provider达到资格前保持Unavailable，不把placeholder冒充功能 |
| Editor14 | Animation document/graph/timeline/compiler/preview | 只保留`animation_sequence`/`animation_graph`等canonical provider binding |
| Editor15 | Material/VFX document/compiler/preview | Demo/Lab不得冒充Material authoring，真实provider完成后接入 |
| Editor23 | UI Asset、theme/icon/a11y/font产品 | UI Asset definition、icon handle与template依赖闭包 |
| Editor25 | Diagnostics/Timeline/Telemetry产品事实 | Debug Observatory variant与Diagnostics definition是否真实可用 |
| Editor12 / 33 | Settings locale与Localization平台 | catalog发布LocalizationKey与locale-aware snapshot |
| Editor13 | Layout schema、restore、migration与unknown plugin placeholder | alias/tombstone和persistence contract输入 |
| Editor50 / 06 | Extension contribution、plugin lifecycle、reconcile/revoke | provider owner/generation与compiled proposal |
| Workbench性能报告 | descriptor/instance clone、generation、registry/session authority成本 | immutable handle和同代catalog形状 |
| Editor52 | builtin definition compiler、provider binding、capability/resource/localization闭包、shipping资格矩阵 | 不复制各domain工具实现 |

## 14. 分层重构里程碑

### M0 · Truth quarantine

冻结40项matrix；把17个functional placeholder、Prefab placeholder及无法证明provider的入口标成Prototype/Unavailable，从default preset和shipping menu移除。保留restore-only placeholder。

### M1 · Typed definition与catalog compiler

引入qualified ID、kind-specific definition、owner/capability/persistence/alias/visibility合同；从同一source生成content binding、reflection、menu和preset，删除capability/content raw-ID side table。

### M2 · Provider与open transaction

建立`ViewContentProvider`/`DocumentToolkitProvider` resolve、lease、spawn、close和first-present receipt；`open_view`不再在metadata实例创建时返回产品成功。

### M3 · Canonical domain cutover

先迁移已有真实UI Asset、Animation、Hierarchy/Inspector/Scene/Assets/Console provider，再由Editor03/15/25接入Prefab、Material和Diagnostics。每个概念只保留一个canonical ID，旧ID走alias/migration。

### M4 · Capability、extension与reload

把domain、native window、resource/runtime adapter表达成capability expression；与Editor50的owner generation、reconcile、revoke、quiesce闭合，验证disable/reload/open race。

### M5 · Resource、localization、icon与persistence

Catalog compile ZUI root/schema/payload/route，resolve icon与LocalizationKey；Editor13迁移layout/payload schema并验证locale/theme/scale改变不破坏instance identity。

### M6 · Qualification与规模

执行40项矩阵、真实窗口、多project/document、restore/reload/fault injection、accessibility与性能测试；只有BuildSet-bound receipt通过的definition可进入Shipping/Default。

## 15. 产品资格门

| Gate | 通过条件 |
|---|---|
| G01 | 40项definition ID唯一，builtin namespace、字符集、长度与owner全部验证 |
| G02 | 旧ID alias/redirect图无环，retired ID有replacement/support window |
| G03 | 每个Shipping definition绑定非placeholder typed provider |
| G04 | Provider owner/generation与catalog generation同代且可取得lease |
| G05 | `open_view`在provider/template/session失败时返回typed failure，不创建可见成功实例 |
| G06 | Open成功receipt绑定instance、provider、template、layout与first-present generation |
| G07 | 17个functional placeholder在provider完成前不可从shipping menu/default preset打开 |
| G08 | Prefab placeholder在真实toolkit资格前明确Unavailable |
| G09 | 19个当前unmatched ID不再依赖content-kind默认分支 |
| G10 | `Placeholder`内容绝不同时发布`placeholder: false`/Available |
| G11 | Capability expression支持多能力交集、替代和typed disabled reason |
| G12 | UI Asset/Animation floating window同时要求domain与native hosting能力 |
| G13 | Reflection的floating/document/exclusive支持来自effective compiled policy |
| G14 | Plugin disable/reload后旧provider不能再spawn，已有实例按policy close/migrate/quiesce |
| G15 | 所有16份当前模板及未来模板通过path、parse、root、schema与artifact identity检查 |
| G16 | Pane payload、route namespace、interaction mode与data source schema兼容 |
| G17 | Template/provider更新不能形成跨generation混合present |
| G18 | Icon key全部resolve，覆盖theme、scale、high contrast与missing fallback |
| G19 | Title/tooltip/category使用LocalizationKey，locale切换保持instance/persistence identity |
| G20 | Demo/Internal/Prototype/Shipping visibility互斥且默认fail closed |
| G21 | Menu、activity reflection、preset与restore resolver由同一catalog snapshot生成 |
| G22 | Descriptor listing顺序在进程、平台与Hash seed变化下稳定 |
| G23 | Persistence key、payload schema和migrator不再隐式等于raw descriptor ID |
| G24 | Unknown/retired/plugin-missing layout恢复为typed unavailable pane且不丢布局位置 |
| G25 | `editor.ui_asset`、Animation、Asset Browser等canonical ID没有并行tool authority |
| G26 | 每个definition具备source jump、owner、diagnostic code和failure telemetry |
| G27 | 40项逐一通过register-open-present-interact-close测试或明确negative admission |
| G28 | 单例、多实例、document、exclusive page、floating window各有真实产品测试 |
| G29 | Project切换、layout restore、locale/theme变化与plugin reload不复活stale instance |
| G30 | 无障碍name/role/focus、键盘入口和reduced-motion/scale适配通过产品检查 |
| G31 | 1K extension definitions下catalog compile、query、menu和restore满足预算且无每帧全量clone |
| G32 | Qualification receipt绑定source fingerprint、BuildSet、catalog digest、资源artifact与测试结果 |

## 16. 缺失测试矩阵

1. 参数化遍历40项definition，断言ID、owner、kind schema、capability closure、provider和visibility。
2. 对当前19个unmatched ID建立negative regression，迁移后禁止重新落入generic Placeholder。
3. 对17个functional ID验证Unavailable或真实tool body，不能只验证host位置。
4. 对16份ZUI执行真实compiler/linker，检查root、payload schema、route、binding和data source。
5. 对UI Asset/Animation/Prefab/Material/Diagnostics验证domain + native host复合能力。
6. 对capability disable、plugin revoke/reload与open并发做generation/lease/fault injection。
7. 对旧ID、unknown plugin、future payload与corrupt layout执行restore/migration，保证用户布局不丢且不伪装Available。
8. 对locale、theme、DPI、high contrast和icon missing执行snapshot/pixel/accessibility检查。
9. 对open transaction在provider resolve、template compile、session create、layout commit与first present各阶段注入失败。
10. 对1K/10K extension definition执行compile/query/list/restore规模测试，并检查稳定排序、内存和clone预算。

## 17. 禁止的临时修补

1. 禁止只给19个ID继续补`descriptor_content_kind` match arm；这会保留第二authority。
2. 禁止给functional descriptor随便挂一份generic ZUI就宣布工具完成。
3. 禁止把`Placeholder`改成空白页、Space或成功字符串隐藏Missing View。
4. 禁止新增`*_window`/点号/下划线alias而不提供迁移和canonical owner。
5. 禁止继续按ID side table补capability，或把多个能力压成一个bool。
6. 禁止以资源文件存在、窗口可创建、菜单可点击或test通过证明domain工具完成。
7. 禁止把domain compiler/document/preview塞进builtin catalog以绕开Editor03/14/15/23/25 owner。
8. 禁止把restore-only placeholder删除；应把它与shipping feature admission严格分型。
9. 禁止在没有真实scale workload和receipt时宣称性能优于Unreal。

## 18. 状态与产出记录

本轮完成29个`builtin_views`文件的逐文件静态审查、40项catalog闭包、5份聚焦测试、16份资源存在性、产品ID caller和14份参考源码对照；新增本专项报告及索引/coverage/owner总账。未修改production/tests，未运行Cargo、GUI、ZUI编译、插件reload、layout restore、pixel/accessibility或性能测试。

静态review完成不表示重构完成。P0-01必须先通过M0把假Available入口降级，再按M1-M6建立compiled catalog与provider-bound open transaction；在G01-G32全部形成BuildSet-bound receipt前，不得把Builtin View/Window目录作为“Editor工具已完成”的产品证据。
