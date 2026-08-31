---
title: Editor Scene Viewport Camera Navigation、Orbit、Pan、Zoom、Fly、Projection、Alignment、Frame Selection、Bookmark、Pilot、Persistence、Input 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor187
review_date: 2026-08-27
baseline_head: 681588f7a1cbfaae3147e8b93e1be6705d810f21
related_code:
  - zircon_runtime/src/core/framework/camera_controller
  - zircon_runtime/src/input/camera_controller
  - zircon_runtime/src/core/framework/input/input_event.rs
  - zircon_runtime/src/core/framework/input/input_frame_snapshot.rs
  - zircon_runtime/src/core/framework/input/mouse_wheel.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/input_state.rs
  - zircon_runtime/src/input/runtime/recording.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_editor/src/scene/viewport/controller
  - zircon_editor/src/scene/viewport/interaction
  - zircon_editor/src/scene/viewport/projection.rs
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/ui/binding/viewport
  - zircon_editor/src/ui/host/editor_event_execution/viewport_event.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_viewport_panel.zui
tests:
  - zircon_runtime/src/tests/camera_controller.rs
  - zircon_runtime/src/input/tests/input_manager/frame_state.rs
  - zircon_runtime/src/input/tests/recording_replay.rs
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/host/binding_dispatch/viewport.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/toolbar_dispatch.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/typed_command.rs
  - zircon_editor/src/tests/host/template_runtime/scene_viewport_toolbar_runtime_projection.rs
  - zircon_editor/src/tests/ui/binding/viewport.rs
  - zircon_editor/src/tests/workbench/chrome_snapshot/viewport_settings.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/66-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/174-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-current-source-review.md
  - docs/plans/optimize/zircon_editor/178-editor-command-registry-keymap-menu-palette-context-routing-remote-automation-current-source-review.md
  - docs/plans/optimize/zircon_editor/179-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-current-source-review.md
  - docs/plans/optimize/zircon_editor/180-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/184-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/37-camera-endpoint-director-rig-controller-blend-shake-cinematic-cut-history-multiview-network-save-scalability-product-integration-review.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/05/2026-07-18-viewport-shared-projection-context.md
  - docs/plans/zircon_editor/editor/05/failure-2026-07-31-scene-mode-input-ownership-hardcut.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
  - docs/plans/mvp/05-f4-basic-authoring.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorViewportClient.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorViewportClient.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/LevelEditorViewport.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/LevelEditorViewport.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Settings/LevelEditorViewportSettings.h
  - dev/UnrealEngine/Engine/Source/Editor/EditorViewport/Private/ViewportClientNavigationHelper.cpp
  - dev/UnrealEngine/Engine/Source/Editor/EditorViewport/Public/ViewportClientNavigationHelper.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Tests/CameraSpeedSettingsTests.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.h
  - dev/godot/scene/debugger/view_3d_controller.cpp
  - dev/godot/scene/debugger/view_3d_controller.h
  - dev/Fyrox/editor/src/camera/mod.rs
  - dev/Fyrox/editor/src/camera/panel.rs
  - dev/Fyrox/editor/src/settings/camera.rs
  - dev/Fyrox/editor/src/settings/scene.rs
  - dev/bevy/crates/bevy_camera_controller/src/lib.rs
  - dev/bevy/crates/bevy_camera_controller/src/free_camera.rs
  - dev/bevy/crates/bevy_camera_controller/src/pan_camera.rs
  - dev/bevy/examples/camera/camera_orbit.rs
  - dev/bevy/examples/camera/projection_zoom.rs
  - dev/bevy/crates/bevy_camera/src/projection.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/CameraEditorUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Samples~/Common/Scripts/AlignSceneView.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/RenderPipeline/Camera/HDAdditionalSceneViewSettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Editor/Camera/UniversalAdditionalSceneViewSettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Utilities/CameraSettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Utilities/CameraSettingsUtilities.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Tests/Editor/CameraSettingsUtilitiesTests.cs
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/66-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/66-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Scene Viewport Camera Navigation 当前源码复核

## 1. 结论

Editor66的核心判定仍成立：Zircon Runtime具备Free、Orbit、Pan三套可复用camera controller，当前Editor Scene Viewport却仍只持有并调用`OrbitCameraController`。产品输入仍是无身份的pointer边沿、裸`Scrolled(f32)`和resize，没有keyboard axis、dt、focus、device/window/view/capture generation或gesture。Runtime能力的存在没有转化为Editor free-fly、速度控制、可配置scheme、focus/cursor生命周期或typed navigation receipt。

当前源码有三类真实进展。第一，controller实现已迁到`zircon_runtime/src/input/camera_controller`，Editor与dynamic adapter都消费同一Orbit kernel；第二，Runtime Input已具`InputFrameSnapshot`、line/pixel `MouseWheelEvent`、mouse motion、touch/gamepad、focus-loss release、bounded recording/replay；第三，Editor新增共享`ViewportProjectionContext`、多选position frame test和更完整的pointer cancel/chord guard。这些基础必须复用。

但产品桥接仍丢弃上述事实：retained pointer route把scroll压成一个`f32`，`ViewportInput`不携Runtime input frame，Orbit pan仍不读`viewport_size`，Perspective和Orthographic zoom仍使用`signum()`，projection切换不保视觉尺度，Frame Selection仍只聚合节点位置、保留旧远距离并忽略horizontal FOV/clipping，上层仍可在底层false时显示`Framed node`。可见`0.25` speed chip仍无consumer；history、bookmark、preview/pilot和per-view camera session仍为0。

本轮不新增P0。Editor66的canonical状态刷新为：**23项P1中14 Open / 9 Partial / 0 Closed；7项P2中5 Open / 2 Partial / 0 Closed；48门中36 Fail / 12 Partial / 0 Pass**。Editor66仍是唯一finding owner，Editor187不重复增加finding数量。

本轮只做静态review与重构计划，不修改production Rust/ZUI，不运行Cargo、Editor、设备矩阵、projection/framing golden、save/reopen、pilot、fault/soak/profile或跨引擎benchmark。Tooling按用户要求排除；未查询、轮询、等待或实时跟踪协调器。现有证据不能支持功能、表现或性能达到或超过Unreal。

## 2. 审查边界与冻结语料

### 2.1 Current working tree边界

冻结时主仓HEAD为`681588f7a1cbfaae3147e8b93e1be6705d810f21`。共享checkout中camera/viewport选择集含大量dirty/untracked内容：旧`core/framework/camera_controller/*/controller.rs`已删除，controller实现已迁入`input/camera_controller`，Editor controller也被拆成多个文件。本报告以读取时磁盘上的current working tree为事实源，不以旧Editor66行号、旧fingerprint或HEAD源码替代当前实现，也未覆盖、回退或格式化其他会话修改。验证期间`scene_viewport_controller_camera.rs`与`editor_state_viewport.rs`又发生并发更新，以下指标、调用链和结论已按更新后的磁盘内容重新冻结。

MVP baseline recovery仍为`in_progress`，F4不能绕过F0-F3资格门。Editor187是后续RED与架构切分输入，不是实现完成或动态验收receipt。

### 2.2 冻结物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Runtime camera kernel | **23 / 1,123 / 966 / 32,729 / 3** | core DTO、input controller实现、dynamic adapter | `6ba86f4351345e0ced82a72b1975b6645a957003d3b72d64d3aac64c0a547228` |
| Runtime input foundation | **7 / 1,301 / 1,180 / 46,060 / 6** | input event/frame、wheel unit、cursor request、manager/focus/recording | `33b2c354f002373aaed5f42408ab5fd55f919897d6b349f58634254c5ad55d0b` |
| Editor navigation core | **13 / 1,624 / 1,465 / 57,418 / 14** | state、input、orbit/pan/zoom、projection、frame、settings/cancel | `061770621082bc153c231b157fd5624b4a34205e7abb85f95da65eeff83c1fef` |
| Editor product route | **15 / 3,731 / 3,403 / 160,310 / 12** | command/codec/event/dispatch/state/toolbar/static speed chip | `aba2dac23107ae2fea0c951c9a4bf4dafceb914b3552c3121303a1d87e2e4ad2` |
| Zircon focused tests | **11 / 3,128 / 2,808 / 108,616 / 77** | controller/input replay、viewport state、binding/dispatch/template/chrome | `f22fe99399466a7c660d978a5aa665b4759b6b30c96764b3aae611a90535e542` |
| Unreal selected set | **8 / 17,420 / 14,521 / 647,980 / 1** | camera transform/input/speed/focus/orbit/fly/projection/pilot | `7f3ef3c8d0c7542d73ee870207fe7841b090535ffcc9b92d7197c82e4cf9f397` |
| Godot selected set | **4 / 9,059 / 7,716 / 350,451 / 0** | schemes、gesture/freelook、focus/cancel、projection、camera preview | `5dbeadb37cbb5a1f24d377e7aa252ae9770002c18605a485bf9b3c1a8dbbb2b4` |
| Fyrox selected set | **4 / 1,314 / 1,193 / 46,233 / 0** | AABB fit、projection-aware input、speed、per-scene camera settings | `b31df90896076114dff12aad35e654bf90ad729af7e038b6f7c88fcecd0bf7dd` |
| Bevy selected set | **6 / 2,012 / 1,840 / 77,312 / 1** | dt movement、scroll unit、touch/grab、smooth orientation、projection | `4f03bb947b052f450b527c14b7fbce810d0fa0f52ba5638774d611f652c16d75` |
| Unity Graphics selected set | **7 / 1,012 / 909 / 45,419 / 0** | SceneView附加设置、alignment、lens/frustum与apply/extract | `6f60e86adf941680381ac3a3458dbf01d1fe78b3edb397ab8202bfbca7ae3181` |

fingerprint方法为：规范化相对路径，计算逐文件SHA-256，生成`path::file_hash`，按路径排序并以当前环境换行连接，再对整体计算SHA-256。它只证明选择集的current-source内容，不代表ABI、artifact、动态行为或性能。Godot、Fyrox、Bevy与Unity Graphics revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal跟随主workspace。

### 2.3 Owner边界

Editor187只拥有camera navigation语义、controller composition、framing、navigation preference、view history/bookmark和pilot orchestration。Per-view/session/product currentness由Editor179拥有；qualified pointer/capture/cancel由Editor180与Editor174拥有；command/keymap由Editor178拥有；scene camera asset/rig/director/lens由Editor30与Runtime37拥有；pilot写回的document transaction由Editor184拥有。本报告只定义消费合同，不重复这些父报告的finding。

## 3. 当前实现拓扑

### 3.1 Controller迁移是模块进展，不是产品接线

Free/Orbit/Pan实现已从旧`core/framework/camera_controller/*/controller.rs`迁到`input/camera_controller`，settings/input/state/output DTO仍位于`core/framework/camera_controller`。Editor和dynamic adapter均改用`zircon_runtime::input::camera_controller::OrbitCameraController`。这减少了数学实现分叉，应保留并继续完成清晰的feature owner hard cutover。

但`SceneViewportState`仍只存`OrbitCameraController`。Play模式的新route会把right/middle/scroll送入同一个Editor camera input入口并阻断authoring Frame Selection，这是合理隔离进展，却没有引入Free/Pan或per-view session。Free的dt movement、walk/run、speed multiplier、friction和cursor intent，以及Pan的keyboard/drag/rotate/zoom均没有Editor consumer。`RuntimeCameraController`和Editor仍各自复制right/middle/scroll drag adapter；dynamic pointer path直接修改active scene camera，而Editor默认维护local snapshot。共享kernel不等于共享validated input/session/receipt。

### 3.2 Runtime Input已有工程底座，Editor桥将其降级

Runtime Input能区分line/pixel wheel并保留x/y与事件序列，frame snapshot包含mouse motion、buttons、touch、gamepad和cursor requests。`FocusLost`会释放active input、清wheel/motion/touch、取消IME并发布cursor grab None；recording/replay有bounded与incomplete状态，并测试focus loss作为release transaction。

Editor retained pointer route却只发布`PointerMoved{x,y}`、三种button press/release、`Scrolled{delta}`和无参数`CancelInteraction`。`ViewportInput`再降为相同裸枚举，没有timestamp、dt、window/view/device/pointer/capture generation、raw delta、unit、focus或gesture。Runtime Input底座与Scene Viewport产品之间没有adapter，不能借前者为后者通过门禁。

### 3.3 数学和numeric合同仍不统一

Orbit input保留`viewport_size`并做clamp，但controller完全不读取它；Perspective pan仍用`distance * pan_sensitivity`作为world-per-pixel。Editor已有正确使用FOV/aspect/viewport的`ViewportProjectionContext::world_units_per_pixel`，Orbit path却没有消费。

Orbit zoom使用`delta.signum()`；Orthographic zoom也以`signum()`计算固定10%倍率，零delta仍走成功尾部并把`view_orientation`置为User。Free controller在`focus_active=false`时仍处理movement/scroll/velocity translation，只有look被阻止；cursor release仍依赖`cursor_grab_changed`。controller settings没有统一finite/range/profile compile，NaN/inf和极端值仍可进入浮点数学。

Dynamic editor-camera ABI adapter会校验transform、FOV、ortho size、near/far与projection kind，这是可复用Partial；但Editor camera命令、controller input和projection transition未共享该validator。

### 3.4 Projection、axis和Frame仍是离散命令

`set_projection_mode`只替换枚举，不从distance/FOV推导ortho extent，也不从ortho extent恢复Perspective distance。`align_view`在camera未初始化时使用`ViewportCameraSnapshot::default()`而非当前Scene Camera lens。Orbit只在真实delta时changed，但Editor的pan/zoom/frame路径仍无条件或错误地把axis view降为User。

Frame Selection的`selection_frame`只遍历selected world position，单对象radius为0；perspective distance以当前offset和固定6为下限，因此远处相机不会可靠拉近。solver只使用vertical FOV和bounding-sphere近似，不满足horizontal FOV/aspect，不调整near/far或large-world clipping。新增多选position test只证明两个点能扩大距离，不证明mesh/subtree/component aggregate bounds或真实画面fit。

`EditorState::frame_selection`只检查primary selection，调用controller后忽略`feedback.camera_updated`并无条件发布`Framed node {id}`。底层NoBounds/invalid/stale/unchanged没有typed receipt，旧false-green保持Open。

### 3.5 产品状态仍是单controller和静态chrome

Editor179已确认Host只有一个Scene controller/size/submission；`ViewInstanceId`存在于workbench布局与消息系统，却没有绑定camera、pivot、controller、history或generation。Play route复用该单一controller的navigation-only入口，不改变这一产品边界。`SceneViewportSettings`只含transform/projection/orientation/display/grid与preview flags，没有scheme、sensitivity、speed、invert、friction、zoom policy或transition。

`workbench_viewport_panel.zui`仍显示固定文本`0.25`，没有binding/event/settings/controller consumer。全仓目标类型`EditorViewportCameraSession`、`ViewportCameraNavigationCoordinator`、`CameraNavigationProfile`、`FrameSelectionReceipt`、`CameraViewHistory`、`CameraBookmark`和`CameraPilotSession`均为0。

## 4. Canonical finding状态账本

### 4.1 P1架构、正确性与产品闭环

| Finding | 当前 | 当前证据 | 必须重构 |
|---|---|---|---|
| ED66-P1-01 Editor只接Orbit，Runtime Free/Pan无产品consumer | Open | state仍只有Orbit；Free/Pan仅unit tests | per-view coordinator组合三controller并接keyboard/dt/cursor |
| ED66-P1-02 `ViewportInput`不能表达工程导航输入帧 | Partial | Runtime `InputFrameSnapshot`完整度提高；Editor桥仍裸枚举 | qualified input adapter保留unit/time/device/view/capture并解析continuous axis/gesture |
| ED66-P1-03 scheme/chord硬编码 | Open | right orbit、middle pan、scroll zoom固定 | versioned profile链接Editor178 keymap并做conflict/admission |
| ED66-P1-04 focus/cursor-grab生命周期不守恒 | Partial | Runtime Input focus loss释放已工程化；Editor未接，Free controller仍可失焦移动 | 同一capture owner处理focus/window/device/disable并返回terminal receipt |
| ED66-P1-05 Orbit viewport死字段且pan尺度错误 | Partial | shared projection context已有正确world-per-pixel；Orbit仍不消费 | camera kernel统一消费validated projection metrics |
| ED66-P1-06 scroll `signum()`破坏幅度/单位/zero | Partial | Runtime wheel unit/magnitude与replay存在；Editor/Orbit/Ortho仍降级 | 在bridge归一单位，连续指数zoom，zero严格无副作用 |
| ED66-P1-07 settings/state缺finite与range验证 | Partial | dynamic camera ABI校验lens/transform；controller/profile仍无compile | `ValidatedCameraNavigationProfile`与input/state repair/reject |
| ED66-P1-08 无统一惯性/平滑/可中断transition | Open | 只有Free velocity damping，其他命令瞬跳 | dt-stable transition，pointer抢占、cancel、retarget与唯一终态 |
| ED66-P1-09 navigation state无per-view session identity | Partial | workbench `ViewInstanceId`存在；Editor179确认controller仍单例 | camera payload挂到canonical viewport session registry和generation |
| ED66-P1-10 projection切换不保持视觉尺度/clip | Open | 只替换enum | pure transition solver输出before/after lens/pivot/transform/warning |
| ED66-P1-11 axis align依赖generic default且view type丢失 | Open | cold align clone default；pan/zoom/frame错误置User | lens继承、axis/auto-ortho policy与有效变化驱动状态迁移 |
| ED66-P1-12 Frame不读取真实aggregate bounds | Open | 仍只聚合node positions | 请求generation-qualified mesh/subtree/component bounds并定义过滤政策 |
| ED66-P1-13 Frame保留旧远距离 | Open | `offset.length().max(FRAME_DISTANCE)`仍存在 | 从bounds/projection/padding独立求目标distance/extent |
| ED66-P1-14 Frame忽略aspect/horizontal FOV/clipping | Partial | shared projection context已有aspect/FOV/near/far事实；frame未使用 | framing solver同时满足横纵fit并处理near/far/behind/large-world |
| ED66-P1-15 Frame失败仍发布成功 | Open | `EditorState::frame_selection`忽略camera_updated | typed receipt；仅Applied更新status/history/invalidation |
| ED66-P1-16 无view history/bookmark | Open | 对应类型/存储/命令为0 | bounded back/forward与stable bookmark CRUD/persistence/migration |
| ED66-P1-17 无preview/pilot scene camera session | Open | 没有target binding/pre-pilot snapshot/writeback | qualified target generation、preview isolation、transactional pilot与restore |
| ED66-P1-18 Editor与dynamic adapter复制且ownership不同 | Partial | 共用Orbit kernel且editor-camera extract override不改Scene；两套drag映射仍重复，dynamic pointer仍写active camera | 共享input/profile/output，显式区分runtime endpoint与editor/pilot owner |
| ED66-P1-19 output/feedback不足以作产品receipt | Open | output和feedback仍以changed/bool为主 | consumed/mode/pivot/lens/speed/capture/reject/warning/invalidation typed receipt |
| ED66-P1-20 静态`0.25` speed chip无consumer | Open | ZUI文本仍存在且无事件 | M0移除/Unavailable；真实控件绑定current-view resolved speed |
| ED66-P1-21 navigation preference无版本化authority | Open | settings没有scheme/speed/sensitivity等 | user/project/view/device override、schema/migration与immutable resolution |
| ED66-P1-22 测试不覆盖导航状态机 | Partial | generic input focus/replay、projection context、多选position frame有新增测试 | 补Editor keyboard/device/projection/bounds/receipt/history/pilot与fault/scale矩阵 |
| ED66-P1-23 无large-world/scene-scale模型 | Open | distance/speed/clip/fallback仍固定世界单位 | distance-scaled speed、高精度anchor/rebase与多数量级golden/profile |

### 4.2 P2质量与资格证据

| Finding | 当前 | 当前证据 | 必须重构 |
|---|---|---|---|
| ED66-P2-01 camera magic constants分散 | Open | Runtime/Editor仍各自定义sensitivity/floor/factor/distance/padding | 单位化validated defaults/profile与solver private constants |
| ED66-P2-02 无结构化navigation diagnostics | Open | 只有局部status/bool/error | per-view input/mode/pivot/projection/capture/reject trace与容量/隐私边界 |
| ED66-P2-03 无hot-path/soak/allocation预算 | Partial | Runtime focus reset已有局部perf test，input recording有界；camera产品无预算 | 60/120/240Hz、history和1/1K/100K frame aggregate profile |
| ED66-P2-04 keyboard-only/touch/pen/reduced-motion政策缺失 | Open | Runtime收touch但Editor camera无consumer | 所有设备共享profile/admission，programmatic transition尊重reduced-motion |
| ED66-P2-05 profile/bookmark/history无迁移和容量 | Open | 三类产品均不存在 | stable id/version、max/LRU、corruption/missing target/scope policy |
| ED66-P2-06 无确定性camera input trace/golden replay | Partial | Runtime generic input recording/replay真实；没有camera initial state/expected receipt | 从normalized input到camera receipt的跨frame golden |
| ED66-P2-07 无同语义跨引擎基线 | Open | 未运行固定scene/input/hardware benchmark | 功能同构后比较orbit/pan/zoom/fly/frame/pilot及p50/p95/p99 |

## 5. 五套参考实现差异

### 5.1 Unreal：主参考是完整viewport camera系统

`FEditorViewportClient`同时拥有view transform、orbit/fly转换、input key/axis/gesture、camera speed setting与distance scaling、Perspective/Orthographic、`FocusViewportOnBox`和camera lock/pilot。Level viewport把锁定actor和camera movement纳入Editor lifecycle，camera speed还有独立测试。Zircon应学习状态、输入、bounds focus、pilot和设置之间的合同关系，不复制UObject或legacy全局viewport形态。

### 5.2 Godot：scheme、cancel、freelook和camera preview是一体

Godot 3D viewport提供navigation mode/scheme、mouse/gesture、freelook speed、orbit/pan/zoom、focus、orthogonal policy和inertia。Window focus out、gizmo cancel会恢复cursor/保存状态；previewing camera/cinema会改变导航控件可见性，并有相机绑定/编辑事务路径。Zircon当前只有右/中/scroll的无身份adapter，尚未达到这条最小产品状态机。

### 5.3 Fyrox：Rust Editor也能完成AABB fit和per-scene保存

Fyrox camera使用hierarchy AABB和Perspective/Orthographic参数计算fit；pointer pan按projection scale，keyboard movement按dt和speed factor，camera transform/projection保存到per-scene settings。Camera Preview还管理selected camera和退出cleanup。它不是Unreal的完整上限，但直接反证Zircon只框node position、静态speed chip和无持久化是合理简化。

### 5.4 Bevy：明确输入单位、dt与cursor/touch边界

Bevy free camera把continuous movement乘`delta_secs`，mouse delta不乘dt，按window focus决定grab，disable会主动release，并支持touch；pan/zoom区分wheel line/pixel并平滑处理。它不提供完整Editor bookmark/pilot，但可作为normalized input与controller时间语义参考。

### 5.5 Unity Graphics：只取SceneView lens/frustum合同

Unity Graphics选择集提供SceneView camera alignment和per-view附加设置，以及CameraSettings/Utilities对near/far/FOV/projection的apply/extract与测试。它不包含闭源Unity Editor全部导航实现，因此只用于lens/frustum/current settings边界，不能作为bookmark/pilot或input系统完成度证据。

## 6. 目标架构

### 6.1 Runtime pure navigation合同

- `CameraNavigationInputFrame`：normalized pointer/keyboard/gesture、dt/time、focus/capture、unit/source；不含Editor document/view。
- `CameraNavigationProfile -> ValidatedCameraNavigationProfile`：controller mode、speed/sensitivity/invert/inertia/range，compile时验证finite/unit/range。
- `CameraProjectionMetrics`：projection、FOV/ortho extent、aspect、viewport、near/far，供pan/zoom/frame共享。
- 保留Free/Orbit/Pan controller，但统一zero、focus、numeric repair和output semantics。
- `CameraNavigationOutput`：transform/pivot/lens/speed/cursor、consumed/changed/transition和typed rejection。
- `CameraFramingSolver`：只消费validated aggregate bounds、lens、viewport和padding，返回Perspective/Orthographic目标，不读取Editor selection。

Runtime Input负责物理事实与record/replay，Scene/inspection负责aggregate bounds与camera endpoint。Runtime不认识toolbar/bookmark/document，普通Editor导航也不得修改Scene camera。

### 6.2 Editor per-view产品层

- `EditorViewportCameraSession`挂在Editor179的`ViewInstanceId + window + session generation`上，拥有camera/pivot/projection/controller/transition/history cursor。
- `ViewportCameraNavigationCoordinator`把Editor180 qualified input和Editor178 resolved keymap/profile编译为Runtime frame，处理capture/focus/cancel与receipt publication。
- `FrameSelectionCoordinator`冻结selection/view/document generation，取aggregate bounds，调用pure solver并发布typed terminal receipt。
- `CameraNavigationPreferenceStore`管理user/project/view/device override、schema/migration和resolved immutable snapshot。
- `CameraViewHistory`与`CameraBookmarkStore`提供有界view-only状态，不污染Scene dirty/undo。
- `CameraPilotSession`保存pre-pilot editor view，绑定qualified scene camera endpoint；写回必须通过Editor184 document transaction。

### 6.3 输入到显示的时序

`platform/runtime input -> qualified viewport envelope -> keymap/chord/profile resolution -> view/capture admission -> normalized camera frame -> Runtime controller/solver -> typed output -> session generation commit -> invalidation receipt -> renderer consumes same-generation camera snapshot`。

zero、stale view、focus loss、invalid profile/lens不得修改session。用户输入必须能抢占programmatic transition；每个capture/transition/pilot只产生一个terminal disposition。

## 7. 分阶段重构计划

### Editor187-M0：真实性止血与RED基线

移除或标记Unavailable的`0.25` speed chip；修复Frame false success、orthographic zero scroll副作用和axis identity误降。先写scroll magnitude/unit、focus loss、projection round-trip、real bounds frame、stale view和pilot RED tests，并冻结Editor179/180依赖接口。

### Editor187-M1：Runtime frame/profile/output和numeric hardening

以现有Input snapshot/wheel/focus/recording为基础构造normalized camera frame和validated profile；统一三controller的focus、zero、finite/range、scroll单位、projection metrics与typed output。完成旧core DTO和新input implementation的清晰owner边界。

### Editor187-M2：per-view session和Free/Pan产品接线

把camera/pivot/controller挂到canonical viewport session registry，接keyboard/mouse free-fly、run/slow/speed和cursor lease。Editor与dynamic runtime共享input/profile/output semantics，但保留runtime endpoint、editor camera和pilot target三种明确owner。

### Editor187-M3：Projection、axis与Frame solver

实现视觉尺度保持的projection transition、lens继承、axis/auto-ortho policy、真实aggregate bounds、aspect/FOV/clipping-aware fit和可中断transition。所有命令复验view/document generation并返回typed receipt。

### Editor187-M4：Profile、keymap与产品controls

建立versioned preference resolution并链接Editor178 keymap/chord；支持scheme、speed、sensitivity、invert、inertia、zoom/pan和accessibility。toolbar/chrome只展示同代resolved snapshot，不持有静态authority。

### Editor187-M5：History、bookmark与持久化

实现bounded back/forward、bookmark CRUD、stable id、per-user/project scope、save/reopen、migration/corruption recovery和missing target policy。多view完全隔离，只记录成功terminal camera state。

### Editor187-M6：Preview/Pilot与document transaction

区分preview-only和pilot；冻结endpoint generation和pre-pilot view，处理target delete/unload/external edit/document replace/cut/undo。Pilot transform/lens写回通过Editor184 transaction，退出只恢复Editor view，不暗中回滚已提交Scene mutation。

### Editor187-M7：设备、规模、故障和性能资格

运行mouse/wheel/trackpad/touch/pen/keyboard设备矩阵、deterministic trace、8小时soak、100K selection bounds、1/4/16 active views和large-world多数量级golden。功能同构后再以固定scene/input/hardware和release build对Unreal/Godot/Fyrox/Bevy报告p50/p95/p99、CPU、allocation和memory。

## 8. 资格门

| Gate | 验收条件 | 当前 | 当前证据 / 缺口 |
|---|---|---|---|
| ED66-G01 | frame区分pointer delta、continuous axis与gesture | Partial | Runtime snapshot区分；Editor Viewport不消费 |
| ED66-G02 | scroll line/pixel/factor归一且保留幅度 | Partial | Runtime wheel合同存在；Editor丢unit，zoom用signum |
| ED66-G03 | mouse delta不乘dt、continuous movement按dt | Partial | 分散controller遵守，未由统一产品frame证明 |
| ED66-G04 | focus/window/seat/device/capture identity可复验 | Partial | 底层focus/cursor/metadata基础存在，Editor事件无identity |
| ED66-G05 | zero input不改transform/pivot/orientation/history | Partial | controller部分no-op；orthographic zero scroll仍返回成功并置User |
| ED66-G06 | NaN/inf/负dt/极端delta Fail closed | Fail | controller input/profile无统一validator |
| ED66-G07 | profile compile验证range/unit/finite | Fail | profile不存在 |
| ED66-G08 | output含typed consumed/changed/rejection | Partial | 有changed/delta/cursor，缺consumed/rejection/session identity |
| ED66-G09 | Editor接入orbit/pan/free-fly | Fail | 只接Orbit |
| ED66-G10 | controller切换保持transform/pivot | Fail | 无产品切换 |
| ED66-G11 | focus lossrelease cursor并终止/暂停运动 | Partial | Runtime Input可release；Editor未接，Free仍可失焦移动 |
| ED66-G12 | Escape/window/device interruption有唯一terminal receipt | Partial | global Cancel和focus release存在，无scope/generation/receipt |
| ED66-G13 | keymap/chord/scheme来自resolved profile | Fail | hard-coded mapping |
| ED66-G14 | binding冲突/不可用输入给typed reason | Fail | 无camera admission/profile link |
| ED66-G15 | pan跨viewport/FOV保持屏幕不变量 | Partial | shared helper正确，Orbit path不消费 |
| ED66-G16 | speed/run/slow真实驱动current view | Fail | Free kernel存在，Editor无consumer且chip静态 |
| ED66-G17 | projection切换保持视觉尺度 | Fail | 只切enum |
| ED66-G18 | projection切换复验FOV/aspect/near/far | Partial | dynamic ABI validator存在，Editor切换未使用 |
| ED66-G19 | axis align继承当前lens | Fail | cold path仍default snapshot |
| ED66-G20 | pan/zoom/frame保持有效axis identity | Fail | 无条件/错误置User |
| ED66-G21 | axis snap/transition可中断retarget | Fail | transition不存在 |
| ED66-G22 | Frame使用真实aggregate bounds | Fail | 只聚合positions |
| ED66-G23 | Frame满足horizontal/vertical FOV/aspect | Fail | frame只用vertical FOV |
| ED66-G24 | Frame处理ortho/near/far/large-world clipping | Fail | 固定ortho和无clip策略 |
| ED66-G25 | camera state绑定ViewInstanceId/session generation | Fail | controller仍单例 |
| ED66-G26 | stale input/output不能提交replacement view | Fail | 无view generation |
| ED66-G27 | 多view camera/pivot/controller完全隔离 | Fail | 无per-view product |
| ED66-G28 | profile override有user/project/view优先级 | Fail | profile不存在 |
| ED66-G29 | settings/profile save-reopen/migration守恒 | Fail | navigation settings authority不存在 |
| ED66-G30 | history有界且只记成功terminal state | Fail | history不存在 |
| ED66-G31 | bookmark CRUD用stable id并可迁移 | Fail | bookmark不存在 |
| ED66-G32 | corruption/missing target有typed disposition | Fail | persistence/target产品不存在 |
| ED66-G33 | Frame failure不发布成功status/history | Fail | 上层仍无条件`Framed node` |
| ED66-G34 | chrome没有无consumer speed authority | Fail | `0.25`仍可见 |
| ED66-G35 | camera command返回typed receipt/invalidation | Fail | 仍压为`camera_updated: bool` |
| ED66-G36 | preview camera不修改Scene/dirty | Fail | 产品preview session不存在 |
| ED66-G37 | pilot target有qualified endpoint generation | Fail | pilot不存在 |
| ED66-G38 | pilot保存/恢复pre-pilot view | Fail | pilot不存在 |
| ED66-G39 | pilot mutation进入document transaction | Fail | pilot不存在 |
| ED66-G40 | external edit/cut/document replace策略明确 | Fail | target lifecycle不存在 |
| ED66-G41 | controller tests覆盖range/invalid/focus | Partial | 有pitch/zoom clamp和idle test，无invalid/focus完整矩阵 |
| ED66-G42 | Editor tests覆盖keyboard/mouse/gesture/device | Fail | 仍只覆盖pointer happy path/route |
| ED66-G43 | normalized trace可确定性重放 | Partial | generic Runtime input replay存在，无camera receipt golden |
| ED66-G44 | 60/120/240Hz结果和手感在容差 | Fail | 无profile |
| ED66-G45 | 8小时soak无漂移 | Fail | 无soak artifact |
| ED66-G46 | 100K selection Frame满足预算 | Fail | 无aggregate path/profile |
| ED66-G47 | large-world极近/极远满足精度/clip预算 | Fail | 无模型/golden |
| ED66-G48 | 同语义跨引擎receipt可复现 | Fail | 未运行benchmark |

## 9. 验证与当前源码守卫

本轮静态守卫确认：`EditorViewportCameraSession`、`ViewportCameraNavigationCoordinator`、`CameraNavigationInputFrame`、`CameraNavigationProfile`、`FrameSelectionReceipt`、`CameraViewHistory`、`CameraBookmark`、`CameraPilotSession`在含untracked内容的当前Runtime/Editor树中均为0；`SceneViewportState`仍只引用Orbit controller；`ViewportInput`仍只有pointer/scroll/resize；ZUI仍含固定`0.25`；Frame仍调用`selected_world_position`并使用`offset.length().max(FRAME_DISTANCE)`；上层仍无条件设置`Framed node`。

后续最低验证矩阵包括：normalized unit/focus/numeric property tests；Free/Orbit/Pan product parity；multi-view replacement/stale rejection；projection round-trip和axis identity；mesh/subtree/component aggregate frame golden；false receipt negative test；profile/keymap/persistence；history/bookmark；preview/pilot undo/redo/lifecycle；设备/fault/scale/soak/profile与同硬件跨引擎对照。

本轮没有运行Cargo或Editor，因此没有build/test/runtime/performance green声明。落盘只执行frontmatter path、finding/gate计数、索引唯一性、source guard、fingerprint currentness和diff whitespace静态检查。

## 10. 最终判定

Runtime Input和camera kernel已经提供了比旧报告更好的工程材料，但Editor Scene Viewport仍是固定鼠标映射、单Orbit、单controller、bool feedback和position-only Frame。继续在`ViewportInput`上增加零散variant、在Editor复制数学或把speed chip接到局部字段，只会扩大Runtime Input与Editor产品的双轨。

正确顺序是：**统一Runtime normalized input/profile/output与projection/framing solver -> canonical per-view camera session/capture -> Free/Pan产品接线 -> projection/axis/real-bounds Frame -> preference/keymap -> history/bookmark -> preview/pilot transaction -> device/scale/fault/performance资格**。在48门全部Pass前，该域应保持“工程化重构待实施”。
