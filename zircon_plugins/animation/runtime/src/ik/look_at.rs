use zircon_runtime::core::math::{Quat, Real, Vec3};

use super::AnimationIkError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LookAtJob {
    target_direction: Vec3,
    local_axis: Vec3,
    clamp_degrees: Real,
    weight: Real,
}

impl LookAtJob {
    pub const fn new(target_direction: Vec3, local_axis: Vec3) -> Self {
        Self {
            target_direction,
            local_axis,
            clamp_degrees: 180.0,
            weight: 1.0,
        }
    }

    pub const fn with_clamp_degrees(mut self, clamp_degrees: Real) -> Self {
        self.clamp_degrees = clamp_degrees;
        self
    }

    pub const fn with_weight(mut self, weight: Real) -> Self {
        self.weight = weight;
        self
    }

    pub fn solve_rotation(self, current: Quat) -> Result<Quat, AnimationIkError> {
        validate(self, current)?;
        if self.weight == 0.0 {
            return Ok(current.normalize());
        }
        let current_axis = current * self.local_axis.normalize();
        let target = self.target_direction.normalize();
        let full_delta = Quat::from_rotation_arc(current_axis, target);
        let angle = full_delta.angle_between(Quat::IDENTITY);
        let limit = self
            .clamp_degrees
            .to_radians()
            .clamp(0.0, std::f32::consts::PI);
        let fraction = if angle <= Real::EPSILON {
            0.0
        } else {
            (limit / angle).min(1.0)
        };
        let delta = Quat::IDENTITY.slerp(full_delta, fraction * self.weight);
        Ok((delta * current).normalize())
    }
}

fn validate(job: LookAtJob, current: Quat) -> Result<(), AnimationIkError> {
    if !job.target_direction.is_finite()
        || !job.local_axis.is_finite()
        || !current.is_finite()
        || !job.clamp_degrees.is_finite()
    {
        return Err(AnimationIkError::NonFiniteInput);
    }
    if job.target_direction.length_squared() <= Real::EPSILON
        || job.local_axis.length_squared() <= Real::EPSILON
    {
        return Err(AnimationIkError::DegenerateAxis);
    }
    if !job.weight.is_finite() || !(0.0..=1.0).contains(&job.weight) {
        return Err(AnimationIkError::InvalidWeight);
    }
    Ok(())
}

#[cfg(test)]
#[path = "look_at/performance_tests.rs"]
mod optimization_batch_20260830ct_tests;
