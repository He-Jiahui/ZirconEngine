---
title: Editor Camera、Rig、Controller、Director、Blend、Shake、Cinematic Cut 与 Preview 当前源码复核
category: zircon_editor
report_id: Editor104
review_date: 2026-08-26
baseline_head: 3282dfad2a3a0dce246dfa8f300d7d30d70ed9a9
baseline_epoch: 524
canonical_owner: Editor30
refreshes:
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/88-editor-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-product-integration-current-source-review.md
related_code:
  - zircon_runtime/src/asset/assets/scene/camera.rs
  - zircon_runtime/src/core/framework/camera_controller
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/scene/components/scene/camera.rs
  - zircon_runtime/src/scene/world/project_io/camera.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/script/vm/gameplay_host/transform.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_editor/src/scene/viewport/controller
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
  - zircon_runtime_interface/src/resource/marker.rs
tests:
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/host/binding_dispatch/viewport.rs
  - zircon_editor/src/tests/editor_event/animation_runtime/graph.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/06-platform-input-process-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/29-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Camera/CameraComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Camera/PlayerCameraManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/SpringArmComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/CinematicCamera/Public/CineCameraComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/MovieSceneTracks/Public/Tracks/MovieSceneCameraCutTrack.h
  - dev/UnrealEngine/Engine/Plugins/Cameras/GameplayCameras/Source/GameplayCameras/Public/Core/CameraRigAsset.h
  - dev/UnrealEngine/Engine/Plugins/Cameras/GameplayCameras/Source/GameplayCameras/Public/Core/CameraNodeEvaluator.h
  - dev/UnrealEngine/Engine/Plugins/Cameras/GameplayCameras/Source/GameplayCameras/Public/Core/BlendStackCameraNode.h
  - dev/UnrealEngine/Engine/Plugins/Cameras/GameplayCameras/Source/GameplayCameras/Public/Core/CameraShakeAsset.h
  - dev/UnrealEngine/Engine/Plugins/Cameras/GameplayCameras/Source/GameplayCameras/Public/Nodes/Collision/CollisionPushCameraNode.h
  - dev/godot/scene/3d/camera_3d.h
  - dev/godot/editor/scene/3d/camera_3d_editor_plugin.h
  - dev/godot/editor/scene/3d/camera_3d_editor_plugin.cpp
  - dev/bevy/crates/bevy_camera/src/camera.rs
  - dev/bevy/crates/bevy_camera/src/projection.rs
  - dev/Fyrox/fyrox-impl/src/scene/camera.rs
  - dev/Fyrox/editor/src/settings/camera.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/UniversalAdditionalCameraData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Editor/Camera/UniversalRenderPipelineCameraEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/Camera/HDCamera.cs
doc_type: current_source_refresh
review_status: complete
implementation_status: pending
source_recheck_required: true
finding_status:
  p0: 5 open
  p1: 49 open
  p1_partial: 11
  p2: 12 open
gate_status:
  fail: 26
  partial: 6
  pass: 0
---

# Editor30/104 · Camera、Rig、Director、Blend、Shake、Cinematic Cut 与 Preview 当前源码复核

## 1. 结论

Zircon 相机底层有真实基础：`SceneCameraAsset` 能持久化 projection、FOV/ortho size、clip、surface/texture/headless target、viewport、order、active、HDR/exposure、clear color、MSAA 和 post-process；`CameraComponent` 进入 World 并被 render extraction 消费；`CameraRenderDescriptor`、deterministic ordering、Base/Overlay stack validator、`ViewportCameraSnapshot` 与 free/orbit/pan controller 都可保留。

但这些没有组成工程级 Camera 产品。`CameraComponent` 14 个字段中 11 个被 reflection skip，Editor 实际只能编辑 FOV/near/far；`ResourceKind` 没有 Camera Rig、Lens、Shake，production source 没有 CameraRig/Director/Blend/Shake/SpringArm/CineCamera/CameraModifier/CameraMode authority。Scene 创建 camera 只是通用节点，没有 camera toolkit、preview、pilot、frustum、safe frame 或 director。

Render 侧支持的 stack 无法从 source 到达：Scene asset/component 没有 render type、stack members、clear-depth、独立 culling/volume mask、dynamic resolution、temporal jitter 或 projection override；World extraction 固定 Base、空 stack、`clear_depth=true`，并把同一 RenderLayerMask 同时当 culling/volume mask。Descriptor tests 不能证明可创作 scene asset。

动态 Runtime 无条件构造 `RuntimeCameraController`。UI 未消费时，mouse 事件又进入通用 Input，又由硬编码 right/middle/wheel 路径直接改 `world.active_camera()` transform；无 enable/profile/InputUser/possession/director gate。脚本 `camera_follow` 直接 `update_transform`，AI LOD 读取全局 active camera，render default、玩法相机和 AI observer 共用一个未持久化选择。

Sequencer Camera/Cut 只是静态 row/control/route；没有 typed camera-cut track、binding、director bridge、shot evaluation 或 cut event。Temporal history 只能以位移/FOV/旋转阈值猜 `CameraCutOrInvalid`，小位移硬切、同位置切镜和主动 history reset 都不能表达。Editor88 的账本保持：5 项 P0 全 Open；60 项 P1 中 49 Open、11 Partial；12 项 P2 全 Open；32 gate 为 26 Fail、6 Partial、0 Pass。没有同场景、同硬件、同画质 benchmark receipt，不能声称优于 Unreal。

## 2. 当前物理范围与逐层事实

| 范围 | 文件 | 行 | 非空行 | bytes | tests | fingerprint |
|---|---:|---:|---:|---:|---:|---|
| Zircon Runtime/Editor/App selected | 103 | 17,518 | 16,112 | 626,486 | 93 | `57bb057923e4b8f533d7307594a8a9f8219a2b65a3ce6c4db56e892ea640565a` |
| Unreal/Godot/Bevy/Fyrox/Unity reference | 25 | 13,380 | 11,458 | 555,712 | 6 | `3b550450dbf24d497dcb655477b41689c6f65b7f7d42a65776550a6aae6a954f` |

去重 union 为 128 文件、30,898 行、1,182,198 bytes、99 个静态 test declarations；fingerprint `53868de388884478e469ba4270bb9fec5aaf3cdaae2f8996989d86316bf099bf`。本轮只静态扫描，不运行 Cargo、WGPU、Editor camera、PIE、Sequencer、temporal capture、split-screen、camera collision 或 soak。

关键源码事实：

1. Scene camera source 是可持久化 endpoint，但 reflection 跳过 11 个 CameraComponent 字段；source 无 schema/version/unknown policy 与 active camera stable identity。
2. `World::set_active_camera` 对 invalid/non-camera 请求静默忽略；无 expected generation、receipt、change event、owner 或 blend。
3. `CameraRenderDescriptor` 已有 Base/Overlay、target、viewport、clear/depth、masks、snapshot 和 validator；Scene extraction 却固定 Base/empty/true，复用 render layer 作为两个 mask。
4. `ViewportCameraSnapshot` 的 aspect、projection override、dynamic resolution、temporal jitter 没有 authoring source；Editor viewport 与 Scene camera 是隐式分叉。
5. free/orbit/pan 是 typed deterministic 数学工具，但 Orbit 同时被 Editor viewport 和 dynamic Runtime 使用，Free/Pan 没有 shipping gameplay owner。
6. dynamic session 无条件创建 RuntimeCameraController，默认 orbit target 取首个 Cube 或 active camera；没有 input action/context/user/possession capability gate。
7. script `camera_follow` 直接写实体 transform，无 CameraComponent 验证、damping、collision、blend 或 receipt；AI plugin 使用 global active camera 计算 LOD。
8. `ResourceKind` 没有 Camera/Rig/Lens/Shake；Sequencer 只有 Camera row/Cut row 和 static Preview/Validate route，没有 runtime consumer。
9. camera history status 用 far-plane 20%、rotation 60°、FOV 15% 等数值猜 cut，不能消费显式 cut/history epoch。

## 3. 参考引擎对照

- Unreal CameraComponent、PlayerCameraManager、SpringArm、CineCamera、MovieScene CameraCutTrack 和 GameplayCameras 的分工表明：endpoint、per-player view target、collision/lag、cinematic lens、typed cut、rig/evaluator、blend stack、shake、debug、Editor asset toolkit 必须分层。
- Godot Camera3D/Editor plugin 提供 per-Viewport current camera、projection/frustum/cull mask、preview toggle、custom camera、gizmo 和 project/unproject；这是单相机 Editor 的可用下限。
- Bevy Camera/Projection 是 ECS render endpoint 参考；Fyrox camera 有反射 projection、viewport、enabled、target、frustum/project/unproject 与 debug frustum；Unity URP/HDRP 明确 Base/Overlay、clear depth、volume mask/trigger、AA、history、jitter、dynamic resolution 和 Editor conditional validation。

## 4. Owner 边界与目标链

| 领域 | owner | Editor30 边界 |
|---|---|---|
| scene endpoint/render extraction/stack/history | Runtime09B/09H1 + Scene | 提供 qualified ViewResult、cut/history epoch |
| Camera Rig/Lens/Shake source/compiler | 新 Runtime/Editor Camera owner | typed documents、artifact、toolkit、diagnostics |
| per-player/per-viewport Director | Runtime Gameplay/Camera owner | activation、target、blend、modifier、possession、receipt |
| Editor viewport navigation/pilot/picking | Editor03/59/66 | transient tool session，不改 gameplay camera authority |
| Sequencer/camera cut | Editor14/83 | typed track/section/binding/evaluation，消费 director |
| input/user/possession | Runtime06/99zb + Editor29 | action/user snapshot、camera mode request |
| AI relevance/LOD | AI owner | per-view family/relevance snapshot，不读取 global active camera |
| document/transaction/job/notification/journal | Editor02/09/10/11 | authoring command、compile/preview job、terminal receipt |

目标闭环：

```text
CameraEndpointComponent + LensProfileDocument + RigDocument + ShakeDocument
  -> CameraCompiler -> CompiledCameraRigArtifact
  -> per-player/per-viewport CameraDirectorInstance
  -> CameraViewResult { endpoint, lens, modifiers, blend, cut_epoch, history_epoch }
  -> render extraction / gameplay / AI / Sequencer / Editor preview
```

## 5. P0：先关闭劫持与不可创作边界

| ID | 当前证据 | 必须重构 |
|---|---|---|
| P0-1 | Dynamic Runtime 无条件劫持 mouse 并直接改 scene camera | 用 InputAction/User/CameraMode lease；Editor tool 与 gameplay controller 分离 |
| P0-2 | Component 大部分字段不可编辑，Rig/Lens/Shake 无资产 | 建立 endpoint inspector、Rig/Lens/Shake source、factory/toolkit |
| P0-3 | Render stack descriptor 无 source route，extraction 固定 Base/empty/true | Scene source 表达 stack/mask/depth/target，编译到 validated artifact |
| P0-4 | global active camera 同时承担 render/玩法/AI observer | per-player/per-viewport director、view family、possession 与 qualified ViewResult |
| P0-5 | Sequencer Cut 是静态 UI，history 只能猜 cut | typed CameraCut track/section、cut event、explicit history reset/epoch |

## 6. P1：Endpoint、Rig、Director、Preview、Sequencer 与质量

| ID | 差异与重构要求 | ID | 差异与重构要求 |
|---|---|---|---|
| P1-1 | endpoint reflection/source schema 不完整 | P1-2 | camera asset schema/version/unknown policy 缺失 |
| P1-3 | active identity/generation/owner 缺失 | P1-4 | perspective/ortho/frustum/projection override 不完整 |
| P1-5 | target/viewport/aspect/mask cross-field validation 缺失 | P1-6 | Base/Overlay stack source/ordering/clear-depth 缺失 |
| P1-7 | dynamic resolution/AA/HDR/exposure source 缺失 | P1-8 | volume mask/trigger/culling mask 分离缺失 |
| P1-9 | lens/filmback/focal/aperture/focus asset 缺失 | P1-10 | rig node/evaluator/parameter schema 缺失 |
| P1-11 | spring arm/collision/lag/occlusion contract 缺失 | P1-12 | shake asset/channel/priority/seed/decay 缺失 |
| P1-13 | modifier stack/additive/override/blend policy 缺失 | P1-14 | compiled camera artifact/digest/dependency 缺失 |
| P1-15 | per-player/per-viewport director identity 缺失 | P1-16 | camera mode/activation/possession lease 缺失 |
| P1-17 | target selection/priority/transition state 缺失 | P1-18 | view target fallback/invalid target receipt 缺失 |
| P1-19 | CameraViewResult/diagnostic/currentness 缺失 | P1-20 | script camera follow typed request/authority 缺失 |
| P1-21 | AI view family/relevance adapter 缺失 | P1-22 | split-screen/multi-viewport isolation 缺失 |
| P1-23 | viewport transient vs scene source contract 缺失 | P1-24 | pilot/view-through/lock/bookmark/safe-frame 缺失 |
| P1-25 | frustum/gizmo/preview/picking tool缺失 | P1-26 | camera inspector transaction/reflection customization 缺失 |
| P1-27 | sequence camera binding/cut track/section schema 缺失 | P1-28 | shot evaluation/time-domain/hold/restore 缺失 |
| P1-29 | director/sequencer runtime bridge 缺失 | P1-30 | cut event/history reset/velocity epoch 缺失 |

## 7. P1：Integration、故障、性能与治理

| ID | 差异与重构要求 | ID | 差异与重构要求 |
|---|---|---|---|
| P1-31 | Editor03/14/22/29/59 owner integration 缺失 | P1-32 | asset catalog/factory/toolkit/reference closure 缺失 |
| P1-33 | compile/preview/pilot job/progress/cancel 缺失 | P1-34 | camera diagnostics/trace/telemetry 缺失 |
| P1-35 | InputUser/action context/mode routing 缺失 | P1-36 | render history/jitter/AA/exposure qualification 缺失 |
| P1-37 | runtime acknowledgement/receipt/provenance 缺失 | P1-38 | network replication/late join/camera authority policy 缺失 |
| P1-39 | save/load/migration/asset reload camera participant 缺失 | P1-40 | world partition/streaming/cell camera policy 缺失 |
| P1-41 | camera collision/occlusion/fault recovery 缺失 | P1-42 | invalid target/device loss/viewport resize state machine 缺失 |
| P1-43 | cut/rig/pose deterministic replay 缺失 | P1-44 | snapshot/diff/merge/conflict authoring 缺失 |
| P1-45 | multi-camera query/paging/history retention budget 缺失 | P1-46 | large stack/director evaluation budget 缺失 |
| P1-47 | camera update CPU/allocation/tail telemetry 缺失 | P1-48 | split-screen/VR/multi-display scale qualification 缺失 |
| P1-49 | camera race/crash/restart/idempotency tests 缺失 | P1-50 | security/permission/pilot lease/redaction 缺失 |
| P1-51 | accessibility/input remap/sensitivity policy 缺失 | P1-52 | plugin/mod rig schema compatibility 缺失 |
| P1-53 | cross-platform projection/layout migration 缺失 | P1-54 | renderer/backend capability fallback 缺失 |
| P1-55 | unique camera authority/navigation hard-cutover 缺失 | P1-56 | stale fixture/route/reference deletion gate 缺失 |
| P1-57 | editor/runtime camera source generation barrier 缺失 | P1-58 | benchmark receipt 同语义/同硬件缺失 |
| P1-59 | full Camera/Director/Sequencer test matrix 缺失 | P1-60 | legacy active_camera/raw transform migration gate 缺失 |

11 个 Partial 仅是 CameraRenderDescriptor/stack validator、Viewport snapshot、controller math、scene render extraction、local history heuristic、Editor viewport tests等基础；不表示 Camera Rig/Director/Cut 产品完成。

## 8. P2 与 32 Gate

P2 全部 Open，覆盖 camera graph procedural authoring、multi-view/VR、cinematic focus/DOF、advanced shake/impulse、AI cinematic handoff、distributed replay、semantic merge、remote director、camera analytics、mod SDK、cross-engine import 和 long-run camera farm。32 gate 当前为 26 Fail/6 Partial/0 Pass；Partial 只来自局部 endpoint/render/controller 基础，所有 source-to-runtime director、cut/history、preview、fault、scale 与 hard-cutover gate 仍未通过。

## 9. 分层重构顺序

1. **劫持切断**：RuntimeCameraController 只在显式 CameraMode/User/lease 下运行；Editor viewport 使用独立 transient controller；脚本 camera_follow 改为 typed request，禁止直接覆写 active camera transform。
2. **Endpoint source**：补齐 CameraComponent reflection、schema/version、active identity、projection/target/viewport/mask/stack/depth/history source；保留 render descriptor validator。
3. **Rig/Lens/Shake compiler**：建立 versioned documents、stable IDs、dependency digest、compiled artifact、diagnostic spans、factory/toolkit/thumbnail/reference。
4. **Director**：建立 per-player/per-viewport instance、view target、mode/possession、blend/modifier/collision/shake、ViewResult、cut/history epoch 和 receipt；AI 消费 view family snapshot。
5. **Sequencer/Preview**：Camera Cut typed track/section/binding/evaluation；Editor提供 pilot/view-through/frustum/safe-frame/bookmark、preview job、transaction/save/undo。
6. **Integration/qualification**：接 InputUser、PIE/network, World Partition, save/reload, temporal history, split-screen/multi-display；注入 invalid target、resize、device loss、late join、crash/restart。
7. **性能资格**：同场景/硬件/画质记录 director evaluation P95/P99、camera stack/viewport CPU/RSS/allocation、history memory、cut latency、split-screen scaling、soak；无 receipt 不宣称优于 Unreal。

## 10. 禁止临时修补与验证边界

- 不得继续给 `world.active_camera()` 增加全局 bool、按键条件或隐藏 fallback 来掩盖 authority 缺失。
- 不得用 14 字段中的少数可反射属性、静态 Camera row、固定 Camera Cut、数值阈值或默认首 camera 冒充 Rig/Director/Cinematic 产品。
- 不得将 Editor viewport transform、AI LOD observer、script camera_follow、render default camera 写入同一全局 active camera。
- 不得把 CameraRenderDescriptor 测试、局部 controller 数学或低负载 single-camera 帧时间当作工程性能结论。

已完成当前工作树递归枚举、Camera source/component/render/stack/controller/script/AI/Editor viewport/Sequencer 逐层阅读、参考路径检查和 fingerprint 冻结；未运行 Cargo 或动态 Camera lane。`source_recheck_required: true` 反映共享 dirty worktree，后续实现前必须重算 selected manifest。Editor104 只刷新 Editor30/88 currentness，不实施生产代码。
