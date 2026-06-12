use serde::{Deserialize, Serialize};

use super::{InternalSceneSystem, SystemSetId, SystemStage};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemRef {
    System(String),
    Set(SystemSetId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemOrderingConstraint {
    Before(SystemRef),
    After(SystemRef),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneSystemDescriptor {
    pub id: String,
    pub stage: SystemStage,
    #[serde(default)]
    pub sets: Vec<SystemSetId>,
    #[serde(default)]
    pub constraints: Vec<SystemOrderingConstraint>,
    pub order: i32,
    pub system: InternalSceneSystem,
}

impl SceneSystemDescriptor {
    pub fn new(id: impl Into<String>, stage: SystemStage, system: InternalSceneSystem) -> Self {
        Self {
            id: id.into(),
            stage,
            sets: Vec::new(),
            constraints: Vec::new(),
            order: 0,
            system,
        }
    }

    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    pub fn in_set(mut self, set: SystemSetId) -> Self {
        self.sets.push(set);
        self
    }

    pub fn with_constraint(mut self, constraint: SystemOrderingConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    pub fn before(self, reference: SystemRef) -> Self {
        self.with_constraint(SystemOrderingConstraint::Before(reference))
    }

    pub fn after(self, reference: SystemRef) -> Self {
        self.with_constraint(SystemOrderingConstraint::After(reference))
    }

    pub fn system(&self) -> InternalSceneSystem {
        self.system
    }
}
