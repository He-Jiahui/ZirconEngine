use super::{PhysicsContactEvent, PhysicsWorldStepPlan};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PhysicsSceneStepResult {
    pub step_plan: PhysicsWorldStepPlan,
    pub contacts: Vec<PhysicsContactEvent>,
}
