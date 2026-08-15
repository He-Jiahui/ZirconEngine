use std::path::{Path, PathBuf};

use crate::asset::{AssetImportError, AssetUri};

use super::{ProjectManager, ProjectPaths};

impl ProjectManager {
    /// Resolves an existing project source to its stable logical `res://` identity.
    ///
    /// Both the source and configured asset root are resolved through the filesystem before
    /// containment is checked, so casing and filesystem aliases cannot change the URI. New
    /// destinations intentionally use `primary_project_source_path_for_uri` instead.
    pub fn project_uri_for_source_path(&self, path: &Path) -> Result<AssetUri, AssetImportError> {
        let (_, resolved_root, resolved_path) = self.resolve_project_source_path(path)?;
        Self::source_uri_from_resolved_project_path(&resolved_root, &resolved_path)
    }

    pub(super) fn resolve_project_source_path(
        &self,
        source_path: &Path,
    ) -> Result<(&Path, PathBuf, PathBuf), AssetImportError> {
        let requested_path = source_path.to_path_buf();
        let resolved_path = ProjectPaths::resolve_existing_path(source_path)?;
        let mut roots = self
            .project_asset_roots()
            .iter()
            .map(|root| {
                ProjectPaths::resolve_existing_path(root)
                    .map(|resolved_root| (root.as_path(), resolved_root))
                    .map_err(|source| AssetImportError::CanonicalProjectAssetRoot {
                        path: root.clone(),
                        source,
                    })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|roots| {
                roots
                    .into_iter()
                    .filter(|(_, resolved_root)| resolved_path.starts_with(resolved_root))
                    .collect::<Vec<_>>()
            })?;

        match roots.len() {
            1 => {
                let (root, resolved_root) = roots.pop().unwrap();
                Ok((root, resolved_root, resolved_path))
            }
            0 => Err(AssetImportError::SourceOutsideProjectAssetRoots {
                path: requested_path,
            }),
            _ => Err(AssetImportError::ambiguous_project_source_path(
                requested_path,
                roots
                    .into_iter()
                    .map(|(root, _)| root.to_path_buf())
                    .collect(),
            )),
        }
    }

    fn source_uri_from_resolved_project_path(
        asset_root: &Path,
        path: &Path,
    ) -> Result<AssetUri, AssetImportError> {
        let relative = path.strip_prefix(asset_root).map_err(|_| {
            AssetImportError::SourceOutsideProjectAssetRoots {
                path: path.to_path_buf(),
            }
        })?;
        let relative = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        Ok(AssetUri::parse(&format!("res://{relative}"))?)
    }

    pub(super) fn source_uri_for_package_path(
        &self,
        package_id: &str,
        package_assets_root: &Path,
        path: &Path,
    ) -> Result<AssetUri, AssetImportError> {
        let relative = path.strip_prefix(package_assets_root).map_err(|error| {
            AssetImportError::Parse(format!(
                "package asset path {} is outside package assets root {}: {error}",
                path.display(),
                package_assets_root.display()
            ))
        })?;
        let relative = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        Ok(AssetUri::parse(&format!(
            "package://{package_id}/{relative}"
        ))?)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use zircon_runtime_interface::project::{render_project_template, ProjectTemplateId};

    use super::ProjectManager;

    static NEXT_TEST_PROJECT: AtomicU64 = AtomicU64::new(1);

    #[cfg(any(unix, windows))]
    #[test]
    fn physical_source_under_an_alias_root_keeps_its_project_uri_identity() {
        let parent = unique_temp_project_root("source-uri-alias");
        let physical_root = parent.join("physical-project");
        write_renderable_empty_template(&physical_root);
        let alias_root = parent.join("project-alias");
        create_directory_alias(&physical_root, &alias_root);

        let manager = ProjectManager::open(&alias_root).unwrap();
        let uri = manager
            .project_uri_for_source_path(&physical_root.join("assets/models/cube.obj"))
            .unwrap();

        assert_eq!(uri.to_string(), "res://models/cube.obj");
        drop(manager);
        remove_directory_alias(&alias_root);
        fs::remove_dir_all(parent).unwrap();
    }

    fn write_renderable_empty_template(root: &Path) {
        let rendered =
            render_project_template(ProjectTemplateId::RenderableEmpty, "SourceUriAlias").unwrap();
        for entry in rendered.entries {
            let destination = entry.path.join_to(root);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(destination, entry.bytes).unwrap();
        }
    }

    fn unique_temp_project_root(label: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sequence = NEXT_TEST_PROJECT.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "zircon_project_manager_{label}_{timestamp}_{sequence}"
        ))
    }

    #[cfg(unix)]
    fn create_directory_alias(target: &Path, link: &Path) {
        std::os::unix::fs::symlink(target, link).unwrap();
    }

    #[cfg(unix)]
    fn remove_directory_alias(link: &Path) {
        fs::remove_file(link).unwrap();
    }

    #[cfg(windows)]
    fn create_directory_alias(target: &Path, link: &Path) {
        let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
        let output = std::process::Command::new("cmd")
            .args(["/D", "/S", "/C"])
            .arg(command)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "create project alias fixture failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[cfg(windows)]
    fn remove_directory_alias(link: &Path) {
        fs::remove_dir(link).unwrap();
    }
}
