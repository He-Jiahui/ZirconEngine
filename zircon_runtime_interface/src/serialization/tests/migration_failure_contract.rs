//! 验证迁移回调失败经文本旧版加载后仍保留 schema、来源版本和原始错误来源。
//!
//! tests/mod.rs 通过路径模块注册本契约；本文件只覆盖错误传播，不重复表完整性用例。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::{
    load_versioned_legacy_schema_zero, Format, LoadError, MigrateError, MigrationChain,
    MigrationStep, SchemaId, VersionedSchema,
};

#[derive(Debug, Serialize, Deserialize)]
struct FailingStepDocument {}

impl VersionedSchema for FailingStepDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.failing-step");
    const VERSION: u32 = 1;

    fn migrations() -> &'static MigrationChain<Self> {
        &FAILING_CHAIN
    }
}

static FAILING_CHAIN: MigrationChain<FailingStepDocument> =
    MigrationChain::new(&[MigrationStep::new(0, fail_migration)]);

fn fail_migration(_value: Value) -> Result<Value, MigrateError> {
    Err(MigrateError::invalid_payload("fixture rejection"))
}

#[test]
fn fixture_migration_reports_count_overflow_through_the_loader() {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "$zircon": {
            "header": {
                "schema_id": "zircon.tests.fixture-document",
                "schema_version": 1
            },
            "payload": { "label": "maximum", "count": u64::MAX }
        }
    }))
    .unwrap();

    let error =
        super::super::load_versioned::<super::FixtureDocument>(&bytes, Format::Text).unwrap_err();
    assert!(matches!(
        error,
        LoadError::Migration(MigrateError::StepFailed {
            from_version: 1,
            source,
            ..
        }) if matches!(*source, MigrateError::InvalidPayload(ref message) if message == "fixture v1 count overflow")
    ));
}

#[test]
fn fixture_migration_preserves_the_largest_representable_document_count() {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "$zircon": {
            "header": {
                "schema_id": "zircon.tests.fixture-document",
                "schema_version": 1
            },
            "payload": { "label": "maximum", "count": u32::MAX - 1 }
        }
    }))
    .unwrap();

    let document =
        super::super::load_versioned::<super::FixtureDocument>(&bytes, Format::Text).unwrap();
    assert_eq!(document.value.count, u32::MAX);
}

#[test]
fn migration_step_failure_preserves_schema_version_and_source() {
    let error =
        load_versioned_legacy_schema_zero::<FailingStepDocument>(b"{}", Format::Text).unwrap_err();

    assert!(matches!(
        error,
        LoadError::Migration(MigrateError::StepFailed {
            schema_id,
            from_version: 0,
            source,
        }) if schema_id == "zircon.tests.failing-step"
            && matches!(*source, MigrateError::InvalidPayload(ref message) if message == "fixture rejection")
    ));
}
