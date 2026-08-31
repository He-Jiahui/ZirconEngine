use std::sync::Arc;

use zircon_runtime::scene::{DynamicScene, World};
use zircon_runtime_interface::project::{RelPath, RelPathError};

use super::PlaySceneSourceError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaySceneSource {
    Persisted(RelPath),
    Snapshot(Arc<str>),
}

impl PlaySceneSource {
    pub fn persisted(path: impl AsRef<str>) -> Result<Self, RelPathError> {
        RelPath::parse(path).map(Self::Persisted)
    }

    pub fn from_world(world: &World) -> Result<Self, PlaySceneSourceError> {
        let scene = DynamicScene::from_world(world)?;
        let document = scene.to_versioned_json_pretty()?;
        Ok(Self::Snapshot(Arc::from(document)))
    }
}
