use std::sync::Arc;

use crate::core::{CoreError, CoreHandle};

use super::super::ASSET_MANAGER_NAME;
use super::{AssetManager, AssetManagerHandle};

pub fn resolve_asset_manager(core: &CoreHandle) -> Result<Arc<dyn AssetManager>, CoreError> {
    core.resolve_manager::<AssetManagerHandle>(ASSET_MANAGER_NAME)
        .map(|holder| holder.shared())
}
