//! Script VM integration, API exposure to gameplay code, and hot reload hooks.
//!
//! Conceptual counterpart: Godot `modules/gdscript` and related scripting layers.

/// Returns the workspace engine name.
pub fn engine_name() -> &'static str {
    "ZirconEngine"
}
