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
    ) -> ShaderPrewarmPermutationRegistryResult<()> {
        let Self {
            geometry_source_ids: overlay_geometry_source_ids,
            geometry_source_descriptors: overlay_geometry_source_descriptors,
            shading_model_ids: overlay_shading_model_ids,
            shading_model_descriptors: overlay_shading_model_descriptors,
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

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;
    use std::time::{SystemTime, UNIX_EPOCH};

    use zircon_runtime::core::framework::render::{
        GEOMETRY_SOURCE_PLUGIN_ID_START, SHADING_MODEL_PLUGIN_ID_START,
    };

    use super::*;

    #[test]
    fn shader_prewarm_permutation_registry_read_reports_typed_read_error() {
        let registry_path = std::env::temp_dir().join(format!(
            "zircon_shader_prewarm_missing_permutation_registry_{}_not_found.json",
            std::process::id()
        ));
        let _ = fs::remove_file(&registry_path);

        let error = ShaderPrewarmPermutationRegistryOverlay::read(&registry_path).unwrap_err();

        match error {
            ShaderPrewarmPermutationRegistryError::Read { path, source } => {
                assert_eq!(path, registry_path);
                assert_eq!(source.kind(), ErrorKind::NotFound);
            }
            other => panic!("expected typed permutation registry read error, got {other:?}"),
        }
    }

    #[test]
    fn shader_prewarm_permutation_registry_read_reports_typed_parse_error() {
        let registry_path = unique_registry_path();
        fs::write(&registry_path, "{not valid json").unwrap();

        let error = ShaderPrewarmPermutationRegistryOverlay::read(&registry_path).unwrap_err();

        match error {
            ShaderPrewarmPermutationRegistryError::Parse { path, source } => {
                assert_eq!(path, registry_path);
                assert!(source.is_syntax());
            }
            other => panic!("expected typed permutation registry parse error, got {other:?}"),
        }

        let _ = fs::remove_file(registry_path);
    }

    #[test]
    fn shader_prewarm_permutation_registry_reports_typed_geometry_id_range_error() {
        let registry_path = unique_registry_path();
        fs::write(
            &registry_path,
            r#"{ "geometry_source_ids": [{ "token": "custom:bad", "id": 1 }] }"#,
        )
        .unwrap();

        let error = ShaderPrewarmPermutationRegistryOverlay::read(&registry_path).unwrap_err();

        match error {
            ShaderPrewarmPermutationRegistryError::GeometrySourceIdBelowPluginRange {
                path,
                id,
                minimum,
            } => {
                assert_eq!(path, registry_path);
                assert_eq!(id, 1);
                assert_eq!(minimum, GEOMETRY_SOURCE_PLUGIN_ID_START);
            }
            other => panic!("expected typed permutation registry id range error, got {other:?}"),
        }

        let _ = fs::remove_file(registry_path);
    }

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
        let mut geometry_source_descriptors = BTreeMap::new();
        let mut shading_model_ids = BTreeMap::new();
        let mut shading_model_descriptors = BTreeMap::new();
        ShaderPrewarmPermutationRegistryOverlay::read(&registry_path)
            .unwrap()
            .merge_into(
                &mut geometry_sources,
                &mut geometry_source_ids,
                &mut geometry_source_descriptors,
                &mut shading_model_ids,
                &mut shading_model_descriptors,
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
    fn shader_prewarm_permutation_registry_merges_custom_geometry_descriptors() {
        let registry_path = unique_registry_path();
        fs::write(
            &registry_path,
            format!(
                r#"{{
                    "geometry_source_descriptors": [{{
                        "id": {},
                        "token": "custom:virtual_geometry",
                        "wgsl_include": "zr_geometry_virtual_geometry.wgsl",
                        "vertex_attributes": ["position", "normal", "tangent", "uv0"],
                        "required_bindings": [
                            {{ "kind": "virtual_geometry_pages", "slot_token": "virtual_geometry.pages" }},
                            {{ "kind": "virtual_geometry_clusters", "slot_token": "virtual_geometry.clusters" }}
                        ],
                        "shader_defines": [
                            {{ "kind": "bool", "name": "ZR_GEOMETRY_SOURCE_VIRTUAL_GEOMETRY", "value": true }}
                        ]
                    }}]
                }}"#,
                GEOMETRY_SOURCE_PLUGIN_ID_START
            ),
        )
        .unwrap();

        let mut geometry_sources = Vec::new();
        let mut geometry_source_ids = BTreeMap::new();
        let mut geometry_source_descriptors = BTreeMap::new();
        let mut shading_model_ids = BTreeMap::new();
        let mut shading_model_descriptors = BTreeMap::new();
        ShaderPrewarmPermutationRegistryOverlay::read(&registry_path)
            .unwrap()
            .merge_into(
                &mut geometry_sources,
                &mut geometry_source_ids,
                &mut geometry_source_descriptors,
                &mut shading_model_ids,
                &mut shading_model_descriptors,
            )
            .unwrap();

        let custom_id = GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START);
        assert_eq!(geometry_sources, vec![custom_id]);
        assert_eq!(
            geometry_source_ids.get("custom:virtual_geometry").copied(),
            Some(custom_id)
        );
        let descriptor = geometry_source_descriptors
            .get(&custom_id)
            .expect("custom geometry descriptor");
        assert_eq!(descriptor.id, custom_id);
        assert_eq!(descriptor.token, "custom:virtual_geometry");
        assert_eq!(descriptor.wgsl_include, "zr_geometry_virtual_geometry.wgsl");

        fs::remove_file(registry_path).ok();
    }

    #[test]
    fn shader_prewarm_permutation_registry_merges_custom_shading_model_descriptors() {
        let registry_path = unique_registry_path();
        fs::write(
            &registry_path,
            format!(
                r#"{{
                    "shading_model_descriptors": [{{
                        "id": {},
                        "token": "toon",
                        "forward_include": "zr_shading_toon_forward.wgsl",
                        "gbuffer_encode_include": "zr_shading_toon_gbuffer.wgsl",
                        "deferred_include": "zr_shading_toon_deferred.wgsl",
                        "required_channels": 7
                    }}]
                }}"#,
                SHADING_MODEL_PLUGIN_ID_START
            ),
        )
        .unwrap();

        let mut geometry_sources = Vec::new();
        let mut geometry_source_ids = BTreeMap::new();
        let mut geometry_source_descriptors = BTreeMap::new();
        let mut shading_model_ids = BTreeMap::new();
        let mut shading_model_descriptors = BTreeMap::new();
        ShaderPrewarmPermutationRegistryOverlay::read(&registry_path)
            .unwrap()
            .merge_into(
                &mut geometry_sources,
                &mut geometry_source_ids,
                &mut geometry_source_descriptors,
                &mut shading_model_ids,
                &mut shading_model_descriptors,
            )
            .unwrap();

        let custom_id = ShadingModelId::new(SHADING_MODEL_PLUGIN_ID_START);
        assert_eq!(
            shading_model_ids.get("custom:toon").copied(),
            Some(custom_id)
        );
        let descriptor = shading_model_descriptors
            .get(&custom_id)
            .expect("custom shading model descriptor");
        assert_eq!(descriptor.id, custom_id);
        assert_eq!(descriptor.token, "custom:toon");
        assert_eq!(descriptor.forward_include, "zr_shading_toon_forward.wgsl");
        assert_eq!(
            descriptor.gbuffer_encode_include,
            "zr_shading_toon_gbuffer.wgsl"
        );
        assert_eq!(descriptor.deferred_include, "zr_shading_toon_deferred.wgsl");

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
