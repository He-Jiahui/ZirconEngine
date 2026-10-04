use serde_json::json;
use zircon_runtime_interface::math::Vec3;
use zircon_runtime_interface::reflect::ReflectedValue;

use crate::core::editing::engine::HistoryContextId;
use crate::core::editing::intent::EditorIntent;
use crate::tests::editing::support::{cube_and_camera, test_state};

use super::support::{cloud_layer_descriptor, read_reflected_field, CLOUD_LAYER_TYPE_PATH};

#[test]
fn reflected_edit_preserves_active_multi_selection() {
    let mut state = test_state();
    let (entity, camera) = cube_and_camera(&state);
    assert!(state.viewport_controller.selection_mut().replace(
        crate::scene::selection::WorldDomain::Edit,
        [camera, entity],
        Some(entity),
    ));
    state.update_name_field("Reflected Multi Selection".to_string());

    assert!(state.apply_inspector_changes().unwrap());
    assert_eq!(
        state
            .viewport_controller
            .selection()
            .active_items()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [camera, entity]
    );
    assert_eq!(
        state.viewport_controller.selection().active_primary(),
        Some(entity)
    );

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(
        state
            .viewport_controller
            .selection()
            .active_items()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [camera, entity]
    );
    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(
        state
            .viewport_controller
            .selection()
            .active_items()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [camera, entity]
    );
}

#[test]
fn reflected_inspector_batch_mutates_all_selected_nodes_in_one_history_record() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    let (cube_name_before, camera_name_before) = state.world.expect_with_world(|scene| {
        (
            scene.find_node(cube).unwrap().name.clone(),
            scene.find_node(camera).unwrap().name.clone(),
        )
    });
    assert!(state.viewport_controller.selection_mut().replace(
        crate::scene::selection::WorldDomain::Edit,
        [camera, cube],
        Some(cube),
    ));
    state.world.expect_with_world_mut(|scene| {
        scene
            .register_component_type(cloud_layer_descriptor())
            .expect("dynamic component descriptor should register");
        scene
            .set_dynamic_component(
                cube,
                CLOUD_LAYER_TYPE_PATH,
                json!({ "coverage": 0.25, "label": "cube" }),
            )
            .expect("dynamic component should attach to cube");
        scene
            .set_dynamic_component(
                camera,
                CLOUD_LAYER_TYPE_PATH,
                json!({ "coverage": 0.5, "label": "camera" }),
            )
            .expect("dynamic component should attach to camera");
    });
    state.update_name_field("Selected Batch".to_string());
    state.update_dynamic_component_field(
        format!("{CLOUD_LAYER_TYPE_PATH}.coverage"),
        "0.8".to_string(),
    );

    assert!(state.apply_inspector_changes().unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().name, "Selected Batch");
        assert_eq!(scene.find_node(camera).unwrap().name, "Selected Batch");
        assert_eq!(
            read_reflected_field(scene, cube, CLOUD_LAYER_TYPE_PATH, "coverage"),
            ReflectedValue::Scalar(0.8)
        );
        assert_eq!(
            read_reflected_field(scene, camera, CLOUD_LAYER_TYPE_PATH, "coverage"),
            ReflectedValue::Scalar(0.8)
        );
    });
    assert_eq!(
        state
            .transactions()
            .history_status(HistoryContextId::Global)
            .unwrap()
            .len,
        1
    );

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().name, cube_name_before);
        assert_eq!(scene.find_node(camera).unwrap().name, camera_name_before);
        assert_eq!(
            read_reflected_field(scene, cube, CLOUD_LAYER_TYPE_PATH, "coverage"),
            ReflectedValue::Scalar(0.25)
        );
        assert_eq!(
            read_reflected_field(scene, camera, CLOUD_LAYER_TYPE_PATH, "coverage"),
            ReflectedValue::Scalar(0.5)
        );
    });
    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().name, "Selected Batch");
        assert_eq!(scene.find_node(camera).unwrap().name, "Selected Batch");
        assert_eq!(
            read_reflected_field(scene, cube, CLOUD_LAYER_TYPE_PATH, "coverage"),
            ReflectedValue::Scalar(0.8)
        );
        assert_eq!(
            read_reflected_field(scene, camera, CLOUD_LAYER_TYPE_PATH, "coverage"),
            ReflectedValue::Scalar(0.8)
        );
    });
}

#[test]
fn reflected_inspector_batch_rejects_missing_dynamic_component_without_partial_mutation() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    let (cube_name_before, camera_name_before) = state.world.expect_with_world(|scene| {
        (
            scene.find_node(cube).unwrap().name.clone(),
            scene.find_node(camera).unwrap().name.clone(),
        )
    });
    state.world.expect_with_world_mut(|scene| {
        scene
            .register_component_type(cloud_layer_descriptor())
            .expect("dynamic component descriptor should register");
        scene
            .set_dynamic_component(
                cube,
                CLOUD_LAYER_TYPE_PATH,
                json!({ "coverage": 0.25, "label": "thin" }),
            )
            .expect("dynamic component should attach to the first selected node");
    });
    assert!(state.viewport_controller.selection_mut().replace(
        crate::scene::selection::WorldDomain::Edit,
        [cube, camera],
        Some(cube),
    ));
    state.update_name_field("Atomic Batch".to_string());
    state.update_dynamic_component_field(
        format!("{CLOUD_LAYER_TYPE_PATH}.coverage"),
        "0.8".to_string(),
    );

    assert!(state.apply_inspector_changes().is_err());
    state.world.expect_with_world(|scene| {
        assert_eq!(scene.find_node(cube).unwrap().name, cube_name_before);
        assert_eq!(scene.find_node(camera).unwrap().name, camera_name_before);
        assert_eq!(
            read_reflected_field(scene, cube, CLOUD_LAYER_TYPE_PATH, "coverage"),
            ReflectedValue::Scalar(0.25)
        );
    });
    assert_eq!(
        state
            .transactions()
            .history_status(HistoryContextId::Global)
            .unwrap()
            .len,
        0
    );
}

#[test]
fn mixed_selection_no_op_apply_produces_no_commands_or_property_changes() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    state.world.expect_with_world_mut(|scene| {
        scene.set_parent_checked(camera, Some(cube)).unwrap();
        let mut cube_transform = scene.find_node(cube).unwrap().transform;
        cube_transform.translation = Vec3::new(12.3456, 2.0, 3.0);
        cube_transform.scale = Vec3::new(1.23456, 1.0, 1.0);
        scene.update_transform(cube, cube_transform).unwrap();
        let mut camera_transform = scene.find_node(camera).unwrap().transform;
        camera_transform.translation = Vec3::new(-9.87654, 5.0, 6.0);
        camera_transform.scale = Vec3::new(0.9876543, 2.0, 2.0);
        scene.update_transform(camera, camera_transform).unwrap();
        scene
            .register_component_type(cloud_layer_descriptor())
            .unwrap();
        scene
            .set_dynamic_component(
                cube,
                CLOUD_LAYER_TYPE_PATH,
                json!({ "coverage": 0.25, "label": "cube" }),
            )
            .unwrap();
        scene
            .set_dynamic_component(
                camera,
                CLOUD_LAYER_TYPE_PATH,
                json!({ "coverage": 0.5, "label": "camera" }),
            )
            .unwrap();
    });
    assert!(state.viewport_controller.selection_mut().replace(
        crate::scene::selection::WorldDomain::Edit,
        [cube, camera],
        Some(camera),
    ));
    state.sync_selection_state();
    let before = state.world.snapshot();

    assert!(!state.apply_inspector_changes().unwrap());
    assert_eq!(state.world.snapshot(), before);
    assert_eq!(
        state
            .transactions()
            .history_status(HistoryContextId::Global)
            .unwrap()
            .len,
        0
    );
}

#[test]
fn edited_translation_axis_is_absolute_only_for_that_axis_on_each_selected_node() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    state.world.expect_with_world_mut(|scene| {
        scene.set_parent_checked(camera, Some(cube)).unwrap();
        let mut cube_transform = scene.find_node(cube).unwrap().transform;
        cube_transform.translation = Vec3::new(1.23456, 2.34567, 3.45678);
        scene.update_transform(cube, cube_transform).unwrap();
        let mut camera_transform = scene.find_node(camera).unwrap().transform;
        camera_transform.translation = Vec3::new(4.56789, 5.67891, 6.78912);
        scene.update_transform(camera, camera_transform).unwrap();
        scene
            .register_component_type(cloud_layer_descriptor())
            .unwrap();
        scene
            .set_dynamic_component(
                cube,
                CLOUD_LAYER_TYPE_PATH,
                json!({ "coverage": 0.25, "label": "cube" }),
            )
            .unwrap();
        scene
            .set_dynamic_component(
                camera,
                CLOUD_LAYER_TYPE_PATH,
                json!({ "coverage": 0.5, "label": "camera" }),
            )
            .unwrap();
    });
    assert!(state.viewport_controller.selection_mut().replace(
        crate::scene::selection::WorldDomain::Edit,
        [cube, camera],
        Some(camera),
    ));
    state.sync_selection_state();
    let before = state.world.expect_with_world(|scene| {
        [cube, camera].map(|node_id| {
            let node = scene.find_node(node_id).unwrap();
            (node.name.clone(), node.parent)
        })
    });

    state.update_translation_field(0, "9.25".to_owned());
    assert!(state.apply_inspector_changes().unwrap());
    state.world.expect_with_world(|scene| {
        for ((node_id, expected), (name, parent)) in [
            (cube, [9.25_f32, 2.34567, 3.45678]),
            (camera, [9.25_f32, 5.67891, 6.78912]),
        ]
        .into_iter()
        .zip(before)
        {
            let node = scene.find_node(node_id).unwrap();
            let actual = node.transform.translation.to_array();
            for (actual, expected) in actual.into_iter().zip(expected) {
                assert_eq!(actual.to_bits(), expected.to_bits());
            }
            assert_eq!(node.name, name);
            assert_eq!(node.parent, parent);
        }
        assert_eq!(
            read_reflected_field(scene, cube, CLOUD_LAYER_TYPE_PATH, "coverage"),
            ReflectedValue::Scalar(0.25)
        );
        assert_eq!(
            read_reflected_field(scene, camera, CLOUD_LAYER_TYPE_PATH, "coverage"),
            ReflectedValue::Scalar(0.5)
        );
    });
}

#[test]
fn draft_from_an_older_selection_revision_does_not_mutate_new_targets() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    state.viewport_controller.selection_mut().replace(
        crate::scene::selection::WorldDomain::Edit,
        [cube],
        Some(cube),
    );
    state.sync_selection_state();
    state.update_name_field("Old Selection Draft".to_owned());
    let before = state.world.snapshot();
    state.viewport_controller.selection_mut().replace(
        crate::scene::selection::WorldDomain::Edit,
        [camera],
        Some(camera),
    );

    assert!(!state.apply_inspector_changes().unwrap());
    assert_eq!(state.world.snapshot(), before);
    assert_eq!(
        state
            .transactions()
            .history_status(HistoryContextId::Global)
            .unwrap()
            .len,
        0
    );
}
