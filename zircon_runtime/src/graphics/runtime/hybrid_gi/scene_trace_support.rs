use std::collections::BTreeSet;

use super::HybridGiRuntimeState;

const ANCESTOR_TRACE_SUPPORT_FALLOFF: f32 = 0.78;
const DESCENDANT_TRACE_SUPPORT_FALLOFF: f32 = 0.84;
const TRACE_SUPPORT_DECAY_Q8: u16 = 192;
const REQUEST_SUPPORT_DECAY_Q8: u16 = 208;
const REQUESTED_PROBE_SUPPORT: f32 = 1.0;
const REQUESTED_ANCESTOR_SUPPORT: f32 = 0.78;
const REQUESTED_ANCESTOR_FALLOFF: f32 = 0.78;
const REQUESTED_DESCENDANT_SUPPORT: f32 = 0.92;
const REQUESTED_DESCENDANT_FALLOFF: f32 = 0.84;

impl HybridGiRuntimeState {
    pub(in crate::graphics::runtime::hybrid_gi) fn assign_scheduled_trace_regions(
        &mut self,
        trace_region_ids: impl IntoIterator<Item = u32>,
    ) {
        let mut seen_region_ids = BTreeSet::new();
        self.scheduled_trace_regions = trace_region_ids
            .into_iter()
            .filter(|region_id| seen_region_ids.insert(*region_id))
            .collect();
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn refresh_recent_lineage_trace_support(&mut self) {
        let probe_ids = self.probe_scene_data.keys().copied().collect::<Vec<_>>();
        for probe_id in probe_ids {
            let current_q8 =
                quantize_support_q8(self.current_lineage_trace_support_score(probe_id));
            let decayed_recent_q8 = self
                .recent_lineage_trace_support_q8
                .get(&probe_id)
                .copied()
                .map(decay_support_q8)
                .unwrap_or_default();
            let refreshed_q8 = current_q8.max(decayed_recent_q8);
            if refreshed_q8 == 0 {
                self.recent_lineage_trace_support_q8.remove(&probe_id);
            } else {
                self.recent_lineage_trace_support_q8
                    .insert(probe_id, refreshed_q8);
            }

            let current_request_q8 =
                quantize_support_q8(self.current_requested_lineage_support_score(probe_id));
            let decayed_recent_request_q8 = self
                .recent_requested_lineage_support_q8
                .get(&probe_id)
                .copied()
                .map(decay_request_support_q8)
                .unwrap_or_default();
            let refreshed_request_q8 = current_request_q8.max(decayed_recent_request_q8);
            if refreshed_request_q8 == 0 {
                self.recent_requested_lineage_support_q8.remove(&probe_id);
            } else {
                self.recent_requested_lineage_support_q8
                    .insert(probe_id, refreshed_request_q8);
            }
        }
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn effective_lineage_trace_support_score(
        &self,
        probe_id: u32,
    ) -> f32 {
        let trace_support = self.current_lineage_trace_support_score(probe_id).max(
            self.recent_lineage_trace_support_q8
                .get(&probe_id)
                .copied()
                .map(dequantize_support_q8)
                .unwrap_or_default(),
        );
        let request_support = self.current_requested_lineage_support_score(probe_id).max(
            self.recent_requested_lineage_support_q8
                .get(&probe_id)
                .copied()
                .map(dequantize_support_q8)
                .unwrap_or_default(),
        );
        trace_support.max(request_support)
    }

    fn current_lineage_trace_support_score(&self, probe_id: u32) -> f32 {
        let mut total_support = 0.0_f32;
        let mut lineage_weight = 1.0_f32;
        let mut current_probe_id = probe_id;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);

        loop {
            total_support +=
                self.single_probe_scene_trace_support(current_probe_id) * lineage_weight;
            let Some(parent_probe_id) = self.probe_parent_probes.get(&current_probe_id).copied()
            else {
                break;
            };
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            lineage_weight *= ANCESTOR_TRACE_SUPPORT_FALLOFF;
            current_probe_id = parent_probe_id;
        }

        total_support + self.descendant_trace_support_score(probe_id)
    }

    fn single_probe_scene_trace_support(&self, probe_id: u32) -> f32 {
        let Some(probe) = self.probe_scene_data.get(&probe_id) else {
            return 0.0;
        };

        self.scheduled_trace_regions
            .iter()
            .filter_map(|region_id| self.trace_region_scene_data.get(region_id))
            .map(|region| {
                let reach = probe.radius_q.saturating_add(region.radius_q).max(1) as f32;
                let max_distance = (reach * 3.0).max(1.0);
                let distance_to_region = probe.position_x_q.abs_diff(region.center_x_q) as f32
                    + probe.position_y_q.abs_diff(region.center_y_q) as f32
                    + probe.position_z_q.abs_diff(region.center_z_q) as f32;
                if distance_to_region >= max_distance {
                    return 0.0;
                }

                let proximity = 1.0 - distance_to_region / max_distance;
                proximity * proximity * (region.coverage_q.min(255) as f32 / 128.0)
            })
            .sum()
    }

    fn descendant_trace_support_score(&self, probe_id: u32) -> f32 {
        let mut stack = self
            .probe_parent_probes
            .iter()
            .filter_map(|(&candidate_probe_id, &parent_probe_id)| {
                (parent_probe_id == probe_id).then_some((candidate_probe_id, 1usize))
            })
            .collect::<Vec<_>>();
        let mut visited_probe_ids = BTreeSet::new();
        let mut best_support = 0.0_f32;

        while let Some((candidate_probe_id, depth)) = stack.pop() {
            if !visited_probe_ids.insert(candidate_probe_id) {
                continue;
            }

            best_support = best_support.max(
                self.single_probe_scene_trace_support(candidate_probe_id)
                    * DESCENDANT_TRACE_SUPPORT_FALLOFF.powi((depth - 1) as i32),
            );
            stack.extend(self.probe_parent_probes.iter().filter_map(
                |(&grandchild_probe_id, &parent_probe_id)| {
                    (parent_probe_id == candidate_probe_id)
                        .then_some((grandchild_probe_id, depth + 1))
                },
            ));
        }

        best_support
    }

    fn current_requested_lineage_support_score(&self, probe_id: u32) -> f32 {
        let direct_support = self
            .current_requested_probe_ids
            .contains(&probe_id)
            .then_some(REQUESTED_PROBE_SUPPORT)
            .unwrap_or_default();
        direct_support
            .max(self.requested_ancestor_support_score(probe_id))
            .max(self.requested_descendant_support_score(probe_id))
    }

    fn requested_ancestor_support_score(&self, probe_id: u32) -> f32 {
        let mut depth = 0usize;
        let mut current_probe_id = probe_id;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);

        while let Some(parent_probe_id) = self.probe_parent_probes.get(&current_probe_id).copied() {
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            depth += 1;
            if self.current_requested_probe_ids.contains(&parent_probe_id) {
                return REQUESTED_ANCESTOR_SUPPORT
                    * REQUESTED_ANCESTOR_FALLOFF.powi((depth - 1) as i32);
            }
            current_probe_id = parent_probe_id;
        }

        0.0
    }

    fn requested_descendant_support_score(&self, probe_id: u32) -> f32 {
        let mut best_support = 0.0_f32;
        let mut stack = self
            .probe_parent_probes
            .iter()
            .filter_map(|(&candidate_probe_id, &parent_probe_id)| {
                (parent_probe_id == probe_id).then_some((candidate_probe_id, 1usize))
            })
            .collect::<Vec<_>>();
        let mut visited_probe_ids = BTreeSet::new();

        while let Some((candidate_probe_id, depth)) = stack.pop() {
            if !visited_probe_ids.insert(candidate_probe_id) {
                continue;
            }
            if self
                .current_requested_probe_ids
                .contains(&candidate_probe_id)
            {
                best_support = best_support.max(
                    REQUESTED_DESCENDANT_SUPPORT
                        * REQUESTED_DESCENDANT_FALLOFF.powi((depth - 1) as i32),
                );
            }
            stack.extend(self.probe_parent_probes.iter().filter_map(
                |(&grandchild_probe_id, &parent_probe_id)| {
                    (parent_probe_id == candidate_probe_id)
                        .then_some((grandchild_probe_id, depth + 1))
                },
            ));
        }

        best_support
    }
}

fn decay_support_q8(value: u16) -> u16 {
    ((u32::from(value) * u32::from(TRACE_SUPPORT_DECAY_Q8)) / 256) as u16
}

fn decay_request_support_q8(value: u16) -> u16 {
    ((u32::from(value) * u32::from(REQUEST_SUPPORT_DECAY_Q8)) / 256) as u16
}

fn quantize_support_q8(value: f32) -> u16 {
    (value.clamp(0.0, u16::MAX as f32 / 256.0) * 256.0).round() as u16
}

fn dequantize_support_q8(value: u16) -> f32 {
    value as f32 / 256.0
}
