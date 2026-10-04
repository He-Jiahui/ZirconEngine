//! 插件公开面、原生装载命名空间与生命周期回退保持分离。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "plugin_surface_lifecycle/inventory.rs"]
mod inventory;
#[path = "plugin_surface_lifecycle/lifecycle_fallback.rs"]
mod lifecycle_fallback;
#[path = "plugin_surface_lifecycle/mirror_docs.rs"]
mod mirror_docs;
#[path = "plugin_surface_lifecycle/native_loader_namespace.rs"]
mod native_loader_namespace;
#[path = "plugin_surface_lifecycle/split_layout.rs"]
mod split_layout;
#[path = "plugin_surface_lifecycle/support.rs"]
mod support;
