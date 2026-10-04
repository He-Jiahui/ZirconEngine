//! 资产入口吸收后，注册、命名空间与查询边界保持由运行时资产模块拥有。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "asset_surface/facade_query.rs"]
mod facade_query;
#[path = "asset_surface/namespace_surface.rs"]
mod namespace_surface;
#[path = "asset_surface/registration.rs"]
mod registration;
#[path = "asset_surface/split_layout.rs"]
mod split_layout;
#[path = "asset_surface/support.rs"]
mod support;
