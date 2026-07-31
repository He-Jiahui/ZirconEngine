use super::{MigrateError, MigrationChain};
use crate::serialization::SchemaId;

impl<T> MigrationChain<T> {
    pub(in crate::serialization) fn validate(
        &self,
        schema_id: &SchemaId,
        target_version: u32,
    ) -> Result<(), MigrateError> {
        let expected_len = target_version as usize;
        for (index, step) in self.steps.iter().take(expected_len).enumerate() {
            let expected_from_version = index as u32;
            if step.from_version == expected_from_version {
                continue;
            }
            if step.from_version < expected_from_version {
                return Err(MigrateError::DuplicateStep {
                    schema_id: schema_id.as_str().to_string(),
                    from_version: step.from_version,
                });
            }
            return Err(MigrateError::OutOfOrderStep {
                schema_id: schema_id.as_str().to_string(),
                expected_from_version,
                found_from_version: step.from_version,
            });
        }

        if self.steps.len() < expected_len {
            return Err(MigrateError::MissingStep {
                schema_id: schema_id.as_str().to_string(),
                from_version: self.steps.len() as u32,
                target_version,
            });
        }
        if self.steps.len() > expected_len {
            let extra_from_version = self.steps[expected_len].from_version;
            if extra_from_version < target_version {
                return Err(MigrateError::DuplicateStep {
                    schema_id: schema_id.as_str().to_string(),
                    from_version: extra_from_version,
                });
            }
            return Err(MigrateError::UnexpectedStep {
                schema_id: schema_id.as_str().to_string(),
                from_version: extra_from_version,
                target_version,
            });
        }
        Ok(())
    }
}
