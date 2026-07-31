use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::{
    Format, LoadError, MigrateError, MigrationChain, MigrationStep, SchemaId, VersionedSchema,
    load_versioned,
};

#[derive(Debug, Serialize, Deserialize)]
struct DuplicateStepDocument {}

impl VersionedSchema for DuplicateStepDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.duplicate-step");
    const VERSION: u32 = 1;

    fn migrations() -> &'static MigrationChain<Self> {
        &DUPLICATE_CHAIN
    }
}

static DUPLICATE_CHAIN: MigrationChain<DuplicateStepDocument> = MigrationChain::new(&[
    MigrationStep::new(0, identity_migration),
    MigrationStep::new(0, identity_migration),
]);

fn identity_migration(value: Value) -> Result<Value, MigrateError> {
    Ok(value)
}

#[test]
fn duplicate_source_version_is_rejected_instead_of_using_table_order() {
    let error = load_versioned::<DuplicateStepDocument>(b"{}", Format::Text)
        .expect_err("duplicate migration steps are ambiguous");

    assert!(matches!(
        error,
        LoadError::Migration(MigrateError::DuplicateStep {
            from_version: 0,
            ..
        })
    ));
}

#[derive(Debug, Serialize, Deserialize)]
struct OutOfOrderDocument {}

impl VersionedSchema for OutOfOrderDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.out-of-order");
    const VERSION: u32 = 2;

    fn migrations() -> &'static MigrationChain<Self> {
        &OUT_OF_ORDER_CHAIN
    }
}

static OUT_OF_ORDER_CHAIN: MigrationChain<OutOfOrderDocument> = MigrationChain::new(&[
    MigrationStep::new(1, identity_migration),
    MigrationStep::new(0, identity_migration),
]);

#[test]
fn out_of_order_table_is_rejected_instead_of_searched() {
    let error = load_versioned::<OutOfOrderDocument>(b"{}", Format::Text).unwrap_err();

    assert!(matches!(
        error,
        LoadError::Migration(MigrateError::OutOfOrderStep {
            expected_from_version: 0,
            found_from_version: 1,
            ..
        })
    ));
}

#[derive(Debug, Serialize, Deserialize)]
struct ExtraStepDocument {}

impl VersionedSchema for ExtraStepDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.extra-step");
    const VERSION: u32 = 1;

    fn migrations() -> &'static MigrationChain<Self> {
        &EXTRA_STEP_CHAIN
    }
}

static EXTRA_STEP_CHAIN: MigrationChain<ExtraStepDocument> = MigrationChain::new(&[
    MigrationStep::new(0, identity_migration),
    MigrationStep::new(1, identity_migration),
]);

#[test]
fn extra_step_past_current_version_is_rejected() {
    let error = load_versioned::<ExtraStepDocument>(b"{}", Format::Text).unwrap_err();

    assert!(matches!(
        error,
        LoadError::Migration(MigrateError::UnexpectedStep {
            from_version: 1,
            target_version: 1,
            ..
        })
    ));
}

#[test]
fn current_version_payload_does_not_bypass_invalid_chain_validation() {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "$zircon": {
            "header": {
                "schema_id": "zircon.tests.duplicate-step",
                "schema_version": 1
            },
            "payload": {}
        }
    }))
    .unwrap();

    let error = load_versioned::<DuplicateStepDocument>(&bytes, Format::Text).unwrap_err();

    assert!(matches!(
        error,
        LoadError::Migration(MigrateError::DuplicateStep {
            from_version: 0,
            ..
        })
    ));
}

#[test]
fn value_domain_callers_use_the_same_complete_chain_validation() {
    let error = DUPLICATE_CHAIN
        .migrate_value(
            &DuplicateStepDocument::SCHEMA,
            Value::Object(Default::default()),
            1,
            DuplicateStepDocument::VERSION,
        )
        .expect_err("TOML adapters must not bypass complete-chain validation");

    assert!(matches!(
        error,
        MigrateError::DuplicateStep {
            from_version: 0,
            ..
        }
    ));
}
