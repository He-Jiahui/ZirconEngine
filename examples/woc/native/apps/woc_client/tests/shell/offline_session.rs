use woc_client::{
    CharacterNameError, OfflinePlayerClass, OfflineSessionDraft, OfflineSessionError,
    OFFLINE_SESSION_LAUNCH_VERSION,
};
use woc_protocol::OfflineWeaponSkinAccount;

const LAUNCH_SEED: u32 = 20_061;

#[test]
fn offline_picker_cannot_synthesize_a_class_or_name() {
    let mut draft = OfflineSessionDraft::default();

    assert_eq!(draft.player_class(), None);
    assert_eq!(draft.skin_variant(), 0);
    assert_eq!(draft.raw_name(), "");
    assert_eq!(
        draft.launch().expect_err("a fresh session requires a name"),
        OfflineSessionError::CharacterName(CharacterNameError::Empty)
    );
    draft.set_raw_name("Mira");
    assert_eq!(
        draft
            .launch()
            .expect_err("a launch requires an explicitly selected class"),
        OfflineSessionError::MissingPlayerClass
    );
}

#[test]
fn class_catalog_and_ids_match_the_pinned_target_order() {
    let ids = OfflinePlayerClass::ALL
        .iter()
        .map(|class| class.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec![
            "warrior", "paladin", "hunter", "rogue", "priest", "shaman", "mage", "warlock",
            "druid",
        ]
    );

    for class in OfflinePlayerClass::ALL {
        assert_eq!(OfflinePlayerClass::parse(class.as_str()), Some(class));
    }
    assert_eq!(
        OfflinePlayerClass::ALL
            .into_iter()
            .map(OfflinePlayerClass::bootstrap_index)
            .collect::<Vec<_>>(),
        vec![0, 3, 4, 2, 5, 6, 1, 7, 8]
    );
    assert_eq!(OfflinePlayerClass::parse("Warrior"), None);
    assert_eq!(OfflinePlayerClass::parse("monk"), None);
}

#[test]
fn every_explicit_class_selection_resets_skin_to_variant_zero() {
    let mut draft = OfflineSessionDraft::default();
    draft.set_skin_variant(4);
    draft.set_player_class(OfflinePlayerClass::Warrior);
    assert_eq!(draft.player_class(), Some(OfflinePlayerClass::Warrior));
    assert_eq!(draft.skin_variant(), 0);

    draft.set_skin_variant(3);
    draft.set_player_class(OfflinePlayerClass::Mage);
    assert_eq!(draft.player_class(), Some(OfflinePlayerClass::Mage));
    assert_eq!(draft.skin_variant(), 0);
}

#[test]
fn launch_is_session_only_and_scopes_persisted_preferences_by_class_and_name() {
    let mut draft = OfflineSessionDraft::default();
    draft.set_player_class(OfflinePlayerClass::Shaman);
    draft.set_skin_variant(2);
    draft.set_raw_name("  Storm  Caller  ");

    let launch = draft.launch().expect("valid offline launch");
    assert_eq!(launch.schema_version, OFFLINE_SESSION_LAUNCH_VERSION);
    assert_eq!(launch.player_class, OfflinePlayerClass::Shaman);
    assert_eq!(launch.player_name, "Storm  Caller");
    assert_eq!(launch.skin_variant, 2);
    assert_eq!(launch.world_seed, LAUNCH_SEED);
    assert_eq!(launch.preference_scope(), "offline:shaman:Storm  Caller");
    let bootstrap = launch.bootstrap();
    assert_eq!(bootstrap.player_class, 6);
    assert_eq!(bootstrap.player_name, "Storm  Caller");
    assert_eq!(bootstrap.world_seed, LAUNCH_SEED);
    assert_eq!(bootstrap.skin_variant, 2);

    assert_eq!(draft.raw_name(), "  Storm  Caller  ");
    assert_eq!(draft.skin_variant(), 2);
}

#[test]
fn host_weapon_skin_account_reaches_the_first_tick_bootstrap() {
    let mut draft = OfflineSessionDraft::default();
    draft.set_player_class(OfflinePlayerClass::Warrior);
    draft.set_raw_name("Vale");
    let mut account = OfflineWeaponSkinAccount::default();
    account.owned[0] = true;
    account.loadout_codes[0] = 1;
    draft.set_weapon_skin_account(account.clone());

    let launch = draft.launch().expect("valid offline launch");
    assert_eq!(launch.weapon_skin_account, account);
    assert_eq!(launch.bootstrap().weapon_skin_account, account);
}

#[test]
fn invalid_launch_does_not_rewrite_the_picker_draft() {
    let mut draft = OfflineSessionDraft::default();
    draft.set_player_class(OfflinePlayerClass::Druid);
    draft.set_skin_variant(3);
    draft.set_raw_name("Map\tMaker");

    assert_eq!(
        draft
            .launch()
            .expect_err("tabs are rejected by the target rule"),
        OfflineSessionError::CharacterName(CharacterNameError::InvalidCharacter { index: 3 })
    );
    assert_eq!(draft.player_class(), Some(OfflinePlayerClass::Druid));
    assert_eq!(draft.skin_variant(), 3);
    assert_eq!(draft.raw_name(), "Map\tMaker");
}

#[test]
fn launch_rejects_a_skin_outside_the_selected_class_catalog() {
    let mut draft = OfflineSessionDraft::default();
    draft.set_player_class(OfflinePlayerClass::Paladin);
    draft.set_skin_variant(2);
    draft.set_raw_name("Aldren");

    assert_eq!(
        draft
            .launch()
            .expect_err("paladin exposes only skin indices zero and one"),
        OfflineSessionError::InvalidSkinVariant {
            player_class: OfflinePlayerClass::Paladin,
            skin_variant: 2,
            skin_count: 2,
        }
    );
    assert_eq!(draft.player_class(), Some(OfflinePlayerClass::Paladin));
    assert_eq!(draft.skin_variant(), 2);
    assert_eq!(draft.raw_name(), "Aldren");
}

#[test]
fn draft_preview_uses_the_selected_class_skin_without_requiring_a_name() {
    let mut draft = OfflineSessionDraft::default();
    assert_eq!(
        draft
            .preview()
            .expect_err("a preview requires an explicit class"),
        OfflineSessionError::MissingPlayerClass
    );

    draft.set_player_class(OfflinePlayerClass::Paladin);
    draft.set_skin_variant(1);
    let paladin = draft.preview().expect("paladin alternate preview");
    assert_eq!(
        paladin.model_asset,
        "assets/m8/models/chars/players/paladin.glb"
    );
    assert_eq!(
        paladin.skin_thumbnail_asset,
        "assets/m8/textures/skins/paladin/alt_a.png"
    );
    assert_eq!(
        paladin.skin_material_asset,
        Some("assets/m8/textures/skins/paladin/alt_a.png")
    );

    draft.set_skin_variant(2);
    assert_eq!(
        draft.preview().expect_err("skin two is outside paladin"),
        OfflineSessionError::InvalidSkinVariant {
            player_class: OfflinePlayerClass::Paladin,
            skin_variant: 2,
            skin_count: 2,
        }
    );
}
