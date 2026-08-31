use std::sync::atomic::{AtomicUsize, Ordering};

use serde::de::IgnoredAny;
use serde::Deserialize;
use zircon_runtime_interface::{ZrByteSlice, ZrRuntimePayloadLimitV1};

use super::*;

static BUSINESS_DESERIALIZE_CALLS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct BusinessDeserializeProbe;

impl<'de> Deserialize<'de> for BusinessDeserializeProbe {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        BUSINESS_DESERIALIZE_CALLS.fetch_add(1, Ordering::SeqCst);
        IgnoredAny::deserialize(deserializer)?;
        Ok(Self)
    }
}

fn depth_limit(max_nesting_depth: usize) -> ZrRuntimePayloadLimitV1 {
    ZrRuntimePayloadLimitV1 {
        max_encoded_bytes: 4 * 1024,
        max_items: 4 * 1024,
        max_nesting_depth,
        max_processing_time_micros: 100_000,
        allow_empty: false,
    }
}

#[test]
fn decode_reports_nesting_as_a_payload_limit() {
    let encoded = format!("{}0{}", "[".repeat(5), "]".repeat(5));
    let input = ZrByteSlice {
        data: encoded.as_ptr(),
        len: encoded.len(),
    };

    let error =
        unsafe { decode::<serde_json::Value>(input, depth_limit(4), json_value_item_count) }
            .expect_err(
                "inbound JSON above the declared nesting depth must be rejected as a limit",
            );

    assert_eq!(
        error,
        BoundedJsonError::NestingDepth {
            observed: 5,
            limit: 4
        }
    );
    assert!(error.is_limit_exceeded());
}

#[test]
fn encode_reports_nesting_as_a_payload_limit() {
    let value: serde_json::Value =
        serde_json::from_str(&format!("{}0{}", "[".repeat(5), "]".repeat(5))).unwrap();

    let error = encode(&value, depth_limit(4), || json_value_item_count(&value))
        .expect_err("outbound JSON above the declared nesting depth must be rejected");

    assert_eq!(
        error,
        BoundedJsonError::NestingDepth {
            observed: 5,
            limit: 4
        }
    );
    assert!(error.is_limit_exceeded());
}

#[test]
fn validate_reports_limits_without_materializing_encoded_bytes() {
    let value = serde_json::json!({"payload": "too large"});
    let limit = ZrRuntimePayloadLimitV1 {
        max_encoded_bytes: 8,
        ..depth_limit(8)
    };

    let error = validate(&value, limit, || 1)
        .expect_err("counting validation must reject the oversized payload");

    assert!(matches!(
        error,
        BoundedJsonError::EncodedBytes { limit: 8, .. }
    ));
}

#[test]
fn nesting_tracker_ignores_delimiters_inside_split_escaped_strings() {
    let mut tracker = JsonNestingTracker::default();
    tracker.inspect(br#"{"value":"\"#, 1).unwrap();
    tracker.inspect(br#""[{}]"}"#, 1).unwrap();
    assert_eq!(tracker.depth, 0);
}

#[test]
fn decode_applies_the_item_limit_to_business_items_not_json_nodes() {
    BUSINESS_DESERIALIZE_CALLS.store(0, Ordering::SeqCst);
    let encoded = br#"[0,1,2]"#;
    let input = ZrByteSlice {
        data: encoded.as_ptr(),
        len: encoded.len(),
    };
    let limit = ZrRuntimePayloadLimitV1 {
        max_items: 2,
        ..depth_limit(8)
    };

    unsafe { decode::<BusinessDeserializeProbe>(input, limit, |_| 1) }
        .expect("one business item must fit even when its JSON representation has more nodes");

    assert_eq!(BUSINESS_DESERIALIZE_CALLS.load(Ordering::SeqCst), 1);
}

#[test]
fn bounded_json_facade_keeps_policy_stages_in_child_owners() {
    let facade = include_str!("../bounded_json.rs");
    let deadline = include_str!("deadline.rs");
    let error = include_str!("error.rs");
    let preflight = include_str!("preflight.rs");
    let writer = include_str!("writer.rs");

    for module in [
        "mod deadline;",
        "mod error;",
        "mod preflight;",
        "mod writer;",
    ] {
        assert!(facade.contains(module));
    }
    assert!(facade.contains("pub(super) unsafe fn decode<T>("));
    assert!(facade.contains("pub(super) fn encode<T: Serialize + ?Sized>("));
    assert!(facade.contains("pub(super) fn validate<T: Serialize + ?Sized>("));
    assert!(!facade.contains("struct "));
    assert!(deadline.contains("pub(super) struct ProcessingDeadline"));
    assert!(deadline.contains("pub(super) struct DeadlineReader"));
    assert!(error.contains("enum BoundedJsonError"));
    assert!(preflight.contains("pub(super) fn preflight_json_graph("));
    assert!(preflight.contains("struct JsonItemCounter"));
    assert!(writer.contains("struct BoundedJsonCountingWriter"));
    assert!(writer.contains("struct BoundedJsonWriter"));
    assert!(writer.contains("struct JsonNestingTracker"));

    for (path, source) in [
        ("bounded_json.rs", facade),
        ("bounded_json/deadline.rs", deadline),
        ("bounded_json/error.rs", error),
        ("bounded_json/preflight.rs", preflight),
        ("bounded_json/writer.rs", writer),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 350,
            "{path} should stay below the production owner budget; got {line_count} lines"
        );
    }
}
