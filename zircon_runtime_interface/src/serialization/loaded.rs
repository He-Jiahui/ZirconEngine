/// Successfully loaded payload plus the version it was migrated from, if any.
/// 版本化读取成功后的当前类型值及其迁移来源。
///
/// # 约束
/// 对于 loader 返回的实例，`value` 已通过 schema/版本检查、迁移（若需要）和目标类型解码。
/// 此公开字段容器本身不校验手工构造的值，也不替调用方验证更高层领域不变量。
/// 只有读取较旧版本并成功迁移时，loader 返回值的 `migrated_from` 才记录输入版本；当前版本
/// 读取以及没有发生升级的结果为 `None`。
///
/// # 参考
/// [`load_versioned`](super::load_versioned) 及明确允许 schema-zero 回退的
/// [`load_versioned_legacy_schema_zero`](super::load_versioned_legacy_schema_zero)。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Loaded<T> {
    /// 经验证并转换为当前 schema 类型的值。
    pub value: T,
    /// 若发生旧版本迁移，保存输入时的 schema 版本。
    pub migrated_from: Option<u32>,
}
