use std::fs;
use std::path::{Path, PathBuf};

use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::audio::AudioChannelLayout;
use crate::core::framework::scene::physics::{PhysicsCombineRule, PhysicsMaterialMetadata};
use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};
use image::{ImageBuffer, ImageFormat, Rgba};

use crate::asset::{
    AlphaMode, AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporter,
    AssetImporterDescriptor, AssetKind, AssetReference, AssetUri, FunctionAssetImporter,
    ImportedAsset, MaterialAsset, PhysicsMaterialAsset, ProjectManifest, ReferenceResolutionError,
    SceneAsset, SceneCameraAsset, SceneDirectionalLightAsset, SceneEntityAsset,
    SceneMeshInstanceAsset, SceneMobilityAsset, SoundAsset, TransformAsset, UiV2ComponentAsset,
    UiV2StyleAsset, UiV2ViewAsset, ZMaterialDocument,
};
use crate::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationGraphAsset, AnimationGraphNodeAsset,
    AnimationGraphParameterAsset, AnimationInterpolationAsset, AnimationSequenceAsset,
    AnimationSequenceBindingAsset, AnimationSequenceTrackAsset, AnimationSkeletonAsset,
    AnimationSkeletonBoneAsset, AnimationStateAsset, AnimationStateKindAsset,
    AnimationStateMachineAsset, AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
};
use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;
use zircon_runtime_interface::ui::v2::UiV2AssetKind;

pub(crate) fn write_valid_wgsl(path: PathBuf) {
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

pub(crate) fn write_checker_png(path: PathBuf) {
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

pub(crate) fn importer_with_first_wave_plugin_fixtures() -> AssetImporter {
    let mut importer = AssetImporter::default();
    importer
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    importer
}

#[cfg(feature = "ui")]
pub(crate) fn ui_document_importer_fixture() -> FunctionAssetImporter {
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new(
            "ui_document_importer.zui_document",
            "ui_document_importer",
            AssetKind::UiWidget,
            2,
        )
        .with_priority(120)
        .with_full_suffixes([".zui"])
        .with_additional_output_kinds([AssetKind::UiLayout, AssetKind::UiStyle])
        .with_required_capabilities(["runtime.asset.importer.ui_document"]),
        import_ui_zui_document_fixture,
    )
}

#[cfg(feature = "ui")]
fn import_ui_zui_document_fixture(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let parsed = crate::ui::v2::UiZuiAssetLoader::load_zui_str(&document).map_err(|source| {
        AssetImportError::UiV2Document {
            context: "parse .zui ui asset fixture",
            source: source.into(),
        }
    })?;
    let imported = match parsed.asset.kind {
        UiV2AssetKind::View => ImportedAsset::UiV2View(UiV2ViewAsset { document: parsed }),
        UiV2AssetKind::Style | UiV2AssetKind::ThemeTokens => {
            ImportedAsset::UiV2Style(UiV2StyleAsset { document: parsed })
        }
        UiV2AssetKind::Component => {
            ImportedAsset::UiV2Component(UiV2ComponentAsset { document: parsed })
        }
    };
    Ok(AssetImportOutcome::new(context.uri.clone(), imported))
}

pub(crate) fn write_triangle_obj(path: PathBuf) {
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

pub(crate) fn sample_sound_asset(uri: &str) -> SoundAsset {
    SoundAsset {
        uri: AssetUri::parse(uri).unwrap(),
        sample_rate_hz: 48_000,
        channel_count: 1,
        channel_layout: AudioChannelLayout::mono(),
        samples: vec![0.0, 0.5, -0.5, 32767.0 / 32768.0],
    }
}

pub(crate) fn write_test_wav(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let sample_rate_hz = 48_000_u32;
    let channel_count = 1_u16;
    let bits_per_sample = 16_u16;
    let block_align = channel_count * (bits_per_sample / 8);
    let byte_rate = sample_rate_hz * block_align as u32;
    let samples = [0_i16, 16_384_i16, -16_384_i16, 32_767_i16];
    let data_size = (samples.len() * std::mem::size_of::<i16>()) as u32;
    let riff_size = 36 + data_size;

    let mut bytes = Vec::with_capacity((riff_size + 8) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&channel_count.to_le_bytes());
    bytes.extend_from_slice(&sample_rate_hz.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }

    fs::write(path, bytes).unwrap();
}

pub(crate) fn write_default_material(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some("Grid".to_string()),
        shader: asset_reference("res://shaders/pbr.wgsl"),
        parent: None,
        options: Default::default(),
        queue: None,
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
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    write_project_material(&path, &material);
}

pub(crate) fn read_project_material(path: &Path) -> MaterialAsset {
    let document = fs::read_to_string(path).unwrap();
    let material = ZMaterialDocument::from_project_toml_str(&document, |reference| {
        Ok::<_, ReferenceResolutionError>(runtime_reference_for_fixture(reference))
    })
    .unwrap();
    MaterialAsset::from_zmaterial_document(material)
}

pub(crate) fn write_project_material(path: &Path, material: &MaterialAsset) {
    let project_root = fixture_project_root(path);
    let document = material
        .to_project_toml_string(|reference| {
            Ok::<_, ReferenceResolutionError>(persisted_reference_for_fixture(
                &project_root,
                path,
                reference,
            ))
        })
        .unwrap();
    fs::write(path, document).unwrap();
}

pub(crate) fn write_default_scene(path: PathBuf) {
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
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: Some(SceneCameraAsset {
                    fov_y_radians: 1.0471976,
                    z_near: 0.1,
                    z_far: 200.0,
                    post_process_settings: None,
                    ..SceneCameraAsset::default()
                }),
                mesh: None,
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                post_process_volume: None,
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
                script_bindings: Vec::new(),
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
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: Some(SceneMeshInstanceAsset {
                    model: asset_reference("res://models/triangle.obj"),
                    mesh: None,
                    material: asset_reference("res://materials/grid.zmaterial"),
                    render_queue: 0,
                    material_queue: 0,
                    order_in_layer: 0,
                    depth_bias: 0.0,
                    morph_weights: Vec::new(),
                    primitives: Vec::new(),
                    lods: Vec::new(),
                }),
                ambient_light: None,
                directional_light: None,
                point_light: None,
                rect_light: None,
                spot_light: None,
                post_process_volume: None,
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
                script_bindings: Vec::new(),
            },
        ],
    };
    write_project_scene(&path, &scene);
}

pub(crate) fn write_static_lit_default_scene(path: PathBuf) {
    write_default_scene(path.clone());
    let document = fs::read_to_string(&path).unwrap();
    let mut scene = SceneAsset::from_project_toml_str(&document, |reference| {
        Ok::<_, ReferenceResolutionError>(runtime_reference_for_fixture(reference))
    })
    .unwrap();
    scene.entities[1].mobility = SceneMobilityAsset::Static;
    scene.entities.push(SceneEntityAsset {
        entity: 3,
        name: "Sun".to_string(),
        parent: None,
        transform: TransformAsset {
            translation: [0.0, 4.0, 2.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
        active: true,
        render_layer_mask: 0x0000_0001,
        mobility: SceneMobilityAsset::Static,
        camera: None,
        mesh: None,
        ambient_light: None,
        directional_light: Some(SceneDirectionalLightAsset {
            direction: [-0.4, -1.0, -0.25],
            color: [1.0, 0.96, 0.9],
            intensity: 3.0,
            volumetric: false,
        }),
        point_light: None,
        rect_light: None,
        spot_light: None,
        post_process_volume: None,
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
        script_bindings: Vec::new(),
    });
    write_project_scene(&path, &scene);
}

fn write_project_scene(path: &Path, scene: &SceneAsset) {
    let project_root = fixture_project_root(path);
    let document = scene
        .to_project_toml_string(|reference| {
            Ok::<_, ReferenceResolutionError>(persisted_reference_for_fixture(
                &project_root,
                path,
                reference,
            ))
        })
        .unwrap();
    fs::write(path, document).unwrap();
}

pub(crate) fn sample_physics_material_asset() -> PhysicsMaterialAsset {
    PhysicsMaterialAsset {
        name: Some("DefaultPhysics".to_string()),
        metadata: PhysicsMaterialMetadata {
            static_friction: 0.9,
            dynamic_friction: 0.6,
            restitution: 0.2,
            friction_combine: PhysicsCombineRule::Maximum,
            restitution_combine: PhysicsCombineRule::Average,
        },
    }
}

pub(crate) fn write_default_physics_material(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        sample_physics_material_asset().to_toml_string().unwrap(),
    )
    .unwrap();
}

pub(crate) fn sample_animation_skeleton_asset() -> AnimationSkeletonAsset {
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

pub(crate) fn write_default_animation_skeleton(path: PathBuf) {
    write_animation_bytes(path, sample_animation_skeleton_asset().to_bytes().unwrap());
}

pub(crate) fn sample_animation_clip_asset() -> AnimationClipAsset {
    AnimationClipAsset {
        name: Some("HeroIdle".to_string()),
        skeleton: asset_reference("res://animation/hero.skeleton.zranim"),
        duration_seconds: 1.0,
        tracks: vec![AnimationClipBoneTrackAsset {
            bone_name: "Hand".to_string(),
            target_id: Some("Root/Hand".to_string()),
            translation: vec3_channel([(0.0, [0.2, 0.8, 0.0]), (1.0, [0.25, 0.85, 0.0])]),
            rotation: quaternion_channel([
                (0.0, [0.0, 0.0, 0.0, 1.0]),
                (1.0, [0.0, 0.38268343, 0.0, 0.9238795]),
            ]),
            scale: vec3_channel([(0.0, [1.0, 1.0, 1.0]), (1.0, [1.05, 1.05, 1.05])]),
        }],
        event_tracks: Vec::new(),
    }
}

pub(crate) fn write_default_animation_clip(path: PathBuf) {
    write_animation_bytes(path, sample_animation_clip_asset().to_bytes().unwrap());
}

pub(crate) fn sample_animation_sequence_asset() -> AnimationSequenceAsset {
    AnimationSequenceAsset {
        name: Some("HeroSequence".to_string()),
        duration_seconds: 2.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            target_id: Some("Root/Hero".to_string()),
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

pub(crate) fn write_default_animation_sequence(path: PathBuf) {
    write_animation_bytes(path, sample_animation_sequence_asset().to_bytes().unwrap());
}

pub(crate) fn sample_animation_graph_asset() -> AnimationGraphAsset {
    AnimationGraphAsset {
        name: Some("HeroGraph".to_string()),
        parameters: vec![
            AnimationGraphParameterAsset {
                name: "speed".to_string(),
                default_value: AnimationParameterValue::Scalar(1.0),
            },
            AnimationGraphParameterAsset {
                name: "advance".to_string(),
                default_value: AnimationParameterValue::Trigger,
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

pub(crate) fn write_default_animation_graph(path: PathBuf) {
    write_animation_bytes(path, sample_animation_graph_asset().to_bytes().unwrap());
}

pub(crate) fn sample_animation_state_machine_asset() -> AnimationStateMachineAsset {
    AnimationStateMachineAsset {
        name: Some("HeroStateMachine".to_string()),
        entry_state: "Locomotion".to_string(),
        states: vec![AnimationStateAsset {
            name: "Locomotion".to_string(),
            kind: AnimationStateKindAsset::GraphRef {
                graph: asset_reference("res://animation/hero.graph.zranim"),
            },
        }],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Locomotion".to_string(),
            to_state: "Locomotion".to_string(),
            duration_seconds: 0.1,
            exit_time: None,
            interruption: Default::default(),
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "advance".to_string(),
                operator:
                    crate::core::framework::animation::AnimationConditionOperatorAsset::Triggered,
                value: None,
            }],
        }],
        layers: Vec::new(),
    }
}

pub(crate) fn write_default_animation_state_machine(path: PathBuf) {
    write_animation_bytes(
        path,
        sample_animation_state_machine_asset().to_bytes().unwrap(),
    );
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

fn fixture_project_root(source_path: &Path) -> PathBuf {
    source_path
        .ancestors()
        .find(|candidate| candidate.join("zircon-project.toml").is_file())
        .expect("asset fixture must live below a project manifest")
        .to_path_buf()
}

fn persisted_reference_for_fixture(
    project_root: &Path,
    source_path: &Path,
    reference: &AssetReference,
) -> PersistedAssetReference {
    if reference.locator.scheme() == ResourceScheme::Builtin {
        return PersistedAssetReference::builtin(reference.locator.clone());
    }
    assert_eq!(reference.locator.scheme(), ResourceScheme::Res);

    let manifest = ProjectManifest::load(project_root.join("zircon-project.toml")).unwrap();
    let roots = manifest
        .asset_roots
        .iter()
        .filter(|root| {
            root.join_to(project_root)
                .join(reference.locator.path())
                .is_file()
        })
        .collect::<Vec<_>>();
    let root = match roots.as_slice() {
        [root] => *root,
        [] => manifest
            .asset_roots
            .iter()
            .find(|root| source_path.starts_with(root.join_to(project_root)))
            .expect("asset fixture source must live below a manifest asset root"),
        _ => panic!(
            "asset fixture reference {} resolves through multiple project asset roots",
            reference.locator
        ),
    };
    let path_hint =
        RelPath::parse(format!("{}/{}", root.as_str(), reference.locator.path())).unwrap();
    PersistedAssetReference::project(
        AssetRef::try_new(
            reference.uuid,
            path_hint,
            reference.locator.label().map(str::to_owned),
        )
        .unwrap(),
    )
}

fn runtime_reference_for_fixture(reference: &PersistedAssetReference) -> AssetReference {
    if let Some(locator) = reference.builtin_locator() {
        return AssetReference::from_locator(locator.clone());
    }
    let reference = reference.project_ref().expect("project fixture reference");
    let (_, relative) = reference
        .path_hint()
        .as_str()
        .split_once('/')
        .expect("project fixture reference must include its asset-root prefix");
    let locator = AssetUri::new(
        ResourceScheme::Res,
        relative.to_owned(),
        reference.sub().map(str::to_owned),
    )
    .unwrap();
    AssetReference::new(reference.guid(), locator)
}

fn write_animation_bytes(path: PathBuf, bytes: Vec<u8>) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, bytes).unwrap();
}

fn scalar_channel(keys: [(f32, f32); 2]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Hermite,
        keys: keys
            .into_iter()
            .map(|(time_seconds, value)| AnimationChannelKeyAsset {
                time_seconds,
                value: AnimationChannelValueAsset::Scalar(value),
                in_tangent: Some(AnimationChannelValueAsset::Scalar(0.0)),
                out_tangent: Some(AnimationChannelValueAsset::Scalar(0.0)),
            })
            .collect(),
    }
}

fn vec3_channel(keys: [(f32, [f32; 3]); 2]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Hermite,
        keys: keys
            .into_iter()
            .map(|(time_seconds, value)| AnimationChannelKeyAsset {
                time_seconds,
                value: AnimationChannelValueAsset::Vec3(value),
                in_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                out_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
            })
            .collect(),
    }
}

fn quaternion_channel(keys: [(f32, [f32; 4]); 2]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Hermite,
        keys: keys
            .into_iter()
            .map(|(time_seconds, value)| AnimationChannelKeyAsset {
                time_seconds,
                value: AnimationChannelValueAsset::Quaternion(value),
                in_tangent: None,
                out_tangent: None,
            })
            .collect(),
    }
}
