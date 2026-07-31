use woc_client::{
    AuthCompletion, AuthFlowEffect, AuthMode, CharacterRosterScreen, CharacterSortMode,
    OnlineEntryState, OnlineShellController, OnlineShellEffect, OnlineShellScreen, RealmDefinition,
    RealmType,
};

fn realm(name: &str, url: &str) -> RealmDefinition {
    RealmDefinition {
        name: name.to_string(),
        base_url: url.to_string(),
        realm_type: RealmType::Normal,
        character_count: 0,
    }
}

#[test]
fn online_entry_needs_auth_without_a_session_and_loads_realms_with_one() {
    let mut shell = OnlineShellController::new(CharacterSortMode::Level);

    assert!(shell
        .enter_online(OnlineEntryState::AuthenticationRequired)
        .expect("unauthenticated entry")
        .is_none());
    assert_eq!(shell.screen(), OnlineShellScreen::Authentication);

    match shell
        .enter_online(OnlineEntryState::Authenticated)
        .expect("restored session entry")
    {
        Some(OnlineShellEffect::LoadRealmDirectory) => {}
        _ => panic!("a restored session must load the realm directory"),
    }
    assert_eq!(shell.screen(), OnlineShellScreen::RealmDirectory);
}

#[test]
fn two_factor_challenge_stays_on_authentication_and_success_loads_realms() {
    let mut shell = OnlineShellController::new(CharacterSortMode::Level);
    shell
        .enter_online(OnlineEntryState::AuthenticationRequired)
        .expect("authentication entry");

    assert!(shell
        .complete_auth(AuthCompletion::TwoFactorRequired)
        .expect("two-factor result")
        .is_none());
    assert_eq!(shell.screen(), OnlineShellScreen::Authentication);
    assert!(shell.auth().two_factor_visible());

    match shell
        .complete_auth(AuthCompletion::Authenticated)
        .expect("authenticated result")
    {
        Some(OnlineShellEffect::LoadRealmDirectory) => {}
        _ => panic!("successful auth must load the realm directory"),
    }
    assert_eq!(shell.screen(), OnlineShellScreen::RealmDirectory);
}

#[test]
fn auth_submit_is_forwarded_without_adding_service_rules() {
    let mut shell = OnlineShellController::new(CharacterSortMode::Level);
    shell
        .enter_online(OnlineEntryState::AuthenticationRequired)
        .expect("authentication entry");
    shell.auth_mut().set_auth_mode(AuthMode::Login);
    shell
        .auth_mut()
        .set_username(" Vale ")
        .expect("username input");
    shell
        .auth_mut()
        .set_password("secret")
        .expect("password input");

    match shell.submit_auth().expect("host request") {
        OnlineShellEffect::Authentication(AuthFlowEffect::Login { username, .. }) => {
            assert_eq!(username, "Vale");
        }
        _ => panic!("login must remain an opaque host authentication request"),
    }
}

#[test]
fn remembered_or_selected_realm_enters_the_character_roster_and_requests_refresh() {
    let mut shell = OnlineShellController::new(CharacterSortMode::Level);
    shell
        .enter_online(OnlineEntryState::Authenticated)
        .expect("restored session entry");

    match shell
        .replace_realm_directory(
            vec![
                realm("Eastbrook", ""),
                realm("Ashenfall", "https://ashen.example"),
            ],
            Some("Ashenfall"),
        )
        .expect("remembered realm")
    {
        Some(OnlineShellEffect::SelectRealmAndLoadCharacters {
            realm_name,
            base_url,
        }) => {
            assert_eq!(realm_name, "Ashenfall");
            assert_eq!(base_url, "https://ashen.example");
        }
        _ => panic!("remembered realm must switch before refreshing characters"),
    }
    assert_eq!(shell.screen(), OnlineShellScreen::CharacterSelection);
}

#[test]
fn character_create_back_and_roster_back_follow_the_target_screen_order() {
    let mut shell = OnlineShellController::new(CharacterSortMode::Level);
    shell
        .enter_online(OnlineEntryState::Authenticated)
        .expect("restored session entry");
    shell
        .replace_realm_directory(vec![realm("Eastbrook", "")], None)
        .expect("realm directory");
    match shell.select_realm("Eastbrook").expect("select realm") {
        OnlineShellEffect::SelectRealmAndLoadCharacters { .. } => {}
        _ => panic!("realm selection must refresh its character roster"),
    }

    shell
        .open_character_create()
        .expect("open character create");
    assert_eq!(shell.screen(), OnlineShellScreen::CharacterCreation);
    assert_eq!(
        shell.characters().screen(),
        CharacterRosterScreen::CreateCharacter
    );

    assert!(shell.back_from_characters().expect("create Back").is_none());
    assert_eq!(shell.screen(), OnlineShellScreen::CharacterSelection);
    assert_eq!(
        shell.characters().screen(),
        CharacterRosterScreen::SelectCharacter
    );

    assert!(shell.back_from_characters().expect("roster Back").is_none());
    assert_eq!(shell.screen(), OnlineShellScreen::Authentication);
}

#[test]
fn realm_back_leaves_online_flow_for_the_mode_selector() {
    let mut shell = OnlineShellController::new(CharacterSortMode::Level);
    shell
        .enter_online(OnlineEntryState::Authenticated)
        .expect("restored session entry");

    match shell.back_from_realms().expect("realm Back") {
        Some(OnlineShellEffect::NavigateToModeSelection) => {}
        _ => panic!("realm Back must return to mode selection"),
    }
    assert_eq!(shell.screen(), OnlineShellScreen::ModeSelection);
}

#[test]
fn character_selection_can_return_to_the_existing_realm_directory() {
    let mut shell = OnlineShellController::new(CharacterSortMode::Level);
    shell
        .enter_online(OnlineEntryState::Authenticated)
        .expect("restored session entry");
    shell
        .replace_realm_directory(vec![realm("Eastbrook", "")], None)
        .expect("realm directory");
    shell.select_realm("Eastbrook").expect("select realm");
    assert_eq!(shell.screen(), OnlineShellScreen::CharacterSelection);

    shell.change_realm().expect("change realm");
    assert_eq!(shell.screen(), OnlineShellScreen::RealmDirectory);
}
