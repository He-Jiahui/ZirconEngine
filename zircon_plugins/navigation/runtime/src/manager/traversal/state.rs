use std::collections::{HashMap, HashSet};

use zircon_runtime::core::framework::navigation::{
    NavLinkMotion, NavMeshHandle, OffMeshTraversePhase, OffMeshTraverseState,
};
use zircon_runtime::core::framework::navigation::{NavMeshLinkAsset, NavMeshLinkCapacity};
use zircon_runtime::core::math::{Real, Vec3};

use super::capacity::BridgeCapacityRuntime;

#[derive(Clone, Debug, Default)]
pub(in crate::manager) struct OffMeshTraversalRuntime {
    pub(super) active: HashMap<u64, ActiveTraversal>,
    pub(super) capacity: BridgeCapacityRuntime,
}

#[derive(Clone, Debug)]
pub(super) struct ActiveTraversal {
    pub(super) contract: OffMeshTraverseState,
    pub(super) capacity: NavMeshLinkCapacity,
    pub(super) capacity_acquired: bool,
    pub(super) motion: NavLinkMotion,
    pub(super) arc_height: Real,
}

impl OffMeshTraversalRuntime {
    pub(super) fn begin(
        &mut self,
        agent_entity: u64,
        nav_mesh: NavMeshHandle,
        link: &NavMeshLinkAsset,
        current: Vec3,
    ) -> Option<Vec3> {
        if let Some(active) = self.active.get(&agent_entity) {
            return Some(Vec3::from_array(active.contract.start));
        }
        let start = Vec3::from_array(link.start);
        let end = Vec3::from_array(link.end);
        let reverse =
            link.bidirectional && current.distance_squared(end) < current.distance_squared(start);
        let (start, end) = if reverse { (end, start) } else { (start, end) };
        if start.distance_squared(end) <= Real::EPSILON {
            return None;
        }
        self.active.insert(
            agent_entity,
            ActiveTraversal {
                contract: OffMeshTraverseState {
                    agent_entity,
                    nav_mesh,
                    link_id: link.id,
                    owner_entity: link.owner_entity,
                    phase: OffMeshTraversePhase::Approach,
                    progress: 0.0,
                    start: start.to_array(),
                    end: end.to_array(),
                },
                capacity: link.capacity.clone(),
                capacity_acquired: false,
                motion: link.motion,
                arc_height: link.arc_height.max(0.0),
            },
        );
        Some(start)
    }

    pub(super) fn clear_agent(&mut self, agent_entity: u64) {
        if let Some(active) = self.active.remove(&agent_entity) {
            self.capacity.release(&active.capacity, agent_entity);
        }
    }

    pub(super) fn retain_agents(&mut self, active_agents: &[u64]) {
        let active_agents = active_agents.iter().copied().collect::<HashSet<_>>();
        let stale = self
            .active
            .keys()
            .filter(|agent| !active_agents.contains(agent))
            .copied()
            .collect::<Vec<_>>();
        for agent in stale {
            self.clear_agent(agent);
        }
        self.capacity.retain_agents(&active_agents);
    }

    pub(super) fn traversing_agents(&self) -> usize {
        self.active
            .values()
            .filter(|active| {
                matches!(
                    active.contract.phase,
                    OffMeshTraversePhase::Traverse | OffMeshTraversePhase::Exit
                )
            })
            .count()
    }

    pub(super) fn queued_agents(&self) -> usize {
        self.capacity.queued_agents()
    }
}
