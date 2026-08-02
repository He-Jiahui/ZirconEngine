use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::json_value::{ArtifactCacheJsonValue, cache_table_to_json, json_table_to_cache};
use crate::asset::{AssetImportError, AssetReference, SceneAsset};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(in crate::asset::artifact) struct ArtifactCacheSceneAsset {
    entities: Vec<ArtifactCacheSceneEntityAsset>,
}

impl From<&SceneAsset> for ArtifactCacheSceneAsset {
    fn from(asset: &SceneAsset) -> Self {
        Self {
            entities: asset
                .entities
                .iter()
                .map(ArtifactCacheSceneEntityAsset::from)
                .collect(),
        }
    }
}

impl ArtifactCacheSceneAsset {
    pub(super) fn into_asset(self) -> Result<SceneAsset, AssetImportError> {
        Ok(SceneAsset {
            entities: self
                .entities
                .into_iter()
                .map(ArtifactCacheSceneEntityAsset::into_asset)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheSceneEntityAsset {
    entity: u64,
    name: String,
    parent: Option<u64>,
    transform: crate::asset::TransformAsset,
    active: bool,
    render_layer_mask: u32,
    mobility: crate::asset::SceneMobilityAsset,
    camera: Option<ArtifactCacheSceneCameraAsset>,
    mesh: Option<ArtifactCacheSceneMeshInstanceAsset>,
    ambient_light: Option<crate::asset::SceneAmbientLightAsset>,
    directional_light: Option<crate::asset::SceneDirectionalLightAsset>,
    point_light: Option<crate::asset::ScenePointLightAsset>,
    rect_light: Option<crate::asset::SceneRectLightAsset>,
    spot_light: Option<crate::asset::SceneSpotLightAsset>,
    post_process_volume: Option<crate::asset::ScenePostProcessVolumeAsset>,
    rigid_body: Option<ArtifactCacheSceneRigidBodyAsset>,
    collider: Option<ArtifactCacheSceneColliderAsset>,
    joint: Option<ArtifactCacheSceneJointAsset>,
    animation_skeleton: Option<crate::asset::SceneAnimationSkeletonAsset>,
    animation_player: Option<crate::asset::SceneAnimationPlayerAsset>,
    animation_sequence_player: Option<crate::asset::SceneAnimationSequencePlayerAsset>,
    animation_graph_player: Option<crate::asset::SceneAnimationGraphPlayerAsset>,
    animation_state_machine_player: Option<crate::asset::SceneAnimationStateMachinePlayerAsset>,
    terrain: Option<crate::asset::SceneTerrainAsset>,
    tilemap: Option<crate::asset::SceneTileMapAsset>,
    prefab_instance: Option<crate::asset::PrefabInstanceAsset>,
    script_bindings: Vec<ArtifactCacheSceneScriptBindingAsset>,
}

impl From<&crate::asset::SceneEntityAsset> for ArtifactCacheSceneEntityAsset {
    fn from(asset: &crate::asset::SceneEntityAsset) -> Self {
        Self {
            entity: asset.entity,
            name: asset.name.clone(),
            parent: asset.parent,
            transform: asset.transform,
            active: asset.active,
            render_layer_mask: asset.render_layer_mask,
            mobility: asset.mobility,
            camera: asset
                .camera
                .as_ref()
                .map(ArtifactCacheSceneCameraAsset::from),
            mesh: asset
                .mesh
                .as_ref()
                .map(ArtifactCacheSceneMeshInstanceAsset::from),
            ambient_light: asset.ambient_light.clone(),
            directional_light: asset.directional_light.clone(),
            point_light: asset.point_light.clone(),
            rect_light: asset.rect_light.clone(),
            spot_light: asset.spot_light.clone(),
            post_process_volume: asset.post_process_volume,
            rigid_body: asset
                .rigid_body
                .as_ref()
                .map(ArtifactCacheSceneRigidBodyAsset::from),
            collider: asset
                .collider
                .as_ref()
                .map(ArtifactCacheSceneColliderAsset::from),
            joint: asset.joint.as_ref().map(ArtifactCacheSceneJointAsset::from),
            animation_skeleton: asset.animation_skeleton.clone(),
            animation_player: asset.animation_player.clone(),
            animation_sequence_player: asset.animation_sequence_player.clone(),
            animation_graph_player: asset.animation_graph_player.clone(),
            animation_state_machine_player: asset.animation_state_machine_player.clone(),
            terrain: asset.terrain.clone(),
            tilemap: asset.tilemap.clone(),
            prefab_instance: asset.prefab_instance.clone(),
            script_bindings: asset
                .script_bindings
                .iter()
                .map(ArtifactCacheSceneScriptBindingAsset::from)
                .collect(),
        }
    }
}

impl ArtifactCacheSceneEntityAsset {
    fn into_asset(self) -> Result<crate::asset::SceneEntityAsset, AssetImportError> {
        Ok(crate::asset::SceneEntityAsset {
            entity: self.entity,
            name: self.name,
            parent: self.parent,
            transform: self.transform,
            active: self.active,
            render_layer_mask: self.render_layer_mask,
            mobility: self.mobility,
            camera: self.camera.map(ArtifactCacheSceneCameraAsset::into_asset),
            mesh: self
                .mesh
                .map(ArtifactCacheSceneMeshInstanceAsset::into_asset),
            ambient_light: self.ambient_light,
            directional_light: self.directional_light,
            point_light: self.point_light,
            rect_light: self.rect_light,
            spot_light: self.spot_light,
            post_process_volume: self.post_process_volume,
            rigid_body: self
                .rigid_body
                .map(ArtifactCacheSceneRigidBodyAsset::into_asset),
            collider: self
                .collider
                .map(ArtifactCacheSceneColliderAsset::into_asset),
            joint: self.joint.map(ArtifactCacheSceneJointAsset::into_asset),
            animation_skeleton: self.animation_skeleton,
            animation_player: self.animation_player,
            animation_sequence_player: self.animation_sequence_player,
            animation_graph_player: self.animation_graph_player,
            animation_state_machine_player: self.animation_state_machine_player,
            terrain: self.terrain,
            tilemap: self.tilemap,
            prefab_instance: self.prefab_instance,
            script_bindings: self
                .script_bindings
                .into_iter()
                .map(ArtifactCacheSceneScriptBindingAsset::into_asset)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheSceneRigidBodyAsset {
    body_type: crate::asset::SceneRigidBodyTypeAsset,
    mass: crate::core::math::Real,
    mass_properties: ArtifactCachePhysicsMassProperties,
    linear_velocity: [crate::core::math::Real; 3],
    angular_velocity: [crate::core::math::Real; 3],
    linear_damping: crate::core::math::Real,
    angular_damping: crate::core::math::Real,
    gravity_scale: crate::core::math::Real,
    ccd_mode: crate::core::framework::scene::physics::PhysicsCcdMode,
    sleep_policy: crate::core::framework::scene::physics::PhysicsSleepPolicy,
    lock_translation: [bool; 3],
    lock_rotation: [bool; 3],
}

impl From<&crate::asset::SceneRigidBodyAsset> for ArtifactCacheSceneRigidBodyAsset {
    fn from(asset: &crate::asset::SceneRigidBodyAsset) -> Self {
        Self {
            body_type: asset.body_type,
            mass: asset.mass,
            mass_properties: asset.mass_properties.into(),
            linear_velocity: asset.linear_velocity,
            angular_velocity: asset.angular_velocity,
            linear_damping: asset.linear_damping,
            angular_damping: asset.angular_damping,
            gravity_scale: asset.gravity_scale,
            ccd_mode: asset.ccd_mode,
            sleep_policy: asset.sleep_policy,
            lock_translation: asset.lock_translation,
            lock_rotation: asset.lock_rotation,
        }
    }
}

impl ArtifactCacheSceneRigidBodyAsset {
    fn into_asset(self) -> crate::asset::SceneRigidBodyAsset {
        crate::asset::SceneRigidBodyAsset {
            body_type: self.body_type,
            mass: self.mass,
            mass_properties: self.mass_properties.into(),
            linear_velocity: self.linear_velocity,
            angular_velocity: self.angular_velocity,
            linear_damping: self.linear_damping,
            angular_damping: self.angular_damping,
            gravity_scale: self.gravity_scale,
            ccd_mode: self.ccd_mode,
            sleep_policy: self.sleep_policy,
            lock_translation: self.lock_translation,
            lock_rotation: self.lock_rotation,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum ArtifactCachePhysicsMassProperties {
    Explicit {
        inertia_tensor: Option<[[crate::core::math::Real; 3]; 3]>,
    },
    AutoFromShape {
        density: crate::core::math::Real,
    },
}

impl From<crate::core::framework::scene::physics::PhysicsMassProperties>
    for ArtifactCachePhysicsMassProperties
{
    fn from(properties: crate::core::framework::scene::physics::PhysicsMassProperties) -> Self {
        match properties {
            crate::core::framework::scene::physics::PhysicsMassProperties::Explicit {
                inertia_tensor,
            } => Self::Explicit { inertia_tensor },
            crate::core::framework::scene::physics::PhysicsMassProperties::AutoFromShape {
                density,
            } => Self::AutoFromShape { density },
        }
    }
}

impl From<ArtifactCachePhysicsMassProperties>
    for crate::core::framework::scene::physics::PhysicsMassProperties
{
    fn from(properties: ArtifactCachePhysicsMassProperties) -> Self {
        match properties {
            ArtifactCachePhysicsMassProperties::Explicit { inertia_tensor } => {
                Self::Explicit { inertia_tensor }
            }
            ArtifactCachePhysicsMassProperties::AutoFromShape { density } => {
                Self::AutoFromShape { density }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheSceneMeshPrimitiveBindingAsset {
    mesh: AssetReference,
    material: AssetReference,
}

impl From<&crate::asset::SceneMeshPrimitiveBindingAsset>
    for ArtifactCacheSceneMeshPrimitiveBindingAsset
{
    fn from(asset: &crate::asset::SceneMeshPrimitiveBindingAsset) -> Self {
        Self {
            mesh: asset.mesh.clone(),
            material: asset.material.clone(),
        }
    }
}

impl From<ArtifactCacheSceneMeshPrimitiveBindingAsset>
    for crate::asset::SceneMeshPrimitiveBindingAsset
{
    fn from(asset: ArtifactCacheSceneMeshPrimitiveBindingAsset) -> Self {
        Self {
            mesh: asset.mesh,
            material: asset.material,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheSceneMeshLodLevelAsset {
    min_distance: crate::core::math::Real,
    model: AssetReference,
    mesh: Option<AssetReference>,
    material: AssetReference,
    primitives: Vec<ArtifactCacheSceneMeshPrimitiveBindingAsset>,
}

impl From<&crate::asset::SceneMeshLodLevelAsset> for ArtifactCacheSceneMeshLodLevelAsset {
    fn from(asset: &crate::asset::SceneMeshLodLevelAsset) -> Self {
        Self {
            min_distance: asset.min_distance,
            model: asset.model.clone(),
            mesh: asset.mesh.clone(),
            material: asset.material.clone(),
            primitives: asset
                .primitives
                .iter()
                .map(ArtifactCacheSceneMeshPrimitiveBindingAsset::from)
                .collect(),
        }
    }
}

impl ArtifactCacheSceneMeshLodLevelAsset {
    fn into_asset(self) -> crate::asset::SceneMeshLodLevelAsset {
        crate::asset::SceneMeshLodLevelAsset {
            min_distance: self.min_distance,
            model: self.model,
            mesh: self.mesh,
            material: self.material,
            primitives: self
                .primitives
                .into_iter()
                .map(crate::asset::SceneMeshPrimitiveBindingAsset::from)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheSceneMeshInstanceAsset {
    model: AssetReference,
    mesh: Option<AssetReference>,
    material: AssetReference,
    render_queue: i32,
    material_queue: i32,
    order_in_layer: i32,
    depth_bias: crate::core::math::Real,
    morph_weights: Vec<crate::core::math::Real>,
    primitives: Vec<ArtifactCacheSceneMeshPrimitiveBindingAsset>,
    lods: Vec<ArtifactCacheSceneMeshLodLevelAsset>,
}

impl From<&crate::asset::SceneMeshInstanceAsset> for ArtifactCacheSceneMeshInstanceAsset {
    fn from(asset: &crate::asset::SceneMeshInstanceAsset) -> Self {
        Self {
            model: asset.model.clone(),
            mesh: asset.mesh.clone(),
            material: asset.material.clone(),
            render_queue: asset.render_queue,
            material_queue: asset.material_queue,
            order_in_layer: asset.order_in_layer,
            depth_bias: asset.depth_bias,
            morph_weights: asset.morph_weights.clone(),
            primitives: asset
                .primitives
                .iter()
                .map(ArtifactCacheSceneMeshPrimitiveBindingAsset::from)
                .collect(),
            lods: asset
                .lods
                .iter()
                .map(ArtifactCacheSceneMeshLodLevelAsset::from)
                .collect(),
        }
    }
}

impl ArtifactCacheSceneMeshInstanceAsset {
    fn into_asset(self) -> crate::asset::SceneMeshInstanceAsset {
        crate::asset::SceneMeshInstanceAsset {
            model: self.model,
            mesh: self.mesh,
            material: self.material,
            render_queue: self.render_queue,
            material_queue: self.material_queue,
            order_in_layer: self.order_in_layer,
            depth_bias: self.depth_bias,
            morph_weights: self.morph_weights,
            primitives: self
                .primitives
                .into_iter()
                .map(crate::asset::SceneMeshPrimitiveBindingAsset::from)
                .collect(),
            lods: self
                .lods
                .into_iter()
                .map(ArtifactCacheSceneMeshLodLevelAsset::into_asset)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheSceneCameraAsset {
    #[serde(default)]
    core_pipeline: crate::core::framework::render::CorePipelineKind,
    projection_mode: crate::core::framework::render::ProjectionMode,
    fov_y_radians: crate::core::math::Real,
    ortho_size: crate::core::math::Real,
    z_near: crate::core::math::Real,
    z_far: crate::core::math::Real,
    target: ArtifactCacheSceneCameraTargetAsset,
    viewport: Option<crate::asset::SceneViewportRectAsset>,
    order: i32,
    active: bool,
    hdr: bool,
    exposure_ev100: crate::core::math::Real,
    clear_color: crate::core::framework::render::RenderCameraClearColor,
    msaa_samples: u32,
    post_process_settings: Option<crate::asset::ScenePostProcessSettingsAsset>,
}

impl From<&crate::asset::SceneCameraAsset> for ArtifactCacheSceneCameraAsset {
    fn from(asset: &crate::asset::SceneCameraAsset) -> Self {
        Self {
            core_pipeline: asset.core_pipeline,
            projection_mode: asset.projection_mode,
            fov_y_radians: asset.fov_y_radians,
            ortho_size: asset.ortho_size,
            z_near: asset.z_near,
            z_far: asset.z_far,
            target: ArtifactCacheSceneCameraTargetAsset::from(&asset.target),
            viewport: asset.viewport,
            order: asset.order,
            active: asset.active,
            hdr: asset.hdr,
            exposure_ev100: asset.exposure_ev100,
            clear_color: asset.clear_color,
            msaa_samples: asset.msaa_samples,
            post_process_settings: asset.post_process_settings,
        }
    }
}

impl ArtifactCacheSceneCameraAsset {
    fn into_asset(self) -> crate::asset::SceneCameraAsset {
        crate::asset::SceneCameraAsset {
            core_pipeline: self.core_pipeline,
            projection_mode: self.projection_mode,
            fov_y_radians: self.fov_y_radians,
            ortho_size: self.ortho_size,
            z_near: self.z_near,
            z_far: self.z_far,
            target: self.target.into_asset(),
            viewport: self.viewport,
            order: self.order,
            active: self.active,
            hdr: self.hdr,
            exposure_ev100: self.exposure_ev100,
            clear_color: self.clear_color,
            msaa_samples: self.msaa_samples,
            post_process_settings: self.post_process_settings,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ArtifactCacheSceneCameraTargetAsset {
    PrimarySurface,
    Texture { texture: AssetReference },
    Headless { size: [u32; 2] },
}

impl From<&crate::asset::SceneCameraTargetAsset> for ArtifactCacheSceneCameraTargetAsset {
    fn from(target: &crate::asset::SceneCameraTargetAsset) -> Self {
        match target {
            crate::asset::SceneCameraTargetAsset::PrimarySurface => Self::PrimarySurface,
            crate::asset::SceneCameraTargetAsset::Texture { texture } => Self::Texture {
                texture: texture.clone(),
            },
            crate::asset::SceneCameraTargetAsset::Headless { size } => {
                Self::Headless { size: *size }
            }
        }
    }
}

impl ArtifactCacheSceneCameraTargetAsset {
    fn into_asset(self) -> crate::asset::SceneCameraTargetAsset {
        match self {
            Self::PrimarySurface => crate::asset::SceneCameraTargetAsset::PrimarySurface,
            Self::Texture { texture } => crate::asset::SceneCameraTargetAsset::Texture { texture },
            Self::Headless { size } => crate::asset::SceneCameraTargetAsset::Headless { size },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheSceneColliderAsset {
    shape: ArtifactCacheSceneColliderShapeAsset,
    sensor: bool,
    layer: u32,
    collision_group: u32,
    collision_mask: u32,
    material: Option<AssetReference>,
    material_override: Option<crate::core::framework::scene::physics::PhysicsMaterialMetadata>,
    local_transform: crate::asset::TransformAsset,
}

impl From<&crate::asset::SceneColliderAsset> for ArtifactCacheSceneColliderAsset {
    fn from(asset: &crate::asset::SceneColliderAsset) -> Self {
        Self {
            shape: ArtifactCacheSceneColliderShapeAsset::from(&asset.shape),
            sensor: asset.sensor,
            layer: asset.layer,
            collision_group: asset.collision_group,
            collision_mask: asset.collision_mask,
            material: asset.material.clone(),
            material_override: asset.material_override.clone(),
            local_transform: asset.local_transform,
        }
    }
}

impl ArtifactCacheSceneColliderAsset {
    fn into_asset(self) -> crate::asset::SceneColliderAsset {
        crate::asset::SceneColliderAsset {
            shape: self.shape.into_asset(),
            sensor: self.sensor,
            layer: self.layer,
            collision_group: self.collision_group,
            collision_mask: self.collision_mask,
            material: self.material,
            material_override: self.material_override,
            local_transform: self.local_transform,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ArtifactCacheSceneColliderShapeAsset {
    Box {
        half_extents: [crate::core::math::Real; 3],
    },
    Sphere {
        radius: crate::core::math::Real,
    },
    Capsule {
        radius: crate::core::math::Real,
        half_height: crate::core::math::Real,
    },
    Cylinder {
        radius: crate::core::math::Real,
        half_height: crate::core::math::Real,
    },
    ConvexHull {
        points: Vec<[crate::core::math::Real; 3]>,
    },
    TriangleMesh {
        mesh: crate::asset::AssetReference,
    },
    HeightField {
        resolution: [u32; 2],
        heights: crate::asset::AssetReference,
    },
    Compound {
        children: Vec<(
            crate::asset::TransformAsset,
            Box<ArtifactCacheSceneColliderShapeAsset>,
        )>,
    },
}

impl From<&crate::asset::SceneColliderShapeAsset> for ArtifactCacheSceneColliderShapeAsset {
    fn from(shape: &crate::asset::SceneColliderShapeAsset) -> Self {
        match shape {
            crate::asset::SceneColliderShapeAsset::Box { half_extents } => Self::Box {
                half_extents: *half_extents,
            },
            crate::asset::SceneColliderShapeAsset::Sphere { radius } => {
                Self::Sphere { radius: *radius }
            }
            crate::asset::SceneColliderShapeAsset::Capsule {
                radius,
                half_height,
            } => Self::Capsule {
                radius: *radius,
                half_height: *half_height,
            },
            crate::asset::SceneColliderShapeAsset::Cylinder {
                radius,
                half_height,
            } => Self::Cylinder {
                radius: *radius,
                half_height: *half_height,
            },
            crate::asset::SceneColliderShapeAsset::ConvexHull { points } => Self::ConvexHull {
                points: points.clone(),
            },
            crate::asset::SceneColliderShapeAsset::TriangleMesh { mesh } => {
                Self::TriangleMesh { mesh: mesh.clone() }
            }
            crate::asset::SceneColliderShapeAsset::HeightField {
                resolution,
                heights,
            } => Self::HeightField {
                resolution: *resolution,
                heights: heights.clone(),
            },
            crate::asset::SceneColliderShapeAsset::Compound { children } => Self::Compound {
                children: children
                    .iter()
                    .map(|(transform, shape)| {
                        (
                            *transform,
                            Box::new(ArtifactCacheSceneColliderShapeAsset::from(shape.as_ref())),
                        )
                    })
                    .collect(),
            },
        }
    }
}

impl ArtifactCacheSceneColliderShapeAsset {
    fn into_asset(self) -> crate::asset::SceneColliderShapeAsset {
        match self {
            Self::Box { half_extents } => {
                crate::asset::SceneColliderShapeAsset::Box { half_extents }
            }
            Self::Sphere { radius } => crate::asset::SceneColliderShapeAsset::Sphere { radius },
            Self::Capsule {
                radius,
                half_height,
            } => crate::asset::SceneColliderShapeAsset::Capsule {
                radius,
                half_height,
            },
            Self::Cylinder {
                radius,
                half_height,
            } => crate::asset::SceneColliderShapeAsset::Cylinder {
                radius,
                half_height,
            },
            Self::ConvexHull { points } => {
                crate::asset::SceneColliderShapeAsset::ConvexHull { points }
            }
            Self::TriangleMesh { mesh } => {
                crate::asset::SceneColliderShapeAsset::TriangleMesh { mesh }
            }
            Self::HeightField {
                resolution,
                heights,
            } => crate::asset::SceneColliderShapeAsset::HeightField {
                resolution,
                heights,
            },
            Self::Compound { children } => crate::asset::SceneColliderShapeAsset::Compound {
                children: children
                    .into_iter()
                    .map(|(transform, shape)| (transform, Box::new(shape.into_asset())))
                    .collect(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheSceneJointAsset {
    joint_type: crate::asset::SceneJointKindAsset,
    connected_entity: Option<u64>,
    anchor: [crate::core::math::Real; 3],
    axis: [crate::core::math::Real; 3],
    limits: Option<[crate::core::math::Real; 2]>,
    collide_connected: bool,
    constraint: ArtifactCachePhysicsJointConstraintMetadata,
    skeleton_binding: Option<crate::core::framework::scene::physics::PhysicsSkeletonJointBinding>,
}

impl From<&crate::asset::SceneJointAsset> for ArtifactCacheSceneJointAsset {
    fn from(asset: &crate::asset::SceneJointAsset) -> Self {
        Self {
            joint_type: asset.joint_type,
            connected_entity: asset.connected_entity,
            anchor: asset.anchor,
            axis: asset.axis,
            limits: asset.limits,
            collide_connected: asset.collide_connected,
            constraint: ArtifactCachePhysicsJointConstraintMetadata::from(&asset.constraint),
            skeleton_binding: asset.skeleton_binding.clone(),
        }
    }
}

impl ArtifactCacheSceneJointAsset {
    fn into_asset(self) -> crate::asset::SceneJointAsset {
        crate::asset::SceneJointAsset {
            joint_type: self.joint_type,
            connected_entity: self.connected_entity,
            anchor: self.anchor,
            axis: self.axis,
            limits: self.limits,
            collide_connected: self.collide_connected,
            constraint: self.constraint.into(),
            skeleton_binding: self.skeleton_binding,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCachePhysicsJointConstraintMetadata {
    linear_limits: [Option<[crate::core::math::Real; 2]>; 3],
    angular_limits: [Option<[crate::core::math::Real; 2]>; 3],
    linear_drives: [crate::core::framework::scene::physics::PhysicsJointDrive; 3],
    angular_drives: [crate::core::framework::scene::physics::PhysicsJointDrive; 3],
    break_force: Option<crate::core::math::Real>,
    break_torque: Option<crate::core::math::Real>,
    projection_linear_tolerance: Option<crate::core::math::Real>,
    projection_angular_tolerance: Option<crate::core::math::Real>,
}

impl From<&crate::core::framework::scene::physics::PhysicsJointConstraintMetadata>
    for ArtifactCachePhysicsJointConstraintMetadata
{
    fn from(
        metadata: &crate::core::framework::scene::physics::PhysicsJointConstraintMetadata,
    ) -> Self {
        Self {
            linear_limits: metadata.linear_limits,
            angular_limits: metadata.angular_limits,
            linear_drives: metadata.linear_drives,
            angular_drives: metadata.angular_drives,
            break_force: metadata.break_force,
            break_torque: metadata.break_torque,
            projection_linear_tolerance: metadata.projection_linear_tolerance,
            projection_angular_tolerance: metadata.projection_angular_tolerance,
        }
    }
}

impl From<ArtifactCachePhysicsJointConstraintMetadata>
    for crate::core::framework::scene::physics::PhysicsJointConstraintMetadata
{
    fn from(metadata: ArtifactCachePhysicsJointConstraintMetadata) -> Self {
        Self {
            linear_limits: metadata.linear_limits,
            angular_limits: metadata.angular_limits,
            linear_drives: metadata.linear_drives,
            angular_drives: metadata.angular_drives,
            break_force: metadata.break_force,
            break_torque: metadata.break_torque,
            projection_linear_tolerance: metadata.projection_linear_tolerance,
            projection_angular_tolerance: metadata.projection_angular_tolerance,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheSceneScriptBindingAsset {
    package: String,
    module: String,
    enabled: bool,
    update: bool,
    fixed_update: bool,
    properties: BTreeMap<String, ArtifactCacheJsonValue>,
}

impl From<&crate::asset::SceneScriptBindingAsset> for ArtifactCacheSceneScriptBindingAsset {
    fn from(asset: &crate::asset::SceneScriptBindingAsset) -> Self {
        Self {
            package: asset.package.clone(),
            module: asset.module.clone(),
            enabled: asset.enabled,
            update: asset.update,
            fixed_update: asset.fixed_update,
            properties: json_table_to_cache(&asset.properties),
        }
    }
}

impl ArtifactCacheSceneScriptBindingAsset {
    fn into_asset(self) -> Result<crate::asset::SceneScriptBindingAsset, AssetImportError> {
        Ok(crate::asset::SceneScriptBindingAsset {
            package: self.package,
            module: self.module,
            enabled: self.enabled,
            update: self.update,
            fixed_update: self.fixed_update,
            properties: cache_table_to_json(self.properties)?,
        })
    }
}
