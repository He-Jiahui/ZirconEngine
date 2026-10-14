use serde_json::Value;

use super::MigrateError;

/// 迁移回调接收并返回拥有所有权的 JSON 值；失败时返回可供迁移链保留的类型化原因。
///
/// [MigrationChain::migrate_value](super::MigrationChain::migrate_value) 按来源版本顺序调用
/// 这些函数指针，并将回调错误包装为 [MigrateError::StepFailed](super::MigrateError::StepFailed)。
pub type MigrationFn = fn(Value) -> Result<Value, MigrateError>;

/// One value-domain migration from `from_version` to its immediate successor.
#[derive(Clone, Copy)]
/// from_version 表示输入值所属版本，因此每个步骤只负责一个相邻版本边；它不会自行决定
/// 表中顺序或执行时机，这些由 [MigrationChain](super::MigrationChain) 的验证和迁移入口统一处理。
pub struct MigrationStep {
    pub(super) from_version: u32,
    pub(super) migrate: MigrationFn,
}

impl MigrationStep {
    /// 声明一个从来源版本到其直接后继版本的值域转换。
    ///
    /// # 参数
    /// from_version 是这条版本边的起点；例如 0 表示把版本 0 的值转换为版本 1。
    /// migrate 是接收拥有所有权的 JSON 值并返回转换结果或 [MigrateError] 的回调。
    ///
    /// # 返回值
    /// 返回仅保存来源版本和回调指针的步骤。此处不执行回调，也不验证该步骤在整条表中的位置。
    ///
    /// # 参考
    /// [MigrationChain::new](super::MigrationChain::new)。
    pub const fn new(from_version: u32, migrate: MigrationFn) -> Self {
        Self {
            from_version,
            migrate,
        }
    }
}
