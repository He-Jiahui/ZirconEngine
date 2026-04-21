use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemStage {
    PreUpdate,
    Update,
    LateUpdate,
    FixedUpdate,
    RenderExtract,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schedule {
    pub stages: Vec<SystemStage>,
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            stages: vec![
                SystemStage::PreUpdate,
                SystemStage::FixedUpdate,
                SystemStage::Update,
                SystemStage::LateUpdate,
                SystemStage::RenderExtract,
            ],
        }
    }
}
