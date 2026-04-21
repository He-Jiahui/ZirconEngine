use std::path::Path;

use zircon_runtime::asset::AssetUri;

pub(in crate::ui::host::editor_asset_manager::manager) fn display_name_for_locator(
    locator: &AssetUri,
) -> String {
    locator
        .label()
        .map(str::to_string)
        .or_else(|| {
            Path::new(locator.path())
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| locator.to_string())
}
