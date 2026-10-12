use crate::core::framework::render::{
    OverlayLineSegment, RenderOverlayExtract, RenderVirtualGeometryBvhVisualizationInstance,
    RenderVirtualGeometryBvhVisualizationNode, RenderVirtualGeometryDebugSnapshot,
    RenderVirtualGeometryExecutionState, RenderVirtualGeometryVisBufferMark, SceneGizmoKind,
    SceneGizmoOverlayExtract, UiRenderSubmission,
};
use crate::core::math::{Vec3, Vec4};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::graphics::{ViewportCameraStackOutputPolicy, ViewportRenderFrame};

use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;
use super::build_virtual_geometry_debug_snapshot::build_virtual_geometry_debug_snapshot;

pub(super) fn build_runtime_frame(
    ui: Option<Arc<UiRenderSubmission>>,
    context: &mut FrameSubmissionContext,
    prepared: PreparedRuntimeSubmission,
    output_policy: ViewportCameraStackOutputPolicy,
) -> ViewportRenderFrame {
    let extract = context.submission_extract();
    let virtual_geometry_debug_snapshot = build_virtual_geometry_debug_snapshot(&extract, context);
    let runtime_overlays = runtime_virtual_geometry_debug_overlays(
        &extract.debug.overlays,
        context,
        virtual_geometry_debug_snapshot.as_ref(),
    );
    let mut frame = ViewportRenderFrame::from_shared_extract(extract, context.size())
        .with_shader_quality(context.shader_quality())
        .with_texture_mip_bias(context.global_material_mip_bias().max(0.0).floor() as u8)
        .with_texture_max_anisotropy(context.texture_max_anisotropy())
        .with_output_target(context.output_target())
        .with_camera_stack_output_policy(output_policy)
        .with_ui(ui)
        .with_frame_visibility(context.visibility_context().frame_visibility.clone())
        .with_previous_motion_vector_camera(context.previous_motion_vector_camera().cloned())
        .with_post_process_override(context.post_process_shared())
        .with_environment_source_cubemap_override(
            context.take_environment_source_cubemap_override(),
        )
        .with_particle_previous_sprites_override(context.take_particle_previous_sprites_override())
        .with_prepared_runtime_sidebands(prepared.into_prepared_runtime_sidebands())
        .with_virtual_geometry_debug_snapshot(virtual_geometry_debug_snapshot.map(Arc::new));
    frame.render_region = frame.render_region().with_local_size(context.render_size());
    if let Some(runtime_overlays) = runtime_overlays {
        frame = frame.with_runtime_overlays(runtime_overlays);
    }
    frame
}

fn runtime_virtual_geometry_debug_overlays(
    source_overlays: &RenderOverlayExtract,
    context: &FrameSubmissionContext,
    snapshot: Option<&RenderVirtualGeometryDebugSnapshot>,
) -> Option<RenderOverlayExtract> {
    let Some(snapshot) = snapshot else {
        return None;
    };
    let visbuffer_debug_marks = build_current_frame_visbuffer_debug_marks(snapshot);
    if snapshot.bvh_visualization_instances.is_empty() && visbuffer_debug_marks.is_empty() {
        return None;
    }

    let mut overlays = source_overlays.clone();
    overlays
        .scene_gizmos
        .extend(build_virtual_geometry_bvh_scene_gizmos(
            &snapshot.bvh_visualization_instances,
        ));
    overlays
        .scene_gizmos
        .extend(build_virtual_geometry_visbuffer_scene_gizmos(
            context,
            &visbuffer_debug_marks,
        ));
    Some(overlays)
}

fn build_virtual_geometry_bvh_scene_gizmos(
    instances: &[RenderVirtualGeometryBvhVisualizationInstance],
) -> Vec<SceneGizmoOverlayExtract> {
    let mut gizmos = Vec::with_capacity(instances.len());
    gizmos.extend(instances.iter().filter_map(|instance| {
        let lines = build_virtual_geometry_bvh_lines(instance);
        (!lines.is_empty()).then(|| SceneGizmoOverlayExtract {
            owner: instance.entity,
            kind: SceneGizmoKind::VirtualGeometryBvh,
            selected: false,
            lines,
            wire_shapes: Vec::new(),
            icons: Vec::new(),
            pick_shapes: Vec::new(),
        })
    }));
    gizmos
}

fn build_virtual_geometry_bvh_lines(
    instance: &RenderVirtualGeometryBvhVisualizationInstance,
) -> Vec<OverlayLineSegment> {
    const MAX_LINES_PER_NODE: usize = 13;

    let nodes_by_id = instance
        .nodes
        .iter()
        .map(|node| (node.node_id, node))
        .collect::<BTreeMap<_, _>>();
    let mut lines = Vec::with_capacity(instance.nodes.len().saturating_mul(MAX_LINES_PER_NODE));

    for node in &instance.nodes {
        let node_color = bvh_node_color(node);
        append_bvh_bounds_wireframe(
            &mut lines,
            Vec3::from_array(node.bounds_center),
            node.bounds_radius,
            node_color,
        );

        if let Some(parent_node_id) = node.parent_node_id {
            if let Some(parent) = nodes_by_id.get(&parent_node_id).copied() {
                lines.push(OverlayLineSegment {
                    start: Vec3::from_array(parent.bounds_center),
                    end: Vec3::from_array(node.bounds_center),
                    color: bvh_connector_color(node),
                });
            }
        }
    }

    lines
}

fn build_virtual_geometry_visbuffer_scene_gizmos(
    context: &FrameSubmissionContext,
    visbuffer_debug_marks: &[RenderVirtualGeometryVisBufferMark],
) -> Vec<SceneGizmoOverlayExtract> {
    let Some(virtual_geometry_extract) = context.virtual_geometry_extract() else {
        return Vec::new();
    };
    let clusters_by_id = virtual_geometry_extract
        .clusters
        .iter()
        .map(|cluster| (cluster.cluster_id, cluster))
        .collect::<BTreeMap<_, _>>();

    let mut gizmos = Vec::with_capacity(visbuffer_debug_marks.len());
    gizmos.extend(visbuffer_debug_marks.iter().filter_map(|mark| {
        let cluster = clusters_by_id.get(&mark.cluster_id).copied()?;
        let lines = build_virtual_geometry_visbuffer_lines(
            cluster.bounds_center,
            cluster.bounds_radius,
            mark,
        );
        (!lines.is_empty()).then(|| SceneGizmoOverlayExtract {
            owner: mark.entity,
            kind: SceneGizmoKind::VirtualGeometryVisBuffer,
            selected: false,
            lines,
            wire_shapes: Vec::new(),
            icons: Vec::new(),
            pick_shapes: Vec::new(),
        })
    }));
    gizmos
}

fn build_current_frame_visbuffer_debug_marks(
    snapshot: &RenderVirtualGeometryDebugSnapshot,
) -> &[RenderVirtualGeometryVisBufferMark] {
    if !snapshot.debug.visualize_visbuffer {
        return &[];
    }

    &snapshot.visbuffer_debug_marks
}

fn build_virtual_geometry_visbuffer_lines(
    center: Vec3,
    radius: f32,
    mark: &RenderVirtualGeometryVisBufferMark,
) -> Vec<OverlayLineSegment> {
    let color = Vec4::new(
        f32::from(mark.color_rgba[0]) / 255.0,
        f32::from(mark.color_rgba[1]) / 255.0,
        f32::from(mark.color_rgba[2]) / 255.0,
        f32::from(mark.color_rgba[3]) / 255.0,
    );
    // Inflate the marker to the cluster bounds scale so it survives the shared
    // depth-tested gizmo pass instead of disappearing inside the source mesh.
    let base_extent = radius.max(0.12);
    let extent = match mark.state {
        RenderVirtualGeometryExecutionState::Resident => base_extent,
        RenderVirtualGeometryExecutionState::PendingUpload => base_extent * 1.15,
        RenderVirtualGeometryExecutionState::Missing => base_extent * 1.3,
    };
    let marker_center = center + Vec3::Y * extent * 1.25;
    let mut lines = Vec::with_capacity(16);
    lines.push(OverlayLineSegment {
        start: center,
        end: marker_center,
        color,
    });
    append_cross_marker(&mut lines, marker_center, extent, color);
    append_bvh_bounds_wireframe(&mut lines, marker_center, extent * 0.95, color);
    lines
}

fn append_bvh_bounds_wireframe(
    lines: &mut Vec<OverlayLineSegment>,
    center: Vec3,
    radius: f32,
    color: Vec4,
) {
    const BOX_EDGES: [(usize, usize); 12] = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    let radius = radius.max(0.025);
    let min = center - Vec3::splat(radius);
    let max = center + Vec3::splat(radius);
    let corners = [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(max.x, max.y, max.z),
        Vec3::new(min.x, max.y, max.z),
    ];

    for (start, end) in BOX_EDGES {
        lines.push(OverlayLineSegment {
            start: corners[start],
            end: corners[end],
            color,
        });
    }
}

fn append_cross_marker(
    lines: &mut Vec<OverlayLineSegment>,
    center: Vec3,
    extent: f32,
    color: Vec4,
) {
    let extent = extent.max(0.025);
    lines.push(OverlayLineSegment {
        start: center - Vec3::new(extent, 0.0, 0.0),
        end: center + Vec3::new(extent, 0.0, 0.0),
        color,
    });
    lines.push(OverlayLineSegment {
        start: center - Vec3::new(0.0, extent, 0.0),
        end: center + Vec3::new(0.0, extent, 0.0),
        color,
    });
    lines.push(OverlayLineSegment {
        start: center - Vec3::new(0.0, 0.0, extent),
        end: center + Vec3::new(0.0, 0.0, extent),
        color,
    });
}

fn bvh_node_color(node: &RenderVirtualGeometryBvhVisualizationNode) -> Vec4 {
    if node.selected_cluster_ids.is_empty() {
        if node.is_leaf {
            Vec4::new(0.35, 0.55, 0.95, 1.0)
        } else {
            Vec4::new(0.25, 0.75, 1.0, 1.0)
        }
    } else if node.selected_cluster_ids.len() == node.resident_cluster_ids.len() {
        Vec4::new(0.2, 1.0, 0.45, 1.0)
    } else if !node.resident_cluster_ids.is_empty() {
        Vec4::new(1.0, 0.85, 0.15, 1.0)
    } else {
        Vec4::new(1.0, 0.35, 0.25, 1.0)
    }
}

fn bvh_connector_color(node: &RenderVirtualGeometryBvhVisualizationNode) -> Vec4 {
    if node.selected_cluster_ids.is_empty() {
        Vec4::new(0.55, 0.65, 0.85, 1.0)
    } else if !node.resident_cluster_ids.is_empty() {
        Vec4::new(1.0, 0.9, 0.3, 1.0)
    } else {
        Vec4::new(1.0, 0.5, 0.35, 1.0)
    }
}

#[cfg(test)]
#[path = "tests/build_runtime_frame.rs"]
mod tests;
