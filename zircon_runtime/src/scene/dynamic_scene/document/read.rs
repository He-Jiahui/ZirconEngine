use crate::scene::dynamic_scene::{DynamicScene, DynamicSceneError};
use zircon_runtime_interface::serialization::{load_versioned, Format};

impl DynamicScene {
    pub fn from_versioned_json(json: &str) -> Result<Self, DynamicSceneError> {
        let loaded = load_versioned::<Self>(json.as_bytes(), Format::Text)?;
        let scene = loaded.value;
        scene.ensure_supported()?;
        Ok(scene)
    }
}
