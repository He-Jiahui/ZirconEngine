---
related_code:
  - zircon_editor/src/core/plugin
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/settings/page.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/module_plugins.rs
  - zircon_editor/assets/ui/editor/host/module_plugins_body.zui
  - zircon_runtime_interface/src/editor_contribution.rs
  - zircon_plugins/plugin_sdk/src/editor_contribution.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Editor/PluginBrowser/Source/PluginBrowser/Private/SPluginBrowser.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/PluginBrowser/Source/PluginBrowser/Private/SPluginTile.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/Interfaces/IPluginManager.h
  - dev/godot/editor/plugins/editor_plugin_settings.cpp
  - dev/Fyrox/editor/src/lib.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/package.json
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 06 · Plugin Manager、Enablement、Live Reload、Settings 与 Diagnostics 工程化差距

## 1. 结论

Zircon Editor的插件底层并非临时空壳。`EditorPluginManager`已经有`Discovered/Validated/Loading/Active/Revoking/Disabled/Faulted`生命周期、loading phase、generation snapshot、capability与extension catalog、panic/fault隔离和恢复入口；`EditorPluginPanelSource`还能从同一个manager generation投影source、phase、state、capabilities与diagnostics。Native产品启动也确实把`NativePluginDevelopmentLiveHostBackend`接到`NativePluginHostHandle`，Unload、Hot Reload和debug artifact watcher都不是未连接的测试占位。旧计划中“live backend完全不可用”的判断已经过期，实施不能据此再造第二个loader。

问题在于真实Plugin Manager没有消费这套生命周期authority，而是维护另一份`EditorPluginStatusReport`：其中`enabled`主要来自项目manifest，`load_state`是字符串，native故障类别还靠diagnostic文本片段猜测。面板因此无法表达desired、durably committed、pending restart、loading phase、effective active、faulted、quarantined和recovery中的区别，也无法确认用户动作最终改变了哪个状态。

这条分裂已经造成两项产品级P0。第一，Enable/Disable先改变Editor capability与插件生命周期并发布新状态，外层随后才保存`zircon-project.toml`；保存失败没有回滚或持久的待修复终态，本次会话与下次启动可得到相反结果。第二，Unload/Hot Reload虽然真实调用native host，却不重新发布`EditorPluginStatusReport`；面板缓存以`Arc::ptr_eq`判断代际，因此仅`mark_layout_dirty`仍复用旧投影，成功卸载后仍可能显示`enabled | loaded`。debug watcher的自动reload结果又只写`stderr`，产品完全看不到新generation、失败和恢复动作。

Feature、package与settings authoring同样没有形成工程闭环：一行插件只暴露一个按优先级挑选的feature动作；带点号的qualified package ID会被feature action parser错误拆分；dependency cycle、provider歧义和缺失provider可以只产生diagnostic而仍保存部分启用结果；packaging和target通过无约束轮转修改。Runtime manifest已有version、description、category、maturity、platform、distribution、dependencies和typed options，但主面板几乎不显示。`SettingsPage`跨ABI只含三个展示字段，materializer能把它放进单次registry，随后catalog merge却根本不复制settings pages，全仓也没有产品consumer。

本报告记录2个P0、32个P1、8个P2。没有运行Cargo、真实Editor、native DLL装卸、artifact watcher、磁盘故障、项目文件并发编辑、1000插件可交互面板或跨版本恢复；性能与竞态结论来自同步调用、状态发布与投影代码，不宣称已经完成与Unreal/Fyrox/Godot的同机性能比较。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| core plugin lifecycle/catalog | 35 / 6,318 | E3：manager、phase、snapshot、capability、materialization、isolation与panel source；fingerprint `dd12aa17...0153d6` |
| product Plugin Manager clean set | 59 / 6,195 | E3：status/enablement、action、live host、projection、pane data与retained conversion；fingerprint `27024dd1...2357f` |
| SDK/settings/manifest authoring | 8 / 1,757 | E3：serialized contribution、SDK builder、settings descriptor、extension merge与package metadata；fingerprint `7285c03e...a7d1f` |
| extension showcase clean set | 4 / 1,899 | E3：静态workspace、navigation/feedback/preview actions；fingerprint `66c6da5c...03d344` |
| focused plugin tests | 33 files / 145 test attributes | E2：测试源码已读；未运行Cargo、DLL、watcher或UI |

fingerprint按相对路径排序，将`path + NUL + per-file SHA-256 + LF`串联后再计算SHA-256。它只标识本轮clean阅读集合，不是schema/version ID，也不能替代构建、运行或产品验收。

### 2.2 在途文件隔离

`zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/render_asset_vfx.rs`成文时有其他Session或用户修改。本轮没有用它证明Plugin Manager动作是否可达；静态showcase结论来自clean的`.zui`、`extension_module_navigation/specs/data_production.rs`、`extension_module_feedback/data_production.rs`和`workbench_preview_actions/extensions.rs`。实施前必须重读binding owner，确认这些preview route是否仍暴露给产品导航。

本轮其余核心插件、真实面板、action、status、SDK与manifest证据路径均为clean。工作树中其他大量在途修改没有回退、没有纳入指纹，也没有用于本报告的稳定算法结论。

### 2.3 本轮追踪的产品链

1. Project open/status refresh -> complete builtin/native catalog -> publish `Arc<EditorPluginStatusReport>`。
2. pane read -> `published_plugin_status_report` -> Arc identity cache -> row projection -> retained dynamic controls。
3. click ->字符串action ID解析 -> reload project manifest -> synchronous native discovery/manager mutation -> manifest candidate mutation -> unconditional save。
4. native Unload/Hot Reload -> `NativePluginDevelopmentLiveHostBackend` -> `NativePluginHostHandle` -> optional debug watcher。
5. watcher event -> bounded debounce channel -> worker hot reload ->仅`eprintln!`结果。
6. serialized contribution -> SDK DTO -> host materializer -> registration-local extension registry -> catalog generation merge；本轮特别核对settings page在哪一层消失。
7. runtime package manifest -> status DTO -> pane row；本轮核对metadata、dependency、options与兼容信息的消费情况。

## 3. 已有工程基础，重构时必须保留

### 3.1 生命周期与generation snapshot

- `EditorPluginManager`已经定义发现、校验、分阶段加载、激活、撤销、禁用与故障状态，且每次状态变化发布不可变generation snapshot；目标是让产品面板消费它，不是另造UI状态机。
- `EditorPluginPanelSource`在一个generation内把manager entry和catalog projection按稳定package ID配对，并延迟读取选中插件的完整registration detail；这正是大型catalog的authority入口。
- extension/capability catalog已经有ticket ownership、撤销和fault隔离基础。UI必须把这些typed state投影出来，而不是从manifest布尔值推断实际贡献是否active。

### 3.2 Native live host与watcher基础

- 产品backend的Unload直接调用`unload_editor_plugin`，Hot Reload直接调用`hot_reload_editor_plugin`；debug build还能对唯一artifact建立non-recursive watcher。
- watcher使用容量1的channel合并重复事件并以350 ms debounce，worker持有weak host，backend drop会停止并join线程。这些资源所有权与有界coalescing值得保留。
- hot reload outcome已有diagnostics，watch setup/cleanup failure也能追加diagnostic。缺口是结果没有进入统一事件/状态authority，而不是底层没有结果。

### 3.3 Manifest、catalog与依赖模型

- `ProjectManifest::save`已经使用atomic write fault boundary；问题是文件事务与live lifecycle不是同一个复合提交协议，不能把“单文件原子替换”误当跨层原子性。
- Runtime package manifest已经包含qualified identity、version、SDK API、kind、category、description、supported targets/platforms、maturity、dependencies、interfaces、options、features与distribution。
- feature dependency report能表达provider、capability、blocked feature与diagnostics。重构应提升为prepare/validate/commit结果，而不是再写一套UI依赖解析器。

### 3.4 Retained pane与稳定投影基础

- pane projection以不可变`Arc`作为缓存代际，同一report的1000次稳定读取测试要求不重建投影；正确修复是可靠发布新generation，不是删除缓存。
- action、pane DTO与native host之间已有明确模块边界，便于把字符串command替换为typed request并保留host adapter。
- 主pane使用滚动容器，产品可以在此基础上加入筛选、选择详情与虚拟化，不需要复用静态showcase作为真实authority。

## 4. P0：项目持久化与实际运行态已经发生分裂

### E-PLUGIN-UX-P0-01 · Enable/Disable在项目清单落盘前改变运行态，保存失败没有补偿

证据链：

1. `dispatch_module_plugin_action`先加载manifest并执行project action，最后才无条件调用`context.save()`。
2. native-aware Enable/Disable同步发现插件，随后调用`set_editor_capabilities_enabled`，更新传入manifest并发布新的project plugin status。
3. 外层save失败只写`Plugin action failed`状态行；没有撤销capability/lifecycle、恢复旧published report、写recovery record或标记`PendingPersistenceRepair`。
4. 因而同一会话可能已经激活/撤销插件贡献，磁盘仍保留相反`enabled`值；重启后运行态翻转，当前面板又可能继续显示动作中途发布的状态。

这不是普通错误提示不足，而是authoritative配置与实际执行状态不一致。修复不能仅把save移动到前面：磁盘提交后live activation仍可能失败。必须建立带expected project revision的prepare/commit协议，并为第二阶段失败定义durable `PendingRuntimeApply/RestartRequired/RollbackRequired`终态与恢复动作。

### E-PLUGIN-UX-P0-02 · Unload/Hot Reload成功后不发布新状态代际，面板可持续显示错误运行态

证据链：

1. live action真实调用native host并返回成功message，但不调用任何`publish_project_plugin_status*`。
2. success分支只有`set_status_line`与`mark_layout_dirty`；pane仍读取同一个`Arc<EditorPluginStatusReport>`。
3. `ModulePluginPaneProjectionCache`使用`Arc::ptr_eq`命中旧pane，layout dirty不会让同一Arc失效。
4. `EditorPluginStatus`中的`load_state`又来自上一次native load report，不从当前`EditorPluginManager`generation读取。
5. debug watcher的自动reload结果只写`stderr`，连status line和layout dirty都没有。

结果是成功Unload后仍可能显示`enabled | loaded`并继续提供Unload/Hot Reload按钮；reload失败也可能保持绿色旧状态。这会误导用户继续编辑依赖已卸载代码的对象，属于产品控制面false-green。每个手动/自动live operation必须发布typed lifecycle event和新的管理snapshot，UI按generation更新并保留last-good、operation outcome与repair action。

## 5. P1：插件管理控制面缺口

### 5.1 Authority、事务与状态

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-PLUGIN-UX-P1-01 | core已有`EditorPluginPanelSource`，真实pane却只消费独立`EditorPluginStatusReport`；全仓没有产品consumer读取manager generation row。 | `PluginManagementSnapshot`以manager/catalog generation为主键，manifest selection、native host状态和operation结果作为同一row的明确子状态。 |
| E-PLUGIN-UX-P1-02 | 一个`enabled: bool`同时暗示项目期望、当前会话贡献、下次启动和目标平台选择；没有desired/effective/pending区分。 | 至少表达`desired_selection`、`durable_revision`、`effective_lifecycle`、`pending_transition`、`restart_requirement`和`last_good_generation`。 |
| E-PLUGIN-UX-P1-03 | `Validated/Loading/Revoking/Faulted/Disabled`、loading phase、quarantine/recovery与manager diagnostics不进入主面板。 | UI直接投影typed lifecycle/phase/fault，提供Retry、Disable、Open Diagnostics和恢复原因，不把Faulted显示成manifest-only或enabled。 |
| E-PLUGIN-UX-P1-04 | native load state靠diagnostic是否包含`" entry failed:"`、`"library is missing"`、`"failed to load"`分类；其他兼容/ABI错误落成`manifest only`。 | native admission/loader发布稳定error code、stage、severity、owner和cause chain；本地化文案不能参与控制流。 |
| E-PLUGIN-UX-P1-05 | projection为每一行无条件生成Packaging、Cycle Targets、Unload与Hot Reload action，包括builtin、catalog-only、disabled和非native row。 | authority为每个command返回`Available/Disabled(reason)/Hidden`，执行端再次校验同一generation与capability。 |
| E-PLUGIN-UX-P1-06 | action request只有字符串ID和plugin ID，没有project revision、manager generation、operation ID、expected state或重复提交保护。 | typed `PluginCommandRequest`携project/document identity、expected revisions和idempotency key；stale click返回Conflict并刷新，不作用于新对象。 |
| E-PLUGIN-UX-P1-07 | 纯Unload/Hot Reload也先load并最终save manifest；即使无配置变化仍重写pretty TOML，可能丢注释/格式并覆盖外部编辑。 | live-only command不得触碰project file；配置命令使用compare-and-swap revision、preserving editor或显式canonical format migration。 |
| E-PLUGIN-UX-P1-08 | watcher成功/失败只写`stderr`，没有event stream、通知、面板generation、失败隔离历史或用户可执行恢复。 | watcher向management service提交correlated operation result；通知和pane消费同一typed event，保留去重计数与last error。 |
| E-PLUGIN-UX-P1-09 | 每次enable/feature/packaging/target点击同步`discover_native_plugins`，artifact路径解析也同步扫描；UI链无进度、取消或time budget。 | catalog scan进入后台coordinator，按filesystem generation增量刷新；command只消费稳定snapshot，慢操作有progress/cancel/timeout。 |
| E-PLUGIN-UX-P1-10 | 主pane没有Refresh、scanning/current/stale状态、最后成功时间或“磁盘已变化但未应用”提示。 | 明确catalog freshness与refresh command；watcher overflow、scan failure和partial catalog均是可见typed state。 |

### 5.2 Feature、package selection与dependency

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-PLUGIN-UX-P1-11 | `module_plugin_feature_action`跨全部optional features按三段优先级只选第一项；其余feature没有独立可达动作。 | 选中插件详情列出全部feature，每项显示desired/effective/required/blocked并独立操作。 |
| E-PLUGIN-UX-P1-12 | feature summary用换行拼全部feature，row中却只给固定单行高度；diagnostics同样压入固定文本区域。 | list row只显示摘要/计数，详情区域虚拟化展示feature与diagnostic；文本可展开、复制和定位。 |
| E-PLUGIN-UX-P1-13 | feature action编码为`prefix.plugin_id.feature_id`，parser用第一次`.`拆分；manifest明确支持`prefix.company.name` qualified package ID。 | action payload使用结构化字段或长度前缀/escaping codec；qualified package与feature ID必须round-trip。 |
| E-PLUGIN-UX-P1-14 | dependency helper在missing provider、multiple providers和cycle时追加diagnostic后可继续；已启用的部分dependency仍写入candidate并最终保存，文案仍称“dependencies enabled”。 | solver先生成无副作用plan；任一fatal conflict使整plan不可提交，或用户明确接受带typed unresolved set的方案，不能静默部分成功。 |
| E-PLUGIN-UX-P1-15 | packaging固定在三种strategy间轮转，不读取package允许/default策略、target/platform或distribution约束。 | 由compatibility solver返回合法选项与原因；UI使用菜单/单选并预览导出影响，不对非法策略试错。 |
| E-PLUGIN-UX-P1-16 | target modes只支持client、server、editor、client+editor、empty的固定序列；其他合法组合直接变empty，且忽略supported targets。 | 使用multi-select typed set并与package/module支持矩阵求交；保留任意合法组合，empty/all语义必须显式。 |
| E-PLUGIN-UX-P1-17 | status/pane丢弃version、SDK API、kind、category、description、maturity、platform、distribution、content roots与接口等丰富manifest信息。 | row保留轻量索引字段，selected detail按需读取registration/package detail，显示兼容、来源、路径、版本、作者/分发和内容范围。 |
| E-PLUGIN-UX-P1-18 | DTO虽有dependency status，主row只压成feature summary，没有依赖/被依赖图、provider选择、阻塞路径或disable影响范围。 | dependency authority提供正反向graph和最短block reason；Disable前列出直接/传递dependents及拟执行plan。 |
| E-PLUGIN-UX-P1-19 | `PluginOptionManifest`在Editor产品链无consumer；插件参数无法按type/default/enum/capability约束编辑或分层保存。 | options进入统一settings schema/store，支持project/user/workspace scope、validation、secret policy、restart/reload effect与source-of-value。 |

### 5.3 Settings与插件authoring

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-PLUGIN-UX-P1-20 | materializer能向registration-local registry注册settings page，但`build_editor_extensions`合并views/drawers/menu/inspector/template/importer/scene/graph/timeline/command时完全漏掉`settings_pages()`。 | catalog generation merge必须覆盖所有正式contribution kinds；以exhaustive enum/compile-time match或contract test阻止新增kind静默丢失。 |
| E-PLUGIN-UX-P1-21 | serialized `SettingsPage`与`SettingsPageDescriptor`只有id、display name、category；没有document/controller/data root/schema/store/validation/lifecycle。 | 定义可执行`SettingsPageContribution`：surface、typed schema、read/write store、scope、permission、apply/cancel、restart/reload semantics与ticket ownership。 |
| E-PLUGIN-UX-P1-22 | 全仓没有settings page产品consumer，也没有从Plugin Manager创建/编辑插件package、module、feature或option的真实workflow。 | Settings导航消费active contribution catalog；Plugin authoring提供模板、manifest schema editor、validation、build/test、open folder与版本迁移，不让用户手改所有TOML。 |

### 5.4 产品UX、live reload与规模

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-PLUGIN-UX-P1-23 | 主pane只有“Project plugins”与固定行列表，没有search、category tree、enabled/disabled/faulted/source filter、selection detail或排序。 | 建立可搜索索引、typed filters、category/source tree和master-detail；搜索覆盖name、ID、description、capability与diagnostic。 |
| E-PLUGIN-UX-P1-24 | Enable/Disable没有dependents确认、experimental/beta/maturity确认、platform/SDK incompatibility说明或影响预览。 | command preparation返回impact plan和required decisions；确认后仍以原generation提交，catalog变化则重新准备。 |
| E-PLUGIN-UX-P1-25 | 没有Pending Restart、Restart Now、deferred apply或“当前会话与下次启动不同”的可见状态。 | capability/loader明确声明live-applicable或restart-required；UI持续展示pending delta并提供restart/revert。 |
| E-PLUGIN-UX-P1-26 | 没有authoritative Install/Update/Uninstall、additional search path/source、版本选择、lockfile或download/build进度。 | 由package service管理来源、解析、下载/构建、校验、原子安装与lock；Plugin Manager只做控制面，安全/ABI enforcement继续由`zircon_plugins`owner实现。 |
| E-PLUGIN-UX-P1-27 | 另有名为Plugin Manager的production `.zui` workspace，硬编码三款插件、版本、依赖和`18 installed/3 updates/1 warning`，feedback返回固定queued文案。 | showcase必须与产品route明确隔离并标注fixture，或改为消费唯一management snapshot；不得让真实外观返回虚假成功。 |
| E-PLUGIN-UX-P1-28 | hot reload没有向用户说明哪些selection、command history、property editor、tool state或插件对象被清理/失效，也没有恢复报告。 | reload protocol声明owned-state inventory、quiesce、snapshot/migrate/drop/rebind步骤；outcome列出保留、重置、失败和人工修复项。 |
| E-PLUGIN-UX-P1-29 | operation结果只有瞬时status line；没有per-plugin timeline、correlation ID、阶段耗时、完整diagnostic、Retry、Copy Details或Open Artifact/Manifest。 | 有界operation journal与selected detail共享typed results；敏感路径按策略展示，错误可复制/导出并关联日志。 |
| E-PLUGIN-UX-P1-30 | 每行固定约112 px并最多并排六个文字按钮；窄pane没有overflow menu/wrap，长feature/diagnostic文本可能裁切。 | compact row只保留状态与primary toggle，次要命令进入菜单/详情；定义min/max宽度、text wrap/ellipsis/tooltip和响应式gate。 |
| E-PLUGIN-UX-P1-31 | status report与pane为全部插件构造完整String/feature/diagnostic树；缓存只优化同Arc重读，没有visible-range materialization或selected detail lazy load。 | snapshot分轻量row index与按需detail，列表虚拟化；scan、projection、clone、node count与frame time都有1/100/1000规模预算。 |
| E-PLUGIN-UX-P1-32 | 主pane没有键盘批量操作、焦点恢复、screen-reader状态语义、localization或高对比度故障表达；action主要靠紧密文字按钮。 | command/selection/focus是稳定模型；状态不只靠颜色，所有动作有可访问名称、禁用原因、键盘路径与本地化资源。 |

## 6. P2：诊断、测试与维护债

| ID | 当前差距 | 必须补齐 |
|---|---|---|
| E-PLUGIN-UX-P2-01 | diagnostics和status message大量是自由字符串，缺code、stage、severity、plugin generation与cause chain。 | 建立跨loader/manager/UI的structured diagnostic envelope，字符串只作为rendered message。 |
| E-PLUGIN-UX-P2-02 | action route通过`.`拼接且无version/escaping/fuzz contract；普通plugin action也没有中央codec。 | typed route payload与round-trip/property/fuzz测试覆盖Unicode、qualified ID、长ID和恶意输入。 |
| E-PLUGIN-UX-P2-03 | watcher固定350 ms且只按路径事件触发，没有文件稳定性/hash、指数退避、burst telemetry或连续失败熔断。 | reload admission检查artifact稳定/identity，失败backoff并可暂停；记录coalesced/dropped/retry计数。 |
| E-PLUGIN-UX-P2-04 | 没有“live lifecycle已变但manifest save失败”的fault-injection回归，也没有补偿失败/重启恢复测试。 | M0加入write fault、rollback fault、process kill与restart reconciliation矩阵。 |
| E-PLUGIN-UX-P2-05 | 没有断言Unload/Hot Reload/watch event发布新Arc/generation并让pane离开旧load state的端到端测试。 | 用fake native host + real manager/pane cache测试success/failure/stale click/retry全链。 |
| E-PLUGIN-UX-P2-06 | feature tests使用简单owner ID，没有qualified package round-trip、provider ambiguity、cycle原子失败或任意target set保存测试。 | 补solver plan与command codec的table/property tests，禁止部分mutation成为成功合同。 |
| E-PLUGIN-UX-P2-07 | 现有1000插件测试只测稳定Arc投影不重建，没有首次构建、滚动、搜索、detail、diagnostic storm、watch storm和内存峰值gate。 | 建立cold/warm p95/p99、allocation、node count、input latency与worker backlog长期基线。 |
| E-PLUGIN-UX-P2-08 | 旧计划/测试中的“backend unavailable”描述会误导实施，而当前产品已接native live backend；showcase又使用production命名。 | 文档与fixture明确snapshot日期、test double和product backend；实现后回填旧记录为superseded/fixed。 |

## 7. 与参考引擎的差异及适用边界

| 参考 | 可验证能力 | 对Zircon的约束 | 不应照搬的部分 |
|---|---|---|---|
| Unreal Plugin Browser | `SPluginBrowser`有name/description search、category和enabled filters、additional directory、显式Refresh、Pending Restart/Restart Now；`SPluginTile`显示version/author/description/maturity并在Disable时处理dependent plugins。 | Zircon必须把catalog浏览、pending state、依赖影响和刷新做成产品合同；不能只在每行塞操作按钮。 | Unreal多数enable change以restart生效；Zircon若主张更强live reload，必须增加quiesce/migration/recovery证据，不能只复制restart UI。 |
| Godot EditorPluginSettings | 递归发现addon config，校验name/author/version/description/script；toggle后读取effective state，拒绝时回滚checkbox；Recovery Mode明确提示插件不会运行，并有create/edit入口。 | Zircon需要有效状态回读、UI rollback、safe/recovery mode和插件authoring，而不是把manifest bool当执行成功。 | Godot脚本插件模型与Zircon native ABI不同，不能从其轻量加载路径推导DLL卸载安全。 |
| Fyrox Editor | hot reload前清理可能持有plugin object的command stack/selection，按source type移除property editor，reload后重新注册。 | Zircon reload outcome必须覆盖插件拥有的Editor状态和贡献重建，不能只报告DLL调用成功。 | Fyrox实现细节不等于Zircon ticket/capability架构；应复用Zircon ownership snapshot实现同等结果。 |
| Bevy App/Plugin | `Plugin`定义build/ready/finish/cleanup与uniqueness，PluginGroup支持确定性组合。 | 生命周期阶段、ready与cleanup是基础语义参考，适合校验Zircon manager phase是否完整。 | 该源码是静态App composition，不是Editor产品Plugin Manager、安装器或native hot-reload authority。 |
| Unity Graphics checkout | HDRP `package.json`包含name/version/description/dependencies/keywords/samples，Editor/Runtime `.asmdef`分层并有version defines。 | Zircon详情页至少应消费自己已经更丰富的package metadata与Editor/Runtime边界。 | 当前checkout不是Unity Package Manager源码，不能据此断言其安装、解析、回滚或UI内部实现。 |

## 8. 目标架构与唯一权威

### 8.1 PluginManagementService

```text
Catalog scanner / project manifest / native host / EditorPluginManager
                            |
                            v
                PluginManagementService
          prepare -> durable commit -> runtime apply
             |             |              |
             +------ typed operation journal ------+
                            |
                            v
        generation-bound PluginManagementSnapshot
                            |
             +--------------+--------------+
             |                             |
      Plugin Manager UI              Settings UI
```

每个row至少包含：

- stable package identity、source、version、maturity与compatibility；
- catalog/manager/project/native generations；
- desired selection、durable project revision与effective lifecycle；
- loading phase、fault/quarantine、pending operation和restart requirement；
- command availability及typed disabled reason；
- lazy detail handle：modules、features、dependencies、options、diagnostics、paths与history。

`EditorPluginStatusReport`和`EditorPluginPanelSource`不能作为两个长期平行authority。实施应选manager snapshot为生命周期真值，吸收manifest/native operation projection后硬切产品consumer；旧DTO只可作为迁移adapter并在同一里程碑删除。

### 8.2 复合提交而非假原子事务

配置与live runtime无法靠一个文件rename获得真正原子性。目标状态机应显式为：

```text
Idle
  -> Preparing(expected project + manager + catalog revisions)
  -> AwaitingDecision(optional impact confirmation)
  -> PersistingManifest
  -> ApplyingRuntime
  -> Applied
     |-> RestartRequired
     |-> PendingRuntimeRepair
     |-> RollingBackManifest -> RolledBack / RollbackRepairRequired
```

prepare阶段无副作用地产生完整plan；manifest提交带expected digest/revision；runtime apply失败后不能假装原子回滚，而要按operation journal执行补偿并留下可恢复终态。重启时reconciler读取durable selection、journal和实际host状态，恢复到可解释结果。

### 8.3 Reload与owned-state协议

每次reload必须有唯一operation ID和以下阶段：admit -> quiesce callbacks/jobs -> enumerate owned Editor/runtime state -> revoke contributions -> unload -> validate/load new artifact -> migrate/rebind -> republish snapshot。任何阶段失败都保留last-good信息、故障stage、哪些状态已清理以及Retry/Disable/Restart动作。

selection、undo command、Inspector editor、tool mode、menu/view、background task和native object若能持有plugin-owned type，必须注册ownership/invalidation hook。不能只依赖DLL层“卸载成功”。这与`zircon_plugins/01`的ABI/foreign ownership enforcement相邻，但本报告拥有Editor产品状态、反馈与恢复闭环。

### 8.4 Settings与options

一个settings contribution必须同时声明typed schema、surface或generic form strategy、store scope、default/source-of-value、validation、permission、apply mode和restart/reload effect。`PluginOptionManifest`应成为package option schema输入，不另造Editor-only option真值；插件自定义页面只覆盖呈现与高级command。

## 9. 硬切重构范围

1. 禁止真实Plugin Manager继续直接消费manifest-derived字符串状态；切到统一management snapshot。
2. 删除`native_load_state`基于message substring的控制流，loader/admission输出typed status code。
3. 删除字符串拼接的feature action payload，全部command使用typed request与revision token。
4. live-only action不再load/save project manifest；配置action不再先改live state后裸save。
5. feature dependency从“边遍历边改candidate并把fatal当diagnostic”切到无副作用plan + 原子admission。
6. packaging/target不再cycle；由compatibility authority给出合法option set。
7. `build_editor_extensions`对正式contribution kind采用exhaustive merge，settings page进入active catalog并有产品consumer。
8. 静态Plugin Manager showcase退出产品route，或完全改接统一snapshot；固定成功文案不得保留在产品动作。
9. 所有manual/automatic reload outcome发布新generation；`mark_layout_dirty`不能替代数据发布。
10. 旧`EditorPluginStatusReport`、旧action ID parser和重复pane projection在迁移里程碑结束时删除，不长期双写。

## 10. 测试先行的依赖序里程碑

### M0 · P0一致性封口

- 先写manifest write failure、runtime apply failure、compensation failure与restart reconciliation测试。
- 写Unload/Hot Reload/watch success/failure必须发布新generation并刷新pane cache的全链测试。
- 暂时保留现有UI，但每个动作必须产生typed operation终态；false-green与session/disk分裂先归零。

### M1 · 唯一management authority

- 定义`PluginManagementSnapshot/Row/DetailHandle/CommandRequest/OperationResult`。
- manager lifecycle、manifest selection、native host和catalog freshness并入一个generation协调器。
- 主pane硬切新snapshot，删除string load-state判断和无条件action eligibility。

### M2 · Package、dependency与配置solver

- feature dependency改为prepare-only plan，加入provider选择、cycle/fatal classification和影响预览。
- packaging/target/platform/version compatibility返回合法options。
- project manifest写入带revision/CAS，live-only action移除文件写入。

### M3 · Settings与插件authoring

- 修复settings page merge omission，定义可执行surface/schema/store contract。
- 接入`PluginOptionManifest`的generic editor与source-of-value。
- 提供Create/Edit/Validate/Build/Open Folder流程和真实diagnostics，不依赖静态fixture。

### M4 · 产品Plugin Manager与规模

- 实装search/filter/category/master-detail、dependency graph、operation timeline、restart/recovery提示和responsive commands。
- row index与detail lazy load、visible range virtualize；建立1/100/1000插件与diagnostic storm预算。
- 键盘、焦点、screen reader、本地化和高对比度验收。

### M5 · Reload recovery与长期gate

- quiesce/owned-state inventory/migrate/rebind协议覆盖selection、undo、Inspector、tool与background task。
- watcher稳定性、backoff、overflow、artifact replace、host shutdown和连续失败熔断。
- Windows真实DLL、process kill、disk full/denied、external manifest edit和跨版本journal恢复矩阵进入CI/nightly。

## 11. 产品级验收门

1. Enable/Disable任一阶段失败后，disk desired state、effective manager state与UI row一致，或显示一个durable typed repair state；不得互相矛盾。
2. manifest写失败不会留下不可见的active/revoked capability；重启reconciler能确定恢复。
3. Unload成功后同一帧序列发布更高generation，pane不再显示Loaded，也不再提供非法Unload。
4. manual/watch Hot Reload成功、失败、quarantine均进入同一operation timeline和row state。
5. stale generation click不会作用于刷新后同ID不同artifact，返回Conflict并重新投影。
6. builtin/catalog-only/disabled/nonreloadable插件不生成非法live action，disabled reason可见。
7. qualified package ID与feature ID可无损round-trip；property/fuzz测试覆盖点号、Unicode和长ID。
8. dependency missing、ambiguity或cycle不会保存部分selection；影响plan明确列出所有变化。
9. Disable有transitive dependents预览和确认，取消后无任何manifest/runtime mutation。
10. packaging/target UI只显示package、platform与distribution允许的组合，任意合法set可保存并读回。
11. version、description、category、maturity、source、SDK compatibility、platform和distribution在详情可查。
12. 每个optional feature独立可见/可操作，并显示required、blocked reason、provider和effective state。
13. settings page contribution经过native DTO -> materializer -> active catalog ->产品mount完整闭环，ticket撤销后确定性卸载。
14. plugin options按typed schema验证，project/user/workspace scope和restart/reload effect可见且round-trip。
15. Create/Edit/Validate Plugin产生真实manifest与diagnostic；产品route不返回fixture成功文案。
16. search/filter/category在1000插件下输入p95满足既定预算，滚动节点数与viewport成比例。
17. diagnostic storm不会无限扩张row高度或内存；detail可复制完整cause chain并关联operation ID。
18. narrow pane、200% DPI、长本地化文本和键盘导航下按钮不重叠、不裁切关键状态、焦点不丢失。
19. reload会报告被保留/清理/迁移的selection、history、property editor和tool state；失败后可Retry/Disable/Restart。
20. Windows真实DLL、artifact atomic replace、watch burst、project file external edit、disk full和process kill均有自动化证据与可恢复终态。

## 12. 依赖、owner与后续复核

- `zircon_plugins/01`拥有package/native ABI、签名/信任、foreign ownership与分发enforcement；本报告拥有Editor控制面、复合提交、状态反馈、settings和reload authoring recovery。
- `zircon_runtime_interface/01`拥有stable ABI DTO/version/handle边界；新增typed contribution和operation event必须先在该边界版本化。
- `zircon_editor/02`拥有统一document/save/recovery authority；project manifest CAS、operation journal与shutdown reconciliation应复用其持久化和故障注入规则。
- `zircon_editor/05`拥有Inspector/property editor；reload对plugin-owned field editor/customization的撤销、fallback和state invalidation必须联合验收。
- 实施前重读所有active owner和dirty binding文件；本报告是2026-08-16源码快照上的canonical gap记录，不授权覆盖其他Session在途实现。

本轮只做review与文档，没有修改Rust产品代码，也没有把静态源码证据提升为运行通过声明。
