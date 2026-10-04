use crate::core::framework::scene::physics::PhysicsMaterialMetadata;
use crate::core::framework::scene::WorldHandle;
use crate::core::math::Real;

use super::{
    PhysicsBackendStatus, PhysicsContactEvent, PhysicsRayCastHit, PhysicsRayCastQuery,
    PhysicsSettings, PhysicsSettingsStoreError, PhysicsShapeCastHit, PhysicsShapeCastQuery,
    PhysicsShapeOverlapHit, PhysicsShapeOverlapQuery, PhysicsTriggerEvent, PhysicsWorldStepPlan,
    PhysicsWorldSyncState,
};

/// 物理插件注册给运行时的后端中立服务契约；场景按世界句柄提交快照，诊断层读取配置和状态。
pub trait PhysicsManager: Send + Sync {
    fn backend_name(&self) -> String;
    fn settings(&self) -> PhysicsSettings;
    /// 可持久化服务先写入配置再更新运行状态；只读实现可保留默认拒绝行为。
    fn store_settings(&self, _settings: PhysicsSettings) -> Result<(), PhysicsSettingsStoreError> {
        Err(PhysicsSettingsStoreError::read_only_backend(
            self.backend_name(),
        ))
    }
    fn default_material(&self) -> PhysicsMaterialMetadata;
    fn backend_status(&self) -> PhysicsBackendStatus;
    fn plan_world_step(&self, world: WorldHandle, delta_seconds: Real) -> PhysicsWorldStepPlan;
    /// 接收完整世界快照，供后端查询、接触检测和调试显示使用。
    fn sync_world(&self, sync: PhysicsWorldSyncState);
    /// 返回已同步状态；缺席表示该世界尚未同步或后端已清理其状态。
    fn synchronized_world(&self, world: WorldHandle) -> Option<PhysicsWorldSyncState>;
    fn ray_cast(&self, query: &PhysicsRayCastQuery) -> Vec<PhysicsRayCastHit>;
    // TODO: [CR-PHYSICS-FRAMEWORK-0001] 确认默认空结果能否表达“后端不支持形状查询”；目前与“没有命中”无法区分；下一步核对服务兼容策略及查询消费者。
    /// 对已同步世界查询形状重叠；兼容实现默认返回空结果。
    fn shape_overlap(&self, _query: &PhysicsShapeOverlapQuery) -> Vec<PhysicsShapeOverlapHit> {
        Vec::new()
    }
    /// 对已同步世界执行形状扫掠；兼容实现默认返回空结果。
    fn shape_cast(&self, _query: &PhysicsShapeCastQuery) -> Vec<PhysicsShapeCastHit> {
        Vec::new()
    }
    /// 一次性取出指定世界的接触事件，供固定更新阶段发布给场景。
    fn drain_contacts(&self, world: WorldHandle) -> Vec<PhysicsContactEvent>;
    /// 一次性取出触发事件；未提供触发支持的兼容实现返回空结果。
    fn drain_triggers(&self, _world: WorldHandle) -> Vec<PhysicsTriggerEvent> {
        Vec::new()
    }
}
