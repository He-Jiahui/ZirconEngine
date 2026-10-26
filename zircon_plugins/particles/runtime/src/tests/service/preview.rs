use zircon_runtime::core::math::{Real, Vec3};

use crate::{
    ParticleEmitterAsset, ParticlePreviewClock, ParticlePreviewError, ParticleScalarRange,
    ParticleSystemAsset, ParticleVec3Range,
};

use super::super::*;

fn component(entity: EntityId, time_scale: Real, playing: bool) -> ParticleSystemComponent {
    let mut emitter = ParticleEmitterAsset::sprite("preview")
        .with_spawn_rate(8.0)
        .with_max_particles(64)
        .with_lifetime(ParticleScalarRange::constant(4.0))
        .with_initial_velocity(ParticleVec3Range::constant(Vec3::X));
    emitter.drag = 0.75;
    ParticleSystemComponent::new(
        entity,
        ParticleSystemAsset::new("preview")
            .with_seed(41)
            .with_emitters(vec![emitter]),
    )
    .with_time_scale(time_scale)
    .with_playing(playing)
}

fn assert_state_unchanged(before: &ParticlesManagerState, after: &ParticlesManagerState) {
    assert_eq!(after.instances, before.instances);
    assert_eq!(after.next_handle, before.next_handle);
    assert_eq!(after.diagnostics, before.diagnostics);
    assert_eq!(
        after.next_diagnostic_sequence,
        before.next_diagnostic_sequence
    );
    assert_eq!(after.dropped_diagnostics, before.dropped_diagnostics);
    assert_eq!(after.diagnostics_dirty, before.diagnostics_dirty);
    assert_eq!(after.last_gpu_feedback, before.last_gpu_feedback);
    assert_eq!(after.capabilities, before.capabilities);
    assert_eq!(after.cached_snapshot, before.cached_snapshot);
    assert!(Arc::ptr_eq(
        &after.diagnostics_snapshot,
        &before.diagnostics_snapshot
    ));
    assert!(Arc::ptr_eq(
        &after.cached_snapshot.as_ref().unwrap().sprites,
        &before.cached_snapshot.as_ref().unwrap().sprites
    ));
}

fn assert_step_rejected(
    manager: &ParticlesManager,
    handle: ParticleEmitterHandle,
    dt: Real,
    error: ParticlePreviewError,
) {
    manager.snapshot();
    let before = manager.lock_state().clone();
    assert_eq!(manager.step_preview(handle, dt), Err(error));
    assert_state_unchanged(&before, &manager.lock_state());
}

#[test]
fn particle_preview_clock_reads_actual_instance_without_invalidating_snapshot() {
    let manager = ParticlesManager::default();
    let first = manager.instantiate(component(1, 2.0, true)).unwrap();
    let second = manager.instantiate(component(2, 0.5, false)).unwrap();
    manager.tick(0.25).unwrap();
    manager.snapshot();
    let before = manager.lock_state().clone();

    assert_eq!(
        manager.preview_clock(first).unwrap(),
        ParticlePreviewClock {
            handle: first,
            age_seconds: 0.5,
            playing: true,
            time_scale: 2.0,
            backend: ParticleSimulationBackend::Cpu,
            fallback_to_cpu: false,
        }
    );
    assert_eq!(manager.preview_clock(second).unwrap().age_seconds, 0.0);
    assert!(!manager.preview_clock(second).unwrap().playing);
    assert_state_unchanged(&before, &manager.lock_state());
}

#[test]
fn particle_preview_step_paused_instance_matches_one_tick_and_preserves_other_handle() {
    let manager = ParticlesManager::default();
    let reference = ParticlesManager::default();
    let target_component = component(1, 2.0, true);
    let first = manager.instantiate(target_component.clone()).unwrap();
    let second = manager.instantiate(component(2, 0.5, true)).unwrap();
    let reference_handle = reference.instantiate(target_component).unwrap();
    assert_eq!(first, reference_handle);
    manager.tick(0.25).unwrap();
    reference.tick(0.25).unwrap();
    manager.pause(first).unwrap();
    let other_before = manager.lock_state().instances[&second].clone();
    manager.snapshot();

    let clock = manager.step_preview(first, 0.25).unwrap();
    reference.tick(0.25).unwrap();
    reference.pause(reference_handle).unwrap();

    assert_eq!(clock.age_seconds, 1.0);
    assert_eq!(clock.time_scale, 2.0);
    assert!(!clock.playing);
    assert_eq!(clock, manager.preview_clock(first).unwrap());
    let state = manager.lock_state();
    assert_eq!(state.instances[&second], other_before);
    assert_eq!(
        state.instances[&first],
        reference.lock_state().instances[&reference_handle]
    );
    assert!(state.cached_snapshot.is_none());
}

#[test]
fn particle_preview_step_preserves_playing_and_applies_fractional_scale_once() {
    let manager = ParticlesManager::default();
    let first = manager.instantiate(component(1, 0.5, true)).unwrap();
    let second = manager.instantiate(component(2, 2.0, true)).unwrap();
    manager.tick(0.25).unwrap();
    let other_before = manager.lock_state().instances[&second].clone();

    let clock = manager.step_preview(first, 0.5).unwrap();

    assert_eq!(clock.age_seconds, 0.375);
    assert_eq!(clock.time_scale, 0.5);
    assert!(clock.playing);
    assert_eq!(manager.lock_state().instances[&second], other_before);
    assert_eq!(clock, manager.preview_clock(first).unwrap());
}

#[test]
fn particle_preview_step_rejects_invalid_raw_deltas_without_mutation() {
    for playing in [false, true] {
        let manager = ParticlesManager::default();
        let first = manager.instantiate(component(1, 2.0, true)).unwrap();
        manager.instantiate(component(2, 1.0, true)).unwrap();
        manager.tick(0.25).unwrap();
        if !playing {
            manager.pause(first).unwrap();
        }
        for dt in [
            0.0,
            -0.25,
            Real::EPSILON,
            Real::EPSILON / 2.0,
            Real::NAN,
            Real::INFINITY,
            Real::NEG_INFINITY,
        ] {
            assert_step_rejected(&manager, first, dt, ParticlePreviewError::InvalidDeltaTime);
        }
    }
}

#[test]
fn particle_preview_step_rejects_zero_underflowed_and_overflowed_scaled_deltas() {
    for playing in [false, true] {
        for (time_scale, dt) in [(0.0, 0.25), (Real::from_bits(1), 0.25), (Real::MAX, 2.0)] {
            let manager = ParticlesManager::default();
            let handle = manager
                .instantiate(component(1, time_scale, playing))
                .unwrap();
            assert_step_rejected(
                &manager,
                handle,
                dt,
                ParticlePreviewError::InvalidScaledDeltaTime,
            );
        }
    }
}

#[test]
fn particle_preview_step_accepts_sub_epsilon_scaled_delta_that_advances_age() {
    let manager = ParticlesManager::default();
    let scale = Real::EPSILON / 4.0;
    let handle = manager.instantiate(component(1, scale, false)).unwrap();

    let clock = manager.step_preview(handle, 1.0).unwrap();

    assert_eq!(clock.age_seconds, scale);
    assert_eq!(clock.time_scale, scale);
    assert!(!clock.playing);
}

#[test]
fn particle_preview_step_rejects_unrepresentable_or_nonfinite_age_progress() {
    for playing in [false, true] {
        for (age, scale, dt) in [
            (16_777_216.0, 1.0, 0.25),
            (Real::MAX, Real::MAX, 0.5),
            (Real::INFINITY, 1.0, 0.25),
        ] {
            let manager = ParticlesManager::default();
            let handle = manager.instantiate(component(1, scale, playing)).unwrap();
            manager
                .lock_state()
                .instances
                .get_mut(&handle)
                .unwrap()
                .age_seconds = age;
            assert_step_rejected(&manager, handle, dt, ParticlePreviewError::AgeNotAdvancing);
        }
    }
}

#[test]
fn particle_preview_unknown_and_removed_handles_do_not_change_remaining_instances() {
    let manager = ParticlesManager::default();
    manager.instantiate(component(1, 1.0, true)).unwrap();
    let removed = manager.instantiate(component(2, 2.0, true)).unwrap();
    manager.tick(0.25).unwrap();
    manager.remove(removed).unwrap();

    for handle in [ParticleEmitterHandle::new(u64::MAX), removed] {
        manager.snapshot();
        let before = manager.lock_state().clone();
        assert_eq!(
            manager.preview_clock(handle),
            Err(ParticlePreviewError::UnknownHandle(handle.raw()))
        );
        assert_state_unchanged(&before, &manager.lock_state());
        assert_step_rejected(
            &manager,
            handle,
            0.25,
            ParticlePreviewError::UnknownHandle(handle.raw()),
        );
    }
}

#[test]
fn particle_preview_gpu_fallback_clock_explicitly_reports_cpu_simulation() {
    let manager = ParticlesManager::default();
    let mut gpu_component = component(1, 2.0, false);
    gpu_component.asset.backend = ParticleSimulationBackend::Gpu;
    let handle = manager.instantiate(gpu_component).unwrap();

    let clock = manager.step_preview(handle, 0.25).unwrap();

    assert_eq!(clock.age_seconds, 0.5);
    assert_eq!(clock.backend, ParticleSimulationBackend::Gpu);
    assert!(clock.fallback_to_cpu);
    assert!(!clock.playing);
    assert_eq!(clock, manager.preview_clock(handle).unwrap());
}

#[test]
fn particle_preview_step_rejects_gpu_without_fallback_before_mutation() {
    for playing in [false, true] {
        let manager = ParticlesManager::default();
        let handle = ParticleEmitterHandle::new(7);
        let mut gpu_component = component(1, 2.0, playing);
        gpu_component.asset.backend = ParticleSimulationBackend::Gpu;
        // Model a strict GPU instance inside the existing private test tree, without an executor.
        let instance = ParticleSystemInstance::new(handle, gpu_component, false, false).unwrap();
        manager.lock_state().instances.insert(handle, instance);
        let clock = manager.preview_clock(handle).unwrap();
        assert_eq!(clock.backend, ParticleSimulationBackend::Gpu);
        assert!(!clock.fallback_to_cpu);
        assert_eq!(clock.age_seconds, 0.0);
        assert_step_rejected(
            &manager,
            handle,
            0.25,
            ParticlePreviewError::GpuStepUnavailable,
        );
    }
}
