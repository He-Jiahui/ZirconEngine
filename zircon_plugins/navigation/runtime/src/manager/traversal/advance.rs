use zircon_runtime::core::framework::navigation::{
    NavLinkMotion, NavMeshAgentDescriptor, OffMeshTraverseEvent, OffMeshTraversePhase,
};
use zircon_runtime::core::math::{Real, Vec3};

use super::state::OffMeshTraversalRuntime;

const MIN_OFF_MESH_AGENT_RADIUS: Real = 0.05;
const OFF_MESH_ENTRY_TRIGGER_RADIUS_MULTIPLIER: Real = 2.25;
const MIN_OFF_MESH_TRAVERSAL_SPEED: Real = 0.1;
const MIN_OFF_MESH_TRAVERSAL_DURATION_SECONDS: Real = 0.01;
const NORMALIZED_PARABOLA_PEAK_SCALE: Real = 4.0;

#[derive(Clone, Debug)]
pub(in crate::manager) struct DirectTraversalPosition {
    pub(in crate::manager) position: Vec3,
    pub(in crate::manager) completed: bool,
}

#[derive(Clone, Debug, Default)]
pub(in crate::manager) struct ActiveTraversalStep {
    pub(in crate::manager) movement_target: Option<Vec3>,
    pub(in crate::manager) direct_position: Option<DirectTraversalPosition>,
    pub(in crate::manager) hold_for_capacity: bool,
    pub(in crate::manager) event: Option<OffMeshTraverseEvent>,
}

impl OffMeshTraversalRuntime {
    pub(super) fn advance(
        &mut self,
        agent_entity: u64,
        current: Vec3,
        agent: &NavMeshAgentDescriptor,
        dt_seconds: Real,
    ) -> Option<ActiveTraversalStep> {
        let mut active = self.active.remove(&agent_entity)?;
        let mut step = ActiveTraversalStep::default();
        match active.contract.phase {
            OffMeshTraversePhase::Approach => {
                let start = Vec3::from_array(active.contract.start);
                let trigger_radius = agent.radius.max(MIN_OFF_MESH_AGENT_RADIUS)
                    * OFF_MESH_ENTRY_TRIGGER_RADIUS_MULTIPLIER;
                if current.distance_squared(start) > trigger_radius * trigger_radius {
                    step.movement_target = Some(start);
                } else if self.capacity.try_acquire(&active.capacity, agent_entity) {
                    active.capacity_acquired = true;
                    active.contract.phase = OffMeshTraversePhase::Traverse;
                    active.contract.progress = 0.0;
                    step.direct_position = Some(DirectTraversalPosition {
                        position: start,
                        completed: false,
                    });
                    step.event = Some(OffMeshTraverseEvent::started(&active.contract));
                } else {
                    step.hold_for_capacity = true;
                }
            }
            OffMeshTraversePhase::Traverse => {
                let start = Vec3::from_array(active.contract.start);
                let end = Vec3::from_array(active.contract.end);
                let distance = start.distance(end);
                let duration = distance / agent.speed.max(MIN_OFF_MESH_TRAVERSAL_SPEED);
                active.contract.progress = (active.contract.progress
                    + dt_seconds / duration.max(MIN_OFF_MESH_TRAVERSAL_DURATION_SECONDS))
                .min(1.0);
                step.direct_position = Some(DirectTraversalPosition {
                    position: traversal_position(
                        start,
                        end,
                        active.contract.progress,
                        active.motion,
                        active.arc_height,
                    ),
                    completed: false,
                });
                if active.contract.progress >= 1.0 {
                    active.contract.phase = OffMeshTraversePhase::Exit;
                }
            }
            OffMeshTraversePhase::Exit => {
                let end = Vec3::from_array(active.contract.end);
                step.direct_position = Some(DirectTraversalPosition {
                    position: end,
                    completed: true,
                });
                step.event = Some(OffMeshTraverseEvent::completed(&active.contract));
                if active.capacity_acquired {
                    self.capacity.release(&active.capacity, agent_entity);
                }
                return Some(step);
            }
        }
        self.active.insert(agent_entity, active);
        Some(step)
    }
}

fn traversal_position(
    start: Vec3,
    end: Vec3,
    progress: Real,
    motion: NavLinkMotion,
    arc_height: Real,
) -> Vec3 {
    let mut position = start.lerp(end, progress);
    if matches!(motion, NavLinkMotion::Parabolic) {
        position.y +=
            NORMALIZED_PARABOLA_PEAK_SCALE * arc_height.max(0.0) * progress * (1.0 - progress);
    }
    position
}
