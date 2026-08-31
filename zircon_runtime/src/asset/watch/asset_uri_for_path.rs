use std::path::Path;

use crate::core::resource::ResourceLocatorError;

use crate::asset::AssetUri;

#[cfg(test)]
mod tests;

pub(super) fn asset_uri_for_path(
    assets_root: &Path,
    path: &Path,
) -> Result<AssetUri, ResourceLocatorError> {
    let relative = match path.strip_prefix(assets_root) {
        Ok(relative) => relative,
        Err(_) => {
            return Err(ResourceLocatorError::EscapeAttempt(
                path.display().to_string(),
            ));
        }
    };
    let mut normalized = String::with_capacity("res://".len() + relative.as_os_str().len());
    normalized.push_str("res://");
    for (index, component) in relative.components().enumerate() {
        if index != 0 {
            normalized.push('/');
        }
        normalized.push_str(&component.as_os_str().to_string_lossy());
    }
    AssetUri::parse(&normalized)
}
