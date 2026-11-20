//! Persistent graph compilation must follow the installed graph and skeleton payloads.
use std::fs;
use std::path::{Path, PathBuf};

use zircon_plugin_animation_runtime::AnimationEvaluationPipeline;
use zircon_runtime::asset::project::{AssetMetaDocument, ProjectPaths};
use zircon_runtime::asset::{
    AssetManager, AssetUuid, ProjectAssetManager, ProjectManifest, SceneAsset,
};
use zircon_runtime::core::framework::animation::{
    AnimationPoseOutput, AnimationPoseSource, AnimationSkeletonAsset,
};
use zircon_runtime::core::resource::{ResourceActiveVersionKey, ResourceManager, ResourceState};
use zircon_runtime::scene::{self, EntityId, LevelSystem};

use super::*;

const SCENE_URI: &str = "res://reload.scene.toml";

#[test]
fn recreated_project_graph_selects_the_new_clip_in_the_warm_level() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let manager = runtime_asset_manager(&runtime.handle());
    let assets = open_project(&manager);
    let (graph, skeleton_id, clip_a, clip_b) = project_graph_inputs(&manager, &assets);
    let reference = graph.publish(
        &manager,
        single_clip_graph(clip_a.clone()).to_bytes().unwrap(),
    );
    let graph_id = ResourceId::from_asset_uuid(reference.uuid);
    let resources = manager.resource_manager();
    let a_key = active_key(&resources, graph_id);
    let a = resources
        .snapshot_expected::<AnimationGraphMarker, AnimationGraphAsset>(
            ResourceHandle::new(graph_id),
            &a_key,
        )
        .unwrap();
    assert_eq!(a.revision(), 1);
    assert_eq!(selected_clip(&a), &clip_a);
    let a_source_hash = a.record().source_hash.clone();
    drop(a);

    let warm = scene::load_level_asset(&runtime.handle(), &*manager, SCENE_URI).unwrap();
    let warm_entity = attach_graph(&warm, graph_id, skeleton_id);
    let warm_handle = warm.world_handle();
    let warm_epoch = warm.capture_world_replacement_epoch();
    let activation = manager
        .current_project_manager()
        .unwrap()
        .activation_identity()
        .clone();
    runtime.tick_level_seconds(&warm, 0.0).unwrap();
    assert_hand_x(hand_x(&graph_pose(&warm, warm_entity)), 1.0, "warm A");
    assert_eq!(compiled_graph_cache_len(&warm), 1);
    let warm_evaluations = graph_evaluations(&warm);
    eprintln!(
        "GRAPH_RELOAD_GRAPH_WARM uuid={} revision=1 hand_x=1",
        reference.uuid
    );

    // The project Source Delete owner forbids persisted referencers. This graph is
    // installed on a live entity, not written back into the opening scene document.
    let original_meta = fs::read(&graph.sidecar).unwrap();
    let removed = manager.delete_project_source(reference.uuid).unwrap();
    assert!(removed
        .iter()
        .any(|status| status.uri == graph.uri.to_string()));
    assert!(!graph.path.exists());
    assert!(!graph.sidecar.exists());
    assert!(manager.resolve_asset_id(&graph.uri).is_none());
    assert!(resources.registry().get(graph_id).is_none());
    assert!(resources
        .projection_snapshot()
        .active()
        .key(graph_id)
        .is_none());
    assert!(resources
        .snapshot::<AnimationGraphMarker, AnimationGraphAsset>(ResourceHandle::new(graph_id))
        .is_none());

    fs::write(&graph.sidecar, original_meta).unwrap();
    assert_eq!(
        graph.publish(
            &manager,
            single_clip_graph(clip_b.clone()).to_bytes().unwrap()
        ),
        reference
    );
    let b_key = active_key(&resources, graph_id);
    let b = resources
        .snapshot_expected::<AnimationGraphMarker, AnimationGraphAsset>(
            ResourceHandle::new(graph_id),
            &b_key,
        )
        .unwrap();
    assert_eq!(b.revision(), 1);
    assert_eq!(selected_clip(&b), &clip_b);
    assert_ne!(a_key, b_key);
    assert_ne!(a_source_hash, b.record().source_hash);
    assert!(manager.is_project_activation_current(&activation));
    eprintln!(
        "GRAPH_RELOAD_GRAPH_READY uuid={} revision=1 removed=true new_payload=true",
        reference.uuid
    );

    // Consume the next real outer frame on the already-warm World first.
    let warm_previous = warm.world_time().last_outer_frame_index().unwrap();
    assert_eq!(warm_previous, 1);
    runtime.tick_level_seconds(&warm, 0.0).unwrap();
    let warm_frame = warm.world_time().last_outer_frame_index().unwrap();
    assert_eq!(warm_frame, warm_previous + 1);
    assert_eq!(warm.world_handle(), warm_handle);
    assert_eq!(warm.capture_world_replacement_epoch(), warm_epoch);
    let warm_evaluated = graph_evaluations(&warm);
    assert_eq!(warm_evaluated, warm_evaluations + 1);
    assert!(manager.is_project_activation_current(&activation));
    assert_eq!(active_key(&resources, graph_id), b_key);
    let warm_b = graph_pose(&warm, warm_entity);
    let warm_x = hand_x(&warm_b);
    eprintln!(
        "GRAPH_RELOAD_GRAPH_WARM_CLOCK from={warm_previous} to={warm_frame} world={} epoch={warm_epoch} eval_from={warm_evaluations} eval_to={warm_evaluated} hand_x={warm_x}",
        warm_handle.get()
    );

    // The cold World has never consumed a frame. First accept the next real
    // outer frame on it; never submit that frame to the old warm World again.
    let cold = scene::load_level_asset(&runtime.handle(), &*manager, SCENE_URI).unwrap();
    assert_ne!(cold.world_handle(), warm_handle);
    assert_eq!(cold.world_time().last_outer_frame_index(), None);
    let cold_entity = attach_graph(&cold, graph_id, skeleton_id);
    runtime.tick_level_seconds(&cold, 0.0).unwrap();
    let cold_frame = cold.world_time().last_outer_frame_index().unwrap();
    assert_eq!(cold_frame, warm_frame + 1);
    let cold_x = hand_x(&graph_pose(&cold, cold_entity));
    assert_hand_x(cold_x, 9.0, "cold B");

    // A late filesystem notification, a different published Arc, or an
    // implicit warm tick invalidates the comparison, not the graph cache.
    assert!(manager.is_project_activation_current(&activation));
    assert_eq!(active_key(&resources, graph_id), b_key);
    let after = resources
        .snapshot_expected::<AnimationGraphMarker, AnimationGraphAsset>(
            ResourceHandle::new(graph_id),
            &b_key,
        )
        .unwrap();
    assert_eq!(after.revision(), 1);
    assert_eq!(selected_clip(&after), &clip_b);
    assert!(std::sync::Arc::ptr_eq(b.resource(), after.resource()));
    assert_eq!(warm.world_handle(), warm_handle);
    assert_eq!(warm.capture_world_replacement_epoch(), warm_epoch);
    assert_eq!(warm.world_time().last_outer_frame_index(), Some(warm_frame));
    assert_eq!(graph_evaluations(&warm), warm_evaluated);
    assert_eq!(graph_pose(&warm, warm_entity), warm_b);
    eprintln!(
        "GRAPH_RELOAD_GRAPH_COLD_CLOCK first_from=none first={cold_frame} world={} warm_still={warm_frame} same_payload=true warm_pose_unchanged=true",
        cold.world_handle().get()
    );
    eprintln!("GRAPH_RELOAD_GRAPH_OBSERVED warm_hand_x={warm_x} cold_hand_x={cold_x}");
    assert_hand_x(warm_x, cold_x, "GRAPH_RELOAD_GRAPH_ABA");
}

#[test]
fn existing_project_graph_reimport_uses_the_new_revision_in_the_warm_level() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let manager = runtime_asset_manager(&runtime.handle());
    let assets = open_project(&manager);
    let (graph, skeleton_id, clip_a, clip_b) = project_graph_inputs(&manager, &assets);
    let old = graph.publish(&manager, single_clip_graph(clip_a).to_bytes().unwrap());
    let graph_id = ResourceId::from_asset_uuid(old.uuid);
    let resources = manager.resource_manager();
    assert_eq!(resources.registry().get(graph_id).unwrap().revision, 1);
    let level = scene::load_level_asset(&runtime.handle(), &*manager, SCENE_URI).unwrap();
    let entity = attach_graph(&level, graph_id, skeleton_id);
    let epoch = level.capture_world_replacement_epoch();
    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert_hand_x(hand_x(&graph_pose(&level, entity)), 1.0, "ordinary A");

    assert_eq!(
        graph.publish(
            &manager,
            single_clip_graph(clip_b.clone()).to_bytes().unwrap()
        ),
        old
    );
    let key = active_key(&resources, graph_id);
    let snapshot = resources
        .snapshot_expected::<AnimationGraphMarker, AnimationGraphAsset>(
            ResourceHandle::new(graph_id),
            &key,
        )
        .unwrap();
    assert_eq!(snapshot.revision(), 2);
    assert_eq!(selected_clip(&snapshot), &clip_b);
    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert_eq!(level.capture_world_replacement_epoch(), epoch);
    assert_eq!(active_key(&resources, graph_id), key);
    assert_hand_x(hand_x(&graph_pose(&level, entity)), 9.0, "ordinary B");
    eprintln!("GRAPH_RELOAD_OVERWRITE_OBSERVED revision=1->2 hand_x=1->9");
}

#[test]
fn recreated_skeleton_rebinds_the_graph_mask_to_the_new_bone_order() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let manager = runtime_asset_manager(&runtime.handle());
    let resources = manager.resource_manager();
    let skeleton_uri = AssetUri::parse("res://graph-reload-mask.skeleton.zranim").unwrap();
    let graph_uri = AssetUri::parse("res://graph-reload-mask.graph.zranim").unwrap();
    let base_uri = AssetUri::parse("res://graph-reload-mask-base.clip.zranim").unwrap();
    let add_uri = AssetUri::parse("res://graph-reload-mask-add.clip.zranim").unwrap();
    let skeleton_id = ResourceId::from_asset_uuid(AssetUuid::new());
    let graph_id = ResourceId::from_asset_uuid(AssetUuid::new());
    let skeleton_record = || {
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        )
    };
    let mut skeleton_a = two_bone_skeleton();
    let mut other = skeleton_a.bones[1].clone();
    other.name = "Other".to_owned();
    skeleton_a.bones.push(other);
    let mut skeleton_b = skeleton_a.clone();
    skeleton_b.bones.swap(1, 2);
    resources
        .register_ready(
            skeleton_record().with_source_hash("skeleton-order-a"),
            skeleton_a,
        )
        .unwrap();
    let a_key = active_key(&resources, skeleton_id);
    let graph = {
        let mut graph = additive_mask_graph(&base_uri, &add_uri);
        // Mask only the additive input; the x=1 base remains unmasked.
        let AnimationGraphNodeAsset::Additive { additive, .. } = &mut graph.nodes[2] else {
            panic!("expected additive fixture node");
        };
        *additive = "masked".into();
        let AnimationGraphNodeAsset::Mask { input, .. } = &mut graph.nodes[3] else {
            panic!("expected mask fixture node");
        };
        *input = "add".into();
        let AnimationGraphNodeAsset::Output { source } = &mut graph.nodes[4] else {
            panic!("expected output fixture node");
        };
        *source = "additive".into();
        graph.nodes.swap(2, 3);
        graph
    };
    for (uri, x) in [(&base_uri, 1.0), (&add_uri, 9.0)] {
        resources
            .register_ready(
                ResourceRecord::new(
                    ResourceId::from_locator(uri),
                    ResourceKind::AnimationClip,
                    uri.clone(),
                ),
                single_hand_translation_clip(&skeleton_uri, x),
            )
            .unwrap();
    }
    resources
        .register_ready(
            ResourceRecord::new(graph_id, ResourceKind::AnimationGraph, graph_uri),
            graph,
        )
        .unwrap();
    let graph_key = active_key(&resources, graph_id);
    let warm = runtime.create_default_level().unwrap();
    let warm_entity = attach_graph(&warm, graph_id, skeleton_id);
    let warm_handle = warm.world_handle();
    let warm_epoch = warm.capture_world_replacement_epoch();
    runtime.tick_level_seconds(&warm, 0.0).unwrap();
    let warm_a = graph_pose(&warm, warm_entity);
    assert_eq!(bone_order(&warm_a), ["Root", "Hand", "Other"]);
    assert_hand_x(hand_x(&warm_a), 10.0, "warm skeleton A");
    assert_eq!(compiled_graph_cache_len(&warm), 1);
    let warm_evaluations = graph_evaluations(&warm);

    let removed = resources.remove_by_locator(&skeleton_uri).unwrap().unwrap();
    assert_eq!(removed.id, skeleton_id);
    assert_eq!(removed.revision, 1);
    assert!(resources
        .projection_snapshot()
        .active()
        .key(skeleton_id)
        .is_none());
    resources
        .register_ready(
            skeleton_record().with_source_hash("skeleton-order-b"),
            skeleton_b,
        )
        .unwrap();
    let b_key = active_key(&resources, skeleton_id);
    let b = resources
        .snapshot_expected::<AnimationSkeletonMarker, AnimationSkeletonAsset>(
            ResourceHandle::new(skeleton_id),
            &b_key,
        )
        .unwrap();
    assert_eq!(b.revision(), 1);
    assert_eq!(b.bones[2].name, "Hand");
    assert_ne!(a_key, b_key);

    let warm_previous = warm.world_time().last_outer_frame_index().unwrap();
    assert_eq!(warm_previous, 1);
    runtime.tick_level_seconds(&warm, 0.0).unwrap();
    let warm_frame = warm.world_time().last_outer_frame_index().unwrap();
    assert_eq!(warm_frame, warm_previous + 1);
    assert_eq!(warm.world_handle(), warm_handle);
    assert_eq!(warm.capture_world_replacement_epoch(), warm_epoch);
    let warm_evaluated = graph_evaluations(&warm);
    assert_eq!(warm_evaluated, warm_evaluations + 1);
    assert_eq!(active_key(&resources, skeleton_id), b_key);
    assert_eq!(active_key(&resources, graph_id), graph_key);
    let warm_b = graph_pose(&warm, warm_entity);
    // The Clip evaluator really did pick up B, independently of the graph mask.
    assert_eq!(bone_order(&warm_b), ["Root", "Other", "Hand"]);
    let warm_x = hand_x(&warm_b);
    eprintln!(
        "GRAPH_RELOAD_SKELETON_WARM_CLOCK from={warm_previous} to={warm_frame} world={} epoch={warm_epoch} eval_from={warm_evaluations} eval_to={warm_evaluated} hand_x={warm_x}",
        warm_handle.get()
    );

    let cold = runtime.create_default_level().unwrap();
    assert_ne!(cold.world_handle(), warm_handle);
    assert_eq!(cold.world_time().last_outer_frame_index(), None);
    let cold_entity = attach_graph(&cold, graph_id, skeleton_id);
    runtime.tick_level_seconds(&cold, 0.0).unwrap();
    let cold_frame = cold.world_time().last_outer_frame_index().unwrap();
    assert_eq!(cold_frame, warm_frame + 1);
    let cold_b = graph_pose(&cold, cold_entity);
    assert_eq!(bone_order(&cold_b), ["Root", "Other", "Hand"]);
    let cold_x = hand_x(&cold_b);
    assert_hand_x(cold_x, 10.0, "cold skeleton B");

    assert_eq!(active_key(&resources, skeleton_id), b_key);
    assert_eq!(active_key(&resources, graph_id), graph_key);
    let after = resources
        .snapshot_expected::<AnimationSkeletonMarker, AnimationSkeletonAsset>(
            ResourceHandle::new(skeleton_id),
            &b_key,
        )
        .unwrap();
    assert_eq!(after.revision(), 1);
    assert_eq!(after.bones[2].name, "Hand");
    assert!(std::sync::Arc::ptr_eq(b.resource(), after.resource()));
    assert_eq!(warm.world_handle(), warm_handle);
    assert_eq!(warm.capture_world_replacement_epoch(), warm_epoch);
    assert_eq!(warm.world_time().last_outer_frame_index(), Some(warm_frame));
    assert_eq!(graph_evaluations(&warm), warm_evaluated);
    assert_eq!(graph_pose(&warm, warm_entity), warm_b);
    eprintln!(
        "GRAPH_RELOAD_SKELETON_COLD_CLOCK first_from=none first={cold_frame} world={} warm_still={warm_frame} same_payload=true warm_pose_unchanged=true",
        cold.world_handle().get()
    );
    eprintln!("GRAPH_RELOAD_SKELETON_OBSERVED order=root-other-hand warm_hand_x={warm_x} cold_hand_x={cold_x}");
    assert_hand_x(warm_x, cold_x, "GRAPH_RELOAD_SKELETON_ABA");
}

struct SourceFile {
    uri: AssetUri,
    path: PathBuf,
    sidecar: PathBuf,
}

impl SourceFile {
    fn new(assets: &Path, name: &str) -> Self {
        Self {
            uri: AssetUri::parse(&format!("res://{name}")).unwrap(),
            path: assets.join(name),
            sidecar: assets.join(format!("{name}.zmeta")),
        }
    }

    fn publish(&self, manager: &ProjectAssetManager, bytes: Vec<u8>) -> AssetReference {
        let source_existed = self.path.exists();
        let receipt = manager
            .publish_generated_project_source(self.uri.clone(), bytes.clone())
            .unwrap();
        assert_eq!(receipt.source_uri(), &self.uri);
        assert_eq!(receipt.previous_source_hash().is_some(), source_existed);
        assert!(receipt.status().is_some());
        assert_eq!(fs::read(&self.path).unwrap(), bytes);
        let meta = AssetMetaDocument::load(&self.sidecar).unwrap();
        assert_eq!(meta.url, self.uri);
        assert_eq!(
            manager.resolve_asset_id(&self.uri),
            Some(ResourceId::from_asset_uuid(meta.uuid))
        );
        AssetReference::new(meta.uuid, self.uri.clone())
    }
}

fn open_project(manager: &ProjectAssetManager) -> PathBuf {
    // The managed animation host already admits this physical smoke directory.
    // Keep each authored project in its own new subdirectory for failure inspection.
    let smoke = PathBuf::from(std::env::var_os("ZIRCON_RENDERING_SMOKE_OUTPUT_DIR").unwrap());
    let smoke = ProjectPaths::resolve_identity(&smoke).unwrap();
    let root = ProjectPaths::resolve_identity(
        smoke
            .operation_path()
            .join(format!("graph-reload-{}", AssetUuid::new())),
    )
    .unwrap();
    assert!(root.is_within(&smoke));
    fs::create_dir(root.operation_path()).unwrap();
    let paths = ProjectPaths::from_root(root.operation_path()).unwrap();
    let manifest = ProjectManifest::new("GraphReload", AssetUri::parse(SCENE_URI).unwrap(), 3);
    paths.ensure_layout(&manifest.asset_roots).unwrap();
    manifest.save(paths.manifest_path()).unwrap();
    let assets = paths.asset_root(&manifest.asset_roots[0]);
    let scene = SceneAsset { entities: vec![] }
        .to_project_toml_string(|_| unreachable!("empty scene has no references"))
        .unwrap();
    fs::write(assets.join("reload.scene.toml"), scene).unwrap();
    manager
        .open_project(root.operation_path().to_str().unwrap())
        .unwrap();
    assets
}

fn project_graph_inputs(
    manager: &ProjectAssetManager,
    assets: &Path,
) -> (SourceFile, ResourceId, AssetReference, AssetReference) {
    let skeleton_file = SourceFile::new(assets, "reload.skeleton.zranim");
    let skeleton = skeleton_file.publish(manager, two_bone_skeleton().to_bytes().unwrap());
    let skeleton_id = ResourceId::from_asset_uuid(skeleton.uuid);
    let clip = |name, x| {
        let mut payload = single_hand_translation_clip(&skeleton.locator, x);
        payload.skeleton = skeleton.clone();
        SourceFile::new(assets, name).publish(manager, payload.to_bytes().unwrap())
    };
    (
        SourceFile::new(assets, "reload.graph.zranim"),
        skeleton_id,
        clip("gesture-a.clip.zranim", 1.0),
        clip("gesture-b.clip.zranim", 9.0),
    )
}

fn single_clip_graph(clip: AssetReference) -> AnimationGraphAsset {
    AnimationGraphAsset {
        name: Some("ReloadGesture".into()),
        parameters: Vec::new(),
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "gesture".into(),
                clip,
                playback_speed: 1.0,
                looping: false,
            },
            AnimationGraphNodeAsset::Output {
                source: "gesture".into(),
            },
        ],
    }
}

fn selected_clip(graph: &AnimationGraphAsset) -> &AssetReference {
    let AnimationGraphNodeAsset::Clip { clip, .. } = &graph.nodes[0] else {
        panic!("expected single-clip graph");
    };
    clip
}

fn attach_graph(level: &LevelSystem, graph_id: ResourceId, skeleton_id: ResourceId) -> EntityId {
    level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        world
            .set_animation_skeleton(
                entity,
                Some(AnimationSkeletonComponent {
                    skeleton: ResourceHandle::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_graph_player(
                entity,
                Some(AnimationGraphPlayerComponent {
                    graph: ResourceHandle::new(graph_id),
                    parameters: BTreeMap::new().into(),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    })
}

fn active_key(resources: &ResourceManager, id: ResourceId) -> ResourceActiveVersionKey {
    let projection = resources.projection_snapshot();
    let row = projection.active().row(id).expect("active resource");
    assert_eq!(row.record.state, ResourceState::Ready);
    row.key.clone()
}

fn graph_evaluations(level: &LevelSystem) -> u64 {
    level.with_world(|world| {
        world
            .resource::<AnimationEvaluationPipeline>()
            .graph_evaluation_count()
    })
}

fn compiled_graph_cache_len(level: &LevelSystem) -> usize {
    level.with_world(|world| {
        world
            .resource::<AnimationEvaluationPipeline>()
            .compiled_graph_cache_len()
    })
}

fn graph_pose(level: &LevelSystem, entity: EntityId) -> AnimationPoseOutput {
    let pose = level.animation_pose(entity).expect("published graph pose");
    assert_eq!(pose.source, AnimationPoseSource::Graph);
    pose
}

fn bone_order(pose: &AnimationPoseOutput) -> Vec<&str> {
    pose.bones.iter().map(|bone| bone.name.as_str()).collect()
}

fn hand_x(pose: &AnimationPoseOutput) -> f32 {
    pose.bones
        .iter()
        .find(|bone| bone.name == "Hand")
        .unwrap()
        .local_transform
        .translation
        .x
}

fn assert_hand_x(observed: f32, expected: f32, stage: &str) {
    assert!(
        (observed - expected).abs() <= 1.0e-4,
        "{stage}: expected Hand.x={expected}, observed {observed}"
    );
}
