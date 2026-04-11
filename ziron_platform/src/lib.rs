//! Windowing, surfaces, input, and target-specific shims for Windows / Linux / macOS / Android / iOS / WASM.
//!
//! Conceptual counterpart: Godot `platform/` + parts of `drivers/`.

/// Returns the workspace engine name.
pub fn engine_name() -> &'static str {
    "ZirconEngine"
}
