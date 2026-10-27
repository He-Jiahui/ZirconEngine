use super::*;
use crate::core::editing::command::EditorCommand;
use crate::core::editing::engine::{
    EditCommandCodecRegistry, EditorTransactionEngine, HistoryContextId, HistoryRetentionDomainV1,
    TransactionJournal, TransactionJournalReplayer,
};
use crate::core::editing::journal_codecs::register_scene_command_codecs;
use crate::core::gateway::EditorRuntimeGatewayHandle;
use serde_json::json;
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
use zircon_runtime::scene::components::{ColliderShape, JointComponent, NodeKind};
use zircon_runtime::scene::{DefaultLevelManager, LevelMetadata};
use zircon_runtime_interface::math::{Quat, Transform, Vec3};

const DYNAMIC: &str = "weather.Deep";
const BINDINGS: &str = "script.bindings";
const HISTORY: HistoryContextId = HistoryContextId::Global;

fn fixture(
    scene: Scene,
    selected: Vec<NodeId>,
    primary: NodeId,
    record_bytes: u64,
) -> EditorTransactionEngine {
    let level = DefaultLevelManager::default().create_level(scene, LevelMetadata::default());
    let mut core = CoreEditContext::new(EditorRuntimeGatewayHandle::detached());
    core.bind_scene(level, SceneSelection::new(selected, Some(primary)))
        .unwrap();
    let domain = HistoryRetentionDomainV1::try_new(HistoryRetentionBudgetV1 {
        record_bytes,
        ..Default::default()
    })
    .unwrap();
    EditorTransactionEngine::with_retention_domain_v1(core, 16, domain).unwrap()
}

fn read<T>(engine: &EditorTransactionEngine, read: impl FnOnce(&Scene) -> T) -> T {
    engine
        .with_context::<CoreEditContext, _>(|core| core.with_scene(read).unwrap())
        .unwrap()
        .unwrap()
}

fn mutate(engine: &EditorTransactionEngine, write: impl FnOnce(&mut Scene)) {
    engine
        .with_context::<CoreEditContext, _>(|core| {
            let (_, errors) = core.with_scene_mut(write).unwrap().into_parts();
            assert!(errors.is_empty());
        })
        .unwrap()
        .unwrap();
}

fn selected(engine: &EditorTransactionEngine) -> SceneSelection {
    engine
        .with_context::<CoreEditContext, _>(CoreEditContext::scene_selection)
        .unwrap()
        .unwrap()
        .unwrap()
}

fn capture(engine: &EditorTransactionEngine) -> EditorCommand {
    let selection = selected(engine);
    read(engine, |scene| {
        EditorCommand::duplicate_nodes(scene, selection.items().to_vec(), selection.primary())
            .unwrap()
    })
}

fn commit(engine: &EditorTransactionEngine, command: EditorCommand) -> TransactionJournal {
    let mut transaction = engine.begin("Duplicate scene nodes", HISTORY).unwrap();
    transaction.push(command).unwrap();
    let id = transaction.commit().unwrap();
    engine.journal_transaction(HISTORY, id).unwrap()
}

fn rich_scene() -> (Scene, NodeId, NodeId) {
    use zircon_runtime::core::framework::render::{
        RenderColorLookupTextureLayout, RenderPostProcessVolumeProfile, VolumetricFogSettings,
    };
    use zircon_runtime_interface::resource::{ResourceHandle, ResourceId};

    let mut scene = Scene::empty();
    scene
        .register_component_type(ComponentTypeDescriptor::new(DYNAMIC, "weather", "Deep"))
        .unwrap();
    let first = scene.spawn_node(NodeKind::Cube).unwrap();
    let second = scene.spawn_node(NodeKind::Camera).unwrap();
    scene
        .update_transform(
            first,
            Transform {
                translation: Vec3::new(1.0, 2.0, 3.0),
                rotation: Quat::from_rotation_y(0.7),
                scale: Vec3::new(2.0, 3.0, 4.0),
            },
        )
        .unwrap();
    let mut mesh = scene.get::<MeshRenderer>(first).unwrap().clone();
    mesh.morph_weights = vec![0.2, 0.7];
    mesh.render_queue = 17;
    scene.insert(first, mesh).unwrap();
    scene
        .insert(
            first,
            ColliderComponent {
                shape: ColliderShape::Compound {
                    children: vec![(
                        Transform::default(),
                        Box::new(ColliderShape::Sphere { radius: 1.5 }),
                    )],
                },
                sensor: true,
                ..Default::default()
            },
        )
        .unwrap();
    let mut settings = PostProcessSettingsComponent::default();
    settings.bloom.intensity = 0.75;
    settings.effect_stack.color_lookup.texture = Some(ResourceHandle::new(
        ResourceId::from_stable_label("duplicate://color-lookup"),
    ));
    settings.effect_stack.color_lookup.texture_layout =
        RenderColorLookupTextureLayout::Texture3d { size: 32 };
    settings.effect_stack.color_lookup.intensity = 0.6;
    settings.effect_stack.blur.radius = 2.5;
    settings.effect_stack.motion_blur.shutter_angle = 180.0;
    settings.effect_stack.motion_blur.samples = 9;
    settings.effect_stack.depth_of_field.aperture = 1.7;
    settings.effect_stack.depth_of_field.focal_length_mm = 85.0;
    settings.effect_stack.depth_of_field.bokeh_blade_count = 9;
    settings.effect_stack.screen_space_reflection.intensity = 0.4;
    settings.effect_stack.screen_space_reflection.max_steps = 96;
    let effect_stack = settings.effect_stack;
    scene.insert(second, settings).unwrap();
    scene
        .insert(
            first,
            PostProcessVolumeComponent {
                priority: 4.0,
                weight: 0.25,
                profile: RenderPostProcessVolumeProfile::default()
                    .with_effect_stack(effect_stack)
                    .with_volumetric_fog(VolumetricFogSettings {
                        // Authoring copies preserve finite values before render sanitization.
                        density: -0.05,
                        albedo: Vec3::new(0.2, 0.4, 0.7),
                        phase_g: 1.2,
                        depth_distribution_exp: 0.005,
                        temporal: false,
                        ..Default::default()
                    }),
                ..Default::default()
            },
        )
        .unwrap();
    scene
        .set_dynamic_component(
            first,
            DYNAMIC,
            json!({
                "nested": {"items": [1, {"label": "source", "flags": [true, false]}]},
                "unreflected": [null, 3.25]
            }),
        )
        .unwrap();
    scene
        .set_dynamic_component(
            first,
            BINDINGS,
            json!([
                {"package": "orb_run", "module": "main", "enabled": false,
                 "update": true, "fixed_update": false,
                 "properties": {"role": "orb", "nested": {"weights": [1, 2, 3]}}},
                {"package": "orb_run", "module": "pickup", "enabled": true,
                 "update": false, "fixed_update": true, "properties": {"points": 5}}
            ]),
        )
        .unwrap();
    (scene, first, second)
}

#[test]
fn duplicate_deep_copies_native_dynamic_and_script_bindings_in_one_undo_redo_record() {
    let (scene, first, second) = rich_scene();
    let originals = [
        scene.node_record(first).unwrap(),
        scene.node_record(second).unwrap(),
    ];
    let expected_dynamic = scene.dynamic_component(first, DYNAMIC).unwrap().clone();
    let expected_bindings = scene.dynamic_component(first, BINDINGS).unwrap().clone();
    let settings = scene
        .get::<PostProcessSettingsComponent>(second)
        .unwrap()
        .clone();
    let volume = scene
        .get::<PostProcessVolumeComponent>(first)
        .unwrap()
        .clone();
    let engine = fixture(
        scene,
        vec![second, first],
        second,
        HistoryRetentionBudgetV1::default().record_bytes,
    );
    let before = selected(&engine);
    let journal = commit(&engine, capture(&engine));
    assert_eq!(engine.history_status(HISTORY).unwrap().len, 1);
    assert_eq!(journal.commands().len(), 1);
    assert_eq!(
        journal.commands()[0].command_type(),
        "zircon.editor.scene.duplicate_nodes"
    );
    let copies = selected(&engine);
    assert_eq!(copies.items().len(), 2);
    let copied_first = copies.items()[0];
    let copied_second = copies.items()[1];
    assert_eq!(copies.primary(), Some(copied_second));
    read(&engine, |scene| {
        assert_eq!(scene.stable_entity_ids().count(), 4);
        for (original, &id) in originals.iter().zip(copies.items()) {
            let mut expected = original.clone();
            expected.id = id;
            assert_eq!(scene.node_record(id), Some(expected));
        }
        assert_eq!(
            scene.dynamic_component(copied_first, DYNAMIC),
            Some(&expected_dynamic)
        );
        assert_eq!(
            scene.dynamic_component(copied_first, BINDINGS),
            Some(&expected_bindings)
        );
        assert_eq!(
            scene.get::<PostProcessSettingsComponent>(copied_second),
            Some(&settings)
        );
        assert_eq!(
            scene.get::<PostProcessVolumeComponent>(copied_first),
            Some(&volume)
        );
    });
    // An edit to an owned nested payload cannot alias the source or redo snapshot.
    mutate(&engine, |scene| {
        scene
            .set_dynamic_component(copied_first, DYNAMIC, json!({"edited": true}))
            .unwrap();
        scene
            .set_dynamic_component(copied_first, BINDINGS, json!([]))
            .unwrap();
        let mut mesh = scene.get::<MeshRenderer>(copied_first).unwrap().clone();
        mesh.morph_weights[0] = 9.0;
        scene.insert(copied_first, mesh).unwrap();
    });
    read(&engine, |scene| {
        assert_eq!(scene.node_record(first), Some(originals[0].clone()));
        assert_eq!(
            scene.dynamic_component(first, DYNAMIC),
            Some(&expected_dynamic)
        );
        assert_eq!(
            scene.dynamic_component(first, BINDINGS),
            Some(&expected_bindings)
        );
    });
    assert!(engine.undo(HISTORY).unwrap());
    assert_eq!(selected(&engine), before);
    read(&engine, |scene| {
        assert_eq!(scene.stable_entity_ids().count(), 2);
        assert!(copies.items().iter().all(|&id| !scene.contains_entity(id)));
    });
    assert!(engine.redo(HISTORY).unwrap());
    assert_eq!(selected(&engine).items(), copies.items());
    read(&engine, |scene| {
        assert_eq!(
            scene.dynamic_component(copied_first, DYNAMIC),
            Some(&expected_dynamic)
        );
        assert_eq!(
            scene.dynamic_component(copied_first, BINDINGS),
            Some(&expected_bindings)
        );
        assert_eq!(
            scene.get::<MeshRenderer>(copied_first),
            originals[0].mesh.as_ref()
        );
        assert_eq!(
            scene.get::<PostProcessSettingsComponent>(copied_second),
            Some(&settings)
        );
    });
}

#[test]
fn duplicate_selected_hierarchy_remaps_native_links_without_copying_unselected_descendants() {
    let mut scene = Scene::empty();
    let external = scene.spawn_node(NodeKind::Empty).unwrap();
    let parent = scene.spawn_node(NodeKind::Cube).unwrap();
    let child = scene.spawn_node(NodeKind::Cube).unwrap();
    let unselected = scene.spawn_node(NodeKind::Cube).unwrap();
    scene.set_parent_checked(parent, Some(external)).unwrap();
    scene.set_parent_checked(child, Some(parent)).unwrap();
    scene.set_parent_checked(unselected, Some(parent)).unwrap();
    scene
        .insert(
            child,
            JointComponent {
                connected_entity: Some(parent),
                ..Default::default()
            },
        )
        .unwrap();
    let engine = fixture(
        scene,
        vec![child, parent, parent],
        child,
        HistoryRetentionBudgetV1::default().record_bytes,
    );
    commit(&engine, capture(&engine));
    let copies = selected(&engine);
    assert_eq!(copies.items().len(), 2);
    let (copied_parent, copied_child) = (copies.items()[0], copies.items()[1]);
    read(&engine, |scene| {
        assert_eq!(scene.stable_entity_ids().count(), 6);
        assert_eq!(scene.parent_of(copied_parent), Some(external));
        assert_eq!(scene.parent_of(copied_child), Some(copied_parent));
        assert_eq!(
            scene
                .get::<JointComponent>(copied_child)
                .unwrap()
                .connected_entity,
            Some(copied_parent)
        );
        assert_eq!(scene.parent_of(unselected), Some(parent));
    });
    assert!(engine.undo(HISTORY).unwrap());
    read(&engine, |scene| {
        assert_eq!(scene.stable_entity_ids().count(), 4);
        assert_eq!(scene.parent_of(child), Some(parent));
        assert_eq!(scene.parent_of(unselected), Some(parent));
    });
    assert!(engine.redo(HISTORY).unwrap());
    assert_eq!(selected(&engine).items(), copies.items());
    read(&engine, |scene| {
        assert_eq!(scene.parent_of(copied_child), Some(copied_parent))
    });
}

#[test]
fn duplicate_large_payload_round_trips_under_unchanged_default_retention_budgets() {
    let mut scene = Scene::empty();
    let ids = (0..4)
        .map(|_| scene.spawn_node(NodeKind::Cube).unwrap())
        .collect::<Vec<_>>();
    let expected = json!({"label": "x".repeat(256 * 1024), "nested": [{"n": 7}]});
    // Empty descriptor catalog compatibility rows must also survive exact copies.
    for &id in &ids {
        scene
            .set_dynamic_component(id, "legacy.raw", expected.clone())
            .unwrap();
    }
    let budget = HistoryRetentionBudgetV1::default();
    assert_eq!(budget.record_bytes, 64 * 1024 * 1024);
    assert_eq!(budget.staging_bytes, 64 * 1024 * 1024);
    let engine = fixture(scene, ids.clone(), ids[0], budget.record_bytes);
    commit(&engine, capture(&engine));
    let copies = selected(&engine);
    assert!(engine.undo(HISTORY).unwrap());
    assert!(engine.redo(HISTORY).unwrap());
    read(&engine, |scene| {
        for &id in ids.iter().chain(copies.items()) {
            assert_eq!(scene.dynamic_component(id, "legacy.raw"), Some(&expected));
        }
    });
    let retained = engine.retention_snapshot_v1().unwrap();
    assert!(retained.committed_bytes > 1024 * 1024);
    assert!(retained.committed_bytes <= budget.committed_bytes);
    assert_eq!(retained.staging_bytes, 0);
}

#[test]
fn duplicate_budget_refusal_and_stale_capture_leave_scene_selection_and_history_untouched() {
    let (scene, first, second) = rich_scene();
    let engine = fixture(scene, vec![first, second], first, 1024);
    let before = selected(&engine);
    let command = capture(&engine);
    let mut scope = engine.begin("Duplicate", HISTORY).unwrap();
    assert!(scope.push(command).is_err());
    drop(scope);
    assert_eq!(selected(&engine), before);
    assert_eq!(engine.history_status(HISTORY).unwrap().len, 0);
    assert_eq!(read(&engine, |scene| scene.stable_entity_ids().count()), 2);

    let (scene, first, second) = rich_scene();
    let engine = fixture(
        scene,
        vec![first],
        first,
        HistoryRetentionBudgetV1::default().record_bytes,
    );
    let command = capture(&engine);
    mutate(&engine, |scene| {
        scene.rename_node(second, "Concurrent change").unwrap();
    });
    let mut scope = engine.begin("Duplicate", HISTORY).unwrap();
    assert!(scope.push(command).is_err());
    drop(scope);
    assert_eq!(engine.history_status(HISTORY).unwrap().len, 0);
    assert_eq!(read(&engine, |scene| scene.stable_entity_ids().count()), 2);
}

#[test]
fn duplicate_journal_replays_complete_snapshot_with_bindings_and_undo_redo() {
    let (scene, first, second) = rich_scene();
    let source = fixture(
        scene,
        vec![first, second],
        first,
        HistoryRetentionBudgetV1::default().record_bytes,
    );
    let journal = commit(&source, capture(&source));
    let copies = selected(&source);
    let journal = TransactionJournal::decode(&journal.encode().unwrap()).unwrap();
    let mut codecs = EditCommandCodecRegistry::new();
    register_scene_command_codecs(&mut codecs).unwrap();
    let (scene, first, second) = rich_scene();
    let target = fixture(
        scene,
        vec![first, second],
        first,
        HistoryRetentionBudgetV1::default().record_bytes,
    );
    TransactionJournalReplayer::new(&codecs)
        .replay(&target, HISTORY, &journal)
        .unwrap();
    for &id in copies.items() {
        assert_eq!(
            read(&target, |scene| scene.node_record(id)),
            read(&source, |scene| scene.node_record(id))
        );
        assert_eq!(
            read(&target, |scene| scene.dynamic_components_for_entity(id)),
            read(&source, |scene| scene.dynamic_components_for_entity(id))
        );
        assert_eq!(
            read(&target, |scene| scene
                .get::<PostProcessSettingsComponent>(id)
                .cloned()),
            read(&source, |scene| scene
                .get::<PostProcessSettingsComponent>(id)
                .cloned())
        );
        assert_eq!(
            read(&target, |scene| scene
                .get::<PostProcessVolumeComponent>(id)
                .cloned()),
            read(&source, |scene| scene
                .get::<PostProcessVolumeComponent>(id)
                .cloned())
        );
    }
    assert!(target.undo(HISTORY).unwrap());
    assert!(target.redo(HISTORY).unwrap());
    assert_eq!(selected(&target).items(), copies.items());
}

#[test]
fn duplicate_invalid_dynamic_copy_rolls_back_whole_batch() {
    let mut scene = Scene::empty();
    let id = scene.spawn_node(NodeKind::Cube).unwrap();
    scene
        .set_dynamic_component(id, "legacy.raw", json!({"value": 3}))
        .unwrap();
    let command = EditorCommand::duplicate_nodes(&scene, vec![id], Some(id)).unwrap();
    // Catalog registration doesn't change authored data generation. The old
    // compatibility row can no longer be attached, exercising post-insert failure.
    scene
        .register_component_type(ComponentTypeDescriptor::new(DYNAMIC, "weather", "Deep"))
        .unwrap();
    let engine = fixture(
        scene,
        vec![id],
        id,
        HistoryRetentionBudgetV1::default().record_bytes,
    );
    let before = selected(&engine);
    let mut scope = engine.begin("Duplicate", HISTORY).unwrap();
    assert!(scope.push(command).is_err());
    drop(scope);
    assert_eq!(engine.history_status(HISTORY).unwrap().len, 0);
    assert_eq!(selected(&engine), before);
    read(&engine, |scene| {
        assert_eq!(scene.stable_entity_ids().collect::<Vec<_>>(), vec![id]);
        assert_eq!(
            scene.dynamic_component(id, "legacy.raw"),
            Some(&json!({"value": 3}))
        );
    });
}

#[test]
fn duplicate_replay_rejects_self_parent_and_missing_external_parent_before_mutation() {
    let (scene, first, second) = rich_scene();
    let source = fixture(
        scene,
        vec![first, second],
        first,
        HistoryRetentionBudgetV1::default().record_bytes,
    );
    let journal = commit(&source, capture(&source));
    let mut payload = journal.commands()[0].payload().clone();
    payload["records"][0]["parent"] = payload["records"][0]["id"].clone();
    assert!(
        EditorCommand::from_journal_duplicate(serde_json::from_value(payload).unwrap()).is_err()
    );

    let mut payload = journal.commands()[0].payload().clone();
    payload["records"][0]["parent"] = json!(999_999);
    let command =
        EditorCommand::from_journal_duplicate(serde_json::from_value(payload).unwrap()).unwrap();
    let (scene, first, second) = rich_scene();
    let target = fixture(
        scene,
        vec![first, second],
        first,
        HistoryRetentionBudgetV1::default().record_bytes,
    );
    let before = selected(&target);
    let mut scope = target.begin("Invalid duplicate replay", HISTORY).unwrap();
    assert!(scope.push(command).is_err());
    drop(scope);
    assert_eq!(target.history_status(HISTORY).unwrap().len, 0);
    assert_eq!(selected(&target), before);
    assert_eq!(read(&target, |scene| scene.stable_entity_ids().count()), 2);
}

#[test]
fn duplicate_small_selection_in_static_scene_round_trips_with_scene_sized_insertion_scratch() {
    use zircon_runtime::scene::components::Mobility;

    let mut scene = Scene::empty();
    let selected_id = scene.spawn_node(NodeKind::Cube).unwrap();
    for _ in 0..96 {
        let parent = scene.spawn_node(NodeKind::Empty).unwrap();
        let child = scene.spawn_node(NodeKind::Empty).unwrap();
        scene.set_mobility(parent, Mobility::Static).unwrap();
        scene.set_mobility(child, Mobility::Static).unwrap();
        scene.set_parent_checked(child, Some(parent)).unwrap();
    }
    let engine = fixture(
        scene,
        vec![selected_id],
        selected_id,
        HistoryRetentionBudgetV1::default().record_bytes,
    );
    commit(&engine, capture(&engine));
    let copies = selected(&engine);
    assert_eq!(copies.items().len(), 1);
    assert!(engine.undo(HISTORY).unwrap());
    assert_eq!(
        read(&engine, |scene| scene.stable_entity_ids().count()),
        193
    );
    assert!(engine.redo(HISTORY).unwrap());
    assert_eq!(selected(&engine).items(), copies.items());
    assert_eq!(
        read(&engine, |scene| scene.stable_entity_ids().count()),
        194
    );
}
