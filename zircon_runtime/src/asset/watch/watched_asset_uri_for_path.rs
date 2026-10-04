use std::path::Path;

use crate::core::resource::ResourceLocatorError;

use crate::asset::AssetUri;

use super::{asset_uri_for_path::asset_uri_for_path, is_meta_sidecar::is_meta_sidecar};

/// 只把实际资产源映射进监听流；元数据侧车与原子写临时文件由项目事务自行管理。
pub(crate) fn watched_asset_uri_for_path(
    assets_root: &Path,
    path: &Path,
) -> Result<AssetUri, ResourceLocatorError> {
    if is_meta_sidecar(path)
        || crate::core::resource::io::is_atomic_write_transaction_path(path)
        || crate::core::resource::io::transaction::is_project_transaction_sibling_path(path)
    {
        return Err(ResourceLocatorError::UnsupportedScheme(
            path.display().to_string(),
        ));
    }
    asset_uri_for_path(assets_root, path)
}
