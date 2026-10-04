//! 资产子系统的契约测试入口；各子模块覆盖从源文件到运行时句柄的不同边界。
mod artifact;
mod assets;
mod facade;
mod formats;
mod load;
mod migration;
mod module_capability_truth;
mod module_lifecycle;
mod pack;
mod pipeline;
pub(crate) mod project;
mod registry;
mod registry_index;
pub(crate) mod support;
mod virtual_geometry_cook;
mod watcher;
