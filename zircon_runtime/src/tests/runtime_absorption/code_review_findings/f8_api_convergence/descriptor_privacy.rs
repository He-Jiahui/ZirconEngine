//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "descriptor_privacy/constructor_retirement.rs"]
mod constructor_retirement;
#[path = "descriptor_privacy/private_fields.rs"]
mod private_fields;
#[path = "descriptor_privacy/status_mirrors.rs"]
mod status_mirrors;
