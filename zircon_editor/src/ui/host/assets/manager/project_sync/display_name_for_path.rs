use std::path::Path;

use zircon_runtime::asset::AssetUri;

pub(super) fn display_name_for_path(source_path: &Path, locator: &AssetUri) -> String {
    let file_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(locator.path());
    if let Some(stripped) = file_name.strip_suffix(".toml") {
        stripped.to_string()
    } else {
        source_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or(file_name)
            .to_string()
    }
}
