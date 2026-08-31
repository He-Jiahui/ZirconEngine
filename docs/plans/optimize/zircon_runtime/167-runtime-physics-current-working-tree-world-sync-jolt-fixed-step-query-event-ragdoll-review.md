---
title: Runtime Physics 当前工作树 World Sync / Fixed Step / Jolt / Query / Event / Ragdoll 工程复审
category: zircon_runtime
report_id: Runtime167
review_date: 2026-08-30
baseline_head: working-tree
baseline_epoch: 2026-08-30
verification_head: working-tree
verification_epoch: 2026-08-30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/99zm-runtime-physics-world-body-shape-collider-material-joint-query-contact-trigger-fixed-step-jolt-character-controller-vehicle-ragdoll-debug-product-integration-current-source-review.md
related_owner_reports:
  - docs/plans/optimize/zircon_plugins/12-first-party-physics-source-runtime-editor-dist-catalog-simulation-collision-joint-ragdoll-product-integration-review.md
  - docs/plans/optimize/zircon_editor/140-editor-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-preview-current-source-review.md
  - docs/plans/optimize/zircon_editor/94-editor-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-preview-product-integration-current-source-review.md
related_failure:
  - docs/plans/zircon_plugins/03/failure-2026-08-01-physics-debug-overlay-provider-missing.md
  - docs/plans/zircon_runtime/runtime/07/failure-2026-07-22-level-system-runtime-state-frame-snapshot.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-world-fixed-component-storage-and-stable-query-index.md
related_code:
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/core/framework/scene/physics
  - zircon_runtime/src/scene/level_system/physics_runtime_enabled.rs
  - zircon_runtime/src/scene/components/scene/physics.rs
  - zircon_runtime/src/asset/assets/physics_material.rs
  - zircon_runtime/src/asset/assets/scene/physics.rs
  - zircon_plugins/physics/runtime
  - zircon_plugins/physics/editor
  - zircon_plugins/physics/dist
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_app/src/entry
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/PhysicsEngine/BodyInstance.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/CollisionQueryParams.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Collision/WorldCollision.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/Chaos/Public/PBDRigidsSolver.h
  - dev/godot/servers/physics_3d
  - dev/godot/modules/godot_physics_3d/godot_physics_server_3d.h
  - dev/Fyrox/fyrox-impl/src/scene/graph/physics
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

# Runtime167 · Physics 当前工作树复审

## 1. 结论

当前工作树的 Physics 有三项可保留的局部改进：`PhysicsWorldSyncState` 的内部快照可以以 `Arc` 共享，world projection 对 owned payload 的移动与 nested shape 投影更完整，Runtime system 已经从裸 scheduler delta 改为读取 typed tick context。它们只减少一次复制或修正输入来源，不改变 Physics 的权威边界、后端语义或产品可达性。

真实链路仍是：Runtime 每个 fixed stage 读取 Scene，按节点全量构建 `PhysicsWorldSyncState`；manager 使用多张按 `WorldHandle` 索引的 `Mutex<HashMap<...>>` 保存 settings、accumulator、snapshot、commands、contacts、triggers 和 Jolt world；builtin 由积分器与几何近似产生结果；Jolt 执行 native update 后再由 Rust projection 修正 constraint；manager 将结果复制到 Level 和 Scene。Jolt 的 `ray_cast`、`shape_cast`、`shape_overlap` 仍为空，Jolt 接触与触发器仍调用 builtin snapshot 扫描，且 default/first-party catalog 仍没有完整的 Physics provider closure。

因此旧 Runtime138 的 P1/P2 没有任何一项可关闭：P1 仍为 17 Open、3 Partial、0 Closed，P2 仍为 4 Open；42 项资格门仍为 38 Fail、4 Partial、0 Pass。当前新增的 Arc/owned projection/typed tick 只能使 full-scan、snapshot clone 和 fixed-clock 相关门保持 Partial，不能宣称 Jolt、query、event、constraint 或性能完成。没有同硬件、同 solver、同 shape/cook、同正确性容差的 Unreal/Godot/Fyrox 对照证据，禁止“性能和表现优于 Unreal”的结论。

## 2. 冻结范围与方法

统计口径为当前工作树，包含测试、build script、manifest 和 ZUI，但不包含 `Tooling`。每个文件按 repository-relative lowercase path 排序，以 `path + NUL + file SHA-256 + LF` 生成集合 fingerprint；行数是 UTF-8 文本分行数，测试统计为 Rust `#[test]` / `#[async_test]` 声明，不能作为功能完成度。

| 范围 | 文件 / 行 / bytes / tests / ignored | fingerprint |
|---|---:|---|---|
| Runtime framework、Scene/Level、Physics asset 纵切面 | 51 / 3,669 / 129,047 / 16 / 0 | `f685ce734035b63792561741c400e2d86fc598cff8d90d407af14b32ba6e4cff` |
| Physics plugin runtime/editor/dist、backend、tests、ZUI | 94 / 13,831 / 480,757 / 92 / 1 | `959f49b7bc36c6a688c72be1b3cf91452a43914337ddc22a15954730f86ff179` |
| first-party catalog、App entry 与 Physics consumers | 218 / 31,033 / 1,140,226 / 484 / 1 | `3baf6de4405e09e13943f9a6f390c4b787fcda91c2a70c378b33c4200e7ebd31` |
| 去重 Zircon selected union | 363 / 48,533 / 1,750,030 / 592 / 2 | `ffa4c9cdf2ed6517b084ddb5c9526cca60114dca5afa00c0bfab33dd9c6eab8f` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics reference selection | 43 / 14,925 / 654,966 / 8 / 0 | `7577c845fe15df1c66fac582fd379338f0d38884c10576e3a3f1171fb7de5905` |

Physics 相关文件存在其他 session/用户的 working-tree 改动。本轮读取这些内容，不回退、不覆盖，不把未提交 patch 视为已通过的产品实现。按用户要求未查询、轮询、等待或实时跟踪协调器；本轮只做静态 review，未运行 Cargo、Jolt native、Client/Editor、PIE、asset cook/reload、fault、sanitizer、scale、soak 或 benchmark。

## 3. 当前调用链事实

```text
PhysicsRuntimeSystem::run_physics_runtime_system
  -> Level replacement epoch
  -> RagdollRuntime animation drive
  -> DefaultPhysicsManager::tick_scene_world
     -> fixed_update_step_plan / settings
     -> drain body command queue
     -> build_world_sync_state (full node_records scan)
     -> Builtin integrator OR JoltManagedWorld::synchronize
     -> contact/trigger publication
  -> Level physics frame snapshot (Arc event slices)

PhysicsManager::{ray_cast, shape_cast, shape_overlap}
  -> manager/query.rs
  -> synchronized snapshot / builtin geometry
  -> Vec result

JoltPhysicsBackend::{ray_cast, shape_cast, shape_overlap}
  -> empty implementation
JoltPhysicsBackend::read_active_states
  -> refresh_events -> builtin O(n^2) snapshot contact/trigger scan
```

`PhysicsManager` 的 public contract 只有 `plan_world_step`、`sync_world`、snapshot、返回 `Vec` 的三类 query 和 drain events（`zircon_runtime/src/core/framework/physics/manager.rs:12-35`）。它没有 world/config/provider/body generation、step receipt、query capacity/overflow、event cursor、fault disposition 或 backend precision。`PhysicsWorldSyncState` 仍为四个 unbounded `Vec`（`world_sync_state.rs:9-16`），不能作为持久 backend storage 或增量 change journal。

## 4. P0：产品真实性与权威断裂

| ID | 状态 | 当前源码证据 | 必须重构 |
|---|---|---|---|
| PHY3-P0-01 | Open | `first_party_runtime_catalog/src/lib.rs:34-99` 没有 Physics registration；Editor catalog `catalog.rs:41-54` 只分发 Navigation/Neural | 用单一 `RuntimeCompositionPlan` 原子选择 runtime provider、Editor authoring、resource、backend 与 artifact；缺项整能力 fail-closed |
| PHY3-P0-02 | Open | `selection.rs:59-88` 对 builtin/Jolt 直接报告 `Ready`；builtin 不是完整碰撞 solver，Jolt feature 还可能未启用 | Ready 必须绑定 provider/build/platform/precision/qualification/world generation；reference backend 只能显式 Experimental/Unavailable |
| PHY3-P0-03 | Open | dist 文案明确“services remain hosted by runtime module”，`dist/src/lib.rs:23-42` 的 systems/events/commands/callback 均为空 | native/source/editor 必须共享同一 executable provider/artifact；若 dist 只是 projection，状态必须是 Unsupported，禁止 Loaded/Ready 表象 |
| PHY3-P0-04 | Open | Jolt 三个 query 入口 `backend/jolt/runtime.rs:527-549` 为空，manager 仍使用 builtin snapshot query | 禁止按 backend 名称静默降级；provider 未实现必须返回 typed Unsupported，并让下游按 precision admission |
| PHY3-P0-05 | Open | `runtime_system.rs:60-123` 缺 manager 时记录空 step/events 后返回；Jolt error `jolt_world.rs:463-469` 清空状态并只保存一条全局错误 | 缺 provider、stale world、sanitize drop、native fault、Scene writeback failure 都必须产生带 generation 的 atomic disposition，不得发布空成功帧 |

## 5. P1：World、fixed step 与状态生命周期

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| PHY3-P1-001 | Partial | `world_sync.rs:24-126` 仍逐 tick 扫描 `node_records`，Arc 只避免读取者再次 deep clone | Scene component change journal、dirty set、persistent SoA backend state 和 bounded immutable publication；增量比例与 1K/10K/100K 曲线需可测 |
| PHY3-P1-002 | Open | `DefaultPhysicsManager` 仍将 settings/accumulator/snapshot/events/commands/Jolt worlds 拆在 8 个 mutex map（`manager.rs:38-52`） | 每个 World 一个 `PhysicsWorldInstance`，拥有 config/provider/body/shape/constraint/query/event/fault generation；明确 lock/scheduler domain |
| PHY3-P1-003 | Partial | `clock.rs:27-52` 已有 accumulator 和 max_substeps；但 `service.rs:198-230` 另有 `fixed_update_step_plan`，对生产 tick 返回最多一步且使用传入 delta | 删除第二 clock，统一消费 `Time<Fixed>` 式 0..N substep；每个 substep 有 tick、delta、overstep、receipt |
| PHY3-P1-004 | Open | settings 先写 mutex、清 Jolt/commands，再持久化；失败时内存可能已经污染（`settings.rs:52-73`） | validate -> prepare -> durable commit -> world rebuild -> generation publish；失败保留 last-good 配置和 artifact |
| PHY3-P1-005 | Open | Jolt world 在全局 `Mutex<HashMap>` 内同步和 native update（`jolt_world.rs:433-453`）；poison recovery 仍返回 inner | per-world executor/lock，poison 进入 Faulted；teardown/restart 有界且不能暴露混合 generation |
| PHY3-P1-006 | Partial | Level 已以 replacement epoch 和 `Arc<[event]>` 发布快照（`physics_runtime_enabled.rs:111-141`），但只保护 Level publication，不保护 body/query/provider generation | frame publication 必须携带 world/config/provider/tick generation，Scene writeback 与事件 cursor 必须同一 receipt 原子提交 |
| PHY3-P1-007 | Open | `sanitize_world_sync_state` 静默 retain/drop 重复或非法 body/collider/joint/material（`world_sync.rs:272-300`） | 不丢弃事实；返回逐对象诊断、reject/drop count 和 source identity，整步按 policy commit 或 rollback |
| PHY3-P1-008 | Open | command queue 有固定容量但无 command sequence、target generation、apply ack；Jolt translate 找不到 entity 直接错误 | command admission token、body handle generation、apply/reject receipt、replay/retention 与 backpressure |

## 6. P1：Body、Shape、Material、Joint 与 Jolt solver

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| PHY3-P1-009 | Open | Scene 节点仍最多一个 rigid body/collider/joint；sync 以 `entity` 为唯一 key（`world_sync.rs:36-123`），compound child 没 stable subshape ID | body 与多 shape/shape instance 分离；source stable ID -> compiled slot -> world generational handle |
| PHY3-P1-010 | Open | builtin `step` 只积分并随后执行简化 constraint；没有 broad phase/narrow phase impulse collision response（`backend/builtin/runtime.rs:307-334`） | builtin 降级为明确 test/reference backend，或实现完整 solver contract；不得在 shipping profile 报 Ready |
| PHY3-P1-011 | Open | Jolt object layer 只有 moving/non-moving（`backend/jolt/runtime.rs:322-341`），Scene layer/group/mask 未编译到 native pair matrix | collision profile artifact、object/channel/query/solver matrix、hot-reload generation 与 overflow diagnostics |
| PHY3-P1-012 | Open | Jolt `create_shape` 从 collider shape 生成 native shape，但 `BodyDesc` 没有可追踪 subshape/material slot；compound conversion 依赖临时 children | offline cooked shape artifact，local TRS、subshape/material/filter identity、reuse/refcount、reload remap |
| PHY3-P1-013 | Open | `BodyCommand::Teleport` 会把 collider transform 直接覆盖为 body transform；Jolt active state 也同步覆盖 collider transform（`runtime.rs:289-291`, `:579-581`） | body pose、collider local pose 和 scene hierarchy 分层；teleport/rebuild 明确 local/world/scale/mirror policy |
| PHY3-P1-014 | Open | Jolt `create_body` 只把 density 作为 `InertiaMultiplier` 等有限字段；Physics Material locator 没有实际 asset resolve/combine | cooked mass/COM/inertia tensor、density precedence、surface/static friction/dynamic friction/restitution/combine 与 asset residency |
| PHY3-P1-015 | Open | Jolt `create_constraint` 只将 `ConstraintDesc` 放入 handle pool（`runtime.rs:390-419`）；真正 native update 后再 `project_constraints`（`:182-233`） | native constraint/limit/drive/motor/break/collision/local frame 必须由 backend solver 执行；projection 只能是明确的 post-solve policy |
| PHY3-P1-016 | Open | mesh/heightfield 注册 API 存在但无 production caller；mesh shape 通过 triangles 组静态 compound，固定内存和线程预算在 `native_world.rs:26-29` | cook/DDC/residency、optimized mesh/heightfield、static restriction、budget admission、OOM/fault/teardown corpus |
| PHY3-P1-017 | Open | `HandlePool` 有 generation，但 public sync/event/query 全部使用裸 EntityId；跨 world、stale shape/subshape、native id 映射不可见 | owner/world/generation/index 组合 handle，所有 write/query/event 校验代际，禁止跨 world 复用 |

## 7. P1：Query、Contact、Trigger 与下游

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| PHY3-P1-018 | Open | query interface 直接返回 `Vec`（`query_interface.rs:8-15`）；manager 每次新建结果和 prepared filter | caller-owned bounded result buffer、scratch reuse/async batch、capacity/overflow/generation/precision receipt |
| PHY3-P1-019 | Open | builtin overlap/raycast 使用 AABB/primitive 近似；Jolt query 空，旋转 capsule/box、mesh、compound 没有 solver geometry parity | query 与 solver 共用 native broad/narrow phase；建立 exact/approximate conformance corpus，禁止隐式 fallback |
| PHY3-P1-020 | Open | `PhysicsQueryFilter` 只有 mask/sensor/excluded/group（`query_filter.rs:5-13`）；没有 mobility、trace/object channel、subshape/predicate | 编译后的 filter profile 覆盖 query/solver/sensor semantics、stable ignored handles 和权限 |
| PHY3-P1-021 | Open | `PhysicsQueryMode::First` 依赖迭代顺序，All 无 max results/overflow（`backend/builtin/query_contact/mode.rs:21-45`） | deterministic tie-break、max-results、overflow/gap、query generation 和 cursor |
| PHY3-P1-022 | Open | contact 只有 world/entity/other/point/normal（`contact_event.rs:6-14`）；`contact.rs:14-39` 只做 pair scan 和 midpoint | native begin/persist/end、pair/manifold/contact/subshape/feature/material/impulse/separation、bounded event journal |
| PHY3-P1-023 | Open | trigger `scan.rs:45-63` 对全部 collider 做两层循环；`previous` 只存 pair/point，没有 world/provider generation 或 gap recovery | backend listener/broad+narrow phase feed、bounded Enter/Stay/Exit journal、consumer ack/resync |
| PHY3-P1-024 | Partial | Level 事件 slice 已用 Arc，但 manager drain 仍从各自 map 移除 Vec；无订阅、backpressure、consumer cursor | 单一 immutable `PhysicsFramePublication` 同时提供 body changes/query/event/metrics，读者按 generation ack |
| PHY3-P1-025 | Open | AI/Animation 等可通过 `PhysicsQueryInterface` 取得结果，却无法知道 backend、精度或 stale 状态；Particles/Sound 只有 capability 宣告 | 下游按 required precision/capability/generation admission；Unsupported/Approximate 必须显式传播 |
| PHY3-P1-026 | Open | diagnostics 只记录 backend/status/error 和一次 step duration；没有 islands/pairs/contacts/query/capacity/step debt/cook residency | per-world diagnostic snapshot、high-water/overflow/fault/cook metrics 与 capture/replay timeline |

## 8. P1/P2：Ragdoll、Character、Vehicle 与高级物理

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| PHY3-P1-027 | Open | `RagdollSkeletonBone` 与 `RagdollBoneProfile` 使用 String bone path；generator 按 local translation length 粗略生成 capsule（`physics/editor/src/ragdoll_profile_editor.rs:13-69`） | skeleton/rig signature、stable bone IDs、mesh/skin/orientation fit、versioned PhysicsAsset artifact |
| PHY3-P1-028 | Open | runtime ragdoll drive 只是把动画 pose 映射到同步 body；没有 physical animation drive profile、authority handoff、recovery 或 network/save receipt | ragdoll instance transaction、native constraints、drive/strength/mode、spawn/despawn/rollback/reload |
| PHY3-P1-029 | Open | 当前没有专用 character controller；普通 query/body 无 step/slope/snap/slide/platform 语义 | dedicated controller service 与 deterministic movement result；参考 Fyrox character contract |
| PHY3-P1-030 | Open | vehicle、soft body、cloth、rope、destruction 没有独立 provider/solver owner | 按产品优先级拆为独立 artifact/provider/capability；禁止挂靠 Physics 总称伪完成 |
| PHY3-P2-001 | Open | 无 async/multi-rate physics、rewind/resimulation、late input correction | fixed snapshot/command/event cursor、rollback/resim 与 network determinism corpus |
| PHY3-P2-002 | Open | 无 large world/origin shift/partition streaming body 与跨 cell constraint | world partition identity、origin shift barrier、streaming residency 和 cross-cell lifecycle |
| PHY3-P2-003 | Open | 无 GPU/CPU query batching、job scheduling、query cache | backend-native batch/query arena、parallel scheduling、budget 与 profiling |
| PHY3-P2-004 | Open | 无 solver visual debugger、capture/replay、remote inspector 和竞争 benchmark | typed capture format、replay oracle、timeline、同语义 Unreal/Godot/Fyrox comparison |

## 9. 参考引擎差异

- Unreal `BodyInstance` 与 `WorldCollision` 将 body/shape lifecycle、每 shape response、CCD/sleep/DOF、mass/inertia、profile query、异步创建输入和 teardown 作为同一产品合同；Zircon 目前把 Scene DTO、manager map、Jolt backend、query fallback 分散在不同 owner。
- Unreal Chaos `PBDRigidsSolver` 拥有 `PrepareAdvanceBy`/`AdvanceSolverBy`、task dispatcher、island/spatial acceleration、event manager、material mirror 和 rewind capture（`PBDRigidsSolver.h`）；Jolt 路径的 post-step Rust projection 和全局 world mutex 不能对标该 solver boundary。
- Godot PhysicsServer3D/DirectSpaceState 以 RID 管理 space/body/shape/joint，提供 contacts budget、shape local transform、query flush/MT command queue，并公开 contact impulse、shape 与 collider identity；Zircon 的 unbounded Vec DTO 和空 Jolt query 不具备同语义。
- Fyrox physics graph 直接拥有 Rapier pipeline、broad/narrow phase、CCD、joint sets、QueryPipeline、debug render、caller-owned query storage 和 kinematic controller 的 slope/autostep/snap/slide；Zircon 还没有这些产品 owner。
- Bevy `Time<Fixed>` 以 accumulator/overstep 驱动 0、1 或多次 schedule，每个 fixed system 看到固定 timestep；Zircon 当前虽有 `clock.rs`，production `tick_scene_world` 仍存在第二套 step planner，必须 hard cutover 为单一 authority。
- Unity Graphics 只作为 consumer 参考：VFX rigid-body collision binder 将 contact point/normal 作为事件 payload；它不能替代 solver，但说明 Physics -> VFX/render/deformation 的 handoff 需要 typed contact identity、resident data 和 event readiness。

## 10. 必须保留与必须删除

保留：Runtime-owned neutral contracts、Scene serialization、`Arc` immutable Level snapshot、typed command queue、Jolt FFI 初始化/销毁底座、builtin geometry 作为明确的 reference/test backend、Physics Material source schema 和 ragdoll 生成意图。

必须删除或 hard cutover：默认 builtin/Jolt `Ready`、多张 global mutex map、每 tick 全 Scene projection、Jolt 激活时 builtin query/contact fallback、post-step Rust constraint projection、triangle compound 生产 mesh 路径、裸 EntityId 跨 generation、silent sanitize/drop、空 dist callback、空 query result 当成功、未携带 receipt 的 Level 事件发布。

## 11. 依赖顺序与资格门

1. **M0 truth gate**：为 catalog/provider reachability、false Ready、双 fixed clock、Jolt empty query/event fallback、sanitize silent drop 建立 RED tests。
2. **M1 world authority**：实现 `PhysicsProviderDescriptor`、`PhysicsWorldInstance`、config/provider/world generation、0..N fixed substeps、atomic `StepReceipt`。
3. **M2 artifact**：Material、CollisionProfile、Shape、多 shape/subshape、PhysicsAsset/Ragdoll source -> cooked artifact -> residency/reload。
4. **M3 backend**：Jolt native filter/material/mass/constraint/query/listener、budget、teardown；builtin 只保留 test/reference profile。
5. **M4 product consumers**：Scene writeback、AI/Navigation/Animation/Particles/Sound 按 precision/generation admission；Editor preview/debug 由同一 publication 消费。
6. **M5 qualification**：fault/scale/soak/determinism/replay、跨平台和同语义竞品 benchmark；全部通过后才允许 Shipping Ready。

当前 Gate 判定：G01 schema roundtrip Partial；G02 generation rejection Partial；G03 command admission Partial；G04 native rigid body Partial；G05-G42 Fail。Partial 只代表局部源代码基础，不能发布能力。

## 12. 本轮边界

本轮只新增 review 文档及其索引/覆盖记录，不修改 Rust、Cargo、manifest、ABI、测试或 ZUI。物理插件与 runtime selected files 仍有 working-tree 漂移，任何实施前都必须重取文件集合、HEAD/epoch、fingerprint 与相关 owner 的 currentness。Tooling 按用户要求排除，未来迁移 Rust 时另立 owner。
