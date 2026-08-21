---
related_code:
  - zircon_plugins/editor_build_export_desktop
  - zircon_plugins/native_window_hosting
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_editor/src/core/commands/contribution.rs
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/ui/host/editor_subsystems.rs
  - zircon_editor/src/ui/host/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard
  - zircon_editor/src/ui/retained_host/app/build_export_wizard_session
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
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
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: false
---

# 03 · Desktop Export、Native Window Hosting、Source/Dist 等价与产品 Provider 工程化差距

## 1. 结论

`editor_build_export_desktop`和`native_window_hosting`都有真实代码，但当前包身份与实际产品身份不一致。Desktop Export真正可运行的pipeline plan、process runner、job/cancellation、panel session、retained projection和默认Workbench入口全部位于`zircon_editor`核心；插件主要再声明一次view、七个operation、三份report template和一个export profile asset。Native Window真正的winit窗口、presenter、floating projection、close/input/redraw lifecycle也位于Editor retained host；插件没有实现window backend/provider，只声明两个已经由core拥有的view和一份不存在的`authoring.zui`。

更关键的是，两个包的两种交付路径都不成立。App的first-party Editor catalog只链接Navigation和Neural，不会把这两个source plugin registration装入产品；两份native projection又都声明`extensions: []`，dist behavior没有command/event/bridge/host-ready回调，所以native loader只能得到空`SerializedContributionBatch`。CI能够编译两份cdylib并验证manifest/ABI指针，却没有验证安装、选择、materialize、打开surface、执行operation、禁用和卸载后的产品行为。因此，“包存在、manifest在catalog中、dist能编译”没有形成“功能可用”的证明。

当前最危险的不是功能少，而是capability被当成实现证明。`editor.extension.native_window_hosting`在Editor配置缺省时由core直接默认启用，即使项目没有选择或安装该包，core窗口也会出现；反过来选择dist包仍得不到任何插件贡献。Desktop Export core instance又无条件出现在builtin shell，完全不需要`editor.extension.build_export_desktop`。于是安装/禁用包、capability snapshot、可见UI和实际provider分别由不同owner决定，Plugin Manager无法对用户陈述真实状态。

本轮登记 **4项P0、48项P1、10项P2**。推荐的硬收敛不是继续补manifest字段，而是先决定唯一产品owner：保留通用window host与export orchestration作为core service；由插件提供可验证的feature provider、typed operation adapter和资源包，或者删除伪插件身份并明确它们是内建Editor模块。source与dist必须从同一canonical contribution/behavior定义生成并通过等价测试。未完成这一步前，不应把这两个包计入独立可安装、可禁用、可分发的第一方插件能力。

## 2. 审查边界与证据

### 2.1 物理范围

| 集合 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Desktop Export包全量 | 16 / 1,563 / 65,713 | E3逐manifest、Cargo、7个Rust、5个ZUI与profile template；7个test attributes、0 ignored |
| Native Window Hosting包全量 | 9 / 476 / 17,299 | E3逐manifest、Cargo、6个Rust；4个test attributes、0 ignored；包内0个ZUI |
| 两包Rust | 13 / 1,535 / 62,633 | E3逐source/dist declaration、registration和tests |
| Core产品实现 | Editor export wizard、retained window host、command registry、native contribution materializer | E3逐descriptor -> command -> session/job，以及capability -> view -> native presenter链 |
| 产品装配 | App first-party Editor catalog、project selection、native dist load | E3确认source未链接、dist空贡献、builtin功能独立存在 |

指纹按相对路径排序，对每个文件计算SHA-256，再对`path + space + hash + LF`清单计算SHA-256。Desktop Export为`2912aa92874f418dff23f92dad9966bd04ba83f02d6a759daa2482c0bd1940e3`，Native Window Hosting为`99d287d61669091a07b8703cf2c99551a4cfdc07d9e8e5eeea75b11283c68d6f`，合并范围为`227b7732a5ba9d27e563c0f5e2a9678a067ef1b16cb61fd50210b5d7bc45cab5`。成文时`git status --short --`两包为空，因此`source_recheck_required: false`；这只说明本轮证据未与在途修改冲突，不表示实现完成。

### 2.2 纵向调用链

本轮实际闭合了以下链路：

1. `plugin.toml` -> Rust `PluginDeclaration` -> source `package_manifest()` -> builtin metadata catalog；
2. project selection -> App first-party Editor catalog -> `EditorPluginRegistrationReport` -> extension registry；
3. native candidate -> V3 editor entry -> registration manifest schema -> `SerializedContributionBatch` -> Editor native materialization；
4. Desktop plugin menu/asset toolkit/inspector binding -> command descriptor -> event或operation factory -> export wizard session/job；
5. builtin Desktop Export view -> retained panel buttons -> core session -> Python/Rust export pipeline；
6. Native Window capability -> builtin view filtering -> floating target/presenter -> winit lifecycle；
7. package enable/disable预期 -> capability snapshot -> visible views/commands/resources -> unload预期。

结果不是“代码完全没有”，而是第2、3、4、6、7条链由互不一致的owner切断。尤其需要区分：`EditorPluginCatalog::builtin()`从manifest投影metadata，不会自动执行插件的`register_editor_extensions()`；native entry存在registration manifest指针，也不等于manifest中含有任何editor contribution。

### 2.3 已有基础

以下实现值得保留，不应在重构时退回临时脚本：

- Desktop Export core已有八阶段typed plan、artifact handoff、bounded stdout/stderr tail、process-tree cancellation、Editor job system、panel state和retained projection。
- 插件的五份ZUI可被loader解析，report control ID与descriptor有静态一致性测试。
- native loader已能验证editor contribution batch schema、package ID，并以candidate registry materialize失败回滚本批贡献。
- Native Window core已有typed view/window identity、winit窗口映射、presenter reconcile、close/input/resize/redraw与测试基础。
- `native_window_hosting` dist使用declaration-only依赖，避免把整个Editor静态链接进cdylib；这是正确方向。

这些基础不能抵消产品装配缺失，但可以作为M0-M3收敛的实现底座。

## 3. P0：当前能力声明与产品行为直接矛盾

### PLUGIN-DESKTOP-WINDOW-P0-001 · Source与dist都无法把两包贡献装入默认Editor产品

`zircon_plugins/first_party_editor_catalog`只有Navigation与Neural两个可选依赖和registration分支；App只委托该catalog。两个source crate虽能在单元测试中手工调用`plugin_registration()`，默认产品没有调用者。两份`declare_plugin!`的native projection均为`systems: []`、`events: []`、`extensions: []`；dist behavior又无command/event/bridge/host-ready回调。native loader仅从匹配editor contribution schema的`SerializedContributionBatch`物化extension，因此选中dist最多得到包metadata、capability与空registry，不会得到source注册的view、menu、template、asset type或inspector。

必须建立一个canonical Editor contribution artifact，由同一声明同时驱动source registration和dist serialized batch；无法序列化的行为必须通过版本化bridge/command provider显式提供。安装测试必须从真实包目录和project selection开始，证明两个交付形态产生等价的extension inventory与行为receipt。

### PLUGIN-DESKTOP-WINDOW-P0-002 · Native Window capability由core默认伪造，与package/provider无关

`editor_subsystems.rs`把`editor.extension.native_window_hosting`列入optional subsystem，但配置缺省时把全部optional subsystem默认启用。`builtin_view_descriptors()`随后用该字符串放行Workbench、Prefab、Prefab Editor和Material Editor窗口。这里没有检查package selection、native entry load、window backend初始化或provider health。即使包完全不存在，capability也可为enabled；选择一个空dist也不会增加native行为。

必须拆分`host.window.native.v1`这类宿主授予能力与`editor.feature.native_window_hosting`这类package提供能力。后者只能由成功admit并初始化的provider发布，并携带provider ID/generation/health；provider失效时应先完成window/document close barrier，再原子撤销能力与贡献。配置只能请求feature，不能凭字符串制造provider存在事实。

### PLUGIN-DESKTOP-WINDOW-P0-003 · Desktop Export插件发布七个不可执行operation和一个无业务owner的profile资产

插件用`EditorCommandDescriptor::operation`注册Generate Plan、Source Template、Library Embed、Native Dynamic、Diagnostics、Create Profile、Open Profile七个operation，但没有为任何一个调用`register_operation`或注册`OperationCommandFactoryRegistration`，也没有`.with_event(...)`。command registry对这种descriptor的event dispatch返回`OperationRequiresInvocation`，而全仓精确反查没有对应factory。`build.export_profile`、`ExportProfileController`和六个profile control ID在包外也没有production consumer；模板名为`windows-release`却写`mode = "debug"`，plugins/features为空。Report template与summary keys同样没有包外controller，真正core panel使用自己的generic ReportBody projection。

必须把`.zpreset`或新的versioned `DesktopExportProfile`确定为唯一source schema，注册Create/Open/Validate/Build operation factory，并由factory生成source-bound `ResolvedExportRequest`、提交job、返回typed receipt。profile inspector必须编辑真实document transaction而不是显示硬编码PropertyRow。若core wizard继续是产品owner，应删除这七个重复operation和第二套asset/report authority，由插件只注册desktop target provider。

### PLUGIN-DESKTOP-WINDOW-P0-004 · Native Window source surface引用物理不存在的ZUI，dist则连该surface也不发布

source plugin注册`plugins://native_window_hosting/editor/authoring.zui`，但9个包文件中没有任何`.zui`，两个可能的物理路径均不存在。当前测试只断言template descriptor存在，从不加载资源。与此同时dist serialized contribution为空，因此动态安装时甚至没有机会报告该资源缺失。结果是linked source路径会在延迟materialize时失败，native dist路径则静默缺少所有surface，两条路径没有一个满足manifest描述的“Optional editor integration for native floating window surfaces”。

必须先决定该插件是否需要authoring UI。若只是平台provider，应删除drawer/template和对core view的重复注册，改为注册可探测的window backend/provider；若确有工具面，则资源必须打入content manifest，loader需在enable事务的prepare阶段验证所有URI、schema、imports和controller，失败时整个package保持Faulted且零贡献。

## 4. P1：工程级完整性差距

### 4.1 Package owner、composition与capability

1. manifest catalog、source registration、native registration和core builtin共有至少四份feature inventory，没有唯一authority。
2. `EditorPluginCatalog::builtin()`测试只比较静态manifest metadata，容易被误读为插件实现已进入产品。
3. App feature gate以`first-party-navigation-editor-plugin`决定整个first-party Editor catalog函数，即使只希望Neural也存在错误耦合。
4. 两个包都没有App feature、Cargo dependency或generated catalog marker，新增source plugin需手工改catalog，scaffold无法保证产品可达。
5. package selection只决定registration报告收集，不证明required host service、resource bundle和behavior provider完整。
6. capability只有字符串集合，没有provided/requested distinction、provider identity、version、generation或health。
7. core optional subsystem与plugin capability共用相同namespace，配置、宿主服务和包功能三种语义混在一起。
8. disabled capability只过滤部分view/command；已有session、window、job、resource/controller的撤销顺序没有package transaction。
9. Plugin Manager无法解释功能来自core builtin还是selected package，也不能展示source/dist差异。
10. 没有“manifest declared -> linked/discovered -> admitted -> initialized -> contributed -> healthy”的统一状态机和receipt。

### 4.2 Desktop Export插件边界

11. core无条件创建`editor.build_export_desktop#1`，不要求插件capability；卸载插件不会关闭或隐藏产品入口。
12. core已经注册`view.build_export.open`，插件又为同一view生成`view.editor.build_export_desktop.open`，形成两套command identity。
13. 插件把core的`EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID`作为自己的template ID，owner namespace与resource owner不一致。
14. source插件surface指向private `asset://` panel，core builtin descriptor则指向`res://ui/editor/host/build_export_desktop_body.zui`，同一view有两份template authority。
15. plugin crate大量`pub use zircon_editor::*`，包API实质暴露core内部wizard surface；Editor重构会成为插件breaking change。
16. dist直接依赖完整editor crate且未采用declaration feature，cdylib会携带不必要的Editor dependency graph，和native_window_hosting的轻量模式不一致。
17. dist诊断明确说wizard仍由editor plugin module托管，但dist并不加载或调用该module的registration函数。
18. `is_stateless=true`与零callback只描述ABI壳，不提供profile、job、report或controller行为。
19.三种report view的`required_stage`和control ID只做结构断言，没有真实report artifact -> typed decoder -> template binding集成测试。
20. report template中的列表/诊断区域是`Space`，插件没有provider决定分页、虚拟化、选择、复制、打开路径或错误状态。
21. SourceTemplate report的“Cargo”、LibraryEmbed的feature matrix、NativeDynamic的package list没有数据schema与空/partial/truncated状态。
22. `PIPELINE_REPORT_PATH`与每个stage `report_path`都固定为`report.json`，descriptor没有artifact identity、run ID、profile fingerprint或stage attempt。
23. `stage_progress_kinds()`只有Pending/Running/Passed/Fatal，缺Skipped、Cancelled、Retrying、Blocked、Stale和Partial。
24. `ExportWizardAction::OpenReport`与`RunStage`是descriptor enum，但插件中没有执行路由；多数stage primary action统一`RunStage`也没有per-stage policy。
25. profile template不是`.zpreset`，没有schema/version/migration/unknown field policy，和Tooling03的preset authority并行。
26. `windows-release`模板实际为debug，测试还固定断言该矛盾，错误已被测试制度化。
27. profile drawer全部字段为硬编码值；未发现load/save/dirty/undo/validation/platform compatibility controller。
28. inspector customization列出四个binding，但对应operation factory缺失；UI显示可操作能力却无执行者。
29. asset type只有icon thumbnail，没有source document codec、dependency scanner、reference fixer、cook policy或artifact mapping。
30. package option`editor.build_export_desktop.default_platform`未发现consumer，改变设置不会影响template、wizard或resolved request。

### 4.3 Native Window Hosting插件边界

31. 插件名称声称hosting，source实现只调用通用`register_authoring_extensions`，没有window service、presenter factory或platform adapter。
32. source注册的Workbench/Prefab view ID已由core builtin拥有；若真正合并registration会产生duplicate descriptor/command风险。
33. plugin operation ID为`view.editor.workbench_window.open`和`view.editor.prefab.open`，core使用`view.prefab.open`等另一套ID，keymap/menu/automation identity分裂。
34. Workbench window core没有对应默认`view.workbench_window.open`命令，插件生成的路径只在手工source registration中存在。
35. package没有asset/content root，却声明`plugins://`资源；即使补文件，也缺明确打包/解析root合同。
36. `base_package_manifest()`未调用`.with_sdk_api_version(zircon_plugin_sdk::SDK_API_VERSION)`，programmatic source manifest默认为`0.1.0`，静态`plugin.toml`为`0.2.0`。
37. tests不比较source/static manifest的SDK version，因此允许同一package ID产生两个兼容性身份。
38. native dist只证明descriptor和behavior指针非空，不断言decoded contribution count、resource availability或window provider health。
39. dist `required_capabilities`请求的正是core默认制造的同名capability，形成“host先声称feature存在，plugin才请求feature”的循环契约。
40. `is_stateless=true`没有覆盖活跃window、focused surface、pending close、monitor/DPI与presenter generation；真实feature显然有状态。
41. unload callback为空，无法在DLL卸载前撤销window callbacks、销毁native handles或等待render/presenter quiescence。
42. `on_host_ready`为空，无法验证event loop线程、window backend、render surface factory和platform restrictions。
43. 没有headless/remote/Wayland/X11/macOS main-thread capability协商，manifest仅以platform枚举粗略宣称支持。
44. core window缺口已由Editor13拥有；本包没有把那些placement/DPI/monitor/lifecycle改进映射成provider contract。

### 4.4 Distribution、测试、CI与文档

45. 两个dist测试只查ABI header和非空registration pointer，不解析manifest确认contributions为空或与source等价。
46. CI standalone job只`cargo check/build` cdylib；不启动Editor、安装包、选择package、打开view、执行command或卸载。
47. plugin workspace tests手工调用source registration，无法覆盖默认App不链接该source crate这一事实。
48. package tests检查descriptor“存在”，不检查每个command有event/factory、每个template URI可通过产品resolver加载、每个controller可解析。

## 5. P2：长期产品化与生态差距

1. 缺少source/dist/plugin-version跨版本兼容矩阵与golden contribution inventory。
2. 缺少第三方desktop target/window backend provider SDK示例，当前只能复制第一方内部crate耦合。
3. 缺少package安装体积、load latency、enable transaction和window/export overhead预算。
4. 缺少signed package/resource hash与实际loaded contribution receipt的关联展示。
5. 缺少export profile/report的localization、accessibility、keyboard-only和长路径/长诊断布局资格。
6. 缺少多项目、多Editor实例、并发export和package update时的identity/isolation模型。
7. 缺少window backend crash/restart、GPU device loss、monitor hotplug与plugin reload组合故障注入。
8. 缺少export provider的remote build/worker capability、artifact provenance与可恢复resume协议。
9. 缺少deprecation/migration策略来收敛重复command/view/template/profile IDs。
10. 缺少用户可导出的“为什么此功能可见/不可用”的provider graph与admission证据包。

## 6. 参考引擎对照

| 参考 | 可借鉴责任边界 | Zircon应吸收 | 不应误推 |
|---|---|---|---|
| Unreal Project Launcher / Packaging Settings | profile/model、build/cook/package/launch状态与Editor UI分层；SlateApplication拥有native window创建、父子/modal、销毁与平台窗口映射 | export request/model与UI分离；window host service是平台owner，feature module只消费稳定service | 不复制Legacy Project Launcher页面体量，也不把Unreal现有实现视为所有可靠性问题都已解决 |
| Godot ExportPreset / EditorExportPlatform | preset与platform exporter是typed owner，平台可校验options/features并报告warning/error；WindowWrapper明确处理embedded/native切换和screen placement | target provider必须真实消费preset并产出validation/build receipt；window wrapper必须由可探测backend支撑 | Godot的单进程module模型不能直接证明Zircon native DLL热卸载安全 |
| Bevy WinitPlugin / WinitWindows | Plugin显式安装event-loop/window backend；entity与winit WindowId映射、scale/monitor/window lifecycle由backend资源维护 | 区分host service capability与feature capability，provider安装成功才发布能力 | Bevy没有完整Editor plugin manager或商业export pipeline，不能补齐authoring结论 |
| Fyrox Build Tools / Editor export | export options真实驱动target、asset copy/conversion、build和run-after-build；window settings保存position/size/maximized | profile字段必须有consumer；可运行验证与产物复制是独立阶段 | Fyrox实现较轻，不是可靠性/安全/分布式构建上限 |
| Unity Graphics BuildProcessors | package在build preprocess中收集active settings、validate/strip资源，并有lifecycle/dispose边界 | rendering/plugin provider应把build-time validation与实际active configuration绑定 | checkout不含Unity Editor核心window与Player build实现，不从中推断完整插件/窗口系统 |

本轮对照强调owner和可交付链，不比较UI外观，也不以代码量判断成熟度。要“超过当前Unreal”，首先必须能证明package selection确实改变provider graph，source/dist语义等价，功能禁用可撤销，export artifact和window生命周期有可复现receipt；当前还没有资格进入性能或体验优越性比较。

## 7. 目标架构

### 7.1 Capability与provider分层

建立三个不可混用的层次：

- `HostServiceCapability`：App/platform真实提供的服务，例如native window event loop、render surface、process runner、filesystem sandbox。
- `PackageFeatureCapability`：包成功初始化后提供的用户功能，例如desktop target provider或native floating-window feature。
- `RequestedCapability`：package希望host授予的权限，不能反向作为自身已实现功能的证据。

每项provided capability都关联`provider_package_id + module_id + version + generation + health + receipt`。Capability snapshot从已admit provider图生成，不从配置字符串直接生成；配置只进入request policy。

### 7.2 Canonical source/dist贡献

定义版本化`EditorContributionBundle`：serializable descriptor贡献与behavior endpoints分开，但由同一Rust declaration/build step生成。Source路径materialize该bundle后绑定in-process behavior；dist路径解码相同bundle后绑定ABI bridge behavior。CI对两者做canonical inventory diff，并逐operation验证`event XOR factory/bridge`恰有一个执行owner。

资源进入`PackageContentManifest`，记录logical URI、relative path、kind、schema、hash、imports与controller requirement。Enable先在detached registry验证全部贡献、资源和behavior，再单次commit；任何缺失都使package Faulted且不发布capability。

### 7.3 Desktop Export边界

`zircon_editor`只保留通用`ExportWorkbenchService`、job/cancel/progress、typed report viewer和host process policy；`zircon_tooling`拥有canonical `ResolvedExportRequest`与build/cook/pack/bundle实现。Desktop插件只拥有：

- desktop target/platform provider与支持矩阵；
- profile schema adapter和migration；
- Create/Open/Validate/Build operation factory；
- desktop-specific option editor和typed report renderer；
- provider health/preflight，例如toolchain、template、signing和host artifact资格。

删除core/plugin重复view和command。若Desktop Export决定永久内建，则反向删除package capability与dist，明确它是Editor builtin，不保留“可独立分发”的假象。

### 7.4 Native Window边界

App/platform层拥有`NativeWindowHostService`，Editor layout层消费logical window requests；插件若保留，必须注册一个`NativeWindowFeatureProvider`，把window kind/policy映射到host service并提供initialize/quiesce/unload。Core view descriptor仍由其真实业务owner持有，window插件不重复注册Workbench/Prefab内容view。

provider enable/disable需执行`prepare -> dirty close decision -> create/reparent/destroy staging -> commit -> receipt`。active callback和native handle必须带generation；unload前撤销callback、停止新窗口、等待present/render完成并销毁handles。Editor13继续拥有placement、DPI、monitor和workspace schema，本报告只拥有package/provider接线。

## 8. 重构路线

### M0 · 关闭假能力与建立失败fixture

- 默认App集成测试证明当前两个source registration不可达、dist贡献为空、native capability无包仍enabled、Desktop view无包仍存在。
- 暂时在Plugin Manager中把两包标为`metadata-only / unavailable`，禁止显示为healthy installed feature。
- 为缺失ZUI、无factory operation、source/static SDK drift和空dist contribution建立失败测试。

### M1 · 决定唯一owner并硬切重复身份

- 对Desktop Export选择“core framework + desktop provider plugin”或“完整builtin”之一；禁止双owner。
- 对Native Window拆分host service与feature provider，移除core默认伪造的package capability。
- 建立command/view/template/profile ID迁移表，旧ID只允许有期限redirect，不并行写入新状态。

### M2 · Canonical contribution与资源包

- 从同一declaration生成source registry bundle和dist serialized bundle。
- 引入PackageContentManifest、hash、prepare-time loader与controller validation。
- 修复Native Window SDK version drift，所有programmatic/static/generated manifest逐字段等价。

### M3 · 真实behavior provider

- Desktop operations绑定typed factories和job receipt；profile收敛到canonical preset/source document。
- Native provider绑定initialize/window service/quiesce/unload，并公开健康诊断。
- bridge API采用versioned typed request/result、deadline、cancellation、generation和bounded diagnostics。

### M4 · 产品enable/disable/update事务

- selection、load、capability publish、extension commit、resource mount成为单一事务。
- disable/update先做dirty/session/window/job barrier，失败保持旧generation完整可用。
- Plugin Manager展示provider graph、来源形态、版本、健康、贡献inventory与最近receipt。

### M5 · Source/dist等价与产品资格

- Windows/Linux/macOS分别运行source与dist安装/打开/执行/关闭/禁用/重载矩阵。
- export使用真实最小项目、真实profile与artifact oracle；window使用真实OS window、DPI/monitor/input/close oracle。
- CI归档contribution diff、provider receipt、截图/窗口树、artifact hashes、process tree和unload结果。

## 9. 验收门

1. 未选择package时，不发布该package的feature capability、view、command、asset type、template或provider。
2. 仅修改Editor config字符串不能制造一个不存在的package provider。
3. 选择source package后，App产品可追踪到唯一registration owner。
4. 选择dist package后，native loader物化非空、schema合法且package ID匹配的contribution bundle。
5. source与dist的canonical descriptor inventory逐字段相同；允许差异必须有显式平台/形态reason code。
6. 每个operation恰有一个event、in-process factory或ABI bridge owner；零个和多个都在enable前失败。
7. 每个template/default document/controller在enable prepare阶段可解析、hash匹配且依赖闭包完整。
8. Native Window包缺失当前`authoring.zui`时稳定进入Faulted，零贡献、零capability发布。
9. `native_window_hosting` static/source/generated manifest的SDK API version一致为当前SDK版本。
10. Desktop Export view/command只有一个canonical ID和一个owner，不再有core/plugin重复入口。
11. Native Window插件不再重复拥有Workbench/Prefab业务view；它只拥有provider合同。
12. Desktop profile名称、mode、platform、strategy、plugins、features和asset filter全部进入resolved request或被validation拒绝。
13. `windows-release`默认不再生成debug请求；profile UI读取真实document而非硬编码字符串。
14. Create/Open/Generate/Build/Diagnostics操作在interactive与headless允许面均返回typed receipt。
15. Report view按run ID/profile fingerprint/stage attempt读取typed artifact，旧report不会污染新run。
16. Pending/Running/Passed/Fatal之外，Cancelled/Skipped/Blocked/Stale/Partial语义在job、UI和report一致。
17. Desktop package disable会撤销其operations/profile/provider，同时通用Editor export framework保持一致状态。
18. Native provider enable只有在event loop、window backend和render surface preflight成功后提交。
19. Native provider disable对dirty documents走统一save/discard/cancel barrier，cancel时零副作用。
20. unload前所有native callback与handle完成generation revoke、quiesce和destroy，DLL可安全释放。
21. provider crash/device loss/monitor hotplug时capability health和用户诊断与真实状态一致。
22. package update失败时旧provider generation、窗口、profile和操作完整可用。
23. Plugin Manager能展示Declared/Discovered/Admitted/Initialized/Contributed/Healthy/Faulted各阶段证据。
24. CI不仅编译cdylib，还从package目录完成install/select/materialize/open/invoke/disable/unload。
25. Windows source/dist真实窗口矩阵通过创建、focus、input、resize、DPI move、close和restore。
26. Linux X11/Wayland与macOS main-thread限制有真实required lane或明确unsupported admission。
27. Desktop export最小项目真实产生可启动host和有效pack，不接受placeholder artifact。
28. cancellation终止完整process tree，保留bounded日志和Cancelled receipt，不误报Fatal/Passed。
29. report/profile/resource/diagnostic输入均有bytes/count/depth/path/latency budget。
30. 100次enable/disable/reload不增长window handles、threads、callbacks、jobs或mounted resource generations。
31. command palette/menu/asset toolkit/inspector/automation从同一canonical operation catalog投影。
32. required CI归档source/dist contribution diff、provider graph、artifact/window oracle和失败注入报告；静态descriptor测试不得代替产品通过。

## 10. 实施边界与交叉计划

- Plugin01继续拥有通用SDK、native ABI、loader admission、签名/hash和dist behavior框架；本篇只拥有两个具体包的接线与等价性。
- Tooling03继续拥有preset/build/cook/pack/platform bundle语义和artifact真实性；本篇拥有Desktop插件如何消费并呈现该owner。
- Editor08拥有通用command/factory/keymap/automation机制；本篇拥有七个Desktop operation的具体执行owner。
- Editor13拥有window layout/placement/DPI/monitor/restore；本篇拥有Native Window package是否真实提供provider以及enable/unload事务。
- Editor06拥有Plugin Manager通用状态与live reload UX；本篇提供这两个包必须暴露的provider graph和receipt。

本轮只完成review与重构计划，没有修改production、tests、manifest、ZUI或CI。没有重复运行已知不可达的`zircon_editor --lib`动态lane：现有基线仍在617.2秒后被239个既有test-build错误和122个warning阻断，因此本报告不声明Cargo或产品测试通过。两包源码范围在成文时干净，实施应从M0失败fixture和owner决策开始，而不是继续增加descriptor或空dist metadata。
