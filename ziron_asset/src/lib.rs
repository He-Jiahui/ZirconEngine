//! Asset identifiers, virtual file system, and load queues.
//!
//! Conceptual counterpart: Godot `core/io` resource loading; Fyrox `fyrox-resource`.

/// Returns the workspace engine name.
pub fn engine_name() -> &'static str {
    "ZirconEngine"
}
