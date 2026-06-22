use std::sync::Arc;

use crate::core::{CoreError, CoreHandle};

use super::super::ASSET_MANAGER_NAME;
use super::AssetManagerHandle;

pub fn resolve_asset_manager(core: &CoreHandle) -> Result<Arc<AssetManagerHandle>, CoreError> {
    core.resolve_manager::<AssetManagerHandle>(ASSET_MANAGER_NAME)
}
