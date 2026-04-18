use std::path::Path;

use crate::AssetUri;

pub(in crate::editor::manager) fn display_name_for_locator(locator: &AssetUri) -> String {
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
