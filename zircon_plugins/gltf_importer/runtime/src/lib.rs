mod capability;
#[cfg(feature = "runtime")]
mod plugin;
#[cfg(feature = "runtime")]
mod subassets;
#[cfg(all(test, feature = "runtime"))]
mod test_fixtures;

#[cfg(feature = "runtime")]
use std::collections::BTreeMap;
#[cfg(feature = "runtime")]
use std::path::Path;

#[cfg(feature = "runtime")]
use subassets::{
    add_gltf_animation_placeholders_and_skin_subassets, add_gltf_material_subassets,
    add_gltf_mesh_subassets, add_gltf_scene_subassets, add_gltf_texture_subassets,
    gltf_label_reference, validate_gltf_texture_import_support, GltfMeshSubasset,
    GltfPrimitiveSubasset,
};
#[cfg(feature = "runtime")]
use zircon_runtime::asset::importer::{
    cook_mesh_asset_derived_data, project_indexed_mesh_primitive,
    remap_gltf_morph_targets_for_flat_normals, resolve_gltf_normal_texture_tangent_uv_attribute,
    validate_required_gltf_material_extension_support, IndexedMeshMissingNormalPolicy,
    IndexedMeshSource,
};
#[cfg(feature = "runtime")]
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, ImportedAsset, MeshAsset,
    MeshAttributeValues, MeshMorphTargetAsset, MeshSdfCookBudget, MeshSdfCookRequest,
    MeshSkinAsset, ModelAsset, ModelPrimitiveAsset, VirtualGeometryCookRequest,
    MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT,
};

pub use capability::{
    GLTF_IMPORTER_DECLARATION, IMPORTER_CAPABILITY, MODULE_NAME, NATIVE_PLUGIN_ID,
    NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY, NATIVE_RUNTIME_REGISTRATION_MANIFEST,
    PLUGIN_ID, RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
};
#[cfg(feature = "runtime")]
pub use plugin::{
    asset_importer_descriptors, dist_module_manifest, module_descriptor, package_manifest,
    plugin_registration, runtime_capabilities, runtime_module_manifest, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, supported_platforms, supported_targets,
    GltfImporterRuntimePlugin, GLTF_IMPORTER_DIST_CRATE_NAME, GLTF_IMPORTER_DIST_RUNTIME_ENTRY,
};

#[cfg(feature = "runtime")]
const STABLE_IMPORTER_SUPPORTED_REQUIRED_EXTENSIONS: &[&str] = &[
    "EXT_texture_webp",
    "KHR_mesh_quantization",
    "KHR_materials_anisotropy",
    "KHR_materials_clearcoat",
    "KHR_materials_emissive_strength",
    "KHR_materials_ior",
    "KHR_materials_transmission",
    "KHR_materials_unlit",
    "KHR_materials_volume",
    "KHR_texture_transform",
];

#[cfg(feature = "runtime")]
pub fn import_gltf(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let preflight_document = parse_gltf_preflight_document(&context.source_bytes)?;
    validate_external_gltf_buffers(context, &preflight_document)?;
    validate_gltf_texture_import_support(&preflight_document)?;
    let (document, buffers, images) = gltf::import(&context.source_path)
        .map_err(|error| AssetImportError::Parse(format!("parse gltf: {error}")))?;
    let mut primitives = Vec::new();
    let mut meshes = Vec::new();
    let mesh_skins = mesh_skin_assets_by_mesh(&document, &buffers);
    let source_hint = context.uri.to_string();
    let virtual_geometry_request = context.virtual_geometry_cook_request()?;
    let mesh_sdf_request = context.mesh_sdf_cook_request()?;
    let mut mesh_sdf_budget = MeshSdfCookBudget::default();

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
            let tangents = reader
                .read_tangents()
                .map(|set| set.collect::<Vec<_>>())
                .unwrap_or_default();
            let colors = reader
                .read_colors(0)
                .map(|set| set.into_rgba_f32().collect::<Vec<_>>())
                .unwrap_or_default();
            let texcoords = reader
                .read_tex_coords(0)
                .map(|set| {
                    set.into_f32()
                        .flat_map(|uv| uv.into_iter())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let texcoords1 = reader
                .read_tex_coords(1)
                .map(|set| {
                    set.into_f32()
                        .flat_map(|uv| uv.into_iter())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let normals_missing = normals.is_empty();
            let tangents_missing = tangents.is_empty() || normals_missing;
            let normal_tangent_uv_attribute = resolve_gltf_normal_texture_tangent_uv_attribute(
                &primitive,
                tangents_missing,
                &texcoords,
                &texcoords1,
            )?;
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

            let mut primitive_asset = project_indexed_mesh_primitive(
                IndexedMeshSource {
                    positions: &positions,
                    normals: &normals,
                    texcoords0: &texcoords,
                    texcoords1: &texcoords1,
                    tangents: &tangents,
                    colors: &colors,
                    indices: &indices,
                    joint_indices: &joint_indices,
                    joint_weights: &joint_weights,
                    missing_normal_policy: IndexedMeshMissingNormalPolicy::Flat,
                },
                mesh_name,
                &source_hint,
                &VirtualGeometryCookRequest::default(),
                &MeshSdfCookRequest::default(),
                &mut mesh_sdf_budget,
            )?;
            let primitive_label = format!("Mesh{}/Primitive{}", mesh.index(), primitive.index());
            let primitive_uri = gltf_label_reference(&context.uri, &primitive_label);
            primitive_asset.mesh = Some(primitive_uri.clone());
            let mut mesh_asset =
                MeshAsset::from_model_primitive(primitive_uri.locator.clone(), &primitive_asset);
            let mut morph_targets = morph_targets_from_reader(&reader);
            if normals_missing {
                remap_gltf_morph_targets_for_flat_normals(&mut morph_targets, &indices)?;
            }
            mesh_asset.morph_targets = morph_targets;
            mesh_asset.skin = mesh_skins.get(&mesh.index()).cloned();
            if tangents_missing {
                if let Some(uv_attribute) = normal_tangent_uv_attribute {
                    mesh_asset.attributes.remove(MESH_ATTRIBUTE_TANGENT);
                    mesh_asset
                        .try_generate_missing_tangents_for_uv(uv_attribute)
                        .map_err(|error| {
                            AssetImportError::Parse(format!(
                                "generate glTF MikkTSpace tangents for {primitive_label}: {error}"
                            ))
                        })?;
                }
            }
            mesh_asset
                .try_rebuild_morph_tangent_frames_for_uv(
                    normals_missing,
                    tangents_missing
                        .then_some(normal_tangent_uv_attribute)
                        .flatten(),
                )
                .map_err(|error| {
                    AssetImportError::Parse(format!(
                        "rebuild glTF morph tangent frames for {primitive_label}: {error}"
                    ))
                })?;
            cook_mesh_asset_derived_data(
                &mut mesh_asset,
                mesh_name,
                &source_hint,
                &virtual_geometry_request,
                &mesh_sdf_request,
                &mut mesh_sdf_budget,
            )?;
            primitives.push(ModelPrimitiveAsset {
                vertices: Vec::new(),
                indices: Vec::new(),
                mesh: Some(primitive_uri),
                mesh_sdf: None,
                virtual_geometry: None,
            });
            mesh_primitives.push(GltfPrimitiveSubasset {
                primitive_index: primitive.index(),
                material_index: primitive.material().index(),
                mesh: mesh_asset,
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
    outcome = add_gltf_texture_subassets(outcome, &context.uri, &document, images)?;
    outcome = add_gltf_material_subassets(outcome, &context.uri, &document);
    outcome = add_gltf_mesh_subassets(outcome, &context.uri, meshes);
    outcome = add_gltf_scene_subassets(outcome, &context.uri, &document);
    outcome = add_gltf_animation_placeholders_and_skin_subassets(
        outcome,
        &context.uri,
        &document,
        &buffers,
    )?;
    Ok(outcome)
}

#[cfg(feature = "runtime")]
fn parse_gltf_preflight_document(source_bytes: &[u8]) -> Result<gltf::Document, AssetImportError> {
    let gltf = gltf::Gltf::from_slice_without_validation(source_bytes)
        .map_err(|error| AssetImportError::Parse(format!("parse gltf: {error}")))?;
    let mut json = gltf.document.into_json();
    let required_extensions = json.extensions_required.clone();
    validate_gltf_required_extensions(&required_extensions)?;
    json.extensions_required.retain(|extension| {
        STABLE_IMPORTER_SUPPORTED_REQUIRED_EXTENSIONS.contains(&extension.as_str())
    });
    let document = gltf::Document::from_json(json)
        .map_err(|error| AssetImportError::Parse(format!("validate gltf: {error}")))?;
    validate_required_gltf_material_extension_support(&document, &required_extensions)?;
    Ok(document)
}

#[cfg(feature = "runtime")]
fn validate_gltf_required_extensions(required: &[String]) -> Result<(), AssetImportError> {
    if let Some(extension) = required.iter().find(|extension| {
        !STABLE_IMPORTER_SUPPORTED_REQUIRED_EXTENSIONS.contains(&extension.as_str())
    }) {
        return Err(AssetImportError::Parse(format!(
            "gltf requires unsupported extension `{extension}`"
        )));
    }
    Ok(())
}

#[cfg(feature = "runtime")]
fn validate_external_gltf_buffers(
    context: &AssetImportContext,
    document: &gltf::Document,
) -> Result<(), AssetImportError> {
    let base_dir = context
        .source_path
        .parent()
        .unwrap_or_else(|| Path::new(""));
    for buffer in document.buffers() {
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

#[cfg(feature = "runtime")]
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

#[cfg(feature = "runtime")]
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

#[cfg(all(test, feature = "runtime"))]
mod tests;

#[cfg(all(test, feature = "runtime"))]
#[path = "tests/index_admission.rs"]
mod index_admission_tests;

#[cfg(all(test, feature = "runtime"))]
#[path = "tests/hotpaths.rs"]
mod hotpath_tests;

#[cfg(all(test, feature = "runtime"))]
#[path = "tests/geometry_convergence.rs"]
mod geometry_convergence_tests;
