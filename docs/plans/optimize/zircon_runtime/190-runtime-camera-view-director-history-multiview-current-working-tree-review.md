---
title: Runtime Camera、View、Director、History 与 Multi-View 当前工作树工程化差距
category: zircon_runtime
report_id: Runtime190
review_date: 2026-08-30
baseline_head: working-tree
related_code:
  - zircon_runtime/src/asset/assets/scene/camera.rs
  - zircon_runtime/src/scene/components/scene/camera.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/core/framework/camera_controller
  - zircon_runtime/src/core/framework/render/camera
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/core/framework/render/frame_extract
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_runtime/src/dynamic_api/session
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/capture/face_view.rs
  - zircon_runtime/tests/runtime_camera_core_pipeline_contract.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/37-camera-endpoint-director-rig-controller-blend-shake-cinematic-cut-history-multiview-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99za-runtime-camera-endpoint-director-rig-controller-blend-shake-cut-history-multiview-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/103-runtime-clock-time-policy-world-fixed-step-timer-cadence-current-source-review.md
  - docs/plans/optimize/zircon_runtime/186-runtime-physics-backend-shape-query-event-lifecycle-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/187-runtime-scene-ecs-world-archetype-query-schedule-generation-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/189-runtime-material-shader-artifact-variant-pipeline-pso-cache-publication-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/187-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/249-editor-material-shader-graph-instance-toolkit-preview-compiler-current-working-tree-review.md
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
  - dev/bevy/crates/bevy_camera/src/camera.rs
  - dev/bevy/crates/bevy_camera/src/projection.rs
  - dev/godot/scene/3d/camera_3d.h
  - dev/godot/scene/3d/camera_3d.cpp
  - dev/Fyrox/fyrox-impl/src/scene/camera.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/UniversalAdditionalCameraData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/Camera/HDCamera.cs
doc_type: current_working_tree_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Runtime Camera、View、Director、History 与 Multi-View 当前工作树工程化差距

## 1. 结论

当前工作树的 Camera 已有可复用的渲染底座，而不是空接口：`SceneCameraAsset` 与 `CameraComponent` 能保存透视/正交、目标、viewport、排序、clear、HDR、曝光和 MSAA；`CameraRenderDescriptor` 能描述 Base/Overlay、stack、target、clear-depth 和独立 culling/volume mask；`resolve_camera_sequence` 有稳定排序与 stack violation 报告；渲染提交循环能逐 camera 选取 extract，并按 camera key 保存 temporal、motion-vector、GI、virtual-geometry、light-grid 与 capture 状态。`ViewportCameraSnapshot::supports_temporal_reprojection_from` 也会对投影、裁剪面、位移、旋转和 FOV 跳变做局部 cut 判定。

这些局部结构不等于工程级 Camera 产品。当前 Runtime 选集为 **142 个文件、23,150 行、846,431 bytes、232 个 test marker、5 个 ignored marker**；五引擎参考为 **17 个文件、11,171 行、467,226 bytes**。World 仍用单一裸 `active_camera: EntityId` 作为选择真值，render extraction 在非法 override 后回退到 active/first/fallback camera；Scene source 没有 stack、clear-depth、独立 layer、dynamic-resolution 或 projection override 字段；`CameraComponent` 14 个字段中 11 个跳过 reflection。没有 `CameraEndpoint`、`CameraDirector`、`CameraRig`、`CameraShake`、`LensProfile`、`CameraCutEvent`、`ViewPurpose` 或 `PlayerCamera` 生产 owner。

因此 Runtime37/99za 的父 P0 继续有效，本报告不重复计算：source-to-artifact、per-player/per-viewport Director、输入 possession、Sequencer cut、XR/split view 和 save/network/replay 都必须先由各自 owner 收敛。本轮登记 **36 项 Runtime 专属 P1（30 Open / 6 Partial）、12 项 P2（12 Open）和 30 个资格门（23 Fail / 7 Partial / 0 Pass）**。本轮只做 review，不修改生产代码、Cargo、ABI、ZUI 或测试。

## 2. 审查边界与证据

### 2.1 纵向检查链

本轮沿 `SceneCameraAsset -> CameraComponent -> World active selection/project IO -> CameraRenderDescriptor/stack -> RenderViewExtract -> frame submission/history -> dynamic session override -> AI/reflection/capture consumers` 逐层阅读。Editor viewport 只作为 Runtime 输入与消费方检查，不把编辑器 snapshot 当作 Runtime authority。

### 2.2 证据等级

- **E3**：当前工作树生产 Rust、调用点、字段和局部测试已读取；文件名统计采用明确的 focused union，避免重复计数。
- **E2**：对照 Runtime37/99za、Scene/Physics/Material 当前报告与本地 Unreal、Unity Graphics、Godot、Bevy、Fyrox源码。
- **E1**：测试 marker、descriptor 和 ignored benchmark 只证明意图；没有运行 Cargo、WGPU、DX12、RenderDoc、PIE、XR、split-screen、fault、scale、soak 或 benchmark。
- **E0**：没有证据支持“性能和表现优于 Unreal”；本报告只确认结构缺口、潜在正确性风险和验收方法。

## 3. 当前可保留底座

1. `SceneCameraAsset` 与 `camera_to_asset` 已形成基本 round-trip 结构，Texture/PrimarySurface/Headless target 的 direct reference 也有明确边界；应作为 canonical codec 输入，而不是继续扩展裸字段。
2. `CameraRenderDescriptor`、`CameraSequenceReport` 和 active camera index 已把 Base/Overlay ordering 与错误原因从渲染隐式分支提升为可测试数据；下一步应把同一 descriptor 做成 Director 输出，而不是复制第二套 stack。
3. `RenderFrameExtract::select_camera_descriptor` 与 camera loop 的 source-state restore 允许一个 render frame 服务多 camera，说明多 view 的提交通路可保留；其缺口是 identity、purpose、budget 和 lifetime，而非再增加 `Vec<Camera>`。
4. `ViewportCameraHistoryKey` 使用 entity/order/type/target/viewport/layers，并以 `Arc<[RenderLayer]>` 避免宽 layer clone；`ViewportRecord` 也已有按 key 的 history/product/runtime maps，这些是 history channel 的早期底座。
5. `RenderHistoryDomainResetReason::CameraCut`、`FrameHistoryInvalidationReason::CameraCut`、camera delta compatibility 与 viewport resize cleanup 已存在；应改为消费显式 cut epoch，而不是删除已有 reset 原因。
6. Runtime Free/Orbit/Pan controller 已在 Input 域拆分为 settings/input/output/state，属于开发相机底座；它不能继续伪装成 shipping Camera Director。

## 4. 当前实现事实与断路

### 4.1 Source、World 与选择真值

1. `CameraComponent` 只有基础投影和输出字段；`core_pipeline`、`projection_mode`、`ortho_size`、target、viewport、order、active、HDR、exposure、clear、MSAA 等字段直接混合 source 与 render policy。
2. `World::active_camera` 同时参与持久化、extract cache key、动态相机覆盖、Editor resync 和 AI LOD；`set_active_camera` 返回 `()`，非法 entity 静默忽略，没有 owner、reason、generation 或 terminal receipt。
3. World spawn/delete/load 的默认相机修复是“找第一台相机或写 0”，删除 active camera 后自动切换 first camera；这不能表达 player、spectator、capture 或 replay 的并行 view。
4. `build_render_camera` 的选择顺序为 request camera、有效 active override、World active camera、first scene camera、fallback camera；回退链没有 shipping policy 或 `Unavailable` 结果，可能掩盖缺少 Camera artifact。
5. source schema 没有 stable camera endpoint identity、schema version、migration/redirect、lens/rig/shake/cut references，也没有条件字段和单位/range定义。

### 4.2 Render descriptor、stack 与 view family

1. descriptor 具备 `stack` 与 `clear_depth`，但 Scene component/source 不具备对应字段；World builder默认 Base、empty stack、clear-depth 与两个由同一 scene mask 转出的 layer set，Editor无法创作真实 stack。
2. stack resolver 只检查 missing/non-overlay/target mismatch/overlay-has-stack，未检查 cycle、duplicate slot、cross-world entity、orphan、priority、purpose 和 fail-closed publication；错误报告也未进入 asset/compiler receipt。
3. `ViewportCameraSnapshot` 的 aspect、projection override、dynamic resolution、temporal jitter 是 derived/transient view 状态，缺少 source generation、view purpose、target capability 与 publication token。
4. culling 与 volume mask 在 World extraction 都从 `render_layer_mask(entity)` 读取；volume trigger、volume contributor priority 和 independent layer migration 没有 source contract。
5. 多 camera 提交会 restore frame source state，但 UI terminal payload、capture、history 和 feature runtime 的“哪个 view 是终端 owner”主要靠 `receives_terminal_ui` 等局部策略，未形成通用 ViewFamily graph。

### 4.3 Controller、script、AI 与跨域 authority

1. Dynamic Session construction 默认创建 Orbit controller；鼠标右键、中键和滚轮在 UI 未消费时直接改 global active camera transform。FocusLost 只影响 Input reducer，不能提供 camera drag/capture terminal receipt。
2. controller 输入是 normalized math DTO，不消费 Input Action、InputUser、context、device lease、viewport owner 或 CameraMode；开发导航和 shipping camera没有 capability hard cut。
3. script `camera_follow` 直接写任意 entity transform，缺 CameraComponent 校验、target generation、damping、collision、owner 和 receipt。
4. AI registration 的 camera consumer 用 `world.active_camera()` 位置决定 LOD；server、local-player、spectator、capture、AI observer 没有 qualified ViewSet。
5. reflection probe 等插件可以自行构造 `ViewportCameraSnapshot`；这是合理的离屏 view 输入，但没有统一 purpose/capability/retirement，插件 view 和 gameplay view 的 history 预算可能互相污染。

### 4.4 Temporal history、capture 与生命周期

1. `ViewportCameraHistoryKey` 没有 World generation、Player、ViewPurpose、Director/source generation、cut epoch、eye index 或 camera session；同一 entity 在不同 owner/purpose 下可能复用错误 history。
2. `ViewportRecord` 维护至少七类 camera-keyed map；除 surface replacement 的整体 clear 外，没有按 camera retirement、LRU/byte budget、last-use frame 或 stale generation 回收。多 view churn 会保留旧 key。
3. temporal history 的 camera cut 多由 delta heuristic 或调用方传入 `FrameHistoryInvalidationReason::CameraCut`；没有 typed `CameraCutEvent`、reason、sequence/timecode、monotonic epoch 和 per-domain reset receipt。
4. `supports_temporal_reprojection_from` 以 f32 阈值判断位移/旋转/FOV/clip 兼容，未接 world rebase、teleport、director handoff、XR eye pair 或 render quality transition 的统一 reason。
5. capture mailbox 按 viewport generation 管理 pending/completed，但 capture view 没有独立 purpose、camera source/artifact/director generation；capture 与 gameplay camera 可能共享错误 LKG/history。

### 4.5 Dynamic API 与故障语义

1. `ZrRuntimeViewportCameraV1` 只承载 transform、projection kind、FOV、ortho size、near、far；没有 viewport/view/purpose/owner/source generation、cut/history epoch、clear/reset、timestamp 或 ack。
2. Editor Simulate 只在固定 default viewport 路由 camera，Runtime 读取后覆盖 extract，不修改 Play World；这是可保留的 bridge，但没有 clear DTO，Editor camera 失效后 Runtime 的旧 override 可能继续存在。
3. payload 有 bounded JSON 与 finite/range validation，却没有 runtime camera capability negotiation、schema migration、stale instance rejection、per-view backpressure 或 typed failure receipt。
4. extract cache key 只含 change tick、lifecycle visibility revision、active_camera、viewport size；Director/source/artifact/quality/cut/owner 变化没有进入 cache identity。

## 5. 继承 P0 与 owner 边界

以下父项继续由 Runtime99za/Editor207/Editor187 拥有，本报告只引用，不新增 canonical P0：

| 父项 | 当前状态 | 本轮 Runtime 处理边界 |
|---|---|---|
| Camera source/schema/compiler/artifact | Open | Runtime只定义 endpoint、lens、rig、shake source 与 immutable program 输入/输出；不在 World render builder 里继续堆字段。 |
| Director/player/view ownership | Open | Runtime Camera service拥有 qualified ViewResult；Gameplay Framework/Input/Net提供 player、possession、permission 与 clock。 |
| Render stack/history | Open | Render owner消费 ViewResult、cut epoch 和 history channel；不读取 Editor snapshot 或裸 active ID。 |
| Cinematic/Sequencer | Open | Cinematic owner定义 typed cut/binding/timecode；Runtime只提供 evaluation instance 和 cut event adapter。 |
| XR/split-screen | Open | XR owner定义 eye/session/compositor；Camera提供 shared rig 与 per-eye result，不伪造 XR backend。 |

## 6. P1 差距与重构要求（36 项）

### 6.1 Source、schema、projection 与 publication

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| RCM6-P1-001 | Partial | 基础 SceneCameraAsset round-trip 存在；增加 schema/version/document identity、unknown policy、migration 与 diagnostic span。 |
| RCM6-P1-002 | Open | CameraComponent 11/14 字段跳过 reflection；改为 typed field schema、单位、range、conditional visibility 与 stable field identity。 |
| RCM6-P1-003 | Open | source 混合 pipeline/output/selection；拆为 EndpointSource、LensSource、OutputSource 与 ViewPolicy。 |
| RCM6-P1-004 | Open | 无 CameraEndpoint/Lens/Rig/Shake resource identity；接 ResourceKind/catalog/factory/cook role 与 redirect。 |
| RCM6-P1-005 | Open | active camera 是裸 EntityId；改为 world-qualified EndpointRef，带 generation、owner、purpose 与 invalidation reason。 |
| RCM6-P1-006 | Open | `set_active_camera` 静默忽略非法值；改为 fallible command、preflight、typed result 和 no mutation on rejection。 |
| RCM6-P1-007 | Partial | Scene target/viewport 有基本枚举与 bounds 结构；补 finite/depth/format/MSAA/HDR/resize capability validation。 |
| RCM6-P1-008 | Open | source 无 stack/clear-depth/independent culling-volume masks；建立 versioned source-to-descriptor codec和 migration。 |
| RCM6-P1-009 | Open | projection 只有 Perspective/Orthographic；补 frustum/off-center/custom/reverse-Z/infinite-Z 与 fallible matrix contract。 |
| RCM6-P1-010 | Open | aspect 只在 target size 时 derived；定义 source aspect policy、sensor fit、crop/overscan/letterbox 与 export一致性。 |

### 6.2 Director、rig、target、input 与 evaluation

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| RCM6-P1-011 | Open | 没有 immutable compiled camera program；建立 source -> semantic IR -> CameraProgramArtifact，Editor/Play/cook共用。 |
| RCM6-P1-012 | Open | 没有 per-World/per-Player/per-Viewport Director；建立 `CameraDirectorInstance` 与 qualified ViewResult。 |
| RCM6-P1-013 | Open | activation 无 priority/lease/timeout/revoke；建立 possession/stack owner 与单一 terminal activation receipt。 |
| RCM6-P1-014 | Open | view target 只支持 entity transform；增加 socket/bone/offset/velocity/bounds/generation/missing policy。 |
| RCM6-P1-015 | Open | Follow/LookAt/Aim typed node、dead/soft zone、axis constraints、prediction 缺失；禁止 script raw transform 旁路。 |
| RCM6-P1-016 | Open | evaluation phase 未固定；形成 target snapshot -> rig -> constraint -> blend -> modifier -> lens -> publish 的预算化 schedule。 |
| RCM6-P1-017 | Open | blend 只有 descriptor stack，没有 pose/lens/post typed blend、interruption/rebase/reverse/cut 语义。 |
| RCM6-P1-018 | Open | modifier/shake 没有 channel、priority、space、owner、fade、seed 与 deterministic sampling。 |
| RCM6-P1-019 | Open | SpringArm/collision/occlusion 没有 Runtime owner；Physics query需 generation、channel、ignore owner、budget、fallback receipt。 |
| RCM6-P1-020 | Open | Dynamic Orbit 与 shipping camera 混用；改为显式 DevCamera capability，默认不接管 gameplay view。 |

### 6.3 Render view、history、capture 与 multi-view

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| RCM6-P1-021 | Partial | Base/Overlay resolver与camera loop是真实底座；把 stack publication、cycle、duplicate、purpose、target capability与failure receipt接通。 |
| RCM6-P1-022 | Open | culling/volume复用单一mask；拆为 typed layer sets、volume trigger、contributor trace与独立迁移。 |
| RCM6-P1-023 | Partial | per-camera history map按 key 工作；key 增加 World/Player/ViewPurpose/Director/source generation/cut epoch/eye。 |
| RCM6-P1-024 | Open | history map 没有 per-camera retirement/budget；增加 last-use、lease、LRU/bytes、viewport destroy 与 stale sweep。 |
| RCM6-P1-025 | Open | camera cut 只有 enum/reason 与 heuristic；建立 monotonic `CameraCutEvent`、history epoch 和 domain reset receipt。 |
| RCM6-P1-026 | Partial | resize/surface replacement会清 history；补 rebase/teleport/director handoff/quality/XR reset的统一 invalidation source。 |
| RCM6-P1-027 | Open | capture mailbox只有 generation；建立 CaptureViewPurpose、source/artifact/director pin、deadline、retirement与failure。 |
| RCM6-P1-028 | Open | RenderViewExtract只有单 selected view + Vec cameras；建立 ViewFamily/Subview graph、terminal output owner与bounded allocation。 |
| RCM6-P1-029 | Open | split-screen/stereo/XR无 per-eye pose/projection/culling/history；对接 XR owner与 shared-rig contract。 |
| RCM6-P1-030 | Open | extract cache key没有 director/cut/source/owner；改为 generation-qualified cache token并拒绝 stale reuse。 |

### 6.4 Cross-domain、diagnostics、persistence 与 qualification

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| RCM6-P1-031 | Open | AI 读取 global active camera；改为 `ObserverViewSet`/relevance policy，区分 server/local/spectator/capture。 |
| RCM6-P1-032 | Open | network/replay/save无 camera endpoint/director/cut participant；增加 stable ref、snapshot、migration 与 deterministic replay record。 |
| RCM6-P1-033 | Partial | Dynamic API 有 bounded camera payload与Simulate override；补 view identity、clear/reset、ack、capability、generation与stale rejection。 |
| RCM6-P1-034 | Open | camera diagnostics 只有局部 order/history report；增加 active owner、rig node、blend、collision、shake、lens、cut trace和degraded reason。 |
| RCM6-P1-035 | Open | 只有局部 camera/stack/history tests；建立 1/2/100 endpoint、multi-player、split view、history churn、device loss、fault 和 replay 矩阵。 |
| RCM6-P1-036 | Open | 没有规模/延迟/内存资格基线；定义 1/10/100 rig、1K nodes、N views、history bytes、p95 evaluation/submit、allocation与retirement指标。 |

## 7. P2 高阶能力（12 项）

1. **RCM6-P2-001**：物理镜头 calibration、filmback、focal/aperture/focus、distortion/breathing 与 unit-preserving export。
2. **RCM6-P2-002**：procedural composition solver，带 dead/soft/safe zone、warm start、残差和 bounded iteration。
3. **RCM6-P2-003**：large-world rebase 与 camera history/cut 的稳定坐标策略。
4. **RCM6-P2-004**：deterministic camera animation/motion matching compiled modifier。
5. **RCM6-P2-005**：spectator/replay/photo-mode 独立 Director permission、time domain 与 capture policy。
6. **RCM6-P2-006**：camera source/activation/blend/cut/shake 的 recording、rollback 与 network reconciliation。
7. **RCM6-P2-007**：按 quality profile 对 collision、occlusion、lens、shake、solver、history channel 分层降级。
8. **RCM6-P2-008**：per-eye foveation、late update、compositor timing 与 shared history policy。
9. **RCM6-P2-009**：plugin camera node/evaluator capability、version、unload drain 与 fallback。
10. **RCM6-P2-010**：大规模 camera source field 与 spatial shake selection 的 bounded indexing。
11. **RCM6-P2-011**：同 scene/rig/path/view count/hardware 的跨引擎画面与行为 golden tests。
12. **RCM6-P2-012**：分布式 camera simulation farm，覆盖 deterministic cut/reset/fault/migration 与性能分位。

## 8. 五引擎差异与可迁移合同

| 参考 | 已验证合同 | Zircon 差异与吸收边界 |
|---|---|---|
| Unreal | CameraComponent有 FOV/aspect/post-process/NotifyCameraCut；PlayerCameraManager维护 ViewTarget、blend、modifier、shake、camera cache；SpringArm负责 probe/collision/lag；GameplayCameras有 Rig asset、node evaluator、blend stack、shake asset与collision node；MovieScene有 typed CameraCutTrack。 | Zircon只有基础 component、global active ID和heuristic history。吸收 per-player Director、immutable evaluation result、typed cut、独立 collision/shake lifecycle；不复制 UObject/旧兼容层。 |
| Unity Graphics | URP Base/Overlay stack、clear depth、volume layer/trigger、normalized viewport、physical camera 与条件 Inspector；HDRP HDCamera保存 per-camera view constants、prev matrices、XR offsets、history channels、valid frames、dynamic resolution。 | Zircon descriptor有局部 stack，source不可达；history缺 owner/purpose/channel identity。吸收 source-to-stack闭环、history channel/retirement、XR view count；不复制 pipeline-specific static cache。 |
| Godot | Camera3D有 perspective/orthographic/frustum、cull/environment/attributes/compositor、current per Viewport、project/unproject/ray query；Editor提供 camera gizmo/preview。 | Zircon camera仅基础 projection，Editor transient snapshot覆盖 source。吸收 fallible projection/query与per-viewport activation；不把单 current camera当多人 Director。 |
| Bevy | `Camera`与`ComputedCameraValues`分离，target/viewport/order/active/sub-camera明确；`CameraProjection`可扩展并在 viewport 改变时更新 frustum。 | Zircon source/derived/override 混合，projection override没有 owner。吸收 source/derived分离、fallible conversion与 sub-camera model；保留 Zircon 的 owner/lease/cut合同。 |
| Fyrox | Camera属性 Reflect/Visit，支持 viewport/target/frustum/project/unproject/ray；Editor CameraSettings持久化 speed/sensitivity/zoom/exposure。 | Zircon reflection只暴露三字段，Editor speed没有真实 consumer。吸收完整 reflection/query/profile persistence；不复制未经验证的 setter/default。 |

共同原则：source 与 derived 分离；每个 World/Player/Viewport/Purpose 有明确 owner；evaluation 输出 immutable、带 generation；cut/failure/reset 是显式事件；history 有 channel、预算和 retirement；Editor 与 Runtime 共用同一 semantic compiler/evaluator。

## 9. 分层重构顺序

1. **M190.0 Truth/owner hard cut**：冻结 CameraEndpoint、ViewPurpose、Player/View identity、禁止 shipping 使用 global active/默认 Orbit 的决策表。
2. **M190.1 Source/schema/admission**：补 versioned source、reflection、typed validation、stack/clear/layer/target codec，非法 source fail-closed。
3. **M190.2 Lens/Rig/Shake compiler**：建立 IR、stable node/parameter identity、immutable CameraProgramArtifact 和 last-good publication。
4. **M190.3 Director/evaluation**：建立 per-World/per-Player/per-Viewport Director、activation lease、target snapshot、phase budget、blend/modifier/collision。
5. **M190.4 Render ViewFamily/history**：把 ViewResult 接入 stack/submit，扩 history key/channel、cut epoch、retirement、capture purpose 与 multi-view。
6. **M190.5 Cross-domain adapters**：Input possession、AI ObserverViewSet、Net/replay/save、Dynamic API clear/ack/generation、plugin unload。
7. **M190.6 Product qualification**：Editor/Play/Simulate/Sequencer/PIE/XR/capture E2E、fault、scale、soak、golden image与p95/RSS报告。

## 10. 资格门（30 个）

| Gate | 状态 | 完成条件 |
|---|---|---|
| RCM6-G01 | Partial | Scene camera可 round-trip，但schema/version/migration/unknown policy未闭合。 |
| RCM6-G02 | Partial | 3/14字段可 generic reflect，其余字段 typed Inspector/schema未达标。 |
| RCM6-G03 | Fail | Endpoint/Lens/Rig/Shake 没有 resource identity、factory、cook role。 |
| RCM6-G04 | Fail | active camera仍是裸 EntityId，无 owner/purpose/generation。 |
| RCM6-G05 | Fail | 非法 `set_active_camera` 无 typed rejection/receipt。 |
| RCM6-G06 | Fail | source无法表达 stack、clear-depth、independent masks。 |
| RCM6-G07 | Partial | descriptor stack resolver有局部 violation report，但无 cycle/duplicate/publication receipt。 |
| RCM6-G08 | Fail | projection/aspect/frustum/off-center/custom source contract未闭合。 |
| RCM6-G09 | Fail | 没有 source -> semantic IR -> immutable CameraProgramArtifact。 |
| RCM6-G10 | Fail | 没有 per-World/per-Player/per-Viewport Director 与 activation lease。 |
| RCM6-G11 | Fail | Dynamic Orbit 仍能在无 capability 时写 global camera。 |
| RCM6-G12 | Fail | script/AI 仍可 raw transform/global camera 旁路。 |
| RCM6-G13 | Fail | target socket/bone/bounds/generation/missing policy不存在。 |
| RCM6-G14 | Fail | evaluation phase/order/budget/typed result不存在。 |
| RCM6-G15 | Fail | blend/modifier/collision/occlusion/shake source与生命周期不存在。 |
| RCM6-G16 | Partial | CameraCut reset enum与heuristic存在，typed event/epoch/reason source未闭合。 |
| RCM6-G17 | Partial | history按 camera key 保存，但缺 World/player/purpose/director/cut/eye identity。 |
| RCM6-G18 | Fail | per-camera history无 retirement、LRU/byte budget和stale sweep。 |
| RCM6-G19 | Fail | capture没有独立 purpose/source/artifact/director pin。 |
| RCM6-G20 | Fail | ViewFamily/subview/terminal owner/multi-view budget不存在。 |
| RCM6-G21 | Fail | split-screen/stereo/XR per-eye view/history/culling不存在。 |
| RCM6-G22 | Partial | Simulate camera payload有 bounded validation且不改 Play World。 |
| RCM6-G23 | Fail | Simulate没有 clear/reset、view identity、ack、generation/stale rejection。 |
| RCM6-G24 | Fail | extract cache key没有 director/source/artifact/owner/cut generation。 |
| RCM6-G25 | Fail | AI、Net、Save、Replay无 qualified camera participant/observer policy。 |
| RCM6-G26 | Fail | diagnostics无法展示 active owner、rig/blend/collision/shake/lens/cut trace。 |
| RCM6-G27 | Partial | camera/stack/history有局部单元测试，缺跨域生命周期/fault矩阵。 |
| RCM6-G28 | Fail | 没有 1/10/100 rig、多 view/history churn、allocation/p95/RSS证据。 |
| RCM6-G29 | Fail | 没有同路径/硬件/quality的 golden image 与跨引擎行为比较。 |
| RCM6-G30 | Fail | no artifact/owner/capability时仍可 fallback default/first camera，shipping fail-closed未达标。 |

## 11. Review-only 边界

本轮没有修改 Runtime、Editor、plugin、Cargo、ABI、ZUI 或测试；没有运行 Cargo/WGPU/PIE/XR/capture/fault/scale/soak/benchmark。下一次实现必须先关闭 M190.0-M190.2 的 authority、source、artifact 和 default fallback 问题，再扩展 rig node、lens preset 或更多 UI按钮。
