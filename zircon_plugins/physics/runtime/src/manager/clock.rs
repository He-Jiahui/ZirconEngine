use zircon_runtime::core::framework::physics::{PhysicsSettings, PhysicsWorldStepPlan};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::Real;

use crate::backend::select_runtime_backend;
use crate::manager::{DefaultPhysicsManager, PhysicsTickPlan};

use super::poison_recovery::recover_lock;

impl DefaultPhysicsManager {
    pub fn advance_clock(&self, world: WorldHandle, delta_seconds: f32) -> PhysicsTickPlan {
        const STEP_EPSILON_SCALE: f32 = 1.0e-4;

        let settings = recover_lock(&self.settings).clone();
        let step_seconds = configured_step_seconds(&settings);
        if !select_runtime_backend(&settings).allows_step(settings.simulation_mode)
            || step_seconds <= 0.0
        {
            return PhysicsWorldStepPlan {
                steps: 0,
                step_seconds,
                remaining_seconds: 0.0,
                interpolation_alpha: 0.0,
            };
        }

        let mut accumulators = recover_lock(&self.accumulators);
        let accumulator = accumulators.entry(world).or_insert(0.0);
        let delta_seconds = if delta_seconds.is_finite() {
            delta_seconds.max(0.0)
        } else {
            0.0
        };
        *accumulator += delta_seconds;

        let max_substeps = settings.max_substeps.max(1);
        let step_epsilon = step_seconds * STEP_EPSILON_SCALE;
        let mut steps = 0;
        while steps < max_substeps && *accumulator + step_epsilon >= step_seconds {
            *accumulator = (*accumulator - step_seconds).max(0.0);
            steps += 1;
        }
        if accumulator.abs() < step_epsilon {
            *accumulator = 0.0;
        }

        PhysicsWorldStepPlan {
            steps,
            step_seconds,
            remaining_seconds: *accumulator,
            interpolation_alpha: physics_step_interpolation_alpha(*accumulator, step_seconds),
        }
    }
}

pub(super) fn configured_step_seconds(settings: &PhysicsSettings) -> Real {
    if settings.fixed_hz == 0 {
        0.0
    } else {
        1.0 / settings.fixed_hz as Real
    }
}

fn physics_step_interpolation_alpha(remaining_seconds: Real, step_seconds: Real) -> Real {
    if remaining_seconds.is_finite() && step_seconds.is_finite() && step_seconds > 0.0 {
        (remaining_seconds / step_seconds).clamp(0.0, 1.0)
    } else {
        0.0
    }
}
