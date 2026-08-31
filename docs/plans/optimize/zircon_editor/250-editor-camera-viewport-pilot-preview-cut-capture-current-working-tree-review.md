---
title: Editor Camera、Viewport、Pilot、Preview、Cut 与 Capture 当前工作树工程化差距
category: zircon_editor
report_id: Editor250
review_date: 2026-08-30
baseline_head: working-tree
related_code:
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/scene/viewport/controller
  - zircon_editor/src/scene/viewport/projection.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/build.rs
  - zircon_editor/src/ui/binding/viewport/command.rs
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_editor/src/core/play/simulate_camera.rs
  - zircon_editor/src/ui/host/editor_host_event_controller/simulate_camera.rs
  - zircon_editor/src/ui/retained_host/app/simulate_camera_sync.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/tests/editing/state/camera_authority.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/104-editor-camera-rig-director-blend-shake-cinematic-cut-current-source-review.md
  - docs/plans/optimize/zircon_editor/151-editor-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-current-source-review.md
  - docs/plans/optimize/zircon_editor/166-editor-cinematic-sequencer-shot-track-binding-take-recorder-movie-render-queue-current-source-review.md
  - docs/plans/optimize/zircon_editor/179-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-current-source-review.md
  - docs/plans/optimize/zircon_editor/187-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/190-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/222-editor-cinematic-sequencer-shot-track-binding-take-recorder-movie-render-queue-current-source-review.md
  - docs/plans/optimize/zircon_editor/249-editor-material-shader-graph-instance-toolkit-preview-compiler-current-working-tree-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorViewportClient.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorViewportClient.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/SLevelViewport.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/SActorPilotViewportToolbar.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Public/LevelEditorCameraEditorState.h
  - dev/godot/editor/scene/3d/node_3d_editor_plugin.h
  - dev/godot/editor/scene/3d/node_3d_editor_plugin.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.cpp
  - dev/godot/editor/scene/3d/gizmos/camera_3d_gizmo_plugin.cpp
  - dev/Fyrox/editor/src/camera/mod.rs
  - dev/Fyrox/editor/src/camera/panel.rs
  - dev/Fyrox/editor/src/settings/camera.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Editor/Camera/UniversalRenderPipelineCameraEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Editor/Camera/UniversalRenderPipelineCameraUI.Drawers.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Editor/Camera/UniversalRenderPipelineCameraUI.PhysicalCamera.Drawers.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/RenderPipeline/Camera/HDCameraPreview.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/RenderPipeline/Camera/HDCameraEditor.cs
doc_type: current_working_tree_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Editor Camera、Viewport、Pilot、Preview、Cut 与 Capture 当前工作树工程化差距

## 1. 结论

当前 Editor 已有一套真实的 Scene viewport 交互底座：`SceneViewportController` 拆分了 camera、navigation、pointer、overlay、selection、gizmo、cancel 与 transaction 文件；`ViewportCameraSnapshot` 支持透视/正交、aspect、projection override、动态分辨率和 temporal jitter；projection helper、frame selection、focus-loss release、input replay、multi-selection transform 与 active-camera resync 都有局部测试。Retained host 也能在 Simulate 模式把 Editor camera 编码为 bounded `ZrRuntimeViewportCameraV1`，并在 Runtime 侧覆盖 extract 而不修改 Play World。

但这仍是“可导航的 Scene viewport”，不是工程级 Camera authoring 产品。本轮 Editor 选集为 **43 个文件、8,953 行、325,131 bytes、86 个 test marker**；五引擎 Editor/graphics 参考为 **12 个文件、18,357 行、674,010 bytes**。Editor state 只有一个 `Option<ViewportCameraSnapshot>` 和一个 `OrbitCameraController`；`SetProjectionMode` 会直接改 transient camera，不区分 Scene source、Preview、Game View 和 Pilot。Camera component projection 中只有 FOV、Near Clip、Far Clip 三个反射字段，完整 target/stack/clear/layers/HDR/exposure 等字段无法通过 Inspector 修改。没有 Camera asset toolkit、Lens/Rig/Shake editor、View Through/Pilot/Lock、Bookmark/Camera Editor State、Camera Preview Session、Camera Debugger、typed Camera Cut 或 Capture Camera session。

Editor250 继承 Editor30/104/151 的 **5 项父 P0 全部 Open**，不重复计数；本轮登记 **34 项 Editor 专属 P1（29 Open / 5 Partial）、12 项 P2（12 Open）和 28 个资格门（23 Fail / 5 Partial / 0 Pass）**。Tooling 按用户要求排除。本轮只写 review/index/coverage，不修改 Editor/Runtime/plugin/Cargo/ABI/ZUI。

## 2. 审查边界与证据

### 2.1 纵向检查链

本轮沿 `ViewportCommand -> SceneViewportController state/input -> projection/render packet/picking/gizmo -> Inspector reflection -> Scene transaction -> retained host surface -> Simulate gateway -> Runtime ViewResult/history -> Sequencer/capture` 逐层检查。通用 viewport host、Scene document 和 Material preview 只作为可复用基础，不被误判为 Camera 产品实现。

### 2.2 证据等级

- **E3**：当前工作树 Editor production Rust、ZUI navigation、Simulate route 和 camera-authority tests 已读取。
- **E2**：对照 Editor187/222/249 与 Unreal LevelEditor/EditorViewport、Unity URP/HDRP camera editor、Godot Node3D editor/gizmo、Fyrox camera panel/settings。
- **E1**：测试只证明局部数学/状态；本轮没有运行 Editor host、UI automation、PreviewWorld、PIE、Sequencer、capture、fault、scale、soak 或 visual benchmark。
- **E0**：没有证据支持 Editor 工作流、画面或性能超过 Unreal/Unity；本报告只登记结构性缺口与验收合同。

## 3. 当前可保留底座

1. `SceneViewportController` 的文件拆分和 `SceneViewportState` 中 settings/selection/mode/viewport/camera/drag/hover 分层可保留；后续应将 camera session 从单一 state slot 提升为 per-view session，而不是把更多模式塞回 controller。
2. `ViewportCameraSnapshot` 与 shared projection context 已统一屏幕投影、world ray、frustum 计算入口；应让它消费 Runtime qualified view，而不是另建一套物理镜头矩阵。
3. `camera_authority` 测试证明 active camera transform、父级变换、undo/redo 与 unrelated transform 不会错误覆盖导航 snapshot；这是 Scene authoring/resync 的正确性基础。
4. Viewport command 已包含 resize、projection、align、frame selection、display/grid/preview lighting/skybox/gizmo 等显式命令，并有 focus-loss release 与 interaction cancel；可作为 camera interaction command registry 的初始来源。
5. Simulate preview camera 具备 mode/instance 检查、bounded JSON、finite/range 验证、last-value coalescing 和不修改 Play World 的行为测试；应扩展为 generation-qualified camera session，而不是删除 bridge。
6. Editor179 的 Scene/Simulate/Game surface 分离、Play preview frame gateway/instance/size/generation identity、Runtime submission receipt 可作为 Camera Preview 与 Capture 结果的 surface owner。

## 4. 当前实现事实与断路

### 4.1 Viewport camera state 与输入

1. `SceneViewportState.camera` 只有一个 transient snapshot；未初始化时从 `scene.active_camera()` 构造。不存在 ViewId、document/world generation、camera purpose、owner lease、history epoch 或 per-pane state。
2. `SceneViewportController::set_projection_mode` 同时修改 `settings.projection_mode` 与 transient camera；它不区分“编辑器导航 projection”与“Scene Camera source projection”，也不产生 transaction/diagnostic。
3. navigation 只使用 Orbit/Pan/Zoom 数学：Perspective pan 依赖 Orbit controller，Orthographic pan 使用固定像素比例，zoom 丢弃滚轮幅度并使用固定 `0.1` factor；Fly、dolly、truck、pedestal、roll 和 physical camera controls 不存在。
4. `frame_selection`、align 和 reset 依赖 active scene camera/selection 的单一上下文；目标缺失、camera invalid、world replaced 或 multi-world 时没有 typed unavailable result。
5. Viewport settings 中 speed/sensitivity/profile、bookmark、camera path、per-layout persistence、per-user preferences 没有 owner；Fyrox/Unreal 的 camera settings/state persistence 没有对应产品链。

### 4.2 Scene Camera authoring 与 Inspector

1. edit-mode projection 将 Camera component 映射为三个字段：FOV Y、Near Clip、Far Clip；`core_pipeline`、projection mode、ortho size、target、viewport、order、active、HDR、exposure、clear、MSAA 与 post-process均被 reflection skip。
2. Scene transaction 能创建/删除 camera node 和写 transform，删除 subtree 会做 camera count preflight；但 Camera-specific property command、conditional projection layout、unit/range、multi-edit、stack membership 和 clear/layer authoring不存在。
3. Editor camera gizmo 只在 render packet 中生成 icon/pick/frustum 近似；没有 orthographic extent、filmback/sensor、focus plane、boom/spring-arm、safe frame、composition guide、show flag、selected/camera generation 或 draw budget。
4. Scene viewport render packet 的 preview lighting/skybox/display/grid 是 Editor presentation settings，不是 Camera source/PreviewWorld environment；无法验证最终 runtime material/post-process/volume/camera settings。

### 4.3 Simulate、Play、Preview 与 surface

1. `simulate_preview_camera` 只在 `PlayKind::Simulate` 且 attached Play instance 时读取 Scene viewport snapshot；payload只有 transform/projection/FOV/ortho/near/far，固定 default viewport route。
2. `sync_simulate_preview_camera` 通过 `(instance, camera)` last-value 去重；route 返回 false 或读取失败会清本地 cache，但没有向 Runtime 发送 clear/reset，因此旧 override 可能继续留在 extract。
3. Simulate bridge 没有 Camera session/document generation、view purpose、owner/possession、cut/history epoch、sequence number、timestamp、ack 或 stale result rejection。
4. Editor179 的 surface lifecycle/generation 只能证明画面槽的来源，不代表 Camera Preview 已有独立 target/quality/time/input/environment/artifact/capture session。
5. Material/Asset Preview 的 shared PreviewScene 与 Capture mailbox 不能代替 Camera Preview：没有 camera source、target endpoint、lens/rigger、director evaluation 或 draw generation绑定。

### 4.4 Cinematic、Pilot、Capture 与 diagnostics

1. Sequencer workspace route 仍是固定 camera-cut table row/selection feedback；没有 Camera binding、typed cut section、shot evaluation、Director lease、pre-animated state 或 runtime cut receipt。
2. Editor 没有 View Through/Pilot/Lock Camera 状态机：没有 enter/exit transaction、capture lost/Esc/scene switch 恢复、owner revoke、write-back preflight 或 undo grouping。
3. 没有 camera bookmark/history asset、跨 workspace/level/document 持久化、stable camera path/shot identity、timecode 或 take metadata。
4. 没有 Camera Debugger 面板来显示 active endpoint、Director owner、rig node、blend/modifier、collision/occlusion、lens、shake、cut/history、source/artifact/drawn generation。
5. Capture 只消费通用 viewport/frame product；没有离屏 Capture Camera 的 purpose、quality profile、safe frame、multi-view/XR、source pin、async deadline、encoder artifact 和 failure receipt。

## 5. 继承 P0 与 owner 边界

Editor250 不新增父 P0。以下 owner 必须先闭合：

| Owner | 继续拥有的边界 | Editor250 的职责 |
|---|---|---|
| Runtime Camera/Director（Runtime190） | endpoint/schema/compiler/artifact、qualified ViewResult、cut/history/multi-view | 编辑 source、创建 session、消费 artifact/result/diagnostic，不复制 evaluator。 |
| Editor Scene/Transaction（Editor182/184/185/247） | Document/world generation、undo/redo/save/CAS、component structural mutation | Camera property/stack/pilot 写入统一 command/history，带 qualified precondition。 |
| Editor Viewport（Editor187/188/189/190/193/194/195） | viewport interaction/layout/display/selection/picking/transform | 提供 per-view navigation/session 与 Camera view adapter，不把 transient state 当 source。 |
| Editor Cinematic（Editor222） | timeline/shot/take/MRQ authoring与operation factory | 使用 typed Camera Cut binding/evaluation receipt，不固定文本或第二套 timeline。 |
| Editor Asset/Material（Editor248/249） | asset catalog/toolkit/preview infrastructure | Camera asset toolkit复用 package/document/PreviewScene/diagnostic 基础。 |

## 6. P1 差距与重构要求（34 项）

### 6.1 View session、navigation 与 persistence

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| EVC4-P1-001 | Partial | 单 transient snapshot与controller可导航；改为 per-ViewSession，带 ViewId、World/Document generation、purpose、owner、history epoch。 |
| EVC4-P1-002 | Open | projection command会覆盖 transient与source语义；拆 EditorNavigationProjection 与 SceneCameraSourceProjection。 |
| EVC4-P1-003 | Partial | Orbit/Pan/Zoom与projection helper存在；补 delta-preserving zoom、viewport/FOV尺度、dolly/truck/fly/roll和input mode。 |
| EVC4-P1-004 | Open | speed/sensitivity是散落常量/设置；建立 versioned per-user/project CameraNavigationProfile，带单位和迁移。 |
| EVC4-P1-005 | Open | align/frame/reset缺 typed unavailable/stale receipt；增加 selection/world/camera generation preflight。 |
| EVC4-P1-006 | Open | bookmark/history/path/persistence不存在；建立 stable BookmarkId、document/project scope、save/reopen/migration。 |
| EVC4-P1-007 | Open | multi-viewport layout没有独立 camera session/linked sync semantics；为 split/quad/maximized view定义 owner与同步策略。 |

### 6.2 Camera component、Inspector、gizmo 与 transactions

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| EVC4-P1-008 | Open | Camera reflection只暴露 FOV/near/far；由 Runtime schema生成完整 typed inspector、units、ranges、conditional fields。 |
| EVC4-P1-009 | Open | projection/ortho/frustum/off-center/custom没有 authoring UI；接同一 Runtime projection compiler。 |
| EVC4-P1-010 | Open | target/viewport/order/active/HDR/exposure/clear/MSAA/stack无法编辑；补 source property command与round-trip tests。 |
| EVC4-P1-011 | Open | culling/volume masks共用 runtime mask且无UI；拆 typed layer/volume trigger contributor。 |
| EVC4-P1-012 | Open | 无 Lens Profile/physical camera UI；支持 sensor/filmback/focal/aperture/focus/shift/distortion capability。 |
| EVC4-P1-013 | Open | 无 Camera Rig/Shake asset toolkit；复用 AssetType/Toolkit/Document/Operation factory，禁止只注册descriptor。 |
| EVC4-P1-014 | Partial | generic Scene transform transaction与delete preflight存在；补 camera property/stack/rig binding/active selection的 atomic transaction。 |
| EVC4-P1-015 | Open | gizmo是短 frustum/icon 代理；补 ortho/filmback/focus/boom/composition/safe-frame、show flags、generation和budget。 |
| EVC4-P1-016 | Open | multi-select/mixed camera values没有 schema-driven apply；采用 field delta/mixed value/partial failure receipt。 |

### 6.3 Preview、Pilot、Play/Simulate 与 capture

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| EVC4-P1-017 | Open | 没有 Camera PreviewSession；建立隔离 PreviewWorld/target/camera/lens/environment/time/input/quality owner。 |
| EVC4-P1-018 | Partial | Simulate payload不改 Play World且有 bounded validation；增加 clear/reset、view/purpose/owner/generation/ack与stale rejection。 |
| EVC4-P1-019 | Open | default viewport route不支持 Game View、split、capture、XR；route按 ViewId 与 qualified surface。 |
| EVC4-P1-020 | Open | last-value coalescing无 sequence/deadline/backpressure；使用 versioned CameraOverrideTicket与terminal receipt。 |
| EVC4-P1-021 | Open | Pilot/View Through/Lock不存在；建立 enter/exit/capture-loss/Esc/scene-switch/revoke状态机。 |
| EVC4-P1-022 | Open | Pilot write-back无 transform/source transaction、precondition、undo grouping；接 Editor transaction/Document generation。 |
| EVC4-P1-023 | Open | camera preview没有 requested/compiled/installed/LKG/drawn generation；显示 Runtime artifact/preview frame receipt。 |
| EVC4-P1-024 | Open | capture没有 CaptureView purpose、source pin、safe frame、quality/XR、async deadline和encoder artifact。 |

### 6.4 Sequencer、diagnostics、lifecycle 与 qualification

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| EVC4-P1-025 | Open | Sequencer camera-cut row仍为固定 route/文本；接 typed CameraBinding/CutSection/Shot range/easing/runtime result。 |
| EVC4-P1-026 | Open | Director/Pilot/Sequencer之间无 lease/priority/pre-animated restore；建立 owner arbitration 与 cut/history handshake。 |
| EVC4-P1-027 | Open | 无 Camera Debugger/trace；显示 active owner、endpoint、rig/blend/collision/lens/shake/cut/history/source generations。 |
| EVC4-P1-028 | Open | world/document/plugin reload不会统一退休 camera session/job/preview；接 generation fence、unload drain、stale callback rejection。 |
| EVC4-P1-029 | Partial | Viewport pointer/cancel/focus loss有局部测试；补 camera capture/pilot/window/viewport generation与terminal receipt矩阵。 |
| EVC4-P1-030 | Open | 无 1/10/100 view、1K camera asset/node、preview churn、multi-edit的 interaction/compile/paint/RSS基线。 |
| EVC4-P1-031 | Open | 无 visual golden 与 Runtime ViewResult 对拍；建立 same source/quality/target 的 capture diff。 |
| EVC4-P1-032 | Open | 无 accessibility/localization/keyboard-only camera workflow；补 command/keymap、screen reader labels、high-contrast与error recovery。 |
| EVC4-P1-033 | Open | Camera source/schema/asset package没有默认 catalog/readiness/unavailable reason；activation需资源/factory/toolkit原子预检。 |
| EVC4-P1-034 | Open | Camera operation没有真实 factory/handler/receipt；open/create/edit/validate/preview/save/reopen必须可执行或明确 unavailable。 |

## 7. P2 高阶能力（12 项）

1. **EVC4-P2-001**：multi-camera layout editor 支持任意 ViewFamily graph、linked navigation、per-view quality 与 terminal output。
2. **EVC4-P2-002**：virtual-production camera/lens calibration、focus pull、timecode、take/review metadata。
3. **EVC4-P2-003**：camera path/rail/spline、procedural composition 与 residual visualization。
4. **EVC4-P2-004**：Director live debugger、node-level step/scrub、history/cut overlay 与 replay comparison。
5. **EVC4-P2-005**：per-node/per-shot preview cache、visibility priority、GPU byte budget 与 async retirement。
6. **EVC4-P2-006**：XR stereo/late update/foveated preview 与 per-eye capture。
7. **EVC4-P2-007**：camera collision/occlusion debug viewport、probe trace与fallback visualization。
8. **EVC4-P2-008**：camera animation/motion-matching graph与runtime artifact diff。
9. **EVC4-P2-009**：semantic camera graph diff/merge，按 stable IDs保留 bookmark/shot/parameter review。
10. **EVC4-P2-010**：plugin camera node toolkit，具 capability、version、unload、diagnostic 与 sandbox。
11. **EVC4-P2-011**：跨引擎 camera authoring/capture golden corpus 与操作延迟/RSS分位对照。
12. **EVC4-P2-012**：远程/协作 camera session，带 ownership、presence、conflict、recording与recovery。

## 8. 五引擎差异与可迁移合同

| 参考 | 已验证合同 | Zircon 差异与吸收边界 |
|---|---|---|
| Unreal Editor | `EditorViewportClient`拥有 viewport camera transform、projection、show flags、navigation commands、camera speed settings；LevelEditor保存 per-world view state，支持四窗格布局、preview actor与`SActorPilotViewportToolbar`的 actor lock/pilot；`LevelEditorCameraEditorState`能 capture/restore location/rotation/FOV。 | Zircon只有单 transient snapshot，无 per-world/view persistence、pilot/lock、show flags和capture/restore state。吸收 session/profile/pilot/restore receipts；不复制 Slate/UObject。 |
| Unity URP/HDRP | Camera editor把 Base/Overlay、stack、clear flags/depth、volume layer/trigger、normalized viewport、HDR/MSAA、physical camera sensor/lens/focus放入条件 Inspector；HDRP有独立 `HDCameraPreview` 创建 preview camera。 | Zircon Inspector只有3字段，stack/volume/physical/preview不存在。吸收 schema-driven conditional UI、preview session和capability warnings。 |
| Godot | Node3D editor viewport/gizmo为 Camera3D提供 frustum/selection/preview 与 project/unproject/ray工具，并维持 editor camera 与 scene camera 的明确切换。 | Zircon gizmo是短 frustum/icon代理，Scene camera与editor projection混写。吸收 source/preview切换与完整 gizmo/query。 |
| Fyrox | Editor `CameraController`/panel支持 orbit/pan/zoom/pick/fit，CameraSettings持久化 speed、sensitivity、zoom range、exposure；scene camera属性由 Reflect/Visit 驱动 inspector。 | Zircon有局部 orbit/pan/zoom/math test，但 speed/profile没有 consumer，reflection丢11字段。吸收 profile persistence与reflection coverage。 |
| Bevy / Graphics runtime | Bevy将 Camera 与 ComputedCameraValues/projection 分离；Unity HDRP HDCamera保存 prev matrix、history validity、XR offsets、actual resolution 和 history channels。 | Zircon transient Snapshot虽有 derived projection，但无 qualified view identity、history channel与preview/draw generation投影。吸收 source/derived和generation展示，不把 Editor 再变成 renderer owner。 |

共同原则：Editor只拥有 source document、per-view interaction/session、transaction和结果投影；Runtime拥有 compiler/evaluator/view/history；Pilot是有 lease 的连接状态，不是随意写 transform；Preview/Capture必须显示真实 artifact/drawn generation；错误、取消、窗口丢焦和世界切换必须有可恢复 terminal receipt。

## 9. 分层重构顺序

1. **M250.0 View truth**：定义 ViewId、World/Document generation、purpose、owner、navigation/source/preview boundary，删除 camera global/transient 混用假设。
2. **M250.1 Source Inspector**：补 Camera schema projection/target/stack/layers/output、typed property editor、conditional UI、transaction/save/reopen。
3. **M250.2 View session/navigation**：建立 per-view camera session、profile/bookmark/history、精确 navigation 与 layout/link semantics。
4. **M250.3 Runtime artifact bridge**：接 Runtime190 CameraProgram/ViewResult、Simulate clear/ack/generation、PreviewSession requested/LKG/installed/drawn receipt。
5. **M250.4 Pilot/Director/Sequencer**：实现 Pilot lease、View Through、CutSection/Shot binding、pre-animated restore与owner arbitration。
6. **M250.5 Gizmo/debug/capture**：完整 frustum/physical lens/focus/boom/safe frame，Camera Debugger、CaptureView purpose、async capture artifact。
7. **M250.6 Product qualification**：open->edit->compile->preview->pilot->cut->capture->save->reopen->reload、multi-view/XR、fault、a11y、golden image、p95/RSS。

## 10. 资格门（28 个）

| Gate | 状态 | 完成条件 |
|---|---|---|
| EVC4-G01 | Partial | 3/14 Camera 字段可 reflection；完整 schema-driven Inspector未达标。 |
| EVC4-G02 | Fail | projection/target/viewport/output/stack/layer/clear 无 source transaction round-trip。 |
| EVC4-G03 | Fail | 没有 Camera/Lens/Rig/Shake asset toolkit、factory、catalog、template readiness。 |
| EVC4-G04 | Partial | generic transform create/delete/undo与active resync存在；Camera property/stack/active command未闭合。 |
| EVC4-G05 | Fail | single transient camera没有 ViewId/World/Document/purpose/owner/history identity。 |
| EVC4-G06 | Partial | projection/ray helper与Orbit/Pan/Zoom测试存在；尺度、Fly、delta-preserving和failure语义未达标。 |
| EVC4-G07 | Fail | speed/sensitivity/profile/bookmark/history/persistence没有产品 owner。 |
| EVC4-G08 | Fail | multi-viewport linked/split/quad camera session不存在。 |
| EVC4-G09 | Fail | physical lens/filmback/focus/shift/distortion authoring不存在。 |
| EVC4-G10 | Fail | gizmo没有完整 frustum/ortho/focus/boom/composition/safe-frame和budget。 |
| EVC4-G11 | Fail | 没有真正 Camera PreviewSession/PreviewWorld/target/time/quality/input owner。 |
| EVC4-G12 | Partial | Simulate bridge有 bounded validation、不改 Play World、last-value coalescing。 |
| EVC4-G13 | Fail | Simulate没有 clear/reset/view identity/ack/generation/stale rejection。 |
| EVC4-G14 | Fail | route固定 default viewport，不支持 Game View/capture/XR/split view。 |
| EVC4-G15 | Fail | Pilot/View Through/Lock/capture-loss/Esc/scene-switch状态机不存在。 |
| EVC4-G16 | Fail | Pilot write-back没有 transaction/precondition/undo grouping。 |
| EVC4-G17 | Fail | Preview UI不显示 requested/compiled/LKG/installed/drawn generation。 |
| EVC4-G18 | Fail | Capture没有 purpose/source pin/safe frame/quality/async artifact receipt。 |
| EVC4-G19 | Fail | Sequencer Camera Cut 仍是固定文本/route，无 typed binding/section/evaluation。 |
| EVC4-G20 | Fail | Director/Pilot/Sequencer没有 lease/priority/pre-animated restore。 |
| EVC4-G21 | Fail | 没有 Camera Debugger/owner/rig/blend/collision/lens/shake/cut/history trace。 |
| EVC4-G22 | Fail | world/document/plugin reload未统一退休 camera session/job/preview/callback。 |
| EVC4-G23 | Partial | pointer/cancel/focus-loss局部测试通过，缺 camera capture/window/viewport generation矩阵。 |
| EVC4-G24 | Fail | 无 1/10/100 view、1K camera、preview churn、multi-edit性能/RSS证据。 |
| EVC4-G25 | Fail | 无 same source/quality/target 的 Runtime ViewResult 与 visual golden 对拍。 |
| EVC4-G26 | Fail | keyboard/a11y/localization/high-contrast camera workflow未验收。 |
| EVC4-G27 | Fail | operation descriptor没有真实 Camera open/create/edit/preview/save factory/receipt。 |
| EVC4-G28 | Fail | no artifact/director/capability时 Editor 仍可用 transient/default camera 成功显示，Unavailable/fail-closed未达标。 |

## 11. Review-only 边界

本轮没有修改 Editor、Runtime、plugin、Cargo、ABI、ZUI 或测试；没有运行 Editor host、UI automation、PreviewWorld、PIE、Sequencer、capture、fault、scale、soak 或 visual benchmark。下一次实现必须先完成 M250.0-M250.3 的 View identity、source Inspector、per-view session 和 Runtime artifact bridge，再增加 Pilot、物理镜头、Camera Debugger 或更多固定 workspace 文案。
