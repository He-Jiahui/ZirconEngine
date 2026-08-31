---
title: Editor Volume、Zone、Trigger、Region、Gameplay、Audio、Post Process 与 Environment 当前源码复核
category: zircon_editor
report_id: Editor158
review_date: 2026-08-27
baseline_head: 3328c6ce8a712098ca42a3265d35655b91c67167
verification_head: a076a662c593f886f9113a804b551c78914efd76
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor37
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/37-volume-zone-trigger-region-gameplay-audio-post-process-environment-authoring-review.md
  - docs/plans/optimize/zircon_editor/111-editor-volume-zone-trigger-region-gameplay-audio-post-process-environment-current-source-review.md
related_code:
  - zircon_runtime/src/core/framework/render/post_process
  - zircon_runtime/src/asset/assets/scene/post_process.rs
  - zircon_runtime/src/scene/components/scene/post_process.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/scene/world/project_io/post_process.rs
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/scene/level_system/physics_runtime_enabled.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/trigger
  - zircon_plugins/physics/runtime/src/backend/jolt/runtime.rs
  - zircon_runtime/src/core/framework/sound/components.rs
  - zircon_plugins/sound/runtime/src/service_types/acoustics.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment
  - zircon_plugins/sound/editor
  - zircon_runtime/src/core/framework/navigation/modifier.rs
  - zircon_plugins/navigation/runtime/src/manager/bake
  - zircon_plugins/navigation/editor
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_volume_editor_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_post_process_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_app/Cargo.toml
tests:
  - zircon_runtime/src/core/framework/render/post_process/volume_component/tests.rs
  - zircon_runtime/src/asset/tests/assets/scene/post_process.rs
  - zircon_runtime/src/scene/tests/render_post_process_extract.rs
  - zircon_runtime/src/core/framework/physics/tests.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/trigger
  - zircon_plugins/sound/runtime/src/engine/source_environment/volume/filter.rs
  - zircon_plugins/sound/editor/src/tests.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/task_pool.rs
  - zircon_plugins/navigation/editor/src/tests/bake_panel_retained.rs
  - zircon_plugins/navigation/editor/src/tests/operation_command.rs
  - zircon_plugins/navigation/editor/src/tests/viewport_overlay_provider.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/99zm-runtime-physics-world-body-shape-collider-material-joint-query-contact-trigger-fixed-step-jolt-character-controller-vehicle-ragdoll-debug-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zn-runtime-audio-sound-clip-streaming-device-mixer-bus-effect-spatial-occlusion-reverb-timeline-event-voice-chat-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zp-runtime-navigation-navmesh-recast-detour-tilecache-crowd-query-pathfinding-obstacle-off-mesh-link-bake-streaming-world-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/139-editor-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-audition-current-source-review.md
  - docs/plans/optimize/zircon_editor/140-editor-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-preview-current-source-review.md
  - docs/plans/optimize/zircon_editor/141-editor-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-preview-current-source-review.md
  - docs/plans/optimize/zircon_editor/144-editor-render-pipeline-render-graph-frame-debugger-capture-lighting-bake-reflection-probe-post-process-debug-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/Volume.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Volume.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/TriggerVolume.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/PhysicsVolume.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/PhysicsVolume.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Sound/AudioVolume.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/AudioVolume.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/AudioDevice.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/PostProcessVolume.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/PostProcessVolume.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/World.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavModifierVolume.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Private/NavModifierVolume.cpp
  - dev/godot/scene/3d/physics/area_3d.h
  - dev/godot/scene/3d/physics/area_3d.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/Volume.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeCollection.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeProfile.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeStack.cs
  - dev/Fyrox/fyrox-impl/src/scene/collider.rs
  - dev/bevy/crates/bevy_ecs/src/event/trigger.rs
finding_status:
  p0_open: 5
  p0_partial: 0
  p0_closed: 0
  p1_open: 35
  p1_partial: 25
  p1_closed: 0
  p2_open: 12
  p2_partial: 0
  p2_closed: 0
gate_status:
  fail: 17
  partial: 15
  pass: 0
---

# 158 · Editor Volume / Zone / Trigger / Region / Gameplay / Audio / Post Process / Environment 工程化差距

## 1. 结论

Editor37 的核心判断仍成立：Zircon 没有一个工程级 `Region` 产品，只有 Post Process、Physics Trigger、Sound Volume 与 Navigation Modifier 四套互不共享 identity、geometry、index、generation 和 diagnostics 的局部实现。Editor 又在它们之外提供固定 `VOL_DamageZone`、`VOL_AudioReverb`、`VOL_Checkpoint`、`VOL_StreamingGate` 的通用 Volume workspace，形成第五套无 runtime owner 的假 authority。全仓生产源码仍没有 `DamageZone`、`CheckpointVolume`、`StreamingGate`、`GameplayRegion`、`SpatialRegionId` 或 `CompiledRegionGeometry` 产品实现。

本轮确认了值得保留的真实底座：Post Process 有 15 个 typed component、参数级插值、mask/priority/weight 与 Box/Sphere local extract；Physics 的 Builtin/Jolt 共享 deterministic Enter/Stay/Exit pair diff，LevelSystem 发布 immutable `Arc` frame snapshot 并拒绝旧 World 写入；Sound descriptor 有有限值校验、gain/low-pass/reverb/convolution 与 strongest resolver；Navigation Editor 有 typed bake operation、V2 progress、before/after snapshot、PIE owner-generation fence 和真实 viewport overlay provider。这些局部进展把 25 项 P1 和 15 个门禁提升为 Partial，但没有关闭任何 P0，也没有形成跨域 Region 产品。

本轮还发现一个此前没有充分写明的当前数据损失：Runtime `RenderPostProcessEffectStackSettings` 包含 color lookup、blur、motion blur、depth of field 和 screen-space reflection，但 `ScenePostProcessEffectStackAsset` 只持久化 tonemap、vignette、grain、dither、chromatic aberration 和 fog；`effect_stack_from_asset()` 用默认值补齐前五项，`effect_stack_to_asset()` 又完全不写它们。因此这些效果经 Scene save/reopen 会被确定性清空。该问题归入既有 P0-2、P1-10、P1-14 和 P1-57，不新增 canonical finding。

目标边界保持为：

`versioned SpatialRegionSource -> deterministic CompiledRegionGeometry -> per-World generation-qualified RegionIndex/Snapshot -> typed domain adapters -> transactional Editor toolkit`

共享的是 identity、geometry、transform、bounds、index、lifecycle 和 diagnostics，不共享 Post Process blend、Physics pair、Sound acoustic combination、Navigation area、Gameplay effect 或 Streaming authority 语义。不能把四个 domain 塞进一个动态 property bag，也不能把 Physics Collider 直接宣布为全部 Region 的唯一 source。

## 2. 审查范围与方法

### 2.1 当前工作树选择集

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 指纹与证据 |
|---|---:|---|
| Zircon Runtime/Plugin/Editor/App selected | **197 / 28,852 / 26,647 / 1,056,339 / 173 / 0** | Post Process、Physics trigger、Sound environment、Navigation bake/editor、Workbench/callback、catalog/App；`1b14a8253a4e01bfe90f14ffb3de20404b0bcb4d343d522e3933f6a90ce960cc` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference | **22 / 25,378 / 21,610 / 870,612 / 5 / 0** | Unreal 13、Godot 2、Unity Graphics 5、Fyrox 1、Bevy 1；`0a311fc4427e32d4193621599c579d209fc95692314128de3223cbe9ab1796d5` |
| 全部选择集 | **219 / 54,230 / 48,257 / 1,926,951 / 178 / 0** | 当前共享 working tree 去重物理语料；`3620bc1ad88f972ffd63267bca9b34b4f791fed12ec4c833b11588719676ba22` |

统计对当前物理文件按规范化路径排序，以 `path + NUL + lowercase(file SHA-256) + LF` 聚合 SHA-256；test/ignored 仅统计 Rust 属性。选择集按明确路径前缀与跨域入口冻结，包含相关未跟踪文件，不用宽泛词法命中夸大覆盖。Tooling 按用户要求排除；本轮是静态 review，没有运行 Cargo、Editor、GPU、audio device、physics backend、navigation bake、fault、scale、soak 或跨引擎 benchmark。

### 2.2 证据等级

1. `Open`：目标 contract、owner 或产品链不存在，或现有行为与目标冲突。
2. `Partial`：存在可执行、可测试且可保留的子链，但 canonical item 的 identity、lifecycle、consumer 或资格证据未闭合。
3. `Closed/Pass`：必须有当前源码、产品装配和相应动态证据；本轮没有任何项达到该等级。
4. 类型名、descriptor、capability、ZUI、测试 fixture、固定 feedback 和可选 Cargo feature 不单独证明产品可达。
5. 负证据通过限定 production roots 的精确类型/调用方检索取得；不会从 `dev/` 或 `docs/` 命中反推 Zircon 已实现。

## 3. 当前 Zircon 产品链

### 3.1 Shared Region：仍为零

1. 四个 domain 分别拥有自己的实体/ID、shape、priority、filter 和更新容器；不存在共享 `SpatialRegionId`、source revision、compiled geometry、world generation 或 lease。
2. 没有 shared broadphase、dirty set、cell partition、immutable query snapshot、stable query ordering 或 contributor trace。
3. Scene/prefab/streaming 只认识各自 component；没有 Region source recipe、unknown-field preservation、artifact digest 或跨域 roundtrip。
4. 没有 provider SPI 来声明某 shape 能否用于 Post Process、Physics、Sound、Navigation、Gameplay 或 Streaming；当前失败分别发生在 render extract、bake heuristic、manager map 或根本不存在的 consumer。
5. 这不是命名问题。先增加 `Region` enum 而不建立 owner、artifact、generation 和 query contract，只会生成第五套运行时 authority。

### 3.2 Post Process：真实 evaluator，Scene contract 有损

1. `volume_registry.rs` 注册 15 个 built-in typed component，拒绝未知 component、错误参数类型和非法 ID；`volume_evaluator.rs` 对每个参数按 override 与 interpolation policy 求值，并保留未设置项。
2. Scene 每个 render collect 仍遍历并排序全部 Post Process components，没有 shared index、dirty cache、camera/view generation 或跨 viewport snapshot。
3. local volume 只读取同实体 Collider。`render_post_process.rs:234-250` 接受 Box/Sphere，Capsule/Cylinder/ConvexHull/TriangleMesh/HeightField/Compound 直接返回 `None`；缺 Collider 同样被排除。
4. `render_post_process_extract.rs` 的局部测试证明没有把 unsupported shape 偷换成 Box/Sphere，但产品没有 stable diagnostic code、source span 或 Inspector remediation，所以作者看到的是效果消失。
5. Runtime effect stack 在 `effect_stack_settings.rs:26-32` 明确包含 color lookup、blur、motion blur、depth of field、SSR；Scene asset 在 `post_process.rs:177-190` 只保存 6 个其它效果。
6. `project_io/post_process.rs:146-170` 对缺失的 5 个字段使用 Runtime default，反向保存也不写这些字段。这是确定性 roundtrip 数据丢失，不是尚未提供 UI 的轻微缺口。
7. Editor 没有 PostProcessVolume source document、property transaction、gizmo、profile reimport、runtime-backed preview 或 applied generation receipt；Post Process workspace 只提供固定内容和字符串 feedback。

### 3.3 Physics Trigger：有稳定 diff，没有工程级事件身份

1. `PhysicsTriggerPair` 只有 `trigger_entity + other_entity`，BTreeMap 提供稳定迭代；双 sensor 会产生两个有方向 pair。
2. current/previous diff 生成 Enter/Stay/Exit，Stay 每次 scan 都发，Exit 复用 previous point。destroy/filter/shape/world 变化没有 typed exit cause。
3. `PhysicsTriggerEvent` 在 `trigger_event.rs:9-14` 只有 world、kind、trigger entity、other entity 和 point；没有 collider shape/subshape、pair generation、step、sequence、normal、filter decision 或 current-overlap handle。
4. `PhysicsFrameStateSnapshot` 是真实进展：内部 generation 递增，contacts/triggers 使用 `Arc`，相同输出复用 snapshot，World replacement 后旧 producer 不得发布。
5. snapshot generation 没有进入每个 event；没有 bounded trigger journal、authority lease、replay/network receipt 或 gameplay dispatcher。
6. 生产调用链没有 Damage、Checkpoint、Streaming 或 Navigation consumer，所以 Enter/Exit 只能证明物理底层存在，不能证明区域玩法存在。

### 3.4 Sound Volume：有 DSP 字段，没有 World owner

1. `SoundVolumeDescriptor` 有 manager-local ID、Sphere/Box、priority、gain、low-pass、reverb send、convolution send 和 crossfade distance，并有有限值与正值校验。
2. Box 只保存 world-space center/extents；`weight.rs:18-37` 做 axis-aligned distance，没有 local transform、rotation、non-uniform transform provenance 或 source revision。
3. `apply/volume_influence.rs:19` 只把 source position 交给 `strongest_volume_influence()`；resolver 依 priority、weight 和 stable ID 选择一个 strongest。没有 listener/portal/acoustic context，也没有 Add/Blend/Override/Min/Max policy。
4. manager 的 `update_volume_impl()`/`remove_volume_impl()` 只更新内部 map。排除定义与测试后，App/World/Scene 没有生产调用者，也没有 unload/rollback/generation receipt。
5. gain/low-pass/convolution 代码是真实局部 DSP，但 Runtime139 已证明 source_environment 没有进入生产 render block；字段存在不能替代可听输出证据。
6. Sound Editor 的 `audio_volume.drawer.zui` 四行全部是 `Space`，33 个 operation descriptor 没有 factory，first-party Editor catalog 没有 Sound provider。

### 3.5 Navigation Modifier：Editor 基础进步，geometry 仍是 heuristic

1. `area_volume.rs:39-43` 用 entity translation 作为 center、`abs(scale) * 0.5` 作为 half extents，完全忽略 rotation 和 authored collider geometry。
2. area override 用 source node/entity position 做 point containment，然后把同一 area 写给该 node 产生的全部 triangle；没有 triangle clipping、centroid policy、overlap priority 或 conflict receipt。
3. bake geometry 把 Box 变成顶面 quad，把 Sphere/Capsule/Cylinder 变成 disc，把 ConvexHull 降为 AABB 顶面；TriangleMesh/HeightField 不生成 geometry。
4. runtime 有 dirty/tiled task、selected-surface settings、diagnostics 和 progress 数据结构，但 Runtime Bake production handler 仍固定失败，tile/cell 也不是可持久化、可回滚的 content-addressed artifact。
5. Editor 已有 typed bake operation factory、V2 progress 单调性检查、before/after generated snapshot、undo、PIE sequence/owner-generation 拒旧和 viewport overlay provider，应保留并接入真实 Runtime owner。
6. `navmesh_modifier.drawer.zui` 仍只有一个业务 `Space`；controller 仍主要由 focused tests 拥有，overlay 只读取当前 PIE/default options，未显示受影响 triangles/tiles、agent conflict 或 provenance。

### 3.6 Gameplay、Streaming 与 Environment：产品不存在

1. production roots 对 DamageZone、CheckpointVolume、StreamingGate、GameplayVolume、GameplayRegion、SpatialRegion 和 CompiledRegionGeometry 的精确检索没有领域实现。
2. 没有 condition/effect/cooldown/authority、pair sequence rejection、network/replay/save codec 或 typed runtime factory。
3. 没有 cell lease、prefetch/cancel、I/O/memory budget、interest、handoff 或 rollback 与 Region generation 的绑定。
4. Post Process、Sound、Navigation 的“环境”含义分别是图像参数、声学影响和行走 area，不能合并为一个 Environment bool 或字符串 type。
5. World Partition/Level Streaming 的其它通用计划只能提供潜在 consumer，不会自动关闭 Region source/index/authority 差距。

### 3.7 Editor、Catalog 与 App：静态事实仍在产品入口

1. Volume workspace 在 `:79` 固定选择 `VOL_DamageZone`，`:152` 固定 Bounds 12x8x6，`:168` 固定 25 DPS/Priority 10，`:184` 固定 24 volumes/12 overlaps/1 warning，dropdown 固定四种 VOL 与四类 domain。
2. Post Process workspace 固定 Global Stack/Cinematic Grade、Bloom 0.65、`LUT_CityWarm`、Interior EV +2.1 Warning 和 `PPV_CityGlobal`。
3. callback 在 `extension_module_feedback.rs:118-139` 与 `:554-564` 只改 status/output text，固定返回 Preview/Apply/Inspect/Validate queued；不提交 document transaction、runtime request 或 receipt。
4. first-party Editor catalog 只返回 Navigation 与 Neural provider；没有 Rendering、Sound、Physics、Gameplay 或 Region Editor provider。
5. first-party Runtime catalog在相应 feature 下能返回 Sound、Navigation、Rendering；`target-editor-host` 装配 advanced render runtime、Navigation runtime/editor 和 Neural editor，但只有 physics/sound contracts，没有 Sound/Physics/Rendering Editor 闭环。
6. Navigation 是当前唯一具有真实 operation factory 和 viewport provider 的本域 Editor 子链，因此 catalog/admission 只能判 Partial，不能据此把通用 Volume workspace 判为可用。

## 4. 五套参考源码对照

| 参考 | 可验证的工程边界 | Zircon 当前差异 | 采用边界 |
|---|---|---|---|
| Unreal `AVolume` / Trigger / Physics / Audio / PostProcess / NavModifier | Volume 是可编辑 Brush；各 domain 保留独立 actor/component 语义。PhysicsVolume/AudioVolume/PostProcessVolume 随 World 注册、更新、排序、卸载；AudioVolumeProxy 跨到 audio thread并按 WorldID 与 body geometry查询；NavModifier 随 transform/property/undo/area registration 失效导航 | Zircon 四域没有共享 identity/geometry/index，Sound 无 World bridge，PP unsupported shape 晚丢，Nav 用点/AABB heuristic，Editor 无 transaction/toolkit | 学习 owner、World lifecycle、shape source、domain adapter 和 invalidation；不复制 Unreal 的 legacy 线性容器或历史耦合 |
| Unity Graphics Volume | `Volume` 在 enable/disable 注册；layer/priority 变化标 dirty；`VolumeCollection` 缓存 layer-mask sorted list；`VolumeManager` 每 camera reset stack，再按 global/local collider distance混合；profile/stack 有独立 ownership/lifetime | Zircon有类似 PP evaluator，但无 registry/index dirty cache、per-camera generation、profile lifecycle，且 Scene effect stack丢 5 类字段 | 学习 source/profile/stack/collection/camera evaluation 分层。Unity 在无 physics/collider 时跳过 local volume 也不是 Zircon 可照搬的作者体验 |
| Godot `Area3D` | CollisionObject3D shape、monitoring/monitorable 进入 physics server；body/area map 按 ObjectID/RID 记录 shape pair 与 refcount；clear/unbind 会发完整 exits；另有 override priority 和 audio bus/reverb | Zircon Trigger pair 没 shape/refcount/sequence/reason，Sound与Physics又是两套无桥的空间事实 | 学习 server-owned overlap lifecycle、shape-level pair 和 teardown completeness；不把 Area3D 的所有 override 属性塞进共享 Region core |
| Fyrox Collider | reflected/inheritable typed shape、sensor、collision/solver group、graph/native handle 与 dirty sync；query来自 PhysicsWorld；validation能报告 parent/geometry source错误 | Zircon Physics collider可用，但没有跨域 compiled geometry/source diagnostics，也没有 Editor Region gizmo/transaction | 学习 authored source与native handle分离、dirty sync和validation；Physics native collider不是所有 domain 的永久唯一 artifact |
| Bevy ECS trigger | typed event/observer 支持 entity/component/global targeting、propagation和生命周期触发 | Zircon没有 typed gameplay Region dispatch，但 Bevy文件本身不提供空间 geometry/index/overlap | 只借鉴 typed dispatch与observer scope；不能把 ECS trigger API当空间区域参考实现 |

参考差异说明了最低工程闭环是 `authoring source -> lifecycle owner -> compiled/query representation -> domain consumer -> observable receipt`。性能目标必须在该闭环之后按同内容、同算法质量、同平台、同统计方式比较；不能用 Zircon 缺功能后的低耗时宣称优于 Unreal。

## 5. Authority 与目标架构

| 层 | 唯一 owner | 必须拥有 | 禁止拥有 |
|---|---|---|---|
| Region Source | Scene/Asset | stable ID、version、shape recipe、transform、domain metadata、unknown fields | Runtime native handle、Editor widget状态 |
| Region Compiler | Runtime neutral build service | validation、normalized geometry、bounds、capabilities、digest、diagnostics | Physics/Sound/Nav 私有策略或 UI fallback |
| Region Runtime | per-World Region service | instance generation、lease、dirty index、immutable query snapshot、lifecycle | Gameplay effect、PP stack或Audio mixer状态 |
| Domain Adapter | PP/Physics/Sound/Nav/Gameplay/Streaming owner | typed policy、consumer-specific artifact、request/receipt、failure reason | 复制 source identity/index 或无代际缓存 |
| Editor Toolkit | domain Editor provider | document/transaction、Inspector、gizmo、preview、job、diagnostics | 固定业务事实、直接改 Runtime map、control-local success |
| App/Catalog | product composition | selected provider closure、capability truth、activation receipt | 隐式从 contract/descriptor 推断 provider可执行 |

关键运行序列必须是：

`EditTransaction -> SourceRevision -> CompileRequest -> RegionArtifact -> WorldInstallGeneration -> DomainApplyReceipt -> QualifiedObservation`

任何阶段失败都保留上一完整 generation；Editor 只显示 authoritative state 或 typed Unavailable/Failed，不显示预设成功文本。

## 6. P0：必须先关闭的断路（5 Open）

| ID | 状态 | 当前证据 | 完整重构出口 |
|---|---|---|---|
| P0-1 | Open | 四域各有 identity/shape/container；无 SpatialRegionId、compiled geometry、World index/snapshot | 建立 shared Region source/compiler/runtime identity 与 query owner，再接 domain adapters |
| P0-2 | Open | PP local只接受Box/Sphere且无作者诊断；Scene save/reopen还丢 LUT/blur/motion blur/DOF/SSR | lossless Scene schema + early capability admission + transactional PP document/toolkit + applied generation receipt |
| P0-3 | Open | Sound descriptor/strongest/DSP存在，但无 Scene/World create-update-remove、production render caller或Editor provider | per-World AudioWorldSystem 投影 Region instance，绑定listener/source、device/render generation和teardown |
| P0-4 | Open | Trigger缺shape/pair generation/sequence/filter/current overlap；Nav用点/AABB；Gameplay/Streaming consumer为零 | authoritative pair journal + exact/admitted geometry + typed gameplay/streaming factories与receipts |
| P0-5 | Open | 两张Workspace与callback继续固定VOL/PPV、数值、warning和queued结果 | M0先改为Fixture/Unavailable；只在真实document/provider/job/snapshot接入后恢复产品命令 |

## 7. P1：Runtime、空间索引、域适配与 Editor（35 Open / 25 Partial）

| ID | 状态 | 当前证据与需要重构的内容 |
|---|---|---|
| P1-01 | Open | 无 stable SpatialRegionId、component lineage、scene/world generation 或 instance lease |
| P1-02 | Open | 无共享 RegionShapeSource；各域 shape 集和fallback规则互不一致 |
| P1-03 | Partial | Physics/PP有local transform和finite检查，Sound/Nav仍丢rotation或用heuristic；需统一handedness、non-uniform scale与bounds |
| P1-04 | Open | 无记录exact shape、bounds、artifact key与query capability的CompiledRegionGeometry |
| P1-05 | Open | 无Region source revision、unknown-field preservation、migration和deterministic digest |
| P1-06 | Open | 无shared broadphase/index、dirty update、cell partition和immutable query snapshot |
| P1-07 | Open | 无统一point/overlap/raycast/nearest/containment query及stable ordering |
| P1-08 | Partial | Physics layer/group/mask、PP camera mask、Nav agent filter各自typed；没有共享domain/team/tag/user metadata |
| P1-09 | Open | 无Region source/component/artifact/world/plugin ownership chain和统一diagnostics |
| P1-10 | Partial | Scene能保存PP/Collider局部字段；无Region identity/recipe，且PP effect stack确定性丢5类字段 |
| P1-11 | Open | PP unsupported/missing collider仍晚期返回None；无shape capability和typed fallback diagnostic |
| P1-12 | Partial | typed evaluator已有priority/mask/interpolation；无stack cache、dirty priority和camera/view generation |
| P1-13 | Partial | blend distance/weight/priority/override进入Runtime extract；未形成versioned compiled artifact |
| P1-14 | Open | 无PP source document/property transaction/undo/save/reimport闭环，且现有Scene转换有损 |
| P1-15 | Open | unsupported PP shape不在authoring/cook/admission早拒绝 |
| P1-16 | Open | Physics pair/event缺shape/subshape、pair generation、step/sequence、normal/reason |
| P1-17 | Partial | Level snapshot有World replacement fence和内部generation；event无filter/current overlap/cause/world generation |
| P1-18 | Open | 无bounded trigger journal、authority/lease、replay/network receipt |
| P1-19 | Partial | current/previous diff能因pair消失生成Exit且World publish拒旧；没有destroy/teleport/filter/shape typed语义 |
| P1-20 | Open | Sound volume无local transform/rotation/listener/source state或stable revision |
| P1-21 | Partial | strongest policy真实可执行；Add/Blend/Override/Min/Max及policy artifact不存在 |
| P1-22 | Open | Scene到Sound manager create/update/remove、generation、unload和rollback断路 |
| P1-23 | Open | influence只看source point；无listener/portal/acoustic context |
| P1-24 | Partial | IR/reverb/convolution字段和局部DSP存在；无production block、latency/tail/budget/device receipt |
| P1-25 | Open | Sound drawer是Space；无schema toolkit、gizmo、audition、transaction或save/reimport |
| P1-26 | Partial | Nav有agent/filter/inheritance字段和bake路径；无source revision及完整provenance artifact |
| P1-27 | Open | Nav area忽略rotation并用AABB point heuristic；无显式approximation policy/error diagnostic |
| P1-28 | Open | 无triangle clipping/centroid policy、overlap priority或conflict receipt |
| P1-29 | Partial | 有dirty/tiled task/progress/generated snapshot；area结果仍非durable tile/cell artifact且Runtime Bake断路 |
| P1-30 | Partial | 有typed operation/progress/undo/provider；modifier drawer和production controller/document仍不完整 |
| P1-31 | Open | 无Gameplay Region condition/authority/cooldown/effect/policy contract |
| P1-32 | Open | Damage/Checkpoint/StreamingGate无factory、save/load、network/replay |
| P1-33 | Open | Gameplay effect未绑定pair/sequence/generation，无法拒绝stale event |
| P1-34 | Open | Streaming无cell lease、prefetch/cancel、budget和generation receipt |
| P1-35 | Open | World Partition region无server/client authority、interest、handoff和rollback |
| P1-36 | Partial | PP/Physics/Nav各有局部spawn/update/unload行为及generation底座；无共享Region lifecycle和device-loss语义 |
| P1-37 | Open | 无Region contributor/provider SPI，缺owner/factory/service不能统一fail-close |
| P1-38 | Partial | 四域各有局部validation/error；无统一stable code/source span/shape/domain/generation/remediation |
| P1-39 | Open | ResourceKind/AssetType没有PostProcessVolume/SoundVolume/NavModifier/GameplayRegion分型产品 |
| P1-40 | Partial | Scene PP持久化和Nav before/after command可复用；无统一source document/revision/dirty/recovery/conflict |
| P1-41 | Open | Sound/Nav drawer为空，PP/Volume为静态模板；无domain defaults/resolved/runtime capability Inspector |
| P1-42 | Open | 无共享Region gizmo/picking/shape edit、snap、multi-select和undo |
| P1-43 | Open | 通用Workspace继续读取固定VOL/PPV业务事实而非Scene/World snapshot |
| P1-44 | Partial | Navigation operation有typed handle/progress/snapshot；其它Inspect/Validate/Apply只写queued文本 |
| P1-45 | Partial | Navigation viewport provider消费真实PIE snapshot；PP/Sound/Physics/Gameplay overlay与trace缺失 |
| P1-46 | Open | PP preview没有绑定camera/viewport frame generation，Workspace preview为固定文本 |
| P1-47 | Open | Sound audition/influence preview不连接真实Sound provider |
| P1-48 | Open | Physics preview没有pair/shape/filter/sequence/current overlap产品投影 |
| P1-49 | Partial | Nav overlay能画真实triangles/links/agents；不显示area affected tiles/conflict/provenance |
| P1-50 | Open | Gameplay/Streaming preview无authority、PIE/network/save state |
| P1-51 | Partial | Navigation runtime/editor已进catalog/App；Sound/Physics/Rendering Editor与Gameplay/Region仍缺失 |
| P1-52 | Partial | Navigation有真实factory/provider/consumer；Sound descriptors与通用Workbench仍无admission closure |
| P1-53 | Partial | Nav task/progress和Runtime task基础存在；无统一bounded cancel/retry/shutdown drain |
| P1-54 | Partial | Nav generated snapshot与多域cache底座存在；无Region content-addressed key/chunk/atomic/GC合同 |
| P1-55 | Partial | shape/descriptor有限值和局部malformed tests存在；无huge mesh/path fuzz、budget和全域panic/OOM证明 |
| P1-56 | Open | 无Physics/Sound/PP/Nav共享source到runtime deterministic golden |
| P1-57 | Partial | Scene PP/Collider有局部roundtrip；无Region identity且PP effect stack字段丢失 |
| P1-58 | Partial | Physics World generation与Nav PIE owner-generation可保留；未覆盖multi-view/listener/agent/client隔离 |
| P1-59 | Open | 无1/1k/100k Region、overlap storm、dirty/query/audio/bake压力基准 |
| P1-60 | Open | 无完整headless cook/package/client/server/editor矩阵或同功能跨引擎方法学 |

## 8. P2：长期能力（12 Open）

| ID | 状态 | 能力 |
|---|---|---|
| P2-01 | Open | GPU broadphase、multi-level spatial index、batched query、SIMD/parallel evaluation |
| P2-02 | Open | SDF/voxel/mesh/composite region、exact distance、portal/acoustic propagation |
| P2-03 | Open | procedural/runtime-painted/destructible/streamed Region geometry artifact |
| P2-04 | Open | cross-scene/global registry、World Partition federation、remote ownership |
| P2-05 | Open | advanced audio portals/occlusion/reverb、binaural/ambisonics、room graph |
| P2-06 | Open | PP graph blending、LUT/stack cache、per-region/camera temporal history |
| P2-07 | Open | dynamic Nav obstacle/area、multi-agent masks、incremental GPU bake |
| P2-08 | Open | deterministic replicated Region state、prediction、rollback、replay/save migration |
| P2-09 | Open | Editor multi-user locks、field merge、presence、review annotations |
| P2-10 | Open | overlapping conflict、unsupported shape、stale bridge、budget hotspot auto audit |
| P2-11 | Open | Region schema/algorithm migration、canary cook、old generation pin/rollback |
| P2-12 | Open | 公开fixture的跨引擎spatial feature/performance/quality benchmark suite |

## 9. 分层重构顺序

### M0 Truthfulness 与有损数据止血

1. 将 Volume/Post Process workspace 的固定结果标为 Fixture/Unavailable，移除 stable/complete 暗示和 control-local success。
2. 先修复 PP Scene schema 与双向转换，保证 LUT/blur/motion blur/DOF/SSR lossless roundtrip，并为旧 schema 提供migration test。
3. 输出 current domain owner/capability matrix；descriptor-only、contract-only或缺factory能力必须fail-close。

### M1 Shared Region Source 与 Compiler

建立 stable ID、versioned source、typed shapes、transform/handedness/finite rules、unknown preservation、deterministic digest、CompiledRegionGeometry、capability table和stable diagnostics。该层不依赖Editor控件、Physics native object或Sound manager。

### M2 Per-World Region Runtime

建立 RegionInstance lease、world/plugin generation、dirty broadphase/cell partition、immutable query snapshot、stable query ordering、install/retire/rollback receipt。完成1/1k/100k source/index correctness与budget基线后再接高级域。

### M3 Post Process 与 Physics Adapter

PP先接lossless Scene source、shape admission、compiled blend policy、camera/view generation stack cache；Physics再扩展shape-level pair identity、step/sequence/filter/current overlap/exit reason和bounded authority journal。Builtin/Jolt必须消费同一 neutral event contract。

### M4 Sound 与 Navigation Adapter

Sound建立AudioWorldSystem、listener/source/acoustic context、combination policy、IR/device/applied-generation receipt；Navigation替换point/AABB heuristic，形成exact或显式approximate的triangle/tile artifact、conflict/provenance和incremental rollback。

### M5 Gameplay 与 Streaming Consumers

建立 GameplayRegion/Damage/Checkpoint/StreamingGate source、compiler和runtime factories；effect/cooldown/authority与pair sequence绑定，Streaming消费cell lease、prefetch/cancel、budget、interest、handoff和rollback。

### M6 Transactional Editor Toolkits

为各domain建立AssetType/ResourceKind、source document、Inspector、gizmo、selection、undo/save/reimport、preview/debug和background job；通用workspace只聚合真实domain snapshots，不成为第五个authority。

### M7 Fault、Scale 与 Release Qualification

完成malformed/fuzz/failure injection、roundtrip、cross-domain golden、multi-world isolation、large-world/overlap storm、headless/package/platform/provider组合及old-generation rollback。性能优于Unreal的声明只能在功能与质量门通过后进行。

## 10. 验收门禁（17 Fail / 15 Partial / 0 Pass）

| Gate | 状态 | 当前证据与通过条件 |
|---|---|---|
| G01 Stable identities/generations | Fail | 无Region source/artifact/instance identity；建立端到端ID与generation |
| G02 Shape/transform admission | **Partial** | 多域有finite/shape validation；PP仍晚丢、Sound/Nav变换有损，需authoring/compile早拒绝 |
| G03 Spatial index/snapshot | Fail | 无shared dirty/revision/ordering/cancel query owner |
| G04 Post Process golden | **Partial** | evaluator有priority/blend/mask tests；Scene丢字段、unsupported无诊断、Editor无preview generation |
| G05 Physics pair contract | Fail | Enter/Stay/Exit有，但shape/sequence/filter/current/reason缺失 |
| G06 Stale/lifecycle isolation | **Partial** | Level publish拒绝旧World；teleport/destroy/filter/shape/unload语义未闭合 |
| G07 Sound lifecycle/output | Fail | 无Scene bridge、production spatial block、combination/device recovery |
| G08 Navigation exact artifact | Fail | area仍点/AABB，geometry多种近似/跳过，Runtime Bake断路 |
| G09 Gameplay authority | Fail | 产品类型、factory、effect/cooldown/network/replay/save均不存在 |
| G10 Streaming leases | Fail | Region到cell/interest/budget/rollback链不存在 |
| G11 Provider/catalog admission | **Partial** | Navigation装配真实；Sound/Physics/Rendering Editor与Gameplay/Region缺失 |
| G12 Artifact atomicity | **Partial** | Nav snapshot/多域cache可复用；无Region key/manifest/atomic/GC/rollback |
| G13 Scene/prefab roundtrip | **Partial** | PP/Collider局部可保存；无Region identity且PP effect stack有损 |
| G14 Inspector/gizmo transaction | Fail | 本域drawer静态/Space，无真实document与gizmo |
| G15 Operation truth | **Partial** | Nav operation/progress真实；Volume/PP/Sound/Physics/Gameplay仍固定queued |
| G16 Runtime-backed overlays | **Partial** | Nav有真实provider；其余域无snapshot/provider trace |
| G17 Malformed/fuzz/budget | **Partial** | 有finite与局部failure tests；无全域fuzz、huge input和OOM/budget证明 |
| G18 Cross-domain deterministic golden | Fail | 无共享source/build/runtime golden或failure injection矩阵 |
| G19 Multi-context isolation | **Partial** | Physics/Nav局部generation fence；多view/listener/agent/client未覆盖 |
| G20 Scale/performance | Fail | 无1/1k/100k Region与overlap/query/bake/audio统计 |
| G21 Headless independence | **Partial** | 局部Runtime不依赖ZUI；无完整Region产品和headless cook/package lane |
| G22 Capability truth | **Partial** | Runtime catalog分feature且Nav闭合；Editor/App provider closure仍与contracts/descriptors不一致 |
| G23 Durable editing/recovery | Fail | 无Region document、autosave/recovery/conflict/hot reload/cancel事务 |
| G24 Last-good failure recovery | Fail | 无跨source/provider/job/disk/device的Region last-good generation证明 |
| G25 Visual/audio/data goldens | **Partial** | PP/trigger/Nav/Sound局部tests存在；没有完整输出与temporal golden |
| G26 Diagnostics filtering/export | **Partial** | 域内errors存在；无按region/domain/entity/world/generation/receipt统一筛选导出 |
| G27 Workspace truthfulness | Fail | VOL/PPV、数值、warning和queued仍为固定业务事实 |
| G28 Unsupported fail-close | Fail | PP静默None、Nav静默近似/跳过，缺stable code/remediation |
| G29 Cook/package manifest | **Partial** | Scene/Nav/PP有局部carrier；无Region geometry/domain/dependency/provenance/platform manifest |
| G30 Cross-platform determinism | Fail | 无shape/handedness/physics backend/Nav bake跨平台证据 |
| G31 Release/migration/rollback | Fail | 无Region schema/algorithm canary、old generation pin和rollback |
| G32 Evidence-derived maturity | Fail | 固定UI与descriptor仍可制造能力外观；无完整动态资格链 |

## 11. 禁止的临时修补

1. 禁止只新增 `Region` enum/trait/ResourceKind，却继续让四域复制 identity、shape、index 和 lifecycle。
2. 禁止把 Physics Collider/native shape 直接作为所有域永久 authority；必须经过neutral compiled geometry与capability contract。
3. 禁止为支持 PP Capsule/Convex 而偷偷降为AABB/Sphere，或继续在render extract返回None而无作者诊断。
4. 禁止仅给 `ScenePostProcessEffectStackAsset` 加字段却不做旧schema migration、双向roundtrip和Editor transaction测试。
5. 禁止让Sound manager通过全局扫描Scene或每帧重建全部volume来补Scene bridge。
6. 禁止以source position替代listener/source/portal/acoustic context，或把strongest硬编码成唯一组合语义。
7. 禁止给Physics event只加一个sequence数字而不定义pair identity、world/step generation、exit cause和bounded journal。
8. 禁止把Navigation area继续按entity origin分类后改名为exact；近似必须显式、可诊断、可度量。
9. 禁止先实现DamageZone/Checkpoint脚本callback，再等待authoritative pair、save/network/replay contract未来补齐。
10. 禁止用无限event/job/query queue、扩大cache或全量每帧排序掩盖索引和backpressure缺失。
11. 禁止为每个domain复制Editor document/gizmo/operation framework，或让Workspace直接修改Runtime manager map。
12. 禁止以类型、测试、capability、optional feature或静态ZUI存在宣称产品可用。
13. 禁止在缺同功能、同质量、同平台、同统计方法的correctness/latency/memory/stability/golden前宣称优于Unreal。

## 12. 跨计划 Owner 与实施边界

1. Editor37是本报告全部5 P0、60 P1、12 P2和32门的唯一canonical owner；Editor111/158只刷新currentness，不重复增加全局finding。
2. Runtime Physics当前报告拥有solver/backend/query/contact基础；本报告只拥有Region到trigger pair/authority产品合同，不复制Physics backend。
3. Runtime Audio99zn拥有device/mixer/render/streaming；本报告拥有Scene Region到AudioWorldSystem投影及volume policy，不复制完整Sound重构。
4. Runtime Navigation99zp与Editor141拥有Nav runtime/editor产品；本报告只要求共享Region geometry进入Nav adapter，实际tile/crowd/query owner仍在Navigation计划。
5. Editor144拥有Rendering Editor与完整Post Process toolkit；本报告拥有Region/shape/lossless Scene边界，不能另建第二套PP Editor。
6. Scene/Asset、document/transaction、jobs、plugin lifecycle、RHI/device-loss或World Partition底层失败由各自owner先修；Region不得建立兼容旁路。
7. 全局MVP 00仍为in_progress。本报告是review与依赖记录，不授权越过M0-M2直接实现高级Gameplay/Streaming/Portal/GPU broadphase。

## 13. 本轮产出边界

本轮只新增current-source review、状态重判、参考差异、架构边界、分层里程碑和门禁；没有修改 Runtime/Plugin/App/Editor production code 或 tests。`review_complete`只表示冻结的219文件范围已经完成静态取证，不表示Editor37实施完成，也不表示任何动态资格门通过。

后续实现前必须重导197个Zircon文件manifest/fingerprint并核对共享working tree drift，重点复查PP Scene schema/converter、local shape admission、Physics event/snapshot、Sound World caller、Nav area geometry、Workbench feedback、catalog/App closure。Tooling继续按用户要求排除；也不得通过等待或轮询协调器阻塞其它审查里程碑。
