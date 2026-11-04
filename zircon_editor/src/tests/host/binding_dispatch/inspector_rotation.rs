use std::sync::Arc;

use super::support;
use crate::core::editing::{engine::HistoryContextId, intent::EditorIntent};
use crate::core::editor_event::InspectorFieldChange;
use crate::core::gateway::InProcessGateway;
use crate::core::play::PlayInstanceId;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::binding_dispatch::apply_inspector_binding;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::math::{EulerRot, Quat, Transform, Vec3};
use zircon_runtime_interface::ui::binding::UiBindingValue;

fn xyz(degrees: [f32; 3]) -> Quat {
    let [x, y, z] = degrees.map(f32::to_radians);
    Quat::from_euler(EulerRot::XYZ, x, y, z)
}

fn binding(changes: impl IntoIterator<Item = (&'static str, UiBindingValue)>) -> EditorUiBinding {
    EditorUiBinding::new(
        "Inspector",
        "TransformRotationXCommit",
        EditorUiEventKind::Submit,
        EditorUiBindingPayload::inspector_field_batch(
            "entity://selected",
            changes
                .into_iter()
                .map(|(field, value)| InspectorFieldChange::new(field, value))
                .collect::<Vec<_>>(),
        ),
    )
}

fn precise_transform() -> Transform {
    Transform {
        translation: Vec3::new(12.345678, -0.012345, 98.76543),
        rotation: xyz([13.123456, 24.234567, -35.345678]),
        scale: Vec3::new(1.234567, 2.345678, 3.456789),
    }
}

fn same_orientation(actual: Quat, expected: Quat) {
    assert!(actual.is_normalized());
    assert!(
        actual.dot(expected).abs() > 0.999999,
        "{actual:?} != {expected:?}"
    );
}

#[test]
fn each_rotation_axis_changes_local_transform_and_undo_redo_preserve_other_fields() {
    for (axis, field) in [
        "transform.rotation_degrees.x",
        "transform.rotation_degrees.y",
        "transform.rotation_degrees.z",
    ]
    .into_iter()
    .enumerate()
    {
        let mut state = support::test_state();
        let cube = support::cube_id(&state);
        let camera = support::camera_id(&state);
        let original = precise_transform();
        state.world.expect_with_world_mut(|scene| {
            scene.update_transform(cube, original).unwrap();
            scene.set_parent_checked(cube, Some(camera)).unwrap();
        });
        state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
        let name = state.name_field.clone();
        assert!(apply_inspector_binding(
            &mut state,
            &binding([(field, UiBindingValue::Float(47.125))])
        )
        .unwrap());
        let changed = state.world.expect_with_world(|scene| {
            let node = scene.find_node(cube).unwrap();
            assert_eq!(node.parent, Some(camera));
            assert_eq!(node.name, name);
            assert_eq!(node.transform.translation, original.translation);
            assert_eq!(node.transform.scale, original.scale);
            let mut expected = [13.123456, 24.234567, -35.345678];
            expected[axis] = 47.125;
            same_orientation(node.transform.rotation, xyz(expected));
            node.transform
        });
        assert_eq!(
            state
                .transactions()
                .history_status(state.scene_history_context().unwrap())
                .unwrap()
                .len,
            1
        );
        assert!(state.apply_intent(EditorIntent::Undo).unwrap());
        state
            .world
            .expect_with_world(|scene| assert_eq!(scene.local_transform(cube), Some(original)));
        assert!(state.apply_intent(EditorIntent::Redo).unwrap());
        state
            .world
            .expect_with_world(|scene| assert_eq!(scene.local_transform(cube), Some(changed)));
    }
}

#[test]
fn rotation_unchanged_submit_and_unrelated_edit_preserve_full_precision_quaternion() {
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    let mut original = precise_transform();
    original.rotation = -original.rotation;
    state
        .world
        .expect_with_world_mut(|scene| scene.update_transform(cube, original).unwrap());
    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
    assert!(!apply_inspector_binding(
        &mut state,
        &binding([
            ("transform.rotation_degrees.x", UiBindingValue::Float(13.12)),
            ("transform.rotation_degrees.y", UiBindingValue::Float(24.23)),
            (
                "transform.rotation_degrees.z",
                UiBindingValue::Float(-35.35)
            ),
        ])
    )
    .unwrap());
    assert_eq!(
        state
            .transactions()
            .history_status(state.scene_history_context().unwrap())
            .unwrap()
            .len,
        0
    );
    assert!(apply_inspector_binding(
        &mut state,
        &binding([("name", UiBindingValue::string("Renamed"))])
    )
    .unwrap());
    state
        .world
        .expect_with_world(|scene| assert_eq!(scene.local_transform(cube), Some(original)));
}

#[test]
fn rotation_mixed_batch_keeps_translation_scale_and_one_atomic_history_entry() {
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    let original = precise_transform();
    state
        .world
        .expect_with_world_mut(|scene| scene.update_transform(cube, original).unwrap());
    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
    let commit = binding([
        ("transform.rotation_degrees.x", UiBindingValue::Float(40.0)),
        ("transform.rotation_degrees.y", UiBindingValue::Float(50.0)),
        ("transform.rotation_degrees.z", UiBindingValue::Float(60.0)),
        ("transform.translation.y", UiBindingValue::Float(7.25)),
        ("transform.scale.z", UiBindingValue::Float(2.5)),
        ("name", UiBindingValue::string("Rotated Cube")),
    ]);
    assert!(apply_inspector_binding(&mut state, &commit).unwrap());
    let changed = state.world.expect_with_world(|scene| {
        let node = scene.find_node(cube).unwrap();
        same_orientation(node.transform.rotation, xyz([40.0, 50.0, 60.0]));
        assert_eq!(
            node.transform.translation,
            Vec3::new(original.translation.x, 7.25, original.translation.z)
        );
        assert_eq!(
            node.transform.scale,
            Vec3::new(original.scale.x, original.scale.y, 2.5)
        );
        assert_eq!(node.name, "Rotated Cube");
        node.transform
    });
    assert_eq!(
        state
            .transactions()
            .history_status(state.scene_history_context().unwrap())
            .unwrap()
            .len,
        1
    );
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state
        .world
        .expect_with_world(|scene| assert_eq!(scene.local_transform(cube), Some(original)));
    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    state
        .world
        .expect_with_world(|scene| assert_eq!(scene.local_transform(cube), Some(changed)));
}

#[test]
fn rotation_invalid_values_and_failed_parent_roll_back_scene_drafts_and_history() {
    for invalid in ["NaN", "inf", "-inf", "1e100", "", "degrees"] {
        let mut state = support::test_state();
        let cube = support::cube_id(&state);
        state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
        let before = state
            .world
            .expect_with_world(|scene| scene.find_node(cube).unwrap());
        let drafts = state.rotation_fields.clone();
        assert!(apply_inspector_binding(
            &mut state,
            &binding([
                ("name", UiBindingValue::string("Must Roll Back")),
                (
                    "transform.rotation_degrees.x",
                    UiBindingValue::string(invalid)
                ),
            ])
        )
        .is_err());
        assert_eq!(state.rotation_fields, drafts);
        assert!(state.inspector_edited_fields.is_empty());
        assert_eq!(
            state
                .transactions()
                .history_status(state.scene_history_context().unwrap())
                .unwrap()
                .len,
            0
        );
        state.world.expect_with_world(|scene| {
            let node = scene.find_node(cube).unwrap();
            assert_eq!(node.transform, before.transform);
            assert_eq!(node.name, before.name);
        });
    }
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
    let before = state
        .world
        .expect_with_world(|scene| scene.find_node(cube).unwrap());
    assert!(apply_inspector_binding(
        &mut state,
        &binding([
            ("transform.rotation_degrees.x", UiBindingValue::Float(45.0)),
            ("parent", UiBindingValue::Unsigned(999999)),
        ])
    )
    .is_err());
    state.world.expect_with_world(|scene| {
        let node = scene.find_node(cube).unwrap();
        assert_eq!(node.transform, before.transform);
        assert_eq!(node.parent, before.parent);
    });
    assert_eq!(
        state
            .transactions()
            .history_status(state.scene_history_context().unwrap())
            .unwrap()
            .len,
        0
    );
}

#[test]
fn rotation_multi_selection_uses_each_nodes_unrounded_local_axes_and_one_undo() {
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    let camera = support::camera_id(&state);
    let first = precise_transform();
    let second = Transform {
        rotation: xyz([-20.123456, 30.234567, 40.345678]),
        ..first
    };
    state.world.expect_with_world_mut(|scene| {
        scene.update_transform(cube, first).unwrap();
        scene.update_transform(camera, second).unwrap();
    });
    assert!(state.viewport_controller.selection_mut().replace(
        crate::scene::selection::WorldDomain::Edit,
        [camera, cube],
        Some(cube),
    ));
    assert!(apply_inspector_binding(
        &mut state,
        &binding([("transform.rotation_degrees.x", UiBindingValue::Float(45.0)),])
    )
    .unwrap());
    state.world.expect_with_world(|scene| {
        same_orientation(
            scene.local_transform(cube).unwrap().rotation,
            xyz([45.0, 24.234567, -35.345678]),
        );
        same_orientation(
            scene.local_transform(camera).unwrap().rotation,
            xyz([45.0, 30.234567, 40.345678]),
        );
    });
    assert_eq!(
        state.viewport_controller.selection().active_items().len(),
        2
    );
    assert_eq!(
        state
            .transactions()
            .history_status(state.scene_history_context().unwrap())
            .unwrap()
            .len,
        1
    );
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.local_transform(cube), Some(first));
        assert_eq!(scene.local_transform(camera), Some(second));
    });
}

#[test]
fn rotation_play_binding_mutates_only_active_play_world_and_supports_undo_redo() {
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    let authored = state
        .world
        .expect_with_world(|scene| scene.local_transform(cube).unwrap());
    assert!(state.enter_play_mode().unwrap());
    let instance = PlayInstanceId::for_test(380);
    let play_level = DefaultLevelManager::default().create_default_level();
    let original = precise_transform();
    play_level.with_world_mut(|scene| scene.update_transform(cube, original).unwrap());
    state
        .context
        .play_gateway_handle()
        .replace_for_play(
            Arc::new(InProcessGateway::for_authoring_level(play_level.clone())),
            Some(instance.raw()),
        )
        .unwrap();
    assert!(state.activate_play_selection_domain(instance));
    assert!(apply_inspector_binding(
        &mut state,
        &binding([
            ("transform.rotation_degrees.y", UiBindingValue::Float(45.0)),
            ("transform.translation.x", UiBindingValue::Float(9.0)),
        ])
    )
    .unwrap());
    let changed = play_level.with_world(|scene| {
        let transform = scene.local_transform(cube).unwrap();
        same_orientation(transform.rotation, xyz([13.123456, 45.0, -35.345678]));
        assert_eq!(transform.translation.x, 9.0);
        assert_eq!(transform.scale, original.scale);
        transform
    });
    state
        .world
        .expect_with_world(|scene| assert_eq!(scene.local_transform(cube), Some(authored)));
    assert_eq!(
        state
            .transactions()
            .history_status(HistoryContextId::PlaySession(instance))
            .unwrap()
            .len,
        1
    );
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    play_level.with_world(|scene| assert_eq!(scene.local_transform(cube), Some(original)));
    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    play_level.with_world(|scene| assert_eq!(scene.local_transform(cube), Some(changed)));
    state
        .world
        .expect_with_world(|scene| assert_eq!(scene.local_transform(cube), Some(authored)));
}

#[test]
fn rotation_static_target_rejection_rolls_back_an_already_applied_multi_selection_prefix() {
    use zircon_runtime::scene::components::Mobility;
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    let camera = support::camera_id(&state);
    // Selection iteration is ordered by entity id: let the mutable target apply first.
    let mut targets = [cube, camera];
    targets.sort_unstable();
    let originals = state.world.expect_with_world_mut(|scene| {
        let originals = targets.map(|entity| scene.local_transform(entity).unwrap());
        scene.set_mobility(targets[1], Mobility::Static).unwrap();
        originals
    });
    state.viewport_controller.selection_mut().replace(
        crate::scene::selection::WorldDomain::Edit,
        targets,
        Some(targets[0]),
    );
    let error = apply_inspector_binding(
        &mut state,
        &binding([("transform.rotation_degrees.x", UiBindingValue::Float(45.0))]),
    )
    .unwrap_err();
    assert!(
        matches!(
            &error,
            crate::ui::binding_dispatch::EditorBindingDispatchError::State(
                crate::ui::workbench::state::EditorStateOperationError::EditCommand(
                    crate::core::editing::engine::EditCommandError::ExternalEffect { source },
                )
            ) if source.to_string().contains("static entity")
        ),
        "{error:?}"
    );
    state.world.expect_with_world(|scene| {
        for (entity, original) in targets.into_iter().zip(originals) {
            assert_eq!(scene.local_transform(entity), Some(original));
        }
    });
    assert_eq!(
        state
            .transactions()
            .history_status(state.scene_history_context().unwrap())
            .unwrap()
            .len,
        0
    );
}

#[test]
fn rotation_drafts_are_scoped_to_selection_and_raw_reflected_rotation_stays_read_only() {
    use crate::core::editing::command::EditorCommand;
    use zircon_runtime_interface::reflect::ReflectedValue;
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    let camera = support::camera_id(&state);
    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
    state.update_rotation_field(0, "45".to_string());
    assert_eq!(
        state
            .snapshot()
            .inspector
            .unwrap()
            .rotation_degrees
            .unwrap()[0],
        "45"
    );
    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();
    assert!(!state.apply_inspector_changes().unwrap());
    state.world.expect_with_world(|scene| {
        let command = EditorCommand::set_reflected_scene_field(
            scene,
            cube,
            "zircon_runtime::scene::components::LocalTransform",
            "rotation",
            ReflectedValue::Vec4(Quat::IDENTITY.to_array()),
        );
        assert!(matches!(
            command,
            Err(crate::core::editing::engine::EditCommandError::ReflectError { source })
                if source.to_string().contains("not editable")
        ));
    });
    assert_eq!(
        state
            .transactions()
            .history_status(state.scene_history_context().unwrap())
            .unwrap()
            .len,
        0
    );
}
