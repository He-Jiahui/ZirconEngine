//! Project manager, viewport tooling, inspectors — editor-only code paths.
//!
//! Conceptual counterpart: Godot `editor/`.

/// Returns the workspace engine name.
pub fn engine_name() -> &'static str {
    "ZirconEngine"
}
