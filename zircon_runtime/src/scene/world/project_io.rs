use std::fs;
use std::path::Path;

use crate::asset::assets::{
    ImportedAsset, SceneAmbientLightAsset, SceneAnimationGraphPlayerAsset,
    SceneAnimationPlayerAsset, SceneAnimationSequencePlayerAsset, SceneAnimationSkeletonAsset,
    SceneAnimationStateMachinePlayerAsset, SceneAsset, SceneBloomSettingsAsset, SceneCameraAsset,
    SceneCameraTargetAsset, SceneChromaticAberrationSettingsAsset, SceneColliderAsset,
    SceneColliderShapeAsset, SceneColorGradingSettingsAsset, SceneDirectionalLightAsset,
    SceneDitherSettingsAsset, SceneEntityAsset, SceneFilmGrainSettingsAsset, SceneFogSettingsAsset,
    SceneJointAsset, SceneJointKindAsset, SceneMeshInstanceAsset, SceneMeshLodLevelAsset,
    SceneMeshPrimitiveBindingAsset, SceneMobilityAsset, ScenePointLightAsset,
    ScenePostProcessEffectStackAsset, ScenePostProcessSettingsAsset, ScenePostProcessVolumeAsset,
    ScenePostProcessVolumeProfileAsset, SceneRectLightAsset, SceneRigidBodyAsset,
    SceneRigidBodyTypeAsset, SceneScriptBindingAsset, SceneSpotLightAsset,
    SceneTonemapOperatorAsset, SceneTonemapSettingsAsset, SceneViewportRectAsset,
    SceneVignetteSettingsAsset, TransformAsset,
};
use crate::asset::importer::AssetImportError;
use crate::asset::project::ProjectManager;
use crate::asset::AssetReference;
use crate::core::resource::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, MaterialMarker, MeshMarker, ModelMarker, PhysicsMaterialMarker,
    ResourceHandle, ResourceId, ResourceLocator, ResourceMarker, ResourceScheme, TextureMarker,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod camera;
mod physics;
mod post_process;
mod references;
mod script;
mod transform;

use super::World;
use crate::core::framework::render::{
    RenderBloomSettings, RenderCameraTarget, RenderChromaticAberrationSettings,
    RenderColorGradingSettings, RenderDitherSettings, RenderFilmGrainSettings, RenderFogSettings,
    RenderPostProcessEffectStackSettings, RenderPostProcessVolumeProfile, RenderTonemapOperator,
    RenderTonemapSettings, RenderViewportRect, RenderVignetteSettings,
};
use crate::scene::components::{
    AmbientLight, AnimationGraphPlayerComponent, AnimationPlayerComponent,
    AnimationSequencePlayerComponent, AnimationSkeletonComponent,
    AnimationStateMachinePlayerComponent, CameraComponent, ColliderComponent, ColliderShape,
    JointComponent, JointKind, MeshRendererLodLevel, MeshRendererPrimitiveBinding, Mobility,
    NodeKind, PointLight, PostProcessSettingsComponent, PostProcessVolumeComponent, RectLight,
    RigidBodyComponent, RigidBodyType, SpotLight,
};
use crate::scene::ecs::Schedule;
use camera::{camera_target_from_asset, camera_to_asset, viewport_rect_from_asset};
use physics::{collider_shape_from_asset, collider_shape_to_asset};
use post_process::{
    post_process_settings_from_asset, post_process_settings_to_asset,
    post_process_volume_from_asset, post_process_volume_to_asset,
};
use references::{
    handle_for_reference, material_handle_for_reference, model_handle_for_reference,
    reference_for_handle, reference_for_material_handle, reference_for_mesh_handle,
    reference_for_model_handle,
};
use script::script_bindings_for_record;
use transform::{transform_from_asset, transform_to_asset};

const PROJECT_FORMAT_VERSION: u32 = 2;
const BUILTIN_CUBE: &str = "builtin://cube";
const BUILTIN_DEFAULT_MATERIAL: &str = "builtin://material/default";
const BUILTIN_MISSING_MODEL: &str = "builtin://missing-model";
const BUILTIN_MISSING_MATERIAL: &str = "builtin://missing-material";
const SCRIPT_BINDINGS_COMPONENT: &str = "script.bindings";

#[derive(Debug, Error)]
pub enum SceneProjectError {
    #[error("project I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("project parse failed: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("asset import failed: {0}")]
    Asset(#[from] AssetImportError),
    #[error("scene asset error: {0}")]
    SceneAsset(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ProjectDocument {
    format_version: u32,
    world: World,
}

impl World {
    pub fn load_scene_from_uri(
        project: &ProjectManager,
        uri: &ResourceLocator,
    ) -> Result<Self, SceneProjectError> {
        let ImportedAsset::Scene(scene) = project.load_artifact(uri)? else {
            return Err(SceneProjectError::SceneAsset(format!(
                "asset {uri} is not a scene"
            )));
        };
        Self::from_scene_asset(project, &scene)
    }

    pub fn from_scene_asset(
        project: &ProjectManager,
        scene: &SceneAsset,
    ) -> Result<Self, SceneProjectError> {
        let mut world = Self::empty();

        for entity in &scene.entities {
            let kind = if entity.camera.is_some() {
                NodeKind::Camera
            } else if entity.ambient_light.is_some() {
                NodeKind::AmbientLight
            } else if entity.directional_light.is_some() {
                NodeKind::DirectionalLight
            } else if entity.point_light.is_some() {
                NodeKind::PointLight
            } else if entity.rect_light.is_some() {
                NodeKind::RectLight
            } else if entity.spot_light.is_some() {
                NodeKind::SpotLight
            } else if entity.post_process_volume.is_some() {
                NodeKind::Empty
            } else if entity
                .mesh
                .as_ref()
                .is_some_and(|mesh| mesh.model.to_string() == BUILTIN_CUBE)
            {
                NodeKind::Cube
            } else if entity.mesh.is_some() {
                NodeKind::Mesh
            } else if !entity.script_bindings.is_empty() {
                NodeKind::Empty
            } else {
                NodeKind::Empty
            };

            let mesh = entity.mesh.as_ref().map(|mesh| {
                let mut renderer = crate::scene::components::MeshRenderer::from_handles(
                    model_handle_for_reference(project, &mesh.model),
                    material_handle_for_reference(project, &mesh.material),
                );
                renderer.mesh = mesh
                    .mesh
                    .as_ref()
                    .map(|reference| handle_for_reference::<MeshMarker>(project, reference));
                renderer.render_queue = mesh.render_queue;
                renderer.material_queue = mesh.material_queue;
                renderer.order_in_layer = mesh.order_in_layer;
                renderer.depth_bias = mesh.depth_bias;
                renderer.morph_weights = mesh.morph_weights.clone();
                renderer.primitives = mesh
                    .primitives
                    .iter()
                    .map(|primitive| MeshRendererPrimitiveBinding {
                        mesh: handle_for_reference::<MeshMarker>(project, &primitive.mesh),
                        material: handle_for_reference::<MaterialMarker>(
                            project,
                            &primitive.material,
                        ),
                    })
                    .collect();
                renderer.lods = mesh
                    .lods
                    .iter()
                    .map(|lod| MeshRendererLodLevel {
                        min_distance: lod.min_distance,
                        model: model_handle_for_reference(project, &lod.model),
                        mesh: lod.mesh.as_ref().map(|reference| {
                            handle_for_reference::<MeshMarker>(project, reference)
                        }),
                        material: material_handle_for_reference(project, &lod.material),
                        primitives: lod
                            .primitives
                            .iter()
                            .map(|primitive| MeshRendererPrimitiveBinding {
                                mesh: handle_for_reference::<MeshMarker>(project, &primitive.mesh),
                                material: handle_for_reference::<MaterialMarker>(
                                    project,
                                    &primitive.material,
                                ),
                            })
                            .collect(),
                    })
                    .collect();
                renderer
            });

            world
                .insert_node_record(crate::scene::components::NodeRecord {
                    id: entity.entity,
                    name: entity.name.clone(),
                    kind,
                    parent: entity.parent,
                    transform: crate::core::math::Transform {
                        translation: crate::core::math::Vec3::from_array(
                            entity.transform.translation,
                        ),
                        rotation: crate::core::math::Quat::from_array(entity.transform.rotation),
                        scale: crate::core::math::Vec3::from_array(entity.transform.scale),
                    },
                    camera: entity.camera.clone().map(|camera| CameraComponent {
                        projection_mode: camera.projection_mode,
                        fov_y_radians: camera.fov_y_radians,
                        ortho_size: camera.ortho_size,
                        z_near: camera.z_near,
                        z_far: camera.z_far,
                        target: camera_target_from_asset(project, camera.target),
                        viewport: camera.viewport.map(viewport_rect_from_asset),
                        order: camera.order,
                        is_active: camera.active,
                        hdr: camera.hdr,
                        exposure_ev100: camera.exposure_ev100,
                        clear_color: camera.clear_color,
                        msaa_samples: camera.msaa_samples,
                    }),
                    mesh,
                    sprite_2d: None,
                    mesh_2d: None,
                    ambient_light: entity.ambient_light.clone().map(|light| AmbientLight {
                        color: crate::core::math::Vec3::from_array(light.color),
                        intensity: light.intensity,
                        affects_lightmapped_meshes: light.affects_lightmapped_meshes,
                    }),
                    directional_light: entity.directional_light.clone().map(|light| {
                        crate::scene::components::DirectionalLight {
                            direction: crate::core::math::Vec3::from_array(light.direction),
                            color: crate::core::math::Vec3::from_array(light.color),
                            intensity: light.intensity,
                        }
                    }),
                    point_light: entity.point_light.clone().map(|light| PointLight {
                        color: crate::core::math::Vec3::from_array(light.color),
                        intensity: light.intensity,
                        range: light.range,
                    }),
                    spot_light: entity.spot_light.clone().map(|light| SpotLight {
                        direction: crate::core::math::Vec3::from_array(light.direction),
                        color: crate::core::math::Vec3::from_array(light.color),
                        intensity: light.intensity,
                        range: light.range,
                        inner_angle_radians: light.inner_angle_radians,
                        outer_angle_radians: light.outer_angle_radians,
                    }),
                    rect_light: entity.rect_light.clone().map(|light| RectLight {
                        color: crate::core::math::Vec3::from_array(light.color),
                        intensity: light.intensity,
                        range: light.range,
                        size: crate::core::math::Vec2::from_array(light.size),
                    }),
                    active: entity.active,
                    render_layer_mask: entity.render_layer_mask,
                    mobility: match entity.mobility {
                        SceneMobilityAsset::Dynamic => Mobility::Dynamic,
                        SceneMobilityAsset::Static => Mobility::Static,
                    },
                    rigid_body: entity
                        .rigid_body
                        .clone()
                        .map(|rigid_body| RigidBodyComponent {
                            body_type: match rigid_body.body_type {
                                SceneRigidBodyTypeAsset::Static => RigidBodyType::Static,
                                SceneRigidBodyTypeAsset::Dynamic => RigidBodyType::Dynamic,
                                SceneRigidBodyTypeAsset::Kinematic => RigidBodyType::Kinematic,
                            },
                            mass: rigid_body.mass,
                            linear_velocity: crate::core::math::Vec3::from_array(
                                rigid_body.linear_velocity,
                            ),
                            angular_velocity: crate::core::math::Vec3::from_array(
                                rigid_body.angular_velocity,
                            ),
                            linear_damping: rigid_body.linear_damping,
                            angular_damping: rigid_body.angular_damping,
                            gravity_scale: rigid_body.gravity_scale,
                            can_sleep: rigid_body.can_sleep,
                            lock_translation: rigid_body.lock_translation,
                            lock_rotation: rigid_body.lock_rotation,
                        }),
                    collider: entity.collider.clone().map(|collider| ColliderComponent {
                        shape: collider_shape_from_asset(collider.shape),
                        sensor: collider.sensor,
                        layer: collider.layer,
                        collision_group: collider.collision_group,
                        collision_mask: collider.collision_mask,
                        material: collider.material.as_ref().map(|reference| {
                            handle_for_reference::<PhysicsMaterialMarker>(project, reference)
                        }),
                        material_override: collider.material_override,
                        local_transform: transform_from_asset(collider.local_transform),
                    }),
                    joint: entity.joint.clone().map(|joint| JointComponent {
                        joint_type: match joint.joint_type {
                            SceneJointKindAsset::Fixed => JointKind::Fixed,
                            SceneJointKindAsset::Distance => JointKind::Distance,
                            SceneJointKindAsset::Hinge => JointKind::Hinge,
                            SceneJointKindAsset::Slider => JointKind::Slider,
                            SceneJointKindAsset::ConeTwist => JointKind::ConeTwist,
                            SceneJointKindAsset::Generic6Dof => JointKind::Generic6Dof,
                        },
                        connected_entity: joint.connected_entity,
                        anchor: crate::core::math::Vec3::from_array(joint.anchor),
                        axis: crate::core::math::Vec3::from_array(joint.axis),
                        limits: joint.limits,
                        collide_connected: joint.collide_connected,
                        constraint: joint.constraint,
                        skeleton_binding: joint.skeleton_binding,
                    }),
                    animation_skeleton: entity.animation_skeleton.clone().map(
                        |animation_skeleton| AnimationSkeletonComponent {
                            skeleton: handle_for_reference::<AnimationSkeletonMarker>(
                                project,
                                &animation_skeleton.skeleton,
                            ),
                        },
                    ),
                    animation_player: entity.animation_player.clone().map(|animation_player| {
                        AnimationPlayerComponent {
                            clip: handle_for_reference::<AnimationClipMarker>(
                                project,
                                &animation_player.clip,
                            ),
                            playback_speed: animation_player.playback_speed,
                            time_seconds: animation_player.time_seconds,
                            weight: animation_player.weight,
                            looping: animation_player.looping,
                            playing: animation_player.playing,
                        }
                    }),
                    animation_sequence_player: entity.animation_sequence_player.clone().map(
                        |animation_sequence_player| AnimationSequencePlayerComponent {
                            sequence: handle_for_reference::<AnimationSequenceMarker>(
                                project,
                                &animation_sequence_player.sequence,
                            ),
                            playback_speed: animation_sequence_player.playback_speed,
                            time_seconds: animation_sequence_player.time_seconds,
                            looping: animation_sequence_player.looping,
                            playing: animation_sequence_player.playing,
                        },
                    ),
                    animation_graph_player: entity.animation_graph_player.clone().map(
                        |animation_graph_player| AnimationGraphPlayerComponent {
                            graph: handle_for_reference::<AnimationGraphMarker>(
                                project,
                                &animation_graph_player.graph,
                            ),
                            parameters: animation_graph_player.parameters,
                            playing: animation_graph_player.playing,
                        },
                    ),
                    animation_state_machine_player: entity
                        .animation_state_machine_player
                        .clone()
                        .map(|animation_state_machine_player| {
                            AnimationStateMachinePlayerComponent {
                                state_machine: handle_for_reference::<AnimationStateMachineMarker>(
                                    project,
                                    &animation_state_machine_player.state_machine,
                                ),
                                parameters: animation_state_machine_player.parameters,
                                active_state: animation_state_machine_player.active_state,
                                playing: animation_state_machine_player.playing,
                            }
                        }),
                })
                .map_err(SceneProjectError::SceneAsset)?;
            if let Some(camera_post_process) = entity
                .camera
                .as_ref()
                .and_then(|camera| camera.post_process_settings)
            {
                world
                    .insert(
                        entity.entity,
                        post_process_settings_from_asset(camera_post_process),
                    )
                    .map_err(SceneProjectError::SceneAsset)?;
            }
            if let Some(post_process_volume) = entity.post_process_volume {
                world
                    .insert(
                        entity.entity,
                        post_process_volume_from_asset(post_process_volume),
                    )
                    .map_err(SceneProjectError::SceneAsset)?;
            }
            if !entity.script_bindings.is_empty() {
                world
                    .set_dynamic_component(
                        entity.entity,
                        SCRIPT_BINDINGS_COMPONENT,
                        serde_json::to_value(&entity.script_bindings).map_err(|error| {
                            SceneProjectError::SceneAsset(format!(
                                "failed to encode script bindings for entity {}: {error}",
                                entity.entity
                            ))
                        })?,
                    )
                    .map_err(SceneProjectError::SceneAsset)?;
            }
        }

        world.normalize_scene_asset_after_load();
        Ok(world)
    }

    pub fn to_scene_asset(
        &self,
        project: &ProjectManager,
    ) -> Result<SceneAsset, SceneProjectError> {
        let entities = self
            .entities
            .iter()
            .copied()
            .filter_map(|entity| self.node_record(entity))
            .map(|record| {
                let mesh = record
                    .mesh
                    .map(|mesh| {
                        Ok::<SceneMeshInstanceAsset, SceneProjectError>(SceneMeshInstanceAsset {
                            model: reference_for_model_handle(project, mesh.model)?,
                            mesh: mesh
                                .mesh
                                .map(|mesh| reference_for_mesh_handle(project, mesh))
                                .transpose()?,
                            material: reference_for_material_handle(project, mesh.material)?,
                            render_queue: mesh.render_queue,
                            material_queue: mesh.material_queue,
                            order_in_layer: mesh.order_in_layer,
                            depth_bias: mesh.depth_bias,
                            morph_weights: mesh.morph_weights,
                            primitives: mesh
                                .primitives
                                .into_iter()
                                .map(|primitive| {
                                    Ok::<SceneMeshPrimitiveBindingAsset, SceneProjectError>(
                                        SceneMeshPrimitiveBindingAsset {
                                            mesh: reference_for_mesh_handle(
                                                project,
                                                primitive.mesh,
                                            )?,
                                            material: reference_for_material_handle(
                                                project,
                                                primitive.material,
                                            )?,
                                        },
                                    )
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                            lods: mesh
                                .lods
                                .into_iter()
                                .map(|lod| {
                                    Ok::<SceneMeshLodLevelAsset, SceneProjectError>(
                                        SceneMeshLodLevelAsset {
                                            min_distance: lod.min_distance,
                                            model: reference_for_model_handle(project, lod.model)?,
                                            mesh: lod
                                                .mesh
                                                .map(|mesh| {
                                                    reference_for_mesh_handle(project, mesh)
                                                })
                                                .transpose()?,
                                            material: reference_for_material_handle(
                                                project,
                                                lod.material,
                                            )?,
                                            primitives: lod
                                                .primitives
                                                .into_iter()
                                                .map(|primitive| {
                                                    Ok::<
                                                        SceneMeshPrimitiveBindingAsset,
                                                        SceneProjectError,
                                                    >(
                                                        SceneMeshPrimitiveBindingAsset {
                                                            mesh: reference_for_mesh_handle(
                                                                project,
                                                                primitive.mesh,
                                                            )?,
                                                            material:
                                                                reference_for_material_handle(
                                                                    project,
                                                                    primitive.material,
                                                                )?,
                                                        },
                                                    )
                                                })
                                                .collect::<Result<Vec<_>, _>>()?,
                                        },
                                    )
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        })
                    })
                    .transpose()?;

                let script_bindings = script_bindings_for_record(self, record.id)?;
                let post_process_settings = self
                    .post_process_settings
                    .get(&record.id)
                    .cloned()
                    .map(post_process_settings_to_asset);
                let post_process_volume = self
                    .post_process_volumes
                    .get(&record.id)
                    .cloned()
                    .map(post_process_volume_to_asset);

                Ok(SceneEntityAsset {
                    entity: record.id,
                    name: record.name,
                    parent: record.parent,
                    transform: TransformAsset {
                        translation: record.transform.translation.to_array(),
                        rotation: record.transform.rotation.to_array(),
                        scale: record.transform.scale.to_array(),
                    },
                    active: record.active,
                    render_layer_mask: record.render_layer_mask,
                    mobility: match record.mobility {
                        Mobility::Dynamic => SceneMobilityAsset::Dynamic,
                        Mobility::Static => SceneMobilityAsset::Static,
                    },
                    camera: record
                        .camera
                        .map(|camera| camera_to_asset(project, camera, post_process_settings))
                        .transpose()?,
                    mesh,
                    ambient_light: record.ambient_light.map(|light| SceneAmbientLightAsset {
                        color: light.color.to_array(),
                        intensity: light.intensity,
                        affects_lightmapped_meshes: light.affects_lightmapped_meshes,
                    }),
                    directional_light: record.directional_light.map(|light| {
                        SceneDirectionalLightAsset {
                            direction: light.direction.to_array(),
                            color: light.color.to_array(),
                            intensity: light.intensity,
                        }
                    }),
                    point_light: record.point_light.map(|light| ScenePointLightAsset {
                        color: light.color.to_array(),
                        intensity: light.intensity,
                        range: light.range,
                    }),
                    rect_light: record.rect_light.map(|light| SceneRectLightAsset {
                        color: light.color.to_array(),
                        intensity: light.intensity,
                        range: light.range,
                        size: light.size.to_array(),
                    }),
                    spot_light: record.spot_light.map(|light| SceneSpotLightAsset {
                        direction: light.direction.to_array(),
                        color: light.color.to_array(),
                        intensity: light.intensity,
                        range: light.range,
                        inner_angle_radians: light.inner_angle_radians,
                        outer_angle_radians: light.outer_angle_radians,
                    }),
                    post_process_volume,
                    rigid_body: record.rigid_body.map(|rigid_body| SceneRigidBodyAsset {
                        body_type: match rigid_body.body_type {
                            RigidBodyType::Static => SceneRigidBodyTypeAsset::Static,
                            RigidBodyType::Dynamic => SceneRigidBodyTypeAsset::Dynamic,
                            RigidBodyType::Kinematic => SceneRigidBodyTypeAsset::Kinematic,
                        },
                        mass: rigid_body.mass,
                        linear_velocity: rigid_body.linear_velocity.to_array(),
                        angular_velocity: rigid_body.angular_velocity.to_array(),
                        linear_damping: rigid_body.linear_damping,
                        angular_damping: rigid_body.angular_damping,
                        gravity_scale: rigid_body.gravity_scale,
                        can_sleep: rigid_body.can_sleep,
                        lock_translation: rigid_body.lock_translation,
                        lock_rotation: rigid_body.lock_rotation,
                    }),
                    collider: record
                        .collider
                        .map(|collider| {
                            Ok::<SceneColliderAsset, SceneProjectError>(SceneColliderAsset {
                                shape: collider_shape_to_asset(collider.shape),
                                sensor: collider.sensor,
                                layer: collider.layer,
                                collision_group: collider.collision_group,
                                collision_mask: collider.collision_mask,
                                material: collider
                                    .material
                                    .map(|material| {
                                        reference_for_handle(
                                            project,
                                            material.id(),
                                            "physics material",
                                        )
                                    })
                                    .transpose()?,
                                material_override: collider.material_override,
                                local_transform: transform_to_asset(collider.local_transform),
                            })
                        })
                        .transpose()?,
                    joint: record.joint.map(|joint| SceneJointAsset {
                        joint_type: match joint.joint_type {
                            JointKind::Fixed => SceneJointKindAsset::Fixed,
                            JointKind::Distance => SceneJointKindAsset::Distance,
                            JointKind::Hinge => SceneJointKindAsset::Hinge,
                            JointKind::Slider => SceneJointKindAsset::Slider,
                            JointKind::ConeTwist => SceneJointKindAsset::ConeTwist,
                            JointKind::Generic6Dof => SceneJointKindAsset::Generic6Dof,
                        },
                        connected_entity: joint.connected_entity,
                        anchor: joint.anchor.to_array(),
                        axis: joint.axis.to_array(),
                        limits: joint.limits,
                        collide_connected: joint.collide_connected,
                        constraint: joint.constraint,
                        skeleton_binding: joint.skeleton_binding,
                    }),
                    animation_skeleton: record
                        .animation_skeleton
                        .map(|animation_skeleton| {
                            Ok::<SceneAnimationSkeletonAsset, SceneProjectError>(
                                SceneAnimationSkeletonAsset {
                                    skeleton: reference_for_handle(
                                        project,
                                        animation_skeleton.skeleton.id(),
                                        "animation skeleton",
                                    )?,
                                },
                            )
                        })
                        .transpose()?,
                    animation_player: record
                        .animation_player
                        .map(|animation_player| {
                            Ok::<SceneAnimationPlayerAsset, SceneProjectError>(
                                SceneAnimationPlayerAsset {
                                    clip: reference_for_handle(
                                        project,
                                        animation_player.clip.id(),
                                        "animation clip",
                                    )?,
                                    playback_speed: animation_player.playback_speed,
                                    time_seconds: animation_player.time_seconds,
                                    weight: animation_player.weight,
                                    looping: animation_player.looping,
                                    playing: animation_player.playing,
                                },
                            )
                        })
                        .transpose()?,
                    animation_sequence_player: record
                        .animation_sequence_player
                        .map(|animation_sequence_player| {
                            Ok::<SceneAnimationSequencePlayerAsset, SceneProjectError>(
                                SceneAnimationSequencePlayerAsset {
                                    sequence: reference_for_handle(
                                        project,
                                        animation_sequence_player.sequence.id(),
                                        "animation sequence",
                                    )?,
                                    playback_speed: animation_sequence_player.playback_speed,
                                    time_seconds: animation_sequence_player.time_seconds,
                                    looping: animation_sequence_player.looping,
                                    playing: animation_sequence_player.playing,
                                },
                            )
                        })
                        .transpose()?,
                    animation_graph_player: record
                        .animation_graph_player
                        .map(|animation_graph_player| {
                            Ok::<SceneAnimationGraphPlayerAsset, SceneProjectError>(
                                SceneAnimationGraphPlayerAsset {
                                    graph: reference_for_handle(
                                        project,
                                        animation_graph_player.graph.id(),
                                        "animation graph",
                                    )?,
                                    parameters: animation_graph_player.parameters,
                                    playing: animation_graph_player.playing,
                                },
                            )
                        })
                        .transpose()?,
                    animation_state_machine_player: record
                        .animation_state_machine_player
                        .map(|animation_state_machine_player| {
                            Ok::<SceneAnimationStateMachinePlayerAsset, SceneProjectError>(
                                SceneAnimationStateMachinePlayerAsset {
                                    state_machine: reference_for_handle(
                                        project,
                                        animation_state_machine_player.state_machine.id(),
                                        "animation state machine",
                                    )?,
                                    parameters: animation_state_machine_player.parameters,
                                    active_state: animation_state_machine_player.active_state,
                                    playing: animation_state_machine_player.playing,
                                },
                            )
                        })
                        .transpose()?,
                    terrain: None,
                    tilemap: None,
                    prefab_instance: None,
                    script_bindings,
                })
            })
            .collect::<Result<Vec<_>, SceneProjectError>>()?;

        Ok(SceneAsset { entities })
    }

    pub fn save_project_to_path(&self, path: impl AsRef<Path>) -> Result<(), SceneProjectError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let document = ProjectDocument {
            format_version: PROJECT_FORMAT_VERSION,
            world: self.clone(),
        };
        fs::write(path, serde_json::to_string_pretty(&document)?)?;
        Ok(())
    }

    pub fn load_project_from_path(path: impl AsRef<Path>) -> Result<Self, SceneProjectError> {
        let json = fs::read_to_string(path)?;
        let mut document: ProjectDocument = serde_json::from_str(&json)?;
        document.world.normalize_after_load();
        Ok(document.world)
    }

    fn normalize_scene_asset_after_load(&mut self) {
        self.normalize_loaded_state(false);
    }

    fn normalize_after_load(&mut self) {
        self.normalize_loaded_state(true);
    }

    fn normalize_loaded_state(&mut self, ensure_default_nodes: bool) {
        self.schedule = Schedule::default();
        if self.kinds.len() != self.entities.len() {
            self.kinds.clear();
            for entity in &self.entities {
                let kind = if self.cameras.contains_key(entity) {
                    NodeKind::Camera
                } else if self.ambient_lights.contains_key(entity) {
                    NodeKind::AmbientLight
                } else if self.directional_lights.contains_key(entity) {
                    NodeKind::DirectionalLight
                } else if self.point_lights.contains_key(entity) {
                    NodeKind::PointLight
                } else if self.rect_lights.contains_key(entity) {
                    NodeKind::RectLight
                } else if self.spot_lights.contains_key(entity) {
                    NodeKind::SpotLight
                } else if self.mesh_renderers.contains_key(entity) {
                    let is_cube = self.mesh_renderers.get(entity).is_some_and(|mesh| {
                        mesh.model.id() == ResourceId::from_stable_label(BUILTIN_CUBE)
                    });
                    if is_cube {
                        NodeKind::Cube
                    } else {
                        NodeKind::Mesh
                    }
                } else {
                    NodeKind::Empty
                };
                self.kinds.insert(*entity, kind);
            }
        }
        self.next_id = self.entities.iter().copied().max().unwrap_or(0) + 1;
        if ensure_default_nodes && self.cameras.is_empty() {
            self.spawn_node(NodeKind::Camera);
        }
        if !self.cameras.contains_key(&self.active_camera) {
            self.active_camera = self.cameras.keys().next().copied().unwrap_or(0);
        }
        if ensure_default_nodes && self.directional_lights.is_empty() {
            self.spawn_node(NodeKind::DirectionalLight);
        }
        for entity in self.entities.iter().copied().collect::<Vec<_>>() {
            self.active_self.entry(entity).or_default();
            self.render_layer_masks.entry(entity).or_default();
            self.mobility.entry(entity).or_default();
        }
        self.rebuild_entity_registry();
        self.rebuild_typed_component_presence();
        self.mark_derived_state_dirty();
        self.flush_scene_systems_now();
    }
}
