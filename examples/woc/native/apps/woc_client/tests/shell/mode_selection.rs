use woc_client::{
    ModeMenuNavigation, ModeSelectionEffect, ModeSelectionError, ModeSelectionModel, ServerMode,
};

#[test]
fn full_entry_defaults_to_online_and_play_is_the_only_commit() {
    let mut model = ModeSelectionModel::new(true);

    assert_eq!(model.selected_mode(), ServerMode::Online);
    assert!(!model.menu_open());
    model
        .select_mode(ServerMode::Offline)
        .expect("choose offline");
    assert_eq!(model.selected_mode(), ServerMode::Offline);
    assert_eq!(model.play(), ModeSelectionEffect::OpenOfflinePicker);

    model
        .select_mode(ServerMode::Online)
        .expect("choose online");
    assert_eq!(model.play(), ModeSelectionEffect::OpenOnlineFlow);
}

#[test]
fn opening_the_menu_activates_the_selected_option_and_toggle_closes_it() {
    let mut model = ModeSelectionModel::new(true);
    model
        .select_mode(ServerMode::Offline)
        .expect("choose offline");

    model.open_menu();
    assert!(model.menu_open());
    assert_eq!(model.active_mode(), Some(ServerMode::Offline));
    model.toggle_menu();
    assert!(!model.menu_open());
    assert_eq!(model.active_mode(), None);
    assert_eq!(model.selected_mode(), ServerMode::Offline);
}

#[test]
fn keyboard_navigation_clamps_then_commits_the_active_option() {
    let mut model = ModeSelectionModel::new(true);
    model.open_menu();

    model
        .move_active(ModeMenuNavigation::Previous)
        .expect("Previous clamps at Online");
    assert_eq!(model.active_mode(), Some(ServerMode::Online));
    model
        .move_active(ModeMenuNavigation::Next)
        .expect("Next selects Offline");
    model
        .move_active(ModeMenuNavigation::Next)
        .expect("Next clamps at Offline");
    assert_eq!(model.active_mode(), Some(ServerMode::Offline));
    model
        .move_active(ModeMenuNavigation::First)
        .expect("Home selects Online");
    model
        .move_active(ModeMenuNavigation::Last)
        .expect("End selects Offline");

    model.commit_active().expect("commit Offline");
    assert_eq!(model.selected_mode(), ServerMode::Offline);
    assert!(!model.menu_open());
}

#[test]
fn escape_style_close_keeps_the_previous_selection() {
    let mut model = ModeSelectionModel::new(true);
    model.open_menu();
    model
        .move_active(ModeMenuNavigation::Next)
        .expect("highlight Offline");
    model.close_menu();

    assert_eq!(model.selected_mode(), ServerMode::Online);
    assert_eq!(model.active_mode(), None);
    assert_eq!(
        model
            .commit_active()
            .expect_err("closed menu cannot commit"),
        ModeSelectionError::MenuClosed
    );
}

#[test]
fn online_only_entry_never_exposes_or_commits_offline() {
    let mut model = ModeSelectionModel::new(false);
    assert!(!model.offline_available());
    assert_eq!(
        model
            .select_mode(ServerMode::Offline)
            .expect_err("play entry is online-only"),
        ModeSelectionError::OfflineUnavailable
    );

    model.open_menu();
    model
        .move_active(ModeMenuNavigation::Next)
        .expect("single option clamps");
    assert_eq!(model.active_mode(), Some(ServerMode::Online));
    assert_eq!(model.play(), ModeSelectionEffect::OpenOnlineFlow);
}
