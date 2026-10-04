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
    let mut shader_modules = BTreeMap::new();
    ShaderPrewarmPermutationRegistryOverlay::read(&registry_path)
        .unwrap()
        .merge_into(
            &mut geometry_sources,
            &mut geometry_source_ids,
            &mut geometry_source_descriptors,
            &mut shading_model_ids,
            &mut shading_model_descriptors,
            &mut shader_modules,
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
    let mut shader_modules = BTreeMap::new();
    ShaderPrewarmPermutationRegistryOverlay::read(&registry_path)
        .unwrap()
        .merge_into(
            &mut geometry_sources,
            &mut geometry_source_ids,
            &mut geometry_source_descriptors,
            &mut shading_model_ids,
            &mut shading_model_descriptors,
            &mut shader_modules,
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
    let mut shader_modules = BTreeMap::new();
    ShaderPrewarmPermutationRegistryOverlay::read(&registry_path)
        .unwrap()
        .merge_into(
            &mut geometry_sources,
            &mut geometry_source_ids,
            &mut geometry_source_descriptors,
            &mut shading_model_ids,
            &mut shading_model_descriptors,
            &mut shader_modules,
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
fn shader_prewarm_permutation_registry_merges_shader_modules() {
    let registry_path = unique_registry_path();
    fs::write(
        &registry_path,
        r#"{
                "shader_modules": [
                    {
                        "import_path": "custom::toon::noise",
                        "content_hash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    }
                ]
            }"#,
    )
    .unwrap();

    let mut geometry_sources = Vec::new();
    let mut geometry_source_ids = BTreeMap::new();
    let mut geometry_source_descriptors = BTreeMap::new();
    let mut shading_model_ids = BTreeMap::new();
    let mut shading_model_descriptors = BTreeMap::new();
    let mut shader_modules = BTreeMap::new();
    ShaderPrewarmPermutationRegistryOverlay::read(&registry_path)
        .unwrap()
        .merge_into(
            &mut geometry_sources,
            &mut geometry_source_ids,
            &mut geometry_source_descriptors,
            &mut shading_model_ids,
            &mut shading_model_descriptors,
            &mut shader_modules,
        )
        .unwrap();

    assert_eq!(
        shader_modules
            .get("custom::toon::noise")
            .map(String::as_str),
        Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
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
