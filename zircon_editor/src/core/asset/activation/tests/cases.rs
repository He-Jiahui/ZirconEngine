use serde_json::{json, Value};
use zircon_runtime::asset::{AssetUri, AssetUuid};

use super::{
    AssetActivationIntent, AssetActivationReceipt, AssetActivationResult, AssetActivationSource,
};

fn intent(source: AssetActivationSource, resource_revision: Option<u64>) -> AssetActivationIntent {
    AssetActivationIntent {
        asset_uuid: AssetUuid::from_stable_label("activation-contract-target"),
        asset_locator: AssetUri::parse("res://graphs/locomotion.zag").expect("valid asset locator"),
        catalog_revision: (1_u64 << 60) + 7,
        resource_revision,
        source,
    }
}

#[test]
fn activation_intent_json_preserves_exact_identity_revisions_and_each_source() {
    for source in [
        AssetActivationSource::DoubleClick,
        AssetActivationSource::Enter,
        AssetActivationSource::OpenCommand,
        AssetActivationSource::ContextMenu,
        AssetActivationSource::Reference,
    ] {
        let captured = intent(source, Some(u64::MAX));
        let value = serde_json::to_value(&captured).expect("serialize activation intent");
        assert_eq!(value["asset_uuid"], json!(captured.asset_uuid));
        assert_eq!(value["asset_locator"], "res://graphs/locomotion.zag");
        assert_eq!(value["catalog_revision"], json!((1_u64 << 60) + 7));
        assert_eq!(value["resource_revision"], json!(u64::MAX));
        assert_eq!(
            serde_json::from_value::<AssetActivationIntent>(value)
                .expect("deserialize activation intent"),
            captured
        );
    }

    let without_resource_revision = intent(AssetActivationSource::Enter, None);
    let value = serde_json::to_value(&without_resource_revision).expect("serialize no revision");
    assert_eq!(value["resource_revision"], Value::Null);
    assert_eq!(
        serde_json::from_value::<AssetActivationIntent>(value)
            .expect("deserialize no resource revision"),
        without_resource_revision
    );
}

#[test]
fn activation_receipt_json_keeps_four_distinct_terminal_results() {
    for (result, variant) in [
        (
            AssetActivationResult::Opened {
                view_instance_id: "asset-view-opened".to_string(),
            },
            "Opened",
        ),
        (
            AssetActivationResult::Reused {
                view_instance_id: "asset-view-reused".to_string(),
            },
            "Reused",
        ),
        (
            AssetActivationResult::Unavailable {
                reason: "catalog revision changed".to_string(),
            },
            "Unavailable",
        ),
        (
            AssetActivationResult::Failed {
                message: "toolkit dispatch failed".to_string(),
            },
            "Failed",
        ),
    ] {
        let receipt = AssetActivationReceipt {
            intent: intent(AssetActivationSource::ContextMenu, Some(11)),
            result,
        };
        let value = serde_json::to_value(&receipt).expect("serialize activation receipt");
        assert!(value["result"].get(variant).is_some(), "{variant} result");
        assert_eq!(
            serde_json::from_value::<AssetActivationReceipt>(value)
                .expect("deserialize activation receipt"),
            receipt
        );
    }
}

#[test]
fn activation_intent_decode_rejects_invalid_locator() {
    let mut value =
        serde_json::to_value(intent(AssetActivationSource::Reference, None)).expect("intent");
    value["asset_locator"] = json!("file:///outside/project");
    assert!(serde_json::from_value::<AssetActivationIntent>(value).is_err());
}
