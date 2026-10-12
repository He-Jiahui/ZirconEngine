use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use zircon_runtime::scene::components::{Name, NodeKind};
use zircon_runtime::scene::ecs::LifecycleEventKind;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::world_sync::LifecycleTargetChange;

use crate::core::editing::command::EditorCommand;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{
    CommandEffect, EditCommand, EditCommandError, SelectionSnapshot,
};
use crate::core::editing::selection::SceneSelection;
use crate::core::gateway::EditorRuntimeGatewayHandle;

#[test]
fn create_apply_is_applied_when_selection_generation_is_exhausted() {
    let level = DefaultLevelManager::default().create_default_level();
    let initial_node_count = level.with_world(|scene| scene.nodes().len());
    let gateway = EditorRuntimeGatewayHandle::detached();
    let mut context = CoreEditContext::new(gateway);
    context
        .bind_scene(level.clone(), SceneSelection::new(Vec::new(), None))
        .expect("bind default level to editor context");
    context
        .restore_selection_snapshot(&SelectionSnapshot::scene(
            u64::MAX,
            SceneSelection::new(Vec::new(), None),
        ))
        .expect("restore maximal selection generation");
    let mut command = EditorCommand::create_node(NodeKind::Cube);

    let error = command
        .apply(&mut context)
        .expect_err("selection failure after create must surface");
    assert_eq!(error.effect, CommandEffect::Applied);
    assert!(matches!(
        error.source,
        EditCommandError::SelectionGenerationExhausted
    ));
    assert_eq!(
        level.with_world(|scene| scene.nodes().len()),
        initial_node_count + 1,
        "the scene mutation must be visible before transaction recovery"
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
fn create_redo_is_applied_when_selection_generation_is_exhausted() {
    let level = DefaultLevelManager::default().create_default_level();
    let initial_node_count = level.with_world(|scene| scene.nodes().len());
    let gateway = EditorRuntimeGatewayHandle::detached();
    let mut context = CoreEditContext::new(gateway);
    context
        .bind_scene(level.clone(), SceneSelection::new(Vec::new(), None))
        .expect("bind default level to editor context");
    let mut command = EditorCommand::create_node(NodeKind::Cube);
    command
        .apply(&mut context)
        .expect("initial create must capture the retained record");
    command
        .revert(&mut context)
        .expect("initial create must be reversible before redo");
    assert_eq!(
        level.with_world(|scene| scene.nodes().len()),
        initial_node_count
    );
    context
        .restore_selection_snapshot(&SelectionSnapshot::scene(
            u64::MAX,
            SceneSelection::new(Vec::new(), None),
        ))
        .expect("restore maximal selection generation");

    let error = command
        .apply(&mut context)
        .expect_err("selection failure after create redo must surface");
    assert_eq!(error.effect, CommandEffect::Applied);
    assert!(matches!(
        error.source,
        EditCommandError::SelectionGenerationExhausted
    ));
    assert_eq!(
        level.with_world(|scene| scene.nodes().len()),
        initial_node_count + 1,
        "the retained record must be reinserted before transaction recovery"
    );

    command
        .revert(&mut context)
        .expect("the retained create record must remain reversible after redo failure");
    assert_eq!(
        level.with_world(|scene| scene.nodes().len()),
        initial_node_count
    );
}

#[test]
fn create_lifecycle_cancellation_precedes_selection_generation_failure() {
    for cause in [
        LifecycleTargetChange::World,
        LifecycleTargetChange::Schedule,
    ] {
        let level = DefaultLevelManager::default().create_default_level();
        let gateway = EditorRuntimeGatewayHandle::detached();
        let mut context = CoreEditContext::new(gateway);
        context
            .bind_scene(level.clone(), SceneSelection::new(Vec::new(), None))
            .unwrap();
        context
            .restore_selection_snapshot(&SelectionSnapshot::scene(
                u64::MAX,
                SceneSelection::new(Vec::new(), None),
            ))
            .unwrap();
        let before = context.selection_snapshot();
        let replacement = DefaultLevelManager::default()
            .create_default_level()
            .with_world_mut(std::mem::take);
        let replacement = Mutex::new(Some(replacement));
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = calls.clone();
        level.with_world_mut(|scene| {
            scene.observe_component_lifecycle::<Name>(LifecycleEventKind::Add, move |world, _| {
                observed.fetch_add(1, Ordering::SeqCst);
                match cause {
                    LifecycleTargetChange::World => {
                        *world = replacement.lock().unwrap().take().unwrap()
                    }
                    LifecycleTargetChange::Schedule => *world.schedule_mut() = Default::default(),
                }
            });
        });
        let mut command = EditorCommand::create_node(NodeKind::Cube);

        let error = command.apply(&mut context).unwrap_err();
        assert_eq!(error.effect, CommandEffect::Interrupted);
        let EditCommandError::LifecycleInterrupted {
            source,
            post_callback_errors,
        } = error.source
        else {
            panic!("create cancellation must remain persistent");
        };
        let EditCommandError::SceneMutation { source, .. } = *source else {
            panic!("create cancellation must retain the typed producer error");
        };
        assert_eq!(source.lifecycle_target_change(), Some(cause));
        assert!(post_callback_errors.is_empty());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(context.selection_snapshot(), before);
    }
}
