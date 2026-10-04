//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "f8_api_convergence/descriptor_builder.rs"]
mod descriptor_builder;
#[path = "f8_api_convergence/descriptor_privacy.rs"]
mod descriptor_privacy;
#[path = "f8_api_convergence/texture_import_settings.rs"]
mod texture_import_settings;
