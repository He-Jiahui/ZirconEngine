use bytemuck::{Pod, Zeroable};

use crate::core::framework::render::{RenderViewFamilyPhaseTargets, TaaQualityPreset};

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
/// TAA shader 的帧级 ABI：质量档位、局部 ViewRect 和历史可用性与图输出一致。
pub(in crate::graphics::scene::scene_renderer) struct TaaResolveParams {
    pub(in crate::graphics::scene::scene_renderer) input_viewport: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer) output_viewport: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer) flags_and_quality: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer) blend_and_clamp: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer) responsive_and_reactive: [f32; 4],
}

impl TaaResolveParams {
    /// 从 ViewFamily 阶段目标构造参数；只有有效历史才允许 shader 使用时间累积。
    pub(in crate::graphics::scene::scene_renderer) fn new(
        phase_targets: RenderViewFamilyPhaseTargets,
        history_valid: bool,
        quality_preset: TaaQualityPreset,
    ) -> Self {
        let quality = TaaResolveQuality::for_preset(quality_preset);
        let input = phase_targets
            .input()
            .expect("temporal reconstruction must declare an input target")
            .viewport();
        let output = phase_targets.output().viewport();
        // Graph-owned textures store each camera ViewRect in local coordinates. The absolute
        // ViewRect is retained by ViewFamily for final output placement, while reconstruction
        // operates on origin-zero primary and secondary images.
        Self {
            input_viewport: [
                0,
                0,
                input.physical_size.x.max(1),
                input.physical_size.y.max(1),
            ],
            output_viewport: [
                0,
                0,
                output.physical_size.x.max(1),
                output.physical_size.y.max(1),
            ],
            flags_and_quality: [
                if history_valid {
                    TAA_RESOLVE_ENABLED
                } else {
                    0
                },
                quality_preset as u32,
                0,
                0,
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
        self.flags_and_quality[0] == TAA_RESOLVE_ENABLED
    }
}

#[cfg(test)]
#[path = "tests/taa_resolve_params.rs"]
mod tests;
