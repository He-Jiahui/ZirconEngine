use std::collections::BTreeMap;

use crate::core::framework::render::{
    OverlayLineSegment, RenderFrameExtract, RenderVirtualGeometryBvhVisualizationInstance,
    RenderVirtualGeometryBvhVisualizationNode, RenderVirtualGeometryDebugSnapshot,
    RenderVirtualGeometryExecutionState, RenderVirtualGeometryVisBufferMark, SceneGizmoKind,
    SceneGizmoOverlayExtract,
};
use crate::core::math::{Vec3, Vec4};

use crate::graphics::{ViewportCameraStackOutputPolicy, ViewportRenderFrame};
use zircon_runtime_interface::ui::surface::UiRenderExtract;

use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;
use super::build_virtual_geometry_debug_snapshot::build_virtual_geometry_debug_snapshot;

pub(super) fn build_runtime_frame(
    extract: RenderFrameExtract,
    ui: Option<UiRenderExtract>,
    context: &FrameSubmissionContext,
    prepared: &PreparedRuntimeSubmission,
    output_policy: ViewportCameraStackOutputPolicy,
) -> ViewportRenderFrame {
    let extract = apply_effective_advanced_extracts(extract, context);
    let extract = apply_effective_post_process_graph(extract, context);
    let extract = apply_effective_particle_previous_state(extract, context);
    let extract = apply_submission_target_size(extract, context);
    let virtual_geometry_debug_snapshot = build_virtual_geometry_debug_snapshot(&extract, context);
    let extract = augment_virtual_geometry_debug_overlays(
        extract,
        context,
        virtual_geometry_debug_snapshot.as_ref(),
    );
    ViewportRenderFrame::from_extract(extract, context.size())
        .with_shader_quality(context.shader_quality())
        .with_output_target(context.output_target())
        .with_camera_stack_output_policy(output_policy)
        .with_ui(ui)
        .with_frame_visibility(context.visibility_context().frame_visibility.clone())
        .with_previous_motion_vector_camera(context.previous_motion_vector_camera().cloned())
        .with_prepared_runtime_sidebands(prepared.prepared_runtime_sidebands())
        .with_virtual_geometry_debug_snapshot(virtual_geometry_debug_snapshot)
}

fn apply_effective_particle_previous_state(
    mut extract: RenderFrameExtract,
    context: &FrameSubmissionContext,
) -> RenderFrameExtract {
    extract.particles.previous_sprites = context.particle_previous_sprites().to_vec();
    extract
}

fn apply_submission_target_size(
    mut extract: RenderFrameExtract,
    context: &FrameSubmissionContext,
) -> RenderFrameExtract {
    extract.apply_viewport_size(context.size());
    extract
}

fn apply_effective_post_process_graph(
    mut extract: RenderFrameExtract,
    context: &FrameSubmissionContext,
) -> RenderFrameExtract {
    extract.post_process.bloom = context.post_process_bloom();
    extract.post_process.exposure = context.post_process_exposure();
    extract.post_process.color_grading = context.post_process_color_grading();
    extract.post_process.effect_stack = context.post_process_effect_stack();
    extract.post_process.volumes.clear();
    extract.view.anti_alias = context.anti_alias_fallback().effective_settings();
    extract.view.camera.temporal_jitter = context.temporal_jitter();
    extract.view.sync_selected_descriptor_camera_payload();
    extract.post_process.stack = context.post_process_stack().clone();
    extract.post_process.graph = context.post_process_graph().clone();
    extract
}

fn apply_effective_advanced_extracts(
    mut extract: RenderFrameExtract,
    context: &FrameSubmissionContext,
) -> RenderFrameExtract {
    extract.geometry.virtual_geometry = context.virtual_geometry_extract().cloned();
    if !context.hybrid_gi_enabled() {
        extract.lighting.hybrid_global_illumination = None;
    }
    extract
}

fn augment_virtual_geometry_debug_overlays(
    mut extract: RenderFrameExtract,
    context: &FrameSubmissionContext,
    snapshot: Option<&RenderVirtualGeometryDebugSnapshot>,
) -> RenderFrameExtract {
    let Some(snapshot) = snapshot else {
        return extract;
    };
    let visbuffer_debug_marks = build_current_frame_visbuffer_debug_marks(snapshot);
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
    let Some(virtual_geometry_extract) = context.virtual_geometry_extract() else {
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
) -> Vec<RenderVirtualGeometryVisBufferMark> {
    if !snapshot.debug.visualize_visbuffer {
        return Vec::new();
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
    use super::*;
    use crate::core::framework::render::{
        AdvancedProfileRuntimePlan, AdvancedProviderAvailability, AntiAliasFallbackReport,
        AntiAliasMode, FallbackSkyboxKind, PreviewEnvironmentExtract, RenderCameraTargetKind,
        RenderCapabilitySummary, RenderMeshSnapshot, RenderOverlayExtract,
        RenderParticlePreviousSpriteSnapshot, RenderPipelineHandle, RenderPluginRendererOutputs,
        RenderProfileBundle, RenderVirtualGeometryNodeClusterCullReadbackOutputs,
        RenderVirtualGeometryReadbackOutputs, SceneViewportRenderPacket, TemporalJitterSample,
        ViewportCameraSnapshot,
    };
    use crate::core::math::{Transform, UVec2, Vec3, Vec4};
    use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
    use crate::graphics::types::{ViewportTextureWritebackStatus, FRAMEWORK_OUTPUT_FORMAT_LABEL};
    use crate::graphics::VisibilityContext;
    use crate::graphics::{CompiledRenderPipeline, RenderPassStage};
    use crate::render_graph::RenderGraphBuilder;

    #[test]
    fn build_runtime_frame_carries_prepared_sideband_and_output_target_into_viewport_frame() {
        let extract = RenderFrameExtract::from_snapshot(
            crate::core::framework::render::RenderWorldSnapshotHandle::new(9),
            empty_scene_snapshot(),
        );
        let previous_camera = ViewportCameraSnapshot {
            transform: Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
            ..ViewportCameraSnapshot::default()
        };
        let output_texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
            "tests/runtime-frame/output-target",
        ));
        let particle_previous_sprites = vec![RenderParticlePreviousSpriteSnapshot {
            entity: 88,
            stable_sprite_key: 5,
            position: Vec3::new(-1.0, 0.0, -2.0),
            size: 0.5,
            aspect_ratio: 1.0,
            billboard_offset: crate::core::math::Vec2::ZERO,
            rotation: 0.0,
            billboard_basis: None,
        }];
        let context = FrameSubmissionContext::new(
            UVec2::new(640, 480),
            UVec2::new(640, 480),
            RenderPipelineHandle::new(1),
            0,
            None,
            Default::default(),
            std::sync::Arc::new(empty_pipeline()),
            RenderCapabilitySummary::default(),
            VisibilityContext::from_extract(&extract),
            Some(previous_camera.clone()),
            super::super::super::super::viewport_record::ViewportCameraHistoryKey::from_camera(
                extract
                    .view
                    .selected_camera_descriptor()
                    .expect("test extract has selected camera descriptor"),
            ),
            Default::default(),
            None,
            crate::graphics::ViewportRenderOutputTarget::Texture {
                handle: output_texture,
                size: UVec2::new(640, 480),
            },
            Default::default(),
            None,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            AntiAliasFallbackReport::exact(AntiAliasMode::Taa),
            1,
            advanced_runtime_plan_with_virtual_geometry(),
            Default::default(),
            Default::default(),
            Default::default(),
            false,
            true,
            None,
            Default::default(),
            None,
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            particle_previous_sprites.clone(),
            0,
            0,
            0,
            None,
            Default::default(),
            Vec::new(),
            Vec::new(),
            None,
            None,
            1,
        );
        let prepared = PreparedRuntimeSubmission::new(
            vec![5],
            vec![9],
            RenderPluginRendererOutputs {
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                        page_request_ids: vec![300],
                        ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                    },
                    ..RenderVirtualGeometryReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            },
        );

        let frame = build_runtime_frame(
            extract,
            None,
            &context,
            &prepared,
            ViewportCameraStackOutputPolicy::new(false, false),
        );

        assert_eq!(frame.viewport_size, UVec2::new(640, 480));
        assert_eq!(
            frame.output_target().kind(),
            RenderCameraTargetKind::Texture
        );
        assert_eq!(frame.output_target().texture_handle(), Some(output_texture));
        assert_eq!(frame.output_target().size(), Some(UVec2::new(640, 480)));
        assert_eq!(
            frame
                .texture_writeback_plan(Some(FRAMEWORK_OUTPUT_FORMAT_LABEL))
                .status(),
            ViewportTextureWritebackStatus::ReadyForSrgbCopy
        );
        assert!(!frame
            .camera_stack_output_policy()
            .owns_final_target_output());
        assert_eq!(
            frame.previous_motion_vector_camera(),
            Some(&previous_camera)
        );
        assert_eq!(
            frame
                .prepared_runtime_sidebands()
                .virtual_geometry_readback_outputs()
                .node_cluster_cull
                .page_request_ids,
            vec![300]
        );
        assert_eq!(
            frame
                .prepared_runtime_sidebands()
                .virtual_geometry_evictable_page_ids(),
            &[9]
        );
        assert_eq!(frame.extract.view.anti_alias.mode, AntiAliasMode::Taa);
        assert_eq!(
            frame.extract.particles.previous_sprites,
            particle_previous_sprites
        );
        assert_ne!(
            frame.extract.view.camera.temporal_jitter,
            TemporalJitterSample::default()
        );
        assert_ne!(
            frame.camera().camera.temporal_jitter,
            TemporalJitterSample::default()
        );
        assert_ne!(
            frame.effective_camera().temporal_jitter,
            TemporalJitterSample::default()
        );
    }

    fn empty_pipeline() -> CompiledRenderPipeline {
        let graph = RenderGraphBuilder::new("empty-runtime-frame-test")
            .compile()
            .unwrap();
        CompiledRenderPipeline {
            handle: RenderPipelineHandle::new(1),
            name: "empty".to_string(),
            renderer_name: "empty".to_string(),
            stages: vec![RenderPassStage::Opaque3d],
            pass_stages: Vec::new(),
            enabled_features: Vec::new(),
            required_extract_sections: Vec::new(),
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            graph,
        }
    }

    fn advanced_runtime_plan_with_virtual_geometry() -> AdvancedProfileRuntimePlan {
        AdvancedProfileRuntimePlan::from_profile_bundle(
            &RenderProfileBundle::advanced_render(),
            &RenderCapabilitySummary {
                virtual_geometry_supported: true,
                hybrid_global_illumination_supported: true,
                supports_storage_buffers: true,
                supports_indirect_draw: true,
                supports_buffer_readback: true,
                ..RenderCapabilitySummary::default()
            },
            &AdvancedProviderAvailability::new()
                .with_virtual_geometry_provider("vg")
                .with_hybrid_gi_provider("hgi"),
        )
    }

    fn empty_scene_snapshot() -> SceneViewportRenderPacket {
        SceneViewportRenderPacket {
            scene: crate::core::framework::render::RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        }
    }

    fn test_mesh(node_id: u64, transform: Transform) -> RenderMeshSnapshot {
        RenderMeshSnapshot {
            node_id,
            stable_instance_key: node_id << 16,
            transform_revision: 0,
            transform,
            model: ResourceHandle::new(ResourceId::from_stable_label("tests/model")),
            mesh: None,
            material: ResourceHandle::new(ResourceId::from_stable_label("tests/material")),
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: crate::core::framework::scene::Mobility::Dynamic,
            static_state: Default::default(),
            render_layer_mask: 1,
        }
    }
}
