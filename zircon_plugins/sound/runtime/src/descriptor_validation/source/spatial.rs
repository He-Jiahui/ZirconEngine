use zircon_runtime::core::framework::sound::{SoundError, SoundSourceDescriptor};

pub(super) fn validate_spatial_settings(source: &SoundSourceDescriptor) -> Result<(), SoundError> {
    let spatial = source.spatial;
    if !(0.0..=1.0).contains(&spatial.spatial_blend)
        || !spatial.min_distance.is_finite()
        || !spatial.max_distance.is_finite()
        || spatial.min_distance < 0.0
        || spatial.max_distance < spatial.min_distance
        || spatial
            .spatial_scale
            .is_some_and(|scale| !scale.is_finite() || scale < 0.0)
        || !spatial.cone_inner_degrees.is_finite()
        || !spatial.cone_outer_degrees.is_finite()
        || spatial.cone_inner_degrees < 0.0
        || spatial.cone_outer_degrees < spatial.cone_inner_degrees
        || spatial.cone_outer_degrees > 360.0
        || !spatial.doppler_factor.is_finite()
        || spatial.doppler_factor < 0.0
    {
        return Err(SoundError::InvalidParameter(
            "source spatial settings are outside the supported range".to_string(),
        ));
    }
    Ok(())
}
