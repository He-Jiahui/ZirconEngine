//! 运行时整合回归的挂载入口；图形与界面分支遵循功能开关，共享夹具和领域测试在各子模块拥有，不在根入口拼装行为。
mod camera_controller;
mod extensions;
mod gizmos;
#[cfg(feature = "graphics")]
mod graphics_surface;
mod picking;
mod plugin_extensions;
mod prelude;
mod runtime_absorption;
#[cfg(feature = "graphics")]
mod runtime_diagnostics;
mod scene_boundary;
mod state;
mod tasks;
mod time;
#[cfg(feature = "ui")]
mod ui_boundary;
