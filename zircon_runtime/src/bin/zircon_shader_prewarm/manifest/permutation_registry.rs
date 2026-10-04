use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use zircon_runtime::core::framework::render::{
    GeometrySourceDescriptor, GeometrySourceId, ShadingModelDescriptor, ShadingModelId,
    GEOMETRY_SOURCE_PLUGIN_ID_START, SHADING_MODEL_PLUGIN_ID_START,
};

use crate::args::{normalized_custom_geometry_source_token, normalized_custom_shading_model_token};
use crate::error::{ShaderPrewarmPermutationRegistryError, ShaderPrewarmPermutationRegistryResult};

pub(crate) const SHADER_PERMUTATION_REGISTRY_FILE: &str = "shader_permutation_registry.json";

#[derive(Clone, Debug, Default)]
pub(crate) struct ShaderPrewarmPermutationRegistryOverlay {
    pub(crate) geometry_source_ids: BTreeMap<String, GeometrySourceId>,
    pub(crate) geometry_source_descriptors: BTreeMap<GeometrySourceId, GeometrySourceDescriptor>,
    pub(crate) shading_model_ids: BTreeMap<String, ShadingModelId>,
    pub(crate) shading_model_descriptors: BTreeMap<ShadingModelId, ShadingModelDescriptor>,
    pub(crate) shader_modules: BTreeMap<String, String>,
}

impl ShaderPrewarmPermutationRegistryOverlay {
    pub(crate) fn read(path: &Path) -> ShaderPrewarmPermutationRegistryResult<Self> {
        let bytes =
            fs::read(path).map_err(|source| ShaderPrewarmPermutationRegistryError::Read {
                path: path.to_path_buf(),
                source,
            })?;
        let document =
            match serde_json::from_slice::<ShaderPrewarmPermutationRegistryDocument>(&bytes) {
                Ok(document) => document,
                Err(source) => {
                    return Err(ShaderPrewarmPermutationRegistryError::Parse {
                        path: path.to_path_buf(),
                        source,
                    });
                }
            };
        Self::from_document(document, path)
    }

    pub(crate) fn merge_into(
        self,
        geometry_sources: &mut Vec<GeometrySourceId>,
        geometry_source_ids: &mut BTreeMap<String, GeometrySourceId>,
        geometry_source_descriptors: &mut BTreeMap<GeometrySourceId, GeometrySourceDescriptor>,
        shading_model_ids: &mut BTreeMap<String, ShadingModelId>,
        shading_model_descriptors: &mut BTreeMap<ShadingModelId, ShadingModelDescriptor>,
        shader_modules: &mut BTreeMap<String, String>,
    ) -> ShaderPrewarmPermutationRegistryResult<()> {
        let Self {
            geometry_source_ids: overlay_geometry_source_ids,
            geometry_source_descriptors: overlay_geometry_source_descriptors,
            shading_model_ids: overlay_shading_model_ids,
            shading_model_descriptors: overlay_shading_model_descriptors,
            shader_modules: overlay_shader_modules,
        } = self;
        for (token, id) in overlay_geometry_source_ids {
            merge_geometry_source_id(geometry_sources, geometry_source_ids, token, id)?;
        }
        for descriptor in overlay_geometry_source_descriptors.into_values() {
            merge_geometry_source_descriptor(geometry_source_descriptors, descriptor)?;
        }
        for (token, id) in overlay_shading_model_ids {
            merge_shading_model_id(shading_model_ids, token, id)?;
        }
        for descriptor in overlay_shading_model_descriptors.into_values() {
            merge_shading_model_descriptor(shading_model_descriptors, descriptor)?;
        }
        for (import_path, content_hash) in overlay_shader_modules {
            merge_shader_module(shader_modules, import_path, content_hash)?;
        }
        Ok(())
    }

    fn from_document(
        document: ShaderPrewarmPermutationRegistryDocument,
        path: &Path,
    ) -> ShaderPrewarmPermutationRegistryResult<Self> {
        let mut overlay = Self::default();
        let mut geometry_sources = Vec::new();
        for entry in document.geometry_source_ids {
            let token = normalized_custom_geometry_source_token(&entry.token)?;
            let id = geometry_source_id_from_registry(entry.id, path)?;
            merge_geometry_source_id(
                &mut geometry_sources,
                &mut overlay.geometry_source_ids,
                token,
                id,
            )?;
        }
        for descriptor in document.geometry_source_descriptors {
            let descriptor = geometry_source_descriptor_from_registry(descriptor, path)?;
            merge_geometry_source_id(
                &mut geometry_sources,
                &mut overlay.geometry_source_ids,
                descriptor.token.clone(),
                descriptor.id,
            )?;
            merge_geometry_source_descriptor(&mut overlay.geometry_source_descriptors, descriptor)?;
        }
        for entry in document.shading_model_ids {
            let token = normalized_custom_shading_model_token(&entry.token)?;
            let id = shading_model_id_from_registry(entry.id, path)?;
            merge_shading_model_id(&mut overlay.shading_model_ids, token, id)?;
        }
        for descriptor in document.shading_model_descriptors {
            let descriptor = shading_model_descriptor_from_registry(descriptor, path)?;
            merge_shading_model_id(
                &mut overlay.shading_model_ids,
                descriptor.token.clone(),
                descriptor.id,
            )?;
            merge_shading_model_descriptor(&mut overlay.shading_model_descriptors, descriptor)?;
        }
        for entry in document.shader_modules {
            merge_shader_module(
                &mut overlay.shader_modules,
                entry.import_path,
                entry.content_hash,
            )?;
        }
        Ok(overlay)
    }
}

pub(crate) fn shader_permutation_registry_paths(
    explicit_paths: &[PathBuf],
    asset_roots: &[PathBuf],
) -> Vec<PathBuf> {
    let mut paths = BTreeSet::new();
    for path in explicit_paths {
        paths.insert(path.clone());
    }
    for asset_root in asset_roots {
        let path = asset_root.join(SHADER_PERMUTATION_REGISTRY_FILE);
        if path.is_file() {
            paths.insert(path);
        }
    }
    paths.into_iter().collect()
}

#[derive(Debug, Default, Deserialize)]
struct ShaderPrewarmPermutationRegistryDocument {
    #[serde(default, alias = "geometry_sources")]
    geometry_source_ids: Vec<ShaderPrewarmGeometrySourceIdRecord>,
    #[serde(default)]
    geometry_source_descriptors: Vec<GeometrySourceDescriptor>,
    #[serde(default, alias = "shading_models")]
    shading_model_ids: Vec<ShaderPrewarmShadingModelIdRecord>,
    #[serde(default)]
    shading_model_descriptors: Vec<ShadingModelDescriptor>,
    #[serde(default)]
    shader_modules: Vec<ShaderPrewarmShaderModuleRecord>,
}

#[derive(Debug, Deserialize)]
struct ShaderPrewarmGeometrySourceIdRecord {
    token: String,
    id: u8,
}

#[derive(Debug, Deserialize)]
struct ShaderPrewarmShadingModelIdRecord {
    token: String,
    id: u8,
}

#[derive(Debug, Deserialize)]
struct ShaderPrewarmShaderModuleRecord {
    import_path: String,
    content_hash: String,
}

fn geometry_source_id_from_registry(
    id: u8,
    path: &Path,
) -> ShaderPrewarmPermutationRegistryResult<GeometrySourceId> {
    if id < GEOMETRY_SOURCE_PLUGIN_ID_START {
        return Err(
            ShaderPrewarmPermutationRegistryError::GeometrySourceIdBelowPluginRange {
                path: path.to_path_buf(),
                id,
                minimum: GEOMETRY_SOURCE_PLUGIN_ID_START,
            },
        );
    }
    Ok(GeometrySourceId::new(id))
}

fn shading_model_id_from_registry(
    id: u8,
    path: &Path,
) -> ShaderPrewarmPermutationRegistryResult<ShadingModelId> {
    if id < SHADING_MODEL_PLUGIN_ID_START {
        return Err(
            ShaderPrewarmPermutationRegistryError::ShadingModelIdBelowPluginRange {
                path: path.to_path_buf(),
                id,
                minimum: SHADING_MODEL_PLUGIN_ID_START,
            },
        );
    }
    Ok(ShadingModelId::new(id))
}

fn geometry_source_descriptor_from_registry(
    mut descriptor: GeometrySourceDescriptor,
    path: &Path,
) -> ShaderPrewarmPermutationRegistryResult<GeometrySourceDescriptor> {
    let token = normalized_custom_geometry_source_token(&descriptor.token)?;
    let id = geometry_source_id_from_registry(descriptor.id.value(), path)?;
    descriptor.token = token;
    descriptor.id = id;
    Ok(descriptor)
}

fn shading_model_descriptor_from_registry(
    mut descriptor: ShadingModelDescriptor,
    path: &Path,
) -> ShaderPrewarmPermutationRegistryResult<ShadingModelDescriptor> {
    let token = normalized_custom_shading_model_token(&descriptor.token)?;
    let id = shading_model_id_from_registry(descriptor.id.value(), path)?;
    descriptor.token = token;
    descriptor.id = id;
    Ok(descriptor)
}

fn merge_geometry_source_id(
    geometry_sources: &mut Vec<GeometrySourceId>,
    geometry_source_ids: &mut BTreeMap<String, GeometrySourceId>,
    token: String,
    id: GeometrySourceId,
) -> ShaderPrewarmPermutationRegistryResult<()> {
    if let Some(existing_id) = geometry_source_ids.get(&token) {
        if *existing_id != id {
            return Err(
                ShaderPrewarmPermutationRegistryError::DuplicateGeometrySourceToken {
                    token,
                    existing_id: existing_id.value(),
                    new_id: id.value(),
                },
            );
        }
        return Ok(());
    }
    if let Some(existing_token) = geometry_source_ids
        .iter()
        .find_map(|(existing_token, existing_id)| (*existing_id == id).then_some(existing_token))
    {
        return Err(
            ShaderPrewarmPermutationRegistryError::DuplicateGeometrySourceId {
                id: id.value(),
                existing_token: existing_token.clone(),
                new_token: token,
            },
        );
    }
    if !geometry_sources.contains(&id) {
        geometry_sources.push(id);
    }
    geometry_source_ids.insert(token, id);
    Ok(())
}

fn merge_shading_model_id(
    shading_model_ids: &mut BTreeMap<String, ShadingModelId>,
    token: String,
    id: ShadingModelId,
) -> ShaderPrewarmPermutationRegistryResult<()> {
    if let Some(existing_id) = shading_model_ids.get(&token) {
        if *existing_id != id {
            return Err(
                ShaderPrewarmPermutationRegistryError::DuplicateShadingModelToken {
                    token,
                    existing_id: existing_id.value(),
                    new_id: id.value(),
                },
            );
        }
        return Ok(());
    }
    if let Some(existing_token) = shading_model_ids
        .iter()
        .find_map(|(existing_token, existing_id)| (*existing_id == id).then_some(existing_token))
    {
        return Err(
            ShaderPrewarmPermutationRegistryError::DuplicateShadingModelId {
                id: id.value(),
                existing_token: existing_token.clone(),
                new_token: token,
            },
        );
    }
    shading_model_ids.insert(token, id);
    Ok(())
}

fn merge_geometry_source_descriptor(
    geometry_source_descriptors: &mut BTreeMap<GeometrySourceId, GeometrySourceDescriptor>,
    descriptor: GeometrySourceDescriptor,
) -> ShaderPrewarmPermutationRegistryResult<()> {
    if let Some(existing_descriptor) = geometry_source_descriptors.get(&descriptor.id) {
        if existing_descriptor != &descriptor {
            return Err(
                ShaderPrewarmPermutationRegistryError::IncompatibleGeometrySourceDescriptor {
                    id: descriptor.id.value(),
                },
            );
        }
        return Ok(());
    }
    geometry_source_descriptors.insert(descriptor.id, descriptor);
    Ok(())
}

fn merge_shading_model_descriptor(
    shading_model_descriptors: &mut BTreeMap<ShadingModelId, ShadingModelDescriptor>,
    descriptor: ShadingModelDescriptor,
) -> ShaderPrewarmPermutationRegistryResult<()> {
    if let Some(existing_descriptor) = shading_model_descriptors.get(&descriptor.id) {
        if existing_descriptor != &descriptor {
            return Err(
                ShaderPrewarmPermutationRegistryError::IncompatibleShadingModelDescriptor {
                    id: descriptor.id.value(),
                },
            );
        }
        return Ok(());
    }
    shading_model_descriptors.insert(descriptor.id, descriptor);
    Ok(())
}

fn merge_shader_module(
    shader_modules: &mut BTreeMap<String, String>,
    import_path: String,
    content_hash: String,
) -> ShaderPrewarmPermutationRegistryResult<()> {
    if let Some(existing_hash) = shader_modules.get(&import_path) {
        if existing_hash != &content_hash {
            return Err(
                ShaderPrewarmPermutationRegistryError::DuplicateShaderModuleContentHash {
                    import_path,
                    existing_content_hash: existing_hash.clone(),
                    new_content_hash: content_hash,
                },
            );
        }
        return Ok(());
    }
    shader_modules.insert(import_path, content_hash);
    Ok(())
}

#[cfg(test)]
#[path = "tests/permutation_registry.rs"]
mod tests;
