use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::assets::{
    AlphaMode, AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationGraphAsset,
    AnimationGraphNodeAsset, AnimationGraphParameterAsset, AnimationInterpolationAsset,
    AnimationSequenceAsset, AnimationSequenceBindingAsset, AnimationSequenceTrackAsset,
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset, AnimationStateAsset,
    AnimationStateMachineAsset, AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
    MaterialAsset, PhysicsMaterialAsset, SceneAnimationGraphPlayerAsset, SceneAnimationPlayerAsset,
    SceneAnimationSequencePlayerAsset, SceneAnimationSkeletonAsset,
    SceneAnimationStateMachinePlayerAsset, SceneAsset, SceneCameraAsset, SceneColliderAsset,
    SceneColliderShapeAsset, SceneEntityAsset, SceneJointAsset, SceneJointKindAsset,
    SceneMeshInstanceAsset, SceneMobilityAsset, SceneRigidBodyAsset, SceneRigidBodyTypeAsset,
    TransformAsset,
};
use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::{AssetReference, AssetUri};
use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::physics::{PhysicsCombineRule, PhysicsMaterialMetadata};
use crate::core::framework::scene::ComponentPropertyPath;
use crate::core::framework::scene::EntityPath;
use crate::core::resource::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, MaterialMarker, ModelMarker, PhysicsMaterialMarker,
    ResourceHandle, ResourceId,
};
use image::{ImageBuffer, ImageFormat, Rgba};

use crate::scene::components::default_render_layer_mask;

pub(super) fn model_handle(label: &str) -> ResourceHandle<ModelMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}

pub(super) fn material_handle(label: &str) -> ResourceHandle<MaterialMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

pub(super) fn project_model_handle(
    project: &ProjectManager,
    uri: &str,
) -> ResourceHandle<ModelMarker> {
    let uri = AssetUri::parse(uri).unwrap();
    ResourceHandle::new(project.asset_id_for_uri(&uri).unwrap())
}

pub(super) fn project_material_handle(
    project: &ProjectManager,
    uri: &str,
) -> ResourceHandle<MaterialMarker> {
    let uri = AssetUri::parse(uri).unwrap();
    ResourceHandle::new(project.asset_id_for_uri(&uri).unwrap())
}

pub(super) fn project_physics_material_handle(
    project: &ProjectManager,
    uri: &str,
) -> ResourceHandle<PhysicsMaterialMarker> {
    let uri = AssetUri::parse(uri).unwrap();
    ResourceHandle::new(project.asset_id_for_uri(&uri).unwrap())
}

pub(super) fn project_animation_skeleton_handle(
    project: &ProjectManager,
    uri: &str,
) -> ResourceHandle<AnimationSkeletonMarker> {
    let uri = AssetUri::parse(uri).unwrap();
    ResourceHandle::new(project.asset_id_for_uri(&uri).unwrap())
}

pub(super) fn project_animation_clip_handle(
    project: &ProjectManager,
    uri: &str,
) -> ResourceHandle<AnimationClipMarker> {
    let uri = AssetUri::parse(uri).unwrap();
    ResourceHandle::new(project.asset_id_for_uri(&uri).unwrap())
}

pub(super) fn project_animation_sequence_handle(
    project: &ProjectManager,
    uri: &str,
) -> ResourceHandle<AnimationSequenceMarker> {
    let uri = AssetUri::parse(uri).unwrap();
    ResourceHandle::new(project.asset_id_for_uri(&uri).unwrap())
}

pub(super) fn project_animation_graph_handle(
    project: &ProjectManager,
    uri: &str,
) -> ResourceHandle<AnimationGraphMarker> {
    let uri = AssetUri::parse(uri).unwrap();
    ResourceHandle::new(project.asset_id_for_uri(&uri).unwrap())
}

pub(super) fn project_animation_state_machine_handle(
    project: &ProjectManager,
    uri: &str,
) -> ResourceHandle<AnimationStateMachineMarker> {
    let uri = AssetUri::parse(uri).unwrap();
    ResourceHandle::new(project.asset_id_for_uri(&uri).unwrap())
}

pub(super) fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_scene_{label}_{unique}"))
}

pub(super) fn create_test_project(root: &PathBuf) -> ProjectManager {
    let paths = ProjectPaths::from_root(root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "SceneSandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_default_material(
        paths
            .assets_root()
            .join("materials")
            .join("grid.material.toml"),
    );
    write_default_physics_material(
        paths
            .assets_root()
            .join("physics")
            .join("default.physics_material.toml"),
    );
    write_default_animation_skeleton(
        paths
            .assets_root()
            .join("animation")
            .join("hero.skeleton.zranim"),
    );
    write_default_animation_clip(
        paths
            .assets_root()
            .join("animation")
            .join("hero.clip.zranim"),
    );
    write_default_animation_sequence(
        paths
            .assets_root()
            .join("animation")
            .join("hero.sequence.zranim"),
    );
    write_default_animation_graph(
        paths
            .assets_root()
            .join("animation")
            .join("hero.graph.zranim"),
    );
    write_default_animation_state_machine(
        paths
            .assets_root()
            .join("animation")
            .join("hero.state_machine.zranim"),
    );
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let mut project = ProjectManager::open(root).unwrap();
    project.scan_and_import().unwrap();
    project
}

fn write_valid_wgsl(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    let x = f32(i32(vertex_index) - 1);
    return vec4f(x, 0.0, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(1.0, 0.4, 0.2, 1.0);
}
"#,
    )
    .unwrap();
}

fn write_checker_png(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |x, y| {
        if (x + y) % 2 == 0 {
            Rgba([255, 255, 255, 255])
        } else {
            Rgba([0, 0, 0, 255])
        }
    })
    .save_with_format(path, ImageFormat::Png)
    .unwrap();
}

fn write_triangle_obj(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        "\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.0 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
",
    )
    .unwrap();
}

fn write_default_material(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some("Grid".to_string()),
        shader: asset_reference("res://shaders/pbr.wgsl"),
        base_color: [0.8, 0.8, 0.8, 1.0],
        base_color_texture: Some(asset_reference("res://textures/checker.png")),
        normal_texture: None,
        metallic: 0.1,
        roughness: 0.8,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
    };
    fs::write(path, material.to_toml_string().unwrap()).unwrap();
}

fn write_default_scene(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let scene = SceneAsset {
        entities: vec![
            SceneEntityAsset {
                entity: 1,
                name: "Camera".to_string(),
                parent: None,
                transform: TransformAsset {
                    translation: [0.0, 2.0, 5.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                active: true,
                render_layer_mask: default_render_layer_mask(),
                mobility: SceneMobilityAsset::Dynamic,
                camera: Some(SceneCameraAsset {
                    fov_y_radians: 1.0471976,
                    z_near: 0.1,
                    z_far: 200.0,
                }),
                mesh: None,
                directional_light: None,
                point_light: None,
                spot_light: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
                terrain: None,
                tilemap: None,
                prefab_instance: None,
            },
            SceneEntityAsset {
                entity: 2,
                name: "Triangle".to_string(),
                parent: None,
                transform: TransformAsset {
                    translation: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                active: true,
                render_layer_mask: default_render_layer_mask(),
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: Some(SceneMeshInstanceAsset {
                    model: asset_reference("res://models/triangle.obj"),
                    material: asset_reference("res://materials/grid.material.toml"),
                }),
                directional_light: None,
                point_light: None,
                spot_light: None,
                rigid_body: Some(SceneRigidBodyAsset {
                    body_type: SceneRigidBodyTypeAsset::Dynamic,
                    mass: 2.5,
                    linear_velocity: [0.25, 0.0, 0.0],
                    angular_velocity: [0.0, 0.25, 0.0],
                    linear_damping: 0.15,
                    angular_damping: 0.05,
                    gravity_scale: 1.0,
                    can_sleep: true,
                    lock_translation: [false, false, false],
                    lock_rotation: [false, true, false],
                }),
                collider: Some(SceneColliderAsset {
                    shape: SceneColliderShapeAsset::Box {
                        half_extents: [0.5, 0.5, 0.5],
                    },
                    sensor: false,
                    layer: 2,
                    collision_group: 4,
                    collision_mask: 0x0000_00ff,
                    material: Some(asset_reference(
                        "res://physics/default.physics_material.toml",
                    )),
                    material_override: Some(PhysicsMaterialMetadata {
                        static_friction: 0.7,
                        dynamic_friction: 0.4,
                        restitution: 0.2,
                        friction_combine: PhysicsCombineRule::Maximum,
                        restitution_combine: PhysicsCombineRule::Average,
                    }),
                    local_transform: TransformAsset {
                        translation: [0.0, 0.5, 0.0],
                        rotation: [0.0, 0.0, 0.0, 1.0],
                        scale: [1.0, 1.0, 1.0],
                    },
                }),
                joint: Some(SceneJointAsset {
                    joint_type: SceneJointKindAsset::Fixed,
                    connected_entity: Some(1),
                    anchor: [0.0, 0.5, 0.0],
                    axis: [0.0, 1.0, 0.0],
                    limits: None,
                    collide_connected: false,
                }),
                animation_skeleton: Some(SceneAnimationSkeletonAsset {
                    skeleton: asset_reference("res://animation/hero.skeleton.zranim"),
                }),
                animation_player: Some(SceneAnimationPlayerAsset {
                    clip: asset_reference("res://animation/hero.clip.zranim"),
                    playback_speed: 1.25,
                    time_seconds: 0.5,
                    weight: 0.8,
                    looping: true,
                    playing: true,
                }),
                animation_sequence_player: Some(SceneAnimationSequencePlayerAsset {
                    sequence: asset_reference("res://animation/hero.sequence.zranim"),
                    playback_speed: 1.0,
                    time_seconds: 0.25,
                    looping: false,
                    playing: true,
                }),
                animation_graph_player: Some(SceneAnimationGraphPlayerAsset {
                    graph: asset_reference("res://animation/hero.graph.zranim"),
                    parameters: std::collections::BTreeMap::from([
                        ("grounded".to_string(), AnimationParameterValue::Bool(true)),
                        ("speed".to_string(), AnimationParameterValue::Scalar(1.5)),
                    ]),
                    playing: true,
                }),
                animation_state_machine_player: Some(SceneAnimationStateMachinePlayerAsset {
                    state_machine: asset_reference("res://animation/hero.state_machine.zranim"),
                    parameters: std::collections::BTreeMap::from([
                        ("grounded".to_string(), AnimationParameterValue::Bool(true)),
                        ("speed".to_string(), AnimationParameterValue::Scalar(1.5)),
                    ]),
                    active_state: Some("Locomotion".to_string()),
                    playing: true,
                }),
                terrain: None,
                tilemap: None,
                prefab_instance: None,
            },
        ],
    };
    fs::write(path, scene.to_toml_string().unwrap()).unwrap();
}

fn write_default_physics_material(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = PhysicsMaterialAsset {
        name: Some("DefaultPhysics".to_string()),
        metadata: PhysicsMaterialMetadata {
            static_friction: 0.9,
            dynamic_friction: 0.6,
            restitution: 0.2,
            friction_combine: PhysicsCombineRule::Maximum,
            restitution_combine: PhysicsCombineRule::Average,
        },
    };
    fs::write(path, material.to_toml_string().unwrap()).unwrap();
}

fn write_default_animation_skeleton(path: PathBuf) {
    write_animation_bytes(path, sample_animation_skeleton_asset().to_bytes().unwrap());
}

fn write_default_animation_clip(path: PathBuf) {
    write_animation_bytes(path, sample_animation_clip_asset().to_bytes().unwrap());
}

fn write_default_animation_sequence(path: PathBuf) {
    write_animation_bytes(path, sample_animation_sequence_asset().to_bytes().unwrap());
}

fn write_default_animation_graph(path: PathBuf) {
    write_animation_bytes(path, sample_animation_graph_asset().to_bytes().unwrap());
}

fn write_default_animation_state_machine(path: PathBuf) {
    write_animation_bytes(
        path,
        sample_animation_state_machine_asset().to_bytes().unwrap(),
    );
}

fn write_animation_bytes(path: PathBuf, bytes: Vec<u8>) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, bytes).unwrap();
}

fn sample_animation_skeleton_asset() -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: Some("HeroSkeleton".to_string()),
        bones: vec![
            AnimationSkeletonBoneAsset {
                name: "Root".to_string(),
                parent_index: None,
                local_translation: [0.0, 0.0, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0, 1.0, 1.0],
            },
            AnimationSkeletonBoneAsset {
                name: "Hand".to_string(),
                parent_index: Some(0),
                local_translation: [0.2, 0.8, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0, 1.0, 1.0],
            },
        ],
    }
}

fn sample_animation_clip_asset() -> AnimationClipAsset {
    AnimationClipAsset {
        name: Some("HeroIdle".to_string()),
        skeleton: asset_reference("res://animation/hero.skeleton.zranim"),
        duration_seconds: 1.0,
        tracks: vec![AnimationClipBoneTrackAsset {
            bone_name: "Hand".to_string(),
            translation: vec3_channel([(0.0, [0.2, 0.8, 0.0]), (1.0, [0.25, 0.85, 0.0])]),
            rotation: quaternion_channel([
                (0.0, [0.0, 0.0, 0.0, 1.0]),
                (1.0, [0.0, 0.38268343, 0.0, 0.9238795]),
            ]),
            scale: vec3_channel([(0.0, [1.0, 1.0, 1.0]), (1.0, [1.05, 1.05, 1.05])]),
        }],
    }
}

fn sample_animation_sequence_asset() -> AnimationSequenceAsset {
    AnimationSequenceAsset {
        name: Some("HeroSequence".to_string()),
        duration_seconds: 2.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            tracks: vec![
                AnimationSequenceTrackAsset {
                    property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
                    channel: vec3_channel([(0.0, [0.0, 0.0, 0.0]), (1.0, [1.0, 0.0, 0.0])]),
                },
                AnimationSequenceTrackAsset {
                    property_path: ComponentPropertyPath::parse("AnimationPlayer.weight").unwrap(),
                    channel: scalar_channel([(0.0, 0.0), (1.0, 1.0)]),
                },
            ],
        }],
    }
}

fn sample_animation_graph_asset() -> AnimationGraphAsset {
    AnimationGraphAsset {
        name: Some("HeroGraph".to_string()),
        parameters: vec![
            AnimationGraphParameterAsset {
                name: "speed".to_string(),
                default_value: AnimationParameterValue::Scalar(1.0),
            },
            AnimationGraphParameterAsset {
                name: "grounded".to_string(),
                default_value: AnimationParameterValue::Bool(true),
            },
        ],
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "idle".to_string(),
                clip: asset_reference("res://animation/hero.clip.zranim"),
                playback_speed: 1.0,
                looping: true,
            },
            AnimationGraphNodeAsset::Blend {
                id: "blend".to_string(),
                inputs: vec!["idle".to_string()],
                weight_parameter: Some("speed".to_string()),
            },
            AnimationGraphNodeAsset::Output {
                source: "blend".to_string(),
            },
        ],
    }
}

fn sample_animation_state_machine_asset() -> AnimationStateMachineAsset {
    AnimationStateMachineAsset {
        name: Some("HeroStateMachine".to_string()),
        entry_state: "Locomotion".to_string(),
        states: vec![AnimationStateAsset {
            name: "Locomotion".to_string(),
            graph: asset_reference("res://animation/hero.graph.zranim"),
        }],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Locomotion".to_string(),
            to_state: "Locomotion".to_string(),
            duration_seconds: 0.1,
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "grounded".to_string(),
                operator: crate::asset::AnimationConditionOperatorAsset::Equal,
                value: Some(AnimationParameterValue::Bool(true)),
            }],
        }],
    }
}

fn scalar_channel(keys: [(f32, f32); 2]) -> crate::asset::AnimationChannelAsset {
    crate::asset::AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Hermite,
        keys: keys
            .into_iter()
            .map(
                |(time_seconds, value)| crate::asset::AnimationChannelKeyAsset {
                    time_seconds,
                    value: crate::asset::AnimationChannelValueAsset::Scalar(value),
                    in_tangent: Some(crate::asset::AnimationChannelValueAsset::Scalar(0.0)),
                    out_tangent: Some(crate::asset::AnimationChannelValueAsset::Scalar(0.0)),
                },
            )
            .collect(),
    }
}

fn vec3_channel(keys: [(f32, [f32; 3]); 2]) -> crate::asset::AnimationChannelAsset {
    crate::asset::AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Hermite,
        keys: keys
            .into_iter()
            .map(
                |(time_seconds, value)| crate::asset::AnimationChannelKeyAsset {
                    time_seconds,
                    value: crate::asset::AnimationChannelValueAsset::Vec3(value),
                    in_tangent: Some(crate::asset::AnimationChannelValueAsset::Vec3([
                        0.0, 0.0, 0.0,
                    ])),
                    out_tangent: Some(crate::asset::AnimationChannelValueAsset::Vec3([
                        0.0, 0.0, 0.0,
                    ])),
                },
            )
            .collect(),
    }
}

fn quaternion_channel(keys: [(f32, [f32; 4]); 2]) -> crate::asset::AnimationChannelAsset {
    crate::asset::AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Hermite,
        keys: keys
            .into_iter()
            .map(
                |(time_seconds, value)| crate::asset::AnimationChannelKeyAsset {
                    time_seconds,
                    value: crate::asset::AnimationChannelValueAsset::Quaternion(value),
                    in_tangent: None,
                    out_tangent: None,
                },
            )
            .collect(),
    }
}
