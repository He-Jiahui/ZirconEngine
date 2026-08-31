use std::collections::BTreeMap;

use zircon_runtime_interface::ui::binding::{
    UiBindingAssetReference, UiBindingCollectionView, UiBindingEntityReference, UiBindingEnumValue,
    UiBindingMapKey, UiBindingValue, UiBindingValueBudget, UiBindingValueValidationError,
    UiEventBinding, UiEventKind, UiEventPath, UiModelProviderId, UiModelProviderKey,
    UiModelProviderVersion, UiModelSchemaId, UiModelSchemaKey, UiModelSchemaVersion,
    UI_BINDING_COLLECTION_VIEW_MAX_LENGTH, UI_BINDING_VALUE_MAX_COLLECTION_ENTRIES,
    UI_BINDING_VALUE_MAX_STRING_BYTES,
};

fn provider_key() -> UiModelProviderKey {
    UiModelProviderKey {
        id: UiModelProviderId::try_new("editor.asset_rows").unwrap(),
        version: UiModelProviderVersion::try_new(4).unwrap(),
    }
}

fn item_schema_key() -> UiModelSchemaKey {
    UiModelSchemaKey {
        id: UiModelSchemaId::try_new("editor.asset_row").unwrap(),
        version: UiModelSchemaVersion::try_new(7).unwrap(),
    }
}

fn collection_view(total_length: u64) -> UiBindingCollectionView {
    UiBindingCollectionView::try_new(
        provider_key(),
        item_schema_key(),
        11,
        128,
        UI_BINDING_COLLECTION_VIEW_MAX_LENGTH,
        total_length,
    )
    .unwrap()
}

fn rich_binding_value() -> UiBindingValue {
    let mut record = BTreeMap::new();
    record.insert(
        "asset".to_string(),
        UiBindingValue::Asset(
            UiBindingAssetReference::try_new("asset://textures/checker").unwrap(),
        ),
    );
    record.insert(
        "entity".to_string(),
        UiBindingValue::Entity(UiBindingEntityReference::try_new(42, 9).unwrap()),
    );
    record.insert(
        "mode".to_string(),
        UiBindingValue::Enum(
            UiBindingEnumValue::try_new(
                "editor.selection_mode",
                "replace",
                Some(UiBindingValue::Unsigned(3)),
            )
            .unwrap(),
        ),
    );
    record.insert("large_ratio".to_string(), UiBindingValue::Float(1.0e100));
    record.insert(
        "optional_label".to_string(),
        UiBindingValue::Optional(Some(Box::new(UiBindingValue::string("selected")))),
    );
    record.insert("missing_label".to_string(), UiBindingValue::Optional(None));
    record.insert(
        "lookup".to_string(),
        UiBindingValue::map([
            (
                UiBindingMapKey::String("primary".to_string()),
                UiBindingValue::Unsigned(1),
            ),
            (UiBindingMapKey::Unsigned(7), UiBindingValue::Bool(true)),
        ])
        .unwrap(),
    );
    record.insert(
        "rows".to_string(),
        UiBindingValue::CollectionView(collection_view(1_000_000)),
    );
    UiBindingValue::record(record).unwrap()
}

#[test]
fn binding_value_rich_contract_round_trips_serde_and_native_binding() {
    let value = rich_binding_value();
    value.validate().unwrap();

    let json = serde_json::to_vec(&value).unwrap();
    assert_eq!(
        serde_json::from_slice::<UiBindingValue>(&json).unwrap(),
        value
    );

    let binary = bincode::serialize(&value).unwrap();
    assert_eq!(
        bincode::deserialize::<UiBindingValue>(&binary).unwrap(),
        value
    );

    let binding = UiEventBinding::new(
        UiEventPath::new("AssetBrowser", "Rows", UiEventKind::Change),
        zircon_runtime_interface::ui::binding::UiBindingCall::new("ApplySelection")
            .with_argument(value),
    );
    let native = binding.native_binding();
    assert!(native.contains("record("));
    assert!(native.contains("collection_view("));
    assert_eq!(
        UiEventBinding::parse_native_binding(&native).unwrap(),
        binding
    );
}

#[test]
fn binding_value_contract_rejects_every_owned_budget_and_identity_overflow() {
    let oversized_collection = UiBindingValue::Array(vec![
        UiBindingValue::Null;
        UI_BINDING_VALUE_MAX_COLLECTION_ENTRIES
            + 1
    ]);
    assert!(matches!(
        oversized_collection.validate(),
        Err(UiBindingValueValidationError::CollectionEntriesExceeded { .. })
    ));
    assert!(serde_json::from_value::<UiBindingValue>(
        serde_json::to_value(&oversized_collection).unwrap()
    )
    .is_err());
    let oversized_native_array = format!(
        "BudgetView/Rows:onChange(Apply([{}]))",
        vec!["null"; UI_BINDING_VALUE_MAX_COLLECTION_ENTRIES + 1].join(",")
    );
    assert!(matches!(
        UiEventBinding::parse_native_binding(&oversized_native_array),
        Err(
            zircon_runtime_interface::ui::binding::UiBindingParseError::InvalidValue(
                UiBindingValueValidationError::CollectionEntriesExceeded { .. }
            )
        )
    ));

    let oversized_string =
        UiBindingValue::String("x".repeat(UI_BINDING_VALUE_MAX_STRING_BYTES + 1));
    assert!(matches!(
        oversized_string.validate(),
        Err(UiBindingValueValidationError::StringBudgetExceeded { .. })
    ));

    let nested = UiBindingValue::Array(vec![UiBindingValue::Array(vec![UiBindingValue::Null])]);
    assert!(matches!(
        nested.validate_with_budget(UiBindingValueBudget::new(2, 32, 1_024, 16)),
        Err(UiBindingValueValidationError::DepthExceeded { .. })
    ));
    assert!(matches!(
        nested.validate_with_budget(UiBindingValueBudget::new(8, 2, 1_024, 16)),
        Err(UiBindingValueValidationError::NodeBudgetExceeded { .. })
    ));

    assert!(matches!(
        UiBindingValue::map([
            (UiBindingMapKey::Unsigned(1), UiBindingValue::Bool(true)),
            (UiBindingMapKey::Unsigned(1), UiBindingValue::Bool(false)),
        ]),
        Err(UiBindingValueValidationError::DuplicateMapKey)
    ));
    assert!(matches!(
        UiBindingEntityReference::try_new(42, 0),
        Err(UiBindingValueValidationError::ZeroGeneration { .. })
    ));
    assert!(matches!(
        UiBindingCollectionView::try_new(
            provider_key(),
            item_schema_key(),
            1,
            0,
            UI_BINDING_COLLECTION_VIEW_MAX_LENGTH + 1,
            1_000_000,
        ),
        Err(UiBindingValueValidationError::CollectionViewWindowExceeded { .. })
    ));
}

#[test]
fn binding_value_json_projection_preserves_existing_shapes_and_tags_typed_values() {
    assert_eq!(
        UiBindingValue::Array(vec![
            UiBindingValue::Unsigned(4),
            UiBindingValue::Bool(true)
        ])
        .to_json_value(),
        serde_json::json!([4, true])
    );

    let projected = rich_binding_value().to_json_value();
    assert_eq!(
        projected["asset"],
        serde_json::json!({"$asset": "asset://textures/checker"})
    );
    assert_eq!(projected["entity"]["$entity"]["id"], 42);
    assert_eq!(
        projected["rows"]["$collection_view"]["total_length"],
        1_000_000
    );
    assert_eq!(projected["optional_label"], "selected");
    assert!(projected["missing_label"].is_null());
}

#[test]
fn controlled_collection_view_reduces_serialized_payload_by_at_least_ninety_five_percent() {
    const MATERIALIZED_ROWS: usize = UI_BINDING_COLLECTION_VIEW_MAX_LENGTH as usize;
    const TOTAL_ROWS: u64 = 1_000_000;

    let rows = (0..MATERIALIZED_ROWS)
        .map(|row| {
            let mut fields = BTreeMap::new();
            fields.insert("id".to_string(), UiBindingValue::Unsigned(row as u64));
            fields.insert(
                "label".to_string(),
                UiBindingValue::string(format!("Asset row {row:04}")),
            );
            UiBindingValue::record(fields).unwrap()
        })
        .collect::<Vec<_>>();
    let materialized = serde_json::to_vec(&UiBindingValue::Array(rows)).unwrap();
    let controlled =
        serde_json::to_vec(&UiBindingValue::CollectionView(collection_view(TOTAL_ROWS))).unwrap();
    let reduction_percent = ((materialized.len() - controlled.len()) * 100) / materialized.len();

    println!(
        "PERF-MVP-RTB-P1-010 materialized_window_rows={MATERIALIZED_ROWS} collection_total_rows={TOTAL_ROWS} materialized_json_bytes={} controlled_view_json_bytes={} reduction_percent={reduction_percent}",
        materialized.len(),
        controlled.len(),
    );
    assert!(
        reduction_percent >= 95,
        "controlled collection view must reduce the serialized payload by at least 95%"
    );
}
