//! Dynamic modules, GDExtension-style hooks, and optional engine feature packs.
//!
//! Conceptual counterpart: Godot `modules/`.

/// Returns the workspace engine name.
pub fn engine_name() -> &'static str {
    "ZirconEngine"
}
