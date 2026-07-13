use super::super::{load_versioned, Format};
use super::FixtureDocument;

#[test]
fn unwrapped_v0_payload_may_own_a_domain_field_named_payload() {
    let loaded = load_versioned::<FixtureDocument>(
        br#"{"name":"legacy","payload":"domain-owned"}"#,
        Format::Text,
    )
    .expect("payload without a header is an unwrapped v0 document");

    assert_eq!(loaded.value.label, "legacy");
    assert_eq!(loaded.value.count, 2);
    assert_eq!(loaded.migrated_from, Some(0));
}

#[test]
fn unwrapped_v0_payload_may_own_header_and_payload_domain_fields() {
    let loaded = load_versioned::<FixtureDocument>(
        br#"{"name":"legacy","header":{"domain":true},"payload":"domain-owned"}"#,
        Format::Text,
    )
    .expect("only the reserved magic key may identify an envelope");

    assert_eq!(loaded.value.label, "legacy");
    assert_eq!(loaded.value.count, 2);
    assert_eq!(loaded.migrated_from, Some(0));
}
