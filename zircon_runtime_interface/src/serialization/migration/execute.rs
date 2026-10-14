use serde_json::Value;

use super::{MigrateError, MigrationChain};
use crate::serialization::SchemaId;

impl<T> MigrationChain<T> {
    // TODO: [CR-SERIALIZATION-0001] 确认公开迁移入口是否应自行拒绝源版本高于目标版本；现有调用方先校验，缺少直接调用该边界的契约测试；下一步补充边界测试并确定错误归属。
    /// Migrates a value-domain payload after validating the schema's complete chain.
    ///
    /// Format adapters such as TOML must use this entry instead of executing migration
    /// function pointers themselves, so current-version inputs cannot bypass chain checks.
    ///
    /// # 使用时机与生命周期
    /// 序列化加载器的旧版路径在来源版本检查后，通过 load_migrated_value 调用此入口。每个回调
    /// 消费上一阶段拥有的值并产生下一阶段的值；本方法返回最终 JSON 值域结果，不负责构造目标 Rust 类型。
    ///
    /// # 参数
    /// schema_id 用作迁移错误上下文；直接调用者应传入与此链对应的标识，本方法不将其与类型 T 的
    /// schema 常量比较。value 是待转换的拥有所有权的 JSON 值。from_version 是该值的来源版本，
    /// target_version 是迁移表需覆盖到的目标版本。
    ///
    /// # 返回值
    /// 成功时返回逐步迁移后的 JSON 值；具体载荷类型的反序列化由加载器在此调用之后完成。
    ///
    /// # 错误
    /// 目标版本对应的链不完整、重复、乱序或含多余步骤时返回相应 [MigrateError]；回调失败时以
    /// [MigrateError::StepFailed] 包装失败步骤和原始错误来源。
    ///
    /// # 约束
    /// 此入口校验目标版本的完整链，但不执行加载器的来源版本合法性检查；加载器会在调用前检查来源
    /// 版本。直接传入大于目标版本的来源值时，当前前向范围为空，验证目标链后会原样返回输入值；
    /// 这一直接调用边界仍由本实现上方的既有 TODO 跟踪。
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
