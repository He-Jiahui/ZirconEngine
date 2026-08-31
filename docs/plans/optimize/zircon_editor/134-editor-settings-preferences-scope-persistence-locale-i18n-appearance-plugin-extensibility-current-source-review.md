---
related_code:
  - zircon_editor/src/core/settings
  - zircon_editor/src/core/i18n
  - zircon_editor/assets/i18n
  - zircon_editor/src/ui/settings
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/context/builder
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/jobs/quota_settings.rs
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/core/editor_event/types.rs
  - zircon_editor/src/core/extension/settings_page_projection.rs
  - zircon_editor/src/core/extension/store/batch.rs
  - zircon_editor/src/core/extension/store/model/lifecycle.rs
  - zircon_editor/src/core/extension/store/model/snapshot.rs
  - zircon_editor/src/core/plugin/materializer.rs
  - zircon_editor/src/core/notifications/presentation.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_apply_command.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_construction.rs
  - zircon_editor/src/ui/host/editor_event_execution/dispatch.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/settings_projection.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/settings_window_actions.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/workbench_surface/option.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/settings_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/popup_state.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes/settings.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes/node.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_settings_window.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_settings_window
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/settings_window
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection/defaults.rs
  - zircon_editor/src/ui/v2_design_tokens.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme
  - zircon_editor/src/ui/retained_host/host_contract/settings_window_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/popup_rows/settings.rs
  - zircon_editor/assets/ui/editor/components/workbench/floating/workbench_preferences.zui
  - zircon_editor/assets/ui/editor/windows/workbench_window.zui
  - zircon_editor/assets/ui/editor/theme/editor_tokens.zui
  - zircon_runtime/src/ui/surface/render/buttons.rs
  - zircon_runtime/src/ui/surface/render/chrome.rs
  - zircon_runtime/src/ui/surface/render/collection_rows/shared.rs
  - zircon_runtime/src/ui/surface/render/command_palette.rs
  - zircon_runtime/src/ui/surface/render/dialog.rs
  - zircon_runtime/src/ui/surface/render/divider.rs
  - zircon_runtime/src/ui/surface/render/drag_overlay.rs
  - zircon_runtime/src/ui/surface/render/dropdowns.rs
  - zircon_runtime/src/ui/surface/render/feedback/colors.rs
  - zircon_runtime/src/ui/surface/render/notification_center.rs
  - zircon_runtime/src/ui/surface/render/popup_rows.rs
  - zircon_runtime/src/ui/surface/render/progress.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls.rs
  - zircon_runtime/src/ui/surface/render/selection_controls.rs
  - zircon_runtime/src/ui/surface/render/skeleton.rs
  - zircon_runtime/src/ui/surface/render/sliders.rs
  - zircon_runtime/src/ui/surface/render/text_fields.rs
  - zircon_runtime_interface/src/editor_contribution.rs
  - zircon_plugins/plugin_sdk/src/editor_contribution.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/10-notification-center-toast-decision-history-actions-retention-accessibility-diagnostic-integration-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/optimize/zircon_editor/107-editor-localization-string-table-culture-translation-fallback-pseudo-preview-current-source-review.md
  - docs/plans/optimize/zircon_editor/123-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-current-source-review.md
  - docs/plans/optimize/zircon_editor/124-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-current-source-review.md
  - docs/plans/optimize/zircon_editor/126-editor-builtin-view-window-descriptor-catalog-content-provider-capability-template-localization-current-source-review.md
  - docs/plans/optimize/zircon_editor/127-editor-workbench-shell-autolayout-constraint-language-responsive-region-binding-geometry-current-source-review.md
  - docs/plans/optimize/zircon_editor/130-editor-command-registry-keymap-menu-palette-context-routing-remote-automation-current-source-review.md
  - docs/plans/optimize/zircon_editor/131-editor-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/132-editor-notification-center-toast-decision-history-actions-retention-accessibility-diagnostic-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/133-editor-logging-diagnostic-journal-output-console-status-routing-retention-export-current-source-review.md
  - docs/plans/optimize/zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/73-runtime-ui-style-theme-token-cascade-selector-pseudo-state-invalidation-transition-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/83-runtime-localization-internationalization-locale-culture-message-format-plural-number-date-string-table-resource-fallback-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/17/failure-2026-07-30-editor-settings-persistence-and-hot-projection.md
  - docs/plans/zircon_editor/editor/06/failure-2026-08-05-settings-page-localization-contract-hardcut.md
  - docs/plans/zircon_editor/editor/12/failure-2026-08-05-plugin-settings-page-localization-contract.md
  - docs/plans/zircon_editor/editor/08/failure-2026-07-23-settings-registry-keymap-user-layer-migration.md
  - docs/plans/zircon_editor/editor/10/failure-2026-08-08-project-document-settings-authority-legacy-entry.md
  - docs/plans/zircon_editor/editor/13/failure-2026-07-23-settings-registry-script-build-batch-window-migration.md
  - docs/plans/zircon_editor/editor/14/failure-2026-07-23-settings-registry-job-category-quota-migration.md
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
refreshes: docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
doc_type: current-source-review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
evidence_captured_at: "2026-08-26T13:16:40+08:00"
---

# 134 - Editor Settings、Preferences、Scope Persistence、Locale/i18n、Appearance 与插件扩展当前源码复核

## 1. 结论

Editor12识别的基础没有退化，当前源码还增加了几项真实进展：`SettingsAuthority`现在发布带通用definition catalog、per-key effective source和typed builtin slot的不可变snapshot；内置definition由11项增至12项；change log与Runtime11 I/O lane继续保持双预算；Settings窗口已有`editor.settings.open`命令、Edit菜单、event、retained-host open链；插件`SettingsPage`和`LocalizationBundle`已升级为versioned contribution，按owner ticket原子admit/revoke；appearance在tick中检测token handle变化，更新V2/host projection并mark presentation dirty。旧报告关于“窗口不可达”和“插件页仍是三段裸字符串”的证据已经过时。

但这些改进尚未组成工程级配置产品。最严重的断点仍是：

1. persistent mutation仍是`authority.set/clear`先发布内存和hot apply，再由feature caller选择是否`persistence.submit`。生产只有Scene Viewport三个Project snap值提交；User locale、appearance、keymap、autosave和四个job quota没有统一durable owner。更严重的是，project transition、错误active path或invalid project source可让store跳过实际写入却返回`Ok(())`，ticket可能终态`Succeeded`而磁盘没有对应generation。
2. Settings窗口虽然可达，12个builtin definition和plugin page metadata也已进入native category/entry projection、clip-bounded paint、hit test与category selection，但row仍是只读metadata：没有effective value/source、typed editor、mutation、apply/reset、search或错误面。窗口只在open时capture；catalog currentness不表达value-only generation，生产也没有持续重投影，因此打开后的窗口不会随value变化更新。
3. i18n覆盖仍是小型presentation岛。两份bundle各67个key，插件bundle已有owner lifecycle，但248个Editor ZUI仍有3,207个直接引号文本属性；生产未发现`EditorTopic::i18n()`订阅者。locale变化不会以统一text revision重投影全部window/pane/dialog、重排版、刷新font fallback或处理RTL。

本轮保留原有3个P0、58个P1和12个P2，并新增`E-SET-P1-59`追踪“suppressed write仍报告成功”。当前状态为：P0 `1 Open / 2 Partial / 0 Closed`，P1 `50 Open / 8 Partial / 1 Closed`，P2 `10 Open / 2 Partial / 0 Closed`。本轮只做current-source review，没有修改生产代码。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 集合 | 文件 / 行 / 非空行 / bytes | tests / ignored | fingerprint |
|---|---:|---:|---|
| settings core | 21 / 4,507 / 4,067 / 154,400 | 41 / 0 | `723a53c2aa1a7ba47b38b5a7dcdc5d23e75619d6db70e3cf55077b031b9e8648` |
| i18n core与bundles | 10 / 1,488 / 1,327 / 54,603 | 11 / 0 | `4e89706a7be774291f92a44f47fd72af6be83b79d47e27b9a5963f8d9ccac50e` |
| product、extension与appearance闭包 | 73 / 22,281 / 20,752 / 780,055 | 64 / 0 | `76784dde99ceeca1b2b81472073fdfb6138f602870dc943c22e707ddba64cf3d` |
| Editor ZUI inventory | 248 / 39,484 / 34,803 / 2,504,587 | 0 / 0 | `eb4341b258d78fdcc9633c8bc4f0e439fa8ec3904fdce0f5888c65bb348dddd5` |
| selected source union | 349 / 67,207 / 60,433 / 3,459,455 | 116 / 0 | `dfd2740a538053fac56bdc9043d8e17306386e99f8c8349a927c66fa4e0a4e4a` |
| reference engines | 23 / 10,193 / 8,617 / 435,618 | 0 / 0 | `1fc2309a3ccc00ef2a13724ddd296ec5a5a09b3ff2657dbb066bbdb386e0a98a` |
| plan sources | 25 / 7,009 / 5,616 / 703,888 | 5 / 2 | `b67a569eab0c5534be560443ff4a843edaf54cd274b5b94e26384202933ee0c9` |
| total evidence union | 397 / 84,409 / 74,666 / 4,598,961 | 121 / 2 | `447f37fd7ebaff102bb18a12d723c317cad2190a411468eaa3c5ede585efd5b7` |

fingerprint按相对路径排序，对`path + NUL + per-file SHA-256 + LF`清单再做SHA-256。它只标识本轮阅读集合，不是schema、document、localization或theme兼容hash。product集合由frontmatter当前生产/测试链和17个实际出现`EditorDesignTokens::workbench_dark()`的runtime render文件组成；Editor ZUI inventory另行覆盖全部248个资产，selected source union已去重。最终快照捕获于`2026-08-26T13:16:40+08:00`；成文期间Settings surface有并行写入，因此不再追逐后续漂移，实施前按`source_recheck_required`重取。

文本inventory采用“属性名边界 + `= "`”统计`text/label/title/display_name/placeholder/aria_label/aria_labelledby/aria_describedby`，当前为3,207处直接字面值；更宽松的assignment candidate计数为3,636。74个ZUI显式提及`editor_tokens.zui`。两份embedded bundle均为70行、67个translation key；runtime render中17个文件共有19处独立dark-token fallback。

### 2.2 在途源码与证据等级

成文时settings、i18n、Preferences ZUI、extension/plugin、viewport、retained host和appearance相关源码均存在共享工作树修改或未跟踪文件。本报告读取的是这些当前文件，没有回退、格式化、暂存或提交它们。相关source可能继续变化，因此`source_recheck_required=true`；实施前必须重取fingerprint、inventory、测试和真实窗口证据。

- E3：`set/clear -> registry diff -> ArcSwap snapshot -> synchronous subscriber -> optional caller submit -> Runtime11 lane -> SettingsStore -> fence/shutdown`逐函数闭环。
- E3：Settings command/menu/event/window projection/ZUI props链，plugin page + bundle materialize/admit/revoke/localize链，以及theme token同步链。
- E3：23个指定Unreal、Godot、Fyrox、Bevy与Unity Graphics参考文件。
- E2：248个Editor ZUI文本和token inventory、17个runtime fallback inventory；未发现生产i18n topic subscriber。Settings metadata已有native row consumer，但没有value/editor/mutation consumer。
- 未覆盖：真实Settings窗口交互、磁盘满/断电/权限撤销、双进程竞争、只读或source-control工程、真实locale字体/RTL、主题切换截图、UIA/读屏、10k definitions、plugin hot unload和跨版本升级。本轮没有运行Cargo，因为任务只写review文档且共享生产源码在途。

### 2.3 当前生产链

1. Builder构造registry，注册8项Editor默认definition和4项job quota，从User store加载后创建唯一`SettingsAuthority`、`SettingsPersistenceService`与`EditorI18nService`。
2. authority内部mutex拥有mutable registry，`ArcSwap`发布immutable snapshot；snapshot带definition catalog、effective source、generation和builtin typed slots。registry仍只允许authority构造前注册。
3. `set/clear`发布后调用唯一同步subscriber。composition root用它同步locale和autosave；subscriber不负责持久化，类型系统也不阻止递归mutation。
4. persistence service不是authority observer。Scene Viewport先写Project snapshot，再为三个snap key提交ticket；worker序列化运行时“当前整个scope layer”，而不是admission时冻结的generation。
5. `save_authority_layer`在project transition、path不匹配或invalid source时可把`prepare... == None`当作`Ok(())`；成功receipt不能证明发生了write/fsync/rename。
6. `editor.settings.open`从command和Edit菜单进入event，retained host捕获`SettingsWindowProjection`并打开Preferences。projection含12项localized definitions和plugin page metadata，但不含value、source、layer、dirty、validation或editor descriptor。
7. Preferences ZUI声明`categories/settings/plugin_pages`数组prop；host projection把它们转换为typed category/entry model，native painter按clip加一行overscan绘制，hit test可选择category并过滤builtin/plugin entries。这关闭了“数组完全无人消费”的旧证据，但row没有value、editor或mutation action；窗口仍只在open时capture。
8. plugin SDK和runtime interface提供SettingsPage schema `/2`与LocalizationBundle schema `/1`；materializer校验bundle owner和key，extension batch原子登记/撤销，projection按locale翻译并排序。page仍没有content provider、asset/controller或setting IDs。
9. embedded catalog只含`en/zh-CN`；fallback为exact -> English -> raw key。i18n事件有bounded queue/coalesce/resync，但产品壳层没有订阅并按text revision重投影。
10. design token handle变化会同步V2 global、host palette/metrics/typography并mark presentation dirty；状态仍分散在settings snapshot、global RwLock、host ArcSwap和thread-local scope，也没有资源准备barrier或回滚receipt。

## 3. 当前实现中必须保留的基础

1. `SettingsKey`、scope、typed schema、default和layer precedence已经由统一core建模；Session > Project > User > default，Session明确不持久化。
2. `SettingsSnapshot`包含immutable definition catalog、per-key source与typed builtin slots；large payload Arc在无关变化时复用，reader无需拿registry mutex。
3. change log受4,096 entries、256 KiB和5分钟三重约束，cursor落后时要求snapshot resync。
4. persistent layer先全量验证再替换；project path canonicalization、operation gate和source cache避免旧project worker写入新project值。
5. Runtime11 keyed I/O lane有1,024 entry/8 MiB预算、typed ticket、retry、cancel-before-start、fence和诊断；UI线程不直接执行文件I/O。
6. store已有versioned envelope、同目录`create_new` temp、`write_all`、`sync_all`、rename与非Windows parent sync。
7. locale projection先捕获同一locale再组合文本；event queue有entry/byte预算、coalesce/resync和generation去旧。
8. plugin bundle/page共享owner ticket并原子admit/revoke，localization key在admission校验，窗口投影能稳定排序builtin和plugin metadata。
9. Settings native surface已用typed category/entry model、clip-bounded visible rows、overscan和stable category identity渲染registry/plugin metadata。
10. theme变更从唯一settings snapshot触发V2/host投影和presentation invalidation；typography已补充空font family和weight验证。

这些能力应被吸收到统一Settings Transaction、Preferences Product Host、Localization Registry和Theme Generation，而不是旁建另一套配置文件、单例或临时UI模型。

## 4. P0复核

### E-SET-P0-01 - Open - 内存修改、hot apply与耐久提交仍是三件事

`SettingsAuthority::set/clear`在返回前已经更新registry、发布snapshot并同步调用subscriber；API不durably admit persistent mutation，也不返回dirty/commit identity。`SettingsPersistenceService`虽在`EditorContext`中长期存在，却不是observer。生产submit仍只在Scene Viewport三个Project snap值路径；User locale、design tokens、keymap overrides、autosave interval和4个job quota没有通用保存owner。

Viewport路径同样先改变内存，再尝试lane admission；store缺席时controller可直接返回成功，admission或worker失败不回滚snapshot。shutdown fence只覆盖已经admit的工作，发现不了从未submit的persistent mutation。目标必须是scope transaction/commit coordinator：candidate snapshot、hot apply、durable admission、document write、restart pending、rollback和product feedback共享transaction identity；feature controller不得自行决定是否保存整个document。

### E-SET-P0-02 - Partial - Settings窗口已可达，但仍不能查看或编辑setting

旧报告“无caller/command/menu”的结论已关闭：当前有`editor.settings.open`、Edit菜单、typed event、retained-host side effect和真实窗口open链。`SettingsWindowProjection`也能从snapshot catalog生成localized category与12项builtin metadata，并合并plugin page projection。

产品核心仍缺失。最新native surface已经消费三个数组：category有typed projection、visible-row paint、hit test和selection，builtin/plugin entry会按选中category绘制label、description、scope/schema及restart marker。可是projection仍不携effective/layer/default/source/dirty/validation/editor descriptor，row没有typed control或mutation action；也没有search、origin、apply/cancel/reset、commit health。窗口打开后不监听value generation，catalog currentness也不代表value freshness。目标是把这条只读metadata浏览链升级为可查询、可编辑、可提交、可恢复的Preferences host。

### E-SET-P0-03 - Partial - localization metadata扩大，但全Editor locale承诺仍不成立

进展包括：bundle由54增至67个key，12项builtin definition和Settings标题可按locale投影；plugin package可贡献versioned localization bundle，SettingsPage的label/description/category使用typed localization key并随owner撤销。这关闭了旧failure文档中“raw V1 page DTO仍在生产”的源码结论。

但248个Editor ZUI仍有3,207个直接引号文本属性，Preferences的root defaults与aria文本仍硬编码英文。builder只发布`EditorTopic::i18n()`，生产未发现subscriber；retained host不会按locale generation重建所有已打开surface。fallback只有exact -> English -> raw，没有完整BCP47 parent chain、message format、number/date/unit、direction、font fallback或glyph policy。因此该项只能标Partial，不能把`editor.language.locale`描述为完整Editor语言切换。

## 5. P1差距状态

### 5.1 Settings authority、schema与插件生命周期

| ID | 状态 | 当前差距 | 必须重构为 |
|---|---|---|---|
| E-SET-P1-01 | Partial | 内置definition增至12项并开始覆盖autosave/quota，但绝大多数Editor行为仍在常量、layout state或feature字段。 | 全Editor adoption catalog，逐owner登记key、scope、default、schema、apply mode、persistence和产品页。 |
| E-SET-P1-02 | Open | registry仍只在authority构造前注册，没有register/unregister/batch/owner generation。 | dynamic definition registry，registration lease绑定builtin/plugin generation，卸载原子撤销并保留orphan value。 |
| E-SET-P1-03 | Open | plugin SettingsPage与真实SettingDefinition仍是两条贡献链。 | contribution原子声明definition set、page/content、bundle、capability和owner lease。 |
| E-SET-P1-04 | Partial | snapshot已有immutable definition catalog与per-key effective source，但没有paged layer/default/override/restart/validation/health query。 | bounded query snapshot返回descriptor、effective、每层值、origin、dirty、validation、restart与health。 |
| E-SET-P1-05 | Partial | page已升级V2 localization DTO，却仍无content asset/provider、setting IDs、order、icon、keywords、scope或actions。 | versioned page contract支持schema-generated或capability-bounded custom provider。 |
| E-SET-P1-06 | Open | authority只有一个可替换同步subscriber，当前同时承担locale/autosave composition callback。 | owner-bound multi-subscriber registry，每consumer有cursor、queue、backpressure、resync和health。 |
| E-SET-P1-07 | Open | subscriber仍在setter线程运行；slow/panic callback会延迟或终止mutation caller。 | commit publish与consumer apply解耦，panic containment、deadline和per-consumer receipt明确。 |
| E-SET-P1-08 | Open | comment禁止subscriber写回，但类型系统不阻止递归set。 | transaction phase与reentrancy policy显式化；递归mutation延后成batch或返回typed cycle error。 |
| E-SET-P1-09 | Partial | project layer cache/authority和operation gate已集中，但SceneViewportController仍拥有store与ticket retry/cancel生命周期。 | project/session composition root绑定store和commit coordinator，所有Project setting共享同一owner。 |
| E-SET-P1-10 | Open | `user_layer_load`在authority构造后固定，坏文件无法reload、repair或更新health。 | mutable source health、reload/retry/recover operation及generation-bound diagnostic。 |
| E-SET-P1-11 | Open | registry revision和部分generation仍用`saturating_add`，极值后identity冻结。 | checked epoch+counter或明确terminal exhaustion，禁止静默复用generation。 |
| E-SET-P1-12 | Open | persistent layer replacement仍逐key增加revision并逐次构造snapshot。 | batch一次validate、计算effective diff、发布snapshot和transaction identity。 |
| E-SET-P1-13 | Open | DesignTokens/KeymapOverrides/MRU schema层主要只检查enum variant。 | structured value具versioned validator、size/depth预算、canonicalization与migration。 |
| E-SET-P1-14 | Open | schema仍只覆盖基础scalar、Color/Chord和少量special type。 | 增加path、duration、bytes、collections、optional、secret/reference及custom editor codec。 |

### 5.2 Persistence、migration、recovery与并发

| ID | 状态 | 当前差距 | 必须重构为 |
|---|---|---|---|
| E-SET-P1-15 | Open | ticket记录change generation，worker却序列化运行时当前整个layer；receipt不证明写的是该generation。 | admission冻结canonical layer snapshot或delta，receipt返回实际durable generation/digest/path。 |
| E-SET-P1-16 | Open | lane key含scope/target/key，而job写整个scope document，不同key会重复整文件写。 | document/scope级lane、debounce和dirty-key合并，只提交最新冻结snapshot。 |
| E-SET-P1-17 | Open | 没有dirty/unsaved/committing/failed/conflicted process state。 | per-scope commit ledger与process persistence health供Preferences/status/close flow查询。 |
| E-SET-P1-18 | Open | worker失败主要是warning和ticket terminal code，除viewport私有队列外无产品错误面。 | 接入Diagnostic Journal与Notification decision，携scope/path/generation/retry/recover action。 |
| E-SET-P1-19 | Open | shutdown fence仍无deadline并调用blocking wait，hung I/O可无限阻塞退出。 | bounded deadline、cancel/abandon policy、emergency spool与non-durable close decision。 |
| E-SET-P1-20 | Open | 无跨进程lock、file generation/CAS、external watch或three-way merge。 | writer lease + on-disk generation/digest，提供reload/merge/overwrite decision。 |
| E-SET-P1-21 | Open | 无backup、LKG、corrupt quarantine、temp crash sweep或recovery journal。 | generation slots、LKG、损坏隔离、启动恢复和有配额temp清理。 |
| E-SET-P1-22 | Open | strict unknown key会让禁用插件的orphan payload阻断整个layer。 | known值与owner-versioned orphan区分，owner恢复后重新claim和验证。 |
| E-SET-P1-23 | Open | schema v1仍无可执行0->1 migration，旧格式只能拒绝。 | fixture/golden连续migration、迁移前backup与future-version read-only策略。 |
| E-SET-P1-24 | Open | 文件名`settings.toml`，内容是JSON envelope。 | 扩展名与canonical format一致，或采用真实TOML并公开schema contract。 |
| E-SET-P1-25 | Open | `read_to_string`在decode前无file-size预算。 | metadata cap、bounded reader和entry/string/depth/node预算。 |
| E-SET-P1-26 | Open | invalid Project source抑制写；invalid User source只保留startup provenance。 | 所有invalid source进入read-only/quarantine，显式recover/replace前禁止覆盖。 |
| E-SET-P1-27 | Open | 无setting alias/deprecation、split/merge transform、platform override或owner version。 | stable SettingId与migration metadata支持rename/type conversion/owner升级。 |
| E-SET-P1-28 | Open | 无secret、managed policy、source-control/read-only checkout或machine/profile layer。 | User/Machine/Project/Session/Managed/Secret stores及明确权限、优先级和审计。 |
| E-SET-P1-59 | Open | `prepare_persistent_layer_for_write == None`会被`save_authority_layer`折叠为`Ok(())`；ticket可Succeeded但没有写文件。 | receipt区分Written/SkippedStale/BlockedInvalid/NoTarget；只有write+sync+rename对应generation后才可Succeeded。 |

### 5.3 Preferences产品、编辑事务与可访问性

| ID | 状态 | 当前差距 | 必须重构为 |
|---|---|---|---|
| E-SET-P1-29 | Closed | `editor.settings.open`已有command、Edit menu、event和retained-host open route。尚无deep link的问题由后续项继续追踪。 | 保留稳定入口，并扩展`show(container/category/section/key)`而不另建旁路。 |
| E-SET-P1-30 | Partial | registry/plugin metadata已生成category与entry rows，并有native paint/hit-test/selection；但只在open时capture，缺owner/order/freshness和动态plugin/value刷新。 | 稳定order/owner/scope的paged tree/rows，按catalog/plugin/value generation增量更新。 |
| E-SET-P1-31 | Open | 没有schema生成的checkbox/number/enum/color/chord/structured editor。 | shared property-editor factory复用Inspector typed contracts并限制custom capability。 |
| E-SET-P1-32 | Open | 没有search、keyword、description、key或owner过滤。 | Unicode-normalized indexed search，结果含breadcrumb和bounded query budget。 |
| E-SET-P1-33 | Open | 不显示effective value来自default/User/Project/Session哪一层。 | 每项显示origin、override stack、editable scope与remove-override动作。 |
| E-SET-P1-34 | Open | 没有live/staged/restart apply模型，`requires_restart`无产品consumer。 | per-setting apply policy及pending restart subsystem，支持apply/cancel/revert。 |
| E-SET-P1-35 | Open | 没有reset item/section/all、import/export、backup或diff preview。 | capability-aware section action，预览affected keys/scope并返回durable receipt。 |
| E-SET-P1-36 | Open | 没有validation、commit progress、write failure、conflict或recovery UI。 | inline typed validation、top-level health banner与Diagnostic Journal linkage。 |
| E-SET-P1-37 | Partial | modal已有aria/focusable等基础props和close lifecycle，但无focus trap、keyboard row navigation、reader/reflow/virtualization证据。 | 完整dialog accessibility、responsive min constraints、virtualized list与Windows UIA/screenshot gate。 |

### 5.4 Locale、text identity与本地化平台

| ID | 状态 | 当前差距 | 必须重构为 |
|---|---|---|---|
| E-SET-P1-38 | Open | locale validator仍硬编码`en/zh-CN`，不来自source registry。 | available locale由admitted bundles与policy snapshot生成，并与setting generation一致。 |
| E-SET-P1-39 | Open | 两bundle key集合相同，但catalog不验证target coverage、required parity或unused key。 | build-time extractor与bundle validator按target报告missing/stale。 |
| E-SET-P1-40 | Open | missing translation静默显示raw key，无counter、owner或source diagnostic。 | typed missing-text record、once-per-key bounded diagnostic与fallback provenance。 |
| E-SET-P1-41 | Open | locale parser仅近似BCP47，不完整处理script、extension、private-use、alias。 | 成熟locale/ICU数据与canonical language-script-region-extension identity。 |
| E-SET-P1-42 | Open | fallback只有exact -> English，不支持parent culture和用户chain。 | language/script/region prioritized chain再进入configured fallback/source。 |
| E-SET-P1-43 | Open | translation value是普通字符串，参数仍是简单named replace。 | ICU/Fluent级plural/select/gender/escaping与typed arguments。 |
| E-SET-P1-44 | Open | 无number/date/time/duration/unit/currency/collation/case mapping服务。 | locale data service统一格式、排序、搜索和输入解析。 |
| E-SET-P1-45 | Partial | plugin bundle已有namespace owner、ticket/revoke和key admission，但无DLC/pack、hot reload、domain priority或collision policy。 | localization source registry按target/domain/priority/owner generation加载、卸载和异步刷新。 |
| E-SET-P1-46 | Open | locale snapshot无direction、writing system、font fallback或glyph coverage metadata。 | culture snapshot携LTR/RTL、font fallback、shaping/line-break并触发text cache rebuild。 |
| E-SET-P1-47 | Open | event sink仍同步运行在setter线程，产品consumer缺席，零consumer语义不可靠。 | 多sink异步revision broadcast，receipt区分Accepted/Applied/NoConsumer/Failed。 |
| E-SET-P1-48 | Open | 无pseudo、text expansion、mirroring、missing glyph、locale screenshot或reader测试。 | en-XA/ar-XB等自动门覆盖全部builtin/plugin surfaces。 |

### 5.5 Appearance、theme generation与资源一致性

| ID | 状态 | 当前差距 | 必须重构为 |
|---|---|---|---|
| E-SET-P1-49 | Open | 整个DesignTokens仍是一个SettingValue，小改动替换大payload并整document写。 | versioned theme resource；setting只保存选中profile与少量override。 |
| E-SET-P1-50 | Open | settings schema不完整验证DesignTokens内部不变量，各consumer仍可解释invalid value。 | authority admission前完整validate/canonicalize，consumer只读validated ThemeSnapshot。 |
| E-SET-P1-51 | Partial | host metrics与typography已保护更多finite/空family/weight条件，但V2/runtime fallback/全部density仍不共享validator。 | 单一validated ThemeSnapshot，所有projection禁止重新解释invalid value。 |
| E-SET-P1-52 | Open | settings snapshot、V2 RwLock、host ArcSwap、thread-local scope各有独立状态/generation。 | 单一ThemeGeneration携host/V2/text/icon/font projection并在frame capture原子绑定。 |
| E-SET-P1-53 | Open | appearance仍process-global，不表达window/display/profile owner。 | base theme与per-window scale/display context分离，支持跨DPI monitor。 |
| E-SET-P1-54 | Open | 248个ZUI只有74个显式token import，legacy theme并存且依赖图未治理。 | canonical theme dependency graph和lint，明确fallback/deprecated assets并删除死主题。 |
| E-SET-P1-55 | Open | 17个runtime render文件19处独立`workbench_dark()` fallback，missing token静默退默认。 | render command携resolved style/theme generation；missing token产生diagnostic。 |
| E-SET-P1-56 | Open | 无theme catalog、preset clone、preview、import/export、rollback或plugin token namespace。 | versioned theme package与preview transaction，定义owner/collision/compatibility。 |
| E-SET-P1-57 | Open | 无contrast、OS high contrast、color vision、reduced transparency/motion或target size验证。 | accessibility theme policy与自动contrast/geometry gate。 |
| E-SET-P1-58 | Open | hot chain会mark presentation dirty，但无font atlas/icon raster/GPU rebuild receipt与失败回滚。 | staged apply barrier准备全部资源后一次publish，失败保留旧generation。 |

## 6. P2差距状态

| ID | 状态 | 当前差距 | 建议收敛 |
|---|---|---|---|
| E-SET-P2-01 | Partial | SettingsPage V2已验证typed localization key和非空category，但page id/ownership/content identity仍不完整。 | 全部字段使用validated stable type，display text只保存localization identity。 |
| E-SET-P2-02 | Open | `available_locales()`只有字典序字符串，无native name、script/region或completion。 | LocaleDescriptor携native/localized name、script/region、completion和pack source。 |
| E-SET-P2-03 | Open | locale queue bytes预算按短字符串，Arc/queue/sink resident cost未计入。 | 保守resident accounting并公开high-water。 |
| E-SET-P2-04 | Open | dropped/resync/failed counters仍用saturating add。 | checked counter或epoch rollover，并记录overflow diagnostic。 |
| E-SET-P2-05 | Open | temp identity仍依赖pid、wall-clock nanos和有限attempt。 | commit journal分配session/transaction/generation/digest identity。 |
| E-SET-P2-06 | Partial | window title和动态category/entry label可从i18n投影，但root defaults与aria仍硬编码英文。 | asset compiler要求display/accessibility text identity并生成审计清单。 |
| E-SET-P2-07 | Open | Preferences固定720-960宽、480-680高与静态导航，未证明小窗/DPI/CJK适配。 | responsive tracks、min-content、scroll和多DPI screenshot matrix。 |
| E-SET-P2-08 | Open | setting metadata无docs URL、examples、unit、precision或risk。 | optional help metadata、unit/precision、examples与owner docs link。 |
| E-SET-P2-09 | Open | 无settings access/change telemetry，难定位slow apply和写热点。 | privacy-aware latency/bytes/coalesce/failure metrics，不记录secret value。 |
| E-SET-P2-10 | Open | 测试偏源码contract，缺key/schema/document/locale parser的property/fuzz corpus。 | property-based与fuzz测试约束panic、allocation和round-trip。 |
| E-SET-P2-11 | Open | settings/i18n错误直接拼接英文，没有stable diagnostic code。 | stable code + structured context，presentation按locale翻译。 |
| E-SET-P2-12 | Open | 生产注释仍引用Editor17/Plan编号等历史owner。 | 用module/schema/owner语义替代计划编号，历史只留文档。 |

## 7. 旧failure与当前源码的关系

| failure | 当前判断 | 后续动作 |
|---|---|---|
| Editor17 settings persistence/hot projection | 部分源码修复存在，但核心durable transaction仍Open；动态validation未完成。 | 保留为M1输入，不得把hot apply测试等同durability。 |
| Editor06/12 SettingsPage localization hardcut | 当前源码已是SettingsPage `/2` + LocalizationBundle `/1` + owner batch；failure中的raw V1结论已过时。 | source-level可关闭旧DTO问题；plugin content/definition结合转入P1-03/P1-05。 |
| Editor08 keymap authority migration | 当前production已走authority，旧旁路hard cut在源码层完成。 | validation仍需在共享下层可构建后补齐。 |
| Editor10 project document legacy authority | production只保留activated authority，legacy helper受test cfg限制。 | source-level完成，动态回归待补。 |
| Editor13 script build batch window | 当前没有对应SettingDefinition，仍是真实未迁移owner。 | 纳入M2 adoption catalog。 |
| Editor14 job quota migration | 四项quota已注册并被typed snapshot消费，静态迁移完成。 | durable Preferences编辑与运行验证仍由P0-01/M3追踪。 |

## 8. 与参考引擎的可验证差异

| 参考 | 仓内源码可验证能力 | Zircon当前差异 | 应吸收的原则 |
|---|---|---|---|
| Unreal | `ISettingsModule`按container/category/section注册object或custom widget，支持Show/Unregister；`ISettingsSection`显式提供CanEdit/Save/Reset/Import/Export/Status与delegates；DeveloperSettings可自动发现和广播property change。Config系统还有层级、tracked mutation、async load与flush；localization manager管理culture、resource source、display string identity和revision。 | Zircon catalog只读且静态，page与definition分离；section action、viewer、dynamic owner、durable section status、culture resource/revision均缺失。 | registration、viewer、save policy、status、config layer和text revision必须是公共模块合同。 |
| Godot | EditorSettings维护property metadata/default/order/basic/restart/change/save与shortcut；SettingsDialog有sectioned inspector、search、advanced、override/revert、restart提示和save flow。ProjectSettings有version、changed set、custom feature override与save；TranslationServer/Domain提供fallback/domain/pseudolocalization；theme manager集中生成。 | Zircon窗口只有metadata props，未连接property editor或save；无restart/override UI、pseudo/domain和集中theme build barrier。 | 同一property authority驱动可搜索UI，locale/theme改变必须触发有generation的系统重建。 |
| Fyrox | Settings plugin真实注册菜单、打开浮动窗口，用reflection Inspector生成字段/分组，支持search/default/OK，并即时热应用renderer quality。 | Zircon已有入口、schema和Inspector基础，却未把它们连接为可编辑产品。 | 先完成最小但真实的端到端产品链，不允许停在静态ZUI或metadata数组。 |
| Bevy | Feathers把theme建模为Resource和stable token component；resource change集中更新background/border/text，missing token给warning和醒目error color。 | Zircon存在多套projection与17个静默dark fallback，无法证明单一generation或missing-token health。 | token consumer显式opt-in、change propagation集中、missing token必须可观测；Bevy本身不代表完整Preferences。 |
| Unity Graphics | preferences通过type discovery收集provider、按DisplayInfo排序、聚合keywords，并由SettingsService打开固定User path；graphics settings group携version、pipeline applicability与category order。 | Zircon page已有bundle identity但无provider invocation、keywords/order/version/applicability和SettingDefinition集合。 | provider discovery、排序、搜索、scope、version和适用性属于extension contract。 |

没有单一参考解决本文所有目标。Unreal配置历史复杂度、Godot全局单例、Fyrox即时reflection mutation、Bevy Feathers当前theme范围和Unity反射provider成本都不能直接照搬。Zircon应吸收其ownership、transaction、extension、query和revision原则，并保持现有bounded Runtime lane、immutable snapshot与retained UI边界。

## 9. 目标架构

### 9.1 EditorSettingsRegistry 与 QuerySnapshot

- definition使用stable SettingId、owner package/generation、scope policy、schema version、default、apply policy、presentation keys和editor factory。
- registration通过owner lease与atomic batch完成；卸载撤销definition/page/bundle，但orphan persistent value保留并隔离。
- immutable query snapshot可分页返回container/category/section/page、effective/default/layers/source、dirty、validation、restart和health。
- builtin与plugin走同一admission；不允许page、definitions、bundle或custom provider半注册。

### 9.2 SettingsTransaction 与 DurableCommitCoordinator

1. caller提交一个或多个typed change及expected generation。
2. authority完整validate/canonicalize，生成candidate snapshot、effective diff和document digest。
3. coordinator按scope/document合并，durably admit journal后返回Accepted receipt。
4. apply policy决定live/staged/restart；consumer按transaction报告Applied/Failed。
5. worker写admission时冻结的snapshot，write/sync/rename后发布Durable receipt。
6. stale project、invalid source、no target、conflict和write failure是不同terminal state；禁止折叠为Succeeded。
7. external writer用generation/digest/CAS检测，进入reload/merge/overwrite decision。

### 9.3 PreferencesProductHost

保留当前command/menu/event、native category/entry paint与selection链，在其上加入paged query和shared typed property editor。窗口按query generation增量刷新，支持deep link、search、scope/source、live/staged apply、reset/import/export、validation、restart、commit health和plugin owner unload。custom page必须有capability、lifecycle、threading和failure boundary。

### 9.4 LocalizationRegistry 与 TextRevision

ZUI与Rust presentation保存`LocalizedTextId(namespace,key,source)`，asset compiler提取并拒绝未豁免literal。Registry按target/domain/owner/priority加载versioned bundles；成熟locale/message-format库提供BCP47 canonicalization、parent fallback、plural/select、number/date/unit/collation与direction。`LocalizationSnapshot`携text revision、fallback chain、font/glyph policy；所有surface在同一revision重投影和layout。

### 9.5 ThemeCompiler 与 ThemeGeneration

settings只选择versioned theme profile和override layer。compiler合并builtin/project/plugin/user tokens，完整验证geometry/color/font/icon/contrast，准备host/V2/text/font/icon/GPU projection；成功后一次发布单一ThemeGeneration。window/frame捕获immutable generation，失败保留旧theme；base theme与per-window DPI/high-contrast context分离。

## 10. 分阶段重构路线

### M0 - Contract freeze与可观测基线

- 冻结当前12个definition、scope、schema、store path、apply consumer与persistent caller matrix。
- 增加ZUI literal、translation parity、theme import/fallback、persistent mutation/submit mismatch lint。
- 为authority、persistence、i18n和theme projection增加generation/health diagnostics。
- 建立format golden、bad/oversize file、subscriber failure和invalid theme corpus。

### M1 - Durable transaction与recovery

- 引入batch candidate、document-level coalescing、durable journal和typed receipt。
- 移除feature-owned submit，把User/Project store绑定到session/project composition root。
- 修复P1-59，区分Written/Skipped/Blocked/Conflict并建立dirty/failed health。
- 加入deadline shutdown、CAS、LKG/quarantine/temp recovery、bounded reader和真实migration。

### M2 - Dynamic registry与plugin contribution

- 支持owner-bound atomic register/unregister和query snapshot。
- 扩展structured schema、validator、alias/deprecation/migration与apply policy。
- 合并page、definitions、bundle和provider admission，保留unloaded owner orphan data。
- 先迁移keymap、jobs、viewport、appearance、locale和script build batch window。

### M3 - Preferences端到端产品

- 将当前入口连接category/search/editor/origin/reset/apply/restart/error/recovery workflow。
- 复用Inspector editor contract，但隔离transaction、permission和custom provider capability。
- 窗口按value/catalog/plugin/locale generation实时重投影，不使用open-time静态capture。
- 完成keyboard、reader、DPI、small-window、long-text和大列表virtualization验证。

### M4 - Localization全产品切换

- 在ZUI compiler/runtime DTO引入LocalizedTextId、extractor、coverage/parity gate和text revision。
- 接入成熟locale与message format，支持parent fallback、plural、format、domain和plugin bundle。
- 迁移全部shell/pane/dialog/command/notification/plugin surface并删除literal旁路。
- 增加pseudo、RTL、font fallback、missing glyph和截图/UIA矩阵。

### M5 - Atomic appearance

- 将large DesignTokens setting迁移为theme profile + overrides。
- 建立single validator、theme compiler和ThemeGeneration publish barrier。
- 收敛legacy token/theme asset图，删除17个runtime hardcoded fallback。
- 增加preview/rollback、multi-window/DPI、high contrast和font/icon/GPU failure evidence。

### M6 - Adoption、性能与旧路径删除

- 迁移其余Editor constants/local preferences和全部plugin settings。
- 删除SceneViewport ticket ownership、metadata-only Preferences rows、single subscriber和literal UI旁路。
- 以10k definitions、100k search terms、1k mutation burst、slow disk、two process、locale/theme storm做门禁。
- 完成旧格式升级/回退、只读/source-control工程、crash recovery和release compatibility。

## 11. 验收门

1. 任意persistent `set/clear`都返回transaction/commit receipt，不存在绕过coordinator的生产caller。
2. lane admission失败时产品不得报告saved；snapshot回滚或显示可查询dirty/degraded状态。
3. worker write失败能在Preferences、Notification和Diagnostic Journal定位scope/path/generation并重试。
4. User locale/theme/keymap/autosave/job quota修改后强制终止，重启只观察已确认durable generation。
5. multi-key burst只产生bounded document commits，receipt generation/digest与实际文件一致。
6. stale project、invalid source或no target必须返回Skipped/Blocked，不得返回Succeeded。
7. 两个Editor竞争同一User/Project文件时检测冲突，不静默last-writer-wins。
8. 0->current及rename/type-split migration有golden、backup与failure recovery；future version只读。
9. disabled plugin orphan key不阻断builtin layer，plugin恢复后可claim并重新验证。
10. oversized/deep document和超长key/value在分配预算内拒绝。
11. shutdown有hard deadline；hung I/O不无限阻塞退出，未durable状态有operator decision。
12. Settings可从menu、command palette和deep link打开并定位指定setting。
13. Settings能枚举并编辑全部12个builtin definition，显示scope、effective source、default和restart policy。
14. 窗口打开期间value/catalog/plugin/locale变化会按generation增量更新，不展示stale capture。
15. plugin动态注册/卸载definitions+page+bundle+provider原子，窗口打开期间卸载无dangling callback。
16. schema editor覆盖bool/int/float/string/enum/color/chord/structured value，invalid input不进入authority。
17. search覆盖label/description/key/category/owner/keyword，10k definitions有固定latency/allocation门。
18. apply/cancel/reset item/section/all/import/export都有transaction receipt和recovery语义。
19. keyboard-only、screen reader、200% DPI、420px窗口和最长locale文本无重叠、失焦或不可达控件。
20. ZUI/Rust extractor覆盖所有用户可见surface，未经批准literal使required validation失败。
21. locale切换后全部open surface在一个text revision更新并重新layout。
22. zero i18n consumer绝不报告Delivered；每surface按cursor apply/resync，slow consumer不阻塞setter。
23. BCP47 canonicalization和language-script-region prioritized fallback通过golden vector。
24. plural/select、number/date/duration/unit与escaping在en/zh及复杂plural locale通过fixture。
25. plugin/domain bundle load/unload/collision/fallback有owner lease与deterministic priority测试。
26. pseudo、RTL、CJK/Arabic font fallback、missing glyph和text expansion有产品截图门。
27. invalid/NaN/negative/out-of-range theme token在authority admission前失败。
28. host、V2、text/icon/font/GPU在同一frame观察同一ThemeGeneration，准备失败保留旧theme。
29. theme asset graph有唯一canonical root；missing token产生diagnostic，17个renderer不再独立fallback。
30. multi-window、跨DPI、OS high contrast、reduced motion/transparency与preview/rollback有Windows验证。
31. settings/i18n/theme property、fuzz与fault tests无ignored且真实运行；attribute count不冒充pass count。
32. 实施前后重取本文fingerprint、definition/bundle/literal/import/fallback inventory，差异被计划或review解释。

## 12. 测试与证据策略

- 单元/属性：key、schema、scope precedence、batch diff、generation exhaustion、locale canonicalization/fallback、message format、theme validator。
- 持久化故障：admission full、slow/full disk、permission、rename/fsync failure、crash point、corrupt primary/LKG、temp orphan、shutdown deadline。
- 并发：multi-thread transaction、reentrant/slow/panic consumer、multi-process CAS、project switch与queued commit、plugin unload与open page。
- 产品：open/deep-link/search/edit/apply/cancel/reset/restart/conflict/recover，覆盖mouse/keyboard/UIA/DPI/long text。
- i18n：extractor parity、missing/unused、pseudo、RTL、font fallback、plural/format、dynamic domain与全surface text revision。
- appearance：compile/preview/rollback、single generation、multi-window/DPI、GPU/font/icon rebuild failure与contrast。
- 性能：10k definitions、100k terms、1k burst、large theme、locale/theme switch全shell rebuild，报告p50/p95/p99与allocation high-water。
- 兼容：现有v1 User/Project fixture、future-version read-only、disabled plugin orphan、只读/source-control工程和跨版本回退。

本轮没有执行Cargo或真实窗口测试。当前116个source test attributes、0 ignored只是inventory，不是绿色证据；25个plan source中的test/ignored也不属于产品验证。实施阶段必须先解除共享下层构建阻断，再按M1底层durability、M2 registry、M3 product的依赖顺序验证。

## 13. 完成定义

只有同时满足以下条件，Editor Settings/Preferences/Locale/Appearance首轮重构才可关闭：

- persistent mutation、hot apply与durable commit属于同一可追踪transaction，feature不再拼接submit；
- Preferences是真实可搜索、可编辑、可恢复、可扩展且可访问的产品，不是静态metadata投影；
- plugin可原子贡献和撤销definition/page/provider/bundle/theme扩展，orphan data不因lifecycle丢失；
- locale切换覆盖全部surface并携统一text revision、fallback、format、direction与font policy；
- theme切换在host/V2/text/icon/font/GPU中以单一generation发布，失败可回滚；
- migration、external conflict、corrupt source、slow/full disk、crash、shutdown和跨版本恢复都有动态证据；
- required Windows测试、真实窗口截图/UIA和性能门通过，结果绑定当前source/build fingerprint；
- suppressed-success write、feature-owned ticket、metadata-only Preferences rows、single subscriber、literal text和漂移theme fallback被删除，而不是永久兼容。

在此之前，Zircon可以称为“具备typed settings、bounded I/O、可达Settings入口及局部locale/theme projection基础”，不能称为达到Unreal/Godot级Editor Preferences、本地化与主题平台，更不能据此宣称工程完整性或性能优于这些参考引擎。
