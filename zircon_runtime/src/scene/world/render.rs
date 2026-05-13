use crate::core::framework::render::{
    aspect_ratio_from_viewport_size, default_viewport_aspect_ratio, DebugOverlayExtract,
    FallbackSkyboxKind, GeometryExtract, GeometryPhaseInput, LightingExtract, ParticleExtract,
    PostProcessExtract, PreviewEnvironmentExtract, RenderBloomSettings, RenderColorGradingSettings,
    RenderDirectionalLightSnapshot, RenderFrameExtract, RenderHybridGiExtract, RenderMeshSnapshot,
    RenderOverlayExtract, RenderPointLightSnapshot, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderSpotLightSnapshot, RenderViewExtract, RenderVirtualGeometryExtract,
    RenderWorldSnapshotHandle, SceneViewportExtractRequest, SceneViewportRenderPacket,
    ViewportCameraSnapshot, VisibilityInput, VisibilityRenderableInput,
};
use crate::core::math::Vec4;

use super::World;
use crate::scene::components::default_render_layer_mask;

const SCENE_CLEAR_COLOR: Vec4 = Vec4::new(0.09, 0.11, 0.14, 1.0);

impl World {
    pub fn to_render_snapshot(&self) -> RenderSceneSnapshot {
        self.to_render_extract()
    }

    pub fn to_render_extract(&self) -> SceneViewportRenderPacket {
        let request = SceneViewportExtractRequest {
            active_camera_override: None,
            camera: None,
            ..SceneViewportExtractRequest::default()
        };
        self.build_viewport_render_packet(&request)
    }

    pub fn build_viewport_render_packet(
        &self,
        request: &SceneViewportExtractRequest,
    ) -> SceneViewportRenderPacket {
        let mut world = self.clone();
        world.build_prepared_viewport_render_packet(request)
    }

    pub(crate) fn build_prepared_viewport_render_packet(
        &mut self,
        request: &SceneViewportExtractRequest,
    ) -> SceneViewportRenderPacket {
        self.run_internal_scene_systems_for_stage(crate::scene::SystemStage::RenderExtract);
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

        let mut directional_lights = self
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
        directional_lights.sort_by_key(|light| light.node_id);

        let mut point_lights = self
            .point_lights
            .iter()
            .filter(|(entity, _)| self.active_in_hierarchy(**entity) == Some(true))
            .map(|(entity, light)| RenderPointLightSnapshot {
                node_id: *entity,
                position: self
                    .world_transform(*entity)
                    .unwrap_or_default()
                    .translation,
                color: light.color,
                intensity: light.intensity,
                range: light.range,
            })
            .collect::<Vec<_>>();
        point_lights.sort_by_key(|light| light.node_id);

        let mut spot_lights = self
            .spot_lights
            .iter()
            .filter(|(entity, _)| self.active_in_hierarchy(**entity) == Some(true))
            .map(|(entity, light)| RenderSpotLightSnapshot {
                node_id: *entity,
                position: self
                    .world_transform(*entity)
                    .unwrap_or_default()
                    .translation,
                direction: light.direction,
                color: light.color,
                intensity: light.intensity,
                range: light.range,
                inner_angle_radians: light.inner_angle_radians,
                outer_angle_radians: light.outer_angle_radians,
            })
            .collect::<Vec<_>>();
        spot_lights.sort_by_key(|light| light.node_id);

        SceneViewportRenderPacket {
            scene: RenderSceneGeometryExtract {
                camera,
                meshes,
                directional_lights,
                point_lights,
                spot_lights,
            },
            overlays: RenderOverlayExtract {
                display_mode: request.settings.display_mode,
                ..RenderOverlayExtract::default()
            },
            preview: build_preview_environment(request),
            virtual_geometry_debug: request.virtual_geometry_debug,
        }
    }

    pub(crate) fn build_prepared_render_frame_extract_for_request(
        &mut self,
        world: RenderWorldSnapshotHandle,
        request: &SceneViewportExtractRequest,
    ) -> RenderFrameExtract {
        self.run_internal_scene_systems_for_stage(crate::scene::SystemStage::RenderExtract);
        let camera = self.build_render_camera(request);
        let (meshes, phase_inputs) = self.collect_render_meshes_and_phase_inputs();
        let directional_lights = self.collect_directional_lights();
        let point_lights = self.collect_point_lights();
        let spot_lights = self.collect_spot_lights();
        let visibility = build_visibility_input(&meshes);

        RenderFrameExtract {
            world,
            view: RenderViewExtract::from_camera(camera.clone()),
            geometry: {
                let mut geometry = GeometryExtract::from_meshes_and_phase_inputs(
                    camera.core_pipeline_kind(),
                    meshes,
                    phase_inputs,
                );
                geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
                    debug: request.virtual_geometry_debug.unwrap_or_default(),
                    ..RenderVirtualGeometryExtract::default()
                });
                geometry.virtual_geometry_debug = request.virtual_geometry_debug;
                geometry
            },
            animation_poses: Vec::new(),
            lighting: LightingExtract {
                directional_lights,
                point_lights,
                spot_lights,
                reflection_probes: Vec::new(),
                baked_lighting: None,
                hybrid_global_illumination: Some(RenderHybridGiExtract::default()),
            },
            post_process: PostProcessExtract {
                preview: build_preview_environment(request),
                display_mode: request.settings.display_mode,
                bloom: RenderBloomSettings::default(),
                color_grading: RenderColorGradingSettings::default(),
            },
            debug: DebugOverlayExtract {
                overlays: RenderOverlayExtract {
                    display_mode: request.settings.display_mode,
                    ..RenderOverlayExtract::default()
                },
            },
            particles: ParticleExtract::default(),
            visibility,
        }
    }

    fn collect_render_meshes_and_phase_inputs(
        &self,
    ) -> (Vec<RenderMeshSnapshot>, Vec<GeometryPhaseInput>) {
        let mut mesh_entries = self
            .mesh_renderers
            .iter()
            .filter(|(entity, _)| self.active_in_hierarchy(**entity) == Some(true))
            .map(|(entity, mesh)| {
                let snapshot = RenderMeshSnapshot {
                    node_id: *entity,
                    transform: self.world_transform(*entity).unwrap_or_default(),
                    model: mesh.model,
                    material: mesh.material,
                    tint: mesh.tint,
                    mobility: self.mobility(*entity).unwrap_or_default(),
                    render_layer_mask: self
                        .render_layer_mask(*entity)
                        .unwrap_or(default_render_layer_mask()),
                };
                (snapshot, mesh.material_alpha_mode)
            })
            .collect::<Vec<_>>();
        mesh_entries.sort_by_key(|(mesh, _)| mesh.node_id);

        let meshes = mesh_entries
            .iter()
            .map(|(mesh, _)| mesh.clone())
            .collect::<Vec<_>>();
        let phase_inputs = mesh_entries
            .iter()
            .enumerate()
            .map(|(mesh_index, (mesh, material_alpha_mode))| {
                GeometryPhaseInput::new(
                    mesh.node_id,
                    mesh_index,
                    *material_alpha_mode,
                    mesh.transform.translation.z,
                )
            })
            .collect::<Vec<_>>();

        (meshes, phase_inputs)
    }

    fn collect_directional_lights(&self) -> Vec<RenderDirectionalLightSnapshot> {
        let mut directional_lights = self
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
        directional_lights.sort_by_key(|light| light.node_id);
        directional_lights
    }

    fn collect_point_lights(&self) -> Vec<RenderPointLightSnapshot> {
        let mut point_lights = self
            .point_lights
            .iter()
            .filter(|(entity, _)| self.active_in_hierarchy(**entity) == Some(true))
            .map(|(entity, light)| RenderPointLightSnapshot {
                node_id: *entity,
                position: self
                    .world_transform(*entity)
                    .unwrap_or_default()
                    .translation,
                color: light.color,
                intensity: light.intensity,
                range: light.range,
            })
            .collect::<Vec<_>>();
        point_lights.sort_by_key(|light| light.node_id);
        point_lights
    }

    fn collect_spot_lights(&self) -> Vec<RenderSpotLightSnapshot> {
        let mut spot_lights = self
            .spot_lights
            .iter()
            .filter(|(entity, _)| self.active_in_hierarchy(**entity) == Some(true))
            .map(|(entity, light)| RenderSpotLightSnapshot {
                node_id: *entity,
                position: self
                    .world_transform(*entity)
                    .unwrap_or_default()
                    .translation,
                direction: light.direction,
                color: light.color,
                intensity: light.intensity,
                range: light.range,
                inner_angle_radians: light.inner_angle_radians,
                outer_angle_radians: light.outer_angle_radians,
            })
            .collect::<Vec<_>>();
        spot_lights.sort_by_key(|light| light.node_id);
        spot_lights
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

fn build_visibility_input(meshes: &[RenderMeshSnapshot]) -> VisibilityInput {
    let renderables = meshes
        .iter()
        .map(|mesh| VisibilityRenderableInput {
            entity: mesh.node_id,
            mobility: mesh.mobility,
            render_layer_mask: mesh.render_layer_mask,
        })
        .collect::<Vec<_>>();
    let renderable_entities = renderables
        .iter()
        .map(|entry| entry.entity)
        .collect::<Vec<_>>();
    let static_entities = renderables
        .iter()
        .filter(|entry| entry.mobility == crate::scene::components::Mobility::Static)
        .map(|entry| entry.entity)
        .collect::<Vec<_>>();
    let dynamic_entities = renderables
        .iter()
        .filter(|entry| entry.mobility == crate::scene::components::Mobility::Dynamic)
        .map(|entry| entry.entity)
        .collect::<Vec<_>>();

    VisibilityInput {
        renderable_entities,
        static_entities,
        dynamic_entities,
        renderables,
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
