use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::super::text::wire::MAX_TEXT_DOCUMENT_BYTES;
use super::super::{
    Format, LoadError, MigrateError, MigrationChain, MigrationStep, SchemaId, VersionedSchema,
    load_versioned,
};
use super::FixtureDocument;

#[test]
fn text_reader_rejects_an_input_larger_than_the_wire_budget_before_parsing() {
    let bytes = vec![b' '; MAX_TEXT_DOCUMENT_BYTES + 1];

    let error = load_versioned::<FixtureDocument>(&bytes, Format::Text)
        .expect_err("oversized text must fail before envelope or payload parsing");

    assert!(matches!(
        error,
        LoadError::TextDocumentTooLarge { max, found }
            if max == MAX_TEXT_DOCUMENT_BYTES && found == MAX_TEXT_DOCUMENT_BYTES + 1
    ));
}

#[test]
fn unwrapped_text_payload_is_recognized_as_v0_and_migrated_in_order() {
    let loaded = load_versioned::<FixtureDocument>(br#"{"name":"legacy"}"#, Format::Text)
        .expect("v0 fixture should migrate");

    assert_eq!(
        loaded.value,
        FixtureDocument {
            label: "legacy".to_string(),
            count: 2,
        }
    );
    assert_eq!(loaded.migrated_from, Some(0));
}

#[derive(Debug, Serialize, Deserialize)]
struct BrokenChainDocument {
    value: Value,
}

impl VersionedSchema for BrokenChainDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.broken-chain");
    const VERSION: u32 = 2;

    fn migrations() -> &'static MigrationChain<Self> {
        &BROKEN_CHAIN
    }
}

static BROKEN_CHAIN: MigrationChain<BrokenChainDocument> =
    MigrationChain::new(&[MigrationStep::new(0, identity_migration)]);

fn identity_migration(value: Value) -> Result<Value, MigrateError> {
    Ok(value)
}

#[test]
fn migration_chain_gap_reports_the_missing_source_version() {
    let error = load_versioned::<BrokenChainDocument>(br#"{"value":null}"#, Format::Text)
        .expect_err("a chain gap must not be skipped");

    assert!(matches!(
        error,
        LoadError::Migration(MigrateError::MissingStep {
            from_version: 1,
            target_version: 2,
            ..
        })
    ));
}

#[test]
fn future_schema_version_is_rejected_before_deserializing_payload() {
    let bytes = serde_json::to_vec(&json!({
        "$zircon": {
            "header": {
                "schema_id": "zircon.tests.fixture-document",
                "schema_version": 3
            },
            "payload": {
                "label": "future",
                "count": 99
            }
        }
    }))
    .unwrap();

    let error = load_versioned::<FixtureDocument>(&bytes, Format::Text)
        .expect_err("future versions require an explicit newer reader");

    assert!(matches!(
        error,
        LoadError::FutureVersion {
            found: 3,
            supported: 2,
            ..
        }
    ));
}

#[test]
fn claimed_text_envelopes_with_duplicate_reserved_header_fields_fail_closed() {
    for bytes in [
        br#"{
            "$zircon": {
                "header": {
                    "schema_id": "zircon.tests.fixture-document",
                    "schema_version": 2
                },
                "header": {
                    "schema_id": "zircon.tests.fixture-document",
                    "schema_version": 2
                },
                "payload": { "label": "duplicate header", "count": 7 }
            }
        }"#
        .as_slice(),
        br#"{
            "$zircon": {
                "header": {
                    "schema_id": "zircon.tests.fixture-document",
                    "schema_id": "zircon.tests.fixture-document",
                    "schema_version": 2
                },
                "payload": { "label": "duplicate schema", "count": 7 }
            }
        }"#
        .as_slice(),
    ] {
        let error = load_versioned::<FixtureDocument>(bytes, Format::Text)
            .expect_err("duplicate reserved fields must not downgrade to legacy version zero");

        assert!(matches!(error, LoadError::InvalidEnvelope { .. }));
    }
}

#[test]
fn current_envelope_loads_without_reporting_a_migration() {
    let bytes = serde_json::to_vec(&json!({
        "$zircon": {
            "header": {
                "schema_id": "zircon.tests.fixture-document",
                "schema_version": 2
            },
            "payload": {
                "label": "current",
                "count": 7
            }
        }
    }))
    .unwrap();

    let loaded = load_versioned::<FixtureDocument>(&bytes, Format::Text).unwrap();

    assert_eq!(loaded.value.label, "current");
    assert_eq!(loaded.value.count, 7);
    assert_eq!(loaded.migrated_from, None);
}

#[test]
fn current_text_envelope_uses_the_direct_typed_decode_path() {
    let source = include_str!("../load.rs");
    let text_load = source
        .split("fn load_text_versioned")
        .nth(1)
        .and_then(|body| body.split("fn load_binary_versioned").next())
        .expect("text and binary load paths must remain separate");

    assert!(
        text_load.contains("serde_json::from_str::<T>(envelope.payload.get())"),
        "current envelopes must decode the borrowed payload directly into T"
    );
    assert!(
        text_load.contains("load_legacy_text_versioned"),
        "only the legacy path may materialize a Value for migration"
    );
}

#[test]
fn current_envelope_validates_the_complete_migration_chain_before_typed_decode() {
    let bytes = serde_json::to_vec(&json!({
        "$zircon": {
            "header": {
                "schema_id": "zircon.tests.broken-chain",
                "schema_version": 2
            },
            "payload": {
                "value": null
            }
        }
    }))
    .unwrap();

    let error = load_versioned::<BrokenChainDocument>(&bytes, Format::Text)
        .expect_err("current payloads must not bypass migration-chain validation");

    assert!(matches!(
        error,
        LoadError::Migration(MigrateError::MissingStep {
            from_version: 1,
            target_version: 2,
            ..
        })
    ));
}

#[test]
fn envelope_for_a_different_schema_is_rejected() {
    let bytes = serde_json::to_vec(&json!({
        "$zircon": {
            "header": {
                "schema_id": "zircon.tests.some-other-document",
                "schema_version": 2
            },
            "payload": {
                "label": "wrong",
                "count": 7
            }
        }
    }))
    .unwrap();

    let error = load_versioned::<FixtureDocument>(&bytes, Format::Text).unwrap_err();

    assert!(matches!(
        error,
        LoadError::SchemaMismatch { expected, found }
            if expected == "zircon.tests.fixture-document"
                && found == "zircon.tests.some-other-document"
    ));
}
