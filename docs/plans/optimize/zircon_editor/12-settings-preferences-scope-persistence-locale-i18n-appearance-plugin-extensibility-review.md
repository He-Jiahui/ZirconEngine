---
related_code:
  - zircon_editor/src/core/settings
  - zircon_editor/src/core/i18n
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/extension/store
  - zircon_editor/src/core/plugin/materializer.rs
  - zircon_editor/src/core/jobs/quota_settings.rs
  - zircon_editor/src/core/notifications/presentation.rs
  - zircon_editor/src/scene/viewport/controller
  - zircon_editor/src/ui/v2_design_tokens.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme.rs
  - zircon_editor/src/ui/workbench/floating_window.rs
  - zircon_editor/assets/i18n
  - zircon_editor/assets/ui/editor/components/workbench/floating/workbench_preferences.zui
  - zircon_editor/assets/ui/editor/theme/editor_tokens.zui
  - zircon_editor/assets/ui/theme
  - zircon_runtime/src/ui/surface/render
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime_interface/src/ui/design_tokens.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/10-notification-center-toast-decision-history-actions-retention-accessibility-diagnostic-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Developer/Settings/Public/ISettingsModule.h
  - dev/UnrealEngine/Engine/Source/Developer/Settings/Public/ISettingsSection.h
  - dev/UnrealEngine/Engine/Source/Runtime/DeveloperSettings/Public/Engine/DeveloperSettings.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigCacheIni.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/ConfigContext.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Internationalization/Internationalization.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Internationalization/TextLocalizationManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Styling/AppStyle.h
  - dev/godot/editor/settings/editor_settings.h
  - dev/godot/editor/settings/editor_settings.cpp
  - dev/godot/editor/settings/editor_settings_dialog.h
  - dev/godot/editor/settings/editor_settings_dialog.cpp
  - dev/godot/core/config/project_settings.h
  - dev/godot/core/string/translation_server.h
  - dev/godot/core/string/translation_domain.h
  - dev/godot/editor/themes/editor_theme_manager.h
  - dev/Fyrox/editor/src/plugins/settings.rs
  - dev/bevy/crates/bevy_feathers/src/theme.rs
  - dev/bevy/crates/bevy_feathers/src/dark_theme.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/CoreRenderPipelinePreferences.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/ICoreRenderPipelinePreferencesProvider.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Properties/PropertiesPreferencesProvider.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Documentation~/add-custom-graphics-settings.md
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 12 · Settings、Preferences、Scope Persistence、Locale/i18n、Appearance 与插件扩展工程化差距

## 1. 结论

Zircon Editor的settings core已经有一组值得保留的工程基础：严格的lowercase dotted key、typed value/schema、User/Project/Session优先级、不可变ArcSwap snapshot、有界change log、project path canonicalization、versioned envelope、temp write + flush + rename，以及基于Runtime keyed I/O lane的有界异步提交。i18n也实现了捕获locale后再投影复合文本、English fallback、事件队列backpressure/resync；appearance能从唯一settings snapshot更新retained host和V2 token projection。这些不是占位代码。

但系统尚未形成工程级“配置产品”。最严重的三个断点是：

1. `SettingsAuthority::set/clear`先发布新内存snapshot并同步hot-apply，然后才由调用方自行决定是否提交持久化。生产中只有Scene Viewport的三个Project snap值调用`SettingsPersistenceService::submit`；User locale、design tokens、keymap和job quotas没有通用durable commit owner。提交拒绝或worker失败也不会回滚、标dirty或阻止“修改成功”的产品反馈，重启可丢配置。
2. Preferences只有一个静态ZUI壳和`FloatingWindow::preferences()`设计描述；生产没有caller、command/menu route、settings枚举、控件绑定、apply/reset、scope/origin、错误或插件页消费。当前11个内置setting和插件`SettingsPageDescriptor`都无法通过产品界面管理。
3. locale setting声称选择Editor presentation语言，但248个生产Editor ZUI资产仍有3,201处`text/label/title/display_name/placeholder/aria_label`字面属性，embedded bundle每种语言只有54个key；生产没有`EditorTopic::i18n()` subscriber，绝大多数shell也不调用translation service。语言切换最多更新少量notification/command文本，主界面保持英文或形成混合语言。

本报告记录3个P0、58个P1、12个P2，给出M0-M6重构路线与30个验收门。Layout Profile、dock/window restore、Workspace State及其迁移单独进入Editor 13，避免把“用户偏好”与“文档/窗口会话状态”混成同一事务。没有修改生产代码。上一轮同一工作树的`zircon_editor --lib`测试编译已在617.2秒后被239个既有test-build错误和122个warning阻断，本轮没有重复相同Cargo lane；结论来自逐文件静态调用链、全部settings/i18n测试源码、UI资产inventory和参考引擎源码，不得描述成动态测试通过。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| settings core与tests | 16 / 3,937 / 135,352 | E3：authority、registry、definition、snapshot、change log、store、startup、persistence、scope/page/defaults及34个test attributes；fingerprint `c41339da...17197e23` |
| i18n core、tests与bundles | 9 / 1,199 / 44,638 | E3：locale、catalog、service、sink/resync、macro/error、两份bundle及10个test attributes；fingerprint `aa970691...1702e95` |
| 产品与appearance接入闭包 | 51 / 14,456 / 533,428 | E3：context、viewport persistence、plugin page materialization、notification translation、retained/V2 appearance和Preferences shell；fingerprint `03640bd8...91173fb` |
| Editor ZUI literal inventory | 248 / 39,353 / 2,498,794 | E2 inventory：3,201个可见/可访问文本属性，77个资产显式导入active editor token资产；fingerprint `1b9887f2...5ed87922` |
| selected combined scope | 312 / 55,961 / 3,108,922 | 当前工作树去重集合；fingerprint `ff3c73da...02e93a28` |

fingerprint按相对路径排序，将`path + NUL + per-file SHA-256 + LF`拼接后计算SHA-256；它只标识本轮阅读集合，不是settings schema、locale resource、theme generation或兼容性hash。

产品闭包由生产文件中`SettingsAuthority/SettingsPersistenceService/SettingsPageDescriptor/EDITOR_LOCALE_KEY/EditorI18nService/EditorDesignTokens/install_editor_v2_design_tokens/apply_host_appearance_from_tokens/FloatingWindow::preferences`引用集合构成，排除dedicated test目录。ZUI literal计数是属性出现次数，不等于去重后的英文短语数量；其意义是证明当前资产编译链没有把主shell文本建模为localizable identity。

### 2.2 在途文件与验证隔离

本轮成文时settings core、i18n core、bundles、Preferences资产、`ui/v2_design_tokens.rs`和host paint theme主文件在scoped `git status`中未显示修改。仓内其他Editor/theme/asset-editor文件存在并行在途改动，本报告没有回退、格式化、暂存或提交。因工作树仍持续变化且动态test binary有已知编译阻断，实施前必须重取source、fingerprint、setting inventory、ZUI literal inventory和测试结果，故`source_recheck_required=true`。

证据等级：

- E3：`set/clear -> snapshot -> hot subscriber -> optional caller submit -> keyed I/O worker -> store write -> shutdown fence`逐函数闭环。
- E3：16个settings文件、9个i18n文件/bundle和全部44个test attributes逐项阅读，0 ignored。
- E3：Preferences descriptor/asset、plugin settings page registration、locale builder sink、notification projection、V2/host appearance同步链闭环。
- E2：248个Editor ZUI资产文本属性、theme imports和17个runtime surface renderer default-token fallback文件inventory。
- 未覆盖：真实Preferences窗口、设置磁盘满/断电、两个Editor进程竞争、网络/只读工程、真实locale字体与RTL、主题切换截图、屏幕阅读器、十万setting规模、插件热卸载和跨版本升级。上述均进入验收门，不冒充已验证能力。

### 2.3 本轮追踪的生产链

1. `EditorBuilder`创建registry，先注册7个settings默认值和4个job quota，再从User store加载，最后构造`SettingsAuthority`、`SettingsPersistenceService`和`EditorI18nService`。
2. authority只允许构造时已有definition；`set/clear`持锁修改registry，发布新snapshot，解锁后同步调用唯一change subscriber。subscriber只负责从snapshot同步i18n，不负责持久化。
3. persistence service不是authority observer。caller必须拿到`SettingChange`后显式`submit`；worker执行时读取authority当前整个scope layer并写一个完整document。
4. 生产唯一submit路径位于Scene Viewport controller：它先改变Project snapshot，再在存在store时提交ticket并自行保留/重试；项目settings加载、store和ticket生命周期也被该controller持有。
5. User locale mutation会同步改变i18n active locale，但没有通用User save；design tokens在retained host tick中被轮询，变化后更新V2 projection、host paint theme并mark presentation dirty。
6. plugin materializer只把`SettingsPageDescriptor(id, display_name, category_path)`放进extension registry；没有注册`SettingDefinition`，也没有生产consumer把page装入Preferences。
7. `FloatingWindow::preferences()`只返回modal/content asset描述。该constructor在生产无caller；ZUI内容只有General/Layout两行和Preferences标题。
8. locale catalog内嵌`en`和`zh-CN`，各54个同集合key；exact locale缺失时退English，再缺失就显示raw key。主ZUI shell不经该catalog。
9. builder为i18n安装message-bus sink，但生产没有对应topic subscriber；普通change的零subscriber dispatch仍被映射为Delivered，resync分支才报告NotConfigured。
10. appearance当前有settings snapshot、process-global V2 `RwLock` projection、process-global host `ArcSwap` snapshot和thread-local paint scope等派生状态；静态theme资产与17个runtime renderer还保留各自默认token fallback。

## 3. 已有工程基础，重构时必须保留

### 3.1 Typed registry、scope与snapshot

- `SettingsKey`拒绝空segment、首尾点和非lowercase ASCII segment，持久化反序列化也走validated constructor。
- `SettingSchema`对Int/Float/String/Enum等基础值执行范围或字节校验；scope写入必须匹配definition owner。
- effective value优先级明确为Session > Project > User > default，Session禁止持久化。
- `SettingsSnapshot`通过ArcSwap发布，内置large payload使用Arc复用，无关setting变化不会重建design tokens/keymap/MRU。
- change log同时受4,096 entry、256 KiB和5分钟约束，cursor落后时明确要求full snapshot。

### 3.2 Layer加载与有界I/O

- persistent layer先完整验证，再整体替换；坏文件不会半应用一部分key。
- project root经过统一ProjectPaths，active project source与write preparation绑定，旧project worker不能把新project layer写入旧文件。
- persistence lane有1,024 entry/8 MiB retained budget、typed ticket、cancel-before-start、retry、fence和shutdown guard；调用线程不直接做文件I/O。
- store使用versioned text envelope、`create_new`临时文件、`write_all`、`sync_all`和同目录rename；非Windows还sync parent directory。

### 3.3 Locale一致投影与backpressure

- 复合notification presentation先捕获一个locale，再翻译title/message/options，避免同一projection混用两代语言。
- locale transition记录settings generation，拒绝较晚到达的旧snapshot。
- locale event queue有32项/64 bytes上限，慢sink时coalesce到最新locale并要求resync；现有并发测试覆盖FIFO和late generation。
- English fallback必需，duplicate locale、非法key和空translation会在catalog构造时失败。

### 3.4 Appearance热应用基础

- retained host每tick读取authority snapshot，只在design token Arc变化时更新appearance。
- V2文档准备阶段把active `EditorDesignTokens`注入style resolver；host palette、metrics和typography从同一token payload投影。
- host metrics对NaN、infinite、negative geometry有局部fallback，scale factor也有finite-positive保护。
- theme token资产已经覆盖77个Workbench ZUI文件，说明集中token namespace具备真实adoption基础。

这些基础应收敛进统一Settings Transaction、Preferences Registry、Localization Platform和Theme Generation，不应在旁边再造另一套用户配置文件或global singleton。

## 4. P0：耐久提交、Preferences产品与locale承诺

### E-SET-P0-01 · 内存修改与耐久提交分离，API成功可在重启后丢失

`SettingsAuthority::set/clear`在返回前已经更新registry、发布snapshot并调用hot subscriber；它既不提交persistent scope，也不返回“尚未持久化”的状态。`SettingsPersistenceService`虽然在context中始终存在，却不是subscriber。生产`.submit`只出现在Scene Viewport三个Project snap值路径，User locale、design tokens、keymap overrides和4个job quota均无通用保存owner。

即便Viewport路径也先改变内存，再尝试lane admission；没有store时直接`Ok(())`，admission失败或worker terminal failure都不回滚snapshot。失败ticket只由viewport controller在下一次snap修改前重试。Editor shutdown只能fence已经admit的工作，无法发现从未submit的User/Project mutation。因此“set成功/hot apply成功”与“重启后仍存在”是两个脱节的事实。

目标必须引入scope transaction/commit coordinator：persistent mutation要么在有界journal中成为durably admitted commit并返回typed receipt，要么保持明确dirty/degraded状态并向产品显示失败；hot apply、disk commit、rollback和restart-required必须共享transaction identity。禁止每个feature controller自行决定是否保存整个settings document。

### E-SET-P0-02 · Preferences是不可达静态壳，无法管理任何真实setting

`FloatingWindow::preferences()`只声明modal overlay和ZUI路径，生产调用搜索为空。`workbench_preferences.zui`固定720-960 x 480-680，只包含General/Layout两个静态list row和标题，没有search、控件、binding、scope、origin、apply/cancel/reset、validation、restart、error或accessibility workflow。`ui/preferences`目录为空。

`SettingsAuthority`又不公开通用definition/value枚举；`SettingsSnapshot`只暴露硬编码内置slot。插件贡献的`SettingsPageDescriptor`虽然能进入extension snapshot，生产没有`.settings_pages()` consumer，descriptor也不携页面资产/controller或setting IDs。所以即使临时加一个菜单命令，窗口仍无法构造可编辑内容。

目标必须是可直接打开的Preferences产品：统一container/category/section/page model，按schema生成或接受受控custom editor，支持搜索、scope与source解释、staged/live apply、reset/import/export、dirty/error/restart状态、keyboard/reader contract和插件owner lease。资产存在、设计测试通过或有constructor都不等于产品功能完成。

### E-SET-P0-03 · locale设置只覆盖极小文本岛，切换后主Editor形成混合语言

embedded catalog只有`en`和`zh-CN`，每份54个key；实际生产translation consumer主要是notification presentation，其他直接translate调用位于service/macro实现或builder测试。248个Editor ZUI资产含3,201个文本属性字面值，Preferences自己的Preferences/General/Layout也写死英文。主shell没有localization identity、编译时提取或运行时localized binding。

builder安装的i18n event sink把change发布到`EditorTopic::i18n()`，但生产没有subscriber。和logging相同，普通locale change在零subscriber时仍可映射为Delivered，只有resync路径识别NotConfigured。retained tick只观察design tokens，不观察locale generation；没有全shell invalidation、layout remeasure、font fallback rebuild或RTL reflow。因此User locale hot-sync成功并不表示产品界面已切换。

目标必须把localizable text identity放进ZUI/compiler/runtime DTO，建立text revision与surface invalidation，所有window/pane/dialog/plugin surface在一个locale generation下重投影并重新布局。CI必须阻止新增用户可见literal绕过资源系统；在此之前不得把`editor.language.locale`描述为全Editor语言切换。

## 5. P1：authority、persistence、Preferences、i18n与appearance缺口

### 5.1 Settings authority、schema与插件生命周期

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-SET-P1-01 | 生产只注册11个setting：7个Editor默认项和4个job quota；绝大多数Editor行为仍由常量、layout state或feature局部字段控制。 | 建立全Editor settings adoption catalog，逐owner登记key、scope、default、schema、apply mode、persistence和产品页。 |
| E-SET-P1-02 | registry只在authority构造前可注册；authority没有register/unregister/batch/owner generation。 | 动态definition registry，registration lease绑定builtin/plugin package generation，卸载原子撤销并保留orphan value。 |
| E-SET-P1-03 | plugin `SettingsPageDescriptor`和真实`SettingDefinition`完全分离。 | plugin contribution同时声明definition set、page/layout、localization bundle、capability与owner lease，admission全有或全无。 |
| E-SET-P1-04 | authority/snapshot没有generic definition、effective value、layer value和origin枚举。 | immutable query snapshot公开bounded paged descriptors、effective/source/default/override/restart/validation状态。 |
| E-SET-P1-05 | `SettingsPageDescriptor`只有三段字符串，不含asset/controller、setting IDs、order、icon、keywords、scope、actions或owner。 | versioned page contract，显示字段使用localization keys并支持schema-generated或custom content provider。 |
| E-SET-P1-06 | authority只有一个可替换同步subscriber，当前被i18n独占。 | 多subscriber registry，每个consumer按generation/cursor订阅，拥有owner、queue、backpressure、resync和health。 |
| E-SET-P1-07 | subscriber在setter线程同步执行；slow/panic consumer会延迟或终止mutation caller。 | commit发布与consumer apply解耦，panic containment、deadline、per-consumer receipt和失败降级明确。 |
| E-SET-P1-08 | comment要求subscriber不得写回，但类型系统不阻止递归set；测试只验证回调可重入读取。 | transaction phase和reentrancy policy显式化；递归mutation进入后续batch或返回typed cycle error。 |
| E-SET-P1-09 | Project settings load/store/ticket生命周期被SceneViewportController拥有。 | project/session authority在project-open/close层绑定store；任意Project setting都共享同一commit coordinator。 |
| E-SET-P1-10 | `user_layer_load`在authority构造后固定，坏文件无法在运行期修复、reload或更新health。 | mutable source health、reload/retry/recover operation及generation-bound diagnostic。 |
| E-SET-P1-11 | registry revision与部分theme generation用`saturating_add`，到`u64::MAX`后identity冻结。 | checked epoch+counter或明确terminal exhaustion；绝不静默复用generation。 |
| E-SET-P1-12 | replace persistent layer逐key增加revision并逐次构造snapshot，成本随变更key数放大。 | validated batch一次计算effective diff、一次发布snapshot和一个batch transaction identity。 |
| E-SET-P1-13 | DesignTokens/KeymapOverrides/MRU specialized schema只做variant匹配，schema层不验证内部不变量。 | 每个structured value有versioned validator、size/depth预算、canonicalization和migration；invalid payload不能进入authority。 |
| E-SET-P1-14 | schema只覆盖Bool/Int/Float/String/Enum/Color/Chord和三个special type。 | 增加path、duration、bytes、list/map/set、optional、secret/reference及custom editor codec，并定义跨平台表示。 |

### 5.2 Persistence、migration、recovery与并发

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-SET-P1-15 | ticket记录change generation，但worker运行时序列化authority“当前整个layer”；receipt不能证明写入的是该generation。 | admission时冻结canonical layer snapshot或commit delta；receipt返回实际durable generation/digest/path。 |
| E-SET-P1-16 | lane key含scope/target/key，而每个job写整个scope document；不同key不能coalesce，会重复整文件写。 | 以document/scope为lane key，debounce并合并dirty keys，按最新完整snapshot做一次atomic commit。 |
| E-SET-P1-17 | 没有dirty/unsaved/committing/failed/conflicted状态，caller只能轮询ticket。 | process-wide persistence health和per-scope commit ledger，Preferences/status/close flow可查询并采取动作。 |
| E-SET-P1-18 | worker失败只发`tracing::warn`和terminal code；除viewport私有队列外没有产品错误面。 | 接入Diagnostic Journal与Notification decision，携path、scope、generation、retry/recover/open-folder action。 |
| E-SET-P1-19 | shutdown使用无deadline fence并调用blocking `guard.wait()`；卡住的I/O可无限延长Editor退出。 | bounded shutdown deadline、cancel/abandon policy、emergency spool和明确non-durable close decision。 |
| E-SET-P1-20 | 没有跨进程lock、file generation/CAS、external edit watch或three-way merge。 | 每文件writer lease + on-disk generation/digest；检测外部修改并提供reload/merge/overwrite decision。 |
| E-SET-P1-21 | 没有backup、last-known-good、corrupt quarantine、temp crash sweep或recovery journal。 | 原子generation slots、LKG、损坏源隔离、启动恢复和有配额的temp清理。 |
| E-SET-P1-22 | strict unknown setting key使整个layer失败；禁用/卸载插件遗留key即可阻断所有内置配置加载。 | known值与orphan extension payload分区；保留未知versioned owner data并在owner恢复时重新验证。 |
| E-SET-P1-23 | schema version为1，但唯一0->1 migration明确拒绝；旧格式只能让用户重建文件。 | 有fixture/golden的连续migration chain、downgrade/read-only策略和迁移前backup。 |
| E-SET-P1-24 | 文件名为`settings.toml`，内容却是JSON versioned envelope，工具、用户和source-control review预期矛盾。 | 扩展名与canonical format一致，或采用真实TOML并保留schema header；格式由公开contract定义。 |
| E-SET-P1-25 | `read_to_string`在decode前无文件size预算，恶意或损坏大文件会先完整分配。 | open metadata cap、bounded reader、entry/string/depth/node预算，超限产生可恢复diagnostic。 |
| E-SET-P1-26 | invalid Project source会抑制写入，invalid User source只保留startup error；未来User submit可能覆盖事故现场。 | 所有invalid source进入read-only/quarantine，未经显式recover/replace不得覆盖。 |
| E-SET-P1-27 | 没有per-setting alias/deprecation、split/merge transform、platform override或owner version。 | schema registry持stable setting ID与migration metadata，支持rename、type conversion和owner升级。 |
| E-SET-P1-28 | 没有secret redaction/encryption、managed policy、source-control/read-only checkout或machine/profile layer。 | 分离User/Machine/Project/Session/Managed/Secret stores，明确优先级、权限、审计和团队协作策略。 |

### 5.3 Preferences产品、编辑事务与可访问性

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-SET-P1-29 | 没有menu/command/palette/API route打开Preferences或定位具体section。 | 注册稳定command与`show(container/category/section/key)`路由，支持deep link和diagnostic跳转。 |
| E-SET-P1-30 | 静态General/Layout导航不来自registry，内置11项和插件page都不可见。 | category tree由query snapshot生成，稳定order、owner、scope和empty-state；增量处理插件load/unload。 |
| E-SET-P1-31 | 没有按schema生成的checkbox/number/enum/color/chord/structured editor。 | shared property-editor factory复用Inspector typed contracts，同时限制custom widget capability。 |
| E-SET-P1-32 | 没有search、keyword、description、setting key或owner过滤。 | Unicode-normalized indexed search，结果显示category breadcrumb、匹配字段和bounded query budget。 |
| E-SET-P1-33 | 不显示effective value来自default/User/Project/Session/Managed哪一层。 | 每项显示origin、override stack、可编辑scope和“remove override”动作。 |
| E-SET-P1-34 | 没有live/staged/restart apply模型，`requires_restart`只是bool且无消费者。 | 每setting声明apply policy；UI聚合pending restart subsystem并支持apply/cancel/revert。 |
| E-SET-P1-35 | 没有reset item/section/all、import/export、backup或diff preview。 | capability-aware section actions，执行前预览affected keys/scope，执行后给durable receipt。 |
| E-SET-P1-36 | 没有validation message、commit progress、write failure、conflict或recovery UI。 | inline typed validation + persistent top-level health banner + Diagnostic Journal linkage。 |
| E-SET-P1-37 | modal contract没有focus trap、keyboard navigation、reader labels、large-text/reflow或virtualization证据。 | 完整dialog accessibility contract、responsive min constraints、虚拟化长列表和真实Windows截图/reader gate。 |

### 5.4 Locale、text identity与本地化平台

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-SET-P1-38 | locale enum硬编码`en/zh-CN`，没有从catalog/source registry生成。 | available locale由已admit bundles和policy snapshot生成，setting validator引用同一catalog generation。 |
| E-SET-P1-39 | 当前两bundle key集合恰好相同，但catalog不验证parity、required coverage或unused key。 | build-time extractor + bundle validator，按target定义required/optional key并报告missing/stale。 |
| E-SET-P1-40 | missing translation静默显示raw key，没有counter、owner或source diagnostic。 | typed missing-text record、bounded once-per-key diagnostic、fallback provenance和CI gate。 |
| E-SET-P1-41 | locale parser只是近似BCP47，script大小写、extension/private-use、alias和canonical registry不完整。 | 使用成熟locale/ICU数据，保存canonical language-script-region-extensions identity。 |
| E-SET-P1-42 | fallback只有exact locale -> English，不支持language/script/region parent和用户fallback chain。 | prioritized culture chain，例如zh-Hant-HK -> zh-Hant -> zh -> configured fallback -> source。 |
| E-SET-P1-43 | translation value只是普通字符串；notification参数用简单named replace。 | ICU/Fluent级message format，支持plural/select/gender、escaping和typed arguments。 |
| E-SET-P1-44 | 没有number/date/time/duration/unit/currency/collation/case mapping服务。 | locale data service统一格式化、排序、搜索与输入解析，禁止各feature自行`format!`用户文本。 |
| E-SET-P1-45 | catalog是两份编译期TOML，没有target/domain、plugin bundle、DLC/pack、hot reload或owner collision策略。 | localization source registry，按namespace/domain/priority/owner generation加载、卸载和异步刷新。 |
| E-SET-P1-46 | locale没有direction、writing system、font fallback或glyph coverage metadata。 | culture snapshot携LTR/RTL、font fallback chain、shaping/line-break policy并触发text cache rebuild。 |
| E-SET-P1-47 | 唯一sink同步运行在locale setter线程，且message-bus生产consumer缺席、零consumer可假Delivered。 | 多sink异步revision broadcast，receipt区分Accepted/Applied/NoConsumer/Failed，surface按cursor resync。 |
| E-SET-P1-48 | 没有pseudo-localization、text expansion、mirroring、missing glyph、locale switch screenshot或reader测试。 | en-XA/ar-XB等自动门，覆盖所有window/pane/dialog/plugin和极端文本长度。 |

### 5.5 Appearance、theme generation与资源一致性

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-SET-P1-49 | 整个`EditorDesignTokens`作为一个SettingValue，任意小改动都替换大payload并整文件写。 | versioned theme profile/resource；settings只存选中theme与少量override，细粒度diff可审计。 |
| E-SET-P1-50 | settings schema对DesignTokens无内部校验；internal API可注入NaN、infinite、负尺寸、空font或无效关系。 | authority admission前完整validate/canonicalize；所有consumer共享同一结果而非各自静默fallback。 |
| E-SET-P1-51 | host metrics只保护部分字段；V2 token injection、runtime fallback、palette和其他density字段不共享同一validator。 | centralized validated `ThemeSnapshot`，任何projection都不再重新解释invalid value。 |
| E-SET-P1-52 | settings snapshot、V2 global RwLock、host global ArcSwap、thread-local scope各有独立状态/generation。 | 单一ThemeGeneration携immutable host/V2/text/icon/font projections，frame capture时原子绑定。 |
| E-SET-P1-53 | process-global appearance不表达window/display/profile owner，多窗口或不同DPI monitor切换只能共享全局token。 | theme profile与scale/display context分离；每window capture同一base theme及自己的validated scale。 |
| E-SET-P1-54 | 248个ZUI中只有77个显式导入editor token资产，legacy base/material/strict主题并存；`editor_unreal_dark.zui`无生产资产引用。 | theme dependency graph和lint，明确canonical root、fallback与deprecated assets，禁止漂移/死主题。 |
| E-SET-P1-55 | 17个runtime surface render模块各自`OnceLock`构造`workbench_dark()` fallback，缺token时会稳定退回硬编码视觉。 | render command必须携resolved style/theme generation；missing token显式diagnostic而非悄悄换主题。 |
| E-SET-P1-56 | 没有theme catalog、preset clone、preview、import/export、rollback或插件token namespace。 | 受版本管理的theme package与preview transaction，owner/collision/compatibility规则明确。 |
| E-SET-P1-57 | 没有contrast、high-contrast OS mode、color-vision、reduced transparency/motion和minimum target size验证。 | accessibility theme policy和自动contrast/geometry gate，允许OS/user override。 |
| E-SET-P1-58 | theme change只mark presentation dirty，没有font atlas/icon raster/GPU resource重建receipt或失败回滚。 | staged theme apply barrier：解析、字体/图标/GPU准备成功后一次publish；失败保留旧generation并显示原因。 |

## 6. P2：质量、诊断与维护缺口

| ID | 当前差距 | 建议收敛 |
|---|---|---|
| E-SET-P2-01 | `SettingsPageDescriptor::new`不验证空display name，category只检查空segment。 | validated constructor，显示名改localization key，ID/category使用稳定type。 |
| E-SET-P2-02 | `available_locales()`按BTree key字典序返回，没有native display name、region或推荐顺序。 | locale descriptor含native/localized name、script/region、completion与pack source。 |
| E-SET-P2-03 | locale event bytes预算按短locale字符串长度，实际Arc/queue/sink持有未计入。 | 采用保守resident accounting并公开high-water。 |
| E-SET-P2-04 | dropped/resync/failed counters使用saturating add，无overflow终态或epoch。 | checked counter或epoch rollover，diagnostic保留overflow事件。 |
| E-SET-P2-05 | temp文件identity依赖pid、wall-clock nanos和32次attempt，没有session/generation/digest。 | commit journal分配稳定transaction temp identity，启动可归属和清理。 |
| E-SET-P2-06 | Preferences固定英文`display_name`和control IDs，但没有localization/accessibility mapping。 | asset compiler要求display/accessibility text identity并生成审计清单。 |
| E-SET-P2-07 | fixed Preferences尺寸与240px导航不能证明小窗口、DPI、CJK和长语言适配。 | responsive tracks、min-content约束、滚动和多DPI screenshot matrix。 |
| E-SET-P2-08 | setting description只有label/description/category，没有docs URL、examples、unit或risk。 | optional help metadata、unit/precision、examples和owner documentation link。 |
| E-SET-P2-09 | 没有settings access/change telemetry，难以定位慢apply、频繁写或失败热点。 | privacy-aware counters/latency/bytes/coalesce/failure metrics，不记录secret value。 |
| E-SET-P2-10 | 测试大量验证源码合同，却没有property/fuzz测试覆盖key/schema/document/locale parser。 | property-based与fuzz corpus，约束panic、allocation和round-trip。 |
| E-SET-P2-11 | settings/i18n错误文本直接拼接英文，无法进入统一diagnostic code与localized presentation。 | stable error code + structured context，显示层按locale翻译。 |
| E-SET-P2-12 | 注释仍引用Editor17/Plan11等历史编号，无法作为长期公开架构语义。 | 用owner/module/schema名称替代计划编号，迁移历史放文档。 |

## 7. 与参考引擎的可验证差异

| 参考 | 仓内源码可验证能力 | Zircon当前差异 | 应吸收的原则 |
|---|---|---|---|
| Unreal | `ISettingsModule`按container/category/section注册object或custom widget，可Show/Unregister；`ISettingsSection`有CanEdit/Save/Reset/Import/Export/Status和delegates；DeveloperSettings支持auto registration/change；localization manager有text revision、culture change、prioritized cultures、sync/async resource source和live table。 | Zircon page与definition断开、无viewer/section操作、无dynamic owner；locale只有两份exact bundle和单sink。 | settings contribution、viewer、save policy和localization source/revision必须是显式模块合同。 |
| Godot | EditorSettings提供真实property list/default/hint/order/save/change；EditorSettingsDialog有search、section、advanced、shortcut、override、debounced save、窗口bounds；TranslationServer/Domain提供locale/fallback/plural/domain/pseudolocalization；theme manager集中重建。 | Zircon Preferences无product controller，settings layer无可枚举metadata；无plural/domain/pseudo/RTL和theme build barrier。 | 统一property authority驱动可搜索UI，locale/theme改变触发全局有代际的重建。 |
| Fyrox | Settings plugin真实注册菜单、打开浮动窗口，用reflection Inspector生成字段、分组、search、default和OK，并热应用renderer quality。 | Zircon已有Inspector/settings schema却没有把两者连接，也无入口/搜索/default/apply。 | 先完成最小但真实的end-to-end settings product，再扩充高级policy；不要停在静态资产。 |
| Bevy | Feathers把theme建模为Resource和stable token component；resource change会更新所有参与的background/border/text，missing token给醒目warning/error color。 | Zircon部分资产使用token，但多套projection/fallback使缺token可静默退默认，无法证明统一generation。 | token consumer显式opt-in、change propagation集中、missing token可观测。Bevy该模块不代表完整Editor Preferences。 |
| Unity Graphics | CoreRenderPipelinePreferences通过type discovery收集provider，按DisplayInfo排序、聚合keywords并由SettingsService打开固定User path；graphics settings group有version、pipeline applicability和category order。 | Zircon plugin只贡献三字符串page，无provider discovery/UI invocation/keywords/order/version/applicability。 | provider discovery、排序、搜索、scope和version属于扩展合同，不应由host硬编码。 |

没有任何单一参考同时解决本文全部目标。Unreal的config历史复杂度、Godot的global singleton、Fyrox的即时reflection mutation、Bevy当前theme属性范围和Unity provider反射成本都不能直接复制。目标是吸收可验证的ownership、transaction、extension和revision原则，再按Zircon的Runtime task、versioned schema与retained UI边界实现。

## 8. 目标架构

### 8.1 Settings Registry 与 Query Snapshot

建立一个process-owned `EditorSettingsRegistry`：

- definition使用stable `SettingId`、owner package/generation、scope policy、schema version、default、apply policy、presentation keys和editor factory。
- registration通过lease和atomic batch完成；卸载后definition撤销，但持久化orphan保留并隔离。
- immutable query snapshot可按container/category/section/page分页，返回effective value、每层override、origin、dirty、validation、restart和health。
- dynamic plugin contribution同时admit definition、page、localization和theme token extension，不允许半注册。

### 8.2 Transactional Commit Coordinator

persistent mutation不再是裸`set + 可选submit`：

1. caller开始transaction，提交一个或多个typed changes及expected generation。
2. authority完整validate/canonicalize，生成immutable candidate snapshot和document digest。
3. commit coordinator按scope/document coalesce，durably admit journal后返回`Accepted` receipt。
4. apply policy决定立即hot apply、staged apply或restart pending；consumer按transaction generation报告Applied/Failed。
5. worker以冻结snapshot写generation slot，fsync/rename后发布Durable receipt。
6. admission/write/apply任一失败进入可见dirty/degraded/conflicted状态；需要时回滚旧snapshot或保留明确pending状态。
7. external writer用file generation/CAS检测，进入reload/merge/overwrite decision，不做last-writer-wins猜测。

### 8.3 Preferences Product Host

Preferences使用真实command和deep link打开。左侧category由registry生成，右侧通过shared property editor渲染schema或受权custom page；顶部有搜索和scope/profile选择，底部只在staged模式显示Apply/Cancel。每项显示effective source、reset override、restart/validation状态；全局banner显示persistence conflict/recovery。插件页在owner unload时安全撤销并保留用户数据。

### 8.4 Localization Platform

ZUI和Rust presentation都保存`LocalizedTextId(namespace,key,source)`，asset compiler提取资源并拒绝未豁免literal。Localization Registry按target/domain/owner加载versioned bundles，使用成熟locale/message-format库提供culture fallback、plural/select、number/date/unit/collation和direction。`LocalizationSnapshot`携text revision、locale chain、font fallback和RTL；surface在一个revision下重建text/layout/accessibility。

### 8.5 Theme Generation

Settings只选择versioned theme profile和override layer。Theme compiler先合并builtin/project/plugin/user tokens，执行完整geometry/color/font/icon/contrast验证，准备host/V2/text/icon/GPU projections；所有结果属于同一个`ThemeGeneration`。window/frame capture immutable generation，准备失败不发布半套theme。process base theme与per-window DPI/high-contrast context分离。

## 9. 分阶段重构路线

### M0 · Contract Freeze 与可观测基线

- 冻结11个现有setting、scope、文件路径、schema和caller matrix；记录所有constants/local state候选。
- 增加ZUI literal、theme import、translation key、persistent mutation/submit caller lint报告。
- 给现有authority、persistence、i18n和theme projection增加generation/health diagnostics，不改变产品行为。
- 建立当前格式golden、坏文件、超大文件、并发/slow sink和theme invalid corpus。

### M1 · Settings Transaction 与 Durable Commit

- 引入batch transaction、candidate snapshot、document-level coalescing和durable receipt。
- 移除feature-ownedsubmit；Project/User由统一project/session composition root绑定store。
- 增加dirty/failed/conflicted health、bounded shutdown和diagnostic/notification路径。
- 增加file generation/CAS、LKG/quarantine/temp recovery、size budgets和真实migration。

### M2 · Dynamic Registry 与 Plugin Contribution

- authority支持owner-bound atomic register/unregister和query snapshot。
- 扩展schema、structured validators、alias/deprecation/migration和apply policy。
- 合并SettingsPage与SettingDefinition contribution；plugin disable/uninstall保留orphan data。
- 将keymap、jobs、viewport、appearance、locale首先迁移为新contract的golden owners。

### M3 · Preferences End-to-End Product

- 注册command/menu/palette/deep link并实现真实modal/nonmodal lifecycle。
- 构建category/search/editor/origin/reset/apply/restart/error/recovery workflow。
- 复用Inspector property editors但隔离transaction、permission和custom widget capability。
- 完成keyboard、reader、DPI、small-window、long-text和大规模virtualization验证。

### M4 · Localization Platform Cutover

- 在ZUI/compiler/runtime引入LocalizedTextId和text revision；建立extraction/parity lint。
- 接入成熟locale与message format，支持fallback/plural/format/domain/plugin bundles。
- 全shell、pane、dialog、notification、command、plugin surface迁移，删除可见literal豁免。
- 加入pseudo、RTL、font fallback、missing glyph和全窗口截图/reader矩阵。

### M5 · Theme Profile 与 Atomic Appearance

- 将large DesignTokens setting迁移为versioned theme profile + overrides。
- 建立完整validator、theme compiler和单一ThemeGeneration。
- 收敛base/material/strict/unreal-dark/token asset图，消除runtime hardcoded fallback漂移。
- 增加preview/rollback、multi-window/DPI、high contrast、font/icon/GPU resource barrier。

### M6 · Adoption、性能与删除旧路径

- 迁移其余Editor feature constants/local preferences和所有插件settings。
- 删除SceneViewport settings ownership、静态Preferences导航、旧page descriptor和单sink旁路。
- 以10k definitions、100k search terms、快速连续修改、slow disk、two-process、locale/theme storm做性能与故障门。
- 完成格式升级/降级、旧项目、只读/source-control工程和crash recovery release gate。

## 10. 验收门

1. 任意persistent `set/clear`都必须返回transaction/commit receipt；不存在绕过统一commit coordinator的生产caller。
2. lane admission失败时产品不得报告saved；snapshot必须回滚或显示可查询dirty/degraded状态。
3. worker write失败可在Preferences、Notification和Diagnostic Journal中定位scope/path/generation并重试。
4. User locale/design tokens/keymap/job quota修改后强制终止进程，重启只观察到已确认durable generation。
5. multi-key burst只产生bounded document commits，ticket durable generation与实际文件generation/digest一致。
6. 两个Editor进程竞争同一User/Project文件时必须检测冲突，不得静默last-writer-wins。
7. 0->current及至少一个rename/type split migration有golden fixture、backup和失败恢复；future version进入只读策略。
8. 未安装插件遗留unknown key不阻断内置settings加载，插件恢复后可重新claim并验证数据。
9. 超大文件、超深JSON、超长key/value在分配预算内拒绝，进程RSS不随输入无界增长。
10. shutdown有硬deadline；slow/hung I/O不会无限阻塞退出，未durable状态有明确operator decision。
11. Preferences可从menu、command palette和deep link打开并定位指定setting。
12. Preferences能枚举并编辑当前全部11个setting，显示正确scope、effective origin、default和restart policy。
13. plugin动态注册/卸载definition+page+bundle是原子的，窗口打开期间卸载不崩溃、不留下可执行dangling callback。
14. schema editor覆盖bool/int/float/string/enum/color/chord/structured value，非法输入不进入authority。
15. Search覆盖label/description/key/category/owner/keyword，10k definitions下有固定latency和allocation门。
16. apply/cancel/reset item/section/all/import/export都有transaction receipt和undo/recovery语义。
17. keyboard-only、screen reader、200% DPI、420px窗口和最长locale文本下无重叠、失焦或不可达控件。
18. ZUI/Rust localization extractor覆盖所有用户可见surface；未经批准的literal会使required validation失败。
19. 切换locale后所有已打开window/pane/dialog/plugin surface在一个text revision下更新并重新layout。
20. zero i18n consumer绝不报告Delivered；每个surface按cursor应用或resync，slow consumer不阻塞setter。
21. BCP47 canonicalization与language-script-region prioritized fallback通过golden vector。
22. plural/select、number/date/duration/unit和escaping在en/zh及至少一个复杂plural locale通过fixture。
23. plugin/domain bundle的load/unload/collision/fallback有owner lease与deterministic priority测试。
24. pseudo-localization、RTL mirroring、CJK/Arabic font fallback、missing glyph和text expansion有产品截图门。
25. invalid/NaN/negative/out-of-range theme token在authority admission前失败，任何projection不观察到半有效值。
26. host、V2、text/icon/font/GPU consumers在同一frame观察同一ThemeGeneration；准备失败保留旧theme。
27. legacy theme asset graph有唯一canonical root；missing token产生diagnostic，17个runtime renderer不再静默使用独立default palette。
28. 多窗口、跨DPI monitor、OS high-contrast、reduced motion/transparency和theme preview/rollback有真实Windows验证。
29. settings/i18n/theme core的property/fuzz/fault tests无ignored，且test binary真实运行；test attribute数量不冒充通过数。
30. 实施前后重取本报告四个scope fingerprint、setting/translation/literal/theme-import inventory；差异必须被计划或review显式解释。

## 11. 测试与证据策略

- 单元/属性：key、schema、scope precedence、batch diff、generation exhaustion、locale canonicalization、fallback、message formatting、theme validator。
- 持久化故障：admission full、slow disk、permission、disk full、rename失败、crash points、corrupt primary/LKG、temp orphan、shutdown deadline。
- 并发：multi-thread set、reentrant consumer、consumer panic、multi-process CAS、project switch与queued commit、plugin unload与open page。
- 产品：open/search/edit/apply/cancel/reset/restart/conflict/recover，覆盖mouse/keyboard/reader/DPI/long text。
- 本地化：extractor parity、missing/unused、pseudo、RTL、font fallback、plural/format、dynamic plugin domain和全surface text revision。
- appearance：theme compile/preview/rollback、single generation、multi-window/DPI、GPU/font/icon rebuild失败和contrast。
- 性能：10k definitions、100k searchable terms、1k burst mutation、large structured theme、locale/theme switch全shell rebuild，报告p50/p95/p99与allocation high-water。
- 兼容：现有version 1 User/Project fixtures、未来version read-only、disabled plugin orphan、只读/source-control工程和跨版本回退。

当前动态状态仍是“未执行”：上一轮lib-test在编译期被239个既有错误阻断。本报告的34个settings test attributes和10个i18n test attributes仅表示源码inventory，不能用作绿色证据。修复共享test-build后，必须先运行focused core/fault tests，再启动真实Editor验证Preferences、locale/theme和Windows filesystem行为。

## 12. 完成定义

只有同时满足以下条件，Editor Settings/Preferences/Locale/Appearance首轮重构才可关闭：

- persistent mutation、hot apply和durable commit属于一个可追踪transaction，不再由feature自行拼接；
- Preferences是真实可达、可搜索、可编辑、可恢复、可扩展且可访问的产品，而不是静态设计资产；
- plugin可以原子贡献和撤销settings/page/localization/theme扩展，unknown data在生命周期变化中不丢失；
- locale切换覆盖全部用户可见surface并携统一text revision、fallback、format、direction和font策略；
- theme切换在所有host/V2/text/icon/GPU consumer中以单一generation发布，失败可回滚；
- migration、external conflict、corrupt source、slow/full disk、crash、shutdown和跨版本恢复都有动态证据；
- required Windows tests、真实窗口截图/可访问性验证和性能门通过，结果绑定当前source/build fingerprint；
- 旧feature-owned submit、静态Preferences导航、single-sink假delivery、literal UI文本和漂移theme fallback被删除，而不是长期兼容保留。

在此之前，Zircon可以称为“已有typed settings、bounded persistence和locale/theme projection基础”，不能称为具备Unreal/Godot级工程化Editor Preferences与本地化平台。
