use crate::core::framework::render::{
    default_viewport_aspect_ratio, DebugOverlayExtract, FallbackSkyboxKind, GeometryExtract,
    GeometryPhaseInput, LightingExtract, ParticleExtract, PostProcessExtract,
    PreviewEnvironmentExtract, ProjectionMode, RenderAmbientLightSnapshot, RenderBloomSettings,
    RenderColorGradingSettings, RenderDirectionalLightSnapshot, RenderFrameExtract,
    RenderHybridGiExtract, RenderLayerSet, RenderMeshSnapshot, RenderOverlayExtract,
    RenderPointLightSnapshot, RenderRectLightSnapshot, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderSpotLightSnapshot, RenderSpriteSnapshot, RenderViewExtract,
    RenderVirtualGeometryExtract, RenderWorldSnapshotHandle, SceneViewportExtractRequest,
    SceneViewportRenderPacket, SpriteExtract, ViewportCameraSnapshot, VisibilityInput,
    VisibilityRenderableInput,
};
use crate::core::math::Vec4;

use super::World;
use crate::scene::components::{default_render_layer_mask, MeshRenderer, Sprite2dComponent};

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
        if !camera.is_active {
            return SceneViewportRenderPacket {
                scene: empty_scene_geometry(camera),
                overlays: RenderOverlayExtract {
                    display_mode: request.settings.display_mode,
                    ..RenderOverlayExtract::default()
                },
                preview: build_preview_environment(request),
                virtual_geometry_debug: request.virtual_geometry_debug,
            };
        }

        let camera_layers = camera.render_layers.clone();
        let mut meshes = self
            .mesh_renderers
            .iter()
            .filter_map(|(entity, mesh)| {
                self.render_mesh_snapshot_for_camera(*entity, mesh, &camera_layers)
            })
            .collect::<Vec<_>>();
        meshes.sort_by_key(|mesh| mesh.node_id);

        let ambient_lights = self.collect_ambient_lights();
        let directional_lights = self.collect_directional_lights();
        let point_lights = self.collect_point_lights();
        let rect_lights = self.collect_rect_lights();
        let spot_lights = self.collect_spot_lights();

        SceneViewportRenderPacket {
            scene: RenderSceneGeometryExtract {
                camera,
                meshes,
                directional_lights,
                point_lights,
                spot_lights,
                ambient_lights,
                rect_lights,
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
        if !camera.is_active {
            return inactive_camera_frame_extract(world, camera, request);
        }
        let (meshes, phase_inputs) =
            self.collect_render_meshes_and_phase_inputs(&camera.render_layers);
        let sprites = self.collect_render_sprites(&camera.render_layers);
        let ambient_lights = self.collect_ambient_lights();
        let directional_lights = self.collect_directional_lights();
        let point_lights = self.collect_point_lights();
        let rect_lights = self.collect_rect_lights();
        let spot_lights = self.collect_spot_lights();
        let visibility = build_visibility_input(&meshes, &sprites);

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
                ambient_lights,
                rect_lights,
                reflection_probes: Vec::new(),
                baked_lighting: None,
                hybrid_global_illumination: Some(RenderHybridGiExtract::default()),
            },
            post_process: PostProcessExtract::from_parts(
                build_preview_environment(request),
                request.settings.display_mode,
                RenderBloomSettings::default(),
                RenderColorGradingSettings::default(),
                false,
                false,
            ),
            debug: DebugOverlayExtract {
                overlays: RenderOverlayExtract {
                    display_mode: request.settings.display_mode,
                    ..RenderOverlayExtract::default()
                },
            },
            sprites: SpriteExtract::from_sprites(camera.core_pipeline_kind(), sprites),
            particles: ParticleExtract::default(),
            visibility,
        }
    }

    fn collect_render_meshes_and_phase_inputs(
        &self,
        camera_layers: &RenderLayerSet,
    ) -> (Vec<RenderMeshSnapshot>, Vec<GeometryPhaseInput>) {
        let mut mesh_entries = self
            .mesh_renderers
            .iter()
            .filter_map(|(entity, mesh)| {
                self.render_mesh_snapshot_for_camera(*entity, mesh, camera_layers)
                    .map(|snapshot| (snapshot, mesh.material_alpha_mode))
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

    fn render_mesh_snapshot_for_camera(
        &self,
        entity: crate::scene::EntityId,
        mesh: &MeshRenderer,
        camera_layers: &RenderLayerSet,
    ) -> Option<RenderMeshSnapshot> {
        if self.active_in_hierarchy(entity) != Some(true) {
            return None;
        }
        let render_layer_mask = self
            .render_layer_mask(entity)
            .unwrap_or(default_render_layer_mask());
        if !camera_layers.intersects_legacy_mask(render_layer_mask) {
            return None;
        }

        Some(RenderMeshSnapshot {
            node_id: entity,
            transform: self.world_transform(entity).unwrap_or_default(),
            model: mesh.model,
            material: mesh.material,
            tint: mesh.tint,
            mobility: self.mobility(entity).unwrap_or_default(),
            render_layer_mask,
        })
    }

    fn collect_render_sprites(&self, camera_layers: &RenderLayerSet) -> Vec<RenderSpriteSnapshot> {
        let mut sprites = self
            .sprite_2d
            .iter()
            .filter_map(|(entity, sprite)| {
                self.render_sprite_snapshot_for_camera(*entity, sprite, camera_layers)
            })
            .collect::<Vec<_>>();
        sprites.sort_by_key(|sprite| (sprite.z_order, sprite.entity));
        sprites
    }

    fn render_sprite_snapshot_for_camera(
        &self,
        entity: crate::scene::EntityId,
        sprite: &Sprite2dComponent,
        camera_layers: &RenderLayerSet,
    ) -> Option<RenderSpriteSnapshot> {
        if self.active_in_hierarchy(entity) != Some(true) {
            return None;
        }
        let render_layer_mask = self
            .render_layer_mask(entity)
            .unwrap_or(default_render_layer_mask());
        if !camera_layers.intersects_legacy_mask(render_layer_mask) {
            return None;
        }

        Some(RenderSpriteSnapshot {
            entity,
            transform: self.world_transform(entity).unwrap_or_default(),
            image: sprite.image,
            material: sprite.material,
            atlas_region: sprite.atlas_region,
            rect: sprite.rect,
            flip_x: sprite.flip_x,
            flip_y: sprite.flip_y,
            anchor: sprite.anchor,
            custom_size: sprite.custom_size,
            color: sprite.color,
            z_order: sprite.z_order,
            render_layer_mask,
            material_alpha_mode: sprite.material_alpha_mode,
        })
    }

    fn collect_ambient_lights(&self) -> Vec<RenderAmbientLightSnapshot> {
        let mut ambient_lights = self
            .ambient_lights
            .iter()
            .filter(|(entity, _)| self.active_in_hierarchy(**entity) == Some(true))
            .map(|(entity, light)| {
                (
                    *entity,
                    RenderAmbientLightSnapshot {
                        color: light.color,
                        intensity: light.intensity,
                        renderer_degraded: false,
                        degradation_reason: None,
                    },
                )
            })
            .collect::<Vec<_>>();
        ambient_lights.sort_by_key(|(entity, _)| *entity);
        ambient_lights.into_iter().map(|(_, light)| light).collect()
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

    fn collect_rect_lights(&self) -> Vec<RenderRectLightSnapshot> {
        let mut rect_lights = self
            .rect_lights
            .iter()
            .filter(|(entity, _)| self.active_in_hierarchy(**entity) == Some(true))
            .map(|(entity, light)| {
                let transform = self.world_transform(*entity).unwrap_or_default();
                RenderRectLightSnapshot {
                    node_id: *entity,
                    position: transform.translation,
                    direction: transform.forward(),
                    color: light.color,
                    intensity: light.intensity,
                    range: light.range,
                    size: light.size,
                    renderer_degraded: true,
                    degradation_reason: Some(
                        "rect light renderer shading is not implemented yet".to_string(),
                    ),
                }
            })
            .collect::<Vec<_>>();
        rect_lights.sort_by_key(|light| light.node_id);
        rect_lights
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
        let projection_mode = if request.settings.projection_mode == ProjectionMode::default() {
            component.projection_mode
        } else {
            request.settings.projection_mode
        };

        let mut camera = ViewportCameraSnapshot {
            transform,
            projection_mode,
            fov_y_radians: component.fov_y_radians,
            ortho_size: component.ortho_size,
            z_near: component.z_near,
            z_far: component.z_far,
            aspect_ratio: default_viewport_aspect_ratio(),
            target: component.target.clone(),
            viewport: component.viewport,
            order: component.order,
            is_active: component.is_active,
            hdr: component.hdr,
            exposure_ev100: component.exposure_ev100,
            clear_color: component.clear_color,
            msaa_samples: component.msaa_samples,
            render_layers: RenderLayerSet::from_legacy_mask(
                self.render_layer_mask(entity)
                    .unwrap_or(default_render_layer_mask()),
            ),
            ..ViewportCameraSnapshot::default()
        };
        if let Some(viewport_size) = request.viewport_size {
            camera.apply_viewport_size(viewport_size);
        } else if let crate::core::framework::render::RenderCameraTarget::Headless { size } =
            &camera.target
        {
            camera.apply_viewport_size(*size);
        }
        camera
    }
}

fn empty_scene_geometry(camera: ViewportCameraSnapshot) -> RenderSceneGeometryExtract {
    RenderSceneGeometryExtract {
        camera,
        meshes: Vec::new(),
        directional_lights: Vec::new(),
        point_lights: Vec::new(),
        spot_lights: Vec::new(),
        ambient_lights: Vec::new(),
        rect_lights: Vec::new(),
    }
}

fn inactive_camera_frame_extract(
    world: RenderWorldSnapshotHandle,
    camera: ViewportCameraSnapshot,
    request: &SceneViewportExtractRequest,
) -> RenderFrameExtract {
    let mut geometry = GeometryExtract::from_meshes_and_phase_inputs(
        camera.core_pipeline_kind(),
        Vec::new(),
        Vec::new(),
    );
    geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        debug: request.virtual_geometry_debug.unwrap_or_default(),
        ..RenderVirtualGeometryExtract::default()
    });
    geometry.virtual_geometry_debug = request.virtual_geometry_debug;

    RenderFrameExtract {
        world,
        view: RenderViewExtract::from_camera(camera),
        geometry,
        animation_poses: Vec::new(),
        lighting: LightingExtract {
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
            reflection_probes: Vec::new(),
            baked_lighting: None,
            hybrid_global_illumination: Some(RenderHybridGiExtract::default()),
        },
        post_process: PostProcessExtract::from_parts(
            build_preview_environment(request),
            request.settings.display_mode,
            RenderBloomSettings::default(),
            RenderColorGradingSettings::default(),
            false,
            false,
        ),
        debug: DebugOverlayExtract {
            overlays: RenderOverlayExtract {
                display_mode: request.settings.display_mode,
                ..RenderOverlayExtract::default()
            },
        },
        sprites: SpriteExtract::default(),
        particles: ParticleExtract::default(),
        visibility: empty_visibility_input(),
    }
}

fn build_visibility_input(
    meshes: &[RenderMeshSnapshot],
    sprites: &[RenderSpriteSnapshot],
) -> VisibilityInput {
    let mut renderables = meshes
        .iter()
        .map(|mesh| VisibilityRenderableInput {
            entity: mesh.node_id,
            mobility: mesh.mobility,
            render_layer_mask: mesh.render_layer_mask,
        })
        .chain(sprites.iter().map(|sprite| VisibilityRenderableInput {
            entity: sprite.entity,
            mobility: crate::scene::components::Mobility::Dynamic,
            render_layer_mask: sprite.render_layer_mask,
        }))
        .collect::<Vec<_>>();
    renderables.sort_by_key(|entry| entry.entity);
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

fn empty_visibility_input() -> VisibilityInput {
    VisibilityInput {
        renderable_entities: Vec::new(),
        static_entities: Vec::new(),
        dynamic_entities: Vec::new(),
        renderables: Vec::new(),
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
