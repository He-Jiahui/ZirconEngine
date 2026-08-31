---
title: Editor Camera Asset、Component、Rig、Controller、Director、Blend、Shake、Cinematic Cut、Play/Simulate 与 Preview 当前源码复核
category: zircon_editor
report_id: Editor207
review_date: 2026-08-28
baseline_head: 11cac2d08a891ee92dcc206fd84a2d15f9e1a3f4
verification_head: a2d8d811c4a3a1fc1db6f5375c491e7e4502533f
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/88-editor-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/104-editor-camera-rig-director-blend-shake-cinematic-cut-current-source-review.md
  - docs/plans/optimize/zircon_editor/151-editor-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-current-source-review.md
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
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity
  - zircon_runtime/src/script/vm/gameplay_host/transform.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_runtime_interface/src/runtime_api/session/camera.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_editor/src/scene/viewport/controller
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/core/play/simulate_camera.rs
  - zircon_editor/src/ui/host/editor_host_event_controller/simulate_camera.rs
  - zircon_editor/src/ui/retained_host/app/simulate_camera_sync.rs
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/ui/asset_editor/preview
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui
  - zircon_plugins/timeline_sequence/editor/src
plan_sources:
  - docs/plans/optimize/zircon_runtime/99za-runtime-camera-endpoint-director-rig-controller-blend-shake-cut-history-multiview-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zy-runtime-cinematic-sequencer-sequence-shot-track-section-binding-hierarchy-evaluation-camera-cut-audio-event-take-recorder-movie-render-queue-network-save-scalability-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/187-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/166-editor-cinematic-sequencer-shot-track-binding-take-recorder-movie-render-queue-current-source-review.md
  - docs/plans/optimize/zircon_editor/206-editor-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-current-source-review.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-08-23-editor-delete-subtree-all-cameras-invariant.md
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
  p1_open: 48
  p1_partial: 12
  p1_closed: 0
  p2_open: 12
  p2_partial: 0
  p2_closed: 0
gate_status:
  fail: 25
  partial: 7
  pass: 0
---

# Editor207 · Camera / Rig / Director / Cut / Play-Simulate / Preview 当前源码复核

## 1. 结论

Zircon当前仍没有工程级Camera产品链。Scene camera endpoint、multi-camera descriptor、Base/Overlay resolver、World v2 active-camera持久化、free/orbit/pan数学、Editor transient viewport camera、camera gizmo和per-camera render history都是真实底座；但排除`dev/docs/tools/.codex/target`后的索引内Rust/ZUI/TOML源码以及当前**2,309个未跟踪Rust/ZUI/TOML文件**对14个核心合同类型的精确检索均为0：`CameraRigDocument`、`CompiledCameraRigArtifact`、`CameraDirector`、`CameraBlend`、`CameraShake`、`SpringArm`、`CineCamera`、`CameraModifier`、`CameraMode`、`CameraCutEvent`、`CameraLensProfile`、`CameraActivationRequest`、`CameraActivationReceipt`和`CameraViewResult`仍不存在。

本轮确认一项Editor151之后的产品进展：Simulate模式现在会从Editor viewport读取transform与projection，通过有界`ZrRuntimeViewportCameraV1`事件路由到对应Play gateway；Dynamic Session校验ABI、finite transform、projection与clip range，并只覆盖`RenderFrameExtract`的主view和selected descriptor，不修改复制后的Play World。runtime测试验证了这条最短链。因此Play/Simulate camera差异不再是完全不可观察，`CAM-ED-P1-057`与`CAM-ED-G27`从Open/Fail校正为Partial。

这条桥仍不是Camera Director。DTO没有player、purpose、source/director generation、tick、owner lease、cut/history epoch或clear disposition；Editor固定发送default viewport，Runtime拒绝所有非default viewport；Editor只在本地去重，camera消失、读取失败或离开同步条件时没有向Runtime发送clear/reset，后者可能继续保留最后一个`editor_camera`。Editor route/lifecycle/multi-instance没有端到端测试，Play仍无possession/eject/debug camera，Simulate也没有activation receipt、restore或sequence ordering。

Scene source到runtime descriptor的断路仍未修。`SceneCameraAsset`有15个endpoint字段，`CameraComponent`有14个字段，但11个被reflection skip；source没有render type、stack、clear depth、独立culling/volume masks、dynamic resolution、temporal jitter或projection override。World extraction继续通过default descriptor隐式得到Base、empty stack与`clear_depth=true`，并把同一scene render layer同时写入culling和volume。

World v2保存裸`active_camera: EntityId`并在load/delete/detach restore时局部修复。Editor delete command还新增了`subtree_component_count::<CameraComponent>`的detach前检查，能阻止单个subtree删除全部camera，这是旧failure现象的source-level进展；但prepare结果没有绑定World generation，Runtime08要求的normalized roots/affected counts/ticket与stale reject仍不存在，精确2/128 camera subtree及100k non-camera产品矩阵也未闭合，所以failure handoff继续Open。

Temporal history key仍只覆盖entity、render order/type、target、viewport和culling/volume layer set。单个live viewport内至少七类per-camera map没有active-set sweep、age/bytes预算或fence-safe retirement；surface extent替换只会整体清空。Velocity继续以far plane 20%、rotation 60度、FOV 15度、ortho 25%和clip 50%的阈值猜测`CameraCutOrInvalid`，无法区分同位置硬切、小幅硬切与连续大运动。

Sequencer继续是静态产品表面：ZUI固定`SEQ_Intro`、`Camera_A`、`Camera Cut 0000-0180 Ready`、12 shots与428 keys；timeline插件没有Camera Cut source/section/binding/evaluator或Director consumer。UI Asset Preview仍只承载UI surface，不是Camera Rig/Lens/Shake preview。

因此Editor30的canonical状态为：**5项P0全部Open；60项P1为48 Open、12 Partial、0 Closed；12项P2全部Open；32个Gate为25 Fail、7 Partial、0 Pass**。没有同scene、同rig、同view count、同硬件、同画质和同输入轨迹的动态receipt，不能声称功能、表现或性能达到或超过Unreal。

## 2. 冻结范围与方法

本报告读取当前共享working tree，以`11cac2d08a891ee92dcc206fd84a2d15f9e1a3f4`标记提交基线；范围内包含其他会话在途修改，本轮不回退、不覆盖、不暂存。文件集由frontmatter所列owner root扩展并去重；物理行以`ReadAllLines`统计，tests统计Rust `#[test]`/`#[tokio::test]`，ignored统计`#[ignore...]`。fingerprint由lowercase repository-relative path、`|`和逐文件SHA-256组成，按路径排序后以LF无尾换行连接并再次SHA-256。

| 选择集 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Runtime camera、controller、session ABI、render/history与consumer纵切面 | **81 / 9,724 / 8,794 / 347,621 / 61 / 2** | `863790b205e28a098dfe0f9dbe8a7f970745bdb420e30595845a40055791046a` |
| Editor viewport、Play/Simulate bridge、asset primitive、preview、Sequencer与测试 | **72 / 14,667 / 13,448 / 520,854 / 107 / 9** | `f242488f1ef992f6c13a040cc8a65fa277828e0cd90b7031acda9317dc15b0c4` |
| Zircon selected union | **153 / 24,391 / 22,242 / 868,475 / 168 / 11** | `0e8c28c000aa23ab1cba6e99488d8768ee8f4d88ad212428fd2416ce09373a06` |
| 五引擎参考集 | **25 / 13,380 / 11,458 / 555,712 / 4 / 0** | `1c4a7e065903e06cd5dbc6cd1dd6007d8f0aa8fa30c1501105c4c98ccd0ab518` |
| all selected | **178 / 37,771 / 33,700 / 1,424,187 / 172 / 11** | `a8be871bb093e711d75be379aeacbb113771f2a5843e29792d78d341fd1cabb0` |

Godot revision为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`，Bevy为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`，Fyrox为`8d815db36494f1badb347547dfc7094bf4fbbdf8`，Unity Graphics为`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal文件由主workspace HEAD和参考集fingerprint冻结。参考集4个test marker均来自Bevy，只说明静态声明。

MVP baseline recovery仍为`in_progress`，F0-F5必须依赖顺序通过；Camera属于高级产品域，本轮只做review。未运行Cargo、Editor、WGPU、Play/Simulate、真实Camera preview、Sequencer、cut/history capture、split-screen/XR、collision、fault、scale、soak或跨引擎benchmark。Tooling按用户要求排除，也没有查询、轮询、等待或实时跟踪协调器。

## 3. 当前产品事实

### 3.1 Endpoint、schema、reflection与持久化

1. `SceneCameraAsset`可round-trip core pipeline、Perspective/Orthographic、FOV/ortho size、clip、surface/texture/headless target、viewport、order、active、HDR、exposure、clear、MSAA与optional post-process。
2. source没有camera专用schema/version、stable field identity、unit/range、unknown policy、migration、semantic diagnostics或capability requirements；asset到component是直接字段映射。
3. `CameraComponent`的14个字段中11个被`#[zr_reflect(skip)]`；generic Inspector只能到达FOV、near和far。
4. `ResourceKind`仍只有26类，没有Camera Rig、Lens Profile或Camera Shake；type registry、factory、toolkit、thumbnail、reference analyzer、cook role与open route均为零。
5. `World::set_active_camera`返回`()`并静默忽略非法或非camera entity；`CameraComponent::is_active`、global active ID和extract override形成三套未统一选择语义。
6. persistent state只保存裸ID；没有World/source generation、redirect、prefab/cook identity、player、viewport、purpose、owner、possession、blend或terminal receipt。

### 3.2 Render descriptor、stack与history

1. `CameraRenderDescriptor`已表达Base/Overlay、stack、clear depth、独立masks、target、viewport和snapshot；ordering与resolver有真实局部测试。
2. Scene source/component没有stack字段。World builder只填写order、target、viewport、clear和两个mask，其余依赖default descriptor，authoring无法到达Overlay/stack/clear-depth。
3. culling与volume均由同一个32-bit scene mask转换；没有独立volume trigger、wide-layer source migration或条件Inspector。
4. `ViewportCameraSnapshot`的aspect、projection override、dynamic resolution和temporal jitter没有Scene source；Editor transient与manual DTO构成旁路。
5. resolver能报告missing/non-overlay/target mismatch/overlay-has-stack；source不可创作，duplicate ownership、cycle、orphan、shipping fail disposition与publication receipt仍不完整。
6. `ViewportCameraHistoryKey`覆盖七类render identity并共享wide layer storage；不含World、player、view purpose、endpoint/director/source generation或cut epoch。
7. `ViewportRecord`至少有camera histories、motion vector camera、particle previous sprites、Hybrid GI、Virtual Geometry、light-grid report和debug snapshot七类keyed map；只有surface replacement整体清空，无camera retirement预算。

### 3.3 Controller、Input、Script与AI authority

1. Core保留Free/Orbit/Pan settings/input/output/state，Runtime Input拥有controller行为；这是正确hard cut，不得恢复旧controller shim。
2. controller只接受normalized数学DTO，不读取Input Action artifact、InputUser、context、device lease、viewport owner或CameraMode。
3. Dynamic Session仍无条件构造Orbit controller；未被Runtime UI消费的right/middle/scroll直接修改global active camera transform。
4. FocusLost只提交给Input reducer，不结束`RuntimeCameraController.drag`；UI consume与camera writer顺序由手写分支决定，没有统一capture/ownership receipt。
5. script `camera_follow`直接写任意entity transform，无CameraComponent验证、target generation、damping、collision、blend、owner或receipt。
6. AI behavior tick每帧读取`world.active_camera()`位置决定LOD；server、spectator、split-screen、capture与AI observer没有qualified view family。

### 3.4 Play/Simulate Camera bridge

1. Retained host每tick只在`PlayKind::Simulate`且绑定Play instance时读取Editor viewport camera，并以last `(instance, camera)`跳过重复发送。
2. `ZrRuntimeViewportCameraV1`只有ABI、transform、projection kind、FOV、ortho size、near和far；有bounded JSON payload和finite/range校验。
3. Runtime在extract cache读取后覆盖render view，不改变Play World；这是明确且可测试的Simulate语义底座。
4. event虽携带viewport handle，Editor固定default handle，Runtime会拒绝非default handle；不支持split view、Game View、capture view或XR view。
5. 没有clear/reset DTO。Editor camera不可读、route返回false或本地状态清空时，Runtime保存的`editor_camera`没有对应撤销命令。
6. 没有source/director generation、sequence、timestamp、owner、purpose、cut/history epoch、ack/receipt；Editor route和lifecycle没有产品级测试。

### 3.5 Editor viewport、Inspector、Preview与Pilot

1. Editor viewport持有单一transient `ViewportCameraSnapshot`与Orbit controller；它不是Scene camera source、per-view Camera Session或Director instance。
2. generic scene transaction可创建/删除camera node、preview/commit transform并undo；camera property transaction因reflection缺口只覆盖3字段。
3. camera gizmo只有icon、pick sphere和短perspective frustum，near抬到`0.05`、far截到`2.5`；无orthographic、filmback、focus plane、boom、composition、安全框、show flag、generation或预算。
4. `UiAssetPreviewHost`只用于UI surface；没有Camera Rig/Lens/Shake preview world、time、target、aspect、input、artifact generation或terminal job receipt。
5. View Through、Pilot、Lock、bookmark product、capture-loss/Esc/scene-switch恢复、transactional write-back和Camera Debugger仍不存在。

### 3.6 Sequencer与Cinematic Cut

1. Sequencer workspace的sequence、camera、cut、shot/key count与range均为固定ZUI文本，route只改变control state或固定feedback。
2. timeline editor插件只提供通用AnimationSequence descriptor和局部key helper；没有Camera binding/cut section/shot evaluator、Director lease或restore state。
3. runtime Animation Sequence只支持entity/property curve采样，没有typed Camera Cut、rational time、pre-animated camera state或history epoch。
4. Editor166拥有通用Cinematic产品；Editor30只能提供Camera binding、Director与Cut/History adapter，不得复制第二套timeline/compiler/evaluator。

## 4. Owner边界与目标合同

| 领域 | 唯一owner | Editor207职责 |
|---|---|---|
| endpoint source/compiler/artifact/director/evaluation | Runtime126 / Runtime Camera | Editor只编辑source并消费artifact、diagnostic、view result与receipt |
| Render stack/history/temporal/capture | Runtime Render | 消费qualified view与cut/history epoch，不从Editor读取第二真值 |
| Rig/Lens/Shake documents与toolkits | Editor30 + Runtime compiler | document/session/transaction/preview，不复制runtime evaluator |
| Scene viewport navigation与Pilot | Editor187 / Editor66 | transient per-view session，Pilot以lease连接scene endpoint |
| Timeline/Shot/Cut | Editor166 / Editor45 | 使用通用timeline identity/evaluator，只扩展typed Camera Cut |
| InputUser/Action/possession | Runtime Input + Editor206 / Editor29 | Camera消费qualified intent与owner lease，不读取raw mouse |
| Play/Simulate process/world | Editor07 | instance lifecycle与gateway；Camera只提供view override/possession adapter |
| collision/occlusion | Runtime Physics | generation/budget/fault-qualified query，不建立Camera私有world |
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
  -> Pilot / Sequencer / Debugger / Play-Simulate adapter
  -> same compiler, evaluator and qualified runtime snapshots
```

最低跨层合同必须包含World/Player/View/Purpose identity、source/artifact/director generation、stable endpoint/target address、activation owner/priority/lease、fixed evaluation phase、bounded target/collision query、typed terminal receipt、explicit cut/history epoch及bounded observation/retirement。

## 5. 五套参考引擎的直接差异

| 参考 | 直接源码合同 | Zircon应吸收 | 不应照抄 |
|---|---|---|---|
| Unreal Engine | CameraComponent有aspect/post-process与`NotifyCameraCut`；PlayerCameraManager有current/pending ViewTarget、blend、modifier、shake与cache；SpringArm有probe/collision/lag；CineCamera有filmback/lens/focus；MovieScene有typed CameraCutTrack；GameplayCameras有Rig build/evaluator/blend stack/shake/collision node | per-player authority、versioned source/build、typed evaluator result、独立blend/modifier/shake/collision生命周期、authoritative cut | UObject宏、历史compat层和未量化默认tick成本 |
| Godot | Camera3D按Viewport current lifecycle，支持perspective/ortho/frustum、cull/environment/attributes/compositor及project/unproject；Editor有custom camera preview | per-viewport activation、明确preview切换、完整projection/frustum工具 | 把单current camera当多人Director上限 |
| Bevy | `Camera`与`ComputedCameraValues`分离，target/viewport/order/active/sub-camera明确，conversion可失败，自定义projection可扩展；controller在独立crate | source/derived分离、fallible conversion、controller边界与显式enable | 用ECS组合本身替代owner/lease/director/cinematic合同 |
| Fyrox | Camera主要属性Reflect/Visit，支持viewport/target/frustum/project/unproject/ray；Editor camera speed/sensitivity/zoom持久化 | reflection/serialization覆盖、normalized viewport、query工具和profile持久化 | 复制局部builder默认或未经验证setter |
| Unity Graphics | URP序列化Base/Overlay stack、clear depth、volume mask/trigger与AA；HDRP按Camera/XR pass/history channel管理history，支持view-count与显式reset | source-to-stack闭环、条件Inspector、history channel/lifecycle、XR view count与reset | pipeline-specific MonoBehaviour/static cache owner；本地Graphics不含Cinemachine，不推测未收录能力 |

共同工程原则是：source与derived runtime state分离；每个player/view/purpose有owner和lifecycle；cut/failure是显式事件；Editor投影同一compiler/evaluator；history与async query有generation、budget和retirement。

## 6. Canonical P0 currentness

| ID | 状态 | 当前证据与硬切目标 |
|---|---|---|
| `CAM-ED-P0-001` | **Open** | Dynamic Session默认安装Orbit并在UI未消费时直接写global camera；迁为默认关闭、capability/InputUser/CameraMode/lease驱动的DevCamera，shipping走Director。 |
| `CAM-ED-P0-002` | **Open** | 14字段仅3字段反射，Rig/Lens/Shake resource/factory/toolkit为零；建立完整endpoint Inspector和三类versioned source/compiler产品。 |
| `CAM-ED-P0-003` | **Open** | stack validator只在descriptor/tests可达，Scene extraction依赖Base/empty/clear-depth默认并复用两个mask；完成source/Inspector/roundtrip/compiler/artifact链。 |
| `CAM-ED-P0-004` | **Open** | raw active ID仍被render fallback、dynamic writer、script和AI混用；建立per-player/per-viewport/purpose Director、possession和qualified ViewResult。 |
| `CAM-ED-P0-005` | **Open** | Sequencer Cut仍是固定UI，Velocity仍猜cut；建立typed binding/track/section、Director handshake、`CameraCutEvent`与monotonic history epoch。 |

## 7. Canonical P1 currentness

| ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| `CAM-ED-P1-001` | **Partial** | generic Inspector仅能编辑FOV/near/far；补typed property、unit、validation、transaction、conditional layout与multi-edit。 |
| `CAM-ED-P1-002` | **Open** | 无camera专用schema/version/unknown policy；补canonical codec、migration、compatibility与diagnostic spans。 |
| `CAM-ED-P1-003` | **Partial** | World v2保存裸active ID且load/delete/undo有fallback，subtree camera count也会预检；升级为generation-qualified stable endpoint reference与typed receipt。 |
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
| `CAM-ED-P1-014` | **Open** | Editor/preview/Play/Simulate/cook/shipping没有共享semantic compiler与immutable artifact。 |
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
| `CAM-ED-P1-050` | **Open** | 无Camera Preview tile、View Through或qualified preview generation；transient viewport和Simulate override都不是替代。 |
| `CAM-ED-P1-051` | **Open** | 无Pilot/Lock/capture-loss/Esc/scene-switch状态机和transactional transform write-back。 |
| `CAM-ED-P1-052` | **Partial** | icon、pick sphere和短frustum存在；缺ortho/filmback/focus/boom/composition/safe-frame、show flag、generation与预算。 |
| `CAM-ED-P1-053` | **Open** | target/input/aspect/device/time/post/collision/platform多上下文Preview Session缺失。 |
| `CAM-ED-P1-054` | **Open** | live Camera Debugger、active stack/owner/node/blend/collision/shake/lens/cut trace缺失。 |
| `CAM-ED-P1-055` | **Open** | Sequencer UX仍为fixed row/feedback，未投影runtime Cut/Director result。 |
| `CAM-ED-P1-056` | **Open** | Input Action -> camera intent链未闭合；raw physical input不等于Action authoring。 |
| `CAM-ED-P1-057` | **Partial** | Simulate可把Editor camera有界覆盖到Play extract且不改World；缺clear/generation/multi-view、Play possession/eject、Director和Editor E2E。 |
| `CAM-ED-P1-058` | **Partial** | descriptor、target generation、history key与局部diagnostic可复用；缺source/artifact/view generation和degraded typed receipt。 |
| `CAM-ED-P1-059` | **Partial** | source roundtrip、controller、ordering/stack/target/history/gizmo/delete/undo/Simulate runtime有局部tests；Director/lease/blend/collision/cut/preview/multi-player/fault矩阵为空。 |
| `CAM-ED-P1-060` | **Open** | 无1/10/100 rig、1K node、多view/collision/trace/allocation/latency资格，也未迁移global/raw camera旁路。 |

汇总：**48 Open / 12 Partial / 0 Closed**；Partial为**1、3、4、8、33、43、47、48、52、57、58、59**。

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
| `CAM-ED-G02` | **Partial** | World v2保存裸ID且load/delete/undo有fallback；stable endpoint/redirect/prefab/cook未达标。 |
| `CAM-ED-G03` | **Fail** | stack/clear-depth/独立masks没有source roundtrip。 |
| `CAM-ED-G04` | **Partial** | descriptor resolver有局部错误报告；source compiler/property-addressed fail-close未达标。 |
| `CAM-ED-G05` | **Fail** | Rig/Lens/Shake create/save/reopen/reference/cook不存在。 |
| `CAM-ED-G06` | **Fail** | Editor/Play/Simulate/shipping/cook没有共享camera compiler/evaluator。 |
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
| `CAM-ED-G20` | **Fail** | typed Camera Cut Track与Preview/Play parity不存在。 |
| `CAM-ED-G21` | **Fail** | explicit cut/history epoch及下游reset/hold合同不存在。 |
| `CAM-ED-G22` | **Fail** | 同位置切camera、小硬切与连续大运动仍不能authoritatively区分。 |
| `CAM-ED-G23` | **Partial** | history key已有render identity；player/view/purpose/director/source/cut隔离未达标。 |
| `CAM-ED-G24` | **Fail** | Preview/View Through/Pilot/Lock恢复与transactional write-back不存在。 |
| `CAM-ED-G25` | **Partial** | camera icon/pick/短frustum存在；focus/boom/composition/safe-frame/generation/budget缺失。 |
| `CAM-ED-G26` | **Fail** | Camera Debugger不存在。 |
| `CAM-ED-G27` | **Partial** | Simulate extract override与no-World-mutation可观察；Play possession/eject、clear/generation/multi-instance parity未达标。 |
| `CAM-ED-G28` | **Fail** | rig/node/multiview/collision性能资格不存在。 |
| `CAM-ED-G29` | **Partial** | lower-layer roundtrip/controller/render/delete/Simulate tests存在；产品compiler/director/preview/fault矩阵未达标。 |
| `CAM-ED-G30` | **Fail** | 本轮未运行Windows camera compiler/runtime/Editor/Play/history lane，且产品实现缺失。 |
| `CAM-ED-G31` | **Fail** | 无真实GPU stack/cut/history/motion/exposure capture资格。 |
| `CAM-ED-G32` | **Fail** | 无activation/target/shake/sequence/resize/multi-player/hot-reload长期soak。 |

Partial为**G01、G02、G04、G23、G25、G27、G29**；无Pass。

## 10. 现存failure handoff

| failure | current-source判定 | 本报告约束 |
|---|---|---|
| delete subtree可删除全部camera | `open_source_guard_partial_dynamic_unverified`；Editor已有subtree camera-count precheck，但无generation-bound Runtime ticket和精确2/128/100k产品矩阵 | M0/M1保留subtree-scoped count，迁为同代prepare ticket；拒绝路径不得detach再restore |
| camera resolution-scale symbol drift | current source已无旧`DEFAULT_RENDER_RESOLUTION_SCALE`符号，但handoff仍Open且无managed product build | 不恢复alias，不从静态修复推导Render06或产品通过 |
| camera table stale map | typed table source repair已存在，原Editor bundle/render extract动态门未执行 | 保留typed component iteration，不恢复旧map或全实体扫描 |
| submit-context camera target sharing | borrowed sequence与terminal target clone的source guard已修正，managed Cargo仍缺 | 不恢复整份extract/camera-list clone，不把guard exit 0当runtime资格 |

Editor151曾把delete-subtree failure错误链接到`docs/plans/performance/01/...`；当前canonical handoff物理路径是`docs/plans/zircon_runtime/runtime/08/failure-2026-08-23-editor-delete-subtree-all-cameras-invariant.md`，本报告已纠正route，但不擅自关闭owner记录。

## 11. 分层重构顺序

1. **M0 Truthfulness与authority hard cut**：默认关闭Dynamic hardcoded writer；定义World/Player/View/Purpose和Camera capability/owner矩阵；清除raw transform产品旁路。
2. **M1 Endpoint schema与Render source闭环**：versioned endpoint、完整reflection/customization/validation、stable reference；补Base/Overlay/stack/clear-depth/独立masks/projection/history source与roundtrip。
3. **M2 Rig/Lens/Shake source与compiler**：三类asset/factory/toolkit、stable identity/reference graph/migration；产出immutable artifact、dependency digest、diagnostics、LKG和publication receipt。
4. **M3 per-player/per-viewport Director**：activation lease、target snapshot、fixed evaluation phase、typed ViewResult、generation/currentness与bounded observation。
5. **M4 Blend/Modifier/Collision/Shake**：transition/interruption、SpringArm、collision/occlusion、modifier/shake channels、Physics query generation/budget和deterministic tests。
6. **M5 Cinematic lens/Cut/History**：filmback/focal/aperture/focus/crop/overscan、typed CutEvent、history epoch/key、temporal/post reset与retirement。
7. **M6 Transactional Camera Editor**：endpoint customization、Rig graph、Lens/Shake editors、Preview/View Through/Pilot/Lock、完整gizmo、多上下文Preview和Camera Debugger。
8. **M7 Sequencer/Input/Script/AI/Play-Simulate集成**：typed Camera Cut、Input Action intent、script Director facade、AI ObserverViewSet、Play possession/eject/multi-instance；把Simulate桥升级为generation-qualified override/clear receipt。
9. **M8 fault/scale/migration/竞争资格**：schema/target/collision/sequence/hot-reload/device loss fault matrix，1/10/100 rig、1K node、多view性能门，长期soak和同语义跨引擎基准。

## 12. 禁止的临时修补

1. 禁止只解除11个`zr_reflect(skip)`就宣称Camera Authoring完成。
2. 禁止把Rig/Lens/Shake/Sequence全部堆进`SceneCameraAsset`巨型component。
3. 禁止复制Camera endpoint为第二套顶层asset真值；只有可复用Rig/Lens/Shake适合独立source。
4. 禁止保留无显式lease的Runtime right/middle/wheel mutation并只改名为debug camera。
5. 禁止把Simulate viewport DTO或Render Extract override包装成Camera Director、possession或Camera Preview。
6. 禁止只清Editor的`last_simulate_camera`而让Runtime继续保留stale override；必须有typed clear与ack/currentness。
7. 禁止用global `active_camera: EntityId`模拟LocalPlayer、viewport、spectator、capture或AI observer。
8. 禁止script/ability/sequence每帧直接覆盖camera transform绕过Director。
9. 禁止把render stack、multi-camera与history key的局部测试当作source-to-product闭环。
10. 禁止用velocity阈值、默认首camera或fixed Sequencer feedback冒充authoritative cut/fallback/success。
11. 禁止在没有同语义功能、画质、硬件与动态receipt时宣称优于Unreal。

本轮仅完成review、currentness重判和重构顺序，没有实施production代码。共享dirty working tree要求进入任何M0-M8前重新冻结selected manifest与failure状态。
