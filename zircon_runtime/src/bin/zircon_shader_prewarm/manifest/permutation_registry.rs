use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use zircon_runtime::core::framework::render::{
    GeometrySourceId, ShadingModelId, GEOMETRY_SOURCE_PLUGIN_ID_START,
    SHADING_MODEL_PLUGIN_ID_START,
};

use crate::args::{normalized_custom_geometry_source_token, normalized_custom_shading_model_token};

pub(crate) const SHADER_PERMUTATION_REGISTRY_FILE: &str = "shader_permutation_registry.json";

#[derive(Clone, Debug, Default)]
pub(crate) struct ShaderPrewarmPermutationRegistryOverlay {
    pub(crate) geometry_source_ids: BTreeMap<String, GeometrySourceId>,
    pub(crate) shading_model_ids: BTreeMap<String, ShadingModelId>,
}

impl ShaderPrewarmPermutationRegistryOverlay {
    pub(crate) fn read(path: &Path) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(|error| {
            format!(
                "failed to read shader prewarm permutation registry {}: {error}",
                path.display()
            )
        })?;
        let document = serde_json::from_slice::<ShaderPrewarmPermutationRegistryDocument>(&bytes)
            .map_err(|error| {
            format!(
                "failed to parse shader prewarm permutation registry {}: {error}",
                path.display()
            )
        })?;
        Self::from_document(document, path)
    }

    pub(crate) fn merge_into(
        self,
        geometry_sources: &mut Vec<GeometrySourceId>,
        geometry_source_ids: &mut BTreeMap<String, GeometrySourceId>,
        shading_model_ids: &mut BTreeMap<String, ShadingModelId>,
    ) -> Result<(), String> {
        for (token, id) in self.geometry_source_ids {
            merge_geometry_source_id(geometry_sources, geometry_source_ids, token, id)?;
        }
        for (token, id) in self.shading_model_ids {
            merge_shading_model_id(shading_model_ids, token, id)?;
        }
        Ok(())
    }

    fn from_document(
        document: ShaderPrewarmPermutationRegistryDocument,
        path: &Path,
    ) -> Result<Self, String> {
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
        for entry in document.shading_model_ids {
            let token = normalized_custom_shading_model_token(&entry.token)?;
            let id = shading_model_id_from_registry(entry.id, path)?;
            merge_shading_model_id(&mut overlay.shading_model_ids, token, id)?;
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
    #[serde(default, alias = "shading_models")]
    shading_model_ids: Vec<ShaderPrewarmShadingModelIdRecord>,
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

fn geometry_source_id_from_registry(id: u8, path: &Path) -> Result<GeometrySourceId, String> {
    if id < GEOMETRY_SOURCE_PLUGIN_ID_START {
        return Err(format!(
            "shader prewarm permutation registry {} assigns geometry source id {id}; plugin geometry source ids must be >= {GEOMETRY_SOURCE_PLUGIN_ID_START}",
            path.display()
        ));
    }
    Ok(GeometrySourceId::new(id))
}

fn shading_model_id_from_registry(id: u8, path: &Path) -> Result<ShadingModelId, String> {
    if id < SHADING_MODEL_PLUGIN_ID_START {
        return Err(format!(
            "shader prewarm permutation registry {} assigns shading model id {id}; plugin shading model ids must be >= {SHADING_MODEL_PLUGIN_ID_START}",
            path.display()
        ));
    }
    Ok(ShadingModelId::new(id))
}

fn merge_geometry_source_id(
    geometry_sources: &mut Vec<GeometrySourceId>,
    geometry_source_ids: &mut BTreeMap<String, GeometrySourceId>,
    token: String,
    id: GeometrySourceId,
) -> Result<(), String> {
    if let Some(existing_id) = geometry_source_ids.get(&token) {
        if *existing_id != id {
            return Err(format!(
                "custom geometry source {token} was assigned both id {} and id {}",
                existing_id.value(),
                id.value()
            ));
        }
        return Ok(());
    }
    if let Some(existing_token) = geometry_source_ids
        .iter()
        .find_map(|(existing_token, existing_id)| (*existing_id == id).then_some(existing_token))
    {
        return Err(format!(
            "custom geometry source id {} is already assigned to {existing_token} and cannot be reused by {token}",
            id.value()
        ));
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
) -> Result<(), String> {
    if let Some(existing_id) = shading_model_ids.get(&token) {
        if *existing_id != id {
            return Err(format!(
                "custom shading model {token} was assigned both id {existing_id} and id {id}"
            ));
        }
        return Ok(());
    }
    if let Some(existing_token) = shading_model_ids
        .iter()
        .find_map(|(existing_token, existing_id)| (*existing_id == id).then_some(existing_token))
    {
        return Err(format!(
            "custom shading model id {id} is already assigned to {existing_token} and cannot be reused by {token}"
        ));
    }
    shading_model_ids.insert(token, id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use zircon_runtime::core::framework::render::{
        GEOMETRY_SOURCE_PLUGIN_ID_START, SHADING_MODEL_PLUGIN_ID_START,
    };

    use super::*;

    #[test]
    fn shader_prewarm_permutation_registry_merges_custom_geometry_and_shading_ids() {
        let registry_path = unique_registry_path();
        fs::write(
            &registry_path,
            format!(
                r#"{{
                    "geometry_source_ids": [{{ "token": "gpu-driven", "id": {} }}],
                    "shading_model_ids": [{{ "token": "custom:toon", "id": {} }}]
                }}"#,
                GEOMETRY_SOURCE_PLUGIN_ID_START, SHADING_MODEL_PLUGIN_ID_START
            ),
        )
        .unwrap();

        let mut geometry_sources = Vec::new();
        let mut geometry_source_ids = BTreeMap::new();
        let mut shading_model_ids = BTreeMap::new();
        ShaderPrewarmPermutationRegistryOverlay::read(&registry_path)
            .unwrap()
            .merge_into(
                &mut geometry_sources,
                &mut geometry_source_ids,
                &mut shading_model_ids,
            )
            .unwrap();

        assert_eq!(
            geometry_sources,
            vec![GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START)]
        );
        assert_eq!(
            geometry_source_ids.get("custom:gpu-driven").copied(),
            Some(GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START))
        );
        assert_eq!(
            shading_model_ids.get("custom:toon").copied(),
            Some(ShadingModelId::new(SHADING_MODEL_PLUGIN_ID_START))
        );

        fs::remove_file(registry_path).ok();
    }

    #[test]
    fn shader_prewarm_permutation_registry_discovers_asset_root_registry() {
        let root = unique_registry_root();
        fs::create_dir_all(&root).unwrap();
        let registry_path = root.join(SHADER_PERMUTATION_REGISTRY_FILE);
        fs::write(&registry_path, r#"{ "geometry_source_ids": [] }"#).unwrap();

        assert_eq!(
            shader_permutation_registry_paths(&[], &[root.clone()]),
            vec![registry_path.clone()]
        );

        fs::remove_file(registry_path).ok();
        fs::remove_dir(root).ok();
    }

    fn unique_registry_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "zircon_shader_prewarm_registry_{}_{}",
            std::process::id(),
            nanos
        ))
    }

    fn unique_registry_path() -> PathBuf {
        let root = unique_registry_root();
        fs::create_dir_all(&root).unwrap();
        root.join(SHADER_PERMUTATION_REGISTRY_FILE)
    }
}
