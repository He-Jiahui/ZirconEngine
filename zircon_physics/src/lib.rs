//! Rigid bodies, queries, and simulation stepping.
//!
//! Conceptual counterpart: Godot `servers/physics`; Fyrox physics integration.
//!
//! Enable **`feature = "jolt"`** for [`rolt`] / Jolt. Requires a native toolchain (CMake, libclang).

/// `true` when crate was built with **`jolt`** (Jolt bindings available).
pub const JOLT_ENABLED: bool = cfg!(feature = "jolt");

/// Returns the workspace engine name.
pub fn engine_name() -> &'static str {
    "ZirconEngine"
}
