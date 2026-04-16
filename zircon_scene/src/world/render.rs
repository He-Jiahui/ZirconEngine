use std::collections::BTreeSet;

use zircon_math::Vec4;

use super::World;
use crate::components::{
    aspect_ratio_from_viewport_size, default_render_layer_mask, default_viewport_aspect_ratio,
    DisplayMode, FallbackSkyboxKind, GridMode, GridOverlayExtract, OverlayBillboardIcon,
    OverlayLineSegment, OverlayPickShape, OverlayWireShape, PreviewEnvironmentExtract,
    RenderDirectionalLightSnapshot, RenderMeshSnapshot, RenderOverlayExtract,
    RenderSceneGeometryExtract, RenderSceneSnapshot, SceneGizmoBuildContext, SceneGizmoKind,
    SceneGizmoOverlayExtract, SceneGizmoProvider, SceneGizmoRegistry, SceneViewportExtractRequest,
    SceneViewportRenderPacket, SelectionAnchorExtract, SelectionHighlightExtract,
    ViewportCameraSnapshot, ViewportIconId,
};

const SCENE_CLEAR_COLOR: Vec4 = Vec4::new(0.09, 0.11, 0.14, 1.0);
const SELECTION_TINT: Vec4 = Vec4::new(1.0, 0.92, 0.55, 0.18);
const ANCHOR_COLOR: Vec4 = Vec4::new(1.0, 0.85, 0.3, 1.0);
const CAMERA_GIZMO_COLOR: Vec4 = Vec4::new(0.42, 0.72, 1.0, 1.0);
const LIGHT_GIZMO_COLOR: Vec4 = Vec4::new(1.0, 0.88, 0.36, 1.0);

impl World {
    pub fn to_render_snapshot(&self) -> RenderSceneSnapshot {
        self.to_render_extract()
    }

    pub fn to_render_extract(&self) -> SceneViewportRenderPacket {
        let request = SceneViewportExtractRequest {
            settings: crate::SceneViewportSettings::default(),
            selection: self.selected_entity.into_iter().collect(),
            active_camera_override: None,
            camera: None,
            viewport_size: None,
        };
        self.build_viewport_render_packet(&request)
    }

    pub fn build_viewport_render_packet(
        &self,
        request: &SceneViewportExtractRequest,
    ) -> SceneViewportRenderPacket {
        let camera = self.build_render_camera(request);
        let selection: BTreeSet<_> = request.selection.iter().copied().collect();

        let mut meshes = self
            .mesh_renderers
            .iter()
            .filter(|(entity, _)| self.active_in_hierarchy(**entity) == Some(true))
            .map(|(entity, mesh)| RenderMeshSnapshot {
                node_id: *entity,
                transform: self.world_transform(*entity).unwrap_or_default(),
                model: mesh.model,
                material: mesh.material,
                tint: mesh.tint,
                mobility: self.mobility(*entity).unwrap_or_default(),
                render_layer_mask: self
                    .render_layer_mask(*entity)
                    .unwrap_or(default_render_layer_mask()),
            })
            .collect::<Vec<_>>();
        meshes.sort_by_key(|mesh| mesh.node_id);

        let mut lights = self
            .directional_lights
            .iter()
            .filter(|(entity, _)| self.active_in_hierarchy(**entity) == Some(true))
            .map(|(entity, light)| RenderDirectionalLightSnapshot {
                node_id: *entity,
                direction: light.direction,
                color: light.color,
                intensity: light.intensity,
            })
            .collect::<Vec<_>>();
        lights.sort_by_key(|light| light.node_id);

        SceneViewportRenderPacket {
            scene: RenderSceneGeometryExtract {
                camera,
                meshes,
                lights,
            },
            overlays: RenderOverlayExtract {
                selection: self.build_selection_highlights(&selection, request),
                selection_anchors: self.build_selection_anchors(&selection, request),
                grid: build_grid_extract(request),
                handles: Vec::new(),
                scene_gizmos: self.build_scene_gizmos(&selection, request),
                display_mode: request.settings.display_mode,
            },
            preview: build_preview_environment(request),
        }
    }

    fn build_render_camera(&self, request: &SceneViewportExtractRequest) -> ViewportCameraSnapshot {
        if let Some(mut camera) = request.camera.clone() {
            if let Some(viewport_size) = request.viewport_size {
                camera.apply_viewport_size(viewport_size);
            }
            return camera;
        }

        let entity = request
            .active_camera_override
            .filter(|entity| self.cameras.contains_key(entity))
            .unwrap_or_else(|| {
                if self.cameras.contains_key(&self.active_camera) {
                    self.active_camera
                } else {
                    self.cameras
                        .keys()
                        .copied()
                        .next()
                        .expect("world always contains a camera")
                }
            });
        let component = self
            .cameras
            .get(&entity)
            .expect("camera override must refer to camera entity");
        let transform = self.world_transform(entity).unwrap_or_else(|| {
            self.find_node(entity)
                .map(|node| node.transform)
                .unwrap_or_default()
        });

        ViewportCameraSnapshot {
            transform,
            projection_mode: request.settings.projection_mode,
            fov_y_radians: component.fov_y_radians,
            ortho_size: 5.0,
            z_near: component.z_near,
            z_far: component.z_far,
            aspect_ratio: request
                .viewport_size
                .map(aspect_ratio_from_viewport_size)
                .unwrap_or_else(default_viewport_aspect_ratio),
        }
    }

    fn build_selection_highlights(
        &self,
        selection: &BTreeSet<u64>,
        request: &SceneViewportExtractRequest,
    ) -> Vec<SelectionHighlightExtract> {
        selection
            .iter()
            .copied()
            .filter(|entity| self.mesh_renderers.contains_key(entity))
            .map(|owner| SelectionHighlightExtract {
                owner,
                outline: true,
                tint: match request.settings.display_mode {
                    DisplayMode::WireOnly => None,
                    DisplayMode::Shaded | DisplayMode::WireOverlay => Some(SELECTION_TINT),
                },
            })
            .collect()
    }

    fn build_selection_anchors(
        &self,
        selection: &BTreeSet<u64>,
        request: &SceneViewportExtractRequest,
    ) -> Vec<SelectionAnchorExtract> {
        if request.settings.gizmos_enabled {
            return Vec::new();
        }

        selection
            .iter()
            .copied()
            .filter(|entity| !self.mesh_renderers.contains_key(entity))
            .map(|owner| SelectionAnchorExtract {
                owner,
                position: self.world_transform(owner).unwrap_or_default().translation,
                size: 0.12,
                color: ANCHOR_COLOR,
            })
            .collect()
    }

    fn build_scene_gizmos(
        &self,
        selection: &BTreeSet<u64>,
        request: &SceneViewportExtractRequest,
    ) -> Vec<SceneGizmoOverlayExtract> {
        if !request.settings.gizmos_enabled {
            return Vec::new();
        }

        let registry = build_default_scene_gizmo_registry();
        let mut gizmos = Vec::new();
        for entity in self
            .entities
            .iter()
            .copied()
            .filter(|entity| self.active_in_hierarchy(*entity) == Some(true))
        {
            for provider in &registry.providers {
                if !provider.supports(self, entity) {
                    continue;
                }
                let mut extract = SceneGizmoOverlayExtract {
                    owner: entity,
                    kind: provider.kind(),
                    selected: selection.contains(&entity),
                    lines: Vec::new(),
                    wire_shapes: Vec::new(),
                    icons: Vec::new(),
                    pick_shapes: Vec::new(),
                };
                provider.build(
                    &SceneGizmoBuildContext {
                        world: self,
                        entity,
                        selected: selection.contains(&entity),
                        camera: &request
                            .camera
                            .clone()
                            .unwrap_or_else(|| self.build_render_camera(request)),
                    },
                    &mut extract,
                );
                gizmos.push(extract);
                break;
            }
        }
        gizmos
    }
}

fn build_grid_extract(request: &SceneViewportExtractRequest) -> Option<GridOverlayExtract> {
    match request.settings.grid_mode {
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

fn build_preview_environment(request: &SceneViewportExtractRequest) -> PreviewEnvironmentExtract {
    PreviewEnvironmentExtract {
        lighting_enabled: request.settings.preview_lighting,
        skybox_enabled: request.settings.preview_skybox,
        fallback_skybox: if request.settings.preview_skybox {
            FallbackSkyboxKind::ProceduralGradient
        } else {
            FallbackSkyboxKind::None
        },
        clear_color: SCENE_CLEAR_COLOR,
    }
}

fn build_default_scene_gizmo_registry() -> SceneGizmoRegistry {
    SceneGizmoRegistry::new(vec![
        Box::new(CameraGizmoProvider),
        Box::new(DirectionalLightGizmoProvider),
    ])
}

struct CameraGizmoProvider;

impl SceneGizmoProvider for CameraGizmoProvider {
    fn kind(&self) -> SceneGizmoKind {
        SceneGizmoKind::Camera
    }

    fn supports(&self, world: &World, entity: u64) -> bool {
        world.cameras.contains_key(&entity)
    }

    fn build(&self, ctx: &SceneGizmoBuildContext<'_>, out: &mut SceneGizmoOverlayExtract) {
        let Some(node) = ctx.world.find_node(ctx.entity) else {
            return;
        };
        let Some(camera) = node.camera.as_ref() else {
            return;
        };
        let color = if ctx.selected {
            CAMERA_GIZMO_COLOR * Vec4::new(1.15, 1.15, 1.15, 1.0)
        } else {
            CAMERA_GIZMO_COLOR
        };
        let position = ctx
            .world
            .world_transform(ctx.entity)
            .unwrap_or(node.transform)
            .translation;
        out.icons.push(OverlayBillboardIcon {
            id: ViewportIconId::Camera,
            position,
            tint: color,
            size: 28.0,
        });
        out.wire_shapes.push(OverlayWireShape::Frustum {
            transform: ctx
                .world
                .world_transform(ctx.entity)
                .unwrap_or(node.transform),
            fov_y_radians: camera.fov_y_radians,
            aspect_ratio: ctx.camera.aspect_ratio,
            z_near: camera.z_near.max(0.05),
            z_far: camera.z_far.min(2.5),
            color,
        });
        out.pick_shapes.push(OverlayPickShape::Sphere {
            center: position,
            radius: 0.4,
        });
    }
}

struct DirectionalLightGizmoProvider;

impl SceneGizmoProvider for DirectionalLightGizmoProvider {
    fn kind(&self) -> SceneGizmoKind {
        SceneGizmoKind::DirectionalLight
    }

    fn supports(&self, world: &World, entity: u64) -> bool {
        world.directional_lights.contains_key(&entity)
    }

    fn build(&self, ctx: &SceneGizmoBuildContext<'_>, out: &mut SceneGizmoOverlayExtract) {
        let Some(node) = ctx.world.find_node(ctx.entity) else {
            return;
        };
        let Some(light) = node.directional_light.as_ref() else {
            return;
        };
        let color = if ctx.selected {
            LIGHT_GIZMO_COLOR * Vec4::new(1.1, 1.1, 1.1, 1.0)
        } else {
            LIGHT_GIZMO_COLOR
        };
        let transform = ctx
            .world
            .world_transform(ctx.entity)
            .unwrap_or(node.transform);
        let position = transform.translation;
        let direction = if light.direction.length_squared() > zircon_math::Real::EPSILON {
            light.direction.normalize_or_zero()
        } else {
            transform.forward()
        };
        out.icons.push(OverlayBillboardIcon {
            id: ViewportIconId::DirectionalLight,
            position,
            tint: color,
            size: 28.0,
        });
        out.wire_shapes.push(OverlayWireShape::Arrow {
            origin: position,
            direction,
            length: 1.5,
            color,
        });
        out.lines.push(OverlayLineSegment {
            start: position,
            end: position + direction * 1.5,
            color,
        });
        out.pick_shapes.push(OverlayPickShape::Sphere {
            center: position,
            radius: 0.4,
        });
    }
}
