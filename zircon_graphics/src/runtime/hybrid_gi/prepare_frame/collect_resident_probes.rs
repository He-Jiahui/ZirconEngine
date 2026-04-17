use crate::types::HybridGiPrepareProbe;

use super::super::hybrid_gi_runtime_state::HybridGiRuntimeState;

const UNINITIALIZED_PROBE_IRRADIANCE_RGB: [u8; 3] = [0, 0, 0];

pub(super) fn collect_resident_probes(runtime: &HybridGiRuntimeState) -> Vec<HybridGiPrepareProbe> {
    runtime
        .resident_slots
        .iter()
        .map(|(&probe_id, &slot)| HybridGiPrepareProbe {
            probe_id,
            slot,
            ray_budget: runtime
                .probe_ray_budgets
                .get(&probe_id)
                .copied()
                .unwrap_or_default(),
            irradiance_rgb: runtime
                .probe_irradiance_rgb
                .get(&probe_id)
                .copied()
                .unwrap_or(UNINITIALIZED_PROBE_IRRADIANCE_RGB),
        })
        .collect()
}
