use zircon_math::Vec4;

use super::World;
use crate::components::{
    aspect_ratio_from_viewport_size, default_render_layer_mask, default_viewport_aspect_ratio,
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderDirectionalLightSnapshot,
    RenderMeshSnapshot, RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
    SceneViewportExtractRequest, SceneViewportRenderPacket, ViewportCameraSnapshot,
};

const SCENE_CLEAR_COLOR: Vec4 = Vec4::new(0.09, 0.11, 0.14, 1.0);

impl World {
    pub fn to_render_snapshot(&self) -> RenderSceneSnapshot {
        self.to_render_extract()
    }

    pub fn to_render_extract(&self) -> SceneViewportRenderPacket {
        let request = SceneViewportExtractRequest {
            settings: crate::SceneViewportSettings::default(),
            selection: Vec::new(),
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
                display_mode: request.settings.display_mode,
                ..RenderOverlayExtract::default()
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
