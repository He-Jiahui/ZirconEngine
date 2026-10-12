//! 构建所有权的公开边界。
//! 调用者经产品构建模块生成草案，经收据模块发行或核验；这两个步骤各自有独立的信任门槛。

#[path = "product_build.rs"]
pub mod product_build;
#[path = "receipt/mod.rs"]
pub mod receipt;
