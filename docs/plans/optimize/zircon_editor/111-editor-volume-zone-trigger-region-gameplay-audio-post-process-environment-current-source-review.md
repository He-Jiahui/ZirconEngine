---
title: Editor Volume、Zone、Trigger、Region、Gameplay Audio、Post Process 与 Environment 当前源码复核
category: zircon_editor
report_id: Editor111
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor37
refreshes:
  - docs/plans/optimize/zircon_editor/37-volume-zone-trigger-region-gameplay-audio-post-process-environment-authoring-review.md
related_code:
  - zircon_runtime/src/asset/assets/scene/post_process.rs
  - zircon_runtime/src/scene/components/scene/post_process.rs
  - zircon_runtime/src/scene/components/scene/physics.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/scene/world/property_access
  - zircon_runtime/src/core/framework/render/post_process
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/core/framework/sound/components.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/trigger
  - zircon_plugins/physics/runtime/src/backend/jolt/runtime.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/sound/runtime/src/components.rs
  - zircon_plugins/sound/runtime/src/service_types/acoustics.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment
  - zircon_plugins/sound/editor
  - zircon_runtime/src/core/framework/navigation/modifier.rs
  - zircon_plugins/navigation/runtime/src/manager/bake
  - zircon_plugins/navigation/editor
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_volume_editor_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_post_process_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings
tests:
  - zircon_runtime/src/core/framework/physics/tests.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_component/tests.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/trigger
  - zircon_plugins/sound/runtime/src/engine/source_environment/volume/filter.rs
  - zircon_plugins/sound/editor/src/tests.rs
  - zircon_plugins/sound/editor/src/live_output/controller.rs
  - zircon_plugins/navigation/runtime/src/manager/bake/task_pool.rs
  - zircon_plugins/navigation/editor/src/tests.rs
  - zircon_plugins/navigation/editor/src/tests/bake_panel_retained.rs
  - zircon_plugins/navigation/editor/src/tests/operation_command.rs
  - zircon_plugins/navigation/editor/src/tests/viewport_overlay_provider.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09h2-exposure-color-bloom-dof-motion-blur-ssr-terminal-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
  - docs/plans/optimize/zircon_editor/18-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/19-navigation-navmesh-settings-agent-area-surface-modifier-obstacle-off-mesh-link-bake-query-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/21-gameplay-ability-effect-attribute-tag-cue-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/Volume.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/TriggerVolume.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/PhysicsVolume.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Sound/AudioVolume.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/PostProcessVolume.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavModifierVolume.h
  - dev/godot/scene/3d/physics/area_3d.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/Volume.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeProfile.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeStack.cs
  - dev/Fyrox/fyrox-impl/src/scene/collider.rs
  - dev/bevy/crates/bevy_ecs/src/event/trigger.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 111 · Editor Volume / Zone / Trigger / Region / Gameplay Audio / Post Process / Environment 工程化差距

## 1. 结论

Zircon 不是没有空间区域能力，而是已有四套局部语义：Post Process volume、Physics trigger、Sound environment、Navigation modifier。它们各自拥有 shape、priority、filter、lifecycle 和 evaluation 规则；Editor 却用固定的 `VOL_DamageZone`、`VOL_AudioReverb`、`VOL_Checkpoint`、`VOL_StreamingGate` workspace 把这些底层原语压成一个没有 owner 的万能 Volume。这个 UI 的 bounds、DPS、overlap、warning 和 queued feedback 不是任何 runtime authority 的投影。

可保留底座包括：Scene 对 `PostProcessVolumeComponent`/`ColliderComponent` 的保存，Post Process 的 typed registry、global/local Box/Sphere extract、camera mask、priority 与连续 blend evaluator；Physics Collider 的 sensor/layer/group/mask 和 Builtin/Jolt 共享 Enter/Stay/Exit 计算；Sound descriptor 的 finite/shape validation、gain/low-pass/reverb/convolution effect；Navigation modifier 的 hierarchy inheritance 与 bake-time area mutation。它们不能被替换为一个 property bag。

当前断路很具体：local Post Process 只有同实体 Box/Sphere Collider 才能 extract，Capsule/Convex/Triangle/HeightField/Compound 会被排除但没有用户可见拒绝诊断；Sound Box 只保存 axis-aligned world center/extents，生产代码找不到 Scene -> Sound manager 创建/更新/销毁桥；Navigation area 使用 Empty 节点 scale 的 AABB，忽略 rotation，并以 source node position 而非实际三角形裁切判定；Physics event 只有实体和 point，没有 pair generation、sequence、shape/subshape 或 filter trace；没有 DamageZone、Checkpoint、StreamingGate、GameplayVolume 产品执行器。

正确边界是共享稳定的空间身份、几何、transform、filter、revision、artifact、index 和 diagnostics，然后分别由 Post Process、Audio、Physics、Navigation、Gameplay、Streaming adapter 消费：

`SpatialRegionSource -> CompiledRegionGeometry -> generation-qualified RegionInstance/Index -> typed domain evaluator -> Editor document/Inspector/gizmo/receipt`

共享 geometry 不意味着共享效果字段、组合规则、查询时机或 authority。Unreal 的 Volume/Trigger/Physics/Audio/PostProcess/NavModifier、Godot Area3D 的 overlap reference count、Unity Volume Stack/Manager 和 Fyrox sensor 都支持这一分层；Bevy trigger event 不能被误作完整空间区域系统。

## 2. 审查范围与证据

### 2.1 当前工作树物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests | 指纹 |
|---|---:|---|
| Zircon Runtime/Editor/Plugin selected | **261 / 43,256 / 40,321 / 1,586,154 / 160** | Scene/PostProcess/Physics、Sound environment、Navigation bake、Workbench/bridge/bindings；`c6494ef877c8fea4166eb6826c954b3439eedee6f5c13feec88ef538e7dd6509` |
| Unreal/Godot/Fyrox/Bevy/Unity reference | **13 / 4,305 / 3,713 / 167,958 / 5** | Unreal Volume/Trigger/Physics/Audio/PostProcess/NavModifier、Godot Area3D、Unity Volume Stack、Fyrox Collider、Bevy trigger；`d6c195fdc9d1bc577b8191eada11bc5c2cfff1f35960a2ca19aa8c89c9cf5c89` |
| Zircon selected union | **274 / 47,561 / 44,034 / 1,754,112 / 165** | current physical working tree union；`a21b9231d6f7fcfc519f394d2b4f6d2e1bff66683f119fbe57f5cc1f069a1cd3` |

本轮范围包含共享 Workbench bridge，因此比旧报告更宽；统计按 root 去重、排序后以 UTF-8 内容计算 SHA-256，test 数仅为属性计数。当前 baseline epoch 524，工作树含在途修改；实施前必须重导 manifest/fingerprint 并重新核对 volume component/registry/route。没有运行 Cargo、physics backend、sound spatial、post-process GPU、navigation bake、Editor interaction 或大世界压力验证。

### 2.2 Post Process 事实

1. Scene asset/component 持久化 active/global/priority/weight/blend distance/profile，global volume 不需要 shape。
2. local volume 依赖同实体 Collider；extract 只接受 Box/Sphere，缺 Collider 或其它 shape 返回空。
3. Box 组合 entity/collider local transform，Sphere 以最大非均匀 scale 形成保守球；这是窄而真实的 evaluator，不是通用区域系统。
4. unsupported shape 会让效果消失，测试虽标注不静默降级，产品没有 source/Inspector/cook diagnostic 告知作者。
5. camera mask/render layer、priority order、stable entity ordering 与 typed parameter interpolation 已有底座。
6. 每次收集遍历 archetype 并排序，没有共享 spatial index、dirty set、cell artifact 或跨 viewport snapshot。
7. Editor 生产搜索只有 Post Process workspace/route，未找到该 component 的完整 Scene property access、transaction、toolkit 或 artifact compiler。

### 2.3 Physics Trigger 事实

1. Collider 的 sensor、layer、group/mask、material、local transform 可保存并被 Builtin/Jolt 使用。
2. trigger pair key 只有 `trigger_entity + other_entity`；BTreeMap 保证稳定迭代，双 sensor 会产生双向 pair。
3. current/previous diff 生成 Enter/Stay/Exit，Stay 每 step 一次，Exit 复用 previous point。
4. `PhysicsTriggerEvent` 只有 world/kind/trigger/other/point，没有 step/sequence/pair generation、shape index、normal、reason 或 filter trace。
5. LevelSystem 提供 immutable `Arc<[PhysicsTriggerEvent]>` snapshot，是可保留的底座；生产代码没有 Damage/Checkpoint/Streaming executor。
6. 没有 overlap reference count、current overlap snapshot、pair lifetime、bounded event journal 或 authority-bound gameplay dispatch。

### 2.4 Sound Environment 事实

1. AudioVolume descriptor 有 Sphere/Box、priority、gain、low-pass、reverb、convolution、crossfade 与有限值 validation。
2. Box 只保存 world center/extents，没有 rotation/local transform；influence 以 source position 的 AABB distance 计算。
3. manager 以 HashMap 存取 volume，按 priority/weight/stable ID 选一个 strongest，没有 Add/Blend/Override/Min/Max 组合语义。
4. selected volume 能执行 gain、low-pass、convolution send，Sound DSP 路径是真实实现。
5. production 搜索未发现 Scene/entity system 调用 `update_volume()`/`remove_volume()`；没有 world revision/generation/lease。
6. Sound editor drawer 的 Space 占位和字符串 Apply/Set Shape/Set IR route 没有 Scene command factory、document、undo/save 或 runtime bridge。

### 2.5 Navigation 与 Gameplay 事实

1. NavMeshModifierDescriptor 支持 Add/Modify/Remove、agent filter、children inheritance、area override、link generation override。
2. area volume 只选 Empty + override_area 节点；center 取 world translation，half extents 取 abs scale * 0.5，rotation 被忽略。
3. contains 只按待处理 source node world position 分类，不做 triangle clipping、centroid coverage 或跨边界 split。
4. 多 area 命中按 collection 第一项，没有 priority/conflict receipt；复杂 mesh 可整批误标或漏标。
5. production 没有 DamageZone、CheckpointVolume、StreamingGate、GameplayVolume 或 typed effect/authority/cooldown consumer。
6. Physics event 不等于 gameplay behavior；没有 authority、lease、condition、cooldown、effect receipt、save/replay 或 network replication contract。
7. Streaming/World Partition 没有从空间区域得到 cell lease、prefetch、budget、cancel、generation 或 rollback 的产品桥。

### 2.6 Editor Workspace 事实

1. Volume workspace 固定列出 Damage/Reverb/Checkpoint/Streaming profile，固定显示 bounds、Pawn capsule、25 DPS、24 volumes/12 overlaps/1 warning。
2. Post Process workspace 只操作模板字段；Inspect/Validate 返回固定 queued 文本，没有 document revision、job ID、world generation 或 receipt。
3. field edit/commit 是 template binding/navigation action，不是 Scene transaction/property write/save acknowledgment。
4. workspace 既不是 PostProcess Inspector，也不是 SoundVolume/NavModifier/Physics trigger 的真实 projection。
5. callback bridge 与 template binding 数量会增加 scope，但没有 domain provider、factory、runtime snapshot、gizmo、overlap stream 或 diagnostics owner。

## 3. P0：必须先关闭的断路（5 项，全部 Open）

### P0-1：共享 Region identity/index 缺失

现有 Post Process、Physics、Sound、Navigation 各自保存一套空间事实，没有 stable region/component identity、compiled geometry artifact、world generation、dirty index 或 immutable query snapshot。必须先建立 shared geometry boundary，再让 domain adapter 保留各自语义。

### P0-2：Post Process 与 Scene/Editor contract 断裂

local volume 依赖 Collider 且只支持 Box/Sphere，unsupported shape/缺 Collider 会静默失效；Editor 没有 component property transaction/toolkit。必须在 authoring/cook 边界早拒绝或生成明确 diagnostic，不能继续由 render extract 晚期丢弃。

### P0-3：Sound volume 没有 Scene/runtime lifecycle bridge

Sound descriptor 和 strongest resolver 有真实 DSP，但没有 Scene entity create/update/remove、world revision、listener/source state、组合 policy 或 Editor owner。必须先接通 generation-qualified lifecycle，不能只扩展 SoundVolume 字段。

### P0-4：Trigger event 不能驱动 gameplay/navigation/streaming 产品

Physics event 缺 pair/shape/sequence/filter/current-overlap 事实，且无 Damage/Checkpoint/Streaming executor；Navigation area 又只做 AABB point classification。必须先建立 authoritative event/geometry contracts，禁止从固定 queued workspace 宣称区域玩法可用。

### P0-5：通用 Volume workspace 是第二 authority

固定 VOL_*、DPS、overlap、warning 与 queued feedback 不读取真实 Scene、PostProcess、Sound、Physics、Navigation、Streaming 状态。必须删除或改为真实 domain-aware toolkit，所有操作经 document transaction -> compiler/runtime receipt。

## 4. P1：Runtime、空间索引、域适配与 Editor（60 项，全部 Open）

1. 定义 stable `SpatialRegionId`、component lineage、scene/world generation 与 instance lease。
2. 定义 `RegionShapeSource`，支持 Box/Sphere/Capsule/Convex/mesh/composite/asset reference。
3. 统一 local/world transform、non-uniform scale、bounds、handedness 与 finite validation。
4. 编译 `CompiledRegionGeometry`，记录 exact shape、bounds、artifact key、query capabilities。
5. 建立 source revision、unknown-field preservation、migration 与 deterministic artifact digest。
6. 建立 shared broadphase/spatial index、dirty update、cell partition、query snapshot。
7. 支持 indexed point/overlap/raycast/nearest/containment query，并返回 stable ordering。
8. 将 domain mask/filter/team/layer/tag/agent/user metadata typed 化。
9. 为 every region 记录 source/component/artifact/world/plugin ownership 与 diagnostics。
10. 让 Scene load/save/prefab/instance/streaming 保留 region identity 和 source recipe。
11. Post Process local volume 支持 shape capability、missing collider diagnostics 与 typed fallback。
12. Post Process evaluator 支持 stack cache、dirty priority、camera/viewport generation。
13. Post Process blend distance/weight/priority/override policy 写入 compiled artifact。
14. Post Process provider 通过 Scene property access、transaction、undo/save/reimport 闭环。
15. Post Process unsupported shape 在 cook/admission 早拒绝而非 extract 空结果。
16. Physics pair identity 增加 shape/subshape、pair generation、step/sequence、normal/reason。
17. Physics event 记录 filter decision、current overlap snapshot、enter/exit cause、world generation。
18. Physics trigger dispatch 使用 bounded journal、authority/lease、replay/network receipt。
19. 提供 entity destroyed/teleport/filter change/shape replace 的 pair lifecycle 语义。
20. Sound volume 增加 local transform/rotation/listener/source state 与 stable revision。
21. Sound volume 支持 Add/Blend/Override/Min/Max/strongest 的 typed combination policy。
22. 接通 Scene -> Sound manager create/update/remove、generation、unload、rollback。
23. Sound spatial evaluator 以 listener/source/portal/acoustic context 计算影响，不只看 source point。
24. Sound reverb/convolution send 有 IR asset、latency、tail、budget、device capability receipt。
25. Sound editor toolkit 支持 descriptor schema、gizmo、audition、transaction、save/reimport。
26. Navigation modifier compiler 记录 source revision、agent/filter、inheritance provenance。
27. Navigation area 使用 exact shape/rotation 或显式 approximate policy，并输出误差诊断。
28. Navigation bake 支持 triangle clipping/centroid policy、overlap priority、conflict receipt。
29. Navigation area artifact 进入 tile/cell build、incremental invalidation、rollback、progress。
30. Navigation editor drawer 绑定真实 document、selection、bake job、preview geometry、error。
31. Gameplay Region contract 定义 condition、authority、cooldown、effect、enter/exit/overlap policy。
32. Damage/Checkpoint/StreamingGate 等 typed consumers 有 runtime factory、save/load、network/replay。
33. Gameplay effects 与 physics event 绑定 pair/sequence/generation，拒绝 stale event。
34. Streaming region 绑定 cell lease、prefetch/cancel、memory/IO budget、generation receipt。
35. World partition region 支持 server/client authority、interest、handoff、rollback。
36. Region runtime lifecycle 处理 spawn/despawn/teleport/disable/enable/hot reload/device loss。
37. 建立 region contributor/provider SPI，缺 owner/factory/service 时 fail-close。
38. 统一 diagnostics code、source span、shape、domain、generation、remediation。
39. Editor AssetType/ResourceKind 为 PostProcess/SoundVolume/NavModifier/GameplayRegion 分型。
40. 建立 source document、revision、dirty/save/autosave/recovery/conflict/undo transaction。
41. schema-driven inspector 显示 domain defaults、resolved values、unsupported capability 与 runtime state。
42. 真实 gizmo/picking/shape edit 支持 local/world transform、snap、multi-select、undo。
43. domain-aware workspace 读取真实 Scene/World snapshot，不再固定 VOL_* 事实。
44. Inspect/Validate/Compile/Bake/Playtest/Apply 都产生 job ID、generation、receipt、diagnostics。
45. overlap/stack/acoustic/nav debug overlay 消费真实 query snapshot 和 contributor traces。
46. Post Process preview 与 camera/viewport frame generation 对齐，避免旧 stack 投影。
47. Sound audition/recorded impulse/volume influence preview 连接真实 Sound provider。
48. Physics trigger preview 显示 pair/shape/filter/sequence/current-overlap，不伪造 hit count。
49. Navigation preview 显示 exact affected triangles/tiles、agent filter、conflicts、bake provenance。
50. Gameplay/Streaming preview 绑定 authority/PIE/network/save state，禁止 control-local success。
51. first-party runtime/editor catalog/App target 显式装配每个 domain provider/factory/toolkit。
52. plugin manifest capability、resource URI、operation/controller/service 做 admission closure。
53. background compile/bake/preview 使用 bounded scheduler、cancel、retry、shutdown drain。
54. artifact/cache 使用 source/recipe/tool/platform key、bulk/chunk manifest、atomic publication、GC。
55. malformed shape/NaN/overflow/unsupported transform/huge mesh/path input fuzz 与 budget rejection。
56. physics/Sound/PostProcess/Navigation cross-domain deterministic source/build/runtime golden。
57. scene/prefab/undo/redo/reimport/hot reload/stream unload roundtrip 不丢 region identity。
58. multi-world/multi-viewport/multi-listener/multi-agent/multi-client generation fence 与 isolation。
59. 1/1k/100k regions、large world cell、frequent dirty update、overlap storm、audio query、bake压力基准。
60. clean headless cook/package/client/server/editor release matrix 与 Unreal/Godot/Unity/Fyrox comparable methodology。

## 5. P2：长期能力（12 项，全部 Open）

1. GPU broadphase、multi-level spatial index、batched query 与 SIMD/parallel region evaluation。
2. SDF/voxel/mesh/composite region 与 exact distance/portal/acoustic propagation。
3. dynamic procedural regions、runtime painting、destruction、streamed geometry artifact。
4. cross-scene/global region registry、world partition federation 与 remote ownership。
5. advanced audio portals/occlusion/reverb zones、binaural/ambisonics、room graph。
6. post-process graph blending、LUT/volume stack caching、temporal history per region/camera。
7. nav dynamic obstacle/area updates、multi-agent masks、incremental GPU bake。
8. deterministic network replicated region state、prediction、rollback、replay and save migration。
9. editor multi-user region locks、field-level merge、presence、review annotations。
10. auto audit for overlapping conflicting domains, unsupported shapes, stale bridges, budget hotspots。
11. region artifact schema/algorithm migration、canary cook、old generation pin/rollback。
12. cross-engine spatial feature/performance/quality benchmark suite with public fixtures。

## 6. 分层重构顺序

### M0：Truthfulness 与第二 authority 清理

冻结通用 Volume stable capability；将固定 VOL_* workspace 和 unsupported shape 结果标为 fixture/unsupported；删除 Sound/Navigation descriptor-only admission，建立 domain owner inventory。

### M1：Shared Region Source/Geometry/Index

建立 stable region identity、typed shape source、transform/bounds、compiled geometry artifact、world generation、spatial index/query snapshot 和 diagnostics。

### M2：Post Process/Physics adapters

先闭合 local Post Process 与 Scene transaction、camera stack；再扩展 Physics pair/shape/sequence/current-overlap event 与 authority dispatch，保留 Builtin/Jolt 共享计算合同。

### M3：Sound/Navigation adapters

建立 Scene -> Sound lifecycle、listener/source combination、IR/reverb receipt；将 Navigation area 从 AABB point heuristic 收敛到 exact shape/triangle/tile policy 与 bake artifact。

### M4：Gameplay/Streaming consumers

建立 typed Damage/Checkpoint/StreamingGate/GameplayRegion documents、compiler、authority/effect/cooldown、cell lease/interest/budget/rollback。

### M5：Editor product/toolkits

为各 domain 装配 AssetType、document/transaction、inspector、gizmo、preview/debug、background jobs、catalog/App feature；workspace 只做真实 projection。

### M6：Fault/Scale/Release qualification

完成 malformed/fault/determinism/roundtrip/multi-world/large-world/cross-platform/headless package/benchmark 门禁；未通过前 capability 不得标 Stable。

## 7. 验收门禁（32 门，当前全部 Fail）

1. stable region/component/source/artifact/instance IDs 与 world/plugin generation 全程一致。
2. shape、transform、bounds、finite、unsupported capability 在 authoring/compile 边界早拒绝。
3. spatial index/query snapshot 更新有 dirty/revision/ordering/cancel 证明。
4. Post Process local/global stack、priority/blend/mask/camera、unsupported shape 通过 golden。
5. Physics Enter/Stay/Exit pair/shape/sequence/filter/current-overlap/exit reason 正确。
6. stale event、teleport、destroy、filter/shape replace、world unload 不影响新 generation。
7. Sound Scene lifecycle、listener/source influence、combination、IR/reverb、device loss 可恢复。
8. Navigation exact geometry、agent/filter、inheritance、overlap conflict、tile artifact 可重现。
9. Gameplay authority/effect/cooldown/replication/replay/save receipt 与 trigger sequence 绑定。
10. Streaming cell lease/prefetch/cancel/budget/interest/rollback 与 region generation 一致。
11. domain contributor/provider/factory/service/catalog/App target admission 闭合。
12. source/recipe/tool/platform/algorithm key、artifact manifest、atomic publication/GC/rollback 正确。
13. Scene/prefab/instance/stream save/load、unknown fields、migration、undo/redo 保持 identity。
14. Inspector/gizmo/multi-select/picking/transaction/dirty/save/cancel 对真实 document 生效。
15. Compile/Bake/Preview/Playtest/Apply 显示 job/generation/receipt/diagnostics，而不是固定 queued。
16. PostProcess/Sound/Physics/Nav/Gameplay/Streaming debug overlay 消费真实 snapshot/provider traces。
17. malformed/NaN/overflow/huge shape/mesh/path fuzz 无 panic/OOM/无界分配。
18. physics/Sound/PostProcess/Nav/Gameplay cross-domain deterministic golden 与 failure injection 通过。
19. multi-world/viewport/listener/agent/client isolation 与 generation fence 无串数据。
20. 1/1k/100k regions、overlap storm、dirty/bake/query、stream budget p50/p95/p99 达标。
21. Editor disabled client/server runtime 不依赖 UI fixture 或 Editor cache。
22. default/editor/client/server/plugin combination 的 capability manifest 与实际 executable provider 一致。
23. save/autosave/recovery/conflict/undo/reimport/hot reload/cancel 不覆盖未提交 region。
24. source external change、provider failure、job panic、disk full、device loss 保留旧完整 generation。
25. Post Process image/temporal history、Sound audio output、Navigation bake、Physics contact 的 visual/audio/data golden 完整。
26. runtime diagnostics 可按 region/domain/entity/world/generation/filter/receipt 筛选导出。
27. first-party workspaces 移除 fixed business facts，所有 fields 都能回读真实 state。
28. unsupported domains/shapes/filters 显示稳定 code/remediation，禁止静默 fallback。
29. package/cook manifest 含实际 geometry/domain artifacts、dependency/provenance 和 platform support。
30. cross-platform float/handedness/shape query/physics backend/nav bake 结果 deterministic。
31. release/migration 支持 schema/algorithm/tool upgrade、canary、old generation pin/rollback。
32. Stable/Complete 只能由 compile、registration、runtime、Editor、fault、platform、scale evidence 派生。

## 8. 禁止的临时修补

1. 禁止再增加万能 Volume property bag，或把 Post Process、Audio、Physics、Navigation、Gameplay、Streaming 效果放进同一字段表。
2. 禁止把 `PhysicsTriggerEvent` 的实体/point 快照直接命名为 Damage/Checkpoint/Streaming 产品。
3. 禁止用 Empty 节点 AABB/point heuristic 替代 Navigation exact shape/triangle/tile policy。
4. 禁止用 Sound HashMap descriptor、source position 或 strongest volume 掩盖 Scene/listener lifecycle 缺失。
5. 禁止 unsupported Post Process shape 晚期返回空 extract 而不产生作者可见诊断。
6. 禁止固定 VOL_*、DPS、overlap、warning、queued 文本代替真实 Scene/runtime/editor state。
7. 禁止只补 ZUI Space、route 或 catalog descriptor 而没有 factory/controller/provider/receipt。
8. 禁止在 render/audio/physics thread 同步执行 cook、bake、文件 I/O 或大范围 query。
9. 禁止以 test attribute、手工截图或一个 happy-path physics test 宣称跨域区域系统完成。
10. 禁止在重新导出 261-file manifest/fingerprint 前实施本报告假设，或通过 lockfile 漂移绕过 `--locked`。

## 9. 本轮产出边界

本轮只新增 Editor111 review、索引与分层计划，没有修改 Runtime、Editor、Interface、Plugin、App 或 tests production code，也没有运行 Cargo、physics/audio/post-process/navigation 动态验证；未查询或实时跟踪协调器。实施必须从 M0 开始，先恢复编译基线并建立 shared region/source/provider inventory，再做任何 Volume workspace UI 扩展。
