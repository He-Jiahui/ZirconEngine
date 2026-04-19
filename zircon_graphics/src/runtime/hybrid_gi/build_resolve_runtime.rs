use std::collections::{BTreeMap, BTreeSet};

use crate::types::HybridGiResolveRuntime;

use super::HybridGiRuntimeState;

const CHILD_SPECIFICITY_BOOST: f32 = 0.3;
const RESIDENT_CHILD_ATTENUATION: f32 = 0.78;
const FARTHER_ANCESTOR_BUDGET_FALLOFF: f32 = 0.72;
const FARTHER_ANCESTOR_BUDGET_SCALE: f32 = 0.6;
const FARTHER_ANCESTOR_IRRADIANCE_INHERITANCE_FALLOFF: f32 = 0.72;
const IRRADIANCE_INHERITANCE_WEIGHT_SCALE: f32 = 0.5;
const DIRECT_LINEAGE_IRRADIANCE_WEIGHT_SCALE: f32 = 0.45;
const DESCENDANT_LINEAGE_IRRADIANCE_FALLOFF: f32 = 0.84;
const ANCESTOR_TRACE_INHERITANCE_FALLOFF: f32 = 0.72;
const TRACE_INHERITANCE_WEIGHT_SCALE: f32 = 0.45;
const DIRECT_LINEAGE_TRACE_WEIGHT_SCALE: f32 = 0.45;
const DESCENDANT_LINEAGE_TRACE_FALLOFF: f32 = 0.84;
const STANDALONE_DIRECT_TRACE_WEIGHT_SCALE: f32 = 0.35;
const SCENE_TRACE_RESOLVE_SCALE: f32 = 0.35;
const SCENE_TRACE_RT_SCALE: f32 = 0.25;

impl HybridGiRuntimeState {
    pub(crate) fn build_resolve_runtime(&self) -> HybridGiResolveRuntime {
        let tracked_probe_ids = self.tracked_runtime_probe_ids();

        HybridGiResolveRuntime {
            probe_rt_lighting_rgb: self
                .probe_rt_lighting_rgb
                .iter()
                .filter_map(|(&probe_id, &rt_lighting_rgb)| {
                    tracked_probe_ids
                        .contains(&probe_id)
                        .then_some((probe_id, rt_lighting_rgb))
                })
                .collect::<BTreeMap<_, _>>(),
            probe_hierarchy_resolve_weight_q8: tracked_probe_ids
                .iter()
                .map(|&probe_id| {
                    (
                        probe_id,
                        HybridGiResolveRuntime::pack_resolve_weight_q8(
                            self.runtime_hierarchy_resolve_weight(probe_id),
                        ),
                    )
                })
                .collect(),
            probe_hierarchy_irradiance_rgb_and_weight: tracked_probe_ids
                .iter()
                .filter_map(|&probe_id| {
                    self.runtime_hierarchy_irradiance(probe_id)
                        .map(|encoded| (probe_id, encoded))
                })
                .collect(),
            probe_hierarchy_rt_lighting_rgb_and_weight: tracked_probe_ids
                .iter()
                .filter_map(|&probe_id| {
                    self.runtime_hierarchy_rt_lighting(probe_id)
                        .map(|encoded| (probe_id, encoded))
                })
                .collect(),
        }
    }

    fn tracked_runtime_probe_ids(&self) -> Vec<u32> {
        let seed_probe_ids = self.resident_slots
            .keys()
            .copied()
            .chain(self.pending_probes.iter().copied())
            .chain(self.pending_updates.iter().map(|update| update.probe_id))
            .chain(self.current_requested_probe_ids.iter().copied())
            .collect::<BTreeSet<_>>()
            ;
        let mut tracked_probe_ids = seed_probe_ids.clone();
        for probe_id in seed_probe_ids {
            let mut current_probe_id = probe_id;
            let mut visited_probe_ids = BTreeSet::from([probe_id]);
            while let Some(parent_probe_id) = self.probe_parent_probes.get(&current_probe_id).copied() {
                if !visited_probe_ids.insert(parent_probe_id) {
                    break;
                }
                tracked_probe_ids.insert(parent_probe_id);
                current_probe_id = parent_probe_id;
            }
        }
        tracked_probe_ids.into_iter().collect()
    }

    fn runtime_hierarchy_resolve_weight(&self, probe_id: u32) -> f32 {
        let resident_child_count = self.resident_descendant_count(probe_id);
        let resident_parent_depth = self.resident_parent_depth(probe_id);
        let farther_ancestor_budget_support =
            self.farther_resident_ancestor_budget_support(probe_id);
        let scene_trace_support = self.effective_lineage_trace_support_score(probe_id).clamp(0.0, 1.5);

        let specificity_weight = 1.0 + resident_parent_depth as f32 * CHILD_SPECIFICITY_BOOST;
        let attenuation_weight = if resident_child_count == 0 {
            1.0
        } else {
            RESIDENT_CHILD_ATTENUATION.powi(resident_child_count as i32)
        };
        let lineage_budget_weight =
            1.0 + farther_ancestor_budget_support * FARTHER_ANCESTOR_BUDGET_SCALE;
        let scene_trace_weight = 1.0 + scene_trace_support * SCENE_TRACE_RESOLVE_SCALE;
        (specificity_weight * attenuation_weight * lineage_budget_weight * scene_trace_weight)
            .clamp(0.25, 2.75)
    }

    fn runtime_hierarchy_irradiance(&self, probe_id: u32) -> Option<[u8; 4]> {
        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;
        let mut current_probe_id = probe_id;
        let mut resident_ancestor_count = 0usize;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);

        while let Some(parent_probe_id) = self.probe_parent_probes.get(&current_probe_id).copied() {
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            if self.resident_slots.contains_key(&parent_probe_id) {
                resident_ancestor_count += 1;
                if resident_ancestor_count > 1 {
                    let Some(ancestor_irradiance_rgb) =
                        self.probe_irradiance_rgb.get(&parent_probe_id).copied()
                    else {
                        current_probe_id = parent_probe_id;
                        continue;
                    };
                    let farther_ancestor_depth = resident_ancestor_count - 2;
                    let support = FARTHER_ANCESTOR_IRRADIANCE_INHERITANCE_FALLOFF
                        .powi(farther_ancestor_depth as i32)
                        * budget_weight(
                            self.probe_ray_budgets
                                .get(&parent_probe_id)
                                .copied()
                                .unwrap_or_default(),
                        );
                    if support > 0.0 {
                        weighted_rgb[0] += ancestor_irradiance_rgb[0] as f32 / 255.0 * support;
                        weighted_rgb[1] += ancestor_irradiance_rgb[1] as f32 / 255.0 * support;
                        weighted_rgb[2] += ancestor_irradiance_rgb[2] as f32 / 255.0 * support;
                        total_support += support;
                    }
                }
            }
            current_probe_id = parent_probe_id;
        }

        if total_support <= f32::EPSILON {
            return self
                .direct_lineage_irradiance_fallback(probe_id)
                .or_else(|| self.descendant_lineage_irradiance_fallback(probe_id));
        }

        Some(HybridGiResolveRuntime::pack_rgb_and_weight(
            [
                weighted_rgb[0] / total_support,
                weighted_rgb[1] / total_support,
                weighted_rgb[2] / total_support,
            ],
            (total_support * IRRADIANCE_INHERITANCE_WEIGHT_SCALE).clamp(0.0, 0.75),
        ))
    }

    fn runtime_hierarchy_rt_lighting(&self, probe_id: u32) -> Option<[u8; 4]> {
        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;
        let mut current_probe_id = probe_id;
        let mut ancestor_depth = 0usize;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);
        let scene_trace_weight = 1.0
            + self
                .effective_lineage_trace_support_score(probe_id)
                .clamp(0.0, 1.5)
                * SCENE_TRACE_RT_SCALE;

        while let Some(parent_probe_id) = self.probe_parent_probes.get(&current_probe_id).copied() {
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            if self.resident_slots.contains_key(&parent_probe_id) {
                let Some(ancestor_rt_lighting_rgb) =
                    self.probe_rt_lighting_rgb.get(&parent_probe_id).copied()
                else {
                    current_probe_id = parent_probe_id;
                    continue;
                };
                let resident_budget_weight = budget_weight(
                    self.probe_ray_budgets
                        .get(&parent_probe_id)
                        .copied()
                        .unwrap_or_default(),
                );
                if resident_budget_weight > f32::EPSILON {
                    let hierarchy_weight =
                        ANCESTOR_TRACE_INHERITANCE_FALLOFF.powi(ancestor_depth as i32);
                    let intensity_weight = runtime_rgb_intensity(ancestor_rt_lighting_rgb);
                    let support = hierarchy_weight
                        * resident_budget_weight
                        * intensity_weight
                        * scene_trace_weight;
                    if support > 0.0 {
                        weighted_rgb[0] += ancestor_rt_lighting_rgb[0] as f32 / 255.0 * support;
                        weighted_rgb[1] += ancestor_rt_lighting_rgb[1] as f32 / 255.0 * support;
                        weighted_rgb[2] += ancestor_rt_lighting_rgb[2] as f32 / 255.0 * support;
                        total_support += support;
                    }
                }
                ancestor_depth += 1;
            }
            current_probe_id = parent_probe_id;
        }

        if total_support <= f32::EPSILON {
            return self
                .direct_lineage_rt_lighting_fallback(probe_id)
                .or_else(|| self.descendant_lineage_rt_lighting_fallback(probe_id));
        }

        Some(HybridGiResolveRuntime::pack_rgb_and_weight(
            [
                weighted_rgb[0] / total_support,
                weighted_rgb[1] / total_support,
                weighted_rgb[2] / total_support,
            ],
            (total_support * TRACE_INHERITANCE_WEIGHT_SCALE).clamp(0.0, 0.75),
        ))
    }

    fn direct_lineage_irradiance_fallback(&self, probe_id: u32) -> Option<[u8; 4]> {
        if self.resident_slots.contains_key(&probe_id) {
            return None;
        }

        let direct_lineage_support = self
            .effective_lineage_trace_support_score(probe_id)
            .clamp(0.0, 1.5);
        if direct_lineage_support <= f32::EPSILON {
            return self.standalone_direct_rt_lighting_fallback(probe_id);
        }

        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;

        if let Some(direct_irradiance_rgb) = self.probe_irradiance_rgb.get(&probe_id).copied() {
            weighted_rgb[0] += direct_irradiance_rgb[0] as f32 / 255.0 * direct_lineage_support;
            weighted_rgb[1] += direct_irradiance_rgb[1] as f32 / 255.0 * direct_lineage_support;
            weighted_rgb[2] += direct_irradiance_rgb[2] as f32 / 255.0 * direct_lineage_support;
            total_support += direct_lineage_support;
        }

        let mut current_probe_id = probe_id;
        let mut ancestor_depth = 0usize;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);
        while let Some(parent_probe_id) = self.probe_parent_probes.get(&current_probe_id).copied() {
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            if let Some(ancestor_irradiance_rgb) =
                self.probe_irradiance_rgb.get(&parent_probe_id).copied()
            {
                let ancestor_support = direct_lineage_support
                    * FARTHER_ANCESTOR_IRRADIANCE_INHERITANCE_FALLOFF.powi(ancestor_depth as i32)
                    * budget_weight(
                        self.probe_ray_budgets
                            .get(&parent_probe_id)
                            .copied()
                            .unwrap_or_default(),
                    );
                if ancestor_support > f32::EPSILON {
                    weighted_rgb[0] += ancestor_irradiance_rgb[0] as f32 / 255.0 * ancestor_support;
                    weighted_rgb[1] += ancestor_irradiance_rgb[1] as f32 / 255.0 * ancestor_support;
                    weighted_rgb[2] += ancestor_irradiance_rgb[2] as f32 / 255.0 * ancestor_support;
                    total_support += ancestor_support;
                }
            }
            ancestor_depth += 1;
            current_probe_id = parent_probe_id;
        }

        if total_support <= f32::EPSILON {
            return None;
        }

        Some(HybridGiResolveRuntime::pack_rgb_and_weight(
            [
                weighted_rgb[0] / total_support,
                weighted_rgb[1] / total_support,
                weighted_rgb[2] / total_support,
            ],
            (total_support * DIRECT_LINEAGE_IRRADIANCE_WEIGHT_SCALE).clamp(0.0, 0.75),
        ))
    }

    fn direct_lineage_rt_lighting_fallback(&self, probe_id: u32) -> Option<[u8; 4]> {
        if self.resident_slots.contains_key(&probe_id) {
            return None;
        }

        let direct_lineage_support = self
            .effective_lineage_trace_support_score(probe_id)
            .clamp(0.0, 1.5);
        if direct_lineage_support <= f32::EPSILON {
            return None;
        }

        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;

        if let Some(direct_rt_lighting_rgb) = self.probe_rt_lighting_rgb.get(&probe_id).copied() {
            let direct_support = direct_lineage_support
                * budget_weight(
                    self.probe_ray_budgets
                        .get(&probe_id)
                        .copied()
                        .unwrap_or_default(),
                )
                * runtime_rgb_intensity(direct_rt_lighting_rgb);
            if direct_support > f32::EPSILON {
                weighted_rgb[0] += direct_rt_lighting_rgb[0] as f32 / 255.0 * direct_support;
                weighted_rgb[1] += direct_rt_lighting_rgb[1] as f32 / 255.0 * direct_support;
                weighted_rgb[2] += direct_rt_lighting_rgb[2] as f32 / 255.0 * direct_support;
                total_support += direct_support;
            }
        }

        let mut current_probe_id = probe_id;
        let mut ancestor_depth = 0usize;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);
        while let Some(parent_probe_id) = self.probe_parent_probes.get(&current_probe_id).copied() {
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            if let Some(ancestor_rt_lighting_rgb) =
                self.probe_rt_lighting_rgb.get(&parent_probe_id).copied()
            {
                let ancestor_support = direct_lineage_support
                    * ANCESTOR_TRACE_INHERITANCE_FALLOFF.powi(ancestor_depth as i32)
                    * budget_weight(
                        self.probe_ray_budgets
                            .get(&parent_probe_id)
                            .copied()
                            .unwrap_or_default(),
                    )
                    * runtime_rgb_intensity(ancestor_rt_lighting_rgb);
                if ancestor_support > f32::EPSILON {
                    weighted_rgb[0] += ancestor_rt_lighting_rgb[0] as f32 / 255.0 * ancestor_support;
                    weighted_rgb[1] += ancestor_rt_lighting_rgb[1] as f32 / 255.0 * ancestor_support;
                    weighted_rgb[2] += ancestor_rt_lighting_rgb[2] as f32 / 255.0 * ancestor_support;
                    total_support += ancestor_support;
                }
            }
            ancestor_depth += 1;
            current_probe_id = parent_probe_id;
        }

        if total_support <= f32::EPSILON {
            return None;
        }

        Some(HybridGiResolveRuntime::pack_rgb_and_weight(
            [
                weighted_rgb[0] / total_support,
                weighted_rgb[1] / total_support,
                weighted_rgb[2] / total_support,
            ],
            (total_support * DIRECT_LINEAGE_TRACE_WEIGHT_SCALE).clamp(0.0, 0.75),
        ))
    }

    fn standalone_direct_rt_lighting_fallback(&self, probe_id: u32) -> Option<[u8; 4]> {
        if self.resident_slots.contains_key(&probe_id) || self.probe_parent_probes.contains_key(&probe_id) {
            return None;
        }

        let direct_rt_lighting_rgb = self.probe_rt_lighting_rgb.get(&probe_id).copied()?;
        let support = budget_weight(
            self.probe_ray_budgets
                .get(&probe_id)
                .copied()
                .unwrap_or_default(),
        ) * runtime_rgb_intensity(direct_rt_lighting_rgb);
        if support <= f32::EPSILON {
            return None;
        }

        Some(HybridGiResolveRuntime::pack_rgb_and_weight(
            [
                direct_rt_lighting_rgb[0] as f32 / 255.0,
                direct_rt_lighting_rgb[1] as f32 / 255.0,
                direct_rt_lighting_rgb[2] as f32 / 255.0,
            ],
            (support * STANDALONE_DIRECT_TRACE_WEIGHT_SCALE).clamp(0.05, 0.45),
        ))
    }

    fn descendant_lineage_irradiance_fallback(&self, probe_id: u32) -> Option<[u8; 4]> {
        let lineage_support = self
            .effective_lineage_trace_support_score(probe_id)
            .clamp(0.0, 1.5);
        if lineage_support <= f32::EPSILON {
            return None;
        }

        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;
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

            if let Some(descendant_irradiance_rgb) =
                self.probe_irradiance_rgb.get(&candidate_probe_id).copied()
            {
                let support = lineage_support
                    * DESCENDANT_LINEAGE_IRRADIANCE_FALLOFF.powi((depth - 1) as i32)
                    * budget_weight(
                        self.probe_ray_budgets
                            .get(&candidate_probe_id)
                            .copied()
                            .unwrap_or_default(),
                    );
                if support > f32::EPSILON {
                    weighted_rgb[0] += descendant_irradiance_rgb[0] as f32 / 255.0 * support;
                    weighted_rgb[1] += descendant_irradiance_rgb[1] as f32 / 255.0 * support;
                    weighted_rgb[2] += descendant_irradiance_rgb[2] as f32 / 255.0 * support;
                    total_support += support;
                }
            }

            stack.extend(self.probe_parent_probes.iter().filter_map(
                |(&grandchild_probe_id, &parent_probe_id)| {
                    (parent_probe_id == candidate_probe_id)
                        .then_some((grandchild_probe_id, depth + 1))
                },
            ));
        }

        if total_support <= f32::EPSILON {
            return None;
        }

        Some(HybridGiResolveRuntime::pack_rgb_and_weight(
            [
                weighted_rgb[0] / total_support,
                weighted_rgb[1] / total_support,
                weighted_rgb[2] / total_support,
            ],
            (total_support * DIRECT_LINEAGE_IRRADIANCE_WEIGHT_SCALE).clamp(0.0, 0.75),
        ))
    }

    fn descendant_lineage_rt_lighting_fallback(&self, probe_id: u32) -> Option<[u8; 4]> {
        let lineage_support = self
            .effective_lineage_trace_support_score(probe_id)
            .clamp(0.0, 1.5);
        if lineage_support <= f32::EPSILON {
            return None;
        }

        let mut weighted_rgb = [0.0_f32; 3];
        let mut total_support = 0.0_f32;
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

            if let Some(descendant_rt_lighting_rgb) =
                self.probe_rt_lighting_rgb.get(&candidate_probe_id).copied()
            {
                let support = lineage_support
                    * DESCENDANT_LINEAGE_TRACE_FALLOFF.powi((depth - 1) as i32)
                    * budget_weight(
                        self.probe_ray_budgets
                            .get(&candidate_probe_id)
                            .copied()
                            .unwrap_or_default(),
                    )
                    * runtime_rgb_intensity(descendant_rt_lighting_rgb);
                if support > f32::EPSILON {
                    weighted_rgb[0] += descendant_rt_lighting_rgb[0] as f32 / 255.0 * support;
                    weighted_rgb[1] += descendant_rt_lighting_rgb[1] as f32 / 255.0 * support;
                    weighted_rgb[2] += descendant_rt_lighting_rgb[2] as f32 / 255.0 * support;
                    total_support += support;
                }
            }

            stack.extend(self.probe_parent_probes.iter().filter_map(
                |(&grandchild_probe_id, &parent_probe_id)| {
                    (parent_probe_id == candidate_probe_id)
                        .then_some((grandchild_probe_id, depth + 1))
                },
            ));
        }

        if total_support <= f32::EPSILON {
            return None;
        }

        Some(HybridGiResolveRuntime::pack_rgb_and_weight(
            [
                weighted_rgb[0] / total_support,
                weighted_rgb[1] / total_support,
                weighted_rgb[2] / total_support,
            ],
            (total_support * DIRECT_LINEAGE_TRACE_WEIGHT_SCALE).clamp(0.0, 0.75),
        ))
    }

    fn resident_descendant_count(&self, probe_id: u32) -> usize {
        let mut count = 0usize;
        let mut stack = self
            .probe_parent_probes
            .iter()
            .filter_map(|(&candidate_probe_id, &parent_probe_id)| {
                (parent_probe_id == probe_id).then_some(candidate_probe_id)
            })
            .collect::<Vec<_>>();
        let mut visited_probe_ids = BTreeSet::new();

        while let Some(candidate_probe_id) = stack.pop() {
            if !visited_probe_ids.insert(candidate_probe_id) {
                continue;
            }
            if self.resident_slots.contains_key(&candidate_probe_id) {
                count += 1;
            }
            stack.extend(self.probe_parent_probes.iter().filter_map(
                |(&grandchild_probe_id, &parent_probe_id)| {
                    (parent_probe_id == candidate_probe_id).then_some(grandchild_probe_id)
                },
            ));
        }

        count
    }

    fn resident_parent_depth(&self, probe_id: u32) -> usize {
        let mut depth = 0usize;
        let mut current_probe_id = probe_id;

        while let Some(parent_probe_id) = self.probe_parent_probes.get(&current_probe_id).copied() {
            if self.resident_slots.contains_key(&parent_probe_id) {
                depth += 1;
            }
            current_probe_id = parent_probe_id;
        }

        depth
    }

    fn farther_resident_ancestor_budget_support(&self, probe_id: u32) -> f32 {
        let mut current_probe_id = probe_id;
        let mut resident_ancestor_count = 0usize;
        let mut total_support = 0.0_f32;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);

        while let Some(parent_probe_id) = self.probe_parent_probes.get(&current_probe_id).copied() {
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            if self.resident_slots.contains_key(&parent_probe_id) {
                resident_ancestor_count += 1;
                if resident_ancestor_count > 1 {
                    let farther_ancestor_depth = resident_ancestor_count - 2;
                    total_support += FARTHER_ANCESTOR_BUDGET_FALLOFF
                        .powi(farther_ancestor_depth as i32)
                        * budget_weight(
                            self.probe_ray_budgets
                                .get(&parent_probe_id)
                                .copied()
                                .unwrap_or_default(),
                        );
                }
            }
            current_probe_id = parent_probe_id;
        }

        total_support.clamp(0.0, 1.5)
    }
}

fn budget_weight(ray_budget: u32) -> f32 {
    (ray_budget.min(256) as f32 / 256.0).clamp(0.0, 1.0)
}

fn runtime_rgb_intensity(rgb: [u8; 3]) -> f32 {
    let max_component = rgb[0].max(rgb[1]).max(rgb[2]) as f32 / 255.0;
    max_component.clamp(0.0, 1.0)
}
