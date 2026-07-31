#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GamepadStickVector {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GamepadMoveFlags {
    pub forward: bool,
    pub back: bool,
    pub strafe_left: bool,
    pub strafe_right: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GamepadLookDelta {
    pub yaw: f64,
    pub pitch: f64,
}

pub fn apply_radial_deadzone(x: f64, y: f64, deadzone: f64) -> GamepadStickVector {
    let magnitude = x.hypot(y);
    if magnitude <= deadzone || magnitude == 0.0 {
        return GamepadStickVector::default();
    }
    let scaled = (magnitude - deadzone) / (1.0 - deadzone);
    let normalized = scaled.min(1.0) / magnitude;
    GamepadStickVector {
        x: x * normalized,
        y: y * normalized,
    }
}

pub fn stick_to_move_flags(x: f64, y: f64, deadzone: f64) -> GamepadMoveFlags {
    if x.hypot(y) < deadzone {
        return GamepadMoveFlags::default();
    }
    let axis_threshold = deadzone * 0.85;
    GamepadMoveFlags {
        forward: y < -axis_threshold,
        back: y > axis_threshold,
        strafe_left: x < -axis_threshold,
        strafe_right: x > axis_threshold,
    }
}

pub fn stick_to_look(
    x: f64,
    y: f64,
    deadzone: f64,
    speed: f64,
    invert_y: bool,
    elapsed_seconds: f64,
) -> GamepadLookDelta {
    let vector = apply_radial_deadzone(x, y, deadzone);
    if vector == GamepadStickVector::default() {
        return GamepadLookDelta::default();
    }
    let pitch_sign = if invert_y { 1.0 } else { -1.0 };
    GamepadLookDelta {
        yaw: -vector.x * speed * elapsed_seconds,
        pitch: pitch_sign * vector.y * speed * elapsed_seconds,
    }
}

pub fn rising_edges(previous: &[bool], current: &[bool]) -> Vec<usize> {
    current
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(index, pressed)| {
            (pressed && !previous.get(index).copied().unwrap_or(false)).then_some(index)
        })
        .collect()
}
