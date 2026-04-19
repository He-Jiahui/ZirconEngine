use std::path::Path;

use zircon_resource::ResourceLocatorError;

use crate::AssetUri;

pub(super) fn asset_uri_for_path(
    assets_root: &Path,
    path: &Path,
) -> Result<AssetUri, ResourceLocatorError> {
    let relative = match path.strip_prefix(assets_root) {
        Ok(relative) => relative,
        Err(_) => {
            return Err(ResourceLocatorError::EscapeAttempt(
                path.display().to_string(),
            ))
        }
    };
    let normalized = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    AssetUri::parse(&format!("res://{normalized}"))
}
