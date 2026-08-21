---
title: Runtime Mesh、Geometry、Section、LOD、Instancing、Skinning、Morph、Deformation、Bounds、Collision、Streaming 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime93
review_date: 2026-08-21
baseline_head: be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1
baseline_epoch: 336
related_code:
  - zircon_runtime/src/asset/assets/mesh
  - zircon_runtime/src/asset/assets/model
  - zircon_runtime/src/asset/importer/ingest
  - zircon_runtime/src/core/framework/render/mesh
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/scene/components/scene/mesh_renderer.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/project_io/mesh.rs
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh
  - zircon_runtime/src/graphics/scene/resources/gpu_model
  - zircon_runtime/src/graphics/scene/resources/prepared
  - zircon_runtime/src/graphics/scene/resources/resource_streamer
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
  - zircon_plugins/gltf_importer/runtime/src
  - zircon_plugins/obj_importer/runtime/src
  - zircon_plugins/physics/runtime/src/backend
tests:
  - zircon_runtime/src/asset/tests/assets/mesh
  - zircon_runtime/src/asset/tests/assets/model.rs
  - zircon_runtime/src/asset/tests/load/mesh.rs
  - zircon_runtime/src/asset/tests/pipeline/manager/model_import.rs
  - zircon_runtime/src/scene/tests/asset_scene/mesh_bindings.rs
  - zircon_runtime/src/scene/tests/render_extract/direct_sections.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache
  - zircon_plugins/gltf_importer/runtime/src/tests
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/backend/tests
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
  - docs/plans/optimize/zircon_runtime/69-runtime-mesh-static-mesh-skeletal-mesh-submesh-lod-instancing-skinning-morph-collision-streaming-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/17-first-party-virtual-geometry-source-runtime-editor-dist-catalog-asset-cook-cluster-page-streaming-culling-raster-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/StaticMesh.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/StaticMeshComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/SkeletalMesh.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SkinnedMeshComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/StaticMeshResources.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/SkeletalRenderPublic.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/GPUSkinCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GPUSkinCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/PhysicsEngine/BodySetup.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/PhysicsEngine/BodySetup.cpp
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
  - dev/godot/scene/3d/mesh_instance_3d.cpp
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
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Mesh、Geometry、Section、LOD、Instancing、Skinning、Morph、Deformation、Bounds、Collision、Streaming 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon的Mesh路径不是空壳。`MeshAsset`已有typed attribute/index/topology、normal/tangent生成、morph target、inverse-bind临时载体、local bounds、SDF与Virtual Geometry数据；内置glTF路径已有meshopt、WebP、skin/animation subasset与scene import；Graphics已有真实vertex/index buffer、GPU Scene current/previous transform、skin/morph history、cached command、indirect workspace、HZB、velocity pass和PNG产品测试。`PreparedModel`还会计算external Mesh dependency composite revision，morph storage也有changed-row upload。这些真实底座应迁入新架构，不能以“追赶Unreal”为由推倒重写。

但完整产品链仍不是工程级Mesh系统。`MeshAsset`、`ModelPrimitiveAsset`、Model root、Mesh subasset与first-party importer可能同时持有多份完整geometry；没有stable section/material slot/LOD family、平台化semantic bulk block、明确schema support window或统一derived generation。实际高优先级glTF插件是version 1/priority 120，丢失source tangent和vertex color、把animation写成generic Data placeholder，并对skinned mesh无条件cook默认Virtual Geometry；低优先级内置glTF却是version 2、保留更多attribute并产生真实animation asset。测试中的“first wave plugin fixture”直接调用内置importer，因而没有验证默认产品真正选择的插件实现。

运行时仍在render submission同步load/clone/convert/create GPU geometry。GPU Scene虽暴露`instance_count`，当前循环对每个pending draw固定`register(..., 1)`并写一条instance；multi-draw不是hardware instancing。更严重的是primitive bounds只用model translation作为center、变换列最大长度作为radius，既没消费`PreparedMesh.local_bounds`，也没有deformed bounds。GPU skin路径在选择前已经clone并CPU skin整primitive；GPU morph也会逐实例扫描target/vertex重建delta payload。动态fallback在draw build中创建新GPU mesh，而每个普通GPU mesh还无条件构建并保留wireframe edge segments。

碰撞路径同样没有工程化：渲染`MeshAsset`与`PhysicsMeshAsset`没有产品级cook/registry闭环；Jolt adapter把每个三角形创建为独立triangle shape后再组成static compound，不是离线cooked triangle mesh/BVH；builtin backend直接拒绝triangle mesh/heightfield/compound，其query路径也不支持这些形状。Mesh/Model没有LOD/section semantic residency、budget reservation、eviction或fence-qualified retirement。

Runtime69唯一登记的Scene保存数据损坏P0仍开放；Runtime09B、09D/64、08C/Editor32、08A和85所拥有的bounds/instancing、同步加载、skin binding、collision cook及外部source dependency P0也仍影响本路径。本篇不重复累计P0，登记 **48项P1、12项P2与48个资格门**，并以current source纠正Runtime69的局部描述。在真实import/save/reopen/animate/render/collide/cook、10K/100K实例、reload/device-loss/OOM/soak以及同画质Unreal对照证据闭合前，不得声称Mesh功能达到Unreal级，更不得声称性能或表现优于Unreal。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / bytes / test attributes | 证据等级 | fingerprint |
|---|---:|---|---|
| Mesh相关Runtime与first-party plugin审查语料 | **420 / 80,945 / 2,972,710 / 未单独归一** | E3逐文件覆盖asset/model/import、scene、GPU Scene、resource prepare、mesh renderer及physics backend | `c891c859fd819d3999d352f800a9057a61a8967694d3d8060ccd19c202d5ecd8` |
| focused tests | **37 / 10,746 / 387,698 / 108** | E3读取asset/import/scene/render product/plugin/physics focused tests | `ac485a918e5690991cb795d2b847ff43bcd81761196662fedf76b4111083ef5f` |
| 五引擎参考切片 | **35 / 49,725 / 1,989,302 / 0** | E2/E3读取Unreal Mesh/Skin/Collision、Bevy Mesh/skin/morph、Fyrox surface、Godot surface/storage与Unity GPUDriven | `f551a04763c17c29385907055400940689723a17430d59ef0c1d905d7a02e6f5` |

冻结集合代表2026-08-21共享working tree，不是只读HEAD或实现验收receipt。Git基线为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator baseline epoch为336。Bevy、Fyrox、Godot与Unity Graphics参考revision分别为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`与`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal镜像`Build.version`为6.0.0/UE5/changelist 0且无独立`.git`，由reference aggregate fingerprint冻结。

冻结时其他Session正在修改glTF/OBJ importer、builtin physics query、runtime glTF/mesh/model importer和Mesh draw context。本文读取并计入这些working-tree版本，但不拥有也未修改它们；因此`source_recheck_required: true`。实施前必须重新生成fingerprint，并特别重验importer descriptor/priority、attribute parity、physics mesh shape和GPU draw path。

### 2.2 Owner边界

| 领域 | Canonical owner | Runtime93责任 | 不重复登记 |
|---|---|---|---|
| source/import/build graph | Runtime85 + Plugins07 + Editor32 | 定义Mesh artifact输入输出、插件一致性消费门 | VFS/source snapshot/worker P0 |
| asset schema/version | Runtime86 | 定义Mesh/Model/skin/morph/section具体schema | 通用migration framework |
| scene persistence/transaction | Runtime61/62 | 关闭Mesh实例字段守恒与bounds消费 | 通用Scene transaction P0 |
| identity/generation | Runtime24/64 | 定义Section/Skin/Morph/Prepared qualified identity | 通用handle exhaustion/lease P0 |
| visibility/GPU Scene/instancing | Runtime09B | 提供Mesh batch compatibility与consumer gates | canonical bounds/true instancing P0 |
| upload/residency/budget | Runtime09D/64/90 | 定义Mesh semantic block与demand | 通用I/O、GPU completion、device owner P0 |
| animation/skin | Runtime08C + Editor32 + Plugins13 | 定义Mesh deformation artifact与消费一致性 | skeleton/pose/inverse-bind主owner P0 |
| collision | Runtime08A | 定义render-source到cooked collision adapter门 | physics backend/query主owner P0 |
| material/pipeline | Runtime91 | 定义stable material slot与batch input | Shader/PSO owner |
| Virtual Geometry | Plugins17 | 定义普通Mesh与VG generation/integration seam | cluster/page/raster实现细节 |

本轮只做current-source review，没有修改Rust、Cargo、asset或tooling，也没有运行Cargo、Editor/App、真实GPU、RenderDoc、cook/export、device loss、OOM、soak或benchmark。用户已明确tooling后续迁移Rust，因此本文不审查Python/Node工具链，也不把外部脚本当成修复。

### 2.3 Runtime69局部纠偏

| Runtime69判断 | current-source复核 | 结论 |
|---|---|---|
| GPU Scene已有instance span挂点 | `sync_gpu_scene_pending_draws`仍逐draw `register(device, key, 1)`并写单元素slice | 挂点存在，真实instancing仍不存在 |
| Prepared local bounds未进入visibility | GPU primitive center取model translation，radius只取transform列长度 | 不仅未消费真实bounds，还默认把每个mesh当单位球变换 |
| morph已有GPU current/previous基础 | delta/weight buffer已有changed-row upload和shadow copy | 保留该基础，但delta payload仍从每实例source Mesh逐target/vertex重建 |
| model external dependency缺乏generation | current source已有composite revision与dependency state recheck | 局部改善；仍是同步轮询/load和silent inline fallback |
| Jolt mesh path是临时测试注册 | current source确有产品backend入口 | 入口真实，但实现为per-triangle native shape + static compound，仍无cook/BVH artifact |
| importer只有一般性不完整 | actual priority-120 plugin与priority-10 builtin存在明确语义分叉，fixture又绕过actual plugin | 新增current-source P1族，不把fixture成功当产品成功 |

## 3. 当前产品链与断裂点

```text
OBJ / glTF / zmesh / model.toml
  -> importer arbitration (actual plugin may outrank richer builtin)
  -> ModelAsset + optional MeshAsset subassets + skin/morph/SDF/VG
  -> generic artifact serialization
  -> SceneMeshInstanceAsset / MeshRenderer
  -> World clone + full MeshRenderer archetype scan + primitive expansion
  -> ensure_scene_resources before visibility
  -> synchronous Mesh/Model load + clone + conversion + bounds + GPU create
  -> per-draw CPU skin/morph prework + pending primitive clones
  -> per-draw GPUScene register(instance_count = 1)
  -> cached/direct/indirect command replay
```

核心断裂有六处：importer selection改变资产语义；Model与Mesh存在双geometry真值；Scene持久字段少于render-visible字段；extract/prepare在稳定帧全量扫描与深clone；bounds、LOD与visibility没有同一artifact/generation；collision与render geometry没有共同source generation和独立cooked lifecycle。

## 4. 当前应保留的真实基础

1. `MeshAttributeValues`、typed builtin semantic、index/topology validation、normal/tangent生成与roundtrip测试应迁入canonical Mesh compiler，不另造平行vertex DTO。
2. 内置glTF的meshopt边界检查、supported required extension过滤、real animation/skin subasset以及reference-only model primitive方向应保留。
3. `model_geometry_revision`把model source与external mesh revision合成一代，是新Prepared generation key的局部内核。
4. GPU Scene的stable key、current/previous transform、morph weight和skin source history、staged upload与double-buffer slot可继续使用，但必须接受qualified identity和真正instance span。
5. morph buffer changed-row upload、indirect workspace、cached command和multi-phase processor是真实优化基础；它们不能替代artifact、residency或instancing。
6. `PreparedMesh.local_bounds`与`PreparedModel.local_bounds`已有计算入口，应成为canonical bounds consumer，而不是继续被unit-sphere proxy覆盖。
7. focused PNG/velocity/morph/VG tests可作为回归底座；需补actual importer、save/reopen、collision、scale/fault和benchmark证据。

## 5. 既有P0 current-source复核

| 唯一owner | 既有P0 | 当前状态与本篇消费门 |
|---|---|---|
| Runtime69 | `material_property_overrides`、`tint`、`material_alpha_mode`保存重开静默丢失 | **开放**；`SceneMeshInstanceAsset`与project IO仍没有三字段，roundtrip test仍未覆盖 |
| Runtime09B | canonical bounds与true instancing未进入产品提交 | **开放**；GPU Scene当前仍是unit-sphere proxy与`instance_count = 1` |
| Runtime09D/64 | render submission同步完整load/decode/clone/GPU create | **开放**；Mesh/Model ensure仍同步执行全部阶段 |
| Runtime08C/Editor32 | imported inverse-bind、joint map与skeleton installation无唯一产品闭环 | **开放**；palette仍从skeleton bind locals推导，MeshSkin只是临时矩阵载体 |
| Runtime08A | render mesh到backend/platform cooked collision artifact缺失 | **开放**；Jolt仍per-triangle组compound，builtin backend拒绝mesh |
| Runtime85/Plugins07 | external buffer/image source未进入不可变source snapshot/dependency truth | **开放**；builtin与plugin glTF仍从source path旁读外部文件 |

上述P0继续由原报告唯一计数。本篇新增0项P0；actual importer语义分叉、unchecked zmesh version、always-on wire segments等登记为Mesh领域P1，不能以重新编号膨胀总表。

## 6. P1：Asset、Schema、Section、LOD与Importer

| ID | 当前差距 | 必须重构的工程合同 |
|---|---|---|
| MESH93-P1-01 | `ZMeshDocument.version`写入并默认为1，但`into_mesh_asset`不检查 | schema authority提供supported version window、migration与unsupported typed rejection；不能让version成为装饰字段 |
| MESH93-P1-02 | `MeshAsset`没有stable section/submesh/material slot/LOD family | 建立`SectionId`、`MaterialSlotId`、per-LOD ranges/bounds与reimport relocation/orphan receipt |
| MESH93-P1-03 | validator不覆盖NaN/Inf、weight normalize/joint range、inverse-bind count、degenerate/winding与morph builtin format | compiler做完整semantic/finiteness/topology/deformation/budget验证并输出typed diagnostics |
| MESH93-P1-04 | infallible `render_mesh_descriptor`以`unwrap_or(&[])`把invalid position变空 | 产品只消费validated fallible artifact；valid-empty与invalid必须可区分 |
| MESH93-P1-05 | 无skin时把VG vertex ordinal编码进joint index槽 | VG、skin、meshlet与custom attribute各有独立typed semantic/layout；组合能力在cook/admission验证 |
| MESH93-P1-06 | Mesh/Model转换和import对缺失或短attribute补默认/resize | 保存source presence/provenance/generated flag；partial channel按recipe生成或typed失败，不伪造导入事实 |
| MESH93-P1-07 | Model inline primitive与external Mesh引用可同时保存完整geometry | hard cutover为单一artifact reference；任何bootstrap/last-good fallback都必须generation-bound、显式policy和receipt |
| MESH93-P1-08 | reference-only Model overview只看inline vertex而得到空统计/bounds | manifest携validated aggregate section/LOD/bounds；Editor/runtime都不临时加载bulk猜overview |
| MESH93-P1-09 | skin、morph、SDF、VG、collision、shadow/RT派生物没有共同source generation | `MeshDerivedArtifactSet`以source/import/platform hash关联，允许独立失败但禁止跨代组合 |
| MESH93-P1-10 | `MeshAssetUsage`只有MainWorld/RenderWorld bool | 定义metadata/bulk CPU retention、GPU residency、collision/nav pin、editor pin、streamable block与release policy |
| MESH93-P1-11 | priority-120 actual glTF plugin与priority-10 builtin在version/output kind/attribute/animation/VG语义上分叉 | importer contract、feature matrix与artifact output必须由同一BuildSet验证；默认选择不能比fallback路径更弱且无告警 |
| MESH93-P1-12 | actual glTF plugin丢tangent/color、animation占位、skinned mesh照常cook VG；两条路径都按mesh取first node skin并忽略node morph override | 精确保留glTF node/mesh/skin/morph语义；多node skin binding、node weights、tangent/color与unsupported extension有真实fixture |

## 7. P1：Scene、LOD、Extract、Bounds与Identity

| ID | 当前差距 | 必须重构的工程合同 |
|---|---|---|
| MESH93-P1-13 | MeshRenderer同时暴露Model、Mesh、primitive bindings和alternate LOD资源 | 编译成单一`ResolvedMeshInstance`；冲突组合在authoring/load typed失败，不靠分支优先级 |
| MESH93-P1-14 | primitive/material override按vector ordinal | Scene只引用stable Section/MaterialSlot identity；reimport产生relocation/orphan report |
| MESH93-P1-15 | Scene没有显式SkinArtifact/SkeletonInstance/joint-map binding | admission以qualified skin/skeleton generation验证，不按同entity存在animation pose猜兼容性 |
| MESH93-P1-16 | render policy缺完整cast/receive shadow、cull、motion、RT、skin-cache与residency表达 | requested/effective typed policy进入submission receipt，unsupported fail-closed或明确fallback |
| MESH93-P1-17 | LOD只比较camera translation与node translation欧氏距离 | 使用world/projected bounds、FOV/projection、scale、screen size、quality bias、hysteresis与camera-cut policy |
| MESH93-P1-18 | LOD以每实例alternate Model/Mesh重复表达 | LOD family归immutable artifact；instance只持forced/min/max/bias/crossfade/residency policy |
| MESH93-P1-19 | LOD仅校验threshold正finite | cook验证阈值单调、section/material/skin/morph兼容、bounds/visual error和fallback chain |
| MESH93-P1-20 | render/collision/nav/SDF/VG各自选LOD且无共同receipt | 各consumer独立选择effective LOD但绑定同source generation，render eviction不得破坏physics/nav pin |
| MESH93-P1-21 | `build_viewport_render_packet` clone整个World，随后全Mesh archetype scan/sort | retained Scene发布created/changed/removed delta；steady frame成本随dirty frontier增长 |
| MESH93-P1-22 | 每primitive clone morph weights，material overrides clone BTreeMap | extract传递generation-qualified packed handle，小对象引用而非大Vec/map复制 |
| MESH93-P1-23 | draw build对每个mesh线性搜索`animation_poses` | extract建立entity/rig到pose slot O(1)索引，并携frame/skeleton/pose generation |
| MESH93-P1-24 | stable key把primitive ordinal塞进有限位宽，release只有截断/碰撞风险 | Runtime24提供checked `SectionInstanceId`；超宽/remove/reuse/reimport有typed结果和模型测试 |

## 8. P1：Prepare、GPU Scene、Instancing与Memory

| ID | 当前差距 | 必须重构的工程合同 |
|---|---|---|
| MESH93-P1-25 | `static_batches`主要服务stats/tests，非唯一可执行batch authority | 删除死parallel plan或并入Runtime09B唯一batch planner，不能同时维护两套“已分组”事实 |
| MESH93-P1-26 | GPU primitive bounds使用translation + transform scale的单位球 | canonical local/world/deformed bounds ABI进入CPU frustum、GPU cull、LOD、streaming与shadow，同代同空间 |
| MESH93-P1-27 | 每pending draw固定注册一个GPU Scene instance | 建立state-compatible persistent instance span；64个同state实例必须共享geometry/primitive并提交`instance_count > 1` |
| MESH93-P1-28 | visibility在Mesh/Model/material/deformation ensure之后 | last-good bounds先做cheap multi-view admission，只为visible/prefetch集合准备昂贵资源 |
| MESH93-P1-29 | `ensure_mesh/model`同步load、clone、convert、bounds、SDF/deformation和GPU create | 消费Runtime64/09D/90异步ticket、budget reservation、completion与atomic PreparedGeneration publish |
| MESH93-P1-30 | Prepared Mesh/Model保留完整CPU asset，plugin model又可复制多份geometry | metadata/bulk分离与retention policy；统计CPU source/canonical/prepared/wire/GPU/retired所有bytes |
| MESH93-P1-31 | `GpuMeshResource::from_asset`对所有mesh无条件HashSet构建并保留wire segments | wireframe edge artifact按需求/capability懒加载或cook；普通solid mesh不支付O(triangle edges) CPU/RSS |
| MESH93-P1-32 | dynamic/CPU-deformed fallback在draw build创建新vertex/index buffer | persistent dynamic geometry/ring arena、dirty range upload、budget和fence retirement；steady framecreate为0 |
| MESH93-P1-33 | GPU mesh `usize -> u32` count/index等转换存在unchecked cast/expect | artifact limit与device capability在cook/admission checked；超限typed split/reject，不panic/截断 |
| MESH93-P1-34 | indirect batch合并多个单实例args，GPU Scene bind-group路径还排除batch | 明确区分multi-draw、instancing与GPU compaction；compatibility key和fallback reason可观测 |

## 9. P1：Skinning、Morph与Dynamic Deformation

| ID | 当前差距 | 必须重构的工程合同 |
|---|---|---|
| MESH93-P1-35 | palette主要按entity/skeleton ResourceId绑定，node skin只保留每mesh first binding | 使用`SkinBindingId + SkeletonInstanceId + joint map + artifact revision`，多node同mesh skin实例独立正确 |
| MESH93-P1-36 | palette从skeleton bind locals推导，未消费imported inverse bind | CPU reference和GPU shader统一`joint_world * inverse_bind` artifact；wrong rig/admission typed失败 |
| MESH93-P1-37 | GPU skin path在选择前clone并CPU skin全部vertices | 先决定path；GPU只消费prepared base geometry与dirty palette，CPU fallback仅按明确capability/budget执行 |
| MESH93-P1-38 | fixed 256-joint storage，`.ok()`静默吞oversize | capability报告joint/influence/palette limits；cook reject、split或显式CPU fallback均有receipt |
| MESH93-P1-39 | current/previous palette可形成per-draw固定大buffer/bind资源 | shared palette arena、offset allocator、changed-only upload、double buffer、fragmentation与fence-safe reuse |
| MESH93-P1-40 | history signature未包含skin/joint map/layout/device完整generation | reload/rebind后旧palette/source/morph history必须被qualified key拒绝 |
| MESH93-P1-41 | GPU morph仍逐实例扫描active targets和所有vertices构建delta rows | immutable morph delta block按Mesh generation一次resident；实例只上传weights/active target table |
| MESH93-P1-42 | direct morph可先CPU morph，skinned GPU source还可能由CPU-morphed primitive临时建GPU mesh | 路径组合在prepare阶段编译；GPU morph+skin不重复CPU工作，fallback对象跨帧复用且可预算 |
| MESH93-P1-43 | morph binding只有weight vector ordinal，node override/target relocation语义不足 | stable MorphTargetId/name/table generation，animation curve compile解析，reimport有relocation/orphan receipt |
| MESH93-P1-44 | deformed bounds没有随pose/morph generation更新 | cook per-joint/per-target保守bounds，runtime按quality union/refit/GPU reduce；cull/LOD/shadow/velocity同代 |

## 10. P1：Streaming、Collision、Diagnostics与Product

| ID | 当前差距 | 必须重构的工程合同 |
|---|---|---|
| MESH93-P1-45 | Mesh无LOD/section/vertex/index/morph/skin semantic residency；revision每帧轮询 | semantic block目录、event-driven dirty closure、single-flight prepare、wanted/resident/pin/evict/ticket/budget状态 |
| MESH93-P1-46 | external Mesh失败静默退回inline primitive，built-in reference-only与plugin duplicated model行为不同 | required dependency失败阻止ready；last-good必须version compatible、有lease、显式policy与reason |
| MESH93-P1-47 | collision由调用方手工注册PhysicsMeshAsset，Jolt逐triangle组compound，builtin不支持 | Runtime08A产出backend/platform keyed cooked triangle/convex/BVH artifact，含weld/material/subshape/scale/cache/refcount |
| MESH93-P1-48 | 缺actual-plugin import-save-reopen-animate-render-collide/cook与规模/fault证据 | 建立真实glTF/OBJ fixtures、Scene守恒、pixel/velocity/collision oracle、10K/100K instance、reload/OOM/device-loss/soak矩阵 |

## 11. P2长期能力

| ID | 延后项 | 提升条件 |
|---|---|---|
| MESH93-P2-01 | Nanite式cluster hierarchy与software/hardware raster融合 | 普通section/LOD/VG artifact、bounds、residency和GPU-driven submission先资格化 |
| MESH93-P2-02 | mesh/task shader管线 | RHI capability、fallback、pipeline cache与传统vertex path parity先完成 |
| MESH93-P2-03 | compute skin cache、tangent recompute与multi-pass复用 | inverse-bind、shared palette/delta arena、history与预算先完成 |
| MESH93-P2-04 | 8/12/variable influences与dual quaternion skinning | 4-weight linear skin artifact/validator/CPU-GPU parity先完成 |
| MESH93-P2-05 | cloth/groom/physics deformation composition | deformation graph、space/order/history与对应owner稳定后 |
| MESH93-P2-06 | runtime procedural/editable mesh与partial topology mutation | immutable artifact、dynamic arena、transaction与collision recook policy先完成 |
| MESH93-P2-07 | mesh page GPU feedback/direct storage/GPU decompression | semantic block、async I/O、upload queue、budget与fault recovery先完成 |
| MESH93-P2-08 | RT BLAS refit/rebuild与skinned RT geometry | RHI/AS owner及deformation generation/bounds/residency先完成 |
| MESH93-P2-09 | runtime adaptive simplification/automatic LOD | offline deterministic LOD cook、quality/crossfade与visual error oracle先完成 |
| MESH93-P2-10 | impostor/HLOD/mesh merge跨World partition | stable section/material identity、partition cell与instance ownership先完成 |
| MESH93-P2-11 | multi-GPU/UMA/mobile特化memory policy | 单device离散/UMA capability与bytes/fence/retention会计先完成 |
| MESH93-P2-12 | 超大世界deformed mesh double-precision/rebase优化 | Runtime23空间合同与render-relative current/previous pipeline先完成 |

## 12. 五引擎差异证据

### 12.1 Unreal Engine

Unreal把`FStaticMeshSourceModel`、`FStaticMeshRenderData`、`FStaticMeshLODResources`、`FStaticMeshSection`、material slot、screen-size LOD、streaming bulk与Nanite settings分层；Skeletal Mesh另有LOD render data、skin weight/profile、inverse reference matrices、morph/cloth与GPU Skin Cache current/previous资源。`UBodySetup`及其cook/derived data把collision生命周期与render resource分离。Zircon应采用这种source/cooked/render/physics分权与per-LOD/per-section身份，不复制UObject/RHI线程模型。

### 12.2 Bevy

Bevy的Mesh具typed attribute ID/layout与`RenderAssetUsages`；`SkinnedMesh`显式持inverse-bind asset和joint entities，`SkinnedMeshBounds`按joint influence生成独立bounds。morph对target count有typed error，render侧区分current/previous weights与uniform/storage能力。它证明即使较轻量引擎也不会把skin binding、inverse bind、deformed bounds和morph instance state塞进一个临时Mesh DTO。

### 12.3 Fyrox

Fyrox区分Mesh、Surface、SurfaceData/Resource、material、geometry buffer与bounds，并对dynamic/static数据变化保持显式资源关系。可借鉴surface作为稳定绘制/material单位、资源共享和bounds owner；不能照搬其scene graph handle模型。

### 12.4 Godot

Godot Mesh/ArrayMesh以surface为单位保存format、material、AABB、blend shape和LOD dictionary，ImporterMesh负责simplification/LOD/shadow mesh等derived build，MeshInstance显式绑定mesh/skeleton/skin/blend。它证明section/LOD/import derived data应在artifact阶段成型，而不是Scene用alternate resource列表临时模拟。

### 12.5 Unity Graphics

本地`dev/Graphics`不含Unity native Mesh importer或physics cook，因此本文不臆造其完整资产实现。可验证的GPUDriven部分使用persistent `InstanceHandle`/allocator、renderer-to-instance mapping、world AABB、LODGroup data、screen-relative thresholds、crossfade与批量update/upload/culling。它正好反证Zircon当前per-draw register-one与unit-sphere proxy不是同级GPU-driven instance system。

## 13. 目标架构与唯一所有权

```text
MeshSourceRecipe (Editor32 / Plugins07)
  -> Runtime85 MeshBuildGraph
  -> MeshArtifactManifest
       { schema/platform/backend/source generation, sections, LODs, semantic blocks }
  -> MeshGeometryArtifact
       { immutable streams, section ranges, material slots, bounds, quantization }
  -> MeshDeformationArtifact
       { skin binding, joint remap, inverse bind, morph table/delta blocks, deformed bounds }
  -> MeshDerivedArtifactSet
       { collision/nav/SDF/VG/shadow/RT, independent status under same source generation }
  -> ResolvedMeshInstance
       { qualified artifact, section overrides, rig binding, LOD/render/residency policy }
  -> MeshSceneDelta / retained RenderScene
  -> MeshResidencyService + Runtime90 UploadService
  -> PreparedMeshGeneration
  -> MeshDeformationInstance + persistent InstanceSpan
  -> MeshSubmissionReceipt
```

| 类型 | 唯一责任 | 禁止承载 |
|---|---|---|
| `MeshSectionId` | 跨reimport稳定section/material identity | Vec ordinal、packed entity bits |
| `MeshArtifactGeneration` | source/import/platform/schema/derived generation | 单一resource revision冒充全部依赖 |
| `MeshGeometryBlockId` | LOD/section/semantic stream可寻址bulk | 完整bincode对象offset猜测 |
| `SkinBindingArtifact` | skeleton signature、joint remap、inverse bind、influence ABI | 按entity猜skin、只存矩阵Vec |
| `MorphTargetTable` | stable target identity与immutable delta blocks | 每实例每帧重建delta |
| `PreparedMeshGeneration` | device generation、resident blocks、GPU allocations、last-good | Editor source、同步文件I/O |
| `MeshInstanceSpan` | compatible primitive的persistent instance range | multi-draw args数量冒充instancing |
| `MeshSubmissionReceipt` | requested/effective LOD、bounds、visibility、fallback、bytes/cost | 只有全局计数、无per-instance原因 |

## 14. 依赖顺序与重构里程碑

| 里程碑 | 内容 | 依赖/退出证据 |
|---|---|---|
| M0 | current behavior characterization与Scene字段守恒 | 关闭Runtime69 P0；actual plugin与builtin golden parity冻结 |
| M1 | canonical section/LOD/geometry/deformation/derived artifact | Runtime85/86 + Editor32；坏mesh拒绝、clean cook hash、无双geometry真值 |
| M2 | single ResolvedMeshInstance、stable override、rig binding与screen-space LOD | Runtime61/62/65；多camera/FOV/scale/hysteresis/residency tests |
| M3 | retained extract、canonical local/world/deformed bounds与cheap visibility | Runtime09B；steady extract随dirty增长，CPU/GPU/LOD/streaming bounds parity |
| M4 | async prepared generation、semantic residency与全bytes预算 | Runtime09D/64/90；render submit无I/O/deep clone/create，reload last-good/fence闭环 |
| M5 | true instance span、palette/morph arena与persistent dynamic geometry | M1-M4；64/10K实例、steady allocation 0、skin/morph velocity parity |
| M6 | cooked collision及所有derived generation集成 | Runtime08A/Plugins17；fixed tick无cook，subshape/material/query identity完整 |
| M7 | product qualification与Unreal公平对照 | clean import/save/reopen/play/cook、fault/soak/capture/benchmark全部绑定BuildSet |

M5不得越过M0-M4直接“优化draw call”。没有stable section、bounds、rig与artifact generation时，batch key、history和residency会再次失效。M6也不得让physics借用renderer私有GPU buffer；它消费同源但独立生命周期的cooked artifact。

## 15. 资格门

| Gate | 必须形成的证据 |
|---|---|
| MESH93-G01 | tint/alpha/material override经memory/document/disk/reopen/extract/pixel完全守恒 |
| MESH93-G02 | legacy Scene migration与unknown/invalid override有固定fixture和typed结果 |
| MESH93-G03 | zmesh/Model/Mesh/Skin/Morph schema都有version、support window、migration与deterministic hash |
| MESH93-G04 | Model/Mesh/primitive/LOD冲突在authoring/load失败，不靠runtime分支选真值 |
| MESH93-G05 | Section/MaterialSlot/SkinBinding/MorphTarget reimport有relocation/orphan receipt |
| MESH93-G06 | actual priority-selected importer与测试fixture使用同一descriptor/implementation/BuildSet |
| MESH93-G07 | builtin/plugin glTF对tangent/color/UV/skin/node morph/animation/VG支持矩阵明确 |
| MESH93-G08 | external glTF buffer/image全部进入immutable source snapshot与dependency hash |
| MESH93-G09 | compiler拒绝NaN/Inf、非法index/topology/weight/joint、degenerate超限和morph schema |
| MESH93-G10 | valid-empty与invalid-empty可区分，产品无`unwrap_or(&[])`伪ready |
| MESH93-G11 | section/material/LOD topology与bounds写manifest，runtime不加载source bulk推目录 |
| MESH93-G12 | skin artifact包含skeleton signature、joint map、inverse bind、influence layout和bind bounds |
| MESH93-G13 | morph artifact包含stable target table与immutable delta blocks，curve compile解析 |
| MESH93-G14 | VG ordinal与joint index独立semantic stream，skinned+VG组合跨backend验证 |
| MESH93-G15 | collision/nav/SDF/VG/shadow/RT均绑定source generation，混代拒绝 |
| MESH93-G16 | clean cook重复hash一致，platform/backend/import setting正确改变artifact key |
| MESH93-G17 | Scene显式绑定artifact、section override、skin/skeleton与policy，不按entity猜 |
| MESH93-G18 | LOD按projected bounds/screen size、FOV、scale、quality和hysteresis选择 |
| MESH93-G19 | forced/min/max/bias/crossfade/residency fallback都有requested/effective receipt |
| MESH93-G20 | LOD切换保持section/material/skin/morph兼容，坏artifact在cook/admission失败 |
| MESH93-G21 | steady extract随dirty instance/section增长，不clone全World或全量scan/sort |
| MESH93-G22 | morph weights/material overrides以generation handle共享，primitive展开无大Vec/map clone |
| MESH93-G23 | local/world/deformed bounds在CPU/GPU/LOD/streaming/shadow中空间与代际一致 |
| MESH93-G24 | identity超宽/remove/reuse/reimport均typed处理，无release截断collision |
| MESH93-G25 | 不可见且无prefetch实例不执行material/deformation/GPU allocation/draw build |
| MESH93-G26 | 64同mesh/material实例形成instance span，不注册64个one-instance primitive |
| MESH93-G27 | indirect instanceCount/baseInstance与compaction一致，direct fallback原因可见 |
| MESH93-G28 | transparent/skinned/custom override等compatibility key有正负测试 |
| MESH93-G29 | stable render frame filesystem/decode/deep asset clone/GPU buffer create为0 |
| MESH93-G30 | LOD/section/vertex/index/morph/skin block有wanted/resident/pin/evict/ticket/budget |
| MESH93-G31 | hot reload完整prepare后原子发布，失败保留last-good，旧GPU代按fence retire |
| MESH93-G32 | solid mesh不构建/保留wire segments；debug wire artifact按需预算和回收 |
| MESH93-G33 | CPU reference与GPU shader都使用imported inverse bind，真实glTF pose/pixel一致 |
| MESH93-G34 | GPU skin不先CPU skin/clone整primitive，steady只上传changed palette ranges |
| MESH93-G35 | palette arena有current/previous、offset reuse、fragmentation/overflow和fence retirement |
| MESH93-G36 | joint/palette limit超出typed reject/split/fallback，不被`.ok()`吞掉 |
| MESH93-G37 | GPU morph不先CPU morph；delta按mesh generation一次resident，多实例只更新weights |
| MESH93-G38 | morph history在reload/target relocation/camera cut/first frame有明确validity |
| MESH93-G39 | 多section character不静默混合animated/bind pose，partial policy有per-section receipt |
| MESH93-G40 | deformed bounds、velocity、shadow、base pass消费同一pose/morph generation |
| MESH93-G41 | 非测试产品从render source生成并加载backend cooked triangle/convex/BVH artifact |
| MESH93-G42 | collision weld/degenerate/material/subshape/scale/query/contact identity有fixture |
| MESH93-G43 | render LOD eviction不破坏collision/nav pin，derived failure独立且generation可见 |
| MESH93-G44 | reference project可clean import、save、reopen、animate、morph、LOD、collide、cook、退出 |
| MESH93-G45 | fault matrix覆盖missing/corrupt/stale mesh、wrong rig、reload race、OOM、device loss/cancel |
| MESH93-G46 | 10K/100K static与代表性skinned/morph crowd soak无无界allocation/history增长 |
| MESH93-G47 | CPU/GPU capture含extract/prepare/deform/cull/draw/VRAM/RSS/top offender和BuildSet |
| MESH93-G48 | 同场景同画质同硬件与Unreal比较visual parity、CPU/GPU/frame/memory/upload；未胜出不得宣称优于Unreal |

## 16. 禁止的临时实现

1. 禁止只给`SceneMeshInstanceAsset`补三个字段而没有schema version、migration、生成式守恒清单与disk reopen测试。
2. 禁止继续让Model inline primitive与external Mesh成为两份可独立变化的runtime真值。
3. 禁止以test fixture调用builtin importer代替actual priority-selected plugin产品测试。
4. 禁止通过默认tangent/color/weight或resize短channel把invalid source伪装成ready。
5. 禁止把section ordinal、entity ID或packed primitive ordinal当持久identity。
6. 禁止把translation/scale单位球称为mesh bounds，或让CPU/GPU/LOD/streaming分别推导不同bounds。
7. 禁止把multi-draw称为instancing；必须证明共享geometry/state和`instance_count > 1`。
8. 禁止标记GPU skin/morph后仍先执行CPU skin/morph，再丢弃或临时上传结果。
9. 禁止用更大固定palette、更多per-draw buffer或`.ok()`掩盖capacity/overflow合同。
10. 禁止所有solid mesh默认生成wire edge CPU副本，或让debug模式成本进入普通产品预算盲区。
11. 禁止render submission同步load、解压、clone完整Mesh/Model或创建steady geometry。
12. 禁止external Mesh/skin失败无日志改画inline/bind pose，也禁止fixed tick临时cook per-triangle collision。
13. 禁止让physics/navigation读取renderer私有GPU资源；必须消费独立cooked artifact。
14. 禁止高级Nanite/mesh shader/compute skin cache抢跑并绕过identity、artifact、bounds与residency。
15. 禁止用unit test数量、draw call单项或单vendor路径证明达到/优于Unreal；必须先证明行为与画质等价。
16. 禁止以compat facade、re-export、双写或永久fallback保留旧geometry authority；迁移必须hard cutover。

## 17. 本轮输出边界

本篇完成Runtime Mesh/Geometry/Section/LOD/Instancing/Skinning/Morph/Deformation/Bounds/Collision/Streaming的current-source E3静态审查，未实施production重构。Runtime69的Scene字段丢失P0及Runtime09B、09D/64、08C/Editor32、08A、85相关P0仍开放并由原owner计数。报告新增0项P0、48项P1、12项P2和G01-G48。

当前判定是：Zircon已有可迁移的Mesh数据、import、GPU history、indirect与pixel-test内核，但actual importer一致性、canonical artifact、stable section/rig/target identity、single geometry truth、screen-space LOD、真实bounds、true instancing、deformation arena、async residency、cooked collision和产品资格都未达到工程级。实施必须从M0数据守恒和actual product characterization开始，再按M1-M7推进；在G01-G48形成BuildSet-bound与真实产品证据前，不得把本报告标记为implemented，也不得声称达到或超过Unreal。
