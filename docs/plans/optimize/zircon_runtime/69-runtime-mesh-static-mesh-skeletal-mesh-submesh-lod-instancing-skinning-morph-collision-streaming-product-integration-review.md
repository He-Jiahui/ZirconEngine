---
title: Runtime Mesh3D、Static Mesh、Skeletal Mesh、Submesh、LOD、Instancing、Skinning、Morph、Collision、Streaming 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime69
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_runtime/src/asset/assets/mesh
  - zircon_runtime/src/asset/assets/model
  - zircon_runtime/src/asset/assets/scene/mesh.rs
  - zircon_runtime/src/core/framework/render/mesh
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/physics/mesh_asset.rs
  - zircon_runtime/src/animation
  - zircon_runtime/src/scene/components/scene/mesh_renderer.rs
  - zircon_runtime/src/scene/components/scene/reflection/mesh_renderer.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render_visibility.rs
  - zircon_runtime/src/scene/world/project_io/mesh.rs
  - zircon_runtime/src/graphics/runtime_prepare_mesh_geometry_seed.rs
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_mesh.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_mesh_sdf.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
  - zircon_runtime/src/graphics/visibility/culling
  - zircon_plugins/physics/runtime/src/backend/jolt/mesh_shape.rs
  - zircon_plugins/animation/runtime/src/gpu_skinning
  - zircon_editor/src/ui/host/editor_asset_manager/manager
tests:
  - zircon_runtime/src/asset/tests/assets/mesh
  - zircon_runtime/src/asset/tests/assets/model.rs
  - zircon_runtime/src/asset/tests/load/mesh.rs
  - zircon_runtime/src/asset/tests/pipeline/manager/model_import.rs
  - zircon_runtime/src/scene/tests/asset_scene/mesh_bindings.rs
  - zircon_runtime/src/scene/tests/render_extract/direct_sections.rs
  - zircon_runtime/src/scene/tests/world_basics/render_extract.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/mesh_queue.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/gpu_scene.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/tests/mesh_gpu_scene.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/65-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/StaticMesh.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/StaticMeshComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/SkeletalMesh.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SkinnedMeshComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SkeletalMeshComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/StaticMeshResources.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/SkeletalRenderPublic.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/GPUSkinCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GPUSkinCache.cpp
  - dev/bevy/crates/bevy_mesh/src/mesh.rs
  - dev/bevy/crates/bevy_mesh/src/skinning.rs
  - dev/bevy/crates/bevy_mesh/src/morph.rs
  - dev/bevy/crates/bevy_pbr/src/render/skin.rs
  - dev/bevy/crates/bevy_pbr/src/render/morph.rs
  - dev/bevy/crates/bevy_pbr/src/render/mesh.rs
  - dev/Fyrox/fyrox-impl/src/scene/mesh/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/mesh/surface.rs
  - dev/Fyrox/fyrox-impl/src/scene/mesh/buffer.rs
  - dev/godot/scene/resources/mesh.h
  - dev/godot/scene/resources/mesh.cpp
  - dev/godot/scene/resources/3d/importer_mesh.cpp
  - dev/godot/scene/3d/mesh_instance_3d.h
  - dev/godot/scene/3d/mesh_instance_3d.cpp
  - dev/godot/scene/3d/skeleton_3d.h
  - dev/godot/scene/3d/skeleton_3d.cpp
  - dev/godot/servers/rendering/renderer_rd/storage_rd/mesh_storage.h
  - dev/godot/servers/rendering/renderer_rd/storage_rd/mesh_storage.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceAllocators.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/InstanceDataSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/LODGroupDataSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/CoreDataSystems/LODGroupDataSystem.Jobs.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Batching/InstanceCullingBatcher.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Culling/InstanceCuller.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/Culling/InstanceOcclusionCuller.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/SceneProcessors/LODGroupProcessor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/SceneProcessors/MeshRendererProcessor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/SceneProcessors/MeshRendererUpdateBatch.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 69 · Runtime Mesh3D、Static Mesh、Skeletal Mesh、Submesh、LOD、Instancing、Skinning、Morph、Collision、Streaming 与 Product Integration 工程化差距

## 1. 结论

Zircon的3D Mesh路径不是空壳。`MeshAsset`已经有typed vertex attributes、index/topology、morph targets、局部bounds、Mesh SDF与Virtual Geometry种子；Scene可以绑定Model、直接Mesh、per-primitive material、morph weights和距离LOD；Graphics存在真实GPU buffer、GPU Scene current/previous transform、skin/morph history、cached mesh commands、indirect workspace、multi-draw replay、HZB、velocity pass和真实PNG产品测试。这些应作为重构底座保留，不能因目标是Unreal级引擎而推倒重写。

但当前实现仍没有形成工程级`source -> compiled mesh artifact -> scene instance -> deformation instance -> prepared generation -> residency/submission receipt`闭环。资产层把vertex stream、skin inverse bind、morph、SDF和Virtual Geometry塞在一个可直接运行的DTO里，却没有section/material slot/LOD family、stable submesh identity、平台cook、独立bulk block与统一derived generation。Scene同时暴露Model、Mesh、primitive bindings和LOD替代资源，运行时再按分支猜哪个是真值；LOD只按节点translation到camera的欧氏距离切换，不使用projected bounds、screen coverage、hysteresis、quality、residency或cross-fade。

Renderer已经拥有真实GPU能力，但产品路径仍为每个pending draw注册`instance_count = 1`；multi-draw只是把多个单实例命令装进indirect参数，并不等于硬件instancing。Prepared Mesh的local bounds未进入当前CPU/GPU可见性真值。GPU skinning会先CPU skin全部顶点，direct morph也会先CPU morph再构建GPU payload；动态fallback可在draw build期间重新创建GPU mesh。morph delta按实例按帧扫描重建，skin palette以每draw固定256矩阵buffer提交，错误又常被`.ok()`或inline primitive fallback静默吞掉。

本轮新增1项P0：`MeshRenderer.material_property_overrides`、`tint`与`material_alpha_mode`已能被reflection修改并实际进入extract/render，但`SceneMeshInstanceAsset`与`scene/world/project_io/mesh.rs`完全不保存它们；项目场景保存、关闭、重开后会静默恢复默认材质覆盖、白色tint和Opaque。这是可见产品数据损坏，不是缺少高级功能。另登记48项P1、12项P2与48项资格门。真实bounds/instancing、同步冷加载、physics cook、inverse-bind/skeleton安装等既有硬阻断继续由Runtime09B、Runtime64/09D、Runtime08A、Runtime08C/Editor32唯一拥有，本篇只定义Mesh消费门，不重复累计P0。

## 2. 审查边界与物理冻结

### 2.1 Owner边界

| 领域 | Canonical owner | Runtime69责任 | 不得重复登记 |
|---|---|---|---|
| Model/Mesh/Skeleton import与authoring | Editor32 | 消费唯一compiled artifact、拒绝不兼容generation | importer UI、retarget/collision/VG authoring P0 |
| Animation pose/graph/skin binding | Runtime08C、Plugins13 | 消费qualified pose与skin generation | animation manager、graph、scene安装与provider接线P0 |
| Renderer/visibility/GPU Scene | Runtime09B | 定义Mesh instance/deformation/submission消费合同 | bounds空间错误、`instance_count=1`、early visibility P0 |
| Material/shader/pipeline | Runtime09C | stable section/material override与variant输入 | shader/PSO/material graph父问题 |
| Render asset streaming/residency | Runtime09D、Runtime64 | 定义Mesh LOD/section/bulk block demand与lease | render-thread同步I/O、clone ownership、GPU budget P0 |
| Stable identity | Runtime24 | stable section/skin/deformation generation消费 | 通用allocator、wrap、stale handle治理 |
| Scene persistence/hierarchy/bounds | Runtime61、Runtime62 | 本篇拥有新增Mesh字段丢失；消费canonical bounds | 通用Scene schema/clone/hierarchy/bounds父问题 |
| Physics mesh cook | Runtime08A | render Mesh与collision artifact generation关联 | TriangleMesh/HeightField无产品cook链P1 |
| Quality/LOD policy | Runtime65 | Mesh LOD提供可解析能力与effective receipt | 全局quality/device profile父问题 |

`zircon_runtime::scene`继续拥有runtime world和Mesh instance；`zircon_runtime::core::framework::render`只发布中立extract/phase DTO；`zircon_runtime::graphics`拥有prepared GPU generation与submission；`zircon_editor`拥有source/import/cook authoring。不得新增第四个root package，不得把Editor source DTO放进renderer，也不得让graphics私有格式成为Scene持久化权威。

### 2.2 Zircon物理冻结

本轮聚焦278个Zircon文件，共59,135行、2,166,523 bytes；按相对路径小写、排序去重，以`path|lowercase SHA-256`逐行LF连接且末尾无LF计算，指纹为`9afe296d977f277d02a7cf1e02e128e5dae5116e46c6df785ec4f47808463b4f`。入选范围含483个Rust test attribute与8个ignored attribute；数字覆盖大量unit/source-shape/cache/pipeline测试，不等于导入、重开、动画、像素、性能和故障产品资格。

冻结时4个入选路径dirty：`clip_event.rs`、`frame_extract.rs`、`build_mesh_draw_build_context.rs`与`scene/tests/asset_scene/mesh_bindings.rs`。结论绑定当前共享working copy；实施前必须重算指纹、精确零搜索和字段守恒矩阵。本轮不修改production/tests，不运行Cargo、GPU capture、Editor、fault、soak或benchmark，符合MVP gate下只做静态review的授权。

### 2.3 参考物理冻结

五类参考共37个文件、50,833行、2,084,265 bytes，指纹为`71498c935f4a9085312a35aa9b5173421874a661bba719c54fce27c9aaf9539a`。本篇把参考机制作为设计证据，不把任何单个引擎的类名或实现复制成Zircon架构。

| 参考 | 可采用证据 | 不可机械照搬 |
|---|---|---|
| Unreal | SourceModels/RenderData/BodySetup分层，LODResources/Section，screen-size/quality MinLOD，streaming，Nanite，GPUSkinCache per-entry/per-LOD/per-section与current/previous | UObject生命周期、宏反射、RHI线程模型 |
| Bevy | typed Mesh attributes与RenderAssetUsages，显式SkinnedMesh inverse-bind/joints，changed-only skin extract，共享current/previous morph/skin buffer与offset allocator | ECS组件拆分粒度与RenderWorld调度原样复制 |
| Fyrox | Mesh/Surface/SurfaceData分离，single-material surface，skinned bounds，明确static/dynamic batching限制，buffer layout/hash mutation guard | scene graph handle与资源管理模型原样复制 |
| Godot | ArrayMesh surface/material/LOD dictionary，cached triangle mesh/collision，custom AABB，importer simplification与shadow mesh，MeshInstance显式mesh/skeleton/skin/blend绑定 | RenderingServer RID与动态Variant API原样复制 |
| Unity Graphics | persistent instance handles/update batches，LODGroup screen-relative阈值、fade/force mask、world size，GPU-driven culling/occlusion | C# Jobs/Burst和Unity对象变更系统原样复制 |

## 3. 可保留的真实底座

### 3.1 Mesh数据与校验基础

`MeshAsset`的attribute map、format validation、index range、topology multiple、morph target结构、normal/tangent生成与文档roundtrip测试是真实基础。`try_render_mesh_descriptor`已经提供fallible入口，Mesh SDF和Virtual Geometry种子也有明确数据类型。重构应把这些升级为canonical compiler与artifact validator，而不是再造另一套vertex DTO。

### 3.2 Scene binding与提取基础

`MeshRendererPrimitiveBinding`能表达per-primitive mesh/material，Scene extract保持稳定节点排序、render/material queue、order、depth bias、morph weights、tint和material override。current component到extract的字段链说明运行时材质覆盖是真实能力，也直接证明P0不是“未实现功能”，而是持久化与运行时事实分裂。

### 3.3 GPU Scene、command cache与indirect底座

GPU Scene已有stable instance key、current/previous transform、skin palette、morph weights与skinned source history；mesh pass具有cached command、pipeline/material/geometry key、indirect workspace、multi-draw count replay和多phase processor。目标是在这些结构上建立真正的instance span、qualified bounds和prepared generation，而不是增加第二套GPU scene或把CPU draw列表改名为GPU-driven。

### 3.4 产品像素与velocity测试

`render_product_mesh_cache`覆盖direct morph、skinned velocity、virtual geometry、material passes与PNG输出，能阻止大量纯DTO伪完成。它们应扩展为真实导入资产和规模矩阵；现有测试不能删除，也不能被只检查stats/source字符串的测试替代。

## 4. 当前产品链逐层事实

| 层 | 当前事实 | 工程级缺口 |
|---|---|---|
| Source/Import | OBJ/glTF与Mesh/Model asset已有路径 | source recipe、section/LOD/skin/morph/collision派生物没有一个原子generation |
| Runtime asset | Mesh含attribute/index/morph/skin/SDF/VG，Model含primitive列表 | 缺stable section/material slot/LOD family/bulk block/cook platform合同 |
| Scene | Model、Mesh、primitive与LOD可同时填写 | 多几何authority、无显式rig/skin binding、LOD语义过浅、三字段保存丢失 |
| Extract | 每帧archetype scan、排序、展开primitive | 无change frontier/shared deformation handle，clone随primitive数增长 |
| Prepare | ensure时同步load、转换、bounds、GPU create | 关键帧I/O/decode/clone，无ticket/lease/last-good/fallback receipt |
| Skin/Morph | CPU与GPU路径、previous history均存在 | GPU路径预付CPU变形，delta/palette无共享arena，错误静默降级 |
| Visibility/Batch | frustum/HZB/indirect/multi-draw存在 | local bounds未消费、每draw一instance、visibility晚于重资源准备 |
| Collision | DTO与Jolt test registration存在 | 无产品cook/import/cache/material/subshape/streaming链 |
| Product evidence | 多类unit与PNG测试 | 无import-save-reopen-animate-render-collide闭环及规模/capture基线 |

## 5. 新增P0

### MESH3D-P0-001：Mesh材质覆盖、tint与alpha mode在Scene保存/重开中静默丢失

`MeshRenderer`公开并实际消费`material_property_overrides`、`tint`和`material_alpha_mode`：reflection能修改morph及相关render字段，`World::to_render_extract`把override clone进per-entity map，并把tint/alpha mode写入每个`RenderMeshSnapshot`和phase input。Graphics随后按这些值选择材质参数与phase。

但`SceneMeshInstanceAsset`只包含model/mesh/material、queue/order/depth、morph、primitives和lods；`mesh_from_asset`与`mesh_to_asset`也只复制这些字段。现有`scene_assets_roundtrip_primitive_mesh_material_bindings`验证queue、depth、morph、primitive和LOD，却没有给三项运行时可见字段建立守恒断言。因此用户调整实例tint、alpha mode或材质参数后，save成功、close、reopen会得到默认白色、Opaque和空override；UI与renderer在保存前后的事实不同且无诊断。

完成定义：扩展canonical Scene schema/version/migration与reader/writer，使三字段以及后续stable section override使用同一生成式字段守恒清单；legacy文档有明确default migration；unknown/invalid material property按typed policy处理；增加memory -> document -> disk -> reopen -> extract -> pixel roundtrip与Exit Play/Save链测试。Runtime61提供通用Scene schema/transaction机制，本项由Runtime69拥有具体Mesh字段闭环，关闭前Editor不得把这些字段标记为可持久保存。

## 6. 目标架构

| 组件 | 所属 | 责任 |
|---|---|---|
| `MeshSourceRecipe` | Editor32/asset pipeline | source URI、import settings、axis/unit、section/material/skeleton/morph/collision recipe |
| `MeshArtifactManifest` | Runtime asset | schema/platform/backend/version/hash、section/LOD/bulk block目录、dependency generation |
| `MeshGeometryArtifact` | Runtime asset | immutable vertex/index streams、bounds、section ranges、attribute layout、quantization/compression |
| `MeshDeformationArtifact` | Runtime asset/Animation adapter | skin joint remap/inverse bind、morph target table与immutable delta blocks |
| `MeshDerivedArtifactSet` | owner adapters | collision/nav/SDF/VG/shadow/RT data，以同一source generation关联但独立失败 |
| `MeshInstanceComponent` | Runtime Scene | artifact、stable section overrides、rig binding、LOD/render/residency policy与instance generation |
| `MeshSceneDelta` | Scene -> render framework | created/changed/removed、qualified bounds、deformation handle、view mask与revision |
| `PreparedMeshGeneration` | Graphics | device generation、resident LOD/section blocks、GPU allocations、pipeline-compatible layout |
| `MeshDeformationInstance` | Graphics | shared palette/morph arena offsets、current/previous generation、fallback reason |
| `MeshSubmissionReceipt` | Graphics/diagnostics | requested/resolved LOD、visibility、batch/instance span、fallback、bytes、CPU/GPU cost |

Artifact必须是唯一运行时geometry真值。Model可作为导入/组合目录，但不能同时保留“external Mesh引用失败则悄悄使用inline primitive”的第二真值。Scene只引用qualified artifact与stable section ID；Renderer不读取Editor source，不在draw build临时cook/clone完整Mesh，也不通过entity ID猜skeleton兼容性。

## 7. P1差距与重构定义

### 7.1 Asset、Section、LOD与Artifact P1-001 至 P1-010

| ID | 差距 | 重构与完成定义 |
|---|---|---|
| MESH3D-P1-001 | `MeshAsset`没有section/submesh/material slot/LOD family | 建立stable SectionId、slot table、per-LOD section range与跨reimport relocation；draw不再以vector ordinal承担持久身份 |
| MESH3D-P1-002 | `MeshSkinAsset`只有inverse-bind矩阵且注释承认是临时容器 | 建立skin binding artifact：skeleton signature、joint remap、inverse bind、influence format、bind-pose bounds与version |
| MESH3D-P1-003 | validation不拒绝NaN/Inf、非法weight、退化三角形或不兼容morph schema | compiler执行finite、normalization、index/topology、degenerate/winding、attribute semantic、budget与target-table校验 |
| MESH3D-P1-004 | `render_mesh_descriptor()`对invalid Mesh以空positions继续构造 | 产品只允许fallible validated artifact入口；空geometry是显式valid-empty或typed rejection，不能由`unwrap_or(&[])`猜测 |
| MESH3D-P1-005 | 无skin时joint-index channel被复用为VG ordinal | semantic不可重载；VG vertex/meshlet ordinal拥有独立typed stream与layout capability，skin/VG组合有负向测试 |
| MESH3D-P1-006 | Mesh到Model转换会为缺失attribute注入默认值 | 保存source presence/provenance与generated flag；shader/cook按capability决定生成或拒绝，不能让默认数据冒充导入事实 |
| MESH3D-P1-007 | Model同时持inline primitive与external Mesh引用 | hard cutover为artifact reference；若保留bootstrap fallback，必须版本绑定、显式policy与receipt，绝不静默择另一份geometry |
| MESH3D-P1-008 | reference-only Model overview按inline vertices计算为空 | Editor32修复authoring overview；runtime manifest直接携带validated aggregate bounds/section/LOD统计，不在UI或render临时解析bulk |
| MESH3D-P1-009 | SDF、VG、collision、shadow/RT派生物没有共同source generation | `MeshDerivedArtifactSet`以source/import/platform hash关联，单项失败可降级但不得混用旧新generation |
| MESH3D-P1-010 | `MeshAssetUsage`只有MainWorld/RenderWorld两个bool | 与Runtime09D定义metadata/bulk、CPU retention、GPU residency、collision pin、editor pin、streamable block和release policy |

### 7.2 Scene、Section Override与LOD P1-011 至 P1-018

| ID | 差距 | 重构与完成定义 |
|---|---|---|
| MESH3D-P1-011 | MeshRenderer同时暴露Model、Mesh、primitive bindings和LOD替代资源 | 编译为单一`ResolvedMeshInstance`；冲突组合在load/authoring失败，不能靠分支优先级选真值 |
| MESH3D-P1-012 | primitive override按ordinal而非stable section/material slot身份 | override引用SectionId/MaterialSlotId，reimport产生relocation/orphan report，未知section不误套到另一section |
| MESH3D-P1-013 | Scene没有显式skin/skeleton/rig binding | instance引用SkinArtifact+SkeletonInstance+joint-map generation，兼容性在admission验证，不按同entity存在组件猜测 |
| MESH3D-P1-014 | 缺cast/receive shadow、visibility/cull、motion vector、RT与skin-cache policy | 只增加经过renderer capability解析的typed policy；requested/effective进入receipt，unsupported fail-closed或明确fallback |
| MESH3D-P1-015 | LOD只比较camera与节点translation的距离 | 基于world bounds projected screen size、FOV/projection、scale、hysteresis、camera cut、quality bias和forced/min/max LOD |
| MESH3D-P1-016 | LOD是每实例重复的alternate Model/Mesh列表 | LOD family归入immutable artifact；instance只持policy/override，多个实例共享geometry与section topology |
| MESH3D-P1-017 | LOD level只校验正finite threshold，缺排序/覆盖/兼容性 | cook验证阈值单调、section/material/skin/morph兼容、bounds误差、triangle reduction和fallback chain |
| MESH3D-P1-018 | render LOD、collision LOD、nav、SDF/VG与residency彼此无合同 | source generation下分别选择effective LOD并记录reason；physics/nav不可因render eviction失去必需数据 |

### 7.3 Extract、Bounds、Visibility与Instancing P1-019 至 P1-026

| ID | 差距 | 重构与完成定义 |
|---|---|---|
| MESH3D-P1-019 | 每帧全Mesh archetype scan、收集并按node ID排序 | Scene发布created/changed/removed delta与stable retained snapshot，steady frame工作量随dirty frontier而非总实例数增长 |
| MESH3D-P1-020 | 每个primitive clone整份morph weights，override也clone进BTreeMap | deformation weights/override使用generation handle与packed shared storage；extract只复制小型qualified reference |
| MESH3D-P1-021 | draw build对每个Mesh线性搜索`animation_poses` | extract建立qualified entity/rig到pose slot index，O(1)解析且包含frame/skeleton/pose generation |
| MESH3D-P1-022 | stable instance key把primitive ordinal压入有限位宽且只debug assert | 由Runtime24提供checked SectionInstanceId；超宽、reimport、remove/reuse有typed结果和小位宽模型测试 |
| MESH3D-P1-023 | framework `static_batches`只被stats/tests读取 | 与Runtime09B统一唯一batch authority；删除死parallel plan或接入真实提交，不维持两个声称可执行的batch事实 |
| MESH3D-P1-024 | PreparedMesh local bounds没有进入CPU/GPU可见性 | 消费Runtime09B修复后的canonical local/world/deformed bounds ABI；CPU frustum、HZB、LOD、streaming使用同代数据 |
| MESH3D-P1-025 | 产品每pending draw注册一个instance，候选分组不产生instance span | Runtime09B建立state-compatible instance batches、persistent slots与GPU-generated/compacted args；64同state实例不产生64个one-instance primitive |
| MESH3D-P1-026 | visibility晚于asset/deformation/GPU Scene准备 | 先用last-known-good bounds做cheap per-view admission，再只为可见/预取集合准备昂贵资源；shadow/reflection等view显式合并需求 |

### 7.4 Skinning、Morph与Dynamic Geometry P1-027 至 P1-040

| ID | 差距 | 重构与完成定义 |
|---|---|---|
| MESH3D-P1-027 | deformation绑定主要以entity相等关联pose/skeleton/mesh | 以SkinBindingId、SkeletonInstanceId、joint-map revision、pose generation做qualified lookup，wrong-rig拒绝 |
| MESH3D-P1-028 | runtime palette使用skeleton local bind推导，未消费imported inverse-bind | Runtime08C/Editor32关闭唯一P0；Runtime69 gate要求shader/CPU reference都使用`joint_world * inverse_bind`同一artifact |
| MESH3D-P1-029 | 所谓GPU skin路径先clone并CPU skin全部vertices | 路径选择先于geometry work；GPU path只上传palette/dirty deformation input，CPU fallback仅在明确capability/预算策略下执行 |
| MESH3D-P1-030 | CPU morph/skin fallback可在draw build每帧创建新GpuMeshResource | dynamic geometry进入persistent/ring arena、dirty range upload与fence retirement；steady deformation不创建vertex/index buffer |
| MESH3D-P1-031 | palette固定256矩阵，`to_storage().ok()`静默吞错误 | capability报告influence/joint/palette limits；oversize在cook/admission失败或选择明确split/CPU fallback并输出receipt |
| MESH3D-P1-032 | current/previous palette按draw创建独立buffer/bind资源 | 参考Bevy/Unreal建立共享palette arena、offset allocator、changed-only upload、double buffer与回收/fragmentation预算 |
| MESH3D-P1-033 | palette compatibility signature只含skeleton ResourceId | key包含skeleton/skin artifact revision、joint map、layout、device generation；reload后旧history不可误配 |
| MESH3D-P1-034 | direct morph先CPU生成morphed primitive又构建GPU payload | 路径先决策；GPU morph直接使用prepared base geometry+resident delta，CPU结果不得被随后丢弃 |
| MESH3D-P1-035 | 每实例每帧扫描所有active target与vertex重建delta rows | delta成为MeshDeformationArtifact immutable bulk block，按mesh generation单次resident，多实例只上传weights/active set |
| MESH3D-P1-036 | morph delta、current/previous weights每帧独立buffer上传 | 建立共享delta arena与per-instance weight ring，changed-only range、frame history swap、capacity/overflow/fallback可观测 |
| MESH3D-P1-037 | morph binding只有weight vector ordinal，无name/target generation | target table有stable MorphTargetId/name/semantic，animation curve到target编译期解析，reimport提供relocation/orphan report |
| MESH3D-P1-038 | Model root inline primitive路径没有完整消费component morph | 所有Model/section/direct Mesh走同一resolved deformation contract；多section target correspondence有产品像素测试 |
| MESH3D-P1-039 | skeleton/load/primitive skin错误常退回静态或局部未skinned draw | character实例采用all-required或显式partial policy，错误输出per-section reason；不得静默混合bind pose与animated sections |
| MESH3D-P1-040 | deformed bounds没有随skin/morph generation更新 | cook保守per-joint/per-target bounds，runtime按quality选择union/refit/GPU reduction；LOD、culling、shadow与motion共享同代bounds |

### 7.5 Streaming、Collision、Diagnostics与Product P1-041 至 P1-048

| ID | 差距 | 重构与完成定义 |
|---|---|---|
| MESH3D-P1-041 | Mesh/Model ensure在render准备路径同步load、转换、bounds与GPU create | 消费Runtime64/09D non-blocking ticket与PreparedGeneration；stable render thread filesystem/decode/deep clone/GPU create为0 |
| MESH3D-P1-042 | referenced Mesh失败会静默回退inline primitive | fallback必须artifact-version-compatible、policy显式、last-good有lease并带reason；required asset失败阻止ready而非显示另一形状 |
| MESH3D-P1-043 | 每帧按visible resource轮询revision并重建dependency state | typed asset events驱动dirty closure、single-flight prepare与原子publish；稳定帧不随全部mesh dependency数量轮询 |
| MESH3D-P1-044 | Mesh没有LOD/section/page semantic streaming | Runtime09D artifact block目录覆盖LOD、vertex/index stream、morph、skin、VG/collision依赖；demand/priority/budget/evict有typed state |
| MESH3D-P1-045 | Jolt TriangleMesh只在测试注册DTO并逐triangle组装shape | Runtime08A建立platform/backend keyed cooked collision artifact、weld/degenerate/material/subshape map、cache/refcount与产品注册 |
| MESH3D-P1-046 | render、collision、nav、SDF、VG与RT派生物可跨代混用 | reimport事务以source generation验证完整dependency set；允许独立降级但每个consumer发布effective generation与last-good状态 |
| MESH3D-P1-047 | stats有queue/cache计数但没有per-instance fallback真值 | `MeshSubmissionReceipt`关联asset/section/skin/morph/view/LOD/residency/pipeline generation及culled/fallback/error/cost reason |
| MESH3D-P1-048 | 测试没有真实import-save-reopen-animate-render-collide与规模基准 | 建立glTF skin/morph/LOD fixture、Scene字段守恒、pixel/velocity/collision oracle、10K/100K instance与fault/reload/device-loss矩阵 |

## 8. P2延后项

| ID | 延后项 | 提升条件 |
|---|---|---|
| MESH3D-P2-001 | Nanite式cluster hierarchy与software/hardware raster融合 | 普通section/LOD/VG artifact、bounds、residency和GPU-driven submission先资格化 |
| MESH3D-P2-002 | Mesh shader/task shader管线 | RHI capability、fallback、pipeline cache与传统vertex path parity先完成 |
| MESH3D-P2-003 | compute skin cache、tangent recompute与multi-pass复用 | 正确inverse-bind、shared palette/delta arena、current/previous history和预算先完成 |
| MESH3D-P2-004 | 8/12 influences、variable influence与dual quaternion skinning | 4-weight线性skin的artifact、validator、CPU/GPU parity和真实角色基线先完成 |
| MESH3D-P2-005 | cloth/groom/physics deformation composition | Runtime31/32与本篇deformation graph、space/order/history合同稳定后 |
| MESH3D-P2-006 | runtime procedural/editable mesh与partial topology mutation | immutable artifact、dynamic vertex arena、transaction和collision recook policy先完成 |
| MESH3D-P2-007 | mesh page GPU feedback/direct storage/GPU decompression | semantic block、普通async I/O、upload queue、budget和fault recovery先完成 |
| MESH3D-P2-008 | ray tracing BLAS refit/rebuild与skinned RT geometry | Runtime28 RHI/AS owner和本篇deformation generation、bounds、residency先完成 |
| MESH3D-P2-009 | automatic runtime LOD generation与adaptive simplification | offline deterministic LOD cook、quality/crossfade与visual error oracle先完成 |
| MESH3D-P2-010 | impostor/HLOD/mesh merge跨World partition | stable section/material identity、partition cell与instance ownership先完成 |
| MESH3D-P2-011 | multi-GPU/UMA/mobile特化的mesh memory policy | 单device离散/UMA capability与bytes/fence/retention会计先完成 |
| MESH3D-P2-012 | 超大世界下deformed mesh double-precision/rebase优化 | Runtime23空间合同与render-relative current/previous pipeline先完成 |

## 9. 分层实施顺序

| 里程碑 | 内容 | 进入条件 | 退出证据 |
|---|---|---|---|
| M0 数据守恒与owner冻结 | 关闭P0，冻结Section/Skin/Morph/LOD identity与owner map | Runtime61 schema transaction可消费 | save/reopen/play/extract/pixel字段守恒，旧文档migration |
| M1 Canonical Mesh artifact | source recipe、manifest、geometry/deformation/derived目录、validator | Editor32 importer owner明确 | clean cook hash、坏mesh拒绝、reimport relocation、无双geometry真值 |
| M2 Scene instance与LOD | 单一ResolvedMeshInstance、stable override、rig binding、screen-space LOD | M1 artifact稳定，Runtime65 quality receipt可用 | multi-camera/FOV/scale/hysteresis/residency LOD tests |
| M3 Retained extract与bounds | change delta、qualified bounds、cheap visibility、prepared generation | Runtime62/09B bounds ABI关闭 | steady extract随dirty增长，CPU/GPU/LOD/streaming bounds parity |
| M4 GPU instancing与deformation | instance span、palette/morph arena、changed-only upload、persistent dynamic geometry | M2/M3 identity和generation稳定 | 64/10K实例、skin/morph velocity、steady allocation 0、fallback receipt |
| M5 Streaming与derived systems | LOD/section/bulk residency、collision cook、atomic derived generation | Runtime09D/64/08A adapters可用 | teleport/reload/OOM、collision/material/subshape、last-good tests |
| M6 Product资格 | reference project、Editor/Play/cook、capture、fault/soak/benchmark | 前述门全部通过 | 同内容同画质CPU/GPU/memory/visual基线与可重放evidence |

M4不得在M0-M3之前以“先做GPU优化”为由重写提交路径；没有stable section、rig、bounds和artifact generation时，batch key、history和residency都会再次失效。M5也不得让physics或navigation读取renderer私有GPU buffer；它们消费同源但独立生命周期的cooked artifact。

## 10. 验收门禁

### 10.1 Owner、Schema与数据守恒 G01-G08

| Gate | 验收标准 |
|---|---|
| MESH3D-G01 | `material_property_overrides`、`tint`、`material_alpha_mode`经memory/document/disk/reopen/extract/pixel完全守恒 |
| MESH3D-G02 | legacy Scene migration有固定fixture；unknown/invalid override不静默丢弃或误套 |
| MESH3D-G03 | Model/Mesh/primitive/LOD冲突组合在authoring/load阶段typed失败，不靠运行时优先级猜测 |
| MESH3D-G04 | SourceRecipe、Artifact、SceneInstance、PreparedGeneration、GpuAllocation五类identity不可混用且均有owner/generation |
| MESH3D-G05 | SectionId、MaterialSlotId、SkinBindingId、MorphTargetId重导入具relocation/orphan receipt |
| MESH3D-G06 | Runtime/Graphics不读取Editor source/cache，Editor不拥有runtime/GPU生命周期 |
| MESH3D-G07 | 所有schema有version、validator、migration/support window与deterministic hash |
| MESH3D-G08 | Runtime08A/08C/09B/09D/24/61/62/64/65及Editor32边界在manifest中唯一，无重复manager/format |

### 10.2 Artifact、Section、Skin与Morph G09-G16

| Gate | 验收标准 |
|---|---|
| MESH3D-G09 | Mesh compiler拒绝NaN/Inf、非法index/topology/weight、退化超限和不兼容attribute/morph schema |
| MESH3D-G10 | validated-empty与invalid-empty结果可区分，产品无`unwrap_or(&[])`伪ready路径 |
| MESH3D-G11 | section/material/LOD topology与bounds写入manifest，runtime不解析source bulk推导目录 |
| MESH3D-G12 | skin artifact包含skeleton signature、joint map、inverse bind、influence layout和bind bounds |
| MESH3D-G13 | morph artifact包含stable target table和immutable delta blocks，curve binding在compile解析 |
| MESH3D-G14 | VG ordinal与joint index使用独立semantic stream，组合layout跨backend验证 |
| MESH3D-G15 | SDF/VG/collision/nav/shadow/RT派生物均绑定source generation，混代被拒绝 |
| MESH3D-G16 | clean cook重复hash一致，platform/backend/import setting变化正确改变artifact key |

### 10.3 Scene、LOD、Extract与Bounds G17-G24

| Gate | 验收标准 |
|---|---|
| MESH3D-G17 | Scene instance显式绑定artifact、stable section overrides、skin/skeleton与policy，不按entity猜兼容性 |
| MESH3D-G18 | LOD按projected bounds/screen size、FOV、scale、quality与hysteresis选择，camera cut无抖动 |
| MESH3D-G19 | forced/min/max/bias/crossfade/residency fallback均有requested/effective receipt |
| MESH3D-G20 | LOD切换保持section/material/skin/morph兼容，错误artifact在cook或admission失败 |
| MESH3D-G21 | steady Scene extract工作量随dirty instance/section增长，不全World scan/sort/clone |
| MESH3D-G22 | morph weights与material overrides以generation handle共享，primitive展开不复制大数组/map |
| MESH3D-G23 | local/world/deformed bounds在CPU frustum、GPU HZB、LOD和streaming中空间/代际一致 |
| MESH3D-G24 | primitive/section identity超宽、remove/reuse/reimport均typed处理，无release位截断collision |

### 10.4 Visibility、Instancing、Residency与Submission G25-G32

| Gate | 验收标准 |
|---|---|
| MESH3D-G25 | 完全不可见且无prefetch需求的实例不执行material resolve、deformation、GPU allocation或draw build |
| MESH3D-G26 | 64同mesh/material实例形成共享geometry/state和instance span，不注册64个one-instance primitive |
| MESH3D-G27 | indirect args的instanceCount/baseInstance与compact结果一致，direct fallback原因可观测 |
| MESH3D-G28 | transparent、skinned、custom material等non-batch组合有明确compatibility key和负向测试 |
| MESH3D-G29 | stable render frame filesystem/decode/deep asset clone/GPU buffer create为0 |
| MESH3D-G30 | LOD/section/vertex/index/morph/skin block有wanted/resident/pin/evict/ticket/budget状态 |
| MESH3D-G31 | hot reload完整prepare后原子发布，失败保留last-good；old GPU generation按fence retire |
| MESH3D-G32 | `MeshSubmissionReceipt`可关联view/section/LOD/batch/instance/residency/pipeline及CPU/GPU成本 |

### 10.5 Skinning、Morph与History G33-G40

| Gate | 验收标准 |
|---|---|
| MESH3D-G33 | CPU reference与GPU shader都使用imported inverse bind，同一真实glTF fixture pose/pixel一致 |
| MESH3D-G34 | GPU skin路径不先CPU skin或clone整primitive，steady frame只上传changed palette ranges |
| MESH3D-G35 | palette arena支持current/previous、offset reuse、fragmentation/overflow和fence-safe retirement |
| MESH3D-G36 | joint/palette limit超出时cook拒绝或显式split/fallback，不被`.ok()`静默吞掉 |
| MESH3D-G37 | GPU morph不先CPU morph；delta按mesh generation一次resident，多实例只更新weights |
| MESH3D-G38 | morph current/previous history在reload、target relocation、camera cut和first frame下有明确validity |
| MESH3D-G39 | 多section character不静默混合animated/bind-pose，partial policy必须显式且有per-section receipt |
| MESH3D-G40 | deformed bounds、velocity、shadow与base pass消费同一pose/morph generation |

### 10.6 Collision、Product、Fault与Performance G41-G48

| Gate | 验收标准 |
|---|---|
| MESH3D-G41 | 非测试产品能从render source生成并加载Jolt cooked triangle/convex artifact，fixed tick无cook |
| MESH3D-G42 | collision weld/degenerate/material/subshape/scale policy与query/contact identity有fixture |
| MESH3D-G43 | render LOD eviction不破坏必需collision/nav数据，derived failure独立降级且generation可见 |
| MESH3D-G44 | 第一方reference project可clean import、save、reopen、animate、morph、LOD、collide、cook和退出 |
| MESH3D-G45 | fault matrix覆盖missing/corrupt/stale Mesh、wrong skeleton、reload race、OOM、device loss和cancel |
| MESH3D-G46 | 10K/100K static实例与代表性skinned/morph crowd长时soak无无界allocation/history/receipt增长 |
| MESH3D-G47 | CPU/GPU capture报告extract、prepare、skin/morph upload、cull、draw、VRAM/RSS与top offender |
| MESH3D-G48 | 同场景同画质同硬件与参考基线比较visual parity、CPU/GPU/frame/memory/upload；未胜出不得宣称优于Unreal |

## 11. 禁止的临时实现

1. 禁止只给`SceneMeshInstanceAsset`补三字段而不建立schema version、legacy migration、字段守恒清单和disk reopen测试。
2. 禁止继续同时维护Model inline primitive与external Mesh两份可独立变化的运行时真值。
3. 禁止把section vector ordinal、entity ID或packed primitive ordinal当持久stable identity。
4. 禁止用更大的固定palette数组掩盖joint limit、buffer数量、bind group和overflow合同。
5. 禁止在标记为GPU skin/morph的路径先执行CPU skin/morph，再把结果丢弃或临时上传。
6. 禁止把multi-draw indirect称为instancing；验收必须证明共享state/geometry与`instance_count > 1`的真实实例span。
7. 禁止把translation和scale近似球当mesh bounds，或让CPU/GPU/LOD/streaming各自推导不同空间的bounds。
8. 禁止render submission同步读取、解压、clone完整Mesh/Model或创建steady-state GPU geometry。
9. 禁止在external Mesh失败时无日志改画inline primitive，或在skin失败时无日志改画bind pose。
10. 禁止在fixed tick临时三角化/cook physics mesh，或让physics直接借用renderer私有GPU资源。
11. 禁止为高级Nanite/mesh shader/compute skin cache抢跑而绕过M0-M3的identity、artifact、bounds和数据守恒。
12. 禁止以unit test数量、draw call下降或单一vendor路径证明“性能优于Unreal”；必须先有视觉与行为等价。

## 12. 当前状态

本篇静态review完成，implementation仍为pending。可保留底座包括typed Mesh attributes/index/morph、局部校验与转换测试、Scene primitive/material/LOD DTO、真实GPU buffer与GPU Scene history、cached command/indirect replay、skin/morph velocity和PNG产品测试。必须重构的是它们之间的source/artifact/instance/prepared authority、stable section/rig/target identity、Scene字段守恒、screen-space LOD、qualified bounds、true instancing、deformation arena、异步residency、collision cook、typed fallback和产品资格。

实施第一步只能是M0：关闭`MESH3D-P0-001`并与Runtime61建立生成式Scene字段守恒；随后由Editor32产出M1 canonical Mesh artifact，再按M2-M6推进。Runtime09B的bounds/instancing、Runtime64/09D的同步冷加载、Runtime08C/Editor32的inverse-bind/skeleton安装和Runtime08A的collision cook未关闭前，本篇相关gate必须保持blocked/partial，不能通过局部adapter或新增第二格式标记完成。
