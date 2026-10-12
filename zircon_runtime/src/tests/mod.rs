//! 运行时整合回归的挂载入口；图形与界面分支遵循功能开关，共享夹具和领域测试在各子模块拥有，不在根入口拼装行为。
#[path = "camera_controller.rs"]
mod camera_controller;
#[path = "extensions/mod.rs"]
mod extensions;
#[path = "gizmos/mod.rs"]
mod gizmos;
#[cfg(feature = "graphics")]
#[path = "graphics_surface/mod.rs"]
mod graphics_surface;
#[path = "picking/mod.rs"]
mod picking;
#[path = "plugins/mod.rs"]
mod plugin_extensions;
#[path = "prelude.rs"]
mod prelude;
#[path = "contracts/mod.rs"]
mod runtime_absorption;
#[cfg(feature = "graphics")]
#[path = "runtime_diagnostics/mod.rs"]
mod runtime_diagnostics;
#[path = "scene_boundary/mod.rs"]
mod scene_boundary;
#[path = "state.rs"]
mod state;
#[path = "tasks.rs"]
mod tasks;
#[path = "time.rs"]
mod time;
#[cfg(feature = "ui")]
#[path = "ui_boundary/mod.rs"]
mod ui_boundary;
