mod subassets;

use subassets::{
    add_gltf_animation_and_skin_placeholders, add_gltf_material_subassets, add_gltf_mesh_subassets,
    add_gltf_scene_subassets, add_gltf_texture_subassets, GltfMeshSubasset, GltfPrimitiveSubasset,
};
use zircon_runtime::asset::{
    cook_virtual_geometry_from_mesh, AssetImportContext, AssetImportError, AssetImportOutcome,
    AssetImporterDescriptor, AssetKind, FunctionAssetImporter, ImportedAsset, MeshVertex,
    ModelAsset, ModelPrimitiveAsset, VirtualGeometryCookConfig,
};
use zircon_runtime::core::math::{Vec2, Vec3};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginModuleManifest,
    plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeExtensionRegistry, plugin::RuntimeExtensionRegistryError,
    plugin::RuntimePluginRegistrationReport, RuntimeTargetMode,
};

pub const PLUGIN_ID: &str = "gltf_importer";
pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_gltf_importer_runtime";
pub const MODULE_NAME: &str = "GltfImporterModule";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.gltf_importer";
pub const IMPORTER_CAPABILITY: &str = "runtime.asset.importer.model.gltf";

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RUNTIME_CAPABILITY, IMPORTER_CAPABILITY]
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    [
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    [
        ExportTargetPlatform::Windows,
        ExportTargetPlatform::Linux,
        ExportTargetPlatform::Macos,
    ]
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(MODULE_NAME, "glTF and GLB model importer plugin")
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        AssetImporterDescriptor::new("gltf_importer.gltf", PLUGIN_ID, AssetKind::Model, 1)
            .with_priority(120)
            .with_source_extensions(["gltf", "glb"])
            .with_additional_output_kinds([
                AssetKind::Mesh,
                AssetKind::Scene,
                AssetKind::Material,
                AssetKind::Texture,
                AssetKind::Data,
            ])
            .with_required_capabilities([IMPORTER_CAPABILITY]),
    ]
}

pub fn package_manifest() -> PluginPackageManifest {
    let mut manifest = PluginPackageManifest::new(PLUGIN_ID, "glTF Importer")
        .with_category("asset_importer")
        .with_supported_targets(supported_targets())
        .with_supported_platforms(supported_platforms())
        .with_capabilities(runtime_capabilities().iter().copied())
        .with_runtime_module(runtime_module_manifest());
    for importer in asset_importer_descriptors() {
        manifest = manifest.with_asset_importer(importer);
    }
    manifest
}

pub fn runtime_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::runtime("gltf_importer.runtime", RUNTIME_CRATE_NAME)
        .with_target_modes(supported_targets())
        .with_capabilities(runtime_capabilities().iter().copied())
}

pub fn runtime_selection() -> ProjectPluginSelection {
    ProjectPluginSelection {
        id: PLUGIN_ID.to_string(),
        enabled: true,
        required: false,
        target_modes: supported_targets().to_vec(),
        packaging: ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: Some(RUNTIME_CRATE_NAME.to_string()),
        editor_crate: None,
        features: Vec::new(),
    }
}

pub fn plugin_registration() -> RuntimePluginRegistrationReport {
    let mut extensions = RuntimeExtensionRegistry::default();
    let mut diagnostics = Vec::new();
    if let Err(error) = register_runtime_extensions(&mut extensions) {
        diagnostics.push(error.to_string());
    }
    RuntimePluginRegistrationReport {
        package_manifest: package_manifest(),
        project_selection: runtime_selection(),
        extensions,
        diagnostics,
    }
}

pub fn register_runtime_extensions(
    registry: &mut RuntimeExtensionRegistry,
) -> Result<(), RuntimeExtensionRegistryError> {
    registry.register_module(module_descriptor())?;
    for importer in asset_importer_descriptors() {
        registry.register_asset_importer(FunctionAssetImporter::new(importer, import_gltf))?;
    }
    Ok(())
}

pub fn import_gltf(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let (document, buffers, images) = gltf::import(&context.source_path)
        .map_err(|error| AssetImportError::Parse(format!("parse gltf: {error}")))?;
    let mut primitives = Vec::new();
    let mut meshes = Vec::new();
    let source_hint = context.uri.to_string();

    for mesh in document.meshes() {
        let mut mesh_primitives = Vec::new();
        let mesh_name = mesh.name();
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));
            let positions = reader
                .read_positions()
                .ok_or_else(|| {
                    AssetImportError::Parse("gltf primitive missing positions".to_string())
                })?
                .flat_map(|position| position.into_iter())
                .collect::<Vec<_>>();
            let normals = reader
                .read_normals()
                .map(|iter| {
                    iter.flat_map(|normal| normal.into_iter())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let texcoords = reader
                .read_tex_coords(0)
                .map(|set| {
                    set.into_f32()
                        .flat_map(|uv| uv.into_iter())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let joint_indices = reader
                .read_joints(0)
                .map(|set| set.into_u16().collect::<Vec<_>>())
                .unwrap_or_default();
            let joint_weights = reader
                .read_weights(0)
                .map(|set| set.into_f32().collect::<Vec<_>>())
                .unwrap_or_default();
            let indices = reader
                .read_indices()
                .map(|indices| indices.into_u32().collect::<Vec<_>>())
                .unwrap_or_else(|| {
                    let vertex_count = positions.len() / 3;
                    (0..vertex_count as u32).collect()
                });

            let primitive_asset = primitive_from_indexed_mesh(
                &positions,
                &normals,
                &texcoords,
                &indices,
                &joint_indices,
                &joint_weights,
                mesh_name,
                &source_hint,
            )?;
            primitives.push(primitive_asset.clone());
            mesh_primitives.push(GltfPrimitiveSubasset {
                primitive_index: primitive.index(),
                material_index: primitive.material().index(),
                primitive: primitive_asset,
            });
        }
        meshes.push(GltfMeshSubasset {
            mesh_index: mesh.index(),
            primitives: mesh_primitives,
        });
    }

    let model = ModelAsset {
        uri: context.uri.clone(),
        primitives,
    };
    let mut outcome = AssetImportOutcome::new(context.uri.clone(), ImportedAsset::Model(model));
    outcome = add_gltf_texture_subassets(outcome, &context.uri, &document, &images)?;
    outcome = add_gltf_material_subassets(outcome, &context.uri, &document);
    outcome = add_gltf_mesh_subassets(outcome, &context.uri, &meshes);
    outcome = add_gltf_scene_subassets(outcome, &context.uri, &document);
    outcome = add_gltf_animation_and_skin_placeholders(outcome, &context.uri, &document);
    Ok(outcome)
}

fn primitive_from_indexed_mesh(
    positions: &[f32],
    normals: &[f32],
    texcoords: &[f32],
    indices: &[u32],
    joint_indices: &[[u16; 4]],
    joint_weights: &[[f32; 4]],
    mesh_name: Option<&str>,
    source_hint: &str,
) -> Result<ModelPrimitiveAsset, AssetImportError> {
    if positions.len() % 3 != 0 {
        return Err(AssetImportError::Parse(
            "vertex positions were not a multiple of 3".to_string(),
        ));
    }
    let vertex_count = positions.len() / 3;
    let mut computed_normals = if normals.is_empty() {
        generate_normals(positions, indices)
    } else {
        normals.to_vec()
    };
    if computed_normals.len() < vertex_count * 3 {
        computed_normals.resize(vertex_count * 3, 0.0);
    }

    let vertices: Vec<MeshVertex> = (0..vertex_count)
        .map(|index| {
            let position = Vec3::new(
                positions[index * 3],
                positions[index * 3 + 1],
                positions[index * 3 + 2],
            );
            let normal = Vec3::new(
                computed_normals[index * 3],
                computed_normals[index * 3 + 1],
                computed_normals[index * 3 + 2],
            );
            let uv = if texcoords.len() >= (index + 1) * 2 {
                Vec2::new(texcoords[index * 2], texcoords[index * 2 + 1])
            } else {
                Vec2::ZERO
            };
            MeshVertex::new(
                position,
                if normal.length_squared() <= f32::EPSILON {
                    Vec3::Y
                } else {
                    normal.normalize_or_zero()
                },
                uv,
            )
            .with_skinning(
                joint_indices.get(index).copied().unwrap_or([0, 0, 0, 0]),
                joint_weights
                    .get(index)
                    .copied()
                    .unwrap_or([0.0, 0.0, 0.0, 0.0]),
            )
        })
        .collect();

    let virtual_geometry = cook_virtual_geometry_from_mesh(
        &vertices,
        indices,
        VirtualGeometryCookConfig {
            mesh_name: mesh_name.map(str::to_owned),
            source_hint: Some(source_hint.to_string()),
            ..VirtualGeometryCookConfig::default()
        },
    );

    Ok(ModelPrimitiveAsset {
        vertices,
        indices: indices.to_vec(),
        virtual_geometry,
    })
}

fn generate_normals(positions: &[f32], indices: &[u32]) -> Vec<f32> {
    let vertex_count = positions.len() / 3;
    let mut normals = vec![0.0_f32; vertex_count * 3];

    for triangle in indices.chunks_exact(3) {
        let a = triangle[0] as usize;
        let b = triangle[1] as usize;
        let c = triangle[2] as usize;
        let position = |index: usize| -> Vec3 {
            Vec3::new(
                positions[index * 3],
                positions[index * 3 + 1],
                positions[index * 3 + 2],
            )
        };
        let face_normal = (position(b) - position(a))
            .cross(position(c) - position(a))
            .normalize_or_zero();
        for index in [a, b, c] {
            normals[index * 3] += face_normal.x;
            normals[index * 3 + 1] += face_normal.y;
            normals[index * 3 + 2] += face_normal.z;
        }
    }

    normals
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};
    use zircon_runtime::asset::{AssetUri, ImportedAssetEntry};

    #[test]
    fn package_declares_gltf_importer() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(manifest
            .asset_importers
            .iter()
            .any(|importer| importer.source_extensions.contains(&"glb".to_string())));
    }

    #[test]
    fn registration_contributes_module_and_importer() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == MODULE_NAME));
        assert!(report
            .extensions
            .asset_importers()
            .descriptors()
            .iter()
            .any(|importer| importer.id == "gltf_importer.gltf"));
    }

    #[test]
    fn importer_decodes_triangle_gltf_into_model_asset() {
        let root = unique_temp_root("gltf_importer_decode");
        fs::create_dir_all(&root).unwrap();
        let gltf_path = write_triangle_gltf(&root);
        let source_bytes = fs::read(&gltf_path).unwrap();
        let outcome = import_gltf(&AssetImportContext::new(
            gltf_path,
            AssetUri::parse("res://models/triangle.gltf").unwrap(),
            source_bytes,
            toml::Table::new(),
        ))
        .unwrap();

        let entry = outcome.root_entry().expect("root gltf asset entry");
        match &entry.asset {
            ImportedAsset::Model(model) => {
                assert_eq!(model.primitives.len(), 1);
                assert_eq!(model.primitives[0].vertices.len(), 3);
                assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
                let virtual_geometry = model.primitives[0]
                    .virtual_geometry
                    .as_ref()
                    .expect("plugin importer should cook virtual geometry");
                assert_eq!(
                    virtual_geometry.debug.source_hint.as_deref(),
                    Some("res://models/triangle.gltf")
                );
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
        assert!(outcome
            .entries
            .iter()
            .any(|entry| matches!(&entry.asset, ImportedAsset::Mesh(_))));
        assert!(entry.dependencies.contains(&label_uri("Scene0")));
        assert!(entry.dependencies.contains(&label_uri("Node0")));
        assert!(entry.dependencies.contains(&label_uri("Mesh0")));
        assert!(entry.dependencies.contains(&label_uri("Mesh0/Primitive0")));
        assert!(entry.dependencies.contains(&label_uri("Material0")));
        assert!(entry.dependencies.contains(&label_uri("Texture0")));
        assert!(entry.dependencies.contains(&label_uri("DefaultMaterial")));
        assert!(entry.dependencies.contains(&label_uri("Animation0")));
        assert!(entry.dependencies.contains(&label_uri("Skin0")));
        assert!(entry
            .dependencies
            .contains(&label_uri("Skin0/InverseBindMatrices")));

        match &entry_for_label(&outcome, "Texture0").asset {
            ImportedAsset::Texture(texture) => {
                assert_eq!(texture.width, 1);
                assert_eq!(texture.height, 1);
                assert_eq!(texture.rgba.len(), 4);
            }
            other => panic!("unexpected Texture0 asset: {other:?}"),
        }
        let material_entry = entry_for_label(&outcome, "Material0");
        assert!(material_entry.dependencies.contains(&label_uri("Texture0")));
        assert!(material_entry
            .dependencies
            .contains(&default_pbr_shader_uri()));
        match &material_entry.asset {
            ImportedAsset::Material(material) => {
                assert_eq!(material.name.as_deref(), Some("TriangleMaterial"));
                assert_eq!(material.base_color, [0.2, 0.3, 0.4, 1.0]);
                assert_eq!(material.metallic, 0.5);
                assert_eq!(material.roughness, 0.6);
                assert_eq!(
                    material.base_color_texture.as_ref().unwrap().locator,
                    label_uri("Texture0")
                );
            }
            other => panic!("unexpected Material0 asset: {other:?}"),
        }
        match &entry_for_label(&outcome, "Mesh0").asset {
            ImportedAsset::Model(model) => assert_eq!(model.primitives.len(), 1),
            other => panic!("unexpected Mesh0 asset: {other:?}"),
        }
        match &entry_for_label(&outcome, "Mesh0/Primitive0").asset {
            ImportedAsset::Mesh(mesh) => assert_eq!(mesh.vertex_count().unwrap(), 3),
            other => panic!("unexpected Mesh0/Primitive0 asset: {other:?}"),
        }
        match &entry_for_label(&outcome, "Node0").asset {
            ImportedAsset::Scene(scene) => {
                assert_eq!(scene.entities.len(), 1);
                let entity = &scene.entities[0];
                assert_eq!(entity.name, "TriangleNode");
                let mesh = entity.mesh.as_ref().expect("node mesh instance");
                assert_eq!(mesh.model.locator, label_uri("Mesh0"));
                assert_eq!(mesh.material.locator, label_uri("Material0"));
            }
            other => panic!("unexpected Node0 asset: {other:?}"),
        }
        match &entry_for_label(&outcome, "Scene0").asset {
            ImportedAsset::Scene(scene) => assert_eq!(scene.entities.len(), 1),
            other => panic!("unexpected Scene0 asset: {other:?}"),
        }
        let default_material_entry = entry_for_label(&outcome, "DefaultMaterial");
        assert!(default_material_entry
            .dependencies
            .contains(&default_pbr_shader_uri()));
        assert!(matches!(
            &default_material_entry.asset,
            ImportedAsset::Material(_)
        ));
        for label in ["Animation0", "Skin0", "Skin0/InverseBindMatrices"] {
            match &entry_for_label(&outcome, label).asset {
                ImportedAsset::Data(data) => assert!(
                    data.text.contains("not implemented yet"),
                    "{label} should carry a diagnostic placeholder"
                ),
                other => panic!("unexpected {label} asset: {other:?}"),
            }
        }

        let _ = fs::remove_dir_all(root);
    }

    fn entry_for_label<'a>(outcome: &'a AssetImportOutcome, label: &str) -> &'a ImportedAssetEntry {
        let locator = label_uri(label);
        outcome
            .entries
            .iter()
            .find(|entry| entry.locator == locator)
            .unwrap_or_else(|| panic!("missing gltf subasset {locator}"))
    }

    fn label_uri(label: &str) -> AssetUri {
        AssetUri::parse(&format!("res://models/triangle.gltf#{label}")).unwrap()
    }

    fn default_pbr_shader_uri() -> AssetUri {
        AssetUri::parse("res://shaders/default_pbr.zshader").unwrap()
    }

    fn unique_temp_root(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("zircon_plugin_{label}_{unique}"))
    }

    fn write_triangle_gltf(root: &Path) -> PathBuf {
        let buffer_path = root.join("triangle.bin");
        let gltf_path = root.join("triangle.gltf");

        let mut bytes = Vec::new();
        for value in [
            0.0_f32, 0.0, 0.0, //
            1.0, 0.0, 0.0, //
            0.0, 1.0, 0.0,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for index in [0_u16, 1, 2] {
            bytes.extend_from_slice(&index.to_le_bytes());
        }
        bytes.extend_from_slice(&[0, 0]);
        for value in [
            1.0_f32, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&0.0_f32.to_le_bytes());
        for value in [0.0_f32, 0.0, 0.0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        fs::write(&buffer_path, bytes).unwrap();

        fs::write(
            &gltf_path,
            r#"{
  "asset": { "version": "2.0" },
  "buffers": [
    { "uri": "triangle.bin", "byteLength": 124 }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 6, "target": 34963 },
    { "buffer": 0, "byteOffset": 44, "byteLength": 64 },
    { "buffer": 0, "byteOffset": 108, "byteLength": 4 },
    { "buffer": 0, "byteOffset": 112, "byteLength": 12 }
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
    },
    {
      "bufferView": 2,
      "componentType": 5126,
      "count": 1,
      "type": "MAT4"
    },
    {
      "bufferView": 3,
      "componentType": 5126,
      "count": 1,
      "type": "SCALAR",
      "min": [0.0],
      "max": [0.0]
    },
    {
      "bufferView": 4,
      "componentType": 5126,
      "count": 1,
      "type": "VEC3"
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
      "name": "TriangleMaterial",
      "pbrMetallicRoughness": {
        "baseColorFactor": [0.2, 0.3, 0.4, 1.0],
        "baseColorTexture": { "index": 0 },
        "metallicFactor": 0.5,
        "roughnessFactor": 0.6
      }
    }
  ],
  "meshes": [
    {
      "name": "TriangleMesh",
      "primitives": [
        {
          "attributes": { "POSITION": 0 },
          "indices": 1,
          "material": 0
        }
      ]
    }
  ],
  "nodes": [{ "name": "TriangleNode", "mesh": 0, "skin": 0 }],
  "skins": [{ "inverseBindMatrices": 2, "joints": [0] }],
  "animations": [
    {
      "samplers": [{ "input": 3, "output": 4, "interpolation": "LINEAR" }],
      "channels": [{ "sampler": 0, "target": { "node": 0, "path": "translation" } }]
    }
  ],
  "scenes": [{ "name": "SceneRoot", "nodes": [0] }],
  "scene": 0
}"#,
        )
        .unwrap();

        gltf_path
    }
}
