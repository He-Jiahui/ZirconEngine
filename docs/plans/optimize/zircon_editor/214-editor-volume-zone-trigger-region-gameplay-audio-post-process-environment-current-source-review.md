---
title: Editor Volume、Zone、Trigger、Region、Gameplay、Audio、Post Process 与 Environment 当前源码复核
category: zircon_editor
report_id: Editor214
review_date: 2026-08-29
baseline_head: a2d8d811c4a3a1fc1db6f5375c491e7e4502533f
verification_head: a2d8d811c4a3a1fc1db6f5375c491e7e4502533f
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor37
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/37-volume-zone-trigger-region-gameplay-audio-post-process-environment-authoring-review.md
  - docs/plans/optimize/zircon_editor/111-editor-volume-zone-trigger-region-gameplay-audio-post-process-environment-current-source-review.md
  - docs/plans/optimize/zircon_editor/158-editor-volume-zone-trigger-region-gameplay-audio-post-process-environment-current-source-review.md
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

# 214 · Editor Volume / Zone / Trigger / Region / Gameplay / Audio / Post Process / Environment 工程化差距

## 1. 结论

Editor37 的 canonical 判断仍成立：Zircon 当前没有一个工程级 `Region` 产品，而是 Post Process、Physics Trigger、Sound Volume 与 Navigation Modifier 四套分离的局部空间实现；Editor 又在这些实现之外展示固定 `VOL_DamageZone`、`VOL_AudioReverb`、`VOL_Checkpoint` 与 `VOL_StreamingGate`，形成没有 runtime owner 的第五套能力外观。对全仓 **20,131** 个生产 Rust/TOML/ZUI 文件做精确合同名检索后，`DamageZone`、`CheckpointVolume`、`StreamingGate`、`GameplayRegion`、`SpatialRegionId`、`CompiledRegionGeometry`、`RegionInstanceKey` 与 `RegionOverlapEvent` 均为零命中。

当前值得保留的是真实但不闭合的域内底座：Post Process 有 typed component registry、参数插值、priority/mask/weight 与 Box/Sphere extract；Physics Builtin/Jolt 有确定性 Enter/Stay/Exit pair diff，LevelSystem 有 immutable `Arc` frame snapshot 和旧 World 发布隔离；Sound 有 descriptor 校验、strongest resolver 与局部 DSP；Navigation Editor 有 typed operation、V2 progress、before/after snapshot、PIE owner-generation fence 和真实 viewport overlay。它们使 25 项 P1 与 15 个门禁维持 `Partial`，但没有建立 shared identity、compiled geometry、per-World index、domain adapter 或 transactional Editor toolkit，因此不能关闭任何 P0。

本轮再次确认一个确定性数据损失：Runtime `RenderPostProcessEffectStackSettings` 包含 color lookup、blur、motion blur、depth of field 与 screen-space reflection，`ScenePostProcessEffectStackAsset` 却只保存 tonemap、vignette、grain、dither、chromatic aberration 与 fog；`effect_stack_from_asset()` 用默认值补齐前五项，`effect_stack_to_asset()` 又不写回它们。Scene save/reopen 会清空已配置效果。该问题继续归入 P0-2、P1-10、P1-14、P1-57，不新增 canonical finding。

目标链保持为：

`versioned SpatialRegionSource -> deterministic CompiledRegionGeometry -> per-World generation-qualified RegionIndex/Snapshot -> typed domain adapters -> transactional Editor toolkit`

共享层只拥有 identity、geometry、transform、bounds、index、lifecycle 与 diagnostics；Post Process blend、Physics pair、Sound acoustic combination、Navigation area、Gameplay effect 与 Streaming lease 仍由各域 owner 负责。禁止用动态 property bag 或 Physics Collider 直接冒充全部空间域的永久 authority。

## 2. 审查范围与证据方法

### 2.1 当前工作树选择集

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 当前指纹 |
|---|---:|---|
| Zircon selected | **172 / 22,035 / 20,157 / 805,071 / 165 / 4** | `7a163629329188879662c60a164c72527f1d83c26f393e96cecfe9d73697d09b` |
| 五套参考 selected | **22 / 25,378 / 21,610 / 870,612 / 5 / 0** | `a2310c075f07ff632fe2f119023481d00ebe50400f17d3183c3e1c6096471b75` |
| 去重并集 | **194 / 47,413 / 41,767 / 1,675,683 / 170 / 4** | 当前物理工作树精确路径选择集 |

Zircon 选择集按 frontmatter 的 Runtime/Plugin/Editor/App 路径展开，根分布为 `zircon_runtime=76`、`zircon_plugins=92`、`zircon_editor=3`、`zircon_app=1`。指纹按规范化路径与文件内容 SHA-256 聚合；tests/ignored 只统计 Rust 属性。全生产精确扫描排除 `dev/`、`docs/`、`tools/`、`target/` 与 `.codex/`。Tooling 依用户要求不作为产品能力证据。

### 2.2 判定规则

1. `Open` 表示目标 contract、owner、consumer 或生命周期链不存在，或当前行为确定性破坏目标。
2. `Partial` 表示已有可执行、可测试且可保留的子链，但 identity、generation、product wiring 或资格证据没有闭合。
3. `Closed/Pass` 必须同时有当前源码、产品装配与对应动态证据；本轮没有项目达到该等级。
4. 类型、descriptor、capability、ZUI、测试 fixture、固定 feedback 与可选 Cargo feature 不单独证明产品可达。
5. 本轮只做静态 review，没有运行 Cargo、Editor、GPU、audio device、physics backend、navigation bake、fault、scale、soak 或跨引擎 benchmark。

## 3. 当前 Zircon 产品事实

### 3.1 Shared Region 仍为零

1. 四个域分别拥有自己的实体/ID、shape、priority、filter 与容器；不存在 shared `SpatialRegionId`、source revision、compiled geometry、world generation 或 lease。
2. 没有 shared broadphase、dirty set、cell partition、immutable query snapshot、stable query ordering 或 contributor trace。
3. Scene/prefab/streaming 不持有 Region source recipe、unknown-field preservation、artifact digest 或跨域 roundtrip。
4. 没有 provider SPI 声明 shape 对 PP/Physics/Sound/Nav/Gameplay/Streaming 的能力；失败发生在晚期 extract、heuristic bake、manager-local map 或不存在的 consumer。
5. 仅新增 `Region` enum/trait 会制造第五套 facade，不能解决 owner、artifact、generation 与 query authority 缺失。

### 3.2 Post Process：evaluator 真实，Scene contract 有损

1. `volume_registry.rs` 注册 15 个 built-in typed component并拒绝未知 component、错误参数类型与非法 ID；`volume_evaluator.rs` 按 override 与 interpolation policy 求值。
2. Scene render collect 仍遍历并排序全部 PP component，没有 shared index、dirty priority cache、camera/view generation 或跨 viewport snapshot。
3. local volume 只读取同实体 Collider；`render_post_process.rs` 只接受 Box/Sphere，Capsule/Cylinder/ConvexHull/TriangleMesh/HeightField/Compound 与缺 Collider 都返回 `None`。
4. 测试证明 unsupported shape 没被暗中降级，但产品没有 stable diagnostic code、source span 或 Inspector remediation，作者实际看到的是效果消失。
5. Scene asset/converter 丢失 LUT、blur、motion blur、DOF 与 SSR；这是 save/reopen 数据损坏，不是 UI 未完成。
6. Editor 没有 PP source document、property transaction、gizmo、profile reimport、runtime-backed preview 或 applied-generation receipt；workspace 与 callback 只输出固定事实。

### 3.3 Physics Trigger：pair diff 可用，事件身份不足

1. `PhysicsTriggerPair` 只有 `trigger_entity + other_entity`，BTreeMap 给出稳定迭代；双 sensor 会产生两个有方向 pair。
2. current/previous diff 生成 Enter/Stay/Exit，Stay 每次 scan 都发送，Exit 复用 previous point；destroy/filter/shape/world 变化没有 typed exit cause。
3. `PhysicsTriggerEvent` 仅包含 world、kind、trigger entity、other entity 与 point；缺 collider shape/subshape、pair generation、step、sequence、normal、filter decision 与 current-overlap handle。
4. `PhysicsFrameStateSnapshot` 的内部 generation、`Arc` 共享与旧 World producer 拒绝发布是真实底座，但 generation 没进入单个 event，也没有 bounded journal、authority lease、replay/network receipt。
5. Runtime Physics owner 报告还表明 contact/trigger 由 DTO 做 O(n²) 重建，不是 native narrow-phase authoritative stream；Damage/Checkpoint/Streaming consumer 仍为零。

### 3.4 Sound Volume：局部声学能力没有 World 投影

1. `SoundVolumeDescriptor` 有 manager-local ID、Sphere/Box、priority、gain、low-pass、reverb send、convolution send、crossfade distance 与有限值校验。
2. Box 只有 world-space center/extents；weight 按 axis-aligned distance 计算，缺 local transform、rotation、non-uniform scale provenance 与 source revision。
3. source environment 只把 source position 交给 strongest resolver；没有 listener/portal/acoustic context，也没有 Add/Blend/Override/Min/Max policy。
4. 整个 Sound plugin 生产源码对 `update_volume()` / `remove_volume()` 的调用为零；唯一调用位于非有限值测试。Scene/World create-update-remove、unload、rollback 与 generation receipt 均断路。
5. gain/low-pass/convolution 是真实局部 DSP，但没有进入生产 render block；Sound Editor drawer 仍是空 `Space`，catalog 也没有 Sound Editor provider。

### 3.5 Navigation Modifier：Editor 链较强，area geometry 仍是 heuristic

1. `area_volume.rs` 只处理 `NodeKind::Empty`，用 translation 作为 center、`abs(scale) * 0.5` 作为 half-extents，忽略 rotation 与 authored collider geometry。
2. area override 用 source node/entity position 做点包含，再把同一 area 写给该 node 的全部 triangle；没有 triangle clipping、centroid policy、overlap priority 或 conflict receipt。
3. bake geometry 对 Box 生成顶面 quad，对 Sphere/Capsule/Cylinder 生成 disc，对 ConvexHull 降为 AABB 顶面，TriangleMesh/HeightField 不生成 geometry。
4. dirty/tiled task、selected-surface settings、diagnostics 与 progress 数据结构存在，但 Runtime Bake production handler 仍固定失败，tile/cell 不是 durable、content-addressed、可回滚 artifact。
5. Editor 的 typed operation、V2 progress、before/after snapshot、undo、PIE stale fence 与 viewport overlay 应保留；modifier drawer 与 production controller/document 仍未闭合。

### 3.6 Gameplay、Streaming、Environment 与产品入口

1. Damage、Checkpoint、StreamingGate、GameplayRegion 没有 condition/effect/cooldown/authority、typed factory、save codec、network/replay 或 effect receipt。
2. Streaming 没有 cell lease、prefetch/cancel、I/O/memory budget、interest、handoff 或 rollback 与 Region generation 的绑定。
3. PP、Sound、Nav 的“环境”分别是图像参数、声学影响与行走 area，不能收敛成一个字符串 type 或 Environment bool。
4. Volume workspace 固定显示 `VOL_DamageZone`、`25 DPS`、`Priority 10`、`24 volumes`、`12 overlaps`、`1 warning`；Post Process workspace 固定显示 warning、Preview ready 与 `PPV_CityGlobal`。
5. callback 只修改 status/output text，返回固定 opened/queued 结果，不提交 document transaction、runtime request 或 receipt。
6. first-party Editor catalog 只有 Navigation 与 Neural provider；Runtime catalog 的 Sound/Navigation/Rendering 条目不能自动补齐 Sound/Physics/Rendering/Gameplay/Region Editor 链。

## 4. 五套参考源码对照

| 参考 | 可验证的工程边界 | Zircon 差距与采用边界 |
|---|---|---|
| Unreal `AVolume` 与各域 Volume | `AVolume` 持有 brush/bounds 和 `EncompassesPoint`；Physics/Audio/PostProcess/NavModifier 各自保留 priority、World 注册/注销、移动更新、查询与域内副作用；Audio 有 proxy，PP 有 GUID 稳定排序，Nav 有 transform/property/undo 失效 | 学习 shared authoring geometry、World lifecycle、domain adapter 与 invalidation；不复制 legacy 线性容器或跨域耦合 |
| Unity Graphics Volume | Volume enable/disable 注册，layer/priority 变化标 dirty；Collection 按 layer/priority 缓存；Manager 管理 per-camera stack reset/evaluate，profile/stack 有独立生命周期 | Zircon PP evaluator 类似，但缺 registry/index dirty cache、camera generation、profile lifecycle且 Scene 有损；采用 source/profile/collection/stack 分层 |
| Godot `Area3D` | Physics server 拥有 monitoring/monitorable；body/area map 按 ObjectID/RID 记录 shape pair/refcount，unbind/clear 生成完整 exit；重力与音频 override 仍为 typed domain policy | 学习 server-owned overlap lifecycle、shape-level pair 与 teardown completeness；不把全部 override 塞进 Region core |
| Fyrox Collider | reflected/inheritable shape、sensor/group、graph/native handle、dirty sync 与 PhysicsWorld query 分离，validation 能报告 parent/geometry source 错误 | 学习 authored source/native handle 分层与 validation；native collider 不能成为全部域唯一 artifact |
| Bevy ECS trigger | typed observer 可按 entity/component/global target 路由并支持传播与生命周期事件 | 只借鉴 typed dispatch 与 observer scope；该文件没有空间 geometry/index/overlap，不能当 Region 实现依据 |

参考实现共同要求 `authoring source -> lifecycle owner -> compiled/query representation -> domain consumer -> observable receipt`。性能目标只能在同功能、同质量、同平台与同统计方法下比较；功能缺失造成的低耗时不能作为优于 Unreal 的证据。

## 5. Authority 与目标架构

| 层 | 唯一 owner | 必须拥有 | 禁止拥有 |
|---|---|---|---|
| Region Source | Scene/Asset | stable ID、version、shape recipe、transform、domain metadata、unknown fields | native handle、Editor widget state |
| Region Compiler | neutral Runtime build service | validation、normalized geometry、bounds、capabilities、digest、diagnostics | Physics/Sound/Nav 私有策略、UI fallback |
| Region Runtime | per-World Region service | instance generation、lease、dirty index、immutable query snapshot、lifecycle | Gameplay effect、PP stack、Audio mixer state |
| Domain Adapter | 各域 owner | typed policy、consumer artifact、request/receipt、failure reason | 复制 source identity/index、无代际缓存 |
| Editor Toolkit | 各域 Editor provider | document/transaction、Inspector、gizmo、preview、job、diagnostics | 固定业务事实、直接改 Runtime map |
| App/Catalog | product composition | provider closure、capability truth、activation receipt | 从 contract/descriptor 推断产品可执行 |

关键序列必须是：

`EditTransaction -> SourceRevision -> CompileRequest -> RegionArtifact -> WorldInstallGeneration -> DomainApplyReceipt -> QualifiedObservation`

任一阶段失败都保留 last-good generation；Editor 只显示 authoritative state 或 typed Unavailable/Failed。

## 6. P0：必须先关闭的断路（5 Open）

| ID | 状态 | 当前证据 | 完整重构出口 |
|---|---|---|---|
| P0-1 | Open | 四域各有 identity/shape/container，无 SpatialRegionId、compiled geometry、World index/snapshot | 建立 shared source/compiler/runtime identity 与 query owner，再接 typed adapters |
| P0-2 | Open | PP local 只接受 Box/Sphere且无作者诊断；Scene roundtrip 丢 LUT/blur/motion blur/DOF/SSR | lossless schema/migration、早期 capability admission、transactional toolkit、applied-generation receipt |
| P0-3 | Open | Sound descriptor/resolver/DSP存在，但无Scene/World bridge、生产render caller或Editor provider | per-World AudioWorldSystem 投影 Region，绑定 listener/source、device/render generation 与 teardown |
| P0-4 | Open | Trigger缺shape/generation/sequence/filter；Nav点/AABB近似；Gameplay/Streaming consumer为零 | authoritative pair journal、exact/admitted geometry、typed factories与receipts |
| P0-5 | Open | Volume/PP workspace 与 callback 固定 VOL/PPV、数值、warning 和 queued | M0 标为 Fixture/Unavailable；接入真实 document/provider/job/snapshot 后恢复命令 |

## 7. P1：Runtime、空间索引、域适配与 Editor（35 Open / 25 Partial）

| ID | 状态 | 当前证据与需要重构的内容 |
|---|---|---|
| P1-01 | Open | 无 stable SpatialRegionId、component lineage、scene/world generation 或 instance lease |
| P1-02 | Open | 无共享 RegionShapeSource；各域 shape 集与 fallback 规则不一致 |
| P1-03 | Partial | Physics/PP有local transform与finite检查；Sound/Nav仍丢rotation或heuristic；需统一handedness、scale与bounds |
| P1-04 | Open | 无记录 exact shape、bounds、artifact key 与 query capability 的 CompiledRegionGeometry |
| P1-05 | Open | 无 source revision、unknown-field preservation、migration 与 deterministic digest |
| P1-06 | Open | 无 shared broadphase/index、dirty update、cell partition 与 immutable snapshot |
| P1-07 | Open | 无统一 point/overlap/raycast/nearest/containment query 与 stable ordering |
| P1-08 | Partial | Physics group/mask、PP camera mask、Nav agent filter各自typed；缺 shared domain/team/tag metadata |
| P1-09 | Open | 无 source/component/artifact/world/plugin ownership chain 与统一 diagnostics |
| P1-10 | Partial | Scene可保存PP/Collider局部字段；无Region identity/recipe，且PP effect stack确定性丢5类字段 |
| P1-11 | Open | PP unsupported/missing collider晚期返回None；无capability与typed fallback diagnostic |
| P1-12 | Partial | evaluator已有priority/mask/interpolation；无stack cache、dirty priority与camera/view generation |
| P1-13 | Partial | blend distance/weight/priority/override进入extract；未形成versioned compiled artifact |
| P1-14 | Open | 无PP source document/property transaction/undo/save/reimport闭环，Scene转换有损 |
| P1-15 | Open | unsupported PP shape不在authoring/cook/admission早拒绝 |
| P1-16 | Open | trigger pair/event缺shape/subshape、pair generation、step/sequence、normal/reason |
| P1-17 | Partial | Level snapshot有World replacement fence与内部generation；event缺current overlap/cause/world generation |
| P1-18 | Open | 无bounded trigger journal、authority/lease、replay/network receipt |
| P1-19 | Partial | pair消失能生成Exit且World publish拒旧；destroy/teleport/filter/shape语义未类型化 |
| P1-20 | Open | Sound volume无local transform/rotation/listener/source state或stable revision |
| P1-21 | Partial | strongest policy真实可执行；Add/Blend/Override/Min/Max与policy artifact不存在 |
| P1-22 | Open | Scene到Sound manager create/update/remove、generation、unload、rollback断路 |
| P1-23 | Open | influence只看source point；无listener/portal/acoustic context |
| P1-24 | Partial | IR/reverb/convolution与局部DSP存在；无production block、latency/tail/budget/device receipt |
| P1-25 | Open | Sound drawer为空；无schema toolkit、gizmo、audition、transaction或save/reimport |
| P1-26 | Partial | Nav有agent/filter/inheritance与bake路径；无source revision及完整provenance artifact |
| P1-27 | Open | Nav area忽略rotation并用AABB point heuristic；无显式approximation policy/error diagnostic |
| P1-28 | Open | 无triangle clipping/centroid policy、overlap priority或conflict receipt |
| P1-29 | Partial | 有dirty/tiled task/progress/snapshot；结果非durable tile/cell artifact且Runtime Bake断路 |
| P1-30 | Partial | 有typed operation/progress/undo/provider；modifier drawer与production controller/document不完整 |
| P1-31 | Open | 无Gameplay Region condition/authority/cooldown/effect/policy contract |
| P1-32 | Open | Damage/Checkpoint/StreamingGate无factory、save/load、network/replay |
| P1-33 | Open | effect未绑定pair/sequence/generation，无法拒绝stale event |
| P1-34 | Open | Streaming无cell lease、prefetch/cancel、budget与generation receipt |
| P1-35 | Open | World Partition region无server/client authority、interest、handoff与rollback |
| P1-36 | Partial | PP/Physics/Nav有局部lifecycle与generation底座；无shared Region lifecycle/device-loss语义 |
| P1-37 | Open | 无Region contributor/provider SPI，缺owner/factory/service不能统一fail-close |
| P1-38 | Partial | 各域有局部validation/error；无统一stable code/source span/domain/generation/remediation |
| P1-39 | Open | ResourceKind/AssetType没有PPVolume/SoundVolume/NavModifier/GameplayRegion分型产品 |
| P1-40 | Partial | Scene PP持久化与Nav before/after可复用；无统一source document/revision/recovery/conflict |
| P1-41 | Open | Sound/Nav drawer为空，PP/Volume静态；无defaults/resolved/runtime capability Inspector |
| P1-42 | Open | 无shared Region gizmo/picking/shape edit、snap、multi-select与undo |
| P1-43 | Open | workspace读取固定VOL/PPV业务事实而非Scene/World snapshot |
| P1-44 | Partial | Nav operation有typed handle/progress/snapshot；其它Inspect/Validate/Apply只写queued |
| P1-45 | Partial | Nav viewport消费真实PIE snapshot；PP/Sound/Physics/Gameplay overlay与trace缺失 |
| P1-46 | Open | PP preview未绑定camera/viewport frame generation，workspace preview为固定文本 |
| P1-47 | Open | Sound audition/influence preview不连接真实Sound provider |
| P1-48 | Open | Physics preview没有pair/shape/filter/sequence/current-overlap投影 |
| P1-49 | Partial | Nav overlay画真实triangles/links/agents；不显示area affected tiles/conflict/provenance |
| P1-50 | Open | Gameplay/Streaming preview无authority、PIE/network/save state |
| P1-51 | Partial | Navigation runtime/editor已进catalog/App；其余Editor与Gameplay/Region仍缺失 |
| P1-52 | Partial | Navigation有真实factory/provider/consumer；Sound descriptor与通用Workbench无admission closure |
| P1-53 | Partial | Nav task/progress与Runtime task基础存在；无统一bounded cancel/retry/shutdown drain |
| P1-54 | Partial | Nav snapshot与多域cache底座存在；无Region content-addressed key/chunk/atomic/GC合同 |
| P1-55 | Partial | shape/descriptor finite与局部malformed tests存在；无huge mesh/path fuzz、budget与OOM证明 |
| P1-56 | Open | 无Physics/Sound/PP/Nav共享source到runtime deterministic golden |
| P1-57 | Partial | Scene PP/Collider有局部roundtrip；无Region identity且PP effect stack字段丢失 |
| P1-58 | Partial | Physics World generation与Nav PIE owner-generation可保留；未覆盖multi-context隔离 |
| P1-59 | Open | 无1/1k/100k Region、overlap storm、dirty/query/audio/bake压力基准 |
| P1-60 | Open | 无完整headless cook/package/client/server/editor矩阵或同功能跨引擎方法学 |

## 8. P2：长期能力（12 Open）

| ID | 状态 | 能力 |
|---|---|---|
| P2-01 | Open | GPU broadphase、multi-level spatial index、batched query、SIMD/parallel evaluation |
| P2-02 | Open | SDF/voxel/mesh/composite region、exact distance、portal/acoustic propagation |
| P2-03 | Open | procedural/runtime-painted/destructible/streamed geometry artifact |
| P2-04 | Open | cross-scene/global registry、World Partition federation、remote ownership |
| P2-05 | Open | advanced audio portals/occlusion/reverb、binaural/ambisonics、room graph |
| P2-06 | Open | PP graph blending、LUT/stack cache、per-region/camera temporal history |
| P2-07 | Open | dynamic Nav obstacle/area、multi-agent masks、incremental GPU bake |
| P2-08 | Open | deterministic replicated Region、prediction、rollback、replay/save migration |
| P2-09 | Open | Editor multi-user locks、field merge、presence、review annotations |
| P2-10 | Open | conflict、unsupported shape、stale bridge、budget hotspot auto audit |
| P2-11 | Open | schema/algorithm migration、canary cook、old generation pin/rollback |
| P2-12 | Open | 公开fixture的跨引擎spatial feature/performance/quality benchmark suite |

## 9. 分层重构顺序

### M0 Truthfulness 与有损数据止血

1. 将 Volume/PP workspace 固定结果标为 Fixture/Unavailable，移除 stable/complete 暗示与 control-local success。
2. 修复 PP Scene schema 与双向转换，为五类缺失效果提供旧 schema migration 与 lossless roundtrip test。
3. 输出 domain owner/capability matrix；descriptor-only、contract-only、缺 factory 或缺 consumer 的能力必须 fail-close。

### M1 Shared Region Source 与 Compiler

建立 stable ID、versioned source、typed shapes、transform/handedness/finite rules、unknown preservation、deterministic digest、CompiledRegionGeometry、capability table 与 stable diagnostics。该层不依赖 Editor 控件、Physics native object 或 Sound manager。

### M2 Per-World Region Runtime

建立 RegionInstance lease、world/plugin generation、dirty broadphase/cell partition、immutable query snapshot、stable ordering、install/retire/rollback receipt。完成 1/1k/100k source/index correctness 与 budget 基线后再接高级域。

### M3 Post Process 与 Physics Adapter

PP 接入 lossless Scene source、shape admission、compiled blend policy、camera/view generation stack cache；Physics 扩展 shape-level pair identity、step/sequence/filter/current overlap/exit reason 与 bounded authority journal。Builtin/Jolt 消费同一 neutral event contract。

### M4 Sound 与 Navigation Adapter

Sound 建立 AudioWorldSystem、listener/source/acoustic context、combination policy、IR/device/applied-generation receipt；Navigation 替换 point/AABB heuristic，形成 exact 或显式 approximate 的 triangle/tile artifact、conflict/provenance 与 incremental rollback。

### M5 Gameplay 与 Streaming Consumers

建立 GameplayRegion/Damage/Checkpoint/StreamingGate source、compiler 与 runtime factories；effect/cooldown/authority 绑定 pair sequence，Streaming 消费 cell lease、prefetch/cancel、budget、interest、handoff 与 rollback。

### M6 Transactional Editor Toolkits

各域建立 AssetType/ResourceKind、source document、Inspector、gizmo、selection、undo/save/reimport、preview/debug 与 background job；通用 workspace 只聚合真实 domain snapshots，不成为第五个 authority。

### M7 Fault、Scale 与 Release Qualification

完成 malformed/fuzz/failure injection、roundtrip、cross-domain golden、multi-world isolation、large-world/overlap storm、headless/package/platform/provider matrix 与 old-generation rollback。性能优于 Unreal 的声明只能在功能、质量和可比性门通过后进行。

## 10. 验收门禁（17 Fail / 15 Partial / 0 Pass）

| Gate | 状态 | 当前证据与通过条件 |
|---|---|---|
| G01 Stable identities/generations | Fail | 无Region source/artifact/instance identity；建立端到端ID与generation |
| G02 Shape/transform admission | Partial | 多域有finite/shape validation；PP晚丢、Sound/Nav变换有损，需compile早拒绝 |
| G03 Spatial index/snapshot | Fail | 无shared dirty/revision/ordering/cancel query owner |
| G04 Post Process golden | Partial | evaluator有tests；Scene丢字段、unsupported无诊断、Editor无preview generation |
| G05 Physics pair contract | Fail | Enter/Stay/Exit有，但shape/sequence/filter/current/reason缺失 |
| G06 Stale/lifecycle isolation | Partial | Level拒绝旧World发布；teleport/destroy/filter/shape/unload语义未闭合 |
| G07 Sound lifecycle/output | Fail | 无Scene bridge、production spatial block、combination/device recovery |
| G08 Navigation exact artifact | Fail | area仍点/AABB，geometry近似/跳过，Runtime Bake断路 |
| G09 Gameplay authority | Fail | 类型、factory、effect/cooldown/network/replay/save均不存在 |
| G10 Streaming leases | Fail | Region到cell/interest/budget/rollback链不存在 |
| G11 Provider/catalog admission | Partial | Navigation装配真实；其它域Editor与Gameplay/Region缺失 |
| G12 Artifact atomicity | Partial | Nav snapshot/多域cache可复用；无Region key/manifest/atomic/GC/rollback |
| G13 Scene/prefab roundtrip | Partial | PP/Collider局部保存；无Region identity且PP effect stack有损 |
| G14 Inspector/gizmo transaction | Fail | drawer静态/空，无真实document与gizmo |
| G15 Operation truth | Partial | Nav operation/progress真实；其它命令仍固定queued |
| G16 Runtime-backed overlays | Partial | Nav有真实provider；其余域无snapshot/provider trace |
| G17 Malformed/fuzz/budget | Partial | 有finite与局部failure tests；无全域fuzz、huge input与OOM/budget证明 |
| G18 Cross-domain deterministic golden | Fail | 无shared source/build/runtime golden或failure injection matrix |
| G19 Multi-context isolation | Partial | Physics/Nav有局部generation fence；多view/listener/agent/client未覆盖 |
| G20 Scale/performance | Fail | 无1/1k/100k Region与overlap/query/bake/audio统计 |
| G21 Headless independence | Partial | 局部Runtime不依赖ZUI；无完整Region产品与headless cook/package lane |
| G22 Capability truth | Partial | Runtime catalog分feature且Nav闭合；Editor/App closure仍不一致 |
| G23 Durable editing/recovery | Fail | 无Region document、autosave/recovery/conflict/hot reload/cancel事务 |
| G24 Last-good failure recovery | Fail | 无跨source/provider/job/disk/device的last-good generation证明 |
| G25 Visual/audio/data goldens | Partial | PP/trigger/Nav/Sound局部tests存在；没有完整输出与temporal golden |
| G26 Diagnostics filtering/export | Partial | 域内errors存在；无按region/domain/world/generation/receipt统一导出 |
| G27 Workspace truthfulness | Fail | VOL/PPV、数值、warning与queued仍为固定业务事实 |
| G28 Unsupported fail-close | Fail | PP静默None、Nav静默近似/跳过，缺stable code/remediation |
| G29 Cook/package manifest | Partial | Scene/Nav/PP有局部carrier；无Region geometry/dependency/provenance/platform manifest |
| G30 Cross-platform determinism | Fail | 无shape/handedness/backend/bake跨平台证据 |
| G31 Release/migration/rollback | Fail | 无schema/algorithm canary、old generation pin与rollback |
| G32 Evidence-derived maturity | Fail | 固定UI与descriptor仍制造能力外观；无完整动态资格链 |

## 11. 禁止的临时修补

1. 禁止只新增 `Region` enum/trait/ResourceKind，却继续让四域复制 identity、shape、index 与 lifecycle。
2. 禁止把 Physics Collider/native shape 直接作为全部域的永久 authority；必须经过 neutral compiled geometry 与 capability contract。
3. 禁止把 unsupported PP/Nav shape 偷换成 AABB/Sphere，或继续静默返回 `None`/跳过而无作者诊断。
4. 禁止只给 Scene PP asset 加字段，不做旧 schema migration、双向 roundtrip 与 Editor transaction test。
5. 禁止让 Sound manager 每帧全量扫描 Scene 或重建 volume 来补 bridge。
6. 禁止用 source position 替代 listener/source/portal/acoustic context，或把 strongest 固化为唯一策略。
7. 禁止只给 Physics event 加 sequence 数字，不定义 pair identity、world/step generation、exit cause 与 bounded journal。
8. 禁止把 Nav 点/AABB 分类改名为 exact；任何近似必须显式、可诊断、可度量。
9. 禁止先实现 DamageZone/Checkpoint callback，再把 authority、save/network/replay 推迟。
10. 禁止用无限 event/job/query queue、扩大 cache 或每帧全量排序掩盖索引与 backpressure 缺失。
11. 禁止为各域复制 Editor document/gizmo/operation framework，或让 workspace 直接修改 Runtime manager map。
12. 禁止以类型、测试、capability、optional feature 或静态 ZUI 存在宣称产品可用。
13. 禁止在缺同功能、同质量、同平台、同统计方法的 correctness/latency/memory/stability/golden 前宣称优于 Unreal。

## 12. 跨计划 Owner 与实施边界

1. Editor37 是本报告 5 P0、60 P1、12 P2 与 32 门的唯一 canonical owner；Editor111/158/214 只刷新 currentness。
2. Runtime Physics 当前报告拥有 solver/backend/query/contact 基础；本报告只拥有 Region 到 trigger pair/authority 合同。
3. Runtime Audio 当前报告拥有 device/mixer/render/streaming；本报告拥有 Scene Region 到 AudioWorldSystem 投影与 volume policy。
4. Runtime Navigation 与 Editor141 拥有 Nav 产品；本报告只要求 shared Region geometry 进入 Nav adapter。
5. Editor144 拥有 Rendering Editor 与完整 PP toolkit；本报告拥有 Region/shape/lossless Scene 边界。
6. Scene/Asset、document/transaction、jobs、plugin lifecycle、RHI/device-loss 与 World Partition 底层失败由各 owner 先修，Region 不建立兼容旁路。
7. 全局 MVP 00 仍为 `in_progress`；本报告是 review 与依赖记录，不授权越过 M0-M2 实现高级 Gameplay/Streaming/Portal/GPU broadphase。

## 13. 本轮产出边界

本轮只新增 current-source review、状态重判、参考差异、架构边界、分层里程碑与门禁；没有修改 Runtime/Plugin/App/Editor production code 或 tests。`review_complete` 仅表示本报告冻结的 194 文件选择集已经完成静态取证，不表示 Editor37 实施完成，也不表示任何动态资格门通过。

实施前必须重导 Zircon 172 文件 manifest/fingerprint 并核对 shared working tree drift，重点复查 PP Scene schema/converter、local shape admission、Physics event/snapshot、Sound World caller、Nav area geometry、Workbench feedback 与 catalog/App closure。Tooling 继续按用户要求排除，也不得通过等待或轮询协调器阻塞其它审查里程碑。
