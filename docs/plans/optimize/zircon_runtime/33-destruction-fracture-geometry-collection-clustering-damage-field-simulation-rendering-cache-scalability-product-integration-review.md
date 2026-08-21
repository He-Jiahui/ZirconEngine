---
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/mesh
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/scene/components/scene/physics.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract
  - zircon_runtime/src/graphics/feature/builtin_render_feature
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
  - zircon_runtime/src/graphics/visibility
  - zircon_plugins/physics/runtime/src/backend
  - zircon_plugins/physics/runtime/src/constraint
  - zircon_plugins/physics/runtime/src/manager
  - zircon_plugins/physics/runtime/src/skeletal
  - examples/woc/scripts/woc_game/src/instances/delve_dynamic_collision_rules.zr
tests:
  - zircon_runtime/src/core/framework/physics/tests.rs
  - zircon_runtime/src/core/framework/render/frame_extract/tests.rs
  - zircon_runtime/src/asset/tests/assets/mesh.rs
  - zircon_runtime/src/asset/tests/assets/mesh
  - zircon_runtime/src/asset/tests/assets/scene/physics_animation.rs
  - zircon_runtime/src/scene/tests/physics_animation_components.rs
  - zircon_plugins/physics/runtime/src/backend/tests.rs
  - zircon_plugins/physics/runtime/src/backend/tests
  - zircon_plugins/physics/runtime/src/manager/tests.rs
  - zircon_plugins/physics/runtime/src/skeletal/tests.rs
  - zircon_plugins/physics/runtime/src/tests.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs
  - zircon_runtime/src/graphics/tests/visibility.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/29-terrain-landscape-heightfield-quadtree-lod-material-layer-foliage-world-partition-physics-navigation-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/31-cloth-fabric-soft-body-garment-simulation-collision-deformation-rendering-wind-lod-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/18-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/Chaos/Public/GeometryCollection
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/Chaos/Public/PhysicsProxy/GeometryCollectionPhysicsProxy.h
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/Chaos/Private/PhysicsProxy/GeometryCollectionPhysicsProxy.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/GeometryCollectionEngine/Public/GeometryCollection
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/GeometryCollectionEngine/Private/GeometryCollection
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/FieldSystem/Source/FieldSystemEngine
  - dev/UnrealEngine/Engine/Plugins/Experimental/Fracture/Source/FractureEngine
  - dev/UnrealEngine/Engine/Plugins/Experimental/ChaosCaching/Source/ChaosCaching
  - dev/bevy/crates
  - dev/Fyrox
  - dev/godot
  - dev/Graphics/Packages/com.unity.shadergraph
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 33 · Destruction、Fracture、Geometry Collection、Clustering、Damage Field、Simulation、Rendering、Cache、Scalability 与 Product Integration 工程化差距

## 1. 结论

Zircon 当前没有 Destruction/Fracture/Geometry Collection 运行时产品。对 Runtime、Runtime Interface、Plugins、Editor、App 与 Hub production 树执行精确搜索，`GeometryCollection`、`Voronoi`、`DamageThreshold`、`Destructible`、`Fracture`、`Shatter`、`BreakingEvent`、`ExternalClusterStrain` 与 `InternalClusterStrain` 均为零命中；`ResourceKind`、`SceneEntityAsset`、`PhysicsWorldSyncState`、`PhysicsBackend`、`RenderFrameExtract` 和 `BuiltinRenderFeature` 也没有破坏资源、piece/cluster identity、strain/damage、breaking event、逐片变换或专用渲染 feature。

现有 Mesh、刚体、Collider、Joint、Jolt bridge、contact/trigger、GPU Scene、static batch、shadow 与 velocity 是真实通用基础，但无法拼接成工程级破坏系统。Scene 每个实体最多持有一份 mesh/body/collider/joint；物理 backend 只创建单 body/shape/constraint，输出按现有 entity 写回；contact event 甚至不含 impulse、subshape、face 或 material。渲染侧一条 `RenderMeshSnapshot` 对应一个 entity transform，`GpuMeshResource` 只有普通 vertex/index buffer，没有 piece bone map、层级 transform palette、可见 piece mask、current/previous piece data或断裂后bounds。

WOC 示例里的 `destructible_wall` 只是按 `objectHealth > 0` 返回3.2或0.0的坐标碰撞半径；`destruction`、`Mindfracture`、`shatter` 是职业、技能或文案标识。它们没有生成碎片、聚类刚体、断裂事件或逐片渲染，不构成引擎能力证据。本篇登记 **0 P0 / 72 P1 / 16 P2**。0 P0 不代表接近完成：Runtime08A 已把 destruction 列为“尚无 owner”的 P2 范围，且当前 catalog 未宣称 Ready，本篇不重复制造 truthfulness P0。新 owner 必须形成 `DestructionSourceAsset -> deterministic FractureBuildArtifact(pieces/interiors/collision/hierarchy/connection graph/LOD) -> DestructionRuntimeInstance -> ClusteredRigid Provider -> Damage/Field/Contact ingress -> Break/Removal/Transform output -> Render/Navigation/Audio/VFX/Network/Cache adapters -> typed receipt`；在闭环完成前，禁止把换模型、隐藏静态墙、spawn若干刚体或一组breakable joints宣称为工程级破坏。

## 2. 审查边界、方法与 currentness

### 2.1 冻结输入

冻结语料为249个文件、66,275行、2,486,488 bytes：136个 Zircon production 文件为15,479行、541,011 bytes；17个 focused test文件为4,285行、150,553 bytes；96个参考文件为46,511行、1,794,924 bytes。指纹算法为按 forward-slash 相对路径排序，逐文件计算小写 SHA-256，形成 `path|file_sha256` 行，以单个 LF 连接且无末尾 LF，再对 UTF-8 payload计算SHA-256；结果为 `230c11c6324a7d2c294b415a39d92a8fc14245e488e915ccc65c4795e8a65392`。

冻结基线为 `main@25e09a23178000f2e783ce2143cf70a8b118d404`，按读取时 working bytes计算。`frame_extract.rs` 与 `frame_extract/geometry.rs` 虽被状态标记为 modified，但 working blob与HEAD blob相同；本篇没有修改或归因production代码。实施前必须重导249项manifest、重算指纹并重查所有在途文件，不能把本篇结论当作未来代码的永久快照。

### 2.2 纵向检查链

本轮逐层检查 source/import -> stable piece/face/transform identity -> authoring fracture -> interior surface/material -> hierarchy/cluster/connection graph -> convex/collision cook -> deterministic artifact/DDC -> Scene persistence -> instance lifecycle -> physics proxy/admission -> damage/contact/field/strain -> cluster break/activation/removal -> event filtering -> current/previous piece transform output -> bounds/visibility/LOD/streaming -> mesh/Nanite/RT/shadow/velocity -> cache record/playback -> gameplay/navigation/audio/VFX/network -> diagnostics/scalability/tests/product evidence。

17个 focused test文件共有84个 `#[test]`/`#[ignore]`/`#[tokio::test]` 属性，只覆盖普通mesh schema/roundtrip、单body/collider/joint、Jolt/builtin contract、contact/trigger/query、frame extract、mesh draw与generic visibility。没有 fracture determinism、piece identity、hierarchy validation、collision cook、cluster strain/break、damage propagation、field、breaking/removal event、piece transform rendering、cache replay或产品场景测试。

### 2.3 参考搜索边界与动态验证限制

Unreal GeometryCollection、Chaos、FractureEngine、FieldSystem与ChaosCaching提供本域正参考。对当前 `dev/bevy/crates`、`dev/Fyrox` 与 `dev/godot` 做文件名搜索，没有 destruction/fracture/voronoi/geometry-collection production 模块；Godot 的 `variant_destruct` 和 GDScript `self_destruction` 是对象销毁语义。Unity Graphics命中的Voronoi仅为Shader Graph/VFX噪声节点和测试资产，不是几何断裂。上述均只作为本地镜像负证据，不外推为完整生态结论，也不能降低Zircon的目标。

本轮是 E3 source-level review，没有运行Cargo、Jolt、WGPU、GPU capture或破坏场景。仓内没有可执行 destruction asset/type/backend/render path，运行普通rigid body或mesh tests不能证明缺失产品。本篇把动态门留给实现里程碑，不能把“未运行”写成通过。

## 3. 当前可保留的真实基础

1. `MeshAsset` 的typed attributes、indices、materials、bounds与LOD可承载piece surface输入；fracture artifact仍需独立保存piece、interior face、hierarchy和connection identity。
2. Scene已能持久化单体mesh、rigid body、collider、joint与asset references，可作为component投影底座；不能把数千piece展开成authoring实体并以entity ID替代piece ID。
3. Physics已有backend-neutral body/shape/constraint contract、Jolt world、fixed-step入口、query、contact/trigger和有界command queue，可作为clustered rigid provider的下层依赖。
4. ConvexHull、TriangleMesh与Compound shape提供collision primitive语言；它们没有per-piece cook artifact、implicit fallback、size-specific policy或cluster sharing。
5. GPU mesh、GPU Scene、indirect draw、visibility、shadow和velocity可以复用资源/提交框架；destruction需要独立piece transform、visibility、bounds、history和retirement合同。
6. Mesh material section与普通PBR可承载外表面；fracture compiler必须生成可追踪的interior surface、UV/tangent/material mapping，而不是运行时随意补色。
7. Runtime22/23/24提供clock、space/unit、generation/epoch治理方向；Destruction必须消费这些owner，不得私建第二套时间、坐标或无代际handle。
8. Runtime04/05/08A/09B/09D已有asset、world、physics、visibility与streaming差距owner；本篇只定义destruction domain及adapter，不复制通用基础设施。

## 4. 当前代码事实与断路

### 4.1 Asset、Scene、Physics 与 Event

1. `ResourceKind` 没有DestructionSource、FractureArtifact、GeometryCollection、DestructionCache或DamageProfile。
2. `SceneEntityAsset` 只有单份`mesh`、`rigid_body`、`collider`与`joint`，没有collection asset、initial state、damage、cluster、cache或removal policy。
3. `PhysicsWorldSyncState` 只有`bodies/colliders/joints/materials`四个数组，没有collection/proxy/particle hierarchy、activation、strain或piece output。
4. `PhysicsBackend`只提供shape/body/constraint create/destroy、body command、step、query与event drain；没有batch cluster creation、field command、damage ingress、break/removal或collection result。
5. `PhysicsBodyCommand`只有velocity、force、impulse、teleport、body type、CCD和sleep；命令以scene entity为目标，无法寻址collection/piece/cluster或携带falloff/field metadata。
6. `PhysicsContactEvent`只有world、两entity、point和normal；缺impulse、relative velocity、subshape/piece、face/material、solver tick与generation，不能稳定驱动damage。
7. Joint可表达固定/距离/铰链等普通约束，但没有connection graph、per-edge strength、cluster hierarchy、accumulated strain或atomic break output。
8. Jolt world以entity为key同步并把active state写回同一body/collider；重建实体会销毁并重建shape/body，不能作为数千piece每帧生命周期策略。

### 4.2 Rendering、Temporal 与 Product

1. `RenderFrameExtract`只有ordinary meshes/poses/particles/visibility等，没有collection instance、piece transforms、visible-piece set、break epoch或dynamic bounds。
2. `GeometryExtract`按`RenderMeshSnapshot`构建phase与static batch，batch成员是mesh index/entity；没有piece subsection、cluster draw range或变换palette。
3. `MeshDraw`持有单mesh、单stable instance key和可选skinned palette；现有skinning语义是骨骼动画，不是piece hierarchy或break activation。
4. `GpuMeshResource`只创建`VERTEX`/`INDEX` buffer；没有bone-map/piece-map SRV、dynamic transform storage、current/previous palette、piece mask或interior section metadata。
5. `BuiltinRenderFeature`没有GeometryCollection/Destruction；普通Mesh/SkinnedMesh/VirtualGeometry不能表达断裂后的逐片状态与资格。
6. visibility只消费mesh bounds和instance identity；静止整体bounds、逐片运动bounds、cluster LOD、occlusion与piece removal没有更新合同。
7. shadow/velocity从普通mesh draw和entity/skinned previous data取数；破坏瞬间新激活piece、teleport、cache seek与remove没有history语义。
8. WOC `destructible_wall`只开关3.2半径，健康归零后不产生碎片、碰撞body、render output或event receipt；它是游戏规则占位，不是runtime destruction。

## 5. 参考实现给出的工程边界

### 5.1 GeometryCollection：破坏资产是带依赖关系的typed collection

Unreal `FGeometryCollection`将Vertices、Faces、Geometry、Transform、Breaking和Material分组，显式保存vertex bone map、face visibility/material、geometry bounds/range、transform hierarchy与simulation flags。`ManagedArrayCollection`对group内等长typed attribute、跨group index dependency、remove/merge/reorder后的重映射负责；Hierarchy、ConnectionGraph、Collision、Anchoring、RemoveOnBreak与Rendering facade进一步隔离语义。Zircon应吸收typed group、stable semantic ID、dependency-aware remap与validation，不复制FName/UObject或任意attribute无约束扩张。

### 5.2 Fracture/Cook：authoring结果必须是确定性、可版本化artifact

FractureEngine覆盖Voronoi、plane、slice、brick、mesh cutter、island split、grout/noise、internal material、collision samples、selection与多种clustering；输入含random seed，cluster可按number/fraction/size/grid并约束connectivity、isolated piece与convexity。GeometryCollection object/cooker还处理size-specific collision、convex/implicit数据、render data和derived-data版本。这里的关键不是功能数量，而是source mutation与runtime artifact分层、随机性可复算、interior/collision/hierarchy一起编译、失败不发布半成品。

### 5.3 Chaos Proxy/Field：运行时是clustered particle系统，不是一袋普通刚体

`FGeometryCollectionPhysicsProxy`维护transform到particle映射、internal cluster identity、game/simulation thread buffer、field command和dirty result拉取；simulation type明确支持user damage threshold或material-strength/connectivity模型，以及external/internal cluster strain。FieldSystem区分construction、transient和persistent command并路由到solver/world。Zircon需要generation-qualified collection proxy、批量创建、cluster activation/break、damage/field admission和结果buffer；禁止每次break逐实体spawn并靠普通joint轮询模拟。

### 5.4 Rendering/Events/Cache：逐片输出必须贯通所有consumer

GeometryCollection render data包含bone map、外/内表面section、subsection、pre-skinned bounds、Nanite与RT资源；scene proxy维护dynamic transform buffer、current/previous transform buffer、transform-to-instance mapping和per-section materials。ChaosCaching按particle记录transform/velocity/curve，保存breaking/collision/trailing/enable事件，并用record/playback token、pending frame queue、compression与version治理生命周期。Zircon要建立一次权威piece output，供raster/shadow/velocity/RT、event、cache、audio/VFX/nav和network按同一tick/generation消费。

### 5.5 负参考的正确使用

Bevy、Fyrox、Godot当前本地树没有本域独立production模块，Unity Graphics只有视觉Voronoi噪声。它们可以继续作为普通ECS、rigid body、render graph和material底座参考，但不能用“其他中型引擎也没有”降低用户要求。正向系统边界以Unreal本地源码为主要证据，最终是否超过Unreal必须由同资产、同场景、同硬件、同画质的raw receipt证明。

## 6. 目标架构与唯一 Owner

```text
Mesh/Destruction Source + Import Provenance
  -> schema migration + topology/manifold/material validation
  -> deterministic Fracture Compiler
       -> Piece Geometry + Interior Surface Artifact
       -> Stable Hierarchy + Connection Graph + Damage Metadata
       -> Convex/Implicit/Query Collision Artifact
       -> Render/LOD/Streaming/RT Metadata
  -> generation-qualified DestructionRuntimeInstance
  -> Clustered-Rigid Provider + Damage/Field/Contact Ingress
  -> Break/Activation/Removal/Event + current/previous Piece Output
  -> Render / Nav / Audio / VFX / Network / Cache adapters
  -> terminal lifecycle, degradation and execution receipts
```

| 领域 | 唯一 owner | Destruction33 只消费/提供 |
|---|---|---|
| Resource/schema/artifact/DDC | Runtime04 | typed source/build/cache kind、dependency、install/retire receipt |
| Scene/ECS/world lifecycle | Runtime05 | component identity、instance generation、world teardown、serialization |
| Fixed clock/rigid backend/query | Runtime08A + Runtime22 | solver tick、body/shape/query/contact capability；本篇新增clustered provider adapter |
| Destruction domain | 新 Runtime Destruction owner | fracture compiler、piece/hierarchy/graph、instance、damage/field、break/removal/output |
| Stable identity/space | Runtime23 + Runtime24 | unit/space/origin与generation/epoch primitive |
| Visibility/GPU Scene | Runtime09B | piece/cluster bounds、LOD/culling、indirect work与visibility receipt |
| Material/shader/PSO | Runtime09C | exterior/interior section、variants与pipeline qualification |
| Residency/streaming | Runtime09D | artifact bundle admission、atomic install、pressure/retire policy |
| Shadow/temporal/RT | Runtime09E + Runtime09H1 + Runtime28 | current/previous output、shadow、velocity/history与RT update/fallback |
| Navigation/audio/VFX/network/gameplay | 各域owner | 消费typed break/removal/damage snapshot，不反向拥有piece truth |
| Authoring/preview | Editor18 + Editor32后续Destruction owner | fracture、cluster、collision、damage、preview共享runtime compiler |

Destruction可以复用ordinary mesh、rigid collision、fixed clock、GPU resource和streaming primitives，但piece identity、fracture artifact、cluster hierarchy、strain/damage、breaking/removal与piece output必须由一个domain owner统一。禁止由Physics、Renderer、Editor和Gameplay各自维护一套碎片列表或damage truth。

## 7. P1：Source、Fracture、Schema 与 Compiler

| ID | 差距 | 必须重构 |
|---|---|---|
| DEST-P1-001 | 无Destruction资源身份 | 新增Source、BuildArtifact、Cache、Damage/Material Profile kind与versioned handle |
| DEST-P1-002 | 无独立source asset | 建`DestructionSourceAsset`，分离原mesh/authoring graph与cooked runtime collection |
| DEST-P1-003 | 无stable piece/face/transform ID | identity跨fracture重算、reimport、cluster、LOD、cache和network保持稳定 |
| DEST-P1-004 | 无typed collection schema | piece geometry、vertices/faces、transform、hierarchy、material、visibility和simulation group有cardinality |
| DEST-P1-005 | 无source拓扑与单位合同 | manifold/orientation/degenerate/self-intersection、space、meters与precision明确验证 |
| DEST-P1-006 | 无authoring fracture graph | cutter、selection、settings、seed、dependency与输出节点成为可序列化typed graph |
| DEST-P1-007 | 无deterministic Voronoi/plane/slice cutter | 相同source/settings/seed产生稳定piece identity、拓扑、排序和digest |
| DEST-P1-008 | 无island/grout/noise/repair政策 | split、gap、surface noise、weld/bridge与repair都有误差界、diagnostic和provenance |
| DEST-P1-009 | 无interior surface生成 | 内表面拓扑、UV/tangent、material slot、seam和collision source与cut同步编译 |
| DEST-P1-010 | 无hierarchy/cluster/connection compiler | parent/children/level、adjacency、contact area、edge strength和anchor形成一致artifact |
| DEST-P1-011 | 无collision cook family | per-piece convex/implicit/query shape、size-specific policy、fallback和共享数据确定性cook |
| DEST-P1-012 | 无DDC/migration/LKG/receipt | schema/artifact独立version，记录source/dependency/compiler/settings/target/hash/cost/repair |

## 8. P1：Scene、Instance、Identity 与 Lifecycle

| ID | 差距 | 必须重构 |
|---|---|---|
| DEST-P1-013 | 无Destruction Scene component | 持有artifact、materials、initial dynamic state、damage、cache、LOD和replication policy |
| DEST-P1-014 | 无collection runtime instance | 建world/entity/artifact generation、provider proxy、piece state、resources与terminal receipt |
| DEST-P1-015 | 无piece/cluster runtime handle | stable ID与slot/generation分离，删除、合并、重建、stale访问可检测 |
| DEST-P1-016 | 无root/piece/cluster transform合同 | local/rest/world/mass-space与parent composition明确，不能以entity transform替代piece truth |
| DEST-P1-017 | 无initial state与activation | static/kinematic/dynamic/sleeping/anchored/disabled和activate-on-damage类型化 |
| DEST-P1-018 | 无严格生命周期 | Requested/Preparing/Active/Suspended/Retiring/Failed/Cancelled每ticket唯一终态 |
| DEST-P1-019 | 无async build/create ownership | task携world/instance/artifact/provider/tick generation，cancel/panic后旧结果不能发布 |
| DEST-P1-020 | 无atomic publication | geometry/collision/hierarchy/render bundle全成后一次install，禁止半集合可见 |
| DEST-P1-021 | 无reload/reimport迁移 | stable piece mapping、state/cache保留条件、LKG与不可迁移理由明确 |
| DEST-P1-022 | 无multi-world/PIE isolation | 相同asset在preview、PIE和多个world中不得共享mutable damage/cluster truth |
| DEST-P1-023 | 无world replace/unload drain | solver task、event、GPU resource、cache writer和consumer lease按序收口 |
| DEST-P1-024 | 无save/snapshot语义 | 区分source配置、initial state、runtime damage state与可选checkpoint，不序列化raw pointer/slot |

## 9. P1：Clustered Physics、Damage、Field 与 Break

| ID | 差距 | 必须重构 |
|---|---|---|
| DEST-P1-025 | 无clustered-rigid provider合同 | 定义batch create、hierarchy、activation、strain、break、results、limits与capability admission |
| DEST-P1-026 | 无deterministic CPU oracle | 建小collection baseline验证compiler、damage、cluster、cache和native differential |
| DEST-P1-027 | 无batch particle/shape creation | 按artifact批量建立piece/cluster并共享collision数据，避免逐实体同步与分配风暴 |
| DEST-P1-028 | 无cluster hierarchy simulation | internal cluster、parent/children、union mass/inertia、decluster/recluster状态可追踪 |
| DEST-P1-029 | 无connection graph运行时 | edge strength、contact area、material、broken state和graph connectivity由单一owner维护 |
| DEST-P1-030 | 无damage model | user threshold与material-strength/connectivity模型、单位、累积/衰减和clamp类型化 |
| DEST-P1-031 | contact缺damage输入 | 增加impulse、relative velocity、piece/subshape、material、tick/generation和dedupe语义 |
| DEST-P1-032 | 无external/internal strain | 支持point/radial/plane/volume等批准field、falloff、metadata、budget与deterministic ordering |
| DEST-P1-033 | 无anchor/kinematic field | construction、persistent、transient field有owner、lifetime、remove与world isolation |
| DEST-P1-034 | 无atomic break transaction | 阈值评估、graph split、cluster activation、mass/state、event/output在同tick原子提交 |
| DEST-P1-035 | 无remove-on-break/sleep/disable政策 | time、distance、size、speed、visibility和gameplay retention有typed state与receipt |
| DEST-P1-036 | 无overload/fault保护 | piece/contact/field/break预算、NaN/energy guard、rollback/LKG、provider fault与degrade可观察 |

## 10. P1：Rendering、Visibility、Temporal、Shadow 与 RT

| ID | 差距 | 必须重构 |
|---|---|---|
| DEST-P1-037 | 无collection render artifact | 编译bone/piece map、exterior/interior sections、subsections、bounds、LOD与RT metadata |
| DEST-P1-038 | 无piece transform GPU resource | typed storage/SRV按instance generation持有current/previous matrix与active mask |
| DEST-P1-039 | 无transform-to-draw mapping | piece/cluster/section映射稳定、可压缩并与physics output generation一致 |
| DEST-P1-040 | 无dynamic resource pool/retirement | transform/mask/indirect/bounds buffer按fence与device generation回收 |
| DEST-P1-041 | 无dynamic/cluster bounds | root、cluster、piece bounds分层更新，fast motion/readback latency有保守fallback |
| DEST-P1-042 | 无piece/cluster visibility | frustum/HZB/LOD/occlusion消费动态bounds，active/remove/internal state进入culling |
| DEST-P1-043 | 无collection indirect submission | section/material/visible piece生成有界indirect work，overflow/degrade有receipt |
| DEST-P1-044 | 无interior material parity | 新暴露cut face的depth/GBuffer/forward/shadow/RT material与UV/tangent一致 |
| DEST-P1-045 | 无current/previous temporal语义 | activation、break、teleport、cache seek、LOD与remove分别定义velocity/history reset |
| DEST-P1-046 | 无shadow逐片更新 | shadow caster bounds、piece transform、LOD和internal face policy与main visibility同代 |
| DEST-P1-047 | 无RT geometry策略 | per-piece/cluster BLAS、instance TLAS、refit/rebuild、compaction、motion与fallback受预算治理 |
| DEST-P1-048 | 无Nanite/virtual geometry adapter | dynamic piece transform、hierarchy/cluster LOD、residency、fallback与non-Nanite parity明确 |

## 11. P1：Events、Cache、Gameplay 与 Cross-System Integration

| ID | 差距 | 必须重构 |
|---|---|---|
| DEST-P1-049 | 无typed breaking event | event含collection/piece/cluster IDs、tick、location、orientation、velocity、mass、bounds和cause |
| DEST-P1-050 | 无collision/trailing/removal event | event过滤、阈值、排序、容量、overflow、dedupe和consumer cursor明确 |
| DEST-P1-051 | 无event generation/lease | stale proxy、world replace、replay/cache切换后旧event不能污染新instance |
| DEST-P1-052 | 无cache schema | per-piece transform/velocity/state、events、artifact/provider/tick generation与timebase版本化 |
| DEST-P1-053 | 无cache record transaction | physics线程pending frame、single recorder token、cancel/finalize、compression和atomic publish |
| DEST-P1-054 | 无cache playback/seek | 插值、event cursor、rest reset、loop/reverse、missing track和live/cache切换可验证 |
| DEST-P1-055 | 无cache streaming/residency | chunk prefetch/map/unmap、budget、in-flight drain、pressure eviction与corruption fail-close |
| DEST-P1-056 | 无gameplay damage adapter | gameplay提交typed damage/field request并读取receipt，不能直接写piece/cluster内部状态 |
| DEST-P1-057 | 无navigation adapter | break后obstacle/nav dirty region、debris policy、rate limit与generation一致 |
| DEST-P1-058 | 无audio/VFX adapter | breaking/collision/trailing按预算聚合，material/energy/size参数来自同一event truth |
| DEST-P1-059 | 无network/rollback policy | authority、event/state replication、late join、checkpoint、prediction与cosmetic debris边界明确 |
| DEST-P1-060 | 无picking/query identity | ray/overlap/hit返回collection/piece/cluster/face/material stable identity与state generation |

## 12. P1：LOD、Streaming、Scalability、Diagnostics 与 Product Qualification

| ID | 差距 | 必须重构 |
|---|---|---|
| DEST-P1-061 | 无simulation LOD | distance/screen importance/visibility/gameplay决定cluster depth、active pieces与update rate |
| DEST-P1-062 | 无render/collision LOD parity | geometry、collision、query、shadow和RT LOD映射与切换误差显式化 |
| DEST-P1-063 | 无debris virtualization | 小碎片的sleep/remove/aggregate/FX-only策略保留碰撞与gameplay正确性边界 |
| DEST-P1-064 | 无artifact chunk/streaming | geometry/collision/hierarchy/render/cache按依赖bundle拆chunk并原子install |
| DEST-P1-065 | 无global budget/admission | pieces/clusters/contacts/fields/events/draws/bytes/CPU/GPU time决定拒绝或质量降级 |
| DEST-P1-066 | 无platform capability matrix | backend batch/cluster、storage/indirect/RT与memory限制映射到Supported/Degraded/Unsupported |
| DEST-P1-067 | 无debug snapshot | 可解释artifact/instance/provider、cluster tree、strain、broken edges、pieces、events、memory/time |
| DEST-P1-068 | 无fault/fuzz矩阵 | malformed mesh、bad hierarchy、stale IDs、OOM、cancel、device loss、provider crash/unload、cache损坏 |
| DEST-P1-069 | 无asset/compiler tests | fracture determinism、interior、identity/remap、hierarchy/graph/collision cook/migration golden为空 |
| DEST-P1-070 | 无physics/render/cache tests | strain/break、piece output、velocity/shadow/RT、event与cache replay differential为空 |
| DEST-P1-071 | 无真实产品场景 | 建wall/pillar/glass/clustered building、repeat damage、large pile、save/reopen/play/export/capture链 |
| DEST-P1-072 | 无跨引擎超越基准 | 同资产/seed/场景/硬件/画质比较fracture error、CPU/GPU、内存、stutter和raw receipt |

## 13. P2：完整性与长期竞争力

| ID | 后续能力 | 前置条件 |
|---|---|---|
| DEST-P2-001 | runtime procedural fracture | deterministic offline compiler、budget、transaction与authoritative fallback完成 |
| DEST-P2-002 | stress propagation/FEM coupling | connection/damage oracle、units、stability和performance gate完成 |
| DEST-P2-003 | plastic deformation/crumbling | dynamic topology、mass/collision recook、render remap与cache/network合同完成 |
| DEST-P2-004 | material-aware anisotropic fracture | physical material strength、grain field、compiler golden与authoring完成 |
| DEST-P2-005 | glass crack propagation | thin-sheet topology、crack identity、optical material、collision与temporal完成 |
| DEST-P2-006 | wood splinter/laminated fracture | directional material、fiber topology、LOD和debris policy完成 |
| DEST-P2-007 | terrain/voxel destruction | Terrain29 ownership、partition、nav/collision recook与streaming transaction完成 |
| DEST-P2-008 | fluids/fire/weather coupling | Water/Weather/VFX/gameplay adapter与damage authority稳定 |
| DEST-P2-009 | cluster union/reclustering at runtime | stable identity、mass/inertia、graph、network/cache和atomic transaction完成 |
| DEST-P2-010 | GPU fracture/cluster solver | CPU oracle、deterministic bounds、readback/fence、fault isolation和portable fallback完成 |
| DEST-P2-011 | deterministic rollback destruction | fixed tick、state digest、checkpoint、bandwidth和late join完成 |
| DEST-P2-012 | large-world partitioned destruction | origin/rebase、cell ownership、cross-cell cluster和stream continuity完成 |
| DEST-P2-013 | plugin cutter/damage/provider SDK | ABI/version/capability/budget/sandbox/unload与artifact compatibility完成 |
| DEST-P2-014 | collaborative fracture authoring | stable semantic IDs、transaction/merge/locking/recovery和source provenance完成 |
| DEST-P2-015 | ML-assisted fracture/LOD | authoritative deterministic fallback、training provenance和error bound完成 |
| DEST-P2-016 | distributed qualification farm | frozen corpus、artifact digest、physics/render capture与raw receipt归档完成 |

## 14. 分层重构里程碑

### M0 · Truth、Owner 与 Baseline

- 维持Destruction Unsupported，冻结identifier/caller/capability清单；禁止换模型、隐藏物体、普通joint或spawn刚体false-ready。
- 批准domain owner、术语、单位、piece/cluster identity、性能预算、CPU oracle和reference scenes。

### M1 · Source Schema、Fracture 与 Compiler

- 建Source、typed collection、stable IDs、topology/material validation和versioned authoring graph。
- 完成deterministic cutter、interior surface、hierarchy/connection graph、collision cook、DDC/LKG和fuzz。

### M2 · Scene、Instance 与 Lifecycle

- 接Scene persistence、component、runtime instance、initial state、atomic bundle install和save/snapshot。
- 完成cancel/stale/reload/multi-world isolation、world replace/unload drain与terminal receipt。

### M3 · Clustered Physics 与 Damage/Field

- 实现provider contract、batch create、cluster hierarchy、connection graph、mass/inertia和CPU oracle。
- 完成contact damage、threshold/material模型、strain/field、anchor、atomic break、remove与fault guard。

### M4 · Piece Output 与 Rendering

- 发布generation-qualified current/previous piece transform、active mask、bounds与event snapshot。
- 完成render artifact、piece palette、section/material、visibility/indirect、shadow、velocity和resource retirement。

### M5 · Virtual Geometry、RT 与 LOD

- 接Nanite/virtual geometry与RT provider，定义per-piece/cluster BLAS/TLAS、residency和fallback。
- 完成simulation/render/collision LOD、debris virtualization、transition parity与platform matrix。

### M6 · Events、Cache 与 Cross-System Adapters

- 建breaking/collision/trailing/removal event contract、filter/budget/cursor与cache record/playback/streaming。
- 接gameplay、query/picking、navigation、audio、VFX和network adapter，禁止复制piece truth。

### M7 · Editor/Product Integration

- 为Editor提供共享fracture/compiler、selection、cluster、collision、damage、preview和debug snapshot合同。
- 交付wall/pillar/glass/building/pile场景的create/import/fracture/save/reopen/play/export/capture证据链。

### M8 · Reliability 与 Scalability

- 完成malformed、OOM、cancel、device loss、provider crash/unload、cache corruption、world replace和长时间soak。
- 在1/10/100/1000 collections和批准piece/contact/event规模下验证budget、degrade和tail latency。

### M9 · 性能与表现超越门

- 同source/seed、相同破坏输入、场景、硬件与画质对比Unreal，统一记录视觉/物理误差与资源成本。
- 只有raw artifact、trace、capture和统计分布可独立复算时，才允许声明性能或表现领先。

## 15. 验收门

1. source对non-manifold、inverted/degenerate/self-intersecting面、NaN/Inf、bad material和超预算输入给typed diagnostic。
2. piece/face/transform/cluster/edge/material stable ID经save/reopen/reimport/migration不无理由漂移。
3. coordinate、unit、precision、weld/bridge/grout/noise和repair在artifact provenance中可复算。
4. 相同source/dependency/compiler/settings/seed/target生成byte-identical artifact与相同digest。
5. Voronoi/plane/slice/brick/mesh cutter的piece排序和stable ID不依赖线程调度或hash迭代顺序。
6. interior faces闭合，UV/tangent/material/collision source与cut topology一致且无运行时猜测。
7. hierarchy无环、parent/children/level一致，connection edge引用有效且remove/merge/reorder正确重映射。
8. per-piece convex/implicit/query collision在批准volume/inertia误差内，fallback原因与成本可观察。
9. importer/compiler cancel、OOM、malformed block或provider unload不会发布半artifact。
10. source/artifact/cache分别version，有reader/writer/migration/LKG矩阵和不可迁移诊断。
11. provider缺失或capability不足时统一返回Unsupported/Degraded，不逐实体静默fallback。
12. instance每个ticket唯一终态，world replace/unload后stale task/output/event不能发布。
13. 相同asset在两个world、preview与PIE中的damage、cluster、cache和quality state完全隔离。
14. piece/cluster handle代际饱和、删除、slot复用和stale访问均fail-close。
15. root/local/rest/world/mass-space变换在origin shift、negative scale与large world下满足误差门。
16. CPU oracle在固定artifact/input/tick政策下break顺序、state和output digest可复算。
17. batch create/remove对批准piece规模无逐piece全局锁、无界分配或scene entity风暴。
18. union mass/inertia、center of mass与cluster parent切换满足analytic/golden误差门。
19. contact输入包含impulse/relative velocity/piece/material/tick/generation并有稳定dedupe与ordering。
20. user threshold和material-strength/connectivity damage在批准单位与graph上得到可复算结果。
21. construction/persistent/transient field的add/remove、falloff、ordering、capacity和world teardown可验证。
22. atomic break在一个tick内同步graph、cluster、mass/state、events与piece output，不暴露中间态。
23. remove/sleep/disable/debris政策不会删除gameplay-required piece，且原因、时间和identity可追踪。
24. NaN、energy spike、contact/field/break overflow进入rollback/degrade/fail，不污染后续tick。
25. GPU piece resource携artifact/instance/tick/output/device generation和fence，consumer拒绝错代。
26. current/previous transform对activation、break、teleport、cache seek、LOD和remove产生正确velocity/history reset。
27. dynamic root/cluster/piece bounds覆盖真实碎片运动，readback/stale时保守fallback不过度膨胀超预算。
28. exterior/interior sections在depth/GBuffer/forward/shadow/RT路径保持material、UV、normal与visibility一致。
29. piece/cluster culling和indirect work在overflow时可解释drop/degrade，不静默漏绘重要碎片。
30. shadow、main visibility、velocity和RT读取同一piece output generation，无影子漂移或幽灵碎片。
31. RT refit/rebuild/compaction和virtual geometry residency在高速break与device pressure下有typed fallback。
32. breaking/collision/trailing/removal event按stable order、capacity、filter和cursor消费，overflow有receipt。
33. cache record只有一个writer，pending frame在线程边界安全合并，cancel/finalize后原子发布。
34. cache seek/loop/reverse/live切换重建piece state与event cursor，结果与live simulation在批准误差内。
35. gameplay/nav/audio/VFX/network/query只消费typed adapter，不能修改或缓存无代际piece内部slot。
36. simulation/render/collision LOD切换保持gameplay、silhouette、mass、shadow与history在批准误差内。
37. global budget在1/10/100/1000 collections与批准piece/contact/event规模下无无界queue或allocation。
38. debug snapshot能解释artifact、cluster tree、broken edges、strain、pieces、events、memory和CPU/GPU分位，关闭读者时无全量trace。
39. wall/pillar/glass/building/pile场景通过source roundtrip、CPU oracle、Jolt differential、WGPU capture、cache replay与soak。
40. 同口径benchmark记录fracture/physics/render error、CPU/GPU、RSS/VRAM、I/O、stutter和统计分布；领先声明可由raw receipt复算。

## 16. Finding 到里程碑映射

| Finding | 主里程碑 |
|---|---|
| DEST-P1-001..012 | M0-M1 |
| DEST-P1-013..024 | M2 |
| DEST-P1-025..036 | M3 |
| DEST-P1-037..048 | M4-M5 |
| DEST-P1-049..060 | M6 |
| DEST-P1-061..072 | M5、M7-M9 |
| DEST-P2-001..016 | 对应P1与验收门完成后独立立项，不得提前并入MVP |

## 17. 禁止的临时修补

1. 禁止给`RigidBodyComponent`或`MeshRenderer`增加`health/breakable`字段后宣称Destruction完成。
2. 禁止以隐藏完整mesh、切换预制破碎mesh或随机spawn普通刚体作为长期runtime架构。
3. 禁止把数千piece展开为普通Scene entity并每帧全量`node_records()`同步到physics/render。
4. 禁止用普通fixed joints列表替代typed connection graph、cluster hierarchy、strain与atomic break。
5. 禁止以数组下标、native BodyHandle或entity ID作为跨reimport/cache/network的piece长期身份。
6. 禁止Editor、runtime procedural path、physics backend和cache分别实现不同fracture/hierarchy/remap规则。
7. 禁止physics、render、shadow、velocity、RT、event与cache读取不同代piece transform或active mask。
8. 禁止break时同步逐piece创建GPU/physics资源、逐个pipeline/draw分配或阻塞readback bounds。
9. 禁止damage直接由缺impulse/subshape/tick的现有contact event猜测，或由gameplay直接写solver内部state。
10. 禁止overflow时静默丢contact、field、break、event、piece draw或cache frame。
11. 禁止把Unity Shader Graph Voronoi噪声当成fracture算法参考，也禁止逐字复制Unreal类型/UObject层次。
12. 禁止在没有同source/seed、同输入、同场景/硬件/画质与raw receipt时宣称超过Unreal。

## 18. 实施前重查清单

1. 重导249个输入manifest并重算`230c11c...5392`指纹；任何变化先标记本篇stale再评估finding。
2. 复核`frame_extract.rs`与`frame_extract/geometry.rs`的working blob状态，确认相邻会话没有改变piece承载边界。
3. 重跑production exact identifier、ResourceKind、Scene、PhysicsBackend/WorldSync/Event、RenderFeature和tests查询。
4. 取得Runtime04/05/08A/09B/09C/09D/09E/09H1/22/23/24/28/29/31与Editor18/32 owner确认。
5. 先批准source schema、stable identity、deterministic compiler和CPU oracle，再选择Jolt/自研cluster backend与GPU表示。
6. 动态lane按Windows优先，先compiler/headless oracle，再Jolt differential、WGPU产品场景、cache/fault/soak和跨引擎benchmark。

## 19. 本轮产出边界

本轮只新增静态review与分层重构计划，没有修改production Runtime、Editor、Interface、Plugin、App代码或tests，没有运行Cargo、Jolt或WGPU。报告不表示Destruction已经可用，也不授权从runtime fracture、GPU solver或高级材质破坏等P2能力抢跑；实现必须从M0 truth/owner与M1 source/compiler开始，以stable piece identity、deterministic CPU oracle、generation-qualified clustered output和真实产品证据逐层收敛。
