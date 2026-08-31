use super::{
    RenderHybridGiExtract, RenderHybridGiFallbackReason, RenderHybridGiMode, RenderHybridGiProfile,
    RenderHybridGiQuality, RenderHybridGiResolvedSettings,
};

impl RenderHybridGiExtract {
    pub fn resolved_settings(
        &self,
        baked_lighting_available: bool,
    ) -> RenderHybridGiResolvedSettings {
        let (requested_mode, quality, trace_budget, card_budget, voxel_budget) = match self.profile
        {
            RenderHybridGiProfile::FullyDynamic => (
                RenderHybridGiMode::DynamicOnly,
                RenderHybridGiQuality::High,
                96,
                192,
                96,
            ),
            RenderHybridGiProfile::IndoorStatic => (
                RenderHybridGiMode::BakedStaticDynamic,
                RenderHybridGiQuality::High,
                64,
                256,
                64,
            ),
            RenderHybridGiProfile::OpenWorld => (
                RenderHybridGiMode::BakedStaticDynamic,
                RenderHybridGiQuality::Medium,
                64,
                192,
                128,
            ),
            RenderHybridGiProfile::Cinematic => (
                RenderHybridGiMode::BakedStaticDynamic,
                RenderHybridGiQuality::High,
                192,
                512,
                192,
            ),
            RenderHybridGiProfile::Custom => (
                self.mode,
                self.quality,
                self.trace_budget,
                self.card_budget,
                self.voxel_budget,
            ),
        };
        let fallback_reason = (requested_mode == RenderHybridGiMode::BakedStaticDynamic
            && !baked_lighting_available)
            .then_some(RenderHybridGiFallbackReason::BakedLightingUnavailable);

        RenderHybridGiResolvedSettings {
            mode: if fallback_reason.is_some() {
                RenderHybridGiMode::DynamicOnly
            } else {
                requested_mode
            },
            profile: self.profile,
            quality,
            trace_budget: non_zero_override(self.trace_budget, trace_budget),
            card_budget: non_zero_override(self.card_budget, card_budget),
            voxel_budget: non_zero_override(self.voxel_budget, voxel_budget),
            fallback_reason,
        }
    }
}

const fn non_zero_override(value: u32, profile_default: u32) -> u32 {
    if value == 0 {
        profile_default
    } else {
        value
    }
}
