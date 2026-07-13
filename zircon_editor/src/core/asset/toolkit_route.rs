use serde::{de, Deserialize, Deserializer, Serialize};
use zircon_runtime_interface::resource::ResourceLocator;

use crate::core::editor_operation::EditorOperationPath;

/// Project-stable route shared by the asset registry and domain toolkits.
///
/// The route stores a canonical locator instead of a machine-local source path. The host resolves
/// that locator through the active project authority only when it restores a toolkit session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AssetToolkitOpenRoute {
    asset_locator: ResourceLocator,
    open_operation: EditorOperationPath,
}

impl<'de> Deserialize<'de> for AssetToolkitOpenRoute {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireRoute {
            asset_locator: ResourceLocator,
            open_operation: String,
        }

        let route = WireRoute::deserialize(deserializer)?;
        let open_operation =
            EditorOperationPath::parse(route.open_operation).map_err(de::Error::custom)?;
        Ok(Self::new(route.asset_locator, open_operation))
    }
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
