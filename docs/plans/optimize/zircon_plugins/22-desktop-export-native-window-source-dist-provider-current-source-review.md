---
related_code:
  - zircon_plugins/editor_build_export_desktop
  - zircon_plugins/native_window_hosting
  - zircon_plugins/first_party_editor_catalog
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_plugins/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_editor/src/core/commands/contribution.rs
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/ui/host/editor_subsystems.rs
  - zircon_editor/src/ui/host/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration
  - zircon_editor/src/ui/retained_host/app/build_export_actions.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session.rs
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/03-desktop-export-native-window-source-dist-provider-integration-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/21-plugin-artifact-marketplace-third-party-package-install-update-trust-non-cargo-product-integration-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Developer/LegacyProjectLauncher/Private/Models/LegacyProjectLauncherModel.h
  - dev/UnrealEngine/Engine/Source/Developer/LegacyProjectLauncher/Private/Widgets/SLegacyProjectLauncher.cpp
  - dev/UnrealEngine/Engine/Source/Developer/DeveloperToolSettings/Classes/Settings/ProjectPackagingSettings.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/SlateApplication.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/godot/editor/export/editor_export_preset.h
  - dev/godot/editor/export/editor_export_platform.h
  - dev/godot/editor/export/editor_export.cpp
  - dev/godot/editor/gui/window_wrapper.cpp
  - dev/bevy/crates/bevy_winit/src/lib.rs
  - dev/bevy/crates/bevy_winit/src/winit_windows.rs
  - dev/Fyrox/editor/src/export/mod.rs
  - dev/Fyrox/fyrox-build-tools/src/export/mod.rs
  - dev/Fyrox/fyrox-build-tools/src/export/pc.rs
  - dev/Fyrox/editor/src/settings/windows.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/BuildProcessors/CoreBuildData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/BuildProcessors/CorePreprocessBuild.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/BuildProcessors/HDRPBuildDataValidator.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
supersedes_currentness_of:
  - docs/plans/optimize/zircon_plugins/03-desktop-export-native-window-source-dist-provider-integration-review.md
source_recheck_required: true
---

# 22 · Desktop Export / Native Window Source-Dist-Provider 当前源码复核

## 1. 结论

本轮不是重新做一次关键词抽样，而是复核两包全部24个tracked文件，并沿first-party Editor catalog、App feature装配、command/operation registry、native contribution materializer、Editor两套导出执行链、builtin view/capability和Runtime native ABI冻结128个当前源码文件。五套参考引擎另冻结18个owner/lifecycle文件。旧Plugins03的62项finding逐项保留编号并按当前working tree重判为：**56 Open、0 Partial、6 Closed**。

当前确有两项进展。第一，`native_window_hosting` source插件已经删除重复的Workbench/Prefab view、operation、drawer、template和不存在的`plugins://native_window_hosting/editor/authoring.zui`；它现在只声明package/capability identity，因此旧P0-004及P1-032至035关闭。第二，App新增独立`first-party-editor-catalog` feature，不再以Navigation feature决定整个Editor catalog调用，因此P1-003关闭。

这些修正没有让两包成为真实产品provider。first-party Editor catalog仍只链接Navigation和Neural，Desktop Export与Native Window均不可由标准source产品装配；两份dist的native projection仍是`extensions: []`，没有command/event/bridge/host-ready/unload行为。native materializer又只检查entry存在且未faulted，不要求贡献非空或行为完整，因此空包可以得到Loaded/Enabled表象。Native Window capability仍由Editor core配置缺省时直接启用，插件本身没有event-loop/window-backend/presenter provider。

Desktop Export core比插件壳完整得多，但核心内部也尚未收敛为单一authority。一条链是`ExportWizardPipelinePlan -> ExportWizardPanelSession -> ProcessCommandRunner`，拥有typed stage、streaming output、bounded tail、cancel和retained projection；另一条链是`DesktopExportJobQueue -> DesktopExportEditorJob -> EditorManager::execute_native_aware_export_build...`，拥有独立profile、queue、progress、report和cancel。插件再声明第三套7个operation、profile asset与report template，却没有任何operation factory或产品controller接入前两条链。当前问题已经从“临时功能少”升级为“同一能力存在三个互不约束的事实源”。

因此本轮不建议继续补空manifest或UI descriptor。必须先硬切唯一产品owner：Editor只保留通用authoring/session/job/window host service；插件要么提供可验证、可启停、source/dist等价的target/window feature provider，要么删除伪插件身份并明确为builtin。Tooling优化按用户要求排除；本文只定义Editor/Plugin边界所需的typed request/provider/receipt合同，不修改或扩展Tooling计划。

## 2. 审查边界与可复验证据

### 2.1 当前树冻结

| 集合 | 文件 / 行 / bytes | SHA-256 | 证据 |
|---|---:|---|---|
| Desktop Export package全量 | 16 / 1,563 / 65,713 | `2912aa92874f418dff23f92dad9966bd04ba83f02d6a759daa2482c0bd1940e3` | E3：manifest、Cargo、source/dist Rust、5份ZUI、profile template与tests逐文件 |
| Native Window Hosting package全量 | 8 / 438 / 16,205 | `172f373927a40abdd4bd4ad0187d6407e4b4c133a3a70553a672f03fdfd9dd31` | E3：manifest、Cargo、source/dist Rust与tests逐文件 |
| 产品装配与command边界 | 10 / 1,730 / 62,651 | `414c203045a41e6ef7d8f49d88d5fe131aebeaa86c74a7ee1e3105ccfdcfb5cd` | E3：catalog、App feature/caller、editor_support、command descriptor/factory registry |
| Core消费者与实现 | 94 / 12,972 / 455,827 | `0a8b202e96c1fda47b96f3f4b2d205903a627ce47ae75302ae1bb46808c18f3d` | E3：export_build、native_registration、build_export_actions、wizard session；E2：builtin window/capability与ABI边界 |
| Zircon focused union | 128 / 16,703 / 600,396 | `bee1d97099418489be87a293104705df2d441ba43ea40431fd56b0ecd6abdfe6` | 上述集合按tracked path去重 |
| Unreal参考 | 5 / 11,973 / 457,552 | 见reference union | launcher model、packaging settings与Slate window owner |
| Godot参考 | 4 / 1,646 / 64,166 | 见reference union | export preset/platform与WindowWrapper |
| Bevy参考 | 2 / 848 / 32,284 | 见reference union | WinitPlugin与entity/WindowId owner |
| Fyrox参考 | 4 / 1,016 / 39,106 | 见reference union | Editor export、build-tools与window settings |
| Unity Graphics参考 | 3 / 224 / 9,093 | 见reference union | build data、preprocess lifecycle与HDRP validation |
| Five-engine reference union | 18 / 15,707 / 602,201 | `1158823f2ba74d2158356faae7a335b4a5ff888892175867dec264369943f508` | 只用于owner、validation、lifecycle和交付链证据 |

指纹算法为：按workspace相对路径ordinal排序，对每个文件计算lowercase SHA-256，再对`path + space + hash + LF`的UTF-8清单计算SHA-256。观察基线为`79f64878f3b9526517644c055ad3bf5cadfccd0f`，观察日期为2026-08-24。

### 2.2 旧报告与同提交修正的时间偏差

Plugins03首次报告提交为`08094b9b9e17f6c80372e15c17b01204038b305b`。该提交相对父提交同时落入Native Window phantom authoring删除、App独立Editor catalog feature和native registration test改进，但报告正文与旧Native包指纹描述的是修正前快照；该提交至当前HEAD，两包和所选装配路径没有committed diff。因此本篇以当前物理源码为准关闭6项，而不是沿用旧文档的时间切片结论。

当前focused union存在其他Session/用户的未提交修改：`zircon_app/Cargo.toml`、`zircon_plugins/Cargo.toml`、command registry、export build manager/wizard execution、native registration manager与Runtime native ABI。本文审读并冻结的是这些文件的当前bytes，不把它们冒充已集成提交；任何实现开始前必须重新取指纹，所以`source_recheck_required: true`。两包自身当前干净。

### 2.3 本轮证据边界

本轮只修改review与索引，没有修改production、tests、Cargo、manifest、ZUI或CI。没有运行Cargo、真实Editor、DLL加载、窗口、导出产物、跨平台、fault、soak或benchmark；因此Closed只表示对应静态根因已经从当前源码移除，不代表两包产品资格通过。当前大工作树同时有大量并行改动，动态验证会混入无关结果，且本专项目标明确为review-first。

## 3. 可保留的工程底座

1. Export wizard已有typed stage plan、artifact handoff、缺输入preflight、streaming stdout/stderr delta、bounded output tail、process-tree cancellation、Editor JobSystem和retained projection。
2. `DesktopExportJobQueue`已有串行admission、queued/active cancellation、typed job failure保留、progress projection与terminal summary；其问题是与wizard并行，不是完全没有工程结构。
3. Export preset/profile读取能从project manifest、`.zpreset`、target mode和platform生成部分typed options；应收敛为一个resolved request，而不是退回脚本字符串。
4. native contribution materialization使用candidate registry，package ID mismatch或materialize失败不会直接污染live registry；这是未来transactional enable的底座。
5. Native Window core已有真实winit host、window/presenter/floating projection链；应把其稳定能力暴露为host service，而不是复制到插件。
6. Native Window source已主动删除core-owned authoring surface，证明硬删除重复owner可行，不需要compat shim。

## 4. 当前源码纵向事实

### 4.1 Source产品装配仍不可达

`zircon_app`现在可独立启用`first-party-editor-catalog`，但`zircon_plugins/first_party_editor_catalog`仍只有Navigation与Neural两个可选dependency/registration branch。Desktop和Native两个source crate没有App feature、dependency或catalog row。`EditorPluginCatalog::builtin()`从静态manifest生成metadata也不会自动执行`register_editor_extensions()`。所以“manifest可被扫描”与“source behavior进入产品”仍是两件事。

### 4.2 Dist空贡献仍可伪装成功

两份dist都从`native_projection`生成registration manifest，而projection的`extensions`为空；behavior table没有command/event/bridge方法，也没有`on_host_ready`、state restore/save或unload工作。`NativeEditorContributionMaterialization::is_registration_usable`只判断entry存在且没有fault，并不要求非空贡献、required behavior endpoint或resource closure；registration projection又把已加载包投影为Loaded/Enabled。结果是“ABI指针存在 + 空batch”可以成为成功外观。

### 4.3 Desktop存在三套事实源

1. 插件source注册7个`EditorCommandDescriptor::operation`，没有`.with_event`，也没有`register_operation`或`OperationCommandFactoryRegistration`。registry会返回`OperationRequiresInvocation`，全仓生产反查没有这些ID的factory。
2. core wizard链拥有`ExportWizardPipelinePlan`、`ExportWizardPanelSession`与process runner，UI按钮使用`workbench.build_export.*` action ID。
3. retained `build_export_actions`另有`DesktopExportJobQueue`，通过`EditorManager::execute_native_aware_export_build_with_cancellation_and_progress`执行，并维护独立profile、progress、report与cancel。

插件的`build.export_profile`、`ExportProfileController`、`editor.build_export_desktop.default_platform`和三份report template没有接入任一core execution owner。profile template仍名为`windows-release`但`mode = "debug"`；drawer显示硬编码值，report业务区域仍以`Space`占位。core builtin又无条件创建`editor.build_export_desktop#1`，所以安装、禁用或卸载插件不会改变真实导出功能。

### 4.4 Native Window仍只是identity package

source registration现在正确地不再重复发布Workbench/Prefab authoring surface，但也没有注册window service、backend factory、presenter adapter、thread/main-loop requirement或lifecycle callback。`editor_subsystems.rs`仍把`editor.extension.native_window_hosting`作为optional subsystem，并在配置缺省时启用全部optional subsystem；builtin view过滤随后把字符串当作窗口能力事实。包缺失时core窗口仍可见，包存在时也没有新增provider。

`base_package_manifest()`仍未写入当前`SDK_API_VERSION`，source programmatic manifest与静态`plugin.toml`保持两个兼容性事实源。dist继续请求同一个由core预先伪造的capability，并宣称stateless；它无法对活跃window、DPI/monitor、pending close、render surface和DLL quiescence负责。

## 5. 旧Plugins03台账逐项重判

### 5.1 P0

| ID | 状态 | 当前源码证据与关闭条件 |
|---|---|---|
| `PLUGIN-DESKTOP-WINDOW-P0-001` | Open | source catalog仍不链接两包；dist仍为空contribution/behavior，且空batch可被判usable。必须由真实package selection生成非空、可执行、source/dist等价的贡献receipt。 |
| `PLUGIN-DESKTOP-WINDOW-P0-002` | Open | Native Window capability仍由core缺省配置制造，与admitted provider无关。必须区分host service、requested permission和package-provided feature。 |
| `PLUGIN-DESKTOP-WINDOW-P0-003` | Open | 7个Desktop operation仍无event/factory/bridge；profile asset、option和report controller仍无产品consumer。必须接入唯一resolved request/job/report owner或删除重复声明。 |
| `PLUGIN-DESKTOP-WINDOW-P0-004` | **Closed** | 当前source registration返回`Ok(())`且不再发布重复view/operation/drawer/template；缺失的`authoring.zui`引用与`extension_ids.rs`已删除，并有针对phantom authoring的回归测试。 |

### 5.2 P1

| ID | 状态 | 当前判定 |
|---|---|---|
| P1-001 | Open | manifest、source、native与core builtin仍是多份feature inventory。 |
| P1-002 | Open | builtin catalog测试仍只能证明metadata，不证明behavior装配。 |
| P1-003 | **Closed** | App已用独立`first-party-editor-catalog` gate调用catalog，Navigation/Neural各自为子feature。 |
| P1-004 | Open | Desktop/Native仍没有App dependency、feature或catalog marker。 |
| P1-005 | Open | selection仍不证明host service、resource与behavior closure。 |
| P1-006 | Open | capability仍缺provided/requested、provider/version/generation/health。 |
| P1-007 | Open | config、host service与package feature仍共用字符串namespace。 |
| P1-008 | Open | disable没有session/window/job/resource/controller撤销事务。 |
| P1-009 | Open | Plugin Manager仍不能解释core/source/dist真实owner。 |
| P1-010 | Open | 缺统一Declared到Healthy状态机；当前空贡献还能成为usable/Loaded/Enabled。 |
| P1-011 | Open | core仍无条件创建Desktop Export实例；当前还并存两套core execution state machine。 |
| P1-012 | Open | core/plugin command identity仍分裂，core两套action/job identity也未收敛。 |
| P1-013 | Open | 插件继续借用core template document ID，owner namespace不闭合。 |
| P1-014 | Open | plugin `asset://` panel与core `res://` body仍是两份template authority。 |
| P1-015 | Open | plugin crate继续暴露大量Editor内部wizard API。 |
| P1-016 | Open | Desktop dist仍依赖完整Editor，未收敛到declaration-only carrier。 |
| P1-017 | Open | dist诊断仍声称source module托管wizard，但不会执行source registration。 |
| P1-018 | Open | stateless空callback仍不提供profile/job/report/controller行为。 |
| P1-019 | Open | report结构测试没有artifact decoder到插件template/controller的产品集成。 |
| P1-020 | Open | report列表/诊断区域仍为`Space`且无分页、虚拟化、复制或路径动作owner。 |
| P1-021 | Open | 三类report没有完整typed data/empty/partial/truncated schema。 |
| P1-022 | Open | report path仍缺run ID、profile fingerprint和stage attempt identity。 |
| P1-023 | Open | 插件progress kind仍缺Skipped/Cancelled/Retrying/Blocked/Stale/Partial。 |
| P1-024 | Open | `OpenReport`/`RunStage` descriptor仍无插件执行路由。 |
| P1-025 | Open | 插件TOML profile、core `.zpreset`、manifest profile及两套core profile构造仍并行。 |
| P1-026 | Open | `windows-release`仍固定为debug，测试继续制度化该矛盾。 |
| P1-027 | Open | drawer仍是硬编码显示，无document transaction/load/save/dirty/undo。 |
| P1-028 | Open | inspector binding对应operation factory仍缺失。 |
| P1-029 | Open | profile asset type仍缺codec、migration、dependency/reference/cook/artifact合同。 |
| P1-030 | Open | `default_platform`仍无production consumer。 |
| P1-031 | Open | Native Window package现在明确只做identity，仍没有hosting provider。 |
| P1-032 | **Closed** | source不再重复注册core-owned Workbench/Prefab view。 |
| P1-033 | **Closed** | 两个分裂的plugin open-operation ID已随phantom authoring删除。 |
| P1-034 | **Closed** | 不再发布无core counterpart的Workbench open command。 |
| P1-035 | **Closed** | 不再声明无content root的`plugins://native_window_hosting/...`资源。 |
| P1-036 | Open | programmatic manifest仍未调用`.with_sdk_api_version(SDK_API_VERSION)`。 |
| P1-037 | Open | tests仍未锁定static/source/generated SDK版本等价。 |
| P1-038 | Open | dist测试仍不验证非空contribution、resource或provider health。 |
| P1-039 | Open | dist仍请求由core同名字符串预造的feature capability。 |
| P1-040 | Open | stateless声明与真实window/monitor/DPI/close/presenter状态矛盾。 |
| P1-041 | Open | unload仍不能撤销callback、handle并等待presenter/render quiescence。 |
| P1-042 | Open | `on_host_ready`仍不验证event loop线程/backend/surface/platform限制。 |
| P1-043 | Open | X11/Wayland/macOS main-thread/headless/remote协商仍缺失。 |
| P1-044 | Open | Editor13窗口改进仍未映射为本包provider contract。 |
| P1-045 | Open | 两个dist测试仍只验证ABI/指针外形，不做decoded source-dist parity。 |
| P1-046 | Open | CI仍未完成install/select/materialize/open/invoke/disable/unload产品链。 |
| P1-047 | Open | source tests仍主要手工调用registration，不能证明默认App可达。 |
| P1-048 | Open | package tests仍偏descriptor/source-shape，不验证产品resolver、factory和controller。 |

P1合计：**43 Open、0 Partial、5 Closed**。当前core wizard/cancellation等进展是可保留底座，但没有直接关闭旧P1的package/provider条件，因此不以“代码更多”误标Partial。

### 5.3 P2

| ID | 状态 | 当前判定 |
|---|---|---|
| P2-001 | Open | 无source/dist/plugin-version兼容矩阵与golden inventory。 |
| P2-002 | Open | 无第三方desktop target/window backend provider SDK示例。 |
| P2-003 | Open | 无安装体积、load latency、enable transaction与运行开销预算。 |
| P2-004 | Open | 无signed artifact/resource hash到loaded contribution receipt关联。 |
| P2-005 | Open | 无profile/report的l10n、a11y、keyboard-only与长内容资格。 |
| P2-006 | Open | 无多项目、多Editor、并发export与package update隔离模型。 |
| P2-007 | Open | 无window crash/device loss/hotplug/reload组合故障注入。 |
| P2-008 | Open | 无remote build/worker、artifact provenance与resume协议。 |
| P2-009 | Open | 无重复command/view/template/profile ID迁移与退役策略。 |
| P2-010 | Open | 无可导出的provider graph/admission证据包。 |

P2合计：**10 Open、0 Partial、0 Closed**。全台账合计：4 P0 + 48 P1 + 10 P2 = 62项，其中**56 Open、0 Partial、6 Closed**。

## 6. 五套参考引擎的责任边界

| 参考 | 本地源码直接证明 | Zircon必须吸收 | 不能外推 |
|---|---|---|---|
| Unreal | `FProjectLauncherModel`持有launcher/profile manager并清理delegate；Packaging Settings用typed build/cook/package配置；`FSlateApplication`拥有AddWindow、modal、destroy和platform window lifecycle。 | Export model/request与UI分离；native window service由host owner，feature只能消费稳定服务；生命周期必须有明确撤销。 | Legacy Project Launcher体量和UE现状不自动等于最优性能或无缺陷。 |
| Godot | `EditorExportPreset`/`EditorExportPlatform`拥有typed options、validation、warning/error与export；`WindowWrapper`处理embedded/native切换、rect/input/restore。 | 平台provider必须真实消费preset并产出validation/build receipt；window wrapper要绑定可探测backend与状态。 | 单进程module模型不能证明Zircon DLL热卸载安全。 |
| Bevy | `WinitPlugin`显式安装window backend；`WinitWindows`维护entity与WindowId映射、创建/销毁、monitor和scale。 | provider安装成功后才能发布feature；host service与package feature必须分层。 | Bevy不提供完整Editor plugin manager或商业export pipeline。 |
| Fyrox | Editor export options进入build-tools的copy/build/run；window settings持久化position/size/maximized。 | 每个profile字段必须有consumer；export、copy、build、run和restore需要独立可验证阶段。 | 较轻实现不是安全、分布式构建或大规模资格上限。 |
| Unity Graphics | build preprocess创建并持有CoreBuildData，pre/post有dispose边界；HDRP validator按platform/settings/version拒绝不合格配置。 | build-time active configuration、validation与lifecycle owner必须绑定同一build generation。 | Graphics checkout不含完整Unity Editor/window/player build，不能补造缺失证据。 |

参考源码共同点不是“类更多”，而是每项能力有真实owner、typed输入、admission、lifecycle和产物/状态证据。Zircon要超过Unreal，首先必须能证明选择某个package会原子改变同一provider graph，并且source/dist、enable/disable、export artifact和window状态可复现；当前尚未达到比较性能与表现优越性的前置条件。

## 7. 目标架构与硬切规则

### 7.1 单一Plugin Composition事实

建立`EditorPluginCompositionPlan`，唯一输入是resolved project/plugin selection与verified artifact generation；唯一输出包含package lifecycle、provided/requested capability、descriptor bundle、resource closure、behavior endpoint和health receipt。App不得按手写feature分支另造source inventory，Editor不得从config字符串另造package feature。

source与dist从同一个versioned `EditorContributionBundle`生成。serializable descriptor与in-process/ABI behavior可以有不同carrier，但每个operation必须在admission前证明恰有一个event、factory或bridge owner。空contribution只有显式`identity_only` package role才允许成功，且绝不能发布业务feature capability。

### 7.2 Desktop Export唯一执行authority

在Editor内合并`ExportWizardPanelSession`和`DesktopExportJobQueue`的重复plan/job/progress/cancel/report状态。保留一个`ExportAuthoringService`，输入versioned `ResolvedExportRequest`，输出generation-qualified job/progress/artifact/terminal receipt。UI、palette、automation和headless只投影同一operation catalog。

Desktop插件若保留，应只拥有desktop target/platform provider、profile adapter/migration、platform preflight与desktop-specific option/report renderer；通用process/job/report viewer留在Editor。插件现有7个operation必须接入唯一service，或者硬删除。若决定功能永久builtin，则删除package capability、dist和重复asset/template身份，不留兼容wrapper。

### 7.3 Native Window的Host Service与Feature分离

App/platform发布`host.window.native.v1`，内容包括event-loop lane、window factory、surface factory、platform restrictions和health；Native Window feature provider成功admit/init后才发布`editor.feature.native_window_hosting.v1`。配置只能请求功能，不能发布功能。

disable/unload事务顺序固定为：stop admission -> dirty document decision -> close/focus barrier -> presenter/render quiesce -> revoke generation callbacks -> destroy native handles -> withdraw contributions/capability -> unload artifact。任何阶段失败保留旧generation或进入Faulted，不允许“capability已撤销但window仍活着”的半状态。

### 7.4 禁止兼容壳

本专项采用hard cutover：删除旧command/view/template/profile/capability ID及其调用点，不增加`pub use`别名、compat module、shim trait或双写。迁移记录应提供old -> new映射和一次性资产/设置迁移，但运行时只接受canonical schema。

## 8. 重构里程碑

### M0 · RED证据与owner决策

1. 增加默认App选择Desktop/Native source却无registration的失败测试。
2. 增加两个dist解码后贡献为空却被判usable/Loaded/Enabled的失败测试。
3. 增加7个Desktop operation无factory、Native capability无provider仍可见的失败测试。
4. 记录并决定Desktop是builtin还是独立package；决定Native包是identity package退役还是完整feature provider。

### M1 · Composition与source/dist同源

1. 建立typed package role、provided/requested capability和lifecycle receipt。
2. 从一个contribution bundle生成source registration与dist serialized batch。
3. App消费generated/compiled composition plan，删除手写包分支。
4. non-empty/resource/behavior closure在candidate registry阶段fail-closed。

### M2 · Desktop authority收敛

1. 合并两套core export job/session状态机，建立唯一request/job/report identity。
2. 选定canonical preset/profile schema并提供migration；删除debug/release矛盾。
3. 接通或删除7个operation、profile drawer与三份report template。
4. disable/cancel/restart/旧report隔离全部基于generation receipt。

### M3 · Native Window provider化

1. App发布真实host window service；plugin只在preflight成功后发布feature。
2. 增加event-loop lane、backend/presenter factory、DPI/monitor/restore状态与health。
3. 实现dirty close、quiesce、generation revoke和unload事务。
4. 明确Windows、X11、Wayland、macOS、headless与remote admission矩阵。

### M4 · 产品与故障资格

1. source/dist分别从真实package目录install/select/materialize/open/invoke/disable/unload。
2. 最小项目导出真实可启动host和有效pack，验证取消、失败、恢复与artifact provenance。
3. 运行100次enable/disable/reload，检查window handle、thread、callback、job和resource generation泄漏。
4. 归档provider graph、contribution diff、window/export oracle、fault与性能证据。

## 9. 验收门

1. 未选择package时，该package的feature、view、command、asset、template和provider全部不存在。
2. config字符串不能制造provider；每个provided capability可追踪到package/module/version/generation/health。
3. source与dist canonical inventory逐字段相等，允许差异必须有typed reason code。
4. identity-only package不发布业务feature；业务package的空contribution在admission阶段失败。
5. 每个operation恰有一个event、factory或bridge owner，并能从产品入口返回typed receipt。
6. Desktop只有一个profile/request/job/progress/cancel/report authority和一组canonical ID。
7. profile每个字段进入resolved request或被validation拒绝；不存在显示可编辑但无consumer的设置。
8. report按run ID、profile fingerprint、stage attempt与artifact hash读取，旧generation不能污染当前UI。
9. Native feature只在event loop、backend、surface和platform preflight成功后提交。
10. disable/unload完成dirty-document barrier、callback revoke、presenter quiesce和handle destroy。
11. Plugin Manager可解释Declared/Discovered/Admitted/Initialized/Contributed/Healthy/Faulted每阶段证据。
12. Windows/X11/Wayland/macOS/headless/remote矩阵有真实通过或明确unsupported admission。
13. export最小产品可启动、pack可验证、取消终止完整process tree且terminal state不误报。
14. 输入、日志、report、resource、window数量与lifecycle均有bytes/count/depth/time预算。
15. required CI归档source/dist diff、provider graph、artifact/window oracle和故障注入报告；source-shape测试不得代替产品通过。

## 10. Owner与实施边界

- Plugins01继续拥有通用SDK/native ABI/loader admission/dist behavior；本文拥有这两个具体package的非空、等价与provider真实性。
- Plugins06继续拥有first-party catalog/profile closure；本文要求Desktop/Native进入唯一composition plan或显式退役。
- Plugins21拥有artifact/install/trust父控制面；本文只消费verified package generation，不重复Marketplace或安装器设计。
- Editor08拥有通用command/factory/automation；本文拥有7个Desktop operation的具体执行owner。
- Editor13拥有通用window placement/DPI/monitor/restore；本文拥有Native package provider、enable/disable与unload事务。
- Editor06拥有Plugin Manager通用状态与UX；本文提供这两个包必须输出的provider graph和receipt。
- Export build/cook/pack的未来Rust语义owner不在本轮展开；Tooling优化与迁移按用户要求暂缓。

本报告是review-only integration candidate。生产修正必须从M0的失败证据与owner决策开始，不得继续增加空dist、重复descriptor或临时UI占位来制造“功能已存在”的外观。
