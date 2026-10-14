//! 锁定未封套旧文本的兼容边界：业务字段可以与封套字段同名，仍按旧 schema 迁移。
//! 这些用例覆盖无完整保留封套的输入，不表示覆盖部分版本头或重复键策略。

use super::super::{load_versioned_legacy_schema_zero, Format};
use super::FixtureDocument;

/// 确认未封套对象中的 `payload` 仍是业务数据并进入 schema-zero 迁移。
#[test]
fn unwrapped_v0_payload_may_own_a_domain_field_named_payload() {
    let loaded = load_versioned_legacy_schema_zero::<FixtureDocument>(
        br#"{"name":"legacy","payload":"domain-owned"}"#,
        Format::Text,
    )
    .expect("payload without a header is an unwrapped v0 document");

    assert_eq!(loaded.value.label, "legacy");
    assert_eq!(loaded.value.count, 2);
    assert_eq!(loaded.migrated_from, Some(0));
}

/// 确认普通 `header` 与 `payload` 字段不会单独声明保留封套。
#[test]
fn unwrapped_v0_payload_may_own_header_and_payload_domain_fields() {
    let loaded = load_versioned_legacy_schema_zero::<FixtureDocument>(
        br#"{"name":"legacy","header":{"domain":true},"payload":"domain-owned"}"#,
        Format::Text,
    )
    .expect("only the reserved magic key may identify an envelope");

    assert_eq!(loaded.value.label, "legacy");
    assert_eq!(loaded.value.count, 2);
    assert_eq!(loaded.migrated_from, Some(0));
}

/// 确认 `$zircon` 只有非对象业务值时仍作为旧格式数据迁移。
#[test]
fn unwrapped_v0_payload_may_own_a_domain_field_named_zircon() {
    let loaded = load_versioned_legacy_schema_zero::<FixtureDocument>(
        br#"{"name":"legacy","$zircon":"domain-owned"}"#,
        Format::Text,
    )
    .expect("a non-envelope magic-shaped domain field is still v0 data");

    assert_eq!(loaded.value.label, "legacy");
    assert_eq!(loaded.value.count, 2);
    assert_eq!(loaded.migrated_from, Some(0));
}
