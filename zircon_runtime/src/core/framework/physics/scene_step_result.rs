use super::{PhysicsContactEvent, PhysicsTriggerEvent, PhysicsWorldStepPlan};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PhysicsSceneStepResult {
    pub step_plan: PhysicsWorldStepPlan,
    pub contacts: Vec<PhysicsContactEvent>,
    pub triggers: Vec<PhysicsTriggerEvent>,
}
