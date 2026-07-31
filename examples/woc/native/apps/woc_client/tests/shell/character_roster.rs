use woc_client::{
    normalize_character_name, CharacterAppearanceRig, CharacterEntryBlock, CharacterNameError,
    CharacterPrimaryAction, CharacterRosterEntry, CharacterRosterError, CharacterRosterModel,
    CharacterRosterScreen, CharacterSortMode,
};

fn character(id: u64, name: &str, level: u16) -> CharacterRosterEntry {
    CharacterRosterEntry {
        id,
        name: name.to_string(),
        class_id: "warrior".to_string(),
        level,
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

fn ids(model: &CharacterRosterModel) -> Vec<u64> {
    model.entries().iter().map(|entry| entry.id).collect()
}

#[test]
fn empty_roster_opens_creation_and_nonempty_roster_selects_the_first_sorted_row() {
    let mut model = CharacterRosterModel::new(CharacterSortMode::Level);
    assert_eq!(model.screen(), CharacterRosterScreen::CreateCharacter);
    assert_eq!(model.selected_id(), None);
    assert_eq!(
        model.primary_action(),
        CharacterPrimaryAction::Disabled(CharacterEntryBlock::NoSelection)
    );

    model
        .replace_entries(vec![character(1, "Mira", 5), character(2, "Aldwin", 20)])
        .expect("valid roster");
    assert_eq!(model.screen(), CharacterRosterScreen::SelectCharacter);
    assert_eq!(ids(&model), vec![2, 1]);
    assert_eq!(model.selected_id(), Some(2));
    assert_eq!(
        model.primary_action(),
        CharacterPrimaryAction::EnterWorld { character_id: 2 }
    );
}

#[test]
fn primary_action_prioritizes_rename_then_takeover_then_enter() {
    let mut model = CharacterRosterModel::new(CharacterSortMode::Level);
    let mut ready = character(1, "Ready", 10);
    let mut online = character(2, "Online", 10);
    online.online = true;
    let mut blocked = character(3, "Legacy", 10);
    blocked.online = true;
    blocked.force_rename = true;
    model
        .replace_entries(vec![ready.clone(), online, blocked])
        .expect("valid roster");

    model.select(3).expect("blocked row");
    assert_eq!(
        model.primary_action(),
        CharacterPrimaryAction::Disabled(CharacterEntryBlock::RenameRequired { character_id: 3 })
    );
    model.select(2).expect("online row");
    assert_eq!(
        model.primary_action(),
        CharacterPrimaryAction::TakeOver { character_id: 2 }
    );
    ready.online = false;
    model.select(1).expect("ready row");
    assert_eq!(
        model.primary_action(),
        CharacterPrimaryAction::EnterWorld { character_id: 1 }
    );
}

#[test]
fn four_sort_modes_match_the_pinned_roster_rules() {
    let mut zara = character(1, "Zara", 5);
    zara.last_played_epoch_ms = Some(100);
    zara.playtime_seconds = 3_600;
    let mut aldwin = character(2, "aldwin", 20);
    aldwin.last_played_epoch_ms = None;
    let mut mira = character(3, "Mira", 12);
    mira.last_played_epoch_ms = Some(200);
    mira.playtime_seconds = 7_200;
    let source = vec![zara, aldwin, mira];
    let mut model = CharacterRosterModel::new(CharacterSortMode::Level);
    model.replace_entries(source).expect("valid roster");
    assert_eq!(ids(&model), vec![2, 3, 1]);

    model.set_sort_mode(CharacterSortMode::Name);
    assert_eq!(ids(&model), vec![2, 3, 1]);
    model.set_sort_mode(CharacterSortMode::Recent);
    assert_eq!(ids(&model), vec![3, 1, 2]);
    model.set_sort_mode(CharacterSortMode::Playtime);
    assert_eq!(ids(&model), vec![3, 1, 2]);
}

#[test]
fn ties_use_case_insensitive_name_then_id_without_mutating_identity_selection() {
    let mut model = CharacterRosterModel::new(CharacterSortMode::Level);
    model
        .replace_entries(vec![
            character(7, "Bex", 10),
            character(3, "Bex", 10),
            character(5, "Ana", 10),
        ])
        .expect("valid roster");
    assert_eq!(ids(&model), vec![5, 3, 7]);
    model.select(7).expect("select Bex 7");

    model.set_sort_mode(CharacterSortMode::Name);
    assert_eq!(model.selected_id(), Some(7));
    model
        .replace_entries(vec![character(5, "Ana", 11), character(7, "Bex", 12)])
        .expect("refresh preserves selected identity");
    assert_eq!(model.selected_id(), Some(7));

    model
        .replace_entries(vec![character(5, "Ana", 11)])
        .expect("missing selection falls back to first row");
    assert_eq!(model.selected_id(), Some(5));
}

#[test]
fn invalid_refresh_is_atomic_and_unknown_selection_is_rejected() {
    let mut model = CharacterRosterModel::new(CharacterSortMode::Level);
    model
        .replace_entries(vec![character(1, "Mira", 5)])
        .expect("initial roster");

    assert_eq!(
        model
            .replace_entries(vec![character(2, "A", 5), character(2, "B", 6)])
            .expect_err("duplicate identity"),
        CharacterRosterError::DuplicateCharacterId { character_id: 2 }
    );
    assert_eq!(ids(&model), vec![1]);
    assert_eq!(model.selected_id(), Some(1));
    assert_eq!(
        model.select(99).expect_err("unknown row"),
        CharacterRosterError::CharacterNotFound { character_id: 99 }
    );

    let mut invalid = character(3, "", 0);
    invalid.class_id.clear();
    assert_eq!(
        model
            .replace_entries(vec![invalid])
            .expect_err("invalid authority row"),
        CharacterRosterError::InvalidField {
            character_id: 3,
            field: "name",
        }
    );
    assert_eq!(ids(&model), vec![1]);
}

#[test]
fn character_name_shape_matches_the_pinned_client_and_server_rule() {
    for (raw, expected) in [
        ("Thrall", "Thrall"),
        ("Jaina Proudmoore", "Jaina Proudmoore"),
        ("Kael'thas", "Kael'thas"),
        ("Rexxar-Misha", "Rexxar-Misha"),
        ("  Uther  ", "Uther"),
        ("Jaina  Proudmoor", "Jaina  Proudmoor"),
        ("Ab", "Ab"),
    ] {
        assert_eq!(
            normalize_character_name(raw).expect("valid character name"),
            expected
        );
    }

    assert_eq!(
        normalize_character_name("   ").expect_err("blank name"),
        CharacterNameError::Empty
    );
    assert_eq!(
        normalize_character_name("A").expect_err("one character"),
        CharacterNameError::InvalidLength { actual: 1 }
    );
    assert_eq!(
        normalize_character_name("123Adventurer").expect_err("digit prefix"),
        CharacterNameError::FirstCharacter
    );
    assert_eq!(
        normalize_character_name("Jaina\tProudmoore").expect_err("tab is not an allowed name byte"),
        CharacterNameError::InvalidCharacter { index: 5 }
    );
    for raw in [
        "Averylongnameherebuttoolong",
        "Adventurer!",
        "-Adventurer",
        "'Adventurer",
    ] {
        assert!(normalize_character_name(raw).is_err(), "{raw} must fail");
    }
}
