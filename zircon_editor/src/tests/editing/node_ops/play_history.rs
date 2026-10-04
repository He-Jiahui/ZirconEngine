use std::sync::Arc;

use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::DefaultLevelManager;

use crate::core::editing::command::EditorCommand;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditCommandError, EditorTransactionEngine, HistoryContextId};
use crate::core::gateway::{EditorRuntimeGatewayHandle, InProcessGateway};

#[test]
fn play_history_routes_scene_commands_to_the_exact_play_gateway() {
    let authoring_level = DefaultLevelManager::default().create_default_level();
    let play_level = DefaultLevelManager::default().create_default_level();
    let authoring_nodes = authoring_level.with_world(|scene| scene.nodes().len());
    let play_nodes = play_level.with_world(|scene| scene.nodes().len());
    let authoring_gateway = EditorRuntimeGatewayHandle::new(Arc::new(
        InProcessGateway::for_authoring_level(authoring_level.clone()),
    ));
    let play_gateway = EditorRuntimeGatewayHandle::detached();
    let instance = crate::core::play::PlayInstanceId::for_test(47);
    play_gateway
        .replace_for_play(
            Arc::new(InProcessGateway::for_authoring_level(play_level.clone())),
            Some(instance.raw()),
        )
        .unwrap();
    let engine = EditorTransactionEngine::new(CoreEditContext::with_world_gateways(
        authoring_gateway,
        play_gateway.clone(),
    ));
    let history = HistoryContextId::PlaySession(instance);

    let mut scope = engine.begin("create play node", history).unwrap();
    scope
        .push(EditorCommand::create_node(NodeKind::Cube))
        .unwrap();
    scope.commit().unwrap();

    assert_eq!(
        authoring_level.with_world(|scene| scene.nodes().len()),
        authoring_nodes,
        "a Play transaction must not mutate the authoring world"
    );
    assert_eq!(
        play_level.with_world(|scene| scene.nodes().len()),
        play_nodes + 1
    );
    assert!(engine.undo(history).unwrap());
    assert_eq!(
        play_level.with_world(|scene| scene.nodes().len()),
        play_nodes
    );
    assert!(engine.redo(history).unwrap());

    let replacement_level = DefaultLevelManager::default().create_default_level();
    let replacement_nodes = replacement_level.with_world(|scene| scene.nodes().len());
    play_gateway
        .replace_for_play(
            Arc::new(InProcessGateway::for_authoring_level(
                replacement_level.clone(),
            )),
            Some(instance.raw()),
        )
        .unwrap();
    assert!(matches!(
        engine.undo(history),
        Err(EditCommandError::WorldRouteStale {
            world_domain: crate::core::play::WorldDomain::Play(found),
        }) if found == instance
    ));
    assert_eq!(
        play_level.with_world(|scene| scene.nodes().len()),
        play_nodes + 1,
        "the historical command remains applied in its original world"
    );
    assert_eq!(
        replacement_level.with_world(|scene| scene.nodes().len()),
        replacement_nodes,
        "a stale history must not mutate the replacement play world"
    );
    assert_eq!(
        authoring_level.with_world(|scene| scene.nodes().len()),
        authoring_nodes
    );
}
