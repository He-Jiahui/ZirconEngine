use std::collections::{BTreeMap, BTreeSet};

use super::runtime_state::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(in crate::hybrid_gi) fn probe_child_probes(&self) -> &BTreeMap<u32, Vec<u32>> {
        &self.probe_child_probes
    }

    pub(in crate::hybrid_gi) fn rebuild_probe_child_probes(&mut self) {
        self.probe_child_probes = probe_child_probes_from_parent_probes(&self.probe_parent_probes);
    }

    pub(in crate::hybrid_gi) fn probe_descendant_ids(&self, probe_id: u32) -> Vec<u32> {
        self.probe_descendant_ids_with_depth(probe_id)
            .into_iter()
            .map(|(descendant_probe_id, _)| descendant_probe_id)
            .collect()
    }

    /// 按父节点索引的稳定顺序遍历后代；访问集截断环，避免谱系评分和待更新排序无限循环。
    pub(in crate::hybrid_gi) fn probe_descendant_ids_with_depth(
        &self,
        probe_id: u32,
    ) -> Vec<(u32, usize)> {
        let mut stack = self
            .probe_child_probes()
            .get(&probe_id)
            .into_iter()
            .flatten()
            .rev()
            .map(|probe_id| (*probe_id, 1usize))
            .collect::<Vec<_>>();
        let mut visited_probe_ids = BTreeSet::new();
        let mut descendants = Vec::new();

        while let Some((candidate_probe_id, depth)) = stack.pop() {
            if !visited_probe_ids.insert(candidate_probe_id) {
                continue;
            }

            descendants.push((candidate_probe_id, depth));
            if let Some(child_probe_ids) = self.probe_child_probes().get(&candidate_probe_id) {
                stack.extend(
                    child_probe_ids
                        .iter()
                        .rev()
                        .map(|child_probe_id| (*child_probe_id, depth + 1)),
                );
            }
        }

        descendants
    }
}

fn probe_child_probes_from_parent_probes(
    probe_parent_probes: &BTreeMap<u32, u32>,
) -> BTreeMap<u32, Vec<u32>> {
    let mut probe_child_probes = BTreeMap::<u32, Vec<u32>>::new();
    for (&probe_id, &parent_probe_id) in probe_parent_probes {
        probe_child_probes
            .entry(parent_probe_id)
            .or_default()
            .push(probe_id);
    }
    probe_child_probes
}

#[cfg(test)]
#[path = "tests/probe_topology.rs"]
mod tests;
