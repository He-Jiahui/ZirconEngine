use crate::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderBlurSettings {
    pub radius: Real,
}

impl Default for RenderBlurSettings {
    fn default() -> Self {
        Self { radius: 0.0 }
    }
}

impl RenderBlurSettings {
    pub fn is_enabled(self) -> bool {
        self.radius > 0.0
    }

    pub fn render_radius(self) -> Real {
        self.radius.max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::RenderBlurSettings;

    #[test]
    fn blur_settings_use_explicit_enable_predicate_and_clamp_upload_radius() {
        let disabled = RenderBlurSettings { radius: -1.0 };
        let enabled = RenderBlurSettings { radius: 2.5 };

        assert!(!disabled.is_enabled());
        assert_eq!(disabled.render_radius(), 0.0);
        assert!(enabled.is_enabled());
        assert_eq!(enabled.render_radius(), 2.5);
    }
}
