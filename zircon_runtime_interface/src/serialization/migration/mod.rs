//! 版本化载荷的前向迁移表与错误类型。
//!
//! [VersionedSchema](crate::serialization::VersionedSchema) 为持久化类型提供静态的
//! [MigrationChain]；链由有序 [MigrationStep] 组成，表结构或回调失败通过
//! [MigrateError] 返回。父级 [serialization](crate::serialization) 模块重导出这些类型，
//! 供加载器和格式适配器使用。
//!
//! 旧版本载荷进入 JSON 值域后，加载器先校验目标版本所需的完整迁移表，再按版本逐步调用静态
//! 函数指针。当前版本读取也会校验整条表。这里没有运行时注册或动态发现入口。

#[path = "chain.rs"]
mod chain;
#[path = "error.rs"]
mod error;
#[path = "execute.rs"]
mod execute;
#[path = "step.rs"]
mod step;
#[path = "validate.rs"]
mod validate;

pub use chain::MigrationChain;
pub use error::MigrateError;
pub use step::MigrationStep;
