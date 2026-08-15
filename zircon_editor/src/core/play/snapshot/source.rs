use std::sync::Arc;

use zircon_runtime::scene::{DynamicScene, World};
use zircon_runtime_interface::project::{RelPath, RelPathError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaySceneSource {
    Persisted(RelPath),
    Snapshot(Arc<str>),
}

impl PlaySceneSource {
    pub fn persisted(path: impl AsRef<str>) -> Result<Self, RelPathError> {
        RelPath::parse(path).map(Self::Persisted)
    }

    pub fn from_world(world: &World) -> Result<Self, String> {
        let scene = DynamicScene::from_world(world).map_err(|error| error.to_string())?;
        let document = scene
            .to_versioned_json_pretty()
            .map_err(|error| error.to_string())?;
        Ok(Self::Snapshot(Arc::from(document)))
    }
}
