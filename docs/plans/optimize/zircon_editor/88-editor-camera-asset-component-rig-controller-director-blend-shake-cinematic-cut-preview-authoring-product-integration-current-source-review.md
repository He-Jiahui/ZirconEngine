---
title: Editor Camera Asset、Component、Rig、Controller、Director、Blend、Shake、Cinematic Cut、Preview Authoring 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor88
review_date: 2026-08-24
baseline_head: c02a7fb7c4b90381b9e701008bc8a2898fc09263
baseline_epoch: 415
freeze_head: 1538a67d526d4c8dff93aa96e189751c06f80ad6
final_recheck_head: 39fe594bdaef6555277386dcc38362a575ada1c6
canonical_owner: Editor30
refreshes:
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
related_reports:
  - docs/plans/optimize/zircon_runtime/99za-runtime-camera-endpoint-director-rig-controller-blend-shake-cut-history-multiview-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/66-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/83-editor-cinematic-sequencer-shot-track-section-binding-hierarchy-evaluation-camera-cut-audio-event-take-recorder-movie-render-queue-product-integration-current-source-review.md
related_code:
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/assets/project_document/scene.rs
  - zircon_runtime/src/scene/components/scene/camera.rs
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/core/framework/camera_controller
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/script/vm/gameplay_host/transform.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/scene/viewport/controller
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/ui/asset_editor/preview
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui
  - zircon_plugins/timeline_sequence/editor/src
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
doc_type: current_source_refresh
review_status: complete
implementation_status: not_started
source_recheck_required: true
canonical_finding_delta:
  p0: 0
  p1: 0
  p2: 0
finding_status:
  open: 66
  partial: 11
  closed: 0
gate_status:
  fail: 26
  partial: 6
  pass: 0
---

# Editor Camera Asset、Component、Rig、Controller、Director、Blend、Shake、Cinematic Cut、Preview Authoring 与 Product Integration 当前源码复核

## 1. 结论

Editor30 的五项 P0 在当前工作树中仍全部成立。Zircon 已有真实的 Scene camera endpoint、typed target/viewport、Perspective/Orthographic、render ordering、Base/Overlay descriptor/validator、free/orbit/pan 数学、World v2 active-camera 持久化、Editor transient viewport camera、可选取的 camera frustum gizmo，以及局部 render/history/controller 测试；这些底座应保留。

但仓库仍没有工程级 Camera 产品链。`CameraComponent` 的 14 个字段仍只有 FOV、near、far 进入 reflection；26 类 `ResourceKind` 中仍没有 Camera Rig、Lens Profile 或 Camera Shake；production 精确检索仍没有 `CameraRig`、`CameraDirector`、`CameraShake`、`SpringArm`、`CineCamera`、`CameraModifier`、`CameraCutEvent` 或 `CameraViewResult`。当前仅有的 `CameraRig` 命中是测试/展示 fixture 文本，不是领域实现。

Scene source 不能表达 render descriptor 已支持的 Base/Overlay、stack、clear depth、独立 culling/volume mask、dynamic resolution、temporal jitter或 projection override。World extraction 仍以默认 descriptor 补这些字段，并把同一 scene render-layer mask 同时复制给 culling 与 volume。手工 DTO 和 renderer tests 证明了底层能力，却没有 source -> Inspector -> transaction -> save/reopen -> compile -> play/capture 的产品闭环。

动态 Runtime 仍固定构造 `RuntimeCameraController`。当前工作树把物理 pointer/mouse/keyboard/touch/gamepad 提交移动到 Runtime UI 传播停止之前，这是 Input truth 的真实修复；但未被 UI 语义消费的右键、中键和滚轮仍同时进入通用 Input 与硬编码 Orbit writer，并直接改 `world.active_camera()` transform。没有 DevCameraMode capability、InputUser、LocalPlayer、viewport owner、possession、director arbitration 或 terminal receipt。focus loss 也没有结束 camera drag。

`active_camera` 的旧结论需要精确纠正：World project JSON v2 和 `WorldPersistentState` 已持久化裸 `EntityId`，删除当前 camera 会选择稳定 fallback，detached subtree undo 也会恢复原 active camera；但 `SceneAsset` 仍只保存 entities，没有 stable endpoint reference。`set_active_camera` 仍静默忽略非法值，裸 ID 也没有 world/source generation、redirect、prefab/cook identity、owner 或 blend，因此不能承担 per-player/per-viewport director。

Editor viewport 已有 camera icon 和短截锥 gizmo，故旧“frustum 为零”证据过时；当前 gizmo 只读取 FOV/near/far，以 viewport snapshot aspect 绘制，并把 far 截到 2.5、near 抬到 0.05。它不是完整 filmback/focus/boom/composition/safe-frame visualization，也没有 source/artifact generation。通用 UI Asset Preview 只预览 ZUI，不是 Camera Preview。View Through、Pilot、Lock、可逆写回和 Camera Debugger仍不存在。

Sequencer 的 `Camera_A`、`Camera Cut 0000-0180 Ready`、Preview/Validate 继续来自静态 ZUI、route 与 fixed feedback。`AnimationSequenceAsset`/compiled sequence只持有 entity/property curve track，没有 typed camera binding、cut section、shot/take、director lease或history epoch。Temporal velocity继续以 pose/projection阈值猜测 `CameraCutOrInvalid`。

本轮不新增 finding，只对 Editor30 的 77 项 canonical 账本做 current-source 重判：**5 P0 Open；60 P1 为 49 Open、11 Partial、0 Closed；12 P2 全部 Open；32 Gate 为 26 Fail、6 Partial、0 Pass**。目标仍是：

```text
CameraEndpointComponent + CameraLensProfileDocument
  + CameraRigDocument + CameraShakeDocument
  -> CameraSemanticCompiler
  -> immutable CompiledCameraProgram
  -> per-World/per-Player/per-View CameraDirectorService
  -> CameraViewResult + CameraCutEvent + HistoryEpoch
  -> Render / Audio / AI / Network / Save adapters

Editor CameraDocumentSession
  -> Inspector / Rig Graph / Preview / View Through / Pilot / Sequencer / Debugger
  -> same compiler, evaluator and qualified runtime snapshots
```

“性能和表现优于当前 Unreal”目前没有同语义证据。Zircon 缺少 rig/director/blend/collision/shake/cinematic 与多玩家产品，不能用较少功能的低成本冒充性能领先。先完成同语义闭环，再在相同硬件、scene、rig、view count、画质和输入轨迹下比较 compile latency、evaluation CPU/allocation、physics query budget、render/history bytes、input-to-view latency、tail latency、画面稳定性和长时间 churn。

## 2. Owner、currentness 与物理冻结

### 2.1 唯一 owner 与不重复计数

| 主题 | 唯一 owner | Editor30 / Editor88 边界 |
|---|---|---|
| Camera source/compiler/director/evaluation/history | Runtime126 | Editor消费 immutable artifact/view/diagnostic，不复制 Runtime evaluator或history truth |
| Camera endpoint、Rig/Lens/Shake authoring、Preview/Pilot | Editor30 | 本文刷新 canonical 账本，不新增第二套 finding |
| Scene viewport navigation/profile/framing/bookmark | Editor66 | Camera-specific View Through/Pilot adapter接入其 per-viewport session，不复制导航控制器 |
| Cinematic source、Sequencer、shot/take/movie queue | Editor83（刷新 Editor45） | Camera Cut extension提供 typed binding/director handshake，不复制通用 timeline/compiler |
| Scene document/transaction/save/recovery | Editor02/03/05/63 | Camera edits提交 typed command/property address，不另建 history/dirty authority |
| Asset type/factory/toolkit/reference graph | Editor04 | Rig/Lens/Shake作为贡献接入，不硬编码平行 catalog |
| Input Action/InputUser/device | Runtime Input + Editor87 | Camera只消费 scoped intent snapshot，不读取 raw mouse或复制 input map |
| Play/PIE/Game View/possession | Editor07 + Runtime Gameplay | 提供 player/viewport/session identity，不让 Editor scene camera冒充 gameplay owner |
| Physics collision/occlusion | Runtime Physics | 提供 generation/budget query snapshot，Camera不私建物理世界 |
| Render/post/history/capture | Runtime Render + Editor22 | 消费 qualified CameraViewResult/CutEpoch，不让 Editor直接写 renderer cache |

### 2.2 Currentness 与共享工作树

- 协调 session 为 `optimize-editor88-camera-authoring-current-review-r1-20260824`，model tier `5.6-sol`、thinking depth `Extra High`；注册基线 `c02a7fb7c4b90381b9e701008bc8a2898fc09263`、epoch `415`。注册请求在 coordinator 启动窗口短暂超时，随后确认为 completed，四个文档路径 lease 无冲突。
- source 冻结 HEAD 为 `1538a67d526d4c8dff93aa96e189751c06f80ad6`。共享工作树不是 clean HEAD；168 个 Zircon selected 文件中有 43 个在途路径，集中在 viewport controller、asset registry、Sequencer ZUI/plugin、dynamic session、World hierarchy和未跟踪 physical-input/detached-batch tests。本文读取当前 working-tree bytes，不回退、不覆盖，也不把未验收改动写成发布能力。
- 最终校验时 HEAD 前移到 `39fe594bdaef6555277386dcc38362a575ada1c6`；区间内只提交了 coordinator failure-graph 文档，没有命中193个selected文件。第三次完整复算的五组fingerprint与首次冻结逐项一致。
- `events.rs` 的 physical-first 输入顺序、World active-camera fallback/undo与 frustum gizmo 是本轮必须保留的时效修正。相机 source/component/render descriptor本身没有新增 Rig/Director/Lens/Shake/Cut 产品。
- Runtime126 仍是当前 Runtime Camera owner；其 baseline 后 World delete/undo 与输入顺序有在途变化，后续 Runtime owner必须重新冻结相关 status。本文只重判 Editor30 authoring/product账本。
- MVP `00-current-source-baseline-recovery` 仍为 `in_progress`。本轮是 review-only 文档交付，不实施 Camera M0-M9，也不展开用户已排除的 tooling 优化。

### 2.3 可复算 selected set

统计按 normalized relative path 排序；每个文件 SHA-256 后，以 `lowercase path + NUL + lowercase hash + LF` 拼接再计算集合 SHA-256。test declarations 是静态词法计数，不表示运行或通过。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Runtime camera 与产品集成纵切面 | **93 / 20,809 / 19,178 / 767,434 / 130 / 2** | `97670459e6bb3efa2f52e10bdd2901dc852d4bddc406fc116e433e74ffd7422a` |
| Editor authoring、viewport、asset、Sequencer与测试 | **75 / 15,772 / 14,667 / 584,343 / 82 / 1** | `75d1c4ddb11356a7bdf97f37aa7a8940995f33cab0af4cd66feaa5da0d651ac5` |
| Zircon selected union | **168 / 36,581 / 33,845 / 1,351,777 / 212 / 3** | `a5d7e5fbed66e6b81b3fdb4080b7f5fa091e8ab2218b0fa9be2ec29c7524cfc3` |
| 五引擎参考集 | **25 / 13,380 / 11,458 / 555,712 / 4 / 0** | Unreal 10、Godot 4、Bevy 4、Fyrox 2、Unity Graphics 5；`a6e30382bf3a6d13a5f827f65b7c80d9fb44ea47721c777d0ffd8d694fc3e0e8` |
| all selected | **193 / 49,961 / 45,303 / 1,907,489 / 216 / 3** | `9e7943535883f2976508fae8cb83fdfd911ceaf9996a1da412ffcada027cd55d` |

Zircon selection覆盖：Scene asset/project document/cache payload、Camera component/World persistence/render extraction、完整 camera-controller目录、render camera/order/stack/view、Dynamic Session camera/input、script/AI、camera submit/history/velocity、focused tests；Editor覆盖完整 viewport controller、interaction、projection/render/settings、asset type registry、generic command/preview锚点、Sequencer产品表面、timeline plugin与compiled animation sequence。参考 selection 为 frontmatter所列五个引擎的 camera/runtime/editor直接合同；不以全仓关键词结果代替逐文件 owner/consumer/lifecycle阅读。

## 3. 当前实现逐层事实

### 3.1 Scene source、component、reflection 与持久化

1. `SceneCameraAsset` 与 `CameraComponent`真实保存 core pipeline、projection、FOV/ortho size、clip、target、viewport、order、active、HDR、exposure、clear、MSAA 和 optional post process。
2. Camera source没有独立 schema version、stable field identity、unit/range、unknown-field policy、migration或semantic diagnostic；project conversion直接映射字段，没有 finite/range/cross-field admission。
3. `CameraComponent` 14 个 public 字段中只有 `fov_y_radians/z_near/z_far`反射；projection、ortho、target、viewport、order、is_active、HDR、exposure、clear和MSAA均被 skip。
4. `SceneAsset`只保存 entity vector；authoring TOML通过 flattened rest保留部分未知字段，但没有明确 version/compatibility policy，也没有 stable default endpoint reference。
5. World JSON v2持久化裸 `active_camera`；load会normalize无效 camera，remove/detach会稳定fallback，undo恢复原active camera。这是可保留进展，但裸ID没有 source/world generation、redirect、prefab/cook identity或diagnostic receipt。
6. `set_active_camera`返回 `()`，非法/非camera请求静默无效；`is_active`、global active selection与render request override形成三个缺少统一authoring语义的选择层。

### 3.2 Render endpoint、stack、history 与 source 断路

1. `CameraRenderDescriptor`已表达 Base/Overlay、stack、clear depth、独立 masks、target、viewport和snapshot；ordering/stack resolver及target tests是真实底座。
2. Scene source/component没有 render type、stack、clear depth或独立 masks。extraction用默认 Base/empty stack/clear depth，并把同一 32-bit scene mask复制给 culling和volume。
3. `ViewportCameraSnapshot`有projection override、dynamic-resolution scale与jitter；它们没有Scene authoring source，Editor transient snapshot和手工DTO成为隐式第二入口。
4. stack resolver有missing/non-overlay/target mismatch/overlay-has-stack诊断，但source无法创作，且orphan/duplicate/cycle/duplicate ownership与shipping fail disposition不完整。
5. renderer已有多camera、custom target、target generation和局部history invalidation；这不等于per-player director或完整source产品。
6. history key仍缺 World/Player/View/Director/source generation/lens/projection/cut epoch；live viewport内旧per-camera map没有统一active-set retirement/age/bytes/fence policy。
7. velocity以translation/rotation/FOV/projection阈值推断`CameraCutOrInvalid`，只能作损坏保护，不能替代authoritative cut。

### 3.3 Controller、Input、Script、AI 与架构边界

1. free/orbit/pan有typed input/settings/state/output和focused数学测试；可复用的是数学 kernel，不是shipping camera authority。
2. stateful controller当前位于`core::framework::camera_controller`，违反“framework只保留中立合同，runtime拥有行为/生命周期”的目标边界。实施必须hard cut，不保留re-export/shim第二authority。
3. Dynamic Session固定安装Orbit controller，右/中键启动drag，scroll直接zoom active scene camera；无profile/capability/owner/InputUser/viewport/possession/director。
4. 当前 physical-first提交保证UI capture不能隐藏物理release；UI consumed仍可阻止hardcoded camera handler，但未消费时同一event继续同时驱动raw Input和camera writer。
5. focus lost只提交Input事件，没有清理camera drag；wheel最终丢失设备精度语义，controller的viewport_size合同也没有成为完整DPI/device policy。
6. script `camera_follow`在宽泛`gameplay.entity`能力下直接写任意transform；AI每tick读取global active camera计算Behavior LOD。两者都绕过qualified view/director authority。

### 3.4 Editor viewport、Inspector、Preview 与 Pilot

1. Editor viewport在第一次导航后持有transient `ViewportCameraSnapshot`；projection使用Editor settings而非Scene camera component，ortho size也保存在transient snapshot，形成可见性不足的分叉。
2. generic scene transaction支持创建/删除camera节点、transform preview/commit和undo；但camera-specific lens/render/activation edits因reflection缺口不可达，也没有Camera authoring document/session。
3. generic Inspector基础可以显示三个反射字段，因此P1-1/P1-48为Partial；不存在projection/lens/target/stack/layer/post/activation的conditional customization、unit、picker或multi-edit闭环。
4. `render_packet.rs`为camera绘制icon、pick sphere与短frustum，证明gizmo不是零；但far/near被展示性截断，orthographic/filmback/focus/boom/composition/safe-frame和generation qualification缺失。
5. 通用`UiAssetPreviewHost`只构建UI surface；它不能预览Camera Rig/Lens/Shake，也没有world/time/aspect/device/source/artifact generation。
6. 当前没有View Through、Pilot、Lock、camera bookmark product、可逆scene transform写回、capture-loss恢复或Camera Debugger。

### 3.5 Sequencer、Cut 与 Cinematic

1. Sequencer ZUI仍硬编码`Camera_A`与`Camera Cut 0000-0180 Ready`，19条route主要改变control selection/popup/text；Preview/Validate返回fixed product feedback。
2. timeline plugin注册AnimationSequence helper/descriptor，但Workbench `workbench.extension.sequencer.*` namespace没有连到真实operation/document/provider。
3. `AnimationSequenceAsset`只含entity binding/property channel，compiled runtime只采样并写scene property；没有camera binding、cut/shot section、director lease、pre-roll/restore、history epoch或capture identity。
4. Camera Cut的通用Cinematic产品由Editor83/Editor45拥有；Editor30只提供camera binding/director/cut adapter。两边都不得各建一套timeline或evaluator。

## 4. 五套参考引擎的工程合同差

| 参考 | 本轮直接源码证据 | Zircon应吸收 | 不应照抄 |
|---|---|---|---|
| Unreal Engine | CameraComponent有aspect/overscan/post-process与`NotifyCameraCut`；PlayerCameraManager有current/pending view target、blend、modifier、shake；SpringArm有probe/collision/lag；CineCamera有filmback/lens/focus；GameplayCameras有Rig asset、evaluator、blend stack、shake和collision node；MovieScene有typed Camera Cut Track | per-player owner、versioned rig/lens/shake、immutable evaluator result、typed cut、独立blend/modifier/collision生命周期、Editor asset toolkit | UObject宏、历史compat layer和默认高tick成本 |
| Godot | Camera3D按Viewport current lifecycle，支持perspective/ortho/frustum、offset、environment/attributes/compositor与frustum query；Editor plugin有preview/custom camera/gizmo | per-viewport activation、明确preview切换、frustum/环境合同 | 把简单setter当完整validation上限 |
| Bevy | `Camera`与`ComputedCameraValues`分离，target/viewport/order/active/sub-camera/output明确，world/viewport conversion可失败，自定义projection可扩展；controller crate与核心camera分开 | authored/derived分离、fallible conversion、自定义projection、controller边界 | 用ECS组合本身代替director/ownership/cinematic产品 |
| Fyrox | Camera多数属性Reflect/Visit，projection/viewport/enabled/environment/exposure/color grading/render target/frustum/project/unproject/ray完整；Editor camera settings持久化speed/sensitivity/zoom | reflection/serialization覆盖、normalized viewport、可查询frustum/ray、持久导航profile | 复制builder局部偶然行为或未验证默认值 |
| Unity Graphics | URP序列化Base/Overlay stack、clear depth、volume mask/trigger、AA/XR并由Camera Editor条件展示；HDRP按Camera/XR pass/history channel维护、reset/free/clean/dynamic-resolution | source-to-stack闭环、条件Inspector、history channel/lifecycle/budget、XR view count与显式reset | pipeline-specific MonoBehaviour/static cache owner；本地Graphics不含Cinemachine，本文不推测未收录能力 |

共同原则不是字段数量，而是：source和derived runtime state分离；每个player/view有owner和生命周期；cut/failure是显式事件；Editor只投影同一compiler/evaluator；history与async query有generation、budget和retirement。

## 5. Canonical P0 重判

| Canonical ID | 状态 | 当前证据与必须动作 |
|---|---|---|
| `CAM-ED-P0-001` | Open | Dynamic Runtime仍固定安装Orbit并在未消费raw mouse时直接写scene camera；默认关闭并迁为capability-gated DevCameraMode。physical-first Input修复不关闭camera ownership问题。 |
| `CAM-ED-P0-002` | Open | 14字段仅3字段反射，Rig/Lens/Shake resource/factory/toolkit仍为零；先建立完整source/schema/compiler/product，不接受只解除skip。 |
| `CAM-ED-P0-003` | Open | render stack只存在descriptor/validator/tests，Scene source/Inspector/roundtrip仍不可达；建立source-to-render闭环或明确Unavailable。 |
| `CAM-ED-P0-004` | Open | 无per-World/per-Player/per-View Director；global active camera继续被render、dynamic writer、script和AI混用。裸ID持久化/删除fallback只能作为partial底座。 |
| `CAM-ED-P0-005` | Open | Sequencer Camera Cut仍是静态UI，runtime无typed cut/director handshake，Temporal仍猜cut；建立authoritative CutEvent与history epoch。 |

## 6. Canonical P1 重判

| Canonical ID | 状态 | 当前差距与重构要求 |
|---|---|---|
| `CAM-ED-P1-001` | Partial | 仅FOV/near/far进入generic Inspector；14字段统一typed property、unit、validation、transaction与multi-edit。 |
| `CAM-ED-P1-002` | Open | Scene camera无独立schema/version/unknown policy；补canonical codec、migration和compiler compatibility。 |
| `CAM-ED-P1-003` | Partial | World v2保存裸active ID且delete/undo有fallback；改为generation-qualified stable endpoint reference、redirect和missing diagnostic。 |
| `CAM-ED-P1-004` | Partial | Perspective/Orthographic和runtime matrix override已存在；补frustum/off-center/custom/physical lens、aspect constraint、reverse/infinite-Z及source admission。 |
| `CAM-ED-P1-005` | Open | target/viewport缺finite、bounds、depth、format/MSAA/HDR与resize跨字段validation。 |
| `CAM-ED-P1-006` | Open | Base/Overlay/stack/clear-depth没有source字段、Inspector和roundtrip。 |
| `CAM-ED-P1-007` | Open | culling/volume继续复用单一scene mask；分离typed layer set与migration。 |
| `CAM-ED-P1-008` | Partial | embedded post-process与volume extract真实存在；补profile reference、weight、trigger、override mask和director合成。 |
| `CAM-ED-P1-009` | Open | 无versioned Camera Lens Profile asset、filmback/focal/aperture/focus/crop/distortion metadata。 |
| `CAM-ED-P1-010` | Open | 无Camera Rig asset、stable graph、parameters、target slots、transitions和dependency contract。 |
| `CAM-ED-P1-011` | Open | 无Camera Shake asset、pattern/envelope/channel/space/seed/scaling/stop policy。 |
| `CAM-ED-P1-012` | Open | rig node/pin/parameter/transition/binding没有stable identity。 |
| `CAM-ED-P1-013` | Open | rig对Lens/Shake/Curve/Target/Post/Nested Rig没有typed reference graph。 |
| `CAM-ED-P1-014` | Open | 无Editor/preview/PIE/cook/shipping共享semantic compiler和immutable artifact。 |
| `CAM-ED-P1-015` | Open | 无camera-specific migration/redirect/LKG publication与stale artifact policy。 |
| `CAM-ED-P1-016` | Open | 无per-player/per-viewport director identity、generation、stack和history。 |
| `CAM-ED-P1-017` | Open | activation无priority、owner lease、timeout/revoke和terminal receipt。 |
| `CAM-ED-P1-018` | Open | view target仍是裸entity transform；补socket/bone/offset/velocity/bounds/generation/missing policy。 |
| `CAM-ED-P1-019` | Open | 无Follow/LookAt/Aim typed node、dead zone、prediction和axis constraint。 |
| `CAM-ED-P1-020` | Open | 无Boom/SpringArm、probe/channel/ignore owner/push/recovery/initial-overlap。 |
| `CAM-ED-P1-021` | Open | 无camera collision与target occlusion分层、fade/shoulder/fallback策略。 |
| `CAM-ED-P1-022` | Open | damping/lag无space、half-life/max speed、time domain和teleport reset。 |
| `CAM-ED-P1-023` | Open | 无single/multi-target framing、screen region、dead/soft zone、safe frame和bounded solver。 |
| `CAM-ED-P1-024` | Open | controller不消费Input Action/InputUser/viewport/focus/consume provenance。 |
| `CAM-ED-P1-025` | Open | development navigation与shipping camera混在Dynamic Session；迁为显式DevCameraMode。 |
| `CAM-ED-P1-026` | Open | 无target snapshot -> rig -> constraint -> blend -> modifier -> lens -> publish固定phase/order/budget。 |
| `CAM-ED-P1-027` | Open | 无pose/lens/post-process typed blend、curve、outgoing lock、preblended和cut。 |
| `CAM-ED-P1-028` | Open | blend interruption的rebase/reverse/cut/restore语义与receipt缺失。 |
| `CAM-ED-P1-029` | Open | 无modifier stack、channel/priority/additive/absolute、owner/lifetime/fade。 |
| `CAM-ED-P1-030` | Open | script camera facade仍可raw transform mutation；改为scoped director handle与bounded command/receipt。 |
| `CAM-ED-P1-031` | Open | 无filmback/sensor、focal/FOV authority、preset与unit模型。 |
| `CAM-ED-P1-032` | Open | 无aperture、manual/tracking/autofocus、offset/smoothing/debug plane。 |
| `CAM-ED-P1-033` | Partial | endpoint exposure/post与volume extract已存在；无endpoint/rig/volume/sequence contributor优先级、trace和cut reset policy。 |
| `CAM-ED-P1-034` | Open | 无sensor fit/crop/gate/overscan/safe-area/letterbox统一preview/render/export policy。 |
| `CAM-ED-P1-035` | Open | 无lens distortion/breathing calibration、renderer capability和Unavailable fallback。 |
| `CAM-ED-P1-036` | Open | Shake pattern没有deterministic seed、channel、space或compiled sampling。 |
| `CAM-ED-P1-037` | Open | 无Shake service的instance/owner/tag/scale/fade/pause/time-domain生命周期。 |
| `CAM-ED-P1-038` | Open | 无typed CameraCutEvent、reason、tick与monotonic history epoch。 |
| `CAM-ED-P1-039` | Open | history identity不含player/viewport/view/endpoint/director generation/cut epoch。 |
| `CAM-ED-P1-040` | Open | Sequencer没有stable camera binding、typed cut section/range/blend/easing/diagnostic。 |
| `CAM-ED-P1-041` | Open | Sequence与director没有play/scrub/stop/loop/jump lease和restore-state handshake。 |
| `CAM-ED-P1-042` | Open | 无Shot/Take/timecode/handles/lens refs/safe-frame/source revision metadata。 |
| `CAM-ED-P1-043` | Partial | renderer已有多camera/target/stack底座；仍未区分gameplay director view、capture/probe、overlay和Editor preview purpose/owner/budget。 |
| `CAM-ED-P1-044` | Open | split-screen/stereo/XR的per-eye pose/projection/history/culling与shared rig边界缺失。 |
| `CAM-ED-P1-045` | Open | network/spectator/replay/server relevance/AI observer camera policy缺失。 |
| `CAM-ED-P1-046` | Open | Rig/Lens/Shake无resource kind、factory、catalog、thumbnail、toolkit、cook role。 |
| `CAM-ED-P1-047` | Partial | generic scene/asset transaction、dirty/history基础存在；无Camera Rig/Lens/Shake document session、preview transaction和conflict语义。 |
| `CAM-ED-P1-048` | Partial | generic Inspector可编辑3字段；补完整分组、条件字段、unit、reference picker、validation和multi-edit。 |
| `CAM-ED-P1-049` | Open | 无Rig graph/stack editor、parameter/target/transition/blend/modifier projection与compiler diagnostics。 |
| `CAM-ED-P1-050` | Open | 无Camera Preview tile、View Through或qualified preview generation；transient Editor camera不是替代品。 |
| `CAM-ED-P1-051` | Open | 无Pilot/Lock/capture-loss/Esc/scene-switch状态机和transactional transform写回。 |
| `CAM-ED-P1-052` | Partial | 已有icon、pick sphere和短frustum；补orthographic/filmback/focus/boom/composition/safe-frame、show flag、generation和预算。 |
| `CAM-ED-P1-053` | Open | 无target/input/aspect/device/time/post/collision/platform多上下文Preview Session。 |
| `CAM-ED-P1-054` | Open | 无live Camera Debugger、active stack/owner/node/blend/collision/shake/lens/cut trace。 |
| `CAM-ED-P1-055` | Open | Sequencer camera UX仍是静态row/fixed feedback，未投影runtime cut/director result。 |
| `CAM-ED-P1-056` | Open | Input Action -> camera intent链未闭合；physical-first raw input不等于Action authoring。 |
| `CAM-ED-P1-057` | Open | Play/Game View没有per-instance possession/eject/debug camera和observable Play/Simulate差异。 |
| `CAM-ED-P1-058` | Partial | render descriptor、target generation和局部diagnostics是真实底座；缺source/artifact/view generation与degraded reason的typed handoff receipt。 |
| `CAM-ED-P1-059` | Partial | source roundtrip、controller、ordering/stack/target/history/gizmo与delete/undo有局部tests；director/lease/blend/collision/cut/preview/multi-player/fault matrix仍为空。 |
| `CAM-ED-P1-060` | Open | 无1/10/100 rig、1K node、多view/collision/trace/allocation/latency资格，也未迁移global/raw camera旁路。 |

## 7. Canonical P2 重判

| Canonical ID | 状态 | 目标 |
|---|---|---|
| `CAM-ED-P2-001` | Open | 有预算、warm start、fallback和残差诊断的procedural composition/collision solver。 |
| `CAM-ED-P2-002` | Open | virtual-production lens calibration、ST map、nodal offset、focus/zoom table和timecode。 |
| `CAM-ED-P2-003` | Open | camera animation/motion-matching作为deterministic compiled modifier。 |
| `CAM-ED-P2-004` | Open | 大规模shake source field、distance/occlusion/frequency/accessibility filter与bounded spatial selection。 |
| `CAM-ED-P2-005` | Open | spectator/replay/photo mode独立director policy、permission、time/capture/network合同。 |
| `CAM-ED-P2-006` | Open | artifact/parameter/target/activation/cut/shake的deterministic recording/rollback。 |
| `CAM-ED-P2-007` | Open | large-world/rebase稳定坐标、history与cut policy。 |
| `CAM-ED-P2-008` | Open | stable node/parameter/transition/shot identity驱动semantic merge/review。 |
| `CAM-ED-P2-009` | Open | plugin camera node schema/evaluator、capability/lease/budget/version/unload fallback。 |
| `CAM-ED-P2-010` | Open | camera quality scalability、collision/occlusion/lens/shake/solver tier与traceable fallback。 |
| `CAM-ED-P2-011` | Open | 同scene/rig/path/hardware/quality的跨引擎行为、画面与性能基准。 |
| `CAM-ED-P2-012` | Open | 分布式camera simulation farm、determinism/cut/reset/fault/迁移与性能分位。 |

## 8. 验收门当前状态

| Gate | 状态 | 当前判定 |
|---|---|---|
| `CAM-ED-G01` | Partial | 3/14字段可走generic Inspector；完整条件字段、unit、undo/save/reopen未达标。 |
| `CAM-ED-G02` | Partial | World v2保存裸ID且delete/undo有fallback；stable endpoint/redirect/prefab/cook未达标。 |
| `CAM-ED-G03` | Fail | stack/clear-depth/masks没有source roundtrip。 |
| `CAM-ED-G04` | Partial | descriptor resolver有局部错误报告；source compiler/property-addressed fail-close未达标。 |
| `CAM-ED-G05` | Fail | Rig/Lens/Shake create/save/reopen/reference/cook均不存在。 |
| `CAM-ED-G06` | Fail | Editor/PIE/shipping/cook没有共享camera compiler/evaluator。 |
| `CAM-ED-G07` | Fail | 缺artifact/capability时仍以默认orbit/first camera fallback。 |
| `CAM-ED-G08` | Fail | Dynamic Runtime默认继续消费右/中键/滚轮并写camera。 |
| `CAM-ED-G09` | Fail | 无per-user/director Action route；raw Input与hardcoded writer仍可同触发。 |
| `CAM-ED-G10` | Fail | 两LocalPlayer/viewport独立director stack/view/history不存在。 |
| `CAM-ED-G11` | Fail | activation owner/revoke/target-loss/teardown terminal receipt不存在。 |
| `CAM-ED-G12` | Fail | Follow/LookAt/damping/time-domain golden不存在。 |
| `CAM-ED-G13` | Fail | Boom/collision/overlap/timeout/teleport contract不存在。 |
| `CAM-ED-G14` | Fail | collision与occlusion policy/fallback不存在。 |
| `CAM-ED-G15` | Fail | pose/lens/post blend和interruption matrix不存在。 |
| `CAM-ED-G16` | Fail | modifier/shake owner/channel/lifecycle不存在。 |
| `CAM-ED-G17` | Fail | script仍可raw transform写入，没有scoped director facade。 |
| `CAM-ED-G18` | Fail | AI继续读取global active camera；network/replay view policy缺失。 |
| `CAM-ED-G19` | Fail | filmback/focal/aperture/focus/crop/overscan source与golden不存在。 |
| `CAM-ED-G20` | Fail | typed Camera Cut Track与Preview/PIE parity不存在。 |
| `CAM-ED-G21` | Fail | explicit cut/history epoch和下游reset/hold合同不存在。 |
| `CAM-ED-G22` | Fail | 同位置切camera、小硬切与连续大运动仍不能authoritatively区分。 |
| `CAM-ED-G23` | Partial | history key/viewport generation有局部identity；player/view/director/source/cut隔离未达标。 |
| `CAM-ED-G24` | Fail | Preview/View Through/Pilot/Lock恢复与transactional写回不存在。 |
| `CAM-ED-G25` | Partial | camera icon/pick/短frustum存在；focus/boom/composition/safe-frame/generation/budget缺失。 |
| `CAM-ED-G26` | Fail | Camera Debugger不存在。 |
| `CAM-ED-G27` | Fail | Play/Simulate possession与mutation差异不可观察。 |
| `CAM-ED-G28` | Fail | rig/node/multiview/collision性能资格不存在。 |
| `CAM-ED-G29` | Partial | lower-layer roundtrip/controller/render/delete tests存在；产品compiler/director/preview/fault矩阵未达标。 |
| `CAM-ED-G30` | Fail | 本轮未运行Windows camera compiler/runtime/Editor/Play/history lane，且产品实现缺失。 |
| `CAM-ED-G31` | Fail | 无真实GPU stack/cut/history/motion/exposure capture资格。 |
| `CAM-ED-G32` | Fail | 无activation/target/shake/sequence/resize/multi-player/hot-reload长期soak。 |

## 9. 分层重构顺序

### M0：Truthfulness、边界硬切与输入劫持封闭

冻结global active camera、raw transform和dynamic mouse caller；默认关闭hardcoded writer，定义Camera capability/owner矩阵；把stateful controller行为从`core::framework` hard cut到Runtime owner，不留shim。

### M1：Endpoint Schema、Inspector 与 Render Source闭环

version Camera endpoint source，完整reflection/customization/validation，建立stable default endpoint；补Base/Overlay/stack/clear-depth/masks/projection override与scene roundtrip。

### M2：Rig、Lens、Shake Source 与 Shared Compiler

新增三类versioned asset/factory/toolkit、stable identity/reference graph/migration；产出immutable CameraProgram、dependency digest、diagnostics、LKG和publication receipt。

### M3：Per-player Director 与 View Result

建立World/Player/View identity、activation lease、target snapshot、evaluation phase、typed CameraViewResult、generation/currentness和bounded observation。

### M4：Blend、Modifier、Collision 与 Shake

实现transition/interruption、SpringArm/collision/occlusion、modifier/shake channel、Physics query generation/budget和deterministic tests。

### M5：Cinematic Lens、Cut 与 History

完成filmback/focal/aperture/focus/crop/overscan、typed CameraCutEvent、history epoch/key、temporal/post reset handoff与retirement。

### M6：Transactional Camera Editor

完成endpoint customization、Rig graph、Lens/Shake editors、Preview/View Through/Pilot/Lock、完整gizmo、multi-context preview和Camera Debugger。

### M7：Sequencer、Input、Script、AI 与 Play 集成

接typed Camera Cut Track、Input Action intent、script director facade、AI ObserverViewSet、Play possession/eject/multi-instance，删除产品旁路。

### M8：Fault、Scale、Migration 与竞争资格

完成schema/target/collision/sequence/hot-reload/device-loss fault matrix、1/10/100 rig与1K node/multiview性能门、长时间soak和同语义跨引擎基准。tooling实现由其owner另行承接。

## 10. 禁止的临时修补

1. 禁止只解除11个`zr_reflect(skip)`就宣称Camera Authoring完成。
2. 禁止把Rig/Lens/Shake/Sequence全部堆进`SceneCameraAsset`巨型component。
3. 禁止复制Camera endpoint为第二套顶层asset真值；可复用Rig/Lens/Shake才是独立source。
4. 禁止保留无开关的Runtime right/middle/wheel scene mutation并只改名为debug camera。
5. 禁止让raw Input和hardcoded camera writer长期消费同一未仲裁event。
6. 禁止用global `active_camera: EntityId`模拟LocalPlayer、viewport、spectator、capture或AI observer。
7. 禁止script/ability/sequence每帧直接覆盖camera transform绕过director。
8. 禁止把render stack只留在descriptor tests而没有source/Inspector/roundtrip。
9. 禁止用大位移/旋转/FOV阈值冒充authoritative Camera Cut。
10. 禁止Preview/Pilot直接写scene而不进入transaction/dirty/recovery。
11. 禁止Editor、PIE、shipping与Sequencer各写一套follow/blend/shake/lens evaluator。
12. 禁止以更多静态Camera/Cut rows、fixed Preview反馈或fixture字符串替代真实产品链。

## 11. 本轮产出与验证边界

本轮只修改review与索引文档，没有修改production Runtime/Editor/App/plugin代码或tests。没有运行Cargo、真实Editor/Play、native input、Camera compiler/director、save/reopen、stack source、Sequencer Cut、GPU capture、collision、split-screen/XR、network/replay、fault/scale/soak/profile或同语义跨引擎benchmark；tooling按用户要求排除。

212个Zircon test declarations只证明所选文件存在局部静态测试，不能作为Camera产品通过。实现必须从M0开始，每个里程碑重新冻结current source、sessions/failures、selected fingerprint、dirty owners和动态证据；任何Closed/Pass都必须同时有source-to-product、failure、scale和currentness证据。
