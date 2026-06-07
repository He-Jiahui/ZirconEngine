use std::collections::HashSet;
use std::f32::consts::{PI, TAU};

use zircon_runtime::core::framework::navigation::NavMeshAgentDescriptor;
use zircon_runtime::core::math::{Quat, Real, Vec3};

use super::DefaultNavigationManager;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct NavigationAgentMotionState {
    pub(super) velocity: Vec3,
}

impl DefaultNavigationManager {
    pub(super) fn agent_velocity(&self, entity: u64) -> Vec3 {
        self.state
            .lock()
            .expect("navigation state lock poisoned")
            .agent_motion
            .get(&entity)
            .map(|state| state.velocity)
            .unwrap_or(Vec3::ZERO)
    }

    pub(super) fn record_agent_velocity(&self, entity: u64, velocity: Vec3) {
        let mut state = self.state.lock().expect("navigation state lock poisoned");
        if velocity.length_squared() <= Real::EPSILON {
            state.agent_motion.remove(&entity);
        } else {
            state
                .agent_motion
                .insert(entity, NavigationAgentMotionState { velocity });
        }
    }

    pub(super) fn clear_agent_velocity(&self, entity: u64) {
        self.state
            .lock()
            .expect("navigation state lock poisoned")
            .agent_motion
            .remove(&entity);
    }

    pub(super) fn retain_agent_motion_for(&self, active_entities: &[u64]) {
        let active_entities = active_entities.iter().copied().collect::<HashSet<_>>();
        self.state
            .lock()
            .expect("navigation state lock poisoned")
            .agent_motion
            .retain(|entity, _| active_entities.contains(entity));
    }
}

pub(super) fn next_agent_velocity(
    previous_velocity: Vec3,
    current: Vec3,
    target: Vec3,
    agent: &NavMeshAgentDescriptor,
    dt_seconds: Real,
) -> Vec3 {
    let desired_velocity = desired_agent_velocity(current, target, agent, dt_seconds);
    let max_delta = finite_non_negative(agent.acceleration) * dt_seconds;
    if max_delta <= Real::EPSILON {
        return previous_velocity;
    }

    let delta = desired_velocity - previous_velocity;
    let delta_length = delta.length();
    if delta_length <= max_delta {
        desired_velocity
    } else {
        previous_velocity + delta.normalize_or_zero() * max_delta
    }
}

pub(super) fn agent_displacement(
    velocity: Vec3,
    current: Vec3,
    target: Vec3,
    agent: &NavMeshAgentDescriptor,
    dt_seconds: Real,
) -> Vec3 {
    let displacement = velocity * dt_seconds;
    let displacement_length = displacement.length();
    if displacement_length <= Real::EPSILON {
        return Vec3::ZERO;
    }

    let distance = (target - current).length();
    let max_travel = if agent.auto_braking {
        (distance - finite_non_negative(agent.stopping_distance)).max(0.0)
    } else {
        distance
    };
    if max_travel <= Real::EPSILON {
        Vec3::ZERO
    } else if displacement_length > max_travel {
        displacement.normalize_or_zero() * max_travel
    } else {
        displacement
    }
}

pub(super) fn realized_velocity(displacement: Vec3, dt_seconds: Real) -> Vec3 {
    if dt_seconds <= Real::EPSILON {
        Vec3::ZERO
    } else {
        displacement / dt_seconds
    }
}

pub(super) fn rotate_toward_movement(
    current_rotation: Quat,
    movement_direction: Vec3,
    agent: &NavMeshAgentDescriptor,
    dt_seconds: Real,
) -> Quat {
    let target_direction = Vec3::new(movement_direction.x, 0.0, movement_direction.z);
    if target_direction.length_squared() <= Real::EPSILON {
        return current_rotation;
    }
    let target_direction = target_direction.normalize_or_zero();
    let target_yaw = target_direction.x.atan2(-target_direction.z);
    let angular_speed = finite_non_negative(agent.angular_speed);
    if angular_speed <= Real::EPSILON {
        return current_rotation;
    }

    let current_forward = current_rotation * Vec3::NEG_Z;
    let current_forward = Vec3::new(current_forward.x, 0.0, current_forward.z);
    if current_forward.length_squared() <= Real::EPSILON {
        return Quat::from_rotation_y(target_yaw);
    }

    let current_forward = current_forward.normalize_or_zero();
    let current_yaw = current_forward.x.atan2(-current_forward.z);
    let delta = shortest_angle_delta(current_yaw, target_yaw);
    let max_step = angular_speed.to_radians() * dt_seconds;
    if delta.abs() <= max_step {
        Quat::from_rotation_y(target_yaw)
    } else {
        Quat::from_rotation_y(current_yaw + delta.signum() * max_step)
    }
}

fn desired_agent_velocity(
    current: Vec3,
    target: Vec3,
    agent: &NavMeshAgentDescriptor,
    dt_seconds: Real,
) -> Vec3 {
    let delta = target - current;
    let distance = delta.length();
    if distance <= Real::EPSILON {
        return Vec3::ZERO;
    }

    let max_speed = finite_non_negative(agent.speed);
    let desired_speed = if agent.auto_braking {
        let stop_distance = finite_non_negative(agent.stopping_distance);
        let available_distance = (distance - stop_distance).max(0.0);
        let travel_speed_limit = if dt_seconds > Real::EPSILON {
            available_distance / dt_seconds
        } else {
            0.0
        };
        let braking_speed = braking_speed_limit(agent.acceleration, available_distance);
        max_speed.min(travel_speed_limit).min(braking_speed)
    } else {
        max_speed
    };

    delta.normalize_or_zero() * desired_speed
}

fn braking_speed_limit(acceleration: Real, available_distance: Real) -> Real {
    let acceleration = finite_non_negative(acceleration);
    if acceleration <= Real::EPSILON {
        Real::INFINITY
    } else {
        (2.0 * acceleration * available_distance).sqrt()
    }
}

fn finite_non_negative(value: Real) -> Real {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn shortest_angle_delta(from: Real, to: Real) -> Real {
    let mut delta = (to - from) % TAU;
    if delta > PI {
        delta -= TAU;
    } else if delta < -PI {
        delta += TAU;
    }
    delta
}
