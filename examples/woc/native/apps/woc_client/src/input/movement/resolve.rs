use woc_protocol::MovementInputFlags;

use crate::GamepadMoveFlags;

/// Host-sampled held sources. This stays presentation/input state: it does not
/// own player position, collision, simulation time, or movement authority.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MovementInputSources {
    pub keyboard: MovementInputFlags,
    pub touch: GamepadMoveFlags,
    pub gamepad: GamepadMoveFlags,
    pub pointer_forward: bool,
    pub autorun: bool,
    pub suspended: bool,
    pub controller_override: Option<MovementInputFlags>,
    pub mouse_camera: bool,
    pub mouselook: bool,
    pub latched_jump: bool,
}

/// Mirrors the target's `readMoveInput` ordering before the result is sampled
/// by the independent 20 Hz movement stream.
pub fn resolve_movement_input(sources: MovementInputSources) -> MovementInputFlags {
    if sources.suspended {
        return MovementInputFlags {
            forward: sources.autorun,
            ..MovementInputFlags::default()
        };
    }
    if let Some(controller) = sources.controller_override {
        return controller;
    }

    let forward = sources.keyboard.forward
        || sources.pointer_forward
        || sources.autorun
        || sources.touch.forward
        || sources.gamepad.forward;
    let back = sources.keyboard.back || sources.touch.back || sources.gamepad.back;
    let jump = sources.keyboard.jump || sources.latched_jump;
    if sources.mouse_camera {
        return MovementInputFlags {
            forward,
            back,
            turn_left: false,
            turn_right: false,
            strafe_left: sources.keyboard.strafe_left
                || sources.keyboard.turn_left
                || sources.touch.strafe_left
                || sources.gamepad.strafe_left,
            strafe_right: sources.keyboard.strafe_right
                || sources.keyboard.turn_right
                || sources.touch.strafe_right
                || sources.gamepad.strafe_right,
            jump,
        };
    }

    MovementInputFlags {
        forward,
        back,
        turn_left: !sources.mouselook && sources.keyboard.turn_left,
        turn_right: !sources.mouselook && sources.keyboard.turn_right,
        strafe_left: sources.keyboard.strafe_left
            || (sources.mouselook && sources.keyboard.turn_left)
            || sources.touch.strafe_left
            || sources.gamepad.strafe_left,
        strafe_right: sources.keyboard.strafe_right
            || (sources.mouselook && sources.keyboard.turn_right)
            || sources.touch.strafe_right
            || sources.gamepad.strafe_right,
        jump,
    }
}
