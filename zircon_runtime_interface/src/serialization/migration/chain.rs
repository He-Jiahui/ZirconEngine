use std::marker::PhantomData;

use super::MigrationStep;

/// Ordered migration steps for one schema type.
///
/// 该类型将静态步骤表与载荷类型 T 在类型层面关联，但不保存 T 的实例。表的连续性不在构造时
/// 检查；加载器和迁移入口会按目标版本验证完整表，避免当前版本读取绕过 schema 不变量。
///
/// # 使用时机与生命周期
/// 通常由 [VersionedSchema::migrations](crate::serialization::VersionedSchema::migrations)
/// 返回进程内有效的静态实例；其中步骤切片本身具有 static 生命周期。
pub struct MigrationChain<T> {
    pub(super) steps: &'static [MigrationStep],
    marker: PhantomData<fn() -> T>,
}

impl<T> MigrationChain<T> {
    /// 将静态步骤切片绑定到此链，不会在构造时验证或重排表项。
    ///
    /// # 参数
    /// steps 按来源版本声明迁移回调。作为
    /// [VersionedSchema](crate::serialization::VersionedSchema) 的迁移表时，应覆盖从版本 0
    /// 到该类型当前版本的每个相邻版本边；读取或迁移入口再按实际目标版本验证此约束。
    ///
    /// # 生命周期
    /// 切片必须在整个进程内有效；本构造函数按引用保存它，不复制步骤。
    ///
    /// # 参考
    /// [MigrationStep]、[VersionedSchema::migrations](crate::serialization::VersionedSchema::migrations)。
    pub const fn new(steps: &'static [MigrationStep]) -> Self {
        Self {
            steps,
            marker: PhantomData,
        }
    }
}
