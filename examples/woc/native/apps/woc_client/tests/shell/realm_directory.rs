use woc_client::{
    realm_population, RealmDefinition, RealmDirectoryEffect, RealmDirectoryError,
    RealmDirectoryModel, RealmPopulationBand, RealmStatus, RealmStatusState, RealmType,
};

fn realm(name: &str, url: &str, realm_type: RealmType, characters: u32) -> RealmDefinition {
    RealmDefinition {
        name: name.to_string(),
        base_url: url.to_string(),
        realm_type,
        character_count: characters,
    }
}

#[test]
fn population_bands_match_the_target_edges_and_offline_precedence() {
    assert_eq!(RealmType::Normal.label_key(), "realmTypes.normal");
    assert_eq!(RealmType::Pvp.label_key(), "realmTypes.pvp");
    assert_eq!(RealmType::Rp.label_key(), "realmTypes.rp");
    assert_eq!(RealmType::RpPvp.label_key(), "realmTypes.rpPvp");

    let offline = realm_population(false, 999, 5);
    assert_eq!(offline.band, RealmPopulationBand::Offline);
    assert_eq!(offline.label_key, "realm.offline");
    assert_eq!(offline.tip_key, "realm.popTipOffline");
    assert_eq!(offline.style_class, "offline");

    assert_eq!(realm_population(true, 5, 5).band, RealmPopulationBand::Full);
    assert_eq!(realm_population(true, 6, 5).band, RealmPopulationBand::Full);
    assert_eq!(realm_population(true, 4, 5).band, RealmPopulationBand::Low);
    assert_eq!(
        realm_population(true, 999, 0).band,
        RealmPopulationBand::High
    );
    assert_eq!(
        realm_population(true, 80, 0).band,
        RealmPopulationBand::High
    );
    assert_eq!(
        realm_population(true, 79, 0).band,
        RealmPopulationBand::Medium
    );
    assert_eq!(
        realm_population(true, 15, 0).band,
        RealmPopulationBand::Medium
    );
    assert_eq!(realm_population(true, 14, 0).band, RealmPopulationBand::Low);
}

#[test]
fn remembered_realm_auto_selects_while_a_missing_preference_shows_source_order() {
    let definitions = vec![
        realm("Eastbrook", "", RealmType::Normal, 2),
        realm("Ashenfall", "https://ashen.example", RealmType::Pvp, 0),
    ];
    let mut model = RealmDirectoryModel::default();

    assert_eq!(
        model
            .replace_directory(definitions.clone(), Some("Ashenfall"))
            .expect("valid directory"),
        RealmDirectoryEffect::SelectRealm {
            realm_name: "Ashenfall".to_string(),
            base_url: "https://ashen.example".to_string(),
        }
    );
    assert_eq!(
        model
            .entries()
            .iter()
            .map(|entry| entry.definition.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Eastbrook", "Ashenfall"]
    );

    assert_eq!(
        model
            .replace_directory(definitions, Some("Unknown"))
            .expect("valid directory"),
        RealmDirectoryEffect::ShowList
    );
}

#[test]
fn directory_validation_is_atomic_and_allows_the_same_server_empty_url() {
    let mut model = RealmDirectoryModel::default();
    model
        .replace_directory(vec![realm("Eastbrook", "", RealmType::Normal, 1)], None)
        .expect("same-server URL may be empty");

    assert_eq!(
        model
            .replace_directory(
                vec![
                    realm("Ashenfall", "one", RealmType::Rp, 0),
                    realm("Ashenfall", "two", RealmType::RpPvp, 0),
                ],
                None,
            )
            .expect_err("duplicate realm identity"),
        RealmDirectoryError::DuplicateRealmName {
            realm_name: "Ashenfall".to_string(),
        }
    );
    assert_eq!(model.entries()[0].definition.name, "Eastbrook");

    assert_eq!(
        model
            .replace_directory(vec![realm("", "one", RealmType::Normal, 0)], None)
            .expect_err("blank realm name"),
        RealmDirectoryError::EmptyRealmName { index: 0 }
    );
    assert_eq!(model.entries()[0].definition.name, "Eastbrook");
}

#[test]
fn status_refresh_preserves_rows_and_recommends_the_first_lowest_online_population() {
    let mut model = RealmDirectoryModel::default();
    model
        .replace_directory(
            vec![
                realm("Eastbrook", "one", RealmType::Normal, 2),
                realm("Ashenfall", "two", RealmType::Pvp, 0),
                realm("Moonvale", "three", RealmType::Rp, 1),
                realm("Down", "four", RealmType::RpPvp, 0),
            ],
            None,
        )
        .expect("valid directory");
    assert!(model
        .entries()
        .iter()
        .all(|entry| entry.status == RealmStatusState::Checking));

    model
        .set_status("Eastbrook", RealmStatus::online(20, 100))
        .expect("Eastbrook status");
    model
        .set_status("Ashenfall", RealmStatus::online(3, 100))
        .expect("Ashenfall status");
    model
        .set_status("Moonvale", RealmStatus::online(3, 100))
        .expect("Moonvale status");
    model
        .set_status("Down", RealmStatus::offline())
        .expect("Down status");
    model.finish_status_refresh();

    assert_eq!(model.recommended_realm_name(), Some("Ashenfall"));
    assert!(model.entries()[1].recommended);
    assert!(!model.entries()[2].recommended, "ties keep source order");
    assert_eq!(
        model.entries()[0].status,
        RealmStatusState::Resolved {
            status: RealmStatus::online(20, 100),
            population: realm_population(true, 20, 100),
        }
    );
}

#[test]
fn selecting_a_row_emits_the_exact_name_and_url_for_host_persistence() {
    let mut model = RealmDirectoryModel::default();
    model
        .replace_directory(
            vec![realm(
                "Ashenfall",
                "https://ashen.example",
                RealmType::Pvp,
                4,
            )],
            None,
        )
        .expect("valid directory");

    assert_eq!(
        model.select("Ashenfall").expect("known row"),
        RealmDirectoryEffect::SelectRealm {
            realm_name: "Ashenfall".to_string(),
            base_url: "https://ashen.example".to_string(),
        }
    );
    assert_eq!(model.back(), RealmDirectoryEffect::NavigateToModeSelection);
}

#[test]
fn unknown_status_and_selection_updates_are_rejected_without_mutation() {
    let mut model = RealmDirectoryModel::default();
    model
        .replace_directory(vec![realm("Eastbrook", "", RealmType::Normal, 1)], None)
        .expect("valid directory");

    assert_eq!(
        model
            .set_status("Unknown", RealmStatus::online(1, 10))
            .expect_err("unknown status row"),
        RealmDirectoryError::RealmNotFound {
            realm_name: "Unknown".to_string(),
        }
    );
    assert_eq!(
        model.select("Unknown").expect_err("unknown selection"),
        RealmDirectoryError::RealmNotFound {
            realm_name: "Unknown".to_string(),
        }
    );
    assert_eq!(model.entries()[0].status, RealmStatusState::Checking);
}
