use crate::core::framework::render::{DisplayMode, RenderMeshSnapshot};
use crate::core::math::{RenderMat4, Vec4};

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::super::resources::{default_pipeline_key, ResourceStreamer};
use super::super::super::super::primitives::render_mat4_or;
use super::super::raster_draws_for_mesh::raster_draws_for_mesh;
use super::super::virtual_geometry_cluster_streaming_tint::virtual_geometry_cluster_streaming_tint;
use super::mesh_draw_build_context::MeshDrawBuildContext;
use super::pending_mesh_draw::{
    indirect_draw_ref_for_cluster_draw, PendingMeshDraw, PendingMeshGeometry,
};
use super::skinning::skin_model_primitive;

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

    let Some(model) = streamer.model(&mesh_instance.model.id()) else {
        return;
    };
    let material = streamer.material(&mesh_instance.material.id());
    let texture = streamer.texture(material.and_then(|material| material.base_color_texture));
    let material_tint = material
        .map(|material| material.base_color)
        .unwrap_or(Vec4::ONE);
    let pipeline_key = material
        .map(|material| material.pipeline_key.clone())
        .unwrap_or_else(default_pipeline_key);
    let base_tint = if build_context.selection.contains(&mesh_instance.node_id)
        && frame.scene.overlays.display_mode != DisplayMode::WireOnly
    {
        mesh_instance.tint * material_tint * Vec4::new(1.0, 0.94, 0.72, 1.0)
    } else {
        mesh_instance.tint * material_tint
    };
    let model_matrix =
        render_mat4_or(mesh_instance.transform.matrix(), RenderMat4::IDENTITY).to_cols_array_2d();
    let cluster_raster_draws = build_context
        .virtual_geometry_cluster_draws
        .as_ref()
        .and_then(|cluster_draws| cluster_draws.get(&mesh_instance.node_id))
        .map(Vec::as_slice);
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
            for (first_index, draw_index_count, draw_tint) in
                raster_draws_for_mesh(mesh.index_count, None, base_tint)
            {
                pending_draws.push(PendingMeshDraw {
                    mesh: PendingMeshGeometry::Skinned(skinned_primitive.clone()),
                    texture: texture.clone(),
                    pipeline_key: pipeline_key.clone(),
                    model_matrix,
                    draw_tint,
                    first_index,
                    draw_index_count,
                    indirect_draw_ref: None,
                });
            }
            continue;
        }

        if build_context.virtual_geometry_enabled {
            if let Some(cluster_raster_draws) = cluster_raster_draws {
                for cluster_draw in cluster_raster_draws {
                    pending_draws.push(PendingMeshDraw {
                        mesh: PendingMeshGeometry::Prepared(mesh.clone()),
                        texture: texture.clone(),
                        pipeline_key: pipeline_key.clone(),
                        model_matrix,
                        draw_tint: base_tint
                            * virtual_geometry_cluster_streaming_tint(*cluster_draw),
                        first_index: 0,
                        draw_index_count: 0,
                        indirect_draw_ref: Some(indirect_draw_ref_for_cluster_draw(
                            mesh_instance.node_id,
                            mesh.index_count,
                            mesh.indirect_order_signature,
                            *cluster_draw,
                        )),
                    });
                }
            } else {
                // Under the prepare-owned VG path, "no cluster draws" is authoritative:
                // the entity collapsed out of unified indirect submission, so renderer-side
                // mesh build must not resurrect a private full-mesh fallback draw.
                continue;
            }
        } else {
            let raster_draws =
                raster_draws_for_mesh(mesh.index_count, cluster_raster_draws, base_tint);
            if raster_draws.is_empty() {
                continue;
            }

            for (first_index, draw_index_count, draw_tint) in raster_draws {
                pending_draws.push(PendingMeshDraw {
                    mesh: PendingMeshGeometry::Prepared(mesh.clone()),
                    texture: texture.clone(),
                    pipeline_key: pipeline_key.clone(),
                    model_matrix,
                    draw_tint,
                    first_index,
                    draw_index_count,
                    indirect_draw_ref: None,
                });
            }
        }
    }
}
