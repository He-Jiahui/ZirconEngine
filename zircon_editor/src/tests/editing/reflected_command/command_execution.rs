use serde_json::json;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::reflect::ReflectedValue;

use crate::core::editing::command::EditorCommand;
use crate::core::editing::engine::HistoryContextId;

use super::support::{
    commit_command, read_reflected_field, scene_with_cloud_layer, transaction_scene,
    CLOUD_LAYER_TYPE_PATH, NAME_TYPE_PATH,
};

#[test]
fn reflected_editor_command_updates_fixed_component_and_undoes() {
    let mut scene = Scene::empty();
    let entity = scene
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let original_name = scene.find_node(entity).unwrap().name.clone();

    let command = EditorCommand::set_reflected_scene_field(
        &scene,
        entity,
        NAME_TYPE_PATH,
        "value",
        ReflectedValue::String("Cloud".to_string()),
    )
    .expect("fixed reflected field command should be captured")
    .expect("name change should create a command");

    assert_eq!(scene.find_node(entity).unwrap().name, original_name);

    let (level, transactions) = transaction_scene(scene, entity);
    commit_command(&transactions, command);
    assert_eq!(
        level.with_world(|scene| scene.find_node(entity).unwrap().name.clone()),
        "Cloud"
    );
    assert!(transactions.undo(HistoryContextId::Global).unwrap());
    assert_eq!(
        level.with_world(|scene| scene.find_node(entity).unwrap().name.clone()),
        original_name
    );

    assert!(transactions.redo(HistoryContextId::Global).unwrap());
    assert_eq!(
        level.with_world(|scene| scene.find_node(entity).unwrap().name.clone()),
        "Cloud"
    );

    let no_op = level.with_world(|scene| {
        EditorCommand::set_reflected_scene_field(
            scene,
            entity,
            NAME_TYPE_PATH,
            "value",
            ReflectedValue::String("Cloud".to_string()),
        )
    });
    let no_op = no_op.expect("same reflected value should be a valid no-op");
    assert!(no_op.is_none());
}

#[test]
fn reflected_editor_command_updates_dynamic_plugin_component_and_undoes() {
    let mut scene = scene_with_cloud_layer();
    let entity = scene
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    scene
        .set_dynamic_component(
            entity,
            CLOUD_LAYER_TYPE_PATH,
            json!({ "coverage": 0.25, "label": "thin" }),
        )
        .expect("dynamic component should attach");

    let command = EditorCommand::set_reflected_scene_field(
        &scene,
        entity,
        CLOUD_LAYER_TYPE_PATH,
        "coverage",
        ReflectedValue::Scalar(0.75),
    )
    .expect("dynamic reflected field command should be captured")
    .expect("coverage change should create a command");

    assert_eq!(
        read_reflected_field(&scene, entity, CLOUD_LAYER_TYPE_PATH, "coverage"),
        ReflectedValue::Scalar(0.25)
    );

    let (level, transactions) = transaction_scene(scene, entity);
    commit_command(&transactions, command);
    assert_eq!(
        level.with_world(|scene| read_reflected_field(
            scene,
            entity,
            CLOUD_LAYER_TYPE_PATH,
            "coverage"
        )),
        ReflectedValue::Scalar(0.75)
    );

    assert!(transactions.undo(HistoryContextId::Global).unwrap());
    assert_eq!(
        level.with_world(|scene| read_reflected_field(
            scene,
            entity,
            CLOUD_LAYER_TYPE_PATH,
            "coverage"
        )),
        ReflectedValue::Scalar(0.25)
    );

    assert!(transactions.redo(HistoryContextId::Global).unwrap());
    assert_eq!(
        level.with_world(|scene| read_reflected_field(
            scene,
            entity,
            CLOUD_LAYER_TYPE_PATH,
            "coverage"
        )),
        ReflectedValue::Scalar(0.75)
    );
}
