use std::collections::BTreeMap;

use zircon_runtime::core::framework::render::{
    RenderHybridGiCompositePolicy, RenderHybridGiMode, HYBRID_GI_SOURCE_BAKED_BASELINE,
    HYBRID_GI_SOURCE_DYNAMIC_DELTA, HYBRID_GI_SOURCE_FULL_DYNAMIC,
};

use super::participation::{HybridGiParticipationState, HybridGiSurfaceParticipation};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HybridGiSourceLedger {
    surface_source_masks: BTreeMap<u64, u32>,
    composite_policy: RenderHybridGiCompositePolicy,
}

impl Default for HybridGiSourceLedger {
    fn default() -> Self {
        Self {
            surface_source_masks: BTreeMap::new(),
            composite_policy: RenderHybridGiCompositePolicy::default(),
        }
    }
}

impl HybridGiSourceLedger {
    pub(super) fn synchronize(
        &mut self,
        mode: RenderHybridGiMode,
        participation: &HybridGiParticipationState,
    ) {
        let baked_generation = participation.light_set_generation();
        self.composite_policy = match (mode, baked_generation) {
            (RenderHybridGiMode::BakedStaticDynamic, Some(generation)) => {
                RenderHybridGiCompositePolicy::baked_baseline_with_dynamic_delta(
                    generation,
                    participation.participation_epoch(),
                )
            }
            _ => RenderHybridGiCompositePolicy::full_dynamic(participation.participation_epoch()),
        };
        self.surface_source_masks = participation
            .surfaces()
            .map(|(stable_instance_key, surface)| {
                let mask = surface_source_mask(mode, baked_generation.is_some(), surface);
                debug_assert!(valid_source_mask(mask));
                (stable_instance_key, mask)
            })
            .collect();
    }

    pub(super) fn composite_policy(&self) -> RenderHybridGiCompositePolicy {
        self.composite_policy
    }

    pub(super) fn surface_source_mask(&self, stable_instance_key: u64) -> Option<u32> {
        self.surface_source_masks.get(&stable_instance_key).copied()
    }

    pub(super) fn surface_dynamic_weight_q8(&self, stable_instance_key: u64) -> u8 {
        self.surface_source_mask(stable_instance_key)
            .filter(|mask| {
                mask & (HYBRID_GI_SOURCE_FULL_DYNAMIC | HYBRID_GI_SOURCE_DYNAMIC_DELTA) != 0
            })
            .map(|_| u8::MAX)
            .unwrap_or(0)
    }
}

fn surface_source_mask(
    mode: RenderHybridGiMode,
    has_baked_generation: bool,
    participation: HybridGiSurfaceParticipation,
) -> u32 {
    if participation == HybridGiSurfaceParticipation::Disabled {
        return 0;
    }
    if mode == RenderHybridGiMode::DynamicOnly || !has_baked_generation {
        return HYBRID_GI_SOURCE_FULL_DYNAMIC;
    }

    match participation {
        HybridGiSurfaceParticipation::BakedStatic
        | HybridGiSurfaceParticipation::HybridReceiver => {
            HYBRID_GI_SOURCE_BAKED_BASELINE | HYBRID_GI_SOURCE_DYNAMIC_DELTA
        }
        HybridGiSurfaceParticipation::DynamicReceiver => HYBRID_GI_SOURCE_DYNAMIC_DELTA,
        HybridGiSurfaceParticipation::Disabled => 0,
    }
}

fn valid_source_mask(mask: u32) -> bool {
    mask & HYBRID_GI_SOURCE_FULL_DYNAMIC == 0 || mask & HYBRID_GI_SOURCE_BAKED_BASELINE == 0
}
