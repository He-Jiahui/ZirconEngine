use crate::scene::ecs::{SystemOrderingConstraint, SystemSetId, SystemStage};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneSystemMetadata {
    id: String,
    stage: SystemStage,
    order: i32,
    sets: Vec<SystemSetId>,
    constraints: Vec<SystemOrderingConstraint>,
}

impl SceneSystemMetadata {
    pub fn new(id: impl Into<String>, stage: SystemStage, order: i32) -> Self {
        Self {
            id: id.into(),
            stage,
            order,
            sets: Vec::new(),
            constraints: Vec::new(),
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
}
