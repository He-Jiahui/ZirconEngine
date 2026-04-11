//! Engine initialization, frame loop glue, and run-to-ship entry points.
//!
//! Conceptual counterpart: Godot `main/`.

/// Returns the workspace engine name.
pub fn engine_name() -> &'static str {
    "ZirconEngine"
}
