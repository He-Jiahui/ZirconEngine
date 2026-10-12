use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use zircon_runtime::scene::components::{Name, NodeKind};
use zircon_runtime::scene::ecs::LifecycleEventKind;
use zircon_runtime::scene::{DefaultLevelManager, NodeId, Scene};
use zircon_runtime_interface::math::{Transform, Vec3};
use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue};
use zircon_runtime_interface::world_sync::LifecycleTargetChange;

use crate::core::editing::command::{EditorCommand, NodeEditState, UpdateNodeCommand};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{
    CommandEffect, CommandExecutionError, EditCommand, EditCommandError, EditorTransactionEngine,
    HistoryContextId,
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

fn install_gateway_target_change(
    scene: &mut Scene,
    kind: LifecycleEventKind,
    entity: Option<NodeId>,
    cause: LifecycleTargetChange,
    calls: Arc<AtomicUsize>,
) {
    let replacement = DefaultLevelManager::default()
        .create_default_level()
        .with_world_mut(std::mem::take);
    let replacement = Mutex::new(Some(replacement));
    scene.observe_component_lifecycle::<Name>(kind, move |world, event| {
        if entity.is_some_and(|entity| entity != event.entity()) {
            return;
        }
        calls.fetch_add(1, Ordering::SeqCst);
        match cause {
            LifecycleTargetChange::World => *world = replacement.lock().unwrap().take().unwrap(),
            LifecycleTargetChange::Schedule => *world.schedule_mut() = Default::default(),
        }
    });
}

fn interrupted_gateway_primary(error: &CommandExecutionError) -> &EditCommandError {
    assert_eq!(error.effect, CommandEffect::Interrupted);
    let EditCommandError::LifecycleInterrupted {
        source,
        post_callback_errors,
    } = &error.source
    else {
        panic!("producer cancellation must persist ahead of the gateway diagnostic");
    };
    assert_eq!(post_callback_errors.len(), 1);
    let EditCommandError::ExternalEffect {
        source: gateway_error,
    } = &post_callback_errors[0]
    else {
        panic!("the actual post-callback gateway error must retain its type");
    };
    assert_eq!(
        gateway_error.downcast_ref::<GatewayError>(),
        Some(&GatewayError::Protocol {
            message: "gateway reported after executing a world write".to_owned(),
        })
    );
    source.as_ref()
}

#[test]
fn create_cancellation_keeps_the_post_callback_gateway_error() {
    for cause in [
        LifecycleTargetChange::World,
        LifecycleTargetChange::Schedule,
    ] {
        let level = DefaultLevelManager::default().create_default_level();
        let gateway = EditorRuntimeGatewayHandle::detached();
        let mut context = CoreEditContext::new(gateway.clone());
        context
            .bind_scene(level.clone(), SceneSelection::new(Vec::new(), None))
            .unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        level.with_world_mut(|scene| {
            install_gateway_target_change(
                scene,
                LifecycleEventKind::Add,
                None,
                cause,
                calls.clone(),
            );
        });
        gateway
            .replace(Arc::new(CallbackThenErrorGateway::new(level.clone())))
            .unwrap();
        let mut command = EditorCommand::create_node(NodeKind::Cube);

        let error = command.apply(&mut context).unwrap_err();
        let EditCommandError::SceneMutation { source, .. } = interrupted_gateway_primary(&error)
        else {
            panic!("create cancellation must retain its typed SceneError");
        };
        assert_eq!(source.lifecycle_target_change(), Some(cause));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn update_cancellation_stops_remaining_fields_and_keeps_gateway_error() {
    for cause in [
        LifecycleTargetChange::World,
        LifecycleTargetChange::Schedule,
    ] {
        let level = DefaultLevelManager::default().create_default_level();
        let (cube, before) = level.with_world(|scene| {
            let cube = scene
                .nodes()
                .iter()
                .find(|node| matches!(node.kind, NodeKind::Cube))
                .unwrap()
                .id;
            (cube, NodeEditState::capture(scene, cube).unwrap())
        });
        let mut after = before.clone();
        after.name = "Interrupted update".to_owned();
        after.transform = Transform::from_translation(Vec3::new(99.0, 0.0, 0.0));
        let mut command = EditorCommand::UpdateNode(
            UpdateNodeCommand::new(cube, before.clone(), after, false).unwrap(),
        );
        let gateway = EditorRuntimeGatewayHandle::detached();
        let mut context = CoreEditContext::new(gateway.clone());
        context
            .bind_scene(level.clone(), SceneSelection::new(vec![cube], Some(cube)))
            .unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        level.with_world_mut(|scene| {
            install_gateway_target_change(
                scene,
                LifecycleEventKind::Replace,
                Some(cube),
                cause,
                calls.clone(),
            );
        });
        gateway
            .replace(Arc::new(CallbackThenErrorGateway::new(level.clone())))
            .unwrap();

        let error = command.apply(&mut context).unwrap_err();
        let EditCommandError::SceneMutation { source, .. } = interrupted_gateway_primary(&error)
        else {
            panic!("update cancellation must retain its typed SceneError");
        };
        assert_eq!(source.lifecycle_target_change(), Some(cause));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            level.with_world(|scene| scene.find_node(cube).unwrap().transform),
            before.transform
        );
    }
}

#[test]
fn reflected_cancellation_keeps_the_typed_source_and_gateway_error() {
    for cause in [
        LifecycleTargetChange::World,
        LifecycleTargetChange::Schedule,
    ] {
        let level = DefaultLevelManager::default().create_default_level();
        let cube = level.with_world(|scene| {
            scene
                .nodes()
                .iter()
                .find(|node| matches!(node.kind, NodeKind::Cube))
                .unwrap()
                .id
        });
        let gateway = EditorRuntimeGatewayHandle::detached();
        let mut context = CoreEditContext::new(gateway.clone());
        context
            .bind_scene(level.clone(), SceneSelection::new(vec![cube], Some(cube)))
            .unwrap();
        let mut command = level
            .with_world(|scene| {
                EditorCommand::set_reflected_scene_field(
                    scene,
                    cube,
                    NAME_TYPE_PATH,
                    "value",
                    ReflectedValue::String("Interrupted reflected update".to_owned()),
                )
            })
            .unwrap()
            .unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        level.with_world_mut(|scene| {
            install_gateway_target_change(
                scene,
                LifecycleEventKind::Replace,
                Some(cube),
                cause,
                calls.clone(),
            );
        });
        gateway
            .replace(Arc::new(CallbackThenErrorGateway::new(level.clone())))
            .unwrap();

        let error = command.apply(&mut context).unwrap_err();
        let EditCommandError::ReflectError { source } = interrupted_gateway_primary(&error) else {
            panic!("reflection cancellation must keep the actual ReflectError");
        };
        let reflected = source
            .downcast_ref::<ReflectError>()
            .expect("no string or io::Error conversion");
        assert_eq!(reflected.lifecycle_target_change(), Some(cause));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

fn duplicate_dispatch_gateway_diagnostic(error: &EditCommandError) -> &GatewayError {
    let EditCommandError::ExternalEffect { source } = error else {
        panic!("a real gateway diagnostic must keep its boxed type");
    };
    source
        .downcast_ref::<GatewayError>()
        .expect("no string or io::Error conversion of the gateway diagnostic")
}

#[test]
fn ordinary_create_keeps_existing_protocol_priority_after_duplicate_callback_and_dispatch_error() {
    let level = DefaultLevelManager::default().create_default_level();
    let initial_node_count = level.with_world(|scene| scene.nodes().len());
    let dispatch_calls = Arc::new(AtomicUsize::new(0));
    let callback_attempts = Arc::new(AtomicUsize::new(0));
    let gateway = EditorRuntimeGatewayHandle::detached();
    let mut context = CoreEditContext::new(gateway.clone());
    context
        .bind_scene(level.clone(), SceneSelection::new(Vec::new(), None))
        .unwrap();
    gateway
        .replace(Arc::new(
            CallbackThenErrorGateway::duplicate_first_write_then_error(
                level.clone(),
                dispatch_calls.clone(),
                callback_attempts.clone(),
            ),
        ))
        .unwrap();
    let mut command = EditorCommand::create_node(NodeKind::Cube);

    let error = command.apply(&mut context).unwrap_err();

    assert_eq!(error.effect, CommandEffect::Applied);
    assert_eq!(
        duplicate_dispatch_gateway_diagnostic(&error.source),
        &GatewayError::Protocol {
            message: "borrowed world callback was invoked more than once".to_owned(),
        }
    );
    assert_eq!(
        level.with_world(|scene| scene.nodes().len()),
        initial_node_count + 1
    );
    assert_eq!(dispatch_calls.load(Ordering::SeqCst), 1);
    assert_eq!(callback_attempts.load(Ordering::SeqCst), 2);

    command
        .revert(&mut context)
        .expect("the one-shot fixture must allow the ordinary compensation write");
    assert_eq!(
        level.with_world(|scene| scene.nodes().len()),
        initial_node_count
    );
    assert_eq!(dispatch_calls.load(Ordering::SeqCst), 2);
    assert_eq!(callback_attempts.load(Ordering::SeqCst), 3);
}

#[test]
fn real_lifecycle_cancellation_keeps_two_gateway_diagnostics_and_stops_fault_drop_dispatch() {
    for cause in [
        LifecycleTargetChange::World,
        LifecycleTargetChange::Schedule,
    ] {
        let level = DefaultLevelManager::default().create_default_level();
        let gateway = EditorRuntimeGatewayHandle::detached();
        let mut context = CoreEditContext::new(gateway.clone());
        context
            .bind_scene(level.clone(), SceneSelection::new(Vec::new(), None))
            .unwrap();
        let observer_calls = Arc::new(AtomicUsize::new(0));
        let dispatch_calls = Arc::new(AtomicUsize::new(0));
        let callback_attempts = Arc::new(AtomicUsize::new(0));
        level.with_world_mut(|scene| {
            install_gateway_target_change(
                scene,
                LifecycleEventKind::Add,
                None,
                cause,
                observer_calls.clone(),
            );
        });
        gateway
            .replace(Arc::new(
                CallbackThenErrorGateway::duplicate_first_write_then_error(
                    level.clone(),
                    dispatch_calls.clone(),
                    callback_attempts.clone(),
                ),
            ))
            .unwrap();
        let engine = EditorTransactionEngine::new(context);
        let mut scope = engine
            .begin(
                "create with duplicate gateway callback",
                HistoryContextId::Global,
            )
            .unwrap();

        let error = scope
            .push(EditorCommand::create_node(NodeKind::Cube))
            .unwrap_err();
        assert!(error.is_interrupted());
        let EditCommandError::LifecycleInterrupted {
            source,
            post_callback_errors,
        } = &error
        else {
            panic!("the actual producer cancellation must remain the primary error");
        };
        let EditCommandError::SceneMutation { operation, source } = source.as_ref() else {
            panic!("the native scene cancellation must retain its original SceneError");
        };
        assert_eq!(*operation, "create node");
        assert_eq!(source.lifecycle_target_change(), Some(cause));
        assert_eq!(post_callback_errors.len(), 2);
        assert_eq!(
            duplicate_dispatch_gateway_diagnostic(&post_callback_errors[0]),
            &GatewayError::Protocol {
                message: "borrowed world callback was invoked more than once".to_owned(),
            }
        );
        assert_eq!(
            duplicate_dispatch_gateway_diagnostic(&post_callback_errors[1]),
            &GatewayError::Protocol {
                message: "gateway reported after executing a world write".to_owned(),
            }
        );
        assert_eq!(observer_calls.load(Ordering::SeqCst), 1);
        assert_eq!(dispatch_calls.load(Ordering::SeqCst), 1);
        assert_eq!(callback_attempts.load(Ordering::SeqCst), 2);

        drop(scope);
        assert_eq!(dispatch_calls.load(Ordering::SeqCst), 1);
        assert_eq!(callback_attempts.load(Ordering::SeqCst), 2);
        assert!(matches!(
            engine.undo(HistoryContextId::Global),
            Err(EditCommandError::EngineFaulted { .. })
        ));
        assert_eq!(dispatch_calls.load(Ordering::SeqCst), 1);
        assert_eq!(callback_attempts.load(Ordering::SeqCst), 2);
        drop(engine);
        assert_eq!(dispatch_calls.load(Ordering::SeqCst), 1);
        assert_eq!(callback_attempts.load(Ordering::SeqCst), 2);
        assert_eq!(observer_calls.load(Ordering::SeqCst), 1);
    }
}
