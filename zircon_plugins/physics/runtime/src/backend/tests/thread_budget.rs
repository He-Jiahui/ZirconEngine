use std::time::Instant;

use zircon_runtime::core::framework::physics::{PhysicsColliderShape, PhysicsSettings};
use zircon_runtime::core::framework::scene::physics::{
    PhysicsMaterialMetadata, PhysicsSleepPolicy,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::Vec3;
use zircon_runtime::core::runtime::tasks::EngineThreadBudget;

use crate::backend::{JoltPhysicsBackend, PhysicsBackend};

#[test]
fn jolt_zero_worker_budget_advances_dynamic_body() {
    let budget = EngineThreadBudget::for_parallelism(4);
    assert_eq!(budget.physics_worker_threads(), 0);
    let mut backend =
        JoltPhysicsBackend::new_with_thread_budget(PhysicsSettings::default(), budget)
            .expect("initialize Jolt with caller-thread jobs");
    let shape = backend
        .create_shape(
            &PhysicsColliderShape::Sphere { radius: 0.5 },
            &PhysicsMaterialMetadata::default(),
        )
        .unwrap();
    let body = backend
        .create_body(&super::body_desc(WorldHandle::new(410), 410, shape))
        .unwrap();
    backend.step(1.0 / 60.0).unwrap();
    let mut active = Vec::new();
    backend.read_active_states(&mut active);
    let state = active.iter().find(|(handle, _)| *handle == body).unwrap();
    assert!(state.1.transform.translation.y < 2.0);
    assert!(state.1.transform.translation.y.is_finite());
}

#[test]
#[ignore = "explicit native physics-step timing sample; no timing pass threshold"]
fn jolt_budgeted_physics_step_timing_sample() {
    const BODIES: usize = 512;
    const SAMPLES: usize = 240;
    for hardware_threads in [4, 16] {
        let budget = EngineThreadBudget::for_parallelism(hardware_threads);
        let mut backend =
            JoltPhysicsBackend::new_with_thread_budget(PhysicsSettings::default(), budget).unwrap();
        let shape = backend
            .create_shape(
                &PhysicsColliderShape::Sphere { radius: 0.5 },
                &PhysicsMaterialMetadata::default(),
            )
            .unwrap();
        for entity in 0..BODIES {
            let mut desc = super::body_desc(WorldHandle::new(411), entity as u64 + 1, shape);
            desc.body.transform.translation =
                Vec3::new((entity % 32) as f32 * 3.0, (entity / 32) as f32 * 3.0, 0.0);
            desc.collider.transform = desc.body.transform;
            desc.body.gravity_scale = 0.0;
            desc.body.linear_velocity = [0.1, 0.0, 0.0];
            desc.body.sleep_policy = PhysicsSleepPolicy::Never;
            backend.create_body(&desc).unwrap();
        }
        for _ in 0..30 {
            backend.step(1.0 / 60.0).unwrap();
        }
        let mut samples = Vec::with_capacity(SAMPLES);
        for _ in 0..SAMPLES {
            let started = Instant::now();
            backend.step(1.0 / 60.0).unwrap();
            samples.push(started.elapsed().as_micros());
        }
        let mut active = Vec::new();
        backend.read_active_states(&mut active);
        assert_eq!(active.len(), BODIES);
        assert!(active
            .iter()
            .all(|(_, state)| state.transform.translation.is_finite()));
        samples.sort_unstable();
        println!(
            "PHYSICS_STEP_TIMING hardware_plan={} task_workers={} jolt_workers={} planned_threads={} bodies={} warmup=30 samples={} p50_us={} p95_us={} max_us={}",
            budget.hardware_threads(), budget.task_worker_threads(), budget.physics_worker_threads(), budget.planned_threads(),
            BODIES, SAMPLES, samples[SAMPLES.div_ceil(2) - 1], samples[(SAMPLES * 95).div_ceil(100) - 1], samples[SAMPLES - 1],
        );
    }
}
