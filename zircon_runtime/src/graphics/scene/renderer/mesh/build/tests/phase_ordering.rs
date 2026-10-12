use crate::core::framework::render::{
    CameraRenderDescriptor, FallbackSkyboxKind, GeometryExtract, GeometryPhaseInput,
    PreviewEnvironmentExtract, ProjectionMode, RenderFrameExtract, RenderLayerSet,
    RenderMaterialAlphaMode, RenderMeshSnapshot, RenderOverlayExtract, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::graphics::ViewportRenderFrame;

use super::{phase_ordered_meshes_with_material_offsets, MaterialPhaseSortOffsets};

#[test]
fn phase_ordering_indexes_extract_inputs_before_queue_projection() {
    let source = include_str!("../phase_ordering.rs");
    let product = source
        .split("#[cfg(test)]")
        .next()
        .expect("product source precedes tests");

    assert!(product.contains("phase_inputs_by_mesh_index"));
    assert!(!product.contains(".phase_inputs\n        .iter()\n        .find("));
}

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
    )
    .into();
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
    )
    .into();
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(320, 240));

    assert_eq!(
        phase_ordered_meshes_with_material_offsets(&frame, |mesh| match mesh.node_id {
            20 => MaterialPhaseSortOffsets {
                queue: None,
                fixed_queue: false,
                render_queue: -5,
                material_queue: 0,
                depth_bias: 0.0,
            },
            30 => MaterialPhaseSortOffsets {
                queue: None,
                fixed_queue: false,
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
    hidden.common.layer_mask = RenderLayerSet::layer(1);
    let mut visible = test_mesh(20);
    visible.common.layer_mask = RenderLayerSet::layer(2);

    let mut fallback_extract =
        test_extract_with_camera_layer(vec![hidden.clone(), visible.clone()], 2);
    let fallback_frame = ViewportRenderFrame::from_extract(fallback_extract, UVec2::new(320, 240));
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
    )
    .into();
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
            environment: crate::core::framework::render::EnvironmentExtract::default(),
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
            environment: crate::core::framework::render::EnvironmentExtract::default(),
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
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(&format!(
            "builtin://test-material/{node_id}"
        ))),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            ..Default::default()
        },
    }
}
