---
title: Editor Scene Contextual Action、Context Menu、Target/Selection Snapshot、Availability、Command Routing、Extension、Transaction、Accessibility、Performance 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor192
review_date: 2026-08-28
baseline_head: 681588f7a1cbfaae3147e8b93e1be6705d810f21
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
  - zircon_editor/src/scene/viewport/pointer/precision/renderer_visible_spatial_pick_source.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport/pointer_dispatch.rs
  - zircon_editor/src/ui/workbench/state/editor_state_keep_play_changes.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
tests:
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu_tests.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/context_menu.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_reference/dropdown_pointer.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/pointer_bridge.rs
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/commands/descriptor_when.rs
  - zircon_editor/src/tests/commands/when.rs
  - zircon_editor/src/core/extension/store/tests.rs
  - zircon_editor/src/tests/editing/state/play_mode.rs
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
  - docs/plans/zircon_editor/editor/05/failure-2026-07-18-viewport-pointer-candidate-regeneration.md
  - docs/plans/zircon_editor/editor/05/failure-2026-07-22-world-inspection-generation-projection.md
  - docs/plans/zircon_editor/editor/05/failure-2026-07-31-scene-mode-input-ownership-hardcut.md
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
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/71-editor-scene-contextual-action-context-menu-target-selection-snapshot-availability-command-routing-extension-transaction-accessibility-performance-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/71-editor-scene-contextual-action-context-menu-target-selection-snapshot-availability-command-routing-extension-transaction-accessibility-performance-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Scene Contextual Action、Context Menu、Target/Selection Snapshot、Availability、Command Routing、Extension、Transaction、Accessibility、Performance 与 Product Integration 当前源码复核

## 1. 结论

Editor71之后，Zircon的通用command、extension、Hierarchy projection与popup底座有实质进展。`EditorCommandDescriptor`已具有stable operation path、menu projection、typed payload schema id、headless route、remote policy、asset write target和capability gate；`CommandEvalCtx`是immutable snapshot，并携带selection count/domain/revision、scene-mode revision、Play state与asset write access。`ContributionSnapshot`具有generation、owner ticket、source和capability过滤，旧generation reader保持immutable。Hierarchy retained projection还保存generation、selection revision、entity/control双向映射与selected entity set。这些基础必须复用。

通用popup也不再完全依赖label生成action id。`TemplatePopupMenuItemState`与pane projection都优先保留显式`action=...`，旧`ED71-P1-14`因此关闭。popup可投影separator、disabled、checked、focused、hovered、pressed、loading、shortcut，打开时按内容测量并限制到shell范围，关闭时移除anchor。Play模式下Scene row会动态插入`Keep Play Changes`，它能映射到typed `MenuAction::KeepPlayChanges`，最终通过真实authoring transaction复制可序列化属性；这是一条窄但真实的业务路径。

Scene Contextual Action产品主体仍未建立。当前request仍由UI hit的control/action/role/display text和物理坐标组成，不含document、world、view、surface、entity/subobject、selection snapshot或revision。Scene识别依赖control/action字符串前缀，target path由显示文本归一化生成；Hierarchy已存在的entity映射和generation完全未被右键链消费。打开后没有immutable menu session、provider catalog、owner generation、invoke-time revalidation或typed receipt。

Scene provider仍生成固定Open/Rename/Duplicate/Delete raw-string数组，只在Play时插入一个Keep项。bridge只为Keep提供业务binding；其余四项保持可点击外观但没有Scene consumer。canonical Delete command、transactional delete/rename和structured command descriptor均已存在，却没有被Context Delete/Rename/Duplicate复用。popup关闭只清`menu_items`和open/focus状态，不清`value`、`value_text`、`context_target`与`context_target_path`，旧上下文仍残留在控件属性中。

Viewport链也未接入。secondary down立即成为`RightPressed`，controller立即武装Orbit；release只结束Orbit，没有click-versus-drag recognizer、context intent、current picking result或tool/mode/capture终态。Outliner native secondary虽然会创建菜单，却只传UI hit。Viewport与Outliner因此不可能共享同一个action graph、target snapshot和availability结果。

本轮不新增P0。Editor71的46项P1当前为 **23 Open / 22 Partial / 1 Closed**，10项P2为 **10 Open**；48门为 **28 Fail / 20 Partial / 0 Pass**。Partial只表示command、extension、Hierarchy、popup、picking或transaction底座可复用，不表示Scene context工作流已完成。

本轮只做review，未修改production Rust，未运行Cargo、Editor、GUI、真实右键业务、transaction E2E、accessibility、fault、scale、soak、profile或同硬件跨引擎benchmark。Tooling按用户要求排除；没有查询、轮询、等待或实时跟踪协调器。当前不能声称该域功能、表现或性能达到或超过Unreal。

## 2. 审查边界与冻结语料

### 2.1 Current working tree

主仓HEAD为`681588f7a1cbfaae3147e8b93e1be6705d810f21`。本报告以2026-08-28读取时当前磁盘为事实源；相关Editor源与测试含其他会话的modified或untracked实现。本轮不回退、不格式化、不吸收这些生产代码，只按实际行为更新review。

MVP baseline recovery仍为`in_progress`。Editor05的pointer candidate、world inspection projection和scene-mode input failure记录证明shared extract、generation projection与mode arbitration已有真实底座，但空间规模、受管产品验证和若干输入所有权闭环仍未完成。它们不阻塞本轮只读审查，也不能替代Scene context产品证据。

### 2.2 冻结物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Editor context/product | **24 / 5,202 / 4,799 / 185,055 / 27** | request/provider/popup、command/extension、Hierarchy、Viewport secondary/picking与Keep transaction链 | `572c3de85b3214da67d02b77acd59f6bb698ffe9cf8240d2640924f2feedf122` |
| Focused tests | **9 / 3,167 / 2,866 / 109,127 / 73** | request/overlay/binding/pointer、command/extension与Keep Play transaction | `ac50d0aeaeb3e101f28bae400f7404fd3fec5f6a45e0ccccb85cbf0f5a344d42` |
| Zircon deduplicated focused set | **33 / 8,369 / 7,665 / 294,182 / 100** | 上述两组路径无重叠 | `03954ee49d54b50c0aa155044b52ac179d57d517ea9a475ed563b340aed8c9a8` |
| Unreal selected set | **4 / 7,956 / 6,786 / 291,423 / 0** | typed context、selection/hit/cursor、ToolMenus、command list与extenders | `074a2a888ebf2ea3672cb73b56a0d9f4a036a6608ffe9db6ea4fd6ae7a655b84` |
| Godot selected set | **2 / 12,586 / 10,697 / 477,240 / 0** | per-open selection/capability菜单、ownership/inheritance/clipboard与plugin path | `805db9a5cc4859b4661cea7cafee00cfbafbfb8ebc4e37f45cfc8f325b5c0486` |
| Fyrox selected set | **3 / 4,672 / 4,236 / 185,097 / 0** | stable UUID、open-time enablement、selection与CommandGroup | `eac702ebaa9f67f3a1ab8224dacdbc78cf54f9d672abee9c324457a17cf96130` |
| Bevy selected set | **2 / 806 / 737 / 29,207 / 3** | popup focus、close/restore、accessible role与context example | `1d7f2d9d96d57c24abc97b37ddf78e414fbf3635e328e28231b29726dce79e2c` |
| Unity Graphics selected set | **5 / 6,507 / 5,571 / 271,920 / 0** | graph-domain dynamic action、status、target与component context dispatch | `0267422337076e63fc4cfaf386655d27fb1926814bc14962d86536f84559fbd4` |

fingerprint按小写规范化相对路径排序，将每个`path + newline + file SHA-256 + newline`聚合后再做SHA-256，只证明本轮working-tree选择集。Godot、Fyrox、Bevy与Unity Graphics revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal跟随主workspace。

### 2.3 Owner边界

Editor192只刷新Editor71拥有的Scene contextual target snapshot、provider/catalog aggregation、per-open session、surface projection和invoke receipt。Editor08/178继续拥有canonical command registry、keymap、menu/palette与remote admission；Editor50/171拥有extension contribution store、owner ticket、generation与revoke；Editor55/176拥有clipboard、duplicate/delete transfer；Editor59/180拥有Viewport picking/selection；Editor60/181拥有Hierarchy/Outliner；Editor61/182拥有Document/World lifecycle；Editor63/184拥有transaction/history；Editor66/187拥有camera input；Editor70/191拥有object visibility。实现必须连接这些owner，不能在Context Menu内部复制第二套selection、command、transaction、extension或picking authority。

## 3. 当前实现拓扑

### 3.1 UI hit被直接当作领域target

`context_menu_provider_for_hit`根据`WorkbenchScene...Item`、`workbench.hierarchy.`与`scene_tree.`前缀猜测Scene row。`target_value_text`优先读取显示文本，`push_path_segment`再将ASCII字母数字、空格、连字符、下划线和点归一化成`workbench://scene/<display text>`。同名、重名、locale切换、rename和非ASCII名称都会破坏identity。

### 3.2 Request没有scene或selection事实

`WorkbenchContextMenuRequestData`只有control/action/dispatch/role/value/path、x/y和raw menu items。它没有Project/Document/World/View/Surface identity、world generation、EntityId、subobject、selection items/revision、pointer device、current pick receipt或source owner。request创建后也没有resolver把这些UI字段升级为qualified domain snapshot。

### 3.3 Hierarchy已有正确底座但右键链绕过它

`SceneHierarchyProjectionState`保存generation、selection revision、rows by entity、control/entity双向map和selected set，logical row patch支持按entity更新。native secondary dispatch却只调用`workbench_context_menu_request_for_hit(&hit, x, y)`；没有读取`entity_for_control`、generation或selection revision。这使现有工程化投影无法参与target currentness。

### 3.4 Scene菜单仍是raw-string产品

Scene provider返回Open、Rename、Duplicate、separator、Delete五个字符串；Play模式按target path前缀插入Keep Play Changes。schema把label、action、danger、icon、disabled等压进`Label|flag,...|shortcut`文本。两个parser分别存在于popup primitives与pane projection，规则相近但没有共享typed schema、版本、section、anchor、provider owner或localization key。

### 3.5 一条Keep业务路径不能证明整个菜单可用

bridge只识别`menu.item.keep_play_changes`，其他Scene action id没有binding。Keep最终进入`MenuAction::KeepPlayChanges`并由EditorState提交authoring transaction；但context target并未传给命令，实际依赖当前selection。Open/Rename/Duplicate/Delete点击仍只更新popup value并关闭，无法产生world delta、transaction或terminal receipt。

### 3.6 Canonical command与transaction底座未接入

`scene.node.delete_selected`具有stable id、Delete chord和`SelectionNonEmpty` when clause；transaction engine已有delete/rename command和journal。Keep Play Changes command还组合ProjectOpen、Playing、SelectionNonEmpty与AssetWritable。Context provider没有引用这些command id，也不使用`CommandEvalCtx`或payload schema，所以菜单外观、command availability和invoke admission可以漂移。

### 3.7 Extension generation没有进入menu session

`EditorMenuItemDescriptor`现可表达path、operation、priority、shortcut、enabled和required capabilities；`ContributionSnapshot`按generation冻结ticket/source/capability并支持revoke后的旧reader immutable。但没有Scene Context Action provider类型、section/anchor模型或menu session。provider revoke/reload不能使已打开菜单失效，callback panic/timeout/oversize也没有Scene context fault domain。

### 3.8 Popup lifecycle仍残留旧上下文

打开时bridge写入menu items、value/value_text、context target/path、anchor与focus/open状态。关闭只隐藏、清menu items并清open/focus/selected；没有清value/value_text/context target/path。UI属性因此可在菜单关闭后保留旧对象文本和路径，也没有focus-return token或session retirement receipt。

### 3.9 Viewport secondary被硬编码为Orbit

pointer dispatcher将secondary down/up直接映射为RightPressed/RightReleased。controller在down时立即创建`ViewportDragSession::Orbit`，move更新camera，release清理；没有距离/时间threshold或short-click分支。scene-mode stack可以先消费输入，这是可复用arbitration底座，但Context gesture本身不在mode effect中，也不消费current renderer-visible picking product。

### 3.10 测试覆盖视觉和局部binding，不覆盖领域闭环

当前测试验证Scene hit产生字符串菜单、popup不递归打开、overlay可见/尺寸/anchor、显式action id、native secondary request、Keep typed binding、command descriptor/when、extension snapshot和Keep transaction。没有测试证明Context Delete/Rename/Duplicate改变指定对象、selection race被拒绝、provider revoke退休菜单、multi-window隔离、keyboard/a11y、fault budget或100K规模。

## 4. 五引擎参考与适用边界

### 4.1 Unreal

`FLevelEditorContextMenu::InitMenuContext`把LevelEditor command list、context type、current typed-element selection、hit-proxy typed element、hit actor、selected editable components与cursor world location放入`FToolMenuContext`。Actor、Component、Element、Scene Outliner与Empty Selection是不同菜单，ToolMenus负责注册/派生/section，LevelEditor extenders收到command list和selected actors。Zircon应借鉴typed context、command reuse、surface-specific projection和owner-aware extension，而不是复制Unreal的全局Editor singleton。

### 4.2 Godot

`SceneTreeDock::_tree_rmb`每次打开都读取top/full selection，按profile editing、root、owner、instance、inherited state、clipboard、script和selection cardinality重建sections；empty selection有不同动作集。插件收到相对edited-scene root的真实node paths，menu action与shortcut共享`TOOL_*`/shortcut owner。它证明availability必须由current domain facts生成，但裸NodePath仍不足以作为Zircon的跨world generation identity。

### 4.3 Fyrox

`SceneNodeContextMenu`为action使用stable UUID，popup placement时动态设置Paste与Open Asset enabled，点击后通过`Command`/`CommandGroup`或明确消息执行delete、paste、replace、make root和revert。它的menu仍长期读取current selection，缺少Zircon目标要求的immutable open snapshot与invoke fence；因此只作为stable identity、dynamic enablement和transaction底线，不作为最终上限。

### 4.4 Bevy

Bevy menu widget明确描述popup open/close、focus child、Escape关闭后focus返回button与accessible role；context example用typed component承载item数据并在关闭时despawn。它是UI lifecycle/a11y参考，不提供Scene authoring authority、selection snapshot或transaction模型。

### 4.5 Unity Graphics

本地Graphics包的VFX/ShaderGraph/RenderGraph菜单按真实graph target、selection和domain status动态添加action，部分action在无效configuration时停止构建；component context dispatcher通过`MenuCommand.context`取得真实component。该镜像不包含完整Unity Scene Hierarchy私有实现，不能用来宣称Scene产品对标完成，但足以反证固定跨domain字符串数组。

## 5. 差异矩阵

| 能力 | Zircon当前事实 | 工程级目标 | 判定 |
|---|---|---|---|
| Target identity | display text/control prefix派生path | qualified document/world/view/entity/subobject handle | Missing |
| Selection/currentness | Hierarchy/Command各有revision底座，request不消费 | immutable target+selection snapshot与invoke fence | Partial |
| Action model | raw strings、两个parser、显式action id可保留 | versioned typed definition、section/anchor/submenu | Partial |
| Availability | Keep按Play插入，command有generic WhenClause | object capability、multi/mixed、lock/root/clipboard reason | Partial |
| Command routing | Keep可执行，Delete/Rename底座存在 | 全action复用canonical command id/payload | Partial |
| Transaction/receipt | Keep和底层edit command有transaction | context invoke统一transaction与terminal receipt | Partial |
| Extension | generation/ticket/capability snapshot存在 | typed Scene provider、session-bound owner generation | Partial |
| Popup lifecycle | anchor/focus/close基础存在 | full context clear、focus return、retirement receipt | Partial |
| Outliner | entity/control generation projection存在 | target snapshot和shared action graph consumer | Partial |
| Viewport | current picking与mode arbitration基础存在 | secondary recognizer、picking target、shared catalog | Partial |
| Accessibility | generic popup state/role基础 | Context key、Shift+F10、reason与focus闭环 | Partial |
| Scale/fault | snapshot/delta/physical rows有基础 | compiled catalog、budgets、fault isolation、100K proof | Partial |

## 6. Canonical finding状态

### 6.1 P1：Target、Selection与currentness

#### ED71-P1-01 [Open]：没有Scene Contextual Action authority

没有拥有snapshot、catalog、session、invoke和receipt的Editor service；UI bridge继续兼任临时产品owner。

#### ED71-P1-02 [Open]：request envelope没有qualified scene identity

request缺Project/Document/World/View/Surface及其generation，无法隔离multi-window、world replacement和ID复用。

#### ED71-P1-03 [Open]：target path由显示文本派生

同名、rename、locale与非ASCII可改变path；它不能作为对象identity或command payload。

#### ED71-P1-04 [Open]：Scene provider依赖control/action前缀猜测

UI模板命名约定被当作domain type system，resource/plugin/custom row可被误分或漏分。

#### ED71-P1-05 [Partial]：Hierarchy generation/entity映射已存在但未进入请求

双向map、generation与selection revision是真实底座；secondary request没有消费任何一项。

#### ED71-P1-06 [Partial]：selection revision底座存在，没有immutable context snapshot

Hierarchy与`CommandEvalCtx`都保存selection revision/domain/count；menu打开时没有冻结items、primary、ordering和source generation。

#### ED71-P1-07 [Open]：右击未选中对象的selection policy未定义

没有replace/retain/extend政策，也没有决定action作用于hit target还是current selection。

#### ED71-P1-08 [Open]：没有entity/subobject target resolution与retirement

UI hit不能解析为qualified entity/component/property，删除、world替换或ID复用也不会使target失效。

#### ED71-P1-09 [Open]：没有重叠目标与subobject disambiguation

Viewport没有候选列表、优先级、pick-through或component/actor层级选择模型。

#### ED71-P1-10 [Open]：popup关闭不清理context target/value

`menu_items`被清空，但value、value_text、context target/path继续留在控件属性中。

### 6.2 P1：Action model、聚合与availability

#### ED71-P1-11 [Partial]：固定五项数组只增加了一个Play条件项

Keep Play Changes证明可以按Play动态插入，但base Scene action仍是固定Open/Rename/Duplicate/Delete数组，没有provider aggregation。

#### ED71-P1-12 [Open]：raw string item schema不能承载工程合同

没有schema version、typed icon、localization key、payload、owner、section、reason或provenance。

#### ED71-P1-13 [Open]：存在两套popup item解析与action identity规则

popup primitives与pane projection各自解析flags和fallback action id，仍会独立漂移。

#### ED71-P1-14 [Closed]：pane投影已保留显式action id

当前`explicit_action_id_is_independent_from_display_label`测试证明`action=...`优先于label fallback；该窄缺陷关闭，但raw schema与target label identity问题仍在其他finding跟踪。

#### ED71-P1-15 [Open]：没有section、slot、anchor和submenu聚合合同

separator只是数组字符串，无法确定性合并built-in、plugin、mode和surface贡献。

#### ED71-P1-16 [Partial]：generic availability底座存在，缺对象能力评估

`WhenClause`支持selection、document kind、scene mode、Play、write和capability；context provider只检查Play与path前缀，不读取对象能力。

#### ED71-P1-17 [Partial]：generic disabled state可投影，缺domain reason

popup row能显示disabled，但request/action没有stable disabled reason、remediation或a11y说明。

#### ED71-P1-18 [Partial]：selection cardinality/revision存在，bulk规则缺失

command context与Hierarchy保存数量/集合/revision；没有single/multi/mixed support、all/any policy或10K bulk admission。

#### ED71-P1-19 [Partial]：AssetWritable是局部底座，object restrictions缺失

command系统可拒绝read-only asset；locked、inherited、root、instance ownership、foreign world和provider authority没有Context policy。

#### ED71-P1-20 [Open]：clipboard与destination能力没有接入

Context Paste/Duplicate没有读取Editor55 clipboard source、destination、remap或write admission。

#### ED71-P1-21 [Open]：empty surface与cursor world context没有模型

Unreal式cursor world位置、surface action和Create/Paste Here无法由当前Scene row request表达。

### 6.3 P1：Command、transaction与receipt

#### ED71-P1-22 [Partial]：只有Keep Play Changes具有业务consumer

Keep能到达typed MenuAction和真实authoring transaction；Open/Rename/Duplicate/Delete仍没有Scene consumer。

#### ED71-P1-23 [Open]：可点击no-op能力真实性缺陷仍在

四个base action会更新popup value并关闭，用户无法从成功外观区分无业务执行。

#### ED71-P1-24 [Partial]：canonical Delete存在但Context Delete断开

`scene.node.delete_selected`与transactional delete command可复用；context action id未映射到该command，也没有target/selection snapshot。

#### ED71-P1-25 [Partial]：Rename/Delete编辑底座存在，常用Context commands未闭合

transaction engine已有rename/delete，Create/Delete Selection也有canonical route；Duplicate、Rename Context、Open、Focus、Copy Path等仍未统一注册和接线。

#### ED71-P1-26 [Partial]：command payload schema底座存在，Context invocation无payload

descriptor可声明`payload_schema_id`；menu click只传action id/label，无法携带target handle、selection snapshot或cursor facts。

#### ED71-P1-27 [Partial]：immutable command eval底座存在，没有两阶段协议

`CommandEvalCtx`携带selection/scene-mode revision；Context open未保存evaluation receipt，invoke也不重新解析target、selection、write、lock和owner generation。

#### ED71-P1-28 [Open]：没有Scene context invocation receipt

缺Executed/Rejected/Stale/Cancelled/Faulted、transaction id、world delta、provider/action id和terminal timestamp。

#### ED71-P1-29 [Partial]：Keep与底层编辑命令有transaction，通用Context未连接

真实transaction/undo能力存在，但Context action没有统一factory、atomic plan和history evidence。

#### ED71-P1-30 [Open]：document/world/view关闭不退休menu session

当前没有session，自然也没有close/reload/world replacement/Play transition的retirement。

#### ED71-P1-31 [Open]：没有重复激活与重入防护

double click、key repeat、callback reentry、nested modal和async completion没有invocation token或idempotency rule。

#### ED71-P1-32 [Partial]：typed bridge错误可保留，锁与异常边界未定义

open/close返回structured bridge error并投影status；provider/command callback期间的锁释放、panic boundary、rollback和terminal receipt仍未建立。

### 6.4 P1：Viewport、跨surface与输入体验

#### ED71-P1-33 [Open]：Viewport secondary button永远启动Orbit

down立即创建Orbit drag session，没有context candidate状态。

#### ED71-P1-34 [Open]：没有secondary click-versus-drag recognizer

缺distance/time/device threshold、capture identity和short-click terminal outcome。

#### ED71-P1-35 [Open]：Context gesture未纳入tool/mode/capture arbitration

mode stack可消费通用input是底座，但没有Context intent/effect，不能与Orbit、modal、gizmo和plugin tool确定仲裁。

#### ED71-P1-36 [Partial]：current picking产品存在，Viewport Context不消费

renderer-visible picking/currentness已由Editor59加强；secondary链不查询它，也不生成target snapshot。

#### ED71-P1-37 [Open]：Outliner与Viewport没有共享动作图

Outliner生成fixed request，Viewport没有Context菜单；两者无共同catalog或availability。

#### ED71-P1-38 [Open]：没有Context Menu key/Shift+F10入口

keyboard入口、anchor选择、focused target和与mouse同语义行为均未实现。

#### ED71-P1-39 [Partial]：popup focus基础存在，focus return未绑定session

打开会设置focused/selected，关闭会清状态和anchor；没有invoker token、Escape/command后的精确focus恢复和accessible target description。

#### ED71-P1-40 [Partial]：active workbench window guard存在，session隔离缺失

host拒绝非active Workbench document的打开请求；没有window/view/document-qualified session，多个同类surface仍会共享固定控件状态。

### 6.5 P1：Extension、performance、diagnostics与qualification

#### ED71-P1-41 [Partial]：`EditorMenuItemDescriptor`增强但不足以表达Scene provider

operation、priority、shortcut、enabled和capabilities可保留；缺target predicate、selection policy、section/anchor、payload factory、reason与surface projection。

#### ED71-P1-42 [Partial]：owner ticket/generation存在，未进入menu session

Contribution store可revoke并保留immutable旧reader；已打开菜单不记录ticket/generation，也不会在revoke时失效。

#### ED71-P1-43 [Open]：没有Scene provider故障隔离、预算与降级

不存在可执行Scene provider registry，因而也没有panic/timeout/oversize隔离、built-in preservation或quarantine receipt。

#### ED71-P1-44 [Partial]：stable action/operation identity进步，排序与localization未统一

显式action id、operation path和priority已有底座；两个parser fallback、raw label、target label path、anchor conflict和localization key仍未收敛。

#### ED71-P1-45 [Partial]：generation/delta底座存在，没有compiled action catalog

extension snapshot与Hierarchy O(delta) projection可复用；每次Context仍直接分配raw Vec，未按provider/action/selection generation编译或缓存。

#### ED71-P1-46 [Partial]：局部测试增强，业务/lifecycle/fault/scale闭环缺失

Keep transaction、command/extension/popup/pointer基础有测试；没有指定Context action到world delta、undo、receipt一致性的E2E，也无100K/fault/soak。

### 6.6 P2

#### ED71-P2-01 [Open]：没有按surface/role的动作排序与可见性偏好

#### ED71-P2-02 [Open]：长菜单没有搜索/过滤与分层发现

#### ED71-P2-03 [Open]：没有Recent/Repeat Context Action

#### ED71-P2-04 [Open]：没有rich tooltip与结果预览

#### ED71-P2-05 [Open]：pen/touch long-press没有产品政策

#### ED71-P2-06 [Open]：没有可选radial/quick action surface

#### ED71-P2-07 [Open]：没有隐私受控的action usage telemetry

#### ED71-P2-08 [Open]：没有Context Action Inspector

#### ED71-P2-09 [Open]：没有contextual help/documentation action规范

#### ED71-P2-10 [Open]：没有跨平台菜单密度与native convention验收

## 7. 目标架构

### 7.1 Identity与snapshot

Editor Scene应提供`SceneContextActionService`。surface先将current pick/Hierarchy entity映射成`QualifiedSceneTarget { project, document, world_epoch, view, surface, entity, subobject, generation }`，再冻结`SceneContextTargetSnapshot`与ordered `SceneSelectionSnapshot`。snapshot必须包含selection revision、target relation、pointer facts、source surface和capability digest，不能含UI display path作为identity。

### 7.2 Provider registry与typed action definition

在Editor50 contribution store上增加owner-aware `SceneContextActionProviderRegistry`，provider返回versioned typed action definition：stable action id、canonical command id、localization key、icon token、section/slot/anchor/submenu、surface roles、selection policy、payload schema、visibility/enabled evaluator和成本声明。registration/revoke必须原子发布generation；重复id/anchor冲突确定拒绝。

### 7.3 Catalog compile与admission

按provider generation、document/world/view、target capability digest、selection revision和surface role编译immutable `SceneContextActionCatalog`。编译先做结构聚合，再用bounded evaluator生成visible/enabled/disabled reason；10K selection必须以summary/capability reduction工作，不能逐action逐target重复扫描。built-in catalog应在plugin fault时保持可用。

### 7.4 Per-open menu session

每次打开创建immutable `SceneContextMenuSession`，保存session id、target/selection snapshot、catalog generation、provider tickets、evaluation receipt、anchor、focus-return token与expiry。UI只能投影session生成的DTO；close、document/world/view replacement、Play/prefab/stage转换、provider revoke和shutdown必须产生terminal retirement。

### 7.5 Invoke与receipt

UI提交`SceneContextActionIntent { session_id, action_id, payload }`。service先验证session/catalog/owner/current target/selection/write/lock，再路由Editor08 canonical command或Editor55/63 transaction owner。每次调用必须生成typed terminal `SceneContextInvocationReceipt`，记录Executed/Rejected/Stale/Cancelled/Faulted、transaction id、world delta摘要、provider/action/command identity与可redact provenance。

### 7.6 Surface与input

Outliner和Viewport只负责产生qualified target和呈现同一catalog。Viewport secondary recognizer先在tool/mode/modal/capture owner内区分click与drag；short-click消费Editor59 current picking product，drag才进入Orbit。Keyboard Context Menu/Shift+F10使用focused/selected target并与mouse共享session；close/invoke后精确恢复focus。

### 7.7 Runtime边界

Context Menu、provider、selection policy和transaction orchestration全部留在Editor。Runtime只暴露中立Scene identity/currentness、object capability和picking facts，不依赖Editor UI/provider类型，不通过Runtime字符串命令反向控制Editor。

### 7.8 Performance、fault与diagnostics

catalog编译、admission与session应有item/depth/time/allocation预算；provider callback在锁外运行并具panic/timeout/oversize隔离。diagnostics记录source generations、target/selection counts、provider/action admission、cache hit/miss、stale/fault/retirement原因和invocation receipt，不记录未经授权的对象名称或payload。

## 8. 依赖有序重构里程碑

### ED71-M0：能力真实性止血

禁用或移除Open/Rename/Duplicate/Delete的no-op可用外观；只保留已接canonical command并能产生terminal receipt的动作。关闭popup时清理全部context属性并恢复focus。

### ED71-M1：Identity、Snapshot与Session内核

接入Document/World/View/Surface/Entity/Subobject qualified identity，复用Hierarchy entity map与selection revision，建立pure snapshot/session state machine及stale/retirement tests。

### ED71-M2：Typed Action Catalog与Admission

删除raw Scene menu item数组和双parser依赖，建立typed definition、section/anchor/submenu、localization、capability reduction、visible/enabled/reason和deterministic compile。

### ED71-M3：Canonical Command、Revalidation与Transaction

把Delete/Rename/Duplicate/Open等动作硬切到Editor08/55/63 owner；引入versioned payload、invoke-time revalidation和typed receipt，删除UI click即成功语义。

### ED71-M4：Outliner产品闭环

Outliner secondary/keyboard使用Hierarchy generation/entity/selection snapshot，完成未选中target policy、multi/mixed/root/inherited/clipboard规则、focus return与a11y。

### ED71-M5：Viewport Gesture与Picking闭环

建立secondary recognizer和terminal arbitration，short-click只消费current picking product，Outliner/Viewport共享catalog，验证Orbit不误启动。

### ED71-M6：Extension Provider与Fault Domain

在Contribution store上注册typed provider，绑定ticket/generation/revoke，增加lock-free callback、panic/timeout/oversize隔离、quarantine和built-in preservation。

### ED71-M7：Localization、Accessibility与跨平台体验

完成localization key、accessible role/name/state/reason、Context key/Shift+F10、focus restore、keyboard navigation及Windows/macOS/Linux密度与native convention。

### ED71-M8：规模、缓存与诊断

按generation增量编译catalog，建立10K selection/100K Outliner budget、bounded allocation、metrics/receipt导出和rapid open/close/revoke soak。

### ED71-M9：动态资格与跨引擎同语义基线

在Windows真实Editor完成business/lifecycle/fault/a11y/scale/profile矩阵，再以同场景、同动作、同selection规模、同硬件与Unreal/Godot/Fyrox进行可复现比较；没有测量前不宣称性能领先。

## 9. 资格门

| Gate | 要求 | 当前状态 |
|---|---|---|
| ED71-G01 | Scene context拥有唯一Editor authority | Fail |
| ED71-G02 | Snapshot含document/world/view/surface generation | Partial |
| ED71-G03 | Target使用qualified object/subobject handle | Fail |
| ED71-G04 | Selection snapshot immutable且带revision | Partial |
| ED71-G05 | duplicate/localized name不影响target identity | Fail |
| ED71-G06 | stale/deleted/reused target在invoke时拒绝 | Fail |
| ED71-G07 | world/document/view replacement会retire session | Fail |
| ED71-G08 | empty/viewport target携带bounded pointer facts | Fail |
| ED71-G09 | Action definition为typed stable schema | Fail |
| ED71-G10 | Section/slot/anchor/submenu可确定编译 | Fail |
| ED71-G11 | Label、localization key与action id分离 | Partial |
| ED71-G12 | Visible/enabled/disabled reason完整投影 | Partial |
| ED71-G13 | single/multi/mixed selection规则可测试 | Partial |
| ED71-G14 | empty surface动作集可测试 | Fail |
| ED71-G15 | read-only/locked authority双阶段校验 | Partial |
| ED71-G16 | root/inherited/instance限制有stable reason | Fail |
| ED71-G17 | clipboard/destination admission复用Editor55 | Fail |
| ED71-G18 | Context、keymap、menu、palette共享command id | Partial |
| ED71-G19 | Invocation payload有版本化typed schema | Partial |
| ED71-G20 | Command `WhenClause`与Scene capability组合 | Partial |
| ED71-G21 | Invoke执行currentness revalidation | Fail |
| ED71-G22 | 每次invoke产生typed terminal receipt | Fail |
| ED71-G23 | authoring修改产生transaction/undo/redo证据 | Partial |
| ED71-G24 | popup关闭不再冒充业务成功 | Fail |
| ED71-G25 | Delete context动作走canonical delete owner | Fail |
| ED71-G26 | Rename/Duplicate走各自canonical owner | Fail |
| ED71-G27 | Viewport secondary click/drag可确定分流 | Fail |
| ED71-G28 | Orbit在context short-click后不误启动 | Fail |
| ED71-G29 | Viewport context只消费current picking product | Partial |
| ED71-G30 | tool/mode/modal/capture仲裁有terminal结果 | Partial |
| ED71-G31 | Outliner/Viewport共享同一action graph | Fail |
| ED71-G32 | Context key/Shift+F10/focus return闭环 | Fail |
| ED71-G33 | accessible role/name/state/reason动态验证 | Partial |
| ED71-G34 | locale切换不改变action/target identity | Fail |
| ED71-G35 | 插件可注册typed Scene context provider | Fail |
| ED71-G36 | provider revoke/reload使旧session失效 | Partial |
| ED71-G37 | provider panic/timeout/oversize被隔离 | Fail |
| ED71-G38 | duplicate id/anchor冲突确定拒绝 | Partial |
| ED71-G39 | item/depth/time/allocation预算可配置验证 | Partial |
| ED71-G40 | catalog与admission按generation增量更新 | Partial |
| ED71-G41 | 10K selection不做actions × targets重复扫描 | Fail |
| ED71-G42 | 100K Outliner打开菜单满足预算 | Partial |
| ED71-G43 | multi-window/view/document session相互隔离 | Partial |
| ED71-G44 | close/reload/Play/prefab/shutdown retirement闭环 | Fail |
| ED71-G45 | malformed contribution fuzz不破坏built-ins | Fail |
| ED71-G46 | E2E证明动作、world delta、undo与receipt一致 | Partial |
| ED71-G47 | profile/soak无UI stall、泄漏或无界增长 | Fail |
| ED71-G48 | 同语义跨引擎benchmark有可复验证据 | Fail |

当前汇总为 **28 Fail / 20 Partial / 0 Pass**。Partial不能作为产品验收通过；所有门都必须由current source与动态证据同时闭合。

## 10. 测试与动态证据矩阵

| 层级 | 必须新增或强化的证据 | 当前状态 |
|---|---|---|
| Pure identity | duplicate/localized/renamed target、ID reuse、world replacement、subobject retirement | Missing |
| Snapshot/session | selected/unselected target政策、selection revision race、open/close/reopen、expiry | Missing |
| Catalog | section/anchor conflict、provider ordering、localization、single/multi/mixed、reason | Missing |
| Command | Delete/Rename/Duplicate/Open与menu/keymap/palette同command id | Partial |
| Transaction | Context action -> exact target delta -> undo/redo -> typed receipt | Partial |
| Extension | register/revoke/reload、panic/timeout/oversize、malformed fuzz、built-in preservation | Missing |
| Outliner | mouse/Context key/Shift+F10、focus return、root/inherited/clipboard、multi-document | Missing |
| Viewport | short-click/drag threshold、mode/modal/capture、current pick、Orbit non-start | Missing |
| Accessibility | role/name/state/reason、keyboard traversal、Escape/invoke focus restore | Partial |
| Lifecycle | close/reload/world replace/Play/prefab/provider unload/shutdown retirement | Missing |
| Scale | 10K selection、100K rows、1/4/16 provider、steady/reload allocation与p50/p95 | Missing |
| Qualification | Windows real Editor、fault/soak/profile、同硬件跨引擎同语义比较 | Missing |

现有73个focused tests只能证明局部底座，不能替代上述产品矩阵。本轮未执行测试；报告未改production code，因此没有把既有或未运行的Cargo结果写成current-source通过。

## 11. Owner路由与禁止重复实现

| 合同 | 唯一owner | Editor192消费方式 |
|---|---|---|
| Command descriptor/when/keymap/menu/palette/remote | Editor08/178 | action引用canonical command id并复用admission |
| Extension ticket/generation/revoke/capability | Editor50/171 | provider registry建立在Contribution store之上 |
| Clipboard/duplicate/delete/remap | Editor55/176 | Context动作只提交typed transfer/command intent |
| Viewport picking/selection/currentness | Editor59/180 | short-click读取current pick与selection snapshot |
| Hierarchy entity map/selection revision/Outliner | Editor60/181 | Outliner target解析使用现有generation projection |
| Document/World lifecycle | Editor61/182 | session identity、retirement与stale fence |
| Transaction/history/journal | Editor63/184 | authoring action通过统一transaction owner |
| Camera secondary navigation | Editor66/187 | recognizer终态为drag后才进入Orbit |
| Object visibility/selection eligibility | Editor70/191 | availability消费effective eligibility，不复制resolver |

禁止在ZUI raw strings、retained bridge、Viewport controller或plugin callback中新增第二套command switch、selection truth、transaction stack、extension generation、picking scan或world identity。禁止以display label、control id或字符串path充当Entity identity。禁止让disabled/no-op UI冒充功能完成。

## 12. 最终判定

Zircon当前拥有比Editor71时代更强的通用基础：explicit action id、immutable command eval、stable command metadata、extension ticket/generation、Hierarchy generation/entity map、current picking与真实Keep Play Changes transaction都应保留。但它们仍是彼此分离的基础设施，Scene Context Menu没有把它们组合成一个工程级产品。

实施优先级应为：先消除四个no-op action的能力真实性问题并清理popup旧context，再建立qualified target/selection snapshot和per-open session；随后硬切typed catalog、canonical command、invoke revalidation、transaction receipt，最后接Outliner/Viewport、extension fault domain、accessibility、lifecycle和scale qualification。未完成这些门之前，不能把“能弹出菜单”描述为Scene contextual action完成，也不能声称性能或表现优于Unreal。
