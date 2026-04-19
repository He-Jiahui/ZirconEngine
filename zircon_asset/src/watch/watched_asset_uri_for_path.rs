use std::path::Path;

use zircon_resource::ResourceLocatorError;

use crate::AssetUri;

use super::{asset_uri_for_path::asset_uri_for_path, is_meta_sidecar::is_meta_sidecar};

pub(crate) fn watched_asset_uri_for_path(
    assets_root: &Path,
    path: &Path,
) -> Result<AssetUri, ResourceLocatorError> {
    if is_meta_sidecar(path) {
        return Err(ResourceLocatorError::UnsupportedScheme(
            path.display().to_string(),
        ));
    }
    asset_uri_for_path(assets_root, path)
}
