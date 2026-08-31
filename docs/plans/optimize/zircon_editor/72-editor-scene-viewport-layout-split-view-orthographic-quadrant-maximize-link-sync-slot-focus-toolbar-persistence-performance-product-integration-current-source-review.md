---
title: Editor Scene Viewport Layout、Split View、Orthographic Quadrant、Maximize、Link Sync、Slot Focus、Toolbar、Persistence、Performance 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor72
review_date: 2026-08-22
baseline_head: a922089697e41e07fa29e3e42a5e4c9afc1ae31b
baseline_epoch: 341
related_code:
  - zircon_editor/src/core/editor_event/workbench/layout_command.rs
  - zircon_editor/src/ui/workbench/layout/document_node.rs
  - zircon_editor/src/ui/workbench/layout/layout_command.rs
  - zircon_editor/src/ui/workbench/layout/manager/apply.rs
  - zircon_editor/src/ui/workbench/layout_preset.rs
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - zircon_editor/src/ui/workbench/view/view_descriptor.rs
  - zircon_editor/src/ui/workbench/view/view_descriptor_builder.rs
  - zircon_editor/src/ui/workbench/view/view_instance.rs
  - zircon_editor/src/ui/workbench/view/view_registry_open_descriptor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/scene_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/game_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_layout/ensure_shell_instances.rs
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/workbench/project/project_editor_workspace.rs
  - zircon_editor/src/ui/workbench/snapshot/workbench/resolve_document_workspace.rs
  - zircon_editor/src/ui/workbench/model/document_tabs/collect.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_content_selection.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute_viewport.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/viewport_surfaces.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs
  - zircon_editor/src/ui/retained_host/app/viewport_image_redraw.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/docked.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/pane_frame.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport/route_mapping.rs
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/chrome_projection.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/set_projection_mode_route.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/viewport_toolbar_pointer_route.rs
  - zircon_editor/src/ui/workbench/state/editor_state.rs
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_state.rs
  - zircon_editor/src/ui/retained_host/viewport/mod.rs
  - zircon_editor/src/ui/retained_host/viewport/poll_viewport_product.rs
  - zircon_editor/src/ui/retained_host/viewport/poll_captured_frame.rs
  - zircon_editor/src/ui/retained_host/viewport/submit_extract.rs
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_viewport_panel.zui
  - zircon_editor/assets/ui/editor/windows/workbench_window.zui
  - zircon_editor/assets/ui/editor/host/workbench_shell.zui
tests:
  - zircon_editor/src/tests/workbench/layout/split_creation.rs
  - zircon_editor/src/tests/workbench/layout/document_attachment.rs
  - zircon_editor/src/tests/workbench/layout/layout_preset_persistence.rs
  - zircon_editor/src/tests/workbench/layout/roundtrip_and_restore.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup/workspace_restore.rs
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
  - zircon_editor/src/tests/workbench/view_model/document_workspace.rs
  - zircon_editor/src/tests/host/pane_presentation/document_projection.rs
  - zircon_editor/src/tests/host/retained_viewport_toolbar_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/retained_viewport_toolbar_pointer/dispatch.rs
  - zircon_editor/src/tests/host/retained_window/native_viewport_image.rs
  - zircon_editor/src/ui/retained_host/ui/tests/scene_document_pane.rs
  - zircon_editor/src/ui/retained_host/viewport/tests/controller_polls_latest_captured_frame_from_render_framework.rs
  - zircon_editor/src/ui/retained_host/viewport/tests/controller_polls_latest_viewport_product_from_render_framework.rs
  - zircon_editor/src/ui/retained_host/viewport/tests/controller_submits_shared_ui_overlay_through_render_framework.rs
  - zircon_editor/src/tests/ui/boundary/workbench_projection_cutover/layout_frames.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_editor/54-editor-workbench-shell-autolayout-constraint-language-responsive-tier-region-binding-geometry-authority-product-integration-review.md
  - docs/plans/optimize/zircon_editor/58-editor-scene-viewport-host-render-target-surface-frame-currentness-multi-viewport-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/66-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/68-editor-scene-viewport-display-mode-lighting-skybox-show-flag-debug-visualization-overlay-composition-profile-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-preview-simulation-tick-audio-environment-world-isolation-recording-scrub-budget-persistence-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/EditorFramework/Public/EditorViewportLayout.h
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Public/LevelViewportLayout.h
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/LevelViewportLayout.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Public/LevelViewportLayoutEntity.h
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/LevelViewportLayoutEntity.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Public/LevelViewportTabContent.h
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/LevelViewportTabContent.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/AssetEditorViewportLayout.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/AssetEditorViewportLayout.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorViewportLayout2x2.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_plugin.h
  - dev/godot/editor/scene/3d/node_3d_editor_plugin.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.h
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.cpp
  - dev/Fyrox/editor/src/scene_viewer/mod.rs
  - dev/Fyrox/editor/src/settings/scene.rs
  - dev/Fyrox/editor/src/settings/windows.rs
  - dev/Fyrox/editor/src/camera/mod.rs
  - dev/Fyrox/editor/src/scene/mod.rs
  - dev/Fyrox/editor/src/lib.rs
  - dev/Fyrox/fyrox-ui/src/dock/config.rs
  - dev/Fyrox/fyrox-ui/src/dock/mod.rs
  - dev/bevy/examples/3d/split_screen.rs
  - dev/bevy/crates/bevy_camera/src/camera.rs
  - dev/bevy/crates/bevy_render/src/camera.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/RenderPipeline/Camera/HDAdditionalSceneViewSettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/SceneViewDrawMode.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Textures/RTHandleSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/BufferedRTHandleSystemTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Tests/Editor/CameraSettingsUtilitiesTests.cs
doc_type: current_source_review
canonical_owner: docs/plans/optimize/zircon_editor/72-editor-scene-viewport-layout-split-view-orthographic-quadrant-maximize-link-sync-slot-focus-toolbar-persistence-performance-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Scene Viewport Layout、Split View、Orthographic Quadrant、Maximize、Link Sync、Slot Focus、Toolbar、Persistence、Performance 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon已经有可保留的通用Workbench split-tree和窗口/抽屉布局底座。`DocumentNode::SplitNode`表达递归轴向分割和比例，`CreateSplit`、`ResizeSplit`、tab attach/detach及project workspace序列化也是真实代码；Retained Host还能给docked/floating pane生成surface key和toolbar frame。这些事实说明当前问题不是完全没有“分割数据结构”，而是该结构没有被编译成多视口产品。

当前产品投影递归遍历split tree后，把所有leaf tab压平成一份`document_tabs`，再用“第一个active，否则第一个tab”选出全局唯一`document_pane`。Retained Host只维护一个`viewport_toolbar_frame`、一个`viewport_content_frame`、一个pointer bridge frame、一个`viewport_size`和一条render submission/poll链。多个leaf可以各有active tab，但产品选择器没有cell identity或focus authority；布局命令可以成功改变并持久化model，而用户仍只能看到一个pane。这个model/product分裂会把移入非首leaf的文档变成不可达内容，属于当前可达的P0能力与工作区完整性问题。

Scene和Game descriptor都沿用`multi_instance = false`，opened instance payload为`Null`；`EditorState`只有一个`SceneViewportController`和一份全局`SceneViewportSettings`。Toolbar pointer route虽然携带`surface_key`，dispatch映射却把它丢弃并直接修改全局controller/chrome。也就是说，即使floating或未来split surface命中自己的toolbar，操作仍可能落到另一个可见pane代表的全局viewport状态。这是第二项当前可达的wrong-target P0，而不是“以后多视口实现时再考虑”的增强项。

工程级实现不能继续在通用layout tree上增加几个菜单字符串或复制四份controller字段。正确边界是：Editor72拥有viewport arrangement product和稳定`ViewportSlotId`；Editor58提供每槽`ViewportSessionId`、surface/render product/currentness；Editor66提供每槽camera session；Editor68/69提供每槽visualization与frame demand。布局预设必须先编译、验证和预算准入，再原子提交slot-to-session、geometry、focus、toolbar、input和render bindings。Maximize只是保留原会话的可见性/placement状态，不得销毁其camera或visual state；link-sync必须有leader、epoch、origin与投影兼容政策，不能互相回写形成环。

本报告新增 **2项P0、48项P1、10项P2**，登记 **48个全部Fail的资格门**。Editor13的通用布局事务/恢复、Editor54的shell geometry、Editor58的multi-view render session/currentness、Editor59的input/capture、Editor66的单视口camera、Editor68的单视口visualization和Editor69的preview cadence仍由各自父报告唯一计数。

本轮是review-only：未修改production Rust/ZUI，未运行Cargo、真实Editor、GUI/GPU、native input、save/reopen、plugin reload、fault/soak/profile或同语义跨引擎benchmark；tooling按用户要求排除。

## 2. 审查边界、currentness与冻结语料

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test declarations | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Zircon layout/model/identity | **21 / 2,665 / 2,442 / 92,186 / 6** | split tree、command、preset、descriptor、instance、workspace、controller/settings | `5592813bc6e6f518a2ba55467fadca708d03d3354881940e1e82d6030df146bc` |
| Zircon product/projection | **24 / 4,840 / 4,466 / 213,864 / 8** | flattened tabs、single pane/frame/product、toolbar route、render poll与ZUI | `4ed6121c9d5b1e4a59a1160a82ce0658b9b6b4033660c9759a42ef72e6c58ff0` |
| Zircon focused tests | **16 / 2,879 / 2,647 / 103,996 / 48** | split model、restore、pane projection、toolbar surface与single product | `6cf61dac25ad8d00aa093ee8f2e6105f2a1b955397b65aad3d0ab288b3f72070` |
| Zircon deduplicated set | **61 / 10,384 / 9,555 / 410,046 / 62** | 三组按规范化路径去重 | `3528a18abcadc313592770594266d8cb2ba3e2aa6b774661024e3dfe633b08ca` |
| Unreal selected set | **10 / 2,039 / 1,670 / 77,818 / 0** | layout config/entity、2x2 assignment、maximized replacement与config persistence | `37185b647dba167b74af41332f2d4ac39522a1a4e4021fb25d016f3c600379fb` |
| Godot selected set | **4 / 12,586 / 10,703 / 500,558 / 0** | 四个真实viewport、1/2/3/4模式、split state、per-slot state与maximize | `b66b3f65e7520ce677e60530691b29a61930baf0ca3c59ef893752a2d85987b9` |
| Fyrox selected set | **8 / 7,407 / 6,734 / 283,136 / 1** | 单SceneViewer、per-scene camera persistence与通用DockingManager | `ece4e97fa814d42318c4ede58dab05d97232007c303d5a8cf79148f62fba8cee` |
| Bevy selected set | **3 / 2,514 / 2,332 / 100,225 / 3** | 四camera physical viewport、per-camera UI、DPI/resize/clamp/projection update | `2dd6dc3ca754fa9bb7146160ecaa1a91a243c0d1912099ed1cb8989936565b28` |
| Unity Graphics selected set | **5 / 1,988 / 1,788 / 93,150 / 6** | per-SceneView additional settings、draw-mode hook、RTHandle viewport/history与tests | `4ac6d92c87550a723e7ebe385892d1084c3b9af874f1168ff67e2dc953918e88` |
| Five-engine deduplicated set | **30 / 26,534 / 23,227 / 1,054,887 / 10** | 五类本地参考按路径去重 | `853565c2a3927d7a90a910e69f3b7060a121d2776bec1e696d430028973f0fe6` |

指标是本轮工作树字节与文本物理统计，不是功能覆盖率。fingerprint按排序后的`path + file SHA-256`清单计算。相关Editor源码仍由其他会话演进，后续实现前必须重取61文件终态并复核父owner。

### 2.2 范围与非范围

本报告拥有Scene Viewport arrangement：1/2/3/4布局预设、稳定cell/slot identity、slot-to-session binding、split ratio、maximize/restore、perspective/orthographic slot assignment、per-slot focus/toolbar、camera link-sync group、viewport布局schema/migration、可见性/admission/render budget及其产品/a11y/performance资格。

Editor13继续拥有通用Workbench profile、dock/tab/window restore与布局事务；Editor54拥有通用shell constraint/geometry；Editor58拥有`ViewportSession`、surface/render target/frame currentness和multi-viewport host；Editor59拥有pointer/capture/picking；Editor66拥有单个viewport的camera navigation/projection/bookmark/pilot；Editor68拥有单个viewport的show flag/view mode/profile；Editor69拥有单个viewport的preview time/frame demand。Editor72只能编排这些合同，不能复制其内部实现。

### 2.3 在途修改隔离

审查时工作树高度dirty，`docs/plans/optimize`、Runtime UI与若干Editor文件有其他会话改动。本报告没有回退、吸收或归因这些变更；冻结统计针对读取时的working-tree内容。报告与三个索引在写前取得协调器精确租约，production源码保持只读。

## 3. 当前实现拓扑与可保留基础

### 3.1 通用split tree是真实底座

`DocumentNode`支持递归`SplitNode { axis, ratio, first, second }`与`Tabs`，`CreateSplit`会把目标节点包装成0.5比例的新split，`ResizeSplit`把比例限制在0.1到0.9。Project workspace可序列化完整`WorkbenchLayout`，所以拓扑模型并非占位符。这一层应保留为通用layout source，但它不是viewport layout descriptor，也没有slot语义。

### 3.2 通用preset会有损压缩center layout

`LayoutPreset`只把center捕获为`SingleDocument`或`Split { axis, panes }`。它不保存递归拓扑、精确split ratio、每个leaf的tab/active、viewport slot identity或per-slot payload；restore按简化形状重建。Project workspace与layout preset是两条不同持久化路径，前者能保留通用tree，后者会有损降级，不能混称“多视口状态已经持久化”。

### 3.3 Scene/Game没有多实例和per-slot payload

`ViewDescriptor::new()`默认`multi_instance = false`，Scene和Game descriptor没有覆盖。`view_registry_open_descriptor`创建的`ViewInstance` payload为`Value::Null`，builtin default只打开`editor.scene#1`和`editor.game#1`并放入一个tab stack。`SceneViewportSettings`虽然可序列化，却没有绑定到具体ViewInstance或slot。

### 3.4 split snapshot在产品投影中被压平

`resolve_document_workspace`保留split node，但`document_tabs::collect`递归收集所有leaf到一份线性列表，只留下`workspace_path`作为来源路径。`document_pane_selection`随后取第一个active或第一个tab。每个leaf都允许自己的active tab，因此“active”不是全局唯一，线性first-match也不代表用户最近点击或键盘焦点。

### 3.5 Retained Host只装配一个document pane

`scene_projection`和`pane_projection`只构建一个`document_pane`。Host data、bootstrap和shell asset都只有一份document tabs strip、viewport toolbar与viewport body；Scene消费全局`chrome.scene_viewport_settings`，Game则得到blank viewport chrome，但仍可能显示来自全局viewport尺寸的metadata。没有cell collection、cell clipping、divider、per-cell overlay或per-cell accessibility node。

### 3.6 Resize、pointer与render都是全局单实例

`recompute_viewport`推导一个`viewport_content_frame`并更新一个`viewport_size`，只发一次`EditorViewportEvent::Resized`并同步一个pointer bridge frame。Render submission调用一次runtime frame submission和一次viewport extract；image redraw只poll一个product/capture，再把它交给一个surface host context。四个cell无法仅靠复制layout node自动获得四条render/currentness链。

### 3.7 Toolbar surface identity在dispatch处丢失

Toolbar frame和`ViewportToolbarPointerRoute`携带`surface_key`，说明UI层已经意识到docked/floating surface identity。但`callback_dispatch/viewport/route_mapping.rs`匹配route时忽略该字段，读取全局chrome并发全局viewport command。该接口表面是per-surface，实际执行仍是singleton，属于必须先止血的wrong-target seam。

### 3.8 测试只证明model和single product

现有测试覆盖split创建、tab attach、workspace/preset roundtrip、单document pane、toolbar surface frame和单viewport product poll。没有测试证明两个leaf同时可见，更没有2x2四个独立camera/product/input/toolbar、maximize/restore、link-sync、schema migration、fault、fair scheduling、GPU预算或无障碍闭环。

## 4. 五引擎参考证据与适用边界

### 4.1 Unreal：稳定entity、固定布局族与非破坏maximize

Unreal注册一、二、三、四pane的多种命令，每个cell由带稳定ConfigKey的`IEditorViewportLayoutEntity`承载，可独立保存widget、focus、viewport type与config。2x2明确分配Top、Front、Perspective、Right，Perspective默认realtime而orthographic默认非realtime；split百分比、layout type、per-cell type/settings和maximized target均写入config。

Layout切换构造新的layout object再替换，不在旧tree上逐步破坏。Maximize把选中entity的既有widget临时替换到overlay，其他entity通过可见集合隐藏；restore归还widget与键盘focus，PIE场景还重新注册viewport并处理capture。Zircon应借鉴identity、state transfer和atomic replacement，不能照搬Unreal的全局editor singleton与legacy config string。

### 4.2 Godot：四个常驻viewport与最小可接受状态量

Godot固定持有四个真实`Node3DEditorViewport`，提供1、2、2-alt、3、3-alt、4等模式和快捷键。Container在模式切换时show/hide现有viewport并重排split container，保存/恢复横纵offset；`get_state()`记录当前布局、split state和四份viewport state，restore逐槽恢复camera position/rotation/distance、orthographic/view type、display/environment/listener/gizmo/grid/frame time/half-resolution/camera preview等。

Maximize同样不销毁slot：隐藏其余viewport并伸展当前viewport，restore重新应用原布局；`last_used_viewport`由点击更新，作为操作目标。Godot的架构不如Unreal可扩展，但它清楚划定了“多视口产品”的最低线：多个常驻实例、每槽状态、active slot和可逆布局切换。

### 4.3 Fyrox：通用Docking和per-scene camera仍不是多视口

Fyrox创建一个`SceneViewer` window，内部只有一张frame image、一个projection dropdown和一套toolbar；每个`GameScene`持有一个`CameraController`。Camera position/yaw/pitch/projection按scene path持久化，通用`DockingManagerLayoutDescriptor`保存窗口布局与splitter。这比Zircon把viewport payload留空更完整，但仍不是1/2/3/4多槽产品，不能作为目标上限。

### 4.4 Bevy：底层四camera viewport与resize语义

Bevy `split_screen`示例创建四个独立camera entity，为每个camera绑定自己的UI target和按钮，通过`CameraPosition`计算物理position/size；窗口resize时逐camera更新`Viewport`。Camera实现会在DPI变化时同时缩放position/size、对目标尺寸clamp，并用每个viewport逻辑尺寸更新projection；unit tests覆盖orthographic/perspective viewport-to-world。

这些证据只说明Zircon底层必须支持per-cell camera/rect/UI target和DPI安全，不说明Bevy已有Editor layout、maximize或持久化产品。

### 4.5 Unity Graphics：仅作为per-view渲染状态与RT历史约束

本地Graphics仓不包含Unity Editor核心SceneView布局源码，因此不能据此声明Unity的split preset行为。可用证据是HDRP为具体`SceneView`创建并绑定additional camera data，draw-mode hook遍历多个`SceneView`实例；RTHandle系统区分current/previous viewport和render-target尺寸、保存scale history，并在Editor中回收曾被临时大分辨率推高的资源上限。Tests覆盖buffer history轮转与camera settings应用。

Zircon应借鉴每槽settings owner、current/previous尺寸和资源回落策略；layout identity、focus和maximize仍以Unreal/Godot为主参考。

## 5. 差异矩阵

| 维度 | 当前Zircon | 工程级目标 | 主要参考 |
|---|---|---|---|
| 布局模型 | 通用递归split tree | versioned viewport layout descriptor + compiled slot graph | Unreal / Godot |
| Cell identity | leaf path与ViewInstance混用，无slot id | stable `ViewportSlotId`，与layout path/session id分离 | Unreal |
| 实例 | Scene/Game单实例、Null payload | 每slot独立session/camera/visual/runtime state | Unreal / Godot |
| 产品投影 | flatten后只选一个document pane | 同generation发布全部visible cells/frames/toolbars | Unreal / Godot |
| Focus/input | first active、单pointer frame | last-used/focused slot、qualified capture和keyboard traversal | Godot / Unreal |
| Maximize | 无 | 非破坏overlay/visibility state + focus/capture restore | Unreal / Godot |
| Link-sync | 无 | leader/epoch/origin、兼容策略、循环抑制与诊断 | 工程目标 |
| Persistence | 通用workspace v1；preset有损 | viewport schema/migration/LKG/per-slot payload | Unreal / Godot |
| Render budget | 单global viewport submission | visible/admitted slots、fair demand、hidden throttle、RT回落 | Unreal / Unity / Bevy |
| Test/资格 | model和single product | 2-4 cell E2E、fault、scale、a11y、GPU/profile | 五引擎综合 |

## 6. Findings

### 6.1 P0

#### ED72-P0-01：已接受并持久化的split layout在产品投影中丢失leaf可见性

通用layout command可以创建split、把tab移动到新leaf并保存workspace；snapshot也保留这棵tree。但后续投影把leaf压平，只显示第一个active/first tab和唯一document pane。用户可能成功执行布局操作，却无法看到或访问移入非首leaf的文档。必须在继续开放split/preset入口前添加RED E2E，证明每个leaf都有产品cell，或在未实现前显式拒绝该能力。

#### ED72-P0-02：Toolbar的surface identity被丢弃，操作可落到错误的全局viewport状态

pointer route携带docked/floating `surface_key`，dispatch却忽略它并操作全局`SceneViewportController`与`scene_viewport_settings`。当Scene pane出现在不同surface，或split/floating命中与全局selection不同步时，Projection/Grid/Display/Align等命令没有可验证目标。必须把route绑定`ViewportSlotId + ViewportSessionId + generation`并在执行前重校验；不能用“当前全局Scene”代替。

### 6.2 P1

#### ED72-P1-01：没有versioned viewport layout descriptor registry

1/2/3/4模式、stable id、display/localization key、slot schema、默认比例、capability和owner均无权威目录。

#### ED72-P1-02：Scene与Game descriptor仍是single-instance

两者未启用`multi_instance`，registry政策从入口就阻止四个Scene view session成为独立实例。

#### ED72-P1-03：没有稳定`ViewportSlotId`

ViewInstance、document leaf path、pane key、surface key和camera/controller没有共同且可持久化的slot identity。

#### ED72-P1-04：`workspace_path`只能定位tree，不能充当slot identity

split重排、tab drag、preset切换会改变路径；用路径持久化per-slot state会把状态错绑到新cell。

#### ED72-P1-05：ViewInstance payload为Null

camera、projection、visual profile、realtime、preview source、sync group或per-slot preference都没有实例级序列化载体。

#### ED72-P1-06：没有typed slot view kind和canonical camera preset

Perspective/Top/Front/Side/Bottom等只存在单viewport projection命令，不是布局slot的声明和恢复合同。

#### ED72-P1-07：没有primary/listener/source policy

多槽时哪一槽拥有audio listener、preview control、selection framing、play pilot或primary metrics没有确定规则。

#### ED72-P1-08：布局和slot没有extension capability与unavailable状态

插件不能注册自定义layout/slot kind；owner缺失时也没有保留位置、说明原因或安全降级。

#### ED72-P1-09：没有layout compiler与全量validation

不存在从descriptor/source tree编译stable slots、验证唯一性/比例/能力/预算并产出immutable artifact的阶段。

#### ED72-P1-10：没有per-cell pane presentation model

Host data只有单`document_pane`，无法原子承载多个cell的tab、pane、toolbar、HUD和empty/unavailable状态。

#### ED72-P1-11：没有per-cell content geometry

只有一个`viewport_content_frame`和size；没有cell clip、divider、content inset、DPI transform和local-to-global mapping。

#### ED72-P1-12：没有per-cell toolbar/HUD model

全局chrome设置被复用，无法表达不同slot的projection、view mode、realtime、camera speed、warning和currentness。

#### ED72-P1-13：没有per-cell pointer route frame

pointer bridge只同步一个frame，无法在重叠floating/maximized/split场景下确定命中、capture owner和local coordinate。

#### ED72-P1-14：没有slot-to-render-product binding

Render submission/poll只处理一个controller/product，缺`slot -> viewport session -> render target/surface`映射。

#### ED72-P1-15：没有per-cell currentness receipt

UI不能证明某张图属于哪个layout revision、slot、camera/visual generation、world revision和requested size。

#### ED72-P1-16：没有per-cell accessibility subtree

屏幕阅读器与自动化无法识别四个viewport、其名字/状态、divider、focused/maximized或不可用原因。

#### ED72-P1-17：没有显式cell focus authority

通用`focused_view`不能区分同一ViewInstance类型的多个slot，也不能表达active tab、focused cell与keyboard target的关系。

#### ED72-P1-18：没有键盘slot遍历

缺Next/Previous viewport、按布局空间顺序移动focus、直接选择slot以及冲突keymap政策。

#### ED72-P1-19：重建布局后没有focus return合同

切换preset、maximize/restore、floating attach/detach后，keyboard focus可能落回shell或错误pane。

#### ED72-P1-20：没有capture ownership handoff/cancel

camera drag、gizmo、splitter和toolbar在cell隐藏、maximize、layout切换、window loss-focus时没有统一terminal disposition。

#### ED72-P1-21：没有splitter与viewport gesture仲裁

divider drag区域、resize cursor、threshold、capture和相邻viewport input没有同一命中/优先级合同。

#### ED72-P1-22：没有per-window last-used slot

Godot级别的最小active target都缺失；menu/shortcut/frame-selection只能猜全局当前Scene。

#### ED72-P1-23：没有focus可视反馈和状态说明

产品资产没有稳定active-cell border、slot label、projection badge、sync/paused/stale状态及无障碍等价表达。

#### ED72-P1-24：没有1/2/3/4 viewport preset产品

Toolbar和Layout menu没有可用的single、horizontal/vertical two、three variants、four/2x2命令与状态投影。

#### ED72-P1-25：没有canonical orthographic quadrant assignment

2x2不能确定Top/Front/Side/Perspective默认槽位、axis/up direction、realtime和grid/workplane政策。

#### ED72-P1-26：没有maximize状态机

缺normal/maximizing/maximized/restoring、target slot、source layout revision、animation/reduced-motion和terminal receipt。

#### ED72-P1-27：没有可逆restore上下文

未保存原split比例、visible set、focus、capture、hover、toolbar popup、render demand和window placement。

#### ED72-P1-28：layout switch/maximize命令不可序列化重入

没有prepare/commit期间的queue/coalesce/reject策略，双击、快捷键和restore race可能产生半状态。

#### ED72-P1-29：没有floating/fullscreen/immersive语义

最大化cell、最大化native window、fullscreen与floating pane之间没有明确边界、PIE政策或monitor/DPI恢复。

#### ED72-P1-30：没有responsive min-size与collapse policy

小窗口、窄cell、长本地化文本和高DPI下何时隐藏toolbar item、叠加label或拒绝布局没有规则。

#### ED72-P1-31：没有camera link-sync group descriptor

缺stable group id、members、leader/follower、enabled mode、owner、scope和persistence schema。

#### ED72-P1-32：没有sync epoch/origin与循环抑制

多向监听若直接互相写camera会形成反馈环；当前没有event origin、generation fence、debounce/coalesce和stale reject。

#### ED72-P1-33：没有projection-aware同步政策

Perspective pose/pivot与orthographic center/zoom不能盲拷贝；旋转、distance、FOV、ortho size和clip plane需typed policy。

#### ED72-P1-34：没有source/world兼容性拒绝

不同document/world/preview source、pilot camera或Play session之间是否可link没有admission和stable reason。

#### ED72-P1-35：没有sync降级、解除和诊断

member销毁、camera拒绝、plugin revoke、world replacement或budget降级时，group没有terminal状态与用户反馈。

#### ED72-P1-36：没有扩展型sync provider边界

正交CAD式联动、comparison view或专业工具不能贡献受能力约束的同步策略，也没有panic/timeout隔离。

#### ED72-P1-37：workspace `layout_version = 1`不是viewport schema/migration

没有独立viewport layout version、slot payload版本、逐版本迁移、future-version拒绝和兼容报告。

#### ED72-P1-38：没有per-slot持久化与scope政策

camera/visual/realtime/sync/focus/maximized state应按user/project/document分层，目前没有owner、override或冲突合并。

#### ED72-P1-39：通用layout preset有损丢弃精确状态

`CenterSplitLayout`只保留axis和pane count，不 round-trip递归比例、tab assignment、active/focus和viewport payload。

#### ED72-P1-40：没有last-known-good/quarantine/recovery

损坏、unknown plugin slot、非法ratio、monitor变化或迁移失败时，没有保留可恢复原文、诊断和安全fallback。

#### ED72-P1-41：没有原子layout activation transaction

正确流程应prepare descriptor、resolve sessions、admit budget、warm required products、commit generation并保留旧LKG；当前没有receipt。

#### ED72-P1-42：没有owner revoke/hot-reload retirement

插件layout/slot/provider若卸载，旧slot、toolbar、sync member、render demand和payload如何退休或placeholder化没有合同。

#### ED72-P1-43：没有per-slot frame demand/realtime policy

Perspective active cell、orthographic静态cell、hidden/maximized-away cell和camera-linked follower仍没有不同tick/render触发策略。

#### ED72-P1-44：没有GPU/CPU/RT memory admission

四槽分辨率、MSAA、visual mode、capture/history和preview world可能突破预算；当前无预估、降级、拒绝或quality ladder。

#### ED72-P1-45：没有可见性节流与公平调度

hidden/occluded/minimized/后台window应停止或降频，多个visible slot需避免active cell饥饿和后台slot垄断。

#### ED72-P1-46：没有布局级telemetry与Inspector

缺layout/slot/session/currentness、requested/effective size、frame demand、GPU bytes、drop/stale、focus/capture与sync trace。

#### ED72-P1-47：没有multi-cell动态资格

缺2/3/4 visible products、independent input/camera/toolbars、switch/maximize、save/reopen、fault、scale、soak和profile tests。

#### ED72-P1-48：没有完整产品、无障碍与本地化验收

菜单仍disabled或缺入口，缺tooltip/shortcut/disabled reason、keyboard/screen reader、RTL/long text、HiDPI/multi-monitor与视觉基线。

### 6.3 P2

#### ED72-P2-01：split ratio使用裸`f32`

通用resize会clamp，但持久化/反序列化边界未证明拒绝NaN/Inf和异常精度；viewport compiler应使用validated ratio type。

#### ED72-P2-02：surface key仍是重复字符串

字符串在toolbar/frame/callback间传递，缺newtype、generation与owner，增加错路由和拼写漂移风险。

#### ED72-P2-03：toolbar高度存在硬编码常量

28px等固定尺寸没有token、DPI、density和本地化约束，四槽紧凑模式会放大浪费或裁切。

#### ED72-P2-04：slot/title/projection label未完全localization-key化

`Scene`、`Game`、`Perspective`、`Lit`等字符串同时承担显示与状态识别风险，应分离stable id和文案。

#### ED72-P2-05：Game pane使用blank viewport chrome

同一metadata链又可能报告全局viewport尺寸，容易制造“Game是独立surface”的错觉，应有typed unavailable/currentness状态。

#### ED72-P2-06：Layout菜单暴露disabled静态项

Default/Gameplay/Rendering等视觉入口没有与真实descriptor/availability绑定，产品真实性较弱。

#### ED72-P2-07：没有紧凑toolbar与overflow规范

多槽时应按priority折叠熟悉图标、保留tooltip与可达菜单，而不是简单缩小文字或裁切。

#### ED72-P2-08：没有确定性layout/sync trace replay

焦点、resize、switch、maximize和sync race难以仅靠日志复现，需要bounded event record与离线重放。

#### ED72-P2-09：没有schema fuzz与golden corpus

ratio/tree/plugin slot/version/unknown field/duplicate id等输入缺fuzz、migration golden和round-trip守恒测试。

#### ED72-P2-10：没有同语义跨引擎性能阈值

目前没有用相同场景、分辨率、cell数、view mode与硬件记录switch latency、first-current frame、CPU/GPU/VRAM和交互延迟。

## 7. 目标架构

### 7.1 权威对象

| 对象 | Owner | 必须包含 |
|---|---|---|
| `ViewportLayoutDescriptorRegistry` | Editor72 | stable layout id/version/owner/slots/tree/defaults/capabilities/localization |
| `CompiledViewportLayout` | Editor72 | validated slot graph、ratios、visibility、ordering、focus path、budget estimate |
| `SceneViewportLayoutSession` | Editor72 | layout revision、slot bindings、focused/last-used/maximized slot、sync groups、LKG |
| `ViewportSlotSession` | Editor72编排，能力由父owner提供 | slot id、ViewInstance、Editor58 session、Editor66 camera、Editor68/69 profile/demand、generation |
| `ViewportLayoutActivationReceipt` | Editor72 | expected/effective generation、admission、warm/current slots、fallback、diagnostics |
| `ViewportCameraSyncGroup` | Editor72 + Editor66 adapter | members、leader、mode、epoch/origin、compatibility、terminal state |

### 7.2 编译与激活流程

1. 从builtin/plugin descriptor或持久化document解析layout source，保留原文和schema版本。
2. 编译stable slot graph，验证id唯一、tree全覆盖、ratio有限、slot capability和view kind。
3. 为每个slot解析或创建`ViewportSessionId`，绑定camera/visual/preview state，不直接构造第二套controller。
4. 计算cell geometry和requested render sizes，执行CPU/GPU/RT memory/frame-demand admission。
5. 对需要首帧current的slot预热render product；失败时保持旧LKG，不发布半布局。
6. 同一generation原子发布pane/toolbar/geometry/hit/focus/render binding和activation receipt。
7. 退休旧binding、capture、sync membership和不可复用session；保留可恢复payload与诊断。

### 7.3 Maximize与restore

Maximize应改变`visible set + placement`，而不是销毁slot。进入时冻结source layout revision、focused slot、capture/tool state和split ratios；对active interaction先要求commit/cancel/transfer terminal receipt，再把目标slot投影到overlay或expanded cell。Restore只接受匹配generation的一次性token，恢复原visible set、focus和render demand；world/document/window已替换时走明确降级，不复活陈旧session。

### 7.4 Camera link-sync

Sync事件必须携带`group_id + epoch + origin_slot + source_camera_generation + typed delta`。Group resolver只向兼容member应用一次，member回报不得以新origin再次广播。Perspective到Perspective可同步pose/pivot；Orthographic之间可同步center/axis-aware zoom；跨投影默认只同步显式允许的pivot/selection frame，不复制FOV或rotation。Pilot/Play/不同world默认拒绝并给stable reason。

### 7.5 性能政策

Maximized/active cell优先获得interactive budget；可见orthographic静态cell以camera/world/visual invalidation触发，linked follower可coalesce；hidden/occluded/minimized slot停止presentation并按父owner政策释放或降级history/RT。Allocator记录requested/effective size、MSAA、history和bytes，临时大尺寸后按Unity式回落上限。四槽调度必须有公平上限，不能让背景preview抢占active manipulation帧。

## 8. 分层里程碑

### ED72-M0：Truthfulness止血与RED guards

- 固化split accepted但single pane、surface key丢失的两项P0 RED tests。
- 未实现多cell前拒绝或隐藏会制造不可达leaf的产品入口。
- 禁止viewport toolbar dispatch忽略surface identity。

### ED72-M1：Schema、Descriptor与Stable Slot Identity

- 建立layout/slot newtype、versioned descriptor、registry、capability和localization schema。
- 编译并验证tree、ratio、slot defaults、owner与unknown/unavailable状态。
- 增加migration golden、fuzz和LKG/quarantine。

### ED72-M2：Per-slot Session Binding

- 依赖Editor58建立slot-to-ViewportSession映射。
- 依赖Editor66/68/69绑定camera、visualization、preview demand，不复制父状态。
- Scene/Game multi-instance和ViewInstance payload完成硬切。

### ED72-M3：Multi-cell Product Projection

- 用compiled cell collection替换flatten + single document pane路径。
- 同generation发布pane/toolbar/HUD/geometry/hit/a11y/render bindings。
- 覆盖docked、floating、HiDPI、min-size与responsive overflow。

### ED72-M4：Focus、Input与Splitter

- 建立focused/last-used slot、keyboard traversal、visual focus和screen reader状态。
- pointer/capture按slot和generation路由，divider与viewport gesture统一仲裁。
- layout/window loss/rebuild产生terminal capture disposition。

### ED72-M5：Preset切换与Atomic Activation

- 实现1/2/3/4及2x2 canonical presets和orthographic assignment。
- prepare/validate/admit/warm/commit，失败保留旧LKG。
- switch request具备queue/coalesce/reject和typed receipt。

### ED72-M6：Maximize、Restore与Immersive

- 实现非破坏maximize overlay/visibility状态机。
- 完成focus/capture/tool/render-demand保存与恢复。
- 明确native maximize、fullscreen、floating、PIE与multi-monitor政策。

### ED72-M7：Camera Link-sync

- 建立group descriptor、leader/member、epoch/origin和projection-aware delta。
- 接入Editor66 current camera session和generation revalidation。
- 完成member revoke/world replacement/failure降级与诊断。

### ED72-M8：Persistence、Extension与Product

- 完成user/project/document scope、per-slot payload、migration与save/reopen。
- 支持owner-aware custom layout/slot/sync provider及reload retirement。
- 完成真实菜单、图标、tooltip、shortcut、disabled reason、本地化和a11y。

### ED72-M9：Budget、Fault、Scale与跨引擎资格

- 完成1/2/3/4 cell CPU/GPU/VRAM、RT回落、fair demand和hidden throttle。
- 执行fault/soak/profile、多window/monitor/DPI、save/reopen/plugin reload矩阵。
- 与Unreal/Godot建立同语义可复现benchmark，48门全Pass后才提升实现状态。

## 9. 资格门

| Gate | 要求 | 当前 |
|---|---|---|
| ED72-G01 | viewport layout拥有唯一Editor authority | Fail |
| ED72-G02 | layout descriptor有stable id、version、owner与capability | Fail |
| ED72-G03 | 每个cell有stable `ViewportSlotId` | Fail |
| ED72-G04 | slot id不依赖tree path、title或显示文本 | Fail |
| ED72-G05 | Scene/Game支持受策略约束的multi-instance | Fail |
| ED72-G06 | slot绑定generation-qualified ViewInstance与ViewportSession | Fail |
| ED72-G07 | per-slot camera/visual/preview状态只由父owner提供 | Fail |
| ED72-G08 | descriptor compile拒绝duplicate/invalid/unsupported source | Fail |
| ED72-G09 | 1/2/3/4及2x2布局有真实产品命令 | Fail |
| ED72-G10 | 每个visible leaf投影独立pane与content frame | Fail |
| ED72-G11 | 每个slot有独立toolbar/HUD/currentness | Fail |
| ED72-G12 | 每个slot有独立pointer local mapping与clip | Fail |
| ED72-G13 | 每个slot绑定独立render product/surface | Fail |
| ED72-G14 | orthographic quadrant assignment确定且可恢复 | Fail |
| ED72-G15 | ratio/tree/min-size在DPI与resize下保持有效 | Fail |
| ED72-G16 | layout generation原子覆盖model、UI、input与render | Fail |
| ED72-G17 | active tab、focused slot与last-used slot语义分离 | Fail |
| ED72-G18 | toolbar command保留slot/surface/generation | Fail |
| ED72-G19 | pointer/capture不能跨slot误投 | Fail |
| ED72-G20 | splitter drag与camera/gizmo/tool手势确定仲裁 | Fail |
| ED72-G21 | keyboard可按空间顺序遍历slot | Fail |
| ED72-G22 | layout rebuild后focus确定恢复 | Fail |
| ED72-G23 | hide/switch/window loss使interaction终态化 | Fail |
| ED72-G24 | focused/maximized/synced/stale有等价视觉与a11y状态 | Fail |
| ED72-G25 | maximize不销毁slot session或per-slot state | Fail |
| ED72-G26 | maximize保存并恢复split、visible set、focus与capture | Fail |
| ED72-G27 | maximize/switch重入有queue/coalesce/reject政策 | Fail |
| ED72-G28 | restore token与layout generation匹配 | Fail |
| ED72-G29 | floating/fullscreen/native maximize边界明确 | Fail |
| ED72-G30 | PIE/pilot/preview transition不误复活旧slot | Fail |
| ED72-G31 | camera sync group有stable identity与scope | Fail |
| ED72-G32 | sync update携带epoch/origin并抑制循环 | Fail |
| ED72-G33 | perspective/orthographic同步使用typed policy | Fail |
| ED72-G34 | incompatible world/source/projection有stable reject reason | Fail |
| ED72-G35 | member revoke/failure/world replacement安全降级 | Fail |
| ED72-G36 | sync provider panic/timeout/oversize被隔离 | Fail |
| ED72-G37 | viewport layout/payload schema有逐版本migration | Fail |
| ED72-G38 | project/user/document scope与override政策确定 | Fail |
| ED72-G39 | exact ratios、slot assignment与payload可round-trip | Fail |
| ED72-G40 | corrupt/future/unknown layout保留原文并回退LKG | Fail |
| ED72-G41 | activation prepare/admit/warm/commit产生typed receipt | Fail |
| ED72-G42 | plugin reload/revoke退休旧layout/slot/sync state | Fail |
| ED72-G43 | per-slot frame demand按active/visible/invalidated调度 | Fail |
| ED72-G44 | CPU/GPU/RT memory预算可准入、降级与诊断 | Fail |
| ED72-G45 | hidden/occluded/minimized/maximized-away slot有节流 | Fail |
| ED72-G46 | 2-4 cell E2E证明独立camera/input/toolbar/current frame | Fail |
| ED72-G47 | fault/soak/profile/a11y/locale/HiDPI矩阵通过 | Fail |
| ED72-G48 | 同语义跨引擎benchmark有可复现receipt | Fail |

## 10. 测试与动态证据矩阵

| 层级 | 必须新增的证据 |
|---|---|
| Pure model | descriptor/slot id、tree compile、ratio validation、preset assignment、sync policy、state machine |
| Persistence | exact ratio/tree/slot payload、version migration、future/corrupt/unknown plugin、LKG、scope merge |
| Projection | 1/2/3/4 cell pane/toolbar/HUD/clip/a11y tree，同generation geometry与render binding |
| Input/focus | mouse/pen/keyboard、divider/camera/gizmo/tool、capture cancel/transfer、last-used/focus return |
| Camera/sync | perspective/ortho combinations、epoch/origin、cycle suppression、stale/replaced world、pilot/Play reject |
| Maximize | double-click/shortcut/reentry、active interaction、floating/fullscreen、restore focus/capture/state |
| Render/currentness | independent slot products、resize/DPI、stale/drop、warm/commit、hidden throttle、RT history回落 |
| Fault | provider panic/timeout、allocation/admission failure、missing owner、plugin revoke、surface/window loss |
| Performance | 1/2/3/4 cell、4K/HiDPI、多window、复杂visual mode、CPU/GPU/VRAM、switch/first-current latency |
| Product | 真实Editor screenshot与native input、save/reopen、locale/RTL/long text、screen reader、multi-monitor |
| Comparative | 与Unreal/Godot相同布局、camera assignment、maximize/restore和交互场景的可复验receipt |

当前没有执行上述动态矩阵。静态split model test、toolbar frame test或单viewport screenshot不能把任何Gate改成Pass。

## 11. Owner路由与禁止重复实现

| 责任 | Canonical owner | Editor72只能做什么 |
|---|---|---|
| 通用layout/profile/dock/tab/window restore | Editor13 | 提交viewport-specific descriptor/payload adapter，不复制Workbench authority |
| shell constraint/responsive/geometry | Editor54 | 消费compiled cell需求并请求geometry，不另写第二套全窗布局器 |
| ViewportSession/surface/render/currentness | Editor58 | 绑定每slot session和产品receipt，不复制render controller |
| pointer/capture/picking | Editor59 | 提供slot geometry/identity并消费qualified input结果 |
| 单viewport camera/navigation/projection | Editor66 | 编排per-slot camera session和sync adapter，不实现camera math副本 |
| 单viewport visualization/show flag/profile | Editor68 | 绑定per-slot profile，不复制resolver |
| preview time/frame demand/audio/environment | Editor69 | 聚合slot visibility/demand policy，不自建preview world |
| Runtime camera/render/world facts | Runtime相关owner | 只消费neutral capability/receipt，不下沉Editor layout/focus/maximize |

禁止用以下临时方案关闭本报告：复制四份`SceneViewportController`字段、用数组索引或tree path当持久化slot id、继续把多个leaf压成tabs后选first active、toolbar dispatch读取全局current Scene、切换布局时销毁并重建camera state、双向camera listener直接互写、把四个viewport全部永久realtime、只保存pane count不保存slot payload、用静态source-shape test替代真实多surface/render/input证据。

## 12. 状态与产出记录

- 审查状态：`complete`，仅表示本轮current-source差距建账完成。
- 实现状态：`not_started`。
- 新增finding：`2 P0 / 48 P1 / 10 P2`。
- 资格门：`0 Pass / 48 Fail`。
- 建议首个实施点：ED72-M0，先用RED E2E固定split leaf不可见和toolbar wrong-target，再建立M1 stable slot schema；不得先复制四个controller。
- 实施前置：重取61个Zircon文件、Editor13/54/58/59/66/68/69父报告和相关Runtime endpoint终态；重新冻结working-tree fingerprint。
- 验证声明：本轮未运行Cargo与动态产品验证，不能宣称功能、性能、表现、无障碍、插件安全或跨平台已达到目标。

## 13. 最终判断

当前Zircon不是“还没加四视图按钮”，而是通用split layout和单viewport产品之间存在结构断层。已有split tree、workspace serialization、surface frame和camera/visual controller是可保留积木；flatten-first-active、single pane/frame/product、Null payload、single-instance descriptor和丢弃surface key则是必须替换的临时实现。

正确路线是先关闭两条当前P0，建立stable slot identity与versioned layout compiler，再接Editor58/66/68/69的per-slot session，随后一次性完成multi-cell projection、focus/input、atomic preset activation、非破坏maximize、camera link-sync和预算持久化。只有48个资格门全部通过，Scene Viewport Layout才可从“模型里能出现SplitNode”提升为接近Unreal/Godot成熟度的工程级产品。
