---
related_code:
  - zircon_editor/src/ui/asset_editor
  - zircon_editor/src/ui/host/asset_editor_sessions
  - zircon_editor/src/ui/host/editor_manager_asset_editor
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor.rs
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor
  - zircon_editor/src/ui/retained_host/app/ui_asset_editor_detail_events
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_ui_asset_conversion
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/ui_asset_detail_fields
  - zircon_editor/src/core/asset/type_registry/builtin.rs
  - zircon_editor/assets/ui/editor/ui_asset_editor.zui
  - zircon_editor/assets/ui/editor/windows/ui_layout_editor_window.zui
  - zircon_editor/assets/ui/editor/components/workbench/composites/chrome/workbench_ui_asset_action_bar.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/ui/workbench_hud_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui
  - zircon_plugins/ui_asset_authoring
  - zircon_plugins/ui_document_importer
  - zircon_plugins/first_party_editor_catalog
  - zircon_runtime/src/ui/v2
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/WidgetBlueprintCompiler.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/WidgetBlueprintFactory.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/WidgetBlueprintEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/Designer/SDesignerView.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/Hierarchy/SHierarchyView.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/Palette/SPaletteView.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/Preview/SWidgetPreview.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/Customizations/WidgetNavigationCustomization.cpp
  - dev/godot/editor/scene/gui/control_editor_plugin.cpp
  - dev/godot/editor/scene/gui/theme_editor_plugin.cpp
  - dev/Fyrox/editor/src/ui_scene/mod.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 23 · UI Asset / HUD / Widget / Binding / Theme / Icon / Accessibility / Menu Flow / Font Atlas Authoring 工程化差距

## 1. 结论

Zircon的真实UI Asset Editor不是一组临时按钮。当前源码已经有typed session、V1/V2解析与last-valid preview、真实`UiSurface`/`UiV2SurfaceBuilder`构建、hierarchy/palette/slot-aware drop、component/reference导航、style cascade与theme compare/refactor、binding CRUD与payload projection、source outline、undo/redo、外部文件watcher、冲突快照、autosave/recovery接入和retained host投影。349个选定文件中有386个test attribute，说明这里已经形成较大的可保留实现，不能以Workbench静态页面为由整体推倒重写。

但这套实现当前存在五项P0。第一，V2视觉编辑经过legacy投影回写，明确丢弃`repeat`、node `slots`、不可达节点和`ThemeTokens`种类，同时全量pretty-print丢失文本trivia；一次合法视觉编辑即可静默改坏资产。第二，Save先在内存中`mark_saved`再直接`fs::write`，写失败会把未落盘内容显示为clean，写成功后的import失败又被吞掉。第三，toolkit允许同一资产多实例，watcher也能发现冲突，但普通Save与“Keep Local and Save”走同一路径并无revision compare-and-swap，可静默覆盖外部或另一实例修改。第四，promote、undo和redo把session/stack mutation与跨文件`write/remove/import`分开执行，任何中途失败都会留下部分提交。第五，可选`ui_asset_authoring`插件引用的四份ZUI资源全部不存在，三个create operation没有factory，默认Editor catalog也不装配该插件；已有资产能打开，不等于用户能可靠创建新资产。

产品完整性同样明显不足。preview preset只是四组硬编码像素，locale选择不改变真实preview文字和布局，Preview Interact只生成binding metadata且没有产品caller；designer没有删除/复制粘贴/缩放/标尺/guide/snap/批量对齐等基本工作流；a11y、Menu Flow、Icon Library和Font Atlas都只是Workbench固定样例，未消费Runtime11A/11B/11C的真实accessibility、font/glyph/atlas或input authority。Binding suggestion甚至用control id/text中的`save`推测`menu_action.workbench.project.save`，并非由action/route schema驱动。

参考源码给出的门槛不是“界面看起来像编辑器”。Unreal `WidgetBlueprintFactory`创建真实asset并校验parent/root；`WidgetBlueprintCompiler`拥有generated class、widget tree、binding与animation诊断；Designer、Hierarchy和Palette围绕preview instance、transaction、drag/drop、filter、safe zone和device resolution工作。Godot Control/Theme editor通过`EditorUndoRedoManager`编辑真实resource、anchor/container与theme；Fyrox UI scene有独立scene、command stack、clipboard、selection、interaction mode、render target resize和每帧update。Zircon可以保持自己的TOML/V2架构，但必须达到同样的authority、transaction、preview fidelity和failure contract。

本轮登记5项P0、60项P1、12项P2。顺序必须先冻结V2无损合同，建立带revision的原子Save和跨资产transaction，再修复创建/catalog闭环；随后完善designer/preview/binding/theme/a11y/font/icon产品能力，接通cook/runtime authority和性能门禁，最后删除Workbench静态第二authority。Runtime UI执行、text/font和GPU UI分别由11A、11B、11C拥有，本篇只建设authoring、diagnostic、preview和artifact消费链，不复制运行时实现。

## 2. 审查边界与证据

### 2.1 当前工作树物理范围

| 子域 | 文件 / 行数 / bytes | test attributes | 证据等级 |
|---|---:|---:|---|
| UI Asset核心model/session/editor算法 | 95 / 25,399 / 889,169 | 27 | E3：binding、preview、source、tree、style、theme、undo与V2 projection逐文件 |
| Host/session/product wiring | 141 / 10,887 / 399,663 | 35 | E3：open/edit/save/watcher/import hydration/retained actions/detail events/pane projection |
| UI插件、asset registry与first-party catalog | 22 / 1,777 / 63,945 | 22 | E3：manifest、descriptor、资源URI、create template、catalog装配与测试 |
| 真实Editor与七份Workbench UI surfaces | 10 / 2,681 / 149,798 | 0 | E3：2个真实shell、action bar、HUD与6个extension逐control |
| Workbench route、feedback与binding | 12 / 4,814 / 210,306 | 2 | E3：从route到固定feedback/control mutation逐分支 |
| Runtime V2、accessibility、icon/font anchor | 19 / 4,884 / 167,498 | 0 | E2/E3：只用于验证Editor投影与runtime合同，不重开11A/11B/11C |
| focused Editor tests | 50 / 20,270 / 698,888 | 300 | E3静态阅读：editing、host、retained、UI、registry与Workbench布局 |
| selected combined scope | 349 / 70,712 / 2,579,267 | 386 | 当前工作树fingerprint `9e2c26ecdac9a059451c8d3d7c14516b4f036e8e13e00b7309b1a23e94f0c126`；0 ignored，40个在途文件 |

行数为物理文本行。fingerprint按相对路径排序，对当前工作树每个选定文件计算SHA-256，再对`path<TAB>hash<LF>`清单计算SHA-256。40个范围内文件已有非本轮产生的修改，主要位于UI Asset核心、focused tests、Workbench route和`zircon_runtime/src/ui/v2/surface_builder.rs`；本轮按现状取证，不吸收、不回退。实施前必须重算fingerprint，逐项复核P0和产品caller，因为这些在途修改可能继续演进。

### 2.2 动态证据边界

本轮没有运行新的Cargo、Editor窗口、字体atlas、screen reader、IME、DPI/device preset或性能测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误和122个warning阻断；在相关源码未修复前重复同一lane不能增加证据。386个test attribute是静态inventory，不是通过数：其中大量测试能证明session mutation、projection和UI control写入，但不能证明磁盘原子性、并发冲突、真实localization、屏幕阅读器、font cook、GPU atlas、跨平台输入或大资产性能。

### 2.3 参考边界

- Unreal UMG用于判断factory/compiler/designer/hierarchy/palette/preview/navigation的owner和产品闭环，不要求复制Blueprint或Slate类型层级。
- Godot Control与Theme editor用于判断anchor/container、theme resource和统一undo/redo的产品下限；它不替代Zircon V2 schema设计。
- Fyrox UI scene用于判断独立编辑scene、clipboard、selection、interaction mode、render target和command ownership；其规模较小，不构成降低目标的理由。
- Bevy在本地主要提供runtime UI/ECS组合参考，没有同级first-party可视化UI editor，因此只作为runtime contract佐证，不作为authoring完成标准。
- 本地Unity Graphics仓不包含UI Builder/TextCore的完整权威源码。本文不猜测Unity闭源Editor行为，只在后续icon/font资源管线中引用可直接证明的graphics resource/atlas做法。

## 3. 必须保留的真实基础

1. 保留`UiAssetEditorSession`的typed document、source、last-valid、selection、preview、style、theme、binding、palette、undo和diagnostic状态，拆分owner而非退回字符串表单。
2. 保留V1通过`EditorTemplateRuntimeService`、V2通过`UiV2SurfaceBuilder`构建真实surface并计算layout的preview路径；修的是状态/input/device fidelity。
3. 保留native/component/reference的palette和slot-aware drop resolution，以及reparent、wrap、unwrap、extract、promote和move算法。
4. 保留style rule identity、matched cascade、pseudo state、imported theme merge/compare/refactor和local override工具；改造成schema驱动的设计系统编辑器。
5. 保留binding entry/payload CRUD、runtime diagnostic projection与last-good source策略；endpoint schema和action registry必须成为新authority。
6. 保留bounded、generation-aware watcher/refresh和external diff snapshot；将它接入revision-qualified Save，而不是另写一个watcher。
7. 保留document toolkit、autosave、recovery和multi-instance产品入口；统一交给Editor02的transaction/save/recovery协议。
8. 保留asset type registry对`UiLayout`、`UiWidget`、`UiStyle`的真实toolkit映射，修复create/catalog/plugin缺口。
9. 保留稳定control/action identity和retained pane projection，逐项替换静态业务数据，不把业务状态重新塞回ZUI字符串。
10. 保留现有focused tests作为回归基线，但把结构断言升级为无损roundtrip、失败注入、真实surface/input、package和视觉证据。

## 4. 目标架构

```text
UiAuthoringProduct
  -> UiAssetFactory + TemplateCatalog + MigrationRegistry
  -> UiDocumentRepository(asset_id, document_revision, source_revision)
       -> lossless syntax tree + typed V2 semantic model + unknown-field sidecar
       -> DocumentTransaction / UndoGroup / CrossAssetTransaction
       -> AtomicSave(compare revision -> temp -> flush -> rename -> reimport receipt)
       -> Autosave/Recovery/ConflictResolver

  -> UiDesignerSession
       -> editable preview scene + authoritative selection
       -> hierarchy/palette/clipboard/drag-drop/transform tools
       -> device/DPI/safe-zone/locale/input/accessibility preview profiles
       -> actual UiSurface input dispatch + state/action trace

  -> UiSchemaRegistry
       -> widget/property/slot/default/inheritance metadata
       -> action/route/binding payload schemas + refactor index
       -> style/token/theme/font/icon/accessibility/menu schemas

  -> Runtime Evidence Providers
       -> compiler/cook artifact + dependency generation
       -> Runtime11A accessibility/input/layout snapshot
       -> Runtime11B font/fallback/shaping/glyph completeness
       -> Runtime11C atlas/residency/batch/clip/submit snapshot

  -> Workbench presentation
       -> projects the same sessions/providers
       -> no fixture state, fixed success string, or second document authority
```

`asset_id + document_revision + source_revision + dependency_generation + preview_generation + runtime_generation`必须贯穿command、diagnostic、job、save、preview和runtime report。任何异步结果、外部修改或GPU/font证据如果generation不匹配，只能标记stale/pending，不得覆盖较新的编辑状态。

## 5. P0：先关闭数据损坏、并发覆盖与不可达创建

### P0-1：V2视觉编辑经lossy legacy投影回写，会静默删除合法语义

`session/lifecycle/v2_projection.rs`先把V2转为legacy `UiAssetDocument`供编辑，再由`legacy_projection_document_to_v2_document`重建V2。重建只遍历root和component root可达树；`flatten_legacy_projection_nodes_into`硬编码`repeat: None`和`slots: BTreeMap::new()`；`ThemeTokens`先降为legacy Style，再回写为`UiV2AssetKind::Style`。不可达但合法的node、repeat、node slots和asset kind因此都可丢失。最终`toml::to_string_pretty`全量重写文档，comments、原始顺序和trivia也没有保存。

这不是格式美观问题，而是一次普通inspector/tree/style操作即可删除运行时compiler会消费的V2字段。现有测试只覆盖部分reference/component/named child mount与root规则，没有repeat、node slots、ThemeTokens、orphan/forward-compatible field或trivia的roundtrip门禁。

必须建立单一typed V2 editing model与lossless syntax mapping。所有mutation在typed node/slot/repeat/style结构上执行，并通过stable node identity做最小文本patch；未知字段和unsupported future variant必须原样保留。迁移必须有schema version、dry-run diff、backup和typed diagnostic。在无损门禁完成前，包含未支持语义的V2资产只能进入显式read-only/source-only模式，不能假装可视觉编辑。

### P0-2：Save在磁盘提交前清理dirty，写入和reimport也不是原子事务

`save_ui_asset_editor_canonical`先锁session并调用`save_to_canonical_source`；该方法替换canonical source后立即`mark_saved()`，随后host才执行`fs::write`。写盘失败时，session已经显示clean且source已canonicalize，磁盘仍是旧内容。写盘使用直接覆盖，没有同目录temp、flush、atomic rename、directory durability或recoverable backup；写成功后的`import_asset`结果被`let _ =`吞掉。local copy也是直接写入。

必须由`UiDocumentRepository`持有Save transaction：先对指定revision生成候选bytes与dependency manifest，写入同文件系统temp并flush，验证可重新解析/导入，执行compare-and-swap和atomic replace，取得reimport receipt后才推进saved revision。失败保持dirty并发布可重试diagnostic；进程崩溃后temp/recovery有明确清理/恢复协议。canonicalization不能在提交前改变用户可见saved state。

### P0-3：多实例和外部修改冲突只被展示，普通Save仍可无条件覆盖

UI asset descriptor明确`with_multi_instance(true)`，watcher、external conflict、last-known-good和diff snapshot也确实存在。但普通Save不检查`entry.conflict`或open-time source revision；“Keep Local and Save”最终调用同一save路径。任一实例按下Ctrl+S都可覆盖磁盘上由另一实例、外部工具或source control更新的内容，写后又清空conflict/diff。

必须以内容hash、文件identity和document revision建立optimistic concurrency。Save要求base revision仍匹配；不匹配时只能选择reload、three-way merge、save copy或带明确force权限的replace。多实例应共享document owner或通过lease/revision广播同一资产的accepted commit。冲突决策、merge结果和force override进入journal，不允许普通Save隐式等价于Keep Local。

### P0-4：promote与undo/redo跨内存、文件和asset import的副作用可部分提交

promote widget/theme先修改session并压入undo，再创建目录和直接写外部资产；写失败时当前文档与undo已经改变。undo/redo先从stack pop并把entry压到对侧，再释放session lock逐个执行external effects；`write/remove/import`中途失败会留下stack已前进、部分文件已改变、其余文件未改变。import/reimport失败仍被吞掉。

必须引入`CrossAssetTransaction`：声明read/write set、base revisions、目标路径、文件操作、asset registry变更和inverse；预校验所有目标后写staging，统一commit或rollback，最后原子推进document与undo stack。undo/redo重放同一个transaction receipt，而不是重新猜测外部状态。故障注入要覆盖第N个write、rename、remove、import、watcher race和崩溃恢复。

### P0-5：UI资产创建插件引用不存在资源且无operation factory，默认产品不可创建

`ui_asset_authoring`声明`plugins://ui_asset_authoring/editor/authoring.zui`和layout/widget/style三个template URI，但四个文件均不存在。`ui_asset.create.layout/widget/style`只出现在descriptor注册，仓内没有对应operation factory/handler；通用create template只会invoke operation。`first_party_editor_catalog`默认只装配Navigation与Neural，没有装配此插件。插件测试只证明descriptor字符串存在，没有检查资源解析或端到端创建。

必须决定UI authoring是builtin核心能力还是稳定first-party plugin，并只保留一个owner。factory要创建可解析的versioned V2 asset、分配stable asset identity、处理目录/命名冲突、写入transaction、import并打开真实toolkit。plugin/package测试必须校验所有URI存在，factory可执行，default catalog能发现；若暂未达到，则manifest和菜单必须显示Unavailable/Experimental，不能注册死入口。

## 6. P1：Document、Schema 与生命周期

### P1-1：canonical source是全量重序列化，不是lossless增量编辑

建立CST/semantic双视图、stable span mapping和minimal patch；保留comments、表顺序、空白、unknown fields和用户手工组织，只有显式Format命令才全量格式化。

### P1-2：V1/V2双模型没有明确迁移生命周期

定义支持矩阵、read-only策略、V1到V2 migration、版本升级/降级限制和deprecation telemetry；不能长期以legacy投影作为公共最小公倍数。

### P1-3：unknown future field没有forward-compatible保存合同

parser/model必须携带unknown field/variant sidecar与source span；Editor不理解的语义可诊断但不得删除，cook/compiler自行决定拒绝或透传。

### P1-4：open、autosave、save copy和recovery使用不同提交语义

统一到Editor02 document repository，所有路径共享revision、validation、atomic write、journal和reimport receipt；save copy只改变target，不绕过验证。

### P1-5：import hydration在编辑热路径同步递归读取文件

每次source变更可递归`read_to_string`/parse imports；visited set只能防cycle，不能限制深度、文件数、字节或CPU。改为background dependency job、typed budget、cancellation和generation-qualified commit。

### P1-6：递归import traversal有深链栈溢出与大图阻塞风险

改用显式queue/stack与最大depth/node/edge/bytes/time；输出cycle path、truncation和pending依赖，不在UI线程遍历任意深度。

### P1-7：每次完整source update触发重验证、hydration和presentation同步

建立增量parse、debounce、priority lane与generation cancellation；typing只更新局部syntax/diagnostic，完整compile/preview按稳定输入或显式请求执行。

### P1-8：undo entry和replay artifact重复clone完整source/document

改用structural edit log、persistent data或chunked text snapshot，并记录真实resident bytes；大文档、长history、跨资产effect必须有budget、spill和profiling gate。

### P1-9：source hash使用process-local hasher，不能成为持久revision identity

持久journal、recovery和multi-process conflict使用稳定content hash/schema；进程内快速hash可保留为cache hint，但不能作为跨进程receipt。

### P1-10：watcher refresh和save commit没有统一file identity

统一canonical path、symlink/case规则、file id、mtime/size/content hash与self-write token；保存产生的事件可确认commit，不能误判外部冲突或吞掉真实race。

## 7. P1：Designer Canvas、Hierarchy 与 Palette

### P1-11：缺少删除节点的产品command

实现selection-aware delete，覆盖root保护、slot/cardinality、reference/component边界、binding/style/a11y依赖提示、transaction与undo。

### P1-12：缺少duplicate、cut、copy、paste与跨文档clipboard

设计versioned clipboard payload、ID remap、resource dependency、slot adaptation、paste target chooser和cross-project policy；clipboard操作必须形成单个undo group。

### P1-13：multi-select模型存在但产品没有创建它的交互

补Ctrl/Shift range、marquee、selection anchor、mixed inspector、bulk edit与稳定primary selection；所有tree/canvas视图共享同一selection authority。

### P1-14：canvas没有zoom、pan、fit、ruler、guide、grid与snap

建立camera/viewport state、pixel/DPI-aware ruler、自定义guide、grid和多来源snap；snap决策要可视化并可临时禁用，不能写死像素启发式。

### P1-15：只有ResizeSlot模式，没有完整anchor/pivot/transform手柄

按layout/container schema提供position/size/anchor/pivot/margin/rotation适用工具；container-managed child必须显示受限原因，避免写入无效属性。

### P1-16：缺少align、distribute、match size和批量层级整理

为multi-selection提供typed layout commands、preview ghost和单事务undo；操作依据visual bounds与container规则，不直接拼props。

### P1-17：hierarchy与palette外观含search，但session无query/filter authority

实现可取消的fuzzy/type/tag/favorite/recent filter、结果计数、keyboard navigation和空状态；query只改变projection，不改变document。

### P1-18：drag/drop resolution依赖估算slot语义，缺少schema receipt

把native/component/reference slot定义统一注册到UiSchemaRegistry，drop preview返回accepted target、conversion、cardinality和diagnostic；commit重验证同一revision。

### P1-19：component/reference导航缺少breadcrumb、cycle和跨资产状态模型

导航栈应携带asset/node/revision，显示editable/read-only/dirty/conflict；检测引用cycle，切换资产前保留selection/viewport并参与close/save决策。

### P1-20：缺少可扩展designer tool/provider合同

插件工具必须声明支持的node/slot/schema、pointer capture、overlay、transaction与lifecycle；卸载或异常时释放capture并恢复Select，不能把工具逻辑散落到host action字符串。

## 8. P1：Preview、Device、Localization 与 Input

### P1-21：四个preview preset只是固定像素尺寸

改为project/device profile：logical size、raster scale、DPI、orientation、safe area/cutout、user scale、color space、platform、input method和font fallback policy均可组合与持久化。

### P1-22：preview没有响应式breakpoint和多设备并排矩阵

支持同一document在多个profile/generation下并排、差异高亮和snapshot；breakpoint命中来自runtime layout结果，不由Editor复制条件判断。

### P1-23：locale selector不解析真实本地化文本

当前fallback/en-US/zh-CN主要改变report选择。必须接入同一localization provider、culture fallback、plural/gender/format参数和pseudo-localization，并让文字、换行、fallback font和layout真实重建。

### P1-24：缺少RTL、竖排、长字符串和glyph coverage preview

提供pseudo-RTL、expansion、CJK/Arabic/Indic/emoji corpus与missing glyph overlay；消费Runtime11B typed completeness，不能以估算宽度或系统字体掩盖问题。

### P1-25：Preview Interact只返回binding metadata

`dispatch_preview_interact_at_preview_index`扫描binding生成DTO，未通过真实`UiSurface` hit-test/input/state/action链；产品侧也没有实际caller。必须把pointer/keyboard/gamepad/touch输入注入隔离preview surface，并关联event/state/action trace。

### P1-26：preview没有deterministic time、animation与async state控制

建立preview clock、pause/step/scrub、seed、network/asset pending模拟和state snapshot；重复capture必须可重现，异步依赖显示pending而非随机跳变。

### P1-27：mock value是局部表达式替换，不是typed preview data source

定义schema-backed mock dataset、scenario、validation、secret policy和provider version；mock与runtime snapshot明确区分，缺字段返回typed diagnostic。

### P1-28：preview编译与呈现没有完整generation receipt

compile、imports、locale、device、mock、font、theme和runtime report都进入preview key；旧job完成不得覆盖新结果，last-good要标明来源revision和不完整原因。

### P1-29：缺少输入焦点、导航和多输入设备可视化

提供focus path、navigation edge、pointer capture、gesture/gamepad/key event monitor和可控输入回放；消费Runtime11A实际事件，不在Editor造第二套路由。

### P1-30：preview视觉验收没有golden与像素/几何分层证据

建立多profile/locale/backend golden、layout geometry diff、font/glyph completeness和像素阈值；截图必须记录source/runtime/font/GPU generation，单纯surface非空不能关闭视觉finding。

## 9. P1：Inspector、Binding、Menu Flow 与 Action Schema

### P1-31：Inspector只有少量typed语义，其余退回TOML literal

UiSchemaRegistry应提供property type、default、range、unit、enum、resource type、visibility、container applicability、validation和editor factory；通用TOML仅为高级fallback。

### P1-32：属性编辑缺少default/inherited/overridden状态与Reset

每个字段显示effective source、local override、theme/style/binding影响和validation；Reset/Promote/Extract必须是typed command并可undo。

### P1-33：缺少resource、color、font、icon和localization专用picker

picker消费Asset04 catalog/reference index，支持compatible filter、preview、recent/favorite、broken ref与dependency receipt；禁止自由字符串冒充asset reference。

### P1-34：binding suggestion以control文本中的`save`做硬编码猜测

删除文本启发式，改由Action/Route/Service schema registry按source event、payload type、capability和scope提供候选；不可用endpoint解释原因。

### P1-35：binding payload schema由当前值反推，不是endpoint权威合同

endpoint注册参数名称、类型、required/default、enum、resource/ref、version和validation；Editor根据schema编辑并生成迁移，runtime只接受匹配版本。

### P1-36：binding使用裸字符串，缺少rename/refactor和引用索引

建立stable endpoint identity、symbol/reference index、rename preview、cross-asset transaction和broken reference diagnostic；显示display name与stable id的区别。

### P1-37：binding验证没有运行时capability和上下文资格

验证当前surface、project plugin、platform、authority、network role和sandbox是否提供endpoint；unsupported和temporarily unavailable必须分开。

### P1-38：Menu Flow只有Workbench固定Screen_Start与统计值

建立真实screen/menu graph asset或derived view，节点对应UI asset/state，边对应typed action/route/condition；支持entry、back stack、modal、focus restore、cycle与unreachable诊断。

### P1-39：navigation authoring没有消费运行时focus graph

将explicit/automatic navigation、tab order、spatial candidates与Runtime11A focus snapshot对照；preview可显示边、冲突、trap和device-specific结果。

### P1-40：action trace、binding diagnostic和menu flow没有统一journal

定义bounded typed trace，关联document/preview/runtime generation与source span；Editor11负责查询/导出，禁止把payload或secret无界写入状态栏字符串。

## 10. P1：Theme、Icon、Accessibility 与 Font Atlas

### P1-41：theme token仍是非typed TOML value集合

定义color、dimension、duration、easing、typography、font、icon、brush等token schema，支持alias/cycle、variant、fallback、unit与usage index；unknown custom token保留扩展命名空间。

### P1-42：theme authoring缺少variant和design-system矩阵

支持light/dark/high-contrast/platform/density/state矩阵、继承链和差异预览；compare/merge必须基于stable token/rule identity并进入transaction。

### P1-43：style cascade inspection没有完整source-span和specificity解释

每个effective property展示匹配selector、specificity、order、pseudo state、theme/import来源和source link；stale preview不得显示为当前结果。

### P1-44：style/theme重构未接入全项目usage与跨资产提交

rename/delete token、selector或class前查询Asset04 reference index，预览受影响资产并使用CrossAssetTransaction；部分失败不能只改当前document。

### P1-45：Accessibility Audit是固定9 issues而非真实semantic snapshot

Editor应从Runtime11A获取role/name/value/state/focus/order/bounds/relationships与diagnostics，关联源node和profile；无snapshot时明确Unavailable，不生成样例成功。

### P1-46：a11y authoring没有字段、规则和修复命令

为label、description、role、heading、live region、hidden、focus、relationships、minimum target和reduced motion提供typed editor；自动修复必须预览diff并可undo。

### P1-47：缺少键盘、screen reader和高对比度资格矩阵

按Windows/macOS/Linux和主要input/screen reader建立受控实机证据；自动规则不能替代真实focus announcement、IME和platform bridge测试。

### P1-48：Icon Library固定312/4/14且未消费runtime icon pipeline

建立icon asset/import/cook catalog、vector/raster/SDF variant、theme tint、DPI、license/provenance和usage index；preview消费Runtime11C真实atlas/renderer结果。

### P1-49：runtime icon atlas没有产品consumer，当前icon还可能画成矩形

Editor不得在独立预览中掩盖11C缺口。Icon Library要显示resolved glyph/texture、atlas page、UV、generation、fallback和render completeness；runtime未支持时显示Blocked。

### P1-50：Font Atlas固定4096 glyph/4 pages/12 missing且与真实font服务断开

连接Runtime11B cooked font/fallback/shaping/glyph artifact与11C GPU residency，显示face/variation/script/locale、glyph source、page/UV、pending/evicted/missing和budget；没有artifact-owned bytes时不能宣称package-ready。

## 11. P1：Build、Cook、性能、扩展与Authority收敛

### P1-51：`ui_document_importer`只解析原始V2，未形成closed cooked artifact

定义compiled UI artifact：schema/compiler version、dependency hashes、resolved component/style/font/icon refs、platform/profile variants、diagnostics与completeness；runtime package不重新读取Editor source。

### P1-52：UI compiler没有Editor可消费的structured build receipt

输出source ranges、severity/code、dependency generation、artifact id、completeness和timing；last-good与current failure同时可见，不能只显示字符串summary。

### P1-53：create/import/save/compile之间没有单一asset generation链

factory、repository、asset manager、compiler、preview和runtime registry使用同一generation protocol；每个阶段只提交匹配base revision的结果。

### P1-54：大UI资产没有authoring性能预算与规模门禁

建立1k/10k/100k node、深层component、巨量rules/bindings/imports与多preview profile基线；测typing latency、tree virtualization、selection、undo、compile、memory和save。

### P1-55：hierarchy、palette、inspector和diagnostic列表缺少统一虚拟化证据

所有大列表采用stable keyed virtualization、incremental filter和bounded projection；焦点、selection和scroll在更新后保持，不能靠全量node rebuild通过小fixture。

### P1-56：插件贡献缺少schema/version/capability隔离

widget、property editor、palette entry、designer tool、binding endpoint和validator均声明version、owner、capability、thread/lifecycle与failure policy；卸载产生typed orphan而非panic或静默丢字段。

### P1-57：UI authoring与runtime text/icon/a11y存在重复authority风险

Editor只消费Runtime11A/11B/11C provider和artifact，不建立私有font database、glyph atlas、focus graph或a11y evaluator；必要的authoring-only规则显式标记来源。

### P1-58：真实UI Asset Editor与Workbench UI Asset Editor形成重复产品入口

Workbench extension必须打开/嵌入同一`editor.ui_asset` session或变成明确的demo fixture；不能继续用`WBP_Inventory`、42 widgets和3 issues制造第二authority。

### P1-59：HUD、Binding、Icon、A11y、Menu、Font surfaces都写固定业务状态

MatchTimer、Ammo、DPI 1.00、Health.Value、icon-warning、Gameplay_HUD、Screen_Start、Inter UI和固定计数必须删除或移到测试fixture；生产surface只投影provider状态与Unavailable原因。

### P1-60：Workbench action/field handler只改control字符串却报告成功

字段Change/Submit必须生成validated document command，action必须调用真实factory/compile/preview/save/audit；只有accepted revision回写presentation。无provider时disable并解释，禁止固定“Validated/Applied/Saved”。

## 12. P2：完整性、诊断与维护性

### P2-1：UI Asset核心出现多份接近千行的高耦合文件

按document repository、schema、designer command、theme service、presentation projection拆 owner；避免继续把新功能堆入`style_state`、`undo_stack`、`lifecycle`和drop resolution。

### P2-2：action id、control id和endpoint id散落为裸字符串

生成/集中typed IDs与schema校验，保留稳定序列化名称；编译期或启动时检测重复、悬空route与资源缺失。

### P2-3：preview/profile/locale和budget常量缺少project policy

区分安全硬上限、engine default、project setting与user preference；输出effective value和来源，避免把1920x1080等样例固化为产品语义。

### P2-4：diagnostic code、severity和source mapping不统一

收敛到Editor11 journal schema，支持source/node/property/binding/asset/GPU关联、去重和cardinality budget；状态栏只显示摘要。

### P2-5：视觉artifact测试有ignored项且元数据不足

关键golden改为受控非ignored lane，记录OS、backend、DPI、font set、locale、profile、source/runtime generation和阈值；手工PNG不能单独关闭finding。

### P2-6：测试偏重结构存在和presentation字符串

增加property/fuzz roundtrip、failure injection、multi-process conflict、real input dispatch、screen reader bridge、clean package和GPU/font completeness测试。

### P2-7：unsupported状态常以bool/Option表达，丢失原因

统一ready/pending/stale/unavailable/unsupported/error/budget outcomes，携带owner、generation和recovery action；presentation据此控制命令可用性。

### P2-8：source outline、projection和preview cache缺少统一memory accounting

记录entry、bytes、generation、hit/miss/evict/build time与peak；共享对象避免重复计费，跨CPU/GPU资源按owner关联。

### P2-9：plugin maturity/capability状态不能由静态manifest自证

用qualification gate生成状态；资源URI、factory、schema、package、platform和failure tests未通过时只能是experimental/partial/unavailable。

### P2-10：缺少authoring telemetry的privacy与内容边界

只记录低基数性能/结果码和hash，禁止上传source text、binding payload、localized content、secret和用户路径；debug capture需显式授权与脱敏。

### P2-11：文档与current source之间没有自动漂移检查

把本篇scope fingerprint、route/resource inventory和P0 contract assertion纳入review tooling；源码变化后标记stale并要求重新取证，不能沿用旧“完成”结论。

### P2-12：术语混用Layout/Widget/Style/ThemeTokens/HUD/Surface

建立schema glossary和owner map，区分source asset kind、compiled artifact、runtime surface、screen/menu flow、theme token与Workbench view，减少迁移和API误解。

## 13. Workbench静态第二Authority清单

| Surface | 当前固定业务事实 | 必须替换的数据源 |
|---|---|---|
| HUD workspace | MatchTimer、WeaponPanel、Gameplay HUD Canvas、240x180 Minimap、Ammo_Clip、DPI 1.00、en-US | 真实UI document + preview profile + localization + binding/action trace |
| UI Asset Editor extension | `WBP_Inventory`、42 widgets、3 issues | 同一`editor.ui_asset` session与document revision |
| UI Binding extension | `Health.Value`、18 bindings、2 invalid | UiSchemaRegistry + reference index + runtime diagnostic |
| Icon Library extension | `icon-warning`、312 icons、4 missing、14 refs | icon asset/cook catalog + usage index + Runtime11C atlas |
| Accessibility Audit extension | `Gameplay_HUD`、9 issues、Contrast AmmoText、focus InventoryGrid | Runtime11A semantic/focus snapshot + authoring rules |
| Menu Flow extension | `Screen_Start`、64 focus rules、2 issues | typed screen/menu graph + runtime focus/action route |
| Font Atlas extension | `Inter UI`、4096 glyphs、4 pages、12 missing | Runtime11B font/glyph completeness + Runtime11C GPU residency |

这些surface可以保留布局与稳定control identity，但必须删除生产路径中的fixture值。测试需要样例时，把fixture放到test-only provider并在UI显著标明sample；生产provider缺失时显示Unavailable，而不是沿用样例值。

## 14. 分层重构里程碑

### M0：冻结无损合同与当前证据

重算349文件fingerprint；建立V2 golden corpus、unknown-field/trivia/repeat/slots/ThemeTokens roundtrip和现有资产备份工具。暂时阻止不受支持V2语义的视觉保存，撤销死create入口和静态成功反馈。

### M1：Document Repository与原子Save

实现lossless CST + typed V2 model、document/source revision、atomic write、reimport receipt、dirty推进和crash recovery；open/autosave/save copy统一协议。

### M2：并发冲突与Cross-Asset Transaction

实现multi-instance共享owner或revision广播、CAS save、three-way merge、force权限和journal；promote/refactor/undo/redo使用read/write set、staging、commit/rollback与故障注入。

### M3：Factory、Catalog、Migration与Cook

确定builtin/plugin owner，补齐真实resource与operation factory；创建versioned V2资产并导入打开。建立V1/V2 migration和closed compiled UI artifact，clean package不读取Editor source。

### M4：Schema-driven Designer

完成widget/property/slot registry、typed inspector、delete/clipboard/multi-select、zoom/pan/ruler/guide/snap、anchor/pivot/transform、align/distribute及virtualized hierarchy/palette。

### M5：真实Preview与输入

接入device/DPI/safe-zone/orientation/locale/RTL/font/input profile、deterministic clock和scenario mock；Preview Interact通过真实surface input/state/action链并输出generation-qualified trace。

### M6：Binding、Menu Flow与导航

建立action/route/service payload schema、stable endpoint identity、reference/refactor index和capability验证；实现真实screen/menu graph并对照runtime focus/navigation snapshot。

### M7：Theme、Icon、Accessibility与Font产品化

完成typed design tokens/variants/cascade explanation、cross-asset theme refactor；Icon/A11y/Font surfaces分别消费Asset04、Runtime11A、11B、11C的真实artifact/snapshot，不再显示fixture统计。

### M8：规模、平台与故障资格

运行大文档、深import、多profile、多locale、多实例、崩溃恢复、screen reader、IME、font fallback、GPU atlas/device loss和clean package门禁；所有artifact记录完整环境与generation。

### M9：Authority硬收敛

Workbench所有UI extension改为同一session/provider投影或test-only demo；删除legacy lossy writer、裸字符串create、直接`fs::write/remove`副作用、固定业务feedback和重复font/icon/a11y authority。

## 15. 验收门禁

1. **V2 repeat**：含nested repeat的资产经过所有designer mutation、undo/redo和save后语义与source span保持，runtime compile结果等价。
2. **V2 slots**：node-level named slots、component mounts和slot props roundtrip无丢失，drop/rename有typed migration。
3. **ThemeTokens**：asset kind、tokens、unknown variants和imports在视觉编辑后保持，不降级成Style。
4. **Unknown/trivia**：comments、unknown fields、表顺序与未触及区域字节稳定；Format是独立显式命令。
5. **Save failure**：write/flush/rename/reimport每一步故障注入后session仍dirty、磁盘仍是旧完整版本，diagnostic可重试。
6. **Crash save**：任意commit阶段终止进程，重启只能得到旧版本或新版本，不得出现截断/混合文件；recovery解释temp来源。
7. **Multi-instance**：两个Editor实例从同一base编辑，第二次Save必须进入merge/conflict，普通Ctrl+S不能覆盖首个commit。
8. **External edit**：外部工具在Save前修改文件，CAS拒绝并提供reload/merge/copy/force；force有显式权限和journal。
9. **Cross-asset promote**：在第N个write/import失败时当前document、目标asset、registry和undo stack全部rollback。
10. **Undo/redo**：跨资产transaction在restart/recovery后仍可验证receipt；重复undo/redo不丢文件、不复用stale revision。
11. **Factory**：default安装可从菜单创建Layout/Widget/Style/ThemeTokens，文件可解析、导入、打开、保存、重启再开。
12. **Plugin resources**：所有descriptor URI在package中存在，启动扫描无悬空surface/template/operation；缺失会阻断qualification。
13. **Migration**：V1到V2提供dry-run diff、backup、diagnostic与幂等测试；unsupported future version只能read-only，不被降级保存。
14. **Designer basics**：delete/duplicate/cut/copy/paste/multi-select/align/distribute/zoom/pan/fit均有真实command、single undo group和selection保持。
15. **Layout tools**：anchor/pivot/container/slot编辑遵守schema；不适用操作禁用并解释，golden geometry与runtime一致。
16. **Hierarchy scale**：100k node资产的filter/scroll/selection虚拟化满足预算，内存与build counter有machine-readable证据。
17. **Device matrix**：至少覆盖desktop/mobile/console、portrait/landscape、DPI/raster scale、safe zone/cutout和user scaling，layout来自真实surface。
18. **Localization**：en/CJK/Arabic/Indic/pseudo/RTL实际替换文字并触发shaping/layout；missing key/glyph显示typed diagnostic。
19. **Preview input**：pointer/keyboard/gamepad/touch经真实hit-test/focus/state/action链，trace关联source、binding和generation。
20. **Preview determinism**：固定clock/seed/mock/profile的capture可复现；旧async结果不能覆盖较新preview generation。
21. **Inspector schema**：所有builtin widget/slot属性有type/default/range/unit/applicability和Reset；unknown plugin字段无损保留。
22. **Binding schema**：payload按endpoint version验证，rename跨资产更新引用；硬编码`save`文本启发式为零。
23. **Menu flow**：entry/back/modal/focus restore/unreachable/cycle在真实graph与runtime trace中一致，非固定Screen_Start fixture。
24. **Theme**：variant、alias/cycle、cascade/specificity和cross-asset token rename通过transaction与多profile视觉对照。
25. **Accessibility**：semantic tree、focus order、name/role/state/bounds与source node可追踪；键盘和至少一条受控screen reader链通过。
26. **Icon**：icon asset经import/cook/atlas/render形成闭环，多DPI/theme下不是rectangle fallback；usage/missing统计来自真实index。
27. **Font**：删除source字体并禁用系统fallback后，package仍显示声明语言；Font Atlas页/UV/residency与Runtime11B/11C一致。
28. **Cook**：compiled UI artifact拥有全部component/style/font/icon依赖和version hash；runtime package不读取`.zui` authoring source。
29. **Performance**：1k/10k/100k node、深import、大rules/bindings下typing、selection、compile、save、memory有阈值和回归趋势。
30. **Failure outcomes**：pending/stale/unavailable/unsupported/error/budget在UI可区分；无provider时命令disabled，不显示固定成功。
31. **Workbench convergence**：七份UI Workbench surface只投影真实provider或test-only sample；生产源码中固定资产名/计数归零。
32. **Current-source evidence**：关键门禁非ignored，artifact记录source fingerprint、OS/backend/DPI/font/locale/profile/generation和阈值；旧截图、结构存在或字符串反馈不能关闭finding。

## 16. 禁止的临时修补

- 不得继续给legacy投影补零散字段后宣称V2无损；必须建立单一typed model与unknown-field保留合同。
- 不得把`mark_saved`移动几行就算原子Save；磁盘replace、revision CAS、reimport receipt和crash recovery必须一体完成。
- 不得在冲突时默认“本地优先”或把普通Save重命名为force；用户决策与base revision必须显式。
- 不得用更多`fs::write`和inverse closure修补promote/undo；跨资产副作用必须由transaction coordinator提交。
- 不得只补四份空ZUI文件；create operation、catalog、factory、asset identity、import与open必须端到端可执行。
- 不得新建Editor私有font atlas、icon atlas、focus graph或a11y tree来让面板有数据；必须消费Runtime11A/11B/11C authority。
- 不得用截图、固定计数或`Space`占位模拟HUD、Binding、Icon、A11y、Menu Flow和Font Atlas完成。
- 不得只增加按钮和快捷键而没有typed command、selection、transaction、undo和failure contract。
- 不得以Bevy缺少可视化UI editor为由降低目标；本篇目标是Unreal/Godot/Fyrox同级工程authoring闭环。
- 不得宣称“表现或性能优于Unreal”；在统一内容、硬件、backend、profile和统计方法的对照门禁通过前，只能报告Zircon自身可复现结果。

## 17. 本轮产出边界

本文是current-source静态review与重构计划，不包含生产代码修改，也不把任何finding标记为implemented。349文件fingerprint覆盖本轮选择范围而非整个仓库；40个在途文件要求实施前重新取证。Runtime UI tree/layout/input/accessibility由11A负责，font/text/shaping/IME由11B负责，GPU atlas/batch/clip/submit由11C负责；Editor23只消费这些contract并建立authoring产品。下一轮Editor审查应继续覆盖尚未纵向深审的Editor产品域，而不是立即在本篇实现零散按钮。
