---
title: Editor Scene Contextual Action、Context Menu、Target/Selection Snapshot、Availability、Command Routing、Extension、Transaction、Accessibility、Performance 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor71
review_date: 2026-08-22
baseline_head: a922089697e41e07fa29e3e42a5e4c9afc1ae31b
baseline_epoch: 341
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes/context_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu/classifier.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu/path.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu/provider.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu/request.rs
  - zircon_editor/src/ui/retained_host/app/workbench_context_menu.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/popup_primitives.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/context_menu.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/popup_state.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_menu_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/workbench/secondary.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_hierarchy_projection.rs
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/core/commands/descriptor.rs
  - zircon_editor/src/core/commands/when.rs
  - zircon_editor/src/core/editor_extension/contribution_descriptors.rs
  - zircon_editor/src/core/extension/store/model/snapshot.rs
  - zircon_editor/src/ui/host/command_eval_projection.rs
  - zircon_editor/src/scene/viewport/interaction/viewport_input.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_input.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport/pointer_dispatch.rs
tests:
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu_tests.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/context_menu.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_reference/dropdown_pointer.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/pointer_bridge.rs
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/commands/descriptor_when.rs
  - zircon_editor/src/tests/commands/when.rs
  - zircon_editor/src/core/extension/store/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/55-editor-structured-clipboard-cut-copy-paste-duplicate-delete-cross-document-remap-drag-payload-product-integration-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/60-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/61-editor-scene-document-authoring-world-open-new-reload-save-close-dirty-transition-autosave-recovery-multi-document-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/66-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/68-editor-scene-viewport-display-mode-lighting-skybox-show-flag-debug-visualization-overlay-composition-profile-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/70-editor-scene-viewport-object-visibility-temporary-hide-isolate-local-view-selection-eligibility-hierarchy-feedback-persistence-performance-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/LevelEditorContextMenu.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Public/LevelEditorContextMenu.h
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/ViewportInteractions/LevelViewportClickSelection.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/SLevelViewport.cpp
  - dev/godot/editor/docks/scene_tree_dock.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.cpp
  - dev/Fyrox/editor/src/world/menu.rs
  - dev/Fyrox/editor/src/world/mod.rs
  - dev/Fyrox/editor/src/lib.rs
  - dev/bevy/crates/bevy_ui_widgets/src/menu.rs
  - dev/bevy/examples/usage/context_menu.rs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/GraphView/Views/VFXView.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Drawing/Views/MaterialGraphView.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Drawing/Blackboard/SGBlackboardCategory.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/RenderGraph/RenderGraphViewer.SidePanel.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/ContextualMenuDispatcher.cs
doc_type: current_source_review
canonical_owner: docs/plans/optimize/zircon_editor/71-editor-scene-contextual-action-context-menu-target-selection-snapshot-availability-command-routing-extension-transaction-accessibility-performance-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Scene Contextual Action、Context Menu、Target/Selection Snapshot、Availability、Command Routing、Extension、Transaction、Accessibility、Performance 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon已经有一套可保留的通用popup视觉与交互底座。Retained Host可以从secondary pointer hit发出右键请求，`WorkbenchContextMenu`支持定位、焦点、键盘移动、typeahead、disabled/separator/checked/icon/danger等基础投影；通用Command Registry已有stable command id、`WhenClause`、selection count/domain/revision与scene mode revision；Extension Store也有generation、capability过滤与owner ticket。这些基础足以承载工程级场景上下文动作，但目前彼此没有组成一条领域正确的产品链。

当前SceneNode provider只按control/action字符串前缀猜测目标，把显示文本清洗成`workbench://scene/<label>`，随后返回固定的Open、Rename、Duplicate、Delete字符串。请求不携带document/world/view identity、实体或subobject handle、selection snapshot/revision、pointer ray/world position、command/extension generation、read-only/lock状态或会话token。点击这些菜单项只会更新popup的`value`并关闭；scene action不进入Command Registry、编辑命令、事务、Undo/Redo或世界修改。Host还会显示“Context menu opened”，因此这是当前可达的P1能力真实性问题，而不是已经可达的数据破坏P0。

Scene Viewport的右键路径更不完整。Secondary down被直接映射为`RightPressed`，controller当场创建`ViewportDragSession::Orbit`；没有移动阈值、短点击分支、当前picking target、selection policy或context request。Outliner与Viewport因而没有共享动作集：前者显示无效菜单，后者根本打不开场景对象菜单。工程级实现必须把“目标解析、选择冻结、动作聚合、可用性评估、UI投影、执行重校验、事务回执”建成Editor Scene领域服务，而不是继续向raw string provider追加条件分支。

Runtime不应拥有context menu。`zircon_editor::scene`应捕获generation-qualified immutable `SceneContextTargetSnapshot`，通过owner-aware `SceneContextActionProviderRegistry`编译`SceneContextActionCatalog`，为每次打开建立不可变`SceneContextMenuSession`，再把动作调用映射到Editor08的canonical command、Editor55/60/63的编辑语义和事务。Runtime只提供中性的entity/hierarchy/component/selection-eligibility事实，不保存编辑器菜单、焦点、插件slot或UI文本。

本报告新增 **46项P1、10项P2**，登记 **48个全部Fail的资格门**。没有新增P0；Editor55的删除数据完整性、Editor59的输入/capture、Editor60的跨World hierarchy、Editor61的document lifecycle、Editor63的transaction scope等父账仍按各自报告计数。当前不能声称Scene contextual action在功能、性能、可扩展性、无障碍或表现上达到Unreal级别。

本轮是review-only：未修改production Rust，未运行Cargo、真实Editor、GUI/GPU、native input、save/reopen、plugin reload、fault/soak/profile或同语义跨引擎benchmark；tooling按用户要求排除。

## 2. 审查边界、currentness与冻结语料

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test declarations | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Zircon context-menu product route | **21 / 1,889 / 1,754 / 81,137 / 4** | request、classifier、provider、popup projection、dispatch、hierarchy projection与ZUI | `cfec54540684cd1a89bb6ef9e1f70bd686c610bb20c09537e2e9c012f5321efc` |
| Zircon command/extension/scene-input seams | **15 / 3,917 / 3,588 / 137,854 / 21** | command admission、extension snapshot、editing route、viewport secondary input与selection projection | `8a6e7664af755e91416f01712b5927b40289c1fa3ccc39eb3fb5fd0ede33550c` |
| Zircon focused tests | **8 / 2,556 / 2,311 / 87,750 / 60** | request/render、native pointer、viewport mapping、command predicate与extension lifecycle | `d6b6260ceb9b26380097e0ae183787019c911d420cbdc204f15b119c5439de2f` |
| Zircon deduplicated focused set | **43 / 8,273 / 7,579 / 303,690 / 85** | 三组按规范化路径去重 | `905d9d9563d2d28ac0248814b991663f698ba46afae413764e789ac6f678f99e` |
| Unreal selected set | **4 / 7,956 / 6,786 / 291,423 / 0** | typed context、selection/hit/cursor world、ToolMenus、extender与viewport interaction | `718f0451f011d67fc1ebef69489105cb8b9ae3a4ea919bdc25ffda69337545d6` |
| Godot selected set | **2 / 12,586 / 10,697 / 477,240 / 0** | SceneTree动态能力、plugin slot、node path metadata与viewport candidate disambiguation | `3a83abec2edd921f55f46adce553f0547b79e9a9b938a9eaaf4ce4e79963305f` |
| Fyrox selected set | **3 / 4,672 / 4,236 / 185,097 / 0** | stable UUID、current selection/clipboard/resource admission与CommandGroup | `7a57c94c63bf6a6045357bbf7d30adcaa30b5e788adf745585275cf48f1c9fb2` |
| Bevy selected set | **2 / 806 / 737 / 29,207 / 3** | popup focus、keyboard、disabled、activation/close与示例边界 | `3b3f2720efad9e7a0edaec52961beb01a12c028b6dd3e9dd0fe1063f27ef6cb3` |
| Unity Graphics selected set | **5 / 6,507 / 5,571 / 271,920 / 0** | graph target-aware menu、selection replacement、dynamic status、read-only降级与Undo | `458b09349a14c78e3da4588150a01cfb225f251a97bd84dc83480673834a0dd6` |
| Five-engine deduplicated set | **16 / 32,527 / 28,027 / 1,254,887 / 3** | 五类本地参考按路径去重 | `30c161cd061e571740733966106410edac1c7ef9214dd3b0cedab7134880dc50` |

指标是本轮工作树字节与文本物理统计，不是功能覆盖率。fingerprint按排序后的`path + file SHA-256`清单计算；后续实现前必须重取，因为相关Editor源码与索引仍由其他会话并行演进。

### 2.2 在途修改隔离

审查期间`docs/plans/optimize`、Runtime UI与若干Editor failure handoff存在其他会话修改；本报告没有把这些改动回退、归因或吸收入finding。选取的43个Zircon文件在冻结统计时没有本轮production改动。报告只描述current source可观察行为，任何后续改动必须重新核对request、popup、command、selection、extension和viewport input终态。

### 2.3 范围与非范围

本报告拥有Scene领域的context target/selection snapshot、contextual action provider/aggregation/admission、per-open menu session、Viewport/Outliner投影、调用重校验与receipt。通用Command Registry/keymap/menu/palette归Editor08；通用extension lifecycle归Editor50；cut/copy/paste/duplicate/delete语义归Editor55；viewport picking/selection归Editor59；Outliner树与hierarchy mutation归Editor60；document/world lifecycle归Editor61；transaction/history归Editor63；camera/frame/pilot归Editor66；show flag/object visibility归Editor68/70。本报告只要求这些owner暴露可组合合同，不重复拥有其内部实现。

## 3. 当前实现拓扑与可保留基础

### 3.1 通用popup控件有真实交互基础

`WorkbenchContextMenu`可以在物理坐标转换后打开，维护open/focused/selected状态，投影separator、disabled、checked、danger和icon，并由既有keyboard/menu pointer层处理焦点、激活与关闭。该视觉/输入层应保留，但必须只消费typed menu session projection，不能继续承担领域动作身份与目标权威。

### 3.2 Scene provider是字符串分类器和固定数组

`context_menu_provider_for_hit()`依据`WorkbenchScene*Item`、`workbench.hierarchy.*`等字符串前缀判断SceneNode。`target_path()`从`value_text`或control id清洗路径，`menu_items()`固定返回五个raw strings。这里没有Scene、World或Entity查询，也没有selection、clipboard、lock、read-only、root、inherited、plugin contribution或command admission。

### 3.3 请求只是一份UI hit副本

`WorkbenchContextMenuRequestData`只有control/action/dispatch/role/value/path、popup坐标和raw item list。它无法证明点击对象属于哪个document/world/view，也无法在菜单保持打开期间判断selection、entity、provider或command generation是否变化。

### 3.4 打开和点击之间没有领域会话

Bridge把target和值复制进ZUI属性。关闭时仅清空`open/menu_items`，不会清除`context_target/path/value`。`select_popup_menu_item()`按解析出的action id更新label、关闭popup并刷新surface；其后只有asset creation、main menu、run mode、layout和module overflow等分支被消费，没有Scene action handler。

### 3.5 通用command和extension底座尚未接入

默认Command Registry已有`scene.node.delete_selected`及`SelectionNonEmpty`，`CommandEvalCtx`也携带selection count/domain/revision；但context Delete不引用该stable id。Extension Store有generation与capability-filtered `EditorMenuItemDescriptor`，其模型却只表达全局menu path/operation/priority/shortcut/enabled，不表达Scene surface、target schema、selection predicate、slot、dynamic admission或per-open currentness。

### 3.6 Outliner已有entity映射，却未参与右键请求

`SceneHierarchyProjectionState`维护generation、selection_revision、entity/control双向映射和selected set。这是构造qualified target snapshot的可保留输入，但generic secondary pointer path直接从template hit生成请求，不查询这份状态，也不先确定“右击未选中行”应替换还是保留selection。

### 3.7 Viewport secondary input被硬编码为Orbit

`UiPointerButton::Secondary`始终映射`EditorViewportEvent::RightPressed`；controller一收到就创建Orbit drag，release直接结束。Pointer move已有current interaction extract和picking route，primary selection也有阈值，但secondary没有点击/拖拽识别、target snapshot或context action分支。

### 3.8 测试证明了视觉请求，没有证明业务结果

现有测试验证request分类、固定menu字符串、popup打开/关闭和secondary映射；没有任何测试从Scene右键动作执行到canonical command、transaction、world delta与undo，也没有stale target、right-click selection、multi-selection、plugin revoke、document replacement、keyboard context key或规模测试。

## 4. 五引擎参考证据与适用边界

### 4.1 Unreal：typed context、ToolMenus与extender组成领域权威

`FLevelEditorContextMenu::InitMenuContext()`把selection、context type、typed hit proxy element/actor、selected components、cursor world location与Level Editor owner装入上下文；`GetContextMenuName()`按component/actor/element/empty/Scene Outliner选择菜单，Actor menu再根据单选、多选、bulk、actor type、viewport state动态构建section。命令列表和viewport menu extender被合并，Scene Outliner复用同一领域动作图。可借鉴的是typed context、section/command/extender和surface复用，不应照搬其legacy global selection/static state。

### 4.2 Godot：每次打开重建节点能力，并给插件传真实路径

`SceneTreeDock::_tree_rmb()`捕获top-level及完整selected node列表，按profile权限、ownership、inherited/instance、root、clipboard、script、单/多选动态添加或禁用rename/replace/cut/copy/paste/duplicate/reparent/delete等动作，并把真实node paths交给plugin context slot。3D viewport保留右键导航，以Alt-right调用候选列表并给每项保存node path metadata、类型、locked/grouped提示。Zircon可以选择“右键短击菜单、拖拽导航”或显式替代手势，但必须形成一致且可发现的产品政策。

### 4.3 Fyrox：stable UUID与CommandGroup是真实底线，但current selection仍不够

`SceneNodeContextMenu`为动作定义UUID，打开时根据clipboard与selected resource更新enabled，点击后通过`MessageSender`提交`AddNodeCommand`、`ReplaceNodeCommand`、`PasteCommand`、delete command或`CommandGroup`，删除外部引用前还可进入确认流程。它证明菜单不能只改UI字符串；同时其handler在点击时直接读当前selection、provider trait只返回menu handle，缺少不可变snapshot与generation revalidation，因此不能作为Zircon的最终架构上限。

### 4.4 Bevy：只作为控件焦点与无障碍参考

`bevy_ui_widgets::menu`为MenuPopup/MenuItem声明accessibility role、modal tab group、focus state、Escape/方向/Home/End/Enter/Space语义、disabled阻止激活和focus-loss close；context-menu example只是spawn/despawn颜色选项。Bevy本地树没有Scene Editor/Outliner contextual action authority，因此不应用该示例的临时ECS实体菜单替代Editor领域session。

### 4.5 Unity Graphics：本地包仅证明graph-domain动态菜单

本地Graphics仓不含Unity Editor核心SceneView/Hierarchy源码，不能据此宣称完成Unity场景菜单对照。可用证据来自VFX/ShaderGraph：根据`evt.target`类型追加动作，右击未选中Node先替换selection，status callback动态返回Normal/Disabled，不可编辑asset把菜单降级为Copy，修改走完整Undo；VFX还按鼠标位置和system bounds添加Rename等动作。这些原则可迁移到Scene contextual action，但目标identity和事务仍必须由Zircon owner定义。

## 5. 差异矩阵

| 维度 | 当前Zircon | 工程级目标 | 优先级 |
|---|---|---|---|
| Target identity | control id、action id、display-derived path | document/world/view-qualified entity/subobject handle | P1 |
| Selection | 不捕获，右击不改变selection | 明确surface policy与immutable selection snapshot | P1 |
| Menu model | raw strings与固定数组 | typed sections/actions/status/arguments/provenance | P1 |
| Availability | Scene动作始终显示 | capability、lock、read-only、root、multi-select与disabled reason | P1 |
| Routing | 点击只改popup value | canonical command/operation + two-phase revalidation | P1 |
| Transaction | 无 | owner-scoped transaction、undo/redo与typed receipt | P1 |
| Viewport | secondary无条件Orbit | click/drag arbitration或明确替代手势 | P1 |
| Cross-surface | Outliner和Viewport分叉 | 同一action graph，不同surface facts/policy | P1 |
| Extension | 仅全局menu path | owner-aware scene context provider、slot与revoke | P1 |
| Lifecycle | 无menu session/currentness | generation-qualified open/invoke/retire state machine | P1 |
| Accessibility | 通用menu基础存在，领域语义缺失 | keyboard context open、focus return、disabled explanation | P1 |
| Performance | 每次字符串重建，无预算 | compiled catalog、bounded aggregation、incremental admission | P1 |

## 6. 新增发现

### 6.1 P0：本轮无新增项

Rename、Duplicate、Delete当前没有Scene mutation consumer，因此未观察到由context menu直接造成的数据丢失、跨World误改或不可撤销修改。UI展示可用动作并在点击后关闭仍是严重P1真实性缺陷。未来一旦把现有display-derived path或current selection直接接到删除命令，风险会升级；实现必须先完成identity/session/revalidation，而不是先接handler。

### 6.2 P1：目标、会话与currentness

#### ED71-P1-01：没有Scene Contextual Action authority

当前authority分散在template hit、fixed provider、popup属性和若干switch分支。应建立`SceneContextActionService`作为capture、aggregate、project、invoke、retire的唯一Editor Scene owner。

#### ED71-P1-02：请求envelope没有qualified scene identity

请求不含project/document/world/view/session generation。两个World里相同control/value或World替换后旧popup都无法区分；必须使用Editor61/58提供的qualified identity。

#### ED71-P1-03：target path由显示文本派生

duplicate names、localized labels、标点、大小写和非ASCII会碰撞或丢失；测试中的fallback甚至生成`workbenchscenepropsitem`。Display text只能用于呈现，不能作为对象address或action argument。

#### ED71-P1-04：Scene provider依赖control/action前缀猜测

模板重命名、插件control、虚拟行复用或错误action prefix都会改变领域分类。Surface必须显式发布typed context target capability，而不是让Host从命名约定反推。

#### ED71-P1-05：已有hierarchy generation/entity映射未进入请求

Outliner已经知道entity、generation、selection revision，但secondary path绕开它。Context snapshot必须由当前projection/current world校验后构造，未知或stale control应拒绝打开。

#### ED71-P1-06：没有immutable selection snapshot和revision fence

菜单打开后selection可通过键盘、脚本、另一viewport或异步刷新变化。需要冻结ordered qualified targets、primary/anchor、selection revision，并在invoke时按动作policy重校验或要求刷新。

#### ED71-P1-07：右击未选中对象的selection policy未定义

当前菜单针对hit label显示，却不会改变selected set；未来若动作读current selection会对错误对象执行。每种surface必须明确replace、preserve-if-member、extend或target-only规则，并将结果纳入同一输入事务。

#### ED71-P1-08：没有entity/subobject target resolution与retirement

Entity在菜单打开期间可能delete、replace、unload、reparent或generation复用。调用前必须解析qualified handle并返回`StaleRefreshRequired`，不能回退到同名对象或当前selection。

#### ED71-P1-09：没有重叠目标与subobject disambiguation

Viewport可能命中actor、component、gizmo、socket、surface或多个深度候选。应复用Editor59的resolved hit list，并允许typed target picker，而不是用第一个display label决定全部动作。

#### ED71-P1-10：popup关闭不清理context target/value

`close_context_menu_if_target()`清空items但保留target/path/value，调试、热重载或错误分支可观察旧上下文。Menu session retire必须原子清除UI projection和领域session，且焦点返回来源surface。

### 6.3 P1：动作模型、聚合与可用性

#### ED71-P1-11：Scene菜单是固定五项数组

对象类型、selection、surface、world mode和插件不会改变动作集。应从stable built-in definitions与provider contributions编译，而非继续扩大`match SceneNode`。

#### ED71-P1-12：raw string item schema不能承载工程合同

`Label|danger,icon=trash`无法类型化表达section、submenu、command id、argument schema、owner、visibility、disabled reason、tooltip、localization key或telemetry policy。UI层必须消费typed projection DTO。

#### ED71-P1-13：存在两套popup item解析与action identity规则

`popup_primitives`能解析explicit `action=...`，`pane_menu_projection`却按label重新生成action id。相同menu item在不同路径可能获得不同身份；必须只保留一个compiled item parser/identity authority。

#### ED71-P1-14：pane投影丢弃显式action id

即便provider提供stable action，projection仍可能把它降回label-derived id。必须通过opaque `SceneContextActionHandle`投影，显示label变化不得改变调用目标。

#### ED71-P1-15：没有section、slot、anchor和submenu聚合合同

无法确定built-in与plugin动作排序、分隔、覆盖和冲突。Registry必须定义stable slot/section、before/after anchor、priority和deterministic tie-break，拒绝未知anchor或循环依赖。

#### ED71-P1-16：没有按对象能力动态评估动作

当前所有SceneNode都有Open/Rename/Duplicate/Delete。Admission至少应消费object kind、component capabilities、selection cardinality、hierarchy role、world mode、tool state和command `WhenClause`。

#### ED71-P1-17：disabled reason没有领域投影

简单disabled flag不能解释“只读资产”“根节点不可删除”“selection含锁定对象”或“provider已卸载”。需要stable reason code、localized text和可选remediation command。

#### ED71-P1-18：单选、多选、混合选择与bulk规则缺失

Rename通常要求单目标，Delete/Duplicate可批量，mixed types可能只保留交集或提供type-specific subgroup。需要显式`SelectionApplicability`与all/any/primary semantics，不能只看count非零。

#### ED71-P1-19：read-only、locked、inherited、root与ownership限制缺失

Context provider不查询document write authority、multi-user lock、prefab/inherited ownership、root protection或external reference。动作必须在打开与执行阶段都验证这些事实。

#### ED71-P1-20：clipboard与destination能力没有接入

当前Scene菜单甚至没有Cut/Copy/Paste；Duplicate也不知道factory/remap能力。Editor55的structured payload/destination plan必须成为admission输入，Context层不重写复制语义。

#### ED71-P1-21：empty surface与cursor world context没有模型

空白Outliner/Viewport应能提供Create、Paste Here、Select All或Play From Here等动作；当前request必须命中control。Snapshot需要screen point、ray、world position/normal和surface-specific empty target。

### 6.4 P1：命令、事务与执行回执

#### ED71-P1-22：Scene menu item没有业务dispatch consumer

点击后只更新value、关闭和重绘。必须把opaque action handle送回`SceneContextActionService::invoke`，不得在ZUI或popup bridge里直接修改Scene。

#### ED71-P1-23：UI形成可用但no-op的能力真实性缺陷

Open/Rename/Duplicate/Delete看似可点击，Host还记录context menu opened；用户无法分辨动作没有执行。M0必须先隐藏/禁用未接线动作并给出准确状态，再逐项开放。

#### ED71-P1-24：已有canonical Delete命令与Context Delete断开

`scene.node.delete_selected`已注册并有`SelectionNonEmpty`，Context字符串不引用它。相同意图在keymap/menu/palette/context/automation必须共享command id、admission和receipt。

#### ED71-P1-25：Rename、Duplicate及常用Scene context commands不完整

Registry没有与context动作对应的canonical rename/duplicate/frame/reveal/pilot/visibility命令集。新增命令时应路由到现有owner，不在Context服务内复制mutation。

#### ED71-P1-26：动作调用没有typed argument payload

仅command id不足以表达目标snapshot、surface、cursor placement、hierarchy destination或scope。需要schema-versioned typed args，远程/自动化可调用性由Command owner显式声明。

#### ED71-P1-27：没有打开时评估与执行时重校验两阶段协议

菜单可用性只是提示，不是授权。Invoke必须再次验证document/world/selection/provider/command generation和对象能力，失败返回typed terminal result而不是静默no-op。

#### ED71-P1-28：没有Scene context invocation receipt

调用方无法区分Executed、Disabled、Stale、OwnerRevoked、Cancelled、Rejected或Failed，也无法获得transaction id、affected targets和diagnostic provenance。必须建立不可伪造的terminal receipt。

#### ED71-P1-29：没有与Transaction/History的显式连接

Scene context action不能绕过Editor63直接调用world mutation。所有authoring修改必须提交owner-scoped command/transaction，并证明undo、redo、dirty/savepoint与失败回滚。

#### ED71-P1-30：document/world/view关闭时没有menu session retirement

项目关闭、scene reload、Play/prefab transition、viewport销毁或window重建都可能留下旧popup。Lifecycle owner必须批量retire相关session、取消pending invocation并清除projection。

#### ED71-P1-31：没有重复激活与重入防护

double click、键盘和pointer竞态、provider callback重入或慢命令可能重复提交。Session应提供one-shot invocation token和terminal state，长操作转为job/decision而非保持popup锁。

#### ED71-P1-32：UI callback期间没有明确锁与异常边界

当前链以可变Host/bridge层层调用，未来直接接provider会诱导持锁回调。聚合应先复制immutable snapshot，释放registry/world锁，再在bounded fault domain执行provider和command admission。

### 6.5 P1：Viewport、跨surface与输入体验

#### ED71-P1-33：Viewport secondary button永远启动Orbit

当前没有任何Scene object context-menu入口。必须选择并文档化短击/拖拽分流或显式modifier手势；不能在相同down事件同时打开菜单和捕获Orbit。

#### ED71-P1-34：没有secondary click-versus-drag recognizer

Primary已有`PRIMARY_NAV_THRESHOLD`，secondary没有距离、时间、device scale或cancel状态。需要独立gesture state machine，drag超过阈值才进入Orbit，短击才capture context snapshot。

#### ED71-P1-35：Context gesture未纳入tool/mode/capture arbitration

Scene mode、gizmo、modal tool、camera navigation与pointer capture可能优先消费secondary。应复用Editor53/59的resource/capture政策，输出明确Consumed/ContextRequested/NavigationStarted/Cancelled结果。

#### ED71-P1-36：Viewport context不消费current picking product

Pointer route已有current/stale interaction extract与visible spatial query，但secondary链不查询。Context snapshot必须只从current resolved hit构造；stale时请求render refresh或显示不可用，不能fallback猜对象。

#### ED71-P1-37：Outliner与Viewport没有共享动作图

相同selection在两个surface应共享Delete/Duplicate/Rename/visibility等canonical动作，只由surface facts决定Frame、Paste Here或Go To等差异。禁止复制两套provider与命令开关。

#### ED71-P1-38：没有键盘Context Menu key/Shift+F10入口

通用popup能键盘导航，但Scene surface没有从focused row/viewport selection打开context session的领域route。必须支持平台context key、Shift+F10和可替换keymap，并使用focused target而非最后一次pointer hit。

#### ED71-P1-39：焦点返回与accessible target描述未绑定session

关闭后应把焦点恢复到原surface/row，屏幕阅读器应获得对象名、类型、selection count与disabled reason。仅在popup subtree写focused布尔值不能证明焦点生命周期正确。

#### ED71-P1-40：multi-window、multi-view与multi-document隔离缺失

Workbench只有一个可见context control，request也没有ViewInstance。多个window/viewport并行打开、窗口重建或焦点切换时必须按window/surface/session identity隔离，并定义全局只允许一个还是每window一个popup stack。

### 6.6 P1：扩展、性能、诊断与验证

#### ED71-P1-41：`EditorMenuItemDescriptor`不足以表达Scene contextual provider

全局menu path、operation和静态enabled不能表达target schema、surface、selection predicate、section slot或动态status。应新增专用provider contract，而不是给全局descriptor塞可选Scene字段。

#### ED71-P1-42：provider owner lifecycle与generation未进入menu session

Extension snapshot有generation/ticket，但context request不保存。Plugin disable/reload后旧item仍可能显示；session必须记录provider lease/generation，revoke时失效并返回`OwnerRevoked`。

#### ED71-P1-43：没有provider故障隔离、预算与降级

第三方provider panic、超时、返回海量item或非法action不能阻塞UI线程或破坏built-ins。需要per-provider item/time/allocation/depth预算、异常隔离、diagnostic与deterministic partial result政策。

#### ED71-P1-44：排序冲突、stable identity与localization未统一

Label同时承担显示、path和action id，会让locale切换改变身份。Action id、localization key、owner、slot和display text必须分离；duplicate id/anchor冲突在compile阶段拒绝并给出owner provenance。

#### ED71-P1-45：没有compiled catalog、增量admission与规模预算

每次打开都重新构造字符串并刷新整个template subtree。目标应缓存immutable catalog，按command/selection/document/provider generation增量评估，支持large selection与100K Outliner而不扫描全World或全row。

#### ED71-P1-46：缺少业务、生命周期、故障与规模测试闭环

现有测试停在request/视觉层。必须覆盖right-click selection、duplicate names、stale entity/world、command/transaction/undo、provider revoke/fault、keyboard/focus、multi-view、大selection、100K rows和profile budget。

### 6.7 P2：产品成熟度增强

#### ED71-P2-01：没有按surface/role的动作排序与可见性偏好

在不破坏canonical section的前提下，可允许用户隐藏非关键动作、调整扩展section或恢复默认；危险动作和平台惯例位置不可任意重排。

#### ED71-P2-02：长菜单没有搜索/过滤与分层发现

复杂对象可能汇聚几十个built-in/plugin动作。应在超过预算时提供可访问搜索或二级palette，而不是无限纵向菜单。

#### ED71-P2-03：没有Recent/Repeat Context Action

可在同一target schema和admission通过时提供“重复上次动作”，但必须重新评估currentness，不能重放旧entity handle。

#### ED71-P2-04：没有rich tooltip与结果预览

Reparent、replace、bulk delete等高影响动作可显示target count、scope与结果摘要；预览必须来自immutable plan，不得提前修改world。

#### ED71-P2-05：pen/touch long-press没有产品政策

触控和笔设备需要long-press、barrel button或替代入口，并与camera gesture/capture冲突测试；不能把mouse阈值直接套用。

#### ED71-P2-06：没有可选radial/quick action surface

高频viewport工作流可在canonical action service之上增加radial projection，但不应形成第二套动作或admission authority。

#### ED71-P2-07：没有隐私受控的action usage telemetry

性能/失败率/disabled原因可帮助产品优化，但对象名、路径和项目内容默认不得采集；telemetry需opt-in、redaction和sampling政策。

#### ED71-P2-08：没有Context Action Inspector开发工具

应能查看snapshot identity、provider provenance、section排序、admission reason和invoke receipt，便于插件作者定位冲突与性能问题。

#### ED71-P2-09：没有contextual help/documentation action规范

类型/组件帮助可作为只读动作贡献，但URL、offline docs、locale和security policy应由统一Help owner管理。

#### ED71-P2-10：没有跨平台菜单密度与native convention验收

Windows/macOS/Linux在secondary gesture、context key、focus、screen-edge placement与accelerator呈现上有差异，应有平台golden和可配置密度，不改变action identity。

## 7. 目标架构

### 7.1 Identity与snapshot

建议的最小领域类型：

```rust
pub struct SceneContextSurfaceId {
    pub window: EditorWindowId,
    pub view: Option<ViewInstanceId>,
    pub kind: SceneContextSurfaceKind,
}

pub struct SceneContextTargetSnapshot {
    pub document: SceneDocumentIdentity,
    pub world: QualifiedWorldIdentity,
    pub surface: SceneContextSurfaceId,
    pub target: SceneContextTarget,
    pub selection: Arc<[QualifiedSceneObjectHandle]>,
    pub primary: Option<QualifiedSceneObjectHandle>,
    pub selection_revision: u64,
    pub hierarchy_generation: Option<u64>,
    pub interaction_generation: Option<u64>,
    pub pointer: Option<SceneContextPointerFacts>,
    pub mode_revision: u64,
    pub authority_revision: u64,
}
```

`SceneContextTarget`至少区分EmptySurface、Object、Component/Subobject、HitCandidateSet和FocusedSelection。Snapshot必须immutable、bounded且不持有World可变借用；显示文本另行投影。

### 7.2 Action definition与provider registry

`SceneContextActionDefinition`应包含stable id、owner、localized label/description/icon key、section/slot/anchor、command/operation、argument builder、target schema、selection applicability、visibility/admission predicate和remote/automation policy。`SceneContextActionProviderRegistry`复用Extension ticket/capability/lifecycle，但拥有独立Scene contract、compile diagnostics和generation。

### 7.3 两阶段聚合与menu session

阶段一由surface捕获target和selection policy，生成snapshot；阶段二从compiled catalog筛选provider、批量读取neutral capabilities、评估visible/enabled/disabled reason，生成immutable `SceneContextMenuSession`。UI只获得session id、typed item handles和presentation DTO，不获得command closure、World pointer或display-derived action id。

### 7.4 Invocation与receipt

`invoke(session, item)`先验证session active、provider owner、document/world/view/selection/target generation，再调用canonical command admission，最后提交Editor55/60/63 owner command或只读operation。建议terminal receipt：`Executed { command, transaction, affected }`、`Disabled { reason }`、`StaleRefreshRequired`、`OwnerRevoked`、`Cancelled`、`Rejected`、`Failed { diagnostic }`。UI状态只能由receipt驱动，不能因popup成功关闭就显示业务成功。

### 7.5 Input与surface policy

Outliner policy通常为“右击selected member保留selection；右击未选中行先replace；空白区使用empty target”。Viewport必须在产品层明确选择：推荐secondary down进入pending recognizer，超过scale-aware threshold转Orbit，release前未超过阈值则从current picking product构造context session；若坚持Godot式RMB导航，则提供明确modifier/keyboard入口和候选列表。两者都调用同一action service。

### 7.6 Runtime边界

Runtime只提供qualified entity、component/type capability、hierarchy facts、read-only neutral facts、bounds/hit/visible-spatial currentness和必要的batch query；它不依赖Editor action id、menu section、ZUI、plugin UI provider或focus。任何把`SceneContextMenuSession`放入`zircon_runtime`的实现都违反固定Editor/Runtime边界。

### 7.7 Performance与故障策略

Compiled catalog按extension generation发布；admission批量读取selection capability summary，避免N actions × M targets重复查询。打开菜单不得扫描全World/全Outliner；provider在无锁snapshot上执行并受item count、submenu depth、time/allocation budget约束。Built-ins不能被第三方故障整体清空，超预算provider只产生diagnostic占位或被省略。

## 8. 依赖有序重构里程碑

### ED71-M0：能力真实性止血

- 隐藏或disabled当前无consumer的Open/Rename/Duplicate/Delete，移除会暗示业务成功的反馈。
- 建立source-shape test，禁止Scene action再次只更新popup value。
- 固定Editor08/50/55/59/60/61/63父owner边界。

### ED71-M1：Identity、Snapshot与Session内核

- 引入surface、target、selection、pointer facts和menu session identity。
- Outliner从`SceneHierarchyProjectionState`构造qualified snapshot。
- 实现open/active/invoking/retired state machine及lifecycle retirement。

### ED71-M2：Typed Action Catalog与Admission

- 建立built-in definitions、section/slot/anchor和compiled catalog。
- 批量解析selection/object/document capabilities。
- 生成visible/enabled/disabled reason的typed projection。

### ED71-M3：Canonical Command、Revalidation与Transaction

- 先接只读Open/Frame/Reveal，再接Rename/Duplicate/Delete。
- 所有修改经canonical command和Editor63 transaction。
- 实现one-shot invoke、two-phase validation与terminal receipt。

### ED71-M4：Outliner产品闭环

- 实现右击selection policy、empty surface、multi-selection与clipboard动作。
- 覆盖root/inherited/locked/read-only/owner状态。
- 完成keyboard context key、focus return和screen reader描述。

### ED71-M5：Viewport Gesture与Picking闭环

- 建立secondary pending/click/drag/cancel recognizer或正式替代手势。
- 只消费current Editor59 picking product，支持candidate disambiguation与cursor world facts。
- 证明Orbit、tool capture、gizmo和context互不误触。

### ED71-M6：Extension Provider与Fault Domain

- 新增owner-aware scene contextual provider API。
- 实现register/replace/revoke、generation fence、collision diagnostics与deterministic order。
- 添加panic/timeout/oversize/malformed contribution隔离。

### ED71-M7：Localization、Accessibility与跨平台体验

- stable id与localization key/display text分离。
- 完成disabled reason、tooltip、accelerator、keyboard/focus与平台placement golden。
- 增加touch/pen入口政策。

### ED71-M8：规模、缓存与诊断

- compiled catalog cache与incremental admission。
- large selection capability summary、100K Outliner和多view预算。
- Context Action Inspector、trace与redacted metrics。

### ED71-M9：动态资格与跨引擎同语义基线

- 完成E2E、undo/redo、save/reload、plugin reload、fault/soak/profile矩阵。
- 对Unreal/Godot/Fyrox建立同语义动作集、选择政策与延迟基线。
- 只有48门全部Pass后才能提升implementation/review truthfulness。

## 9. 资格门

| Gate | 要求 | 当前 |
|---|---|---|
| ED71-G01 | Scene context拥有唯一Editor authority | Fail |
| ED71-G02 | Snapshot含document/world/view/surface generation | Fail |
| ED71-G03 | Target使用qualified object/subobject handle | Fail |
| ED71-G04 | Selection snapshot immutable且带revision | Fail |
| ED71-G05 | duplicate/localized name不影响target identity | Fail |
| ED71-G06 | stale/deleted/reused target在invoke时拒绝 | Fail |
| ED71-G07 | world/document/view replacement会retire session | Fail |
| ED71-G08 | empty/viewport target携带bounded pointer facts | Fail |
| ED71-G09 | Action definition为typed stable schema | Fail |
| ED71-G10 | Section/slot/anchor/submenu可确定编译 | Fail |
| ED71-G11 | Label、localization key与action id分离 | Fail |
| ED71-G12 | Visible/enabled/disabled reason完整投影 | Fail |
| ED71-G13 | single/multi/mixed selection规则可测试 | Fail |
| ED71-G14 | empty surface动作集可测试 | Fail |
| ED71-G15 | read-only/locked authority双阶段校验 | Fail |
| ED71-G16 | root/inherited/instance限制有stable reason | Fail |
| ED71-G17 | clipboard/destination admission复用Editor55 | Fail |
| ED71-G18 | Context、keymap、menu、palette共享command id | Fail |
| ED71-G19 | Invocation payload有版本化typed schema | Fail |
| ED71-G20 | Command `WhenClause`与Scene capability组合 | Fail |
| ED71-G21 | Invoke执行currentness revalidation | Fail |
| ED71-G22 | 每次invoke产生typed terminal receipt | Fail |
| ED71-G23 | authoring修改产生transaction/undo/redo证据 | Fail |
| ED71-G24 | popup关闭不再冒充业务成功 | Fail |
| ED71-G25 | Delete context动作走canonical delete owner | Fail |
| ED71-G26 | Rename/Duplicate走各自canonical owner | Fail |
| ED71-G27 | Viewport secondary click/drag可确定分流 | Fail |
| ED71-G28 | Orbit在context short-click后不误启动 | Fail |
| ED71-G29 | Viewport context只消费current picking product | Fail |
| ED71-G30 | tool/mode/modal/capture仲裁有terminal结果 | Fail |
| ED71-G31 | Outliner/Viewport共享同一action graph | Fail |
| ED71-G32 | Context key/Shift+F10/focus return闭环 | Fail |
| ED71-G33 | accessible role/name/state/reason动态验证 | Fail |
| ED71-G34 | locale切换不改变action/target identity | Fail |
| ED71-G35 | 插件可注册typed Scene context provider | Fail |
| ED71-G36 | provider revoke/reload使旧session失效 | Fail |
| ED71-G37 | provider panic/timeout/oversize被隔离 | Fail |
| ED71-G38 | duplicate id/anchor冲突确定拒绝 | Fail |
| ED71-G39 | item/depth/time/allocation预算可配置验证 | Fail |
| ED71-G40 | catalog与admission按generation增量更新 | Fail |
| ED71-G41 | 10K selection不做actions × targets重复扫描 | Fail |
| ED71-G42 | 100K Outliner打开菜单满足预算 | Fail |
| ED71-G43 | multi-window/view/document session相互隔离 | Fail |
| ED71-G44 | close/reload/Play/prefab/shutdown retirement闭环 | Fail |
| ED71-G45 | malformed contribution fuzz不破坏built-ins | Fail |
| ED71-G46 | E2E证明动作、world delta、undo与receipt一致 | Fail |
| ED71-G47 | profile/soak无UI stall、泄漏或无界增长 | Fail |
| ED71-G48 | 同语义跨引擎benchmark有可复验证据 | Fail |

## 10. 测试与动态证据矩阵

| 层级 | 必须新增的证据 |
|---|---|
| Pure model | target/action id、selection policy、section ordering、collision、disabled reason、session state machine |
| Outliner integration | selected/unselected/empty/right-click、duplicate names、multi-select、root/locked/read-only、focus return |
| Viewport input | short-click/drag/threshold/cancel、scene mode/capture、stale picking、overlap candidate、HiDPI |
| Command/transaction | Open/Frame/Rename/Duplicate/Delete、typed args、revalidation、undo/redo、dirty/savepoint、double activation |
| Lifecycle | world/document/view/window replacement、Play/prefab transition、plugin revoke/reload、shutdown |
| Fault | provider panic/timeout/oversize/malformed、command rejection、transaction failure、diagnostic provenance |
| Accessibility | context key、Shift+F10、arrow/typeahead/Escape、disabled announcement、screen reader、locale/RTL |
| Performance | 1/100/10K selection、100K Outliner、many providers、warm/cold catalog、allocation与UI-frame budget |
| Product | real Editor screenshot/input capture、screen-edge placement、multi-window、save/reopen、cross-platform |
| Comparative | 与Unreal/Godot/Fyrox对齐的相同对象/selection/action scenario和可复现延迟/行为receipt |

当前没有执行上述动态矩阵。静态source-shape或popup截图只能证明投影存在，不能把任何Gate改成Pass。

## 11. Owner路由与禁止重复实现

| 责任 | Canonical owner | Editor71只能做什么 |
|---|---|---|
| 通用command/keymap/menu/palette/remote | Editor08 | 引用stable command、补Scene args/admission adapter |
| 通用extension store/lifecycle | Editor50 | 增加专用Scene provider contract并复用ticket/generation |
| clipboard/duplicate/delete语义 | Editor55 | 聚合动作并提交owner command |
| picking/selection/capture | Editor59 | 读取current hit与执行surface selection policy |
| Outliner/hierarchy mutation | Editor60 | 使用projection identity，提交rename/reparent等owner命令 |
| document/world lifecycle | Editor61 | 绑定session identity与retirement |
| transaction/history | Editor63 | 提交owner-scoped transaction并投影receipt |
| camera/frame/pilot | Editor66 | 提供context action adapter，不复制camera逻辑 |
| visualization/object visibility | Editor68/70 | 聚合typed action，不拥有visibility resolver |
| Runtime entity/hierarchy/query | Runtime Scene owner | 只消费neutral batch facts，不下沉菜单状态 |

禁止用以下临时方案关闭本报告：label/path作为entity id、点击时读取未限定current selection、在popup bridge里直接改World、为Outliner和Viewport各写一套switch、给`EditorMenuItemDescriptor`堆Scene可选字段、让plugin返回任意closure并持有World、把context session放进Runtime、用静态source-shape test替代transaction/undo/input动态证据。

## 12. 状态与产出记录

- 审查状态：`complete`，仅表示本轮current-source差距建账完成。
- 实现状态：`not_started`。
- 新增finding：`0 P0 / 46 P1 / 10 P2`。
- 资格门：`0 Pass / 48 Fail`。
- 建议首个实施点：ED71-M0，先撤销no-op动作的可用表象，再建立M1 identity/session；不得先把Delete字符串接到current selection。
- 实施前置：重取43个Zircon文件、相关父报告、Extension/Command generation和Viewport/Outliner currentness；重新冻结working-tree fingerprint。
- 验证声明：本轮未运行Cargo与动态产品验证，不能宣称功能、性能、表现、无障碍、插件安全或跨平台已达到目标。

## 13. 最终判断

当前Zircon不是“场景右键菜单功能较少”，而是尚未建立Scene contextual action这一领域系统。现有popup控件、command predicate、extension generation、hierarchy entity map和viewport picking都是可保留积木；固定字符串provider、display-derived target path、无session的popup属性和secondary直接Orbit则是必须替换的临时实现。

正确路线是先冻结qualified target/selection snapshot和per-open session，再编译typed action catalog，随后通过canonical command、two-phase revalidation和transaction receipt逐项开放动作，最后让Outliner、Viewport、keyboard与插件共享同一action graph。只有当48个资格门全部通过，才能把Scene contextual action从“有一个会弹出的菜单控件”提升为工程级编辑器能力。
