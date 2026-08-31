use crate::scene::ecs::{
    SceneSystemTickPolicy, SystemOrderingConstraint, SystemSetId, SystemStage,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SceneSystemThreadAffinity {
    #[default]
    MainThreadOnly,
    WorkerSafe,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneSystemMetadata {
    id: String,
    stage: SystemStage,
    order: i32,
    sets: Vec<SystemSetId>,
    constraints: Vec<SystemOrderingConstraint>,
    thread_affinity: SceneSystemThreadAffinity,
    tick_policy: SceneSystemTickPolicy,
}

impl SceneSystemMetadata {
    pub fn new(id: impl Into<String>, stage: SystemStage, order: i32) -> Self {
        Self {
            id: id.into(),
            stage,
            order,
            sets: Vec::new(),
            constraints: Vec::new(),
            thread_affinity: SceneSystemThreadAffinity::MainThreadOnly,
            tick_policy: SceneSystemTickPolicy::for_stage(stage),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn stage(&self) -> SystemStage {
        self.stage
    }

    pub fn order(&self) -> i32 {
        self.order
    }

    pub fn sets(&self) -> &[SystemSetId] {
        &self.sets
    }

    pub fn constraints(&self) -> &[SystemOrderingConstraint] {
        &self.constraints
    }

    pub const fn thread_affinity(&self) -> SceneSystemThreadAffinity {
        self.thread_affinity
    }

    pub const fn tick_policy(&self) -> SceneSystemTickPolicy {
        self.tick_policy
    }

    pub fn with_set(mut self, set: SystemSetId) -> Self {
        self.sets.push(set);
        self
    }

    pub fn with_sets(mut self, sets: impl IntoIterator<Item = SystemSetId>) -> Self {
        self.sets.extend(sets);
        self
    }

    pub fn with_constraint(mut self, constraint: SystemOrderingConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    pub fn with_constraints(
        mut self,
        constraints: impl IntoIterator<Item = SystemOrderingConstraint>,
    ) -> Self {
        self.constraints.extend(constraints);
        self
    }

    pub const fn with_thread_affinity(mut self, affinity: SceneSystemThreadAffinity) -> Self {
        self.thread_affinity = affinity;
        self
    }

    pub const fn with_tick_policy(mut self, tick_policy: SceneSystemTickPolicy) -> Self {
        self.tick_policy = tick_policy;
        self
    }
}
