use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{
    AlphaMode, Asset, AssetImportContext, AssetImportError, AssetImportOutcome,
    AssetImporterDescriptor, AssetKind, AssetLoadState, AssetLoadStates, AssetManagementRecordSets,
    AssetManager, AssetMetaDocument, AssetReference, AssetSourceUnit, AssetUri, AssetUuid,
    DependencyLoadState, FunctionAssetImporter, ImportedAsset, MaterialAsset,
    MaterialAssetManagementRecordSet, MaterialTextureSlotValue, MeshAsset,
    MeshAssetManagementRecordSet, ModelAsset, ModelAssetManagementRecordSet, ProjectAssetManager,
    ProjectManager, ProjectManifest, ProjectPaths, RecursiveDependencyLoadState,
    SceneAssetManagementRecordSet, SceneEntityManagementRecordSet, ShaderAsset,
    ShaderAssetManagementRecordSet, TextureAsset, TextureUploadSupport,
};
use crate::core::framework::render::RenderMaterialManagementRecordSet;
use crate::core::resource::ResourceState;

#[test]
fn project_manager_imports_minimal_gltf_material_shader_mesh_sample() {
    let root = unique_temp_project_root("project_manager_minimal_asset_flow");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new("MinimalAssetFlow", uri("res://models/hero.gltf#Scene0"), 1)
        .save(paths.manifest_path())
        .unwrap();

    write_minimal_textured_gltf(paths.assets_root().join("models").join("hero.gltf"));
    write_sample_shader_package(&paths);
    write_default_pbr_shader_package(&paths);
    write_sample_material(&paths);
    write_bc1_texture(
        paths
            .assets_root()
            .join("textures")
            .join("hero_albedo_bc1.dds"),
    );

    let mut manager = project_manager_with_sample_importers(&root);
    let imported = manager.scan_and_import().unwrap();

    assert!(imported.len() >= 12);
    assert_ready_record(&manager, "res://models/hero.gltf", AssetKind::Model);
    assert_ready_record(
        &manager,
        "res://models/hero.gltf#Texture0",
        AssetKind::Texture,
    );
    assert_ready_record(
        &manager,
        "res://models/hero.gltf#Material0",
        AssetKind::Material,
    );
    assert_ready_record(&manager, "res://models/hero.gltf#Mesh0", AssetKind::Model);
    assert_ready_record(
        &manager,
        "res://models/hero.gltf#Mesh0/Primitive0",
        AssetKind::Mesh,
    );
    assert_ready_record(&manager, "res://models/hero.gltf#Node0", AssetKind::Scene);
    assert_ready_record(&manager, "res://models/hero.gltf#Scene0", AssetKind::Scene);
    assert_ready_record(&manager, "res://shaders/lit_sample", AssetKind::Shader);
    assert_ready_record(
        &manager,
        "res://shaders/default_pbr.zshader",
        AssetKind::Shader,
    );
    assert_ready_record(
        &manager,
        "res://materials/hero_surface.zmaterial",
        AssetKind::Material,
    );
    assert_ready_record(
        &manager,
        "res://textures/hero_albedo_bc1.dds",
        AssetKind::Texture,
    );

    assert_dependencies(
        &manager,
        "res://models/hero.gltf#Scene0",
        &[
            "res://models/hero.gltf#Node0",
            "res://models/hero.gltf#Mesh0",
            "res://models/hero.gltf#Mesh0/Primitive0",
            "res://models/hero.gltf#Material0",
        ],
    );
    assert_dependencies(
        &manager,
        "res://models/hero.gltf#Material0",
        &[
            "res://models/hero.gltf#Texture0",
            "res://shaders/default_pbr.zshader",
        ],
    );
    assert_dependencies(
        &manager,
        "res://materials/hero_surface.zmaterial",
        &[
            "res://shaders/lit_sample",
            "res://textures/hero_albedo_bc1.dds",
        ],
    );

    let asset_manager = project_asset_manager_with_sample_importers();
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    assert_loaded_with_dependencies::<crate::asset::SceneAsset>(
        &asset_manager,
        "res://models/hero.gltf#Scene0",
    );
    assert_loaded_with_dependencies::<ModelAsset>(&asset_manager, "res://models/hero.gltf");
    assert_loaded_with_dependencies::<MeshAsset>(
        &asset_manager,
        "res://models/hero.gltf#Mesh0/Primitive0",
    );
    assert_loaded_with_dependencies::<MaterialAsset>(
        &asset_manager,
        "res://materials/hero_surface.zmaterial",
    );
    assert_loaded_with_dependencies::<ShaderAsset>(&asset_manager, "res://shaders/lit_sample");
    assert_loaded_with_dependencies::<TextureAsset>(
        &asset_manager,
        "res://textures/hero_albedo_bc1.dds",
    );

    let root_model = load_model(&manager, "res://models/hero.gltf");
    assert_eq!(root_model.primitives.len(), 1);
    assert_eq!(root_model.primitives[0].vertices.len(), 3);
    assert_eq!(root_model.primitives[0].indices, vec![0, 1, 2]);
    assert_eq!(
        root_model.primitives[0]
            .mesh
            .as_ref()
            .map(|reference| reference.locator.clone()),
        Some(uri("res://models/hero.gltf#Mesh0/Primitive0"))
    );

    let scene = load_scene(&manager, "res://models/hero.gltf#Scene0");
    let scene_record = resource_record(&manager, "res://models/hero.gltf#Scene0");
    assert_eq!(scene.entities.len(), 1);
    let entity_mesh = scene.entities[0].mesh.as_ref().expect("scene mesh entity");
    assert_eq!(
        entity_mesh.model.locator,
        uri("res://models/hero.gltf#Mesh0")
    );
    assert_eq!(
        entity_mesh.material.locator,
        uri("res://models/hero.gltf#Material0")
    );
    assert!(entity_mesh.mesh.is_none());
    assert_eq!(entity_mesh.primitives.len(), 1);
    assert_eq!(
        entity_mesh.primitives[0].mesh.locator,
        uri("res://models/hero.gltf#Mesh0/Primitive0")
    );
    assert_eq!(
        entity_mesh.primitives[0].material.locator,
        uri("res://models/hero.gltf#Material0")
    );
    let scene_overview = scene.overview();
    assert_eq!(scene_overview.direct_reference_count, 4);
    assert_eq!(scene_overview.direct_mesh_reference_count, 1);
    assert_eq!(scene_overview.mesh_primitive_binding_count, 1);
    let scene_management = SceneAssetManagementRecordSet::from_records(vec![
        scene.management_record(scene_record.id())
    ]);
    assert_eq!(scene_management.summary.scene_count, 1);
    assert_eq!(scene_management.summary.entity_count, 1);
    assert_eq!(scene_management.summary.mesh_instance_count, 1);
    assert_eq!(scene_management.summary.direct_reference_count, 4);
    assert_eq!(scene_management.summary.direct_mesh_reference_count, 1);
    assert_eq!(scene_management.summary.mesh_primitive_binding_count, 1);
    assert_eq!(scene_management.summary.mesh_material_binding_count, 1);
    let scene_entity_management = SceneEntityManagementRecordSet::from_records(
        scene_management.records[0].entity_management_records(),
    );
    assert_eq!(
        scene_entity_management.records[0]
            .entity
            .direct_mesh_reference_count,
        1
    );
    assert_eq!(
        scene_entity_management.records[0]
            .entity
            .mesh_primitive_binding_count,
        1
    );
    assert_eq!(
        scene_entity_management.summary.direct_mesh_reference_count,
        1
    );
    assert_eq!(
        scene_entity_management.summary.mesh_primitive_binding_count,
        1
    );

    let mesh_model = load_model(&manager, "res://models/hero.gltf#Mesh0");
    let mesh_model_record = resource_record(&manager, "res://models/hero.gltf#Mesh0");
    let model_management = ModelAssetManagementRecordSet::from_records(vec![
        mesh_model.management_record(mesh_model_record.id())
    ]);
    assert_eq!(model_management.summary.model_count, 1);
    assert_eq!(model_management.summary.mesh_reference_count, 1);
    assert_eq!(model_management.summary.vertex_count, 3);
    assert_eq!(model_management.summary.index_count, 3);

    let mesh = load_mesh(&manager, "res://models/hero.gltf#Mesh0/Primitive0");
    let mesh_record = resource_record(&manager, "res://models/hero.gltf#Mesh0/Primitive0");
    let mesh_overview = mesh.overview().unwrap();
    assert_eq!(mesh_overview.vertex_count, 3);
    assert_eq!(mesh_overview.index_count, 3);
    let mesh_management = MeshAssetManagementRecordSet::from_results(vec![(
        mesh_record.id(),
        mesh.management_record(mesh_record.id()),
    )]);
    assert_eq!(mesh_management.summary.valid_mesh_count, 1);
    assert_eq!(mesh_management.summary.vertex_count, 3);
    assert_eq!(mesh_management.summary.index_count, 3);

    let gltf_material = load_material(&manager, "res://models/hero.gltf#Material0");
    assert_eq!(
        gltf_material
            .base_color_texture
            .as_ref()
            .map(|reference| reference.locator.clone()),
        Some(uri("res://models/hero.gltf#Texture0"))
    );

    let sample_material = load_material(&manager, "res://materials/hero_surface.zmaterial");
    let sample_shader = load_shader(&manager, "res://shaders/lit_sample");
    assert_eq!(
        sample_material.shader.locator,
        uri("res://shaders/lit_sample")
    );
    let base_color_slot = sample_material
        .texture_slots
        .get("base_color")
        .expect("base_color slot");
    assert_eq!(
        base_color_slot
            .reference
            .as_ref()
            .map(|reference| reference.locator.clone()),
        Some(uri("res://textures/hero_albedo_bc1.dds"))
    );
    assert_eq!(base_color_slot.fallback.as_deref(), Some("white"));
    let sample_material_readiness = sample_material.readiness_report_with_shader_contract(
        &sample_shader,
        |reference| {
            manager
                .registry()
                .get_by_locator(&reference.locator)
                .is_some()
        },
        |reference| {
            manager
                .registry()
                .get_by_locator(&reference.locator)
                .is_some()
        },
    );
    assert!(
        sample_material_readiness.is_ready(),
        "{sample_material_readiness:#?}"
    );
    let material_management = MaterialAssetManagementRecordSet::from_records(vec![sample_material
        .management_record(
            resource_record(&manager, "res://materials/hero_surface.zmaterial").id(),
        )]);
    assert_eq!(material_management.summary.material_count, 1);
    assert_eq!(material_management.summary.texture_reference_count, 2);
    assert_eq!(material_management.summary.fallback_texture_slot_count, 1);

    let shader_management =
        ShaderAssetManagementRecordSet::from_records(vec![sample_shader
            .management_record(resource_record(&manager, "res://shaders/lit_sample").id())]);
    let aggregate_management = AssetManagementRecordSets::from_record_sets(
        model_management,
        mesh_management,
        scene_management,
        scene_entity_management,
        material_management,
        RenderMaterialManagementRecordSet::default(),
        shader_management,
    );
    assert_eq!(aggregate_management.summary.managed_record_count, 6);
    assert_eq!(aggregate_management.summary.degraded_record_count, 0);
    assert_eq!(
        aggregate_management
            .summary
            .entity_direct_mesh_reference_count,
        1
    );
    assert_eq!(
        aggregate_management
            .summary
            .entity_mesh_primitive_binding_count,
        1
    );

    let compressed_texture = load_texture(&manager, "res://textures/hero_albedo_bc1.dds");
    assert_eq!(
        compressed_texture
            .upload_readiness(TextureUploadSupport::uncompressed_only())
            .unsupported_reason(),
        Some("gpu device does not support BC compressed textures")
    );

    let _ = fs::remove_dir_all(root);
}

fn project_manager_with_sample_importers(root: &Path) -> ProjectManager {
    let mut manager = ProjectManager::open(root).unwrap();
    manager
        .importer_mut()
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    manager
        .register_asset_importer(dds_container_importer())
        .unwrap();
    manager
}

fn project_asset_manager_with_sample_importers() -> ProjectAssetManager {
    let manager = ProjectAssetManager::default();
    manager
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    manager
        .register_asset_importer(dds_container_importer())
        .unwrap();
    manager
}

fn dds_container_importer() -> FunctionAssetImporter {
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new(
            "test.texture.dds.container",
            "test.texture",
            AssetKind::Texture,
            1,
        )
        .with_source_extensions(["dds"])
        .with_priority(130),
        import_dds_container_texture,
    )
}

fn import_dds_container_texture(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let texture = TextureAsset::new_container(
        context.uri.clone(),
        4,
        4,
        "dds/DXT1",
        context.source_bytes.clone(),
        1,
        1,
    );
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Texture(texture),
    ))
}

fn write_minimal_textured_gltf(path: PathBuf) {
    write_text(
        path,
        r#"
{
  "asset": { "version": "2.0" },
  "buffers": [
    {
      "uri": "data:application/octet-stream;base64,AAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAgD8AAAAAAAABAAIA",
      "byteLength": 42
    }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 6, "target": 34963 }
  ],
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 3,
      "type": "VEC3",
      "min": [0.0, 0.0, 0.0],
      "max": [1.0, 1.0, 0.0]
    },
    {
      "bufferView": 1,
      "componentType": 5123,
      "count": 3,
      "type": "SCALAR"
    }
  ],
  "images": [
    {
      "uri": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4////fwAJ+wP9KobjigAAAABJRU5ErkJggg=="
    }
  ],
  "textures": [
    { "source": 0 }
  ],
  "materials": [
    {
      "name": "HeroGLTFMaterial",
      "pbrMetallicRoughness": {
        "baseColorTexture": { "index": 0 },
        "baseColorFactor": [0.8, 0.9, 1.0, 1.0]
      }
    }
  ],
  "meshes": [
    {
      "name": "HeroTriangle",
      "primitives": [
        {
          "attributes": { "POSITION": 0 },
          "indices": 1,
          "material": 0
        }
      ]
    }
  ],
  "nodes": [
    { "name": "Hero", "mesh": 0 }
  ],
  "scenes": [
    { "name": "MainScene", "nodes": [0] }
  ],
  "scene": 0
}
"#,
    );
}

fn write_sample_shader_package(paths: &ProjectPaths) {
    let shader_uri = uri("res://shaders/lit_sample");
    let mut meta = AssetMetaDocument::new(
        AssetUuid::from_stable_label("minimal-asset-flow/lit-sample-shader"),
        shader_uri,
        AssetKind::Shader,
    );
    meta.unit = AssetSourceUnit::Compound;
    meta.save(paths.assets_root().join("shaders").join("lit_sample.zmeta"))
        .unwrap();

    write_text(
        paths
            .assets_root()
            .join("shaders")
            .join("lit_sample")
            .join("lit.zshader"),
        r#"
version = 1
name = "Lit Sample"
wgsl_files = ["lit.wgsl"]

[[entry_points]]
name = "vs_main"
stage = "vertex"

[[entry_points]]
name = "fs_main"
stage = "fragment"

[[properties]]
name = "base_color"
kind = "vec4"
required = true
default = [1.0, 1.0, 1.0, 1.0]

[[properties]]
name = "roughness"
kind = "float"
default = 1.0

[[texture_slots]]
name = "base_color"
kind = "texture2d"
required = true
default = "white"
sampler = "linear_repeat"
"#,
    );
    write_text(
        paths
            .assets_root()
            .join("shaders")
            .join("lit_sample")
            .join("lit.wgsl"),
        r#"
struct VsOut {
    @builtin(position) position: vec4f,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var positions = array<vec2f, 3>(
        vec2f(0.0, 0.5),
        vec2f(-0.5, -0.5),
        vec2f(0.5, -0.5),
    );
    var out: VsOut;
    out.position = vec4f(positions[vertex_index], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4f {
    let base_color = vec4f(1.0, 0.9, 0.7, 1.0);
    let roughness = 0.65;
    let base_color_texture = base_color;
    return vec4f(base_color_texture.rgb * roughness, base_color_texture.a);
}
"#,
    );
}

fn write_default_pbr_shader_package(paths: &ProjectPaths) {
    let shader_uri = uri("res://shaders/default_pbr.zshader");
    let mut meta = AssetMetaDocument::new(
        AssetUuid::from_stable_label("minimal-asset-flow/default-pbr-shader"),
        shader_uri,
        AssetKind::Shader,
    );
    meta.unit = AssetSourceUnit::Compound;
    meta.save(
        paths
            .assets_root()
            .join("shaders")
            .join("default_pbr.zshader.zmeta"),
    )
    .unwrap();

    write_text(
        paths
            .assets_root()
            .join("shaders")
            .join("default_pbr.zshader")
            .join("default_pbr.zshader"),
        r#"
version = 1
name = "Default PBR Sample"
wgsl_files = ["default_pbr.wgsl"]

[[entry_points]]
name = "vs_main"
stage = "vertex"

[[entry_points]]
name = "fs_main"
stage = "fragment"

[[properties]]
name = "base_color"
kind = "vec4"
default = [1.0, 1.0, 1.0, 1.0]

[[texture_slots]]
name = "base_color"
kind = "texture2d"
default = "white"
sampler = "linear_repeat"
"#,
    );
    write_text(
        paths
            .assets_root()
            .join("shaders")
            .join("default_pbr.zshader")
            .join("default_pbr.wgsl"),
        r#"
struct VsOut {
    @builtin(position) position: vec4f,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var positions = array<vec2f, 3>(
        vec2f(0.0, 0.5),
        vec2f(-0.5, -0.5),
        vec2f(0.5, -0.5),
    );
    var out: VsOut;
    out.position = vec4f(positions[vertex_index], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4f {
    let base_color = vec4f(1.0, 1.0, 1.0, 1.0);
    let base_color_texture = base_color;
    return base_color_texture;
}
"#,
    );
}

fn write_sample_material(paths: &ProjectPaths) {
    let shader = AssetReference::from_locator(uri("res://shaders/lit_sample"));
    let texture = AssetReference::from_locator(uri("res://textures/hero_albedo_bc1.dds"));
    let mut property_values = BTreeMap::new();
    property_values.insert(
        "base_color".to_string(),
        toml::Value::Array(vec![
            toml::Value::Float(1.0),
            toml::Value::Float(0.85),
            toml::Value::Float(0.55),
            toml::Value::Float(1.0),
        ]),
    );
    let mut base_color_slot = MaterialTextureSlotValue::new(texture.clone());
    base_color_slot.fallback = Some("white".to_string());
    let mut texture_slots = BTreeMap::new();
    texture_slots.insert("base_color".to_string(), base_color_slot);
    let material = MaterialAsset {
        name: Some("HeroSurface".to_string()),
        shader,
        base_color: [1.0, 0.85, 0.55, 1.0],
        base_color_texture: Some(texture),
        normal_texture: None,
        metallic: 0.0,
        roughness: 0.65,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values,
        texture_slots,
        validation_diagnostics: Vec::new(),
    };
    write_text(
        paths
            .assets_root()
            .join("materials")
            .join("hero_surface.zmaterial"),
        &material.to_toml_string().unwrap(),
    );
}

fn write_bc1_texture(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, dds_legacy_bytes("DXT1", 8)).unwrap();
}

fn write_text(path: PathBuf, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, text.trim_start()).unwrap();
}

fn assert_ready_record(manager: &ProjectManager, uri: &str, kind: AssetKind) {
    let record = resource_record(manager, uri);
    assert_eq!(record.kind, kind);
    assert_eq!(record.state, ResourceState::Ready);
    assert!(
        record.diagnostics.is_empty(),
        "{uri} should not emit diagnostics: {:?}",
        record.diagnostics
    );
}

fn assert_dependencies(manager: &ProjectManager, from: &str, expected: &[&str]) {
    let record = resource_record(manager, from);
    let mut actual = record.dependency_ids.clone();
    actual.sort();
    let mut expected_ids = expected
        .iter()
        .map(|dependency| resource_record(manager, dependency).id())
        .collect::<Vec<_>>();
    expected_ids.sort();
    assert_eq!(
        actual, expected_ids,
        "{from} dependencies should match the project sample graph"
    );
}

fn assert_loaded_with_dependencies<TAsset: Asset>(manager: &ProjectAssetManager, uri_text: &str) {
    let handle = manager.load::<TAsset>(&uri(uri_text)).unwrap();
    assert_eq!(manager.load_state(handle), AssetLoadState::Loaded);
    assert_eq!(
        manager.dependency_load_state(handle),
        DependencyLoadState::Loaded
    );
    assert_eq!(
        manager.recursive_dependency_load_state(handle),
        RecursiveDependencyLoadState::Loaded
    );
    assert_eq!(
        manager.load_states(handle),
        AssetLoadStates {
            load_state: AssetLoadState::Loaded,
            dependency_load_state: DependencyLoadState::Loaded,
            recursive_dependency_load_state: RecursiveDependencyLoadState::Loaded,
        }
    );
    assert!(manager.is_loaded_with_dependencies(handle));
}

fn resource_record<'a>(
    manager: &'a ProjectManager,
    uri_text: &str,
) -> &'a crate::core::resource::ResourceRecord {
    manager
        .registry()
        .get_by_locator(&uri(uri_text))
        .unwrap_or_else(|| panic!("missing resource record for {uri_text}"))
}

fn load_model(manager: &ProjectManager, uri_text: &str) -> crate::asset::ModelAsset {
    match manager.load_artifact(&uri(uri_text)).unwrap() {
        ImportedAsset::Model(model) => model,
        other => panic!("unexpected model artifact for {uri_text}: {other:?}"),
    }
}

fn load_mesh(manager: &ProjectManager, uri_text: &str) -> crate::asset::MeshAsset {
    match manager.load_artifact(&uri(uri_text)).unwrap() {
        ImportedAsset::Mesh(mesh) => mesh,
        other => panic!("unexpected mesh artifact for {uri_text}: {other:?}"),
    }
}

fn load_scene(manager: &ProjectManager, uri_text: &str) -> crate::asset::SceneAsset {
    match manager.load_artifact(&uri(uri_text)).unwrap() {
        ImportedAsset::Scene(scene) => scene,
        other => panic!("unexpected scene artifact for {uri_text}: {other:?}"),
    }
}

fn load_material(manager: &ProjectManager, uri_text: &str) -> MaterialAsset {
    match manager.load_artifact(&uri(uri_text)).unwrap() {
        ImportedAsset::Material(material) => material,
        other => panic!("unexpected material artifact for {uri_text}: {other:?}"),
    }
}

fn load_shader(manager: &ProjectManager, uri_text: &str) -> crate::asset::ShaderAsset {
    match manager.load_artifact(&uri(uri_text)).unwrap() {
        ImportedAsset::Shader(shader) => shader,
        other => panic!("unexpected shader artifact for {uri_text}: {other:?}"),
    }
}

fn load_texture(manager: &ProjectManager, uri_text: &str) -> TextureAsset {
    match manager.load_artifact(&uri(uri_text)).unwrap() {
        ImportedAsset::Texture(texture) => texture,
        other => panic!("unexpected texture artifact for {uri_text}: {other:?}"),
    }
}

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}

fn dds_legacy_bytes(fourcc: &str, payload_bytes: usize) -> Vec<u8> {
    let mut bytes = vec![0_u8; 128];
    bytes[0..4].copy_from_slice(b"DDS ");
    write_u32_le(&mut bytes, 4, 124);
    write_u32_le(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_LINEARSIZE);
    write_u32_le(&mut bytes, 12, 4);
    write_u32_le(&mut bytes, 16, 4);
    write_u32_le(&mut bytes, 20, payload_bytes as u32);
    write_u32_le(&mut bytes, 76, 32);
    write_u32_le(&mut bytes, 80, DDPF_FOURCC);
    bytes[84..88].copy_from_slice(fourcc.as_bytes());
    write_u32_le(&mut bytes, 108, DDSCAPS_TEXTURE);
    bytes.extend(vec![1_u8; payload_bytes]);
    bytes
}

fn write_u32_le(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

const DDPF_FOURCC: u32 = 0x0000_0004;
const DDSCAPS_TEXTURE: u32 = 0x0000_1000;
const DDSD_CAPS: u32 = 0x0000_0001;
const DDSD_HEIGHT: u32 = 0x0000_0002;
const DDSD_WIDTH: u32 = 0x0000_0004;
const DDSD_PIXELFORMAT: u32 = 0x0000_1000;
const DDSD_LINEARSIZE: u32 = 0x0008_0000;
const DDSD_REQUIRED_FLAGS: u32 = DDSD_CAPS | DDSD_HEIGHT | DDSD_WIDTH | DDSD_PIXELFORMAT;
