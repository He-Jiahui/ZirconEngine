use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::DefaultLevelManager;

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
