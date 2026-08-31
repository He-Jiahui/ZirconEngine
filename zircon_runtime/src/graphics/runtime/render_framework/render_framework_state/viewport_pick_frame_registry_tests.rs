use crate::core::framework::render::{
    render_mesh_stable_instance_key, EnvironmentExtract, FallbackSkyboxKind,
    PreviewEnvironmentExtract, RenderFrameExtract, RenderMeshSnapshot, RenderOverlayExtract,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderViewportHandle,
    RenderViewportPickPolicy, RenderViewportPickPurpose, RenderViewportPickRequest,
    RenderWorldSnapshotHandle, RendererCommon, ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::graphics::visibility::{FrameVisibility, ViewVisibilityContext, VisibilityViewKey};
use crate::graphics::ViewportRenderFrame;

use super::viewport_pick_frame_registry::{ViewportPickFrameRegistry, ViewportPickFrameSnapshot};

#[test]
fn pick_frame_registry_retains_three_exact_generations_per_viewport() {
    let viewport = RenderViewportHandle::new(7);
    let mut registry = ViewportPickFrameRegistry::default();

    for generation in 1..=4 {
        registry.publish(snapshot(viewport, generation));
    }

    assert!(registry.resolve(viewport, 1).is_none());
    assert_eq!(registry.resolve(viewport, 2).unwrap().generation(), 2);
    assert_eq!(registry.resolve(viewport, 3).unwrap().generation(), 3);
    assert_eq!(registry.resolve(viewport, 4).unwrap().generation(), 4);
    assert!(registry.resolve(viewport, 5).is_none());
}

#[test]
fn pick_frame_snapshot_matches_full_presented_request_identity() {
    let viewport = RenderViewportHandle::new(11);
    let snapshot = snapshot(viewport, 23);
    let first_instance = render_mesh_stable_instance_key(1, 0);
    let hidden_instance = render_mesh_stable_instance_key(2, 0);
    let third_instance = render_mesh_stable_instance_key(3, 7);
    let request = RenderViewportPickRequest::new(
        viewport,
        UVec2::new(320, 180),
        UVec2::new(17, 19),
        23,
        29,
        RenderViewportPickPurpose::Press,
        RenderViewportPickPolicy::default(),
    );

    assert!(snapshot.matches_request(request));
    assert_eq!(snapshot.world_generation(), 23);
    assert_eq!(
        snapshot.visible_stable_instance_keys(),
        &[first_instance, third_instance]
    );
    assert_eq!(snapshot.hit_proxy_token_for_instance(hidden_instance), None);
    let first_token = snapshot
        .hit_proxy_token_for_instance(first_instance)
        .expect("visible primitive token");
    let first_hit = snapshot
        .resolve_hit_proxy_token(first_token)
        .expect("visible primitive identity");
    assert_eq!(first_hit.entity, 1);
    assert_eq!(first_hit.instance, first_instance);
    assert_eq!(first_hit.subobject, 0);
    let third_hit = snapshot
        .resolve_hit_proxy_token(
            snapshot
                .hit_proxy_token_for_instance(third_instance)
                .expect("visible primitive token"),
        )
        .expect("visible primitive identity");
    assert_eq!(third_hit.entity, 3);
    assert_eq!(third_hit.instance, third_instance);
    assert_eq!(third_hit.subobject, 7);
    assert!(snapshot.resolve_hit_proxy_token(0).is_none());

    let mut wrong_size = request;
    wrong_size.viewport_size.x += 1;
    assert!(!snapshot.matches_request(wrong_size));
}

#[test]
fn pick_frame_snapshot_retains_the_rendered_virtual_geometry_decision() {
    let snapshot = snapshot_with_virtual_geometry(RenderViewportHandle::new(13), 31, true);

    assert!(snapshot.virtual_geometry_enabled());
}

fn snapshot(viewport: RenderViewportHandle, generation: u64) -> ViewportPickFrameSnapshot {
    snapshot_with_virtual_geometry(viewport, generation, false)
}

fn snapshot_with_virtual_geometry(
    viewport: RenderViewportHandle,
    generation: u64,
    virtual_geometry_enabled: bool,
) -> ViewportPickFrameSnapshot {
    let first_instance = render_mesh_stable_instance_key(1, 0);
    let hidden_instance = render_mesh_stable_instance_key(2, 0);
    let third_instance = render_mesh_stable_instance_key(3, 7);
    let frame = ViewportRenderFrame::from_extract(
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(5).with_generation(generation),
            scene_snapshot_with_meshes(vec![
                mesh_snapshot(1, first_instance),
                mesh_snapshot(2, hidden_instance),
                mesh_snapshot(3, third_instance),
            ]),
        ),
        UVec2::new(320, 180),
    )
    .with_frame_visibility(FrameVisibility {
        entities: vec![1, 2, 3],
        stable_instance_keys: vec![first_instance, hidden_instance, third_instance],
        views: vec![ViewVisibilityContext {
            view: VisibilityViewKey::MainCamera,
            visible: vec![0, 2],
            ..ViewVisibilityContext::default()
        }],
        ..FrameVisibility::default()
    });
    ViewportPickFrameSnapshot::from_rendered_frame(
        viewport,
        generation,
        &frame,
        virtual_geometry_enabled,
    )
}

fn scene_snapshot_with_meshes(meshes: Vec<RenderMeshSnapshot>) -> RenderSceneSnapshot {
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
        environment: EnvironmentExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    }
}

fn mesh_snapshot(entity: u64, stable_instance_key: u64) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: entity,
        stable_instance_key,
        transform_revision: 0,
        transform: Transform::default(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("pick/model")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "pick/material",
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        common: RendererCommon::default(),
    }
}
