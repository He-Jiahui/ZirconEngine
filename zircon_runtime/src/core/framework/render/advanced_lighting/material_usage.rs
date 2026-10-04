use serde::{Deserialize, Serialize};

use super::StandardPbrMaterialFeatures;

/// View-local summary used to keep advanced PBR graph work material-driven.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvancedPbrMaterialFrameUsage {
    pub clearcoat: bool,
    pub anisotropy: bool,
    pub dielectric_f0_override: bool,
    pub specular_transmission: bool,
    pub diffuse_transmission: bool,
    pub late_forward_opaque: bool,
}

impl AdvancedPbrMaterialFrameUsage {
    pub const fn is_empty(self) -> bool {
        !self.clearcoat
            && !self.anisotropy
            && !self.dielectric_f0_override
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

    /// 汇总当前相机可见材质的高级叶片；提交阶段调用后供编译图决定前向路径与场景色拷贝。
    pub fn record(&mut self, features: &StandardPbrMaterialFeatures) {
        self.clearcoat |= features.uses_clearcoat();
        self.anisotropy |= features.uses_anisotropy();
        self.dielectric_f0_override |= features.uses_dielectric_f0_override();
        self.specular_transmission |=
            features.specular_transmission.is_finite() && features.specular_transmission > 0.0;
        self.diffuse_transmission |=
            features.diffuse_transmission.is_finite() && features.diffuse_transmission > 0.0;
        self.late_forward_opaque |= (features.uses_clearcoat()
            || features.uses_anisotropy()
            || features.uses_dielectric_f0_override())
            && !features.uses_transmission();
    }
}

#[cfg(test)]
#[path = "tests/material_usage.rs"]
mod tests;
