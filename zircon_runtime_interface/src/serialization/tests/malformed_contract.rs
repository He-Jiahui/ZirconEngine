use serde_json::json;

use super::super::{Format, LoadError, load_versioned};
use super::FixtureDocument;

#[test]
fn malformed_json_and_malformed_envelope_have_distinct_errors() {
    let malformed_text = load_versioned::<FixtureDocument>(b"{", Format::Text).unwrap_err();
    assert!(matches!(malformed_text, LoadError::MalformedText { .. }));

    let missing_payload = serde_json::to_vec(&json!({
        "$zircon": {
            "header": {
                "schema_id": "zircon.tests.fixture-document",
                "schema_version": 2
            }
        }
    }))
    .unwrap();
    let malformed_envelope =
        load_versioned::<FixtureDocument>(&missing_payload, Format::Text).unwrap_err();
    assert!(matches!(
        malformed_envelope,
        LoadError::InvalidEnvelope { .. }
    ));
}

#[test]
fn current_payload_decode_failure_reports_schema_and_version() {
    let bytes = serde_json::to_vec(&json!({
        "$zircon": {
            "header": {
                "schema_id": "zircon.tests.fixture-document",
                "schema_version": 2
            },
            "payload": {
                "label": "bad",
                "count": "not-a-number"
            }
        }
    }))
    .unwrap();

    let error = load_versioned::<FixtureDocument>(&bytes, Format::Text).unwrap_err();

    assert!(matches!(
        error,
        LoadError::PayloadDecode {
            schema_id,
            schema_version: 2,
            ..
        } if schema_id == "zircon.tests.fixture-document"
    ));
}

#[test]
fn envelope_header_rejects_unknown_fields() {
    let bytes = serde_json::to_vec(&json!({
        "$zircon": {
            "header": {
                "schema_id": "zircon.tests.fixture-document",
                "schema_version": 2,
                "legacy_hint": "must-not-be-dropped"
            },
            "payload": {
                "label": "current",
                "count": 7
            }
        }
    }))
    .unwrap();

    let error = load_versioned::<FixtureDocument>(&bytes, Format::Text).unwrap_err();

    assert!(matches!(error, LoadError::InvalidEnvelope { .. }));
}

#[test]
fn envelope_rejects_outer_fields_without_reclassifying_it_as_legacy() {
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
        },
        "domain_owned": true
    }))
    .unwrap();

    let error = load_versioned::<FixtureDocument>(&bytes, Format::Text).unwrap_err();

    assert!(matches!(error, LoadError::InvalidEnvelope { .. }));
}
