use crate::core::framework::render::RenderMaterialValidationError;

use super::AlphaMode;

pub fn validate_alpha_mode(alpha_mode: &AlphaMode) -> Vec<RenderMaterialValidationError> {
    match alpha_mode {
        AlphaMode::Mask { cutoff } if !cutoff.is_finite() || !(0.0..=1.0).contains(cutoff) => {
            vec![RenderMaterialValidationError::InvalidMaskCutoff { cutoff: *cutoff }]
        }
        _ => Vec::new(),
    }
}
