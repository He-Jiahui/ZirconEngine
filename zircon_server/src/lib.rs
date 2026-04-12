//! High-level server-style facades over graphics, physics, audio, and display.
//!
//! Conceptual counterpart: Godot `servers/` (RenderingServer, PhysicsServer, AudioServer, …).

/// Returns the workspace engine name.
pub fn engine_name() -> &'static str {
    "ZirconEngine"
}
