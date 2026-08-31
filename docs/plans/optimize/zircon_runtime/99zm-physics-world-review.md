---
title: Runtime Physics / World / Body / Shape / Collider / Material / Joint / Query / Contact / Trigger / Fixed Step / Jolt / Character / Vehicle / Ragdoll / Debug 当前源码复审
category: zircon_runtime
report_id: Runtime138
review_date: 2026-08-24
baseline_head: ed543173cbd825fe3b7e1f6c81d52c9ca3391095
baseline_epoch: 422
verification_head: ed543173cbd825fe3b7e1f6c81d52c9ca3391095
verification_epoch: 422
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_plugins/12-first-party-physics-source-runtime-editor-dist-catalog-simulation-collision-joint-ragdoll-product-integration-review.md
  - docs/plans/optimize/zircon_editor/18-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-authoring-review.md
related_code:
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/core/framework/scene/physics
  - zircon_runtime/src/asset/assets/physics_material.rs
  - zircon_runtime/src/asset/assets/scene/physics.rs
  - zircon_runtime/src/scene/components/scene/physics.rs
  - zircon_runtime/src/scene/level_system/physics_runtime_enabled.rs
  - zircon_runtime/src/scene/world/project_io/physics.rs
  - zircon_runtime/src/scene/world/property_access/entries/physics.rs
  - zircon_runtime/src/scene/world/property_access/write/physics.rs
  - zircon_plugins/physics
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/src/entry
plan_sources:
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_plugins/12-first-party-physics-source-runtime-editor-dist-catalog-simulation-collision-joint-ragdoll-product-integration-review.md
  - docs/plans/optimize/zircon_editor/18-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-authoring-review.md
  - docs/plans/zircon_plugins/03/failure-2026-08-01-physics-debug-overlay-provider-missing.md
  - docs/plans/zircon_runtime/runtime/07/failure-2026-07-22-level-system-runtime-state-frame-snapshot.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-world-fixed-component-storage-and-stable-query-index.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/PhysicsEngine/BodyInstance.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/CollisionQueryParams.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Collision/WorldCollision.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/Chaos/Public/PBDRigidsSolver.h
  - dev/UnrealEngine/Engine/Plugins/Experimental/Toolsets/PhysicsToolsets/Source/PhysicsToolsets/Private/PhysicsToolsets/PhysicsAssetToolset.cpp
  - dev/godot/servers/physics_3d
  - dev/godot/modules/godot_physics_3d/godot_physics_server_3d.h
  - dev/Fyrox/fyrox-impl/src/scene/graph/physics
  - dev/Fyrox/fyrox-impl/src/scene/rigidbody.rs
  - dev/Fyrox/fyrox-impl/src/scene/collider.rs
  - dev/Fyrox/fyrox-impl/src/scene/joint.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Models/Blocks/Implementations/Collision/CollisionShape.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Runtime/Utilities/EventBinding/Implementation/VFXRigidBodyCollisionEventBinder.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Nodes/MeshDeformation/ComputeDeformNode.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime138 · Physics 当前源码复审

## 1. 结论

当前Physics不是工程级物理系统，而是“较完整的合同外形、两个不等价的局部后端、近似查询/事件层、断开的资产与Editor壳”拼成的功能演示。它已有可保留基础：Runtime拥有中立DTO和manager接口；Scene可以序列化RigidBody/Collider/Joint及若干工程字段；Jolt FFI能创建刚体、执行native update；query filter对较大excluded集合已避免纯线性查找；内部world snapshot已改为`Arc`共享；Level/World已有epoch guard，能拒绝一部分过期写回；测试也覆盖了不少局部schema和算法。

这些基础没有形成同一产品事实。普通Client和Editor Host没有选择Physics provider，first-party runtime/editor catalog也不发布Physics；默认feature不链接Jolt，却仍能通过builtin路径得到`Ready`表象。生产fixed tick直接使用scheduler delta且每帧至多一步，忽略`PhysicsSettings.fixed_hz/max_substeps`；builtin产品路径只做重力、阻尼、轴锁和Scene写回，并不调用其自身constraint solver，也没有碰撞响应。Jolt虽执行native update，但layer/group/mask、collider local transform、root scale、material asset、native query、native contact listener和native constraint都未闭合。

查询和事件不能被当成后端真实能力：manager不论Jolt是否激活，都在共享DTO快照上执行builtin线性/近似几何；contact/trigger同样从body/collider快照做最坏`O(n^2)`重建。返回值没有world/generation/collider/subshape/material/feature/overflow/precision身份，AI sight却直接消费该结果，无法知道它是近似fallback。Physics Material虽然有TOML asset和locator，Jolt同步完全不解析其内容，只使用collider override或全局默认值；static friction和combine rule也未执行。

Ragdoll和Editor尤其不能算产品完成。Ragdoll只是用字符串骨名生成`Empty`节点、单一body/collider和Generic6Dof descriptor；没有Physics Asset importer/cook、稳定bone/shape identity、transaction/save、preview、despawn、physical animation或native articulation。Editor的四份Physics ZUI仍是`Space`占位，命令多为打开view，debug overlay缺真实provider，Workbench显示固定的124 bodies/32 contacts、82 kg和Ice等伪运行反馈。

因此当前没有证据支持“性能和表现优于当前Unreal”。相反，全场景扫描、owned DTO重建、多把global mutex、Jolt全world串行锁、`O(n^2)`近似事件、逐查询`Vec`分配和硬编码native容量使这一主张目前不可成立。本报告只建立重构合同和资格门，不修改生产代码。

旧Runtime08A的20项P1重判为 **17 Open、3 Partial、0 Closed**，4项P2全部Open；旧Plugins12的48项P1重判为 **46 Open、2 Partial、0 Closed**，12项P2全部Open；Editor18的60项P1和12项P2全部Open。三份相关failure handoff均仍为Open。

## 2. 审查边界、方法与currentness

### 2.1 冻结物理范围

统计口径为当前工作树物理行、非空行、文件bytes、Rust test declaration和ignored declaration。fingerprint按normalized lowercase path排序，对每个文件拼接`path + NUL + lowercase(file SHA-256) + LF`后再取SHA-256。产品consumer集合由限定目录中实际包含`physics`的Rust/TOML/ZUI文件组成；它不是整个App/AI/Particles/Sound crate的总代码量。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Runtime Physics路径、Scene/asset/property/diagnostics纵切面 | **62 / 4,411 / 4,118 / 159,001 / 30 / 0** | `31bc891fd07b09ed162b86910625129deb4d3d665b38733e5902ddf917da64e2` |
| Physics插件source/editor/runtime/dist及测试全量 | **94 / 13,631 / 12,606 / 476,660 / 91 / 0** | `ee982df04acac5caa44412c7fe9146748ec895a5cb73b7a760a9ad4d387a2cab` |
| App/catalog/AI/Particles/Sound/diagnostics含Physics consumer | **27 / 7,474 / 6,890 / 264,394 / 72 / 0** | `784885d2603c73211ea927a64cb27a98fc0f6c95566fa4d85f227028bd423859` |
| Zircon selected union | **183 / 25,516 / 23,614 / 900,055 / 193 / 0** | `ac2bca31829a8a49e91b0867c5cc56d889c00d28cfd9873b21f7bdac2ab115c0` |
| 五引擎参考选择集 | **25 / 15,634 / 13,426 / 629,978 / 9 / 0** | `c378749c7cc6f62f9d0dead6f6293f1b0f8a0813f5389c654c4b348c54d5177a` |

参考revision：Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`。`dev/UnrealEngine`没有独立Git元数据，由5个选择文件及参考集合fingerprint冻结。

### 2.2 检查方法

1. 逐文件读取Runtime contracts、Scene component/schema/property/serialization、asset metadata/import、Level fixed tick、diagnostics以及Physics plugin的source/editor/runtime/dist、builtin/Jolt/query/contact/joint/ragdoll/tests。
2. 沿`App profile -> first-party catalog -> provider materialization -> manager registration -> World sync -> fixed tick -> backend -> Scene writeback -> Level events/diagnostics`追踪普通产品链，不从类型名或测试推断可达性。
3. 沿`asset/source -> importer/cook -> resident artifact -> shape/material/joint creation -> backend generation -> reload/rebuild`核对资产生命周期；沿`route -> command -> document/transaction -> preview -> save/cook -> runtime artifact`核对Editor链。
4. 搜索AI、Particles、Sound、Animation/Ragdoll、Navigation和debug overlay的直接消费者，核对能力声明是否对应真实可执行调用。
5. 对Runtime08A、Plugins12、Editor18和三份failure handoff逐项重判。断开的foundation或局部优化最多Partial；普通产品失败条件仍存在时不允许Closed。
6. 参考Unreal的BodyInstance/WorldCollision/Chaos solver/Physics Asset toolset，Godot的server/direct-space/MT command queue，Fyrox的Rapier world/query/contact/body/controller，Bevy唯一fixed clock，以及Unity Graphics的VFX collision/event/deformation consumer边界。

### 2.3 动态证据边界

- Session基线、冻结与验证HEAD均为`ed543173cbd825fe3b7e1f6c81d52c9ca3391095` / epoch 422。
- Physics相关源文件含其他Session/用户的既有working-tree改动。本文审查当前物理内容，不覆盖、不回退，也不把未提交内容标记为已集成资格。
- 本轮为review-only，没有运行Cargo、真实Client/Editor、Jolt native build、PIE、资产cook/reload、GPU/debug capture、跨平台、fault/scale/soak/profile或跨引擎同语义benchmark。
- 静态调用图足以证明的零provider、零consumer、双clock、Jolt query空实现、事件fallback和ZUI占位不因未运行Cargo而改变；动态资格门一律保守判定。
- Tooling按用户要求排除，未来迁移Rust时单独审查，不进入本报告统计和里程碑。

## 3. 当前真实产品链路

```text
App Client / Editor Host
  +-- default target/profile --------------------x Physics provider not selected
  +-- first-party runtime/editor catalogs -------x Physics omitted
  +-- builtin capability/catalog ----------------> may still advertise Physics/Partial

Scene source
  RigidBody? + Collider? + Joint? per node
      -> every fixed tick scans all node_records
      -> owned PhysicsWorldSyncState -> Arc publication
      -> PhysicsManager
          +-- builtin product branch: gravity/damping/axis lock only
          |      -> no collision response; does not use builtin constraint arena
          +-- Jolt branch: native rigid-body update under global world lock
                 -> moving/non-moving object layers only
                 -> local collider transform/root scale/material assets lost

Queries and events
  manager snapshot -> builtin approximate geometry/linear scan -> Vec hits
  manager snapshot -> O(n^2) overlap/contact reconstruction -> cloned events
  (Jolt native broad phase, manifolds, listeners and constraints are bypassed)

Editor / Ragdoll
  route/command -> mostly open view -> Space placeholder / static feedback
  generator -> String bone names -> Empty nodes + body/collider/Generic6Dof
            -> no PhysicsAsset artifact, transaction/save/preview/despawn
```

## 4. 必须保留的基础

1. 保留Runtime-owned neutral contracts和Scene序列化，但把owned/String-heavy DTO分为source schema、compiled artifact、runtime handle/view三层。
2. 保留`Arc<PhysicsWorldSyncState>`内部共享和World/Level epoch guard；继续收敛为单一generation-bound world publication，而不是多张独立mutex map。
3. 保留Jolt FFI和native update底座，但将Jolt限制为plugin-owned provider，Runtime只拥有backend-neutral capability、lifecycle和receipt。
4. 保留builtin query几何和局部constraint测试作为明确的reference/test backend；在有完整solver之前禁止它对shipping profile自报Ready。
5. 保留Scene已有CCD、sleep、mass、material、joint metadata字段，但由单一compiler验证并生成后端可执行artifact，禁止每层各自解释。
6. 保留Physics Material TOML source和locator，补齐import/cook/dependency/derived-data/residency/rebuild链。
7. 保留typed command queue、容量错误和epoch currentness检查，补齐command sequence、apply receipt、body generation和world fault transaction。
8. 保留Editor contribution ID、drawer/view/command和ragdoll generator意图，但必须接真实document、transaction、preview world、overlay provider和保存/cook执行器。

## 5. 当前最高风险差异

### 5.1 Composition、authority、settings与world lifecycle

| ID | 状态 | 当前证据 | 工程级目标 |
|---|---|---|---|
| PH-P1-001 | Open | 普通Client与Editor Host不选择Physics provider，first-party runtime/editor catalog均省略Physics | target composition显式选择单一provider并产生artifact-bound activation receipt |
| PH-P1-002 | Open | default feature不链接Jolt；builtin路径仍可形成`Ready`表象 | 无合格solver时Fail Closed；backend、precision、platform和artifact身份进入readiness |
| PH-P1-003 | Open | Runtime contract默认`PhysicsBackendState::Ready`，dist/native projection又可为空行为 | readiness只能由已激活provider和通过资格的world generation发布 |
| PH-P1-004 | Open | Physics capability静态列出七项Partial，却不表达后端、精度、限制和实际可执行route | capability matrix绑定provider/version/limits/query precision/event semantics和测试receipt |
| PH-P1-005 | Open | `PhysicsSettings`持有另一套fixed_hz/max_substeps，但production每帧只用scheduler delta执行一步 | Bevy式单一fixed clock authority；每个substep有独立tick/delta/receipt |
| PH-P1-006 | Open | settings先改内存/清错再持久化；除backend字符串变化外，既有world不重建 | validation + prepare + durable commit + generation publish；失败保留last-good |
| PH-P1-007 | Partial | internal synchronized snapshot已改为`Arc`，但每tick仍全Scene扫描并重建owned vectors | change journal/dirty set增量compile，persistent SoA backend state和bounded snapshot view |
| PH-P1-008 | Open | settings/clock/sync/events/commands/errors/Jolt world分散于多张global mutex map | 单一`PhysicsWorldInstance`拥有代际、配置、命令、solver、query、event和fault状态 |
| PH-P1-009 | Open | Jolt所有world位于全局mutex，sync/native update期间持锁；poison后继续取inner | per-world scheduler/lock domain，poison进入terminal fault并可有界恢复/teardown |
| PH-P1-010 | Partial | World/Level epoch guard能拒绝部分stale writeback，但missing manager、sanitize drop和scene write error仍可报成功 | atomic step transaction含input generation、applied output、event cursor和typed disposition |
| PH-P1-011 | Open | typed command queue有4096容量，但缺sequence/apply ack；不存在body的命令后续静默跳过 | command admission token、target generation、apply/reject receipt、retention和replay policy |
| PH-P1-012 | Open | Jolt错误会删除world/sync/events并返回零步/空事件，缺last-good和per-world fault generation | explicit Faulted world、diagnostic chain、bounded restart或deterministic terminal teardown |

### 5.2 Body、shape、collider、material、joint与solver

| ID | 状态 | 当前证据 | 工程级目标 |
|---|---|---|---|
| PH-P1-013 | Open | 一个Scene node最多一个RigidBody、Collider、Joint；shape无稳定child/subshape identity | body与多shape分离，shape/subshape/material slot使用stable source ID和generational runtime handle |
| PH-P1-014 | Open | builtin产品分支只积分重力/阻尼/轴锁，不调用`BuiltinPhysicsBackend` constraint arena，也无碰撞响应 | reference backend要么实现并通过solver合同，要么从shipping能力和Ready状态移除 |
| PH-P1-015 | Open | Jolt object layer只区分moving/non-moving，忽略authored layer/group/mask/collision matrix | compiled broadphase/object/filter layers、pair table、query mask与hot-reload generation |
| PH-P1-016 | Open | native shape以body transform创建，忽略collider local transform和root scale；change detection也不比较local transform | shape instance保存local TRS、world scale policy、dirty reason和backend rebuild/update receipt |
| PH-P1-017 | Open | compound child只使用translation/rotation并要求scale为1；teleport还会用body transform覆盖collider transform | versioned compound artifact、child IDs/local TRS/material/filter和增量subshape更新 |
| PH-P1-018 | Open | mesh/heightfield没有production `register_mesh_asset` caller；Jolt把每个三角形建成shape再塞静态compound | offline cook/DDC、native optimized mesh/heightfield、material slots、streaming residency和cook diagnostics |
| PH-P1-019 | Open | Physics Material asset只解析metadata，world sync保留locator但Jolt不读取；static friction/combine规则不执行 | compiled material table、dependency lease、static/dynamic friction/restitution/combine和reload rebuild |
| PH-P1-020 | Open | 质量只对部分primitive/compound推导，显式inertia只是uniform scalar，缺COM和完整tensor | cooked mass properties、COM/inertia tensor、density/material precedence和editor preview parity |
| PH-P1-021 | Open | CCD/sleep/kinematic等字段在schema、validator、builtin和Jolt之间执行不一致 | 单一semantic compiler输出backend executable descriptor及unsupported/fallback disposition |
| PH-P1-022 | Open | 六类joint只存HandlePool descriptor；Jolt步进后由Rust `project_constraints()`修正并写回 | native constraints、双local frame、limit/drive/motor/break/projection、collision和lifecycle events |
| PH-P1-023 | Open | native容量固定16 MiB/16384 bodies/65536 pairs/16384 contacts，thread count也硬编码 | profile/device/scene预算、admission、high-water、overflow policy、telemetry和scale qualification |
| PH-P1-024 | Open | unsafe native初始化、allocator/thread/temp arena没有平台矩阵和teardown/oom/fault corpus | audited FFI ownership、platform-qualified allocator/thread policy、fault injection和sanitizer evidence |

### 5.3 Query、contact、trigger与downstream语义

| ID | 状态 | 当前证据 | 工程级目标 |
|---|---|---|---|
| PH-P1-025 | Open | Jolt backend ray/shape/overlap方法为空；manager总是查询DTO快照 | backend-owned broadphase/query pipeline，provider不可执行时明确Unsupported而非静默fallback |
| PH-P1-026 | Partial | snapshot已用`Arc`且大excluded集合哈希化，但每次查询仍返回新`Vec`并扫描owned DTO | caller-owned/bounded result storage、scratch arena、async batch和zero-allocation hot path |
| PH-P1-027 | Open | filter只有mask/sensor/excluded entities/required group，缺mobility、complexity、profile、predicate等 | compiled filter含object/channel/profile/mobility/complex/subshape/material及stable ignore handles |
| PH-P1-028 | Open | `First`依赖迭代顺序；All/Closest无max-results、overflow、tie-break和generation | deterministic ordering、capacity、overflow disposition、world/query generation和cursor |
| PH-P1-029 | Open | hit不含collider/subshape/material/face/feature/penetration/precision/backend identity | versioned typed hit，字段可用性和exact/approximate/fallback语义显式化 |
| PH-P1-030 | Open | rotated box/capsule/cylinder/convex多用AABB/primitive近似；mesh/heightfield/compound缺完整查询 | 与solver shape相同的backend geometry/query acceleration及precision conformance corpus |
| PH-P1-031 | Open | contact/trigger由body/collider快照最坏`O(n^2)`重建，不来自Jolt narrow phase/listener | native pair lifecycle、broad/narrow-phase feed、bounded event buffer和same-step cursor |
| PH-P1-032 | Open | contact只有entity/point/normal，缺pair/subshape/manifold/impulse/separation/material和Begin/Persist/End身份 | stable pair/manifold/contact IDs、impulse/feature/material、lifecycle和overflow/gap receipt |
| PH-P1-033 | Open | trigger虽有Enter/Stay/Exit枚举，但没有world generation、pair generation、容量和丢失恢复 | generation-bound pair state、bounded journal、gap/resync protocol和consumer cursor |
| PH-P1-034 | Open | Runtime把事件clone进World，再以`Vec`复制到Level；没有订阅/backpressure/retention owner | immutable frame publication、typed subscriptions、bounded retention与consumer acknowledgement |
| PH-P1-035 | Open | AI sight直接调用Physics raycast，无法获知结果来自近似fallback；Particles/Sound多为能力声明或内部开关 | downstream按required precision/capability admission，禁止能力字符串替代可执行consumer |
| PH-P1-036 | Open | diagnostics只给backend/status/fixed_hz/error与单次duration，不能解释query/event/solver状态 | per-world generation、step debt、islands/pairs/contacts/query/capacity/fault/cook/residency metrics |

### 5.4 Ragdoll、gameplay、Editor、distribution与qualification

| ID | 状态 | 当前证据 | 工程级目标 |
|---|---|---|---|
| PH-P1-037 | Open | RagdollProfile以String bone path和默认mass 1表达body；leaf-name fallback依赖唯一名称 | Skeleton/Rig signature和stable bone IDs，versioned body/shape/constraint mapping artifact |
| PH-P1-038 | Open | spawn为每骨创建`Empty`节点和Generic6Dof，`remove`不销毁已创建节点/body | PhysicsAsset/Ragdoll instance拥有完整spawn/despawn、reload、world teardown和rollback transaction |
| PH-P1-039 | Open | Animated/Simulated/Blended仅做简单字符串pose映射，写World错误被丢弃 | physical animation、drive profile、authority handoff、recovery/get-up和fault-visible pose receipt |
| PH-P1-040 | Open | 没有工程级character controller；现有刚体/查询不能提供step/slope/snap/slide行为 | dedicated controller service、kinematic move result、slope/autostep/snap/platform interaction合同 |
| PH-P1-041 | Open | vehicle/soft body/cloth/rope/destruction均无owner，不能由Physics capability总称代替 | 分包provider与独立artifact/solver/coupling contract，按产品优先级逐项资格化 |
| PH-P1-042 | Open | Editor inspector只能改primitive shape/少量joint字段，无法author完整shape/material/joint semantics | schema-driven inspector、multi-shape tree、local frames、limits/drives/material/profile和validation |
| PH-P1-043 | Open | 四份Physics ZUI是`Space`占位；create/debug命令多为打开view或排队假动作 | retained product UI消费真实document/operation/progress/result，所有命令有transaction receipt |
| PH-P1-044 | Open | debug overlay只有DTO builder，没有`ViewportOverlayProvider`；独立failure handoff仍Open | generation-bound overlay provider消费solver/query/event snapshot，支持过滤、pick和capture |
| PH-P1-045 | Open | Workbench硬编码124 bodies/32 contacts、82 kg、Ice等文本，制造运行成功表象 | 所有运行反馈由真实world/selection/artifact diagnostics投影，unavailable时显式失败 |
| PH-P1-046 | Open | Physics Material与Ragdoll没有完整Editor asset toolkit、PreviewWorld、transaction/save/cook/reimport | source document -> compiler -> last-good artifact -> preview -> save/cook的单一产品链 |
| PH-P1-047 | Open | source/export/native/editor的Ready语义不同，native dist是metadata shell且行为/系统/事件为空 | source/native/dist同语义provider或明确Unsupported；artifact closure和activation receipt一致 |
| PH-P1-048 | Open | 193个测试声明主要覆盖API shape/局部算法；无普通产品、native parity、fault/scale/soak/benchmark证据 | product reachability、semantic conformance、cross-platform determinism及同负载竞争基准资格 |

### 5.5 P2高级能力保持Open

| ID | 状态 | 缺失能力 |
|---|---|---|
| PH-P2-001 | Open | async physics、多rate world和明确的game/render interpolation contract |
| PH-P2-002 | Open | rewind/resimulation、rollback snapshot、late input correction和network determinism |
| PH-P2-003 | Open | large world、origin shift、partition/streaming body与跨cell constraint生命周期 |
| PH-P2-004 | Open | GPU/CPU批量query specialization、job scheduling和query cache |
| PH-P2-005 | Open | vehicle/tires/suspension/transmission和可扩展vehicle solver |
| PH-P2-006 | Open | soft body、cloth、rope与two-way rigid coupling |
| PH-P2-007 | Open | destruction/fracture/cluster/field与breaking event/cache |
| PH-P2-008 | Open | production character controller、crowd controller和moving-platform edge corpus |
| PH-P2-009 | Open | articulation、physical animation、muscle/control-rig和ragdoll recovery |
| PH-P2-010 | Open | automatic convex decomposition、mesh sanitation/cook farm和DDC sharing |
| PH-P2-011 | Open | visual debugger、capture/replay、remote inspector、solver/profiler timeline |
| PH-P2-012 | Open | 跨平台确定性语料、长时soak、规模曲线和同语义Unreal/Godot/Fyrox基准 |

## 6. Runtime08A逐项重判

| 原ID | 状态 | 当前判定 |
|---|---|---|
| P1-1 default/release无真实Jolt | Open | default feature仍为空，普通产品也未选择Jolt provider |
| P1-2 engine fixed schedule与PhysicsSettings双clock | Open | production仍以scheduler delta单步，settings clock仍是旁路 |
| P1-3 settings非failure-atomic且既有world不更新 | Open | 仅backend字符串变化会清world，持久化失败仍可污染内存状态 |
| P1-4 fixed tick全Scene projection/deep clone | **Partial** | internal snapshot已用Arc，逐tick全扫描和owned重建仍在 |
| P1-5 per-world state拆成global mutex maps | Open | settings/clock/sync/event/command/error/Jolt world仍分裂 |
| P1-6 query绕过Jolt broad phase并clone world | **Partial** | internal Arc避免一次深clone，但Jolt query仍空、manager仍查DTO并分配Vec |
| P1-7 filter/capacity/determinism弱 | Open | hashed exclusions只是局部优化，结果预算/overflow/tie-break/generation仍缺 |
| P1-8 builtin不是solver却Ready | Open | 产品路径没有碰撞响应且不调用其constraint solver |
| P1-9 Jolt只有moving/non-moving layer | Open | authored layer/group/mask/matrix仍未进入native solver |
| P1-10 contact/trigger非native且生命周期/信息弱 | Open | 仍由DTO近似重建 |
| P1-11 六类joint只是plugin projection | Open | 仍未创建Jolt native constraint |
| P1-12 collider local transform/scale/multishape丢失 | Open | local transform/root scale仍丢，单node单collider仍在 |
| P1-13 mesh/heightfield无cook/import/DDC | Open | 无production registration caller，triangle compound仍在 |
| P1-14 mass/inertia/material partial | Open | COM/tensor/material asset/static friction/combine仍未执行 |
| P1-15 scene/reflection/editor/runtime validation drift | Open | 多层validator和可写字段范围仍不一致 |
| P1-16 ragdoll helper无产品资产/editor/lifecycle/native articulation | Open | helper与占位Editor仍未闭环 |
| P1-17 missing manager/invalid sync/native failure视作成功 | **Partial** | epoch guard和Jolt错误记录是进展，但missing manager、silent drop和零步空事件仍伪成功 |
| P1-18 capability缺backend/precision/limits/executability | Open | 静态Partial表仍在 |
| P1-19 Jolt capacity/thread/unsafe硬编码 | Open | native预算与线程仍为常量 |
| P1-20 tests/milestones混淆API shape与产品Physics | Open | 普通产品/native parity/fault/scale/perf证据仍缺 |
| P2-1 character controller/gameplay locomotion | Open | 无专用controller产品 |
| P2-2 vehicles/soft/cloth/destruction | Open | 无owner |
| P2-3 async/rewind/resim/network determinism | Open | 无owner |
| P2-4 debug/profiling/capacity visualization | Open | overlay provider缺失，diagnostics不足且Workbench伪数据仍在 |

合计：P1为17 Open、3 Partial、0 Closed；P2为4 Open。

## 7. Plugins12逐项重判

### 7.1 P1

| 原ID | 状态 | 当前判定 |
|---|---|---|
| NPHY-P1-001 ordinary App无provider | Open | Client/Editor Host未选择Physics |
| NPHY-P1-002 Editor host无runtime/editor Physics | Open | App及first-party Editor catalog仍断开 |
| NPHY-P1-003 runtime catalog无Physics | Open | first-party runtime catalog仍省略 |
| NPHY-P1-004 editor catalog无Physics | Open | first-party editor catalog仍省略 |
| NPHY-P1-005 builtin catalog广告Physics但provider可能为零 | Open | false-positive capability仍在 |
| NPHY-P1-006 profiles省略/不匹配 | Open | target/profile/provider选择仍不闭合 |
| NPHY-P1-007 generated export是第二composition authority | Open | export/provider closure仍未归一 |
| NPHY-P1-008 generated产品无backend-jolt | Open | 默认feature仍不带Jolt |
| NPHY-P1-009 runtime default空、editor/dist不传播 | Open | feature/provider传播仍断开 |
| NPHY-P1-010 无effective config receipt | Open | settings与provider/artifact identity仍不可追踪 |
| NPHY-P1-011 native dist无状态metadata shell | Open | dist仍无真实world/service |
| NPHY-P1-012 native behavior callbacks为空 | Open | 无可执行behavior |
| NPHY-P1-013 native registration systems/events为空 | Open | 无native system/event projection |
| NPHY-P1-014 七项Partial capability无backend matrix | Open | readiness仍非artifact-bound |
| NPHY-P1-015 source/export/native/editor Ready不一致 | Open | 同名能力仍可指向不同/空行为 |
| NPHY-P1-016 diagnostics不足 | Open | 无world/query/contact/capacity/cook诊断 |
| NPHY-P1-017 双fixed clocks | Open | production与settings clock仍分裂 |
| NPHY-P1-018 tick_scene_world单scheduler delta且忽略settings | Open | 未修复 |
| NPHY-P1-019 settings非原子且无config generation | Open | 未修复 |
| NPHY-P1-020 full scan/deep clone | **Partial** | Arc snapshot减少内部clone，全Scene projection仍在 |
| NPHY-P1-021 多张global maps | Open | 未收敛到world owner |
| NPHY-P1-022 global Jolt map lock | Open | native update仍在全局锁域内 |
| NPHY-P1-023 poison recovery | Open | 仍继续使用poisoned inner state |
| NPHY-P1-024 builtin false Ready | Open | 未修复 |
| NPHY-P1-025 builtin无真实solver | Open | 产品分支仍只有积分 |
| NPHY-P1-026 Jolt filter仅moving/nonmoving | Open | 未修复 |
| NPHY-P1-027 Jolt query空/manager fallback | Open | 未修复 |
| NPHY-P1-028 query clone/Vec/filter | **Partial** | Arc与hashed exclusion是局部进展，Vec和近似fallback仍在 |
| NPHY-P1-029 query First/determinism/capacity/generation | Open | 未修复 |
| NPHY-P1-030 primitives忽略rotation/scale，mesh unsupported | Open | 未修复 |
| NPHY-P1-031 contact/trigger approximate | Open | 未修复 |
| NPHY-P1-032 contact lifecycle/details/capacity缺失 | Open | 未修复 |
| NPHY-P1-033 无native constraint | Open | 未修复 |
| NPHY-P1-034 joint frame/orientation/break/motor不完整 | Open | 未修复 |
| NPHY-P1-035 mesh/heightfield triangle compound且无cook/caller | Open | 未修复 |
| NPHY-P1-036 hard-coded native capacity/thread | Open | 未修复 |
| NPHY-P1-037 sanitize静默丢失/Scene写错被吞 | Open | epoch guard不等于错误闭合 |
| NPHY-P1-038 同entity单body/collider且无shape identity | Open | 未修复 |
| NPHY-P1-039 mass/material字段未完整执行 | Open | 未修复 |
| NPHY-P1-040 ragdoll generator过度简化 | Open | 未修复 |
| NPHY-P1-041 String bone paths | Open | 未修复 |
| NPHY-P1-042 ragdoll Empty node/runtime resource无asset lifecycle | Open | 未修复 |
| NPHY-P1-043 animation/physics authority过度简单 | Open | 未修复 |
| NPHY-P1-044 editor command只有descriptor | Open | 无真实authoring executor |
| NPHY-P1-045 ZUI Space placeholders | Open | 四份占位仍在 |
| NPHY-P1-046 overlay DTO无provider | Open | failure handoff仍Open |
| NPHY-P1-047 ragdoll create/workbench伪动作 | Open | 仍只打开view/显示静态文本 |
| NPHY-P1-048 tests无product/native parity/fault/scale/perf | Open | 未补齐动态资格 |

合计：P1为46 Open、2 Partial、0 Closed。

### 7.2 P2

Plugins12的12项P2全部Open：character controller；vehicles；soft/cloth/rope；destruction；rewind；async multi-rate；CPU/GPU query specialization；large world/streaming；跨平台deterministic corpus；capture/inspector；automatic decomposition；competitive benchmark。

## 8. Editor18逐项重判

Editor18的60项P1没有一项满足Closed条件。为避免在Runtime报告复制60份旧证据，以下按原编号连续分组列出当前失败条件；状态均为Open。

| 原ID范围 | 状态 | 当前仍开放的失败条件 |
|---|---|---|
| P1-1..4 | Open | default catalog/runtime不对称；materialization只有ID；无Physics Material toolkit；Ragdoll asset依赖插件临时加载 |
| P1-5..10 | Open | drawer/view重叠；capability/readiness弱；schema用String；disable lifecycle缺失；注册测试只查ID/view |
| P1-11..16 | Open | RigidBody过度通用；Collider/Joint inspector不完整；multi-edit不安全；validator漂移；shape无可逆handles |
| P1-17..22 | Open | transform/scale语义不清；无hit proxy/mass preview；play-edit policy缺失；无CollisionProfileAsset；Jolt忽略matrix |
| P1-23..28 | Open | Workbench mask固定；query/solver语义混淆；backend layer/cook诊断缺失；reload不rebuild；contact无identity；query debug无真实结果 |
| P1-29..34 | Open | trigger预览和profile impact缺失；无mesh registration caller/bake executor；decomposition静态；triangle mesh validation不完整 |
| P1-35..40 | Open | heightfield无terrain revision/tile cook；compound child无stable ID；material默认/范围/combine错误；无DDC/export/source revision ack |
| P1-41..46 | Open | String骨名；generator只按translation估capsule；ragdoll view占位；无transaction/save/preview owner/physical animation |
| P1-47..52 | Open | overlay无provider generation/filter；debug toggle只开view；diagnostics为空；capacity/stale cleanup缺失；38条route只导航；动作固定queued |
| P1-53..58 | Open | 固定124/32和Ice/82kg伪数据；无PreviewWorld/统一validate；测试把DTO与近似backend当产品；无真实Jolt editor/dist |
| P1-59..60 | Open | 无大规模authoring性能预算；无同质量竞品基线 |

Editor18的12项P2也全部Open：Character Controller authoring；Vehicle chassis/wheel/suspension/tire editor；Destruction/fracture/Geometry Collection；Soft body/cloth/rope；Fluid/particle collision/buoyancy；rewind/resimulation/network determinism debugger；Physics recording/scrub/state diff；Constraint profile library/batch retarget；automatic collision quality/golden contact；third-party backend/plugin cook governance；distributed cook/remote cache；multi-user semantic diff/merge。

## 9. Failure handoff状态

| Handoff | 状态 | 当前判定 |
|---|---|---|
| `zircon_plugins/03/failure-2026-08-01-physics-debug-overlay-provider-missing.md` | Open | 仍无真实`ViewportOverlayProvider`、共享extract和受管Cargo green证据 |
| `zircon_runtime/runtime/07/failure-2026-07-22-level-system-runtime-state-frame-snapshot.md` | Open | epoch guard是真实Partial；sealed snapshot、publication contract和性能资格仍缺 |
| `zircon_runtime/runtime/08/failure-2026-07-22-world-fixed-component-storage-and-stable-query-index.md` | Open | 固定组件/查询index有foundation，但受管验证、稳定query identity和性能闭环仍缺 |

## 10. 参考引擎差异

### 10.1 Unreal Engine / Chaos

- `BodyInstance.h`展示的不是字段清单，而是body/shape lifecycle、每shape response/material、collision profile、CCD/sleep/DOF/weld、mass/inertia、async command与teardown、serialization和cached async creation input组成的产品合同。
- `WorldCollision.cpp`把Raycast/Sweep/Overlap的Test/Single/Multi/Profile/ObjectType变体统一派发到physics interface，并带profiler/analyzer、component shape和rich query params。Zircon当前Jolt query为空、manager走近似DTO fallback，维度不同。
- `PBDRigidsSolver.h`拥有multi-buffer、dirty particle buffer、island/spatial acceleration、真正的joint/suspension/character constraints、event manager、task push/pull和rewind/resim。Zircon的global maps、post-step Rust projection与owned frame clone不能对标。
- `PhysicsAssetToolset.cpp`创建真实package/asset，维护Body/Shape/Constraint CRUD，调用`Modify()`、失效physics meshes并刷新asset change。Zircon ragdoll helper和打开view命令不是同类产品。

### 10.2 Godot

- PhysicsServer3D以RID拥有shape/space/area/body/soft-body/joint，支持多shape local transform、collision mask、contacts budget、CCD、force/impulse和双local constraint frame。
- DirectSpaceState提供有界结果容量和ray/point/shape/cast/collide/rest信息；MT wrapper有command queue、sync/flush/finish和访问限制。Zircon目前既无caller-owned容量，也无明确MT ownership与flush fence。

### 10.3 Fyrox

- PhysicsWorld直接拥有Rapier pipeline、broad/narrow phase、CCD、islands、joint sets、query pipeline、debug render和performance stats；query使用caller-owned storage并返回TOI状态。
- Collider/body/contact暴露interaction/solver groups、manifold/contact feature、同步标志、kinematic mode、force-at-point、dominance、wake/sleep、CCD、mass/COM。
- character controller具备offset、slide、autostep、slope climb/slide、snap-to-ground并直接消费query pipeline。Zircon不能用普通RigidBody加raycast替代该合同。

### 10.4 Bevy

- `Time<Fixed>`是唯一fixed clock；schedule按accumulator执行零到多次，每个substep看到正确delta和overstep。Zircon把scheduler delta与PhysicsSettings clock并列且生产只执行一步，必须先消除双authority。

### 10.5 Unity Graphics

- 本地Unity Graphics不是rigid-body solver参考，只能用于consumer边界。VFX collision block显式表达behavior、solid/inverted、radius、contact attributes、restitution/friction和event语义；rigid-body collision binder是条件化事件桥；Compute Deformation node消费resident buffer index/current data。
- 因此Unity Graphics不能为Zircon solver完成度背书，只说明Physics到VFX/render/deformation的consumer handoff也应typed、resident且可验证。

参考结论不是复制任何引擎的类层次，而是确立最低工程维度：唯一world owner、generation/handle、compiled artifacts、真实backend query/event/constraint、fixed schedule、bounded buffer、product authoring和可复现资格。

## 11. 目标架构与hard cutover

### 11.1 Owner和artifact

```text
App (host / composition only)
  Target + Profile + selected Physics provider
      -> RuntimeCompositionPlan / activation receipt

Editor (authoring truth)
  Physics Material / Collision Profile / Physics Shape / Physics Asset / Ragdoll source
      -> transaction + revision + validation diagnostics
      -> Runtime-owned neutral semantic compiler
      -> backend compiler extension (Jolt plugin)
      -> immutable artifact set + dependency digest + last-good

Runtime (world truth)
  FixedSchedule sole clock
      -> PhysicsWorldInstance { world_generation, config_generation, provider_generation }
          +-- persistent bodies/shapes/constraints/material tables
          +-- command admission/apply journal
          +-- backend solver + native query + native event feed
          +-- bounded immutable PhysicsFramePublication
                    +--> Scene writeback
                    +--> gameplay/AI/navigation/particles/sound
                    +--> Editor preview/debug/inspector

Jolt plugin (implementation only)
  provider factory + backend compiler + native world/query/event/constraint
  no App composition truth, no Scene truth, no duplicate public manager
```

建议的核心类型不是更多字符串DTO，而是以下有明确owner的代际对象：

| 类型 | Owner | 必须包含 |
|---|---|---|
| `PhysicsProviderDescriptor` | Runtime catalog + provider | provider/build/platform/precision/features/limits及qualification receipt |
| `PhysicsProjectArtifact` | Runtime asset/compiler | collision profiles、material table、cooking policy、fixed-step policy、schema/dependency digest |
| `PhysicsShapeArtifact` | Runtime neutral header + provider payload | stable shape/subshape IDs、local TRS、material/filter slots、mass properties、cook diagnostics |
| `PhysicsAssetArtifact` | Runtime asset/compiler | skeleton signature、body/shape/constraint graph、bone mappings、profiles和revision |
| `PhysicsWorldInstance` | Runtime | world/config/provider generation、persistent state、backend、journals、budgets、fault |
| `PhysicsBodyHandle` / `PhysicsShapeHandle` / `PhysicsConstraintHandle` | Runtime instance | owner/world/generation/index，禁止跨world或stale复用 |
| `PhysicsQueryBatch` / `PhysicsQueryResultBuffer` | Runtime query service | caller capacity、precision requirement、overflow、generation、typed hits |
| `PhysicsFramePublication` | Runtime | tick/substep/world generation、body changes、pair/contact/trigger journals、metrics/fault receipt |
| `PhysicsPreviewSession` | Editor | source revision、artifact generation、preview world、selection、debug filters和transaction |

### 11.2 必须删除或替换的旧路径

| 旧路径 | hard cutover |
|---|---|
| 默认`Ready`和静态Partial capability | 删除；只有激活并资格化的provider/world可发布Ready |
| App/catalog/builtin多套Physics composition判断 | 删除；只消费RuntimeCompositionPlan中的selected provider |
| production `fixed_update_step_plan`与PhysicsSettings独立clock并存 | 删除Physics私有clock authority；只消费统一FixedSchedule substep |
| 每tick全Scene scan构建owned sync state | 替换为component change journal + compiled instance delta |
| 多张global mutex maps和global Jolt world lock | 替换为per-world owner和明确scheduler/lock domain |
| Jolt激活时仍使用builtin snapshot query/contact/trigger | 删除静默fallback；由provider实现或返回Unsupported |
| Jolt step后Rust `project_constraints()` | 删除；constraint必须进入backend，reference backend另行明确 |
| mesh/heightfield逐triangle compound | 删除产品路径；只接受qualified cooked native artifact |
| String bone/material/backend/profile/group身份 | 替换为source stable ID + compiled slot + runtime generational handle |
| `let _ =` Scene/pose写回和sanitize silent drop | 替换为atomic disposition/diagnostic/receipt，不允许伪成功 |
| metadata-only native dist和空systems/events | 删除或显式Unsupported，不保留兼容壳 |
| Space占位、打开view假命令、固定124/32与82kg/Ice反馈 | 删除；UI只能投影真实document/operation/runtime snapshot |
| 一个node一个Collider/Joint的结构性限制 | hard cut到body与多shape/constraint source graph，不留re-export shim |

### 11.3 性能目标的证据规则

“优于Unreal”只能由固定hardware/OS/compiler/backend配置、同一scene/query/contact/constraint workload、相同正确性容差和公开原始样本证明。至少同时报告median、p95、p99、max、allocations、resident bytes、build/cook time、step debt、overflow/fault和正确性差异。只比较单个空world FPS、单元测试耗时或不同功能集的平均帧时一律不计。

## 12. 依赖顺序与实施里程碑

| 里程碑 | 依赖 | 交付物 | 完成条件 |
|---|---|---|---|
| M0 Physics truth freeze | 全局MVP 00 owner/composition前置 | false-ready、provider reachability、dual-clock、Jolt fallback、silent-drop RED tests；source/deletion matrix | 测试能在当前产品路径稳定暴露失败，且不新增兼容层 |
| M1 Composition与fail-closed | Runtime composition compiler | selected provider、artifact closure、activation receipt、Unavailable/Faulted lifecycle | 普通Client/Editor Host行为与profile一致；无provider绝不Ready |
| M2 Source schema与artifact | Resource/asset/import/cook基础 | CollisionProfile/Material/Shape/PhysicsAsset source与compiled artifact；stable IDs；last-good | save/reopen/cook/reload确定性，错误不替换last-good |
| M3 World owner与fixed schedule | Time/World/Level generation基础 | 单一PhysicsWorldInstance、persistent state、delta journal、command receipt、frame publication | 0..N substeps语义正确；stale/partial failure不污染Scene或事件 |
| M4 Jolt backend completeness | M2-M3 | native filters/shapes/materials/mass/constraints/query/listener、budget/metrics/teardown | backend conformance corpus通过，激活Jolt时没有builtin semantic fallback |
| M5 Editor与Ragdoll产品 | M2/M4 + Editor operation/document | Physics Material/Profile/Asset toolkits、inspector/gizmo/preview/overlay、ragdoll transaction/cook/runtime lifecycle | 同一artifact驱动Editor preview与runtime；无占位/伪反馈 |
| M6 Gameplay与advanced provider | M4/M5 | character controller先行；再按产品需求拆分vehicle/soft/destruction/articulation | 每项独立provider/capability/资格，不挂靠Physics总称伪完成 |
| M7 Qualification与竞争基准 | M1-M6 | fault/scale/soak/determinism/cross-platform/capture/benchmark corpus | 全部门通过后才允许shipping Ready和任何性能优越声明 |

执行时必须遵守底层优先：若产品测试在Scene storage、stable identity、fixed schedule、asset residency、operation或composition失败，应由对应owner先修复，不得在Physics加本地shim。Physics为可选extension，Runtime拥有neutral contract和world truth；Jolt实现留在plugin；Editor只拥有authoring/preview，不拥有运行时solver truth。

## 13. 资格门

### 13.1 Correctness、identity与artifact

| Gate | 状态 | 通过要求 |
|---|---|---|
| G01 source schema roundtrip | **Partial** | 现有Scene/TOML字段可部分roundtrip；须覆盖全部shape/joint/material/profile/unknown-field与migration |
| G02 stale generation rejection | **Partial** | 已有部分World/Level epoch guard；须覆盖body/shape/constraint/query/event/provider/config全部generation |
| G03 bounded command admission | **Partial** | queue有4096上限；须补sequence、apply/reject receipt、target generation和retention |
| G04 native rigid-body step | **Partial** | Jolt basic update存在；须在普通产品provider链和完整descriptor下通过 |
| G05 single fixed clock | Fail | production 0..N substep、pause/scale/clamp/overstep只能由统一FixedSchedule决定 |
| G06 body/shape identity | Fail | 多shape、subshape、material slot、stale handle和cross-world misuse corpus |
| G07 transform correctness | Fail | rotated/scaled parent、collider local TRS、negative/nonuniform scale与teleport corpus |
| G08 material correctness | Fail | source/cook/reload、static/dynamic friction、restitution/combine和material slot parity |
| G09 mass/inertia correctness | Fail | density/override/COM/full tensor/compound与backend parity |
| G10 constraint correctness | Fail | local frames、limits/drives/motors/break/projection/collision/lifecycle事件 |
| G11 mesh/heightfield artifact | Fail | deterministic cook、native shape、DDC/residency/reload、bad mesh diagnostics |
| G12 atomic world step | Fail | command/sync/solver/writeback/event任一步失败均不发布混合generation |

### 13.2 Query、event、performance与scale

| Gate | 状态 | 通过要求 |
|---|---|---|
| G13 native ray query | Fail | Jolt broadphase，filter/precision/hit identity与ground-truth corpus |
| G14 native sweep query | Fail | TOI/start penetration/rotation/scale/compound/subshape corpus |
| G15 native overlap query | Fail | bounded results、deterministic ordering、overflow和exact pair identity |
| G16 zero-allocation hot query | Fail | caller-owned buffers/scratch reuse，steady-state allocation为零或预算内 |
| G17 contact manifold feed | Fail | native begin/persist/end、feature/impulse/material/subshape和pair generation |
| G18 trigger journal | Fail | bounded Enter/Stay/Exit、gap/resync、consumer cursor和world generation |
| G19 incremental world sync | Fail | dirty比例扩展曲线，禁止每tick全Scene projection |
| G20 multi-world concurrency | Fail | 多world并行不由global Jolt mutex串行，调度与teardown无死锁 |
| G21 capacity/overflow | Fail | bodies/shapes/pairs/contacts/constraints/query/events的admission和high-water可观测 |
| G22 scale/performance curve | Fail | 1K/10K/100K bodies、static/dynamic mix、contacts/queries/constraints及p99/alloc/memory |

### 13.3 Product、Editor与distribution

| Gate | 状态 | 通过要求 |
|---|---|---|
| G23 ordinary Client reachability | Fail | 默认/显式profile的provider选择、Unavailable和activation receipt一致 |
| G24 Editor Host reachability | Fail | runtime provider与Editor authoring provider均由catalog选中并可诊断 |
| G25 source/native/dist parity | Fail | 同artifact同semantic；空dist明确Unsupported，不能Loaded/Enabled |
| G26 Physics Material toolkit | Fail | document/history/save/reimport/compile/preview及runtime reload |
| G27 Collision Profile toolkit | Fail | layer/channel/pair matrix、query/solver影响预览及backend limit diagnostics |
| G28 Body/Collider/Joint inspector | Fail | 完整schema、multi-edit、validation、transaction、undo/redo和play-edit policy |
| G29 Physics Asset/Ragdoll toolkit | Fail | stable skeleton binding、generation、shape/constraint CRUD、preview、cook和runtime lifecycle |
| G30 viewport debug overlay | Fail | 真实provider、pick/filter/query/contact/constraint/capacity显示和capture |
| G31 truthful feedback | Fail | Workbench/diagnostics全部来自真实snapshot；删除固定数字、材质和状态 |
| G32 save/cook/package closure | Fail | source dependency、artifact hash、provider payload、target package和load receipt闭合 |

### 13.4 Reliability、determinism与高级系统

| Gate | 状态 | 通过要求 |
|---|---|---|
| G33 fault injection | Fail | allocator/FFI/persistence/job/lock/command/writeback/cook故障均有typed terminal disposition |
| G34 teardown/reload soak | Fail | world/scene/provider/project反复创建销毁与hot reload无泄漏、stale write或死锁 |
| G35 cross-platform backend | Fail | Windows/Linux目标、compiler/native library、precision和feature matrix有artifact证据 |
| G36 deterministic corpus | Fail | 同平台repeat、跨thread-count、跨平台允许差异及hash/divergence report |
| G37 rewind/resimulation | Fail | checkpoint、command/event cursor、restore/resim和late input correctness |
| G38 character controller | Fail | slope/autostep/snap/slide/platform/ceiling/depenetration及规模测试 |
| G39 ragdoll/physical animation | Fail | authority handoff、drive、collision、recovery、network/save和Editor/runtime parity |
| G40 downstream consumer admission | Fail | AI/Particles/Sound/Navigation/Animation按precision/capability/generation准入 |
| G41 visual capture/profiler | Fail | 可复现capture、solver/query/event timeline、capacity/step debt和remote inspection |
| G42 competitive benchmark | Fail | 相同功能、场景、精度、硬件与统计方法下对Unreal/Godot/Fyrox公开复测 |

合计：42项Gate为 **38 Fail、4 Partial、0 Pass**。Partial只表示局部source foundation，不允许发布shipping-ready能力。

## 14. Owner边界与非重复计数

1. App只拥有进程、target/profile选择与composition提交；Physics provider解析、world状态和solver不进入App。
2. Runtime拥有neutral Physics contracts、project artifact header、world/fixed-step authority、handles、query/event publication和downstream view。
3. Jolt插件拥有native backend/compiler payload/FFI/query/listener/constraint实现；不得另建Scene/clock/composition truth。
4. Editor拥有Physics source document、transaction、preview与诊断投影；运行时数据只能来自Runtime snapshot/provider receipt。
5. Resource/Asset、Time、World/Level、Operation、Animation、Navigation、Render/VFX各自已有专项owner；本报告只记录Physics使用这些边界时的缺口，不重复发明本地服务。
6. Cloth、Destruction、Terrain heightfield、Animation/Ragdoll和Editor viewport的父问题继续由对应专项唯一计数；Physics负责backend contract和交叉资格。
7. Runtime08A、Plugins12、Editor18仍保留历史编号；本报告只刷新currentness，不把同一失败在总账重复计数为新增finding。

## 15. 首个实施切片

在全局MVP 00和composition前置允许Physics实施后，首个切片只做“truth gate”，不直接扩充shape或Editor面板：

1. 添加普通Client、Editor Host、first-party runtime/editor catalog的provider reachability RED测试，证明当前Physics在产品图中缺失。
2. 添加无provider/default feature/builtin无solver却Ready的fail-closed RED测试。
3. 添加统一FixedSchedule 0/1/N substep与PhysicsSettings双clock拒绝测试。
4. 添加Jolt激活时ray/sweep/overlap/contact/trigger/constraint不得静默走builtin fallback的RED测试。
5. 添加collider local transform/root scale/material asset和collision matrix进入native descriptor的RED测试。
6. 添加missing manager、sanitize drop、Scene writeback failure和Jolt fault必须产生typed disposition且不发布空成功帧的RED测试。
7. 冻结删除矩阵后，先交付`PhysicsProviderDescriptor + PhysicsWorldInstance generation + StepReceipt`最小vertical slice；不保留旧Ready/default manager兼容壳。

该切片只建立真实产品边界和失败语义。mesh cook、Editor toolkit、character controller和高级solver在此之前开工，会继续把新功能堆到错误authority上。

## 16. 本轮未做事项

- 未修改production、tests、Cargo、manifest或ZUI；未实现任何Physics修复。
- 未运行Cargo check/test、真实Client/Editor、Jolt native backend、PIE、asset cook/reload、debug overlay或ragdoll preview。
- 未运行sanitizer/Miri、fault injection、scale/soak、跨平台determinism、network rewind或性能profile。
- 未执行与Unreal/Godot/Fyrox的同语义benchmark，因此不作性能优越声明。
- 未覆盖用户明确排除的Tooling优化；Tooling未来Rust迁移另立计划。
- 当前selected文件含其他Session/用户改动；实施前必须重取HEAD、epoch、文件集和fingerprint，再按测试驱动及受管验证流程推进。
