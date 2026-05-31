use zircon_runtime::core::framework::sound::{SoundError, SoundVolumeDescriptor, SoundVolumeShape};

use super::common::validate_vec3;

pub(crate) fn validate_volume_descriptor(volume: &SoundVolumeDescriptor) -> Result<(), SoundError> {
    if !volume.interior_gain.is_finite()
        || !volume.exterior_gain.is_finite()
        || !volume.reverb_send.is_finite()
        || !volume.crossfade_distance.is_finite()
        || volume.crossfade_distance < 0.0
    {
        return Err(SoundError::InvalidParameter(
            "volume gains, reverb send, and crossfade distance must be finite".to_string(),
        ));
    }
    if let Some(cutoff) = volume.low_pass_cutoff_hz {
        if !cutoff.is_finite() || cutoff <= 0.0 {
            return Err(SoundError::InvalidParameter(
                "volume low-pass cutoff must be positive and finite".to_string(),
            ));
        }
    }
    match &volume.shape {
        SoundVolumeShape::Sphere { center, radius } => {
            validate_vec3("volume sphere center", *center)?;
            if !radius.is_finite() || *radius < 0.0 {
                return Err(SoundError::InvalidParameter(
                    "volume sphere radius must be non-negative and finite".to_string(),
                ));
            }
        }
        SoundVolumeShape::Box { center, extents } => {
            validate_vec3("volume box center", *center)?;
            validate_vec3("volume box extents", *extents)?;
            if extents.iter().any(|extent| *extent < 0.0) {
                return Err(SoundError::InvalidParameter(
                    "volume box extents must be non-negative".to_string(),
                ));
            }
        }
    }
    Ok(())
}
