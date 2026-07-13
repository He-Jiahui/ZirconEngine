use std::collections::HashMap;

use zircon_runtime::asset::importer::AssetImportError;
use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::project::{AssetMetaDocument, PreviewState};
use zircon_runtime::core::framework::render::ShaderIdePreviewVariant;
use zircon_runtime::core::resource::{ResourceKind, ResourceState};
use zircon_runtime::graphics::write_shader_ide_env_for_project;

use crate::ui::host::editor_asset_manager::{
    AssetCatalogRecord, PreviewArtifactKey, PreviewCache, PreviewScheduler, ReferenceGraph,
};

use super::super::super::{EditorAssetChangeKind, EditorAssetChangeRecord};
use super::super::default_editor_asset_manager::DefaultEditorAssetManager;
use super::super::reference_analysis::direct_references;
use super::{
    display_name_for_path::display_name_for_path, meta_path_for_source::meta_path_for_source,
    preview_source_mtime::preview_source_mtime,
};

impl DefaultEditorAssetManager {
    pub fn sync_from_project(&self, project: ProjectManager) -> Result<(), AssetImportError> {
        let preview_cache = PreviewCache::new(project.paths().cache_root())?;
        let mut catalog_by_uuid = HashMap::new();
        let mut uuid_by_locator = HashMap::new();
        let mut preview_scheduler = PreviewScheduler::default();

        for metadata in project.registry().values() {
            let locator = metadata.primary_locator().clone();
            if locator.label().is_some() {
                continue;
            }
            let source_path = project.source_path_for_uri(&locator)?;
            let meta_path = meta_path_for_source(&source_path);
            let meta = AssetMetaDocument::load(&meta_path)?;
            let preview_state = meta.preview_state;
            let direct_references = if metadata.state == ResourceState::Ready {
                let imported = project.load_artifact_by_id(metadata.id())?;
                direct_references(&imported)
            } else {
                Vec::new()
            };
            let preview_artifact_path =
                preview_cache.path_for(&PreviewArtifactKey::thumbnail(meta.uuid));
            let file_name = source_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string();
            let extension = source_path
                .extension()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let diagnostics = metadata
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.clone())
                .collect::<Vec<_>>();
            let asset_uuid = meta.uuid;

            let record = AssetCatalogRecord {
                asset_uuid,
                asset_id: metadata.id(),
                locator: locator.clone(),
                kind: metadata.kind,
                display_name: display_name_for_path(&source_path, &locator),
                file_name,
                extension,
                meta_path,
                meta,
                source_mtime_unix_ms: preview_source_mtime(&source_path),
                source_hash: metadata.source_hash.clone(),
                preview_state,
                preview_artifact_path,
                dirty: preview_state == PreviewState::Dirty,
                diagnostics,
                direct_references,
            };
            if record.dirty {
                preview_scheduler.mark_dirty(record.asset_uuid);
            }

            uuid_by_locator.insert(locator, record.asset_uuid);
            catalog_by_uuid.insert(record.asset_uuid, record);
        }

        let reference_graph = ReferenceGraph::rebuild(catalog_by_uuid.values());
        refresh_shader_ide_env_after_import(&project)?;
        let primary_asset_root = project.primary_project_asset_root()?.to_path_buf();
        let change = {
            let mut state = self
                .state
                .write()
                .expect("editor asset state lock poisoned");
            state.project_root = Some(project.paths().root().to_path_buf());
            state.assets_root = Some(primary_asset_root);
            state.cache_root = Some(project.paths().cache_root().to_path_buf());
            state.project_name = project.manifest().name.clone();
            state.default_scene_uri = Some(project.manifest().default_scene.clone());
            state.catalog_revision += 1;
            state.project = Some(project);
            state.catalog_by_uuid = catalog_by_uuid;
            state.uuid_by_locator = uuid_by_locator;
            state.reference_graph = reference_graph;
            state.preview_cache = Some(preview_cache);
            state.preview_scheduler = preview_scheduler;

            EditorAssetChangeRecord {
                kind: EditorAssetChangeKind::CatalogChanged,
                catalog_revision: state.catalog_revision,
                uuid: None,
                locator: None,
            }
        };
        self.broadcast(change);
        Ok(())
    }
}

fn refresh_shader_ide_env_after_import(project: &ProjectManager) -> Result<(), AssetImportError> {
    let has_ready_shader = project
        .registry()
        .values()
        .any(|record| record.kind == ResourceKind::Shader && record.state == ResourceState::Ready);
    if !has_ready_shader {
        return Ok(());
    }

    let preview_variants = [ShaderIdePreviewVariant::default_forward()];
    write_shader_ide_env_for_project(project, None, &preview_variants)
        .map(|_| ())
        .map_err(|error| {
            AssetImportError::ShaderValidation(format!("refresh shader IDE environment: {error}"))
        })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use zircon_runtime::asset::project::{
        AssetMetaDocument, AssetSourceUnit, ProjectManifest, ProjectPaths,
    };
    use zircon_runtime::asset::{AssetKind, AssetUri, AssetUuid};
    use zircon_runtime::plugin::PluginPackageManifest;

    use super::*;

    #[test]
    fn sync_from_project_keeps_error_assets_without_artifacts_in_catalog() {
        let root = unique_temp_project_root("sync_error_asset_without_artifact");
        let paths = ProjectPaths::from_root(&root).unwrap();
        paths
            .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
            .unwrap();
        ProjectManifest::new(
            "BrokenAssetProject",
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();
        let material_path = paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("materials")
            .join("broken.material.toml");
        fs::create_dir_all(material_path.parent().unwrap()).unwrap();
        fs::write(&material_path, "not valid toml = [").unwrap();

        let mut project = ProjectManager::open(&root).unwrap();
        let records = project.scan_and_import().unwrap();
        assert!(records.iter().any(
            |record| record.state == ResourceState::Error && record.artifact_locator.is_none()
        ));

        let manager = DefaultEditorAssetManager::new();
        manager.sync_from_project(project).unwrap();
        let catalog = manager.catalog_snapshot_record();
        let broken = catalog
            .assets
            .iter()
            .find(|asset| asset.locator == "res://materials/broken.material.toml")
            .expect("broken material remains visible in editor catalog");
        assert!(!broken.diagnostics.is_empty());
        assert!(broken.direct_reference_uuids.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn sync_from_project_exposes_zmeta_package_and_compound_shader_details() {
        let root = unique_temp_project_root("sync_zmeta_compound_shader");
        let package_root = unique_temp_project_root("sync_zmeta_package");
        let paths = ProjectPaths::from_root(&root).unwrap();
        paths
            .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
            .unwrap();
        ProjectManifest::new(
            "ZMetaEditorProject",
            AssetUri::parse("res://shaders/unlit_shader").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();

        let shader_uri = AssetUri::parse("res://shaders/unlit_shader").unwrap();
        let shader_meta_path = paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("unlit_shader.zmeta");
        let mut shader_meta =
            AssetMetaDocument::new(AssetUuid::new(), shader_uri.clone(), AssetKind::Shader);
        shader_meta.unit = AssetSourceUnit::Compound;
        shader_meta.save(&shader_meta_path).unwrap();

        let shader_dir = paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("unlit_shader");
        fs::create_dir_all(&shader_dir).unwrap();
        fs::write(
            shader_dir.join("unlit.zshader"),
            r#"
kind = "surface"
version = 2
shading_model = "unlit"
wgsl_files = ["unlit.wgsl"]
"#,
        )
        .unwrap();
        fs::write(
            shader_dir.join("unlit.wgsl"),
            r#"
fn zr_material_surface(input: ZrSurfaceInput) -> ZrSurfaceOutput {
    var surface = zr_surface_default(input);
    surface.base_color = vec4f(1.0, 1.0, 1.0, 1.0);
    return surface;
}
"#,
        )
        .unwrap();

        let package_asset_path = package_root.join("assets").join("nav").join("agent.json");
        fs::create_dir_all(package_asset_path.parent().unwrap()).unwrap();
        fs::write(&package_asset_path, r#"{ "agent": true }"#).unwrap();
        let package_manifest = PluginPackageManifest::new("navigation", "Navigation")
            .with_package_identity("com", "zircon", "navigation");

        let mut project = ProjectManager::open(&root).unwrap();
        project
            .register_package_asset_roots(
                package_manifest.package_id(),
                package_manifest.asset_roots_or_default(),
                &package_root,
            )
            .unwrap();
        project.scan_and_import().unwrap();

        let manager = DefaultEditorAssetManager::new();
        manager.sync_from_project(project).unwrap();

        let catalog = manager.catalog_snapshot_record();
        assert!(catalog
            .folders
            .iter()
            .any(|folder| folder.folder_id == "package://com.zircon.navigation"));
        let shader = catalog
            .assets
            .iter()
            .find(|asset| asset.locator == "res://shaders/unlit_shader")
            .expect("compound shader is visible in editor catalog");
        assert!(
            shader.diagnostics.is_empty(),
            "compound shader fixture must import before editor detail projection: {:?}",
            shader.diagnostics
        );
        let details = manager
            .asset_details_record(&shader.uuid)
            .expect("shader details");
        assert_eq!(details.unit, AssetSourceUnit::Compound);
        assert!(details.package_id.is_none());
        assert!(details
            .included_files
            .contains(&"res://shaders/unlit_shader/unlit.zshader".to_string()));
        assert!(details
            .included_files
            .contains(&"res://shaders/unlit_shader/unlit.wgsl".to_string()));
        assert!(
            details
                .subassets
                .iter()
                .any(|subasset| subasset.locator.ends_with("#zshader:unlit.zshader")),
            "zshader subasset should be projected from .zmeta entries: {:?}",
            details.subassets
        );
        assert!(details
            .subassets
            .iter()
            .any(|subasset| subasset.locator.ends_with("#wgsl:unlit.wgsl")));

        let package_asset = catalog
            .assets
            .iter()
            .find(|asset| asset.locator == "package://com.zircon.navigation/nav/agent.json")
            .expect("package asset is visible in editor catalog");
        let package_details = manager
            .asset_details_record(&package_asset.uuid)
            .expect("package details");
        assert_eq!(
            package_details.package_id.as_deref(),
            Some("com.zircon.navigation")
        );
        assert_eq!(package_details.unit, AssetSourceUnit::Single);

        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(package_root);
    }

    #[test]
    fn sync_from_project_refreshes_shader_ide_environment_after_import() {
        let root = unique_temp_project_root("sync_shader_ide_env");
        let paths = ProjectPaths::from_root(&root).unwrap();
        paths
            .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
            .unwrap();
        ProjectManifest::new(
            "Shader Ide Sandbox",
            AssetUri::parse("res://shaders/hero").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();
        write_shader_ide_surface_package(&paths);

        let mut project = ProjectManager::open(&root).unwrap();
        project.scan_and_import().unwrap();

        let manager = DefaultEditorAssetManager::new();
        manager.sync_from_project(project).unwrap();

        let shader_uri = AssetUri::parse("res://shaders/hero").unwrap();
        let ide_root = ProjectPaths::from_root(&root)
            .unwrap()
            .cache_root()
            .join(zircon_runtime::core::framework::render::SHADER_IDE_ENV_CACHE_DIR);
        let module_map_path =
            ide_root.join(zircon_runtime::core::framework::render::SHADER_IDE_MODULE_MAP_FILE);
        let preview_path = ide_root.join(
            zircon_runtime::core::framework::render::shader_ide_preview_relative_path(
                &shader_uri,
                zircon_runtime::core::framework::render::SHADER_IDE_PREVIEW_DEFAULT_VARIANT,
            ),
        );
        let segment_path = ide_root.join(
            zircon_runtime::core::framework::render::shader_ide_preview_segments_relative_path(
                &shader_uri,
                zircon_runtime::core::framework::render::SHADER_IDE_PREVIEW_DEFAULT_VARIANT,
            ),
        );

        let module_map = fs::read_to_string(module_map_path).unwrap();
        assert!(module_map.contains("shader_ide_sandbox::hero"));
        assert!(module_map.contains("generated/res_shaders_hero.material.wgsl"));
        assert!(fs::read_to_string(preview_path)
            .unwrap()
            .contains("fn zr_material_surface"));
        assert!(fs::read_to_string(segment_path)
            .unwrap()
            .contains("generated_material"));

        let _ = fs::remove_dir_all(root);
    }

    fn write_shader_ide_surface_package(paths: &ProjectPaths) {
        let shader_uri = AssetUri::parse("res://shaders/hero").unwrap();
        let shader_meta_path = paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("hero.zmeta");
        let mut shader_meta =
            AssetMetaDocument::new(AssetUuid::new(), shader_uri, AssetKind::Shader);
        shader_meta.unit = AssetSourceUnit::Compound;
        fs::create_dir_all(shader_meta_path.parent().unwrap()).unwrap();
        shader_meta.save(&shader_meta_path).unwrap();

        let shader_dir = paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("hero");
        fs::create_dir_all(&shader_dir).unwrap();
        fs::write(
            shader_dir.join("hero.zshader"),
            r#"
kind = "surface"
version = 2
shading_model = "standard_pbr"
wgsl_files = ["hero.wgsl"]

[[properties]]
name = "base_color"
kind = "vec4"
default = [0.8, 0.4, 0.2, 1.0]
"#,
        )
        .unwrap();
        fs::write(
            shader_dir.join("hero.wgsl"),
            r#"
#include <self::material>

fn zr_material_surface(input: ZrSurfaceInput) -> ZrSurfaceOutput {
    var surface = zr_surface_default(input);
    surface.base_color = zr_mat_base_color();
    return surface;
}
"#,
        )
        .unwrap();
    }

    fn unique_temp_project_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("zircon_editor_{label}_{nanos}"))
    }
}
