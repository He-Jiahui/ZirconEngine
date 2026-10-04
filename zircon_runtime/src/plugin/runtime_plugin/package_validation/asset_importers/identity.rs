//! 单个导入器的清单身份由命名空间、所属包、版本和包内唯一性共同决定；重复性来自一次构建的包投影。
mod metadata;
mod uniqueness;

use crate::asset::AssetImporterDescriptor;

pub(super) fn validate_runtime_plugin_package_asset_importer_identity(
    package_id: &str,
    importer: &AssetImporterDescriptor,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    metadata::validate_runtime_plugin_package_asset_importer_metadata(
        package_id,
        importer,
        diagnostics,
    );
    uniqueness::validate_runtime_plugin_package_asset_importer_id_uniqueness(
        importer.id.as_str(),
        is_duplicate,
        diagnostics,
    );
}
