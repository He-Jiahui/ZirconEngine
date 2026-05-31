use zircon_runtime::core::framework::sound::{
    SoundError, SoundListenerDescriptor, SoundParameterId,
};

use super::helpers::{bool_from_value, unsupported_automation_parameter};

pub(super) fn apply_listener_parameter(
    listener: &mut SoundListenerDescriptor,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "active" => listener.active = bool_from_value(value),
        "doppler_tracking" => listener.doppler_tracking = bool_from_value(value),
        "position_x" => listener.position[0] = value,
        "position_y" => listener.position[1] = value,
        "position_z" => listener.position[2] = value,
        "forward_x" => listener.forward[0] = value,
        "forward_y" => listener.forward[1] = value,
        "forward_z" => listener.forward[2] = value,
        "up_x" => listener.up[0] = value,
        "up_y" => listener.up[1] = value,
        "up_z" => listener.up[2] = value,
        "velocity_x" => listener.velocity[0] = value,
        "velocity_y" => listener.velocity[1] = value,
        "velocity_z" => listener.velocity[2] = value,
        "left_ear_offset_x" => listener.left_ear_offset[0] = value,
        "left_ear_offset_y" => listener.left_ear_offset[1] = value,
        "left_ear_offset_z" => listener.left_ear_offset[2] = value,
        "right_ear_offset_x" => listener.right_ear_offset[0] = value,
        "right_ear_offset_y" => listener.right_ear_offset[1] = value,
        "right_ear_offset_z" => listener.right_ear_offset[2] = value,
        _ => return Err(unsupported_automation_parameter("listener", parameter)),
    }
    Ok(())
}
