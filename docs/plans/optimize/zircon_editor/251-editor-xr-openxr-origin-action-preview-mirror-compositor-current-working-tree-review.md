---
title: Editor XR/OpenXR、Origin、Action、Stereo Preview、Mirror 与 Compositor 当前工作树工程化差距
category: zircon_editor
report_id: Editor251
review_date: 2026-08-30
baseline_head: working-tree
related_code:
  - zircon_editor/Cargo.toml
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/scene/viewport/controller
  - zircon_editor/src/scene/viewport/projection.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/core/play/controller/preview_routing.rs
  - zircon_editor/src/core/play/simulate_camera.rs
  - zircon_editor/src/core/play/preview_frame.rs
  - zircon_editor/src/ui/host/editor_host_event_controller/simulate_camera.rs
  - zircon_editor/src/ui/retained_host/app/simulate_camera_sync.rs
  - zircon_editor/src/ui/binding/viewport/command.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_runtime/src/core/framework/render/view_family
  - zircon_runtime/src/core/framework/render/camera
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/core/framework/render/backend_types/capability.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/pipeline.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/command_encoder/render_pass.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/66-runtime-xr-openxr-device-session-stereo-view-tracking-input-late-update-foveation-compositor-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99g-runtime-xr-openxr-device-session-stereo-view-tracking-input-late-update-foveation-compositor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/190-runtime-camera-view-director-history-multiview-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/29-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-authoring-review.md
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/187-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/250-editor-camera-viewport-pilot-preview-cut-capture-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/190-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/OpenXR.uplugin
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Private/OpenXRHMD.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Private/OpenXRHMD_Swapchain.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Private/OpenXRHMD_Layer.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRInput/Private/OpenXRInput.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/SActorPilotViewportToolbar.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Public/LevelEditorCameraEditorState.h
  - dev/UnrealEngine/Engine/Source/Runtime/HeadMountedDisplay/Public/IXRTrackingSystem.h
  - dev/godot/modules/openxr/openxr_api.cpp
  - dev/godot/modules/openxr/action_map/openxr_action_map.cpp
  - dev/godot/modules/openxr/editor/openxr_action_map_editor.cpp
  - dev/godot/modules/openxr/extensions/openxr_fb_foveation_extension.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_plugin.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/XR/XRPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/XR/XRSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/XR/XRSystemUniversal.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/XR/XRPassTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/XR/XRLayoutTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/XR/XRLayoutStackTests.cs
  - dev/bevy/crates/bevy_camera/src/camera.rs
  - dev/Fyrox/editor/src/camera/mod.rs
  - dev/Fyrox/editor/src/camera/panel.rs
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Editor251：Editor XR/OpenXR 当前工作树工程化差距

## 1. 结论

当前 Editor 有真实的普通 Scene viewport 与 Play preview 底座：`SceneViewportController` 已拆出 camera/navigation/pointer/overlay/selection/gizmo/transaction 文件；`ViewportProjectionContext` 提供投影、屏幕尺度、world ray 与 invalid-value 的局部 fail-closed；`PlayPreviewFrame` 保存 gateway/session/instance/generation provenance；Simulate camera bridge 能把有限的 transform/projection/FOV/ortho/near/far payload 路由到 attached Play instance，并不修改 Play World。

这套底座仍完全不是 XR Editor。对 `zircon_editor` 的 tracked Rust/TOML/ZUI 精确搜索没有 `OpenXR`、`XR`、`HMD`、`HeadMounted`、foveation、predicted display、`xrLocateViews`、`multiview_mask` 或 stereo-view owner；`zircon_editor/Cargo.toml` 也没有 XR loader、graphics binding 或 provider feature。本轮聚焦选集为 **145 个文件、11,755 行、411,016 bytes、101 个 test marker、0 个 ignored marker**；参考选集为 **21 个文件、23,660 行、927,829 bytes**。

Editor state 仍只有一个 transient `Option<ViewportCameraSnapshot>` 和一个 `OrbitCameraController`。`current_camera` 从 Scene 的单一 `active_camera` 读取，`set_projection_mode` 直接同时修改 editor settings 与 transient snapshot；render packet 把普通 camera 交给 Runtime 的单 view 请求，preview environment 是 procedural gradient。没有 XR Origin/Reference Space、per-eye ViewId、device/session panel、action map/profile authoring、stereo PreviewWorld、runtime capability handshake、mirror view、composition layer、foveation、late-update trace、XR capture 或 device-loss recovery。

Editor251 继承 Runtime66/106 的 Runtime XR canonical owner、Runtime190 的 View/History owner、Editor29 的普通 Input owner、Editor250 的普通 Camera/Pilot owner，不重复登记这些跨域 P0。本轮新增 **28 项 Editor 专属 P1（24 Open / 4 Partial）、12 项 P2（12 Open）和 28 个资格门（22 Fail / 6 Partial / 0 Pass）**。本轮只写 review/index/coverage，不修改 Editor/Runtime/plugin/Cargo/ABI/ZUI 或测试；Tooling 按用户要求排除。

## 2. 审查边界与证据

### 2.1 纵向检查链

本轮沿 `ViewportCommand -> SceneViewportState/controller -> projection/render packet/gizmo -> PlayPreviewFrame/gateway -> Simulate camera bridge -> Runtime ViewFamily/capability/RHI -> XR Editor authoring/preview/diagnostics` 逐文件检查。普通 camera、surface、Play frame 与 input action 只记录可复用基础，不把它们误判为 XR 语义。

### 2.2 证据等级与反例隔离

- **E3**：当前 Editor production Rust、Cargo、viewport controller、projection/picking、Play preview/Simulate gateway 和局部 tests 已读取。
- **E2**：Runtime66/106/190 与 Editor29/187/250/190 作为 owner 基线；Unreal OpenXR/LevelEditor、Godot OpenXR/editor、Unity Graphics XR pass/layout/tests、Bevy camera 与 Fyrox camera panel 作为本地参考。
- **E1**：普通 projection、gateway provenance 和 pointer tests 只证明局部数学/生命周期；没有真实头显、OpenXR loader、PreviewWorld、XR compositor 或 motion-to-photon 测试。
- **E0**：没有证据证明 Editor XR workflow、画面或性能达到或超过 Unreal/Unity；静态 `multiview_mask: None` 只能证明 Runtime 缺口，不能替代 Editor 实现。

### 2.3 当前精确搜索结果

| 搜索 | 当前 Editor 结果 | 判定 |
|---|---:|---|
| `OpenXR/openxr` | 0 | 无 XR loader/provider/asset/feature |
| whole-word `XR` | 0 | 无 XR 类型、surface、workspace 或 capability |
| `HMD/HeadMounted` | 0 | 无设备/tracking/editor origin owner |
| `foveat*` / predicted display | 0 | 无 foveation、late-latch 或 timing UI |
| `xrLocateViews/xrWaitFrame/xrBeginFrame/xrEndFrame` | 0 | Editor 无 frame/session bridge |
| `multiview_mask` | 0 | Editor 无 RenderPass/XR layout consumer |

这些反例与 Runtime 中 WGPU 的 `multiview_mask: None`、普通 `ViewportCameraSnapshot`、single default viewport、Game/Play preview frame 必须分开。普通 multi-camera、Base/Overlay、procedural preview skybox 或 gamepad input 不能冒充 XR Editor 完成度。

## 3. 当前可保留底座

1. `SceneViewportController` 的模块拆分、interaction cancel、focus-loss release、selection/gizmo transaction 与 `ViewportProjectionContext` 可作为 XR Editor 的 authoring shell；但 camera session 必须从单一 state slot 提升为 per-view/per-device session。
2. `PlayPreviewFrameIdentity` 已保留 gateway、runtime session、transport epoch、instance、surface size 和 frame generation；这是接 XR mirror/preview frame provenance 的起点，不是 XR view identity。
3. `simulate_preview_camera` 的 mode/instance guard、bounded encoding、last-value coalescing 与“不修改 Play World”语义可复用；必须扩为 generation-qualified view group 与 clear/ack，不应直接在此塞入 OpenXR handles。
4. Runtime190 的 source/derived View identity、CameraProgram/ViewResult、history epoch 与 ViewFamily 方向可由 Editor 消费；Editor 不应复制 Runtime evaluator 或自行拼 per-eye matrices。
5. Editor29 的 action/context/binding authoring基础、Editor250 的 camera transaction/Pilot边界和 Editor190 的 preview surface generation 可分别作为 XR action、camera origin、preview surface 的 owner 输入。

## 4. 当前实现事实与断路

### 4.1 Viewport camera 与普通 projection

1. `SceneViewportState` 只有 `camera: Option<ViewportCameraSnapshot>`、一个 `orbit_target` 和一个 `OrbitCameraController`；没有 `ViewId`、XR device/session generation、view index/count、reference-space或 pose validity。
2. `current_camera` 没有 Scene/Preview/Game/Mirror/XR purpose 分区；未初始化时从 `scene.active_camera()`和硬编码默认值构造，多个 viewport/pane 只能竞争同一 transient语义。
3. `set_projection_mode` 直接写 settings 与 transient camera，`build_scene_camera_snapshot` 只取 FOV/near/far，aspect 默认 `16/9` 后再按普通 viewport resize；无 per-eye FOV tangent、display time、array slice、runtime recommended rect或 lens distortion。
4. `ViewportProjectionContext` 的 ray/projection 是 Editor 自己基于 `perspective`/`orthographic` 计算，存在 NaN/near/far 等局部检查；它不消费 Runtime `XrViewFamily` 的 pose/projection，也无 late-latch 后重投影或 input-to-view timestamp。
5. `build_render_packet` 固定 `active_camera_override: None`，只发送一个 `camera: Some(camera.clone().into())`，preview lighting/skybox 使用 procedural gradient；没有 stereo pair、XR output target、composition layer或 mirror policy。

### 4.2 Play、Simulate 与 Preview surface

1. `PlaySessionController::capture_preview_frame` 固定使用 `ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1`，只请求单个 width/height 的 RGBA frame；`PlayPreviewFrameIdentity` 没有 ViewId、eye、projection/view generation、OpenXR session或 swapchain image。
2. `route_preview_input` 只在 `PlayKind::Play` 转发普通 `ZrRuntimeEventV1`；Simulate 则保留 Editor input。没有 XR action set、interaction profile、subaction path、pose/space state或 haptic route。
3. `route_simulate_camera` payload 只有 transform/projection/FOV/ortho/near/far，固定 default viewport route；没有 clear/reset、view group、eye count、purpose、owner、sequence、predicted display time 或 ack/stale rejection。
4. `sync_simulate_preview_camera` 仅缓存 `(instance, camera)`；失败或 route false 会清本地 cache，却没有向 Runtime 发送 clear，因此旧 camera override 可能继续影响普通 extract，更不可能正确退休 XR session/view。
5. Editor 现有 `PlayPreviewFrame` generation 可阻止部分旧 frame 显示，但没有 XR runtime generation、session state、swapchain image lease、mirror surface generation、device-loss/reconnect receipt；frame capture 失败只返回 gateway error。

### 4.3 XR authoring、input 与 device lifecycle 缺席

1. `zircon_editor/Cargo.toml` 和默认 catalog/route 没有 OpenXR dependency、feature、provider、AssetType、toolkit、document、operation或 capability row；无法创建 XR project settings、Origin、ActionSet、InteractionProfile或 render policy。
2. 没有 XR Origin/Stage/Local/Viewer/LocalFloor/Unbounded reference-space authoring、recenter/height/playspace可视化、tracking validity或 anchor graph；Scene transform gizmo不能替代 reference-space semantics。
3. 没有 per-device/session UI：runtime availability、system/form factor、view configuration、environment blend、refresh rate、hand/eye permission、extension capability、session state STOPPING/LOSS/READY/VISIBLE/FOCUSED均不可见。
4. Editor29 的普通 Action/Context/Binding 不表达 OpenXR action path、subaction path、interaction profile、pose action、grip/aim space、active/changed state、haptic amplitude/frequency/duration；也没有 XR-specific rebinding validation。
5. 没有 XR controller/hand/eye-gaze/hand-joint gizmo、permission prompt、device assignment、haptic test或 deterministic fake device；普通 mouse/gamepad event 不能证明 XR input 可用。

### 4.4 Renderer、compositor、capture 与 diagnostics 缺席

1. Editor 没有 `XRPass`/layout/stack consumer，也没有 single-pass/multipass、view count、per-eye viewport/slice、shared culling或 per-view history UI；Runtime190 的 ViewFamily 仍无法在 Editor 显示或编辑。
2. 没有 composition layer authoring/preview：projection/depth/quad/cylinder/equirect/native UI layer 的 order、alpha、space、size、quality、visibility mask与 failure reason均无 owner。
3. 没有 foveation/VRS profile、dynamic resolution per eye、depth motion、late update/late latch timing、reprojection或 frame pacing diagnostics；普通 viewport `preview_lighting`/`preview_skybox`是展示开关而非 XR quality contract。
4. 没有 Game View/mirror view 的 independent surface、crop/letterbox/eye selection、spectator camera或 compositor output provenance；默认 RGBA preview不能代表 headset output。
5. Capture 没有 XR `CaptureViewPurpose`、eye/layout/space/time pin、swapchain readback policy、per-eye artifact或 motion-to-photon receipt；也没有 device-loss/retry/partial-frame semantics。
6. 没有 Camera/XR Debugger 来展示 active device/session、view layout、predicted time、pose validity、space graph、action state、late update、layer/foveation、swapchain lease、history generation和degraded reason。

### 4.5 Operation、reload、scale 与 evidence

1. `ViewportCommand` 只包含 pointer/buttons/resize/projection/alignment/display/grid/snap/preview lighting/skybox/gizmo/selection；没有 XR enable/disable, choose runtime, recenter, mirror, view layout, action profile, capture或 diagnostic command。
2. Scene/Play/plugin reload 没有统一 XR session/job/preview retirement、callback fence或 stale response rejection；一旦后续接入 native loader，现有 generic Play transition gate 不能代替 session supervisor。
3. 没有 0-device/fake-runtime/one-eye/dual-eye/multiview/multipass/quad-view/hand-tracking/eye-gaze/foveation/device-loss/thermal/reconnect 的 Editor test matrix；101 个局部 test marker不覆盖这些状态。
4. 没有 1/2/4/8 view、多个 viewport、多个 device、preview churn、capture queue、per-eye GPU/RSS/p95、input-to-photon与surface recovery基线；不能宣称性能达到 Unreal。
5. 没有 same source/runtime/quality/device 的 headset image、mirror image、depth/motion与action-state golden corpus，也没有 Editor-only unavailable/fail-closed acceptance。

## 5. Owner 边界与不重复计数

| Owner | 继续拥有的边界 | Editor251 的职责 |
|---|---|---|
| Runtime66/106/99g | XR provider/loader、instance/system/session、frame pacing、graphics binding/swapchain、XrViewFamily、space/action runtime、compositor/foveation | 只消费 capability/session/view receipts，提供 authoring/preview/diagnostic adapter，不复制 native lifecycle。 |
| Runtime190 | CameraEndpoint/Director/ViewPurpose/ViewResult、history epoch、multi-view identity | 将 XR origin/camera source映射为 typed request，显示 runtime result；不另建 per-eye evaluator。 |
| Editor29 | 普通 Action/Context/Binding/Rebind/accessibility owner | 扩展 XR action/profile editor，但不把 XR path塞进普通 key/mouse enum。 |
| Editor250 | 普通 viewport/camera/Pilot/Preview/Capture owner | 提供 ViewSession shell、camera source transaction与 surface routing；XR-specific device/session由本报告 owner补齐。 |
| Editor251 | XR settings/origin/action authoring、XR preview/mirror/debug/capture UI与 qualification | 建立 editor-side provider/catalog/document/operation 与 Runtime receipts 的唯一投影。 |

## 6. P1：Editor 专属工程缺口（28 项）

| ID | 状态 | 差异与重构要求 |
|---|---|---|
| EXR5-P1-001 | Open | 无 XR Editor provider/catalog；建立 manifest-driven provider、AssetType、factory、availability与unavailable reason。 |
| EXR5-P1-002 | Open | 无 XR project/device profile document；建立 versioned source、scope、migration、target/runtime selection。 |
| EXR5-P1-003 | Partial | generic settings/document/CAS可复用；补 XR settings transaction、validation、save/reopen与last-good。 |
| EXR5-P1-004 | Open | 无 XR Origin/Stage/Reference Space authoring；建立 stable OriginId、space graph与recenter transaction。 |
| EXR5-P1-005 | Open | 无 tracking origin/height/playspace/anchor gizmo；提供 pose validity、space generation与degraded overlay。 |
| EXR5-P1-006 | Open | 单 transient camera无法表达 eye/view group；建立 per-view session、eye layout与ViewFamily projection consumer。 |
| EXR5-P1-007 | Open | 无 XR device/system/session inspector；显示 runtime/system/form factor/capabilities与state transition receipt。 |
| EXR5-P1-008 | Open | 无 runtime capability handshake；Editor activation必须消费 Runtime66 capability artifact并 fail-closed。 |
| EXR5-P1-009 | Open | 普通 Input Action editor不支持 XR action set/path/subaction/profile；建立 typed XR action document。 |
| EXR5-P1-010 | Open | 无 controller grip/aim/pose/action gizmo与binding preview；按 interaction profile生成可验证 projection。 |
| EXR5-P1-011 | Open | 无 hand/eye-gaze/hand-joint permission与debug surface；建立 privacy/capability/revocation状态。 |
| EXR5-P1-012 | Partial | 普通 Play input route与focus/cancel存在；补 XR action sync、active/changed、timestamp与owner lease。 |
| EXR5-P1-013 | Open | gamepad rumble不能替代 XR haptics；建立 amplitude/frequency/duration/stop command与receipt。 |
| EXR5-P1-014 | Open | 无 session lifecycle UI/operation；建立 install/start/stop/recenter/reconnect/cancel及terminal receipt。 |
| EXR5-P1-015 | Open | 无 XR PreviewWorld/PreviewSession；隔离 target/origin/time/input/environment/quality并绑定 generation。 |
| EXR5-P1-016 | Partial | PlayPreviewFrame已有 gateway/frame generation；补 session/device/view/eye/layout provenance与stale frame rejection。 |
| EXR5-P1-017 | Open | `capture_preview_frame`固定 default viewport/RGBA；建立 headset/mirror/eye/layout-aware frame request。 |
| EXR5-P1-018 | Open | 无 XR mirror/Game View surface；定义 independent mirror camera、crop/letterbox、eye select与surface owner。 |
| EXR5-P1-019 | Open | 无 XRPass/layout consumer；显示 single/multipass、view count、slice/viewport、shared culling与history channels。 |
| EXR5-P1-020 | Open | 无 composition layer editor；建立 projection/depth/quad/cylinder/equirect/native UI source与order validation。 |
| EXR5-P1-021 | Open | 无 foveation/VRS/dynamic resolution UI；绑定 Runtime capability、per-eye budget与degraded receipt。 |
| EXR5-P1-022 | Open | 无 late-update/space/action/frame timing debugger；显示 predicted display time、locate time、late pose与missed frame。 |
| EXR5-P1-023 | Open | 无 XR CaptureView；绑定 purpose/source/artifact/director/eye/layout/time/swapchain policy与async artifact receipt。 |
| EXR5-P1-024 | Open | 无 device loss/reconnect/reload retirement；建立 generation fence、callback drain与recoverable terminal states。 |
| EXR5-P1-025 | Partial | generic command/transition gate可复用；增加 XR command registry、preflight、progress/cancel与remote safety。 |
| EXR5-P1-026 | Open | 没有 fake OpenXR/test runtime adapter；建立 deterministic session/state/action/pose/swapchain simulator。 |
| EXR5-P1-027 | Open | 无 cross-device/multiview/pixel/input-to-photon golden tests与performance/RSS evidence。 |
| EXR5-P1-028 | Open | 无 XR documentation/status projection；移除 capability-only/成功文案，所有不可用路径必须明确 Unavailable。 |

## 7. P2：扩展能力（12 项，当前全部 Open）

1. **EXR5-P2-001**：OpenXR extension registry UI，支持 vendor capability、version、dependency、permission与unload sandbox。
2. **EXR5-P2-002**：hand tracking/eye gaze/face tracking authoring、privacy consent、record/replay与retarget preview。
3. **EXR5-P2-003**：quad-view/foveated view layout、per-region quality、variable-rate image与thermal policy。
4. **EXR5-P2-004**：composition layer timeline、space-relative UI、depth-tested quad与live compositor preview。
5. **EXR5-P2-005**：virtual production calibration、lens/display profile、room-scale capture与timecode。
6. **EXR5-P2-006**：multi-device collaborative XR session、ownership/presence/conflict/recovery。
7. **EXR5-P2-007**：remote headset inspector、networked pose/action trace与secure session attach。
8. **EXR5-P2-008**：motion reprojection/space warp analysis、frame pacing heatmap与latency budget visualization。
9. **EXR5-P2-009**：XR-specific asset cook/package/chunk/install and runtime compatibility matrix。
10. **EXR5-P2-010**：per-eye visual regression corpus across Unreal/Unity/Godot reference scenes and displays。
11. **EXR5-P2-011**：accessibility profiles for seated/standing/one-hand/low-vision XR input and comfort warnings。
12. **EXR5-P2-012**：headset farm automation with deterministic fault, thermal, reconnect and long-soak reports。

## 8. 参考引擎对照

| 参考 | 已证明的工程合同 | Zircon Editor 差距与吸收边界 |
|---|---|---|
| Unreal | OpenXR plugin、HMD tracking system、swapchain/layer owner、OpenXR input、LevelEditor pilot toolbar与CameraEditorState分层；editor/runtime共享明确 device/session owner。 | Zircon只有普通 viewport/camera/gateway。吸收 provider/session receipt、pilot/camera state、layer/swapchain diagnostics；不复制 UObject/module macro。 |
| Godot | OpenXR API、action map/interaction profile editor、foveation/hand/eye/visibility extensions与Node3D editor分离，session/frame和extension可独立演进。 | Zircon无 XR type、action profile或extension registry。吸收 source/provider/editor adapter 分层；不把 Variant/RID直接暴露到 ZUI。 |
| Unity Graphics | `XRPass`/`XRSystem`表达 view count、layout、per-eye viewport/slice、single/multipass、late latch、depth/motion；Editor tests覆盖 layout stack/reuse。 | Zircon render packet 只有单 camera/default RGBA。吸收 typed ViewFamily/layout receipt和可测 stack；不复制 pipeline-specific static caches。 |
| Bevy | Camera target/manual texture view提供 backend-neutral render target landing point与computed camera分离。 | 可用于 mirror/external view adapter，但没有 OpenXR lifecycle证据；不能把 manual texture当 XR 完成。 |
| Fyrox | Camera editor panel/settings是普通 editor camera authoring边界，未提供完整 OpenXR owner。 | 只吸收 camera settings/authoring可用性；工程 XR 目标仍以 Unreal/Godot/Unity合同为准。 |

共同原则：XR source、runtime capability、device/session、ViewFamily、Editor PreviewSession 与 displayed frame 必须各有 stable identity；Editor 只提交 typed source/command，Runtime 返回 immutable view/session/artifact receipt；所有 unavailable、loss、stale、partial-eye、device-loss路径必须可见且 fail-closed。

## 9. 分层重构顺序

1. **M251.0 Editor XR truth**：建立 provider/catalog/feature、XR project profile、availability与 capability receipt，删除 capability-only/静态成功语义。
2. **M251.1 Origin/Action authoring**：建立 Origin/ReferenceSpace/ActionSet/InteractionProfile document、typed Inspector、transaction、migration与permission policy。
3. **M251.2 Runtime bridge**：接 Runtime66/190 的 session/ViewFamily/CameraProgram，扩 Simulate 为 view-group clear/ack/generation bridge。
4. **M251.3 Preview/Mirror**：建立隔离 XR PreviewWorld/PreviewSession、per-eye frame/layout/surface、mirror/Game View和 stale frame rejection。
5. **M251.4 Compositor/Quality**：接 layer graph、depth/motion、foveation/VRS、late-update timing、capture source与quality budget。
6. **M251.5 Device lifecycle/diagnostics**：实现 state/loss/reconnect/reload drain、Camera/XR Debugger、action/pose/space/swapchain trace。
7. **M251.6 Qualification**：fake runtime、headset lab、single/multi/quad view、fault/device loss、golden image、input-to-photon、GPU/RSS/p95、thermal/soak与same-source cross-engine对拍。

## 10. 资格门（28 个）

| Gate | 状态 | 完成条件 |
|---|---|---|
| EXR5-G01 | Partial | generic Editor provider/catalog机制可复用，但 XR provider/feature/asset factory不存在。 |
| EXR5-G02 | Fail | XR profile/source没有 document/schema/version/migration/last-good。 |
| EXR5-G03 | Fail | Origin/ReferenceSpace/anchor没有 stable identity、space generation与recenter transaction。 |
| EXR5-G04 | Fail | device/system/session availability与state没有 Editor receipt。 |
| EXR5-G05 | Fail | runtime capability handshake没有 fail-closed activation。 |
| EXR5-G06 | Partial | 普通 camera/viewport transaction存在，但没有 per-device/per-eye ViewSession。 |
| EXR5-G07 | Fail | 没有 XR action set/path/profile/pose/haptic authoring。 |
| EXR5-G08 | Partial | 普通 input route可复用，但没有 XR action sync/active/timestamp/lease。 |
| EXR5-G09 | Fail | 没有 XR PreviewWorld/PreviewSession/target/time/quality/input owner。 |
| EXR5-G10 | Partial | Play frame generation/provenance存在，但没有 session/device/eye/layout identity。 |
| EXR5-G11 | Fail | preview capture固定 default viewport/RGBA，没有 headset/mirror/eye policy。 |
| EXR5-G12 | Fail | 没有 XRPass/ViewFamily layout、single/multipass或 per-eye history projection。 |
| EXR5-G13 | Fail | composition layer source/order/space/depth/alpha没有 authoring/validation。 |
| EXR5-G14 | Fail | foveation/VRS/dynamic resolution/late latch没有 editor owner。 |
| EXR5-G15 | Fail | mirror/Game View没有独立 surface/crop/eye selection/output receipt。 |
| EXR5-G16 | Fail | CaptureView没有 source/artifact/time/eye/layout/swapchain pin。 |
| EXR5-G17 | Fail | device loss/reconnect/reload没有 session/job retirement与stale callback fence。 |
| EXR5-G18 | Fail | 没有 fake OpenXR adapter或 deterministic pose/action/swapchain simulator。 |
| EXR5-G19 | Partial | command/transition gate可复用，但 XR command/preflight/progress/cancel未闭合。 |
| EXR5-G20 | Fail | 没有 XR debugger 展示 predicted time、pose、space、action、layer、history、lease。 |
| EXR5-G21 | Fail | 没有 hand/eye/permission/privacy/comfort authoring与diagnostic。 |
| EXR5-G22 | Fail | no artifact/director/capability时仍可显示普通 transient preview，XR unavailable未明确投影。 |
| EXR5-G23 | Fail | 没有 0/1/2/4/8 view、single/multi/quad、multi-viewport interaction基线。 |
| EXR5-G24 | Fail | 没有 GPU/RSS/p95/input-to-photon/thermal/soak evidence。 |
| EXR5-G25 | Fail | 没有 same source/runtime/quality/device 的 headset/mirror/depth/motion golden。 |
| EXR5-G26 | Fail | 无障碍、座姿/站姿、单手、低视力、comfort warning工作流未验证。 |
| EXR5-G27 | Partial | generic preview/frame tests存在，但无跨域 session/loss/reconnect/fault矩阵。 |
| EXR5-G28 | Fail | 无 XR editor documentation/status schema，无法给出 stable unavailable reason 与 receipt。 |

## 11. Review-only 边界

本轮没有修改 Editor、Runtime、plugin、Cargo、ABI、ZUI 或测试；没有运行 Editor host、UI automation、OpenXR runtime、头显、PreviewWorld、PIE、GPU capture、CTS、motion-to-photon、fault、thermal、soak 或 benchmark。实现时必须先关闭 M251.0-M251.3 的 provider/source/origin/action/session/view identity 与 mirror provenance，再接 compositor、foveation、late update 和高级扩展。Runtime66/106/99g 仍是 Runtime XR 唯一 canonical owner；Editor251 只负责 Editor 侧缺口，不将“普通 camera + RGBA preview”标记为 XR 完成。
