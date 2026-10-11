//! Stable module identity shared by Foundation consumers and runtime assembly.

/// Canonical runtime module identity for foundation services.
///
/// 用于模块注册、依赖声明和内建模块映射的规范名称。
///
/// # 约束与参考
///
/// ConfigManager 的规范服务名使用同一前缀；修改时须同步
/// [crate::core::manager::CONFIG_MANAGER_NAME]。
pub const FOUNDATION_MODULE_NAME: &str = "FoundationModule";
