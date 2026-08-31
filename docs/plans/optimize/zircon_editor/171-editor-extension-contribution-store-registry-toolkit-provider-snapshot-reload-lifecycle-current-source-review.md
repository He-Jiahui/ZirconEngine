---
title: Editor Extension、Contribution Store、Registry、Toolkit、Provider、Snapshot、Reload 与 Lifecycle 当前源码复核
category: zircon_editor
report_id: Editor171
review_date: 2026-08-27
baseline_head: 64942164497096a82cbb4a721405d9ffe367bccf
production_baseline: 982baa1ba87bc8c25fe44312507a4af15027e058
canonical_owner: Editor50
refreshes:
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/123-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-current-source-review.md
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
  - zircon_editor/src/ui/settings
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/editor
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_runtime_interface/src/editor_contribution.rs
tests:
  - zircon_editor/src/core/extension/store/tests.rs
  - zircon_editor/src/core/extension/toolkit/tests
  - zircon_editor/src/core/plugin/manager/tests
  - zircon_editor/src/scene/modes/tests.rs
  - zircon_editor/src/scene/modes/tests
  - zircon_editor/src/tests/editor_plugin_sdk.rs
  - zircon_editor/src/tests/editor_event/runtime/extensions_registration
  - zircon_editor/src/tests/host/manager/document_toolkit_lifecycle.rs
  - zircon_editor/src/ui/settings/tests.rs
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

# 171 · Editor Extension / Contribution Store / Registry / Toolkit / Provider / Lifecycle 工程化复核

## 1. 最终结论

当前 Extension 子系统不是“只有临时占位”，但也远未形成 Unreal/Godot 级别的工程化扩展运行时。可以保留的底座已经相当明确：`ContributionStore` 将 19 个 contribution family 发布为 immutable `Arc<ContributionSnapshot>`，每项内部保存 ticket、source 与 batch capability；mutation 通过 snapshot 浅克隆和 family 级 `Arc::make_mut` 保持旧 reader 不变；固定 4,096 条 change journal 能给出 generation reset。Inspector/field editor 已有真实 Workbench 消费，scene mode 与 viewport overlay 有局部 panic boundary，overlay 有 capability toggle 与 fault quarantine。`DocumentToolkitRegistry` 也已使用 checked document/generation allocation、save/close lease、不可变 descriptor snapshot、锁外 save/autosave callback，并把 descriptor 捕获和 trait-object 析构移出 registry mutex。

但主控制面仍由两套互不闭合的权威组成。`EditorPluginManagerSnapshot::active_extensions()` 只被 manager/SDK tests 读取；产品启动仍从 `EditorHostRunConfig::editor_plugin_registrations` 直接调用 `register_editor_plugin_registration()`。manager 的 Active/Disabled/Faulted generation 不决定 Workbench 的 mounted callbacks，Workbench 的实际安装也不回写 manager mounted receipt。当前精确搜索仍没有 `EditorExtensionReconciler`、`ExtensionMountLease`、`ExtensionOwnerRef`、`MountReceipt`、desired/mounted set 或 leak census。

注册也不是跨 registry 原子事务。Host 在持有 shell lock 时先 prepare scene mode/overlay，构造 command candidate，然后直接安装 manager views、overlay 和 scene registry，最后才调用 `ContributionStore::contribute()`；runtime consumer 又在该函数成功返回后单独 install。后段失败没有逆序 rollback，旧 generation 也没有继续可见的 commit guarantee。生产代码没有 `ContributionStore::revoke()` caller；本轮看到的 `settings_page_projection.rs` 与 `materializer.rs` 两个额外命中均位于测试模块，不能被计为产品卸载链。

Capability 与 callback safety 仍按 family 手写。Store 只保存 batch capability；asset importer query 忽略 item requirement，scene mode registry没有 capability，overlay 只保存 item requirement而未合并 batch/plugin activation。Inspector `can_handle()`仍在 shell mutex 内执行，reflection build 又在持锁期间调用 customization build 与 field-editor factory。局部 `catch_unwind` 不能处理 hang、abort、FFI UB、无限分配、deadline 或 cancellation。

本轮不新增 canonical finding，继续由 Editor50 拥有 5 个 P0、60 个 P1、15 个 P2。当前状态为：P0 **4 Open / 1 Partial / 0 Closed**；P1 **46 Open / 14 Partial / 0 Closed**；P2 **15 Open**；40 个资格门为 **22 Fail / 18 Partial / 0 Pass**。没有动态 correctness、race、fault、reload、unload、scale 或跨引擎同场景证据，禁止把 descriptor、单元测试或局部 snapshot 进展写成“达到或超过 Unreal”。

## 2. 审查边界与 currentness

### 2.1 Owner 与去重

1. Editor171 只刷新 Editor50/123，不重复登记 Editor06 的 package discovery/enablement、Editor08 的 command admission、Editor05 的 Inspector/property 语义、Editor02 的 save/recovery、Editor12 的 Settings 产品或 Plugins01 的 native ABI/unload。
2. 本报告拥有 package active generation 如何转成 mounted owner generation，以及 command/view/scene/overlay/store/runtime consumer/toolkit 如何共用 prepare、publish、quiesce、revoke 与 terminal receipt。
3. Settings、graph、timeline、drawer等 domain owner负责功能内容；Editor171只裁决 contribution 是否被真实产品消费并受统一 lifecycle 控制。
4. Tooling 按用户要求排除；本轮没有查询、轮询、等待或实时跟踪协调器状态。

### 2.2 冻结点

| 项目 | 当前值 |
|---|---|
| 当前磁盘冻结时间 | `2026-08-27T15:33:03.5413709+08:00` |
| Git HEAD | `64942164497096a82cbb4a721405d9ffe367bccf` |
| production baseline | `982baa1ba87bc8c25fe44312507a4af15027e058` |
| working tree | 冻结时 `git status --short` 为 8,134 条；裁决针对 fingerprint 对应的当前磁盘内容，不假装等同 HEAD |
| 动态证据 | 未运行 Cargo、Editor、reload/unload、panic/hang、race、scale、soak 或 benchmark lane |

### 2.3 可复算 selected set

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | Fingerprint |
|---|---:|---|
| Zircon extension/plugin/Host/App/Interface/tests | **140 / 29,355 / 26,711 / 1,035,365 / 302 / 15** | `cffa06ce561b7d8f8106d27c0fee40dfd0a46080b6d0c153105067d597d4fa95` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics | **14 / 16,849 / 14,257 / 638,005 / 19 / 0** | `e1cc884ca7324242a4de28e0d5741d34095077a2d22b77392c20be942d40eae2` |
| 全部选择集 | **154 / 46,204 / 40,968 / 1,673,370 / 321 / 15** | `83b9ef8910af133a07f6a53144ade0a8bfadb9cbc0ca61c3e6f10bcb8c7e9242` |

Fingerprint 使用 workspace-relative 小写 `/` 路径和逐文件 SHA-256 组成 `path + NUL + hash + LF` 清单，再对清单做 SHA-256。Zircon scope 递归展开 frontmatter 的 related/test roots并按物理路径去重；reference scope 是 14 个明确文件。实施前必须重新复算，不能把共享工作树后续变化偷偷并入本冻结结论。

## 3. 当前源码事实

### 3.1 manager active catalog 与 Workbench mount 是两套权威

`core/plugin/manager/snapshot.rs` 的 `build_active_extensions()`按 entry state 生成 `EditorExtensionCatalogReport`，但 `active_extensions()`的所有读取都在 manager tests、project-selection tests、snapshot-publication tests和`tests/editor_plugin_sdk.rs`。产品端没有订阅 manager generation、计算 desired/mounted diff 或执行 reconcile 的 caller。

相反，`ui/retained_host/app.rs` 仍直接遍历 run config 的 registrations。App/entry/composition/first-party plugin路径合计持续传递这份并列注册表，Host 无法证明它与 manager 当前 generation相同。结果是 manager报告 Disabled/Faulted 后 callback仍可挂载，或 manager报告 Active 而贡献从未进入 Workbench。

### 3.2 Host 注册只在局部使用 candidate，跨 family 会部分提交

`register_editor_extension_owned()`先验证 store与view绑定，再执行下列顺序：

```text
freeze ContributionBatch
  -> prepare scene-mode candidate（会调用 factory）
  -> prepare overlay candidate（会 create provider）
  -> clone/build command registry candidate
  -> mutate manager view registry
  -> install overlay registry
  -> install scene-mode registry
  -> ContributionStore.contribute()
  -> append OwnedContribution(owner string, ticket)
  -> replace command registry
  -> caller installs runtime event consumers
```

command candidate和Store publish各自有局部原子性，但并不存在覆盖全部步骤的 commit id、candidate digest 或 rollback receipt。尤其 view/overlay/scene已安装后，Store失败会把可执行 side effect留在产品状态；runtime consumer又不属于同一个提交。`plugin_registration_gate`只串行化调用，不能提供事务性。

### 3.3 ContributionStore 是真实底座，但不是 lifecycle authority

Store当前覆盖 view、drawer、menu、inspector、field editor、template、pane source、importer、asset type、localization、settings、scene mode、overlay、graph、timeline、command和operation factory共19类。`IndexedContribution<T>`对所有类保存 ticket/source/batch capability，snapshot query按batch capability过滤；contribute/revoke使用旧snapshot浅克隆，只对有键变化的family执行COW，因此旧报告“每次深拷贝全部family map”已不精确。

未闭合点仍很集中：

1. `TicketRecord`保留完整`ContributionBatch`，与snapshot重复持有trait object/callback root。
2. empty batch仍分配ticket、推进generation并写journal。
3. ticket、generation和replay边界继续使用`saturating_add/sub`，耗尽后会失去唯一语义而非fail closed。
4. journal只有4,096条count上限，没有bytes/age/criticality；delta只有source/ticket/kind/count和reset，没有exact IDs、digest、cause、cursor或causal parent。
5. source/ticket虽存在于所有内部entry，公共snapshot只为少数view/template/asset type路径暴露。
6. query大多返回全量iterator，调用者collect；没有page、bytes、deadline或stale/partial receipt。
7. Store revoke只有tests调用；没有任何产品owner通过ticket触发跨registry teardown。

### 3.4 capability在不同 family 中产生不同答案

Store过滤的是`ContributionBatch.required_capabilities`。command注册又把batch与menu item capability手工合并；overlay active entry只复制`ViewportOverlayProviderRegistration.required_capabilities`，没有接收batch requirement；scene mode registration没有capability字段；asset importer产品查询先按batch过滤，然后只按suffix匹配，忽略descriptor item requirement。

`CapabilitySet`本身只是`BTreeSet<String>`，没有canonical namespace、长度/总字节预算、schema version、principal permission或policy generation。当前“enabled”实际混合 package activation、feature availability和security permission，无法形成所有query/factory/callback共享的`EffectiveCapabilityPredicate`。

### 3.5 callback隔离只覆盖scene/overlay局部

scene-mode factory、id、enter/exit/update/input/overlay/drop通过`run_editor_plugin_boundary()`和`IsolatedSceneMode`捕获panic；overlay extract也捕获panic并把provider置为faulted。这个基础应保留，但它只有owner字符串和最后一条错误，没有invocation identity、deadline、cancel、thread affinity、output budget、generation reset或repair probe。

危险调用仍存在：

1. `extension_access.rs::inspector_customization()`持有shell lock后调用外部`can_handle()`。
2. `refresh_reflection()`持有shell lock构造chrome，Inspector chain的matching/build和field-editor factory可在该锁域执行。
3. pane-data source已先克隆出锁再逐个`snapshot()`，但仍串行且无boundary、timeout、cancel或bytes预算。
4. toolkit save/capture/path callback在registry锁外是进展，但panic仍可穿过公开API；`catch_unwind`本身也无法恢复native hang/abort/UB。

### 3.6 Toolkit P0 已有实质修复，但只够降为 Partial

`DocumentToolkitRegistry::register()`现在先在锁外执行`toolkit.descriptor().clone()`，`RegistryEntry`保存owned descriptor，`publish_snapshot()`只遍历纯数据。`clear()`和`commit_close()`均先在锁内切换map/snapshot，再显式解锁并析构retired trait object；focused tests也覆盖descriptor只捕获一次和Drop重入。这纠正了Editor123中“descriptor callback和trait-object drop仍在mutex内”的过时描述。

仍不能关闭P0-05：snapshot每次仍重建整个descriptor array；Toolkit registry与Host `DirtyRegistry`没有共同lifecycle transaction；poison继续静默`into_inner()`；save/capture/path callback没有typed fault boundary；`autosave_source_path()`不持save lease；close只检查active save，不等待pane/provider callback或background job。局部锁域安全不等于完整document/extension retirement。

### 3.7 Settings与其他产品消费者的真实边界

Settings新增了有价值的projection：`SettingsPageProjection`按capability、locale和contribution generation冻结plugin page；`SettingsWindowProjection`把built-in setting与plugin category合并，并有en/zh-CN与revoke测试。serialized schema也新增LocalizationBundle和SettingsPage，且page key必须存在于owner bundle。

然而全仓`SettingsWindowProjection::capture()`调用都在`ui/settings/tests.rs`。产品没有Settings window/controller捕获projection，更没有plugin page content provider、read/write transaction、validation、apply/restart或unmount UI。P1-32只能从Open降到Partial。Drawer仍没有ContributionSnapshot production reader；graph/timeline只有registry/materialization/tests，没有open/edit/save/compile产品链。Inspector和field editor属于真实reader，但继续受锁内callback与owner-generation缺失约束。

### 3.8 serialized host-safe schema扩展了，但与进程内schema仍分裂

`zircon_runtime_interface::SerializedEditorContribution`当前只有View、Drawer、Menu、Command、AssetType、LocalizationBundle、SettingsPage七种variant；每种有硬版本字符串，batch排序并拒绝同kind/id重复，materializer在registry clone上all-or-nothing发布。这是P1-11/12的真实Partial。

进程内`EditorExtensionRegistry`和`ContributionBatch`还支持Inspector、field editor、template/pane、importer、scene mode、overlay、graph/timeline、operation factory等family。manager `build_editor_extensions()`对单项错误只追加diagnostic并继续构建剩余registry，不是package-generation all-or-nothing。新增family仍需手工同步Interface enum、SDK、materializer、registry、Store、Host install/revoke和product projection，编译期没有exhaustive lifecycle接入门。

### 3.9 测试证据不能替代产品资格

选择集有302个`#[test]`和15个ignored test/benchmark。Store测试覆盖namespace、collision、ticket revoke、old snapshot、journal reset、template replacement；Toolkit覆盖save/close lease、write authority、descriptor/drop锁域；scene mode覆盖factory mismatch和panic isolation。四个focused ignored性能证据涉及Inspector hash、layout dedup/bitset和1,000 toolkit descriptor capture。

缺失的仍是manager-to-Workbench mounted generation集成测试、跨registry后段失败rollback、disable/reload/project close quiesce、stale reader/callback/job fence、native unload、panic/hang/FFI fault、1/100/1K/10K owner与19-family store规模、长时间reload leak census和跨平台动态矩阵。本轮没有执行这些测试，不能把静态存在计为通过。

## 4. 本地参考源码对照

### 4.1 Unreal：模块、feature、menu owner和toolkit是分层生命周期

`IModularFeatures`明确提供register/unregister以及registered/unregistered event；`FModuleManager`区分loaded、unload和abandon并发布module change；ToolMenus可按owner一次撤销所有注册，`FToolMenuOwnerScoped`把注册归属写入作用域；AssetEditorToolkit又单独拥有host、save、close和layout生命周期。Zircon不应复制其具体API，但必须达到“模块状态、owner资源、可执行callback和toolkit实例可分别追踪并在卸载前收敛”的最低合同。

### 4.2 Godot：EditorPlugin实例是add/remove对称入口

Godot `EditorPlugin`为dock、inspector plugin、tool menu和custom type提供add/remove对称API，`EditorNode`/`EditorData`集中持有plugin实例和enabled state。它也不是自动提供跨进程安全的终极答案，但说明extension不是若干永不撤销的全局registry写入；实例退役必须由Editor owner驱动并撤掉具体UI/callback资源。

### 4.3 Bevy与Fyrox：phase和plugin callback边界必须显式

Bevy `Plugin`将build、ready、finish、cleanup与uniqueness写成显式phase，App维护plugin state；Fyrox EditorPlugin暴露start/exit/sync/update/message等Editor-owned入口。二者的plugin contract不直接解决Zircon native unload或UI owner receipt，但都比“manager report一套、Host direct registration另一套”更可裁决。

### 4.4 Unity Graphics：provider discovery必须绑定适用域

MaterialUpgraderRegistry按RenderPipelineAsset type发现provider并验证supported pipeline attribute；Preferences provider把display/search/UI contract归入同一provider。该实现没有Zircon需要的hot-unload fence，不能照抄为上限；可借鉴的是provider在发现时就声明适用域，query不应在每个family里重新解释裸字符串capability。

## 5. Editor50 finding重判

### 5.1 汇总

| 级别 | Open | Partial | Closed | 合计 |
|---|---:|---:|---:|---:|
| P0 | 4 | 1 | 0 | 5 |
| P1 | 46 | 14 | 0 | 60 |
| P2 | 15 | 0 | 0 | 15 |

### 5.2 P0

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **P0-01** manager active snapshot没有Workbench production consumer | Open | `active_extensions()`读取仍全部为tests；App仍直接安装run-config registrations。必须由reconciler消费manager generation并发布desired/mounted receipt。 |
| **P0-02** 没有统一revoke、quiesce和unload fence | Open | 所有`ContributionStore::revoke()`命中均在tests；command/view/scene/overlay/runtime consumer没有owner lease。必须逆依赖撤销并等待reader/callback/job归零。 |
| **P0-03** capability admission在family间不一致 | Open | batch、menu、importer、scene mode、overlay分别解释；没有effective predicate/principal generation。必须由同一编译产物控制query/factory/callback。 |
| **P0-04** 外部callback仍可锁内或无隔离执行 | Open | Inspector `can_handle/build`和field factory可在shell/reflection锁域执行；pane/toolkit无统一supervisor。必须锁外执行、deadline/cancel/budget/quarantine并丢弃stale result。 |
| **P0-05** Toolkit纯数据发布、drop与document lifecycle事务 | Partial | descriptor捕获和trait-object drop已移出mutex；snapshot仍全量重建，poison/fault/Dirty/autosave/close fence未闭合。必须以typed transaction和terminal receipt完成。 |

### 5.3 P1

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **P1-01** active_extensions只被tests/status读取 | Open | 无production reconciler。 |
| **P1-02** startup direct registration绕过phase/enablement/fault | Open | retained App仍直接循环registrations。 |
| **P1-03** project native report只替换manager rows | Open | 无project generation mount/revoke diff。 |
| **P1-04** OwnedContribution只有owner string+ticket | Open | 无package/build/instance generation。 |
| **P1-05** direct caller统一为`editor.extension.direct` | Open | 多direct/builtin实例不能归因。 |
| **P1-06** Builtin source折叠所有builtin owner | Open | `ContributionSource::Builtin`不携module identity。 |
| **P1-07** command/view/scene/overlay/store无共同commit id | Open | 只有局部candidate。 |
| **P1-08** 后段失败无跨family rollback | Open | view/overlay/scene可先于Store发布。 |
| **P1-09** runtime consumer在Store成功后另行install | Open | 不属于mount transaction。 |
| **P1-10** 只有template/pane支持局部替换 | Partial | replacement保留ticket与旧reader，但非owner整批generation替换。 |
| **P1-11** materialization可能部分family成功 | Partial | serialized batch是candidate原子发布；manager in-process build仍逐项记录错误并保留其他family。 |
| **P1-12** manager/serialized schema漏family | Partial | settings/localization已加入，serialized仍只有7类，缺pane/overlay/field/factory/graph/timeline等。 |
| **P1-13** ticket ownership无完整family receipt | Partial | Store内部有19类keys/counts/full batch；没有稳定ID列表、mount generation或公开terminal receipt。 |
| **P1-14** revoke不等待snapshot/route/callback reader | Open | old Arc reader可无限存活。 |
| **P1-15** disable/reload无drain deadline或leak census | Open | 对应类型和产品流程均不存在。 |
| **P1-16** snapshot只保存batch capability | Open | item/effective predicate未编译进entry。 |
| **P1-17** importer item capability不进入query | Open | query仅batch capability+suffix。 |
| **P1-18** scene mode不检查capability | Open | registry/create/push无capability参数。 |
| **P1-19** overlay漏batch/plugin activation capability | Open | active entry只复制item requirement。 |
| **P1-20** capability为任意String且无预算 | Open | 无typed ID/namespace/bytes gate。 |
| **P1-21** activation与feature requirement混用 | Open | 单一字符串集合承担多种语义。 |
| **P1-22** PluginContributionId无version/build/principal | Open | 只验证点分段字符串。 |
| **P1-23** ticket/generation使用saturation | Open | Store耗尽不fail closed。 |
| **P1-24** descriptor/batch缺完整预算 | Partial | 已有namespace、duplicate、cross-reference和localization key校验；无count/depth/bytes/locale/callback预算。 |
| **P1-25** `.zui`主要检查suffix | Open | 无canonical AssetRef/ArtifactId/owner proof。 |
| **P1-26** scene-mode prepare执行有副作用factory | Open | `candidate_registry()`明确调用`create()`。 |
| **P1-27** graph schema cross-reference不闭合 | Open | 无typed compiler artifact。 |
| **P1-28** timeline缺clock/binding compatibility | Open | descriptor尚未接Editor14编译协议。 |
| **P1-29** Inspector first-match无priority/specificity | Open | Vec顺序决定winner，歧义不fail-close。 |
| **P1-30** registry/batch/interface重复校验会漂移 | Open | 多层仍手工同步family。 |
| **P1-31** drawer无真实production reader | Open | Store query仅定义/测试/materialization。 |
| **P1-32** settings page未形成Store到UI闭环 | Partial | locale/capability projection和tests存在；production capture/content/read-write mount为零。 |
| **P1-33** graph/timeline无open/edit/save/compile consumer | Open | 只有descriptor注册和测试。 |
| **P1-34** executable command registry与snapshot family分裂 | Open | command candidate另行替换且无owner lease。 |
| **P1-35** field editor缺完整catalog/owner/priority | Partial | ticket-owned catalog与reflection consumer存在；无owner generation、priority、serialized SDK和callback policy。 |
| **P1-36** active_extensions缺product assertion | Open | tests只检查report，不检查Workbench mounted state。 |
| **P1-37** pane snapshot串行且无timeout/cancel/budget | Open | callback虽已锁外，执行合同仍全部缺失。 |
| **P1-38** Inspector can_handle在shell mutex内 | Open | `extension_access.rs`仍可复现。 |
| **P1-39** Inspector阶段无invocation identity | Open | 无supervisor/correlation/quarantine。 |
| **P1-40** field factory无thread/reentry/cost声明 | Open | 仍是裸function pointer。 |
| **P1-41** catch_unwind不能处理hang/abort/FFI UB | Partial | scene/overlay局部panic boundary存在；高风险native可终止worker/process隔离不存在。 |
| **P1-42** overlay输出无count/bytes/path/time预算 | Open | extract直接返回并collect完整Vec。 |
| **P1-43** overlay fault无generation reset/probe | Partial | fault flag/quarantine/last error存在；repair、新代和probe不存在。 |
| **P1-44** active scene mode不随owner disable退出 | Open | 无owner-scoped pop/fence。 |
| **P1-45** callback普遍无affinity/cancel/deadline/reentry | Open | 局部panic捕获不满足policy。 |
| **P1-46** contribute/revoke复制索引且无规模裁决 | Partial | snapshot是Arc浅克隆、只COW有变化family；无100/1K/10K Store benchmark和chunk/persistent决策。 |
| **P1-47** ticket保留整batch并重复callback root | Open | `TicketRecord.batch`仍完整持有。 |
| **P1-48** 空batch推进generation/journal | Open | 没有Unchanged fast path。 |
| **P1-49** journal只有固定count | Open | 无bytes/age/criticality。 |
| **P1-50** delta无exact IDs | Partial | generation、reset、source/ticket/kind/count存在；无ID page/digest/cause。 |
| **P1-51** snapshot多数family不公开source/ticket | Partial | internal entry统一保存；公共API只暴露少数family。 |
| **P1-52** query无page/bytes/deadline | Open | caller仍全量iterate/collect。 |
| **P1-53** toolkit snapshot每次重建descriptor array | Partial | entry已保存owned descriptor和稳定snapshot storage；mutation仍全量collect。 |
| **P1-54** poison `into_inner()`无invariant receipt | Open | Store/Toolkit/overlay mutex均有静默恢复点。 |
| **P1-55** toolkit save panic逃出API | Open | 无`ToolkitFault` boundary。 |
| **P1-56** autosave_source_path不持save lease | Open | 只锁内clone toolkit后直接callback。 |
| **P1-57** close lease不fence pane/provider/job | Open | 只检查active save。 |
| **P1-58** toolkit generation不绑定manager/plugin | Open | snapshot只有本地u64。 |
| **P1-59** clear/close缺每项terminal receipt | Partial | map切换后锁外drop且返回descriptor；无two-phase retire/fault/leak状态。 |
| **P1-60** ContributionChange缺lifecycle cause/cursor | Open | 只有generation/ticket/source/kind/count。 |

### 5.4 P2

| Finding | 状态 | 当前差距 |
|---|---|---|
| **P2-01** | Open | registry snapshot无schema version/migration policy。 |
| **P2-02** | Open | diagnostics主要为字符串，缺code/severity/owner/generation/remediation。 |
| **P2-03** | Open | rejection不返回candidate digest。 |
| **P2-04** | Open | BTree/Vec顺序仍隐式充当priority。 |
| **P2-05** | Open | owner展示label与security principal未分离。 |
| **P2-06** | Open | manager/plugin fault没有用户可执行repair action。 |
| **P2-07** | Open | scene/overlay只保留最后错误，缺bounded fault history。 |
| **P2-08** | Open | Store无live/retired bytes和高水位telemetry。 |
| **P2-09** | Open | snapshot reader hold time不可观察。 |
| **P2-10** | Open | toolkit save latency/fault未接统一diagnostic bus。 |
| **P2-11** | Open | SDK无declared thread affinity lint/admission。 |
| **P2-12** | Open | reload/close/panic/reentry/scale产品矩阵缺失。 |
| **P2-13** | Open | 跨平台native unload、ABI和long-running leak资格缺失。 |
| **P2-14** | Open | direct/builtin固定owner削弱归因。 |
| **P2-15** | Open | 文档/状态仍可能以descriptor存在冒充feature available。 |

## 6. Canonical资格门

| Gate | 状态 | 当前裁决 |
|---|---|---|
| `EXT-GATE-01` package admission | Partial | manager有in-memory catalog admission，未绑定mounted commit。 |
| `EXT-GATE-02` manifest schema | Partial | package与7类serialized schema存在，family不完整。 |
| `EXT-GATE-03` build identity | Fail | contribution owner不携version/build hash。 |
| `EXT-GATE-04` trust principal | Fail | 无principal/permission/revocation。 |
| `EXT-GATE-05` loading phase | Partial | manager有phase，direct Host安装绕过。 |
| `EXT-GATE-06` capability predicate | Partial | batch/item字符串过滤存在，语义不统一。 |
| `EXT-GATE-07` desired set | Partial | active extension report可近似输入，没有reconciler。 |
| `EXT-GATE-08` mounted set | Fail | 无authoritative mounted generation/receipt。 |
| `EXT-GATE-09` owner generation | Partial | owner string+ticket存在，没有build/instance generation。 |
| `EXT-GATE-10` cross-family prepare | Partial | scene/overlay/command有candidate，未覆盖全部side effect。 |
| `EXT-GATE-11` candidate digest | Fail | 无stable digest。 |
| `EXT-GATE-12` atomic publish | Partial | Store和serialized registry局部原子；Host跨registry非原子。 |
| `EXT-GATE-13` rollback | Partial | candidate失败保持局部旧值；已安装family无逆序rollback。 |
| `EXT-GATE-14` command lease | Fail | command无owner-generation lease。 |
| `EXT-GATE-15` view lease | Fail | manager view注册无revoke lease。 |
| `EXT-GATE-16` scene-mode lease | Fail | registry无owner remove/fence。 |
| `EXT-GATE-17` overlay lease | Fail | provider无owner remove/fence。 |
| `EXT-GATE-18` store ticket | Partial | ticket/keys/counts/revoke report存在，产品未使用。 |
| `EXT-GATE-19` runtime-consumer lease | Fail | prepare/install独立且无unmount receipt。 |
| `EXT-GATE-20` template replacement | Partial | ticket内atomic replace存在，非整owner reload。 |
| `EXT-GATE-21` reader fence | Fail | old Arc reader可无限持有。 |
| `EXT-GATE-22` callback fence | Fail | 无invocation tracking/drain。 |
| `EXT-GATE-23` job cancellation | Fail | extension job owner未建立。 |
| `EXT-GATE-24` quiesce deadline | Fail | 无Quiescing状态与deadline。 |
| `EXT-GATE-25` revoke receipt | Partial | Store局部`RevokeReport`存在，无统一product caller/terminal。 |
| `EXT-GATE-26` leak census | Fail | 无类型、指标或资格测试。 |
| `EXT-GATE-27` Inspector isolation | Fail | can_handle/build仍可锁内执行。 |
| `EXT-GATE-28` field-editor isolation | Fail | factory无supervisor。 |
| `EXT-GATE-29` pane-data isolation | Partial | callback已锁外，但无deadline/cancel/budget/quarantine。 |
| `EXT-GATE-30` native process boundary | Fail | 无可终止worker/process隔离。 |
| `EXT-GATE-31` overlay budget | Fail | 无输出/time budget。 |
| `EXT-GATE-32` scene-mode fault policy | Partial | panic隔离与typed error存在，无generation repair。 |
| `EXT-GATE-33` toolkit save lease | Partial | foreground/capture lease存在，path/panic/fault未闭合。 |
| `EXT-GATE-34` toolkit close lease | Partial | save exclusion和rollback存在，未覆盖callback/job/dirty。 |
| `EXT-GATE-35` dirty lifecycle commit | Fail | Toolkit与DirtyRegistry仍是独立权威。 |
| `EXT-GATE-36` project replacement | Fail | manager row replacement不驱动mount diff。 |
| `EXT-GATE-37` reload repair | Fail | 无drain/reconcile/retry/rollback状态机。 |
| `EXT-GATE-38` snapshot cursor/resync | Partial | bounded count journal和reset存在，无page/bytes/cursor。 |
| `EXT-GATE-39` scale benchmark | Partial | 有少量ignored 1K/layout microbench，无19-family/reload矩阵。 |
| `EXT-GATE-40` cross-platform unload | Fail | 未实现也未执行。 |

## 7. 目标架构与Hard Cutover

```text
Package Catalog Generation
  -> Admission(package, build, principal, phase, capability policy)
  -> DesiredExtensionSet<OwnerGeneration>
  -> ExtensionReconciler(diff desired, mounted)
  -> MountPlan
       command/view/scene/overlay/store/runtime consumer/toolkit/provider receipts
  -> Prepare(all pure declarations; callback probes supervised)
  -> Publish(commit id, candidate digest, manager generation)
  -> MountedExtensionSet + product projections

Disable / Reload / Project Close
  -> Close admission
  -> Quiesce callbacks/jobs/readers(deadline)
  -> Exit active scene/toolkit UI
  -> Revoke reverse dependency order
  -> Drop callback roots outside locks
  -> Terminal receipt + leak census
  -> Native unload only after Quiesced
```

Hard cutover要求：

1. App不得再直接永久安装`editor_plugin_registrations`；registration只能成为manager catalog输入。
2. 删除`editor.extension.direct`和无身份`Builtin`作为多owner归属；所有实例使用`ExtensionOwnerRef(package, build, principal, instance generation)`。
3. `register_editor_extension_owned()`由单函数多处mutation迁移为prepare/publish/rollback plan；发布前不得执行不可逆factory side effect。
4. command/view/scene/overlay/store/runtime consumer返回共同`MountReceipt`，任何family不得维护不可枚举的第二owner cache。
5. `ContributionStore::revoke()`不能作为最终unmount；只能由reconciler在callback/job/reader fence后作为逆序步骤调用。
6. batch/item/activation/principal编译成一个immutable predicate，所有query/factory/callback验证同一owner generation。
7. Inspector、field、pane、scene、overlay、toolkit callback全部进入统一supervisor；锁内只冻结immutable input/plan，锁外执行。
8. Settings/drawer/graph/timeline没有production consumer时必须报告Unavailable；不得以descriptor、projection test或queued文案冒充产品完成。
9. serialized与in-process contribution schema必须由共同声明生成或以exhaustive compiler gate约束；新增family必须同时提供mount/revoke/product/qualification实现。

## 8. 分层重构计划

| Milestone | 内容 | 退出条件 |
|---|---|---|
| M0 | P0锁域与双权威封口 | manager generation成为唯一desired输入；Toolkit/Dirty纯数据事务；Inspector callback锁外RED/Green tests。 |
| M1 | Identity与schema | OwnerRef、build/principal/instance generation、typed capability、统一family schema冻结。 |
| M2 | Mount transaction | 全family prepare、digest、single publish、rollback、MountReceipt完成；删除direct install旁路。 |
| M3 | Callback supervisor | invocation ID、affinity、deadline、cancel、budget、panic/fault/quarantine/stale CAS覆盖所有callback族。 |
| M4 | Quiesce/revoke/unload | disable/reload/project close执行admission close、reader/job/callback fence、逆序撤销、leak census。 |
| M5 | 产品消费 | Settings/drawer/graph/timeline和现有Inspector/scene/overlay接入mounted generation并明确Unavailable。 |
| M6 | Toolkit/document convergence | Toolkit、Dirty、save/autosave/close/pane/provider/job合并为document lifecycle transaction。 |
| M7 | 资格与超越性 | 1/100/1K/10K owner/family、reload storm、fault/hang/native unload、soak、跨平台和同场景跨引擎artifact。 |

M0必须先于继续扩展新family。仅添加更多descriptor、注册函数、fixed projection或microbenchmark会扩大无法卸载的callback表面，不构成工程化进展。

## 9. 逐owner检查台账

| Owner/文件簇 | 已检查的真实实现 | 仍需重构 |
|---|---|---|
| plugin manager snapshot/catalog | active catalog、phase/state、generation、diagnostics | production reconciler、desired/mounted authority、owner generation |
| App/run config/first-party assembly | registration输入与真实startup调用 | 删除direct install，统一进入manager admission |
| Host extension registration | validation、scene/overlay/command candidates、Store ticket | pure MountPlan、single commit、rollback、runtime consumer纳入事务 |
| ContributionBatch/Store | 19 family、namespace/collision、COW snapshot、ticket、bounded count journal | typed schema/predicate、checked IDs、no-op、bounded bytes/page、product revoke |
| EditorExtensionRegistry/materializer/Interface | in-process registry、7类serialized schema、atomic serialized candidate | exhaustive family schema、package all-or-nothing、统一SDK/Host生成 |
| Inspector/field editor | ticket catalog、retained/reflection产品reader、typed declarative surface | priority/specificity、锁外supervisor、owner generation、callback policy |
| template/pane | atomic ticket replacement、pane callback锁外 | owner整批replace、deadline/cancel/budget/quarantine |
| scene mode | candidate registry、factory validation、isolated callback lifecycle | declarative probe、capability、owner remove、active mode exit/fence |
| viewport overlay | owner string、item cap、toggle、panic quarantine | batch predicate、budget、generation reset、repair、remove/fence |
| Settings/localization | typed descriptor/key、locale/cap projection、tests | production Settings window/content/read-write/apply/unmount |
| drawer/graph/timeline | descriptor、Store/materialization/tests | 真实product reader与open/edit/save/compile lifecycle |
| Toolkit registry/save/write authority | checked IDs、save/close lease、immutable snapshot、锁外I/O/drop | Dirty transaction、typed fault、path lease、callback/job fence、incremental snapshot |
| focused tests/bench | collision/revoke/old reader/fault/lease/lock域micro evidence | cross-registry rollback、reconcile/reload/unload/race/hang/10K/soak |
| Unreal/Godot/Fyrox/Bevy/Unity refs | owner remove、module/plugin phase、toolkit/provider scope | 只吸收边界；不能把局部参考当作完整安全协议 |

## 10. 本轮closeout与限制

本轮只完成静态review、本地参考源码对照、Editor50 finding/gate重判、refactor plan、selected-set fingerprint与索引记录；没有修改Editor、Runtime、App、Interface、Cargo或tests，也没有运行动态命令。共享dirty working tree在冻结后仍可能变化，Editor171只对frontmatter与fingerprint所列当前磁盘快照负责。

Editor50只有在5个P0全部关闭、60个P1逐项有实现和真实production consumer/retirement证据、40门全部Pass后才可完成。后续实现应从M0开始：先建立manager-to-Workbench reconciler和跨registry失败RED tests，同时把Inspector callback移出shell锁并把Toolkit/Dirty收敛为纯数据lifecycle transaction；否则继续增加contribution family只会重复当前双权威与不可卸载问题。
