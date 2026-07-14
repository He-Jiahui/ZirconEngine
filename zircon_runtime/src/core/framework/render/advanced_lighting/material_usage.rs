use serde::{Deserialize, Serialize};

use super::StandardPbrMaterialFeatures;

/// View-local summary used to keep advanced PBR graph work material-driven.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvancedPbrMaterialFrameUsage {
    pub clearcoat: bool,
    pub anisotropy: bool,
    pub specular_transmission: bool,
    pub diffuse_transmission: bool,
    pub late_forward_opaque: bool,
}

impl AdvancedPbrMaterialFrameUsage {
    pub const fn is_empty(self) -> bool {
        !self.clearcoat
            && !self.anisotropy
            && !self.specular_transmission
            && !self.diffuse_transmission
            && !self.late_forward_opaque
    }

    pub const fn requires_forward_path(self) -> bool {
        !self.is_empty()
    }

    pub const fn requires_scene_color_copy(self) -> bool {
        self.specular_transmission
    }

    pub const fn uses_transmission(self) -> bool {
        self.specular_transmission || self.diffuse_transmission
    }

    pub const fn requires_late_forward_opaque_pass(self) -> bool {
        self.late_forward_opaque
    }

    pub fn record(&mut self, features: &StandardPbrMaterialFeatures) {
        self.clearcoat |= features.uses_clearcoat();
        self.anisotropy |= features.uses_anisotropy();
        self.specular_transmission |=
            features.specular_transmission.is_finite() && features.specular_transmission > 0.0;
        self.diffuse_transmission |=
            features.diffuse_transmission.is_finite() && features.diffuse_transmission > 0.0;
        self.late_forward_opaque |= (features.uses_clearcoat() || features.uses_anisotropy())
            && !features.uses_transmission();
    }
}

#[cfg(test)]
mod tests {
    use super::AdvancedPbrMaterialFrameUsage;
    use crate::core::framework::render::StandardPbrMaterialFeatures;

    #[test]
    fn render_advanced_material_frame_usage_only_requests_copy_for_specular_transmission() {
        let mut usage = AdvancedPbrMaterialFrameUsage::default();
        usage.record(&StandardPbrMaterialFeatures {
            diffuse_transmission: 0.4,
            ..Default::default()
        });

        assert!(usage.requires_forward_path());
        assert!(!usage.requires_scene_color_copy());

        usage.record(&StandardPbrMaterialFeatures {
            specular_transmission: 0.7,
            ..Default::default()
        });
        assert!(usage.requires_scene_color_copy());
    }

    #[test]
    fn render_advanced_material_frame_usage_separates_opaque_forward_from_transmission() {
        let mut usage = AdvancedPbrMaterialFrameUsage::default();
        usage.record(&StandardPbrMaterialFeatures {
            clearcoat: 1.0,
            ..Default::default()
        });
        assert!(usage.requires_late_forward_opaque_pass());

        let mut transmitted = AdvancedPbrMaterialFrameUsage::default();
        transmitted.record(&StandardPbrMaterialFeatures {
            clearcoat: 1.0,
            specular_transmission: 0.5,
            ..Default::default()
        });
        assert!(!transmitted.requires_late_forward_opaque_pass());
        assert!(transmitted.uses_transmission());
        assert!(transmitted.requires_scene_color_copy());
    }
}
