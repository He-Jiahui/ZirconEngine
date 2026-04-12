//! Input actions, rebinding, and aggregation over keyboard, mouse, gamepad, and touch.
//!
//! Conceptual counterpart: Godot `Input` / `InputMap` (above OS-specific capture in platform).

/// Returns the workspace engine name.
pub fn engine_name() -> &'static str {
    "ZirconEngine"
}
