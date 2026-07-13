use std::collections::BTreeSet;

use crate::hybrid_gi::HybridGiPrepareProbe;

use super::super::HybridGiRuntimeState;

const UNINITIALIZED_PROBE_IRRADIANCE_RGB: [u8; 3] = [0, 0, 0];

pub(super) fn collect_resident_probes(runtime: &HybridGiRuntimeState) -> Vec<HybridGiPrepareProbe> {
    let mut resident_probe_ids = BTreeSet::new();
    let mut resident_probes = runtime
        .resident_probe_slots()
        .map(|(probe_id, slot)| HybridGiPrepareProbe {
            probe_id,
            slot,
            stable_instance_key: 0,
            source_mask: zircon_runtime::core::framework::render::HYBRID_GI_SOURCE_FULL_DYNAMIC,
            dynamic_weight_q8: u8::MAX,
            ray_budget: runtime
                .probe_ray_budgets()
                .get(&probe_id)
                .copied()
                .unwrap_or_default(),
            irradiance_rgb: runtime
                .probe_irradiance_rgb()
                .get(&probe_id)
                .copied()
                .unwrap_or(UNINITIALIZED_PROBE_IRRADIANCE_RGB),
        })
        .inspect(|probe| {
            resident_probe_ids.insert(probe.probe_id);
        })
        .collect::<Vec<_>>();

    if runtime.scene_representation_owns_runtime() {
        resident_probes.extend(
            runtime
                .scene_representation()
                .screen_probe_runtime_descriptors()
                .into_iter()
                .filter_map(|probe| {
                    resident_probe_ids
                        .insert(probe.probe_id())
                        .then_some(HybridGiPrepareProbe {
                            probe_id: probe.probe_id(),
                            slot: probe.slot(),
                            stable_instance_key: probe.stable_instance_key(),
                            source_mask: probe.source_mask(),
                            dynamic_weight_q8: probe.dynamic_weight_q8(),
                            ray_budget: probe.ray_budget(),
                            irradiance_rgb: probe.irradiance_rgb(),
                        })
                }),
        );
    }

    resident_probes
}
