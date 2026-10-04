//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "asset_loaders/animation_binary.rs"]
mod animation_binary;
#[path = "asset_loaders/artifact_importer.rs"]
mod artifact_importer;
#[path = "asset_loaders/mesh_obj.rs"]
mod mesh_obj;
#[path = "asset_loaders/texture.rs"]
mod texture;
