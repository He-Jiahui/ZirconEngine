use super::gltf_labeled_subassets::{
    add_gltf_animation_and_skin_placeholders, add_gltf_material_subassets, add_gltf_mesh_subassets,
    add_gltf_scene_subassets, add_gltf_texture_subassets, GltfMeshSubasset, GltfPrimitiveSubasset,
};
use super::primitive_from_indexed_mesh::primitive_from_indexed_mesh;
use crate::asset::assets::ModelAsset;
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome, ImportedAsset};

pub(crate) fn import_gltf(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
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
