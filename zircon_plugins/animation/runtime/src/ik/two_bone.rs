use zircon_runtime::core::math::{Real, Vec3};

use super::AnimationIkError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwoBoneIkSolution {
    pub root: Vec3,
    pub mid: Vec3,
    pub tip: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwoBoneIkJob {
    target: Vec3,
    pole: Option<Vec3>,
    weight: Real,
}

impl TwoBoneIkJob {
    pub const fn new(target: Vec3) -> Self {
        Self {
            target,
            pole: None,
            weight: 1.0,
        }
    }

    pub const fn with_pole(mut self, pole: Vec3) -> Self {
        self.pole = Some(pole);
        self
    }

    pub const fn with_weight(mut self, weight: Real) -> Self {
        self.weight = weight;
        self
    }

    pub fn solve_positions(
        self,
        root: Vec3,
        mid: Vec3,
        tip: Vec3,
    ) -> Result<TwoBoneIkSolution, AnimationIkError> {
        validate(self, root, mid, tip)?;
        let upper = (mid - root).length();
        let lower = (tip - mid).length();
        if upper <= Real::EPSILON || lower <= Real::EPSILON {
            return Err(AnimationIkError::DegenerateChain);
        }
        if self.weight == 0.0 {
            return Ok(TwoBoneIkSolution { root, mid, tip });
        }
        let target_delta = self.target - root;
        let target_distance = target_delta.length();
        let direction = target_delta.try_normalize().unwrap_or(Vec3::X);
        let distance = target_distance.clamp((upper - lower).abs(), upper + lower);
        let pole = self.pole.unwrap_or(mid - root);
        let bend = bend_direction(direction, pole, mid - root);
        let along = ((upper * upper + distance * distance - lower * lower)
            / (2.0 * distance.max(Real::EPSILON)))
        .clamp(-upper, upper);
        let height = (upper * upper - along * along).max(0.0).sqrt();
        let solved_mid = root + direction * along + bend * height;
        let solved_tip = root + direction * distance;
        Ok(TwoBoneIkSolution {
            root,
            mid: mid.lerp(solved_mid, self.weight),
            tip: tip.lerp(solved_tip, self.weight),
        })
    }
}

fn bend_direction(direction: Vec3, pole: Vec3, fallback: Vec3) -> Vec3 {
    let projected = pole - direction * pole.dot(direction);
    projected
        .try_normalize()
        .or_else(|| {
            let projected = fallback - direction * fallback.dot(direction);
            projected.try_normalize()
        })
        .unwrap_or_else(|| direction.any_orthonormal_vector())
}

fn validate(job: TwoBoneIkJob, root: Vec3, mid: Vec3, tip: Vec3) -> Result<(), AnimationIkError> {
    if !job.target.is_finite()
        || job.pole.is_some_and(|pole| !pole.is_finite())
        || !root.is_finite()
        || !mid.is_finite()
        || !tip.is_finite()
    {
        return Err(AnimationIkError::NonFiniteInput);
    }
    if !job.weight.is_finite() || !(0.0..=1.0).contains(&job.weight) {
        return Err(AnimationIkError::InvalidWeight);
    }
    Ok(())
}

#[cfg(test)]
#[path = "two_bone/performance_tests.rs"]
mod optimization_batch_20260830ct_tests;
