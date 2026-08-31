use std::collections::BTreeMap;

use crate::math::{Quat, Transform, Vec3};
use crate::reflect::ReflectedValue;
use crate::resource::ResourceId;
use crate::world_sync::{
    AssetReloadFrameApplyReportDto, ComponentSelector, ComponentWorldQuery, EntityRow,
    InvalidationBatch, QueryFilter, WatchKey, WatchRegistration, WatchToken, WorldFact,
    WorldHierarchyQuery, WorldHierarchyRow, WorldInspectionFieldRow, WorldQuery, WorldQueryResult,
};

fn json_round_trip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    serde_json::from_str(&serde_json::to_string(value).unwrap()).unwrap()
}

#[test]
fn world_query_contract_round_trips_filters_selectors_and_generation_hint() {
    let query = WorldQuery::Components(ComponentWorldQuery {
        filter: QueryFilter {
            with: vec!["zircon.scene.Transform".to_string()],
            without: vec!["zircon.scene.Hidden".to_string()],
        },
        select: vec![
            ComponentSelector::new("zircon.scene.Name"),
            ComponentSelector::new("zircon.scene.Transform"),
        ],
        generation_hint: Some(41),
    });

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
    let current = WorldQuery::Components(ComponentWorldQuery {
        generation_hint: Some(9),
        ..ComponentWorldQuery::default()
    });
    let stale = WorldQuery::Components(ComponentWorldQuery {
        generation_hint: Some(8),
        ..ComponentWorldQuery::default()
    });

    assert_eq!(
        current.component_result_for_generation(9, rows.clone()),
        WorldQueryResult::NotModified { generation: 9 }
    );
    assert_eq!(
        stale.component_result_for_generation(9, rows.clone()),
        WorldQueryResult::ComponentRows {
            generation: 9,
            rows: rows.clone(),
        }
    );
    assert_eq!(
        WorldQuery::default().component_result_for_generation(9, rows.clone()),
        WorldQueryResult::ComponentRows {
            generation: 9,
            rows,
        }
    );
}

#[test]
fn hierarchy_query_round_trips_generation_bearing_runtime_rows() {
    let query = WorldQuery::Hierarchy(WorldHierarchyQuery {
        generation_hint: Some(11),
    });
    let rows = vec![WorldHierarchyRow {
        entity: 7,
        parent: None,
        depth: 0,
        display_name: "Runtime Root".to_string(),
        kind: "Entity".to_string(),
        subtree_hash: 19,
        active_in_hierarchy: true,
        has_children: false,
    }];

    assert_eq!(json_round_trip(&query), query);
    assert_eq!(
        query.hierarchy_result_for_generation(12, rows.clone()),
        WorldQueryResult::HierarchyRows {
            generation: 12,
            rows,
        }
    );
}

#[test]
fn focused_inspector_query_round_trips_fields_and_missing_entity_state() {
    let query = WorldQuery::inspection_fields(7, Some(11));
    let fields = vec![WorldInspectionFieldRow {
        component_type_path: "zircon.scene.Name".to_string(),
        component_display_name: "Name".to_string(),
        field_name: "value".to_string(),
        field_display_name: "Value".to_string(),
        value_type_path: "String".to_string(),
        value: ReflectedValue::String("Runtime Root".to_string()),
        writable: true,
        serializable: true,
        plugin_owned: false,
    }];

    assert_eq!(json_round_trip(&query), query);
    assert_eq!(
        query.inspection_fields_result_for_generation(12, 7, Some(fields.clone())),
        WorldQueryResult::InspectionFields {
            generation: 12,
            entity: 7,
            fields,
        }
    );
    assert_eq!(
        query.inspection_fields_result_for_generation(12, 9, None),
        WorldQueryResult::EntityMissing {
            generation: 12,
            entity: 9,
        }
    );
}

#[test]
fn transform_snapshot_query_carries_world_replacement_identity() {
    let transform = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
    let query = WorldQuery::transform_snapshot(7);
    let result = query.transform_snapshot_result(11, 13, 7, Some(transform));

    assert_eq!(
        result,
        WorldQueryResult::TransformSnapshot {
            generation: 11,
            world_replacement_epoch: 13,
            entity: 7,
            transform,
        }
    );
    assert_eq!(json_round_trip(&query), query);
}

#[test]
fn entity_rows_are_canonicalized_by_stable_entity_identity() {
    let rows = vec![
        EntityRow {
            entity: 9,
            components: BTreeMap::new(),
        },
        EntityRow {
            entity: 2,
            components: BTreeMap::new(),
        },
    ];

    assert_eq!(
        WorldQuery::default().component_result_for_generation(1, rows),
        WorldQueryResult::ComponentRows {
            generation: 1,
            rows: vec![
                EntityRow {
                    entity: 2,
                    components: BTreeMap::new(),
                },
                EntityRow {
                    entity: 9,
                    components: BTreeMap::new(),
                },
            ],
        }
    );
}

#[test]
fn saturated_generation_never_returns_not_modified() {
    let query = WorldQuery::Components(ComponentWorldQuery {
        generation_hint: Some(u64::MAX),
        ..ComponentWorldQuery::default()
    });

    assert_eq!(
        query.component_result_for_generation(u64::MAX, Vec::new()),
        WorldQueryResult::ComponentRows {
            generation: u64::MAX,
            rows: Vec::new(),
        }
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
            WorldFact::WorldReplaced {
                replacement_epoch: 17,
            },
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
    assert!(encoded.contains("world_replaced"));
    assert_eq!(decoded, batch);
}

#[test]
fn canonical_dirty_tokens_require_strictly_increasing_runtime_tokens() {
    let canonical = InvalidationBatch {
        generation: 23,
        dirty: vec![WatchToken::new(2), WatchToken::new(5)],
        facts: Vec::new(),
    };
    let duplicated = InvalidationBatch {
        dirty: vec![WatchToken::new(2), WatchToken::new(2)],
        ..canonical.clone()
    };
    let unordered = InvalidationBatch {
        dirty: vec![WatchToken::new(5), WatchToken::new(2)],
        ..canonical.clone()
    };

    assert!(canonical.has_canonical_dirty_tokens());
    assert!(!duplicated.has_canonical_dirty_tokens());
    assert!(!unordered.has_canonical_dirty_tokens());
}

#[test]
fn world_sync_contract_rejects_retired_or_unknown_wire_fields() {
    let query = r#"{
        "kind":"components",
        "data":{
            "filter":{"with":[],"without":[]},
            "select":[],
            "generation_hint":null,
            "legacy_generation":4
        }
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
