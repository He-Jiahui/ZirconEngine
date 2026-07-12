use zircon_plugin_navigation_recast::RecastCrowdAgentState;
use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NavAgentWritebackMode, NavDesiredVelocity, NavMeshAgentDescriptor,
    NAV_DESIRED_VELOCITY_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Real, Transform, Vec3};
use zircon_runtime::scene::World;

use crate::manager::agent_motion::rotate_toward_movement;

pub(super) fn write_agent_state(
    world: &mut World,
    entity: u64,
    agent: &NavMeshAgentDescriptor,
    state: &RecastCrowdAgentState,
    dt_seconds: Real,
    report: &mut NavAgentTickReport,
) {
    match agent.writeback_mode {
        NavAgentWritebackMode::Transform => {
            if !agent.update_position {
                return;
            }
            write_transform(world, entity, agent, state, dt_seconds, report);
        }
        NavAgentWritebackMode::DesiredVelocity => {
            let desired = NavDesiredVelocity {
                linear: state.avoidance_velocity,
            };
            let value = match serde_json::to_value(desired) {
                Ok(value) => value,
                Err(error) => {
                    report.blocked_agents += 1;
                    report.diagnostics.push(format!(
                        "agent {entity} desired velocity serialization failed: {error}"
                    ));
                    return;
                }
            };
            match world.set_dynamic_component(entity, NAV_DESIRED_VELOCITY_COMPONENT_TYPE, value) {
                Ok(changed) => report.moved_agents += usize::from(changed),
                Err(error) => {
                    report.blocked_agents += 1;
                    report.diagnostics.push(format!(
                        "agent {entity} desired velocity writeback failed: {error}"
                    ));
                }
            }
        }
    }
}

fn write_transform(
    world: &mut World,
    entity: u64,
    agent: &NavMeshAgentDescriptor,
    state: &RecastCrowdAgentState,
    dt_seconds: Real,
    report: &mut NavAgentTickReport,
) {
    let Some(transform) = world.world_transform(entity) else {
        report.blocked_agents += 1;
        report
            .diagnostics
            .push(format!("agent {entity} has no world transform"));
        return;
    };
    let movement = Vec3::from_array(state.velocity);
    let updated = Transform {
        translation: Vec3::from_array(state.position),
        rotation: if agent.update_rotation && movement.length_squared() > Real::EPSILON {
            rotate_toward_movement(transform.rotation, movement, agent, dt_seconds)
        } else {
            transform.rotation
        },
        ..transform
    };
    match world.update_transform(entity, updated) {
        Ok(changed) => report.moved_agents += usize::from(changed),
        Err(error) => {
            report.blocked_agents += 1;
            report
                .diagnostics
                .push(format!("agent {entity} could not move: {error}"));
        }
    }
}
