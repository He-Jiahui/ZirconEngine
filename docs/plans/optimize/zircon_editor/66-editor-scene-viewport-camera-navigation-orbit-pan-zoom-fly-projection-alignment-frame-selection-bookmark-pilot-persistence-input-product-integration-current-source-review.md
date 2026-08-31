---
title: Editor Scene Viewport Camera Navigation、Orbit、Pan、Zoom、Fly、Projection、Alignment、Frame Selection、Bookmark、Pilot、Persistence、Input 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor66
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - zircon_runtime/src/core/framework/camera_controller
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
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/host/binding_dispatch/viewport.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/toolbar_dispatch.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/typed_command.rs
  - zircon_editor/src/tests/host/template_runtime/scene_viewport_toolbar_runtime_projection.rs
  - zircon_editor/src/tests/ui/binding/viewport.rs
  - zircon_editor/src/tests/workbench/chrome_snapshot/viewport_settings.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/58-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/37-camera-endpoint-director-rig-controller-blend-shake-cinematic-cut-history-multiview-network-save-scalability-product-integration-review.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
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
doc_type: current_source_review
canonical_owner: docs/plans/optimize/zircon_editor/66-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Scene Viewport Camera Navigation、Orbit、Pan、Zoom、Fly、Projection、Alignment、Frame Selection、Bookmark、Pilot、Persistence、Input 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon当前不是从零开始。Runtime已经有独立的`FreeCameraController`、`OrbitCameraController`、`PanCameraController`，各自具有settings、state、input与`CameraControllerOutput`；free controller支持dt、walk/run、指数速度缩放、摩擦和cursor-grab intent，pan controller支持键盘、drag、rotation与viewport-aware scaling。Editor也有真实的per-controller camera snapshot、orbit target、透视/正交、六向对齐、Frame Selection、toolbar command与pointer route。这些底座应保留。

但产品路径只接入`OrbitCameraController`。`ViewportInput`只有pointer move、三键press/release、scroll和resize，没有键盘、modifier、dt、focus、window/seat/device、gesture、raw motion、capture或terminal reason；右键orbit、中键pan、滚轮zoom被硬编码。Orbit input携带`viewport_size`却完全不读取，pan按“到target的距离乘常数”换算每像素位移，zoom对输入调用`signum()`并丢弃幅度。Runtime free/pan能力因此只是孤立数学模块，不是Editor可达的工程级导航系统。

相机产品状态同样不完整。投影切换只改枚举，不保持屏幕尺度或构图；平移、缩放、零delta事件和Frame Selection都会无条件把轴向视图降为`User`。Frame Selection只合并选中节点的世界位置，不读取真实render/aggregate bounds；单对象半径为零，当前相机越远越不会拉近，宽高比与near/far不参与求解。上层即使底层返回false仍显示“Framed node”。主工作台还显示硬编码`0.25`速度chip，但没有事件、设置绑定或控制器consumer。

Unreal的对应面是camera transform、orbit/fly转换、delta/impulse消费、速度范围、input key/axis/gesture、透视/正交视图、bounds/aspect/clipping-aware focus、camera lock/pilot与事务的组合系统。Godot提供可配置导航方案、modifier、mouse/gesture、cancel、inertia、freelook、FOV、auto-ortho与pilot undo；Fyrox至少将真实层级AABB、projection/aspect fit和per-scene camera persistence串入Editor；Bevy明确归一化滚轮行/像素单位，区分mouse delta与dt movement，并实现focus-aware cursor grab、touch和轴向平滑；Unity Graphics证明SceneView相机也需要per-view附加设置、有效near/far/FOV与明确apply/extract合同。

Editor58、59、30、63与Runtime37已分别拥有多视口session/currentness、通用input/capture、Camera资产/rig/director、transaction和runtime camera endpoint父问题。本报告不重复抬高其P0。本轮新增 **0项P0、23项P1、7项P2**，登记 **48个全部Fail的资格门**。目标不是重写现有三套控制器，而是补出Runtime的validated navigation input/profile/output与projection-aware framing math，以及Editor的per-view `EditorViewportCameraSession + ViewportCameraNavigationCoordinator + FrameSelectionCoordinator + CameraViewHistory + CameraPilotSession`。

本轮是review-only：未修改production Rust，未运行Cargo、真实Editor、键鼠/触控板/触摸设备、projection/framing golden、save/reopen、pilot undo、focus/capture、fault/soak/profile或同语义跨引擎benchmark；tooling按用户要求排除。当前不能声称相机导航功能、表现或性能达到或超过Unreal。

## 2. 审查边界、currentness与冻结语料

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test declarations | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Zircon Runtime navigation kernel | **20 / 1,123 / 982 / 32,715 / 7** | free/orbit/pan settings、state、input、output、dynamic adapter与tests | `f156e9f2084d2aa02ebdb59727ae6eabdfa90b27d0d0457b76ac7bb15bb6b837` |
| Zircon Editor navigation core | **13 / 1,326 / 1,203 / 46,003 / 9** | camera state、pointer input、orbit/pan/zoom、projection、alignment、frame与settings | `4018f291660cc8b9c0dc69370a88672a4c7e94632a0fc06950bc8a5ee93bd9b3` |
| Zircon Editor product route | **13 / 2,626 / 2,400 / 120,153 / 8** | command/codec/event/dispatch/workbench state、toolbar与静态speed chip | `aa0ab1ef038e9dcd848a436060f9304360fdb88a1e5d3927acf57d4d97eab250` |
| Zircon focused tests | **8 / 2,129 / 1,933 / 73,915 / 49** | controller、binding、dispatch、toolbar projection与chrome snapshot | `e1229baf7483c05aa6555d99391c2c42ced1b3c7be3477ce341499ee27987fb7` |
| Unreal selected set | **8 / 17,412 / 14,521 / 647,980 / 1 spec** | viewport camera、input/gesture、focus、settings、navigation helper、pilot与speed tests | `cc0a09978975ba2f29657752e13aa30dbfd4e7cd0a065592009038b7244ce712` |
| Godot selected set | **4 / 9,055 / 7,716 / 350,451 / 0** | navigation controller、scheme、gesture、inertia、freelook、projection、focus与pilot | `9b020bf6a0337e588ca37d2aa0469141e848b8c801a519c6c00abfef7df0da18` |
| Fyrox selected set | **4 / 1,310 / 1,193 / 46,233 / 0** | editor camera、AABB fit、projection、input、preview与per-scene persistence | `75794d1db42fc82968d0e9ed89e7eea0531158f57e1ffffeb85ecb780c32a82b` |
| Bevy selected set | **6 / 2,006 / 1,840 / 77,312 / 1** | free/pan controller、touch/grab、scroll normalization、projection与orbit/zoom examples | `5c23792ed479e27bbb7d1252c14e89737e29aecae2d8a8a5db12536169c34e14` |
| Unity Graphics selected set | **7 / 1,007 / 909 / 45,419 / 1 disabled lexical attribute** | SceneView camera creation/settings、preview、alignment、frustum validation与apply | `f109df967d51ec84628b2c5a84322456c99c8d54a8ea14fd2994d689f7e7c1fa` |

fingerprint按规范化相对路径与逐文件SHA-256基于本轮working-tree内容计算，只证明所列源码被读取；它不是ABI、artifact、动态测试或性能receipt。主仓与Unreal镜像基线为`bee4c707b714738346b49bba15c59468b8bd9b39`；Godot、Fyrox、Bevy与Unity Graphics revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`。

### 2.2 在途修改隔离

共享checkout存在大量其他Session修改。本轮54份focused Zircon文件在冻结时均为clean；相邻的viewport pointer overlay/router、renderer-visible pick source、runtime picking adapter、pointer tests与dispatch存在非本轮修改，本报告只把它们作为Editor59父边界，不把其working-tree内容计入camera navigation fingerprint，也未覆盖或回退。

coordinator Session为`optimize-editor66-viewport-camera-review-r1-20260822`，baseline epoch为339；本报告与三个共享索引取得精确lease。MVP `00-current-source-baseline-recovery`仍为`in_progress`，F4不能绕过F0-F3验收，因此本轮没有用Cargo结果包装静态审查结论。

### 2.3 范围与非范围

本报告覆盖用户在Scene Viewport内如何orbit、pan、zoom、free-fly、调整速度、切换投影、轴向对齐、Frame Selection、保存/恢复视图、使用bookmark/history以及preview/pilot一个scene camera的产品链。

多viewport实例、render surface/frame currentness归Editor58；通用pointer capture、picking、gizmo与cancel generation归Editor59；Camera资产、lens、rig、director、blend、shake、cut归Editor30与Runtime37；pilot修改scene camera时的document transaction归Editor63。Editor66只拥有camera navigation语义、controller composition、navigation preferences、framing、view history/bookmark与pilot orchestration，不重新发明这些父域的identity或transaction authority。

## 3. 当前实现拓扑与可保留基础

### 3.1 Runtime三套控制器是真实底座

`FreeCameraController`将movement axis按dt积分，区分walk/run，滚轮以指数调整speed multiplier，空输入用指数阻尼衰减velocity，并输出`CursorGrabIntent`。`OrbitCameraController`维护target并支持orbit/pan/zoom/focus。`PanCameraController`支持键盘pan、drag pan、rotation与zoom bounds。`CameraControllerOutput`能返回transform和translation/rotation/scale delta。这些都比Editor当前产品面更完整，应以补合同、补校验和统一组合为主，不应删除后重写。

### 3.2 Editor只接入orbit，free/pan处于不可达状态

`SceneViewportState`只保存`OrbitCameraController`。右键开始`ViewportDragSession::Orbit`，中键开始`Pan`，滚轮直接调用orbit zoom；正交pan/zoom又在Editor私有实现。Runtime free/pan没有Editor consumer，free controller的dt、velocity、speed multiplier与cursor grab完全没有进入Scene Viewport。

动态Runtime另有一份`RuntimeCameraController`，同样手写right/middle/scroll到orbit adapter，并直接修改active scene camera。Editor与dynamic runtime因此共享数学内核但复制输入映射、drag state、camera resolution和错误吞没，尚无统一的controller composition或typed receipt。

### 3.3 输入协议不足以描述工程级导航

`ViewportInput`只有pointer坐标/按键边沿、一个裸`f32` scroll与resize。它不能区分滚轮line和pixel、键盘连续轴、modifier、touch/pen/gesture、raw vs accelerated motion、dt/timestamp、focus/window/seat/device、cursor capture、DPI或取消原因。Scene Mode可以consume事件，但相机控制器拿不到完整输入上下文。

现有映射固定为right orbit、middle pan、scroll zoom，没有Maya/Modo/Godot/Unreal/trackpad方案、chord冲突解析、反转、灵敏度、run/slow或每设备profile。Runtime free controller已有部分字段，却没有Editor设置编译和产品入口。

### 3.4 Orbit输入宣称viewport-aware，数学实际忽略viewport

`OrbitCameraInput::with_viewport_size`会clamp并保存尺寸，但`orbit/controller.rs`没有读取`input.viewport_size`。Perspective pan使用`distance * pan_sensitivity`作为world-per-pixel，不含vertical FOV、viewport height、projection或DPI。相同屏幕拖动在不同FOV和分辨率下没有屏幕空间不变量。

Editor正交pan按`ortho_size * 2 / height`换算，方向合理，但这套数学与Runtime `PanCameraController`、共享`ViewportProjectionContext::world_units_per_pixel`并存，形成三个不一致的尺度模型。

### 3.5 滚轮幅度与设备单位被丢弃

Orbit zoom只用`delta.signum()`，Editor正交zoom也只用`signum()`和固定10%。高分辨率滚轮或触控板的`0.1`与传统滚轮的`1.0`得到相同结果，多格滚动也被压成一步。正交`delta == 0`仍返回`camera_updated = true`并把orientation改为`User`。

Bevy先把pixel scroll按`MouseScrollPixelsPerLine`归一化，再保留完整delta；Godot读取wheel factor和gesture factor。Zircon必须先定义输入单位和归一化边界，不能靠调常数修复。

### 3.6 投影、对齐与Frame Selection是分散命令，不是camera state machine

`set_projection_mode`只改settings和snapshot enum，不换算perspective distance、FOV与orthographic size，视觉尺度会跳变。`align_view`若camera尚未初始化，会从通用default snapshot构造，而不是从active scene camera继承FOV、near/far和其他lens事实。

pan、zoom、Frame Selection和零位移事件都把`ViewOrientation`设为`User`。轴向正交视图仅平移或缩放后本应仍是同一轴向视图；当前标签/state先丢失，后续auto-ortho、grid plane、bookmark或快捷切换无法依赖它。

### 3.7 Frame Selection只框位置，且上层可报告假成功

`selection_frame`只对selected node world position求min/max。它不读取mesh/renderable bounds、子树、component、gizmo sub-element或aggregate bounds；单个大mesh和单个empty的radius都为0。Perspective distance以当前offset的length作为下限，所以相机已经很远时Frame Selection不会拉近；aspect ratio、horizontal FOV、near/far与large-world clipping也不参与求解。

`EditorState::frame_selection`只要有primary selection就忽略`apply_command`的结果，设置“Framed node {id}”并返回true。若所有选中对象都缺world position或非finite，controller返回false，产品仍宣称成功。这是明确的receipt truthfulness缺陷。

### 3.8 可见速度chip没有权威

`workbench_viewport_panel.zui`显示`WorkbenchViewportSpeed`文本`0.25`，但节点没有event，`SceneViewportSettings`没有camera speed，Editor state没有free controller，Runtime speed multiplier也没有binding。它只能是静态装饰，不能作为相机速度已实现的证据。实现前应将其标为Unavailable或移除，最终由typed navigation profile snapshot驱动。

## 4. 五引擎参考证据与适用边界

### 4.1 Unreal：主参考是viewport camera系统

`FEditorViewportClient`同时处理`InputKey`、`InputAxis`、`InputGesture`、focus/mouse capture、perspective/orthographic transform、orbit/fly转换、camera controller impulse、delta-time movement与camera speed settings。`FViewportClientNavigationHelper`区分location、rotation、orbit和impulse delta，并在consume后清零，避免同一输入跨tick重复消费。

`FocusViewportOnBox`从真实`FBox`求center/radius，读取viewport尺寸与aspect，按FOV求perspective距离；orthographic路径设置zoom、调整near/far并可transition。Level viewport的actor/camera lock保存pre-pilot camera transform与pilot transform，移动locked actor，维护transaction、camera cut与invalidation。其价值在完整生命周期和事实边界，不要求复制Unreal类层级。

`CameraSpeedSettingsTests`验证current speed、absolute range、UI range互相约束。Zircon当前没有对应validated range或产品绑定。

### 4.2 Godot：输入方案、惯性、cancel与pilot是一体的

`View3DController`定义navigation mode/scheme/mouse button、zoom style、freelook scheme、view type和orthogonal mode。它保存immediate/interpolated/previous cursor，支持right-click或Escape取消并恢复previous cursor，处理mouse motion、wheel factor、magnify/pan gesture、FOV快捷键、axis view、freelook与mouse capture/warp。

Editor settings公开orbit/translation/zoom inertia、sensitivity、invert、navigation scheme、button/modifier、freelook speed/sensitivity/scheme和auto orthogonal。进入/退出freelook会同步referential并恢复鼠标位置；camera preview/pilot会把编辑器camera运动写回preview camera，并以idle合并undo。Zircon应学习状态守恒与设置编译，而不是照搬Godot常数。

### 4.3 Fyrox：Rust Editor的最小工程基线仍高于当前产品面

Fyrox `CameraController`组合pivot/hinge/camera，支持centered rotation、orbit、drag、键盘六向移动、speed up/slow down、perspective/orthographic分支与dt update。`fit_object`遍历目标及descendants合并真实AABB，拒绝degenerate bounds后使用camera `fit`并传入render target aspect，分别返回perspective distance或orthographic position/vertical size。

交互结束后它按scene path保存position、yaw、pitch与projection；camera preview还显式建立/释放render target和override。Fyrox并不完美，但证明Rust实现无需牺牲真实bounds、projection fit和持久化边界。

### 4.4 Bevy：输入单位与时间语义必须明确

Bevy first-party free camera将静态config与动态state分开，支持walk/run、指数scroll speed、friction、axis snap、touch、focused window cursor grab和disable时release。它明确mouse delta已是本帧累计位移，不再乘dt；键盘velocity才按dt积分。pan camera的mouse drag通过`world_to_viewport -> viewport_to_world_2d`保持屏幕空间语义。

Projection API以viewport resize更新aspect/orthographic area，支持多种orthographic scaling mode。projection zoom example保留完整scroll delta并对范围clamp。Zircon可直接借鉴单位与验证原则，但Bevy controller本身不是完整Editor产品。

### 4.5 Unity Graphics：只取SceneView附加设置与frustum事实

Graphics仓不是Unity Editor核心源码，不能用来证明Unity完整导航实现。可用证据是HDRP/URP在`SceneView.onCameraCreated`上为每个SceneView camera附加pipeline data，HDRP settings通过linked component apply并触发repaint；camera preview先校验viewport尺寸与render target。

`CameraSettings.Frustum`区分raw/effective near/far，约束minimum clip distance与FOV，并以明确mode计算或使用projection matrix；`ApplySettings`集中应用frustum/culling/frame/volume事实。其测试中的`[Test]`已被注释，因此只能作为源码设计证据，不能算通过的动态资格。

## 5. 差异矩阵

| 能力 | Zircon current source | Unreal / 其他参考 | 结论 |
|---|---|---|---|
| Controller底座 | Runtime有free/orbit/pan，Editor只接orbit | 组合orbit/fly/pan/gesture与per-view state | 应保留内核，补统一Editor composition |
| 输入身份 | 裸pointer/scroll/resize | key/axis/gesture/device/focus/timestamp/capture | 输入协议不完整 |
| 单位 | scroll `f32`且后续`signum` | line/pixel/factor归一化并保留幅度 | 高精度输入语义丢失 |
| 时间 | Editor无dt/continuous axis | mouse delta与dt movement明确分流 | free-fly不可产品化 |
| 配置 | hard-coded按键与常数 | scheme/chord/invert/sensitivity/inertia/speed | 无navigation profile |
| 屏幕尺度 | orbit忽略viewport/FOV | projection-aware world-per-pixel | pan不具跨viewport不变量 |
| 投影切换 | 只改enum | perspective/ortho transform与scale conversion | 构图跳变 |
| 轴向视图 | pan/zoom/frame即变User | view type与auto-ortho持续维护 | state语义错误 |
| Frame Selection | world position AABB | render bounds、aspect、FOV、ortho、clipping | 大对象/单对象fit错误 |
| 结果 | bool `camera_updated` | consumed deltas、typed state/transition/invalidation | 无可诊断receipt |
| 持久化 | 只存projection/orientation等chrome | per-scene/per-view camera/profile/history | camera session不可恢复 |
| Bookmark/history | 无 | view history/bookmark/preset | 重复导航工作流缺失 |
| Preview/pilot | 无camera binding session | preview/lock/pilot、restore、transaction | 无可逆场景相机编辑 |
| 产品UI | toolbar有frame/projection/align；speed为固定0.25 | 控件绑定真实settings/state | 产品真实性不足 |
| Tests | happy path与静态binding为主 | range、cancel、fit、projection、input state | 关键状态机无RED |

## 6. 新增发现

### 6.1 P1：架构、正确性与产品闭环

#### ED66-P1-01：Editor只接入Orbit，Runtime Free/Pan没有产品consumer

`SceneViewportState`只能保存orbit controller，导致Runtime已实现的dt movement、walk/run、speed scaling、friction、keyboard pan/rotation和cursor grab均不可达。应通过per-view navigation coordinator组合现有controller，而不是在Editor继续复制数学。

#### ED66-P1-02：`ViewportInput`不能表达工程级导航输入帧

协议缺keyboard axis、modifier、dt/timestamp、focus、window/seat/device、DPI、raw motion、gesture/touch/pen、capture状态和terminal reason。应由host把平台事件归一为qualified `CameraNavigationInputFrame`，Scene Mode和camera navigation基于同一frame做admission与consume。

#### ED66-P1-03：按键方案与chord被硬编码，无法配置或检测冲突

right/middle/scroll固定映射没有scheme、binding key、priority、invert、run/slow或temporary override。需要versioned `CameraNavigationProfile`和与Editor08 keymap authority的link/冲突验证，不能再新增另一套UI私有快捷键表。

#### ED66-P1-04：focus与cursor-grab生命周期不守恒

Runtime free controller的`focus_active`只阻止look，仍会在失焦时处理movement axis、scroll speed和velocity translation；focus loss且`cursor_grab_changed == false`也不会请求release。Editor又没有focus/capture输入。必须把focus loss、window replacement、device disconnect、Escape和controller disable定义为可复验terminal transition。

#### ED66-P1-05：Orbit的`viewport_size`是死字段，perspective pan尺度错误

调用方传入viewport size，controller却完全不读取；world-per-pixel只由distance和常数决定。应输入validated projection metrics，以`2 * distance * tan(fov_y/2) / viewport_height`或等价ray-plane求解建立屏幕空间不变量，并对orthographic复用同一尺度合同。

#### ED66-P1-06：scroll `signum()`破坏幅度、设备与零输入语义

Orbit和Editor orthographic zoom都丢弃delta magnitude，不能区分pixel/line/gesture、多格滚动或精细输入；orthographic零delta还报告changed并破坏orientation。必须先标准化单位，再使用连续、可clamp的指数/对数尺度函数，并保证零输入严格无副作用。

#### ED66-P1-07：camera navigation settings/state缺finite与range验证

orbit sensitivity、target、pitch、min distance、zoom fraction、free speed/friction/dt、pan zoom range和camera FOV/near/far都可直接进入浮点数学。需要compile/validate阶段和runtime repair/reject reason；NaN、inf、负viewport、极端dt与逆序范围不能污染transform或velocity。

#### ED66-P1-08：没有统一的惯性、平滑与可中断transition模型

Runtime free只有velocity damping，orbit/pan/zoom与align/frame均瞬时跳变；不同路径没有统一time source、cancel或retarget。应建立可选、dt-stable、可中断的camera transition state，pointer direct manipulation与programmatic frame/align分别采用明确policy。

#### ED66-P1-09：camera navigation state没有per-view session identity

camera、orbit target、controller state和drag都在单一`SceneViewportState`中，不能与多viewport实例、document generation、focus owner或render surface currentness绑定。Editor58拥有session registry父合同；Editor66必须定义其camera/navigation payload和replace/reset规则，不得另建平行viewport identity。

#### ED66-P1-10：投影切换不保持屏幕尺度、构图或有效clip事实

Perspective/Orthographic只切enum，未从distance/FOV推导ortho size，也未从ortho size恢复perspective distance；near/far、aspect和pivot不复验。应由pure projection transition solver返回before/after lens、transform、pivot与warnings，并支持round-trip tolerance。

#### ED66-P1-11：轴向对齐依赖generic defaults，且导航错误丢失view type

camera未初始化时`align_view`使用default snapshot而非scene camera lens。pan、zoom、frame甚至零delta都会把orientation设为User，轴向视图身份不可靠。只改变rotation/orbit的实际输入才应离开axis view；pan/zoom/focus应保持orientation，axis preset与auto-ortho需有明确policy。

#### ED66-P1-12：Frame Selection没有读取真实aggregate bounds

只合并node position会把单mesh、子树、skinned object、particle、volume、gizmo sub-element和多component对象视为零尺寸或错误尺寸。应从Runtime bounds/inspection authority取得generation-qualified aggregate bounds，并定义hidden、locked、unloaded、invalid和mixed selection政策。

#### ED66-P1-13：Frame Selection保留旧远距离，无法可靠“框选”单对象

distance以当前offset length为下限，远处相机不会拉近；单对象radius为0后退到固定6单位，实际大小完全不参与。solver应从bounds、projection、padding与view direction计算目标distance/ortho extent，不把旧距离当硬下限。

#### ED66-P1-14：Frame Selection忽略宽高比、horizontal FOV与clipping

当前只看vertical FOV，窄viewport下横向超界；near/far不随bounds或large-world position调整，也无不可见/behind-camera处理。需要同时满足horizontal/vertical fit，并返回clipping adjustment或typed rejection。

#### ED66-P1-15：Frame Selection上层可在失败时发布成功状态

`EditorState::frame_selection`忽略controller bool/Result，只要有primary selection就显示成功。必须返回typed `FrameSelectionReceipt`，只有`Applied`才能更新status、history和repaint；`NoBounds`、`StaleSelection`、`InvalidLens`等要明确展示或诊断。

#### ED66-P1-16：没有camera view history与bookmark工作流

用户不能back/forward、保存/命名/删除视图、回到frame前状态或在project/user scope恢复。需要stable bookmark id、bounded history、projection/lens/pivot/orientation快照与迁移；Editor58仍拥有per-view storage位置，Editor66拥有camera payload和行为。

#### ED66-P1-17：没有preview/pilot/possess scene camera的可逆session

Viewport camera不能绑定Camera node、预览其lens、进入pilot后把导航写回目标并退出恢复原编辑器视图。需要generation-qualified target、pre-pilot snapshot、external target change处理、transaction merge、undo/redo、target deletion/unload和camera cut/invalidation语义；scene mutation必须经Editor63 transaction。

#### ED66-P1-18：Editor与dynamic Runtime复制adapter并采用不同camera ownership

两处都手写drag state和right/middle/scroll映射；dynamic Runtime直接改active camera，Editor维护local snapshot。应共享validated navigation kernel和input semantics，但保留不同owner：Runtime gameplay/debug controller修改runtime endpoint，Editor默认只修改editor camera，除非显式pilot。

#### ED66-P1-19：`CameraControllerOutput`与`ViewportFeedback`不足以作为产品receipt

Runtime output只有transform delta、changed和可选cursor intent；Editor进一步压成`camera_updated: bool`。它不能表达consumed input、active mode、pivot/projection/speed变化、transition、capture owner、invalidation scope、rejection或warning。需要typed output/receipt，presentation只投影receipt，不从bool猜终态。

#### ED66-P1-20：可见`0.25`速度chip是无consumer的静态产品声明

它没有event、binding、settings字段或controller连接。M0应立即标为Unavailable或移除；真实控件必须绑定当前view的validated speed multiplier/range，支持keyboard/wheel调整并反映controller receipt。

#### ED66-P1-21：navigation preferences没有版本化authority和作用域

`SceneViewportSettings`只存transform/projection/display/grid等chrome，不含scheme、sensitivity、invert、speed、friction、zoom/pan policy、axis mode或transition。需要user/project/view override层级、schema version、default source、migration和immutable resolved snapshot。

#### ED66-P1-22：测试只证明happy path和route存在，不覆盖导航状态机

现有测试验证一次free/orbit/pan结果、idle source guard、基本perspective navigation、cancel和toolbar binding；没有不同viewport/FOV不变量、scroll unit、zero/NaN/inf、focus loss、capture、keyboard fly、gesture、projection round-trip、真实bounds frame、false receipt、bookmark/pilot或save/reopen。必须先补RED矩阵。

#### ED66-P1-23：没有large-world与scene-scale导航模型

速度、min/max distance、near/far与frame fallback都是固定世界单位；没有distance-scaled speed、world origin/rebase、高精度camera anchor、极近/极远scene或clip precision策略。要优于Unreal，必须先定义多数量级场景下的精度、速度和clipping不变量，再以相同场景测试。

### 6.2 P2：质量、可维护性与资格证据

#### ED66-P2-01：camera magic constants分散且语义重复

Runtime和Editor分别维护sensitivity、distance floor、zoom fraction、ortho size、frame distance/padding等常数。它们应进入validated defaults/profile或pure solver private constants，并为每个值记录单位和作用域；不能继续靠跨文件手调。

#### ED66-P2-02：没有结构化navigation diagnostics

应按view/session记录input source、controller mode、dt、normalized delta、speed、pivot、projection transition、frame rejection和capture terminal reason，并有隐私/采样/容量边界。diagnostic不能替代receipt，但必须能解释“相机为什么没动/跳了”。

#### ED66-P2-03：没有hot-path、soak与allocation预算

需定义60/120/240Hz input-to-camera p50/p95/p99、每帧allocation、controller update CPU、history memory和8小时soak漂移预算；Frame Selection还需1/1K/100K selection bounds聚合预算。未取得profile前不能声称性能优于Unreal。

#### ED66-P2-04：缺少keyboard-only、touch/pen与reduced-motion产品政策

导航必须可由keyboard完成核心操作，gesture/touch/pen要有明确mapping，focus/capture提示要可访问；programmatic align/frame transition应尊重reduced-motion。它们应消费同一navigation profile，不是额外旁路。

#### ED66-P2-05：profile、bookmark与history缺schema迁移/容量策略

需要stable id/version、max entries、LRU或显式保留、corruption recovery、missing camera target处理、project/user scope和跨版本migration。失败或取消的navigation不得污染history/bookmark recentness。

#### ED66-P2-06：缺少确定性输入trace与golden replay

应能记录归一化input frame、initial state和expected receipt，在Windows设备矩阵与headless math test中重放。golden需覆盖不同frame rate、event batching、scroll pixel/line和focus interruption，避免手工试用成为唯一证据。

#### ED66-P2-07：缺少同语义跨引擎表现/性能基线

最终比较必须固定scene bounds、viewport、FOV、输入轨迹、frame rate、硬件、warm-up和统计口径，分别比较orbit/pan/zoom/fly/frame/pilot。先达到功能同构，再谈超过Unreal；不同功能集或只比较平均帧率无效。

## 7. 目标架构与职责边界

### 7.1 Runtime：保留控制器，补统一数学与验证合同

Runtime建议收敛为以下可复用值与pure services：

- `CameraNavigationInputFrame`：已经归一化的pointer/keyboard/gesture delta、dt、focus/capture事实与unit tag，不含Editor document/view identity。
- `CameraNavigationProfile`与`ValidatedCameraNavigationProfile`：controller kind、scheme-independent sensitivity/speed/invert/inertia/range；构造时验证finite、range和单位。
- `CameraProjectionMetrics`：projection、FOV/ortho extent、aspect、viewport extent、near/far，供pan/zoom/frame共享。
- 保留`FreeCameraController`、`OrbitCameraController`与`PanCameraController`，但统一zero-input、focus、numeric repair和output semantics。
- `CameraNavigationOutput`：transform、pivot、projection/lens delta、speed、cursor intent、consumed/changed、transition与typed rejection。
- `CameraFramingSolver`：消费validated bounds、lens、viewport和padding policy，返回perspective/orthographic target state；不读取Editor selection。

Runtime scene/inspection负责真实aggregate bounds和camera endpoint事实。Runtime不能认识Editor toolbar、document、bookmark或undo，也不能因Editor导航默认修改scene camera。

### 7.2 Editor：per-view camera session与产品编排

Editor58的`ViewInstanceId`/session registry落地后，每个view持有：

- `EditorViewportCameraSession`：editor camera state、pivot、projection/lens、active controller、transition、history cursor与generation。
- `ViewportCameraNavigationCoordinator`：把qualified host input和resolved profile编译为Runtime input frame，处理capture/focus/cancel与receipt publication。
- `CameraNavigationPreferenceStore`：user/project/view override、migration、device profile与keymap link。
- `FrameSelectionCoordinator`：冻结selection generation，请求Runtime aggregate bounds，调用pure solver，复验view/document generation并发布typed receipt。
- `CameraViewHistory`与`CameraBookmarkStore`：bounded back/forward与命名持久视图，不进入Scene undo。
- `CameraPilotSession`：绑定scene camera endpoint、保存pre-pilot editor view、按Editor63 transaction写回target，并处理delete/unload/reload/undo/cut。

### 7.3 输入到渲染的时序

`platform event -> qualified host input -> Editor08 keymap/chord resolution -> active ViewInstanceId/capture admission -> normalized CameraNavigationInputFrame -> selected Runtime controller -> CameraNavigationOutput -> Editor session generation commit -> typed repaint/invalidation receipt -> renderer consumes same-generation camera snapshot`。

零输入、stale view、focus loss、invalid profile或invalid lens不得修改session。直接pointer manipulation可即时响应；align/frame transition必须可中断，新的用户输入抢占旧transition并留下唯一terminal receipt。

### 7.4 视图状态与Scene transaction分离

普通orbit/pan/zoom/fly、projection、bookmark和history属于Editor view state，不污染Scene dirty或authoring undo。只有pilot明确绑定scene camera后，target transform/lens修改才成为document transaction。退出pilot恢复Editor view，但不偷偷回滚已经提交的scene camera transaction。

## 8. 分阶段重构计划

### ED66-M0：真实性止血与RED基线

移除/禁用无consumer的`0.25` speed chip；修复Frame Selection false success与zero-delta副作用；为scroll magnitude、focus loss、projection round-trip、bounds frame、stale view和pilot写RED tests。冻结Editor58/59依赖接口。

### ED66-M1：Runtime input/profile/output与numeric hardening

定义normalized input frame、validated profile、projection metrics和typed output；修复free focus/grab、orbit viewport dead field、scroll unit与finite/range guard。保留三套controller API的清晰职责并迁移tests。

### ED66-M2：Editor per-view navigation session与free-fly接入

在Editor58 session identity上挂camera session，接入keyboard/mouse free-fly、speed/run/slow、cursor capture与terminal cancel。删除Editor和dynamic runtime重复输入语义，只保留各自owner adapter。

### ED66-M3：Projection、axis alignment与Frame Selection solver

实现perspective/orthographic视觉尺度保持、axis/auto-ortho policy、真实aggregate bounds、aspect/FOV/clipping-aware fit、可中断transition和typed frame receipt。所有camera commands复验view generation。

### ED66-M4：Navigation profile、keymap与产品控件

建立versioned preference resolution，接Editor08 keymap/chord，提供scheme、speed、sensitivity、invert、inertia、zoom/pan policy和accessibility controls。toolbar/chrome只展示同代snapshot，不保留静态文本authority。

### ED66-M5：History、bookmark与持久化

实现bounded back/forward、bookmark CRUD、per-user/per-project作用域、save/reopen、migration/corruption recovery和missing target policy。多view互不串写，history只记录成功terminal state。

### ED66-M6：Camera preview/pilot与transaction

建立preview-only和pilot两种绑定，保存pre-pilot view，复验camera endpoint generation，按Editor63合并scene transform/lens transaction，处理target deletion、external edit、undo/redo、document replace、cut与exit restore。

### ED66-M7：规模、故障、表现与跨引擎资格

运行设备矩阵、deterministic trace replay、8小时soak、large-world/多数量级场景、100K selection bounds、60/120/240Hz profile与fault injection；达到功能同构后，以固定输入轨迹与场景对Unreal/Fyrox/Godot/Bevy进行可复现实测。

## 9. 资格门

以下门在本轮全部为Fail；存在类型名或局部函数不计通过。

| Gate | 要求 | 当前 | 必需证据 |
|---|---|---|---|
| ED66-G01 | input frame显式区分pointer delta、continuous axis与gesture | Fail | schema/unit tests |
| ED66-G02 | scroll pixel/line/factor统一归一且保留幅度 | Fail | device conversion matrix |
| ED66-G03 | mouse delta不乘dt、continuous movement按dt积分 | Fail | frame-rate invariance tests |
| ED66-G04 | focus/window/seat/device/capture identity可复验 | Fail | qualified input tests |
| ED66-G05 | zero input严格不改transform/pivot/orientation/history | Fail | zero property tests |
| ED66-G06 | NaN/inf/负dt/极端delta Fail closed | Fail | numeric fuzz/property tests |
| ED66-G07 | profile compile验证range、unit与finite | Fail | invalid profile matrix |
| ED66-G08 | controller output含typed consumed/changed/rejection | Fail | receipt contract tests |
| ED66-G09 | Editor真实接入orbit/pan/free-fly | Fail | product integration tests |
| ED66-G10 | controller切换保持camera transform/pivot守恒 | Fail | mode round-trip tests |
| ED66-G11 | focus loss总能release cursor并终止/暂停运动 | Fail | host focus tests |
| ED66-G12 | Escape/window replace/device disconnect有唯一terminal receipt | Fail | interruption matrix |
| ED66-G13 | keymap/chord/scheme来自同一resolved profile | Fail | keymap link tests |
| ED66-G14 | binding冲突与不可用输入给出typed reason | Fail | conflict/admission tests |
| ED66-G15 | pan在不同viewport/FOV下保持屏幕空间不变量 | Fail | projection golden tests |
| ED66-G16 | speed/run/slow/scroll调整真实驱动当前view controller | Fail | product state tests |
| ED66-G17 | perspective/orthographic切换保持视觉尺度 | Fail | projection round-trip golden |
| ED66-G18 | projection切换复验FOV/aspect/near/far | Fail | invalid lens tests |
| ED66-G19 | axis align继承当前camera lens而非generic default | Fail | cold-session align tests |
| ED66-G20 | pan/zoom/frame保持有效axis view identity | Fail | view-state transition tests |
| ED66-G21 | axis snap/transition可中断且可retarget | Fail | transition state tests |
| ED66-G22 | Frame Selection使用真实aggregate bounds | Fail | mesh/subtree/component bounds tests |
| ED66-G23 | Frame同时满足horizontal/vertical FOV与aspect | Fail | wide/tall viewport golden |
| ED66-G24 | Frame处理orthographic、near/far与large-world clipping | Fail | projection/clipping matrix |
| ED66-G25 | camera state绑定ViewInstanceId与session generation | Fail | multi-view identity tests |
| ED66-G26 | stale input/output不能提交到replacement view | Fail | replacement race tests |
| ED66-G27 | 多view camera/pivot/controller state完全隔离 | Fail | concurrent viewport tests |
| ED66-G28 | profile override有user/project/view明确优先级 | Fail | resolution tests |
| ED66-G29 | settings/profile save-reopen与migration守恒 | Fail | persistence round-trip |
| ED66-G30 | history back/forward有界且只记成功terminal state | Fail | history state tests |
| ED66-G31 | bookmark CRUD使用stable id并可迁移 | Fail | bookmark repository tests |
| ED66-G32 | corruption/missing target恢复有typed disposition | Fail | fault/migration tests |
| ED66-G33 | Frame failure不会发布成功status或history | Fail | product receipt test |
| ED66-G34 | toolbar/chrome没有无consumer speed/static authority | Fail | source guard + UI integration |
| ED66-G35 | camera command返回typed receipt与invalidation scope | Fail | dispatch contract tests |
| ED66-G36 | preview camera绑定不修改Scene或dirty | Fail | preview isolation tests |
| ED66-G37 | pilot target使用qualified endpoint generation | Fail | stale/delete target tests |
| ED66-G38 | pilot保存并恢复pre-pilot editor view | Fail | enter/exit round-trip |
| ED66-G39 | pilot scene mutation进入document transaction | Fail | undo/redo integration |
| ED66-G40 | external target edit/cut/document replace有明确策略 | Fail | lifecycle race matrix |
| ED66-G41 | controller unit tests覆盖range、invalid和focus | Fail | Runtime test suite |
| ED66-G42 | Editor tests覆盖keyboard/mouse/gesture/device matrix | Fail | host input suite |
| ED66-G43 | normalized trace可确定性重放 | Fail | golden replay suite |
| ED66-G44 | 60/120/240Hz结果与手感指标在容差内 | Fail | frame-rate profile |
| ED66-G45 | 8小时soak无transform/velocity/history漂移 | Fail | soak artifact |
| ED66-G46 | 100K selection Frame满足CPU/allocation预算 | Fail | scale profile |
| ED66-G47 | large-world极近/极远导航满足精度与clip预算 | Fail | large-world golden/profile |
| ED66-G48 | 同语义跨引擎功能/表现/性能receipt可复现 | Fail | fixed-scene Unreal/Fyrox/Godot/Bevy benchmark |

## 10. 测试与验证矩阵

### 10.1 Runtime unit / property / fuzz

- input unit conversion、zero/NaN/inf/extreme dt、profile compile、pitch/distance/speed/zoom range；
- orbit/pan/free controller的frame-rate、viewport/FOV、focus/capture与mode switch不变量；
- projection transition和framing solver的perspective/orthographic/aspect/clipping golden；
- randomized input trace不得生成non-finite transform、unbounded speed或重复terminal receipt。

### 10.2 Editor integration / product

- multi-view/per-document隔离，view replacement与stale input拒绝；
- keyboard fly、mouse orbit/pan、wheel/trackpad/gesture zoom、run/slow和cursor capture；
- toolbar speed/profile、projection、align、Frame Selection的真实state/receipt/repaint链；
- history/bookmark save/reopen/migration，以及Frame failure不显示成功；
- preview/pilot enter/update/undo/redo/target delete/document replace/exit restore。

### 10.3 Fault / scale / performance

- device disconnect、focus loss、window close、plugin/provider revoke、invalid lens、bounds unavailable与settings corruption；
- 1/1K/100K selection bounds、1/4/16 active view、60/120/240Hz输入、8小时soak与large-world；
- CPU、allocation、input-to-camera latency、repaint/invalidation、history memory与capture release trace；
- 固定scene/input trace在Unreal、Godot、Fyrox、Bevy/Zircon上的功能、表现与性能对照。

### 10.4 本轮未运行

本轮未运行Cargo、真实Editor、native input设备、projection/framing render golden、save/reopen、pilot、fault、soak、profile或跨引擎benchmark。原因不是以静态检查代替验证，而是MVP baseline recovery仍未完成且本轮被限定为review-only。

## 11. Owner路由与非重复计数

| 父问题 | canonical owner | Editor66处理方式 |
|---|---|---|
| ViewInstanceId、multi-viewport、frame currentness、camera isolation | Editor58 | 依赖其session identity，只定义camera payload |
| pointer capture、picking、gizmo、cancel generation | Editor59 | 复用capture lifecycle，不重复P0 |
| Camera asset/lens/rig/director/blend/shake/cut | Editor30、Runtime37 | pilot只绑定endpoint，不重建camera domain |
| document transaction/history/qualified target | Editor63 | pilot scene mutation接入其authority |
| command registry/keymap/chord | Editor08 | navigation profile引用其binding identity |
| Runtime World bounds/inspection | Runtime99k、Runtime111 | Frame请求真实bounds，不在Editor猜mesh |

本报告新增问题只统计camera navigation自身差距。Editor58关于per-view persistence/bookmark、Editor59关于capture、Editor30关于pilot、Editor63关于transaction identity的既有条目继续由父报告唯一计数；这里的gate只把它们写成Editor66交付依赖。

## 12. 最终判定

当前实现可判定为“存在可复用camera controller primitives与最小Scene Viewport orbit产品链”，不能判定为“工程级Editor camera navigation”。最危险的不是缺少动画，而是输入单位、focus/capture、projection/framing、per-view identity和产品receipt没有形成同一状态机；这会让后续新增fly、bookmark或pilot继续堆出并行实现。

正确重构顺序是先止血假speed/假Frame成功和zero-delta副作用，再补Runtime validated input/profile/output与projection-aware math，随后在Editor58/59身份上接通per-view free/orbit/pan，最后实现history/bookmark/pilot和性能资格。完成48门前，不得把静态chip、存在类型名、一次happy-path测试或手工“能转动”宣传为相机导航完成，更不得宣称表现或性能超过Unreal。
