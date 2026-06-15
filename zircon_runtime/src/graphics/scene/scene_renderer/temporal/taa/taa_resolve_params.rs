use bytemuck::{Pod, Zeroable};

use crate::core::framework::render::TaaQualityPreset;
use crate::core::math::UVec2;

const TAA_RESOLVE_ENABLED: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
struct TaaResolveQuality {
    history_blend_weight: f32,
    motion_rejection_scale: f32,
    variance_clip_gamma: f32,
    depth_disocclusion_threshold: f32,
    reactive_luma_threshold: f32,
    reactive_velocity_scale: f32,
    responsive_history_multiplier: f32,
    responsive_confidence_cap: f32,
}

impl TaaResolveQuality {
    const fn for_preset(preset: TaaQualityPreset) -> Self {
        match preset {
            TaaQualityPreset::Low => Self {
                history_blend_weight: 0.82,
                motion_rejection_scale: 30.0,
                variance_clip_gamma: 1.25,
                depth_disocclusion_threshold: 0.02,
                reactive_luma_threshold: 0.1,
                reactive_velocity_scale: 12.0,
                responsive_history_multiplier: 0.35,
                responsive_confidence_cap: 0.55,
            },
            TaaQualityPreset::Medium => Self {
                history_blend_weight: 0.9,
                motion_rejection_scale: 24.0,
                variance_clip_gamma: 1.0,
                depth_disocclusion_threshold: 0.01,
                reactive_luma_threshold: 0.07,
                reactive_velocity_scale: 16.0,
                responsive_history_multiplier: 0.25,
                responsive_confidence_cap: 0.45,
            },
            TaaQualityPreset::High => Self {
                history_blend_weight: 0.94,
                motion_rejection_scale: 18.0,
                variance_clip_gamma: 0.85,
                depth_disocclusion_threshold: 0.006,
                reactive_luma_threshold: 0.05,
                reactive_velocity_scale: 20.0,
                responsive_history_multiplier: 0.18,
                responsive_confidence_cap: 0.35,
            },
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer) struct TaaResolveParams {
    pub(in crate::graphics::scene::scene_renderer) viewport_and_flags: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer) blend_and_clamp: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer) responsive_and_reactive: [f32; 4],
}

impl TaaResolveParams {
    pub(in crate::graphics::scene::scene_renderer) fn new(
        viewport_size: UVec2,
        history_valid: bool,
        quality_preset: TaaQualityPreset,
    ) -> Self {
        let quality = TaaResolveQuality::for_preset(quality_preset);
        Self {
            viewport_and_flags: [
                viewport_size.x.max(1),
                viewport_size.y.max(1),
                if history_valid {
                    TAA_RESOLVE_ENABLED
                } else {
                    0
                },
                quality_preset as u32,
            ],
            blend_and_clamp: [
                quality.history_blend_weight,
                quality.motion_rejection_scale,
                quality.variance_clip_gamma,
                quality.depth_disocclusion_threshold,
            ],
            responsive_and_reactive: [
                quality.reactive_luma_threshold,
                quality.reactive_velocity_scale,
                quality.responsive_history_multiplier,
                quality.responsive_confidence_cap,
            ],
        }
    }

    #[cfg(test)]
    pub(super) fn is_enabled(self) -> bool {
        self.viewport_and_flags[2] == TAA_RESOLVE_ENABLED
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::TaaQualityPreset;
    use crate::core::math::UVec2;

    use super::TaaResolveParams;

    #[test]
    fn taa_resolve_params_clamp_viewport_and_encode_quality_constants() {
        let params = TaaResolveParams::new(UVec2::new(0, 720), true, TaaQualityPreset::Medium);

        assert!(params.is_enabled());
        assert_eq!(
            params.viewport_and_flags,
            [1, 720, 1, TaaQualityPreset::Medium as u32]
        );
        assert_eq!(params.blend_and_clamp[0], 0.9);
        assert!(params.blend_and_clamp[1] > 1.0);
        assert_eq!(params.blend_and_clamp[2], 1.0);
        assert!(params.blend_and_clamp[3] > 0.0);
        assert_eq!(params.responsive_and_reactive[0], 0.07);
        assert!(params.responsive_and_reactive[1] > 1.0);
        assert!(params.responsive_and_reactive[2] < 1.0);
        assert!(params.responsive_and_reactive[3] < 1.0);
    }

    #[test]
    fn taa_resolve_params_map_quality_presets_to_blend_and_rejection() {
        let low = TaaResolveParams::new(UVec2::new(1280, 720), true, TaaQualityPreset::Low);
        let high = TaaResolveParams::new(UVec2::new(1280, 720), true, TaaQualityPreset::High);

        assert!(high.blend_and_clamp[0] > low.blend_and_clamp[0]);
        assert!(high.blend_and_clamp[1] < low.blend_and_clamp[1]);
        assert!(high.blend_and_clamp[2] < low.blend_and_clamp[2]);
        assert!(high.blend_and_clamp[3] < low.blend_and_clamp[3]);
        assert!(high.responsive_and_reactive[0] < low.responsive_and_reactive[0]);
        assert!(high.responsive_and_reactive[1] > low.responsive_and_reactive[1]);
        assert!(high.responsive_and_reactive[2] < low.responsive_and_reactive[2]);
        assert!(high.responsive_and_reactive[3] < low.responsive_and_reactive[3]);
    }

    #[test]
    fn taa_resolve_params_disable_history_weight_when_history_is_invalid() {
        let params = TaaResolveParams::new(UVec2::new(1280, 720), false, TaaQualityPreset::High);

        assert!(!params.is_enabled());
        assert_eq!(params.viewport_and_flags[3], TaaQualityPreset::High as u32);
        assert_eq!(params.blend_and_clamp[0], 0.94);
        assert_eq!(params.responsive_and_reactive[2], 0.18);
    }
}
