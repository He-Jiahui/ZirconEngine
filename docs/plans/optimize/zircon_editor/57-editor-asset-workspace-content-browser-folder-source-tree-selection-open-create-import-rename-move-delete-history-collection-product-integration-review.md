---
title: Editor Asset Workspace、Content Browser、Folder/Source Tree、Selection、Open/Create/Import/Rename/Move/Delete、History/Collection 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor57
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_editor/assets/ui/editor/asset_browser.zui
  - zircon_editor/assets/ui/editor/assets_activity.zui
  - zircon_editor/src/core/asset
  - zircon_editor/src/core/editor_event/types.rs
  - zircon_editor/src/ui/binding/asset
  - zircon_editor/src/ui/binding_dispatch/asset
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_editor/src/ui/host/editor_event_execution/asset_event.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/asset_access.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/retained_host/app/assets
  - zircon_editor/src/ui/retained_host/app/backend_refresh.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer
  - zircon_editor/src/ui/retained_host/app/asset_drag_payload
  - zircon_editor/src/ui/retained_host/asset_pointer
  - zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer
  - zircon_editor/src/ui/layouts/views/asset_browser
  - zircon_editor/src/ui/layouts/views/asset_surface_presentation.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/snapshot/asset
tests:
  - zircon_editor/src/tests/editing/asset_workspace.rs
  - zircon_editor/src/tests/editor_asset_type_registry
  - zircon_editor/src/tests/host/retained_asset_pointer.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch
  - zircon_editor/src/tests/host/retained_menu_pointer/asset_browser_controls_visual_screenshot.rs
  - zircon_editor/src/tests/ui/asset_browser
  - zircon_editor/src/ui/layouts/views/asset_browser/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/55-editor-structured-clipboard-cut-copy-paste-duplicate-delete-cross-document-remap-drag-payload-product-integration-review.md
  - docs/plans/optimize/zircon_editor/56-editor-search-filter-query-index-result-find-usage-reference-navigation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/51-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/87-runtime-asset-reference-identity-locator-guid-subasset-redirector-rename-move-resolution-repair-migration-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/ContentBrowserData/Public/ContentBrowserDataSource.h
  - dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SContentBrowser.cpp
  - dev/UnrealEngine/Engine/Source/Developer/AssetTools/Private/AssetRenameManager.h
  - dev/UnrealEngine/Engine/Source/Developer/AssetTools/Private/AssetRenameManager.cpp
  - dev/godot/editor/docks/filesystem_dock.h
  - dev/godot/editor/docks/filesystem_dock.cpp
  - dev/godot/editor/file_system/editor_file_system.h
  - dev/godot/editor/file_system/editor_file_system.cpp
  - dev/fyrox/editor/src/asset/item.rs
  - dev/fyrox/editor/src/asset/mod.rs
  - dev/fyrox/editor/src/asset/preview/cache.rs
  - dev/bevy/crates/bevy_asset/src/path.rs
  - dev/bevy/crates/bevy_asset/src/event.rs
  - dev/bevy/crates/bevy_asset/src/io/source.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeProfileFactory.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Material/AssetReimportUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor-PrivateShared/AssetCallbacks/AssetCreationUtil.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Asset Workspace、Content Browser、Folder/Source Tree、Selection、Open/Create/Import/Rename/Move/Delete、History/Collection 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon Editor当前资产工作区不是纯占位。目录与预览有不可变generation，catalog、resource、details与reference graph有真实数据，folder和asset顺序稳定；Asset Browser的目录选择、单项选择、查询、kind filter、list/thumbnail切换、utility tab、reference跳转和模型导入都能沿真实event chain执行。Asset Type Registry也能登记creation template、toolkit和context command，operation dispatch会检查command enablement、remote-callability、write target、source authority与transaction要求。这些底座应保留。

但是当前产品面尚不能承担工程级Content Browser。最直接的能力真实性错误是：工具栏的`Locate selected asset`事件没有资产参数，执行时只打开`editor.assets`并请求preview refresh；资产行/卡片的生产pointer route只发`SelectItem`，没有双击、Enter或任何`OpenAsset`绑定，因此已实现的toolkit打开内核在资产浏览器中不可达。更深的扩展断层是catalog只携带封闭`ResourceKind`，workspace把每个真实条目重新推导成`AssetTypeId::from_resource_kind`；插件声明的`ai.behavior_tree`、replication schema、particle或自定义data type会被压成builtin类型，其toolkit、presentation和context command无法从真实资产恢复。

后台刷新还有数据完整性阻断。default scene的任一asset/resource change都会把`reload_default_scene`置真，随后重新打开ProjectManager、scan/import、从磁盘加载default scene并直接`replace_world`；整个调用不查询dirty document、history、当前打开scene或冲突决策，可能静默丢弃尚未保存的authoring world。该调用者必须接入Editor02/03统一的dirty transition，而不能以“资源刷新”名义绕过文档权威。

本报告登记 **4项P0、56项P1、12项P2与44个资格门**。Editor57唯一拥有Asset Browser实例状态、source tree、selection/activation、可见action、mutation orchestration、history/favorites/collections及产品receipt；Editor04继续拥有catalog/import/reimport/thumbnail/reference算法，Editor02/03拥有dirty document与scene transition，Editor08拥有通用command/operation基础设施，Editor55拥有通用clipboard/delete/duplicate/drag transfer语义，Editor56拥有query/index/find usage，Runtime85/86/87拥有build/type/reference父合同。不得把这些父问题复制成第二套实现。

## 2. 审查边界、currentness与证据等级

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | 证据等级 | 说明 |
|---|---:|---|---|
| Zircon资产产品链 | **209 / 33,959 / 30,988 / 1,205,005 / 250** | E3 | ZUI、asset core、catalog/details/preview、event/runtime access、retained pointer/effects、workspace snapshot/layout和focused tests；另有3项ignore |
| 当前真实交互链 | 目录选择、资产选择、搜索/过滤、view/tab、reference跳转、Quick Import | E3 | 从可见control/pointer，经binding/event/effect，到state/runtime副作用逐调用点核对 |
| 参考引擎切片 | **17 / 22,457 / 19,478 / 842,734 / 21** | E2/E3 | Unreal Content Browser/rename、Godot FileSystem dock/index、Fyrox asset browser、Bevy source/path/event、Unity Graphics consumer |

209份Zircon文件按normalized relative path排序，将每个`path + NUL + lowercase file SHA-256 + LF`串联后计算working-tree fingerprint，结果为`b62334ab04a3e22ee0fd9cbfdf74a1bd065b721624feda05a181b267ad06f6ea`。17份参考源码按同一算法计算，fingerprint为`6cc4f7152ec189fb0002bde57974c3efed0331b4f944d99438868b935f72252b`。冻结Git基线为`bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator baseline epoch为335。

聚焦Zircon语料中有22份非本轮产生的dirty文件，涉及asset dirty registry、binding codec、manager deactivation、asset layout/pointer与focused tests。本轮按working tree审查，只编辑本报告及三个索引；实施前必须重算fingerprint，并复核这些在途文件是否改变selection、pointer activation、preview cache或project deactivation终态。

### 2.2 当前产品链矩阵

| 产品能力 | 当前入口 | 当前执行事实 | 工程结论 |
|---|---|---|---|
| Folder navigation | tree/content folder click | `SelectFolder`更新一个全局`selected_folder_id` | 真实单步导航，无history/instance state |
| Asset selection | row/card click | `SelectItem`更新单一UUID并同步details/preview | 真实单选，无多选、anchor、keyboard model |
| Asset activation | 无生产入口 | `OpenAsset`内核只在测试/其他命令调用 | toolkit存在但Browser不可打开资产 |
| Locate selected | toolbar icon | 只打开`editor.assets`并请求preview refresh | label与行为不一致，不携带identity |
| Search/filter | text/kind chip | 当前目录全量扫描、ASCII lowercase、单kind | 真实局部过滤；全局query owner归Editor56 |
| Create asset | global creation menu | registry生成模板，operation有write-target准入 | 内核真实，但Browser/右键无create surface |
| Context command | snapshot投影 | 生产UI无consumer；invoke只有API与测试调用 | 扩展声明不可执行 |
| Quick Import | path field + `Import` | 仅OBJ/glTF/GLB，硬编码stage/import并把mesh插入scene | 通用文案下的模型专用复合副作用 |
| Drag | asset row/card press | 生成typed asset payload供reference/property drop | 没有folder destination move/copy |
| External change | watcher/resource/editor events | catalog/detail/preview刷新；default scene直接replace world | 局部刷新真实，scene transition不安全 |
| Source tree | `res://`与package roots | 全树递归materialize且始终展开 | 无provider/source authority与expand state |
| History/favorite/collection | 无 | event、state、surface、persistence均不存在 | 尚未开始产品实现 |

### 2.3 必须保留的工程基础

1. 保留`EditorAssetCatalogGeneration`、details/preview generation、publish epoch与change stream；替换的是产品projection与mutation协调，不是退回可变全局Vec。
2. 保留folder构建中的`BTreeMap`、display-name稳定排序和UUID tie-break；大型项目优化必须保持确定性。
3. 保留Activity snapshot构建后派生Explorer snapshot的单次业务投影思想；后续可以改成共享Arc/page model，不应恢复两次catalog遍历。
4. 保留`navigate_to_asset`的“UUID定位父目录并选择”原子意图，修复`LocateSelectedAsset`时应复用并扩展为reveal/focus receipt。
5. 保留Asset Type Registry的batch materialization、owner验证、toolkit/creation/context descriptor和enabled registry cache。
6. 保留`editor_operation_dispatch`对command enablement、remote-callability、write target、read-only source和transaction operation的准入。
7. 保留typed `UiDragPayloadKind::Asset`及locator/UUID数据，不以字符串路径重新建立第二权威。
8. 保留reference row/pointer bridge与UUID优先导航；它是详情页局部能力，不冒充全局Find Usage。
9. 保留selected details的同步generation校验和resource revision投影，后续action receipt必须引用同一代际。
10. 保留preview admission与paint-only invalidation，修复的是UI decoded-image cache的预算和生命周期。

### 2.4 唯一owner边界

| 问题 | 唯一实现owner | Editor57责任 |
|---|---|---|
| catalog/import/reimport/thumbnail/reference graph算法 | Editor04、Runtime85 | 提供产品入口、目标目录、进度、结果与冲突receipt |
| dirty/save/autosave/recovery | Editor02 | 所有资产/scene mutation先过dirty transition |
| scene open/replace/history | Editor03 | 登记watcher reload这一遗漏caller并禁止绕过 |
| command/keymap/menu/remote | Editor08 | 只定义asset domain command与availability provider |
| jobs/progress/cancel/shutdown | Editor09 | 把长操作接入job，不新建私有线程系统 |
| extension reload/toolkit lifecycle | Editor50 | 真实catalog必须保留exact AssetTypeId并消费既有registry |
| copy/cut/paste/duplicate/delete/drag通用语义 | Editor55 | 在Browser中选择目标、展示preflight并消费transaction receipt |
| query/index/find usage/navigation result | Editor56 | Browser只持query session和结果projection |
| asset type/schema/version | Runtime86 | catalog/snapshot不再压缩exact type |
| rename/move/reference repair | Runtime87 | mutation UI必须等待qualified repair plan/receipt |

## 3. P0：必须先关闭的数据完整性与能力真实性问题

### ED57-P0-01 · default scene watcher可绕过dirty/history并直接替换authoring world

`plan_asset_backend_refresh`只要发现default scene URI出现在asset或resource change中就设置`reload_default_scene`。`apply_asset_refresh_plan`随后同步调用`reload_default_scene`；该函数重新open project、`scan_and_import`、从磁盘load scene、`prepare_authoring_world`并`runtime.replace_world`。调用链没有document identity、dirty generation、save/discard/cancel决策、history barrier、scene instance qualification或rollback。

这不是普通refresh，而是一个后台触发的破坏性document transition。必须立即禁止直接替换：watcher只能发布`ExternalSceneRevisionObserved`，交给Editor02/03的统一transition coordinator；dirty时必须进入明确决策或conflict状态，clean时也要以expected document/revision CAS执行，并保留失败后的原world。Editor57负责关闭该入口和产品反馈，Editor02/03负责共享transition实现。

### ED57-P0-02 · `Locate selected asset`没有资产身份，成功执行的行为与可见文案不一致

`EditorAssetEvent::LocateSelectedAsset`是无字段variant。执行分支仅`open_view(shell, "editor.assets", "Opened assets")`并追加`AssetPreviewRefreshRequested`，既不读取当前selection，也不调用已有`navigate_to_asset`，更不清理阻挡过滤、展开source tree、滚动、聚焦或返回失败原因。用户点击“Locate selected asset”得到的是“打开另一个资产面板”。

必须把事件替换为qualified target或明确从调用窗口捕获selection token，并返回`AssetRevealReceipt { browser_instance, target, folder, query_adjustment, scroll/focus outcome, catalog_generation }`。没有selection、target stale、source hidden或filter阻挡时必须可见失败；修复前该按钮应禁用或改成真实行为名称。

### ED57-P0-03 · Asset Browser条目没有打开/激活路由，已实现toolkit内核在主产品入口不可达

生产`dispatch_shared_asset_content_pointer_click`对folder只发`SelectFolder`，对item只发`SelectItem`。ZUI、pointer bridge和asset content route没有double-click、Enter或`OpenAsset`事件。全仓生产`OpenAsset`构造只剩normalization/activity log/handler等基础设施，实际构造集中在测试和其他专题命令；详情面板还只是把toolkit id/open operation显示成文本。

必须建立统一`AssetActivationIntent`，覆盖double-click、Enter、Open command、context menu和reference activation；由exact AssetTypeId解析enabled toolkit，以catalog generation和asset revision校验stale，返回opened/reused/unavailable/failed receipt。不能用“handler存在”或测试直接构造event作为Browser功能完成证据。

### ED57-P0-04 · catalog把插件资产类型压缩为`ResourceKind`，真实资产无法恢复自定义toolkit/context/presentation

插件能够定义开放`AssetTypeId`、creation template、toolkit与context command，但`EditorAssetCatalogRecord`只保存`ResourceKind`。`asset_type_id_for_locator`、selected item/subasset/reference projection都调用`AssetTypeId::from_resource_kind`，随后enabled registry只能查到builtin映射。自定义AI、network、particle、import settings或第三方类型即使成功生成资产，也会被展示/打开成coarse Data等builtin类型，或找不到自己的toolkit。

必须让Runtime86的exact `AssetTypeId + schema/version + compatibility state`进入catalog、details、selection、reference与operation target，coarse kind只能用于通用分组/图标fallback。插件卸载或类型未知时应保留opaque identity并显示Unavailable/Compatibility诊断，不能静默改扮builtin资产。

## 4. P1：工程级能力差距

### 4.1 Activation、action与能力真实性（ED57-P1-01至P1-08）

- **ED57-P1-01**：selection与activation没有分离的typed state machine；单击、双击、Enter、预览和打开文档需要一致而可中断的语义。
- **ED57-P1-02**：Browser没有Open、Open With、Edit、Preview或Reveal in OS等可发现command，toolkit可用性也没有进入action model。
- **ED57-P1-03**：asset item/folder没有真实context-menu provider；通用workbench classifier只区分SceneNode、ModuleNode和GenericWorkbench。
- **ED57-P1-04**：`selection.context_commands`虽被registry投影，生产布局/bridge没有渲染、enablement刷新或invoke caller。
- **ED57-P1-05**：toolkit view id与open operation被当成详情文本显示，不是可执行action，也没有unavailable reason。
- **ED57-P1-06**：creation menu只进入全局菜单投影，没有selected folder附近的Create按钮、空白区右键和folder context入口。
- **ED57-P1-07**：action availability没有统一消费selection cardinality、source writability、asset compatibility、operation busy/lease和principal capability。
- **ED57-P1-08**：asset action没有统一receipt投影；用户无法区分accepted、queued、running、partial、conflict、cancelled、failed与completed。

### 4.2 Browser实例、导航、selection与持久状态（ED57-P1-09至P1-16）

- **ED57-P1-09**：所有Activity与Browser窗口共享一个`selected_folder_id`、`selected_asset_uuid`、query和kind filter，多窗口无法独立浏览。
- **ED57-P1-10**：只有view mode和utility tab按Activity/Explorer分开；selection、source、query、sort、column与navigation state没有`BrowserInstanceId`。
- **ED57-P1-11**：没有back/forward history、history branch截断、Up语义、导航原因或stale entry恢复。
- **ED57-P1-12**：source tree每次递归生成全部folder且全部展开，没有expanded set、lazy children、reveal path或持久展开状态。
- **ED57-P1-13**：selection仅一个UUID，没有ordered multi-selection、anchor/focus item、range/toggle规则和selection generation。
- **ED57-P1-14**：catalog同步只在UUID消失时清selection；资产保留UUID但外部移动后，selected folder不会跟随，content list与details可出现不同位置事实。
- **ED57-P1-15**：没有rename-in-place、pending focus/scroll target、selection restore或操作完成后的new/renamed item reveal。
- **ED57-P1-16**：Browser实例状态未进入workspace/layout persistence，也没有schema version、project qualification或失效迁移。

### 4.3 Content source、folder与exact type模型（ED57-P1-17至P1-24）

- **ED57-P1-17**：folder record只有字符串id/prefix/name/children/count，没有`ContentSourceId`、provider id、source generation或stable folder identity。
- **ED57-P1-18**：folder/source不携带read/write/create/delete/rename/move/import能力，package与project source只能靠调用方猜测只读性。
- **ED57-P1-19**：一个`res://`根把物理资产来源压成单一Assets树，无法表达多个project content root、generated/remote/cooked source或同路径不同provider。
- **ED57-P1-20**：package root只有`package://id`与display name，没有mount health、版本、trust、availability、offline状态或provider-owned action。
- **ED57-P1-21**：catalog/details/reference/subasset record均缺exact AssetTypeId，P0修复必须覆盖全链而非只给selected row加旁路字段。
- **ED57-P1-22**：catalog item缺schema/version/compatibility/currentness摘要，Browser不能诚实区分可编辑、只可预览、需迁移、缺插件和损坏。
- **ED57-P1-23**：subasset只在详情中列出，缺stable activation target、parent/label导航、open/reveal与provider capability。
- **ED57-P1-24**：source加入、卸载、重挂载或权限变化没有qualified source-tree delta与selection/history reconciliation。

### 4.4 Create/Rename/Move/Delete/Duplicate/Reimport操作面（ED57-P1-25至P1-32）

- **ED57-P1-25**：event、state、UI和operation projection均没有Create Folder。
- **ED57-P1-26**：没有Rename Asset/Folder入口、inline edit、名称预检、case-only rename、冲突建议与reference-repair receipt。
- **ED57-P1-27**：没有Move Asset/Folder入口、destination picker、source/provider preflight、cross-source policy或atomic publication。
- **ED57-P1-28**：没有Delete入口、referencer/dirty/open-document/source-control影响预览、trash policy或可撤销结果；通用删除语义继续由Editor55拥有。
- **ED57-P1-29**：没有Duplicate入口、unique-name negotiation、subasset/reference remap、目标目录选择与new item reveal；portable clone语义继续由Editor55拥有。
- **ED57-P1-30**：没有Reimport入口、recipe/options/source dependencies选择、批量范围、stale artifact状态与失败保留last-known-good。
- **ED57-P1-31**：没有Refresh/Rescan Source的显式产品命令、范围选择、generation receipt和当前后台扫描状态。
- **ED57-P1-32**：没有bulk operation plan；多选动作所需的per-item can/do、共同destination、partial policy、rollback/compensation与结果表均不存在。

### 4.5 Quick Import与创建编排（ED57-P1-33至P1-40）

- **ED57-P1-33**：可见文案是通用`Quick Import`/`Import`，实际只接受OBJ、glTF与GLB模型；必须改成Model Import或接入importer registry chooser。
- **ED57-P1-34**：格式选择硬编码在`canonical_model_source_path`，没有按source probe/descriptor展示可用importer、冲突与recipe；算法owner为Editor04/Runtime85。
- **ED57-P1-35**：外部source固定stage到primary `assets/models`，忽略当前selected folder、source writability和用户目标选择。
- **ED57-P1-36**：导入成功后隐式调用`runtime.import_mesh_asset`把模型插入当前scene，资产操作与scene authoring副作用没有明确命令边界。
- **ED57-P1-37**：copy source、import model、生成animation assets、逐项import、准备default material与scene insert不在一个transaction/operation plan中，失败可留下部分产物。
- **ED57-P1-38**：derived skeleton/clip逐个生成和导入，没有batch publication、rollback、idempotency key或partial receipt。
- **ED57-P1-39**：default material通过独立`import_asset`隐式创建/加载，失败与重复调用没有成为主操作的显式dependency outcome。
- **ED57-P1-40**：Quick Import没有progress/cancel、import options、overwrite/conflict决策、diagnostic list、source provenance、open/reveal created assets和retry入口。

### 4.6 Watcher、外部变化与状态协调（ED57-P1-41至P1-48）

- **ED57-P1-41**：asset/resource/editor change只按URI/UUID聚合为刷新布尔值，没有project、document、browser instance、source generation与producer qualification。
- **ED57-P1-42**：外部变化没有统一`AssetExternalChangeDecision`，无法表达auto-accept、dirty conflict、defer、ignore、reload、merge或failed。
- **ED57-P1-43**：catalog、details、preview、resource与scene refresh分步执行，没有共同expected generation和单一publication receipt。
- **ED57-P1-44**：refresh plan丢弃原始change identity/reason/range，失败后无法准确重试、诊断或证明哪些变化已消费。
- **ED57-P1-45**：外部rename/move保留UUID时没有原子更新selected folder、history entry、expanded path、focus与open document route。
- **ED57-P1-46**：resource change主要转成render/presentation invalidation，没有面向用户的asset currentness、reload failed或last-known-good状态。
- **ED57-P1-47**：default scene reload在retained host同步重新open ProjectManager并`scan_and_import`，没有job admission、cancel、deadline、progress或UI-thread budget。
- **ED57-P1-48**：project deactivation/source removal没有统一关闭inline action、drag、selection、preview lease、history与pending operation的instance teardown receipt。

### 4.7 Drag/drop、favorites、collections与工作流（ED57-P1-49至P1-56）

- **ED57-P1-49**：typed asset drag payload只服务reference/property等consumer，没有folder/source destination的Move/Copy plan。
- **ED57-P1-50**：folder/item没有drag enter/over/leave/drop provider validation，不能展示invalid destination、read-only、cycle、conflict或partial原因。
- **ED57-P1-51**：没有modifier-driven copy/move/link语义、跨provider transfer policy、drop effect或提交后的qualified receipt。
- **ED57-P1-52**：没有Favorites模型、source-tree projection、持久化、missing target状态或跨项目qualification。
- **ED57-P1-53**：没有静态/动态Collections、collection provider、membership transaction、shared/read-only状态或collection history。
- **ED57-P1-54**：没有Recently Opened/Recently Imported/Modified/Conflict等virtual source，用户只能靠物理目录与局部搜索定位资产。
- **ED57-P1-55**：没有provider-owned virtual folders、generated/cooked/developer/plugin分类或按capability显示的source actions。
- **ED57-P1-56**：keyboard、menu、toolbar、context menu、drag/drop和automation没有收敛到同一asset command/action provider，未来继续加按钮会扩大语义漂移。

## 5. P2：质量、性能与可维护性差距

- **ED57-P2-01**：`PREVIEW_IMAGE_CACHE`是thread-local无界`HashMap<String, Image>`，没有entry/byte预算、LRU、project teardown或generation eviction。
- **ED57-P2-02**：presentation路径直接`Image::load_from_path`，cache miss可在UI构建时同步做文件读取/解码。
- **ED57-P2-03**：preview key随revision/dirty变化会永久保留旧Image，load失败还会缓存default image，缺retry/currentness策略。
- **ED57-P2-04**：每次snapshot递归materialize完整folder tree，没有展开驱动的lazy page或增量tree delta。
- **ED57-P2-05**：visible folder/assets每次对catalog线性扫描和分配；query/index优化由Editor56拥有，Browser仍需paged consumer。
- **ED57-P2-06**：Explorer由Activity完整snapshot clone而来，避免了第二次业务遍历，却会复制大Vec/String；应共享immutable page/model。
- **ED57-P2-07**：`asset_browser.zui`接近900行并混合toolbar/source/content/details/preview/reference/plugin面，后续action扩展需要模块化组件边界。
- **ED57-P2-08**：大量可见文案硬编码英文，Browser command/status/diagnostic没有localization key；通用i18n owner为Editor12/33。
- **ED57-P2-09**：selection、tree、item activation、context menu和drag没有完整keyboard/focus/accessibility state/action合同。
- **ED57-P2-10**：现有pointer/layout/screenshot测试覆盖单击和结构，但没有真实double-click/Enter/open toolkit/rename/drop/context command端到端证据。
- **ED57-P2-11**：没有10万/100万资产、多source、深目录、preview churn和多Browser实例的memory/latency/soak基线。
- **ED57-P2-12**：缺少action latency、catalog-to-visible延迟、cache residency、stale receipt、conflict、partial/rollback和后台reload拦截的结构化telemetry。

## 6. 参考引擎对照与适用结论

| 参考 | 本轮源码事实 | Zircon应吸收 | 不应照抄 |
|---|---|---|---|
| Unreal | `UContentBrowserDataSource`由provider拥有Can/Delete/Rename/Copy/Move、bulk和完整drag enter/over/leave/drop；`SContentBrowser`有per-instance history、favorites、collections、rename/delete command和search history；RenameManager处理referencer、soft path、redirector、source control与save | provider-owned capability/action、per-browser state、bulk preflight、rename/move control plane、collection/favorite product面 | 不复制UObject/package细节，也不把redirector当成修复稳定identity的借口 |
| Godot | FileSystem dock/editor file system包含open、move/rename、duplicate/delete、import/reimport/rescan、favorites/history与dependency更新链 | 文件系统变化与Editor状态协调、显式reimport/rescan、用户可见错误与依赖处理 | 不把物理文件路径重新提升为唯一资产identity |
| Fyrox | asset item双击打开，drag/drop到目录前调用`can_resource_be_moved`，通过`MoveTo`消息和resource manager执行；Browser维护selection/path | 最低产品闭环必须含activation、destination preflight和真实move结果 | Fyrox本身不是本项目的性能/大规模目标上限 |
| Bevy | `AssetPath`分离source、path与label；`AssetSource`可有reader/writer/watcher和processed/unprocessed端；事件含Added/Modified/Removed/Unused/LoadedWithDependencies | source/provider identity、subasset label和source能力必须进入合同 | Bevy不是Editor产品，不从其缺少Content Browser推导Zircon也可省略工作流 |
| Unity Graphics | 本地包内consumer通过ProjectWindow启动命名、unique path、CreateAsset/Save/Refresh/ShowCreatedAsset；批量reimport用Start/StopAssetEditing、progress与`finally`闭合 | 创建后inline naming/reveal、批量边界、progress和finally式收尾 | 本地corpus不含完整Unity Editor AssetDatabase/Project Browser，只作consumer旁证，不宣称其内部事务细节 |

结论不是“复制一个Unreal窗口”。必须先建立Zircon自己的exact identity、source provider、transaction/receipt和instance state，再让UI消费这些合同；否则增加收藏夹、右键菜单或拖拽只会给临时路径逻辑增加更多入口。

## 7. 目标架构与核心合同

### 7.1 分层

```text
AssetWorkspaceService
  -> AssetBrowserInstanceRegistry
     -> AssetBrowserInstanceState(history, source, query session, selection, expansion)
  -> ContentSourceProviderRegistry
     -> ContentSourceSnapshot / FolderPage / AssetItemPage / CapabilitySet
  -> AssetActivationRouter
     -> exact AssetTypeId -> enabled toolkit -> AssetActivationReceipt
  -> AssetActionProviderRegistry
     -> availability + preflight + operation invocation + result projection
  -> AssetMutationCoordinator
     -> prepare -> dirty/source/reference checks -> commit/publish -> receipt/compensation
  -> AssetChangeReconciler
     -> qualified external delta -> document decision -> instance reconciliation
  -> AssetCollectionService
     -> favorites / static collection / dynamic collection / virtual source
```

### 7.2 最小typed合同

```rust
struct AssetBrowserInstanceId(Uuid);

struct QualifiedAssetItemId {
    project_id: ProjectId,
    source_id: ContentSourceId,
    asset_uuid: AssetUuid,
    subasset: Option<SubassetId>,
    catalog_generation: u64,
}

struct AssetItemDescriptor {
    id: QualifiedAssetItemId,
    locator: AssetUri,
    asset_type: AssetTypeId,
    schema_version: SchemaVersion,
    compatibility: AssetCompatibilityState,
    capabilities: AssetCapabilitySet,
}

enum AssetActionOutcome {
    Rejected(AssetActionDiagnostic),
    Queued(AssetOperationTicket),
    Completed(AssetMutationReceipt),
    Partial(AssetPartialReceipt),
    Conflict(AssetConflictReceipt),
    Cancelled(AssetCancellationReceipt),
}
```

`AssetMutationReceipt`至少携带operation id、idempotency key、source/catalog generation before/after、old/new qualified ids、reference repair receipt、document transition receipt、created/removed artifacts、rollback/compensation状态和per-item diagnostics。UI不得从“目录刷新后看起来变了”反推操作成功。

### 7.3 状态原则

1. 每个Browser窗口有独立instance state；Activity可以订阅同一workspace service，但不能共享可变navigation/selection。
2. selection保存qualified identity与generation，locator只作展示/导航提示。
3. source provider声明读写、创建、重命名、移动、复制、删除、导入、watch与virtual collection能力。
4. exact AssetTypeId从Runtime catalog一直保留到item、selection、reference、toolkit和operation target。
5. 所有长操作返回ticket并通过Editor09 job投影progress/cancel；所有mutation结束于receipt。
6. 外部变化先进入reconciler，再决定catalog/UI/document/world副作用，绝不直接替换dirty world。
7. UI只渲染action provider给出的availability与diagnostic，不根据path scheme或字符串扩展名猜权限。

## 8. 依赖顺序与重构里程碑

### M57.0 · 能力真实性硬切

关闭四项P0：禁用后台直接scene replace；修正/禁用伪Locate；给item接入统一activation；在exact type未闭合前对未知/插件类型显示Unavailable而非伪装builtin。增加四条失败先行产品测试。

### M57.1 · Exact item/source合同

与Runtime86协作让AssetTypeId/schema/compatibility进入catalog；引入`ContentSourceId`、provider、folder/item stable id、source capability与generation。保留现有ResourceKind作为coarse presentation字段。

### M57.2 · Browser实例状态

引入`AssetBrowserInstanceId`、独立navigation/query/selection/expansion/view/utility state，完成back/forward、lazy tree、multi-selection、focus/anchor、persist/restore与external move reconciliation。

### M57.3 · Activation与action provider

统一double-click、Enter、menu、context和automation activation；接入enabled toolkit lifecycle、Open With、context command、creation template与可见availability/diagnostics。

### M57.4 · Mutation coordinator

把Create Folder、Rename、Move、Delete、Duplicate、Reimport和bulk preflight接到Editor55/Runtime87父合同；实现expected generation、dirty/source/reference检查、transaction、rollback/compensation及receipt。

### M57.5 · Import/create产品编排

把Quick Import改成registry-driven import operation，明确目标folder、recipe/options、progress/cancel、derived outputs、partial policy和created item reveal；资产导入与scene placement拆成两个显式command。

### M57.6 · External change reconciler

用qualified delta替代布尔refresh plan，接入Editor02/03 dirty transition、open document/toolkit、selection/history/preview和last-known-good；后台工作经Editor09 admission。

### M57.7 · Drag、favorites、collections与virtual source

以同一action/mutation provider实现folder drop preflight、copy/move/link；增加project-qualified favorites、static/dynamic collections、recent/imported/conflict virtual sources。

### M57.8 · 大规模与资格闭合

完成paged source/item/tree、bounded decoded preview cache、异步decode、100k/1M规模、多实例/多source、fault/rollback、keyboard/a11y、localization、telemetry和动态产品验收。

## 9. 资格门（44项）

### Activation与能力真实性

- **ED57-G01**：可见item双击与Enter均打开或复用exact toolkit，并返回同类型receipt。
- **ED57-G02**：单击只改变selection，不触发隐式open；双击判定不产生两次错误selection副作用。
- **ED57-G03**：无toolkit/缺插件/版本不兼容时显示明确Unavailable原因，不回退成错误builtin editor。
- **ED57-G04**：Locate携带qualified target，能展开、导航、滚动并聚焦；无selection/stale/filter阻挡有可见结果。
- **ED57-G05**：toolkit/context/creation action是真实可执行control，不再只显示descriptor文本。
- **ED57-G06**：keyboard、toolbar、menu、context、drag与automation对同一action得到同一availability/preflight。
- **ED57-G07**：descriptor-level capability与global command capability都在projection和invoke时校验。
- **ED57-G08**：所有accepted action产生operation id与terminal receipt，没有静默成功。

### Instance、source与selection

- **ED57-G09**：两个Browser实例可保持不同folder/query/selection/history，互不覆写。
- **ED57-G10**：back/forward/up在rename/move/source removal后按policy恢复或报告stale。
- **ED57-G11**：source tree只materialize展开分支，并能reveal qualified item路径。
- **ED57-G12**：10万folder深/宽fixture下展开、折叠与navigate满足预算且无全树clone。
- **ED57-G13**：多选支持anchor/range/toggle、keyboard focus和稳定顺序，catalog delta后按identity协调。
- **ED57-G14**：外部move保留UUID时folder/list/details/focus/history在一个reconciliation receipt后相符。
- **ED57-G15**：多个project content source和package/remote/read-only source保留独立identity/capability。
- **ED57-G16**：project关闭/重开时instance state有schema migration，旧project identity不会串入新project。

### Mutation与import

- **ED57-G17**：Create Folder支持名称预检、unique suggestion、source capability、undo/receipt与reveal。
- **ED57-G18**：Rename支持case-only、冲突、open document、referencer/source-control preflight及原子结果。
- **ED57-G19**：Move支持folder cycle检测、跨source policy、reference repair与失败保留原位置。
- **ED57-G20**：Delete显示referencer/dirty/open-document影响，失败不丢资产或authoring状态。
- **ED57-G21**：Duplicate保留exact type/schema并完成identity/reference remap，新item可定位。
- **ED57-G22**：Reimport保留last-known-good，展示recipe/source dependency/progress/cancel与诊断。
- **ED57-G23**：bulk action逐项preflight、明确all-or-nothing/partial policy并返回结果表。
- **ED57-G24**：mutation receipt含before/after generation、old/new identity、artifact/reference/document outcome。
- **ED57-G25**：导入目标默认当前writable folder，可显式改选；read-only source不会隐式stage到别处。
- **ED57-G26**：Import Asset与Place in Scene为两个显式、可独立撤销/失败的command。

### External change与生命周期

- **ED57-G27**：dirty default scene收到外部变化时不替换world，进入可测试的decision/conflict状态。
- **ED57-G28**：clean scene reload以expected document/revision执行，失败保留旧world与history。
- **ED57-G29**：asset/resource/editor事件保留producer、project/source、identity、sequence与generation。
- **ED57-G30**：catalog/details/preview/resource/document协调结果有单一receipt，不出现半新半旧可见状态。
- **ED57-G31**：event gap/overflow触发bounded resync并标记currentness，不静默继续。
- **ED57-G32**：长scan/import/reload经job admission，支持progress/cancel/deadline/shutdown。
- **ED57-G33**：project/plugin/source teardown撤销pending callback/preview/drag/action lease且无跨project回调。
- **ED57-G34**：插件reload期间exact type/toolkit generation fenced，旧toolkit不接收新资产activation。

### 性能、质量与动态产品证据

- **ED57-G35**：decoded preview cache有entry/byte预算、LRU、generation/project eviction与residency telemetry。
- **ED57-G36**：UI presentation线程不做未预算文件读取/图片解码。
- **ED57-G37**：10万资产目录navigation/selection/action p95/p99有基线，1M索引通过paged provider工作。
- **ED57-G38**：多个Browser实例共享immutable catalog/page，不按实例复制完整资产字符串集合。
- **ED57-G39**：连续import/reimport/preview churn soak后内存回落，无旧cache key无限增长。
- **ED57-G40**：所有item/tree/action有keyboard focus、semantic name/state/action和screen-reader验证。
- **ED57-G41**：可见文案、状态、错误与动态action display name使用localization key并可热切locale。
- **ED57-G42**：真实Editor动态测试覆盖open、locate、create、rename、move、delete、reimport、drop和external conflict，不以直接构造event替代入口。
- **ED57-G43**：fault injection覆盖磁盘满、权限、source失联、plugin卸载、partial artifact/reference repair和rollback failure。
- **ED57-G44**：产品资格报告能证明每个可见action的authority、preflight、operation、receipt、currentness与失败恢复。

## 10. 验证矩阵

| 层级 | 必须新增的证据 | 不能替代的证据 |
|---|---|---|
| Unit | instance history、selection reducer、source capability、action availability、mutation receipt、change reconciliation | 字符串contains/source-shape测试 |
| Contract | exact type从Runtime record到toolkit、provider can/do、Editor55/Runtime87 mutation handoff、Editor02/03 dirty transition | 直接调用底层handler后断言Ok |
| UI integration | 真实pointer double-click、Enter、context menu、inline rename、drag/drop、progress/cancel、focus/reveal | ZUI节点存在或截图像素相似 |
| Product E2E | create/import/open/rename/move/delete/reimport、外部scene变化、plugin reload、多Browser/多source | 只刷新catalog后观察行数变化 |
| Fault | permission/disk/source/plugin/cancel/crash、partial commit、rollback failure与recovery | happy-path临时目录测试 |
| Performance | 100k/1M、深/宽树、多实例、preview churn、memory residency、p95/p99 | debug build单次wall-clock |
| Accessibility | keyboard-only、focus restore、screen reader、reduced motion、locale switch | 仅检查control_id或label字符串 |

本轮是review-only，没有运行Cargo、Editor、真实文件mutation、watcher conflict、plugin reload、screen reader、fault、soak或benchmark。实施验收必须在prefer-windows-validation规则下选择命令和target；只有明确Linux-specific要求才进入WSL。

## 11. 实施硬规则

1. 不新增“临时可用”的path-string action；所有目标必须是qualified identity，path只作显示/提示。
2. 不在ZUI callback里直接做filesystem、scan/import、scene replacement或reference rewrite。
3. 不为Browser另建importer、query、clipboard、dirty、job或reference subsystem；只消费唯一owner合同。
4. 不以刷新catalog后观察到变化作为mutation成功；必须消费terminal receipt。
5. 不保留旧事件作为兼容shim。P0硬切时同步迁移caller/test并删除伪语义variant。
6. 不把插件未知类型降级成builtin Data；保留opaque exact type和兼容性诊断。
7. 不把Bevy/Fyrox的局部简洁实现当作Zircon规模上限；用Unreal/Godot工作流证明产品闭环，用本项目typed/generation/transaction规则提高安全和性能。
8. 不在本轮或后续实现中优化tooling；用户已明确tooling之后迁移到Rust，本报告只定义Editor/Runtime产品合同。

## 12. Currentness与后续复核

1. 实施前重新导出209份focused manifest，重算Zircon fingerprint并逐项复核22份dirty文件。
2. 全仓重查`EditorAssetEvent::OpenAsset`生产构造、`LocateSelectedAsset`、`invoke_asset_context_command`、`reload_default_scene`和`AssetTypeId::from_resource_kind` caller。
3. 重读Editor02/03/04/08/09/50/55/56与Runtime85/86/87最新owner状态，任何已关闭项从本报告删除或改为consumer gate。
4. Unity Graphics本地corpus仍不包含完整Unity Editor AssetDatabase/Project Browser；除非本地参考树补齐，不得扩大Unity结论。
5. 参考引擎升级或目录移动后重算17文件fingerprint，并记录revision/适用边界。

本报告完成的是当前源码的首轮纵向深审和重构建账，不表示任何P0/P1/P2已经实现，也不表示动态产品资格已经通过。
