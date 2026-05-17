use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use crate::asset::AssetImportError;
use crate::core::resource::ResourceLocator;
use crate::plugin::PluginPackageManifest;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageAssetRegistry {
    roots_by_package_id: BTreeMap<String, PathBuf>,
}

impl PackageAssetRegistry {
    pub fn register_root(
        &mut self,
        package_id: impl Into<String>,
        assets_root: impl AsRef<Path>,
    ) -> Result<(), AssetImportError> {
        let package_id = validate_package_id(package_id.into())?;
        let assets_root = assets_root.as_ref();
        if assets_root.as_os_str().is_empty() {
            return Err(AssetImportError::Parse(format!(
                "package {package_id} asset root cannot be empty"
            )));
        }
        self.roots_by_package_id
            .insert(package_id, assets_root.to_path_buf());
        Ok(())
    }

    pub fn register_manifest_roots(
        &mut self,
        manifest: &PluginPackageManifest,
        package_root: impl AsRef<Path>,
    ) -> Result<(), AssetImportError> {
        let asset_roots = manifest.asset_roots_or_default();
        if asset_roots.len() != 1 {
            return Err(AssetImportError::Parse(format!(
                "package {} declares {} asset roots; package:// currently requires exactly one root",
                manifest.package_id(),
                asset_roots.len()
            )));
        }
        let asset_root = validate_relative_asset_root(&asset_roots[0])?;
        self.register_root(
            manifest.package_id(),
            package_root.as_ref().join(asset_root),
        )
    }

    pub fn root_for_package(&self, package_id: &str) -> Option<&Path> {
        self.roots_by_package_id
            .get(package_id)
            .map(PathBuf::as_path)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Path)> {
        self.roots_by_package_id
            .iter()
            .map(|(package_id, root)| (package_id.as_str(), root.as_path()))
    }
}

fn validate_package_id(package_id: String) -> Result<String, AssetImportError> {
    let probe = ResourceLocator::parse(&format!("package://{package_id}/__package_root_probe"))?;
    if probe.package_id() != Some(package_id.as_str()) {
        return Err(AssetImportError::Parse(format!(
            "invalid package asset id {package_id}"
        )));
    }
    Ok(package_id)
}

fn validate_relative_asset_root(asset_root: &str) -> Result<&Path, AssetImportError> {
    let path = Path::new(asset_root);
    if path.as_os_str().is_empty() {
        return Err(AssetImportError::Parse(
            "package asset root cannot be empty".to_string(),
        ));
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::Prefix(_) | Component::RootDir | Component::ParentDir
        )
    }) {
        return Err(AssetImportError::Parse(format!(
            "package asset root {asset_root} must be relative and contained by the package root"
        )));
    }
    Ok(path)
}
