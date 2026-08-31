---
title: Runtime Physics Backend / Shape / Query / Event / Fixed Step / Jolt / Character / Ragdoll 当前工作树复审
category: zircon_runtime
report_id: Runtime186
review_date: 2026-08-30
baseline_head: working-tree
baseline_epoch: 2026-08-30
verification_head: working-tree
verification_epoch: 2026-08-30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/167-runtime-physics-current-working-tree-world-sync-jolt-fixed-step-query-event-ragdoll-review.md
  - docs/plans/optimize/zircon_runtime/99zm-runtime-physics-world-body-shape-collider-material-joint-query-contact-trigger-fixed-step-jolt-character-controller-vehicle-ragdoll-debug-product-integration-current-source-review.md
related_owner_reports:
  - docs/plans/optimize/zircon_plugins/12-first-party-physics-source-runtime-editor-dist-catalog-simulation-collision-joint-ragdoll-product-integration-review.md
  - docs/plans/optimize/zircon_editor/227-editor-physics-current-working-tree-authoring-preview-overlay-ragdoll-review.md
related_failure:
  - docs/plans/zircon_plugins/03/failure-2026-08-01-physics-debug-overlay-provider-missing.md
  - docs/plans/zircon_runtime/runtime/07/failure-2026-07-22-level-system-runtime-state-frame-snapshot.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-world-fixed-component-storage-and-stable-query-index.md
related_code:
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/core/framework/scene/physics
  - zircon_runtime/src/scene/components/scene/physics.rs
  - zircon_runtime/src/asset/assets/scene/physics.rs
  - zircon_runtime/src/scene/level_system/physics_runtime_enabled.rs
  - zircon_plugins/physics/runtime/src/backend
  - zircon_plugins/physics/runtime/src/manager
  - zircon_plugins/physics/runtime/src/constraint
  - zircon_plugins/physics/runtime/src/skeletal
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/PhysicsEngine/BodyInstance.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/PhysicsEngine/PhysicsAsset.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/GameFramework/CharacterMovementComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/CollisionQueryParams.h
  - dev/godot/servers/physics_3d
  - dev/godot/modules/godot_physics_3d
  - dev/Fyrox/fyrox-impl/src/scene/rigidbody.rs
  - dev/Fyrox/fyrox-impl/src/scene/collider.rs
  - dev/Fyrox/fyrox-impl/src/scene/joint.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Runtime/Utilities/EventBinding/Implementation/VFXRigidBodyCollisionEventBinder.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Runtime186 · Physics Backend 与生命周期复审

## 1. 结论

Runtime167 已确认 Physics 的 composition、双时钟、全量 world sync、Jolt 空 query、fallback 事件和没有首方 provider 等主干断点。本轮逐文件复核 `zircon_runtime/src/core/framework/physics`、Scene/asset projection、Physics plugin 的 builtin/Jolt/backend/manager/constraint/skeletal 全部源码后，结论没有变好：当前是“可序列化的合同外形 + 一个只做积分的 builtin + 一个只做 native 刚体步进的 Jolt + 从快照猜测的 query/contact/trigger”组合，而不是可作为工程产品的物理 authority。

本轮新增的关键证据是数据正确性和生命周期断裂：

- `PhysicsWorldSyncState` 仍是五个无界 `Vec`，没有 body/collider/subshape/material/joint 的 stable source identity、generation、artifact revision 或 capacity disposition；每个 fixed tick 仍从 `World::node_records()` 全量扫描。
- Scene 的 `ColliderComponent.local_transform` 在 sync 阶段会合并到 transform，但 Jolt body 创建和后续 teleport/active-state 写回把 collider transform 直接覆盖成 body transform，丢失局部位置、旋转、scale；compound child 还要求 scale 必须为 `[1,1,1]`。
- Jolt broad phase 只有 moving/non-moving 两层，忽略 Scene 的 layer/group/mask/collision matrix；native shape 创建没有 material asset lookup，Jolt 只使用 collider override 或全局默认 material。
- Jolt 的三个 `PhysicsBackend` query 入口是空实现；manager 对所有 backend 都走 builtin 快照几何。contact/trigger 在 builtin 和 Jolt 都是对 collider 列表做 `O(n^2)` 重建，事件没有 subshape、material、impulse、penetration、tick、generation 或 overflow。
- Jolt `NativeWorld` 使用固定 16 MiB temp allocator、16384 bodies、65536 pairs、16384 contacts 和固定 job pool；线程/预算/overflow 没有 profile/device/scene admission，也没有 scale qualification。`read_active_states` 只发布 active body，静止体的最终状态依赖旧快照。
- `HandlePool` 有代际句柄，但 world manager 将 settings、clock、snapshot、commands、events 和 Jolt world 拆成多张 `Mutex<HashMap>`；poison recovery 继续取 inner，故障会被降级成空成功。
- builtin step 只实现重力、阻尼、轴锁与旋转积分；它的约束投影和几何 query 不能证明 collision solver、CCD、睡眠、摩擦和 restitution 的产品语义。Jolt 也在 native update 后再次用 Rust projection 修约束，native constraint 并未建立。
- character controller、vehicle、ragdoll 仍没有独立 runtime authority。当前 skeletal profile 只由字符串骨路径生成 capsule/body/constraint 描述，缺 PhysicsAsset cook、骨稳定 identity、physical animation、controller sweep、vehicle wheel/suspension 和 articulation。

因此本报告不新增 P0，继续继承 Runtime167 的 composition/false-ready/clock/provider P0；新增 **30 项 P1、12 项 P2、26 个资格门**。P1 判定为 **24 Open、6 Partial、0 Closed**；P2 为 **12 Open、0 Partial、0 Closed**；资格门为 **24 Fail、2 Partial、0 Pass**。没有同 solver、同 shape cook、同 tick/thread/hardware 与 correctness tolerance 的对照证据，不得声称性能或表现优于 Unreal。

## 2. 审查边界与方法

本轮读取以下生产纵切面：framework physics 31 个模块、Scene physics component/schema/property/project IO、physics material asset、Level fixed-stage、plugin runtime/backend/builtin/Jolt/manager/constraint/skeletal 94 个文件、runtime diagnostics、first-party catalog 与 Physics event consumers。测试只用于确认合同边界，不将测试数量当作功能完成度；Tooling 按用户要求排除。

沿以下链路逐点核对：

```text
manifest/profile -> first-party catalog -> provider activation -> World sync
  -> fixed clock/substeps -> body/shape/material/constraint compiler
  -> backend admission/native step -> query/contact/event publication
  -> Scene writeback -> Level snapshot -> AI/Particles/Animation/Editor consumers
```

参考侧实际存在的选择集包括 Unreal BodyInstance/PhysicsAsset/CharacterMovement/CollisionQueryParams、Godot `servers/physics_3d`、Fyrox rigidbody/collider/joint、Bevy fixed clock 与 Unity VFX rigid-body collision event binder。没有把不存在的 Godot/Unreal private 文件写入 evidence。

## 3. 当前真实调用链

| 链路 | 当前事实 | 工程判定 |
|---|---|---|
| Composition | `first_party_runtime_catalog/src/lib.rs` 没有 Physics registration；App/editor host 的默认选择仍只可得到 contracts | 继承 P0，Fail |
| Readiness | builtin/Jolt selection 可报告 `Ready`；builtin 不是完整 collision solver，Jolt query 仍空 | 继承 P0，Fail |
| Scene projection | `build_world_sync_state` 对每个 node 计算 world transform 并构造 owned vectors；无 change journal | P1-001 |
| Body/collider | 一个 node 一套 body/collider；collider local transform 进入 sync 后在 Jolt 被覆盖 | P1-006/P1-007 |
| Material | TOML asset/locator 存在，backend 只读取 override/default，material table 未编译 | P1-010 |
| Step | Runtime typed tick 已接入，但 `fixed_update_step_plan` 与 manager clock 仍分裂；Jolt NativeWorld 固定预算 | P1-002/P1-015 |
| Query | manager 读取快照走 builtin geometry；Jolt backend query 三个空函数 | P1-017..019 |
| Events | builtin/Jolt 每步扫描 collider pair，事件 DTO 只有实体/点/法线或 trigger point | P1-020..022 |
| Constraint | builtin 是局部 projection，Jolt native body update 后再 Rust projection；没有 native constraint lifecycle | P1-012 |
| Writeback | active-only native states、`let _ =` Scene writes、missing body 静默跳过 | P1-014/P1-016 |
| Skeletal | ragdoll profile generator 只生成字符串 bone path + capsule；无 controller/vehicle/articulation | P1-024..026 |

## 4. 继承 P0（本轮不重复计数）

Runtime167 的 PHY3-P0-01..05 继续 Open：Physics 没有 default/first-party provider closure，false Ready，dist/source/runtime 不同 provider，Jolt query 空实现却被 manager fallback，缺 manager/stale/native fault/writeback failure 会产生空或部分成功帧。本轮所有 P1/P2 都必须在这些 P0 关闭后才可验收。

## 5. P1 差异与重构要求

| ID | 状态 | 当前源码证据 | 必须重构 |
|---|---|---|---|
| PHY4-P1-001 | Partial | `build_world_sync_state` 每 tick 遍历所有 `node_records`，Arc 只减少一次读者 clone | Scene change journal + stable component revision + dirty compiler；增量比例和 1K/10K/100K 曲线需可测 |
| PHY4-P1-002 | Open | `PhysicsSettings.fixed_hz/max_substeps`、manager `advance_clock` 与 `fixed_update_step_plan` 并存 | 单一 fixed-clock authority；每个 0..N substep 有 tick/overstep/receipt，禁止 scheduler delta 直接代替物理时钟 |
| PHY4-P1-003 | Open | `PhysicsWorldSyncState` 是无界 bodies/colliders/joints/materials Vec，没有容量/overflow identity | source schema、compiled artifact、runtime SoA/view 三层；发布携带 world generation、source revision、counts/capacity |
| PHY4-P1-004 | Open | settings、accumulator、snapshot、commands、contacts、triggers、Jolt worlds 分布于多个 global mutex map | 每 world 一个 `PhysicsWorldInstance`，统一 config/solver/query/event/fault/retirement lock domain |
| PHY4-P1-005 | Open | `recover_lock` 对 poisoned mutex 直接 `into_inner`；Jolt error 会清理 map 并返回零步空事件 | poison -> terminal Faulted generation；last-good snapshot、diagnostic chain、bounded restart/teardown |
| PHY4-P1-006 | Open | node 最多一个 RigidBody/Collider，shape compound child 没 stable child id | body 与多 shape 分离；stable body/shape/subshape/material slot identity 和 generational handles |
| PHY4-P1-007 | Open | sync 合并 `local_transform`，但 Jolt `create_body` 只用 body transform，teleport/read state 又覆盖 collider transform | native shape instance 保存 local TRS、root scale policy、dirty reason；body/collider 分离写回 |
| PHY4-P1-008 | Open | Jolt compound 要求 child scale 全为 1；`combine_transforms` 未定义负 scale/shear policy | versioned compound artifact，明确 scale/shear/negative determinant、child order 与 unsupported disposition |
| PHY4-P1-009 | Open | layer filter 有 Scene 字段，Jolt layers 只有 moving/non-moving 两层 | 编译 collision layer/group/mask/matrix 为 broadphase/pair/query filter table，并支持 generation reload |
| PHY4-P1-010 | Open | Jolt material 由 override 或 default；`PhysicsWorldSyncState.materials` 不参与 backend lookup | material asset import/cook/dependency lease；static/dynamic friction、restitution、combine rule 进入 backend descriptor |
| PHY4-P1-011 | Open | mesh/heightfield 仅测试调用 `register_mesh_asset`；Jolt mesh 将 triangles 组成静态 compound | offline mesh/heightfield cook、DDC、native optimized shape、material slots、streaming residency 与失败 receipt |
| PHY4-P1-012 | Open | Jolt `create_constraint` 只存 `ConstraintDesc`，step 后由 `project_constraint` 修正 body | native constraint creation/destruction、local frames、limit/drive/motor/break/collision policy 和 event lifecycle |
| PHY4-P1-013 | Open | mass resolver 只支持 primitive analytic volume；显式 inertia 限制 uniform scalar，缺 COM/tensor | cooked mass properties artifact：density precedence、COM、principal axes、full inertia tensor 与 editor preview parity |
| PHY4-P1-014 | Open | Jolt `read_active_states` 只读取 active bodies；inactive body 不进入 output，Scene writeback 多处 `let _ =` | frame output 必须是完整 body state或明确 unchanged mask；writeback receipt 对 missing/stale/error 分类 |
| PHY4-P1-015 | Open | `NativeWorld` 固定 16 MiB/16384 bodies/65536 pairs/16384 contacts、固定 job system 参数 | device/profile/scene budget admission、high-water telemetry、overflow policy、thread affinity 与 1K/10K/100K qualification |
| PHY4-P1-016 | Open | `HandlePool` 仅本地代际；command queue 只有 world/entity/value，没有 sequence/ack/target generation | command ticket + target body generation + apply/reject receipt、bounded retention、replay/rollback policy |
| PHY4-P1-017 | Open | Jolt `ray_cast` 为空，manager/query 走 snapshot `ray_cast_collider`；没有 backend precision metadata | provider query API 返回 typed result/unsupported，含 world/backend generation、subshape、material、precision/overflow |
| PHY4-P1-018 | Open | Jolt `shape_cast` 为空；builtin sweep 只支持有限 primitive proxy，复杂 shape 返回空 Vec | native sweep/shape cast 或明确 capability denial；定义 initial overlap、backface、multi-hit、sort、distance tolerance |
| PHY4-P1-019 | Open | Jolt `shape_overlap` 为空；builtin pairwise 只覆盖 box/sphere/capsule 等近似组合，compound/mesh 路径缺失 | native broadphase/narrowphase overlap，明确 compound/mesh/heightfield、sensor、filter、capacity 与 deterministic ordering |
| PHY4-P1-020 | Open | `compute_contact_events` 双重 collider 扫描且只输出 midpoint/normal；无 manifold/impulse/penetration | backend contact listener + stable contact key + manifold points/impulse/material/subshape/tick，事件 buffer bounded |
| PHY4-P1-021 | Open | trigger `BTreeMap` 以 entity pair 保存 point，enter/stay/exit 由快照差推断；world reset/teleport 无 policy | backend trigger lifecycle、pair generation、teleport/reset semantics、event cursor/overflow与 deterministic replay |
| PHY4-P1-022 | Open | builtin/Jolt 都调用 builtin contact/trigger helpers；Jolt native contact listener 未注册 | 禁止跨 backend fallback；fallback 只能显式 Experimental oracle，shipping profile 未实现时 fail closed |
| PHY4-P1-023 | Partial | builtin `integrate_builtin_physics_steps` 有重力/阻尼/轴锁/旋转；没有 collision response、CCD、sleep/island、friction/restitution solver | reference backend 需实现完整 solver contract 或从 Ready/capability 移除，并通过 penetration/energy/stack/CCD corpus |
| PHY4-P1-024 | Open | Ragdoll profile 生成器使用 `String` bone path，默认 capsule/mass/constraint；无 PhysicsAsset importer/cook | stable skeleton/bone IDs、PhysicsAsset artifact、shape/constraint authoring、physical animation、rebuild/delete/reload |
| PHY4-P1-025 | Open | `SkeletalPoseTargets`/`SimulatedPoseFeed` 只在 Runtime system 中写 feed；无 animation-to-physics authority、collision LOD 或 root-motion arbitration | fixed-step animation drive、pose sample identity、blend/drive policy、teleport/reset、network/replay semantics |
| PHY4-P1-026 | Open | 没有 character controller API；scene consumers 只能用 ray/query 快照，没有 sweep/step offset/ground state | controller service：capsule sweep、step/slope/slide/ground/walkable surface、moving platform、network prediction |
| PHY4-P1-027 | Open | 没有 vehicle body/wheel/suspension/tire/friction resource，Workbench 的 Vehicle 只是字符串 option | vehicle artifact/runtime service，wheel contact query、substep, drivetrain, replication and deterministic test corpus |
| PHY4-P1-028 | Open | `PhysicsContactEvent`/`TriggerEvent` 不含 event kind for contact, subshape/material/impulse/tick/generation | versioned event envelope、world/body/collider/subshape identity、normal/point precision、overflow and consumer ack |
| PHY4-P1-029 | Open | manager snapshot/query/event APIs 返回裸 `Vec`；无 quota、backpressure、cancel、read consistency | query ticket/borrowed page or bounded output with `complete/overflow/unsupported/stale` disposition and frame snapshot identity |
| PHY4-P1-030 | Open | `apply_commands_to_sync` 对不存在 entity 静默 `continue`；`apply_commands_to_scene` 与 Jolt command semantics 不同 | command admission preflight、atomic apply receipt、backend parity tests、Scene/native reconciliation and stale target diagnostics |

## 6. P2 差异

| ID | 当前问题 | 重构方向 |
|---|---|---|
| PHY4-P2-001 | gravity 为 builtin/Jolt 路径中的隐式常量或 settings 外部默认，缺 world gravity asset | world physics profile + units/gravity source + deterministic serialization |
| PHY4-P2-002 | rotation/scale validation 只做 finite，未定义 quaternion normalization、zero scale、non-uniform policy | canonical transform validator and backend conversion receipt |
| PHY4-P2-003 | query result sort 以 distance/entity 猜排序，缺 stable subshape tie-break | explicit query ordering contract and deterministic oracle |
| PHY4-P2-004 | sleep policy 只映射 Jolt AllowSleeping，builtin 不实现 island/sleep thresholds | shared sleep metrics, thresholds, wake reasons and backend parity |
| PHY4-P2-005 | CCD 只有 Disabled/LinearCast，缺 speculative/contact generation与per-shape policy | compiled CCD profile, speed/size admission and tunneling corpus |
| PHY4-P2-006 | `PhysicsMaterialMetadata` 没有 combine rule 的完整有效性/版本/asset hash边界 | versioned material artifact and migration diagnostics |
| PHY4-P2-007 | mesh asset `AssetReference` 在 runtime backend map 中无 residency lease/eviction | asset dependency lease, cook key, unload/reload receipt |
| PHY4-P2-008 | world replacement 清理多张 map，但无统一 retirement fence | world teardown barrier and stale observer retirement |
| PHY4-P2-009 | diagnostics 只记录 step duration/error，缺 solver phases/query/event/capacity counters | structured per-world diagnostics snapshot and trace correlation |
| PHY4-P2-010 | backend tests 直接 `expect` 创建 Jolt，未覆盖 native unavailable/allocator overflow/device loss | fault-injection test matrix and no-native qualification profile |
| PHY4-P2-011 | Physics manager trait 返回 String/settings clone，缺 object identity/provider version | immutable descriptor snapshot + provider generation |
| PHY4-P2-012 | no network/replay snapshot schema for body/constraint/contact state | quantized deterministic snapshot, correction and replay checksum |

## 7. 资格门

| Gate | 验收条件 | 当前 |
|---|---|---|
| PHY4-G01 | default Client/Editor Host selects exactly one qualified Physics provider | Fail |
| PHY4-G02 | provider disabled/unavailable cannot report Ready | Fail |
| PHY4-G03 | one fixed clock produces 0..N substeps honoring fixed_hz/max_substeps | Fail |
| PHY4-G04 | world sync is incremental and reports source/world generation | Fail |
| PHY4-G05 | body/collider/subshape handles survive reorder and reject stale commands | Fail |
| PHY4-G06 | local collider transform/root scale round-trips through backend | Fail |
| PHY4-G07 | authored layer/group/mask/matrix reaches native broadphase and queries | Fail |
| PHY4-G08 | material asset cook/reload changes native friction/restitution deterministically | Fail |
| PHY4-G09 | mesh/heightfield cook produces bounded native artifact, not triangle-per-shape fallback | Fail |
| PHY4-G10 | all advertised constraints are native or explicitly unavailable | Fail |
| PHY4-G11 | mass/COM/inertia tensor matches editor/runtime artifact | Fail |
| PHY4-G12 | static/inactive body state is published without stale omission | Fail |
| PHY4-G13 | capacity/thread budget has admission, overflow and telemetry | Fail |
| PHY4-G14 | Jolt ray/shape/overlap query has real native implementation or typed denial | Fail |
| PHY4-G15 | initial-overlap, filtering, ordering and precision are tested for every query mode | Fail |
| PHY4-G16 | contacts expose manifold/material/impulse/subshape/tick identity | Fail |
| PHY4-G17 | trigger enter/stay/exit handles reset/teleport/world replacement | Fail |
| PHY4-G18 | no backend silently consumes another backend's fallback events | Fail |
| PHY4-G19 | builtin solver passes collision/stack/friction/CCD/sleep corpus or is non-shipping | Fail |
| PHY4-G20 | command queue produces admission/apply/reject receipts | Fail |
| PHY4-G21 | native fault preserves last-good and enters terminal generation | Fail |
| PHY4-G22 | ragdoll uses cooked PhysicsAsset with stable bone/shape/constraint IDs | Fail |
| PHY4-G23 | character controller has sweep/ground/step/slope/moving-platform tests | Fail |
| PHY4-G24 | vehicle has wheel/suspension/tire deterministic tests | Fail |
| PHY4-G25 | diagnostics/capture/replay records backend and world generation | Partial |
| PHY4-G26 | same scene/cook/tick/thread/hardware correctness-first benchmark vs Unreal/Fyrox/Godot | Partial |

## 8. 参考引擎差异

- Unreal `BodyInstance`/Chaos 将 body instance、shape/material/filter、physics scene registration、async scene and state publication 视为同一 lifecycle；PhysicsAsset 则把骨稳定 identity、body/constraint editing、package and preview 管理纳入资产。Zircon 目前拆开 Scene vectors、Jolt records 与字符串 ragdoll generator。
- Godot `servers/physics_3d` 的 direct-space query、body/area servers 和 command synchronization 说明 query/event 必须属于同一个 space generation，不能从另一份 snapshot 猜测；Zircon manager 的 Jolt query 空函数和 builtin fallback 违反该边界。
- Fyrox 的 rigidbody/collider/joint handles、scene graph ownership 与 inspector 直接对应 Rapier world objects；Zircon 的 node-level single collider、compound tuple 和 local transform overwrite 无法保留该 identity。
- Bevy fixed clock 只应作为统一时间语义参考；Editor preview 和 Runtime 必须消费相同 substep，不应各自从 frame delta 计算。
- Unity Graphics VFX rigid-body collision binder 只消费 typed collision event；Zircon 当前 event 缺 material/subshape/impulse/identity，无法支持同等 consumer 合同。

## 9. 重构顺序与 owner

1. 先由 Runtime167/Plugins12 完成 provider composition、false-ready、single clock 与 world instance hard cutover。
2. 再建立 `PhysicsSourceDocument -> PhysicsCookArtifact -> PhysicsWorldInstance`，固定 stable IDs、local TRS、material/filter/mass/constraint semantics。
3. 将 Jolt native query/listener/constraint/mesh cook 接入同一 backend contract；builtin 限定为 reference oracle 或补齐 solver 后才可 shipping。
4. 建立 bounded command/query/event tickets、last-good fault generation、World retirement fence 与 structured diagnostics。
5. 最后接入 `CharacterControllerService`、`VehicleService`、`PhysicsAsset/RagdollService`，并以 deterministic replay、scale、fault 和 competitor corpus 取得资格。

Runtime186 只写 review 和重构合同；没有修改 Rust、Cargo、Jolt ABI 或资产文件。Tooling 迁移按用户要求另立纵切面。
