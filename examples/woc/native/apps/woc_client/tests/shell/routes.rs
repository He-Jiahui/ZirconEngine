use woc_client::{
    parse_shell_route, AuthFlowEffect, AuthRoute, AuthScreen, CharacterAppearanceRig,
    CharacterCreateRoute, CharacterRosterEntry, CharacterRoute, CharacterSortMode, ModeRoute,
    OfflinePlayerClass, OfflineRoute, OnlineCharacterEffect, OnlineEntryState, OnlineShellEffect,
    RealmDefinition, RealmRoute, RealmType, ServerMode, ShellHostEffect, ShellRoute,
    ShellRouteDispatchError, ShellRouteEffect, ShellRouteError, WelcomeRoute, WocShellController,
    WocShellEffect, WocShellScreen, OFFLINE_WORLD_SEED,
};

fn realm(name: &str) -> RealmDefinition {
    RealmDefinition {
        name: name.into(),
        base_url: String::new(),
        realm_type: RealmType::Normal,
        character_count: 1,
    }
}

fn character(name: &str) -> CharacterRosterEntry {
    CharacterRosterEntry {
        id: 7,
        name: name.into(),
        class_id: "mage".into(),
        level: 12,
        skin_variant: 0,
        appearance_rig: CharacterAppearanceRig::Class,
        mainhand_item_id: None,
        offhand_item_id: None,
        online: false,
        force_rename: false,
        last_played_epoch_ms: Some(1),
        playtime_seconds: 5,
    }
}

#[test]
fn parses_every_static_auth_route() {
    let routes = [
        ("woc.shell.auth.set_username", AuthRoute::SetUsername),
        ("woc.shell.auth.set_password", AuthRoute::SetPassword),
        ("woc.shell.auth.set_email", AuthRoute::SetEmail),
        ("woc.shell.auth.set_two_factor", AuthRoute::SetSecondFactor),
        ("woc.shell.auth.submit", AuthRoute::Submit),
        ("woc.shell.auth.back", AuthRoute::Back),
        ("woc.shell.auth.toggle_mode", AuthRoute::ToggleMode),
        (
            "woc.shell.auth.open_forgot",
            AuthRoute::OpenPasswordResetRequest,
        ),
        (
            "woc.shell.auth.forgot.set_username",
            AuthRoute::SetForgotUsername,
        ),
        ("woc.shell.auth.forgot.submit", AuthRoute::SubmitForgot),
        ("woc.shell.auth.forgot.back", AuthRoute::BackFromForgot),
        (
            "woc.shell.auth.reset.set_password",
            AuthRoute::SetResetPassword,
        ),
        (
            "woc.shell.auth.reset.set_confirmation",
            AuthRoute::SetResetConfirmation,
        ),
        ("woc.shell.auth.reset.submit", AuthRoute::SubmitReset),
        ("woc.shell.auth.reset.back", AuthRoute::BackFromReset),
    ];

    for (route, expected) in routes {
        assert_eq!(
            parse_shell_route(route),
            Ok(ShellRoute::Auth(expected)),
            "{route} must retain its auth intent"
        );
    }
}

#[test]
fn parses_mode_and_offline_picker_routes() {
    let routes = [
        (
            "woc.shell.mode.toggle_menu",
            ShellRoute::Mode(ModeRoute::ToggleMenu),
        ),
        (
            "woc.shell.mode.select.online",
            ShellRoute::Mode(ModeRoute::Select(ServerMode::Online)),
        ),
        (
            "woc.shell.mode.select.offline",
            ShellRoute::Mode(ModeRoute::Select(ServerMode::Offline)),
        ),
        ("woc.shell.mode.play", ShellRoute::Mode(ModeRoute::Play)),
        (
            "woc.shell.mode.copy_contract",
            ShellRoute::Mode(ModeRoute::CopyContractAddress),
        ),
        (
            "woc.shell.offline.back",
            ShellRoute::Offline(OfflineRoute::Back),
        ),
        (
            "woc.shell.offline.enter_world",
            ShellRoute::Offline(OfflineRoute::Submit),
        ),
        (
            "woc.shell.offline.set_name",
            ShellRoute::Offline(OfflineRoute::SetName),
        ),
    ];
    for (route, expected) in routes {
        assert_eq!(parse_shell_route(route), Ok(expected));
    }

    for player_class in OfflinePlayerClass::ALL {
        let route = format!("woc.shell.offline.select_class.{}", player_class.as_str());
        assert_eq!(
            parse_shell_route(&route),
            Ok(ShellRoute::Offline(OfflineRoute::SelectClass(player_class)))
        );
    }
    for skin_variant in 0..4 {
        let route = format!("woc.shell.offline.select_skin.{skin_variant}");
        assert_eq!(
            parse_shell_route(&route),
            Ok(ShellRoute::Offline(OfflineRoute::SelectSkin(skin_variant)))
        );
    }
}

#[test]
fn parses_static_character_roster_routes() {
    let routes = [
        (
            "woc.shell.characters.change_realm",
            CharacterRoute::ChangeRealm,
        ),
        (
            "woc.shell.characters.toggle_sort",
            CharacterRoute::ToggleSort,
        ),
        (
            "woc.shell.characters.sort.level",
            CharacterRoute::SetSort(CharacterSortMode::Level),
        ),
        (
            "woc.shell.characters.sort.name",
            CharacterRoute::SetSort(CharacterSortMode::Name),
        ),
        (
            "woc.shell.characters.sort.recent",
            CharacterRoute::SetSort(CharacterSortMode::Recent),
        ),
        (
            "woc.shell.characters.sort.playtime",
            CharacterRoute::SetSort(CharacterSortMode::Playtime),
        ),
        ("woc.shell.characters.back", CharacterRoute::Back),
        ("woc.shell.characters.new", CharacterRoute::OpenCreate),
        ("woc.shell.characters.primary", CharacterRoute::Primary),
        (
            "woc.shell.characters.takeover.cancel",
            CharacterRoute::CancelTakeOver,
        ),
        (
            "woc.shell.characters.takeover.confirm",
            CharacterRoute::ConfirmTakeOver,
        ),
        (
            "woc.shell.characters.delete.set_confirmation",
            CharacterRoute::SetDeleteConfirmation,
        ),
        (
            "woc.shell.characters.delete.cancel",
            CharacterRoute::CancelDelete,
        ),
        (
            "woc.shell.characters.delete.submit",
            CharacterRoute::SubmitDelete,
        ),
    ];
    for (route, expected) in routes {
        assert_eq!(
            parse_shell_route(route),
            Ok(ShellRoute::Characters(expected))
        );
    }
}

#[test]
fn parses_static_character_creation_routes() {
    let routes = [
        (
            "woc.shell.characters.create.set_name",
            CharacterCreateRoute::SetName,
        ),
        (
            "woc.shell.characters.create.back",
            CharacterCreateRoute::Back,
        ),
        (
            "woc.shell.characters.create.submit",
            CharacterCreateRoute::Submit,
        ),
    ];
    for (route, expected) in routes {
        assert_eq!(
            parse_shell_route(route),
            Ok(ShellRoute::Characters(CharacterRoute::Create(expected)))
        );
    }

    for player_class in OfflinePlayerClass::ALL {
        let route = format!(
            "woc.shell.characters.create.select_class.{}",
            player_class.as_str()
        );
        assert_eq!(
            parse_shell_route(&route),
            Ok(ShellRoute::Characters(CharacterRoute::Create(
                CharacterCreateRoute::SelectClass(player_class)
            )))
        );
    }
    for skin_variant in 0..4 {
        let route = format!("woc.shell.characters.create.select_skin.{skin_variant}");
        assert_eq!(
            parse_shell_route(&route),
            Ok(ShellRoute::Characters(CharacterRoute::Create(
                CharacterCreateRoute::SelectSkin(skin_variant)
            )))
        );
    }
}

#[test]
fn parses_static_realm_and_welcome_routes() {
    let routes = [
        (
            "woc.shell.realms.back",
            ShellRoute::Realms(RealmRoute::Back),
        ),
        (
            "woc.shell.welcome.continue",
            ShellRoute::Welcome(WelcomeRoute::Continue),
        ),
        (
            "woc.shell.welcome.join_discord",
            ShellRoute::Welcome(WelcomeRoute::JoinDiscord),
        ),
        (
            "woc.shell.welcome.open_armory",
            ShellRoute::Welcome(WelcomeRoute::OpenArmory),
        ),
    ];
    for (route, expected) in routes {
        assert_eq!(parse_shell_route(route), Ok(expected));
    }
}

#[test]
fn rejects_unknown_and_host_dynamic_routes_without_inventing_payloads() {
    for route in [
        "woc.shell.realms.row",
        "woc.shell.realms.row.eastbrook",
        "woc.shell.offline.select_skin.4",
        "woc.shell.characters.create.select_class.bard",
        "woc.shell.welcome.delete_everything",
    ] {
        assert_eq!(
            parse_shell_route(route),
            Err(ShellRouteError::UnknownRoute(route.into()))
        );
    }
}

#[test]
fn dispatches_auth_routes_as_opaque_host_effects() {
    let mut shell = WocShellController::new(true, CharacterSortMode::Level);
    assert_eq!(
        shell.play().expect("online Play route source"),
        Some(WocShellEffect::ProbeOnlineSession)
    );
    shell
        .resolve_online_entry(OnlineEntryState::AuthenticationRequired)
        .expect("host requires authentication");

    shell
        .dispatch_shell_route("woc.shell.auth.open_forgot", None)
        .expect("forgot password route");
    assert_eq!(
        shell.online().auth().screen(),
        AuthScreen::PasswordResetRequest
    );
    shell
        .dispatch_shell_route("woc.shell.auth.forgot.back", None)
        .expect("forgot password Back");
    assert_eq!(shell.online().auth().screen(), AuthScreen::SignIn);

    assert!(shell
        .dispatch_shell_route("woc.shell.auth.set_username", Some(" Vale "))
        .expect("username route")
        .is_none());
    assert!(shell
        .dispatch_shell_route("woc.shell.auth.set_password", Some("secret"))
        .expect("password route")
        .is_none());

    match shell
        .dispatch_shell_route("woc.shell.auth.submit", None)
        .expect("submit route")
    {
        Some(ShellRouteEffect::Woc(WocShellEffect::Online(OnlineShellEffect::Authentication(
            AuthFlowEffect::Login { username, .. },
        )))) => assert_eq!(username, "Vale"),
        _ => panic!("auth submit must remain an opaque host request"),
    }
}

#[test]
fn dispatches_offline_routes_through_prepare_welcome_and_start() {
    let mut shell = WocShellController::new(true, CharacterSortMode::Level);
    assert!(shell
        .dispatch_shell_route("woc.shell.mode.select.offline", None)
        .expect("offline selection")
        .is_none());
    assert!(shell
        .dispatch_shell_route("woc.shell.mode.play", None)
        .expect("offline Play")
        .is_none());
    assert_eq!(shell.screen(), WocShellScreen::OfflinePicker);

    shell
        .dispatch_shell_route("woc.shell.offline.set_name", Some("Vale"))
        .expect("offline name");
    shell
        .dispatch_shell_route("woc.shell.offline.select_class.mage", None)
        .expect("offline class");
    shell
        .dispatch_shell_route("woc.shell.offline.select_skin.2", None)
        .expect("offline skin");
    match shell
        .dispatch_shell_route("woc.shell.offline.enter_world", None)
        .expect("offline submit")
    {
        Some(ShellRouteEffect::Woc(WocShellEffect::PrepareOfflineSession { launch })) => {
            assert_eq!(launch.player_class, OfflinePlayerClass::Mage);
            assert_eq!(launch.player_name, "Vale");
            assert_eq!(launch.skin_variant, 2);
            assert_eq!(launch.world_seed, OFFLINE_WORLD_SEED);
        }
        _ => panic!("offline submit must prepare but not start the world"),
    }
    assert_eq!(shell.screen(), WocShellScreen::Welcome);

    assert!(matches!(
        shell
            .dispatch_shell_route("woc.shell.welcome.continue", None)
            .expect("welcome continue"),
        Some(ShellRouteEffect::Woc(
            WocShellEffect::StartOfflineWorld { .. }
        ))
    ));
    assert_eq!(shell.screen(), WocShellScreen::Loading);
}

#[test]
fn exposes_host_only_actions_and_rejects_missing_or_wrong_screen_input() {
    let mut shell = WocShellController::new(true, CharacterSortMode::Level);
    assert_eq!(
        shell
            .dispatch_shell_route("woc.shell.mode.copy_contract", None)
            .expect("copy contract host effect"),
        Some(ShellRouteEffect::Host(ShellHostEffect::CopyContractAddress))
    );
    assert_eq!(
        shell
            .dispatch_shell_route("woc.shell.welcome.join_discord", None)
            .expect_err("Welcome link must stay screen-gated"),
        ShellRouteDispatchError::InvalidScreen {
            route: ShellRoute::Welcome(WelcomeRoute::JoinDiscord),
            actual: WocShellScreen::ModeSelection,
        }
    );

    shell.play().expect("online Play");
    shell
        .resolve_online_entry(OnlineEntryState::AuthenticationRequired)
        .expect("authentication entry");
    assert_eq!(
        shell
            .dispatch_shell_route("woc.shell.auth.set_username", None)
            .expect_err("input route requires host text"),
        ShellRouteDispatchError::MissingTextValue {
            route: ShellRoute::Auth(AuthRoute::SetUsername),
        }
    );
    assert!(shell.online().auth().username().is_empty());
}

#[test]
fn root_host_callbacks_enable_character_routes_without_bypassing_the_online_flow() {
    let mut shell = WocShellController::new(true, CharacterSortMode::Level);
    shell.play().expect("online Play");
    shell
        .resolve_online_entry(OnlineEntryState::Authenticated)
        .expect("restored session");
    assert!(shell
        .replace_realm_directory(vec![realm("Eastbrook")], None)
        .expect("realm callback")
        .is_none());
    assert!(matches!(
        shell.select_realm("Eastbrook").expect("realm selection"),
        WocShellEffect::Online(OnlineShellEffect::SelectRealmAndLoadCharacters { .. })
    ));
    shell
        .replace_characters(vec![character("Vale")])
        .expect("character callback");

    match shell
        .dispatch_shell_route("woc.shell.characters.sort.name", None)
        .expect("sort route")
    {
        Some(ShellRouteEffect::Woc(WocShellEffect::Online(OnlineShellEffect::Character(
            OnlineCharacterEffect::PersistSortAndRefresh { mode },
        )))) => assert_eq!(mode, CharacterSortMode::Name),
        _ => panic!("sort must stay an online host refresh request"),
    }

    shell
        .dispatch_shell_route("woc.shell.characters.new", None)
        .expect("open create");
    shell
        .dispatch_shell_route("woc.shell.characters.create.set_name", Some("Nova"))
        .expect("create name");
    shell
        .dispatch_shell_route("woc.shell.characters.create.select_class.mage", None)
        .expect("create class");
    match shell
        .dispatch_shell_route("woc.shell.characters.create.submit", None)
        .expect("create submit")
    {
        Some(ShellRouteEffect::Woc(WocShellEffect::Online(OnlineShellEffect::Character(
            OnlineCharacterEffect::Create {
                name,
                player_class,
                skin_variant,
            },
        )))) => {
            assert_eq!(name, "Nova");
            assert_eq!(player_class, OfflinePlayerClass::Mage);
            assert_eq!(skin_variant, 0);
        }
        _ => panic!("create must remain an online host request"),
    }
}
