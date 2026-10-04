use super::{PhysicsContactEvent, PhysicsTriggerEvent, PhysicsWorldStepPlan};

/// 固定更新交回场景系统的单帧结果；接触和触发事件随后进入场景事件队列。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PhysicsSceneStepResult {
    pub step_plan: PhysicsWorldStepPlan,
    pub contacts: Vec<PhysicsContactEvent>,
    pub triggers: Vec<PhysicsTriggerEvent>,
}
