use crate::core::framework::render::{
    default_viewport_aspect_ratio, render_mesh_stable_instance_key, render_mesh_transform_revision,
    sort_render_cameras, CameraRenderDescriptor, DebugOverlayExtract, FallbackSkyboxKind,
    GeometryExtract, GeometryPhaseInput, LightingExtract, ParticleExtract, PostProcessExtract,
    PostProcessVolumeExtract, PreviewEnvironmentExtract, ProjectionMode,
    RenderAmbientLightSnapshot, RenderCameraOrderInput, RenderCameraOrderReport,
    RenderDirectionalLightSnapshot, RenderExposureSettings, RenderFrameExtract,
    RenderHybridGiExtract, RenderLayerSet, RenderMeshLodSelection, RenderMeshSnapshot,
    RenderMeshStaticState, RenderOverlayExtract, RenderPointLightSnapshot, RenderRectLightSnapshot,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderSpotLightSnapshot, RenderSpriteSnapshot,
    RenderViewExtract, RenderVirtualGeometryExtract, RenderWorldSnapshotHandle,
    SceneViewportExtractRequest, SceneViewportRenderPacket, SpriteExtract, ViewportCameraSnapshot,
    VisibilityInput, VisibilityRenderableInput,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, Vec3, Vec4};

use super::World;
use crate::scene::components::{
    default_render_layer_mask, MeshRenderer, MeshRendererLodLevel, MeshRendererPrimitiveBinding,
    PostProcessSettingsComponent, Sprite2dComponent,
};

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

    pub fn render_camera_order_report(&self) -> RenderCameraOrderReport {
        sort_render_cameras(
            self.scene_camera_descriptors()
                .into_iter()
                .filter_map(|camera| {
                    camera
                        .entity
                        .map(|entity| RenderCameraOrderInput::from_descriptor(entity, camera))
                }),
        )
    }

    pub(crate) fn build_prepared_viewport_render_packet(
        &mut self,
        request: &SceneViewportExtractRequest,
    ) -> SceneViewportRenderPacket {
        self.run_internal_scene_systems_for_stage(crate::scene::SystemStage::RenderExtract);
        let (camera_descriptor, _) = self.build_render_camera(request);
        let camera = camera_descriptor.camera.clone();
        if !camera_descriptor.is_active() {
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

        let camera_layers = camera_descriptor.culling_mask.clone();
        let camera_position = camera.transform.translation;
        let mut meshes = self
            .mesh_renderers
            .iter()
            .flat_map(|(entity, mesh)| {
                self.render_mesh_snapshots_for_camera(
                    *entity,
                    mesh,
                    &camera_layers,
                    camera_position,
                )
            })
            .collect::<Vec<_>>();
        meshes.sort_by_key(|mesh| mesh.node_id);

        let ambient_lights = self.collect_ambient_lights(&camera_layers);
        let directional_lights = self.collect_directional_lights(&camera_layers);
        let point_lights = self.collect_point_lights(&camera_layers);
        let rect_lights = self.collect_rect_lights(&camera_layers);
        let spot_lights = self.collect_spot_lights(&camera_layers);

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
        let (camera_descriptor, scene_camera_entity) = self.build_render_camera(request);
        let core_pipeline = camera_descriptor.camera.core_pipeline_kind();
        let camera_layers = camera_descriptor.culling_mask.clone();
        let view = self.build_render_view_extract(camera_descriptor, scene_camera_entity);
        let extract_layers = self.render_extract_layers_for_view(&view);
        if !view.camera.is_active {
            return inactive_camera_frame_extract(world, view, request);
        }
        let (meshes, phase_inputs) = self.collect_render_meshes_and_phase_inputs(
            &extract_layers,
            view.camera.transform.translation,
        );
        let sprites = self.collect_render_sprites(&extract_layers);
        let particles =
            self.collect_render_particles(&camera_layers, view.camera.transform.translation);
        let ambient_lights = self.collect_ambient_lights(&camera_layers);
        let directional_lights = self.collect_directional_lights(&camera_layers);
        let point_lights = self.collect_point_lights(&camera_layers);
        let rect_lights = self.collect_rect_lights(&camera_layers);
        let spot_lights = self.collect_spot_lights(&camera_layers);
        let visibility = build_visibility_input(&meshes, &sprites, &particles);

        let post_process_settings = scene_camera_entity
            .and_then(|entity| self.post_process_settings.get(&entity))
            .cloned()
            .unwrap_or_default();
        let post_process_volumes =
            self.collect_post_process_volumes(&camera_layers, view.camera.transform.translation);
        let camera_exposure_ev100 = view.camera.exposure_ev100;

        RenderFrameExtract {
            world,
            view,
            geometry: {
                let mut geometry = GeometryExtract::from_meshes_and_phase_inputs(
                    core_pipeline,
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
            post_process: build_post_process_extract(
                request,
                camera_exposure_ev100,
                post_process_settings,
                post_process_volumes.extracts,
            ),
            debug: DebugOverlayExtract {
                overlays: RenderOverlayExtract {
                    display_mode: request.settings.display_mode,
                    ..RenderOverlayExtract::default()
                },
            },
            sprites: SpriteExtract::from_sprites(core_pipeline, sprites),
            particles,
            visibility,
        }
    }

    fn collect_render_meshes_and_phase_inputs(
        &self,
        camera_layers: &RenderLayerSet,
        camera_position: Vec3,
    ) -> (Vec<RenderMeshSnapshot>, Vec<GeometryPhaseInput>) {
        let mut mesh_entries = self
            .mesh_renderers
            .iter()
            .flat_map(|(entity, mesh)| {
                self.render_mesh_snapshots_for_camera(*entity, mesh, camera_layers, camera_position)
                    .into_iter()
                    .map(|snapshot| {
                        (
                            snapshot,
                            mesh.material_alpha_mode,
                            mesh.render_queue,
                            mesh.material_queue,
                            mesh.order_in_layer,
                            mesh.depth_bias,
                        )
                    })
            })
            .collect::<Vec<_>>();
        mesh_entries.sort_by_key(|(mesh, ..)| mesh.node_id);

        let meshes = mesh_entries
            .iter()
            .map(|(mesh, ..)| (*mesh).clone())
            .collect::<Vec<_>>();
        let phase_inputs = mesh_entries
            .iter()
            .enumerate()
            .map(|(mesh_index, entry)| {
                let (
                    mesh,
                    material_alpha_mode,
                    render_queue,
                    material_queue,
                    order_in_layer,
                    depth_bias,
                ) = entry;
                GeometryPhaseInput::new(
                    mesh.node_id,
                    mesh_index,
                    *material_alpha_mode,
                    mesh.transform.translation.z,
                )
                .with_render_queue(*render_queue)
                .with_material_queue(*material_queue)
                .with_order_in_layer(*order_in_layer)
                .with_depth_bias(*depth_bias)
            })
            .collect::<Vec<_>>();

        (meshes, phase_inputs)
    }

    fn render_mesh_snapshots_for_camera(
        &self,
        entity: crate::scene::EntityId,
        mesh: &MeshRenderer,
        camera_layers: &RenderLayerSet,
        camera_position: Vec3,
    ) -> Vec<RenderMeshSnapshot> {
        if self.active_in_hierarchy(entity) != Some(true) {
            return Vec::new();
        }
        let render_layer_mask = self
            .render_layer_mask(entity)
            .unwrap_or(default_render_layer_mask());
        if !camera_layers.intersects_legacy_mask(render_layer_mask) {
            return Vec::new();
        }

        let transform = self.world_transform(entity).unwrap_or_default();
        let mobility = self.mobility(entity).unwrap_or_default();
        let static_state =
            RenderMeshStaticState::from_transform_static(mobility == Mobility::Static);
        let source = mesh_render_source_for_camera(mesh, transform, camera_position);
        if !source.primitives.is_empty() {
            return source
                .primitives
                .iter()
                .enumerate()
                .map(|(primitive_ordinal, primitive)| RenderMeshSnapshot {
                    node_id: entity,
                    stable_instance_key: render_mesh_stable_instance_key(
                        entity,
                        primitive_ordinal as u32,
                    ),
                    transform_revision: render_mesh_transform_revision(&transform),
                    transform,
                    model: source.model,
                    mesh: Some(primitive.mesh),
                    material: primitive.material,
                    mesh_lod: source.mesh_lod,
                    morph_weights: mesh.morph_weights.clone(),
                    tint: mesh.tint,
                    mobility,
                    static_state,
                    render_layer_mask,
                })
                .collect();
        }

        vec![RenderMeshSnapshot {
            node_id: entity,
            stable_instance_key: render_mesh_stable_instance_key(entity, 0),
            transform_revision: render_mesh_transform_revision(&transform),
            transform,
            model: source.model,
            mesh: source.mesh,
            material: source.material,
            mesh_lod: source.mesh_lod,
            morph_weights: mesh.morph_weights.clone(),
            tint: mesh.tint,
            mobility,
            static_state,
            render_layer_mask,
        }]
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
            image_mode: sprite.image_mode,
            color: sprite.color,
            z_order: sprite.z_order,
            render_layer_mask,
            material_alpha_mode: sprite.material_alpha_mode,
        })
    }

    fn collect_ambient_lights(
        &self,
        camera_layers: &RenderLayerSet,
    ) -> Vec<RenderAmbientLightSnapshot> {
        let mut ambient_lights = self
            .ambient_lights
            .iter()
            .filter(|(entity, _)| {
                self.active_in_hierarchy(**entity) == Some(true)
                    && self.entity_intersects_camera_layers(**entity, camera_layers)
            })
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

    fn collect_directional_lights(
        &self,
        camera_layers: &RenderLayerSet,
    ) -> Vec<RenderDirectionalLightSnapshot> {
        let mut directional_lights = self
            .directional_lights
            .iter()
            .filter(|(entity, _)| {
                self.active_in_hierarchy(**entity) == Some(true)
                    && self.entity_intersects_camera_layers(**entity, camera_layers)
            })
            .map(|(entity, light)| RenderDirectionalLightSnapshot {
                node_id: *entity,
                light_id: *entity,
                layer_mask: self
                    .render_layer_mask(*entity)
                    .unwrap_or(default_render_layer_mask()),
                direction: light.direction,
                color: light.color,
                intensity: light.intensity,
                shadow: None,
            })
            .collect::<Vec<_>>();
        directional_lights.sort_by_key(|light| light.node_id);
        directional_lights
    }

    fn collect_point_lights(
        &self,
        camera_layers: &RenderLayerSet,
    ) -> Vec<RenderPointLightSnapshot> {
        let mut point_lights = self
            .point_lights
            .iter()
            .filter(|(entity, _)| {
                self.active_in_hierarchy(**entity) == Some(true)
                    && self.entity_intersects_camera_layers(**entity, camera_layers)
            })
            .map(|(entity, light)| RenderPointLightSnapshot {
                node_id: *entity,
                light_id: *entity,
                layer_mask: self
                    .render_layer_mask(*entity)
                    .unwrap_or(default_render_layer_mask()),
                position: self
                    .world_transform(*entity)
                    .unwrap_or_default()
                    .translation,
                color: light.color,
                intensity: light.intensity,
                range: light.range,
                shadow: None,
            })
            .collect::<Vec<_>>();
        point_lights.sort_by_key(|light| light.node_id);
        point_lights
    }

    fn collect_rect_lights(&self, camera_layers: &RenderLayerSet) -> Vec<RenderRectLightSnapshot> {
        let mut rect_lights = self
            .rect_lights
            .iter()
            .filter(|(entity, _)| {
                self.active_in_hierarchy(**entity) == Some(true)
                    && self.entity_intersects_camera_layers(**entity, camera_layers)
            })
            .map(|(entity, light)| {
                let transform = self.world_transform(*entity).unwrap_or_default();
                RenderRectLightSnapshot {
                    node_id: *entity,
                    light_id: *entity,
                    layer_mask: self
                        .render_layer_mask(*entity)
                        .unwrap_or(default_render_layer_mask()),
                    position: transform.translation,
                    direction: transform.forward(),
                    color: light.color,
                    intensity: light.intensity,
                    range: light.range,
                    size: light.size,
                    shadow: None,
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

    fn collect_spot_lights(&self, camera_layers: &RenderLayerSet) -> Vec<RenderSpotLightSnapshot> {
        let mut spot_lights = self
            .spot_lights
            .iter()
            .filter(|(entity, _)| {
                self.active_in_hierarchy(**entity) == Some(true)
                    && self.entity_intersects_camera_layers(**entity, camera_layers)
            })
            .map(|(entity, light)| RenderSpotLightSnapshot {
                node_id: *entity,
                light_id: *entity,
                layer_mask: self
                    .render_layer_mask(*entity)
                    .unwrap_or(default_render_layer_mask()),
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
                shadow: None,
            })
            .collect::<Vec<_>>();
        spot_lights.sort_by_key(|light| light.node_id);
        spot_lights
    }

    pub(super) fn entity_intersects_camera_layers(
        &self,
        entity: crate::scene::EntityId,
        camera_layers: &RenderLayerSet,
    ) -> bool {
        let render_layer_mask = self
            .render_layer_mask(entity)
            .unwrap_or(default_render_layer_mask());
        camera_layers.intersects_legacy_mask(render_layer_mask)
    }

    fn build_render_camera(
        &self,
        request: &SceneViewportExtractRequest,
    ) -> (CameraRenderDescriptor, Option<crate::scene::EntityId>) {
        if let Some(mut camera) = request.camera.clone() {
            if let Some(viewport_size) = request.viewport_size {
                camera.apply_target_size(viewport_size);
            }
            return (camera, None);
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
        let mut camera = self.build_render_camera_descriptor_for_component(entity, component);
        if request.settings.projection_mode != ProjectionMode::default() {
            camera.camera.projection_mode = request.settings.projection_mode;
        }
        if let Some(viewport_size) = request.viewport_size {
            camera.apply_target_size(viewport_size);
        }
        (camera, Some(entity))
    }

    fn build_render_view_extract(
        &self,
        camera: CameraRenderDescriptor,
        scene_camera_entity: Option<crate::scene::EntityId>,
    ) -> RenderViewExtract {
        let view = match scene_camera_entity {
            Some(entity) => RenderViewExtract::from_camera(camera.camera.clone()).with_cameras(
                self.scene_camera_descriptors()
                    .into_iter()
                    .filter(CameraRenderDescriptor::is_active)
                    .map(|descriptor| {
                        if descriptor.entity == Some(entity) {
                            camera.clone()
                        } else {
                            descriptor
                        }
                    })
                    .collect(),
            ),
            None => {
                RenderViewExtract::from_camera(camera.camera.clone()).with_cameras(vec![camera])
            }
        };
        if let Some(entity) = scene_camera_entity {
            view.with_scene_camera_order_report(entity, self.render_camera_order_report())
        } else {
            view
        }
    }

    fn render_extract_layers_for_view(&self, view: &RenderViewExtract) -> RenderLayerSet {
        let selected_layers = view
            .selected_camera_descriptor()
            .map(|camera| camera.culling_mask.clone())
            .unwrap_or_default();
        view.cameras
            .iter()
            .filter(|camera| {
                camera.entity == view.scene_camera_entity
                    || !matches!(
                        camera.target,
                        crate::core::framework::render::RenderCameraTarget::PrimarySurface
                    )
            })
            .fold(selected_layers, |layers, camera| {
                layers.union(&camera.culling_mask)
            })
    }

    fn build_render_camera_for_entity(
        &self,
        entity: crate::scene::EntityId,
    ) -> CameraRenderDescriptor {
        let component = self
            .cameras
            .get(&entity)
            .expect("camera order projection must refer to camera entity");
        self.build_render_camera_descriptor_for_component(entity, component)
    }

    fn scene_camera_descriptors(&self) -> Vec<CameraRenderDescriptor> {
        let mut cameras = self
            .cameras
            .keys()
            .copied()
            .map(|entity| self.build_render_camera_for_entity(entity))
            .collect::<Vec<_>>();
        cameras.sort_by(|left, right| {
            (
                left.render_order,
                left.target_key(),
                left.entity.unwrap_or(crate::scene::EntityId::MAX),
            )
                .cmp(&(
                    right.render_order,
                    right.target_key(),
                    right.entity.unwrap_or(crate::scene::EntityId::MAX),
                ))
        });
        cameras
    }

    fn build_render_camera_descriptor_for_component(
        &self,
        entity: crate::scene::EntityId,
        component: &crate::scene::components::CameraComponent,
    ) -> CameraRenderDescriptor {
        let transform = self.world_transform(entity).unwrap_or_else(|| {
            self.find_node(entity)
                .map(|node| node.transform)
                .unwrap_or_default()
        });
        let camera = ViewportCameraSnapshot {
            transform,
            projection_mode: component.projection_mode,
            fov_y_radians: component.fov_y_radians,
            ortho_size: component.ortho_size,
            z_near: component.z_near,
            z_far: component.z_far,
            aspect_ratio: default_viewport_aspect_ratio(),
            is_active: component.is_active && self.active_in_hierarchy(entity) == Some(true),
            hdr: component.hdr,
            exposure_ev100: component.exposure_ev100,
            msaa_samples: component.msaa_samples,
            ..ViewportCameraSnapshot::default()
        };
        let mut descriptor = CameraRenderDescriptor {
            entity: Some(entity),
            render_order: component.order,
            target: component.target.clone(),
            viewport_rect: component.viewport,
            clear: component.clear_color.into(),
            culling_mask: RenderLayerSet::from_legacy_mask(
                self.render_layer_mask(entity)
                    .unwrap_or(default_render_layer_mask()),
            ),
            volume_mask: RenderLayerSet::from_legacy_mask(
                self.render_layer_mask(entity)
                    .unwrap_or(default_render_layer_mask()),
            ),
            camera,
            ..CameraRenderDescriptor::from_camera_payload(
                Some(entity),
                ViewportCameraSnapshot::default(),
            )
        };
        if let crate::core::framework::render::RenderCameraTarget::Headless { size } =
            &descriptor.target
        {
            descriptor.apply_target_size(*size);
        }
        descriptor
    }
}

struct MeshRenderSource<'a> {
    model: crate::core::resource::ResourceHandle<crate::core::resource::ModelMarker>,
    mesh: Option<crate::core::resource::ResourceHandle<crate::core::resource::MeshMarker>>,
    material: crate::core::resource::ResourceHandle<crate::core::resource::MaterialMarker>,
    mesh_lod: Option<RenderMeshLodSelection>,
    primitives: &'a [MeshRendererPrimitiveBinding],
}

fn mesh_render_source_for_camera<'a>(
    mesh: &'a MeshRenderer,
    transform: Transform,
    camera_position: Vec3,
) -> MeshRenderSource<'a> {
    if let Some((lod_index, lod)) = mesh_lod_for_camera(mesh, transform, camera_position) {
        return MeshRenderSource {
            model: lod.model,
            mesh: lod.mesh,
            material: lod.material,
            mesh_lod: Some(RenderMeshLodSelection::new(
                lod_index.min(u32::MAX as usize) as u32,
                lod.min_distance,
            )),
            primitives: &lod.primitives,
        };
    }

    MeshRenderSource {
        model: mesh.model,
        mesh: mesh.mesh,
        material: mesh.material,
        mesh_lod: None,
        primitives: &mesh.primitives,
    }
}

fn mesh_lod_for_camera<'a>(
    mesh: &'a MeshRenderer,
    transform: Transform,
    camera_position: Vec3,
) -> Option<(usize, &'a MeshRendererLodLevel)> {
    let distance = (transform.translation - camera_position).length();
    if !distance.is_finite() {
        return None;
    }

    let mut choice = None;
    let mut choice_min_distance = 0.0;
    for (index, lod) in mesh.lods.iter().enumerate() {
        let min_distance = lod.min_distance;
        if !min_distance.is_finite() || min_distance <= 0.0 || distance < min_distance {
            continue;
        }
        if choice.is_none() || min_distance >= choice_min_distance {
            choice = Some((index, lod));
            choice_min_distance = min_distance;
        }
    }
    choice
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
    view: RenderViewExtract,
    request: &SceneViewportExtractRequest,
) -> RenderFrameExtract {
    let mut geometry = GeometryExtract::from_meshes_and_phase_inputs(
        view.camera.core_pipeline_kind(),
        Vec::new(),
        Vec::new(),
    );
    let camera_exposure_ev100 = view.camera.exposure_ev100;
    geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        debug: request.virtual_geometry_debug.unwrap_or_default(),
        ..RenderVirtualGeometryExtract::default()
    });
    geometry.virtual_geometry_debug = request.virtual_geometry_debug;

    RenderFrameExtract {
        world,
        view,
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
        post_process: build_post_process_extract(
            request,
            camera_exposure_ev100,
            PostProcessSettingsComponent::default(),
            Vec::new(),
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
    particles: &ParticleExtract,
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
        .chain(
            particles
                .emitters
                .iter()
                .map(|entity| VisibilityRenderableInput {
                    entity: *entity,
                    mobility: crate::scene::components::Mobility::Dynamic,
                    render_layer_mask: default_render_layer_mask(),
                }),
        )
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

fn build_post_process_extract(
    request: &SceneViewportExtractRequest,
    camera_exposure_ev100: crate::core::math::Real,
    settings: PostProcessSettingsComponent,
    volumes: Vec<PostProcessVolumeExtract>,
) -> PostProcessExtract {
    let mut post_process = PostProcessExtract::from_parts_with_effect_stack(
        build_preview_environment(request),
        request.settings.display_mode,
        settings.bloom,
        settings.color_grading,
        settings.effect_stack,
        false,
        false,
    );
    post_process.exposure = RenderExposureSettings::manual_ev100(camera_exposure_ev100);
    post_process.volumes = volumes;
    post_process
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
