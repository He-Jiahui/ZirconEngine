use crate::scene::ecs::SystemStage;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneSystemMetadata {
    id: String,
    stage: SystemStage,
    order: i32,
}

impl SceneSystemMetadata {
    pub fn new(id: impl Into<String>, stage: SystemStage, order: i32) -> Self {
        Self {
            id: id.into(),
            stage,
            order,
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
}
