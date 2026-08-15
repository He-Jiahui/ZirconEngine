---
related_code:
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/core/framework/scene/physics
  - zircon_runtime/src/scene/level_system
  - zircon_runtime/src/runtime_diagnostics
  - zircon_plugins/physics/plugin.toml
  - zircon_plugins/physics/runtime
  - zircon_plugins/physics/editor
  - zircon_plugins/physics/dist
  - zircon_plugins/animation/runtime
  - zircon_editor/src/core/editor_extension.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_plugins/03-physics.md
  - docs/plans/performance/01/2026-07-18-runtime-core-framework-physics-static-review.md
  - docs/plans/zircon_plugins/03/failure-2026-08-01-physics-debug-overlay-provider-missing.md
  - docs/plans/zircon_runtime/runtime/07/failure-2026-07-22-level-system-runtime-state-frame-snapshot.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-world-fixed-component-storage-and-stable-query-index.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/PhysicsEngine/BodyInstance.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/CollisionQueryParams.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/WorldCollision.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Collision/WorldCollision.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Collision/WorldCollisionAsync.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/Chaos/Public/PBDRigidsSolver.h
  - dev/godot/servers/physics_3d/physics_server_3d.h
  - dev/godot/servers/physics_3d/direct_states/physics_direct_space_state_3d.h
  - dev/godot/servers/physics_3d/physics_server_3d_wrap_mt.h
  - dev/Fyrox/fyrox-impl/src/scene/graph/physics/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/collider.rs
  - dev/Fyrox/fyrox-impl/src/scene/rigidbody.rs
  - dev/Fyrox/fyrox-impl/src/scene/joint.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 08A · Physics Runtime 工程化差距

## 1. 结论

Zircon Physics 已经不是空壳。中立层拥有 body、collider、joint、material、query、contact、trigger、fixed-step plan、skeletal pose feed 等 DTO；插件注册了 `physics.step` 与 `physics.sync_to_scene`；body command 有每 world 4,096 条硬上限和有限数校验；backend handle 带 generation；LevelSystem 发布的 contact/trigger frame snapshot 使用 `Arc`，并已引入 world replacement epoch 防止旧 world 结果覆盖新 world。插件 manifest 把成熟度写成 `experimental`，七项 runtime capability 都写成 `partial`，这一点比虚报 completed 更可靠。后续重构必须保留这些边界。

但当前产品交付尚不能被称为工程级物理系统。默认 physics runtime feature 为空，`dist` 和 editor 都未透传 `backend-jolt`，默认设置在无 Jolt feature 时是 `backend = "unconfigured"`、`simulation = Disabled`。也就是说，普通 workspace、native dist 和 export 路径并没有证明真实 solver 会进入产物；可手动选择的 builtin 只做积分、近似 overlap 和少量位置投影，不是碰撞求解器。

即使启用 Jolt，产品 manager 仍每个 fixed tick 扫描全部 scene node，深拷贝整份 body/collider/joint/material snapshot，再在 Jolt manager 中重建映射。查询不会调用 Jolt broad phase，而是克隆 synchronized world 后线性扫描；Jolt backend 的 `ray_cast`、`shape_cast`、`shape_overlap` 方法为空。contact/trigger 也没有来自 Jolt listener/manifold，而是从同步描述做 `O(N^2)` 近似。六类 joint 虽有 DTO，但 Jolt 后端没有建立 native constraint，而是在 native step 后运行插件侧投影。 authored collision layer/mask/matrix、local collider transform、scale、static friction、combine rule 等关键语义没有完整进入 native solver。

因此本轮登记 20 项 P1 和 4 项 P2，没有新增 P0。P1 不是要求一次补齐 Unreal Chaos 的全部功能，而是先把“交付了真实 Jolt 后端、一个固定时钟、持久 world、增量同步、native query/event/constraint、可烘焙 collision asset、可验证 authoring”变成产品事实。角色控制器、车辆、软体、破坏、网络 rewind/resimulation 等属于 P2 扩展能力，在 MVP 核心闭环和可量化基线完成前不得通过临时实现抢跑。

## 2. 审查边界与物理覆盖

### 2.1 已读范围

| 范围 | Rust 文件 | Rust 行数 | `#[test]` | 证据等级 |
|---|---:|---:|---:|---|
| `core/framework/physics` | 32 | 1,095 | 9 | E3：公共合同、校验、serde 与 manager/query surface |
| `core/framework/scene/physics` | 11 | 423 | 3 | E3：scene material/mass/CCD/sleep/joint metadata |
| physics runtime plugin | 77 | 12,366 | 76 | E3：manager、builtin、Jolt、constraint、query、events、skeletal、systems |
| physics editor plugin | 7 | 483 | 4 | E3：registration、debug DTO、ragdoll profile editor helper |
| physics native dist | 1 | 98 | 2 | E3：dynamic entry 与 manifest projection |

此外沿调用链复核了 scene property/asset/level-system integration、runtime diagnostics、animation pose feed、editor extension registry、app/export manifest tests 和 Cargo feature 传播。当前 physics plugin 一共 85 个 Rust 文件、12,947 行、82 个 test 属性；没有找到 Criterion/`#[bench]`、property-based、Loom、sanitizer、soak 或规模基准证据。测试数量不能替代真实 Jolt 产物、产品运行和负载证据。

本轮对照 Unreal BodyInstance、WorldCollision、async collision query 和 Chaos solver buffer/rewind surface；Godot PhysicsServer3D/direct space state；Fyrox 长期持有的 Rapier PhysicsWorld、dirty sync 与 caller-owned query storage；Bevy 单一 fixed clock。Unity Graphics 参考树针对 SRP、render graph、shader 和 GPU resource lifetime，不提供物理 owner，因此没有为满足引擎名单而制造错误类比，Unity 对照延后到 graphics 审查。

### 2.2 明确未做

- 没有修改 production code，没有运行 Cargo、Jolt native build、真实 editor/app/export、跨平台、长时稳定性或性能工具。本篇是 current-source 静态审查和重构计划，不是实现验收。
- `zircon_plugins/physics/runtime/src/plugin.rs` 与 `runtime_system.rs` 正被其他 Session 修改。本轮已复读当前 diff，看到 replacement epoch 防护正在增强；实现前仍必须重新取指纹并复核，本文标记 `source_recheck_required`。
- 没有把 navigation avoidance、animation IK、network replication 或 renderer picking 算作 Physics 已覆盖能力。它们只在边界处记录依赖，分别由后续 08B-08E、09/10 和 Editor 专篇拥有。
- 用户目标包含“表现和性能优于 Unreal”。静态结构不能证明这一目标。本篇把目标转换成可测 workload、正确性、延迟、吞吐、内存和产品交付门禁；没有基准数据前禁止写“优于”。

## 3. 当前闭环与必须保留的能力

### 3.1 中立合同和能力降级是正确方向

Physics contract 位于 `zircon_runtime::core::framework::physics`，Jolt 绑定位于可选 physics plugin，没有把具体后端类型泄漏到 scene component。`PhysicsBackendStatus` 能区分 Disabled、Unavailable 与 Ready，manifest 也没有把 partial 写成 complete。这符合 runtime absorption/support module 和可选 plugin 的仓库边界。重构应扩展 backend-specific capability matrix，而不是把 Jolt API 直接暴露给 engine-wide component。

### 3.2 identity、command budget 和 world replacement 防护已有基础

backend body/shape/constraint handle 使用 index + generation，避免简单复用整数句柄造成 ABA。body command queue 按 world 设 4,096 上限并拒绝非有限输入。LevelSystem physics frame snapshot 记录 generation 和 replacement epoch，旧 world 的 step/result 可以被拒绝；当前 `runtime_system.rs` 正把 world mutation、event publish 和 sync 回写迁移到 replacement-epoch 条件路径。这些机制应进入新的 per-world runtime state，而不是在重构时删掉。

### 3.3 scene serialization 的字段覆盖高于实际执行覆盖

scene asset 已能保存 body type、velocity、mass properties、CCD、sleep、material、box/sphere/capsule/cylinder/convex/triangle mesh/heightfield/compound、joint type、drive、constraint metadata 和 skeleton binding。Ragdoll profile TOML 也有较严格的有限数、形状、路径和拓扑校验，并在 spawn 失败时回滚。问题是 authoring、cook 和 native execution 没有兑现这组数据，而不是 DTO 数量不足。后续应以“字段从 editor 到 cooked artifact 到 backend 到 query/event/debug 的端到端语义一致”为门禁。

## 4. P1 差距清单

### P1-1：默认构建与发布产物没有交付真实 Jolt 后端

`zircon_plugins/physics/runtime/Cargo.toml` 的 `default = []`，Jolt 只在 `backend-jolt` feature 下依赖 `joltc-sys`。`physics/dist`、`physics/editor` 和 animation runtime 对 physics runtime 的依赖都没有启用该 feature；workspace 中也没有产品 crate 负责把它传播到 native dist。无 feature 时 manager 默认 `unconfigured/Disabled`，builtin 只是可显式选择的 downgrade。

目标建立一份唯一的 Physics backend build/profile matrix。client、server、editor、native dynamic、static-linked export 分别声明需要的 backend、target、compiler/runtime、determinism/profile；export plan 将 feature 解析为 lockable artifact identity。native dist 必须有可执行 smoke 证明加载后 backend 是预期 Jolt build，而不是仅断言 manifest 中出现 crate name。若某 target 不支持 Jolt，产品必须在 build/admission 阶段给出结构化拒绝，不得静默得到 Disabled physics。

### P1-2：engine fixed schedule 与 PhysicsSettings 形成两个互相矛盾的时钟 authority

WorldDriver 已按 engine `RuntimeTimeAdvance.fixed_step_plan` 将 FixedFirst/FixedUpdate/FixedPostUpdate 运行零次或多次，并给系统固定 delta。Physics manager 同时公开 `advance_clock`，维护 per-world accumulator、`fixed_hz` 和 `max_substeps`；但产品 `tick_scene_world` 不调用它，而是由 `fixed_update_step_plan` 为当前 system invocation 生成恰好一步，`step_seconds = context.delta_seconds`。因此 PhysicsSettings 的 fixed rate/max substeps 和 public `plan_world_step` 属于另一套未进入产品执行的时钟。

Bevy `Time<Fixed>` 的 accumulator/timestep 是 fixed schedule 的唯一 authority，schedule 根据 overstep 运行零次或多次。Zircon 应作同样的所有权裁决：首选由 engine time/schedule 决定 fixed tick，PhysicsSettings 只拥有 solver substep/iteration，而不再拥有第二个 game clock；若确实支持异步 physics frequency，则由一个明确的 PhysicsClock owner 产出独立 simulation generation，并通过 interpolation/extrapolation bridge 消费。旧 accumulator/public API 要硬切或正式接入，不能长期保留两种语义。

### P1-3：settings 更新不是失败原子事务，也不会使既有 world 达到新配置

`store_settings` 先修改内存 settings、清空全局 backend error，再调用 `core.store_config`；持久化失败会返回 error，但内存已经变化。只有 backend 名变化会清空 Jolt worlds 与 body commands；fixed_hz、max_substeps、simulation mode、gravity、collision matrix、solver group 或 material 行为变化都不会重建/迁移 `JoltManagedWorld`，其构造时保存的 settings 可永久落后。配置加载错误又被静默 fallback 成 defaults。

目标采用 `validate -> prepare per-world transition -> persist/stage -> atomic publish -> retire`。每项 setting 声明 live-update、next-step、world-rebuild 或 process-restart 语义；失败保持旧 settings + 旧 world 完整可用。world runtime 记录 applied settings generation，diagnostics 能列出 desired/applied generation、pending transition、失败类别与 affected worlds。fixed timestep 改变还要定义 accumulator/overstep 迁移，不得隐式清零或沿用错误余量。

### P1-4：每个 fixed tick 从 scene 全量投影并深拷贝 PhysicsWorldSyncState

`build_world_sync_state` 从 `world.node_records()` 开始扫描全部 node，为每个 body/collider/joint 重新计算/复制 transform、shape、material、constraint 和 string metadata。Jolt manager 再构造 collider `HashMap`、desired `BTreeMap`，clone body/collider/material；sync 回 scene 又遍历 body。现有 change detection 只在第二层比较两份 owned snapshot，无法消除 scene 全量 projection。稳定 100k-node scene 即使只有一个 body 变化，成本仍与全 scene/physics object 数量相关。

Fyrox 的 PhysicsWorld 长期持有 native set/query pipeline，并通过 `needs_sync_model` 避免无变化时写 native collider。Zircon 目标是持久 `PhysicsWorldRuntime`：scene mutation/change tick 产生 create/update/remove delta；稳定 Entity 到 Body/Shape/Constraint handle table；world transform 只对 dirty hierarchy frontier 计算；backend 只读 dirty commands；回写只包含 active bodies。debug snapshot 是按需视图，不再承担每帧同步协议。迁移完成后删除 full snapshot 热路径，禁止维持 full + delta 双实现。

### P1-5：per-world 可变状态被拆成多把全局 HashMap mutex，无法原子发布一个 simulation generation

manager 分别为 accumulator、synced world、contacts、trigger pairs、triggers、body commands、Jolt worlds 和 global last error 持有 `Arc<Mutex<HashMap<WorldHandle, ...>>>`。一次 step 会多次拿锁并在不同时间发布，consumer 可能看到不同 generation 的 sync/event/error。Jolt worlds 的全局 map lock 还覆盖 synchronize + native update，不相关 worlds 也会串行。poison recovery 直接取 inner value 继续，无法证明 native world、handle table 与 event buffers 仍一致。

目标让 `WorldHandle -> Arc<PhysicsWorldSlot>` 只负责 slot 查找；每个 slot 拥有单一 lifecycle lock/actor、backend、command ingress、active-state egress、query snapshot、event buffers、settings generation 和 health state。step 在 slot 内形成 generation transaction，完成后一次发布 immutable read view。不同 world 可以并行；poison/native fault 将 slot 标记 Faulted 并进入明确恢复流程，不能假装健康继续。

### P1-6：产品查询绕开 Jolt broad phase并克隆整份 synchronized world

manager 的 ray/shape cast/overlap 先从 mutex 中 clone `PhysicsWorldSyncState`，再用 builtin geometry 对所有 colliders 线性扫描。即使 Jolt 正在模拟，结果仍不是 Jolt shape/broad phase 的权威语义。更直接的证据是 Jolt backend trait 中 `ray_cast`、`shape_cast`、`shape_overlap` 三个实现体为空。mesh、heightfield、compound、旋转 box 等查询会 unsupported 或走 proxy/AABB 近似，无法与实际 solver shape 一致。

目标由 active backend 提供 persistent broad/narrow phase query view，并定义 query snapshot generation。同步查询可写入 caller-owned/reused storage；批量/异步查询返回 ticket，在固定 generation 上执行并有 deadline/result budget。fallback backend 必须显式返回 Unsupported/Approximate capability，不能把近似结果包装成 Jolt 查询结果。query 与 collision response 使用同一 layer/filter/material/subshape identity。

### P1-7：query contract 缺少工程化过滤、容量和确定性语义

当前 filter 只有 collision mask、sensor include、excluded entities 和 required group；`excluded_entities.contains` 对每 collider 是线性查找。每次 query 返回新 `Vec`。`First` 是当前存储遍历中第一个，而不是定义清楚的 any-hit/closest/deterministic-first。合同没有 caller capacity、overflow、trace/owner tag、object type/response channel、simple/complex、mobility、material/subshape/face、initial overlap、callback predicate 或 per-query profiling id。

Unreal `FCollisionQueryParams` 明确提供 complex、initial overlap、face index、mobility、ignore mask、inline ignored actor/component、trace/owner tag 和 stat id，并把 response/object query 参数分开；WorldCollision 有 async trace/sweep/overlap buffer。Godot direct space state 让 caller 提供结果缓冲区和最大结果数。Zircon 应定义 `QueryMode::{Any, Closest, AllSorted}`、固定 generation、max results/overflow、stable tie-break、filter plan 编译、batch/async ticket 和 caller-owned buffer。高层 convenience `Vec` 只能是冷路径 wrapper。

### P1-8：builtin backend 不是碰撞求解器，却可以报告 Ready

builtin 会积分重力、阻尼、translation/rotation，并计算 primitive/proxy overlap；它没有 broad phase、collision impulse、penetration resolution、friction/restitution、stacking、sleep islands、warm start 或 CCD。contact/trigger 是 pairwise `O(N^2)`；contact point 取近似中点，normal 多为 center-to-center。Fixed/Distance 只有位置投影，Hinge/Slider/ConeTwist/Generic6Dof solver 分支没有等价 native 求解。mesh/heightfield/compound 和旋转/缩放形状语义也不完整。

目标把 builtin 定义成 `ReferenceApproximation` 或 `TestFallback`，capability matrix 精确标注 integration、primitive query、approximate overlap 等支持级别；生产 profile 默认拒绝把它当真实 solver。若未来保留，用于 deterministic unit fixtures 和无 native tool 环境，不承担“可玩产品物理”。`Ready` 必须进一步包含 backend class、compiled feature、solver/query/event/constraint capability level。

### P1-9：Jolt 只有 moving/non-moving 两个 object layer，authoring collision filtering 没进入 native solver

Jolt `layers.rs` 只定义 `OBJECT_LAYER_NON_MOVING = 0` 与 `MOVING = 1`，pair filter 只区分静态/动态。scene authored layer、collision group、collision mask、collision matrix 和 solver groups 主要在 plugin 的 query/event 后处理使用，不能阻止 Jolt native contact 与 collision response。object layer 在 body type 切换时也只变更 static/moving。

目标编译 project collision profile 为 immutable `CollisionFilterGeneration`：object channels、trace channels、response table、sensor policy、solver groups 映射成 Jolt ObjectLayer/BroadPhaseLayer/filter callbacks，并能在 cook/export 中验证容量。backend collision、scene query、event、debug display 使用同一 generation。profile reload 要么通过受控 remap/reinsert，要么重建 world，不能只换 manager-side matrix。

### P1-10：contact/trigger 不是 native 事件流，contact 生命周期和物理信息不足

Jolt runtime 没有 ContactListener/manifold extraction。`read_active_states` 后调用 `refresh_events`，从当前同步 collider 描述重新做 pairwise overlap。contact 每个重叠帧生成一条只有 entity、other、point、normal 的事件，没有 Begin/Persist/End、impulse、penetration、relative velocity、subshape、face 或 material；trigger 虽有 Enter/Stay/Exit，但仍不是 native sensor callback。事件又同时 clone 到 ECS event 和 LevelSystem retained snapshot，没有容量、overflow、排序和 consumer lag 合同。

目标从 Jolt listener/contact manifold 采集 backend event，映射 stable entity/shape/subshape/material identity，并在 frame boundary 形成有界双缓冲。定义 contact Begin/Persist/End、trigger Enter/Stay/Exit、removed-body end policy、stable pair key、deterministic ordering、coalescing、overflow telemetry 和 per-consumer cursor。debug/Gameplay/Audio 不得各自重算碰撞。builtin 只能产生标注 approximate 的测试事件。

### P1-11：六类 joint 的 Jolt 实现是插件侧投影，不是 native constraint

Jolt backend 把 constraint 放在本地 handle pool，native update 后调用 `project_constraints(dt)`，再把位置/速度推回 body。Fixed/Distance 等使用简化规则，Hinge/Slider/ConeTwist/Generic6Dof 也没有与 Jolt native constraint、solver iteration、warm start、motor/limit/impulse 完全一致的执行。`max_force`、break force/torque、projection tolerances、`collide_connected`、body B angular correction 等 metadata 没有端到端兑现。

目标每个 public joint type 映射 native Jolt constraint settings/handle，定义 local frames、limits、drives、spring/damping、force mode、break event、collision disable 和 runtime mutation。unsupported 组合在 authoring/cook 阶段结构化失败。验收必须是 native handle 创建、solver behavior 和 break/drive product scene，不再以“DTO + projection + unit test”判 complete。

### P1-12：collider local transform、scale 与多 shape 语义在 Jolt 边界丢失

scene sync 能计算 entity world transform 与 collider local transform的组合，但 Jolt body 创建使用 body transform + shape；回读又把 `desc.collider.transform` 直接设为 body transform，local offset/rotation 被丢掉。world/entity scale 没有一致地烘入 native primitive/convex/mesh/compound dimensions，compound child API也只传 position/rotation。所有 sync map 以 EntityId 为 key，等价于每 entity 一个 body/collider/joint。

Godot body/area 可 `add_shape` 多次并为每个 shape 保存 local transform。Zircon 目标拆分 `BodyProxyId` 与 `ShapeInstanceId`：一个 body 可拥有零到多 shape instance，每个 instance 有 stable handle、local transform、scale policy、material/filter/sensor/subshape metadata。non-uniform scale 对 primitive、convex、mesh 的支持/烘焙规则必须明确；negative/near-zero scale 在 cook 或 runtime update 处拒绝。回读只更新 body state，不覆盖 authored shape local state。

### P1-13：TriangleMesh/HeightField 只有 DTO 与测试注册，没有产品 cook/import/derived-data 链

Jolt shape conversion 需要 `PhysicsMeshAsset` 已注册到 backend；全仓搜索中 `register_mesh_asset` 只出现在 Jolt backend tests。没有 product importer/cooker 把 render mesh 或 terrain 转换为验证后的 physics artifact，也没有 convex decomposition、welding、index/material mapping、platform cooking、content hash/cache key、streaming residency 或 asset reload 迁移。运行时引用可序列化，不代表 native shape 能创建。

目标建立 `PhysicsCookArtifact`：source asset + import settings + backend/version/platform/profile hash，包含 validated triangle mesh/heightfield/convex/compound 数据、bounds、material slots 和 diagnostics。cook 在 editor/import/export 共享同一 pipeline，产物进入 AssetManager/pack，runtime 只加载 immutable cooked blob并建立 shape cache/refcount。坏 mesh、退化 triangle、超预算数据在 cook 阶段失败；不得在 fixed tick 临时 cook。

### P1-14：mass、inertia 与 material contract 只实现了部分语义

auto/explicit mass 对 primitive/compound 有部分体积支持，但 convex/mesh/heightfield 不完整，capsule analytic inertia 缺失；explicit inertia tensor 最终被压成 uniform scalar multiplier，非对角或非均匀比例被拒绝。Jolt body creation 主要使用 dynamic friction，static friction、authored friction/restitution combine rule 没有完整映射。`PhysicsMaterialMetadata::default()` 由 derive 得到全零摩擦/恢复，这不是稳健的 engine default。

目标定义可烘焙 mass properties 和 native material table：质量、center of mass、完整对称 inertia tensor/rotation、density source、scale policy；material 有物理合理默认值、static/dynamic friction、restitution、combine priority、surface type和material slot。backend capability/validation 明确哪些 shape/body 组合可自动计算。scene、cook、solver、query hit 和 contact event 均引用 stable material id。

### P1-15：scene property/reflection/editor 的校验与 runtime schema 不一致

World property setter 对 mass、damping、friction、restitution、radius、layer/group 等字段可接受负值或非有限值；density 的简单 `<= 0` 还会让 NaN 穿过。错误通常在后续 sync sanitize 时被静默删除，作者只看到“没有物理”。property surface 只完整创建少数 primitive collider，convex/mesh/heightfield/compound 和高级 joint metadata 主要停留在 serde。RigidBody reflection 把部分可写字段标为 readonly，而底层又存在 setter，脚本、Inspector 和序列化能力互相矛盾。

目标让 component schema 成为唯一 validation owner，所有 World/property/reflection/editor/script/import 路径复用同一 typed validator 和 structured diagnostic。setter 要么原子接受并 bump dirty generation，要么保持旧值并返回错误。Inspector 为 shape union、asset reference、layer/response profile、mass/inertia、joint frames/limits/drives/break、CCD/sleep 提供完整 authoring。silent drop 只允许在损坏资产恢复模式，并必须产生 entity/field diagnostic。

### P1-16：ragdoll 是库级 helper，没有产品资产、编辑、生命周期与 native articulation 闭环

Ragdoll profile 解析、生成 helper、spawn rollback 和 animation pose feed tests 是有价值基础，但 production 搜索没有找到 `RagdollProfile::from_toml`、`spawn_configured` 或 editor generation helper 的非测试调用。Editor 的 “Generate Ragdoll Profile From Skeleton” 只是打开 view，没有 command handler 改写资产。runtime 每步扫描 `node_records()`、clone `RagdollRuntime`/BTreeMap，根据 bone leaf name 匹配；重复 leaf name 时静默丢行。spawn 返回 entities，却没有 skeleton destroy、profile reload、scene unload 的生命周期 owner。

约束仍是简化投影，不具备 articulation、自碰撞表、collision-aware body generation、joint limit fitting、physical animation strength profile、LOD、break、recovery/get-up、network state。根骨连接到 skeleton entity，但 skeleton 未必是 physics body，Jolt sync 可把缺失 connected body 静默改成 world constraint。

目标建立 typed `RagdollAsset` + importer/editor transaction + runtime `RagdollInstanceId` owner。骨骼使用 stable skeleton bone id，不用 leaf string 猜测；生成器输出可视化 body/joint/material/filter、validation report和可撤销 asset mutation。spawn/despawn/reload/scene replacement 必须回收 native handles。physical animation、blend/recovery、LOD、自碰撞和网络策略在 native constraint 能力完成后分层交付。

### P1-17：缺失 manager、无效 sync 和 native failure 经常被当成成功空帧

runtime system resolve 不到 `DefaultPhysicsManager` 时记录空 step 并返回 Ok；sync 系统找不到 manager/world snapshot 也直接 Ok。sanitize 会 retain/drop invalid or duplicate records但没有 entity diagnostic。Jolt step 失败会删除整个 world并把文本写入一条 global `last_backend_error`，没有 per-world generation、错误分类、retry/backoff 或 degraded policy。配置加载失败也静默默认。

目标为每个 world 建立 `Uninitialized/Ready/Degraded/Faulted/Rebuilding/Disabled` health state。required physics profile 缺 manager/backend/cooked asset 时，scene start 或 export admission 明确失败；optional profile 才允许 disabled，且 UI/diagnostics 可见。每个 dropped entity、backend error、event overflow 和 query unsupported 都带 world/entity/asset/generation/code，并受 cardinality/retention budget 控制。

### P1-18：capability 只描述功能名，没有描述 backend、精度、限制和可执行性

当前七项 capability 都是 `Partial`，这是诚实但不足的声明。consumer 无法知道当前产物是否编译 Jolt、builtin 是否 approximate、query 是否支持 mesh、constraint 是否 native、events 是否有 manifold、最大 body/query/event 容量、thread/determinism/rollback 能力。backend status 的 Ready 也不能区分真实 solver与积分 fallback。

目标发布 machine-readable `PhysicsCapabilityReport`，按 backend/build/profile列出 solver、shape、query、filter、event、constraint、cook、async、determinism、platform 和 limits。report 来自实际 linked backend 的自描述并绑定 artifact digest，而不是手写 manifest。Editor feature UI、runtime admission、export validation、tests 都消费同一 report。

### P1-19：Jolt native world 的容量、线程和 unsafe ownership 是硬编码且未形成平台预算

每个 Jolt world 创建自己的 temp allocator/thread pool，使用固定约 16 MiB temp storage、16,384 bodies/contact constraints、65,536 pairs 和固定 collision steps。没有 project/platform profile、memory budget、共享 worker policy、overflow行为或 telemetry。native wrapper 持 raw pointers并手工 `Send`，Jolt global init 使用 process-wide `OnceLock`；当前测试没有覆盖多 world 并发 create/step/drop、plugin unload、panic/native fault、thread shutdown 和 allocator lifetime。

目标建立 `PhysicsBackendDevice/Runtime` 级 global owner，管理 Jolt init、job system、allocator和world slots；容量来自 cooked project profile并有拒绝/增长策略。unsafe wrapper逐字段记录 thread-affinity、aliasing、destroy order和FFI callback lifetime，使用 concurrency/soak/sanitizer验证。plugin unload 前必须 drain worlds/tasks/callbacks并证明无 raw pointer/worker存活。

### P1-20：现有测试与里程碑把“API 形状完成”误当成“产品物理完成”

Physics03 记录曾在没有 Jolt ContactListener、使用同步 collider 近似事件时把事件 milestone 标 complete；在 JoltC 未提供所需 native constraint API、使用 plugin projection时把六类 constraint 标 complete；ragdoll主要凭 API/tests 标 complete；debug overlay milestone 标 complete 后，又出现仍 open 的 `physics-debug-overlay-provider-missing`。当前 82 个 tests 没有真实 native dist/export feature证明、规模 benchmark、cross-platform、fault、soak、product scene或长期 determinism。

目标重新定义 evidence gate：DTO/serde 是 Contract；native handle/solver/query/listener 是 Backend；editor/import/cook 是 Authoring；app/editor/export scene 是 Product；负载、fault、platform 是 Acceptance。只有五层都满足相关 capability 才能 complete。source-string assertion 和单元近似测试不能提升 Backend/Product 状态。旧 Physics03 的 M3/M4/M5/M6 应重开为本报告对应 milestones，不删除历史记录。

## 5. P2 扩展差距

### P2-1：缺少角色控制器与 gameplay locomotion 物理层

没有 production character controller、step/slope/ground probe、moving platform、push interaction、crouch resize、network prediction 或 animation root-motion contract。该能力不能用一个 kinematic capsule + raycast 临时拼接。应在 native queries/filter/event 和 fixed-clock完成后建立独立 controller runtime，定义 movement state、contact cache、penetration recovery、platform velocity、deterministic input/reconciliation及产品基准。

### P2-2：车辆、软体、布料、破坏与复杂场景物理尚无 owner

Godot PhysicsServer3D 至少有显式 soft-body server surface；Unreal Chaos 还覆盖 vehicles、cloth/destruction 等更大域。Zircon 当前没有车辆轮胎/悬挂、soft body/cloth coupling、fracture/destruction、buoyancy/field或large-world physics owner。它们必须作为后续独立模块和资产管线规划，不得塞进 `DefaultPhysicsManager`。MVP 后按实际游戏需求排序，不以“功能列表齐全”优先于核心 solver正确性。

### P2-3：异步 simulation、rewind/resimulation 和 network determinism 尚未设计

Unreal Chaos solver 明确有 multibuffer、dirty particle buffer、push/pull state、async task shutdown和rewind/resim surface。Zircon当前在主 fixed schedule内持可变 World 扫描、step、事件和回写，没有 simulation thread、input command generation、buffered result、rollback snapshot、resimulation、late correction或determinism fingerprint。Network 审查前不能假设现有 PhysicsWorldSyncState 可直接用于复制/回滚。

目标先在同步单线程路径证明正确和增量，再定义 optional async physics lane：immutable input generation、command buffer、double/triple buffered result、latency/interpolation policy、world replacement cancellation、rollback budget和determinism scope。不同平台/浮点后端不承诺 bitwise determinism时，要明确 server authoritative/reconciliation策略。

### P2-4：physics debug、profiling 与容量可视化没有产品闭环

physics editor 能生成 debug collider DTO，但 open failure 已确认没有真实 `ViewportOverlayProviderRegistration` 把 canonical geometry发布到共享 viewport extract。runtime diagnostics 主要记录一次 step duration/backend文字，没有 body/active/sleep/island/pair/contact/query/constraint/cook/cache/command/event/allocator/job指标，也没有 draw mask、selection过滤、generation或stale-frame清理。

目标由 PhysicsWorldRuntime发布低成本 counters和按需 debug extract；Editor Physics-owned overlay provider消费同一 generation，支持 body/shape/AABB/contact normal/manifold/joint/COM/sleep/island/filter显示，有容量和过期清理。诊断默认不物化全量 geometry，capture时才在预算内生成。

## 6. 参考引擎差距裁决

| 工程问题 | Zircon 当前 | 参考源码给出的边界 | Zircon 裁决 |
|---|---|---|---|
| fixed time | engine schedule与physics accumulator并存 | Bevy `Time<Fixed>` 单 accumulator驱动零到多次 fixed schedule | engine fixed clock唯一；solver substep另名另责 |
| world state | 每 tick全量 owned snapshot | Fyrox长期持有 body/collider/joint sets与query pipeline，dirty时sync | persistent per-world backend + delta command |
| query | clone world + linear scan +新Vec | Fyrox caller-owned query storage；Godot caller buffer/max results；Unreal sync/async query参数与buffer | backend query view、reused buffer、batch/async ticket |
| collision filtering | Jolt只分moving/non-moving | Godot body layer/mask；Unreal channel/object/response/query params | 编译 collision profile为backend filter generation |
| shapes | 一 entity一 collider，local/scale丢失 | Godot一 body多 shape并保存local transform | body/shape instance identity分离 |
| contacts | descriptor overlap近似 | native physics server/solver产生contact和state callback | native listener + bounded lifecycle event |
| solver/async | fixed stage内同步执行 | Chaos dirty/multibuffer/push-pull/rewind/resim | 先正确同步增量，再加可选async/rollback lane |
| completion proof | DTO、projection、test即可标complete | 参考引擎能力落在真实world/backend/product owner | Contract/Backend/Authoring/Product/Acceptance五层门禁 |

参考源码不是要求复制 Unreal 的类结构，也不是以文件数量衡量复杂度。需要吸收的是稳定 owner、长期状态、明确 generation、真实后端语义、异步/缓冲边界和产品证据。若 Zircon 选择比 Unreal 更简单的架构，应通过更低锁竞争、更少复制、更清晰 capability和相同/更好的 workload数据证明，而不是用临时近似缩短代码。

## 7. 目标架构

### 7.1 owner 与数据流

| Owner | 职责 | 禁止承担 |
|---|---|---|
| Engine Time/WorldDriver | 唯一 fixed game clock、schedule generation | Jolt solver iterations、physics私有第二时钟 |
| PhysicsBackendRuntime | Jolt init、job/allocator、backend build/capability、global teardown | scene entity/Editor asset mutation |
| PhysicsWorldSlot | backend world、handle table、settings generation、commands、events、health | 全局跨world mutex map内长时间step |
| Scene Physics Extract | 依据change tick输出create/update/remove和dirty transform | 每帧完整 `node_records()` snapshot |
| Physics Query View | 固定simulation generation的sync/batch/async query | clone整个world作为查询输入 |
| Physics Cook Pipeline | mesh/heightfield/convex/compound artifact和cache key | fixed tick临时cook |
| Physics Authoring | typed validation、asset transaction、preview/debug overlay | descriptor-only菜单或静默sanitize |
| Level/Animation/Network bridges | 消费immutable result generation和事件cursor | 重算collision或私自维护第二份物理真相 |

建议主链为：

1. scene mutation写 component change tick，并把 entity/hierarchy dirty frontier交给 Physics Extract。
2. Extract 将 create/update/remove/material/filter/settings commands写入有界 per-world ingress，附 scene generation和replacement epoch。
3. PhysicsWorldSlot在一个 simulation transaction中应用commands、step native backend、收集active body与native events。
4. transaction一次发布 `PhysicsReadGeneration`，包含active body page、query view/ticket source、bounded event pages、health/counters。
5. FixedPostUpdate只回写active dynamic bodies；LevelSystem、animation、audio、gameplay和debug持generation/Arc page读取，不clone完整world。
6. world replacement/reload关闭admission，取消旧ticket，等待step或按deadline fault，发布新slot；旧generation在reader释放后回收。

### 7.2 硬切要求

- 删除产品热路径中的 `PhysicsWorldSyncState` 全量输入协议；它可保留为按需调试/存档 DTO，但不能继续双写。
- 删除 manager query 对 synchronized world clone + builtin linear scan 的依赖；每个 backend显式实现或拒绝query。
- 删除 Jolt约束投影作为 “Jolt constraint” 的声明；迁移到native constraint后再恢复complete状态。
- builtin后端从production-ready候选中移除，除非未来实现完整solver并通过相同门禁。
- 清除第二套未消费的physics fixed clock，或将其升级为有明确async ownership的唯一physics clock。不能仅把其中一个标 deprecated并长期双轨。

## 8. 分层重构里程碑

### M0：能力真相、源码复核与交付基线

- 复核 `plugin.rs`/`runtime_system.rs` 外部修改和所有 physics owner 指纹。
- 生成 backend build/profile/capability matrix；修正 default/dist/editor/export feature传播。
- 将 builtin标为 test/approximate；重开 Physics03 M3-M6 false-green条目。
- 建立最小真实产品场景：editor preview、runtime app、native dist/export各加载同一Jolt artifact并上报digest/capability。

退出条件：任何 profile 都不会静默 Disabled；可从产物和运行态证明实际 backend；历史 milestone状态不再掩盖未实现native能力。

### M1：唯一 fixed clock 与 settings generation transaction

- 裁决 engine fixed clock与solver substep合同，硬切另一个时钟authority。
- settings validate/persist/publish/rebuild原子化，world记录desired/applied generation。
- 定义 simulation disabled/unavailable/faulted/rebuild状态和product admission。

退出条件：30/60/120 Hz、hitch/max catch-up、pause/time scale、runtime setting变化有确定step序列；持久化失败不改变运行态。

### M2：persistent PhysicsWorldSlot 与增量 scene bridge

- 创建 per-world slot、stable proxy/handle tables、bounded command ingress、immutable read generation。
- scene使用change tick和dirty hierarchy frontier输出delta；active body page回写。
- 合并分散mutex map，支持不同world并行和replacement cancellation。

退出条件：stable generation的node projection、shape clone、snapshot clone均为0；zero-change tick只产生常数级调度成本；world replacement无stale commit。

### M3：Jolt语义收敛

- collision profile编译到native layer/filter；多shape/local transform/scale语义一致。
- material、mass/inertia、CCD/sleep/body type runtime update完整映射。
- native body/shape生命周期、capacity、job/allocator和unsafe teardown通过平台验证。

退出条件：solver、query、event、debug对相同shape/filter/material给出一致identity和结果；unsupported在authoring/cook前失败。

### M4：native query、contact/trigger 与 constraint

- 实现 Jolt ray/sweep/overlap与 caller-owned/batch/async query API。
- 接入native contact/sensor listener、bounded lifecycle events和overflow telemetry。
- 六类joint映射native constraint、drive/limit/break/collide-connected。

退出条件：不再调用builtin geometry替代Jolt query/event/constraint；正确性和规模基准通过；旧projection实现删除。

### M5：physics cook/import/asset pipeline

- 建立backend/version/platform/profile keyed PhysicsCookArtifact。
- 接入mesh/terrain/convex/compound importer、validation、cache、pack和runtime residency。
- 定义asset reload、shape sharing/refcount和world migration。

退出条件：非测试产品可加载TriangleMesh/HeightField；fixed tick无cook；损坏/超预算资产在cook/export门禁失败。

### M6：scene/editor authoring、ragdoll与debug产品闭环

- component schema统一property/reflection/script/editor validation。
- 完整shape/joint/material/filter/mass authoring和undo/redo。
- RagdollAsset生成/编辑/spawn/despawn/reload/animation bridge；真实Physics overlay provider。

退出条件：从skeleton/mesh资产到编辑、保存、cook、Play/export、debug的产品链可复现；无descriptor-only入口或silent drop。

### M7：异步、network与高级physics能力

- 在M0-M6基线后评估simulation thread、多缓冲、rewind/resim、network reconciliation。
- 角色控制器优先于车辆/软体/布料/破坏；每项独立owner、资产、性能和product gate。
- 根据真实项目需求和benchmark决定是否引入Jolt扩展或独立specialized solver。

退出条件：每项能力有明确需求、owner、预算和与core physics的generation contract；不得以临时helper进入production profile。

## 9. 验收矩阵

### 9.1 正确性与语义

| 场景 | 必测内容 |
|---|---|
| primitive/compound/mesh | local offset/rotation、uniform/non-uniform scale、subshape/material identity、create/update/remove |
| rigid body | static/kinematic/dynamic切换、gravity/damping、sleep/wake、CCD高速穿透、COM/inertia |
| material | static/dynamic friction坡面、restitution、combine rule、material slot/contact hit |
| filtering | object/trace channel、response matrix、sensor、ignore entity/shape、runtime profile update |
| joints | 六类native constraint、limits、drive、break、collision disable、body removal |
| events | contact Begin/Persist/End、trigger Enter/Stay/Exit、body destroy/world replace、overflow |
| query | any/closest/all、initial overlap、mesh face/subshape、batch/async、generation/stale ticket |
| lifecycle | editor Play、pause/resume、scene reload、world replacement、backend fault、plugin unload |

### 9.2 性能与规模

至少覆盖 scene nodes/physics bodies/colliders 为 1、1k、100k；changed ratio为0%、0.1%、1%、10%、100%；fixed 30/60/120 Hz；queries每帧1、1k、100k；contacts sparse/dense；world数1/8/64。采集 extract visits、dirty commands、shape build、clone bytes、alloc/realloc、slot/global lock wait、broad/narrow candidates、solver/island时间、active body readback、query queue/latency、event depth/drop、native temp allocator high-water、CPU p50/p95/p99和resident memory。

硬门禁：stable generation全量node projection/shape/snapshot clone为0；query成本由broad-phase candidates和hit count主导；稳定result/event buffer realloc为0；不同world不因全局Jolt map lock串行；超容量产生可诊断拒绝/overflow，不发生静默丢失或无界增长。与Unreal比较必须固定硬件、场景、solver质量、tick rate、线程数和输出语义；不能用降低碰撞质量获得“更快”。

### 9.3 构建、平台与产品

- Windows MSVC、Linux clang及仓库声明支持的其他target分别验证native init/step/query/drop、debug/release、dynamic/static export。
- `cargo tree`/artifact manifest/runtime self-report三方证明Jolt build id、feature、compiler、ABI和digest；不能只查Cargo文本。
- Editor新建/导入/编辑/Play/停止、runtime app启动/退出、exported client/server、scene reload和plugin unload均有真实产品测试。
- Jolt native fault、invalid cooked asset、settings persistence failure、world replacement race、command/query/event overflow和worker shutdown有fault injection。
- 30分钟高碰撞soak、1,000次world create/drop、连续Play/reload、memory/handle/thread leak检测通过。

## 10. 实施约束

- 当前仓库MVP仍未完成。本报告只授权review；实现必须按M0依赖顺序进入现有 owner计划，不得直接跳到车辆、布料或大规模异步重写。
- `zircon_runtime` 保持backend-neutral contract和scene bridge，Jolt继续由physics plugin拥有；不得新建平行root physics crate逃避现有边界。
- source存在其他Session修改。实现前通过coordinator重新authorize/claim，复核open failures和current source；不覆盖 `plugin.rs`、`runtime_system.rs` 的外部工作。
- 每个milestone都必须保存命令、产物、测试和性能artifact。没有真实product/backend证据时只能写 `implementation_pending` 或 `static_complete_dynamic_pending`。
- 文档中的100k workload是规模曲线门禁，不表示所有场景必须同时模拟100k复杂动态刚体；报告必须公开shape/contact/solver配置，防止基准失真。

## 11. 本轮状态

本轮完成 Physics runtime/editor/dist、scene合同与产品调用点的首轮E3静态审查，未改production code。Physics进入 `review_complete / implementation_pending / source_recheck_required`；08总单元仍在进行，后续依次审查 Audio、Animation、Navigation 和 Network。Graphics/RHI/renderer不由本篇覆盖。
