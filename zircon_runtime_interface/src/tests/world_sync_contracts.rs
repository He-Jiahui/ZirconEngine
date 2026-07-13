use std::collections::BTreeMap;

use crate::resource::ResourceId;
use crate::world_sync::{
    AssetReloadFrameApplyReportDto, ComponentSelector, EntityRow, InvalidationBatch, QueryFilter,
    WatchKey, WatchRegistration, WatchToken, WorldFact, WorldQuery, WorldQueryResult,
};

fn json_round_trip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    serde_json::from_str(&serde_json::to_string(value).unwrap()).unwrap()
}

#[test]
fn world_query_contract_round_trips_filters_selectors_and_generation_hint() {
    let query = WorldQuery {
        filter: QueryFilter {
            with: vec!["zircon.scene.Transform".to_string()],
            without: vec!["zircon.scene.Hidden".to_string()],
        },
        select: vec![
            ComponentSelector::new("zircon.scene.Name"),
            ComponentSelector::new("zircon.scene.Transform"),
        ],
        generation_hint: Some(41),
    };

    let encoded = serde_json::to_string(&query).unwrap();
    let decoded = json_round_trip(&query);

    assert!(encoded.contains("generation_hint"));
    assert_eq!(decoded, query);
}

#[test]
fn world_query_returns_not_modified_only_for_a_matching_generation_hint() {
    let rows = vec![EntityRow {
        entity: 7,
        components: BTreeMap::from([(
            "zircon.scene.Name".to_string(),
            serde_json::json!({ "value": "Hero" }),
        )]),
    }];
    let current = WorldQuery {
        generation_hint: Some(9),
        ..WorldQuery::default()
    };
    let stale = WorldQuery {
        generation_hint: Some(8),
        ..WorldQuery::default()
    };

    assert_eq!(
        current.result_for_generation(9, rows.clone()),
        WorldQueryResult::NotModified { generation: 9 }
    );
    assert_eq!(
        stale.result_for_generation(9, rows.clone()),
        WorldQueryResult::Rows(rows.clone())
    );
    assert_eq!(
        WorldQuery::default().result_for_generation(9, rows.clone()),
        WorldQueryResult::Rows(rows)
    );
}

#[test]
fn watch_contract_round_trips_every_typed_key_and_token() {
    let resource_id = ResourceId::from_stable_label("res://scenes/hero.zscene");
    let registrations = [
        WatchRegistration::new(WatchKey::Subtree { root: 3 }),
        WatchRegistration::new(WatchKey::ComponentType {
            type_name: "zircon.scene.Transform".to_string(),
        }),
        WatchRegistration::new(WatchKey::Asset { resource_id }),
        WatchRegistration::new(WatchKey::WorldStructure),
    ];

    for registration in registrations {
        assert_eq!(json_round_trip(&registration), registration);
    }

    let token = WatchToken::new(17);
    assert_eq!(json_round_trip(&token), token);
    assert_eq!(token.value(), 17);
}

#[test]
fn invalidation_batch_round_trip_preserves_generation_tokens_and_facts() {
    let scene = ResourceId::from_stable_label("res://scenes/hero.zscene");
    let batch = InvalidationBatch {
        generation: 23,
        dirty: vec![WatchToken::new(2), WatchToken::new(5)],
        facts: vec![
            WorldFact::Spawned(10),
            WorldFact::Despawned(11),
            WorldFact::Reparented {
                entity: 12,
                new_parent: Some(10),
            },
            WorldFact::SceneLoaded { scene },
            WorldFact::SceneUnloaded { scene },
            WorldFact::AssetReloadApplied(AssetReloadFrameApplyReportDto {
                applied: 3,
                failed: 1,
                stale: 2,
                pending_count: 4,
            }),
        ],
    };

    let encoded = serde_json::to_string(&batch).unwrap();
    let decoded = json_round_trip(&batch);

    assert!(encoded.contains("asset_reload_applied"));
    assert_eq!(decoded, batch);
}

#[test]
fn world_sync_contract_rejects_retired_or_unknown_wire_fields() {
    let query = r#"{
        "filter":{"with":[],"without":[]},
        "select":[],
        "generation_hint":null,
        "legacy_generation":4
    }"#;
    let watch = r#"{
        "key":{"kind":"world_structure"},
        "view_id":"editor.hierarchy"
    }"#;

    assert!(serde_json::from_str::<WorldQuery>(query).is_err());
    assert!(serde_json::from_str::<WatchRegistration>(watch).is_err());
}

#[test]
fn world_sync_wire_tags_and_module_boundary_are_stable() {
    let not_modified = serde_json::to_value(WorldQueryResult::NotModified { generation: 12 })
        .expect("serialize not-modified result");
    let world_structure = serde_json::to_value(WatchRegistration::new(WatchKey::WorldStructure))
        .expect("serialize world-structure watch");

    assert_eq!(
        not_modified,
        serde_json::json!({
            "kind": "not_modified",
            "data": { "generation": 12 }
        })
    );
    assert_eq!(
        world_structure,
        serde_json::json!({ "key": { "kind": "world_structure" } })
    );

    let crate_root = include_str!("../lib.rs");
    let contract_sources = concat!(
        include_str!("../world_sync/query.rs"),
        include_str!("../world_sync/watch.rs"),
        include_str!("../world_sync/invalidation.rs")
    );
    assert!(crate_root.contains("pub mod world_sync;"));
    for forbidden in [
        "zircon_runtime::",
        "zircon_editor::",
        "ViewInstanceId",
        "std::net",
        "std::fs",
    ] {
        assert!(
            !contract_sources.contains(forbidden),
            "world-sync DTO boundary must not contain {forbidden}"
        );
    }
}
