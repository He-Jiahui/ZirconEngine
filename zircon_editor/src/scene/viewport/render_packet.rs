use crate::scene::viewport::{
    FallbackSkyboxKind, GridMode, GridOverlayExtract, OverlayBillboardIcon, OverlayLineSegment,
    OverlayPickShape, OverlayWireShape, PreviewEnvironmentExtract, RenderOverlayExtract,
    RenderSceneSnapshot, SceneGizmoKind, SceneGizmoOverlayExtract, SceneViewportExtractRequest,
    SceneViewportSettings, SelectionAnchorExtract, ViewportCameraSnapshot, ViewportIconId,
};
use zircon_runtime::scene::components::{NodeKind, SceneNode};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::{Real, UVec2, Vec4};

const SCENE_CLEAR_COLOR: Vec4 = Vec4::new(0.09, 0.11, 0.14, 1.0);
const ANCHOR_COLOR: Vec4 = Vec4::new(1.0, 0.85, 0.3, 1.0);
const CAMERA_GIZMO_COLOR: Vec4 = Vec4::new(0.42, 0.72, 1.0, 1.0);
const LIGHT_GIZMO_COLOR: Vec4 = Vec4::new(1.0, 0.88, 0.36, 1.0);

pub(in crate::scene::viewport) fn build_render_packet(
    scene: &Scene,
    settings: &SceneViewportSettings,
    camera: &ViewportCameraSnapshot,
    selected: Option<u64>,
    viewport_size: UVec2,
) -> RenderSceneSnapshot {
    let mut packet = scene.build_viewport_render_packet(&SceneViewportExtractRequest {
        settings: settings.render_settings(),
        active_camera_override: None,
        camera: Some(camera.clone().into()),
        viewport_size: Some(viewport_size),
        virtual_geometry_debug: None,
    });
    packet.overlays = RenderOverlayExtract {
        // Runtime10 projects the gateway-owned HighlightSet during runtime frame production.
        highlights: None,
        selection_anchors: build_selection_anchors(scene, selected, settings),
        grid: build_grid_extract(settings),
        handles: Vec::new(),
        scene_gizmos: Vec::new(),
        display_mode: settings.display_mode,
    };
    packet.preview = PreviewEnvironmentExtract {
        lighting_enabled: settings.preview_lighting,
        skybox_enabled: settings.preview_skybox,
        fallback_skybox: if settings.preview_skybox {
            FallbackSkyboxKind::ProceduralGradient
        } else {
            FallbackSkyboxKind::None
        },
        clear_color: SCENE_CLEAR_COLOR,
    };
    packet
}

pub(in crate::scene::viewport) fn apply_interaction_overlays(
    packet: &mut RenderSceneSnapshot,
    interaction: &crate::scene::viewport::ViewportInteractionExtract,
) {
    packet.overlays.handles = interaction.handles().iter().cloned().collect();
    packet.overlays.scene_gizmos = interaction.scene_gizmos().iter().cloned().collect();
}

pub(in crate::scene::viewport) fn build_scene_gizmos(
    scene: &Scene,
    selected: Option<u64>,
    settings: &SceneViewportSettings,
    camera: &ViewportCameraSnapshot,
) -> Vec<SceneGizmoOverlayExtract> {
    if !settings.gizmos_enabled {
        return Vec::new();
    }

    let mut gizmos = Vec::new();
    for node in scene.nodes() {
        if !matches!(node.kind, NodeKind::Camera | NodeKind::DirectionalLight) {
            continue;
        }

        let entity = node.id;
        if scene.active_in_hierarchy(entity) != Some(true) {
            continue;
        }

        let is_selected = selected == Some(entity);
        let gizmo = match node.kind {
            NodeKind::Camera => build_camera_gizmo(scene, node, is_selected, camera),
            NodeKind::DirectionalLight => {
                build_directional_light_gizmo(scene, node, is_selected, camera)
            }
            NodeKind::Empty
            | NodeKind::Cube
            | NodeKind::Mesh
            | NodeKind::AmbientLight
            | NodeKind::PointLight
            | NodeKind::RectLight
            | NodeKind::SpotLight => None,
        };

        if let Some(gizmo) = gizmo {
            gizmos.push(gizmo);
        }
    }
    gizmos
}

fn build_selection_anchors(
    scene: &Scene,
    selected: Option<u64>,
    settings: &SceneViewportSettings,
) -> Vec<SelectionAnchorExtract> {
    if settings.gizmos_enabled {
        return Vec::new();
    }

    selected
        .into_iter()
        .filter(|entity| {
            scene
                .find_node(*entity)
                .is_some_and(|node| node.mesh.is_none())
        })
        .filter_map(|owner| {
            scene
                .world_transform(owner)
                .map(|transform| SelectionAnchorExtract {
                    owner,
                    position: transform.translation,
                    size: 0.12,
                    color: ANCHOR_COLOR,
                })
        })
        .collect()
}

fn build_grid_extract(settings: &SceneViewportSettings) -> Option<GridOverlayExtract> {
    match settings.grid_mode {
        GridMode::Hidden => None,
        GridMode::VisibleNoSnap => Some(GridOverlayExtract {
            visible: true,
            snap_enabled: false,
        }),
        GridMode::VisibleAndSnap => Some(GridOverlayExtract {
            visible: true,
            snap_enabled: true,
        }),
    }
}

fn build_camera_gizmo(
    scene: &Scene,
    node: &SceneNode,
    selected: bool,
    camera: &ViewportCameraSnapshot,
) -> Option<SceneGizmoOverlayExtract> {
    let entity = node.id;
    let camera_component = node.camera.as_ref()?;
    let color = if selected {
        CAMERA_GIZMO_COLOR * Vec4::new(1.15, 1.15, 1.15, 1.0)
    } else {
        CAMERA_GIZMO_COLOR
    };
    let transform = scene.world_transform(entity).unwrap_or(node.transform);
    let position = transform.translation;
    Some(SceneGizmoOverlayExtract {
        owner: entity,
        kind: SceneGizmoKind::Camera,
        selected,
        lines: Vec::new(),
        wire_shapes: vec![OverlayWireShape::Frustum {
            transform,
            fov_y_radians: camera_component.fov_y_radians,
            aspect_ratio: camera.aspect_ratio,
            z_near: camera_component.z_near.max(0.05),
            z_far: camera_component.z_far.min(2.5),
            color,
        }],
        icons: vec![OverlayBillboardIcon {
            id: ViewportIconId::Camera,
            position,
            tint: color,
            size: 28.0,
        }],
        pick_shapes: vec![OverlayPickShape::Sphere {
            center: position,
            radius: 0.4,
        }],
    })
}

fn build_directional_light_gizmo(
    scene: &Scene,
    node: &SceneNode,
    selected: bool,
    _camera: &ViewportCameraSnapshot,
) -> Option<SceneGizmoOverlayExtract> {
    let entity = node.id;
    let light = node.directional_light.as_ref()?;
    let color = if selected {
        LIGHT_GIZMO_COLOR * Vec4::new(1.1, 1.1, 1.1, 1.0)
    } else {
        LIGHT_GIZMO_COLOR
    };
    let transform = scene.world_transform(entity).unwrap_or(node.transform);
    let position = transform.translation;
    let direction = if light.direction.length_squared() > Real::EPSILON {
        light.direction.normalize_or_zero()
    } else {
        transform.forward()
    };
    Some(SceneGizmoOverlayExtract {
        owner: entity,
        kind: SceneGizmoKind::DirectionalLight,
        selected,
        lines: vec![OverlayLineSegment {
            start: position,
            end: position + direction * 1.5,
            color,
        }],
        wire_shapes: vec![OverlayWireShape::Arrow {
            origin: position,
            direction,
            length: 1.5,
            color,
        }],
        icons: vec![OverlayBillboardIcon {
            id: ViewportIconId::DirectionalLight,
            position,
            tint: color,
            size: 28.0,
        }],
        pick_shapes: vec![OverlayPickShape::Sphere {
            center: position,
            radius: 0.4,
        }],
    })
}
