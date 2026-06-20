use crate::scene::dynamic_scene::{DynamicScene, DynamicSceneError};

impl DynamicScene {
    pub fn to_versioned_json_pretty(&self) -> Result<String, DynamicSceneError> {
        serde_json::to_string_pretty(self).map_err(|error| DynamicSceneError::Parse {
            reason: error.to_string(),
        })
    }
}
