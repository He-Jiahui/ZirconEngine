//! GPU abstraction, render graph hooks, and draw submission.
//!
//! Conceptual counterpart: Godot `servers/rendering`; Fyrox `fyrox-graphics`.

/// Returns the workspace engine name.
pub fn engine_name() -> &'static str {
    "ZirconEngine"
}
