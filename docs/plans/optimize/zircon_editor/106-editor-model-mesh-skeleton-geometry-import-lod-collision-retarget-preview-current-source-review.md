---
title: Editor Model、Mesh、Skeleton、Geometry Import、LOD、Collision、Retarget 与 Preview 当前源码复核
category: zircon_editor
report_id: Editor106
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor32
refreshes:
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
related_code:
  - zircon_editor/src/core/asset/type_registry/builtin.rs
  - zircon_editor/src/core/editing/paths.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh/generate_preview_artifact.rs
  - zircon_editor/src/ui/retained_host/app/assets/workspace.rs
  - zircon_editor/assets/ui/editor/asset_browser.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_retarget_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation/workbench_extension_collision_proxy_workspace.zui
  - zircon_runtime/src/asset/assets/model
  - zircon_runtime/src/asset/assets/mesh
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_runtime/src/core/framework/animation/asset/skeleton.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs
  - zircon_runtime/src/scene/components/scene/mesh_renderer.rs
  - zircon_plugins/gltf_importer/runtime
  - zircon_plugins/asset_importers/model/runtime
  - zircon_plugins/virtual_geometry/editor
  - zircon_app/Cargo.toml
tests:
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets/texture_variant_tests.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets/material.rs
  - zircon_plugins/gltf_importer/runtime/src/tests/hotpaths.rs
  - zircon_plugins/gltf_importer/runtime/src/tests/index_admission.rs
  - zircon_plugins/asset_importers/model/runtime/src/tests/importers.rs
  - zircon_plugins/asset_importers/model/runtime/src/tests/registration.rs
  - zircon_plugins/virtual_geometry/editor/src/tests.rs
  - zircon_editor/src/tests/editing/import.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/18-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/StaticMeshEditor/Private/StaticMeshEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/StaticMeshEditor/Private/StaticMeshEditorActions.cpp
  - dev/UnrealEngine/Engine/Source/Editor/StaticMeshEditor/Private/SStaticMeshEditorViewport.cpp
  - dev/UnrealEngine/Engine/Source/Editor/SkeletalMeshEditor/Private/SkeletalMeshEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private/AnimationEditorPreviewScene.cpp
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Pipelines/Public/InterchangeGenericAssetsPipeline.h
  - dev/UnrealEngine/Engine/Plugins/Animation/IKRig/Source/IKRig/Public/Retargeter/IKRetargeter.h
  - dev/godot/editor/import/3d/resource_importer_scene.cpp
  - dev/godot/scene/resources/3d/importer_mesh.cpp
  - dev/godot/editor/scene/3d/mesh_editor_plugin.cpp
  - dev/godot/editor/scene/3d/skeleton_3d_editor_plugin.cpp
  - dev/godot/editor/scene/3d/bone_map_editor_plugin.cpp
  - dev/godot/scene/3d/retarget_modifier_3d.cpp
  - dev/Fyrox/fyrox-impl/src/resource/gltf/mod.rs
  - dev/Fyrox/fyrox-impl/src/resource/gltf/surface.rs
  - dev/Fyrox/fyrox-impl/src/resource/fbx/mod.rs
  - dev/bevy/crates/bevy_gltf/src/loader/mod.rs
  - dev/bevy/crates/bevy_mesh/src/mesh.rs
  - dev/bevy/crates/bevy_mesh/src/skinning.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/LODGroupDataSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/SceneProcessors/LODGroupProcessor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/ShaderLibrary/LODCrossFade.hlsl
doc_type: current_source_refresh
review_status: complete
implementation_status: pending
source_recheck_required: true
finding_status:
  p0: 5 open
  p1: 60 open
  p2: 12 open
gate_status:
  fail: 32
  partial: 0
  pass: 0
---

# Editor32/106 · Model、Mesh、Skeleton、Geometry Import、LOD、Collision、Retarget 与 Preview 当前源码复核

## 1. 结论

Zircon 的几何 Runtime 有可保留底座：`MeshAsset` 支持 typed attributes、16/32 位 index、usage、morph、skin metadata、Mesh SDF 与 Virtual Geometry；normal/tangent、meshopt、quantization、WebP、PBR/texture transform、labeled mesh/material/texture/scene/skeleton/clip subasset、CPU/GPU skinning、morph payload、previous palette 和局部 LOD 选择都是真实代码。不能把这些底座误报成 Editor 产品完成。

Editor 的 Quick Import 只有路径 TextField 和 callback 定义，没有 file picker/drop/paste 的生产事件调用者；默认路径为空，标准按钮在 canonical path 校验处失败。即使外部注入 OBJ/glTF/GLB，流程也直接 import 并把一个默认材质 Model 节点写进当前 World，没有 import recipe、source preview、selection tree、material mapping、scene hierarchy、atomic publish、reimport diff 或 rollback。

核心 glTF importer 与 split `gltf_importer` provider 的 schema/priority/animation/skin 语义不同，profile/availability 会改变结果。Core 将几何放入 Mesh subasset 后，Model root 只留 Mesh reference；`ModelAsset::overview()`、`render_mesh_descriptors()`仍读 inline payload，可把有几何模型报告成 0 vertices、0 indices、空 bounds。`ModelPrimitiveAsset` 没有 exactly-one inline/reference authority。

glTF inverse bind matrices进入 `MeshSkinAsset` 却没有进入生产 skinning palette；Renderer 由 `AnimationSkeletonAsset` local reference pose 重新 compose bind world。Skeleton 以 bone name 绑定，缺 stable bone id、duplicate-name policy、parent/cycle/finite validation、socket、BoneMap、SkeletonProfile、retarget asset；Importer Scene 又把 skeleton/player 固定为 `None`。同 mesh 多 skin 只保留首个 node skin。

LOD 只是 Scene instance distance-to-origin thresholds，Inspector 只读，没 reduction/cook、screen size、hysteresis、crossfade、streaming residency、platform override 或 asset-level group。PhysicsMesh 仅有 payload，没有 render-to-collision converter、primitive/convex cook、backend qualification；Collision Proxy 与 Retarget workbench 只有固定字段/反馈，没有 artifact 或 Runtime consumer。Model/Mesh/Skeleton 只有 catalog presentation，thumbnail/Virtual Geometry toolkit 也不闭合。

目标是唯一链：`GeometryImportSource + versioned ImportRecipe -> normalized GeometryScene -> Model/Mesh/Skeleton/Skin source assets -> derived LOD/Collision/VG/SDF artifacts -> atomic publication receipt -> dedicated toolkit/preview -> explicit scene instantiation`。Core 与 plugin importer 只能有一个语义 authority。

## 2. 当前物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 指纹与说明 |
|---|---:|---|
| Zircon Editor/Runtime/Plugin selected | **64 / 12,306 / 11,230 / 454,733 / 57 / 0** | 当前 model/mesh/importer/animation/skinning/LOD/editor/workspace/plugin 选择；fingerprint `65fb2c4f41f77ca431efea65d36f84aa054223dd34ffb8960e02dc0d467aac4f` |
| Unreal/Godot/Fyrox/Bevy/Unity reference | **22 / 28,133 / 24,721 / 1,089,338 / 32 / 0** | Static/Skeletal/Persona/Interchange/IKRetarget、Godot import/editor/retarget、Fyrox/Bevy mesh、Unity GPU LOD/crossfade；fingerprint `a295bd17d0bd5bae3c9639f050979c81d950cb6dcc6af64261e7a925cdfa2e4b` |
| Zircon selected union | **86 / 40,439 / 35,951 / 1,544,071 / 89 / 0** | 两组路径不重叠；fingerprint `86ac071f8234c2b9bb21650de46d355b107dcd3a8231006a7d74a16a2e65f5dc` |

实施前需重算 manifest；当前 shared dirty worktree 中核心 `import_gltf.rs`、`import_mesh.rs`、`model_mesh_subassets.rs`、`primitive_from_indexed_mesh.rs`、`zircon_app/Cargo.toml` 可能由其他工作流在途，本报告不把它们视为本轮修改。

逐层事实：

1. Asset Browser import TextField 没有 `events`；`invoke_mesh_import_path_edited` 只有定义/安装，没有 production caller。Quick Import 只接受 OBJ/glTF/GLB，STL/PLY/DXF provider 没有标准入口。
2. Quick Import 直接修改当前 authoring World，导入、发布、Scene instantiate 共用一个操作；取消、冲突、覆盖、reimport diff、source provenance 和 failure cleanup 缺失。
3. glTF importer 额外解析第一个 skin 并写 sibling skeleton/clip，和 Core labeled subasset 重复；Scene source 不安装 skeleton/player、primitive material mapping 或 source hierarchy 的完整 runtime binding。
4. `MeshSkinAsset` 只有 inverse bind matrices；同 mesh 多 skin 按首个 node skin 选择。Renderer `skinning.rs` 使用 skeleton local reference pose 与按 name 查找的 animation pose，没有消费 IBM。
5. Skeleton decode 没有 stable bone identity、parent range/order/cycle、duplicate name、finite TRS、socket、retarget signature 或 compatibility validation。缺失 pose bone 静默回退 bind transform。
6. `MeshAsset::validate` 校验 topology/index/attribute/morph 长度，但未覆盖 finite、weights normalization/range、joint bounds、degenerate triangle、duplicate morph identity、skin/IBM count。
7. Model primitive 同时允许 inline 与 Mesh reference；root overview/descriptor 只读 inline，resource streamer/catalog 可能发布空统计。没有 resolved overview artifact。
8. OBJ importer 丢弃 `tobj` material 结果；split OBJ/Model plugin 也没有 Material/Texture subasset/scene binding 的完整事务。
9. Normal/tangent 缺失时有生成/固定 fallback，但 provenance、算法版本、source-authored 与 derived policy 不进入 recipe/artifact。
10. Scene LOD 只按 entity transform origin 距离选阈值；无 bounds/projected size/hysteresis/crossfade/dither/forced/platform/streaming policy，Inspector 列表只读且 importer 写空。
11. PhysicsMesh payload 与 Mesh SDF/VG derived data 没有统一 cook key；内建 physics backend 对 TriangleMesh/HeightField/Compound 的能力没有进入 authoring preflight。
12. Collision Proxy/Retarget workbench 的按钮/文本最终只返回固定反馈或导航；没有 CollisionSetup/Retargeter/BoneMap/Solver asset、job、artifact、preview diff 或 runtime install。
13. Model/Mesh/Skeleton 没有 dedicated toolkit、preview scene、orbit camera、wireframe/UV/normals/bounds/LOD/collision/bone/morph inspection；thumbnail 走 placeholder。
14. Virtual Geometry Editor 声明的 `plugins://virtual_geometry/editor/authoring.zui` 在包中不存在；descriptor/command 测试不能证明模板可加载。
15. Core glTF provider priority 10，split provider priority 120；availability 组合会改变 glTF schema、animation placeholder、skin/IBM 与材质/attribute结果，必须收敛成单一语义 authority。

## 3. 参考引擎对照

- Unreal StaticMeshEditor、SkeletalMeshEditor、Persona、Interchange Pipeline 与 IKRig 把 import source/recipe、reimport/conflict、preview scene、LOD/collision/physics asset、skeleton/retarget asset、derived artifact 分层；Quick Import 不能承担这些边界。
- Godot `ResourceImporterScene`/`ImporterMesh` 与 Mesh/Skeleton/BoneMap/Retarget plugins 证明 material/surface/blend-shape/LOD/collision/post-import 都需要正式资源和 editor transaction。
- Fyrox glTF/FBX importer 提供 ModelImportOptions、material search、linked graph、animation/skin/bone surface assignment 与 normal/tangent policy；Bevy 只作 typed Mesh/skinning/glTF runtime 参照，不是 Editor 标杆。
- Unity Graphics 的 GPU-driven LOD data processor 与 URP crossfade 说明 Runtime 需要 screen-size/group/crossfade 数据；本地参考不含 Unity Model Importer，不能扩展为伪证据。

## 4. P0：必须先关闭的正确性与产品断路

| ID | 差异 | 必须重构 |
|---|---|---|
| P0-1 | Quick Import 可见输入面无生产事件，默认路径为空 | file/drop/paste 统一 SourceRequest、picker、admission、job、receipt |
| P0-2 | Core/split glTF provider 语义分裂且高 priority 可覆盖 | profile capability preflight、唯一 importer authority、旧 provider hard-cutover |
| P0-3 | Skin/Skeleton/IBM 错位且 Scene 不安装绑定 | 独立 Skin asset、stable joint map、IBM/reference-pose contract、typed Scene binding |
| P0-4 | Model 引用 Mesh 后 overview/descriptor 读空 inline | resolved Model artifact/Reference resolver，禁止 inline/reference 双 authority |
| P0-5 | 无 Geometry Toolkit，LOD/Collision/Retarget 只有 fixture | Model/Mesh/Skeleton/Collision/Retarget toolkits、preview scene、derived job 与 runtime receipt |

## 5. P1：Import、Geometry、Skin、LOD、Toolkit 逐项差异

| ID | 差异 | ID | 差异 |
|---|---|---|---|
| P1-01 | source 无 stable identity | P1-02 | ImportRecipe 无 version/schema |
| P1-03 | picker/drop/paste 不统一 | P1-04 | sidecar staging/rollback 缺失 |
| P1-05 | target naming/override policy 缺失 | P1-06 | import 与 scene instantiate 耦合 |
| P1-07 | import preview/selection tree 缺失 | P1-08 | structured import diagnostics 缺失 |
| P1-09 | reimport 无三方 diff | P1-10 | atomic publication 缺失 |
| P1-11 | provider/profile 无 preflight | P1-12 | batch/headless import 无统一合同 |
| P1-13 | Model primitive authority 含混 | P1-14 | node/mesh/primitive 无 stable identity |
| P1-15 | material slot/section identity 缺失 | P1-16 | overview 无 resolver/qualification |
| P1-17 | finite/range validation 不完整 | P1-18 | degenerate geometry diagnostics 缺失 |
| P1-19 | normal/tangent provenance 缺失 | P1-20 | VG ordinal 复用 joint slot |
| P1-21 | morph identity/range 不完整 | P1-22 | usage/build/runtime 策略未闭合 |
| P1-23 | OBJ material 链丢失 | P1-24 | format parity 无 golden corpus |
| P1-25 | bone 无 stable identity | P1-26 | skeleton parent/cycle/finite validation 缺失 |
| P1-27 | Skin 必须独立 typed asset | P1-28 | 同 Mesh 多 Skin 不可表达 |
| P1-29 | IBM/reference pose authority 未冻结 | P1-30 | animation channel 不能靠 name |
| P1-31 | glTF Scene 未安装 Skeleton/player | P1-32 | Editor sibling derivation 重复 importer |
| P1-33 | socket/attachment 缺失 | P1-34 | BoneMap/SkeletonProfile 缺失 |
| P1-35 | Retarget asset/solver/compiler 缺失 | P1-36 | Retarget Preview/Apply 是固定反馈 |
| P1-37 | LOD 应是 asset-level typed group | P1-38 | origin-distance 选择不可靠 |
| P1-39 | LOD hysteresis/crossfade 缺失 | P1-40 | reduction/cook service 缺失 |
| P1-41 | custom LOD import/reimport 缺失 | P1-42 | LOD/streaming/VG 策略未统一 |
| P1-43 | Collision Setup schema 缺失 | P1-44 | render-to-PhysicsMesh converter 缺失 |
| P1-45 | primitive/convex decomposition 缺失 | P1-46 | physics backend qualification 缺失 |
| P1-47 | Mesh SDF 不能替代 Collision | P1-48 | derived key/invalidation 不完整 |
| P1-49 | Model Toolkit 缺失 | P1-50 | Mesh Toolkit 缺失 |
| P1-51 | Skeleton Toolkit 缺失 | P1-52 | shared preview scene 缺失 |
| P1-53 | thumbnail 全部 placeholder | P1-54 | preview 无 stale/qualification 状态 |
| P1-55 | Virtual Geometry template 缺失 | P1-56 | commands/menus/WhenClause 缺失 |
| P1-57 | asset health/observability 缺失 | P1-58 | fault/cancel/recovery matrix 缺失 |
| P1-59 | 大资产规模预算缺失 | P1-60 | cross-platform quality/performance qualification 缺失 |

## 6. P2 与 32 Gate

P2 全部 Open：USD variant/layer composition、CAD tessellation profile、geometry processing graph、non-destructive mesh editing、UV unwrap/packing/lightmap qualification、skin weight painting、mesh merge/instancing/HLOD、remote cook、runtime motion adaptation、geometry diff/review、import plugin SDK/conformance kit、跨引擎任务基准。

32 个 Gate 当前为 **32 Fail / 0 Partial / 0 Pass**。最低门禁必须证明：同一 Source/Recipe 在 Editor、batch、headless、Cook 与 Runtime 生成同一 artifact digest；reference/inline 不会产生空 overview；每个 skin 的 IBM/joint map 与 skeleton generation 可追溯；material/mesh/morph/LOD/collision/retarget 依赖在 reimport diff 中可解释；preview、thumbnail、scene instantiate、cancel、crash、backend unavailable 都有 receipt；大型 glTF/FBX/OBJ/CAD 语料的 wall-time/RSS/allocation/streaming/LOD/crossfade 与跨平台结果有冻结基准。

## 7. 分层重构顺序

1. **M0 importer authority**：冻结 Core/plugin profile，删除高 priority 覆盖完整实现的路径，provider capability 在 import 前明确失败。
2. **M1 source/recipe/staging**：建立 GeometryImportSource、versioned ImportRecipe、sidecar staging、target naming、selection tree、transactional reimport/diff。
3. **M2 normalized geometry/artifacts**：Model/Mesh/Material/Texture/Skeleton/Skin/Animation 使用 stable node/mesh/primitive/field ids；生成 resolved overview、bounds、dependency、artifact manifest。
4. **M3 skin/animation/retarget**：独立 Skin/IBM/joint map，skeleton validation、BoneMap/SkeletonProfile、Retargeter asset/solver/preview/apply，Scene 安装 typed binding。
5. **M4 LOD/collision/VG/SDF**：asset-level LOD group、screen-size/hysteresis/crossfade、reduction service、CollisionSetup/PhysicsMesh/convex cook、VG/SDF derived keys 与 backend qualification。
6. **M5 toolkits/preview**：Model/Mesh/Skeleton/Collision/Retarget/VG toolkit 共用 preview scene、camera、selection、diagnostics、thumbnail 和 stale/cancel state。
7. **M6 scale/qualification**：导入/reimport/cook/preview fault matrix、large corpus、multi-platform/backend、CPU/GPU skinning、streaming/LOD/crossfade、artifact corruption/rollback benchmark。

## 8. 禁止临时修补与验证边界

- 不得给 Quick Import TextField 增加一个 callback 就宣称 import product 完成；不得将 import 与 Scene 写入绑成不可回滚的单操作。
- 不得靠 provider priority、文件扩展名或默认首 skin 掩盖 Core/plugin/多 skin authority；不得把 generic Data、empty inline payload 或 placeholder preview 当 typed asset。
- 不得从 Skeleton bone name 猜 IBM、duplicate bone 或 retarget；不得把 Mesh SDF/VG 当 PhysicsMesh/CollisionSetup。
- 不得把 origin-distance threshold、固定 Retarget feedback、只读 LOD list、thumbnail placeholder 或 test attribute 当工程质量证明。
- 本轮只完成静态逐文件复核、参考引擎对照、分级差异与重构顺序，没有修改生产代码，也没有运行 Cargo、Importer corpus/fuzz、skinned render、LOD、collision cook、retarget solver、VG template、scale 或 cross-platform 动态测试。实施前必须重算当前 64 文件 manifest 与 provider/profile 组合。
