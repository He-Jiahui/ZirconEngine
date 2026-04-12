//! Transports, RPC or message framing, and game-level net sync.
//!
//! Conceptual counterpart: Godot `modules/multiplayer` / high-level networking.

/// Returns the workspace engine name.
pub fn engine_name() -> &'static str {
    "ZirconEngine"
}
