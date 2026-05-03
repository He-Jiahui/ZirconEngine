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
    let (document, buffers, _) = gltf::import(&context.source_path)
        .map_err(|error| AssetImportError::Parse(format!("parse gltf: {error}")))?;
    let mut primitives = Vec::new();
    let source_hint = context.uri.to_string();

    for mesh in document.meshes() {
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

            primitives.push(primitive_from_indexed_mesh(
                &positions,
                &normals,
                &texcoords,
                &indices,
                &joint_indices,
                &joint_weights,
                mesh_name,
                &source_hint,
            )?);
        }
    }

    Ok(AssetImportOutcome::new(ImportedAsset::Model(ModelAsset {
        uri: context.uri.clone(),
        primitives,
    })))
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
    use zircon_runtime::asset::AssetUri;

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

        match outcome.imported_asset {
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

        let _ = fs::remove_dir_all(root);
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
        fs::write(&buffer_path, bytes).unwrap();

        fs::write(
            &gltf_path,
            r#"{
  "asset": { "version": "2.0" },
  "buffers": [
    { "uri": "triangle.bin", "byteLength": 42 }
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
  "meshes": [
    {
      "primitives": [
        {
          "attributes": { "POSITION": 0 },
          "indices": 1
        }
      ]
    }
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0
}"#,
        )
        .unwrap();

        gltf_path
    }
}
