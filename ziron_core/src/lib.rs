//! Core types, ECS integration points, and engine lifecycle.
//!
//! Conceptual counterpart: Godot `core/` + scene roots; Fyrox `fyrox-core`.

/// Returns the workspace engine name.
pub fn engine_name() -> &'static str {
    "ZirconEngine"
}
