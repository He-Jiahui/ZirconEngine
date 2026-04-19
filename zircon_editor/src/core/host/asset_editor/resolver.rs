use std::fmt;
use std::sync::Arc;

use zircon_runtime::core::{CoreError, CoreHandle};

use crate::core::host::module::EDITOR_ASSET_MANAGER_NAME;

use super::EditorAssetManager;

#[derive(Clone)]
pub struct EditorAssetManagerHandle {
    inner: Arc<dyn EditorAssetManager>,
}

impl EditorAssetManagerHandle {
    pub fn new(inner: Arc<dyn EditorAssetManager>) -> Self {
        Self { inner }
    }

    pub fn shared(&self) -> Arc<dyn EditorAssetManager> {
        self.inner.clone()
    }
}

impl fmt::Debug for EditorAssetManagerHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EditorAssetManagerHandle").finish()
    }
}

pub fn resolve_editor_asset_manager(
    core: &CoreHandle,
) -> Result<Arc<dyn EditorAssetManager>, CoreError> {
    core.resolve_manager::<EditorAssetManagerHandle>(EDITOR_ASSET_MANAGER_NAME)
        .map(|holder| holder.shared())
}
