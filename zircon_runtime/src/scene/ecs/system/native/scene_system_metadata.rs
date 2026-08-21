use crate::scene::ecs::{SystemOrderingConstraint, SystemSetId, SystemStage};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SceneSystemThreadAffinity {
    #[default]
    MainThreadOnly,
    WorkerSafe,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SceneSystemClockDomain {
    #[default]
    Virtual,
    Real,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneSystemMetadata {
    id: String,
    stage: SystemStage,
    order: i32,
    sets: Vec<SystemSetId>,
    constraints: Vec<SystemOrderingConstraint>,
    thread_affinity: SceneSystemThreadAffinity,
    clock_domain: SceneSystemClockDomain,
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
            clock_domain: SceneSystemClockDomain::Virtual,
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

    pub const fn clock_domain(&self) -> SceneSystemClockDomain {
        self.clock_domain
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

    pub const fn with_clock_domain(mut self, clock_domain: SceneSystemClockDomain) -> Self {
        self.clock_domain = clock_domain;
        self
    }
}
