use zircon_runtime::core::framework::sound::{SoundError, SoundSourceDescriptor};

pub(super) fn validate_source_parameter_bindings(
    source: &SoundSourceDescriptor,
) -> Result<(), SoundError> {
    for binding in &source.parameter_bindings {
        if binding.source_parameter.as_str().trim().is_empty()
            || binding.synth_parameter.as_str().trim().is_empty()
        {
            return Err(SoundError::InvalidParameter(
                "source parameter bindings require non-empty parameter ids".to_string(),
            ));
        }
        if !is_supported_source_parameter_binding(binding.source_parameter.as_str()) {
            return Err(SoundError::InvalidParameter(format!(
                "unsupported source parameter binding {}",
                binding.source_parameter.as_str()
            )));
        }
    }
    Ok(())
}

fn is_supported_source_parameter_binding(parameter: &str) -> bool {
    matches!(
        parameter,
        "gain"
            | "speed"
            | "playing"
            | "looped"
            | "muted"
            | "position_x"
            | "position_y"
            | "position_z"
            | "forward_x"
            | "forward_y"
            | "forward_z"
            | "velocity_x"
            | "velocity_y"
            | "velocity_z"
            | "spatial_blend"
            | "spatial_scale"
            | "min_distance"
            | "max_distance"
            | "cone_inner_degrees"
            | "cone_outer_degrees"
            | "doppler_factor"
            | "occlusion_enabled"
    )
}
