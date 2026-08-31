---
title: Editor Model、Mesh、Skeleton、Geometry Import、LOD、Collision、Retarget 与 Preview 当前源码复核
category: zircon_editor
report_id: Editor209
review_date: 2026-08-28
baseline_head: a2d8d811c4a3a1fc1db6f5375c491e7e4502533f
verification_head: a2d8d811c4a3a1fc1db6f5375c491e7e4502533f
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor32
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/106-editor-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-current-source-review.md
  - docs/plans/optimize/zircon_editor/153-editor-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-current-source-review.md
related_code:
  - zircon_editor/src/core/asset/import_flow
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/core/editing/paths.rs
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_editor/src/ui/retained_host/app/assets/workspace.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets/mesh_import.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/asset/mesh_import_path.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/simulation_physics.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
  - zircon_editor/assets/ui/editor/asset_browser.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_retarget_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation/workbench_extension_collision_proxy_workspace.zui
  - zircon_runtime/src/asset/assets/model
  - zircon_runtime/src/asset/assets/mesh
  - zircon_runtime/src/asset/importer/ingest
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/source_plan.rs
  - zircon_runtime/src/asset/project/manager/durable_transaction.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/core/framework/animation
  - zircon_runtime/src/animation/manager/pose.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/model_geometry_resolution.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/core/framework/physics/mesh_asset.rs
  - zircon_plugins/animation/runtime/src/evaluation/skeleton_target_table.rs
  - zircon_plugins/gltf_importer/runtime
  - zircon_plugins/asset_importers/model/runtime
  - zircon_plugins/virtual_geometry/editor
  - zircon_plugins/physics/runtime/src/backend/jolt
  - zircon_app/Cargo.toml
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/69-runtime-mesh-static-mesh-skeletal-mesh-submesh-lod-instancing-skinning-morph-collision-streaming-product-integration-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/18-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-authoring-review.md
  - docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md
failure_records:
  - docs/plans/optimize/zircon_plugins/07/failure-2026-08-23-gltf-empty-texture-source-option-contract.md
  - docs/plans/zircon_runtime/runtime/04/failure-2026-07-17-woc-gltf-meshopt-webp-import.md
  - docs/plans/zircon_runtime/runtime/04/failure-2026-08-01-runtime04-gltf-importer-current-compile-errors.md
  - docs/plans/zircon_runtime/runtime/04/failure-2026-08-18-missing-subasset-parent-fallback.md
  - docs/plans/zircon_runtime/frameworks/04/failure-2026-08-24-gltf-workspace-dependency-catalog-drift.md
  - docs/plans/zircon_editor/editor/09/failure-2026-07-22-asset-import-duplicate-admission-backpressure.md
  - docs/plans/zircon_editor/editor/09/failure-2026-08-05-import-diagnostics-editor-log-source-bridge.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/StaticMeshEditor/Private/StaticMeshEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/SkeletalMeshEditor/Private/SkeletalMeshEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private/AnimationEditorPreviewScene.cpp
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Pipelines/Public/InterchangeGenericAssetsPipeline.h
  - dev/UnrealEngine/Engine/Plugins/Interchange/Runtime/Source/Pipelines/Private/InterchangeGenericAssetsPipeline.cpp
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
finding_status:
  p0_open: 4
  p0_partial: 1
  p1_open: 38
  p1_partial: 20
  p1_closed: 2
  p2_open: 12
  p2_partial: 0
  p2_closed: 0
gate_status:
  fail: 21
  partial: 10
  pass: 1
---

# Editor209 · Model / Mesh / Skeleton / Geometry Import / LOD / Collision / Retarget / Preview 当前源码复核

## 1. 结论

Editor153之后，模型导入的Editor接入继续变得更可信。`EditorAssetImportFlow::submit_model_source`只提交一个Runtime-owned compound model job，`EditorModelImportTicket`直接携带`ProjectImportReceipt`；新增diagnostics bridge把committed、failed和submission rejected终态投影到`EditorLogService`的`LogSource::import()`。`DeferredSubmissionDiagnostic`还覆盖了job在submission arm之前完成的竞态，避免同一终态重复或丢失。插件侧新增geometry convergence测试，验证reference-only root、Mesh自持authored tangent/color、normal texture有效UV与clearcoat tangent拒绝。这些都是应保留的底座，而不是完成Geometry产品的证据。

产品断路仍然没有改变。Quick Import依旧只有字符串TextField和按钮，没有file picker、OS drop、paste payload或typed preflight；Editor路径校验接受`.gltf`，Runtime对外部source却明确拒绝`.gltf`并只允许`.obj/.glb`。外部OBJ只快照主文件和同名MTL，不发现MTL纹理；目标固定到`res://models/{filename}`且重复路径只能失败。没有版本化Geometry Import Recipe、normalized scene selection、output manifest、target/conflict policy或三方reimport diff。更严重的是事务成功后`complete_model_import`仍调用`runtime.import_mesh_asset`修改当前World，资产发布和场景创作仍是一个隐式产品动作。

几何权威仍是P0。Core与plugin glTF测试都确认root `ModelPrimitiveAsset`清空inline `vertices/indices`并引用Mesh subasset；渲染streamer能够解析引用Mesh并把dependency revision纳入prepared geometry revision，这是正确方向。但`ModelAsset::render_mesh_descriptors`、`primitive_overviews`、`overview`和management record继续只读取inline payload，因此引用式模型仍被报告为0 vertices、0 indices和空bounds。schema又允许inline与reference同时存在并称Mesh“mirrors”payload，没有exactly-one验证、Unavailable状态或generation-qualified resolved overview。

Skin/animation没有形成可运行的导入链。Core会生成typed Skeleton/Clip和path-derived target ID，Animation target table能诊断parent、cycle、duplicate与ambiguous name；可是`MeshSkinAsset`仍只有IBM vector，同一Mesh被多个node/skin实例引用时仍first-wins。`AnimationSkeletonBoneAsset`没有stable ID，renderer按bone name拼pose，并用Skeleton local reference pose计算`posed_world * bind_world.inverse()`，完全没有消费Mesh里保存的authored IBM。Core与plugin的glTF Scene builder都把`animation_skeleton`、`animation_player`及其扩展播放器写成`None`；plugin还把Animation输出为“not implemented yet”的generic Data placeholder。

LOD、Collision、Retarget和Preview仍只拥有零散底座。Scene instance LOD确实会在运行时选择不超过相机到entity origin距离的最大`min_distance`，但importer始终生成空LOD列表，也没有asset group、reduction provider、screen coverage、hysteresis或crossfade。Physics runtime有typed triangle/heightfield payload和Jolt validation，却没有Render Mesh到PhysicsMesh的qualified converter、CollisionSetup source/cook或Editor backend preflight。Collision Proxy和Retarget表面存在大量route与ZUI，但bake/preview/apply只返回queued/fixed sample feedback。Model/Mesh/Skeleton没有toolkit；Texture之外仍默认placeholder thumbnail；Virtual Geometry editor descriptor仍指向物理不存在的`plugins://virtual_geometry/editor/authoring.zui`。

目标架构保持为：`GeometryImportSource + VersionedImportRecipe -> NormalizedGeometryScene -> Model/Mesh/Material/Skeleton/Skin source assets -> qualified LOD/Collision/VG/SDF derived artifacts -> atomic ProjectImportReceipt -> isolated toolkits/preview -> explicit undoable AddToScene`。Core与plugin必须只有一个glTF语义authority；Editor只消费Runtime合同，不能复制parser、resource truth或derived compiler。

## 2. 当前物理范围与证据等级

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 指纹与说明 |
|---|---:|---|
| Zircon Editor/Runtime/Plugin/App selected | **258 / 49,151 / 45,205 / 1,798,206 / 304 / 49** | importer、transaction、Model/Mesh/Skeleton/Skin/animation、LOD、toolkit/thumbnail、workbench、physics与产品装配；fingerprint `c3b657df26514923d8b46b19d2c1ed58007a8c9d626d317259b0ba0e0e9c291f` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference | **21 / 29,331 / 25,946 / 1,135,717 / 32 / 0** | Interchange/Static/Skeletal/Persona/IKRetarget、Godot importer/toolkits/BoneMap、Fyrox/Bevy skin、Unity LOD/crossfade；fingerprint `8c523e2d1192cdb02f4c8dadf57046b074f8a83b919e1957f8f383197b0bfcec` |
| 全部选择集 | **279 / 78,482 / 71,151 / 2,933,923 / 336 / 49** | 两组不重叠；fingerprint `3cbcaa6f274b153dc15630001a73444483f67aa4bcd3ada2a7eb00eee55fce0b` |

本轮基于`a2d8d811c4a3a1fc1db6f5375c491e7e4502533f`及共享dirty working tree取证。Godot、Fyrox、Bevy、Graphics参考revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`a7e4c051d256a781ab362c64316b125a1e104694`；本地Unreal镜像没有`.git`元数据，只能用上述物理文件指纹固定。以上是working-tree物理快照，不是commit manifest，实施前必须重算。按用户要求，本轮排除`tools/`，也没有查询、轮询、等待或实时跟踪协调器。

## 3. 当前已存在且必须保留的底座

1. `MeshAsset`已有typed attributes、16/32-bit indices、topology、usage、morph、skin metadata、Mesh SDF、Virtual Geometry、normal/tangent生成和format/length/index/topology validation。
2. Core glTF路径已有single-read decode、meshopt、WebP、quantization、PBR/texture transform、typed Mesh/Material/Texture/Scene/Skeleton/Clip subasset；WOC 949 GLB与关联failure仍未完成managed动态验收。
3. Runtime project manager已有source digest/config hash、zmeta、stable UUID保留、dependency resolution、targeted/full generation、watch echo过滤、generation fence、journal recovery和durable compound commit。
4. Editor model import已有job/cancel边界、`ProjectImportReceipt`、terminal log projection及pre-arm completion竞态处理；P1-10原子发布保持Closed。
5. Runtime resource streamer会解析Model的Mesh references并把dependency resource id/revision/state写入composite geometry revision；该正确render path不能替代错误的Model management overview。
6. Animation插件已有path-derived `AnimationTargetId`、dense target slot和parent/cycle/duplicate/ambiguous/unresolved诊断；CPU/GPU skinning与previous palette路径是真实实现。
7. Scene能序列化primitive/material binding、morph weights和per-instance LOD，Renderer能实际执行distance LOD选择；这只是Runtime底座，不是asset-level authoring产品。
8. Physics runtime已有typed triangle/heightfield payload与Jolt shape validation；VG/SDF cook request已有typed settings和部分derived key基础。
9. Editor asset type registry、thumbnail cache、source-digest stale check、job system、notification与catalog projection可复用于Geometry Toolkit。

## 4. 参考实现差异

| 参考 | 直接源码事实 | Zircon缺失的工程合同 |
|---|---|---|
| Unreal Interchange | Generic Assets Pipeline分离Common Mesh、Mesh、Animation、Material pipeline，并保存reimport strategy、命名/目录、transform offset与conflict处理 | 只有generic TOML settings和固定target path，没有geometry recipe、conflict model、selection preview或稳定subpipeline合同 |
| Unreal Static/Skeletal/Persona | 独立toolkit、AdvancedPreviewScene、LOD统计/重导、自定义LOD、box/sphere/capsule/convex collision transaction与骨架动画预览 | 无Model/Mesh/Skeleton toolkit，LOD/Collision/Retarget动作不产生资产事务 |
| Unreal IK Retargeter | Retargeter是持久资产，拥有source/target rig、retarget poses、operation stack、controller与version/reinitialize边界 | 只有静态ZUI与固定feedback，没有source/profile/solver/compiler/receipt |
| Godot 3D import | import options覆盖generated LOD、shadow mesh、lightmap、post-import；BoneMap/SkeletonProfile参与retarget import并有独立编辑器 | 无normalized scene option graph、post-import contract、BoneMap/Profile与真实LOD generation |
| Fyrox glTF/FBX | `ModelImportOptions`、material search、animation graph/player、逐skin joint family与IBM进入scene graph | 同Mesh多skin first-wins，Scene不安装player，IBM不进入renderer |
| Bevy glTF/Mesh | loader settings控制materials/cameras/source/bounds；`SkinnedMesh`显式引用joint entities与inverse bindposes | Skin不是独立typed instance binding，Scene节点没有Skin引用 |
| Unity Graphics | LOD group保存screen-relative thresholds、fade width、world reference point/size、forced mask，并有dither crossfade shader | 只按entity origin distance选择，没有group、screen size、hysteresis/crossfade/platform policy |

Unity本地参考只覆盖Graphics Runtime，不包含Unity Model Importer；本报告不把本地不可见的Unity Editor能力当作证据。

## 5. P0 当前状态

| ID | 状态 | 当前证据 | 必须重构为 |
|---|---|---|---|
| P0-1 | `Partial` | production path edit、Runtime job、receipt和terminal import log已存在；仍无picker/drop/paste typed request，`.gltf` UI/Runtime准入矛盾 | `GeometryImportSourceRequest`统一入口并在job前完成capability/admission |
| P0-2 | `Open` | Core `zircon.builtin.model.gltf`与plugin `gltf_importer.gltf`同时匹配gltf/glb，版本相差1、priority 10/120；registry按availability、priority、suffix、slot排序 | 保留一个canonical implementation，plugin只作package adapter；删除旧provider并跑semantic corpus |
| P0-3 | `Open` | Mesh skin只有IBM，multi-skin first-wins，Skeleton无stable ID，renderer不消费IBM，Scene skeleton/player为None | 独立Skin asset、stable joint map、IBM/reference-pose合同与typed Scene binding |
| P0-4 | `Open` | root Model已reference-only，但overview/descriptor继续读取空inline，dual payload仍合法 | exactly-one geometry authority、resolved Model artifact与qualification-aware overview |
| P0-5 | `Open` | Model/Mesh/Skeleton无toolkit；LOD/Collision/Retarget workbench没有domain artifact或runtime receipt | isolated Preview Scene、dedicated toolkits、LOD/Collision/Retarget source/compiler/job/receipt |

## 6. P1 Import Source、Recipe、Reimport 与发布事务

| ID | 状态 | 当前证据 | 必须重构为 |
|---|---|---|---|
| P1-01 | `Partial` | AssetUri、source digest、mtime、watch echo与source snapshots存在，但geometry source及dependency set没有稳定整体身份 | `GeometryImportSourceId`绑定主文件、sidecars、provider、generation与digest |
| P1-02 | `Open` | zmeta有format version，`import_settings`仍是generic TOML；仅VG/SDF有typed request | 版本化`GeometryImportRecipe`覆盖axis/unit/material/skin/LOD/collision/normal/tangent与migration |
| P1-03 | `Partial` | TextField edit已接production，placeholder提到drop/paste但没有对应typed event/payload，仍只传路径字符串 | picker/drop/paste统一生成typed source request，UI只投影admission结果 |
| P1-04 | `Partial` | 外部OBJ主文件与同名MTL可原子snapshot/staging；外部`.gltf`被拒绝，buffer/image/OBJ texture依赖未发现 | provider-owned dependency discovery、containment、快照、原子staging/rollback |
| P1-05 | `Partial` | target固定为`res://models/{filename}`且duplicate直接拒绝 | preview选择target folder/name、rename/replace/keep-both policy与conflict receipt |
| P1-06 | `Open` | durable import完成后`workspace.rs`仍调用`runtime.import_mesh_asset`改当前World | Import只发布资产；Add to Scene成为独立可撤销Editor command |
| P1-07 | `Open` | 无normalized scene tree、mesh/material/animation selection与output manifest preview | staging result可选择并冻结为publish manifest |
| P1-08 | `Partial` | terminal结果进入typed import log source；parser/importer仍大量聚合字符串且无node/accessor/source span | diagnostic code、source span、subasset identity、suggestion与severity |
| P1-09 | `Open` | generic reimport/UUID保留存在，几何没有old source/local override/new source三方diff | stable IDs驱动reimport diff、conflict resolution与override preservation |
| P1-10 | `Closed` | Runtime compound transaction原子提交source/artifact/meta/registry/resource并返回generation-qualified receipt | 保持单一Runtime owner；扩展receipt内容，不恢复Editor多步写入 |
| P1-11 | `Partial` | registry有capability status、availability rank和priority，Quick Import前不可见最终provider/profile/version | preflight展示chosen provider、capability、schema、reason和fallback policy |
| P1-12 | `Partial` | `AssetManager::import_model_source`是中立Runtime合同，production caller仍只有Editor job，没有batch/headless/Cook parity | Editor/batch/headless/Cook共用request、recipe、receipt与digest测试 |

## 7. P1 Model、Mesh、Material、Morph 与几何数据合同

| ID | 状态 | 当前证据 | 必须重构为 |
|---|---|---|---|
| P1-13 | `Open` | `ModelPrimitiveAsset`允许inline与reference并称mirror，无exactly-one validation | source Model只保留stable Mesh reference；legacy inline显式migration后删除 |
| P1-14 | `Open` | glTF labels依赖数组index，缺跨reorder/reimport的node/mesh/primitive semantic ID | normalized scene分配stable source keys并持久映射subasset UUID |
| P1-15 | `Open` | primitive material依赖存在，但Model/Mesh无stable section/material-slot schema | `MeshSectionId + MaterialSlotId`贯穿import、scene override、reimport和render |
| P1-16 | `Open` | referenced-only Model overview仍返回0统计和空bounds；render streamer另有正确resolved path | resolver输入qualified Mesh generation；未解析返回Unavailable而非0伪事实 |
| P1-17 | `Partial` | Mesh校验attribute format/length、index range、topology multiple和morph length | 增加finite、weight range/normalization、joint range、IBM count与transform validation |
| P1-18 | `Open` | 无zero-area/duplicate triangle/non-manifold/winding/NaN quality report | 分离hard validation与quality diagnostics，定位primitive/triangle证据 |
| P1-19 | `Open` | normal/tangent可生成且plugin新增UV/tangent convergence test，但source-authored/derived、算法版本和参数不进入artifact | provenance写入recipe、cook key、artifact manifest和preview |
| P1-20 | `Open` | VG ordinal继续复用joint index slot 0/1，weighted primitive只能跳过该路径 | 独立vertex semantic/cluster remap buffer，禁止占用skinning channel |
| P1-21 | `Partial` | morph target有optional name和attribute-length验证；无stable ID、unique name、finite/range和reimport mapping | stable MorphTargetId、channel schema、weight policy与source mapping |
| P1-22 | `Partial` | Mesh已有asset usage与GPU preparation，但geometry recipe/platform cook/runtime residency未统一 | source/build/runtime usage分层并进入platform artifact与streaming policy |
| P1-23 | `Open` | OBJ读取MTL以完成parser但丢弃`tobj` material结果，未生成Material/Texture/section binding | OBJ normalized material graph、texture dependency、slot mapping与fallback diagnostic |
| P1-24 | `Partial` | Core glTF/WOC、OBJ fixture与plugin geometry convergence测试丰富；没有Core/plugin/format shared golden manifest | shared corpus比较subasset graph、payload digest、diagnostics与render golden |

## 8. P1 Skeleton、Skin、Animation Binding 与 Retarget

| ID | 状态 | 当前证据 | 必须重构为 |
|---|---|---|---|
| P1-25 | `Partial` | Animation插件可从bone path生成`AnimationTargetId`；Skeleton schema仍不持久化stable ID | Skeleton保存StableBoneId/source key，path只作显示与迁移信息 |
| P1-26 | `Partial` | target table检查empty/name/path、parent range/cycle/duplicate target，pose检查部分finite TRS；decode/import/cook未统一 | `SkeletonValidator`在import、decode、compile、cook前统一执行 |
| P1-27 | `Open` | Skin仍是Mesh内IBM vector；plugin另产generic Data sibling，没有Skeleton/joint引用 | 独立typed Skin asset持有Skeleton ref、joint map、IBM、mesh bind transform |
| P1-28 | `Open` | Core/plugin `mesh_skin_assets_by_mesh`都按首个node skin写Mesh metadata | skin属于node instance binding，同Mesh可关联任意多个Skin |
| P1-29 | `Open` | authored IBM被保存但renderer由Skeleton bind pose反推inverse | 冻结IBM/reference pose/mesh bind space数学合同，CPU/GPU共用qualified palette |
| P1-30 | `Partial` | Clip已有path `target_id`和skeleton-scoped target table；Runtime pose/renderer仍保留name fallback和name map | stable ID成为唯一运行绑定，legacy name只在migration层显式诊断 |
| P1-31 | `Open` | Core与plugin Scene均写`animation_skeleton: None`、`animation_player: None`，plugin Animation还是placeholder | normalized Scene节点安装typed Skeleton/Skin/player/clip selection |
| P1-32 | `Closed` | Editor production树对私有glTF animation derivation/写回路径为0；canonical subasset由Runtime transaction发布 | source guard保持删除状态，禁止host sibling文件写回 |
| P1-33 | `Open` | Skeleton没有socket/attachment schema、preview handle或scene binding | stable SocketId、bone binding、local transform、migration与toolkit操作 |
| P1-34 | `Open` | production无BoneMap/SkeletonProfile domain；同名Retarget UI不是资产 | typed profile/bone map、validation、auto-map suggestion与versioned artifact |
| P1-35 | `Open` | 无Retarget source asset、operation stack、solver、compiler或runtime config | source/target rig、pose、chain/op stack、compile artifact与backend contract |
| P1-36 | `Open` | Preview/Apply只返回`SK_Mannequin -> SK_Robot`和queued固定字符串 | 双角色同步preview、error metrics、transactional bake/apply与receipt |

## 9. P1 LOD、Collision、Derived Data 与运行时资格

| ID | 状态 | 当前证据 | 必须重构为 |
|---|---|---|---|
| P1-37 | `Open` | LOD是`SceneMeshInstanceAsset/MeshRenderer`上的引用列表，不属于Model/Mesh资产 | typed `MeshLodGroupSource`与per-platform cooked group |
| P1-38 | `Open` | selection只计算camera到entity translation距离，不使用bounds、size或projection | 基于qualified bounds与projected screen coverage，多view一致 |
| P1-39 | `Open` | Mesh LOD没有hysteresis/crossfade/dither；texture mip hysteresis不能替代 | per-view state、transition width/time、dither/crossfade与forced override |
| P1-40 | `Open` | 全仓无geometry reduction provider/cook service | deterministic bounded reduction provider、job/cancel/progress/DDC/quality report |
| P1-41 | `Open` | Core/plugin importer都写空LOD，Editor无custom LOD source/reimport | 每级source identity、mapping、reimport与material/section compatibility |
| P1-42 | `Open` | Scene LOD、texture streaming、VG cluster LOD与resource residency相互独立 | `GeometryResidencyPolicy`协调classic/VG/fallback/streaming/platform预算 |
| P1-43 | `Open` | 无CollisionSetup source schema或ResourceKind；同名ZUI不是domain | simple/convex/triangle/heightfield source、channel/profile与cook settings |
| P1-44 | `Open` | `PhysicsMeshAsset`只有runtime payload，production无Render Mesh converter | qualified Mesh generation到PhysicsMesh artifact的deterministic converter |
| P1-45 | `Open` | 无box/sphere/capsule fit、convex decomposition、hull edit或transaction | provider-neutral primitive/decomposition service和可撤销authoring |
| P1-46 | `Open` | Jolt runtime验证shape/body组合，Editor import/cook前不做target backend preflight | backend capability snapshot进入recipe/admission/cook receipt |
| P1-47 | `Partial` | Mesh SDF与PhysicsMesh在Runtime类型上分离；Editor无CollisionSetup/UI/debug artifact | UI、schema、cook key、debug draw与diagnostic明确区分两类artifact |
| P1-48 | `Partial` | VG/SDF typed cook request、source/config hash与artifact基础存在；LOD/Collision/Retarget不在统一invalidation graph | derived manifest记录source generation、recipe/provider/toolchain/platform和依赖边 |

## 10. P1 Toolkit、Preview、可观测性与规模资格

| ID | 状态 | 当前证据 | 必须重构为 |
|---|---|---|---|
| P1-49 | `Open` | Model只有type presentation，无toolkit | Model hierarchy/section/material/LOD/collision/reimport toolkit |
| P1-50 | `Open` | Mesh无toolkit | topology/attribute/UV/normal/tangent/morph/skin/VG/SDF inspection toolkit |
| P1-51 | `Open` | AnimationSkeleton无toolkit | hierarchy/reference pose/socket/profile/validation toolkit |
| P1-52 | `Open` | 无Geometry共享isolated Preview Scene | 统一world/camera/light/floor/selection/debug mode，不污染当前Level |
| P1-53 | `Open` | Texture用SourceImage，其余包括Model/Mesh/Skeleton默认placeholder | generation-qualified bounded render thumbnail provider |
| P1-54 | `Partial` | 通用preview job已有source digest key、stale comparison和cache；Geometry仍只生成placeholder | loading/ready/stale/failed/cancelled/unsupported状态与receipt |
| P1-55 | `Open` | VG Editor descriptor指向缺失`authoring.zui`，测试只验证字符串注册 | package content admission验证并加载真实template/controller |
| P1-56 | `Open` | Retarget/Collision有route/navigation/feedback，但没有domain command factory、WhenClause或transaction | toolkit command registry、context enablement、menu/palette和operation receipt |
| P1-57 | `Partial` | import result已有`LogSource::import` terminal projection；无parser span、subasset/derived/toolkit/runtime统一health snapshot | source/subasset/derived/toolkit/runtime统一health与可跳转诊断 |
| P1-58 | `Partial` | Editor cancel、generation fence、journal recovery、stale CAS和pre-arm terminal diagnostic已存在；几何fault matrix不完整 | missing sidecar/crash/disk-full/cancel/recovery/corrupt cache每项唯一终态 |
| P1-59 | `Partial` | WOC 949 GLB corpus及ignored microbench存在；无Editor/transaction RSS与large-scene预算验收 | 冻结source/decoded bytes、peak RSS、wall time、alloc、artifact size与preview latency |
| P1-60 | `Open` | 无Windows/Linux、Editor/Client/Server、render/physics provider联合资格矩阵 | 同recipe/digest/visual/perf/fault基线与平台差异receipt |

## 11. P2 高级能力

| ID | 状态 | 能力 |
|---|---|---|
| P2-01 | `Open` | USD layer/variant/payload/scene composition；必须建立在stable normalized scene上 |
| P2-02 | `Open` | CAD tessellation profile、unit/precision/assembly identity和native provider qualification |
| P2-03 | `Open` | content-addressed geometry processing graph，复用正式job/DDC而非工具按钮串联 |
| P2-04 | `Open` | non-destructive modifier stack与source rebase，不原地破坏可重导入Mesh |
| P2-05 | `Open` | UV unwrap/packing/lightmap channel、overlap/padding/texel-density资格 |
| P2-06 | `Open` | skin weight paint/prune/normalize/mirror及可撤销骨架编辑 |
| P2-07 | `Open` | mesh merge、instancing proxy和HLOD，由World Partition/HLOD owner编排 |
| P2-08 | `Open` | remote/distributed geometry cook，需immutable input和provider/toolchain attestation |
| P2-09 | `Open` | bounded runtime retarget、motion adaptation与profile hot swap |
| P2-10 | `Open` | stable ID驱动geometry/reimport diff、capture与团队审查 |
| P2-11 | `Open` | normalized scene writer、diagnostic API、stream reader与import plugin conformance kit |
| P2-12 | `Open` | 跨引擎复杂角色导入、LOD/Collision、retarget、reimport override任务基准 |

## 12. 当前Authority与断路

| 对象/表面 | 当前真实authority | 当前断路 | 目标authority |
|---|---|---|---|
| Quick Import | path string + Editor callback + Runtime job/log | picker/drop/paste与`.gltf`准入不一致 | typed Geometry Source admission |
| 资产发布 | Runtime compound durable transaction | receipt后自动Scene insert | ProjectImportReceipt +独立AddToScene command |
| Import settings | zmeta generic TOML | 无geometry schema/version/migration | Versioned GeometryImportRecipe |
| glTF | Core与plugin版本/输出语义不同 | priority/availability改变被选语义 | one canonical implementation |
| Model | reference-only root与dual-capable schema | overview只读inline | resolved qualified Model artifact |
| Skin | Mesh内IBM vector + generic Data | 无joint/Skeleton ref，multi-skin first-wins | independent typed Skin asset |
| Animation target | path ID compiler + legacy name fallback | Skeleton不持久化stable ID，renderer按name | stable target ID end-to-end |
| Scene animation | imported hierarchy | skeleton/player固定None | typed node Skin/Skeleton/player binding |
| LOD | per-instance `min_distance` | origin distance，无cook/crossfade | asset LOD group + per-view runtime state |
| Collision | PhysicsMesh runtime payload | 无source/cooker/toolkit/preflight | CollisionSetup + qualified cook artifact |
| Retarget | fixed ZUI/feedback | 无domain/runtime | Retarget source/op/compiler/receipt |
| Preview | generic placeholder cache | 无isolated geometry preview | shared qualified Preview Scene |
| Virtual Geometry Editor | descriptor字符串 | package缺template | content-qualified toolkit package |

## 13. 关联Failure状态

| Failure | 当前状态 | 本报告判定 |
|---|---|---|
| glTF empty texture source option | `Open` | typed fail-closed合同和managed compile未完成，不能宣称plugin provider可用 |
| WOC meshopt/WebP 949 GLB | `Open` | 源码路径存在，语料门和managed动态验收仍待完成 |
| Runtime04 glTF current compile errors | `Open` | source fixes不能替代当前managed validation |
| missing subasset parent fallback | `Open` | missing label必须fail closed，不能退回parent伪装成功 |
| glTF dependency catalog drift | source implemented / validation pending | workspace依赖收敛仍未成为GREEN |
| duplicate import admission/backpressure | source implemented / validation pending | shared flight和typed budget存在，固定返回与产品trace仍待验收 |
| import diagnostics log bridge | source complete / managed validation pending | 本轮把源码进展计入Partial，不提升为Closed |

## 14. 分层重构顺序

1. **M0 Truthfulness与Provider止血**：修复`.gltf`准入矛盾；隐藏/禁用fixture动作和缺模板surface；冻结canonical glTF provider；给reference-only overview、multi-skin/IBM、Scene binding建立失败测试。
2. **M1 Source/Recipe/Normalized Scene**：GeometryImportSource、dependency discovery、versioned recipe、selection preview、structured diagnostics、target/conflict policy和output manifest。
3. **M2 Model/Mesh authority**：硬切inline/reference，stable node/mesh/primitive/section/material IDs，resolved overview，finite/quality validation和normal/tangent/morph provenance。
4. **M3 Skeleton/Skin/Animation**：stable bone ID、统一validator、independent Skin/IBM/joint map、multi-skin instance、Scene player与CPU/GPU palette parity。
5. **M4 Toolkits/Preview**：Model/Mesh/Skeleton toolkit，共享isolated Preview Scene、真实thumbnail、stale/cancel/failure状态和显式Add to Scene。
6. **M5 LOD**：asset group、custom/generated sources、reduction provider、screen threshold、hysteresis/crossfade、multi-view与streaming/VG coordination。
7. **M6 Collision**：CollisionSetup、render-mesh converter、primitive/convex provider、PhysicsMesh artifact、backend preflight与debug preview。
8. **M7 Retarget**：SkeletonProfile/BoneMap、source/target rig、pose/operation stack、compiler、同步preview和transactional Apply。
9. **M8 Qualification**：Core/plugin/format semantic corpus、fault/recovery、large asset/RSS、Windows/Linux、Editor/Client/Server与render/physics provider矩阵。

M0-M3属于Runtime/Plugin共享原因，Editor只能消费合同；M4-M7由Editor提供authoring transaction与preview，但不得复制Runtime资源或compiler真值。

## 15. 32项验收门状态

| Gate | 状态 | 当前判定 |
|---|---|---|
| G01 | `Partial` | path edit、button route与import log存在；无picker/drop/paste typed request |
| G02 | `Fail` | import完成后仍自动修改当前Scene |
| G03 | `Partial` | OBJ+MTL可原子staging；glTF sidecar和OBJ texture discovery不完整 |
| G04 | `Fail` | 无完整versioned GeometryImportRecipe |
| G05 | `Fail` | 无selection preview/output manifest对照 |
| G06 | `Partial` | compound durable publication结构成立；本轮未执行fault动态门 |
| G07 | `Fail` | Core/plugin glTF语义、version和priority仍分裂 |
| G08 | `Partial` | registry有capability/availability/priority，产品入口不展示selection/profile |
| G09 | `Fail` | Model dual/empty geometry authority仍合法 |
| G10 | `Fail` | referenced Mesh overview继续写0伪事实 |
| G11 | `Fail` | 无stable material section/slot ID |
| G12 | `Partial` | 基础format/length/index/topology校验存在，finite/skin/morph质量矩阵缺失 |
| G13 | `Fail` | normal/tangent provenance与visual golden缺失 |
| G14 | `Fail` | VG仍占joint semantic |
| G15 | `Partial` | target compiler检查parent/name/cycle且pose检查finite；import/decode/cook未统一 |
| G16 | `Fail` | Skin不引用Skeleton/joint map，IBM合同未冻结 |
| G17 | `Fail` | multi-skin仍first-wins |
| G18 | `Fail` | renderer不消费authored IBM |
| G19 | `Partial` | path target ID和duplicate diagnostic存在；Skeleton ID不持久、rename仍不稳定 |
| G20 | `Fail` | imported Scene无Skeleton/Skin/player选择 |
| G21 | `Pass` | Editor私有sibling glTF parse/write生产路径已删除，subasset由Runtime正式事务发布 |
| G22 | `Fail` | 无asset LOD group/custom-generated/screen/crossfade/platform policy |
| G23 | `Fail` | LOD仍按origin distance，未证明CPU/GPU/multi-view一致 |
| G24 | `Fail` | 无mesh reduction provider |
| G25 | `Fail` | 无CollisionSetup及render-to-PhysicsMesh cook |
| G26 | `Fail` | backend不支持只在runtime conversion暴露，authoring/cook前不阻断 |
| G27 | `Partial` | Runtime类型区分SDF/PhysicsMesh；Editor schema/UI/debug/cook未闭合 |
| G28 | `Fail` | Retarget只有fixture，无asset/solver/receipt |
| G29 | `Fail` | Model/Mesh/Skeleton toolkit与isolated Preview Scene缺失 |
| G30 | `Partial` | thumbnail key/stale基础存在；Geometry仍placeholder且无完整状态/预算 |
| G31 | `Partial` | cancel/generation fence/journal recovery/stale CAS/terminal diagnostic存在；fault matrix未覆盖 |
| G32 | `Fail` | 跨平台/产品形态/provider visual/perf矩阵未建立 |

总计：**21 Fail / 10 Partial / 1 Pass**。`Pass`只表示G21当前静态源码合同成立，不代表Geometry链动态验收完成。

## 16. 禁止的临时修补

1. 不得把path callback、import log或transaction receipt单独包装成“Geometry Import已完成”。
2. 不得继续在成功import后隐式修改Scene，也不得用Undo补丁掩盖资产与场景事务耦合。
3. 不得通过调priority、version或profile默认值掩盖两套glTF语义；必须删除重复authority。
4. 不得在Model overview中把未解析reference当0，也不得继续保存无验证的inline+reference双真值。
5. 不得从Skeleton local pose反推并替代authored IBM，不得以first skin或bone name fallback作为正式合同。
6. 不得把texture mip hysteresis、VG cluster LOD或Mesh SDF改名冒充Mesh LOD/Collision产品。
7. 不得给Collision/Retarget按钮接后台job后继续返回固定样例；job必须产出typed artifact与receipt。
8. 不得用placeholder thumbnail、静态ZUI、descriptor注册测试、ignored microbench或少量geometry convergence测试声称toolkit/quality/performance通过。

## 17. 本轮验证边界

本轮只完成review、当前源码状态重判、参考实现对照、分层重构计划与静态物理范围冻结；没有修改生产代码，没有运行Cargo、真实Editor、Importer corpus/fuzz、skinned render、LOD/crossfade、collision cook、retarget solver、VG template、large asset/RSS、跨平台或provider组合动态测试。已有failure记录中的“source complete / managed validation pending”只作为源码现状线索，不当作GREEN。

Editor209刷新Editor32/106/153，不增加canonical finding总数。当前结果为P0 **4 Open / 1 Partial**，P1 **38 Open / 20 Partial / 2 Closed**，P2 **12 Open**。实施前必须重新冻结共享dirty owner和物理manifest，并按M0开始；Tooling继续按用户要求排除。
