---
title: Editor Camera Asset、Component、Rig、Controller、Director、Blend、Shake、Cinematic Cut 与 Preview 当前源码复核
category: zircon_editor
report_id: Editor151
review_date: 2026-08-26
baseline_head: 166720dcb59c57fb4b33c34b859dc1a3f572b222
verification_head: e82381c81813c6d1947218fe788056e7994dccfc
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/88-editor-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/104-editor-camera-rig-director-blend-shake-cinematic-cut-current-source-review.md
related_code:
  - zircon_runtime/src/asset/assets/scene/camera.rs
  - zircon_runtime/src/scene/components/scene/camera.rs
  - zircon_runtime/src/scene/world/project_io/camera.rs
  - zircon_runtime/src/scene/world/project_io/document.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/core/framework/camera_controller
  - zircon_runtime/src/input/camera_controller
  - zircon_runtime/src/core/framework/render/camera
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity
  - zircon_runtime/src/script/vm/gameplay_host/transform.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_editor/src/scene/viewport/controller
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/ui/asset_editor/preview
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui
  - zircon_plugins/timeline_sequence/editor/src
  - zircon_runtime_interface/src/resource/marker.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/99za-runtime-camera-endpoint-director-rig-controller-blend-shake-cut-history-multiview-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zy-runtime-cinematic-sequencer-sequence-shot-track-section-binding-hierarchy-evaluation-camera-cut-audio-event-take-recorder-movie-render-queue-network-save-scalability-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/66-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/83-editor-cinematic-sequencer-shot-track-section-binding-hierarchy-evaluation-camera-cut-audio-event-take-recorder-movie-render-queue-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/150-editor-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-current-source-review.md
  - docs/plans/performance/01/failure-2026-08-23-editor-delete-subtree-all-cameras-invariant.md
  - docs/plans/zircon_runtime/render/06/failure-2026-08-15-camera-resolution-scale-symbol-drift.md
  - docs/plans/zircon_runtime/render/07/failure-2026-08-08-camera-table-render-extract-stale-map.md
  - docs/plans/zircon_runtime/runtime/07/failure-2026-07-23-submit-context-camera-target-sharing-anchor-drift.md
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
  - dev/godot/scene/3d/camera_3d.cpp
  - dev/godot/editor/scene/3d/camera_3d_editor_plugin.h
  - dev/godot/editor/scene/3d/camera_3d_editor_plugin.cpp
  - dev/bevy/crates/bevy_camera/src/camera.rs
  - dev/bevy/crates/bevy_camera/src/projection.rs
  - dev/bevy/crates/bevy_camera_controller/src/free_camera.rs
  - dev/bevy/crates/bevy_camera_controller/src/pan_camera.rs
  - dev/Fyrox/fyrox-impl/src/scene/camera.rs
  - dev/Fyrox/editor/src/settings/camera.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/UniversalAdditionalCameraData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Editor/Camera/UniversalRenderPipelineCameraEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/Camera/HDCamera.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Utilities/CameraSettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/RenderPipeline/Camera/HDCameraEditor.cs
finding_status:
  p0_open: 5
  p1_open: 49
  p1_partial: 11
  p1_closed: 0
  p2_open: 12
  p2_partial: 0
  p2_closed: 0
gate_status:
  fail: 26
  partial: 6
  pass: 0
---

# Editor151 · Camera / Rig / Director / Cut / Preview 当前源码复核

## 1. 结论

Zircon当前仍没有工程级Camera产品链。Scene camera endpoint、render descriptor、multi-camera ordering、Base/Overlay stack validator、World v2 active-camera持久化、free/orbit/pan数学、Editor transient viewport camera和短frustum gizmo都是真实底座；但仓库中没有Camera Rig、Lens Profile、Camera Shake source asset，没有semantic compiler/immutable artifact，没有per-player/per-viewport Director、activation lease、blend/modifier/collision/shake evaluation，也没有authoritative Camera Cut与history epoch。production精确类型搜索仍没有`CameraRigDocument`、`CompiledCameraRigArtifact`、`CameraDirector`、`CameraBlend`、`CameraShake`、`SpringArm`、`CineCamera`、`CameraModifier`、`CameraMode`、`CameraCutEvent`、`CameraLensProfile`、`CameraActivationRequest/Receipt`或`CameraViewResult`。

本轮确认一项架构进展：Free/Orbit/Pan的stateful controller实现已经从`core::framework::camera_controller`迁到`input::camera_controller`，Core只保留settings/input/output/state等中性合同。这修正了Editor88记录的层级放置问题，应保留且不得恢复兼容shim；它没有解决产品authority。Dynamic Session仍默认构造`RuntimeCameraController`，未被Runtime UI消费的右键、中键、滚轮会直接读取`world.active_camera()`并改写scene transform，FocusLost只进入Input reducer而不取消controller drag。

Camera source与runtime descriptor之间仍断路。`SceneCameraAsset`有projection、target、viewport、HDR/exposure、clear、MSAA与optional post-process，`CameraComponent`也有14个字段，但只有FOV/near/far进入reflection。Scene source没有render type、stack、clear depth、独立culling/volume masks、dynamic resolution、temporal jitter或projection override；World extraction通过默认descriptor隐式得到Base、empty stack与`clear_depth=true`，并把同一scene layer mask同时写入culling和volume。

World v2现在持久化裸`active_camera: EntityId`，删除/undo也有局部fallback，这是旧报告必须修正的真实进展；但裸ID没有World/source generation、player、viewport、purpose、owner、possession、blend或terminal receipt。render fallback、Dynamic writer、script `camera_follow`和AI LOD仍共享或绕过这个global authority，不能承担Director。

Temporal history key已扩展到entity、render order/type、target、viewport及完整culling/volume layer sets，比Editor104所见更强；但仍缺World、player、view purpose、endpoint/director/source generation和cut epoch。per-camera history/runtime maps只在surface extent替换时整体清空，没有active-key retirement、age/bytes/fence预算。Velocity继续用far-plane 20%、rotation 60度、FOV 15度、ortho/clip相对变化阈值推断`CameraCutOrInvalid`，只能作为损坏保护，无法区分同位置切镜、小幅硬切和连续大运动。

Sequencer继续是静态产品表面：ZUI固定`SEQ_Intro`、`Camera_A`、`Camera Cut 0000-0180 Ready`、`12 shots`和`428 keys`；Preview/Validate固定返回`24 fps`与`1 gap`。routes只改变control state或反馈，没有Camera binding/cut section/shot evaluator、Director lease、restore state或Runtime consumer。

因此Editor30/88/104的canonical状态不变：**5项P0全部Open；60项P1为49 Open、11 Partial、0 Closed；12项P2全部Open；32个Gate为26 Fail、6 Partial、0 Pass**。没有同scene、同rig、同view count、同硬件、同画质和同输入轨迹的动态receipt，不能声称性能或表现优于Unreal。

## 2. 冻结范围与方法

本报告读取当前共享工作树，以`166720dcb59c57fb4b33c34b859dc1a3f572b222`标记提交基线。范围内包含其他会话在途修改，本轮不回退、不覆盖、不暂存。物理行按文件读取；tests统计Rust `#[test]`，ignored统计`#[ignore...]`；fingerprint由排序后的lowercase相对路径、`|`和逐文件SHA-256以LF连接后再次SHA-256。

| 选择集 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Runtime camera、controller、render/history与consumer纵切面 | **111 / 16,229 / 14,799 / 584,032 / 98 / 0** | `9e1a302d0ed8f1426b74d9b777c31c82175991ea10c3edbe2b6f941065eeeb39` |
| Editor viewport、asset primitive、preview、Sequencer与测试 | **78 / 16,340 / 15,142 / 605,433 / 114 / 0** | `0010920afd42aa966f3731a2dae61240f6f8a83843018aaab4b08ff95e373351` |
| Zircon selected union | **189 / 32,569 / 29,941 / 1,189,465 / 212 / 0** | `12d60c99b58aae01d8f3a06b0435525829913a8b9f3d3a4bb3c494fae56e2021` |
| 五引擎参考集 | **25 / 13,380 / 11,458 / 555,712 / 4 / 0** | `1c4a7e065903e06cd5dbc6cd1dd6007d8f0aa8fa30c1501105c4c98ccd0ab518` |
| all selected | **214 / 45,949 / 41,399 / 1,745,177 / 216 / 0** | `595db68d03e6665b6bae8fd51bdfbc0e171227f553330edad46448ec618deea3` |

Godot revision为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`，Bevy为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`，Fyrox为`8d815db36494f1badb347547dfc7094bf4fbbdf8`，Unity Graphics为`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal以vendored文件fingerprint冻结。参考集4个test marker来自Bevy，只表示静态声明。

本轮未运行Cargo、Editor、WGPU、PIE、真实Camera preview、Sequencer、cut/history capture、split-screen/XR、collision、fault、scale、soak或跨引擎benchmark。Tooling按用户要求排除，也没有查询、轮询、等待或实时跟踪协调器。

## 3. 当前产品事实

### 3.1 Endpoint、schema、reflection与持久化

1. `SceneCameraAsset`可round-trip core pipeline、Perspective/Orthographic、FOV/ortho size、clip、surface/texture/headless target、viewport、order、active、HDR、exposure、clear、MSAA与post-process。
2. source没有camera专用schema/version、stable field identity、unit/range、unknown policy、migration、semantic diagnostics或capability requirements；asset到component转换是直接字段映射。
3. `CameraComponent`的14个字段中11个被`#[zr_reflect(skip)]`：projection、ortho、target、viewport、order、active、HDR、exposure、clear和MSAA均不能走generic Inspector。
4. `ResourceKind`仍是26类，没有Camera Rig、Lens Profile或Camera Shake marker；Editor type registry、factory、toolkit、thumbnail、reference analyzer、cook role与open route均为零。
5. `World::set_active_camera`返回`()`并静默忽略非法或非camera entity；`CameraComponent::is_active`、global active ID与extract request override是三套未统一的选择语义。
6. World persistent state保存裸ID并在load/delete/detach undo时局部修复，但没有stable endpoint reference、generation、redirect、prefab/cook identity、missing diagnostic或change receipt。

### 3.2 Render descriptor、stack与history

1. `CameraRenderDescriptor`已表达Base/Overlay、stack、clear depth、独立masks、target、viewport、snapshot；ordering与stack resolver有真实测试和局部diagnostics。
2. Scene source/component没有上述stack字段。`build_render_camera_descriptor_for_component`只显式填写render order、target、viewport、clear和两个mask，其余来自default descriptor，因此authoring无法到达Overlay/stack/clear-depth配置。
3. culling与volume都从同一个32-bit `RenderLayerMask`转换；没有独立volume trigger、wide-layer source migration或条件Inspector。
4. `ViewportCameraSnapshot`的aspect、projection override、dynamic resolution和temporal jitter没有Scene source，手工DTO与Editor transient camera形成旁路入口。
5. stack resolver已有missing/non-overlay/target mismatch/overlay-has-stack报告，但source不可创作，duplicate ownership、cycle、orphan、shipping fail disposition与publication receipt不完整。
6. `ViewportCameraHistoryKey`保存七类render identity，且wide layer set可共享存储；但历史/Hybrid GI/Virtual Geometry/light-grid/motion-vector/particle map均无活跃集合退休与预算合同。
7. surface resize会整体take histories并清空相关maps，这是正确的extent invalidation，不是camera/director/cut lifecycle。

### 3.3 Controller、Input、Script与AI authority

1. Core保留Free/Orbit/Pan settings/input/state/output，Runtime Input拥有controller行为；这是正确的module boundary hard cut，应删除旧路径残留引用而不是恢复re-export。
2. controller接收normalized DTO并产生deterministic transform，可复用为数学kernel；它不读取Input Action artifact、InputUser、context、device lease、viewport owner或CameraMode。
3. Dynamic Session始终构造Orbit controller，target取首个Cube或active camera，right/middle/scroll直接修改global active camera；没有profile/capability/possession/director gate。
4. Runtime UI消费能阻止后续hardcoded handler，但未消费事件同时进入通用Input与camera writer；FocusLost没有结束drag，wheel精度与DPI/device provenance也未进入controller合同。
5. script `camera_follow`在通用entity能力下直接`update_transform`任意entity，没有CameraComponent验证、target generation、damping、collision、blend、owner或receipt。
6. AI behavior tick每帧读取`world.active_camera()`位置决定LOD；server、spectator、split-screen、capture与AI observer无法选择qualified view family。

### 3.4 Editor viewport、Inspector、Preview与Pilot

1. Editor viewport有独立transient `ViewportCameraSnapshot`和navigation state；其projection/settings不由Scene camera source驱动，也没有明确View Through/Pilot切换合同。
2. generic transaction能创建/删除camera node、preview/commit transform并undo；camera-specific property transaction因reflection缺口只覆盖FOV/near/far。
3. `render_packet.rs`绘制camera icon、pick sphere和短frustum，near被抬到`0.05`、far被截到`2.5`；缺orthographic、filmback、focus plane、boom、composition、安全框、show flag、generation与预算。
4. `UiAssetPreviewHost`只用于UI surface，不是Camera Rig/Lens/Shake Preview；没有preview world、time、target、aspect、input、artifact generation或terminal job receipt。
5. View Through、Pilot、Lock、bookmark product、capture-loss/Esc/scene-switch恢复、transactional transform write-back与Camera Debugger仍不存在。

### 3.5 Sequencer与Cinematic Cut

1. Workbench Sequencer的sequence、camera、cut、shot/key计数与range均为固定ZUI文本；19类交互主要更新selection、field或固定feedback。
2. timeline editor plugin提供AnimationSequence capability/descriptor，Workbench namespace没有连接真实document/operation/provider。
3. `AnimationSequenceAsset`和compiled sequence只支持entity/property curve采样，没有stable camera binding、typed cut section、shot/take、pre-roll、restore state或director handshake。
4. Editor83拥有通用Cinematic产品；Editor30只应提供Camera binding、Director和Cut/History adapter，不能复制第二套timeline/compiler/evaluator。

## 4. Owner边界与目标合同

| 领域 | 唯一owner | Editor151职责 |
|---|---|---|
| endpoint source/compiler/artifact/director/evaluation | Runtime Camera owner | Editor只编辑source并消费artifact、diagnostic、view result与receipt |
| Render stack/history/temporal/capture | Runtime Render | 消费qualified view与cut/history epoch，不从Editor读取第二真值 |
| Rig/Lens/Shake documents与toolkits | Editor30 + Runtime compiler | document/session/transaction/preview，不复制runtime evaluator |
| Scene viewport navigation与Pilot | Editor66 | per-viewport transient session，Pilot以lease连接scene endpoint |
| Timeline/Shot/Cut | Editor83 | 使用通用timeline identity/evaluator，只扩展typed Camera Cut |
| InputUser/Action/possession | Runtime Input/Gameplay + Editor150 | Camera消费qualified intent与owner lease，不读取raw mouse |
| collision/occlusion | Runtime Physics | 提供generation/budget/fault-qualified query，不建立Camera私有world |
| AI relevance | Runtime AI | 消费ObserverViewSet，不读取global active camera |
| save/reload/network/replay | Runtime Scene/Net/Save | artifact/source/director generation与迁移，不保存Editor transient state |

```text
CameraEndpointComponent + LensProfileDocument + RigDocument + ShakeDocument
  -> CameraSemanticCompiler
  -> immutable CompiledCameraRigArtifact
  -> per-World/per-Player/per-Viewport CameraDirectorInstance
  -> CameraViewResult { purpose, endpoint, pose, lens, post, generation,
                        cut_epoch, history_epoch, diagnostics }
  -> Render / Audio / AI / Network / Save / Replay adapters

Editor CameraDocumentSession
  -> Inspector / Rig Graph / Lens+Shake editors / Preview / View Through
  -> Pilot / Sequencer / Debugger
  -> same compiler, evaluator and qualified runtime snapshots
```

最低跨层合同必须包含World/Player/View/Purpose identity、source/artifact/director generation、stable endpoint/target address、activation owner/priority/lease、fixed evaluation phase、bounded target/collision query、typed terminal receipt、explicit cut/history epoch及bounded observation/retirement。

## 5. 五套参考引擎的直接差异

| 参考 | 直接源码合同 | Zircon应吸收 | 不应照抄 |
|---|---|---|---|
| Unreal Engine | CameraComponent有post-process与`NotifyCameraCut`；PlayerCameraManager有current/pending ViewTarget、transition blend、modifier/shake；SpringArm有probe/collision/lag；CineCamera有filmback/lens/focus/focal/aperture；MovieScene有typed CameraCutTrack；GameplayCameras有Rig asset/build、evaluator、blend stack、shake与async collision node | per-player authority、versioned source/build、typed evaluator result、独立blend/modifier/shake/collision生命周期、authoritative cut | UObject宏、历史compat层和未量化的默认tick成本 |
| Godot | Camera3D按Viewport current lifecycle，支持perspective/ortho/frustum、cull/environment/attributes/compositor及project/unproject；Editor有custom camera切换、Inspector preview与SubViewport | per-viewport activation、明确preview切换、完整projection/frustum工具 | 把简单setter或单current camera当多人Director上限 |
| Bevy | `Camera`与`ComputedCameraValues`分离，target/viewport/order/active/sub-camera明确，conversion可失败，自定义projection可扩展；controller在独立crate并以component显式启用 | source/derived分离、fallible conversion、controller边界与显式enable | 用ECS组合本身替代owner/lease/director/cinematic合同 |
| Fyrox | Camera projection与主要属性Reflect/Visit，支持viewport/target/frustum/project/unproject/ray；Editor camera speed/sensitivity/zoom持久化 | reflection/serialization覆盖、normalized viewport、query工具和profile持久化 | 复制builder局部行为或未经验证的默认值 |
| Unity Graphics | URP序列化Base/Overlay stack、clear depth、volume mask/trigger与AA，Camera Editor按条件展示；HDRP以Camera/XR pass/history channel区分history并提供reset/free/clean/dynamic-resolution | source-to-stack闭环、条件Inspector、history channel/lifecycle、XR view count与显式reset | pipeline-specific MonoBehaviour/static cache owner；本地Graphics不含Cinemachine，不推测未收录能力 |

共同工程原则是：source与derived runtime state分离；每个player/view/purpose有owner和lifecycle；cut/failure是显式事件；Editor投影同一compiler/evaluator；history与async query有generation、budget和retirement。

## 6. Canonical P0 currentness

| ID | 状态 | 当前证据与硬切目标 |
|---|---|---|
| `CAM-ED-P0-001` | **Open** | Dynamic Session默认安装Orbit并在UI未消费时直接写global scene camera；迁为默认关闭、capability-gated、InputUser/CameraMode/owner lease驱动的DevCamera，shipping controller走Director。 |
| `CAM-ED-P0-002` | **Open** | 14字段仅3字段反射，Rig/Lens/Shake resource/factory/toolkit为零；建立完整endpoint Inspector和三类versioned source/compiler产品，不能只解除skip。 |
| `CAM-ED-P0-003` | **Open** | stack validator只在descriptor/tests可达，Scene extraction依赖Base/empty/clear-depth默认且复用两个mask；完成source/Inspector/roundtrip/compiler/artifact链。 |
| `CAM-ED-P0-004` | **Open** | persisted raw active ID仍被render fallback、dynamic writer、script和AI混用；建立per-player/per-viewport/purpose Director、possession和qualified ViewResult。 |
| `CAM-ED-P0-005` | **Open** | Sequencer Cut仍是固定UI，Velocity仍猜cut；建立typed binding/track/section、Director handshake、`CameraCutEvent`与monotonic history epoch。 |

## 7. Canonical P1 currentness

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| `CAM-ED-P1-001` | **Partial** | generic Inspector仅能编辑FOV/near/far；14字段需要typed property、unit、validation、transaction、conditional layout与multi-edit。 |
| `CAM-ED-P1-002` | **Open** | 无camera专用schema/version/unknown policy；补canonical codec、migration、compatibility与diagnostic spans。 |
| `CAM-ED-P1-003` | **Partial** | World v2保存裸active ID且delete/undo有fallback；升级为generation-qualified stable endpoint reference、redirect与typed missing receipt。 |
| `CAM-ED-P1-004` | **Partial** | Perspective/Orthographic和runtime projection override存在；缺frustum/off-center/custom/physical lens、aspect约束、reverse/infinite-Z与source admission。 |
| `CAM-ED-P1-005` | **Open** | target/viewport缺finite、bounds、depth、format、MSAA/HDR与resize跨字段validation。 |
| `CAM-ED-P1-006` | **Open** | Base/Overlay、stack、ordering ownership与clear-depth没有source/Inspector/roundtrip。 |
| `CAM-ED-P1-007` | **Open** | culling/volume继续复用单一scene mask；拆为typed layer sets、volume trigger与migration。 |
| `CAM-ED-P1-008` | **Partial** | embedded post-process与volume extract存在；缺profile reference、weight、trigger、override mask、contributor trace和Director合成。 |
| `CAM-ED-P1-009` | **Open** | 无versioned Lens Profile、filmback、focal/aperture/focus/crop/distortion metadata。 |
| `CAM-ED-P1-010` | **Open** | 无Camera Rig asset、stable graph、parameters、target slots、transition与dependency contract。 |
| `CAM-ED-P1-011` | **Open** | 无Camera Shake asset、pattern/envelope/channel/space/seed/scaling/stop policy。 |
| `CAM-ED-P1-012` | **Open** | rig node/pin/parameter/transition/binding没有stable identity。 |
| `CAM-ED-P1-013` | **Open** | Rig到Lens/Shake/Curve/Target/Post/Nested Rig没有typed reference graph。 |
| `CAM-ED-P1-014` | **Open** | Editor/preview/PIE/cook/shipping没有共享semantic compiler与immutable artifact。 |
| `CAM-ED-P1-015` | **Open** | camera migration、redirect、LKG publication与stale artifact policy缺失。 |
| `CAM-ED-P1-016` | **Open** | per-player/per-viewport/purpose Director identity、generation、stack与history缺失。 |
| `CAM-ED-P1-017` | **Open** | activation缺priority、owner lease、timeout、revoke与single terminal receipt。 |
| `CAM-ED-P1-018` | **Open** | view target仍是裸entity transform；补socket/bone/offset/velocity/bounds/generation/missing policy。 |
| `CAM-ED-P1-019` | **Open** | Follow/LookAt/Aim typed node、dead zone、prediction与axis constraint缺失。 |
| `CAM-ED-P1-020` | **Open** | Boom/SpringArm、probe/channel/ignore owner/push/recovery/initial-overlap缺失。 |
| `CAM-ED-P1-021` | **Open** | camera collision与target occlusion分层、fade/shoulder/fallback策略缺失。 |
| `CAM-ED-P1-022` | **Open** | damping/lag缺space、half-life、max speed、time domain与teleport reset。 |
| `CAM-ED-P1-023` | **Open** | single/multi-target framing、screen region、dead/soft zone、安全框与bounded solver缺失。 |
| `CAM-ED-P1-024` | **Open** | controller虽已正确迁层，仍不消费Input Action/InputUser/viewport/focus/consume provenance。 |
| `CAM-ED-P1-025` | **Open** | development navigation与shipping camera仍混在Dynamic Session；迁为显式DevCameraMode。 |
| `CAM-ED-P1-026` | **Open** | target snapshot -> rig -> constraint -> blend -> modifier -> lens -> publish没有固定phase/order/budget。 |
| `CAM-ED-P1-027` | **Open** | pose/lens/post-process typed blend、curve、outgoing lock、preblended与cut缺失。 |
| `CAM-ED-P1-028` | **Open** | blend interruption的rebase/reverse/cut/restore语义与receipt缺失。 |
| `CAM-ED-P1-029` | **Open** | modifier stack、channel/priority/additive/absolute、owner/lifetime/fade缺失。 |
| `CAM-ED-P1-030` | **Open** | script camera facade仍能raw transform mutation；改为scoped Director handle与bounded command/receipt。 |
| `CAM-ED-P1-031` | **Open** | filmback/sensor、focal/FOV authority、preset与unit模型缺失。 |
| `CAM-ED-P1-032` | **Open** | aperture、manual/tracking/autofocus、offset/smoothing/debug plane缺失。 |
| `CAM-ED-P1-033` | **Partial** | endpoint exposure/post与volume extract存在；缺endpoint/rig/volume/sequence contributor优先级、trace和cut reset。 |
| `CAM-ED-P1-034` | **Open** | sensor fit/crop/gate/overscan/safe-area/letterbox没有统一preview/render/export policy。 |
| `CAM-ED-P1-035` | **Open** | lens distortion/breathing calibration、renderer capability与Unavailable fallback缺失。 |
| `CAM-ED-P1-036` | **Open** | Shake pattern没有deterministic seed、channel、space或compiled sampling。 |
| `CAM-ED-P1-037` | **Open** | Shake service的instance/owner/tag/scale/fade/pause/time-domain生命周期缺失。 |
| `CAM-ED-P1-038` | **Open** | typed CameraCutEvent、reason、tick与monotonic history epoch缺失。 |
| `CAM-ED-P1-039` | **Open** | history identity虽扩充render字段，仍不含World/player/view/purpose/director/source generation/cut epoch。 |
| `CAM-ED-P1-040` | **Open** | Sequencer没有stable camera binding、typed cut section/range/blend/easing/diagnostic。 |
| `CAM-ED-P1-041` | **Open** | Sequence与Director没有play/scrub/stop/loop/jump lease和restore-state handshake。 |
| `CAM-ED-P1-042` | **Open** | Shot/Take/timecode/handles/lens refs/safe-frame/source revision metadata缺失。 |
| `CAM-ED-P1-043` | **Partial** | renderer有multi-camera/target/stack底座；未区分gameplay、capture/probe、overlay、Editor preview的purpose/owner/budget。 |
| `CAM-ED-P1-044` | **Open** | split-screen/stereo/XR的per-eye pose/projection/history/culling与shared rig边界缺失。 |
| `CAM-ED-P1-045` | **Open** | network/spectator/replay/server relevance/AI observer camera policy缺失。 |
| `CAM-ED-P1-046` | **Open** | Rig/Lens/Shake无resource kind、factory、catalog、thumbnail、toolkit与cook role。 |
| `CAM-ED-P1-047` | **Partial** | generic scene/asset transaction、dirty/history可复用；无Camera文档session、preview transaction与conflict语义。 |
| `CAM-ED-P1-048` | **Partial** | generic Inspector能编辑3字段；缺完整分组、条件字段、unit、picker、validation与multi-edit。 |
| `CAM-ED-P1-049` | **Open** | 无Rig graph/stack editor、parameter/target/transition/blend/modifier投影与compiler diagnostics。 |
| `CAM-ED-P1-050` | **Open** | 无Camera Preview tile、View Through或qualified preview generation；transient viewport camera不是替代。 |
| `CAM-ED-P1-051` | **Open** | 无Pilot/Lock/capture-loss/Esc/scene-switch状态机和transactional transform write-back。 |
| `CAM-ED-P1-052` | **Partial** | icon、pick sphere和短frustum存在；缺ortho/filmback/focus/boom/composition/safe-frame、show flag、generation与预算。 |
| `CAM-ED-P1-053` | **Open** | target/input/aspect/device/time/post/collision/platform多上下文Preview Session缺失。 |
| `CAM-ED-P1-054` | **Open** | live Camera Debugger、active stack/owner/node/blend/collision/shake/lens/cut trace缺失。 |
| `CAM-ED-P1-055` | **Open** | Sequencer UX仍为fixed row/feedback，未投影runtime Cut/Director result。 |
| `CAM-ED-P1-056` | **Open** | Input Action -> camera intent链未闭合；physical-first raw input不等于Action authoring。 |
| `CAM-ED-P1-057` | **Open** | Play/Game View没有per-instance possession/eject/debug camera与可观察Play/Simulate差异。 |
| `CAM-ED-P1-058` | **Partial** | descriptor、target generation、history key与局部diagnostic可复用；缺source/artifact/view generation和degraded typed receipt。 |
| `CAM-ED-P1-059` | **Partial** | source roundtrip、controller、ordering/stack/target/history/gizmo/delete/undo有局部tests；Director/lease/blend/collision/cut/preview/multi-player/fault矩阵为空。 |
| `CAM-ED-P1-060` | **Open** | 无1/10/100 rig、1K node、多view/collision/trace/allocation/latency资格，也未迁移global/raw camera旁路。 |

汇总：**49 Open / 11 Partial / 0 Closed**；Partial仅为**1、3、4、8、33、43、47、48、52、58、59**。

## 8. Canonical P2 currentness

| ID | 状态 | 目标 |
|---|---|---|
| `CAM-ED-P2-001` | **Open** | 有预算、warm start、fallback和残差diagnostic的procedural composition/collision solver。 |
| `CAM-ED-P2-002` | **Open** | virtual-production lens calibration、ST map、nodal offset、focus/zoom table和timecode。 |
| `CAM-ED-P2-003` | **Open** | camera animation/motion-matching作为deterministic compiled modifier。 |
| `CAM-ED-P2-004` | **Open** | 大规模shake source field、distance/occlusion/frequency/accessibility filter与bounded spatial selection。 |
| `CAM-ED-P2-005` | **Open** | spectator/replay/photo mode独立Director policy、permission、time/capture/network合同。 |
| `CAM-ED-P2-006` | **Open** | artifact/parameter/target/activation/cut/shake的deterministic recording/rollback。 |
| `CAM-ED-P2-007` | **Open** | large-world/rebase稳定坐标、history与cut policy。 |
| `CAM-ED-P2-008` | **Open** | stable node/parameter/transition/shot identity驱动semantic merge/review。 |
| `CAM-ED-P2-009` | **Open** | plugin camera node schema/evaluator、capability/lease/budget/version/unload fallback。 |
| `CAM-ED-P2-010` | **Open** | camera quality scalability、collision/occlusion/lens/shake/solver tier与traceable fallback。 |
| `CAM-ED-P2-011` | **Open** | 同scene/rig/path/hardware/quality的跨引擎行为、画面与性能基准。 |
| `CAM-ED-P2-012` | **Open** | 分布式camera simulation farm、determinism/cut/reset/fault/迁移与性能分位。 |

## 9. 32项资格门

| Gate | 状态 | 当前判定 |
|---|---|---|
| `CAM-ED-G01` | **Partial** | 3/14字段可走generic Inspector；完整条件字段、unit、undo/save/reopen未达标。 |
| `CAM-ED-G02` | **Partial** | World v2保存裸ID且delete/undo有fallback；stable endpoint/redirect/prefab/cook未达标。 |
| `CAM-ED-G03` | **Fail** | stack/clear-depth/独立masks没有source roundtrip。 |
| `CAM-ED-G04` | **Partial** | descriptor resolver有局部错误报告；source compiler/property-addressed fail-close未达标。 |
| `CAM-ED-G05` | **Fail** | Rig/Lens/Shake create/save/reopen/reference/cook不存在。 |
| `CAM-ED-G06` | **Fail** | Editor/PIE/shipping/cook没有共享camera compiler/evaluator。 |
| `CAM-ED-G07` | **Fail** | 缺artifact/capability时仍以default Orbit/first camera fallback。 |
| `CAM-ED-G08` | **Fail** | Dynamic Runtime默认继续消费右/中键/滚轮并写camera。 |
| `CAM-ED-G09` | **Fail** | 无per-user/Director Action route；raw Input与hardcoded writer仍可同触发。 |
| `CAM-ED-G10` | **Fail** | 两LocalPlayer/viewport独立Director stack/view/history不存在。 |
| `CAM-ED-G11` | **Fail** | activation owner/revoke/target-loss/teardown terminal receipt不存在。 |
| `CAM-ED-G12` | **Fail** | Follow/LookAt/damping/time-domain golden不存在。 |
| `CAM-ED-G13` | **Fail** | Boom/collision/overlap/timeout/teleport合同不存在。 |
| `CAM-ED-G14` | **Fail** | collision与occlusion policy/fallback不存在。 |
| `CAM-ED-G15` | **Fail** | pose/lens/post blend与interruption matrix不存在。 |
| `CAM-ED-G16` | **Fail** | modifier/shake owner/channel/lifecycle不存在。 |
| `CAM-ED-G17` | **Fail** | script仍可raw transform写入，没有scoped Director facade。 |
| `CAM-ED-G18` | **Fail** | AI继续读取global active camera；network/replay view policy缺失。 |
| `CAM-ED-G19` | **Fail** | filmback/focal/aperture/focus/crop/overscan source与golden不存在。 |
| `CAM-ED-G20` | **Fail** | typed Camera Cut Track与Preview/PIE parity不存在。 |
| `CAM-ED-G21` | **Fail** | explicit cut/history epoch及下游reset/hold合同不存在。 |
| `CAM-ED-G22` | **Fail** | 同位置切camera、小硬切与连续大运动仍不能authoritatively区分。 |
| `CAM-ED-G23` | **Partial** | history key已有render identity；player/view/purpose/director/source/cut隔离未达标。 |
| `CAM-ED-G24` | **Fail** | Preview/View Through/Pilot/Lock恢复与transactional write-back不存在。 |
| `CAM-ED-G25` | **Partial** | camera icon/pick/短frustum存在；focus/boom/composition/safe-frame/generation/budget缺失。 |
| `CAM-ED-G26` | **Fail** | Camera Debugger不存在。 |
| `CAM-ED-G27` | **Fail** | Play/Simulate possession与mutation差异不可观察。 |
| `CAM-ED-G28` | **Fail** | rig/node/multiview/collision性能资格不存在。 |
| `CAM-ED-G29` | **Partial** | lower-layer roundtrip/controller/render/delete tests存在；产品compiler/director/preview/fault矩阵未达标。 |
| `CAM-ED-G30` | **Fail** | 本轮未运行Windows camera compiler/runtime/Editor/Play/history lane，且产品实现缺失。 |
| `CAM-ED-G31` | **Fail** | 无真实GPU stack/cut/history/motion/exposure capture资格。 |
| `CAM-ED-G32` | **Fail** | 无activation/target/shake/sequence/resize/multi-player/hot-reload长期soak。 |

Partial只为**G01、G02、G04、G23、G25、G29**；无Pass。

## 10. 现存failure handoff

| failure | current-source判定 | 本报告约束 |
|---|---|---|
| delete subtree可删除全部camera | `open_source_correctness_blocker_dynamic_unverified`；单subtree包含全部camera仍可能令active变0 | M0/M1必须先做generation-bound camera-count preflight，拒绝路径不得detach再restore |
| camera resolution-scale symbol drift | current source已无旧`DEFAULT_RENDER_RESOLUTION_SCALE`符号，但handoff仍Open且无managed product build | 不恢复alias，不从静态修复推导Render06或产品通过 |
| camera table stale map | typed table source repair已存在，原Editor bundle/render extract动态门未执行 | 保留typed component iteration，不恢复旧map或全实体扫描 |
| submit-context camera target sharing | borrowed sequence与terminal target clone的source guard已修正，managed Cargo仍缺 | 不恢复整份extract/camera-list clone，不把guard exit 0当runtime资格 |

## 11. 分层重构顺序

1. **M0 Truthfulness与authority hard cut**：默认关闭Dynamic hardcoded writer；定义World/Player/View/Purpose和Camera capability/owner矩阵；保留controller的Core-contract/Runtime-behavior分层，清除旧路径与raw transform产品旁路。
2. **M1 Endpoint schema与Render source闭环**：versioned endpoint、完整reflection/customization/validation、stable default reference；补Base/Overlay/stack/clear-depth/独立masks/projection/history source和roundtrip。
3. **M2 Rig/Lens/Shake source与compiler**：三类asset/factory/toolkit、stable identity/reference graph/migration；产出immutable artifact、dependency digest、diagnostics、LKG和publication receipt。
4. **M3 per-player/per-viewport Director**：activation lease、target snapshot、fixed evaluation phase、typed ViewResult、generation/currentness与bounded observation。
5. **M4 Blend/Modifier/Collision/Shake**：transition/interruption、SpringArm、collision/occlusion、modifier/shake channels、Physics query generation/budget和deterministic tests。
6. **M5 Cinematic lens/Cut/History**：filmback/focal/aperture/focus/crop/overscan、typed CutEvent、history epoch/key、temporal/post reset handoff与retirement。
7. **M6 Transactional Camera Editor**：endpoint customization、Rig graph、Lens/Shake editors、Preview/View Through/Pilot/Lock、完整gizmo、多上下文Preview和Camera Debugger。
8. **M7 Sequencer/Input/Script/AI/Play集成**：typed Camera Cut、Input Action intent、script Director facade、AI ObserverViewSet、Play possession/eject/multi-instance，删除产品旁路。
9. **M8 fault/scale/migration/竞争资格**：schema/target/collision/sequence/hot-reload/device loss fault matrix，1/10/100 rig、1K node、多view性能门，长时间soak和同语义跨引擎基准。

## 12. 禁止的临时修补

1. 禁止只解除11个`zr_reflect(skip)`就宣称Camera Authoring完成。
2. 禁止把Rig/Lens/Shake/Sequence全部堆进`SceneCameraAsset`巨型component。
3. 禁止复制Camera endpoint为第二套顶层asset真值；只有可复用Rig/Lens/Shake适合独立source。
4. 禁止保留无显式lease的Runtime right/middle/wheel mutation并只改名为debug camera。
5. 禁止让raw Input和hardcoded writer长期消费同一未仲裁event。
6. 禁止用global `active_camera: EntityId`模拟LocalPlayer、viewport、spectator、capture或AI observer。
7. 禁止script/ability/sequence每帧直接覆盖camera transform绕过Director。
8. 禁止把render stack、multi-camera与history key的局部测试当作source-to-product闭环。
9. 禁止用velocity阈值、默认首camera或fixed Sequencer feedback冒充authoritative cut/fallback/success。
10. 禁止在没有同语义功能、画质、硬件与动态receipt时宣称优于Unreal。

本轮仅完成review、currentness重判和重构顺序，没有实施production代码。共享dirty worktree要求进入任何M0-M8前重新冻结selected manifest与failure状态。
