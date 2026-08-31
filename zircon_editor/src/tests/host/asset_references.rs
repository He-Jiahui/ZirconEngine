use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{
    module_descriptor as asset_module_descriptor, AssetReference, AssetUri, PhysicsMaterialAsset,
    ReferenceResolutionError, SceneAnimationGraphPlayerAsset, SceneAnimationPlayerAsset,
    SceneAnimationSequencePlayerAsset, SceneAnimationSkeletonAsset,
    SceneAnimationStateMachinePlayerAsset, SceneAsset, SceneColliderAsset, SceneColliderShapeAsset,
    SceneEntityAsset, SceneMobilityAsset, TransformAsset, ASSET_MODULE_NAME,
};
use zircon_runtime::core::framework::animation::{
    AnimationClipAsset, AnimationGraphAsset, AnimationGraphNodeAsset, AnimationSequenceAsset,
    AnimationSkeletonAsset, AnimationStateAsset, AnimationStateMachineAsset,
};
use zircon_runtime::core::framework::scene::physics::{
    PhysicsCombineRule, PhysicsMaterialMetadata,
};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};
use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;

use crate::core::document::AuthoringSceneInstaller;
use crate::core::project::{
    NewProjectDraft, NewProjectTemplate, ProjectAuthority, SceneCreateRequest,
};
use crate::core::recovery::ProjectSessionEffect;
use crate::tests::support::env_lock;
use crate::ui::host::editor_asset_manager::{editor_asset_manager_handle, EditorAssetManager};
use crate::ui::host::module::{module_descriptor, EDITOR_MANAGER_NAME, EDITOR_MODULE_NAME};
use crate::ui::host::EditorManager;

fn unique_temp_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}.json"))
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}"))
}

fn editor_runtime_with_config_path(path: &Path) -> CoreRuntime {
    std::env::set_var("ZIRCON_CONFIG_PATH", path);
    let runtime = CoreRuntime::new();
    runtime.store_config_value(
        crate::ui::host::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
        serde_json::json!([
            crate::ui::host::EDITOR_SUBSYSTEM_ANIMATION_AUTHORING,
            crate::ui::host::EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING,
            crate::ui::host::EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS,
            crate::ui::host::EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING,
        ]),
    );
    runtime
        .register_module(foundation_module_descriptor())
        .unwrap();
    runtime.register_module(asset_module_descriptor()).unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime.activate_module(ASSET_MODULE_NAME).unwrap();
    runtime.activate_module(EDITOR_MODULE_NAME).unwrap();
    crate::tests::support::configure_editor_test_runtime_build_set(&runtime);
    runtime
}

struct AcceptingSceneInstaller;

impl AuthoringSceneInstaller for AcceptingSceneInstaller {
    type Error = &'static str;

    fn prepare_scene_transition(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn install_scene(
        &mut self,
        _document: &crate::core::project::ProjectSceneDocument,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[test]
fn create_scene_document_refreshes_editor_assets_before_document_publication() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_scene_document_catalog");
    let project_root = unique_temp_dir("zircon_editor_scene_document_catalog_project");
    let runtime = editor_runtime_with_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let core = runtime.handle();
    let editor_assets = zircon_runtime::core::manager::resolve_manager_service(
        &core,
        editor_asset_manager_handle(&core).unwrap(),
    )
    .unwrap();
    let created = ProjectAuthority::default()
        .create_project(&NewProjectDraft {
            project_name: "Scene Catalog".to_string(),
            location: project_root.to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        })
        .unwrap();
    let root = created.root.clone();
    drop(created);

    manager.open_project(&root).unwrap();
    let ticket = manager.scene_picker_ticket().unwrap();
    let scene_uri = AssetUri::parse("res://scenes/editor-created.scene.toml").unwrap();
    let mut installer = AcceptingSceneInstaller;
    manager
        .create_scene_document(
            ticket,
            SceneCreateRequest::new(scene_uri.clone()),
            &mut installer,
        )
        .unwrap();

    let catalog = editor_assets.catalog_snapshot();
    assert!(
        catalog
            .assets
            .iter()
            .any(|asset| asset.locator == scene_uri.to_string()),
        "editor asset catalog was not refreshed for the created scene: {:?}",
        catalog
            .assets
            .iter()
            .map(|asset| asset.locator.as_str())
            .collect::<Vec<_>>()
    );
    assert!(
        catalog
            .assets
            .iter()
            .all(|asset| !asset.locator.contains(".zircon-scene-staging-")),
        "editor asset catalog retained a transient scene staging source"
    );

    let close = manager
        .begin_project_close()
        .expect("begin project close")
        .expect("active close operation");
    for effect in [
        ProjectSessionEffect::FocusBinding,
        ProjectSessionEffect::AssetJobs,
        ProjectSessionEffect::UserInterface,
        ProjectSessionEffect::Play,
    ] {
        manager
            .prepare_project_close_effect(&close, effect)
            .expect("prepare headless close effect");
        manager
            .commit_project_close_effect(&close, effect)
            .expect("commit headless close effect");
    }
    manager
        .commit_project_close(&close)
        .expect("commit manager close effects");
    manager
        .prepare_project_close_effect(&close, ProjectSessionEffect::WorkspaceProjection)
        .expect("prepare headless workspace close");
    manager
        .commit_project_close_effect(&close, ProjectSessionEffect::WorkspaceProjection)
        .expect("commit headless workspace close");
    manager
        .finalize_project_close(&close)
        .expect("finalize project close");
    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_asset_manager_tracks_scene_animation_and_physics_references() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_asset_references");
    let project_root = unique_temp_dir("zircon_editor_asset_references_project");
    let runtime = editor_runtime_with_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let core = runtime.handle();
    let editor_assets = zircon_runtime::core::manager::resolve_manager_service(
        &core,
        editor_asset_manager_handle(&core).unwrap(),
    )
    .unwrap();

    let paths = ProjectPaths::from_root(&project_root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    write_animation_reference_project(&paths);

    manager.open_project(&project_root).unwrap();

    let snapshot = editor_assets.catalog_snapshot();
    let uuid_by_locator = snapshot
        .assets
        .iter()
        .map(|asset| (asset.locator.clone(), asset.uuid.clone()))
        .collect::<HashMap<_, _>>();

    let scene = snapshot
        .assets
        .iter()
        .find(|asset| asset.locator == "res://scenes/main.scene.toml")
        .expect("scene asset should exist");
    assert_eq!(
        sorted_strings(scene.direct_reference_uuids.clone()),
        sorted_strings(vec![
            uuid_by_locator["res://physics/materials/default.physics_material.toml"].clone(),
            uuid_by_locator["res://animation/hero.skeleton.zranim"].clone(),
            uuid_by_locator["res://animation/hero.clip.zranim"].clone(),
            uuid_by_locator["res://animation/hero.sequence.zranim"].clone(),
            uuid_by_locator["res://animation/hero.graph.zranim"].clone(),
            uuid_by_locator["res://animation/hero.state_machine.zranim"].clone(),
        ]),
    );

    let scene_details = editor_assets
        .asset_details(&scene.uuid)
        .expect("scene details should resolve");
    assert_eq!(
        sorted_reference_locators(&scene_details.direct_references),
        vec![
            "res://animation/hero.clip.zranim".to_string(),
            "res://animation/hero.graph.zranim".to_string(),
            "res://animation/hero.sequence.zranim".to_string(),
            "res://animation/hero.skeleton.zranim".to_string(),
            "res://animation/hero.state_machine.zranim".to_string(),
            "res://physics/materials/default.physics_material.toml".to_string(),
        ],
    );

    let clip_details = editor_assets
        .asset_details(&uuid_by_locator["res://animation/hero.clip.zranim"])
        .expect("clip details should resolve");
    assert_eq!(
        sorted_reference_locators(&clip_details.referenced_by),
        vec![
            "res://animation/hero.graph.zranim".to_string(),
            "res://scenes/main.scene.toml".to_string(),
        ],
    );

    let graph_details = editor_assets
        .asset_details(&uuid_by_locator["res://animation/hero.graph.zranim"])
        .expect("graph details should resolve");
    assert_eq!(
        sorted_reference_locators(&graph_details.referenced_by),
        vec![
            "res://animation/hero.state_machine.zranim".to_string(),
            "res://scenes/main.scene.toml".to_string(),
        ],
    );

    let state_machine_details = editor_assets
        .asset_details(&uuid_by_locator["res://animation/hero.state_machine.zranim"])
        .expect("state machine details should resolve");
    assert_eq!(
        sorted_reference_locators(&state_machine_details.referenced_by),
        vec!["res://scenes/main.scene.toml".to_string()],
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
    let _ = fs::remove_dir_all(project_root);
}

fn write_animation_reference_project(paths: &ProjectPaths) {
    write_physics_material(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("physics")
            .join("materials")
            .join("default.physics_material.toml"),
    );
    write_animation_bytes(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("animation")
            .join("hero.skeleton.zranim"),
        skeleton_asset().to_bytes().unwrap(),
    );
    write_animation_bytes(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("animation")
            .join("hero.clip.zranim"),
        clip_asset().to_bytes().unwrap(),
    );
    write_animation_bytes(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("animation")
            .join("hero.sequence.zranim"),
        sequence_asset().to_bytes().unwrap(),
    );
    write_animation_bytes(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("animation")
            .join("hero.graph.zranim"),
        graph_asset().to_bytes().unwrap(),
    );
    write_animation_bytes(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("animation")
            .join("hero.state_machine.zranim"),
        state_machine_asset().to_bytes().unwrap(),
    );

    let scene_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("scenes")
        .join("main.scene.toml");
    if let Some(parent) = scene_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        scene_path,
        scene_asset()
            .to_project_toml_string(persist_test_project_reference)
            .unwrap(),
    )
    .unwrap();
}

fn write_physics_material(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, physics_material_asset().to_toml_string().unwrap()).unwrap();
}

fn write_animation_bytes(path: PathBuf, bytes: Vec<u8>) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, bytes).unwrap();
}

fn physics_material_asset() -> PhysicsMaterialAsset {
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

fn skeleton_asset() -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: Some("HeroSkeleton".to_string()),
        bones: Vec::new(),
    }
}

fn clip_asset() -> AnimationClipAsset {
    AnimationClipAsset {
        name: Some("HeroClip".to_string()),
        skeleton: asset_reference("res://animation/hero.skeleton.zranim"),
        duration_seconds: 1.0,
        tracks: Vec::new(),
        event_tracks: Vec::new(),
    }
}

fn sequence_asset() -> AnimationSequenceAsset {
    AnimationSequenceAsset {
        name: Some("HeroSequence".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: Vec::new(),
    }
}

fn graph_asset() -> AnimationGraphAsset {
    AnimationGraphAsset {
        name: Some("HeroGraph".to_string()),
        parameters: Vec::new(),
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "idle".to_string(),
                clip: asset_reference("res://animation/hero.clip.zranim"),
                playback_speed: 1.0,
                looping: true,
            },
            AnimationGraphNodeAsset::Output {
                source: "idle".to_string(),
            },
        ],
    }
}

fn state_machine_asset() -> AnimationStateMachineAsset {
    AnimationStateMachineAsset {
        name: Some("HeroStateMachine".to_string()),
        entry_state: "Locomotion".to_string(),
        states: vec![AnimationStateAsset::graph_ref(
            "Locomotion",
            asset_reference("res://animation/hero.graph.zranim"),
        )],
        transitions: Vec::new(),
        layers: Vec::new(),
    }
}

fn scene_asset() -> SceneAsset {
    SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 1,
            name: "Hero".to_string(),
            parent: None,
            transform: TransformAsset::default(),
            active: true,
            render_layer_mask: 0x0000_0001,
            mobility: SceneMobilityAsset::Dynamic,
            camera: None,
            mesh: None,
            ambient_light: None,
            directional_light: None,
            point_light: None,
            rect_light: None,
            spot_light: None,
            post_process_volume: None,
            rigid_body: None,
            collider: Some(SceneColliderAsset {
                shape: SceneColliderShapeAsset::Box {
                    half_extents: [0.5, 1.0, 0.5],
                },
                sensor: false,
                layer: 0,
                collision_group: 0,
                collision_mask: u32::MAX,
                material: Some(asset_reference(
                    "res://physics/materials/default.physics_material.toml",
                )),
                material_override: None,
                local_transform: TransformAsset::default(),
            }),
            joint: None,
            animation_skeleton: Some(SceneAnimationSkeletonAsset {
                skeleton: asset_reference("res://animation/hero.skeleton.zranim"),
            }),
            animation_player: Some(SceneAnimationPlayerAsset {
                clip: asset_reference("res://animation/hero.clip.zranim"),
                playback_speed: 1.0,
                time_seconds: 0.0,
                weight: 1.0,
                looping: true,
                playing: true,
            }),
            animation_sequence_player: Some(SceneAnimationSequencePlayerAsset {
                sequence: asset_reference("res://animation/hero.sequence.zranim"),
                playback_speed: 1.0,
                time_seconds: 0.0,
                looping: true,
                playing: true,
            }),
            animation_graph_player: Some(SceneAnimationGraphPlayerAsset {
                graph: asset_reference("res://animation/hero.graph.zranim"),
                parameters: HashMap::new().into_iter().collect(),
                playing: true,
            }),
            animation_state_machine_player: Some(SceneAnimationStateMachinePlayerAsset {
                state_machine: asset_reference("res://animation/hero.state_machine.zranim"),
                parameters: HashMap::new().into_iter().collect(),
                active_state: Some("Locomotion".to_string()),
                playing: true,
            }),
            terrain: None,
            tilemap: None,
            prefab_instance: None,
            script_bindings: Vec::new(),
        }],
    }
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

fn persist_test_project_reference(
    reference: &AssetReference,
) -> Result<PersistedAssetReference, ReferenceResolutionError> {
    match reference.locator.scheme() {
        ResourceScheme::Builtin => Ok(PersistedAssetReference::builtin(reference.locator.clone())),
        ResourceScheme::Res => {
            let path_hint = RelPath::parse(format!("assets/{}", reference.locator.path()))
                .map_err(|source| ReferenceResolutionError::Path {
                    path: reference.locator.to_string(),
                    source,
                })?;
            let project_reference = AssetRef::try_new(
                reference.uuid,
                path_hint,
                reference.locator.label().map(str::to_string),
            )
            .map_err(|source| ReferenceResolutionError::AssetRef { source })?;
            Ok(PersistedAssetReference::project(project_reference))
        }
        _ => Err(ReferenceResolutionError::UnsupportedScheme {
            locator: reference.locator.clone(),
        }),
    }
}

fn sorted_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values
}

fn sorted_reference_locators(
    references: &[crate::ui::host::editor_asset_manager::EditorAssetReferenceRecord],
) -> Vec<String> {
    let mut locators = references
        .iter()
        .map(|reference| reference.locator.clone())
        .collect::<Vec<_>>();
    locators.sort();
    locators
}
