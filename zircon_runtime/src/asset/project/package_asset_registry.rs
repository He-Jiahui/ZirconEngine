use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use crate::asset::AssetImportError;
use crate::core::resource::ResourceLocator;
use zircon_runtime_interface::project::RelPath;

use super::ProjectPaths;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageAssetRegistry {
    project_roots: Vec<PathBuf>,
    roots_by_package_id: BTreeMap<String, PathBuf>,
}

impl PackageAssetRegistry {
    /// Registers project assets under one physical project identity.
    ///
    /// Callers may provide an OS path alias, but registry roots are published from the resolved
    /// project root so downstream indexing and scanning do not retain lexical aliases.
    pub fn register_project_roots(
        &mut self,
        project_root: impl AsRef<Path>,
        asset_roots: &[RelPath],
    ) -> Result<(), AssetImportError> {
        if asset_roots.is_empty() {
            return Err(AssetImportError::MissingProjectAssetRoot);
        }
        let project_root = project_root.as_ref();
        let canonical_project_root =
            ProjectPaths::resolve_existing_path(project_root).map_err(|source| {
                AssetImportError::CanonicalProjectRoot {
                    path: project_root.to_path_buf(),
                    source,
                }
            })?;
        let mut resolved = Vec::with_capacity(asset_roots.len());
        for relative in asset_roots {
            let root = relative.join_to(&canonical_project_root);
            if !root.starts_with(&canonical_project_root) {
                return Err(AssetImportError::ProjectAssetRootOutsideProject {
                    project_root: canonical_project_root.clone(),
                    root,
                });
            }
            let canonical_asset_root =
                ProjectPaths::resolve_existing_path(&root).map_err(|source| {
                    AssetImportError::CanonicalProjectAssetRoot {
                        path: root.clone(),
                        source,
                    }
                })?;
            if !canonical_asset_root.starts_with(&canonical_project_root) {
                return Err(AssetImportError::CanonicalProjectAssetRootEscape {
                    project_root: canonical_project_root.clone(),
                    asset_root: canonical_asset_root,
                });
            }
            if resolved.contains(&canonical_asset_root) {
                return Err(AssetImportError::DuplicateProjectAssetRoot {
                    root: canonical_asset_root,
                });
            }
            resolved.push(canonical_asset_root);
        }
        self.project_roots = resolved;
        Ok(())
    }

    pub fn project_roots(&self) -> &[PathBuf] {
        &self.project_roots
    }

    pub fn primary_project_root(&self) -> Result<&Path, AssetImportError> {
        self.project_roots
            .first()
            .map(PathBuf::as_path)
            .ok_or(AssetImportError::MissingProjectAssetRoot)
    }

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
        // `package://` is a logical identity. Resolve its filesystem root once so aliases such
        // as Windows junctions and SUBST drives cannot leak into later source-path lookups.
        let resolved_assets_root = ProjectPaths::resolve_root(assets_root)?;
        self.roots_by_package_id
            .insert(package_id, resolved_assets_root);
        Ok(())
    }

    pub fn register_package_roots<Root>(
        &mut self,
        package_id: impl Into<String>,
        asset_roots: impl IntoIterator<Item = Root>,
        package_root: impl AsRef<Path>,
    ) -> Result<(), AssetImportError>
    where
        Root: AsRef<str>,
    {
        let package_id = package_id.into();
        let asset_roots = asset_roots
            .into_iter()
            .map(|root| root.as_ref().to_string())
            .collect::<Vec<_>>();
        if asset_roots.len() != 1 {
            return Err(AssetImportError::Parse(format!(
                "package {} declares {} asset roots; package:// currently requires exactly one root",
                package_id,
                asset_roots.len()
            )));
        }
        let asset_root = validate_relative_asset_root(&asset_roots[0])?;
        self.register_root(package_id, package_root.as_ref().join(asset_root))
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
