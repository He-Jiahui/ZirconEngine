use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderLayerSet,
    RenderMeshSnapshot, RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
    ViewportCameraSnapshot,
};
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};
use crate::scene::components::{default_render_layer_mask, Mobility};

pub(super) fn overlay_mesh(
    node_id: u64,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    layer: u32,
    tint: Vec4,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform: Transform {
            scale: Vec3::new(0.72, 0.72, 1.0),
            ..Transform::default()
        },
        model,
        mesh: None,
        material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint,
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        render_layer_mask: render_layer_set(layer),
    }
}

pub(super) fn sampled_mesh(
    node_id: u64,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    translation: Vec3,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform: Transform {
            translation,
            scale: Vec3::new(0.56, 0.72, 1.0),
            ..Transform::default()
        },
        model,
        mesh: None,
        material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        render_layer_mask: RenderLayerSet::from_legacy_mask(default_render_layer_mask()),
    }
}

pub(super) fn colored_mesh_on_layer(
    node_id: u64,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    transform: Transform,
    tint: Vec4,
    layer: u32,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform,
        model,
        mesh: None,
        material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint,
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        render_layer_mask: render_layer_set(layer),
    }
}

pub(super) fn sampled_fullscreen_mesh(
    node_id: u64,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
) -> RenderMeshSnapshot {
    sampled_fullscreen_mesh_with_mask(
        node_id,
        model,
        material,
        RenderLayerSet::from_legacy_mask(default_render_layer_mask()),
    )
}

pub(super) fn sampled_fullscreen_mesh_on_layer(
    node_id: u64,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    layer: u32,
) -> RenderMeshSnapshot {
    sampled_fullscreen_mesh_with_mask(node_id, model, material, render_layer_set(layer))
}

fn render_layer_set(layer: u32) -> RenderLayerSet {
    RenderLayerSet::layer(layer)
}

fn sampled_fullscreen_mesh_with_mask(
    node_id: u64,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    render_layer_mask: RenderLayerSet,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform: Transform {
            scale: Vec3::new(1.8, 1.8, 1.0),
            ..Transform::default()
        },
        model,
        mesh: None,
        material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        render_layer_mask,
    }
}

pub(super) fn build_snapshot(
    meshes: Vec<RenderMeshSnapshot>,
    viewport_size: UVec2,
) -> RenderSceneSnapshot {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Perspective,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(viewport_size);

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
    }
}
