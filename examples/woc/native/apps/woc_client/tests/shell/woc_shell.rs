use woc_client::{
    CharacterSortMode, OfflinePlayerClass, OnlineEntryState, OnlineShellEffect, ServerMode,
    WocShellController, WocShellEffect, WocShellScreen, OFFLINE_SESSION_LAUNCH_VERSION,
    OFFLINE_WORLD_SEED,
};

#[test]
fn online_default_requests_a_host_session_probe_before_selecting_a_subscreen() {
    let mut shell = WocShellController::new(true, CharacterSortMode::Level);

    assert_eq!(shell.screen(), WocShellScreen::ModeSelection);
    assert_eq!(shell.mode().selected_mode(), ServerMode::Online);
    assert!(matches!(
        shell.play().expect("online Play"),
        Some(WocShellEffect::ProbeOnlineSession)
    ));
    assert_eq!(shell.screen(), WocShellScreen::ModeSelection);
}

#[test]
fn host_session_resolution_routes_online_play_to_login_or_realm_loading() {
    let mut shell = WocShellController::new(true, CharacterSortMode::Level);
    shell.play().expect("online Play");

    assert!(shell
        .resolve_online_entry(OnlineEntryState::AuthenticationRequired)
        .expect("unauthenticated host result")
        .is_none());
    assert_eq!(shell.screen(), WocShellScreen::Authentication);

    shell
        .back_from_authentication()
        .expect("authentication Back");
    assert_eq!(shell.screen(), WocShellScreen::ModeSelection);

    shell.play().expect("online Play again");
    match shell
        .resolve_online_entry(OnlineEntryState::Authenticated)
        .expect("restored host result")
    {
        Some(WocShellEffect::Online(OnlineShellEffect::LoadRealmDirectory)) => {}
        _ => panic!("restored session must load realms through the online shell"),
    }
    assert_eq!(shell.screen(), WocShellScreen::RealmDirectory);
}

#[test]
fn offline_selection_opens_only_the_fresh_session_picker() {
    let mut shell = WocShellController::new(true, CharacterSortMode::Level);
    shell
        .select_mode(ServerMode::Offline)
        .expect("choose Offline");

    assert!(shell.play().expect("offline Play").is_none());
    assert_eq!(shell.screen(), WocShellScreen::OfflinePicker);
    assert_eq!(shell.offline().draft().raw_name(), "");
}

#[test]
fn offline_picker_prepares_then_starts_one_fresh_session_through_loading() {
    let mut shell = WocShellController::new(true, CharacterSortMode::Level);
    shell
        .select_mode(ServerMode::Offline)
        .expect("choose Offline");
    shell.play().expect("offline Play");
    shell
        .set_offline_class(OfflinePlayerClass::Mage)
        .expect("offline class");
    shell.set_offline_name("Vale").expect("offline name");

    match shell
        .submit_offline_picker()
        .expect("offline picker submit")
    {
        WocShellEffect::PrepareOfflineSession { launch } => {
            assert_eq!(launch.player_class, OfflinePlayerClass::Mage);
            assert_eq!(launch.player_name, "Vale");
            assert_eq!(launch.schema_version, OFFLINE_SESSION_LAUNCH_VERSION);
            assert_eq!(launch.world_seed, OFFLINE_WORLD_SEED);
        }
        _ => panic!("picker submit must only prepare a fresh session"),
    }
    assert_eq!(shell.screen(), WocShellScreen::Welcome);

    assert!(matches!(
        shell.continue_offline_welcome().expect("Welcome Continue"),
        WocShellEffect::StartOfflineWorld { .. }
    ));
    assert_eq!(shell.screen(), WocShellScreen::Loading);

    assert!(matches!(
        shell.finish_offline_loading().expect("world ready"),
        WocShellEffect::EnterOfflineWorld { .. }
    ));
    assert_eq!(shell.screen(), WocShellScreen::InWorld);
}

#[test]
fn offline_picker_back_reactivates_the_mode_selector() {
    let mut shell = WocShellController::new(true, CharacterSortMode::Level);
    shell
        .select_mode(ServerMode::Offline)
        .expect("choose Offline");
    shell.play().expect("offline Play");

    shell
        .back_from_offline_picker()
        .expect("offline picker Back");
    assert_eq!(shell.screen(), WocShellScreen::ModeSelection);
}

#[test]
fn realm_back_returns_the_root_shell_to_mode_selection() {
    let mut shell = WocShellController::new(true, CharacterSortMode::Level);
    shell.play().expect("online Play");
    shell
        .resolve_online_entry(OnlineEntryState::Authenticated)
        .expect("restored host result");

    assert!(matches!(
        shell.back_from_realms().expect("realm Back"),
        Some(WocShellEffect::NavigateToModeSelection)
    ));
    assert_eq!(shell.screen(), WocShellScreen::ModeSelection);
}

#[test]
fn online_only_host_rejects_offline_before_the_picker_can_open() {
    let mut shell = WocShellController::new(false, CharacterSortMode::Level);

    assert!(shell.select_mode(ServerMode::Offline).is_err());
    assert_eq!(shell.screen(), WocShellScreen::ModeSelection);
}
