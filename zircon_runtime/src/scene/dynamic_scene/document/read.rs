use super::legacy::LegacyProjectDocument;
use crate::scene::dynamic_scene::{DynamicScene, DynamicSceneError};

impl DynamicScene {
    pub fn from_versioned_json(json: &str) -> Result<Self, DynamicSceneError> {
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|error| DynamicSceneError::Parse {
                reason: error.to_string(),
            })?;

        if value.get("world").is_some() {
            let document: LegacyProjectDocument =
                serde_json::from_value(value).map_err(|error| DynamicSceneError::Parse {
                    reason: error.to_string(),
                })?;
            return Self::from_world(&document.world);
        }

        let scene: Self =
            serde_json::from_value(value).map_err(|error| DynamicSceneError::Parse {
                reason: error.to_string(),
            })?;
        scene.ensure_supported()?;
        Ok(scene)
    }
}
