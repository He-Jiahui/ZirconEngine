use std::collections::BTreeMap;

use crate::core::framework::render::{
    OverlayLineSegment, RenderFrameExtract, RenderVirtualGeometryBvhVisualizationInstance,
    RenderVirtualGeometryBvhVisualizationNode, RenderVirtualGeometryDebugSnapshot,
    RenderVirtualGeometryExecutionState, RenderVirtualGeometryExtract,
    RenderVirtualGeometryVisBufferMark, SceneGizmoKind, SceneGizmoOverlayExtract,
};
use crate::core::math::{Vec3, Vec4};

use crate::graphics::types::VirtualGeometryPrepareFrame;
use crate::graphics::ViewportRenderFrame;
use crate::ui::surface::UiRenderExtract;

use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;
use super::build_virtual_geometry_debug_snapshot::build_virtual_geometry_debug_snapshot;

pub(super) fn build_runtime_frame(
    extract: RenderFrameExtract,
    ui: Option<UiRenderExtract>,
    context: &FrameSubmissionContext,
    prepared: &PreparedRuntimeSubmission,
) -> ViewportRenderFrame {
    let virtual_geometry_debug_snapshot =
        build_virtual_geometry_debug_snapshot(context, prepared.virtual_geometry_prepare.as_ref());
    let extract = augment_virtual_geometry_debug_overlays(
        extract,
        context,
        prepared.virtual_geometry_prepare.as_ref(),
        virtual_geometry_debug_snapshot.as_ref(),
    );
    ViewportRenderFrame::from_extract(extract, context.size)
        .with_ui(ui)
        .with_virtual_geometry_debug_snapshot(virtual_geometry_debug_snapshot)
        .with_hybrid_gi_prepare(prepared.hybrid_gi_prepare.clone())
        .with_hybrid_gi_scene_prepare(prepared.hybrid_gi_scene_prepare.clone())
        .with_hybrid_gi_resolve_runtime(prepared.hybrid_gi_resolve_runtime.clone())
        .with_prepare_derived_virtual_geometry_cluster_selections(
            prepared
                .virtual_geometry_prepare
                .as_ref()
                .and_then(|prepare| {
                    context
                        .virtual_geometry_extract
                        .as_ref()
                        .map(|extract| prepare.cluster_selections(extract))
                }),
        )
        .with_virtual_geometry_prepare(prepared.virtual_geometry_prepare.clone())
}

fn augment_virtual_geometry_debug_overlays(
    mut extract: RenderFrameExtract,
    context: &FrameSubmissionContext,
    prepare: Option<&VirtualGeometryPrepareFrame>,
    snapshot: Option<&RenderVirtualGeometryDebugSnapshot>,
) -> RenderFrameExtract {
    let Some(snapshot) = snapshot else {
        return extract;
    };
    let visbuffer_debug_marks = build_current_frame_visbuffer_debug_marks(
        snapshot,
        context.virtual_geometry_extract.as_ref(),
        prepare,
    );
    if snapshot.bvh_visualization_instances.is_empty() && visbuffer_debug_marks.is_empty() {
        return extract;
    }

    extract
        .debug
        .overlays
        .scene_gizmos
        .extend(build_virtual_geometry_bvh_scene_gizmos(
            &snapshot.bvh_visualization_instances,
        ));
    extract
        .debug
        .overlays
        .scene_gizmos
        .extend(build_virtual_geometry_visbuffer_scene_gizmos(
            context,
            &visbuffer_debug_marks,
        ));
    extract
}

fn build_virtual_geometry_bvh_scene_gizmos(
    instances: &[RenderVirtualGeometryBvhVisualizationInstance],
) -> Vec<SceneGizmoOverlayExtract> {
    instances
        .iter()
        .filter_map(|instance| {
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
        })
        .collect()
}

fn build_virtual_geometry_bvh_lines(
    instance: &RenderVirtualGeometryBvhVisualizationInstance,
) -> Vec<OverlayLineSegment> {
    let nodes_by_id = instance
        .nodes
        .iter()
        .map(|node| (node.node_id, node))
        .collect::<BTreeMap<_, _>>();
    let mut lines = Vec::new();

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
    let Some(virtual_geometry_extract) = context.virtual_geometry_extract.as_ref() else {
        return Vec::new();
    };
    let clusters_by_id = virtual_geometry_extract
        .clusters
        .iter()
        .map(|cluster| (cluster.cluster_id, cluster))
        .collect::<BTreeMap<_, _>>();

    visbuffer_debug_marks
        .iter()
        .filter_map(|mark| {
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
        })
        .collect()
}

fn build_current_frame_visbuffer_debug_marks(
    snapshot: &RenderVirtualGeometryDebugSnapshot,
    extract: Option<&RenderVirtualGeometryExtract>,
    prepare: Option<&VirtualGeometryPrepareFrame>,
) -> Vec<RenderVirtualGeometryVisBufferMark> {
    if !snapshot.debug.visualize_visbuffer {
        return Vec::new();
    }

    if let (Some(extract), Some(prepare)) = (extract, prepare) {
        let marks = prepare.same_frame_visbuffer_debug_marks(extract);
        if !marks.is_empty() {
            return marks;
        }
    }

    snapshot.visbuffer_debug_marks.clone()
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
    let mut lines = Vec::new();
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
mod tests {
    use super::build_current_frame_visbuffer_debug_marks;
    use crate::core::framework::render::{
        RenderVirtualGeometryCluster, RenderVirtualGeometryDebugSnapshot,
        RenderVirtualGeometryDebugState, RenderVirtualGeometryExtract,
        RenderVirtualGeometryInstance, RenderVirtualGeometryVisBufferMark,
    };
    use crate::core::math::{Transform, Vec3};
    use crate::graphics::types::{VirtualGeometryPrepareFrame, VirtualGeometryPreparePage};

    #[test]
    fn same_frame_visbuffer_marks_follow_prepare_unified_draw_fallback_when_snapshot_marks_are_empty(
    ) {
        let entity = 101_u64;
        let extract = RenderVirtualGeometryExtract {
            cluster_budget: 2,
            page_budget: 1,
            clusters: vec![
                RenderVirtualGeometryCluster {
                    entity,
                    cluster_id: 1,
                    page_id: 10,
                    lod_level: 10,
                    parent_cluster_id: None,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.5,
                    screen_space_error: 0.25,
                },
                RenderVirtualGeometryCluster {
                    entity,
                    cluster_id: 2,
                    page_id: 20,
                    lod_level: 10,
                    parent_cluster_id: Some(1),
                    bounds_center: Vec3::X,
                    bounds_radius: 0.5,
                    screen_space_error: 0.2,
                },
            ],
            pages: Vec::new(),
            instances: vec![RenderVirtualGeometryInstance {
                entity,
                source_model: None,
                transform: Transform::default(),
                cluster_offset: 0,
                cluster_count: 2,
                page_offset: 0,
                page_count: 0,
                mesh_name: Some("OverlayUnitTest".to_string()),
                source_hint: Some("unit-test".to_string()),
            }],
            debug: RenderVirtualGeometryDebugState {
                visualize_visbuffer: true,
                ..RenderVirtualGeometryDebugState::default()
            },
        };
        let snapshot = RenderVirtualGeometryDebugSnapshot {
            instances: extract.instances.clone(),
            debug: extract.debug,
            ..RenderVirtualGeometryDebugSnapshot::default()
        };
        let prepare = VirtualGeometryPrepareFrame {
            visible_entities: vec![entity],
            visible_clusters: Vec::new(),
            cluster_draw_segments: Vec::new(),
            resident_pages: vec![VirtualGeometryPreparePage {
                page_id: 10,
                slot: 0,
                size_bytes: 4096,
            }],
            pending_page_requests: Vec::new(),
            available_slots: Vec::new(),
            evictable_pages: Vec::new(),
        };

        let marks =
            build_current_frame_visbuffer_debug_marks(&snapshot, Some(&extract), Some(&prepare));

        assert_eq!(
            marks,
            vec![RenderVirtualGeometryVisBufferMark {
                instance_index: Some(0),
                entity,
                cluster_id: 1,
                page_id: 10,
                lod_level: 10,
                state:
                    crate::core::framework::render::RenderVirtualGeometryExecutionState::Resident,
                color_rgba: [179, 212, 35, 255],
            }],
            "expected same-frame visbuffer overlays to follow the prepare fallback draw list even before the stored snapshot is backfilled from execution segments"
        );
    }

    #[test]
    fn same_frame_visbuffer_marks_stay_empty_when_visualization_is_disabled() {
        let entity = 101_u64;
        let extract = RenderVirtualGeometryExtract {
            cluster_budget: 1,
            page_budget: 1,
            clusters: vec![RenderVirtualGeometryCluster {
                entity,
                cluster_id: 1,
                page_id: 10,
                lod_level: 10,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                screen_space_error: 0.25,
            }],
            pages: Vec::new(),
            instances: vec![RenderVirtualGeometryInstance {
                entity,
                source_model: None,
                transform: Transform::default(),
                cluster_offset: 0,
                cluster_count: 1,
                page_offset: 0,
                page_count: 0,
                mesh_name: Some("OverlayUnitTest".to_string()),
                source_hint: Some("unit-test".to_string()),
            }],
            debug: RenderVirtualGeometryDebugState::default(),
        };
        let snapshot = RenderVirtualGeometryDebugSnapshot {
            instances: extract.instances.clone(),
            debug: extract.debug,
            ..RenderVirtualGeometryDebugSnapshot::default()
        };
        let prepare = VirtualGeometryPrepareFrame {
            visible_entities: vec![entity],
            visible_clusters: Vec::new(),
            cluster_draw_segments: Vec::new(),
            resident_pages: vec![VirtualGeometryPreparePage {
                page_id: 10,
                slot: 0,
                size_bytes: 4096,
            }],
            pending_page_requests: Vec::new(),
            available_slots: Vec::new(),
            evictable_pages: Vec::new(),
        };

        let marks =
            build_current_frame_visbuffer_debug_marks(&snapshot, Some(&extract), Some(&prepare));

        assert!(
            marks.is_empty(),
            "expected same-frame visbuffer overlays to stay disabled when visualize_visbuffer is false"
        );
    }
}
