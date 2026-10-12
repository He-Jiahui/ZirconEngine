//! 资产子系统的契约测试入口；各子模块覆盖从源文件到运行时句柄的不同边界。
#[path = "artifact.rs"]
mod artifact;
#[path = "assets/mod.rs"]
mod assets;
#[path = "facade.rs"]
mod facade;
#[path = "formats/mod.rs"]
mod formats;
#[path = "load/mod.rs"]
mod load;
#[path = "migration/mod.rs"]
mod migration;
#[path = "module_capability_truth.rs"]
mod module_capability_truth;
#[path = "module_lifecycle.rs"]
mod module_lifecycle;
#[path = "pack.rs"]
mod pack;
#[path = "pipeline/mod.rs"]
mod pipeline;
#[path = "project/mod.rs"]
pub(crate) mod project;
#[path = "registry.rs"]
mod registry;
#[path = "registry_index/mod.rs"]
mod registry_index;
#[path = "support.rs"]
pub(crate) mod support;
#[path = "virtual_geometry_cook.rs"]
mod virtual_geometry_cook;
#[path = "watcher.rs"]
mod watcher;
