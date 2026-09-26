use serde_json::Value;

use super::{MigrateError, MigrationChain};
use crate::serialization::SchemaId;

impl<T> MigrationChain<T> {
    // TODO: [CR-SERIALIZATION-0001] 确认公开迁移入口是否应自行拒绝源版本高于目标版本；现有调用方先校验，缺少直接调用该边界的契约测试；下一步补充边界测试并确定错误归属。
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
