use std::collections::BTreeSet;

use super::resolve_runtime::HybridGiResolveRuntime;

impl HybridGiResolveRuntime {
    pub fn parent_probe_id(&self, probe_id: u32) -> Option<u32> {
        self.probe_parent_probes.get(&probe_id).copied()
    }

    pub fn has_parent_topology(&self) -> bool {
        !self.probe_parent_probes.is_empty()
    }

    #[cfg(test)]
    pub fn parent_probe_count(&self) -> usize {
        self.probe_parent_probes.len()
    }

    #[cfg(test)]
    pub fn replace_probe_parent_probes_for_test(
        &mut self,
        probe_parent_probes: std::collections::BTreeMap<u32, u32>,
    ) {
        self.probe_parent_probes = probe_parent_probes;
    }

    pub fn parent_probe_chain(&self, probe_id: u32) -> Vec<(u32, usize)> {
        let mut chain = Vec::new();
        let mut current_probe_id = probe_id;
        let mut visited_probe_ids = BTreeSet::from([probe_id]);
        let mut depth = 0usize;

        while let Some(parent_probe_id) = self.parent_probe_id(current_probe_id) {
            if !visited_probe_ids.insert(parent_probe_id) {
                break;
            }
            depth += 1;
            chain.push((parent_probe_id, depth));
            current_probe_id = parent_probe_id;
        }

        chain
    }

    pub fn descendant_probe_chain(&self, probe_id: u32) -> Vec<(u32, usize)> {
        let mut chain = Vec::new();
        let mut stack = self
            .probe_parent_probes
            .iter()
            .filter_map(|(&candidate_probe_id, &parent_probe_id)| {
                (parent_probe_id == probe_id).then_some((candidate_probe_id, 1usize))
            })
            .collect::<Vec<_>>();
        let mut visited_probe_ids = BTreeSet::from([probe_id]);

        while let Some((candidate_probe_id, depth)) = stack.pop() {
            if !visited_probe_ids.insert(candidate_probe_id) {
                continue;
            }

            chain.push((candidate_probe_id, depth));
            stack.extend(self.probe_parent_probes.iter().filter_map(
                |(&grandchild_probe_id, &parent_probe_id)| {
                    (parent_probe_id == candidate_probe_id)
                        .then_some((grandchild_probe_id, depth + 1))
                },
            ));
        }

        chain
    }
}
