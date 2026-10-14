//! Runtime absorption layer for the built-in high-level engine subsystems.

#![cfg_attr(all(test, windows), recursion_limit = "256")]

extern crate self as zircon_runtime;

#[cfg(all(feature = "native-host", target_arch = "wasm32"))]
compile_error!("native-host requires a native target; use --no-default-features --features core-min for the portable Wasm core");

#[cfg(feature = "dev-dynamic-linking")]
#[allow(unused_imports, clippy::single_component_path_imports)]
use zr_dev_deps_dylib as _;

#[path = "core/mod.rs"]
pub mod core;
#[cfg(feature = "diagnostic-log")]
#[path = "diagnostic_log/mod.rs"]
pub mod diagnostic_log;
#[cfg(feature = "dynamic-api")]
#[path = "dynamic_api/mod.rs"]
pub mod dynamic_api;
#[path = "engine_module/mod.rs"]
pub mod engine_module;
#[path = "prelude.rs"]
pub mod prelude;
#[path = "runtime_diagnostics/mod.rs"]
mod runtime_diagnostics;

#[cfg(feature = "animation")]
#[path = "animation/mod.rs"]
pub mod animation;
#[cfg(feature = "native-host")]
#[path = "asset/mod.rs"]
pub mod asset;
#[cfg(feature = "native-host")]
#[path = "scene/mod.rs"]
pub mod scene;
#[cfg(feature = "text")]
#[path = "text/mod.rs"]
pub mod text;
#[cfg(feature = "ui")]
#[path = "ui/mod.rs"]
pub mod ui;

pub use crate::core::resource;

/// Core rendering, scene rasterization, and host-agnostic GPU services.
#[cfg(feature = "graphics")]
#[path = "graphics/mod.rs"]
pub mod graphics;
#[cfg(feature = "graphics")]
#[path = "render_graph/mod.rs"]
pub mod render_graph;
#[cfg(feature = "graphics")]
#[path = "rhi.rs"]
pub mod rhi;

#[path = "builtin/mod.rs"]
pub mod builtin;
#[cfg(feature = "native-host")]
#[path = "foundation/mod.rs"]
pub mod foundation;
#[cfg(feature = "native-host")]
#[path = "input/mod.rs"]
pub mod input;
#[cfg(feature = "navigation")]
#[path = "navigation/mod.rs"]
pub mod navigation;
#[cfg(feature = "native-host")]
#[path = "operation/mod.rs"]
pub mod operation;
#[cfg(feature = "native-host")]
#[path = "platform/mod.rs"]
pub mod platform;
#[cfg(feature = "native-host")]
#[path = "plugin/mod.rs"]
pub mod plugin;
#[cfg(feature = "script")]
#[path = "script/mod.rs"]
pub mod script;

pub use zircon_runtime_reflection_macros::{
    zircon_host_function, zircon_host_module, ZirconScriptType,
};
#[cfg(all(test, feature = "native-host"))]
#[path = "tests/mod.rs"]
mod tests;
