use super::*;

#[test]
fn command_eval_projection_uses_active_mode_and_world_selection() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    selection.extend(WorldDomain::Edit, [11, 12]);
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let base = Box::new(RecordingMode::new(
        "scene.select",
        InputOutcome::PassThrough,
        events.clone(),
    ));
    let mut stack = SceneModeStack::new(SceneModeActivation::Select, base, &mut ctx).unwrap();

    let edit_projection =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(WhenClause::SceneModeActive(SceneModeId::new("scene.select")).eval(&edit_projection));
    assert!(WhenClause::SelectionNonEmpty.eval(&edit_projection));

    stack
        .push_overlay(
            custom_activation("scene.navigation"),
            Box::new(RecordingMode::new(
                "scene.navigation",
                InputOutcome::Consumed,
                events,
            )),
            &mut ctx,
        )
        .unwrap();
    ctx.selection_mut()
        .activate_play_domain(PlayInstanceId::for_test(1));
    ctx.selection_mut().clear_active();
    let empty_play_projection =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(
        WhenClause::SceneModeActive(SceneModeId::new("scene.navigation"))
            .eval(&empty_play_projection)
    );
    assert!(!WhenClause::SelectionNonEmpty.eval(&empty_play_projection));

    ctx.selection_mut().select_only(play_domain(), 99);
    let selected_play_projection =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(WhenClause::SelectionNonEmpty.eval(&selected_play_projection));
}

#[test]
fn command_eval_generation_tracks_selection_identity_and_mode_topology() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut selection = SelectionModel::default();
    selection.activate_play_domain(PlayInstanceId::for_test(1));
    selection.set_active_domain(WorldDomain::Edit);
    selection.select_only(WorldDomain::Edit, 11);
    selection.select_only(play_domain(), 29);
    let settings = SceneViewportSettings::default();
    let mut ctx = SceneModeCtx::new(&mut selection, &settings);
    let base = Box::new(RecordingMode::new(
        "scene.select",
        InputOutcome::PassThrough,
        events.clone(),
    ));
    let mut stack = SceneModeStack::new(SceneModeActivation::Select, base, &mut ctx).unwrap();
    let snapshot = CommandEvalSnapshotHandle::default();

    let initial = stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(snapshot.replace(initial));
    assert_eq!(snapshot.generation(), 1);

    // Edit and Play have the same count and per-domain generation, so the domain itself must
    // remain part of the command snapshot identity.
    assert!(ctx.selection_mut().set_active_domain(play_domain()));
    let same_count_play =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(snapshot.replace(same_count_play));
    assert_eq!(snapshot.generation(), 2);

    assert!(ctx.selection_mut().set_active_domain(WorldDomain::Edit));
    let same_count_edit =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(snapshot.replace(same_count_edit));
    assert_eq!(snapshot.generation(), 3);

    // A Play mutation must not invalidate the active Edit command snapshot.
    ctx.selection_mut().select_only(play_domain(), 30);
    let inactive_play_mutation =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(!snapshot.replace(inactive_play_mutation));
    assert_eq!(snapshot.generation(), 3);

    assert!(ctx.selection_mut().set_active_domain(play_domain()));
    let changed_play_domain =
        stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(snapshot.replace(changed_play_domain));
    assert_eq!(snapshot.generation(), 4);

    let equivalent = stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(!snapshot.replace(equivalent));
    assert_eq!(snapshot.generation(), 4);

    stack
        .push_overlay(
            custom_activation("scene.navigation"),
            Box::new(RecordingMode::new(
                "scene.navigation",
                InputOutcome::Consumed,
                events,
            )),
            &mut ctx,
        )
        .unwrap();
    let pushed = stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(snapshot.replace(pushed));
    assert_eq!(snapshot.generation(), 5);

    assert_eq!(
        stack.pop(&mut ctx),
        Some(SceneModeId::new("scene.navigation"))
    );
    let popped = stack.project_command_eval_ctx(CommandEvalCtx::interactive(), ctx.selection());
    assert!(snapshot.replace(popped));
    assert_eq!(snapshot.generation(), 6);
}
