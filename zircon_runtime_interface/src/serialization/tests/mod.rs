mod binary_contract;
mod binary_malformed_contract;
mod legacy_detection;
mod load_contract;
mod malformed_contract;
mod migration_contract;
mod migration_failure_contract;
mod schema_id_contract;
mod write_contract;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{MigrateError, MigrationChain, MigrationStep, SchemaId, VersionedSchema};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct FixtureDocument {
    label: String,
    count: u32,
}

impl VersionedSchema for FixtureDocument {
    const SCHEMA: SchemaId = SchemaId::new("zircon.tests.fixture-document");
    const VERSION: u32 = 2;

    fn migrations() -> &'static MigrationChain<Self> {
        &FIXTURE_MIGRATIONS
    }
}

static FIXTURE_MIGRATIONS: MigrationChain<FixtureDocument> = MigrationChain::new(&[
    MigrationStep::new(0, migrate_fixture_v0_to_v1),
    MigrationStep::new(1, migrate_fixture_v1_to_v2),
]);

fn migrate_fixture_v0_to_v1(mut value: Value) -> Result<Value, MigrateError> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| MigrateError::invalid_payload("fixture v0 must be an object"))?;
    let label = object
        .remove("name")
        .ok_or_else(|| MigrateError::invalid_payload("fixture v0 is missing name"))?;
    object.insert("label".to_string(), label);
    object.insert("count".to_string(), Value::from(1));
    Ok(value)
}

fn migrate_fixture_v1_to_v2(mut value: Value) -> Result<Value, MigrateError> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| MigrateError::invalid_payload("fixture v1 must be an object"))?;
    let count = object
        .get("count")
        .and_then(Value::as_u64)
        .ok_or_else(|| MigrateError::invalid_payload("fixture v1 count must be unsigned"))?;
    object.insert("count".to_string(), Value::from(count + 1));
    Ok(value)
}
