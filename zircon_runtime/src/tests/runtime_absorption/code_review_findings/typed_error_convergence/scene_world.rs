//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "scene_world/dynamic_components.rs"]
mod dynamic_components;
#[path = "scene_world/fixed_mutation.rs"]
mod fixed_mutation;
#[path = "scene_world/property_access.rs"]
mod property_access;
#[path = "scene_world/typed_mutation_surface.rs"]
mod typed_mutation_surface;
