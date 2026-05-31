mod subassets;
#[cfg(test)]
mod test_fixtures;

use std::collections::BTreeMap;
use std::path::Path;

use subassets::{
    add_gltf_animation_and_skin_placeholders, add_gltf_material_subassets, add_gltf_mesh_subassets,
    add_gltf_scene_subassets, add_gltf_texture_subassets, gltf_label_reference, GltfMeshSubasset,
    GltfPrimitiveSubasset,
};
use zircon_runtime::asset::{
    cook_virtual_geometry_from_mesh, AssetImportContext, AssetImportError, AssetImportOutcome,
    AssetImporterDescriptor, AssetKind, FunctionAssetImporter, ImportedAsset, MeshAttributeValues,
    MeshMorphTargetAsset, MeshSkinAsset, MeshVertex, ModelAsset, ModelPrimitiveAsset,
    VirtualGeometryCookConfig, MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION,
    MESH_ATTRIBUTE_TANGENT,
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
    validate_external_gltf_buffers(context)?;
    let (document, buffers, images) = gltf::import(&context.source_path)
        .map_err(|error| AssetImportError::Parse(format!("parse gltf: {error}")))?;
    let mut primitives = Vec::new();
    let mut meshes = Vec::new();
    let mesh_skins = mesh_skin_assets_by_mesh(&document, &buffers);
    let source_hint = context.uri.to_string();

    for mesh in document.meshes() {
        let mut mesh_primitives = Vec::new();
        let mesh_name = mesh.name();
        for primitive in mesh.primitives() {
            let mode = primitive.mode();
            if mode != gltf::mesh::Mode::Triangles {
                return Err(AssetImportError::Parse(format!(
                    "unsupported gltf primitive mode {mode:?} at Mesh{}/Primitive{}; only Triangles is supported",
                    mesh.index(),
                    primitive.index()
                )));
            }
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

            let mut primitive_asset = primitive_from_indexed_mesh(
                &positions,
                &normals,
                &texcoords,
                &indices,
                &joint_indices,
                &joint_weights,
                mesh_name,
                &source_hint,
            )?;
            primitive_asset.mesh = Some(gltf_label_reference(
                &context.uri,
                &format!("Mesh{}/Primitive{}", mesh.index(), primitive.index()),
            ));
            primitives.push(primitive_asset.clone());
            mesh_primitives.push(GltfPrimitiveSubasset {
                primitive_index: primitive.index(),
                material_index: primitive.material().index(),
                morph_targets: morph_targets_from_reader(&reader),
                primitive: primitive_asset,
            });
        }
        meshes.push(GltfMeshSubasset {
            mesh_index: mesh.index(),
            skin: mesh_skins.get(&mesh.index()).cloned(),
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

fn validate_external_gltf_buffers(context: &AssetImportContext) -> Result<(), AssetImportError> {
    let gltf = gltf::Gltf::from_slice(&context.source_bytes)
        .map_err(|error| AssetImportError::Parse(format!("parse gltf: {error}")))?;
    let base_dir = context
        .source_path
        .parent()
        .unwrap_or_else(|| Path::new(""));
    for buffer in gltf.document.buffers() {
        let gltf::buffer::Source::Uri(uri) = buffer.source() else {
            continue;
        };
        if uri
            .get(..5)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:"))
        {
            continue;
        }
        let buffer_path = base_dir.join(uri);
        if !buffer_path.exists() {
            return Err(AssetImportError::Parse(format!(
                "parse gltf: missing external buffer `{uri}` referenced by Buffer{} at {}",
                buffer.index(),
                buffer_path.display()
            )));
        }
    }
    Ok(())
}

fn morph_targets_from_reader<'a, 's, F>(
    reader: &gltf::mesh::Reader<'a, 's, F>,
) -> Vec<MeshMorphTargetAsset>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
{
    reader
        .read_morph_targets()
        .enumerate()
        .filter_map(|(index, (positions, normals, tangents))| {
            let mut attributes = BTreeMap::new();
            if let Some(positions) = positions {
                attributes.insert(
                    MESH_ATTRIBUTE_POSITION.to_string(),
                    MeshAttributeValues::Float32x3(positions.collect()),
                );
            }
            if let Some(normals) = normals {
                attributes.insert(
                    MESH_ATTRIBUTE_NORMAL.to_string(),
                    MeshAttributeValues::Float32x3(normals.collect()),
                );
            }
            if let Some(tangents) = tangents {
                attributes.insert(
                    MESH_ATTRIBUTE_TANGENT.to_string(),
                    MeshAttributeValues::Float32x3(tangents.collect()),
                );
            }
            (!attributes.is_empty()).then(|| MeshMorphTargetAsset {
                name: Some(format!("MorphTarget{index}")),
                attributes,
            })
        })
        .collect()
}

fn mesh_skin_assets_by_mesh(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
) -> BTreeMap<usize, MeshSkinAsset> {
    let mut mesh_skins = BTreeMap::new();
    for node in document.nodes() {
        let Some(mesh) = node.mesh() else {
            continue;
        };
        let Some(skin) = node.skin() else {
            continue;
        };
        let Some(matrices) = skin
            .reader(|buffer| Some(&buffers[buffer.index()].0))
            .read_inverse_bind_matrices()
        else {
            continue;
        };

        // MeshAsset has one optional skin payload today, so keep the first
        // node-level binding until dedicated Skin subassets carry richer links.
        mesh_skins
            .entry(mesh.index())
            .or_insert_with(|| MeshSkinAsset {
                inverse_bind_matrices: matrices.collect(),
            });
    }
    mesh_skins
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
        mesh: None,
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
mod tests;
