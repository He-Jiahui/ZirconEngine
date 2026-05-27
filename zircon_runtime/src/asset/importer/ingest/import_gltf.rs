use super::gltf_labeled_subassets::{
    add_gltf_animation_and_skin_placeholders, add_gltf_material_subassets, add_gltf_mesh_subassets,
    add_gltf_scene_subassets, add_gltf_texture_subassets, GltfMeshSubasset, GltfPrimitiveSubasset,
};
use std::collections::BTreeMap;

use super::primitive_from_indexed_mesh::primitive_from_indexed_mesh;
use crate::asset::assets::{
    MeshAttributeValues, MeshMorphTargetAsset, MeshSkinAsset, ModelAsset, MESH_ATTRIBUTE_NORMAL,
    MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT,
};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome, ImportedAsset};

pub(crate) fn import_gltf(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
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

        // A MeshAsset currently owns one optional skin payload, so keep the first
        // node-level skin association deterministically until skin subassets carry
        // richer multi-skin bindings.
        mesh_skins
            .entry(mesh.index())
            .or_insert_with(|| MeshSkinAsset {
                inverse_bind_matrices: matrices.collect(),
            });
    }
    mesh_skins
}
