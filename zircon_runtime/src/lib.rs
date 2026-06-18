//! Runtime absorption layer for the built-in high-level engine subsystems.

extern crate self as zircon_runtime;

pub mod core;
pub mod diagnostic_log;
pub mod dynamic_api;
pub mod engine_module;
pub mod prelude;

// `ui` must be declared before `asset` (asset types reference UI template loaders).
pub mod animation;
pub mod asset;
pub mod scene;
pub mod ui;

pub use crate::core::resource;

pub mod graphics;
pub mod render_graph;
pub mod rhi;
mod rhi_wgpu;

pub mod builtin;
pub mod foundation;
pub mod input;
pub mod navigation;
pub mod platform;
pub mod plugin;
pub mod script;

pub use zircon_runtime_reflection_macros::{
    zircon_host_function, zircon_host_module, ZirconScriptType,
};
#[cfg(test)]
mod tests;
