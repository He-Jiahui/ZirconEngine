use std::path::Path;

use super::AssetImporter;
use crate::asset::assets::ImportedAsset;
use crate::asset::{AssetImportError, AssetUri};

impl AssetImporter {
    pub fn import_from_source(
        &self,
        source_path: &Path,
        uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        let lower_name = source_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        if lower_name.ends_with(".material.toml") {
            return self.import_material(source_path);
        }
        if lower_name.ends_with(".physics_material.toml") {
            return self.import_physics_material(source_path);
        }
        if lower_name.ends_with(".scene.toml") {
            return self.import_scene(source_path);
        }
        if lower_name.ends_with(".ui.toml") {
            return self.import_ui_asset(source_path);
        }
        if lower_name.ends_with(".skeleton.zranim")
            || lower_name.ends_with(".clip.zranim")
            || lower_name.ends_with(".sequence.zranim")
            || lower_name.ends_with(".graph.zranim")
            || lower_name.ends_with(".state_machine.zranim")
        {
            return self.import_animation_asset(source_path);
        }

        let extension = source_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        match extension.as_str() {
            "png" | "jpg" | "jpeg" => self.import_texture(source_path, uri),
            "wgsl" => self.import_shader(source_path, uri),
            "obj" => self.import_obj(source_path, uri),
            "gltf" | "glb" => self.import_gltf(source_path, uri),
            "fbx" => Err(AssetImportError::UnsupportedFormat(format!(
                "fbx importer is not implemented for this milestone: {}",
                source_path.display()
            ))),
            other => Err(AssetImportError::UnsupportedFormat(format!(
                "{other} from {}",
                source_path.display()
            ))),
        }
    }
}
