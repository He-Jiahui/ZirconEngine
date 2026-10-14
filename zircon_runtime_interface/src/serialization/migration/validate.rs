use super::{MigrateError, MigrationChain};
use crate::serialization::SchemaId;

impl<T> MigrationChain<T> {
    /// 校验步骤表是否恰好覆盖版本 0 到 target_version 的每条相邻版本边。
    ///
    /// 当前版本的读取也调用此检查；返回的首个结构错误通过 [MigrateError] 交给加载器。
    pub(in crate::serialization) fn validate(
        &self,
        schema_id: &SchemaId,
        target_version: u32,
    ) -> Result<(), MigrateError> {
        // 下标 i 必须恰好代表 i 到 i+1；即使读取当前版本也不能跳过链完整性校验。
        let expected_len = target_version as usize;
        // 先报告所需前缀中的首个重复或乱序项，再检查缺失或多余项，保持错误分类顺序稳定。
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
