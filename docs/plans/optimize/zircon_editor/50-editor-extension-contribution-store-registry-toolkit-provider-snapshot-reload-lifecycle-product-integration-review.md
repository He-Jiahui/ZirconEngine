---
related_code:
  - zircon_editor/src/core/extension
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editor_extension
  - zircon_editor/src/core/plugin
  - zircon_editor/src/scene/modes
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_scene_modes.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/extension_access.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/run_config.rs
  - zircon_editor/src/ui/workbench/shell_state.rs
  - zircon_app/src/entry/entry_runner/editor.rs
tests:
  - zircon_editor/src/core/extension/store/tests.rs
  - zircon_editor/src/core/extension/toolkit/tests
  - zircon_editor/src/core/plugin/manager/tests
  - zircon_editor/src/scene/modes/tests.rs
  - zircon_editor/src/tests/editor_plugin_sdk.rs
  - zircon_editor/src/tests/editor_event/runtime/extensions_registration
  - zircon_editor/src/tests/host/manager/document_toolkit_lifecycle.rs
plan_sources:
  - docs/zircon_editor/core/plugin.md
  - docs/plans/performance/01/2026-07-30-editor-core-editor-extension-current-review.md
  - docs/plans/performance/01/2026-08-15-editor-extension-contribution-overlay-current-architecture-review.md
  - docs/plans/performance/01/2026-08-16-editor-core-plugin-catalog-lifecycle-current-architecture-review.md
  - docs/plans/zircon_editor/editor/02/failure-2026-08-01-plugin-registration-runtime-consumer-atomicity.md
  - docs/plans/zircon_editor/editor/06/failure-2026-07-28-plugin-contribution-ticket-revoke-contract.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Features/IModularFeatures.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Features/ModularFeatures.cpp
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Public/ToolMenus.h
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Public/ToolMenuOwner.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/Toolkits/AssetEditorToolkit.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/godot/editor/plugins/editor_plugin.h
  - dev/godot/editor/editor_node.cpp
  - dev/godot/editor/editor_data.cpp
  - dev/Fyrox/editor/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Tools/MaterialUpgrader/MaterialUpgraderRegistry.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/ICoreRenderPipelinePreferencesProvider.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 50 · Editor Extension / Contribution Store / Registry / Toolkit / Provider / Snapshot / Reload / Lifecycle 产品集成工程化差距

## 1. 结论

Zircon Editor已经具备一批值得保留的扩展基础：`EditorExtensionRegistry`和`ContributionBatch`能描述view、menu、Inspector、template、asset、scene mode、overlay、graph、timeline、command与operation factory；`ContributionStore`用immutable `Arc<ContributionSnapshot>`发布、按ticket撤销并维护有界change journal；view/template/asset type能追溯部分source；scene mode和viewport overlay已有panic boundary、fault状态与context checkpoint；`DocumentToolkitRegistry`提供checked document/generation分配、save/close lease和锁外save I/O。当前代码不是“什么都没有”。

但这些组件尚未形成一个真实的Editor extension runtime。最严重的问题是两套激活权威彼此断开：`EditorPluginManagerSnapshot::active_extensions`按loading phase和enablement构建，却没有production consumer把该快照挂到Workbench；App/RetainedHost启动则直接遍历`editor_plugin_registrations`并注册到命令、view、scene mode、overlay和ContributionStore，绕过manager状态。项目native registration随后只发布到Plugin Manager，未进入Workbench。因而“Active/Disabled/Faulted”与用户实际可见、可执行的扩展集合可以永久矛盾。

其次，产品没有任何统一撤销路径。`ContributionStore::revoke()`只有测试调用，`OwnedContribution`只追加并为template局部替换查ticket；command、view、scene mode、overlay和runtime consumer各自安装后没有同一owner/generation的reconcile或quiescence。即使manager切换为Disabled、项目关闭或插件reload，旧callback、trait object和UI入口仍可存活。能力过滤也不是统一安全门：snapshot只索引batch capability；importer自己的capability被忽略；scene mode完全不检查capability；overlay只检查item capability而漏掉batch/plugin capability。

回调边界同样不完整。overlay和scene mode已有局部`catch_unwind`，但Inspector `can_handle/build/validate`、field editor factory与pane-data `snapshot()`未统一经过plugin boundary；其中Inspector匹配和reflection refresh可在Workbench shell/world锁持有期间调用外来代码。`DocumentToolkitRegistry`又在mutex内调用`descriptor()`并drop callback-owned对象，且在map已经变更后重建snapshot；panic或重入会造成死锁、旧snapshot或跨registry状态分裂。现有测试甚至明确接受direct save panic逃出线程。

本报告登记 **5项P0、60项P1、15项P2和40个资格门**。Editor06继续拥有discovery/enablement/live reload流程，Editor08拥有command executor和registration lease，Editor05拥有Inspector/property authoring语义，Editor02拥有document/save/autosave/recovery事务，Plugins01拥有native ABI/unload安全；Editor50唯一拥有这些family如何以同一`ExtensionOwnerGeneration`完成prepare、commit、publish、quiesce、revoke和reconcile的共享运行时合同。

## 2. 审查边界、currentness 与证据等级

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | 证据等级 | 本轮检查重点 |
|---|---:|---|---|
| `core/extension`完整模块 | 30 / 5,267 / 170,756 | E3 | ContributionStore、batch、snapshot、lifecycle、Inspector/field editor、document toolkit及全部同目录测试 |
| extension descriptor合同 | 6 / 1,959 / 61,491 | E3 | registry、authoring descriptor、template、view、overlay provider与capability字段 |
| `core/plugin`完整模块 | 35 / 6,323 / 227,022 | E3 | catalog、manager phase/state、materialization、registration、isolation、SDK与manager tests |
| scene mode / overlay runtime | 23 / 3,015 / 97,662 | E3 | registry、prepare/install、active instance、callback isolation、capability与viewport消费 |
| Workbench/Host产品消费闭包 | 31 / 5,999 / 227,905 | E3 | startup registration、snapshot消费、Inspector/template/asset/toolkit、dirty与UI投影 |
| 聚焦产品测试 | 8 / 4,124 / 150,688 | E3 | plugin registration、overlay lifecycle、toolkit、SDK、validation与Workbench projection |
| App composition/startup | 5 / 2,416 / 90,553 | E3 | first-party/native registration来源、RunConfig传递和RetainedHost直接安装 |
| 当前计划与owner文档 | 12 / 3,168 / 305,918 | E2/E3 | 历史failure、性能审查、Editor02/05/06/08/12与Plugins01去重 |
| Unreal、Godot、Fyrox、Bevy、Unity Graphics | 14 / 16,849 / 638,005 | E2/E3 | owner unregister、module/plugin phase、toolkit lifecycle、provider discovery/priority与适用边界 |
| 去重冻结合计 | 164 / 49,120 / 1,970,000 | E2/E3 | 当前工作树fingerprint `a265ff46731682f428a5fe264cae3bf093fec0f3db160c1ab591fceb38bf87ea` |

指纹按164个selected path去重排序，对每个文件取lowercase SHA-256，再以`forward/slash/path<TAB>hash`和LF连接、无末尾LF后取总SHA-256。冻结日期为2026-08-19，基线提交为`25e09a23178000f2e783ce2143cf70a8b118d404`。

### 2.2 在途文件与动态边界

1. 冻结范围内有7个非本轮产生的在途文件：App editor启动增加test-only字段忽略；plugin catalog/test修正import与snapshot借用；viewport accessor调整test import；RunConfig公开Hub handshake并补测试；ContributionStore测试把借用ID转为owned String。这些diff未新增active-extension产品consumer、revoke/reconcile、统一capability或callback隔离，当前P0结论不依赖旧版本。
2. MVP00 session持有其中若干source/test lease。本轮只读这些文件，不修改、不回退；实施前必须重算完整指纹并重跑动态路径。
3. 本轮为review-only，没有运行Cargo、GUI、reload、panic/fault injection或规模基准。确定性调用链可证明P0存在，但不把旧test结果或静态路径存在冒充动态资格。
4. 全仓production adoption反查确认`ContributionStore::revoke()`没有产品caller，`active_extensions()`只有tests/SDK assertions消费；graph/timeline/drawer/settings等snapshot family也无产品reader。公开API和单测存在不等于产品能力存在。
5. `run_editor_plugin_boundary`只能捕获Rust unwind；它不能包含abort、FFI UB、hang、无限分配或超时。本文保留这层基础但不把它写成进程级sandbox。

### 2.3 检查方法

按`package discovery -> admission -> loading phase/enablement -> extension materialization -> validation -> capability resolution -> prepare -> cross-registry commit -> snapshot publication -> UI/toolkit callback -> disable/project close/reload -> quiesce -> revoke -> unload -> leak census`正向阅读；再从Workbench view/menu/Inspector/template/asset/scene/overlay/toolkit逐个反查production reader。每个family都检查owner identity、generation、callback affinity、预算、失败原子性、撤销顺序和旧snapshot寿命。

## 3. 必须保留的工程基础

1. 保留immutable `Arc<ContributionSnapshot>`和每family copy-on-write发布，不退回caller持锁读取可变registry。
2. 保留ticket-owned ContributionStore和candidate validation；扩展为owner generation与跨registry receipt，而不是删除ticket。
3. 保留plugin namespace校验和确定性`BTreeMap`顺序，但不要把字典顺序当显式priority。
4. 保留bounded change journal与reset语义，补bytes/time/exact IDs/page而不是恢复无界历史。
5. 保留scene mode的context checkpoint与enter/exit/input/update/overlay隔离。
6. 保留overlay provider fault/quarantine状态和锁外extract思路，补owner generation、预算与repair。
7. 保留registration先构造candidate command registry的做法，把所有family纳入同一prepare plan。
8. 保留checked DocumentId/toolkit generation分配、immutable toolkit snapshot和save/close互斥lease。
9. 保留save I/O在registry mutex外执行；descriptor也必须变成注册时冻结的数据，不能重新引入锁内callback。
10. 保留UI template与pane source同ticket替换的原子子能力，将其推广到完整owner generation。
11. 保留plugin materialization的typed diagnostics和fault状态，但失败必须使整个owner generation不可见。
12. 保留batch-level capability作为owner activation门，并与item-specific requirement取并集。
13. 保留source/owner namespace检查，升级为PackageId+BuildSet+PluginGeneration+Principal。
14. 保留field editor、Inspector、scene mode、overlay等trait扩展能力，但全部通过统一callback supervisor。
15. 保留Editor02/05/06/08各自domain authority；Editor50只提供共享挂载与撤销事务。

## 4. 当前产品断路

```text
Path A: startup-visible extensions
App / RunConfig.editor_plugin_registrations
  -> RetainedHost startup loop
  -> register_editor_plugin_registration()
  -> mutate command/view/scene-mode/overlay registries
  -> ContributionStore.contribute()
  -> append OwnedContribution
  -> visible/executable forever (no product revoke)

Path B: manager-declared active extensions
Plugin catalog + discovery + loading phase + enablement
  -> EditorPluginManagerSnapshot.active_extensions
  -> tests / SDK assertions only
  -X-> no Workbench ExtensionReconciler

Project native load report
  -> materialize registry + manager registration report
  -> active_extensions catalog
  -X-> no mount into Path A

Disable / project close / reload
  -> manager state/catalog changes
  -X-> command/view/scene/overlay/store/runtime-consumer revoke
```

同一次`register_editor_extension_owned()`也不是跨family原子事务：view、overlay、scene mode先安装，ContributionStore随后才分配ticket并发布，command registry最后替换。ticket/generation exhaustion或后续步骤失败时没有统一rollback receipt。即使一次启动成功，manager generation、ContributionStore generation、command registry generation、view registry、scene registry和overlay registry之间也没有共同commit ID可供UI或operator验证。

## 5. P0：当前正确性与生命周期断路

### E-EXT-P0-01 · Plugin Manager激活快照与Workbench真实挂载是两套不相交权威

`manager/snapshot.rs`每代构造`active_extensions`，只纳入`EditorPluginState::Active`；生产反查没有reader。`retained_host/app.rs:262-266`却直接安装RunConfig registrations，不咨询manager phase/enablement。项目native registration进入manager后也没有挂载调用。结果是manager可报告Disabled而旧入口仍可执行，也可报告Active而项目扩展从未出现在Workbench。

目标：新增唯一`EditorExtensionReconciler`消费manager generation的desired set，与当前mounted set做owner-generation diff；所有启动、project open/close、enable/disable、fault/reload只提交manager desired state，禁止另一路直接永久安装。

### E-EXT-P0-02 · 产品没有统一revoke/quiesce，disable、close和reload无法退休旧callback

`ContributionStore::revoke()`只有测试调用；`OwnedContribution`只append和查template ticket。command、view、scene mode、overlay、runtime consumer均没有同一owner的逆向撤销。旧snapshot和registry中的`Arc<dyn ...>`可跨manager状态变化继续存活；对动态代码而言，未证明callback清零前卸载会产生不可接受的vtable/code lifetime风险。

目标：每次mount返回`ExtensionMountLease`，记录所有family receipt；unmount先close admission，再等待in-flight callback/job和snapshot reader fence，按逆依赖序撤销UI/command/provider/store，发布terminal generation与leak census，最后才允许Plugins01/Editor06卸载binary。

### E-EXT-P0-03 · Capability在四条执行路径语义不一致，可绕过owner禁用门

ContributionStore的`IndexedContribution`只保存batch capability。Importer、scene/graph/timeline/overlay descriptor还有item-level capability，但asset importer query忽略item字段；SceneModeRegistry完全不检查；overlay registry只检查item字段而漏掉batch/plugin set。Menu能力被特例合并到command，证明当前规则依family手写。一个owner被禁用时，capability为空的scene/overlay callback仍可能执行。

目标：admission时把`owner activation requirements + family/item requirements + principal permission`编译为不可变`EffectiveCapabilityPredicate`；所有snapshot query与所有callback入口使用同一结果和generation，capability变化触发reconcile而不是只刷新部分容器。

### E-EXT-P0-04 · Inspector、field editor和pane-data回调未隔离，部分在Workbench锁内执行

Inspector customization选择会在shell mutex持有时调用外来`can_handle`；reflection refresh在shell/world访问期间构建customization和field editor；pane source复制后直接调用`snapshot()`，没有boundary、deadline、budget或quarantine。panic可终止主路径，重入可死锁，hang可冻结Editor；与overlay/scene mode已有的局部隔离形成不一致安全等级。

目标：锁内只冻结owner-generation callback plan与immutable input；锁外由`EditorExtensionCallbackSupervisor`执行，绑定affinity、deadline/cancel、panic/fault、output budget与quarantine；结果再次以generation/CAS提交，stale结果丢弃并可诊断。

### E-EXT-P0-05 · DocumentToolkitRegistry在mutex内调用trait并drop对象，panic/重入可拆裂map、snapshot与dirty authority

`publish_snapshot()`在registry mutex内遍历并调用每个`toolkit.descriptor()`；register已经写入两个map后才publish。`clear()`和`commit_close()`也会在锁作用域内重建descriptor或drop trait object。descriptor panic/重入可能死锁，或留下map已变而snapshot未更新。Host随后还要独立注册/注销DirtyRegistry，失败时没有共同commit。direct `save()` panic被测试接受为线程panic而非typed fault。

目标：注册时在锁外取得并验证owned descriptor，entry永久保存该值；锁内只操作纯数据并构造candidate snapshot，Arc drop移到锁外。Toolkit registry与DirtyRegistry通过一个DocumentLifecycle transaction提交；所有toolkit callback由supervisor转成typed terminal receipt。

## 6. P1：激活、Owner 与跨 Registry 事务

| ID | 当前差距 | 目标重构 |
|---|---|---|
| E-EXT-P1-01 | `active_extensions`每代重建但无production consumer。 | Reconciler订阅manager generation并发布desired/mounted一致性receipt。 |
| E-EXT-P1-02 | startup直接注册绕过loading phase、enablement和fault state。 | App只提交catalog/registration input，实际mount由manager/reconciler决定。 |
| E-EXT-P1-03 | project native reports只替换manager rows，不挂载或撤销Workbench family。 | Project generation切换生成完整desired diff与terminal old-project fence。 |
| E-EXT-P1-04 | `OwnedContribution`只有owner string+ticket且append-only。 | 使用qualified OwnerGeneration与RAII mount lease，禁止无代owner查找。 |
| E-EXT-P1-05 | direct registration都使用`editor.extension.direct`，多个caller无法区分。 | builtin/direct也必须有唯一ModuleId/InstanceGeneration。 |
| E-EXT-P1-06 | `ContributionSource::Builtin`折叠所有builtin owner。 | source携module/package/build/generation并与presentation label分离。 |
| E-EXT-P1-07 | command/view/scene/overlay/store/runtime consumer没有共同commit generation。 | prepare plan覆盖所有family，单一commit receipt列出每个registry generation。 |
| E-EXT-P1-08 | view/scene/overlay先安装，store contribute和command publish在后；后段失败无rollback。 | 所有owner mutation先生成reversible candidate，commit失败保持旧代完全可见。 |
| E-EXT-P1-09 | runtime event consumer在extension mount后单独安装，没有共同可见边界。 | consumer作为同一mount plan family，失败阻止整代publish。 |
| E-EXT-P1-10 | 只有template+pane source支持ticket内原子替换。 | owner reload替换整批family，不允许旧command配新view或反之。 |
| E-EXT-P1-11 | catalog extension materialization按item合并并累计diagnostic，部分family可在后项失败前存活。 | package generation all-or-nothing materialization，diagnostic不改变candidate可见性。 |
| E-EXT-P1-12 | manager materialization遗漏settings、pane source、overlay、operation factory和field editor等family。 | 使用一个exhaustive contribution schema；新增family未接入即编译失败。 |
| E-EXT-P1-13 | ticket ownership枚举主要只覆盖view，无法统一审计owner全部资源。 | MountReceipt列出每family stable contribution IDs和撤销结果。 |
| E-EXT-P1-14 | revoke不等待旧snapshot、route plan或active callback reader。 | reader/callback lease fence达到0后才返回Quiesced。 |
| E-EXT-P1-15 | disable/reload没有drain deadline、forced fault、leak census或operator reconcile。 | terminal policy区分Drain/Cancel/Quarantine/ProcessRestartRequired并保留receipt。 |

## 7. P1：Schema、Capability 与 Admission

| ID | 当前差距 | 目标重构 |
|---|---|---|
| E-EXT-P1-16 | snapshot只索引batch `required_capabilities`。 | 每项保存编译后的effective predicate与policy generation。 |
| E-EXT-P1-17 | AssetImporter descriptor-level capability不参与产品query。 | importer可见与可执行都使用同一effective predicate。 |
| E-EXT-P1-18 | SceneMode descriptor capability完全未被registry执行。 | register/create/push/update均验证owner generation和effective capability。 |
| E-EXT-P1-19 | Overlay只检查item capability，漏掉batch/plugin activation capability。 | preparation时取并集，capability撤销立即disable并quiesce active extract。 |
| E-EXT-P1-20 | capability是任意`String`/`BTreeSet<String>`，无长度、字符、namespace或总量门。 | canonical CapabilityId、interned compiled set及items/bytes预算。 |
| E-EXT-P1-21 | `required_capabilities`同时指owner enablement与item feature门，语义混用。 | 拆ActivationRequirement、FeatureRequirement与PrincipalPermission。 |
| E-EXT-P1-22 | PluginContributionId只有dot-separated字符串，无package version/build/generation/principal。 | ExtensionOwnerRef绑定PackageId、BuildSet、PluginGeneration和trust principal。 |
| E-EXT-P1-23 | Contribution ticket与generation用`saturating_add`，到MAX后冻结/复用语义不明。 | checked exhaustion进入terminal degraded state并拒绝新mutation。 |
| E-EXT-P1-24 | display/category/title/description/keywords等大多只做nonblank，缺字符与总bytes门。 | descriptor schema逐字段和整批执行count/depth/bytes/locale预算。 |
| E-EXT-P1-25 | `.zui` template主要校验suffix，未证明root、scheme、canonical path或artifact owner。 | 使用AssetRef/ArtifactId解析，禁止逃逸、绝对路径和未签名外部source。 |
| E-EXT-P1-26 | scene mode prepare通过真实`create()`验证，随后激活再次create，执行有副作用factory两次。 | factory admission只做declarative probe；实例创建只在commit后按明确原因执行。 |
| E-EXT-P1-27 | graph node只校验少量ID；pin/type/operation/asset/schema cross-reference不闭合。 | GraphSchemaCompiler产生typed executable schema与diagnostics artifact。 |
| E-EXT-P1-28 | timeline track/editor只验证浅层ID，缺payload schema、clock/domain和binding compatibility。 | 编译TrackTypeSchema并与Editor14 runtime/compiler contract对齐。 |
| E-EXT-P1-29 | Inspector/field editor按容器顺序first-match，没有显式priority、specificity或冲突receipt。 | typed target predicate+priority+owner tie-break，歧义fail-close。 |
| E-EXT-P1-30 | EditorExtensionRegistry与ContributionBatch重复注册/校验规则，已经存在family漂移。 | 单一schema builder生成SDK registry、batch validation和snapshot indexing。 |

## 8. P1：产品消费、Callback 与 Fault Domain

| ID | 当前差距 | 目标重构 |
|---|---|---|
| E-EXT-P1-31 | drawer contribution进入snapshot但无production reader。 | 接入真实drawer host，否则availability保持Unavailable。 |
| E-EXT-P1-32 | settings page进入catalog/descriptor，但shared store路径无完整产品消费；Editor06已拥有manager遗漏。 | Editor50提供mount transport，Editor12实现真实settings provider/UI语义。 |
| E-EXT-P1-33 | graph editor/palette与timeline editor/track仅存在descriptor/tests，无产品controller。 | domain owner接入create/open/edit/save/compile/runtime闭环前不宣称可用。 |
| E-EXT-P1-34 | command和operation factory在独立mutable registry执行，ContributionSnapshot相应family无reader。 | Store只保留manifest；可执行binding由同代CommandMountLease拥有。 |
| E-EXT-P1-35 | field editor不能经EditorExtensionRegistry/plugin catalog完整注册，production direct batch也无caller。 | SDK schema显式支持field editor factory、owner、priority与callback policy。 |
| E-EXT-P1-36 | `active_extensions()`的真实消费只在tests/SDK断言。 | 加产品reconciler integration test，删除把快照存在等同激活的断言。 |
| E-EXT-P1-37 | pane-data sources串行`snapshot()`，无panic/deadline/cancel/version/output budget。 | supervisor并行或分帧采集bounded snapshot，返回stale/fault/partial receipt。 |
| E-EXT-P1-38 | Inspector `can_handle`可在shell mutex内执行。 | 锁外匹配immutable target descriptor，结果按shell generation验证。 |
| E-EXT-P1-39 | Inspector build/surface/validate缺owner identity、boundary与quarantine。 | 所有阶段携CallbackInvocationId并进入统一fault policy。 |
| E-EXT-P1-40 | field editor factory在reflection snapshot构建时执行，未声明线程/重入/资源合同。 | 注册时声明affinity与cost class；构建结果有bytes/node/deadline门。 |
| E-EXT-P1-41 | `catch_unwind`无法包含hang、abort、FFI UB或无限分配。 | native/高风险callback采用进程隔离或可终止worker，Rust unwind只是一层防线。 |
| E-EXT-P1-42 | overlay每帧输出primitive数量、字符串、路径与CPU时间无硬预算。 | per-provider count/bytes/time budget，超限降级并隔离到下一代repair。 |
| E-EXT-P1-43 | overlay fault为永久布尔值，没有reload generation/reset probe和operator receipt。 | quarantine绑定provider generation；新代可probe，旧代永不自动复活。 |
| E-EXT-P1-44 | active scene mode不会因owner disable/reload自动pop/exit。 | reconciler先退出active instance并等待callback fence，再撤销factory。 |
| E-EXT-P1-45 | 扩展callback普遍没有UI thread/world affinity、cancel、deadline或reentry声明。 | CallbackPolicy成为每个可执行贡献的必填schema。 |

## 9. P1：Store、Snapshot 与 Document Toolkit

| ID | 当前差距 | 目标重构 |
|---|---|---|
| E-EXT-P1-46 | 每次contribute/revoke按family clone BTreeMap和完整snapshot，规模成本未资格化。 | 测量后选择persistent map/chunked index；先建立100/1k/10k contribution基准。 |
| E-EXT-P1-47 | ticket record保留整份cloned batch，与snapshot trait objects形成重复强引用。 | retention只保存撤销所需ID/index delta，callback root由mount lease唯一拥有。 |
| E-EXT-P1-48 | 空batch contribute也推进generation并占journal，测试用其制造4097变化。 | no-op返回Unchanged receipt，不污染generation与observer。 |
| E-EXT-P1-49 | change journal固定4096条，只按count，不按bytes、时间或owner criticality。 | count+bytes+age预算及明确ResetRequired原因。 |
| E-EXT-P1-50 | delta只有kind/ticket和各family count，没有exact changed IDs或before/after generation。 | bounded ID pages、owner generation、digest与continuation。 |
| E-EXT-P1-51 | 多数snapshot family不暴露source/ticket，consumer无法诊断来源或定向撤销。 | 每个IndexedContribution携OwnerRef、ContributionId、MountGeneration。 |
| E-EXT-P1-52 | snapshot query无page/bytes/deadline，caller通常全量collect。 | indexed bounded query/page与family-specific projection。 |
| E-EXT-P1-53 | toolkit snapshot每次全量重建descriptor数组，并通过trait重新取descriptor。 | entry保存owned descriptor，纯数据incremental snapshot不调用外来代码。 |
| E-EXT-P1-54 | mutex poison统一`into_inner()`且不发布degraded/invariant诊断。 | 恢复后校验双map/snapshot invariant，失败停止admission并报告operator action。 |
| E-EXT-P1-55 | direct toolkit save panic逃出API；只有上层job偶然转成Panicked。 | registry callback boundary本身返回typed ToolkitFault，不依赖caller线程模型。 |
| E-EXT-P1-56 | `autosave_source_path()`不持save lease，可与close/save/rename并发取得陈旧路径。 | autosave capture在同一document revision lease内冻结source identity。 |
| E-EXT-P1-57 | autosave payload含无界PathBuf和Vec<u8>，分配后才交给上层。 | producer-side bytes/path/deadline budget和stream/staging artifact。 |
| E-EXT-P1-58 | active_saves用`usize`和`saturating_sub`，不平衡会被静默隐藏。 | SaveLeaseId集合与checked terminal transition，重复finish为typed invariant fault。 |
| E-EXT-P1-59 | toolkit register/close/clear与DirtyRegistry分别变更，rollback failure可留下split authority。 | DocumentLifecycleTransaction统一toolkit、dirty、tab/session publication。 |
| E-EXT-P1-60 | 产品只有UI asset与animation两个toolkit family，插件没有owner-qualified toolkit factory路径。 | 先建立安全factory/mount contract；未接入的asset editor保持Unavailable而非generic fallback。 |

## 10. P2：长期工程能力

| ID | 能力 | 目标 |
|---|---|---|
| E-EXT-P2-01 | Extension runtime inspector | 展示desired/mounted generation、owner、family资源、callback、fault与revoke状态。 |
| E-EXT-P2-02 | Dependency-aware extension graph | 声明extension-to-extension、view-command-provider和toolkit依赖并拓扑reconcile。 |
| E-EXT-P2-03 | Transactional hot reload | staging新代、state handoff、atomic swap、old-generation drain与rollback。 |
| E-EXT-P2-04 | Extension state migration | versioned per-owner state schema、upgrade/downgrade/reject和crash recovery。 |
| E-EXT-P2-05 | Callback tracing | 统一记录owner generation、affinity、queue/wall/CPU、alloc、output与fault。 |
| E-EXT-P2-06 | Extension resource governor | 按owner限制CPU、memory、snapshot、overlay、job、file/network与diagnostic预算。 |
| E-EXT-P2-07 | Sandbox profiles | trusted builtin、signed native、WASM/out-of-process等不同隔离等级。 |
| E-EXT-P2-08 | Declarative compatibility lab | 多BuildSet/plugin/schema组合验证load/reload/reject与state migration。 |
| E-EXT-P2-09 | Extension dependency visualizer | 在Editor中查看provider、consumer、capability和blocked reason。 |
| E-EXT-P2-10 | Deterministic registration replay | 仅重放declarative mount plan，比较digest，不执行历史callback副作用。 |
| E-EXT-P2-11 | Snapshot retention diagnostics | 识别长期持有旧generation的reader和callback Arc root。 |
| E-EXT-P2-12 | Priority policy tooling | 检查Inspector/provider/menus的显式priority、冲突和shadowing。 |
| E-EXT-P2-13 | Extension SDK conformance suite | 第三方包可运行admission、fault、budget、disable/reload、leak和compat测试。 |
| E-EXT-P2-14 | Per-owner performance qualification | 100/1k/10k descriptor与高频callback workload的CPU/RSS/latency基线。 |
| E-EXT-P2-15 | Support bundle projection | 导出脱敏manifest、generation、fault、receipt和dependency graph，不导出secret state。 |

## 11. 参考引擎对照与适用边界

| 参考 | 当前源码证据 | Zircon应吸收 | 不应误抄 |
|---|---|---|---|
| Unreal Modular Features / ToolMenus | 显式Register/Unregister事件；ToolMenus有owner stack、`UnregisterOwner`与scoped owner；ModuleManager有unload/change通知。 | owner-scoped paired removal、generation event、先退休consumer再卸载module。 | Unreal部分实现也会在锁附近广播；不能照搬锁内callback。 |
| Unreal AssetEditorToolkit | tab spawner有Register/Unregister，toolkit有init、close reason、save/save-as、host start/finish与layout restore。 | toolkit是完整lifecycle owner，不只是descriptor+save函数。 | 不复制其大型继承层；保留Zircon typed composition。 |
| Godot EditorPlugin | dock/menu/import/inspector/gizmo等均有add/remove配对；disable会remove、clear active sets再delete plugin。 | 每family必须有对称撤销，disable是产品资源退休而非只改状态enum。 | Godot单进程扩展模型不是native fault sandbox证明。 |
| Bevy Plugin | build/ready/finish/cleanup显式阶段，unique plugin与group order可控。 | phase与order应进入declarative lifecycle和duplicate admission。 | Bevy会resume panic，不是Editor第三方callback隔离基线。 |
| Fyrox EditorPlugin | 一个plugin owner接收start/exit/sync/mode/scene/UI/suspend/resume/update/message；container回调时暂取出plugin避免别名。 | 生命周期回调归同一owner，重入/别名需显式设计。 | 暂取出容器不等同动态卸载、deadline或跨registry事务。 |
| Unity Graphics provider registry | TypeCache发现provider，按pipeline type分组、校验类型并按priority排序；Preferences provider有keywords/header/GUI。 | typed discovery、priority与真实provider行为，而非三字段metadata。 | static lazy registry没有disable/unload authority，不能作为Zircon终态模型。 |

参考源码只能证明成熟引擎如何组织owner与生命周期，不能证明Zircon性能已经达到或超过Unreal。竞争性结论仍需同功能、同资产、同硬件、同质量设置的CPU/GPU/RSS/VRAM/latency统计证据。

## 12. 目标架构

```text
EditorPluginManagerSnapshot(manager_generation, desired active owners)
  -> ExtensionReconciler.plan(previous mounted generation)
       -> validate schema / capability / dependency / budgets
       -> prepare immutable family candidates
       -> prepare callback supervisors and toolkit factories
       -> produce ExtensionMountPlan + rollback plan
  -> atomic commit fence
       -> command/view/menu/inspector/template/asset
       -> scene-mode/overlay/graph/timeline/toolkit/consumer
       -> ContributionSnapshot manifest
  -> ExtensionRuntimeSnapshot(mounted_generation, receipts)

Disable / project close / reload
  -> close owner admission
  -> cancel/drain callbacks and jobs
  -> exit active scene/toolkit instances
  -> revoke family leases in reverse dependency order
  -> publish terminal generation + leak census
  -> unload binary only after zero callback/reader/code roots
```

核心identity：

```text
ExtensionOwnerRef = PackageId + PackageVersion + BuildSetId + PluginGeneration + PrincipalId
ContributionRef   = ExtensionOwnerRef + FamilyId + ContributionId + MountGeneration
CallbackLease     = ContributionRef + InvocationId + Affinity + Deadline + Budget
MountReceipt      = DesiredManagerGeneration + PreviousMountedGeneration
                    + NewMountedGeneration + FamilyReceipts + TerminalDisposition
```

`ContributionStore`不再充当可执行对象的第二生命周期owner。它发布manifest/read model；真实callback root由family mount lease持有。Document toolkit entry保存owned descriptor和host-neutral state，mutex内不调用trait；save/autosave/close通过DocumentLifecycle owner与Editor02 transaction/dirty authority连接。

## 13. 依赖顺序与重构里程碑

### M0 · Truth Freeze 与 RED Contract

- 冻结manager desired set与实际mounted set的差异测试；加入project native Active但不可见、Disabled但仍可执行的RED产品测试。
- 为每family建立production consumer inventory和Unavailable truth；禁止descriptor/tests冒充产品采用。
- 建立callback-under-lock、panic/reentry、ticket/generation exhaustion和partial commit fault injection。

### M1 · Owner、Capability 与 Declarative Schema

- 引入ExtensionOwnerRef、ContributionRef、checked generations和统一family schema。
- 编译effective capability，删除scene/overlay/importer的手写旁路。
- 合并EditorExtensionRegistry/ContributionBatch验证来源，增加items/bytes/depth/path预算。

### M2 · Atomic Extension Reconciler

- Manager成为desired activation唯一权威；App/RetainedHost删除直接永久安装路径。
- 所有family实现prepare/commit/rollback/revoke lease和共同MountReceipt。
- project open/close、enable/disable、fault/reload以同一generation diff驱动。

### M3 · Callback Supervisor 与 Unload Fence

- Inspector、field editor、pane source、scene、overlay、toolkit统一callback policy。
- 锁外执行、deadline/cancel/output budget、quarantine、stale CAS和operator diagnostics。
- 与Editor06/Plugins01实现quiesce -> leak census -> binary unload硬门。

### M4 · Document Toolkit Registry Hard Cut

- descriptor变为注册时冻结owned data；mutex内只做纯数据candidate mutation。
- Toolkit/Dirty/Tab lifecycle统一transaction，save/autosave/close返回typed receipt。
- 增加panic/reentry/close-save race/oversize payload与rollback tests。

### M5 · Product Consumer Closure

- 接入drawer、settings、field editor、graph/timeline的真实domain controller，或显式Unavailable。
- active extension snapshot、Workbench projection、command/menu和provider都读取同一mounted generation。
- UI展示disabled/faulted/quarantined原因，不保留可触发旧callback的入口。

### M6 · Scale、Soak 与 Failure Qualification

- 100/1k/10k contributions、frequent capability toggles、snapshot reader retention和reload storm基准。
- 测量commit latency、UI stall、alloc/RSS、old-generation retention、callback p95/p99与shutdown deadline。
- crash/abort/hang/native fault需要进程级隔离与恢复证据，不以catch_unwind代替。

### M7 · Competitive Qualification

- 使用与Unreal/Godot等同功能extension workload和相同硬件，分别报告startup、enable/disable、hot reload、Editor interaction和memory。
- 正确性、泄漏、故障、soak与artifact/BuildSet一致性先通过，再允许“优于Unreal”的结论。

## 14. 验收门

| Gate | 验收内容 |
|---|---|
| E-EXT-G01 | manager Active owner集合与mounted owner集合在同一generation receipt中完全一致 |
| E-EXT-G02 | Disabled owner的view/menu/command/scene/overlay/Inspector/template/toolkit入口全部不可达 |
| E-EXT-G03 | project close退休所有project owner且旧generation callback计数归零 |
| E-EXT-G04 | reload只使新generation可见，旧snapshot不能调用旧代码 |
| E-EXT-G05 | 任一family prepare失败时所有registry保持旧代不变 |
| E-EXT-G06 | commit fault injection证明rollback完整或进入可reconcile typed unknown state |
| E-EXT-G07 | direct/builtin/plugin owner均有唯一qualified identity，无共享默认owner |
| E-EXT-G08 | ticket、manager、mount、snapshot与toolkit generation耗尽均fail-close |
| E-EXT-G09 | batch+item+principal capability取并集，所有query和callback使用同一predicate |
| E-EXT-G10 | capability变化通过reconcile原子更新可见性和可执行性 |
| E-EXT-G11 | schema新增family未接入materialize/validate/mount/revoke会编译失败 |
| E-EXT-G12 | descriptor整批满足items/bytes/depth/string/path预算 |
| E-EXT-G13 | template引用只能解析到允许的AssetRef/ArtifactId和owner root |
| E-EXT-G14 | graph/timeline schema cross-reference在产品controller前完整编译 |
| E-EXT-G15 | Inspector匹配冲突有确定priority或typed rejection，不依赖插入顺序 |
| E-EXT-G16 | shell/world/registry mutex持有期间不执行任何外来callback或Drop |
| E-EXT-G17 | Inspector、field editor、pane source panic均转typed fault且Editor继续运行 |
| E-EXT-G18 | callback重入测试无死锁，stale结果不能覆盖新generation |
| E-EXT-G19 | callback hang超过deadline后可隔离/终止，不冻结UI主线程 |
| E-EXT-G20 | overlay每帧有count/bytes/time硬门及可观察降级 |
| E-EXT-G21 | scene mode owner disable会exit active instance后再撤销factory |
| E-EXT-G22 | quarantine绑定generation，新代repair不会复活旧代对象 |
| E-EXT-G23 | native unload前callback/job/reader/code root leak census为0 |
| E-EXT-G24 | ContributionStore no-op不推进generation、不写journal |
| E-EXT-G25 | change journal受count/bytes/age约束并返回exact gap/reset receipt |
| E-EXT-G26 | 每个snapshot row可追溯OwnerRef、ContributionRef和MountGeneration |
| E-EXT-G27 | snapshot query有count/bytes/deadline page，产品不全量collect无界结果 |
| E-EXT-G28 | 10k contribution workload有commit/query/revoke CPU、alloc、RSS基线 |
| E-EXT-G29 | 旧snapshot长期持有可被诊断且不会阻塞无限期unload |
| E-EXT-G30 | toolkit descriptor只在锁外取得一次，snapshot重建不调用trait |
| E-EXT-G31 | toolkit panic/reentry/Drop测试无死锁且map/snapshot invariant保持 |
| E-EXT-G32 | save/close/autosave lease使用checked ID，重复finish不会静默饱和 |
| E-EXT-G33 | autosave payload在分配前满足bytes/path/deadline预算 |
| E-EXT-G34 | Toolkit/Dirty/Tab注册关闭在同一document lifecycle receipt内提交 |
| E-EXT-G35 | save/close/clear fault injection后reopen能恢复唯一authoritative状态 |
| E-EXT-G36 | 无真实consumer的drawer/settings/graph/timeline/toolkit显示Unavailable |
| E-EXT-G37 | startup、project、enable/disable、reload与shutdown均通过真实RetainedHost产品测试 |
| E-EXT-G38 | extension diagnostics绑定BuildSet、owner generation、callback和mount receipt |
| E-EXT-G39 | 同硬件同workload报告startup/reload/UI latency/RSS且含统计置信信息 |
| E-EXT-G40 | source fingerprint、finding counts、frontmatter路径、links、LF/BOM/trailing-space与`git diff --check`通过 |

## 15. 与其他报告的唯一 Owner 边界

| 报告 | 继续拥有 | Editor50只拥有 |
|---|---|---|
| Editor02 | document revision、transaction、dirty、save/autosave/recovery语义 | toolkit作为extension family的mount/revoke和锁内callback安全 |
| Editor05 | Inspector/property surface、customization/field editor authoring语义 | customization/provider的owner generation、callback supervisor和撤销 |
| Editor06 | plugin discovery、phase、enablement、settings diagnostics、live reload UX | manager desired set到所有Workbench family的原子reconcile runtime |
| Editor08 | command executor、capability/principal admission、command registration lease | command family参与同一extension mount generation和跨registry commit |
| Editor12 | settings scope/persistence/i18n/appearance与页面产品体验 | settings provider贡献的transport、生命周期和callback policy |
| Editor14 | graph/timeline/animation domain语义与compiler/runtime parity | generic graph/timeline descriptor family的admission/mount truth |
| Plugins01 | package/native ABI、signature/trust、foreign ownership和binary unload | Editor侧callback root quiescence、lease revoke和zero-root receipt |
| Tooling32/34/35 | global ownership、type-erasure与transaction治理 | 当前Editor extension具体owner/callback/commit实现与产品gate |

## 16. 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| 164文件静态审查与产品consumer反查 | review_complete | 2026-08-19 | 49,120行、1,970,000 bytes；fingerprint `a265ff46731682f428a5fe264cae3bf093fec0f3db160c1ab591fceb38bf87ea` |
| 五家参考引擎适用性对照 | review_complete | 2026-08-19 | 14文件、16,849行；owner unregister、phase、toolkit与provider边界 |
| P0/P1/P2与owner去重 | review_complete | 2026-08-19 | 5 P0 / 60 P1 / 15 P2 / 40 gates |
| Production重构 | pending | - | 本篇不修改production或tests；M0-M7均未实施 |
