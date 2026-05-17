use serde::Deserialize;

use crate::scene::World;

use super::{DynamicScene, DynamicSceneError};

#[derive(Deserialize)]
struct LegacyProjectDocument {
    world: World,
}

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
        scene.ensure_supported_version()?;
        Ok(scene)
    }

    pub fn to_versioned_json_pretty(&self) -> Result<String, DynamicSceneError> {
        serde_json::to_string_pretty(self).map_err(|error| DynamicSceneError::Parse {
            reason: error.to_string(),
        })
    }
}
