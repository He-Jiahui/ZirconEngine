use std::path::{Path, PathBuf};

use zircon_runtime::asset::ZShaderDocumentV2;

use crate::error::{ShaderPrewarmAssetScanError, ShaderPrewarmAssetScanResult};

pub(super) fn meta_path_for_single_source(meta_path: &Path) -> PathBuf {
    let file_name = meta_path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".zmeta"))
        .unwrap_or_default();
    meta_path.with_file_name(file_name)
}

pub(super) fn is_inside_compound_shader_source(path: &Path, inventory_paths: &[PathBuf]) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    let Some(parent_name) = parent.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    parent.parent().is_some_and(|grandparent| {
        let metadata_path = grandparent.join(format!("{parent_name}.zmeta"));
        inventory_paths
            .iter()
            .any(|candidate| candidate == &metadata_path)
    })
}

pub(super) fn is_zmeta(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.ends_with(".zmeta"))
}

pub(super) fn has_sidecar_zmeta(path: &Path, inventory_paths: &[PathBuf]) -> bool {
    let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    let metadata_path = path.with_file_name(format!("{file_name}.zmeta"));
    inventory_paths
        .iter()
        .any(|candidate| candidate == &metadata_path)
}

pub(super) fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

pub(super) fn stable_label_for_path(asset_root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(asset_root).unwrap_or(path);
    let normalized = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    format!("asset-scan://{normalized}")
}

pub(super) fn wgsl_files_for_document(
    package_dir: &Path,
    document: &ZShaderDocumentV2,
    inventory_paths: &[PathBuf],
) -> ShaderPrewarmAssetScanResult<Vec<PathBuf>> {
    if !document.wgsl_files().is_empty() {
        return Ok(document.wgsl_files().iter().map(PathBuf::from).collect());
    }
    inventory_paths
        .iter()
        .filter(|path| path.starts_with(package_dir) && has_extension(path, "wgsl"))
        .map(|path| {
            path.strip_prefix(package_dir)
                .map(PathBuf::from)
                .map_err(
                    |source| ShaderPrewarmAssetScanError::ShaderSourceOutsidePackageDir {
                        source_path: path.clone(),
                        package_dir: package_dir.to_path_buf(),
                        source,
                    },
                )
        })
        .collect()
}

pub(super) fn primary_zshader_path(
    package_dir: &Path,
    inventory_paths: &[PathBuf],
) -> Option<PathBuf> {
    inventory_paths
        .iter()
        .find(|path| path.starts_with(package_dir) && has_extension(path, "zshader"))
        .cloned()
}

pub(super) fn content_hash(source: &str) -> String {
    blake3::hash(source.as_bytes()).to_hex().to_string()
}
