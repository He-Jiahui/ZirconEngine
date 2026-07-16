use serde::{Deserialize, Serialize};
use zircon_runtime_interface::resource::ResourceLocator;

use crate::core::editor_operation::EditorOperationPath;

/// Project-stable route shared by the asset registry and domain toolkits.
///
/// The route stores a canonical locator instead of a machine-local source path. The host resolves
/// that locator through the active project authority only when it restores a toolkit session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetToolkitOpenRoute {
    asset_locator: ResourceLocator,
    open_operation: EditorOperationPath,
}

impl AssetToolkitOpenRoute {
    pub fn new(asset_locator: ResourceLocator, open_operation: EditorOperationPath) -> Self {
        Self {
            asset_locator,
            open_operation,
        }
    }

    pub fn asset_locator(&self) -> &ResourceLocator {
        &self.asset_locator
    }

    pub fn open_operation(&self) -> &EditorOperationPath {
        &self.open_operation
    }
}
