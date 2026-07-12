use std::collections::{HashMap, HashSet, VecDeque};

use zircon_runtime::asset::NavMeshLinkCapacity;

#[derive(Clone, Debug, Default)]
pub(super) struct BridgeCapacityRuntime {
    groups: HashMap<u64, CapacityGroup>,
}

#[derive(Clone, Debug)]
struct CapacityGroup {
    limit: usize,
    active: HashSet<u64>,
    queue: VecDeque<u64>,
}

impl BridgeCapacityRuntime {
    pub(super) fn try_acquire(
        &mut self,
        capacity: &NavMeshLinkCapacity,
        agent_entity: u64,
    ) -> bool {
        let NavMeshLinkCapacity::Shared { group, limit } = capacity else {
            return true;
        };
        let group_state = self.groups.entry(*group).or_insert_with(|| CapacityGroup {
            limit: (*limit).max(1) as usize,
            active: HashSet::new(),
            queue: VecDeque::new(),
        });
        group_state.limit = (*limit).max(1) as usize;
        if group_state.active.contains(&agent_entity) {
            return true;
        }
        if !group_state.queue.contains(&agent_entity) {
            group_state.queue.push_back(agent_entity);
        }
        if group_state.active.len() >= group_state.limit
            || group_state.queue.front().copied() != Some(agent_entity)
        {
            return false;
        }
        group_state.queue.pop_front();
        group_state.active.insert(agent_entity);
        true
    }

    pub(super) fn release(&mut self, capacity: &NavMeshLinkCapacity, agent_entity: u64) {
        let NavMeshLinkCapacity::Shared { group, .. } = capacity else {
            return;
        };
        let remove_group = if let Some(group_state) = self.groups.get_mut(group) {
            group_state.active.remove(&agent_entity);
            group_state.queue.retain(|queued| *queued != agent_entity);
            group_state.active.is_empty() && group_state.queue.is_empty()
        } else {
            false
        };
        if remove_group {
            self.groups.remove(group);
        }
    }

    pub(super) fn retain_agents(&mut self, active_agents: &HashSet<u64>) {
        self.groups.retain(|_, group| {
            group.active.retain(|agent| active_agents.contains(agent));
            group.queue.retain(|agent| active_agents.contains(agent));
            !group.active.is_empty() || !group.queue.is_empty()
        });
    }

    pub(super) fn queued_agents(&self) -> usize {
        self.groups.values().map(|group| group.queue.len()).sum()
    }
}
