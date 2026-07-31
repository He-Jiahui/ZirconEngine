use woc_client::{
    CharacterAppearanceRig, CharacterEntryBlock, CharacterRosterEntry, CharacterRosterScreen,
    CharacterSortMode, OfflinePlayerClass, OnlineCharacterEffect, OnlineCharacterFlow,
    OnlineCharacterFlowError,
};

fn character(id: u64, name: &str) -> CharacterRosterEntry {
    CharacterRosterEntry {
        id,
        name: name.to_string(),
        class_id: "warrior".to_string(),
        level: 10,
        skin_variant: 0,
        appearance_rig: CharacterAppearanceRig::Class,
        mainhand_item_id: None,
        offhand_item_id: None,
        online: false,
        force_rename: false,
        last_played_epoch_ms: None,
        playtime_seconds: 0,
    }
}

#[test]
fn roster_refresh_routes_empty_accounts_to_creation_and_selects_the_first_sorted_row() {
    let mut flow = OnlineCharacterFlow::new(CharacterSortMode::Level);

    flow.replace_roster(Vec::new()).expect("empty roster");
    assert_eq!(flow.screen(), CharacterRosterScreen::CreateCharacter);

    let mut low = character(1, "Mira");
    low.level = 4;
    let mut high = character(2, "Aldwin");
    high.level = 20;
    flow.replace_roster(vec![low, high]).expect("valid roster");

    assert_eq!(flow.screen(), CharacterRosterScreen::SelectCharacter);
    assert_eq!(flow.roster().selected_id(), Some(2));
}

#[test]
fn create_back_returns_to_roster_while_roster_back_returns_to_login() {
    let mut flow = OnlineCharacterFlow::new(CharacterSortMode::Level);
    flow.replace_roster(vec![character(1, "Mira")])
        .expect("valid roster");
    flow.open_create().expect("new character");
    flow.set_create_class(OfflinePlayerClass::Mage)
        .expect("select Mage");
    flow.set_create_skin(3).expect("select Mage skin");
    flow.set_create_name("Medivh").expect("type name");

    assert_eq!(flow.back().expect("create Back"), None);
    assert_eq!(flow.screen(), CharacterRosterScreen::SelectCharacter);
    flow.open_create().expect("reopen creation");
    assert_eq!(flow.create_draft().raw_name(), "Medivh");
    assert_eq!(flow.create_draft().player_class(), OfflinePlayerClass::Mage);
    assert_eq!(flow.create_draft().skin_variant(), 3);
    assert_eq!(flow.back().expect("second create Back"), None);
    assert_eq!(
        flow.back().expect("roster Back"),
        Some(OnlineCharacterEffect::NavigateToLogin)
    );
}

#[test]
fn four_sort_routes_persist_the_choice_until_authority_refresh_selects_the_first_row() {
    let mut flow = OnlineCharacterFlow::new(CharacterSortMode::Level);
    flow.replace_roster(vec![character(1, "Zara"), character(2, "Aldwin")])
        .expect("valid roster");
    flow.select(1).expect("select Zara");

    assert_eq!(
        flow.set_sort_mode(CharacterSortMode::Name)
            .expect("sort roster"),
        OnlineCharacterEffect::PersistSortAndRefresh {
            mode: CharacterSortMode::Name,
        }
    );
    assert_eq!(flow.roster().selected_id(), Some(1));
    assert_eq!(flow.roster().entries()[0].id, 2);

    flow.replace_roster(vec![character(1, "Zara"), character(2, "Aldwin")])
        .expect("authority refresh");
    assert_eq!(
        flow.roster().selected_id(),
        Some(2),
        "target refresh selects the first sorted row"
    );
}

#[test]
fn primary_action_enters_offline_rows_and_requires_explicit_takeover_confirmation() {
    let mut online = character(2, "Online");
    online.online = true;
    let mut rename = character(3, "Legacy");
    rename.online = true;
    rename.force_rename = true;
    let mut flow = OnlineCharacterFlow::new(CharacterSortMode::Level);
    flow.replace_roster(vec![character(1, "Ready"), online, rename])
        .expect("valid roster");

    flow.select(1).expect("ready row");
    assert_eq!(
        flow.primary_action().expect("enter world"),
        OnlineCharacterEffect::EnterWorld { character_id: 1 }
    );

    flow.select(2).expect("online row");
    assert_eq!(
        flow.primary_action().expect("request confirmation"),
        OnlineCharacterEffect::ConfirmTakeOver {
            character_id: 2,
            character_name: "Online".to_string(),
        }
    );
    assert_eq!(
        flow.confirm_takeover().expect("confirmed takeover"),
        OnlineCharacterEffect::TakeOverAndEnter { character_id: 2 }
    );
    assert_eq!(
        flow.confirm_takeover()
            .expect_err("confirmation is one shot"),
        OnlineCharacterFlowError::NoPendingTakeOver
    );

    flow.select(3).expect("rename row");
    assert_eq!(
        flow.primary_action()
            .expect_err("rename requirement blocks entry"),
        OnlineCharacterFlowError::EntryBlocked(CharacterEntryBlock::RenameRequired {
            character_id: 3,
        })
    );
}

#[test]
fn create_request_uses_the_shared_class_catalog_and_normalized_name() {
    let mut flow = OnlineCharacterFlow::new(CharacterSortMode::Level);
    flow.replace_roster(Vec::new()).expect("empty roster");
    assert_eq!(
        flow.create_draft().player_class(),
        OfflinePlayerClass::Warrior
    );

    flow.set_create_class(OfflinePlayerClass::Paladin)
        .expect("select Paladin");
    assert_eq!(flow.create_draft().skin_variant(), 0);
    assert_eq!(
        flow.set_create_skin(2)
            .expect_err("Paladin exposes two variants"),
        OnlineCharacterFlowError::InvalidSkinVariant {
            player_class: OfflinePlayerClass::Paladin,
            skin_variant: 2,
            skin_count: 2,
        }
    );
    flow.set_create_skin(1).expect("Paladin variant one");
    flow.set_create_name("  Uther  ").expect("name edit");

    let request = flow.submit_create().expect("create request");
    assert_eq!(
        request,
        OnlineCharacterEffect::Create {
            name: "Uther".to_string(),
            player_class: OfflinePlayerClass::Paladin,
            skin_variant: 1,
        }
    );
    flow.complete_create().expect("API create completed");
    assert_eq!(flow.screen(), CharacterRosterScreen::SelectCharacter);
    assert_eq!(flow.create_draft().raw_name(), "");
    assert_eq!(
        flow.create_draft().player_class(),
        OfflinePlayerClass::Paladin
    );
    assert_eq!(flow.create_draft().skin_variant(), 1);
}

#[test]
fn rename_request_is_available_only_for_the_forced_rename_row() {
    let mut forced = character(2, "Legacy");
    forced.force_rename = true;
    let mut flow = OnlineCharacterFlow::new(CharacterSortMode::Level);
    flow.replace_roster(vec![character(1, "Ready"), forced])
        .expect("valid roster");

    assert_eq!(
        flow.submit_rename(1, "Renamed")
            .expect_err("ready row cannot be renamed here"),
        OnlineCharacterFlowError::RenameNotRequired { character_id: 1 }
    );
    assert_eq!(
        flow.submit_rename(2, "  New Legacy  ")
            .expect("rename request"),
        OnlineCharacterEffect::Rename {
            character_id: 2,
            name: "New Legacy".to_string(),
        }
    );
}

#[test]
fn delete_requires_an_offline_row_and_a_typed_case_insensitive_name_match() {
    let mut online = character(2, "Online");
    online.online = true;
    let mut flow = OnlineCharacterFlow::new(CharacterSortMode::Level);
    flow.replace_roster(vec![character(1, "Mira"), online])
        .expect("valid roster");

    assert_eq!(
        flow.open_delete(2)
            .expect_err("online deletion is disabled"),
        OnlineCharacterFlowError::CannotDeleteOnline { character_id: 2 }
    );
    flow.open_delete(1).expect("offline delete dialog");
    flow.set_delete_confirmation("wrong")
        .expect("confirmation edit");
    assert_eq!(
        flow.submit_delete().expect_err("typed name mismatch"),
        OnlineCharacterFlowError::DeleteConfirmationMismatch { character_id: 1 }
    );

    flow.set_delete_confirmation("  mIrA ")
        .expect("case-insensitive match");
    assert_eq!(
        flow.submit_delete().expect("delete request"),
        OnlineCharacterEffect::Delete {
            character_id: 1,
            confirmation: "  mIrA ".to_string(),
        }
    );
    assert!(flow.delete_dialog().is_none());
}

#[test]
fn roster_refresh_drops_confirmations_for_characters_that_disappeared() {
    let mut online = character(2, "Online");
    online.online = true;
    let mut flow = OnlineCharacterFlow::new(CharacterSortMode::Level);
    flow.replace_roster(vec![character(1, "Mira"), online])
        .expect("valid roster");
    flow.select(2).expect("online row");
    flow.primary_action().expect("takeover prompt");
    flow.open_delete(1).expect("delete dialog");

    flow.replace_roster(vec![character(3, "Fresh")])
        .expect("realm refresh");

    assert_eq!(flow.delete_dialog(), None);
    assert_eq!(
        flow.confirm_takeover()
            .expect_err("stale takeover must be dropped"),
        OnlineCharacterFlowError::NoPendingTakeOver
    );
}
