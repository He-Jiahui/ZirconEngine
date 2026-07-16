use crate::scene::dynamic_scene::{DynamicScene, DynamicSceneError};
use zircon_runtime_interface::serialization::write_versioned_text;

impl DynamicScene {
    pub fn to_versioned_json_pretty(&self) -> Result<String, DynamicSceneError> {
        self.ensure_supported()?;
        Ok(write_versioned_text(self)?)
    }
}
