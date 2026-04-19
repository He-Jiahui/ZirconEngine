use std::fmt;
use std::sync::Arc;

use super::AssetManager;

#[derive(Clone)]
pub struct AssetManagerHandle {
    inner: Arc<dyn AssetManager>,
}

impl AssetManagerHandle {
    pub fn new(inner: Arc<dyn AssetManager>) -> Self {
        Self { inner }
    }

    pub fn shared(&self) -> Arc<dyn AssetManager> {
        self.inner.clone()
    }
}

impl fmt::Debug for AssetManagerHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AssetManagerHandle").finish()
    }
}
