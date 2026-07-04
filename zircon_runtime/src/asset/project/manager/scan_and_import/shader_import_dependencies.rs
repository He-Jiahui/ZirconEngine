use std::collections::{HashMap, HashSet};

use crate::asset::project::ProjectPaths;
use crate::asset::{
    ArtifactStore, AssetId, AssetImportError, AssetUri, ImportedAsset, ShaderAsset,
};
use crate::core::framework::render::{
    is_builtin_shader_module_token, is_generated_shader_module_token,
};
use crate::core::resource::ResourceRecord;

pub(super) fn append_shader_import_dependencies(
    artifact_store: &ArtifactStore,
    paths: &ProjectPaths,
    imported: &[ResourceRecord],
    dependencies_by_id: &mut HashMap<AssetId, Vec<AssetUri>>,
) -> Result<(), AssetImportError> {
    let shaders = imported_shader_assets(artifact_store, paths, imported)?;
    let include_modules = include_module_locators_by_import_path(&shaders);

    for (shader_id, _locator, shader) in shaders {
        let dependencies = dependencies_by_id.entry(shader_id).or_default();
        for import in &shader.imports {
            if import.redirect.is_some() || generated_or_builtin_module(&import.source) {
                continue;
            }
            let Some(module_locator) = include_modules.get(&import.source) else {
                continue;
            };
            if !dependencies.contains(module_locator) {
                dependencies.push(module_locator.clone());
            }
        }
    }

    Ok(())
}

fn imported_shader_assets(
    artifact_store: &ArtifactStore,
    paths: &ProjectPaths,
    imported: &[ResourceRecord],
) -> Result<Vec<(AssetId, AssetUri, ShaderAsset)>, AssetImportError> {
    let mut shaders = Vec::new();
    for record in imported {
        let Some(artifact_uri) = record.artifact_locator.as_ref() else {
            continue;
        };
        if let ImportedAsset::Shader(shader) = artifact_store.read(paths, artifact_uri)? {
            shaders.push((record.id(), record.primary_locator.clone(), shader));
        }
    }
    Ok(shaders)
}

fn include_module_locators_by_import_path(
    shaders: &[(AssetId, AssetUri, ShaderAsset)],
) -> HashMap<String, AssetUri> {
    let mut modules = HashMap::new();
    let mut duplicate_paths = HashSet::new();
    for (_id, locator, shader) in shaders {
        if !shader.kind.is_include() {
            continue;
        }
        let Some(import_path) = shader
            .import_path
            .as_deref()
            .filter(|path| !path.is_empty())
        else {
            continue;
        };
        if duplicate_paths.contains(import_path) {
            continue;
        }
        if modules
            .insert(import_path.to_string(), locator.clone())
            .is_some()
        {
            modules.remove(import_path);
            duplicate_paths.insert(import_path.to_string());
        }
    }
    modules
}

fn generated_or_builtin_module(import_path: &str) -> bool {
    is_builtin_shader_module_token(import_path) || is_generated_shader_module_token(import_path)
}
