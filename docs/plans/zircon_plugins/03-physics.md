# 03 · Physics 插件完善计划（Rigidbody / Collider / Constraint / Ragdoll / Query）

> 状态：工程化细化版 v2 · 优先级：P1 · 前置：[01 插件架构核心](01-plugin-architecture-core.md) M1
> 关联计划：`.codex/plans/Physics + Full Animation Support 新计划.md` · 现状文档：`docs/zircon_plugins/physics/runtime.md`
> 参考实现：Godot `servers/physics_3d`（PhysicsServer3D body/shape/joint/area API 形态）、Jolt 官方 Samples（约束族与 ragdoll）

## 1. 目标

把 `zircon_plugins/physics` 从 Experimental 推进到可支撑真实游戏的物理插件：真实 Jolt 后端、完整刚体与碰撞体形状族、trigger 闭环、约束/关节族、骨骼物理（ragdoll，与 animation 插件融合）、带过滤的完整查询 API。

## 2. 现状基线（实查）

成熟度 Experimental / Partial（7 capability 已声明，见 `zircon_plugins/physics/plugin.toml`）。

中立契约层 `zircon_runtime/src/core/framework/physics/` 已相当完整（**实现缺、DTO 在**）：

- `body_type.rs`（`PhysicsBodyType`）、`collider_shape.rs`（`PhysicsColliderShape`：**仅 Box/Sphere/Capsule 三变体**）、`joint_type.rs`（`PhysicsJointType`：Fixed/Distance/Hinge/Slider/ConeTwist/Generic6Dof **六种已全**）、`joint_drive.rs`、`joint_constraint_metadata.rs`、`query_filter.rs`（`PhysicsQueryFilter { collision_mask, include_sensors, excluded_entities, required_collision_group }`）、`ray_cast_query.rs`/`ray_cast_hit.rs`/`shape_cast_query.rs`/`shape_cast_hit.rs`/`shape_overlap_query.rs`/`shape_overlap_hit.rs`、`trigger_event.rs`/`trigger_event_kind.rs`、`contact_event.rs`、`skeleton_joint_binding.rs`、`simulation_mode.rs`、`settings.rs`、`manager.rs`（`PhysicsManager` trait）、`backend_state.rs`/`backend_status.rs`、`*_sync_state.rs` 族。

插件实现 `zircon_plugins/physics/runtime/src/`：

- `manager/`（builtin_step/clock/query/service/settings/validation/world_sync）、`query_contact/`（raycast：aabb/sphere/capsule/quadratic；overlap：pairwise/proxies/distance；contact/filter/geometry）、`trigger/`（event/pair/point/scan）、`scene_hook.rs`（FixedUpdate 步进入口）、`backend.rs`（**薄占位，无 trait 抽象**）、`module.rs`。
- `editor/src/lib.rs` 仅骨架。

缺口（按严重度）：

| # | 缺口 | 证据 |
|---|------|------|
| P1 | Jolt 后端仅占位；`backend.rs` 无后端 trait，builtin 步进直接长在 manager 上 | `runtime/src/backend.rs`、`manager/builtin_step.rs` |
| P2 | 约束/关节：六种 `PhysicsJointType` DTO 在，零实现 | `framework/physics/joint_type.rs` vs runtime 无 constraint 模块 |
| P3 | Ragdoll：`skeleton_joint_binding.rs` DTO 在，运行态/资产/动画交接全缺 | runtime 无 skeletal 模块 |
| P4 | `PhysicsColliderShape` 缺 Cylinder/ConvexHull/TriangleMesh/HeightField/Compound 五变体与物理材质完整绑定 | `collider_shape.rs` 三变体 |
| P5 | 刚体缺 CCD、休眠、质量属性（auto-from-shape/显式惯性张量）管理 | `manager/settings.rs`、`body_sync_state.rs` |
| P6 | 查询缺 sweep 多命中排序、trigger 包含策略统一；builtin raycast 仅解析三种形状 | `query_contact/raycast/` |
| P7 | scene hook 形态：无 `physics.step`/`physics.sync_to_scene` 系统锚点（01-M1 首批迁移对象） | `scene_hook.rs` |

## 3. 架构设计

中立契约维持在 `zircon_runtime::core::framework::physics`；组件（RigidBody/Collider/Joint）维持 scene 静态组件 + 字段补全。后端裁决维持：**Jolt 是唯一必交付真实后端，builtin 保留为无后端降级**，同一 crate 内 feature gate（`jolt`），不新增外部 crate。

### 3.1 PhysicsBackend trait（`runtime/src/backend/` [backend.rs 改造为目录]）

```rust
/// 不透明 handle，Godot RID 风格：u32 index + u32 generation，后端内部映射。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub struct BodyHandle(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub struct ShapeHandle(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub struct ConstraintHandle(u64);

pub trait PhysicsBackend: Send {
    fn create_shape(&mut self, shape: &PhysicsColliderShape, material: &PhysicsMaterialMetadata)
        -> Result<ShapeHandle, PhysicsBackendError>;
    fn create_body(&mut self, desc: &BodyDesc) -> Result<BodyHandle, PhysicsBackendError>;
    fn create_constraint(&mut self, desc: &ConstraintDesc) -> Result<ConstraintHandle, PhysicsBackendError>;
    fn destroy_body(&mut self, body: BodyHandle);
    fn destroy_constraint(&mut self, constraint: ConstraintHandle);
    /// 批量写：change-detection 收集的本步全部变更（位姿/速度/力/模式切换）。
    fn apply_commands(&mut self, commands: &[BodyCommand]);
    fn step(&mut self, dt: Real);
    /// 批量读：active bodies 的位姿/速度回读（仅醒着的 body）。
    fn read_active_states(&mut self, out: &mut Vec<(BodyHandle, BodySyncState)>);
    fn ray_cast(&self, query: &PhysicsRayCastQuery, filter: &PhysicsQueryFilter, out: &mut Vec<PhysicsRayCastHit>);
    fn shape_cast(&self, query: &PhysicsShapeCastQuery, filter: &PhysicsQueryFilter, out: &mut Vec<PhysicsShapeCastHit>);
    fn shape_overlap(&self, query: &PhysicsShapeOverlapQuery, filter: &PhysicsQueryFilter, out: &mut Vec<PhysicsShapeOverlapHit>);
    fn drain_events(&mut self, out: &mut PhysicsEventBuffer);   // contact + trigger enter/stay/exit
}

pub struct BodyDesc {
    pub body_type: PhysicsBodyType,
    pub shape: ShapeHandle,
    pub transform: PhysicsTransform,
    pub mass: MassProperties,        // Auto | Explicit { mass, center_of_mass, inertia }
    pub collision_layer: u32, pub collision_mask: u32,
    pub is_sensor: bool, pub ccd: bool, pub sleep: SleepPolicy,
}
```

- `backend/builtin/`：现有 `manager/builtin_step.rs` + `query_contact/` 收编为 trait 实现（行为保持，作为无 `jolt` feature 的降级；不要求新形状全集，TriMesh/HeightField 在 builtin 上 `Unsupported` 结构化报错）。
- `backend/jolt/`：绑定选型裁决为 **`joltc-sys`**（JoltPhysics 官方 JoltC C API 的 sys 绑定；理由：C ABI 稳定、跨 MSVC/clang 可控、避免 C++ 直绑定的 ABI 风险）。vendored 源码经 `build.rs` cmake 构建，CI 缓存产物。
- ECS ↔ backend 同步：`manager/world_sync.rs` [改造] 升级为 change-detection 驱动——`physics.step` 系统的查询带 `Changed<Collider>`/`Changed<RigidBody>` 过滤，仅同步变更；回写仅 `read_active_states` 返回的 active bodies。

### 3.2 组件与形状族（解决 P4/P5）

- `PhysicsColliderShape` [改造 `framework/physics/collider_shape.rs`] 新增五变体：

```rust
pub enum PhysicsColliderShape {
    Box { half_extents: [Real; 3] },
    Sphere { radius: Real },
    Capsule { radius: Real, half_height: Real },
    Cylinder { radius: Real, half_height: Real },                 // [新增]
    ConvexHull { points: Vec<[Real; 3]> },                        // [新增] 烘焙后顶点
    TriangleMesh { mesh: PhysicsMeshAssetRef },                   // [新增] static-only
    HeightField { resolution: [u32; 2], heights: PhysicsMeshAssetRef }, // [新增]
    Compound { children: Vec<(PhysicsTransform, Box<PhysicsColliderShape>)> }, // [新增]
}
```

- `RigidBody` 组件补全：`PhysicsBodyType` 切换（运行期切换经命令缓冲，下一 FixedUpdate 生效）、`MassProperties`、CCD 开关、`SleepPolicy { threshold_linear, threshold_angular, time_until_sleep }`。
- 速度/力写接口经 `PhysicsManager` 命令队列：FixedUpdate 外的写入缓冲到下一步进（`BodyCommand::{SetLinearVelocity, ApplyForce, ApplyImpulse, Teleport, SetBodyType, …}`）。
- `Trigger`：Collider 的 `is_sensor` 标志（沿用 `BodyDesc.is_sensor` 与契约 `trigger_event_kind.rs`）；事件统一 `PhysicsTriggerEvent { kind: Enter|Stay|Exit, pair }` 经 `register_event` 进事件总线，替换现有 `trigger/scan.rs` 轮询消费形态。

### 3.3 约束/关节族（解决 P2，`runtime/src/constraint/` [新增]）

六种 `PhysicsJointType` 全部实现，每种一个 owner 模块，参数 struct 定稿：

```rust
pub struct ConstraintDesc {
    pub joint_type: PhysicsJointType,            // 契约枚举，现有
    pub body_a: BodyHandle, pub body_b: Option<BodyHandle>,   // None = 接世界
    pub anchor_a: PhysicsTransform, pub anchor_b: PhysicsTransform,
    pub params: JointParams,
}
pub enum JointParams {
    Fixed,
    Distance { min: Real, max: Real, stiffness: Option<SpringSettings> },
    Hinge   { limit: Option<AngleLimit>, motor: Option<JointMotor> },     // motor 复用契约 joint_drive.rs
    Slider  { limit: Option<LinearLimit>, motor: Option<JointMotor> },
    ConeTwist { swing_limit: [Real; 2], twist_limit: Real },
    Generic6Dof { linear: [AxisConstraint; 3], angular: [AxisConstraint; 3] }, // ragdoll 用
}
```

- `Joint` 静态组件：引用两个实体 + 锚点 + `JointParams`；实体 → BodyHandle 解析在 `physics.step` 同步段完成。
- builtin 后端实现 Fixed/Distance 两种（降级最小集），其余四种 `Unsupported` 报错；Jolt 后端全六种。

### 3.4 Ragdoll 与骨骼物理（解决 P3，`runtime/src/skeletal/` [新增]）

- `RagdollProfile` 资产（`.ragdoll.toml`）：骨骼名 → body/shape/Generic6Dof limit 映射（复用契约 `skeleton_joint_binding.rs` 的绑定 DTO）；编辑器从 skeleton 自动生成初始 profile（M6）。
- 运行态三模式（`RagdollMode`）：
  - `Animated`：全部 body kinematic，逐 FixedUpdate 用动画目标姿态驱动；
  - `Simulated`：物理驱动，姿态回写 animation 的骨骼姿态通道；
  - `Blended { weight }`：按权重混合，权重范围可由 avatar mask 限定（见 04 §3.3）。
- **与 animation 的交接接口（与 [04 Animation](04-animation.md) §3.6 对偶，双方逐字一致）**：
  - animation `animation.evaluate`（PostUpdate）产出目标姿态写入 `SkeletalPoseTargets` 资源 → physics 下一 `physics.step`（FixedUpdate）消费；
  - physics 在 `physics.sync_to_scene`（FixedPostUpdate）把模拟姿态写入 `SimulatedPoseFeed` 资源（带插值 alpha），animation 下一帧 blend 阶段读取；
  - 物理插件不直接写 animation/editor 状态，两个资源均由 physics 插件 `register_resource` 注册、契约 DTO 放 `framework/physics/skeletal_pose.rs` [新增]。

### 3.5 查询 API（解决 P6）

- `raycast / shape_cast(sweep) / overlap` 统一沿用契约 `PhysicsQueryFilter`，新增 `QueryMode`（`First | Closest | All`，All 按距离排序）进各 `*_query.rs` [改造]。
- 查询在任意阶段可用（backend 不可变借用），写操作仅经命令缓冲在 FixedUpdate 生效——读写分离由 §3.1 的 `apply_commands`/`step` 边界保证。

### 3.6 ECS 集成（01-M1 首批验证对象）

- `scene_hook.rs` 删除，迁移为（`runtime_plugin` 注册路径 [改造]）：
  - `physics.step` ∈ FixedUpdate（in_set `physics.main`）：drain 命令缓冲 → change-detection 同步 → `backend.step(dt)` → `drain_events` 发布事件；
  - `physics.sync_to_scene` ∈ FixedPostUpdate：active body 位姿回写 transform + `SimulatedPoseFeed`。
- `register_event::<PhysicsContactEvent>` / `::<PhysicsTriggerEvent>`；`register_resource::<PhysicsWorldSettings>`（现 `settings.rs`）/`::<SkeletalPoseTargets>`/`::<SimulatedPoseFeed>`。

## 4. 模块文件树

```
zircon_runtime/src/core/framework/physics/
  collider_shape.rs            [改造] +5 形状变体
  skeletal_pose.rs             [新增] SkeletalPoseTargets/SimulatedPoseFeed DTO
zircon_plugins/physics/runtime/src/
  backend/mod.rs               [改造自 backend.rs] PhysicsBackend trait + handle 类型
  backend/builtin/mod.rs       [新增] 现 builtin_step/query_contact 收编
  backend/jolt/mod.rs          [新增] joltc-sys 绑定（feature jolt）
  backend/jolt/conversion.rs   [新增] 契约 DTO ↔ Jolt 类型映射
  constraint/{fixed,distance,hinge,slider,cone_twist,six_dof}.rs  [新增]
  skeletal/profile.rs          [新增] RagdollProfile 资产解析
  skeletal/runtime.rs          [新增] 三模式状态机与姿态交接
  manager/world_sync.rs        [改造] change-detection 化
  manager/command_buffer.rs    [新增] BodyCommand 队列
  query_contact/**             [改造] QueryMode/All 排序；收编进 backend/builtin
  trigger/**                   [改造] 事件改 register_event 发布
  scene_hook.rs                [删除] 迁移为系统锚点
```

## 5. 里程碑与任务分解

### M1 Backend trait 与 Jolt 接通（含 01-M1 锚点迁移）

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M1-T1 | PhysicsBackend trait + builtin 收编 | backend/* | — | 既有 query/trigger 测试全量保绿（行为不变断言） |
| M1-T2 | scene hook → physics.step/sync_to_scene 系统锚点 | scene_hook.rs 删除、注册路径 | 01-M1-T4 | `physics_step_anchor_registered_in_fixed_update` |
| M1-T3 | joltc-sys 接通：shape（三现有变体）/body/step | backend/jolt/* | M1-T1 | `jolt_box_stack_settles_deterministically`（固定 seed 容差快照） |
| M1-T4 | world_sync change-detection 化 + 命令缓冲 | world_sync.rs、command_buffer.rs | M1-T2 | `unchanged_bodies_skip_sync`、`force_applied_outside_fixed_update_lands_next_step` |

### M2 刚体完整化与形状族

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M2-T1 | 形状五变体（契约 + Jolt 映射；builtin Unsupported 路径） | collider_shape.rs、backend/jolt/conversion.rs | M1-T3 | `convex_hull_round_trips_through_jolt`、`trimesh_on_builtin_reports_unsupported` |
| M2-T2 | MassProperties/CCD/SleepPolicy/BodyType 运行期切换 | manager/settings.rs、backend/* | M1-T4 | `auto_mass_matches_shape_volume`、`kinematic_to_dynamic_preserves_velocity` |
| M2-T3 | QueryMode + sweep 多命中排序 | query_contact/*、各 *_query.rs | M1-T1 | `query_all_returns_distance_sorted_hits`、过滤矩阵测试 |

### M3 Trigger 与事件总线

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M3-T1 | trigger enter/stay/exit 经 drain_events 闭环 | trigger/*、backend/* | M2 | `trigger_lifecycle_enter_stay_exit_contract` |
| M3-T2 | ContactEvent/TriggerEvent 接 register_event | 注册路径 | 01-M2、M3-T1 | `trigger_event_reaches_event_store` |

### M4 约束族

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M4-T1 | ConstraintDesc/JointParams + Joint 组件解析 | constraint/*、backend trait | M2 | `joint_resolves_entity_pair_to_handles` |
| M4-T2 | 六约束 Jolt 实现 + motor/limit | constraint/*、backend/jolt | M4-T1 | `hinge_pendulum_period_matches_analytic`（容差）、`slider_limit_clamps_travel`、`six_dof_swing_twist_respects_limits` |
| M4-T3 | builtin Fixed/Distance 降级实现 | backend/builtin | M4-T1 | `builtin_unsupported_joint_reports_structured_error` |

### M5 Ragdoll（前置：[04 Animation](04-animation.md) M2 骨骼姿态通道）

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M5-T1 | skeletal_pose.rs DTO + 双资源注册 | framework/physics、注册路径 | 04-M2 | `pose_targets_visible_to_physics_step` |
| M5-T2 | RagdollProfile 资产解析与 body/constraint 生成 | skeletal/profile.rs | M4-T2 | `ragdoll_profile_spawns_expected_body_count` |
| M5-T3 | 三模式状态机 + 交接闭环 | skeletal/runtime.rs | M5-T1/T2 | `ragdoll_drop_golden_snapshot`、`animated_to_simulated_switch_has_no_pose_pop` |

### M6 Editor 调试

| 任务 | 内容 | 改动文件 | 依赖 | 新增测试 |
|------|------|---------|------|---------|
| M6-T1 | 碰撞体线框/trigger 染色 overlay（gizmos 通道，`View/Debug Overlays/Physics`） | physics/editor | [10 规范](10-editor-integration.md) | overlay 注册快照测试 |
| M6-T2 | 物理调试面板（`ai-physics-collision-layout.png` 布局）+ step 耗时进诊断 store | physics/editor | M6-T1 | editor 契约测试 |
| M6-T3 | RagdollProfile 编辑器（skeleton 自动生成初始 profile） | physics/editor | M5 | `generated_profile_covers_all_mapped_bones` |

## 6. 验收命令

```bash
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked
cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --features jolt --locked
```

## 7. 风险

- joltc-sys 跨平台构建（MSVC/clang + cmake）需进 CI 矩阵；先 Windows + Linux，macOS 随 [09 发行计划](09-export-publishing.md) M3 补齐。
- 确定性测试在不同 SIMD 路径下可能漂移：金样用容差断言而非逐位相等。
- Ragdoll 依赖 04-M2（骨骼姿态通道），排期上 M5 必须晚于之；契约 DTO（skeletal_pose.rs）可先行落地解耦排期。
- builtin 降级集的边界（哪些 Unsupported）必须写进 capability_statuses，避免用户在无 jolt 构建下静默缺功能。
