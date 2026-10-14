//! 锁定文本读取的错误分类：JSON 语法错误、已声明封套的结构错误和 payload 类型错误分别报告。
//! 现有用例另确认完整封套拒绝未知字段且不会回退为旧格式，不覆盖所有畸形或重复键排列。

use serde_json::json;

use super::super::{load_versioned, Format, LoadError};
use super::FixtureDocument;

/// 区分无效 JSON 与完整封套签名下缺少必需 payload 的结构错误。
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

/// 确认当前 schema 的 payload 解码错误保留 schema 标识与版本上下文。
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

/// 确认封套 header 不接受未定义字段。
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

/// 确认完整封套外出现额外字段时报告封套错误，而不是改按旧格式加载。
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
