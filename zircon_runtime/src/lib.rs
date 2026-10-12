//! Runtime absorption layer for the built-in high-level engine subsystems.

#![cfg_attr(all(test, windows), recursion_limit = "256")]

extern crate self as zircon_runtime;

#[cfg(all(feature = "native-host", target_arch = "wasm32"))]
compile_error!("native-host requires a native target; use --no-default-features --features core-min for the portable Wasm core");

#[cfg(feature = "dev-dynamic-linking")]
#[allow(unused_imports, clippy::single_component_path_imports)]
use zr_dev_deps_dylib as _;

pub mod core;
#[cfg(feature = "diagnostic-log")]
pub mod diagnostic_log;
#[cfg(feature = "dynamic-api")]
pub mod dynamic_api;
pub mod engine_module;
pub mod prelude;
mod runtime_diagnostics;

#[cfg(feature = "animation")]
pub mod animation;
#[cfg(feature = "native-host")]
pub mod asset;
#[cfg(feature = "native-host")]
pub mod scene;
#[cfg(feature = "text")]
pub mod text;
#[cfg(feature = "ui")]
pub mod ui;

pub use crate::core::resource;

#[cfg(feature = "graphics")]
pub mod graphics;
#[cfg(feature = "graphics")]
pub mod render_graph;
#[cfg(feature = "graphics")]
pub mod rhi;

pub mod builtin;
#[cfg(feature = "native-host")]
pub mod foundation;
#[cfg(feature = "native-host")]
pub mod input;
#[cfg(feature = "navigation")]
pub mod navigation;
#[cfg(feature = "native-host")]
pub mod operation;
#[cfg(feature = "native-host")]
pub mod platform;
#[cfg(feature = "native-host")]
pub mod plugin;
#[cfg(feature = "script")]
pub mod script;

pub use zircon_runtime_reflection_macros::{
    zircon_host_function, zircon_host_module, ZirconScriptType,
};
#[cfg(all(test, feature = "native-host"))]
mod tests;
