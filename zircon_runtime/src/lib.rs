//! Runtime absorption layer for the built-in high-level engine subsystems.

extern crate self as zircon_runtime;

pub mod core;
#[cfg(feature = "diagnostic-log")]
pub mod diagnostic_log;
#[cfg(feature = "dynamic-api")]
pub mod dynamic_api;
pub mod engine_module;
pub mod prelude;

#[cfg(feature = "animation")]
pub mod animation;
pub mod asset;
pub mod scene;
#[cfg(feature = "ui")]
pub mod ui;

pub use crate::core::resource;

#[cfg(feature = "graphics")]
pub mod graphics;
#[cfg(feature = "graphics")]
pub mod render_graph;
#[cfg(feature = "graphics")]
pub mod rhi;
#[cfg(feature = "graphics")]
mod rhi_wgpu;

pub mod builtin;
pub mod foundation;
pub mod input;
#[cfg(feature = "navigation")]
pub mod navigation;
pub mod platform;
pub mod plugin;
#[cfg(feature = "script")]
pub mod script;

pub use zircon_runtime_reflection_macros::{
    zircon_host_function, zircon_host_module, ZirconScriptType,
};
#[cfg(test)]
mod tests;
