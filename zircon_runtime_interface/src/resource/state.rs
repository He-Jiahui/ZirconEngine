use serde::{Deserialize, Serialize};

/// 注册表对外发布的导入与重载阶段；失败后的恢复须经资源管理器的 Reloading 边界。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 显式导入发布可从 Error 直接恢复到 Ready；其它入口仍按资源管理器的操作类型验证迁移。
///
/// # 阶段与恢复
///
/// Pending 表示尚未完成就绪发布，Ready 表示记录已处于就绪阶段，Reloading 和 Error 表达重载及失败阶段。普通就绪更新不能无条件跳过 Error；显式导入入口携带恢复授权。
///
/// # 载荷与时间边界
///
/// 记录阶段不等同于持有载荷租约。重载失败时，读取端仍可能取得最后有效载荷；枚举标签本身不执行加载、迁移或版本更新。
///
/// # 参考
///
/// [`crate::resource::ResourceRecord::with_state`]、[`crate::resource::ResourceHandle`]、[`crate::resource::ResourceRecord::failure_reason`]。
pub enum ResourceState {
    Pending,
    Ready,
    Error,
    Reloading,
}
