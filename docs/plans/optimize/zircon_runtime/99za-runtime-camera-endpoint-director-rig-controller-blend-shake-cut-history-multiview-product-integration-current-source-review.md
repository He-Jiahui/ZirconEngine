---
title: Runtime Camera Endpoint、Director、Rig、Controller、Blend、Shake、Cut、History、Multi-View 与 Product Integration Current Source Review
category: zircon_runtime
report_id: Runtime126
review_date: 2026-08-23
baseline_head: f79dc502a1e8db5f7cbcc17fbeb297af1e193f7e
observed_head: f79dc502a1e8db5f7cbcc17fbeb297af1e193f7e
baseline_epoch: 379
supersedes:
  - docs/plans/optimize/zircon_runtime/37-camera-endpoint-director-rig-controller-blend-shake-cinematic-cut-history-multiview-network-save-scalability-product-integration-review.md
related_code:
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/assets/project_document/scene.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene.rs
  - zircon_runtime/src/scene/components/scene/camera.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/core/framework/camera_controller
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/core/framework/render/view_matrix_pair.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_runtime/src/dynamic_api/session
  - zircon_runtime/src/script/vm/gameplay_host/transform.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
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
  - docs/plans/optimize/zircon_runtime/37-camera-endpoint-director-rig-controller-blend-shake-cinematic-cut-history-multiview-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/65-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-review.md
  - docs/plans/performance/01/failure-2026-08-23-editor-delete-subtree-all-cameras-invariant.md
  - docs/plans/zircon_runtime/render/06/failure-2026-08-15-camera-resolution-scale-symbol-drift.md
  - docs/plans/zircon_runtime/render/07/failure-2026-08-08-camera-table-render-extract-stale-map.md
  - docs/plans/zircon_runtime/runtime/07/failure-2026-07-23-submit-context-camera-target-sharing-anchor-drift.md
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
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
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99za · Runtime Camera Current Source Review

## 1. 结论

Runtime37 对系统级缺口的主要裁决在当前源码中仍成立，但旧报告关于“active camera 不持久化”的描述已经过时。当前 `WorldProjectDocumentV2`、`WorldPersistentState` 与 canonical project JSON 已保存 `active_camera`，World 创建、删除与回退也已有基础策略；graphics 侧已经形成真实的多 camera render endpoint、Base/Overlay stack、target readiness 报告、viewport generation guard、view-family resolution/history 数学和多类 temporal/product history。这些实现是应保留并收敛的底座，不应重写成单 camera 临时路径。

然而它们仍没有组成工程级 Camera 产品。持久化的 `active_camera` 只是裸 `EntityId = u64`，没有 World/source generation、stable endpoint reference、缺失重定向、prefab/cook 或跨版本合同；`set_active_camera` 对非法实体静默无效，删除包含全部 camera 的子树仍有已登记的“active 归零且无 camera”父阻断。Scene source 仍无法表达 Base/Overlay、stack、clear depth、独立 culling/volume mask、dynamic resolution 和 projection override，render DTO 的能力不能从作者源到达。`Perspective` 仍兼任默认值与 absence sentinel，source 入口也没有统一 finite/range/cross-field admission。

玩法 authority 仍为 **0**。没有 per-World/per-Player/per-View `CameraDirectorService`、possession/owner lease、view-target snapshot、rig program、collision、blend、modifier、shake、显式 cut 或统一 `CameraViewResult`。dynamic session 每次固定构造 Orbit controller，UI 未消费时 raw 右键/中键/滚轮直接改 `world.active_camera()`；focus loss 不结束 drag，Orbit 保存但不使用 viewport size，wheel 最终只保留 `signum`。脚本继续直接覆盖实体 transform，AI LOD 继续读取全局 active camera。更严重的是，这些有状态 controller 行为位于 `core::framework::camera_controller`，违反“framework 只保留中立合同、runtime 拥有业务生命周期”的固定架构。

历史管理虽比旧报告更完整，却仍是长期负载风险。`ViewportCameraHistoryKey` 只编码 entity/order/type/target/viewport/culling/volume，没有 World/Player/View/Director/source generation、pipeline/lens/projection/cut epoch；单个 `ViewportRecord` 内至少七类按 camera key 的 map 只在整个 viewport 销毁时整体释放，没有 active-set retirement、age/bytes budget 或 fence-safe逐项回收。pipeline、render size 和 dynamic-resolution 的局部失效是真进展，但不能替代显式 cut/history epoch。Velocity 仍以阈值猜 cut，无法可靠识别同位置硬切，也会把合法高速运动误判为 cut。

当前重判为 **0 项本地新增 P0；65 P1 Open、7 P1 Partial、0 Closed；16 P2 Open；35 Gate Fail、5 Gate Partial、0 Pass**。本轮新增 finding 为 0，只对 Runtime37 的 72 项 P1、16 项 P2 与 40 项资格门按当前源码重判。删除子树可清空全部 camera 的 correctness blocker 继续由既有 performance/Editor/World owner 关闭，本报告不复制 P0 计数。

目标链保持：`versioned CameraEndpoint/Lens/Rig/Shake source -> deterministic compiler -> immutable CameraProgram -> CameraDirectorService(World/Player/View ownership) -> CameraViewResult -> render/audio/AI/network/save adapters -> bounded receipts`。本轮只做静态 review 和文档记录，没有修改 production、tests、Cargo、ABI 或参考源码；没有运行真实 Editor、GUI/GPU、native input、save/reopen、network/replay、fault/soak/profile 或同语义跨引擎 benchmark。MVP 未完成，`source_recheck_required` 保持 true。

## 2. 审查边界与物理冻结

### 2.1 Focused 集合

| 范围 | 文件 / 行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Camera source/runtime | 43 / 10,261 / 370,343 / 36 / 0 | `c004c8ad4b6424290947faefd38bff2d3c3df4a55c079b7f41d97bb995931222` |
| Render history/adapters | 44 / 6,356 / 217,177 / 65 / 0 | `12826d9ceee2471da0cb16422219c7a273673c12aa27e108683e2c1ce3782e03` |
| Product consumers | 5 / 1,904 / 77,001 / 18 / 0 | `de42249cf9e3951fac5fbadc78efe726bcd7515f70cd6043e52b439d2455ed14` |
| Focused tests | 22 / 5,166 / 190,383 / 70 / 0 | `8c9fd59c694ec827cda496df9114a59452d1d89859ca28bf31c165fd9b3815bc` |
| Zircon focused total | 114 / 23,687 / 854,904 / 189 / 0 | `837e27a198943e1eb25c7675cf6d579028e0c0e2b3cccefd59c0c245b2a91aa4` |
| Selected five-engine evidence | 17 / 11,171 / 467,226 / 4 Rust test declarations | `7ed4002331b21b37eed2439e942350b33a1e59d749f56fe73ef6d953abfd5fe5` |

fingerprint 算法与本系列 current-source review 一致：仓库相对路径转 `/`、小写、ordinal 排序去重；每项编码为 `lowercase-path + NUL + lowercase per-file SHA-256`，以 LF 连接且末尾无 LF，再对 UTF-8 payload 计算 SHA-256。它只冻结本轮实际读取集合，不是 Camera runtime identity、artifact digest 或 release identity。

### 2.2 Currentness 与共享工作树

- baseline 与 observed HEAD 都是 `f79dc502a1e8db5f7cbcc17fbeb297af1e193f7e`，registration baseline epoch 为 379。
- 工作树存在大量其他 session 与用户在途改动；本轮只租用本报告、Runtime 索引、根索引和覆盖台账，不回退、不归因任何其他文件。
- Runtime37 的 132 文件旧冻结不能代表当前链。当前 focused set 按 source/runtime、render history/adapters、product consumers 与 tests 重新去重，共 114 文件、23,687 行、854,904 bytes、189 项 test declaration。
- 参考文件仍为 Unreal 10、Bevy 2、Godot 2、Fyrox 1、Unity Graphics 2，但按当前物理内容重算了 fingerprint；参考源码不是绝对正确模板，只有显式 owner、生命周期、失败和规模合同可作为差异证据。

### 2.3 纵向检查链

本轮沿 `SceneCameraAsset -> project document/artifact -> CameraComponent/reflection -> World load/save/delete/select -> render extract -> descriptor/order/stack/target/view family -> camera loop/visibility/temporal history -> dynamic controller/script/AI -> tests/product` 逐层检查。零关键词命中不单独证明缺失；只有 source、owner、consumer、lifecycle 与测试同时断线时才保留缺口。

## 3. 当前实现事实

### 3.1 Source、World 与持久化

1. `SceneCameraAsset` 和 `CameraComponent` 保存 core pipeline、Perspective/Orthographic、FOV/ortho size、near/far、surface/texture/headless target、viewport、order、active、HDR、exposure、clear、MSAA 与 optional post-process。它们是真实 render endpoint source，不是占位结构。
2. Camera 没有独立 schema version、field identity、migration、unknown-field policy、validation report 或 dependency manifest。component 的 14 个字段只有 FOV/near/far 进入 reflection，其余 11 个跳过。
3. project conversion 是直接字段映射，没有 finite/range/cross-field admission。`Perspective` sentinel、非法 clip/FOV/viewport 和 layer loss 可继续进入后续链。
4. World project format 已是 v2，并为缺失 camera/light 做 normalization；`WorldPersistentState` 与 project JSON 已保存 `active_camera`。这是 Runtime37 后的实质进展。
5. `active_camera` 仍是裸实体 ID。`set_active_camera` 返回 `()` 并静默忽略无效值；delete 后以稳定 first camera 回退，但删除包含全部 camera 的 subtree 可以留下零 camera/active 0，已由既有 failure handoff 登记。
6. `build_viewport_render_packet` 仍 clone 完整 World，再对 clone 做 render extract。camera selection 依次考虑 request override、active、first 和 fallback descriptor；选择到 inactive camera 时产生空 geometry，而不是切到另一个 active endpoint。

### 3.2 Render endpoint、stack、target 与 view family

1. `ViewportCameraSnapshot` 已包含 projection override、dynamic resolution 和 jitter；Scene camera source 不包含它们，所以这些能力只能由手工 DTO 或上层临时装配触达。
2. render camera DTO 支持 Base/Overlay、clear depth、独立 culling/volume mask、stack 和 target。Scene source只有单一 32-bit layer mask，且同时复制到 culling/volume；没有可扩展 layer set 或迁移合同。
3. stack resolver 能报告 missing overlay、non-overlay reference、target mismatch 与 overlay-has-stack，但未覆盖 orphan、duplicate、cycle、duplicate ownership；违规时仍可能产生可提交序列，缺 shipping fail policy。
4. camera ordering 是稳定的，ambiguity 只报告而不按 profile fail/resolve。viewport clamp 也没有完整 depth/range admission。
5. target registry 已有 pending/direct import/conversion/blocked format 等状态、三代 target product ring、viewport generation guard 与显式 viewport destroy。这些是 target lifecycle 的可保留底座。
6. camera loop 可以提交多 camera、不同 target/UI/stack policy 和 planar reflection；visibility 也能生成 main/custom target/shadow 额外视图。它证明了多 render endpoint，不证明 per-player director、split-screen ownership 或 XR view authority。
7. view family 已有 resolution policy/controller、temporal key 和 phase，但 scope 仍主要是 raw `view_family_id + viewport_generation + upscaler`，没有 World/Player/View/Director generation 或 explicit cut disposition。

### 3.3 Controller、Script 与 AI authority

1. Free/Orbit/Pan controller 有可执行数学与 focused tests，但没有 Input Action、InputUser、MappingContext、owner lease、dt/clock、collision、rig node 或 lifecycle service。
2. dynamic session 每次构造固定 1280x720 Orbit controller。UI 有先消费机会；未消费的 raw right/middle/wheel 直接修改当前 World active camera transform。
3. focus lost 不取消 drag state。Orbit 的 `viewport_size` 虽被 clamp，却不参与旋转/平移数学；wheel delta 最终取 `signum()`，丢失幅度和设备语义。
4. Pan zoom 修改 `Transform.scale`，把 view intent 混入 scene transform。script `camera_follow` 也直接覆盖任意实体 transform，没有 bounded command、ownership 或 receipt。
5. AI runtime 每 tick 读取 `world.world_transform(world.active_camera())` 做 LOD/relevance，继续把 render default、玩家视图与 AI observer 混成一个全局相机。
6. 有状态 controller 行为放在 `core::framework::camera_controller`。M0 必须先裁定 hard cut：framework 只保留中立 input/pose/rig 合同，具体 controller、director、ownership 与 lifecycle 进入 `core::runtime` owner；禁止用 re-export/shim 保留第二套 authority。

### 3.4 Temporal history 与资源回收

1. `ViewportCameraHistoryKey` 包含 entity/order/type/target/viewport/culling/volume，但不包含 World、Player、View、Director、source generation、pipeline、lens/projection 或 cut epoch。
2. 单个 `ViewportRecord` 至少有 `hybrid_gi_runtimes`、`virtual_geometry_runtimes`、`light_grid_reports`、`virtual_geometry_debug_snapshots`、`camera_histories`、`motion_vector_cameras`、`particle_previous_sprites` 七类 per-camera map。
3. `destroy_viewport` 会整体释放这些 map 和 history；target product 也有 generation ring。这不等于 camera retirement：同一 live viewport 内切换、删除、重建或不断创建 endpoint 时，旧 key 没有 active-set sweep、age/bytes budget 或 fence-safe逐项回收。
4. pipeline、render size 和 dynamic-resolution 的局部 invalidation 已存在，但不属于 source/director/cut 统一 currentness receipt；同位置硬切仍可能继承旧 temporal state。
5. velocity 会拒绝 NaN、非法或过大 delta，并以位移/旋转/投影阈值推断 `CameraCutOrInvalid`。这只能作为损坏回退，不能承担 authoritative cut。

### 3.5 测试与产品证据

focused tests 真实覆盖 controller 数学、camera source roundtrip/core pipeline、camera ordering、stack/target、per-camera history/map、camera loop 与 velocity。当前没有 director oracle、possession/lease、network/save/replay parity、split-screen input/audio/UI isolation、XR、shipping input negative、camera retirement soak 或六类产品 source-to-capture fixture。PBR viewer、Editor自由视口和手工 render descriptor 不得作为 shipping gameplay camera 完整性的替代证据。

## 4. 五套参考引擎的可迁移工程合同

| 参考 | 本轮实际证据 | Zircon 应吸收的合同 | 不应照抄的部分 |
|---|---|---|---|
| Unreal Engine | `CameraComponent` 有 aspect/overscan/HMD/post-process/cut；`PlayerCameraManager` per-player 持有 view target/cache/blend/modifier/shake/network cut；GameplayCameras 有 rig build status、evaluator result、blend entry/layer/freeze/pop、shake 与 sync/async collision；CineCamera 有 filmback/lens/focus；MovieScene cut track 是 typed binding | per-player owner、immutable/evaluable rig、typed cut、独立 blend/modifier/shake 生命周期、collision generation 与 cinematic identity | 不复制 UObject/reflection 宏、历史兼容层或默认 tick 成本 |
| Bevy | authored `Camera` 与 `ComputedCameraValues` 分离；viewport/world conversion fallible；target/viewport/order/active/sub-camera/output mode/custom projection/reverse-Z 明确 | source/derived 分离、fallible admission、custom projection、明确 target/output contract | 不把 ECS component 组合本身当成 director/ownership 完整方案 |
| Godot | Camera 按 Viewport 进入/退出并收到 became/lost 通知；支持 frustum projection/query、environment/attributes/compositor | per-viewport activation lifecycle、frustum query、环境/合成器关联 | Godot near/far setter 也并非完整验证，不能用作 Zircon admission 上限 |
| Fyrox | camera 多数字段 Reflect+Visit、UUID 类型、normalized viewport、多个 camera/render target、projection/environment/exposure/color grading；render target 有意不持久化 | 反射/序列化覆盖、稳定类型、normalized viewport、可声明 non-persistent runtime handle | builder 中也存在局部可疑/未用字段，不能当无缺陷模板 |
| Unity Graphics | URP 序列化 Base/Overlay stack、clear depth、独立 volume mask/trigger、AA/XR/history；HDRP history key 按 Camera/XR pass/channel，限制 user channels，支持 free/reset/view-count realloc/CleanUnused、动态分辨率与 cut reset | source-to-stack闭环、显式 history channel/lifecycle/budget、XR view count、camera cut reset | 不复制 pipeline-specific MonoBehaviour ownership或静态全局 cache |

参考结论不是“字段越多越工程化”，而是四个共同原则：作者 source 与 runtime derived state 分离；每个 view 有明确 owner/lifecycle；失败与 cut 是显式事件；history 与异步工作由 generation、budget 和 retirement 管理。

## 5. 唯一 Owner、父子关系与目标合同

Editor30 继续拥有 Camera 作者工具、preview、asset/component/rig UI 与其既有父 P0/P1；Editor45 拥有 Sequencer/shot/take/movie render 作者链。Runtime24 拥有通用 stable identity/generation，Runtime65 拥有 quality/dynamic-resolution 父合同，Runtime66 拥有 XR 平台父合同，Runtime56 拥有 InputUser/Action/MappingContext。Runtime126 只拥有 Camera 可执行 source/compiler/director/evaluation/adapter/history 子合同，不重复累计父 finding。

`CameraViewResultV1` 至少应包含 `world_generation/player_id/view_id/director_generation/source_digest/evaluation_tick/pose/lens/projection/viewport/culling_set/volume_set/post_process/cut_kind/history_epoch/quality/disposition`。Render、AI、audio、network、save 与 script facade 只能消费同代 immutable snapshot 或提交 typed command；不得读取 Editor document、controller 内部 state 或裸全局 active-camera transform。

## 6. P1 Current-Source 重判

### 6.1 Source、Schema、Compiler 与 Artifact

| Finding | 状态 | 当前源码裁决与目标合同 |
|---|---|---|
| CAM-P1-001 | Open | 无 Camera subsystem capability/package identity；定义 runtime/editor/client/server capability、version、provider 与 maturity。 |
| CAM-P1-002 | Open | SceneCamera 仍是无独立版本内嵌结构；建立 stable field ID、unit、range 与 unknown policy。 |
| CAM-P1-003 | Partial | World v2 已保存裸 `active_camera`，但不是 stable endpoint reference；补 World/source generation、missing/redirect、prefab/cook 合同。 |
| CAM-P1-004 | Open | 无 Lens Profile 资源；sensor/focal/aperture/focus/distortion 进入 versioned source。 |
| CAM-P1-005 | Open | 无 Camera Rig 资源；node/edge/parameter 使用 stable identity 与 typed reference。 |
| CAM-P1-006 | Open | 无 Camera Shake 资源；pattern/channel/seed/duration/space/attenuation 进入 source。 |
| CAM-P1-007 | Open | source 缺 finite/range/cross-field admission；compile 前 fail-close。 |
| CAM-P1-008 | Open | 无 endpoint/rig/lens/shake/curve/collision/post-process dependency manifest。 |
| CAM-P1-009 | Open | 无 deterministic semantic compiler 与 diagnostic digest。 |
| CAM-P1-010 | Open | 无 immutable `CameraProgram` artifact。 |
| CAM-P1-011 | Open | 无 DDC/LKG/publication transaction；compile 失败不能保留并证明同源 last-good。 |
| CAM-P1-012 | Open | 无 endpoint/lens/rig/shake migration/downgrade loss/rollback receipt。 |

### 6.2 Endpoint、Projection、Render Source 与 Stack

| Finding | 状态 | 当前源码裁决与目标合同 |
|---|---|---|
| CAM-P1-013 | Open | Camera reflection 仍只暴露 3/14 字段；统一 schema 呈现全部作者属性与权限。 |
| CAM-P1-014 | Open | `Perspective` 仍是 absence sentinel；改为 typed optional override。 |
| CAM-P1-015 | Partial | render snapshot 可携 arbitrary projection matrix override，但 source/API 仍只有 Perspective/Orthographic；补 off-center/asymmetric/custom/physical typed policy。 |
| CAM-P1-016 | Open | aspect 仍主要由 target 计算；补 sensor/gate fit/crop/letterbox/overscan 与成本。 |
| CAM-P1-017 | Open | near/far 缺统一 admission；定义 reverse/infinite-Z 与 finite/cross-field 规则。 |
| CAM-P1-018 | Open | viewport 仍主要晚期 clamp；source 阶段验证 size/depth/order/target bounds。 |
| CAM-P1-019 | Open | Base/Overlay/stack/clear depth 无法从 Scene source 持久化到提交。 |
| CAM-P1-020 | Open | resolver 未覆盖 orphan/duplicate/cycle/duplicate ownership，违规仍可产生序列。 |
| CAM-P1-021 | Open | source 单一 32-bit mask 同时用于 culling/volume；补独立可扩展 set 与 migration。 |
| CAM-P1-022 | Open | dynamic-resolution policy 不在 endpoint source/director effective quality。 |
| CAM-P1-023 | Partial | target 状态、format conversion、generation ring 与 viewport guard 已存在；camera activation 尚未与 target generation/readiness 原子绑定。 |
| CAM-P1-024 | Open | camera ambiguity 仍只 report；shipping profile 缺 deterministic fail/resolve receipt。 |

### 6.3 Director、View Target、Evaluation 与 Ownership

| Finding | 状态 | 当前源码裁决与目标合同 |
|---|---|---|
| CAM-P1-025 | Open | 无 per-World/per-Player/per-View `CameraDirectorService`。 |
| CAM-P1-026 | Open | global active camera 继续混合 render default、gameplay view、AI observer 与 capture。 |
| CAM-P1-027 | Open | activation 无 owner/priority/lifetime/revoke lease 与 terminal disposition。 |
| CAM-P1-028 | Open | 无 possession、join/leave 与 viewport handoff transaction。 |
| CAM-P1-029 | Open | view target 仍是裸 entity transform；补 socket/bone/generation/missing policy snapshot。 |
| CAM-P1-030 | Open | 无固定 Input intent -> target -> rig -> constraint -> blend -> publish phase。 |
| CAM-P1-031 | Open | evaluation 未基于 tick-start frozen input/target snapshot。 |
| CAM-P1-032 | Open | 无统一原子发布的 `CameraViewResult`。 |
| CAM-P1-033 | Open | script 仍可直接写 transform；改为 bounded director command + receipt。 |
| CAM-P1-034 | Open | AI 仍读取 global active camera；改为 qualified `ObserverViewSet`。 |
| CAM-P1-035 | Open | dynamic session 固定安装 Orbit；改为 profile/capability 显式 owner lifecycle。 |
| CAM-P1-036 | Open | world/director generation 与 late-result rejection 未定义。 |

### 6.4 Rig、Controller、Collision、Blend、Modifier 与 Shake

| Finding | 状态 | 当前源码裁决与目标合同 |
|---|---|---|
| CAM-P1-037 | Open | 无 typed rig node registry、schema、cost 或 evaluator。 |
| CAM-P1-038 | Open | controller 不消费 Input Action/InputUser/MappingContext。 |
| CAM-P1-039 | Open | focus loss 不取消 drag；pointer/focus/capture lifecycle 无唯一终态。 |
| CAM-P1-040 | Open | Orbit viewport size 未参与数学，wheel 丢失幅度；固定 pixel/angle/world/DPI 合同。 |
| CAM-P1-041 | Open | 无 SpringArm/Boom node。 |
| CAM-P1-042 | Open | 无 camera collision owner、generation、budget 与 late-result policy。 |
| CAM-P1-043 | Open | 无 push/fade/reframe/teleport occlusion strategy。 |
| CAM-P1-044 | Open | damping/lag 无 clock、space、reset 合同。 |
| CAM-P1-045 | Open | 无 pose/lens/post-process 独立通道 blend、interrupt/completion receipt。 |
| CAM-P1-046 | Open | 无 modifier stack 的 priority/exclusive/additive/lease/budget/failure isolation。 |
| CAM-P1-047 | Open | 无 deterministic shake seed/stream/pattern/attenuation lifecycle。 |
| CAM-P1-048 | Open | 无 recoil/impulse typed channel 与合并政策。 |

### 6.5 Cut、History、Multi-View、Network 与 Save

| Finding | 状态 | 当前源码裁决与目标合同 |
|---|---|---|
| CAM-P1-049 | Open | 无 typed CameraCut event、reason、tick 与 history epoch。 |
| CAM-P1-050 | Open | Velocity 仍靠阈值猜 cut；explicit cut 必须 authoritative。 |
| CAM-P1-051 | Open | history key 缺 World/View/Director/Endpoint source generation 与 pipeline identity。 |
| CAM-P1-052 | Partial | pipeline、render size、dynamic-resolution 已有分散 invalidation；仍缺统一 lens/projection/cut epoch 与 currentness receipt。 |
| CAM-P1-053 | Open | 七类 per-camera map 无 active-set retirement、age/bytes/fence 回收。 |
| CAM-P1-054 | Open | Sequencer 没有 runtime typed cut-track evaluator/director handshake。 |
| CAM-P1-055 | Open | ShotId/TakeId/CameraBindingId 未进入 artifact、frame 与 capture receipt。 |
| CAM-P1-056 | Open | 多 camera render endpoint 与 gameplay DirectorView 未建立显式映射。 |
| CAM-P1-057 | Partial | view-family、resolution/history math 与多 target views 已存在；仍无 per-player authority、input/UI/audio listener 隔离。 |
| CAM-P1-058 | Open | 无 XR eye/view、late update、foveation 与 shared-culling camera contract。 |
| CAM-P1-059 | Open | 无 network/spectator/replay camera policy 与 deterministic role contract。 |
| CAM-P1-060 | Open | save/load 不含 director target/blend/shake/cut/history state。 |

### 6.6 Scalability、Reliability、Diagnostics 与 Product Qualification

| Finding | 状态 | 当前源码裁决与目标合同 |
|---|---|---|
| CAM-P1-061 | Open | 无 per-view rig/collision/history/target CPU/GPU/count/bytes budget。 |
| CAM-P1-062 | Open | 无 1/2/4/16/100 views 的 extract/cull/submit/cache scale gate。 |
| CAM-P1-063 | Open | NaN/cycle/missing ref/invalid target/overflow/OOM fault matrix 缺失。 |
| CAM-P1-064 | Open | async collision/cancel/timeout/unload/late completion 唯一终态缺失。 |
| CAM-P1-065 | Open | device loss 与 authoritative director/sequence state 未分层。 |
| CAM-P1-066 | Partial | 已有 target/history/product diagnostics；仍缺 director owner、blend、constraint、cut 与 budget 诊断。 |
| CAM-P1-067 | Open | 无 deterministic director/evaluator golden oracle。 |
| CAM-P1-068 | Open | 无 network/save/replay `CameraViewResult` parity。 |
| CAM-P1-069 | Open | 无 split-screen/XR/capture view/input/UI/audio/history 矩阵。 |
| CAM-P1-070 | Open | 无 shipping profile 下 UI consume/focus/debug-off 输入不劫持相机的负测试。 |
| CAM-P1-071 | Partial | 已有 target/stack/render product fixtures；未贯通 six-mode source -> save -> play/export/capture。 |
| CAM-P1-072 | Open | 无同 scene/rig/path/hardware/quality 的 Unreal/Unity raw competitive baseline。 |

## 7. P2：高阶能力

16 项仍全部 Open，不能用来替代 P1 闭环。

| Finding | 能力 |
|---|---|
| CAM-P2-001 | 程序化 composition/constraint solver 与目标优先级优化 |
| CAM-P2-002 | Virtual production lens calibration、distortion/ST map 与 tracking |
| CAM-P2-003 | 自动 focus、subject-aware focus pull 与 rack focus |
| CAM-P2-004 | lens breathing、anamorphic squeeze、blade/bokeh 与 chromatic model |
| CAM-P2-005 | camera animation、motion matching 与 handheld motion library |
| CAM-P2-006 | procedural shake source field 与空间传播 |
| CAM-P2-007 | spectator/replay/photo/debug drone 权限化 mode stack |
| CAM-P2-008 | deterministic camera recording、rollback 与 scrub |
| CAM-P2-009 | large-world origin rebase 与 double-precision director state |
| CAM-P2-010 | portal/mirror/recursive view family 与 recursion budget |
| CAM-P2-011 | multi-display、off-axis cave/projection wall 校准 |
| CAM-P2-012 | mobile/VR comfort、motion sickness 与 accessibility policy |
| CAM-P2-013 | ML framing/subject selection 作为可禁用 provider |
| CAM-P2-014 | plugin rig node sandbox、cost declaration 与 hot-reload migration |
| CAM-P2-015 | semantic camera source merge 与 multi-user live preview |
| CAM-P2-016 | 大规模 camera simulation farm 与跨 backend 图像/时序 oracle |

## 8. 资格门 Current-Source 重判

| Gate | 状态 | 必须证明 |
|---|---|---|
| CAM-G01 | Fail | capability 不从 viewer、Editor viewport 或静态 Sequencer UI 推导。 |
| CAM-G02 | Fail | endpoint/lens/rig/shake source roundtrip 与 unknown 字段无静默丢失。 |
| CAM-G03 | Fail | 相同 source/dependency/toolchain/target 重复 compile digest 一致。 |
| CAM-G04 | Fail | compile 失败保留同源 LKG 并报告 stale/rollback。 |
| CAM-G05 | Fail | invalid FOV/clip/viewport/lens/target 在 GPU 前 fail-close。 |
| CAM-G06 | Fail | Perspective/Orthographic override 可显式表达且无 sentinel 歧义。 |
| CAM-G07 | Partial | active ID 已保存，但 stable reference 尚未跨 reopen/prefab/cook/generation。 |
| CAM-G08 | Fail | Base/Overlay/stack/clear/layers 从 Scene source 到提交闭环。 |
| CAM-G09 | Partial | resolver 覆盖四类错误；orphan/duplicate/cycle/ownership 与 shipping disposition 未闭合。 |
| CAM-G10 | Partial | target format/generation report 与 viewport guard 已有；activation/readiness transaction 未闭合。 |
| CAM-G11 | Fail | 每个 World/Player/View 只有一个 active director generation。 |
| CAM-G12 | Fail | lease revoke、possession 与 unload 后旧 owner 不能写 view。 |
| CAM-G13 | Fail | evaluation 只消费 tick-start frozen input/target snapshot。 |
| CAM-G14 | Fail | `CameraViewResult` 在单一 commit 点发布全部通道。 |
| CAM-G15 | Fail | script/AI/audio/render 不直接读写 director 内部可变 state。 |
| CAM-G16 | Fail | shipping 关闭 development Orbit 后输入不改 Scene camera。 |
| CAM-G17 | Fail | UI consume、focus loss、pointer cancel 均终止 gesture。 |
| CAM-G18 | Fail | InputUser/MappingContext/viewport 在 split-screen 下隔离。 |
| CAM-G19 | Fail | rig node order/parameter/tie-break 有 deterministic oracle。 |
| CAM-G20 | Fail | async collision late result 按 world/director/tick generation 拒绝。 |
| CAM-G21 | Fail | blend 各通道、interrupt 与 completion 语义固定。 |
| CAM-G22 | Fail | Shake seed/stream 跨重启、调度、平台与 late join 一致。 |
| CAM-G23 | Fail | explicit CameraCut 更新 epoch 并传播全部 temporal consumer。 |
| CAM-G24 | Fail | 同位置硬切与合法高速连续运动正确区分。 |
| CAM-G25 | Fail | history key 包含 view/source/director/pipeline generation。 |
| CAM-G26 | Fail | retired camera 的七类 map 受 age/bytes/fence 回收。 |
| CAM-G27 | Fail | Scene reload/entity reuse 不继承旧 camera history。 |
| CAM-G28 | Fail | Sequencer cut 只经 director handshake 获取/释放 ownership。 |
| CAM-G29 | Fail | Shot/Take/Binding identity 贯通 artifact、frame、capture。 |
| CAM-G30 | Fail | split-screen 每 view 的 viewport/history/input/UI/audio 隔离。 |
| CAM-G31 | Fail | XR 多 eye shared/independent、late update、foveation/history 明确。 |
| CAM-G32 | Fail | network/save/replay 恢复 target/blend/shake/cut 且可复现。 |
| CAM-G33 | Fail | dedicated/headless 不创建无意义 render state但保留 authority。 |
| CAM-G34 | Fail | device loss 只降级 adapter，不改变 director state。 |
| CAM-G35 | Fail | 所有 queue/map/task 有 count/bytes/age/time budget 与 drop receipt。 |
| CAM-G36 | Fail | malformed/OOM/timeout/unload/skew fault matrix 通过。 |
| CAM-G37 | Partial | target/stack product tests 已有；六类产品 create/save/play/export/capture 未闭合。 |
| CAM-G38 | Fail | 1/2/4/16/100 views 有 CPU/GPU/RAM/VRAM/stutter raw receipts。 |
| CAM-G39 | Partial | viewport/pipeline/render-size 使部分状态失效；source/build/device/driver currentness 未统一。 |
| CAM-G40 | Fail | 优于 Unreal/Unity 绑定同语义、同硬件、同质量 raw evidence。 |

## 9. 分层重构里程碑

### CAM-M0 · Truth、Owner 与 Hard-Cut ADR

冻结 endpoint/director/view/result/history 术语，关闭“删除全部 camera 后无有效 endpoint”的父 correctness blocker；定义 `core::framework` 只保留中立合同、`core::runtime` 拥有 controller/director 生命周期的 hard cut。列出旧 API 删除表，禁止 facade/re-export/shim 双轨。

### CAM-M1 · Endpoint Schema、Admission 与 Render Source

建立 versioned `CameraEndpointDocument`、stable endpoint reference、完整 reflection/property admission 和 migration；把 projection、Base/Overlay、stack、clear depth、独立 layers、dynamic resolution 与 target policy 从 source 贯通到 render adapter；消除 `Perspective` sentinel。

### CAM-M2 · Lens、Rig、Shake Compiler 与 Artifact

建立 Lens/Rig/Shake versioned source、dependency manifest、deterministic compiler、immutable `CameraProgram`、diagnostic digest、DDC/LKG 与 publication transaction。compile artifact 不得携临时 Vec index 或 Editor pointer。

### CAM-M3 · Director、Ownership 与 Evaluation

建立 per-World/per-Player/per-View director generation、owner lease、possession、frozen input/target snapshot、固定 phase、single-commit `CameraViewResult`；script/AI/audio/render 改为 adapter/command consumer。

### CAM-M4 · Rig、Collision、Blend、Modifier 与 Shake

落地 typed node registry、Input Action intent、SpringArm/Boom、sync/async collision、occlusion、clock/space damping、multi-channel blend、modifier 与 deterministic shake；所有异步结果按 generation 拒绝且有 budget/receipt。

### CAM-M5 · Cut、History、Persistence 与 Sequencer

建立 typed CameraCut、history epoch/domain invalidation、bounded active-set history registry 与 fence-safe retirement；接入 Director save/load/network/replay；Sequencer cut 使用 typed binding 与 ownership handshake。

### CAM-M6 · Multi-View、Reliability 与真实产品

分离 `RenderEndpointId` 与 `DirectorViewId`，闭合 split-screen、spectator、capture、headless 和 XR adapter；完成 six-mode product fixtures、fault matrix、device loss 与 parity。

### CAM-M7 · Scale 与 Competitive Qualification

执行 1/2/4/16/100 views 的 CPU/GPU/RAM/VRAM/stutter 基线，绑定 quality/capability receipts；用同 scene/rig/path/hardware/quality 与 Unreal/Unity 比较。没有 raw evidence 不得宣称性能或表现优于参考引擎。

第一实施切片应先写 RED tests：删除 subtree 后至少保留/重建有效 camera、非法 source fail-close、Perspective override 无 sentinel、Scene source-to-stack、camera key retirement、shipping profile Orbit 不劫持输入；随后只实现 M0/M1 所需 owner/schema hard cut，不从 Shake helper 或新 Orbit mode 开始。

## 10. Finding 到里程碑与父项映射

| Runtime finding | 里程碑 | 共享父 owner |
|---|---|---|
| CAM-P1-001..012 | M0-M2 | Editor30、Runtime04/24 |
| CAM-P1-013..024 | M1 | Editor30、Runtime09/24/65 |
| CAM-P1-025..036 | M0/M3 | Runtime05/24/56、Editor30 |
| CAM-P1-037..048 | M3-M4 | Runtime22/56、Editor29/30 |
| CAM-P1-049..060 | M5-M6 | Runtime22/24/66、Editor45 |
| CAM-P1-061..072 | M6-M7 | Runtime59/65、PERF-MVP、产品资格 owner |
| CAM-P2-001..016 | M7 后独立立项 | 不阻塞 MVP，不得反向替代 P1 |

## 11. 禁止的临时修补

- 不给 `active_camera` 再加全局布尔、字符串 mode 或第二个裸 ID 冒充 director。
- 不用 transform lerp 冒充 blend、random offset 冒充 Shake、raycast helper 冒充 collision node。
- 不把 DOF 字段改名后冒充 physical lens profile。
- 不继续用 `Perspective` 默认值表达“没有 override”。
- 不把手工 render descriptor/stack test 当作可作者化 source 证据。
- 不用 velocity 阈值猜测替代显式 CameraCut/history epoch。
- 不让 script、AI、Sequencer、Editor 或 development controller 绕过 director 写 endpoint transform。
- 不在 live viewport 内为每个新 camera key 永久增长 HashMap/GPU history。
- 不在 `core::framework` 保留具体 controller behavior，也不用 re-export/compat module 维持双 authority。
- 不用 viewer 截图、单 camera 60 FPS 或无同语义基线的峰值数字证明“优于 Unreal”。

## 12. 实施职责蓝图

| 目标 owner | 职责 |
|---|---|
| Camera source/schema | Endpoint/Lens/Rig/Shake documents、stable IDs、validation、migration |
| Camera compiler/artifact | dependency、node plan、lens/shake tables、digest、LKG/publication |
| Runtime CameraDirector | Player/View instance、lease、target snapshot、evaluation、publish、save/network |
| Runtime controller/rig providers | Input intent、nodes、collision、blend、modifier、shake；不直接 render submit |
| Framework neutral contracts | typed intent、pose/lens/result/receipt trait 与 DTO；无业务 state/lifecycle |
| Render camera adapter | `CameraViewResult` 到 descriptor/view family/cut/history disposition |
| Viewport history registry | qualified key、budget、retirement、fence-safe回收与 diagnostics |
| Editor30/45 consumers | 围绕同一 schema/compiler/evaluator 做 authoring/preview/Sequencer |
| Product qualification | six-mode fixtures、fault/parity/scale、raw capture 与 competitive receipt |

最终 crate 与目录位置由 CAM-M0 ADR 根据固定 `zircon_app / zircon_runtime / zircon_editor` 包形态和 runtime 内部 `core/{runtime,framework,manager,math,resource}` spine 决定。本报告固定职责、依赖方向与 hard-cut 要求，不提前创建并行目录或第二套 authority。

## 13. 本轮产出边界

本轮只完成 Runtime Camera 当前源码复核、五套参考引擎对照、72 项 P1/16 项 P2/40 项 Gate 重判和 M0-M7 重构顺序。没有实施 Camera 代码，也没有把 existing partial render infrastructure 误记为 director/product closure。下一轮实施前必须重新检查 source currentness、并发 session、父 failure handoff 和受影响 API；任何关闭项都需要 source-to-product、failure、scale 与 currentness 证据。
