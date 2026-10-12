//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "../../../../errors/loader/manifest_sources/collection_candidate.rs"]
mod collection_candidate;
#[path = "../../../../errors/loader/manifest_sources/compat_registration.rs"]
mod compat_registration;
