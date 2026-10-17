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
    // SAFETY: this fixture only retires World/schedule ownership; native lifecycle
    // cancellation prevents adoption of any replacement owner's snapshot.
    scene.observe_component_lifecycle::<Name>(kind, unsafe {
        zircon_runtime::core::framework::retained_memory::RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |world, event| {
        if entity.is_some_and(|entity| entity != event.entity()) {
            return;
        }
        calls.fetch_add(1, Ordering::SeqCst);
        match cause {
            LifecycleTargetChange::World => *world = replacement.lock().unwrap().take().unwrap(),
            LifecycleTargetChange::Schedule => *world.schedule_mut() = Default::default(),
        }
    }) });
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

fn create_history_context_v1(level: zircon_runtime::scene::LevelSystem) -> CoreEditContext {
    let mut context = CoreEditContext::new(EditorRuntimeGatewayHandle::detached());
    context
        .bind_scene(level, SceneSelection::new(Vec::new(), None))
        .unwrap();
    context
}

type CreateCallbackStateV1 = (String, Arc<AtomicUsize>);

fn create_callback_extra_v1(
    _: &CreateCallbackStateV1,
    _: &Scene,
    _: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
    Ok(())
}

fn first_create_name_v1(
    state: &CreateCallbackStateV1,
    world: &mut Scene,
    event: &zircon_runtime::scene::ecs::ComponentLifecycleEvent,
) {
    if state.1.fetch_add(1, Ordering::SeqCst) == 0 {
        world.insert(event.entity(), Name(state.0.clone())).unwrap();
    }
}

#[test]
fn create_history_retains_first_callback_payload_on_undo_redo() {
    use zircon_runtime::core::framework::retained_memory::RetainedLifecycleObserverV1;
    let level = DefaultLevelManager::default().create_default_level();
    let calls = Arc::new(AtomicUsize::new(0));
    let name = "first callback payload".repeat(128);
    level.with_world_mut(|world| {
        world.observe_component_lifecycle::<Name>(
            LifecycleEventKind::Add,
            RetainedLifecycleObserverV1::new(
                (name.clone(), calls.clone()),
                create_callback_extra_v1,
                first_create_name_v1,
            ),
        );
    });
    let engine = EditorTransactionEngine::new(create_history_context_v1(level.clone()));
    let mut scope = engine
        .begin("bounded callback create", HistoryContextId::Global)
        .unwrap();
    scope
        .push(EditorCommand::create_node(NodeKind::Cube))
        .unwrap();
    scope.commit().unwrap();
    let id = level.with_world(|world| {
        world
            .nodes()
            .iter()
            .find(|node| node.name == name)
            .unwrap()
            .id
    });
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    engine.undo(HistoryContextId::Global).unwrap();
    assert!(!level.with_world(|world| world.contains_entity(id)));
    engine.redo(HistoryContextId::Global).unwrap();
    assert_eq!(
        level.with_world(|world| world.get::<Name>(id).unwrap().0.clone()),
        name
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn create_history_refuses_callback_snapshot_budget_before_spawn_or_callback() {
    use crate::core::editing::engine::{HistoryRetentionBudgetV1, HistoryRetentionDomainV1};
    use zircon_runtime::core::framework::retained_memory::RetainedLifecycleObserverV1;
    use zircon_runtime::scene::components::MeshRenderer;
    use zircon_runtime_interface::retained_memory::RetentionContractError;

    let domain = HistoryRetentionDomainV1::try_new(HistoryRetentionBudgetV1 {
        record_bytes: 32 * 1024,
        history_bytes: 1024 * 1024,
        committed_bytes: 4 * 1024 * 1024,
        staging_bytes: 1024 * 1024,
    })
    .unwrap();
    // A proof survives moves, but not clone, replacement, registration or removal.
    let mut world = Scene::empty();
    let quote = world
        .create_history_quote_v1(NodeKind::Cube, domain.model(), 32 * 1024)
        .unwrap();
    let mut moved = std::mem::take(&mut world);
    moved.validate_create_history_quote_v1(&quote).unwrap();
    assert!(matches!(
        world.validate_create_history_quote_v1(&quote),
        Err(RetentionContractError::Stale)
    ));
    assert!(matches!(
        moved.clone().validate_create_history_quote_v1(&quote),
        Err(RetentionContractError::Stale)
    ));
    let observer = moved.observe_component_lifecycle::<Name>(
        LifecycleEventKind::Add,
        RetainedLifecycleObserverV1::read_only_v1(|_, _| {}),
    );
    assert!(matches!(
        moved.validate_create_history_quote_v1(&quote),
        Err(RetentionContractError::Stale)
    ));
    let before_remove = moved
        .create_history_quote_v1(NodeKind::Cube, domain.model(), 32 * 1024)
        .unwrap();
    moved.remove_observer(observer).unwrap();
    assert!(matches!(
        moved.validate_create_history_quote_v1(&before_remove),
        Err(RetentionContractError::Stale)
    ));

    for mesh in [false, true] {
        let level = DefaultLevelManager::default().create_default_level();
        let engine = EditorTransactionEngine::with_retention_domain_v1(
            create_history_context_v1(level.clone()),
            128,
            domain.clone(),
        )
        .unwrap();
        let mut seed = engine
            .begin("existing history", HistoryContextId::Global)
            .unwrap();
        seed.push(EditorCommand::create_node(NodeKind::Cube))
            .unwrap();
        seed.commit().unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let bad = level.with_world_mut(|world| {
            let observer = RetainedLifecycleObserverV1::new(
                ("large-callback".repeat(16 * 1024), calls.clone()),
                create_callback_extra_v1,
                first_create_name_v1,
            );
            if mesh {
                // Quote the complete MeshImport entry, including its Replace tail.
                world.observe_component_lifecycle::<MeshRenderer>(
                    LifecycleEventKind::Replace,
                    observer,
                )
            } else {
                world.observe_component_lifecycle::<Name>(LifecycleEventKind::Add, observer)
            }
        });
        let command = || {
            if mesh {
                let renderer = MeshRenderer::default();
                EditorCommand::import_mesh(renderer.model, renderer.material)
            } else {
                EditorCommand::create_node(NodeKind::Cube)
            }
        };
        let before = level.with_world(|world| (world.nodes().len(), world.world_generation()));
        let charges = domain.snapshot().unwrap();
        let mut refused = engine
            .begin("refused callback", HistoryContextId::Global)
            .unwrap();
        let error = refused.push(command()).unwrap_err();
        assert!(matches!(error, EditCommandError::Retention(_)));
        drop(refused);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            level.with_world(|world| (world.nodes().len(), world.world_generation())),
            before
        );
        assert_eq!(domain.snapshot().unwrap(), charges);
        assert!(
            engine
                .history_status(HistoryContextId::Global)
                .unwrap()
                .can_undo
        );
        engine.undo(HistoryContextId::Global).unwrap();
        assert_eq!(level.with_world(|world| world.nodes().len()), before.0 - 1);

        // Removing the bad producer must leave this same owner usable.
        let legal_name = if mesh {
            "legal mesh tail"
        } else {
            "legal cube callback"
        }
        .repeat(16);
        level.with_world_mut(|world| {
            world.remove_observer(bad).unwrap();
            let observer = RetainedLifecycleObserverV1::new(
                (legal_name.clone(), calls.clone()),
                create_callback_extra_v1,
                first_create_name_v1,
            );
            if mesh {
                world.observe_component_lifecycle::<MeshRenderer>(
                    LifecycleEventKind::Replace,
                    observer,
                );
            } else {
                world.observe_component_lifecycle::<Name>(LifecycleEventKind::Add, observer);
            }
        });
        let mut retry = engine
            .begin("legal callback retry", HistoryContextId::Global)
            .unwrap();
        retry.push(command()).unwrap();
        retry.commit().unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let id = level.with_world(|world| {
            let id = world
                .nodes()
                .iter()
                .find(|node| node.name == legal_name)
                .unwrap()
                .id;
            if mesh {
                let actual = world.get::<MeshRenderer>(id).unwrap();
                let expected = MeshRenderer::default();
                assert_eq!(actual.model, expected.model);
                assert_eq!(actual.material, expected.material);
            }
            id
        });
        engine.undo(HistoryContextId::Global).unwrap();
        assert!(!level.with_world(|world| world.contains_entity(id)));
        engine.redo(HistoryContextId::Global).unwrap();
        assert_eq!(
            level.with_world(|world| world.get::<Name>(id).unwrap().0.clone()),
            legal_name
        );
    }
}

#[derive(Debug)]
struct CreateCallbackExtraV1(u32);
zircon_runtime_interface::impl_retained_owner_v1!(CreateCallbackExtraV1 {});
impl zircon_runtime::scene::ecs::Component for CreateCallbackExtraV1 {}

fn child_create_extra_v1(
    state: &CreateCallbackStateV1,
    _: &Scene,
    m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
    // Parent replacement and child authored Name coexist. The child label is
    // allocated with this exact requested capacity, before any snapshot clone.
    m.owned(state)?;
    m.array::<u8>(
        state
            .0
            .len()
            .checked_add(" child".len())
            .ok_or(zircon_runtime_interface::retained_memory::RetentionContractError::Overflow)?,
    )
}

fn child_create_callback_v1(
    state: &CreateCallbackStateV1,
    world: &mut Scene,
    event: &zircon_runtime::scene::ecs::ComponentLifecycleEvent,
) {
    if state.1.fetch_add(1, Ordering::SeqCst) != 0 {
        return;
    }
    world.insert(event.entity(), Name(state.0.clone())).unwrap();
    world
        .insert(event.entity(), CreateCallbackExtraV1(71))
        .unwrap();
    let mut name = String::with_capacity(state.0.len() + " child".len());
    name.push_str(&state.0);
    name.push_str(" child");
    world.spawn((Name(name),)).unwrap();
}

#[test]
fn create_history_preserves_callback_child_and_custom_component_semantics() {
    use zircon_runtime::core::framework::retained_memory::RetainedLifecycleObserverV1;
    let level = DefaultLevelManager::default().create_default_level();
    let calls = Arc::new(AtomicUsize::new(0));
    let name = "created parent".repeat(16);
    level.with_world_mut(|world| {
        world.observe_component_lifecycle::<Name>(
            LifecycleEventKind::Add,
            RetainedLifecycleObserverV1::new(
                (name.clone(), calls.clone()),
                child_create_extra_v1,
                child_create_callback_v1,
            ),
        );
    });
    let engine = EditorTransactionEngine::new(create_history_context_v1(level.clone()));
    let mut scope = engine
        .begin("callback side effects", HistoryContextId::Global)
        .unwrap();
    scope
        .push(EditorCommand::create_node(NodeKind::Cube))
        .unwrap();
    scope.commit().unwrap();
    let child_name = format!("{name} child");
    let (parent, child) = level.with_world(|world| {
        let parent = world
            .nodes()
            .iter()
            .find(|node| node.name == name)
            .unwrap()
            .id;
        let child = world
            .nodes()
            .iter()
            .find(|node| node.name == child_name)
            .unwrap()
            .id;
        assert_eq!(world.get::<CreateCallbackExtraV1>(parent).unwrap().0, 71);
        (parent, child)
    });
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    engine.undo(HistoryContextId::Global).unwrap();
    assert!(!level.with_world(|world| world.contains_entity(parent)));
    assert!(level.with_world(|world| world.contains_entity(child)));
    engine.redo(HistoryContextId::Global).unwrap();
    level.with_world(|world| {
        assert_eq!(world.get::<Name>(parent).unwrap().0, name);
        assert!(world.contains_entity(child));
        // Existing Create replay restores NodeRecord's fixed fields. Opaque
        // callback-only components and independent children are not replayed.
        assert!(!world.contains_component::<CreateCallbackExtraV1>(parent));
    });
    assert_eq!(calls.load(Ordering::SeqCst), 3);
}

#[test]
fn create_history_lifecycle_provider_breach_is_applied_not_admission_refusal() {
    use zircon_runtime::core::framework::retained_memory::RetainedLifecycleObserverV1;
    let level = DefaultLevelManager::default().create_default_level();
    level.with_world_mut(|world| {
        world.observe_component_lifecycle::<Name>(
            LifecycleEventKind::Add,
            RetainedLifecycleObserverV1::new(
                (),
                |_, _, _| Ok(()),
                |_, world, event| {
                    // Deliberately dishonest producer: the post-write borrowed
                    // check must reject before cloning this unreserved payload.
                    world
                        .insert(event.entity(), Name("breach".repeat(4096)))
                        .unwrap();
                },
            ),
        );
    });
    let count = level.with_world(|world| world.nodes().len());
    let mut context = create_history_context_v1(level.clone());
    let mut command = EditorCommand::create_node(NodeKind::Cube);
    let error = command.apply(&mut context).unwrap_err();
    assert_eq!(error.effect, CommandEffect::Applied);
    assert!(matches!(error.source, EditCommandError::Retention(_)));
    assert_eq!(level.with_world(|world| world.nodes().len()), count + 1);
    assert!(level.with_world(|world| {
        world
            .nodes()
            .iter()
            .any(|node| node.name.starts_with("breachbreach"))
    }));
}

fn reentrant_create_extra_v1(
    state: &CreateCallbackStateV1,
    _: &Scene,
    m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
    // A owns B's immutable source before B exists. Cover B's state clone,
    // callback-owner header/function fields, and B's eventual Name replacement
    // across the complete operation rather than summing only initial buckets.
    m.owned(state)?;
    m.array::<usize>(4)?;
    m.owned(state)
}

fn reentrant_mesh_callback_v1(
    state: &CreateCallbackStateV1,
    world: &mut Scene,
    event: &zircon_runtime::scene::ecs::ComponentLifecycleEvent,
) {
    state.1.fetch_add(1, Ordering::SeqCst);
    world.insert(event.entity(), Name(state.0.clone())).unwrap();
}

fn reentrant_name_callback_v1(
    state: &CreateCallbackStateV1,
    world: &mut Scene,
    _: &zircon_runtime::scene::ecs::ComponentLifecycleEvent,
) {
    use zircon_runtime::core::framework::retained_memory::RetainedLifecycleObserverV1;
    use zircon_runtime::scene::components::MeshRenderer;
    if state.1.fetch_add(1, Ordering::SeqCst) != 0 {
        return;
    }
    world.observe_component_lifecycle::<MeshRenderer>(
        LifecycleEventKind::Add,
        RetainedLifecycleObserverV1::new(
            (state.0.clone(), state.1.clone()),
            create_callback_extra_v1,
            reentrant_mesh_callback_v1,
        ),
    );
}

#[test]
fn create_history_reserves_reentrant_lifecycle_producer_before_first_dispatch() {
    use zircon_runtime::core::framework::retained_memory::RetainedLifecycleObserverV1;
    let level = DefaultLevelManager::default().create_default_level();
    let calls = Arc::new(AtomicUsize::new(0));
    let name = "late mesh observer".repeat(128);
    level.with_world_mut(|world| {
        world.observe_component_lifecycle::<Name>(
            LifecycleEventKind::Add,
            RetainedLifecycleObserverV1::new(
                (name.clone(), calls.clone()),
                reentrant_create_extra_v1,
                reentrant_name_callback_v1,
            ),
        );
    });
    let engine = EditorTransactionEngine::new(create_history_context_v1(level.clone()));
    let mut scope = engine
        .begin("reentrant bounded create", HistoryContextId::Global)
        .unwrap();
    scope
        .push(EditorCommand::create_node(NodeKind::Cube))
        .unwrap();
    scope.commit().unwrap();
    let id = level.with_world(|world| {
        world
            .nodes()
            .iter()
            .find(|node| node.name == name)
            .unwrap()
            .id
    });
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "new B must run in this same Create"
    );
    engine.undo(HistoryContextId::Global).unwrap();
    engine.redo(HistoryContextId::Global).unwrap();
    assert_eq!(
        level.with_world(|world| world.get::<Name>(id).unwrap().0.clone()),
        name
    );
    assert_eq!(calls.load(Ordering::SeqCst), 4);

    use crate::core::editing::engine::{HistoryRetentionBudgetV1, HistoryRetentionDomainV1};
    let domain = HistoryRetentionDomainV1::try_new(HistoryRetentionBudgetV1 {
        record_bytes: 32 * 1024,
        history_bytes: 1024 * 1024,
        committed_bytes: 4 * 1024 * 1024,
        staging_bytes: 1024 * 1024,
    })
    .unwrap();
    let level = DefaultLevelManager::default().create_default_level();
    let engine = EditorTransactionEngine::with_retention_domain_v1(
        create_history_context_v1(level.clone()),
        128,
        domain.clone(),
    )
    .unwrap();
    let mut seed = engine
        .begin(
            "history before cumulative refusal",
            HistoryContextId::Global,
        )
        .unwrap();
    seed.push(EditorCommand::create_node(NodeKind::Cube))
        .unwrap();
    seed.commit().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let state = ("q".repeat(12 * 1024), calls.clone());
    level.with_world_mut(|world| {
        // A alone fits; its registered producer must also cover future B.
        let alone = world.observe_component_lifecycle::<Name>(
            LifecycleEventKind::Add,
            RetainedLifecycleObserverV1::new(
                state.clone(),
                create_callback_extra_v1,
                first_create_name_v1,
            ),
        );
        assert!(world
            .create_history_quote_v1(NodeKind::Cube, domain.model(), 32 * 1024)
            .is_ok());
        world.remove_observer(alone).unwrap();
        world.observe_component_lifecycle::<Name>(
            LifecycleEventKind::Add,
            RetainedLifecycleObserverV1::new(
                state,
                reentrant_create_extra_v1,
                reentrant_name_callback_v1,
            ),
        );
    });
    let before = level.with_world(|world| (world.nodes().len(), world.world_generation()));
    let charges = domain.snapshot().unwrap();
    let mut refused = engine
        .begin("cumulative A and B refusal", HistoryContextId::Global)
        .unwrap();
    assert!(matches!(
        refused.push(EditorCommand::create_node(NodeKind::Cube)),
        Err(EditCommandError::Retention(_))
    ));
    drop(refused);
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "neither A nor late B may run before reservation"
    );
    assert_eq!(
        level.with_world(|world| (world.nodes().len(), world.world_generation())),
        before
    );
    assert_eq!(domain.snapshot().unwrap(), charges);
    assert!(
        engine
            .history_status(HistoryContextId::Global)
            .unwrap()
            .can_undo
    );
    engine.undo(HistoryContextId::Global).unwrap();
    assert_eq!(level.with_world(|world| world.nodes().len()), before.0 - 1);
}
