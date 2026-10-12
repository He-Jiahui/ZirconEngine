use std::path::Path;

pub(super) fn asset_importer_array<'a>(
    table: &'a toml::Table,
    relative_path: &Path,
) -> Option<&'a Vec<toml::Value>> {
    let Some(importers) = table.get("asset_importers") else {
        return None;
    };
    Some(importers.as_array().unwrap_or_else(|| {
        panic!("plugin manifest {relative_path:?} asset_importers should be an array")
    }))
}
