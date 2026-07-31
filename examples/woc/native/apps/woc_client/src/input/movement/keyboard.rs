use woc_protocol::MovementInputFlags;

use crate::Keybinds;

/// Resolves physical held keys through the persisted binding table. Attack Move
/// suppresses only an exactly equal action binding, matching the target's
/// physical-key rule rather than broadly reserving a letter key.
pub fn keyboard_movement_flags(
    keybinds: &Keybinds,
    held_codes: &[&str],
    attack_move_enabled: bool,
) -> MovementInputFlags {
    let mut flags = MovementInputFlags::default();
    for code in held_codes {
        if attack_move_enabled
            && (keybinds.code_at("attackMove", 0) == Some(*code)
                || keybinds.code_at("attackMove", 1) == Some(*code))
        {
            continue;
        }
        match keybinds.held_action_for_code(code) {
            Some("forward") => flags.forward = true,
            Some("back") => flags.back = true,
            Some("turnLeft") => flags.turn_left = true,
            Some("turnRight") => flags.turn_right = true,
            Some("strafeLeft") => flags.strafe_left = true,
            Some("strafeRight") => flags.strafe_right = true,
            Some("jump") => flags.jump = true,
            _ => {}
        }
    }
    flags
}
