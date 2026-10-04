//! Cargo 工具客户端库的两条公开路径。
//! CLI 与集成测试共享构建收据契约和插件工具契约；入口解析留在主程序。

pub mod build;
pub mod plugin;
