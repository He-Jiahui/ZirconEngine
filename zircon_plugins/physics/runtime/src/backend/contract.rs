//! 定义 manager 与物理 provider 之间的后端契约，统一对象生命周期、命令、步进、状态读取、查询和事件出口。

use zircon_runtime::core::framework::{
    physics::{
        PhysicsBodySyncState, PhysicsColliderShape, PhysicsQueryFilter, PhysicsRayCastHit,
        PhysicsRayCastQuery, PhysicsShapeCastHit, PhysicsShapeCastQuery, PhysicsShapeOverlapHit,
        PhysicsShapeOverlapQuery,
    },
    scene::physics::PhysicsMaterialMetadata,
};
use zircon_runtime::core::math::Real;

use super::{
    BodyCommand, BodyDesc, BodyHandle, ConstraintDesc, ConstraintHandle, PhysicsBackendError,
    PhysicsEventBuffer, ShapeHandle,
};

/// 单个可独立调用的物理 provider 接口；manager 与低层客户端都通过此契约操作后端。
pub trait PhysicsBackend: Send {
    /// 返回用于选择与诊断的稳定 provider 名称。
    fn name(&self) -> &'static str;
    /// 按 provider 支持范围创建碰撞形状，并返回此实例管理的形状句柄。
    fn create_shape(
        &mut self,
        shape: &PhysicsColliderShape,
        material: &PhysicsMaterialMetadata,
    ) -> Result<ShapeHandle, PhysicsBackendError>;
    /// 用已创建的形状和同步状态创建刚体；句柄由当前 provider 实例管理。
    fn create_body(&mut self, desc: &BodyDesc) -> Result<BodyHandle, PhysicsBackendError>;
    /// 创建引用当前 provider 刚体句柄的约束。
    fn create_constraint(
        &mut self,
        desc: &ConstraintDesc,
    ) -> Result<ConstraintHandle, PhysicsBackendError>;
    /// 释放未被刚体引用的形状；对象仍在使用或句柄无效时返回错误。
    fn destroy_shape(&mut self, shape: ShapeHandle) -> Result<(), PhysicsBackendError>;
    /// 释放未被约束引用的刚体；对象仍在使用或句柄无效时返回错误。
    fn destroy_body(&mut self, body: BodyHandle) -> Result<(), PhysicsBackendError>;
    /// 释放约束句柄。
    fn destroy_constraint(
        &mut self,
        constraint: ConstraintHandle,
    ) -> Result<(), PhysicsBackendError>;
    // BUG: [CR-PHYSICS-BACKEND-0002] 公开 trait 直调不经过 manager 的有限值校验；Builtin 可保存 NaN 线速度；仅非 Static 刚体且对应平移轴未锁定时，step 才会把 NaN 写入该轴位移并仍标记活动。其他情形仍可能留下非有限速度状态。
    // 证据：manager/command_buffer.rs::queue_body_command、builtin/runtime.rs::apply_commands、builtin/step.rs::integrate_body_sync_state；关联 PHY4-P1-030。
    /// 将命令批次应用到后端刚体；Builtin 先验证句柄，Jolt 还验证形状与静态刚体约束，再写入命令。
    fn apply_commands(&mut self, commands: &[BodyCommand]) -> Result<(), PhysicsBackendError>;
    /// 使用单个有限正时间步推进 provider 状态。
    fn step(&mut self, dt: Real) -> Result<(), PhysicsBackendError>;
    /// 将本次需要写回的活动刚体状态追加到输出。
    fn read_active_states(&mut self, out: &mut Vec<(BodyHandle, PhysicsBodySyncState)>);
    // TODO: [CR-PHYSICS-BACKEND-0004] Jolt 的三个 trait 查询当前不写输出且接口没有 Unsupported 结果；manager 查询另走同步快照上的共享几何 helper。
    // 需要明确直接 trait 调用的能力/无结果语义，勿把 manager 快照查询误说成 Jolt backend 查询；证据：jolt/runtime.rs 查询实现、manager/query.rs；关联 PHY4-P1-017..019。
    /// 在 provider 的查询空间执行射线查询并追加命中。
    fn ray_cast(
        &self,
        query: &PhysicsRayCastQuery,
        filter: &PhysicsQueryFilter,
        out: &mut Vec<PhysicsRayCastHit>,
    );
    /// 在 provider 的查询空间执行形状扫掠并追加命中。
    fn shape_cast(
        &self,
        query: &PhysicsShapeCastQuery,
        filter: &PhysicsQueryFilter,
        out: &mut Vec<PhysicsShapeCastHit>,
    );
    /// 在 provider 的查询空间执行形状重叠查询并追加命中。
    fn shape_overlap(
        &self,
        query: &PhysicsShapeOverlapQuery,
        filter: &PhysicsQueryFilter,
        out: &mut Vec<PhysicsShapeOverlapHit>,
    );
    // 事件刷新时点在当前实现间不同：Builtin 在 step 结束时刷新，Jolt 在 read_active_states 结束时刷新；manager 固定 step→read→drain。
    // trait 直调方需沿用该顺序，合同疑问继续由既有 CR-PHYSICS-JOLT-0003 / PHY4-P1-021 追踪，不另建问题。
    /// 将已生成的接触与触发事件追加到输出缓冲区。
    fn drain_events(&mut self, out: &mut PhysicsEventBuffer);
}
