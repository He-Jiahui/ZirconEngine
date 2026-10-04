use std::sync::Arc;

use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::reflect::ReflectedValue;

use crate::core::editing::command::EditorCommand;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{
    CommandEffect, EditCommand, EditCommandError, EditorTransactionEngine, HistoryContextId,
};
use crate::core::editing::selection::SceneSelection;
use crate::core::gateway::{EditorRuntimeGatewayHandle, GatewayError};

use super::support::CallbackThenErrorGateway;

const NAME_TYPE_PATH: &str = "zircon_runtime::scene::components::Name";

#[test]
fn create_apply_is_applied_when_gateway_fails_after_the_callback() {
    let level = DefaultLevelManager::default().create_default_level();
    let initial_node_count = level.with_world(|scene| scene.nodes().len());
    let gateway = EditorRuntimeGatewayHandle::detached();
    let mut context = CoreEditContext::new(gateway.clone());
    gateway
        .replace(Arc::new(CallbackThenErrorGateway::new(level.clone())))
        .expect("replace gateway with callback-then-error fixture");
    let mut command = EditorCommand::create_node(NodeKind::Cube);

    let error = command
        .apply(&mut context)
        .expect_err("gateway error after create must surface");
    assert_eq!(error.effect, CommandEffect::Applied);
    let EditCommandError::ExternalEffect { source } = error.source else {
        panic!("post-callback gateway error must remain typed");
    };
    assert_eq!(
        source.downcast_ref::<GatewayError>(),
        Some(&GatewayError::Protocol {
            message: "gateway reported after executing a world write".to_owned(),
        })
    );
    assert_eq!(
        level.with_world(|scene| scene.nodes().len()),
        initial_node_count + 1,
        "the callback mutation must be visible before transaction recovery"
    );

    command
        .revert(&mut context)
        .expect("the retained create record must support recovery");
    assert_eq!(
        level.with_world(|scene| scene.nodes().len()),
        initial_node_count
    );
}

#[test]
fn update_apply_is_applied_when_gateway_fails_after_the_callback() {
    let level = DefaultLevelManager::default().create_default_level();
    let (cube, original_name) = level.with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .map(|node| (node.id, node.name.clone()))
            .expect("default editor level must contain a cube")
    });
    let gateway = EditorRuntimeGatewayHandle::detached();
    let mut context = CoreEditContext::new(gateway.clone());
    gateway
        .replace(Arc::new(CallbackThenErrorGateway::new(level.clone())))
        .expect("replace gateway with callback-then-error fixture");
    let mut command = level
        .with_world(|scene| EditorCommand::rename_node(scene, cube, "Gateway Updated".to_owned()))
        .expect("capture cube rename command")
        .expect("renaming to a distinct name must create a command");

    let error = command
        .apply(&mut context)
        .expect_err("gateway error after update must surface");
    assert_eq!(error.effect, CommandEffect::Applied);
    assert_eq!(
        level.with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        "Gateway Updated",
        "the callback mutation must be visible before transaction recovery"
    );

    command
        .revert(&mut context)
        .expect("the update command must remain reversible after recovery");
    assert_eq!(
        level.with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        original_name
    );
}

#[test]
fn reflected_undo_compensates_when_gateway_errors_after_successful_restore() {
    let level = DefaultLevelManager::default().create_default_level();
    let (cube, original_name) = level.with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .map(|node| (node.id, node.name.clone()))
            .expect("default editor level must contain a cube")
    });
    let gateway = EditorRuntimeGatewayHandle::detached();
    let mut context = CoreEditContext::new(gateway.clone());
    context
        .bind_scene(level.clone(), SceneSelection::new(vec![cube], Some(cube)))
        .expect("bind default level to editor context");
    let engine = EditorTransactionEngine::new(context);
    let command = level
        .with_world(|scene| {
            EditorCommand::set_reflected_scene_field(
                scene,
                cube,
                NAME_TYPE_PATH,
                "value",
                ReflectedValue::String("Gateway Reflected".to_owned()),
            )
        })
        .expect("capture reflected name command")
        .expect("a distinct reflected value must create a command");
    let mut scope = engine
        .begin("set reflected name", HistoryContextId::Global)
        .expect("begin reflected transaction");
    scope.push(command).expect("apply reflected command");
    scope.commit().expect("commit reflected command");

    gateway
        .replace(Arc::new(CallbackThenErrorGateway::new(level.clone())))
        .expect("replace gateway with callback-then-error fixture");

    let error = engine
        .undo(HistoryContextId::Global)
        .expect_err("gateway error after reflected restore must surface");
    let EditCommandError::ExternalEffect { source } = error else {
        panic!("gateway error after reflected restore must remain typed");
    };
    assert_eq!(
        source.downcast_ref::<GatewayError>(),
        Some(&GatewayError::Protocol {
            message: "gateway reported after executing a world write".to_owned(),
        })
    );
    assert_eq!(
        level.with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        "Gateway Reflected",
        "the transaction recovery must reapply the reflected write"
    );

    assert!(
        engine.undo(HistoryContextId::Global).unwrap(),
        "the compensated reflected command must remain undoable"
    );
    assert_eq!(
        level.with_world(|scene| scene.find_node(cube).unwrap().name.clone()),
        original_name
    );
}

#[test]
fn delete_undo_compensates_when_gateway_errors_after_successful_restore() {
    let level = DefaultLevelManager::default().create_default_level();
    let cube = level.with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .map(|node| node.id)
            .expect("default editor level must contain a cube")
    });
    let gateway = EditorRuntimeGatewayHandle::detached();
    let mut context = CoreEditContext::new(gateway.clone());
    context
        .bind_scene(level.clone(), SceneSelection::new(vec![cube], Some(cube)))
        .expect("bind default level to editor context");
    let engine = EditorTransactionEngine::new(context);
    let delete = level
        .with_world(|scene| EditorCommand::delete_node(scene, cube))
        .expect("capture cube deletion command");
    let mut scope = engine
        .begin("delete cube", HistoryContextId::Global)
        .expect("begin deletion transaction");
    scope.push(delete).expect("apply cube deletion");
    scope.commit().expect("commit cube deletion");
    assert!(level.with_world(|scene| !scene.contains_entity(cube)));

    gateway
        .replace(Arc::new(CallbackThenErrorGateway::new(level.clone())))
        .expect("replace gateway with callback-then-error fixture");

    let error = engine
        .undo(HistoryContextId::Global)
        .expect_err("gateway error after restore must surface");
    let EditCommandError::ExternalEffect { source } = error else {
        panic!("gateway error after restore must remain typed");
    };
    assert_eq!(
        source.downcast_ref::<GatewayError>(),
        Some(&GatewayError::Protocol {
            message: "gateway reported after executing a world write".to_owned(),
        })
    );
    assert!(
        level.with_world(|scene| !scene.contains_entity(cube)),
        "the transaction recovery must reapply the delete after a post-callback error"
    );

    assert!(
        engine.undo(HistoryContextId::Global).unwrap(),
        "the compensated delete must retain its batch for a later undo retry"
    );
    assert!(level.with_world(|scene| scene.contains_entity(cube)));
}
