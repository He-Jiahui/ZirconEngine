use std::collections::{BTreeMap, BTreeSet};

use crate::hybrid_gi::{
    HybridGiResolveProbeSceneData, HybridGiResolveRuntime, HybridGiResolveTraceRegionSceneData,
};

use super::super::HybridGiRuntimeState;
use super::scene_fallback::{quantize_positive, quantize_signed, POSITIVE_RADIUS_SCALE};

impl HybridGiRuntimeState {
    pub(crate) fn build_resolve_runtime(&self) -> HybridGiResolveRuntime {
        let tracked_probe_ids = self.tracked_runtime_probe_ids();
        let mut probe_hierarchy_irradiance_rgb_and_weight = BTreeMap::new();
        let mut probe_scene_driven_hierarchy_irradiance_ids = BTreeSet::new();
        let mut probe_scene_driven_hierarchy_irradiance_quality_q8 = BTreeMap::new();
        let mut probe_scene_driven_hierarchy_irradiance_freshness_q8 = BTreeMap::new();
        let mut probe_scene_driven_hierarchy_irradiance_revision = BTreeMap::new();
        let mut probe_hierarchy_rt_lighting_rgb_and_weight = BTreeMap::new();
        let mut probe_scene_driven_hierarchy_rt_lighting_ids = BTreeSet::new();
        let mut probe_scene_driven_hierarchy_rt_lighting_quality_q8 = BTreeMap::new();
        let mut probe_scene_driven_hierarchy_rt_lighting_freshness_q8 = BTreeMap::new();
        let mut probe_scene_driven_hierarchy_rt_lighting_revision = BTreeMap::new();

        for &probe_id in &tracked_probe_ids {
            let Some((
                encoded,
                scene_driven,
                scene_truth_quality,
                scene_truth_freshness,
                scene_truth_revision,
            )) = self.runtime_hierarchy_irradiance_entry(probe_id)
            else {
                continue;
            };
            if scene_driven {
                probe_scene_driven_hierarchy_irradiance_ids.insert(probe_id);
                probe_scene_driven_hierarchy_irradiance_quality_q8.insert(
                    probe_id,
                    HybridGiResolveRuntime::pack_scene_truth_quality_q8(scene_truth_quality),
                );
                probe_scene_driven_hierarchy_irradiance_freshness_q8.insert(
                    probe_id,
                    HybridGiResolveRuntime::pack_scene_truth_freshness_q8(scene_truth_freshness),
                );
                probe_scene_driven_hierarchy_irradiance_revision
                    .insert(probe_id, scene_truth_revision);
            }
            probe_hierarchy_irradiance_rgb_and_weight.insert(probe_id, encoded);
        }
        for &probe_id in &tracked_probe_ids {
            let Some((
                encoded,
                scene_driven,
                scene_truth_quality,
                scene_truth_freshness,
                scene_truth_revision,
            )) = self.runtime_hierarchy_rt_lighting_entry(probe_id)
            else {
                continue;
            };
            if scene_driven {
                probe_scene_driven_hierarchy_rt_lighting_ids.insert(probe_id);
                probe_scene_driven_hierarchy_rt_lighting_quality_q8.insert(
                    probe_id,
                    HybridGiResolveRuntime::pack_scene_truth_quality_q8(scene_truth_quality),
                );
                probe_scene_driven_hierarchy_rt_lighting_freshness_q8.insert(
                    probe_id,
                    HybridGiResolveRuntime::pack_scene_truth_freshness_q8(scene_truth_freshness),
                );
                probe_scene_driven_hierarchy_rt_lighting_revision
                    .insert(probe_id, scene_truth_revision);
            }
            probe_hierarchy_rt_lighting_rgb_and_weight.insert(probe_id, encoded);
        }

        let mut probe_scene_data = self
            .probe_scene_data()
            .iter()
            .filter_map(|(&probe_id, scene_data)| {
                tracked_probe_ids.contains(&probe_id).then_some((
                    probe_id,
                    HybridGiResolveProbeSceneData::new(
                        scene_data.position_x_q(),
                        scene_data.position_y_q(),
                        scene_data.position_z_q(),
                        scene_data.radius_q(),
                    ),
                ))
            })
            .collect::<BTreeMap<_, _>>();
        if self.scene_representation_owns_runtime() {
            for probe in self
                .scene_representation()
                .screen_probe_runtime_descriptors()
                .into_iter()
                .filter(|probe| tracked_probe_ids.contains(&probe.probe_id()))
            {
                let bounds_center = probe.bounds_center();
                probe_scene_data.entry(probe.probe_id()).or_insert_with(|| {
                    HybridGiResolveProbeSceneData::new(
                        quantize_signed(bounds_center.x),
                        quantize_signed(bounds_center.y),
                        quantize_signed(bounds_center.z),
                        quantize_positive(probe.bounds_radius(), POSITIVE_RADIUS_SCALE),
                    )
                });
            }
        }
        let trace_region_scene_data = self
            .scheduled_trace_region_ids()
            .iter()
            .filter_map(|&region_id| {
                self.trace_region_scene_data()
                    .get(&region_id)
                    .map(|scene_data| {
                        (
                            region_id,
                            HybridGiResolveTraceRegionSceneData::new(
                                scene_data.center_x_q(),
                                scene_data.center_y_q(),
                                scene_data.center_z_q(),
                                scene_data.radius_q(),
                                scene_data.coverage_q(),
                                scene_data.rt_lighting_rgb(),
                            ),
                        )
                    })
            })
            .collect();
        let probe_parent_probes = self
            .probe_parent_probes()
            .iter()
            .filter_map(|(&probe_id, &parent_probe_id)| {
                (tracked_probe_ids.contains(&probe_id)
                    && tracked_probe_ids.contains(&parent_probe_id))
                .then_some((probe_id, parent_probe_id))
            })
            .collect();
        let probe_rt_lighting_rgb = self
            .probe_rt_lighting_rgb()
            .iter()
            .filter_map(|(&probe_id, &rt_lighting_rgb)| {
                tracked_probe_ids
                    .contains(&probe_id)
                    .then_some((probe_id, rt_lighting_rgb))
            })
            .collect::<BTreeMap<_, _>>();
        let probe_hierarchy_resolve_weight_q8 = tracked_probe_ids
            .iter()
            .map(|&probe_id| {
                (
                    probe_id,
                    HybridGiResolveRuntime::pack_resolve_weight_q8(
                        self.runtime_hierarchy_resolve_weight(probe_id),
                    ),
                )
            })
            .collect();

        HybridGiResolveRuntime::new(
            probe_scene_data,
            trace_region_scene_data,
            probe_parent_probes,
            probe_rt_lighting_rgb,
            probe_hierarchy_resolve_weight_q8,
            probe_hierarchy_irradiance_rgb_and_weight,
            probe_hierarchy_rt_lighting_rgb_and_weight,
            probe_scene_driven_hierarchy_irradiance_ids,
            probe_scene_driven_hierarchy_rt_lighting_ids,
            probe_scene_driven_hierarchy_irradiance_quality_q8,
            probe_scene_driven_hierarchy_irradiance_freshness_q8,
            probe_scene_driven_hierarchy_irradiance_revision,
            probe_scene_driven_hierarchy_rt_lighting_quality_q8,
            probe_scene_driven_hierarchy_rt_lighting_freshness_q8,
            probe_scene_driven_hierarchy_rt_lighting_revision,
        )
    }

    fn tracked_runtime_probe_ids(&self) -> Vec<u32> {
        let scene_screen_probe_ids = if self.scene_representation_owns_runtime() {
            self.scene_representation()
                .screen_probe_runtime_descriptors()
                .into_iter()
                .map(|probe| probe.probe_id())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let seed_probe_ids = self
            .resident_probe_ids()
            .chain(self.pending_probe_ids())
            .chain(
                self.pending_update_requests()
                    .iter()
                    .map(|update| update.probe_id()),
            )
            .chain(self.current_requested_probe_ids().iter().copied())
            .chain(
                self.scene_representation_owns_runtime()
                    .then_some(self.probe_scene_data().keys().copied())
                    .into_iter()
                    .flatten(),
            )
            .chain(scene_screen_probe_ids)
            .collect::<BTreeSet<_>>();
        let mut tracked_probe_ids = seed_probe_ids.clone();
        for probe_id in seed_probe_ids {
            let mut current_probe_id = probe_id;
            let mut visited_probe_ids = BTreeSet::from([probe_id]);
            while let Some(parent_probe_id) =
                self.probe_parent_probes().get(&current_probe_id).copied()
            {
                if !visited_probe_ids.insert(parent_probe_id) {
                    break;
                }
                tracked_probe_ids.insert(parent_probe_id);
                current_probe_id = parent_probe_id;
            }
        }
        tracked_probe_ids.into_iter().collect()
    }
}
