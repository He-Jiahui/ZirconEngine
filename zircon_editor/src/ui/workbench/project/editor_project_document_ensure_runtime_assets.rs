use std::ffi::OsStr;

use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::scene::world::SceneProjectError;

use super::constants::{DEFAULT_CUBE_OBJ, DEFAULT_PBR_WGSL, DEFAULT_SCENE_URI};
use super::editor_project_document::EditorProjectDocument;
use super::project_root_path::project_root_path;
use super::runtime_asset_helpers::{default_material_asset, parse_asset_uri, write_if_missing};

impl EditorProjectDocument {
    pub fn ensure_runtime_assets(
        root: impl AsRef<std::path::Path>,
    ) -> Result<(), SceneProjectError> {
        let root = project_root_path(root)?;
        let paths = ProjectPaths::from_root(&root)?;
        paths.ensure_layout()?;

        if !paths.manifest_path().exists() {
            let default_scene = parse_asset_uri(DEFAULT_SCENE_URI)?;
            let project_name = root
                .file_name()
                .and_then(OsStr::to_str)
                .filter(|value| !value.is_empty())
                .unwrap_or("ZirconProject");
            ProjectManifest::new(project_name, default_scene, 1).save(paths.manifest_path())?;
        }

        write_if_missing(
            paths.assets_root().join("shaders").join("pbr.wgsl"),
            DEFAULT_PBR_WGSL,
        )?;
        write_if_missing(
            paths
                .assets_root()
                .join("materials")
                .join("default.material.toml"),
            default_material_asset()?
                .to_toml_string()
                .map_err(|error| super::runtime_asset_helpers::invalid_data(error.to_string()))?,
        )?;
        write_if_missing(
            paths.assets_root().join("models").join("cube.obj"),
            DEFAULT_CUBE_OBJ,
        )?;

        Ok(())
    }
}
