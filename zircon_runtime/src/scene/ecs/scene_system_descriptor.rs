use serde::{Deserialize, Serialize};

use super::{InternalSceneSystem, SystemStage};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneSystemDescriptor {
    pub id: String,
    pub stage: SystemStage,
    pub order: i32,
    pub system: InternalSceneSystem,
}

impl SceneSystemDescriptor {
    pub fn new(id: impl Into<String>, stage: SystemStage, system: InternalSceneSystem) -> Self {
        Self {
            id: id.into(),
            stage,
            order: 0,
            system,
        }
    }

    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    pub fn system(&self) -> InternalSceneSystem {
        self.system
    }
}
