use std::{path::PathBuf, sync::Arc};

use zircon_runtime::scene::{DynamicScene, World};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaySceneSource {
    Persisted(PathBuf),
    Snapshot(Arc<str>),
}

impl PlaySceneSource {
    pub fn persisted(path: impl Into<PathBuf>) -> Self {
        Self::Persisted(path.into())
    }

    pub fn from_world(world: &World) -> Result<Self, String> {
        let scene = DynamicScene::from_world(world).map_err(|error| error.to_string())?;
        let document = scene
            .to_versioned_json_pretty()
            .map_err(|error| error.to_string())?;
        Ok(Self::Snapshot(Arc::from(document)))
    }
}
