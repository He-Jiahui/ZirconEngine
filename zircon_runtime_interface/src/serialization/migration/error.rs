use thiserror::Error;

/// Typed failure produced while validating or executing a migration chain.
///
/// 结构错误来自完整迁移表检查；[StepFailed](Self::StepFailed) 则为具体来源版本的回调失败附上
/// schema 上下文，并通过 error source 保留原始原因。
#[derive(Debug, Error)]
pub enum MigrateError {
    /// 回调无法把输入值解释为预期的旧版结构或完成转换。
    #[error("invalid migration payload: {0}")]
    InvalidPayload(String),
    /// 到达目标版本前缺少必需的相邻版本步骤。
    #[error(
        "schema {schema_id} has no migration step from version {from_version} toward {target_version}"
    )]
    MissingStep {
        /// 迁移表所属的 schema 标识。
        schema_id: String,
        /// 表中下一个应提供的来源版本。
        from_version: u32,
        /// 本次表验证要求覆盖到的目标版本。
        target_version: u32,
    },
    /// 迁移表为同一个来源版本提供了多个步骤。
    #[error("schema {schema_id} has multiple migration steps from version {from_version}")]
    DuplicateStep {
        /// 迁移表所属的 schema 标识。
        schema_id: String,
        /// 重复出现的来源版本。
        from_version: u32,
    },
    /// 当前表项来源版本大于该位置要求的版本，表项未按连续顺序排列。
    #[error(
        "schema {schema_id} migration table expected version {expected_from_version}, found {found_from_version}"
    )]
    OutOfOrderStep {
        /// 迁移表所属的 schema 标识。
        schema_id: String,
        /// 根据表下标应出现的来源版本。
        expected_from_version: u32,
        /// 实际表项声明的来源版本。
        found_from_version: u32,
    },
    /// 目标版本之后仍有迁移项，且该项不是目标范围内的重复步骤。
    #[error(
        "schema {schema_id} migration table has unexpected step {from_version} at target version {target_version}"
    )]
    UnexpectedStep {
        /// 迁移表所属的 schema 标识。
        schema_id: String,
        /// 超出目标版本范围的步骤来源版本。
        from_version: u32,
        /// 本次表验证的目标版本。
        target_version: u32,
    },
    /// 来源版本对应的迁移回调失败；source 保存回调返回的原始迁移错误。
    #[error("schema {schema_id} migration step from version {from_version} failed: {source}")]
    StepFailed {
        /// 迁移表所属的 schema 标识。
        schema_id: String,
        /// 失败回调处理的来源版本。
        from_version: u32,
        /// 原始回调错误，作为标准错误来源继续暴露。
        #[source]
        source: Box<MigrateError>,
    },
}

impl MigrateError {
    /// 构造由迁移回调返回的值域拒绝错误。
    ///
    /// # 参数
    /// message 描述输入值为何不符合该回调要求；接受可转换为 String 的消息。
    ///
    /// # 返回值
    /// 返回 [MigrateError::InvalidPayload](Self::InvalidPayload)，供回调或调用者继续包装。
    pub fn invalid_payload(message: impl Into<String>) -> Self {
        Self::InvalidPayload(message.into())
    }
}
