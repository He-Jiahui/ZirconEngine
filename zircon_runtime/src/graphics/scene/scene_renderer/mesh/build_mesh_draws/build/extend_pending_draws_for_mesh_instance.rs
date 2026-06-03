use std::sync::Arc;

use crate::asset::{MeshAsset, ModelPrimitiveAsset};
use crate::core::framework::render::{DisplayMode, RenderMeshSnapshot};
use crate::core::math::{RenderMat4, Vec4};
use crate::core::resource::{MaterialMarker, ResourceHandle, ResourceId};

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::super::resources::{
    default_pipeline_key, GpuMeshResource, ResourceStreamer,
};
use super::super::super::super::primitives::render_mat4_or;
use super::super::raster_draws_for_mesh::raster_draws_for_mesh;
use super::mesh_draw_build_context::MeshDrawBuildContext;
use super::pending_mesh_draw::{PendingMeshDraw, PendingMeshGeometry};
use super::skinning::{skin_mesh_asset_primitive, skin_model_primitive};

pub(super) fn extend_pending_draws_for_mesh_instance(
    pending_draws: &mut Vec<PendingMeshDraw>,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    build_context: &MeshDrawBuildContext,
    mesh_instance: &RenderMeshSnapshot,
) {
    if let Some(allowed_entities) = build_context.allowed_virtual_geometry_entities.as_ref() {
        if !allowed_entities.contains(&mesh_instance.node_id) {
            return;
        }
    }

    let instance_tint = if build_context.selection.contains(&mesh_instance.node_id)
        && frame.overlays().display_mode != DisplayMode::WireOnly
    {
        mesh_instance.tint * Vec4::new(1.0, 0.94, 0.72, 1.0)
    } else {
        mesh_instance.tint
    };
    let model_matrix =
        render_mat4_or(mesh_instance.transform.matrix(), RenderMat4::IDENTITY).to_cols_array_2d();

    // Direct mesh snapshots bypass the model-primitive loop, so mirror the same CPU skinning path here.
    if let Some(mesh_handle) = mesh_instance.mesh.as_ref() {
        let mesh_id = mesh_handle.id();
        if let Some(mesh) = streamer.mesh(&mesh_id) {
            if let Some(dynamic_primitive) =
                dynamic_direct_mesh_primitive(streamer, frame, mesh_instance, &mesh_id)
            {
                push_dynamic_mesh_draws(
                    pending_draws,
                    streamer,
                    mesh_instance.material,
                    mesh_instance.mobility,
                    mesh.index_count,
                    &dynamic_primitive,
                    instance_tint,
                    model_matrix,
                );
            } else {
                push_prepared_mesh_draws(
                    pending_draws,
                    streamer,
                    mesh_instance.material,
                    mesh_instance.mobility,
                    mesh,
                    instance_tint,
                    model_matrix,
                );
            }
            return;
        }
    }

    let Some(model) = streamer.model(&mesh_instance.model.id()) else {
        return;
    };
    let skinned_primitives = frame
        .extract
        .animation_poses
        .iter()
        .find(|entry| entry.entity == mesh_instance.node_id)
        .and_then(|entry| {
            let model_asset = streamer.load_model_asset(mesh_instance.model.id())?;
            let skeleton = streamer.load_animation_skeleton_asset(entry.skeleton)?;
            Some(
                model_asset
                    .primitives
                    .iter()
                    .map(|primitive| skin_model_primitive(primitive, &skeleton, &entry.pose).ok())
                    .collect::<Vec<_>>(),
            )
        });

    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
        if let Some(skinned_primitive) = skinned_primitives
            .as_ref()
            .and_then(|primitives| primitives.get(mesh_index))
            .cloned()
            .flatten()
        {
            push_dynamic_mesh_draws(
                pending_draws,
                streamer,
                mesh_instance.material,
                mesh_instance.mobility,
                mesh.index_count,
                &skinned_primitive,
                instance_tint,
                model_matrix,
            );
            continue;
        }

        let raster_draws = raster_draws_for_mesh(
            mesh.index_count,
            material_tinted(streamer, mesh_instance.material, instance_tint),
        );
        if raster_draws.is_empty() {
            continue;
        }

        for (first_index, draw_index_count, draw_tint) in raster_draws {
            pending_draws.push(PendingMeshDraw {
                mesh: PendingMeshGeometry::Prepared(mesh.clone()),
                mobility: mesh_instance.mobility,
                texture: streamer.texture(
                    streamer
                        .material(&mesh_instance.material.id())
                        .and_then(|material| material.base_color_texture),
                ),
                material_uniform: streamer.material_uniform(&mesh_instance.material.id()),
                pipeline_key: streamer
                    .material(&mesh_instance.material.id())
                    .map(|material| material.pipeline_key.clone())
                    .unwrap_or_else(default_pipeline_key),
                model_matrix,
                draw_tint,
                first_index,
                draw_index_count,
                indirect_draw_ref: None,
            });
        }
    }
}

fn material_tinted(
    streamer: &ResourceStreamer,
    material: ResourceHandle<MaterialMarker>,
    instance_tint: Vec4,
) -> Vec4 {
    let material_tint = streamer
        .material(&material.id())
        .map(|material| material.base_color)
        .unwrap_or(Vec4::ONE);
    instance_tint * material_tint
}

fn dynamic_direct_mesh_primitive(
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    mesh_instance: &RenderMeshSnapshot,
    mesh_id: &ResourceId,
) -> Option<ModelPrimitiveAsset> {
    skinned_direct_mesh_primitive(streamer, frame, mesh_instance, mesh_id)
        .or_else(|| morphed_direct_mesh_primitive(streamer, mesh_instance, mesh_id))
}

fn skinned_direct_mesh_primitive(
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    mesh_instance: &RenderMeshSnapshot,
    mesh_id: &ResourceId,
) -> Option<ModelPrimitiveAsset> {
    let pose_entry = frame
        .extract
        .animation_poses
        .iter()
        .find(|entry| entry.entity == mesh_instance.node_id)?;
    let mesh_asset = streamer.mesh_asset(mesh_id)?;
    let skeleton = streamer.load_animation_skeleton_asset(pose_entry.skeleton)?;
    skin_mesh_asset_primitive(
        mesh_asset,
        &skeleton,
        &pose_entry.pose,
        &mesh_instance.morph_weights,
    )
    .ok()
}

fn morphed_direct_mesh_primitive(
    streamer: &ResourceStreamer,
    mesh_instance: &RenderMeshSnapshot,
    mesh_id: &ResourceId,
) -> Option<ModelPrimitiveAsset> {
    let mesh_asset = streamer.mesh_asset(mesh_id)?;
    morphed_mesh_asset_primitive(mesh_asset.as_ref(), &mesh_instance.morph_weights)
}

fn morphed_mesh_asset_primitive(
    mesh_asset: &MeshAsset,
    morph_weights: &[f32],
) -> Option<ModelPrimitiveAsset> {
    if !morph_weights
        .iter()
        .any(|weight| weight.abs() > f32::EPSILON)
    {
        return None;
    }
    mesh_asset.to_morphed_model_primitive(morph_weights).ok()
}

fn push_dynamic_mesh_draws(
    pending_draws: &mut Vec<PendingMeshDraw>,
    streamer: &ResourceStreamer,
    material_id: ResourceHandle<MaterialMarker>,
    mobility: crate::core::framework::scene::Mobility,
    index_count: u32,
    dynamic_primitive: &ModelPrimitiveAsset,
    instance_tint: Vec4,
    model_matrix: [[f32; 4]; 4],
) {
    let material = streamer.material(&material_id.id());
    let texture = streamer.texture(material.and_then(|material| material.base_color_texture));
    let material_uniform = streamer.material_uniform(&material_id.id());
    let pipeline_key = material
        .map(|material| material.pipeline_key.clone())
        .unwrap_or_else(default_pipeline_key);
    for (first_index, draw_index_count, draw_tint) in raster_draws_for_mesh(
        index_count,
        material_tinted(streamer, material_id, instance_tint),
    ) {
        pending_draws.push(PendingMeshDraw {
            mesh: PendingMeshGeometry::Dynamic(dynamic_primitive.clone()),
            mobility,
            texture: texture.clone(),
            material_uniform: material_uniform.clone(),
            pipeline_key: pipeline_key.clone(),
            model_matrix,
            draw_tint,
            first_index,
            draw_index_count,
            indirect_draw_ref: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::morphed_mesh_asset_primitive;
    use crate::asset::{
        AssetUri, MeshAsset, MeshAttributeValues, MeshIndices, MeshMorphTargetAsset,
        MESH_ATTRIBUTE_POSITION,
    };
    use crate::core::framework::render::RenderMeshTopology;
    use crate::core::math::Vec3;

    #[test]
    fn morphed_mesh_asset_primitive_ignores_zero_weights_for_static_direct_mesh_fallback() {
        let mesh = morph_test_mesh();

        assert!(morphed_mesh_asset_primitive(&mesh, &[0.0]).is_none());
    }

    #[test]
    fn morphed_mesh_asset_primitive_applies_nonzero_weights_for_dynamic_direct_mesh() {
        let mesh = morph_test_mesh();

        let primitive = morphed_mesh_asset_primitive(&mesh, &[0.5]).expect("morphed primitive");

        assert!(Vec3::from_array(primitive.vertices[0].position)
            .abs_diff_eq(Vec3::new(1.0, 0.0, 0.5), 1.0e-6));
        assert_eq!(primitive.indices, vec![0, 1, 2]);
    }

    fn morph_test_mesh() -> MeshAsset {
        let mut mesh = MeshAsset::new(
            AssetUri::parse("res://meshes/direct-morph.zmesh").unwrap(),
            RenderMeshTopology::TriangleList,
            BTreeMap::from([(
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![
                    [1.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                ]),
            )]),
            Some(MeshIndices::U32(vec![0, 1, 2])),
        )
        .unwrap();
        mesh.morph_targets = vec![MeshMorphTargetAsset {
            name: Some("Lift".to_string()),
            attributes: BTreeMap::from([(
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 3]),
            )]),
        }];
        mesh
    }
}

fn push_prepared_mesh_draws(
    pending_draws: &mut Vec<PendingMeshDraw>,
    streamer: &ResourceStreamer,
    material_id: ResourceHandle<MaterialMarker>,
    mobility: crate::core::framework::scene::Mobility,
    mesh: &Arc<GpuMeshResource>,
    instance_tint: Vec4,
    model_matrix: [[f32; 4]; 4],
) {
    let material = streamer.material(&material_id.id());
    let texture = streamer.texture(material.and_then(|material| material.base_color_texture));
    let material_uniform = streamer.material_uniform(&material_id.id());
    let pipeline_key = material
        .map(|material| material.pipeline_key.clone())
        .unwrap_or_else(default_pipeline_key);
    for (first_index, draw_index_count, draw_tint) in raster_draws_for_mesh(
        mesh.index_count,
        material_tinted(streamer, material_id, instance_tint),
    ) {
        pending_draws.push(PendingMeshDraw {
            mesh: PendingMeshGeometry::Prepared(mesh.clone()),
            mobility,
            texture: texture.clone(),
            material_uniform: material_uniform.clone(),
            pipeline_key: pipeline_key.clone(),
            model_matrix,
            draw_tint,
            first_index,
            draw_index_count,
            indirect_draw_ref: None,
        });
    }
}
