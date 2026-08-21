---
related_code:
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/assets/project_document/scene.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene.rs
  - zircon_runtime/src/scene/components/scene/camera.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/scene/world/compiled_binding
  - zircon_runtime/src/scene/world/property_access
  - zircon_runtime/src/core/framework/camera_controller
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/core/framework/render/view_matrix_pair.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/extract_cache.rs
  - zircon_runtime/src/script/vm/gameplay_host/transform.rs
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity
  - zircon_app/src/bin/zircon_shader_pbr_viewer/camera.rs
  - zircon_app/src/bin/zircon_shader_pbr_viewer/scene.rs
  - zircon_editor/src/scene/viewport/controller
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
tests:
  - zircon_runtime/src/asset/tests/assets/scene/camera.rs
  - zircon_runtime/src/scene/tests/render_extract/camera_order.rs
  - zircon_runtime/src/scene/tests/asset_scene/product_fields.rs
  - zircon_runtime/src/scene/tests/derived_state/runtime_freshness.rs
  - zircon_runtime/src/scene/tests/ecs_identity_storage.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/render_extract.rs
  - zircon_runtime/src/scene/tests/render_extract/direct_sections.rs
  - zircon_runtime/src/tests/camera_controller.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets
  - zircon_runtime/tests/runtime_camera_core_pipeline_contract.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/06-platform-input-process-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/09h2-exposure-color-bloom-dof-motion-blur-ssr-terminal-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_editor/29-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-authoring-review.md
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/45-cinematic-sequencer-shot-track-binding-take-recorder-movie-render-queue-authoring-review.md
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
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 37 · Camera Endpoint、Director、Rig、Controller、Blend、Shake、Cinematic Cut、History、Multi-View、Network、Save、Scalability 与 Product Integration 工程化差距

## 1. 结论

Zircon已经有一条真实但仅到“渲染endpoint”的Camera链。`SceneCameraAsset`能保存pipeline、projection、FOV/ortho、clip、surface/texture/headless target、viewport、order、active、HDR、exposure、clear、MSAA与内嵌post-process；`CameraComponent`进入World，`CameraRenderDescriptor`进入frame extract，camera loop能按target/order提交Base/Overlay序列，viewport record也按camera key保存部分history。这些代码必须保留，不能退回单相机临时渲染。

它仍不是工程级游戏相机系统。Scene没有active camera持久身份，component的14个字段只有FOV/near/far进入可编辑reflection；Base/Overlay、stack、clear-depth、独立culling/volume mask、dynamic resolution和projection override只存在于render DTO，不能从Scene source到达。`ViewportRenderSettings::Perspective`还同时充当默认值与“没有override”的哨兵，使Orthographic scene camera无法通过该设置显式切回Perspective。source入口没有统一finite/range/cross-field admission，非法FOV、far<=near或NaN仍可进入投影矩阵与GPU链。

玩法authority完全缺失。动态Runtime总是创建Orbit controller，在UI未消费时右键、中键、滚轮直接修改`world.active_camera()`；`camera_follow`脚本直接覆盖任意实体transform；AI LOD也读取同一个全局camera。没有per-player/per-viewport director、possession/lease、view target、rig evaluator、collision、blend、modifier、shake、lens profile或camera cut event。当前101个聚焦测试覆盖asset roundtrip、controller数学、target/stack/order和render history局部合同，但没有可执行的director、cinematic、network/save、split-screen/XR或产品输入仲裁链。

历史管理同样不能承担长期产品负载。`ViewportCameraHistoryKey`包含entity/order/type/target/viewport/layers，却不包含World/source/director generation、core pipeline、lens/projection generation或显式history epoch；一个ViewportRecord内至少七个按该key增长的HashMap没有camera retirement/prune。Velocity只能用位移、旋转和投影参数阈值猜测`CameraCutOrInvalid`，无法识别同位置硬切，也可能把快速连续运动误判成cut。

目标链必须是：`CameraEndpointComponent + versioned Lens/Rig/Shake source -> deterministic compiler -> immutable CameraProgram artifacts -> CameraDirectorService(World/Player/View ownership) -> CameraViewResult(pose/lens/projection/post-process/cut/history epoch) -> render/audio/AI/network adapters -> bounded per-view receipts`。Editor30继续拥有5项父P0和P1-01..60跨域要求；本篇不重复顶层阻断，登记 **0个新P0 / 72个Runtime子P1 / 16个P2**。

## 2. 审查边界、方法与 currentness

### 2.1 冻结语料

| 输入 | 文件 / 行 / bytes | 说明 |
|---|---:|---|
| Runtime production与直接consumer | 77 / 17,079 / 641,147 | camera命名owner加Scene、dynamic session、script、AI、catalog、frame extract与post-process入口 |
| 聚焦测试 | 29 / 7,361 / 262,533 | 101个`#[test]`/`#[tokio::test]`，0 ignore |
| 产品/作者控制面 | 9 / 3,161 / 147,796 | PBR viewer、Editor viewport、Sequencer静态面与Editor30父报告 |
| 参考实现 | 17 / 11,171 / 467,226 | Unreal 10、Bevy 2、Godot 2、Fyrox 1、Unity Graphics 2 |
| 合计 | 132 / 38,772 / 1,518,702 | 排序后逐文件SHA-256 manifest的复合SHA-256为`c97f89c409a078c1c515d4f86e9aed73a2a6dc01d68426ff2c409dce22b6d280` |

冻结时相关集合有8条status记录：6份render文件被Git标为`M`但worktree blob与HEAD完全一致；`scene/tests/ecs_identity_storage.rs`有用户在途的mutable-query测试适配，与camera语义无关；Editor30报告为本轮组合中的untracked计划文档。本文不回退、不归因这些改动，并以当前物理文件为证据。

### 2.2 纵向检查链

本轮按source document/cache artifact -> Scene load/save/reflection/property access -> World selection/lifecycle -> controller/script/AI authority -> render descriptor/order/stack/target -> projection/visibility/post-process -> per-camera history -> product input/Sequencer -> network/save/scale逐层检查。关键词零命中不单独证明能力缺失；只有在资源类型、catalog、Scene字段、runtime owner、consumer与测试同时断线时才登记缺口。

### 2.3 动态证据边界

本轮未重复启动已知不可达的`zircon_editor --lib` lane；其最近一次在617秒后以239个既有错误退出。报告属于源码E3静态审查，不把未运行测试写成通过，也不把PBR viewer或Editor自由视口相机当作shipping gameplay camera证据。

## 3. 当前可保留的真实基础

1. Scene Camera asset/component/project conversion与artifact cache已保存基本render endpoint字段和texture direct reference。
2. Camera target支持primary surface、texture与headless，viewport/order/HDR/MSAA/clear也进入descriptor。
3. active/in-hierarchy、render layer过滤、world transform和camera table遍历避免了全entity扫描。
4. Camera ordering按order/target/entity稳定排序并报告相同order/target歧义。
5. Camera stack resolver能识别missing/non-overlay/target mismatch/overlay-with-stack四类违规。
6. Camera loop流式提交多个camera并区分terminal output、UI attachment与planar reflection派生camera。
7. `ViewProjectionMatrixPair`分离jittered/unjittered矩阵，velocity path保存previous camera。
8. Viewport history key区分entity、target、viewport、base/overlay及宽render-layer集合。
9. Free/Orbit/Pan controller已拆分settings、state、input和output，数学单测可继续复用。
10. Texture/headless/primary target的产品测试与visual export形成了render endpoint正向底座。

## 4. 当前代码事实与断路

| 链路 | 当前事实 | 工程缺口 |
|---|---|---|
| Source | `SceneAsset`只有entities，Camera字段无独立schema/version | active selection、Rig/Lens/Shake与migration无权威 |
| Reflection | 14个Camera字段中11个`zr_reflect(skip)` | runtime/editor/script看到的endpoint合同不一致 |
| Selection | `World.active_camera: EntityId`，invalid set静默忽略，load后取首个camera | 不持久、不分player/view、无lease/generation/receipt |
| Fallback | selected camera inactive时返回空scene，不选择其它active camera | “active endpoint”与“director view”语义混淆 |
| Override | 非default projection才覆盖Scene值 | Perspective既是合法值又是absence sentinel |
| Stack | descriptor有type/stack/clear-depth；asset/component没有 | 测试可构造，产品source不可达 |
| Layers | 一个32-bit Scene mask同时复制到culling和volume | 语义合并且无法表达32以上layer的source |
| Validation | source/property write无finite/range/cross-field validator | NaN、无效clip/viewport/target可晚到GPU失败 |
| Controller | dynamic session固定Orbit并直接改Scene transform | development navigation劫持shipping input/authority |
| Script/AI | script直接写transform，AI读global active camera | 无director facade、player/view policy或权限 |
| Lens/Cut | DOF有aperture/focal length，velocity有cut heuristic | 没有物理lens authority或显式cut/history epoch |
| History | 至少7个camera-key HashMap无retirement/prune | key churn导致CPU/GPU state与report长期驻留 |
| Multi-view | 多target/viewport存在，stereo/XR/view family ownership不存在 | split-screen、XR、spectator与capture没有共同ViewId |
| Product | PBR viewer和Editor viewport各自拥有orbit state，Sequencer仅静态UI | 无同一compiler/evaluator/receipt贯通产品 |

## 5. 唯一 Owner、父子 Finding 与目标合同

Editor30继续拥有P0-1..5以及P1-01..60父要求；Editor45拥有Sequencer/shot/take/movie render作者工具父链。Runtime37只拥有可执行子合同，不重复计算这些父finding。O08拥有World camera source/artifact/director/evaluation，O09拥有render endpoint、view family与history消费，O03/O04拥有schema/artifact，O07拥有Input/Player/View身份，O11拥有network/save，O13拥有资源预算，O17拥有产品与竞争资格。

`CameraViewResultV1`至少包含`world_generation/player_id/view_id/director_generation/source_digest/evaluation_tick/pose/lens/projection/viewport/culling_set/volume_set/post_process/cut_kind/history_epoch/quality/disposition`。Render不得读取Editor document或controller内部state；script、AI、audio、network和render只消费同代director snapshot或typed command/result。

## 6. P1：Source、Schema、Compiler 与 Artifact

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| CAM-P1-001 | 没有Camera subsystem capability/package identity | 定义runtime/editor/client/server capability、version、provider与maturity |
| CAM-P1-002 | SceneCamera是无版本内嵌结构 | CameraEndpoint schema有stable field ID、unit、range、unknown policy |
| CAM-P1-003 | active camera identity不在Scene source | 保存stable endpoint reference与缺失/重定向策略 |
| CAM-P1-004 | 没有Lens Profile资源类型 | versioned sensor/focal/aperture/focus/distortion source进入BuildSet |
| CAM-P1-005 | 没有Camera Rig资源类型 | node/edge/parameter使用stable identity与typed reference |
| CAM-P1-006 | 没有Camera Shake资源类型 | pattern、channel、seed、duration、space与attenuation进入source |
| CAM-P1-007 | 字段缺finite/range/cross-field admission | FOV/ortho/clip/viewport/lens/target在compile前fail-close |
| CAM-P1-008 | 没有dependency manifest | endpoint/rig/lens/shake/curve/collision profile/post-process依赖可追踪 |
| CAM-P1-009 | 没有deterministic compiler | 相同source/dependency/toolchain/target得到相同diagnostic与digest |
| CAM-P1-010 | 没有immutable CameraProgram artifact | 固化node plan、parameters、lens tables、blend/shake与adapter声明 |
| CAM-P1-011 | 没有DDC/LKG/publication transaction | compile失败保留同源last-good并报告stale/rollback disposition |
| CAM-P1-012 | 没有migration/downgrade | endpoint/lens/rig/shake独立版本并生成loss/backup/rollback receipt |

## 7. P1：Endpoint、Projection、Render Source 与 Stack

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| CAM-P1-013 | Camera reflection只暴露3/14字段 | endpoint属性通过统一schema呈现并保留权限/只读语义 |
| CAM-P1-014 | Perspective被当作absence sentinel | override使用`Option<ProjectionMode>`或typed override source |
| CAM-P1-015 | projection只有Perspective/Orthographic | 支持off-center/asymmetric/custom/physical lens与明确Unsupported |
| CAM-P1-016 | aspect只由target计算 | 定义sensor/gate fit/crop/letterbox/overscan与resolution cost |
| CAM-P1-017 | near/far无可靠admission | finite、near>0、far>near、reverse/infinite-Z策略进入compile |
| CAM-P1-018 | viewport只晚期clamp | source阶段验证size/depth/order/target bounds并输出disposition |
| CAM-P1-019 | Base/Overlay无法持久化 | endpoint source保存render type、stack stable refs与clear policy |
| CAM-P1-020 | 未引用overlay被序列静默排除 | resolver报告orphan/duplicate/cycle/ownership violation |
| CAM-P1-021 | culling与volume共用单一mask | 独立可扩展RenderLayerSet source并定义legacy 32-bit migration |
| CAM-P1-022 | dynamic resolution不在Scene endpoint | per-view policy由quality owner解析并进入compiled view contract |
| CAM-P1-023 | target readiness与camera activation分离 | target generation/format/size/readiness决定Supported/Degraded/Failed |
| CAM-P1-024 | camera ambiguity只做report | shipping profile按policy fail/resolve并发布选择receipt |

## 8. P1：Director、View Target、Evaluation 与 Ownership

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| CAM-P1-025 | 没有CameraDirectorService | 每World按PlayerId/ViewId建立唯一实例与generation |
| CAM-P1-026 | global active camera承担多种语义 | 区分render default、gameplay view、AI observer、capture endpoint |
| CAM-P1-027 | activation没有owner lease | request携owner/priority/lifetime/revoke token与terminal disposition |
| CAM-P1-028 | 没有possession handoff | player/session/viewport join-leave原子切换director ownership |
| CAM-P1-029 | view target只是裸entity transform | target snapshot含stable identity、socket/bone、generation与missing policy |
| CAM-P1-030 | 没有evaluation phase合同 | Input intent -> target snapshot -> rig -> constraint -> blend -> publish固定顺序 |
| CAM-P1-031 | update读取可变World | tick开始冻结输入，evaluation不跨线程读取Editor/World可变状态 |
| CAM-P1-032 | 没有CameraViewResult | pose/lens/projection/post-process/cut/history在单一commit点发布 |
| CAM-P1-033 | script直接写camera transform | script facade只提交bounded director command并返回receipt |
| CAM-P1-034 | AI读取global camera | AI消费明确ObserverViewSet及relevance/LOD policy |
| CAM-P1-035 | dynamic session固定构造Orbit | development controller由profile/capability显式安装和卸载 |
| CAM-P1-036 | multi-world/session隔离未定义 | service/task/cache绑定world+director generation并拒绝late result |

## 9. P1：Rig、Controller、Collision、Blend、Modifier 与 Shake

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| CAM-P1-037 | 没有typed rig node registry | Follow/LookAt/Aim/Offset/Framing/Boom等节点有schema与cost |
| CAM-P1-038 | controller不消费Input Action | InputUser/MappingContext产生intent，UI consumption与focus可证明 |
| CAM-P1-039 | focus loss不取消drag state | pointer/focus/capture lifecycle保证每次gesture唯一终态 |
| CAM-P1-040 | Orbit viewport字段未形成数学合同 | sensitivity声明pixel/angle/world单位及DPI/viewport缩放语义 |
| CAM-P1-041 | 没有SpringArm/Boom | arm length、socket offset、inheritance与lag进入compiled node |
| CAM-P1-042 | 没有camera collision owner | sync/async query绑定tick/generation/filter/budget与late-result policy |
| CAM-P1-043 | 没有occlusion strategy | push/fade/reframe/teleport选择与geometry/material adapter分离 |
| CAM-P1-044 | damping/lag无clock/space合同 | fixed/variable/cinematic clock、local/world space与reset条件固定 |
| CAM-P1-045 | 没有pose/lens/post-process blend | 通道独立curve、duration、weight、interrupt与completion receipt |
| CAM-P1-046 | 没有modifier stack | priority、exclusive/additive、owner lease、budget和failure isolation |
| CAM-P1-047 | 没有deterministic Shake service | stable seed/stream、pattern state、attenuation、channel与lifecycle |
| CAM-P1-048 | 没有recoil/impulse合并政策 | gameplay impulse经typed channel组合，禁止直接叠写transform |

## 10. P1：Cinematic、History、Multi-View、Network 与 Save

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| CAM-P1-049 | 没有typed Camera Cut事件 | cut携source/destination/view/reason/tick/history epoch |
| CAM-P1-050 | Velocity靠阈值猜cut | explicit cut优先，heuristic只作为invalid fallback并可观察 |
| CAM-P1-051 | history key缺source/director generation | key绑定World/View/Director/Endpoint generation与pipeline identity |
| CAM-P1-052 | history key缺lens/projection epoch | lens、projection、render size、quality变化发布domain disposition |
| CAM-P1-053 | per-camera maps无retirement | active-set sweep、age/bytes budget、lease与GPU fence后回收 |
| CAM-P1-054 | Sequencer没有runtime cut track | typed binding/section evaluator经director ownership handshake |
| CAM-P1-055 | shot/take metadata无runtime身份 | ShotId/TakeId/CameraBindingId进入artifact、capture与receipt |
| CAM-P1-056 | 多camera render与玩家view混淆 | RenderEndpointId和DirectorViewId分离并显式映射 |
| CAM-P1-057 | split-screen没有View family authority | 每player viewport/scissor/history/input/audio listener独立 |
| CAM-P1-058 | stereo/XR contract缺失 | ViewFamily含多个eye/view、late update、foveation与shared culling policy |
| CAM-P1-059 | network/spectator/replay没有camera policy | 复制intent/target/cut/seed或结果按role选择，禁止逐帧盲复制全部float |
| CAM-P1-060 | save/load没有director state | 保存source digest、target、blend/shake/cut/history epoch并迁移 |

## 11. P1：Scalability、Reliability、Diagnostics、Tests 与 Product Qualification

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| CAM-P1-061 | 没有per-view CPU/GPU预算 | rig/collision/history/render target按profile设time/count/bytes上限 |
| CAM-P1-062 | 多camera geometry/extract成本无scale gate | 1/2/4/16/100 views记录extract/cull/submit/cache成本曲线 |
| CAM-P1-063 | malformed source fault matrix缺失 | NaN/cycle/missing ref/invalid target/overflow/OOM均fail-close |
| CAM-P1-064 | async collision/unload fault缺失 | cancel、timeout、late completion和world unload唯一终态 |
| CAM-P1-065 | device loss语义未分层 | render降级不改变authoritative director/sequence state |
| CAM-P1-066 | diagnostics只偏render target | 发布director graph、active owner、blend、constraint、cut/history与budget |
| CAM-P1-067 | 测试无director oracle | golden evaluator覆盖固定tick、thread schedule、reload与platform一致性 |
| CAM-P1-068 | 无network/save/replay parity | live、restore、late join、replay逐tick比较CameraViewResult |
| CAM-P1-069 | 无split-screen/XR/capture矩阵 | view family、target、history、UI、audio listener组合均验证 |
| CAM-P1-070 | 没有shipping input negative test | UI消费、focus loss、debug off时相机transform必须不被劫持 |
| CAM-P1-071 | 产品fixture不贯通source到capture | third-person、first-person、cinematic、split-screen、spectator、photo mode闭环 |
| CAM-P1-072 | 没有同语义竞争性基线 | 相同scene/rig/path/hardware/quality记录raw frame-time、latency、image与memory |

## 12. P2：高阶能力

| Finding | 能力 |
|---|---|
| CAM-P2-001 | 程序化composition/constraint solver与目标优先级优化 |
| CAM-P2-002 | Virtual production lens calibration、distortion/ST map与tracking |
| CAM-P2-003 | 自动focus、depth/subject-aware focus pull与rack focus |
| CAM-P2-004 | lens breathing、anamorphic squeeze、blade/bokeh与chromatic model |
| CAM-P2-005 | camera animation、motion matching与handheld motion library |
| CAM-P2-006 | procedural shake source field与空间传播 |
| CAM-P2-007 | spectator/replay/photo/debug drone的权限化mode stack |
| CAM-P2-008 | deterministic camera recording、rollback与scrub |
| CAM-P2-009 | large-world origin rebase与double-precision director state |
| CAM-P2-010 | portal/mirror/recursive view family与recursion budget |
| CAM-P2-011 | multi-display、off-axis cave/projection wall校准 |
| CAM-P2-012 | mobile/VR comfort、motion sickness与accessibility policies |
| CAM-P2-013 | ML framing/subject selection作为可禁用provider |
| CAM-P2-014 | plugin rig nodes的sandbox、cost declaration与hot reload migration |
| CAM-P2-015 | semantic camera source merge与multi-user live preview |
| CAM-P2-016 | 大规模camera simulation farm与跨backend图像/时序oracle |

## 13. 参考引擎差异矩阵

| 参考 | 可迁移结构 | Zircon当前差异 | 不应照搬 |
|---|---|---|---|
| Unreal Engine | CameraComponent/Perspective-Ortho/overscan，PlayerCameraManager view target/cache/modifier/shake，SpringArm collision，Cine lens，Camera Cut track，Gameplay Cameras rig/evaluator/blend stack | 缺director、rig artifact、lens/shake、collision、cut与ownership全链 | UObject/Actor/Blueprint宏体系与历史兼容层 |
| Bevy | Camera/ComputedCameraValues、target/viewport/order/sub-view、Projection更新和world/viewport fallible conversion | Zircon已有相近endpoint，但source validation、computed generation与conversion error面不足 | ECS组件本身不提供完整gameplay director |
| Godot | Camera3D current/viewport ownership、projection/frustum/ray API、cull mask、environment/attributes与notification lifecycle | Zircon无current-per-viewport、public projection query和environment ownership | 单一SceneTree约定不能替代多World/Player/View身份 |
| Fyrox | Camera节点的Reflect/Visit/UUID、projection/viewport/skybox/exposure等持久化下界 | Zircon reflection只暴露少数字段，source与运行时DTO分裂 | 节点API下界不是director/blend/network完成度 |
| Unity URP/HDRP | URP camera stack/renderer/volume/AA/XR policy；HDRP per-camera view constants、history validity、volume/sky/exposure/XR与clear request | Zircon stack source不可达，history key/retirement/view family和per-camera adapter不足 | MonoBehaviour与RenderPipeline静态全局状态不能成为目标owner模型 |

五套参考共同证明endpoint、computed view、director/evaluator、history和product authoring必须分层；没有一套单独给出Zircon所需的deterministic compiler、跨网络/save和竞争性资格完整答案。

## 14. 分层重构里程碑

### M0 · Truth、Owner 与 Parent Closure

冻结O08/O09边界、Camera IDs、父子finding映射；先关闭dynamic input hijack和假Ready，不新增第二套active-camera单例。

### M1 · Endpoint Schema、Validation 与 Render Source

版本化现有endpoint，补齐reflection、active reference、projection override、stack/layers/target validation和migration；保持当前多target渲染测试。

### M2 · Lens、Rig、Shake Source 与 Compiler

建立资源、dependency manifest、deterministic compiler、immutable artifacts、DDC/LKG与negative fixtures。

### M3 · Director、Ownership 与 Evaluation

实现per-World/Player/View service、lease/possession、target snapshot、固定evaluation phases和immutable CameraViewResult。

### M4 · Rig Nodes、Collision、Blend、Modifier 与 Shake

接Input Action、typed rig nodes、async collision、blend interruption、modifier isolation和deterministic shake。

### M5 · Cinematic、History 与 Persistence

接typed cut/shot/take、history epoch/key/retirement、Sequencer handshake、network/save/replay parity。

### M6 · Multi-View、Reliability 与 Product

关闭split-screen、XR、spectator、capture、fault/device-loss、diagnostics与六类产品fixture。

### M7 · Scale 与 Competitive Qualification

完成view数量曲线、CPU/GPU/RAM/VRAM/latency/image raw receipts；未达到同场景门前不得宣称优于Unreal/Unity。

## 15. 验收门

| Gate | 必须证明 |
|---|---|
| CAM-G01 | Camera capability不从PBR viewer、Editor viewport或静态Sequencer UI推导 |
| CAM-G02 | endpoint/lens/rig/shake source roundtrip与unknown字段无静默丢失 |
| CAM-G03 | 相同source/dependency/toolchain/target重复compile digest一致 |
| CAM-G04 | compile失败保留同源LKG并报告stale/rollback |
| CAM-G05 | invalid FOV/clip/viewport/lens/target在GPU前fail-close |
| CAM-G06 | Perspective和Orthographic override均可显式表达，无sentinel歧义 |
| CAM-G07 | active endpoint stable reference经save/reopen/prefab/cook保持 |
| CAM-G08 | Base/Overlay/stack/clear/layers能从Scene source到提交闭环 |
| CAM-G09 | orphan/duplicate/cycle/mismatch stack均有deterministic disposition |
| CAM-G10 | target generation/format/size失效不会提交stale resource |
| CAM-G11 | 每个World/Player/View只有一个active director generation |
| CAM-G12 | lease revoke、possession切换与unload后旧owner不能继续写view |
| CAM-G13 | evaluation只消费tick-start frozen input/target snapshot |
| CAM-G14 | CameraViewResult在单一commit点发布全部通道 |
| CAM-G15 | script/AI/audio/render不直接读写director内部可变state |
| CAM-G16 | shipping profile关闭development Orbit后输入不修改Scene camera |
| CAM-G17 | UI消费、focus loss、pointer cancel都终止gesture且不泄漏capture |
| CAM-G18 | InputUser/MappingContext/viewport路由在split-screen下不串线 |
| CAM-G19 | rig node order、parameter和tie-break通过deterministic oracle |
| CAM-G20 | async collision late result按world/director/tick generation拒绝 |
| CAM-G21 | blend pose/lens/post-process通道及interrupt/complete语义固定 |
| CAM-G22 | Shake seed/stream在重启、线程调度、平台与late join一致 |
| CAM-G23 | explicit CameraCut更新history epoch并传播所有temporal consumer |
| CAM-G24 | 同位置硬切被识别，连续高速运动不被错误当作authoritative cut |
| CAM-G25 | camera history key包含view/source/director/pipeline代次 |
| CAM-G26 | retired camera的七类history/runtime/report map受age/bytes/fence回收 |
| CAM-G27 | Scene reload/entity reuse不能继承旧camera history |
| CAM-G28 | Sequencer cut只经director handshake取得和释放ownership |
| CAM-G29 | Shot/Take/Binding identity贯通artifact、frame和capture receipt |
| CAM-G30 | split-screen每view的viewport/history/input/UI/audio listener隔离 |
| CAM-G31 | XR多eye共享/独立数据、late update、foveation和history策略明确 |
| CAM-G32 | network/save/replay恢复target/blend/shake/cut且结果可复现 |
| CAM-G33 | dedicated/headless不创建无意义render state但保留所需authority |
| CAM-G34 | device loss只降级render adapter，不改变director state |
| CAM-G35 | 所有queue/map/task有count/bytes/age/time budget与drop receipt |
| CAM-G36 | malformed/OOM/timeout/unload/skew fault matrix通过 |
| CAM-G37 | 六类产品fixture通过create/save/play/export/capture与state inspection |
| CAM-G38 | 1/2/4/16/100 views有CPU/GPU/RAM/VRAM/stutter raw receipts |
| CAM-G39 | source/build/device/driver变化使旧accepted receipt自动过期 |
| CAM-G40 | 优于Unreal/Unity结论绑定同scene/rig/path/hardware/quality/raw evidence |

## 16. Finding 到里程碑与父项映射

| Runtime finding | 里程碑 | 父owner |
|---|---|---|
| CAM-P1-001..012 | M0-M2 | Editor30 P1-02、09..15，O03/O04/O08 |
| CAM-P1-013..024 | M1 | Editor30 P0-2/3、P1-01..08，O08/O09 |
| CAM-P1-025..036 | M0/M3 | Editor30 P0-1/4、P1-16..30，O07/O08 |
| CAM-P1-037..048 | M3-M4 | Editor30 P1-19..37、56，O07/O08 |
| CAM-P1-049..060 | M5-M6 | Editor30 P0-5、P1-38..45、Editor45，O08/O09/O11 |
| CAM-P1-061..072 | M6-M7 | Editor30 P1-58..60，O13/O17 |
| CAM-P2-001..016 | M7后独立立项 | 不阻塞MVP，不得反向替代P1 |

## 17. 禁止的临时修补

- 不给`active_camera`再加一个全局布尔/字符串mode冒充director。
- 不以Scene transform lerp冒充blend、以random offset冒充Shake、以raycast helper冒充collision node。
- 不把DOF的aperture/focal length字段改名后冒充physical lens profile。
- 不继续用Perspective默认值表达“没有override”。
- 不把camera stack测试中的手工descriptor当作可作者化source证据。
- 不用velocity阈值猜测替代显式CameraCut/history epoch。
- 不让script、AI、Sequencer或Editor直接绕过director写endpoint transform。
- 不为每个新camera key永久增长HashMap或GPU history。
- 不以PBR viewer截图、空headless submit或单camera 60 FPS作为产品资格。

## 18. 实施文件与职责蓝图

| 目标owner | 职责 |
|---|---|
| Camera source/schema模块 | Endpoint/Lens/Rig/Shake documents、stable IDs、validation、migration |
| Camera compiler/artifact模块 | dependency、compiled node plan、lens/shake tables、digest、LKG |
| World CameraDirector service | Player/View instances、lease、target snapshot、evaluation、publish、save/network |
| Controller/rig provider层 | Input intent、nodes、collision、blend、modifier、shake，禁止直接render提交 |
| Render camera adapter | CameraViewResult到descriptor/view family/cut/history disposition |
| Viewport history registry | generation key、budget、retirement、fence-safe回收与diagnostics |
| Editor30/45 consumer | 围绕同一schema/compiler/evaluator做authoring、preview、Sequencer，不复制runtime |
| Product qualification | 六类fixture、fault/parity/scale、raw capture和竞争性receipt |

最终crate与目录位置必须由M0 ADR依据现有Runtime模块边界确定；本报告固定职责与依赖方向，不提前用新目录制造第二套authority。

## 19. 本轮产出边界

本轮只完成Runtime Camera执行链的差异、owner、72个子P1、16个P2、40个gate和父子关闭关系登记；`implementation_status`仍为`pending`。实施必须先修正truth/source/validation与input hijack，再建立director/artifact，随后接rig/history/cinematic/multi-view，最后做规模和竞争性资格；不能从再加一个Orbit mode或Shake helper开始。
