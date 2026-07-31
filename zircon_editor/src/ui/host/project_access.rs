use std::path::{Path, PathBuf};
use std::sync::Arc;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::{asset_manager_handle, AssetManager};
use zircon_runtime::asset::{AssetImportError, AssetUri};
use zircon_runtime::core::framework::foundation::ConfigManager;
use zircon_runtime::core::manager::{resolve_manager_service, ManagerResolver};
use zircon_runtime::diagnostic_log::write_log;
use zircon_runtime::scene::LevelMetadata;

use crate::core::project::ProjectAuthority;
use crate::ui::host::editor_asset_manager::{editor_asset_manager_handle, EditorAssetManager};
use crate::ui::workbench::project::EditorProjectDocument;

use super::editor_error::EditorError;
use super::editor_ui_host::EditorUiHost;

impl EditorUiHost {
    pub(super) fn open_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<EditorProjectDocument, EditorError> {
        let opened = ProjectAuthority::default().open_project(&path)?;
        self.open_prepared_project(opened.into_project())
    }

    pub(super) fn open_prepared_project(
        &self,
        project: ProjectManager,
    ) -> Result<EditorProjectDocument, EditorError> {
        let asset_manager = self.asset_manager()?;
        let project_info = asset_manager.open_prepared_project(project)?;
        let project = asset_manager.current_project_snapshot().ok_or_else(|| {
            EditorError::Project("runtime did not retain the opened project generation".to_string())
        })?;
        let editor_asset_manager = self.editor_asset_manager()?;
        editor_asset_manager.refresh_from_runtime_project()?;
        self.restart_ui_asset_workspace_watcher()?;
        let document = EditorProjectDocument::load_from_activated_project(&project, project_info)?;
        let catalog = editor_asset_manager.catalog_snapshot();
        write_log(
            "editor_project_open",
            project_opened_diagnostic(
                &document.root_path,
                &document.project_info.name,
                document.manifest.library_version,
                &document.project_info.default_scene_uri,
                document.project_info.asset_count,
                document.project_info.ready_asset_count,
                document.project_info.failed_asset_count,
                document.project_info.registry_diagnostic_count,
                catalog.catalog_revision,
                catalog.publish_epoch,
                catalog.assets.len(),
                document.project_settings.startup_status(),
            ),
        );
        Ok(document)
    }

    pub(super) fn close_project(&self) -> Result<Option<PathBuf>, EditorError> {
        let closed_root = self.asset_manager()?.close_project()?;
        let Some(closed_root) = closed_root else {
            return Ok(None);
        };

        let _ = post_committed_project_close_sync(
            "stop_ui_asset_workspace_watcher",
            self.restart_ui_asset_workspace_watcher(),
        );
        let _ = post_committed_project_close_sync(
            "deactivate_editor_asset_projection",
            self.editor_asset_manager().and_then(|manager| {
                manager
                    .deactivate_runtime_project()
                    .map_err(EditorError::from)
            }),
        );
        Ok(Some(closed_root))
    }

    pub(super) fn save_project(
        &self,
        path: impl AsRef<Path>,
        world: &zircon_runtime::scene::Scene,
    ) -> Result<PathBuf, EditorError> {
        let workspace = self.project_workspace();
        let expected_root = ProjectAuthority::default().resolve_existing_project_root(&path)?;
        let asset_manager = self.asset_manager()?;
        let project = asset_manager.current_project_snapshot().ok_or_else(|| {
            EditorError::Project("cannot save without an active project generation".to_string())
        })?;
        if project.paths().root() != expected_root {
            return Err(EditorError::Project(format!(
                "active project generation {} does not match save target {}",
                project.paths().root().display(),
                expected_root.display()
            )));
        }
        EditorProjectDocument::save_to_project(&project, world, Some(&workspace))?;
        let project_root = project.paths().root().to_path_buf();
        // The scene commit is now durable. Catalog and watcher refreshes must not turn that
        // successful authoring save back into a dirty, apparently failed operation.
        let default_scene = project.manifest().default_scene.to_string();
        let Some(_) = post_persist_project_save_sync(
            "reimport_default_scene",
            asset_manager.import_asset(&default_scene),
        ) else {
            return Ok(project_root);
        };
        let Some(editor_asset_manager) =
            post_persist_project_save_sync("resolve_editor_assets", self.editor_asset_manager())
        else {
            return Ok(project_root);
        };
        let Some(_) = post_persist_project_save_sync(
            "refresh_editor_assets",
            editor_asset_manager.refresh_from_runtime_project(),
        ) else {
            return Ok(project_root);
        };
        let _ = post_persist_project_save_sync(
            "restart_ui_asset_workspace_watcher",
            self.restart_ui_asset_workspace_watcher(),
        );
        Ok(project_root)
    }

    pub(super) fn create_runtime_level(
        &self,
        scene: zircon_runtime::scene::Scene,
    ) -> Result<zircon_runtime::scene::LevelSystem, EditorError> {
        Ok(zircon_runtime::scene::create_level(
            &self.runtime_core()?,
            scene,
            LevelMetadata::default(),
        )?)
    }

    pub(super) fn config_manager(&self) -> Result<Arc<dyn ConfigManager>, EditorError> {
        let resolver = ManagerResolver::new(self.runtime_core()?);
        Ok(resolver.resolve(resolver.config_handle()?)?)
    }

    pub(super) fn asset_manager(&self) -> Result<Arc<dyn AssetManager>, EditorError> {
        let core = self.runtime_core()?;
        Ok(resolve_manager_service(
            &core,
            asset_manager_handle(&core)?,
        )?)
    }

    pub(super) fn editor_asset_manager(&self) -> Result<Arc<dyn EditorAssetManager>, EditorError> {
        let core = self.runtime_core()?;
        Ok(resolve_manager_service(
            &core,
            editor_asset_manager_handle(&core)?,
        )?)
    }

    pub(super) fn resolve_ui_asset_path(
        &self,
        asset_id: impl AsRef<str>,
    ) -> Result<PathBuf, EditorError> {
        let asset_id = normalize_ui_asset_asset_id(asset_id.as_ref());
        if let Some(relative) = asset_id.strip_prefix("res://") {
            let uri = AssetUri::parse(&format!("res://{relative}"))?;
            return self
                .asset_manager()?
                .current_project_source_path(&uri)?
                .ok_or_else(|| {
                    EditorError::UiAsset(format!(
                        "cannot resolve {asset_id} without an open project"
                    ))
                });
        }
        Ok(PathBuf::from(asset_id))
    }

    pub(super) fn resolve_asset_locator_path(
        &self,
        asset_locator: &AssetUri,
    ) -> Result<PathBuf, EditorError> {
        self.asset_manager()?
            .current_project_source_path(asset_locator)?
            .ok_or_else(|| {
                EditorError::UiAsset(format!(
                    "cannot resolve {asset_locator} without an open project"
                ))
            })
    }

    pub(super) fn current_project_snapshot(&self) -> Result<Option<ProjectManager>, EditorError> {
        Ok(self.asset_manager()?.current_project_snapshot())
    }
}

pub(crate) fn resolve_existing_project_asset_path(
    project: &ProjectManager,
    asset_id: &str,
) -> Result<PathBuf, EditorError> {
    let uri = AssetUri::parse(asset_id)?;
    Ok(project.source_path_for_uri(&uri)?)
}

fn project_opened_diagnostic(
    project_root: &Path,
    project_name: &str,
    manifest_version: u32,
    default_scene_uri: &str,
    registry_asset_count: usize,
    registry_ready_asset_count: usize,
    registry_failed_asset_count: usize,
    registry_diagnostic_count: usize,
    project_generation: u64,
    project_generation_publish_epoch: u64,
    catalog_asset_count: usize,
    settings_source: &str,
) -> String {
    // Product diagnostics are whitespace-delimited key/value records, so every free-form field
    // must be encoded as one token before F1/F5 tooling can compare it across machines.
    let project_root = percent_encode_diagnostic_token(&project_root.to_string_lossy());
    let manifest_identity =
        percent_encode_diagnostic_token(&format!("{project_name}@v{manifest_version}"));
    let scene_uri = percent_encode_diagnostic_token(default_scene_uri);
    let settings_source = percent_encode_diagnostic_token(settings_source);
    format!(
        "editor_project_open result=completed project_root={project_root} manifest_identity={manifest_identity} scene_uri={scene_uri} registry_asset_count={registry_asset_count} registry_ready_asset_count={registry_ready_asset_count} registry_failed_asset_count={registry_failed_asset_count} registry_diagnostic_count={registry_diagnostic_count} project_generation={project_generation} project_generation_publish_epoch={project_generation_publish_epoch} catalog_asset_count={catalog_asset_count} settings_source={settings_source}",
    )
}

pub(crate) fn percent_encode_diagnostic_token(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[(byte >> 4) as usize]));
            encoded.push(char::from(HEX[(byte & 0x0f) as usize]));
        }
    }
    encoded
}

fn post_persist_project_save_sync<T, E>(phase: &str, result: Result<T, E>) -> Option<T>
where
    E: std::fmt::Display,
{
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            write_log(
                "editor_project_save",
                project_save_post_persist_sync_diagnostic(phase, &error),
            );
            None
        }
    }
}

fn project_save_post_persist_sync_diagnostic(
    phase: &str,
    error: &impl std::fmt::Display,
) -> String {
    format!("editor_project_save result=post_persist_sync_failed phase={phase} error={error}")
}

fn post_committed_project_close_sync<T, E>(phase: &str, result: Result<T, E>) -> Option<T>
where
    E: std::fmt::Display,
{
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            write_log(
                "editor_project_close",
                project_close_post_commit_sync_diagnostic(phase, &error),
            );
            None
        }
    }
}

fn project_close_post_commit_sync_diagnostic(
    phase: &str,
    error: &impl std::fmt::Display,
) -> String {
    format!("editor_project_close result=post_commit_sync_failed phase={phase} error={error}")
}

pub(crate) fn resolve_project_asset_write_path(
    project: &ProjectManager,
    asset_id: &str,
) -> Result<PathBuf, EditorError> {
    let uri = AssetUri::parse(asset_id)?;
    Ok(project.existing_or_primary_project_source_path_for_uri(&uri)?)
}

pub(crate) fn project_asset_id_for_source_path(
    project: &ProjectManager,
    source_path: &Path,
) -> Result<Option<String>, EditorError> {
    match project.project_uri_for_source_path(source_path) {
        Ok(uri) => Ok(Some(uri.to_string())),
        Err(AssetImportError::SourceOutsideProjectAssetRoots { .. }) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn normalize_ui_asset_asset_id(asset_id: &str) -> &str {
    asset_id
        .split_once('#')
        .map(|(path, _)| path)
        .unwrap_or(asset_id)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        post_persist_project_save_sync, project_opened_diagnostic,
        project_save_post_persist_sync_diagnostic,
    };

    #[test]
    fn project_open_diagnostic_records_the_catalog_generation() {
        let diagnostic = project_opened_diagnostic(
            Path::new("C:\\F1 Project"),
            "F1 Save",
            1,
            "res://scenes/main.scene.toml",
            7,
            6,
            1,
            2,
            3,
            9,
            7,
            "persisted-v1",
        );

        assert!(diagnostic.starts_with("editor_project_open result=completed"));
        assert!(diagnostic.contains("project_root=C%3A%5CF1%20Project"));
        assert!(diagnostic.contains("manifest_identity=F1%20Save%40v1"));
        assert!(diagnostic.contains("scene_uri=res%3A%2F%2Fscenes%2Fmain.scene.toml"));
        assert!(diagnostic.contains("registry_asset_count=7"));
        assert!(diagnostic.contains("registry_ready_asset_count=6"));
        assert!(diagnostic.contains("registry_failed_asset_count=1"));
        assert!(diagnostic.contains("registry_diagnostic_count=2"));
        assert!(diagnostic.contains("project_generation=3"));
        assert!(diagnostic.contains("project_generation_publish_epoch=9"));
        assert!(diagnostic.contains("catalog_asset_count=7"));
        assert!(diagnostic.contains("settings_source=persisted-v1"));
    }

    #[test]
    fn post_persist_save_sync_failure_is_diagnostic_only() {
        assert_eq!(
            post_persist_project_save_sync("reimport_default_scene", Ok::<_, &str>(7)),
            Some(7)
        );
        assert_eq!(
            post_persist_project_save_sync("refresh_editor_assets", Err::<(), _>("catalog stale")),
            None
        );

        let diagnostic =
            project_save_post_persist_sync_diagnostic("refresh_editor_assets", "catalog stale");
        assert!(diagnostic.contains("result=post_persist_sync_failed"));
        assert!(diagnostic.contains("phase=refresh_editor_assets"));
        assert!(diagnostic.contains("error=catalog stale"));
    }
}
