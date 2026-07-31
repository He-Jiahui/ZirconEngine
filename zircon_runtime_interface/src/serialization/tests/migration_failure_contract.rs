use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::{
    Format, LoadError, MigrateError, MigrationChain, MigrationStep, SchemaId, VersionedSchema,
    load_versioned,
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
fn migration_step_failure_preserves_schema_version_and_source() {
    let error = load_versioned::<FailingStepDocument>(b"{}", Format::Text).unwrap_err();

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
