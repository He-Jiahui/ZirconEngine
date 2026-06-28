use crate::core::framework::render::{
    build_mesh_phase_queue, GeometryPhaseInput, MeshPhaseInput, RenderMeshSnapshot,
    RenderPhaseMeshSource, RenderPhaseQueue, RenderQueueValue,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::ViewportRenderFrame;

use super::super::super::mesh_draw::MeshCommandSortInput;

#[derive(Clone, Copy)]
pub(super) struct PhaseOrderedMeshSnapshot<'a> {
    pub(super) snapshot: &'a RenderMeshSnapshot,
    pub(super) command_sort_input: MeshCommandSortInput,
}

pub(super) fn phase_ordered_meshes<'a>(
    frame: &'a ViewportRenderFrame,
    streamer: &ResourceStreamer,
) -> Vec<PhaseOrderedMeshSnapshot<'a>> {
    phase_ordered_meshes_with_material_offsets(frame, |mesh| material_sort_offsets(streamer, mesh))
}

fn phase_ordered_meshes_with_material_offsets<'a>(
    frame: &'a ViewportRenderFrame,
    material_sort_offsets: impl Fn(&RenderMeshSnapshot) -> MaterialPhaseSortOffsets,
) -> Vec<PhaseOrderedMeshSnapshot<'a>> {
    let camera_layers = frame.extract.view.selected_camera_layers();
    let phase_queue = &frame.extract.geometry.phase_queue;
    if phase_queue.items.is_empty() {
        return frame
            .meshes()
            .iter()
            .filter(|mesh| camera_layers.intersects(&mesh.render_layer_mask))
            .map(|mesh| PhaseOrderedMeshSnapshot {
                snapshot: mesh,
                command_sort_input: MeshCommandSortInput::new(
                    mesh.transform.translation.z,
                    mesh.node_id,
                ),
            })
            .collect();
    }

    let material_adjusted_phase_queue =
        material_adjusted_phase_queue(frame, &material_sort_offsets)
            .unwrap_or_else(|| frame.extract.geometry.phase_queue.clone());
    meshes_from_phase_queue(
        frame,
        &material_adjusted_phase_queue,
        &material_sort_offsets,
    )
}

fn meshes_from_phase_queue<'a>(
    frame: &'a ViewportRenderFrame,
    phase_queue: &RenderPhaseQueue,
    material_sort_offsets: &impl Fn(&RenderMeshSnapshot) -> MaterialPhaseSortOffsets,
) -> Vec<PhaseOrderedMeshSnapshot<'a>> {
    let camera_layers = frame.extract.view.selected_camera_layers();
    phase_queue
        .items
        .iter()
        .filter_map(|item| match item.mesh_source {
            RenderPhaseMeshSource::MeshIndex(index) => {
                let snapshot = frame.meshes().get(index)?;
                if !camera_layers.intersects(&snapshot.render_layer_mask) {
                    return None;
                }
                let command_sort_input =
                    command_sort_input_for_mesh_index(frame, index, material_sort_offsets)
                        .unwrap_or_else(|| {
                            MeshCommandSortInput::new(
                                snapshot.transform.translation.z,
                                snapshot.node_id,
                            )
                        });
                Some(PhaseOrderedMeshSnapshot {
                    snapshot,
                    command_sort_input,
                })
            }
            RenderPhaseMeshSource::SpriteIndex(_) => None,
        })
        .collect()
}

fn command_sort_input_for_mesh_index(
    frame: &ViewportRenderFrame,
    mesh_index: usize,
    material_sort_offsets: &impl Fn(&RenderMeshSnapshot) -> MaterialPhaseSortOffsets,
) -> Option<MeshCommandSortInput> {
    let input = frame
        .extract
        .geometry
        .phase_inputs
        .iter()
        .find(|input| input.mesh_index == mesh_index)?;
    let mesh = frame.meshes().get(mesh_index)?;
    let offsets = material_sort_offsets(mesh);
    Some(MeshCommandSortInput {
        depth: input.depth,
        depth_bias: input.depth_bias + offsets.depth_bias,
        queue: material_adjusted_queue(
            &input.material_alpha_mode,
            input.render_queue,
            input.material_queue,
            offsets,
        ),
        camera_order: 0,
        sorting_layer: 0,
        order_in_layer: input.order_in_layer,
        y_sort: None,
        ui_z_index: input.ui_z_index,
        tie_breaker: input.entity,
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct MaterialPhaseSortOffsets {
    queue: Option<RenderQueueValue>,
    render_queue: i32,
    material_queue: i32,
    depth_bias: f32,
}

fn material_sort_offsets(
    streamer: &ResourceStreamer,
    mesh: &RenderMeshSnapshot,
) -> MaterialPhaseSortOffsets {
    streamer
        .material(&mesh.material.id())
        .map(|material| MaterialPhaseSortOffsets {
            queue: material.render_queue_value,
            render_queue: material.render_queue,
            material_queue: material.material_queue,
            depth_bias: material.depth_bias,
        })
        .unwrap_or_default()
}

fn material_adjusted_phase_queue(
    frame: &ViewportRenderFrame,
    material_sort_offsets: &impl Fn(&RenderMeshSnapshot) -> MaterialPhaseSortOffsets,
) -> Option<RenderPhaseQueue> {
    let phase_inputs = frame.extract.geometry.phase_inputs.as_slice();
    (!phase_inputs.is_empty()).then(|| {
        build_mesh_phase_queue(
            frame.extract.view.core_pipeline,
            phase_inputs.iter().map(|input| {
                let offsets = frame
                    .meshes()
                    .get(input.mesh_index)
                    .map(|mesh| material_sort_offsets(mesh))
                    .unwrap_or_default();
                mesh_phase_input_with_material_offsets(input, offsets)
            }),
        )
    })
}

fn mesh_phase_input_with_material_offsets(
    input: &GeometryPhaseInput,
    offsets: MaterialPhaseSortOffsets,
) -> MeshPhaseInput {
    MeshPhaseInput {
        entity: input.entity,
        mesh_index: input.mesh_index,
        queue: material_adjusted_queue(
            &input.material_alpha_mode,
            input.render_queue,
            input.material_queue,
            offsets,
        ),
        depth: input.depth,
        depth_bias: input.depth_bias + offsets.depth_bias,
        camera_order: 0,
        sorting_layer: 0,
        order_in_layer: input.order_in_layer,
        y_sort: None,
        ui_z_index: input.ui_z_index,
    }
}

fn material_adjusted_queue(
    alpha_mode: &crate::core::framework::render::RenderMaterialAlphaMode,
    input_render_queue: i32,
    input_material_queue: i32,
    offsets: MaterialPhaseSortOffsets,
) -> RenderQueueValue {
    let queue = if let Some(queue) = offsets.queue {
        queue.with_material_offset_i32(input_render_queue)
    } else {
        RenderQueueValue::from_authored_queue(
            alpha_mode,
            input_render_queue.saturating_add(offsets.render_queue),
        )
    };
    queue.with_material_offset_i32(input_material_queue.saturating_add(offsets.material_queue))
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        CameraRenderDescriptor, FallbackSkyboxKind, GeometryExtract, GeometryPhaseInput,
        PreviewEnvironmentExtract, ProjectionMode, RenderFrameExtract, RenderLayerSet,
        RenderMaterialAlphaMode, RenderMeshSnapshot, RenderOverlayExtract,
        RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
        ViewportCameraSnapshot,
    };
    use crate::core::framework::scene::Mobility;
    use crate::core::math::{Transform, UVec2, Vec4};
    use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
    use crate::graphics::ViewportRenderFrame;

    use super::{phase_ordered_meshes_with_material_offsets, MaterialPhaseSortOffsets};

    #[test]
    fn phase_ordered_meshes_follow_extract_phase_queue_instead_of_mesh_vector_order() {
        let mut extract = test_extract(vec![test_mesh(30), test_mesh(10), test_mesh(20)]);
        extract.geometry = GeometryExtract::from_meshes_and_phase_inputs(
            extract.view.core_pipeline,
            extract.geometry.meshes.clone(),
            vec![
                GeometryPhaseInput::new(30, 0, RenderMaterialAlphaMode::Blend, 3.0),
                GeometryPhaseInput::new(10, 1, RenderMaterialAlphaMode::Opaque, 1.0),
                GeometryPhaseInput::new(20, 2, RenderMaterialAlphaMode::Mask { cutoff: 0.5 }, 2.0),
            ],
        );
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(320, 240));

        assert_eq!(
            phase_ordered_meshes_with_material_offsets(&frame, |_| {
                MaterialPhaseSortOffsets::default()
            })
            .into_iter()
            .map(|mesh| mesh.snapshot.node_id)
            .collect::<Vec<_>>(),
            vec![10, 20, 30]
        );
    }

    #[test]
    fn phase_ordered_meshes_apply_material_sort_offsets_to_extract_phase_queue() {
        let mut extract = test_extract(vec![test_mesh(10), test_mesh(20), test_mesh(30)]);
        extract.geometry = GeometryExtract::from_meshes_and_phase_inputs(
            extract.view.core_pipeline,
            extract.geometry.meshes.clone(),
            vec![
                GeometryPhaseInput::new(10, 0, RenderMaterialAlphaMode::Opaque, 1.0),
                GeometryPhaseInput::new(20, 1, RenderMaterialAlphaMode::Opaque, 2.0),
                GeometryPhaseInput::new(30, 2, RenderMaterialAlphaMode::Opaque, 3.0),
            ],
        );
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(320, 240));

        assert_eq!(
            phase_ordered_meshes_with_material_offsets(&frame, |mesh| match mesh.node_id {
                20 => MaterialPhaseSortOffsets {
                    queue: None,
                    render_queue: -5,
                    material_queue: 0,
                    depth_bias: 0.0,
                },
                30 => MaterialPhaseSortOffsets {
                    queue: None,
                    render_queue: 0,
                    material_queue: -3,
                    depth_bias: -2.5,
                },
                _ => MaterialPhaseSortOffsets::default(),
            })
            .into_iter()
            .map(|mesh| mesh.snapshot.node_id)
            .collect::<Vec<_>>(),
            vec![20, 30, 10]
        );
    }

    #[test]
    fn phase_ordered_meshes_filter_meshes_by_selected_camera_layers() {
        let mut hidden = test_mesh(10);
        hidden.render_layer_mask = RenderLayerSet::layer(1);
        let mut visible = test_mesh(20);
        visible.render_layer_mask = RenderLayerSet::layer(2);

        let mut fallback_extract =
            test_extract_with_camera_layer(vec![hidden.clone(), visible.clone()], 2);
        let fallback_frame =
            ViewportRenderFrame::from_extract(fallback_extract, UVec2::new(320, 240));
        assert_eq!(
            ordered_node_ids(&fallback_frame),
            vec![20],
            "mesh vector fallback must respect selected camera layers"
        );

        fallback_extract = test_extract_with_camera_layer(vec![hidden, visible], 2);
        fallback_extract.geometry = GeometryExtract::from_meshes_and_phase_inputs(
            fallback_extract.view.core_pipeline,
            fallback_extract.geometry.meshes.clone(),
            vec![
                GeometryPhaseInput::new(10, 0, RenderMaterialAlphaMode::Opaque, 1.0),
                GeometryPhaseInput::new(20, 1, RenderMaterialAlphaMode::Opaque, 2.0),
            ],
        );
        let phase_frame = ViewportRenderFrame::from_extract(fallback_extract, UVec2::new(320, 240));

        assert_eq!(
            ordered_node_ids(&phase_frame),
            vec![20],
            "phase queue path must respect selected camera layers"
        );
    }

    fn ordered_node_ids(frame: &ViewportRenderFrame) -> Vec<u64> {
        phase_ordered_meshes_with_material_offsets(frame, |_| MaterialPhaseSortOffsets::default())
            .into_iter()
            .map(|mesh| mesh.snapshot.node_id)
            .collect()
    }

    fn test_extract(meshes: Vec<RenderMeshSnapshot>) -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(9),
            RenderSceneSnapshot {
                scene: RenderSceneGeometryExtract {
                    camera: ViewportCameraSnapshot::default(),
                    meshes,
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
            },
        )
    }

    fn test_extract_with_camera_layer(
        meshes: Vec<RenderMeshSnapshot>,
        layer: u32,
    ) -> RenderFrameExtract {
        let mut camera = ViewportCameraSnapshot::default();
        camera.projection_mode = ProjectionMode::Perspective;
        let mut descriptor = CameraRenderDescriptor::from_camera_payload(Some(7), camera.clone());
        descriptor.culling_mask = RenderLayerSet::layer(layer);
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(10),
            RenderSceneSnapshot {
                scene: RenderSceneGeometryExtract {
                    camera,
                    meshes,
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
            },
        );
        extract.select_camera_descriptor(descriptor);
        extract
    }

    fn test_mesh(node_id: u64) -> RenderMeshSnapshot {
        RenderMeshSnapshot {
            node_id,
            stable_instance_key: node_id << 16,
            transform_revision: 0,
            transform: Transform::default(),
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(&format!(
                "builtin://test-model/{node_id}"
            ))),
            mesh: None,
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                &format!("builtin://test-material/{node_id}"),
            )),
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        }
    }
}
