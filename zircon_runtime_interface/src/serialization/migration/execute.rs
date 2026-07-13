use serde_json::Value;

use super::{MigrateError, MigrationChain};
use crate::serialization::SchemaId;

impl<T> MigrationChain<T> {
    /// Migrates a value-domain payload after validating the schema's complete chain.
    ///
    /// Format adapters such as TOML must use this entry instead of executing migration
    /// function pointers themselves, so current-version inputs cannot bypass chain checks.
    pub fn migrate_value(
        &self,
        schema_id: &SchemaId,
        mut value: Value,
        from_version: u32,
        target_version: u32,
    ) -> Result<Value, MigrateError> {
        self.validate(schema_id, target_version)?;
        for version in from_version..target_version {
            let step = &self.steps[version as usize];
            value = (step.migrate)(value).map_err(|source| MigrateError::StepFailed {
                schema_id: schema_id.as_str().to_string(),
                from_version: version,
                source: Box::new(source),
            })?;
        }
        Ok(value)
    }
}
