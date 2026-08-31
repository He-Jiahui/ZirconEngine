---
title: Runtime Destruction、Fracture、Geometry Collection、Clustering、Damage Field、Simulation、Rendering、Cache、Scalability、Editor 与 Product Integration 当前源码工程化差距
report_id: Runtime146
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
historical_refresh_of: Runtime33
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/mesh
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/importer
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh
  - zircon_runtime/src/graphics/scene/resources/resource_streamer
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
  - zircon_runtime/src/graphics/visibility
  - zircon_plugins/physics/runtime/src
  - zircon_plugins/physics/zircon_plugin.toml
  - zircon_plugins/first_party_runtime_catalog
  - zircon_app/src/entry
  - zircon_editor/src/core/asset/type_registry
tests:
  - zircon_runtime/src/asset/tests
  - zircon_runtime/src/core/framework/physics/tests
  - zircon_runtime/src/graphics/tests
  - zircon_plugins/physics/runtime/src/tests
  - zircon_editor/src/tests/editor_asset_type_registry
plan_sources:
  - docs/plans/optimize/zircon_runtime/33-destruction-fracture-geometry-collection-clustering-damage-field-simulation-rendering-cache-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zm-runtime-physics-world-body-shape-collider-material-joint-query-contact-trigger-fixed-step-jolt-character-controller-vehicle-ragdoll-debug-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99n-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99b-runtime-temporal-aa-velocity-history-dynamic-resolution-upscaling-reconstruction-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/Chaos
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/GeometryCollectionEngine
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/FieldSystem/Source/FieldSystemEngine
  - dev/UnrealEngine/Engine/Plugins/Experimental/Fracture
  - dev/UnrealEngine/Engine/Plugins/Experimental/ChaosCaching
  - dev/UnrealEngine/Engine/Source/Programs/HeadlessChaos/Private/HeadlessChaosTestGeometryCollection.cpp
  - dev/UnrealEngine/Engine/Source/Programs/HeadlessChaos/Private/HeadlessChaosTestGeometryCollection
  - dev/bevy/crates
  - dev/Fyrox/fyrox-impl/src
  - dev/godot/modules
  - dev/godot/scene
  - dev/godot/servers
  - dev/Graphics/Packages
---

# Runtime Destruction、Fracture、Geometry Collection 与 Product Integration 当前源码工程化差距

## 1. 结论

当前 Zircon 没有 Destruction、Fracture、Geometry Collection、Clustered Rigid 或 Damage Field 运行时产品。对 `zircon_runtime`、`zircon_runtime_interface`、`zircon_editor`、`zircon_app` 与 `zircon_plugins` 中排除 `tests/test_sources/benches/target` 及 test-named 文件后的 **11,951 个 production Rust 文件**执行精确领域扫描，`geometry collection/geometrycollection/destructible/fracture/voronoi/damage threshold/external cluster strain/internal cluster strain/breaking event/field system/chaos cache/clustered rigid/remove on break/debris` 为 **0 个命中文件、0 条命中**。精确词 `destruction` 只有 **8 个文件、8 条**，逐条均是 window、surface、session、preview 或 owner 的通用销毁生命周期，不是游戏世界破坏。

仓内真实存在 typed Mesh/import/artifact cache、per-World Physics/Jolt、shape/body/constraint、fixed step、GPU Scene current/previous transform、dirty-range upload、resource revision、Mesh LOD、chunk residency 和基础 diagnostics。这些可保留为通用前置，但没有 destruction-owned stable piece identity、source/build artifact、interior faces、hierarchy/connection graph、cluster solver、damage/field ingress、atomic break transaction、piece output、breaking event、simulation cache、piece rendering、Editor authoring或产品场景。Jolt 后端仍以逐 body/shape/constraint 同步为核心，query backend 是空实现；contact DTO不含 impulse、relative velocity、subshape、material、tick或generation；TriangleMesh还会为每个三角形建立一个Jolt TriangleShape再组成StaticCompound，不能承载工程级碎裂规模。

产品代码没有把 Destruction 标为 Ready/Executed，因此本文不重复创建 false-ready P0，登记 **0 项新的 Destruction-owned P0**。历史72项P1按当前 working bytes重判为 **55 Open / 17 Partial / 0 Closed**，16项P2全部Open；40项资格门为 **34 Fail / 6 Partial / 0 Pass**。Partial只说明共享owner中有可复用合同，不能解释为破坏产品链已经启动。目标必须硬切到：

```text
DestructionSourceAsset + authoring graph + provenance
  -> deterministic FractureCompiler
  -> FractureBuildArtifact
     (pieces/interiors/collision/hierarchy/connection graph/LOD/render metadata)
  -> generation-qualified per-World DestructionRuntimeInstance
  -> admitted ClusteredRigid Provider + Damage/Field/Contact ingress
  -> atomic Break/Removal/Transform/Event output
  -> Render/Nav/Audio/VFX/Network/Query/Cache typed adapters
  -> runtime-backed Editor authoring、preview 与 qualification receipts
```

## 2. 审查边界、方法与 currentness

### 2.1 冻结范围

本文记录读取时 `main@d931175883f23c1d8e08aab018e0154c9d04a9a8` 的 selected working bytes。共享工作树在最终冻结时有 **4,481 个 changed paths、3,274 个 tracked changes、1,207 个 untracked paths**，其中3,718个位于本轮高层范围；扫描期间HEAD与working tree均受其他Session持续推进。文档验证时主分支已前进到`1b2684b40ae3eba7abfcdfae3fe7e341b4906ec8`，工作树增至5,398个changed paths；这批冻结后bytes没有被追认到表中。本文不归因、不覆盖、不回退任何既有改动，所有结论在实现前必须重取指纹并执行source recheck。用户已明确暂不优化 tooling，本轮没有扫描或规划未来将迁移到Rust的 tooling 实现。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 工作树指纹 |
|---|---:|---|
| Source、Scene、Import 与 Editor carrier | **131 / 23,465 / 21,427 / 814,499 / 152 / 39** | `ec0cf0f32a7a89a8454d24f5923af851f53deabe1dfa891cb0627b6f6eda870c` |
| Physics、Jolt、World 与 Query 前置 | **105 / 11,898 / 10,976 / 412,499 / 55 / 2** | `a7a898c69c9d69817c430e80d48e654ad8d6fe9931d442e1bccc53fb2374ecd6` |
| Render、GPU Scene、Visibility 与 Streaming 前置 | **239 / 48,145 / 44,205 / 1,783,349 / 475 / 67** | `30442b6dcece9a7942a34bd490656f0de6e4a875adf6e8d73abed3d59a0bfa80` |
| Catalog、App、focused tests 与产品证据 | **226 / 51,835 / 49,151 / 1,834,073 / 546 / 4** | `0172bcfd27c3942665539420a36d2aa8e67db0a91b8b05653362b76fdeb320b1` |
| Zircon selected union | **698 / 134,439 / 124,908 / 4,814,288 / 1,212 / 111** | `8d0c7461f570714286ebfc596ed144c4466e8cc7e5ac8e67a8ce1a908af68eea` |
| Unreal Geometry Collection core | **98 / 33,758 / 28,815 / 1,239,620 / 0 / 0** | `54538a878a3115fb43ad063f1617636c3c18ceddb39eb8d3c8660ae558f9175b` |
| Unreal runtime、PhysicsProxy 与 Field System | **80 / 36,183 / 30,573 / 1,469,410 / 0 / 0** | `a2a12cb558f956ba08f88a5f07b07faff7981f0ba4c69c098c5b8efed8448a6a` |
| Unreal authoring、Fracture、Cache 与 Editor | **264 / 81,986 / 67,760 / 3,060,848 / 0 / 0** | `3fc33d69dda2798014ee6e28d62a0c1c4c07700b49b46605d021ef0a98c547af` |
| Unreal render/tests 与 secondary evidence | **31 / 15,686 / 13,080 / 684,273 / 78 / 6** | `cf3102ba23173d3daaadb8063c7cb47f730ae951716807637bd76286bec5ffa2` |
| reference selected union | **473 / 167,613 / 140,228 / 6,454,151 / 78 / 6** | `04fb4a1d51f102544bccbe0fe3ec2a17075eb83ecee93d947888686bf71f13a3` |

指纹算法为：repository-relative path转`/`并小写排序；每个文件取当前bytes的lowercase SHA-256；聚合输入为按行连接且末尾无LF的`path|file_sha256` UTF-8 payload，再取SHA-256。tests/ignored由统一静态检测器统计；另一次对HeadlessChaos Geometry Collection测试的独立宏扫描在26个C++文件、12,111行中识别出107个测试声明，差异来自宏形态，不应把表中78误读为完整测试数。

### 2.2 纵向扫描链

本轮逐层核对 ResourceKind/source schema -> importer/compiler/version/DDC -> stable piece/face/cluster identity -> interior/collision/hierarchy/connection graph artifact -> Scene persistence -> per-World instance/lifecycle -> Jolt provider/cluster/body/shape -> contact/damage/field -> atomic break/removal -> transform/event/cache output -> GPU piece resource/bounds/visibility/material/shadow/RT -> LOD/streaming/budget -> gameplay/nav/audio/VFX/network/query adapters -> Editor/catalog/App/tests/product evidence。没有发现被plugin facade、example、inline test、feature descriptor或字符串命名隐藏的第二套Destruction production owner。

### 2.3 证据等级与执行限制

本文达到E3 source-level review。没有运行Cargo、Jolt、WGPU、App、Editor、PIE、asset cook、roundtrip、GPU capture、fault/fuzz、scale/soak或竞争benchmark；当前没有Destruction source/component/provider/pass可以形成有意义的端到端动态证据，运行普通Mesh/Physics/GPU Scene测试不能提高领域完成度。实施必须先取得capability truth、source roundtrip、deterministic compiler和CPU oracle的RED证据。

## 3. 当前产品链事实

### 3.1 Resource、Source、Import 与 Scene

1. `ResourceKind`只有26种既有资源，没有DestructionSource、GeometryCollection、FractureGraph、DamageProfile、FieldAsset或SimulationCache；Editor builtin registry严格镜像这26种。
2. `SceneEntityAsset`最多持有一个mesh/body/collider/joint及animation、terrain、tilemap、prefab、scripts等普通组件；没有collection component、artifact handle、initial dynamic state、damage/cache/LOD/replication policy。
3. `MeshAsset`有topology、typed attributes、indices、skin、Morph、SDF与virtual geometry；`.zmesh`有版本和基本attribute/index/topology验证，但没有piece/interior/transform/hierarchy/connection/collision-family schema。
4. importer真实支持`.zmesh`、model、glTF和OBJ，能形成subasset并回填SDF/VG；没有fracture recipe、cutter graph、seed、provenance、interior material、piece identity或collection cook。
5. generic artifact cache已经有content hash、revision/state、bounded LRU bytes、external lease、hash/size verification和diagnostics；它是DDC/chunk的可复用前置，不是FractureBuildArtifact或Chaos Cache。

### 3.2 Physics、Cluster、Damage 与 Field

1. neutral physics只同步body/collider/joint/material；每个DTO以`EntityId`为身份，step result只有plan、contacts与triggers，没有cluster hierarchy、connection graph、strain、break或piece transform output。
2. contact event仅含world/entity/other/point/normal，缺少damage所需的impulse、relative velocity、piece/subshape/face/material、tick、generation和dedupe信息。
3. Jolt插件按单body/shape/constraint维护per-world map，handle pool的slot+generation是可保留基础，但没有batch collection create、shared collision family、internal cluster或atomic break transaction。
4. backend query methods仍为空，manager使用builtin snapshot query；query hit只含entity/distance/position/normal，无法返回collection/piece/cluster/face/material stable identity。
5. shape支持Box/Sphere/Capsule/Cylinder/ConvexHull/TriangleMesh/HeightField/Compound；TriangleMesh逐三角创建shape再组成StaticCompound，HeightField也展开三角，自动质量只覆盖解析primitive/compound。此路径在碎片数量、内存、cook成本与质量属性上都不合格。
6. Jolt固定容量为16MiB、16,384 bodies、65,536 pairs和16,384 contacts，pending body command上限4,096；这些是局部常量，不是按pieces/clusters/fields/events/draws/bytes/CPU/GPU统一admission的Destruction预算。
7. Physics manifest只声明physics、raycast、overlap、shape cast、trigger、constraint、skeletal joint，且状态为partial/experimental；没有Destruction capability。这是诚实的缺失，不是隐藏支持。

### 3.3 Render、GPU Scene、Bounds 与 Temporal

1. 42个builtin render features中没有Destruction/GeometryCollection；`RenderFrameExtract`没有collection、piece transform、active mask、break generation或dynamic bounds sideband。
2. ordinary `GeometryExtract`、`RenderMeshSnapshot`、`MeshDraw`和static batches按model/mesh/material/layer工作；geometry source仅有static、skinned、morphed、skinned+morphed。
3. `GpuMeshResource`只表达普通vertex/index/static bounds；streamer把普通`MeshAsset`整件加载成primitive/GPU resource，没有piece/interior section、bone map、cluster LOD或atomic collection bundle。
4. GPU Scene已具备stable key、contiguous instance span、current/previous transform、dirty-range upload、skin/Morph与revision invalidation，是重要共享前置；它没有piece-to-draw mapping、active mask、break/reset history、dynamic cluster bounds或collection generation。
5. 当前bounds主要来自普通mesh/static source或近似primitive，不能覆盖高速碎片运动；visibility/shadow/RT也没有同代piece output、interior visibility和overflow/degrade合同。

### 3.4 Event、Cache、Adapter、Editor 与 Product Truth

1. 没有typed breaking、collision、trailing、removal event，也没有per-consumer cursor、filter、capacity、overflow receipt、generation lease和stable ordering。
2. 通用asset artifact/chunk residency不能记录per-piece transform/velocity/state/events，也没有single-writer record、seek/loop/reverse/live切换或solver cache adapter。
3. gameplay、navigation、audio、VFX、network与query没有Destruction typed adapter；现有普通contact/query也不携带piece identity。
4. Editor只有Physics authoring/debug/diagnostics/ragdoll等通用面，没有fracture mode、cutter、selection、cluster、connection graph、interior material、damage field、cache或runtime preview toolkit。
5. 首方catalog、App composition、CI和examples均无Destruction注册或qualification。WOC的`destructible_wall`只在`objectHealth > 0`时返回3.2碰撞半径，否则返回0.0；它没有piece、solver、render或event合同，只能作为未来迁移fixture。

## 4. 参考引擎证据与采用边界

### 4.1 Unreal 是主参考

1. `FGeometryCollection`明确分离Vertices、Faces、Geometry、Breaking、Materials等group，持有BoneMap、face visibility/internal material、geometry ranges/bounds、Transform hierarchy与Rigid/Clustered simulation type；这证明collection必须是typed artifact，不能把多个普通Mesh Entity拼成长期schema。
2. Geometry Collection object拥有source/import、internal materials、size-specific collision、damage/removal、derived/render data和Dataflow authoring；source、cooked artifact、runtime state与Editor graph是分层owner。
3. `GeometryCollectionPhysicsProxy`覆盖GT/PT mapping、internal cluster、external/internal strain、break/remove/global events、damage threshold/propagation/model、field commands、triple buffering、interpolation与replication；`FDamageCollector`按transform收集damage。
4. FractureEngine提供Voronoi、plane/slice/brick/mesh cutter、deterministic seed、chance、grout/noise、interior material、collision samples、island split，以及k-means/grid/size/connectivity/convexity等聚类策略。
5. Field System使用composable scalar/int/vector nodes、metadata/filter/resolution、construction/buffer commands以及solver/world routing，说明Damage Field应是有owner、lifetime、ordering与budget的typed ingress，不是临时函数参数。
6. Chaos Cache记录per-particle transform/time/velocity/curve/channel/events，具备compression、version、MPSC pending writes与record/playback；Geometry Collection adapter在pre/post solve阶段录制和重放breaking/collision/trailing/enable events。
7. dedicated renderer拥有bone map、current/previous transform SRV、dynamic buffer、exterior/interior section、Nanite、RT、hit proxy与per-bone editor selection。破坏渲染不是普通MeshRenderer逐碎片实例化即可替代。
8. HeadlessChaos的26个Geometry Collection测试文件覆盖hierarchy、clustering、collision、events、fields、mass、proximity、serialization、simulation、streaming、visibility等；这给出了最低可靠性矩阵，不代表Zircon应逐字复制Unreal API或默认值。

### 4.2 Bevy、Fyrox、Godot 与 Unity Graphics 的负证据

本地镜像中，Bevy 1,930个目标源码文件、Fyrox 719个、排除thirdparty后的Godot 4,684个均无本轮精确领域命中；Unity Graphics 6,228个文件只有5个文件、21条Voronoi命中，均是Shader Graph/VFX noise或rain sample，不是fracture。它们在ECS、render、resource或shader层仍可作共享机制参考，但不能作为Destruction产品能力参考；这只是本地镜像负证据，不外推到这些生态的全部包、插件或商业组件。

## 5. 目标架构与 owner 边界

1. `zircon_runtime_interface`只定义稳定resource/component/capability/receipt身份，不承载solver或Editor行为。
2. `zircon_runtime::core::resource`拥有versioned source、artifact、cache、dependency、publish/install/retire、LKG与residency合同。
3. `zircon_runtime::core::framework::physics`拥有provider-neutral clustered rigid、damage/field/contact ingress、atomic break output和CPU oracle；Jolt只实现provider adapter。
4. `zircon_runtime::core::framework::render`与graphics owner只消费generation-qualified render artifact/piece output，不读取Jolt内部slot或重算damage truth。
5. Scene持久化source/artifact handle与initial policy；per-World Runtime instance独占mutable damage、cluster、cache cursor和quality state。
6. gameplay/nav/audio/VFX/network/query只经typed adapter消费event/output，不直接修改piece、cluster或backend handle。
7. `zircon_editor`只调用Runtime source/compiler/preview/debug/transaction合同；不能另建一套fracture、cluster或damage算法。
8. `zircon_app`只做composition和capability admission；首方catalog必须在全链通过前维持Unsupported/Experimental truth。

## 6. P0：false-ready 阻断

当前没有Destruction capability、ResourceKind、component、feature或catalog Ready声明，因此本轮 **0项新P0**。一旦后续把普通Mesh、Jolt rigid body、`destructible_wall`或GPU Scene实例化包装为Destruction Ready，而未通过本文Gate 1-40，应立即登记P0并撤回声明。

## 7. P1：Source、Compiler、Artifact 与 DDC

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| DEST-P1-001 | Open | 26种ResourceKind无Destruction身份；新增Source、BuildArtifact、Cache、Damage/Material Profile kind与versioned handle |
| DEST-P1-002 | Open | 无独立source asset；建立`DestructionSourceAsset`，分离原mesh、authoring graph与cooked runtime collection |
| DEST-P1-003 | Open | 无stable piece/face/transform ID；身份跨fracture重算、reimport、cluster、LOD、cache和network保持稳定 |
| DEST-P1-004 | Open | 无typed collection schema；piece geometry、vertices/faces、transform、hierarchy、material、visibility与simulation group须有cardinality |
| DEST-P1-005 | Partial | Mesh只有基础topology/index/attribute验证；补manifold、orientation、degenerate、自交、space、meters、precision及typed diagnostic |
| DEST-P1-006 | Open | 无authoring fracture graph；cutter、selection、settings、seed、dependency与输出节点必须成为可序列化typed graph |
| DEST-P1-007 | Open | 无deterministic Voronoi/plane/slice cutter；相同source/settings/seed须产生稳定piece identity、拓扑、排序和digest |
| DEST-P1-008 | Open | 无island/grout/noise/repair政策；split、gap、surface noise、weld/bridge/repair须有误差界、diagnostic和provenance |
| DEST-P1-009 | Open | 无interior surface生成；内表面拓扑、UV/tangent、material slot、seam和collision source必须与cut同步编译 |
| DEST-P1-010 | Open | 无hierarchy/cluster/connection compiler；parent/children/level、adjacency、contact area、edge strength和anchor形成一致artifact |
| DEST-P1-011 | Partial | 通用shape family存在但无piece cook；实现per-piece convex/implicit/query、size-specific policy、fallback、共享数据与deterministic cook |
| DEST-P1-012 | Partial | 通用artifact hash/version/cache/residency可复用；增加source/artifact/cache独立version、migration、LKG与含成本/repair的typed receipt |

## 8. P1：Scene、Runtime Instance 与 Lifecycle

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| DEST-P1-013 | Open | Scene无Destruction component；持有artifact、materials、initial dynamic state、damage、cache、LOD和replication policy |
| DEST-P1-014 | Open | 无collection runtime instance；建立world/entity/artifact generation、provider proxy、piece state、resources与terminal receipt |
| DEST-P1-015 | Open | 无piece/cluster runtime handle；stable ID与slot/generation分离，删除、合并、重建和stale访问必须可检测 |
| DEST-P1-016 | Open | 无root/piece/cluster transform合同；明确local/rest/world/mass-space与parent composition，禁止entity transform替代piece truth |
| DEST-P1-017 | Open | 无initial state/activation；类型化static/kinematic/dynamic/sleeping/anchored/disabled及activate-on-damage |
| DEST-P1-018 | Partial | 通用runtime/session/provider有生命周期前置；Destruction仍需Requested/Preparing/Active/Suspended/Retiring/Failed/Cancelled且每ticket唯一终态 |
| DEST-P1-019 | Partial | 通用task generation/cancel可复用；build/create task须携world/instance/artifact/provider/tick generation并阻止旧结果发布 |
| DEST-P1-020 | Partial | artifact revision/publish提供原子性前置；geometry/collision/hierarchy/render bundle完整后才能一次install |
| DEST-P1-021 | Open | 无reload/reimport迁移；定义stable piece mapping、state/cache保留条件、LKG和不可迁移理由 |
| DEST-P1-022 | Partial | Physics已有per-world map；同asset在preview、PIE和多个world中的damage/cluster/cache truth仍需专属隔离测试 |
| DEST-P1-023 | Partial | 通用world teardown/drain前置存在；需按序收口solver task、event、GPU resource、cache writer和consumer lease |
| DEST-P1-024 | Open | 无save/snapshot语义；分离source配置、initial state、runtime damage state与可选checkpoint，禁止序列化raw slot/pointer |

## 9. P1：Clustered Physics、Damage、Field 与 Break

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| DEST-P1-025 | Open | 无clustered-rigid provider；定义batch create、hierarchy、activation、strain、break、results、limits与capability admission |
| DEST-P1-026 | Open | 无deterministic CPU oracle；建立小collection baseline验证compiler、damage、cluster、cache与Jolt differential |
| DEST-P1-027 | Open | 无batch particle/shape creation；按artifact批建piece/cluster并共享collision data，消除逐entity同步与分配风暴 |
| DEST-P1-028 | Open | 无cluster hierarchy simulation；internal cluster、parent/children、union mass/inertia、decluster/recluster须可追踪 |
| DEST-P1-029 | Open | 无connection graph runtime；edge strength、contact area、material、broken state和connectivity由单一owner维护 |
| DEST-P1-030 | Open | 无damage model；user threshold、material strength/connectivity、单位、累积/衰减和clamp必须类型化 |
| DEST-P1-031 | Open | contact只含point/normal；增加impulse、relative velocity、piece/subshape、material、tick/generation和dedupe语义 |
| DEST-P1-032 | Open | 无external/internal strain；支持批准field shape/falloff/metadata、budget与deterministic ordering |
| DEST-P1-033 | Open | 无anchor/kinematic field；construction/persistent/transient field须有owner、lifetime、remove和world isolation |
| DEST-P1-034 | Open | 无atomic break transaction；threshold、graph split、activation、mass/state、event/output须在同tick原子提交 |
| DEST-P1-035 | Open | 无remove-on-break/sleep/disable policy；time/distance/size/speed/visibility/gameplay retention须有typed state与receipt |
| DEST-P1-036 | Open | 无领域overload/fault保护；piece/contact/field/break预算、NaN/energy guard、rollback/LKG和provider fault必须可观察 |

## 10. P1：Render、Visibility、Temporal、Shadow 与 RT

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| DEST-P1-037 | Open | 无collection render artifact；编译piece/bone map、exterior/interior sections、subsections、bounds、LOD和RT metadata |
| DEST-P1-038 | Open | 无piece transform GPU resource；typed storage/SRV按instance generation持有current/previous matrix与active mask |
| DEST-P1-039 | Open | 无transform-to-draw mapping；piece/cluster/section映射必须稳定、可压缩并与physics output generation一致 |
| DEST-P1-040 | Partial | GPU资源revision/retirement可复用；transform/mask/indirect/bounds pool仍需按fence/device generation回收 |
| DEST-P1-041 | Partial | 普通mesh/static bounds提供前置；root/cluster/piece动态bounds须分层更新并为fast motion/readback stale提供保守fallback |
| DEST-P1-042 | Open | 无piece/cluster visibility；frustum/HZB/LOD/occlusion须消费动态bounds及active/remove/interior state |
| DEST-P1-043 | Open | 无collection indirect submission；section/material/visible piece生成有界work，overflow/degrade须有receipt |
| DEST-P1-044 | Open | 无interior material parity；cut face的depth/GBuffer/forward/shadow/RT material、UV与tangent必须一致 |
| DEST-P1-045 | Partial | GPU Scene已有current/previous transform；activation/break/teleport/cache seek/LOD/remove仍需专属velocity/history reset |
| DEST-P1-046 | Open | 无shadow逐片更新；shadow caster bounds、piece transform、LOD与interior policy须与main visibility同代 |
| DEST-P1-047 | Open | 无RT geometry政策；per-piece/cluster BLAS、TLAS、refit/rebuild、compaction、motion与fallback受预算治理 |
| DEST-P1-048 | Open | 无Nanite/virtual geometry adapter；定义dynamic piece transform、cluster LOD、residency、fallback与non-VG parity |

## 11. P1：Event、Cache、Gameplay Adapter 与 Query

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| DEST-P1-049 | Open | 无typed breaking event；含collection/piece/cluster IDs、tick、location、orientation、velocity、mass、bounds和cause |
| DEST-P1-050 | Open | 无collision/trailing/removal event；定义filter、threshold、ordering、capacity、overflow、dedupe和consumer cursor |
| DEST-P1-051 | Open | 无event generation/lease；stale proxy、world replace、replay/cache切换后旧event不得污染新instance |
| DEST-P1-052 | Open | 无cache schema；版本化per-piece transform/velocity/state、events、artifact/provider/tick generation与timebase |
| DEST-P1-053 | Open | 无cache record transaction；physics pending frame、single recorder token、cancel/finalize、compression与atomic publish |
| DEST-P1-054 | Open | 无cache playback/seek；插值、event cursor、rest reset、loop/reverse、missing track和live/cache切换必须可验证 |
| DEST-P1-055 | Partial | 通用artifact chunk residency有hash/size/lease/eviction前置；补simulation-cache prefetch/map/unmap、in-flight drain与corruption fail-close |
| DEST-P1-056 | Open | 无gameplay damage adapter；gameplay只提交typed damage/field request并读取receipt，禁止写piece/cluster内部状态 |
| DEST-P1-057 | Open | 无navigation adapter；break后obstacle/nav dirty region、debris policy、rate limit与generation必须一致 |
| DEST-P1-058 | Open | 无audio/VFX adapter；breaking/collision/trailing按预算聚合，material/energy/size来自同一event truth |
| DEST-P1-059 | Open | 无network/rollback policy；定义authority、event/state replication、late join、checkpoint、prediction与cosmetic debris边界 |
| DEST-P1-060 | Open | query hit只有entity；ray/overlap/hit须返回collection/piece/cluster/face/material stable identity与state generation |

## 12. P1：LOD、Streaming、Budget、Diagnostics、Tests 与 Product

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| DEST-P1-061 | Open | 无simulation LOD；distance/screen importance/visibility/gameplay决定cluster depth、active pieces与update rate |
| DEST-P1-062 | Partial | 普通Mesh LOD可复用；geometry/collision/query/shadow/RT LOD映射、hysteresis与切换误差仍为空 |
| DEST-P1-063 | Open | 无debris virtualization；small piece的sleep/remove/aggregate/FX-only须保留collision/gameplay正确性边界 |
| DEST-P1-064 | Partial | 通用artifact chunk/residency存在；geometry/collision/hierarchy/render/cache须按依赖bundle拆chunk并原子install |
| DEST-P1-065 | Partial | body command与artifact residency有局部上限；建立pieces/clusters/contacts/fields/events/draws/bytes/CPU/GPU统一admission |
| DEST-P1-066 | Partial | Physics manifest诚实标partial/experimental；补backend batch/cluster、storage/indirect/RT与memory的Supported/Degraded/Unsupported矩阵 |
| DEST-P1-067 | Partial | 通用diagnostics可复用；补artifact/instance/provider、cluster tree、strain、broken edges、pieces、events、memory/time快照 |
| DEST-P1-068 | Open | 无领域fault/fuzz；覆盖malformed mesh、bad hierarchy、stale IDs、OOM、cancel、device loss、provider crash/unload、cache损坏 |
| DEST-P1-069 | Open | 无asset/compiler tests；fracture determinism、interior、identity/remap、hierarchy/graph/collision cook/migration golden为空 |
| DEST-P1-070 | Open | 无physics/render/cache tests；strain/break、piece output、velocity/shadow/RT、event与cache replay differential为空 |
| DEST-P1-071 | Open | 无真实产品场景；交付wall/pillar/glass/clustered building/repeat damage/large pile及save/reopen/play/export/capture链 |
| DEST-P1-072 | Open | 无跨引擎超越基准；同asset/seed/scene/hardware/quality比较fracture error、CPU/GPU、memory、stutter与raw receipt |

## 13. P2：基础资格门之后的高级能力

| ID | 状态 | 延后条件 |
|---|---|---|
| DEST-P2-001 | Open | runtime procedural fracture；先完成deterministic offline compiler、budget、transaction与authoritative fallback |
| DEST-P2-002 | Open | stress propagation/FEM coupling；先完成connection/damage oracle、units、stability与performance gates |
| DEST-P2-003 | Open | plastic deformation/crumbling；先完成dynamic topology、mass/collision recook、render remap与cache/network合同 |
| DEST-P2-004 | Open | material-aware anisotropic fracture；先完成physical material strength、grain field、compiler golden与authoring |
| DEST-P2-005 | Open | glass crack propagation；先完成thin-sheet topology、crack identity、optical material、collision与temporal |
| DEST-P2-006 | Open | wood splinter/laminated fracture；先完成directional material、fiber topology、LOD和debris policy |
| DEST-P2-007 | Open | terrain/voxel destruction；先与Terrain owner冻结partition、nav/collision recook与streaming transaction |
| DEST-P2-008 | Open | fluids/fire/weather coupling；先稳定Water/Weather/VFX/gameplay adapter与damage authority |
| DEST-P2-009 | Open | runtime cluster union/reclustering；先完成stable identity、mass/inertia、graph、network/cache和atomic transaction |
| DEST-P2-010 | Open | GPU fracture/cluster solver；先完成CPU oracle、deterministic bounds、readback/fence、fault isolation和portable fallback |
| DEST-P2-011 | Open | deterministic rollback destruction；先完成fixed tick、state digest、checkpoint、bandwidth和late join |
| DEST-P2-012 | Open | large-world partitioned destruction；先完成origin/rebase、cell owner、cross-cell cluster和stream continuity |
| DEST-P2-013 | Open | plugin cutter/damage/provider SDK；先完成ABI/version/capability/budget/sandbox/unload与artifact compatibility |
| DEST-P2-014 | Open | collaborative fracture authoring；先完成stable semantic ID、transaction/merge/locking/recovery和provenance |
| DEST-P2-015 | Open | ML-assisted fracture/LOD；先完成authoritative deterministic fallback、training provenance和error bound |
| DEST-P2-016 | Open | distributed qualification farm；先完成frozen corpus、artifact digest、physics/render capture与raw receipt归档 |

## 14. 分层重构里程碑

### M0 · Truth、Owner 与 Baseline

- 维持Destruction Unsupported，冻结identifier/caller/capability清单；禁止普通Mesh、多RigidBody、WOC血量开关或Jolt依赖false-ready。
- 批准owner、术语、单位、identity、artifact边界、CPU oracle、预算与reference scene corpus。

### M1 · Source、Schema、Compiler 与 DDC

- 建source/fracture graph、stable IDs、typed collection、validation、cutters、interior、hierarchy、connection graph与collision family。
- 完成deterministic artifact、digest、version/migration/LKG、chunk bundle和fault/fuzz。

### M2 · Scene、Instance 与 Lifecycle

- 接Scene roundtrip、component、per-World instance、initial state、save/snapshot与reload/reimport remap。
- 完成ticket终态、async generation、atomic install、multi-world isolation和world drain。

### M3 · Clustered Physics、Damage 与 Field

- 建provider-neutral clustered-rigid contract、CPU oracle、batch create、hierarchy/graph、mass/inertia与connection damage。
- 完成contact ingress、field graph、anchor/strain、atomic break、removal policy、fault guard和Jolt differential。

### M4 · Render Artifact 与 Piece Output

- 编译piece map、exterior/interior sections、LOD/bounds/RT metadata，建立current/previous transform与active-mask resource。
- 完成dynamic pool/fence、piece mapping、bounds、visibility、indirect work与generation-qualified frame extract。

### M5 · Material、Shadow、Temporal、RT 与 Virtual Geometry

- depth/GBuffer/forward/shadow/velocity读取同代piece output，定义break/teleport/cache/LOD/remove history reset。
- 完成interior parity、RT refit/rebuild/fallback、virtual geometry residency与cross-path captures。

### M6 · Event、Cache 与 Cross-system Adapter

- 完成typed events、filter/cursor/overflow/generation lease和simulation cache record/playback/streaming。
- gameplay/nav/audio/VFX/network/query全部转为typed adapter，禁止跨owner读取backend slot。

### M7 · LOD、Streaming、Budget 与 Diagnostics

- 完成simulation/render/collision LOD、debris virtualization、artifact/cache streaming与global admission。
- 交付platform matrix、debug snapshot、telemetry、fault matrix和1/10/100/1000 collection规模门。

### M8 · Editor 与 Product Integration

- Editor只消费Runtime compiler/preview/debug/transaction，交付fracture、cluster、connection、interior、damage field、cache与LOD工作流。
- 交付wall/pillar/glass/building/pile的create/import/save/reopen/play/export/capture证据链。

### M9 · Reliability 与性能表现超越门

- 完成malformed/OOM/cancel/device loss/provider crash/unload/cache corruption、长时间soak与deterministic differential。
- 同硬件、同source/seed、同场景/镜头/画质与同输入对比Unreal；任何领先声明必须由raw receipt复算。

## 15. 资格门当前状态

| Gate | 状态 | 当前判定 |
|---:|---|---|
| 1 | Fail | 无Destruction source，无法对non-manifold、inverted/degenerate、自交、NaN/Inf与超预算输入给领域diagnostic |
| 2 | Fail | 无piece/face/transform/cluster/edge/material stable ID及roundtrip/migration |
| 3 | Fail | 无coordinate/unit/precision、grout/noise/repair provenance |
| 4 | Fail | 无FractureCompiler、artifact或deterministic digest |
| 5 | Fail | 无批准cutter及piece ordering/identity determinism |
| 6 | Fail | 无interior face、UV/tangent/material/collision source合同 |
| 7 | Fail | 无hierarchy/connection graph validator与remap |
| 8 | Fail | 通用shape不等于piece collision cook，无volume/inertia误差、fallback与成本receipt |
| 9 | Fail | 无compiler/provider流程，不能证明cancel/OOM/malformed/unload不发布半artifact |
| 10 | Partial | 通用asset/artifact version、hash、revision和LKG前置存在；Destruction source/artifact/cache独立矩阵为空 |
| 11 | Fail | manifest未虚报能力，但没有统一cluster capability admission与Unsupported/Degraded receipt |
| 12 | Fail | 无instance ticket、world generation、唯一终态与stale output/event拒绝 |
| 13 | Partial | Physics已有per-world map；无Destruction mutable damage/cluster/cache state及preview/PIE隔离证据 |
| 14 | Fail | 通用handle generation存在，但无piece/cluster handle与饱和/复用/stale门 |
| 15 | Fail | 无piece transform truth，无法验证origin shift、negative scale与large world |
| 16 | Fail | 无deterministic CPU destruction oracle与output digest |
| 17 | Fail | 无batch collection create/remove及规模分配/锁门 |
| 18 | Fail | 无cluster union mass/inertia与parent切换golden |
| 19 | Fail | contact缺impulse/relative velocity/piece/material/tick/generation |
| 20 | Fail | 无threshold、material strength、connectivity damage可复算结果 |
| 21 | Fail | 无construction/persistent/transient field owner、ordering、capacity或teardown |
| 22 | Fail | 无atomic break transaction，graph/state/event/output均不存在 |
| 23 | Fail | 无remove/sleep/disable/debris policy与gameplay retention |
| 24 | Fail | 无NaN/energy/contact/field/break overflow rollback/degrade |
| 25 | Partial | GPU资源revision/retirement/fence有共享前置；无piece resource及artifact/instance/tick/output generation |
| 26 | Partial | GPU Scene有current/previous transform；无activation/break/teleport/cache/LOD/remove history policy |
| 27 | Fail | 无dynamic root/cluster/piece bounds与readback stale fallback |
| 28 | Fail | 无exterior/interior section跨depth/GBuffer/forward/shadow/RT parity |
| 29 | Fail | 无piece culling/indirect work及overflow receipt |
| 30 | Fail | 无shadow/main/velocity/RT同代piece output |
| 31 | Fail | 无RT或virtual geometry高速break、pressure与typed fallback |
| 32 | Fail | 无breaking/collision/trailing/removal event、cursor、capacity和overflow |
| 33 | Fail | 无single-writer simulation cache record与atomic publish |
| 34 | Fail | 无cache seek/loop/reverse/live切换及state/event cursor重建 |
| 35 | Fail | 无gameplay/nav/audio/VFX/network/query typed adapter |
| 36 | Fail | 普通Mesh LOD存在但无simulation/render/collision LOD parity |
| 37 | Partial | body/contact/command/artifact有局部上限；无Destruction global budget/admission与1/10/100/1000 evidence |
| 38 | Partial | 通用diagnostics存在；无artifact/cluster/strain/broken edge/piece/event/time/memory快照或零读者成本证明 |
| 39 | Fail | 无wall/pillar/glass/building/pile source、oracle、Jolt differential、capture、cache replay或soak |
| 40 | Fail | 无同口径fracture/render error、CPU/GPU、RSS/VRAM、I/O、stutter raw benchmark |

## 16. Finding 到里程碑映射

| Finding | 主里程碑 |
|---|---|
| DEST-P1-001..012 | M0-M1 |
| DEST-P1-013..024 | M2 |
| DEST-P1-025..036 | M3 |
| DEST-P1-037..048 | M4-M5 |
| DEST-P1-049..060 | M6 |
| DEST-P1-061..072 | M7-M9 |
| DEST-P2-001..016 | 对应P1与资格门完成后独立立项，不得提前并入MVP |

## 17. 禁止的临时修补

1. 禁止给`MeshRenderer`、RigidBody或Scene Entity增加`destructible: bool`后宣称Geometry Collection完成。
2. 禁止运行时把一个mesh随意切成若干独立Entity/body且没有stable piece ID、artifact、hierarchy、connection graph和atomic transaction。
3. 禁止把Jolt ordinary body/compound、每三角shape的TriangleMesh路径或WOC血量归零碰撞开关包装为工程级破坏。
4. 禁止Editor、CPU oracle、Jolt provider、Render和Cache各自维护不同piece ordering、cluster tree或damage rule。
5. 禁止用contact point/normal推测impulse、piece、material或break cause；缺字段必须扩展owner合同。
6. 禁止逐piece创建Scene Entity、逐frame重建GPU buffer、同步readback bounds或无fence销毁in-flight资源。
7. 禁止fracture source、collision cook、render sections和cache分别生成不相关identity，或以数组index长期充当stable ID。
8. 禁止break过程中先发布physics、后补render/event/cache，让consumer看到半collection或错代piece。
9. 禁止用普通Mesh LOD、GPU Scene instance或artifact chunk存在证明simulation LOD、piece rendering或Chaos Cache完成。
10. 禁止fallback静默改变damage、collision、render或network语义；必须给reason、quality、cost和terminal receipt。
11. 禁止逐字复制Unreal类型、默认阈值或目录；采用其owner边界和资格门，并以Zircon fixed spine重建合同。
12. 禁止在没有同asset/seed/scene/hardware/quality/input与raw receipts时宣称性能或表现超过Unreal。

## 18. 实施前重查清单

1. 重导selected manifest并重算本文指纹；HEAD或working bytes变化先标记报告stale再评估finding。
2. 重跑production exact domain、ResourceKind、Scene、Physics capability、RenderFeature、Editor registry与catalog查询。
3. 取得Asset/Schema/Physics/Temporal/Scalability、Terrain/Nav/Audio/VFX/Network与Editor owner确认，避免跨报告重复造底座。
4. 先批准source/schema/compiler/CPU oracle，再选择Jolt cluster adapter和GPU渲染方案；不得由依赖便利性倒推架构。
5. 首个RED切片覆盖capability truth、source roundtrip、malformed validation、deterministic artifact及小collection CPU break oracle。
6. 动态验证按Windows优先，先compiler/headless oracle，再Jolt differential、WGPU capture、Editor产品链、fault/scale/soak和跨引擎benchmark。

## 19. 本轮产出边界

本轮只新增静态review与分层重构计划，没有修改production Runtime、Editor、Interface、Plugin、App代码或tests，没有运行Cargo、Jolt或WGPU。报告不表示Destruction已经可用，也不授权从P2高级能力开工；实现必须从M0 truth/owner与M1 source/compiler开始，以stable identity、deterministic CPU oracle、atomic break output、generation-qualified rendering和真实产品证据逐层收敛。
