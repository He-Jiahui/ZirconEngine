use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;

use crate::core::framework::scene::SCENE_MODULE_NAME;
use crate::core::framework::time::ClockDomainId;
use crate::core::math::Real;
use crate::core::{CoreError, CoreRuntime, TimePolicy, TimePolicyTransaction};
use crate::plugin::RuntimeExtensionRegistry;
use crate::scene::{
    FixedStepFailurePhase, SystemStage, SystemTickContext, World, create_default_level,
    module_descriptor,
};

#[test]
fn level_tick_repeats_fixed_loop_stages_for_drained_fixed_steps() {
    let (runtime, level, events) = fixed_update_fixture();
    apply_fixed_timestep(&level, Duration::from_millis(10));
    install_fixed_update_recorder(&level, events.clone());

    let advance = runtime.advance_time_by(Duration::from_millis(35), 8);
    level.tick(&runtime.handle(), advance).unwrap();

    let events = events.lock().unwrap();
    assert_eq!(events.as_slice(), &[0.01, 0.01, 0.01]);
}

#[test]
fn fixed_loop_stages_run_per_fixed_step() {
    let (runtime, level, _) = fixed_update_fixture();
    apply_fixed_timestep(&level, Duration::from_millis(10));
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("plugins01.runtime").unwrap();
    for (stage, label) in [
        (SystemStage::FixedFirst, "fixed-first"),
        (SystemStage::FixedUpdate, "fixed-update"),
        (SystemStage::FixedPostUpdate, "fixed-post-update"),
    ] {
        let events_for_system = Arc::clone(&events);
        registry
            .register_runtime_scene_system(
                owner,
                format!("plugins01.fixed-loop.{label}"),
                stage,
                move || {
                    let events = Arc::clone(&events_for_system);
                    move |_| {
                        events.lock().unwrap().push(label);
                        Ok(())
                    }
                },
            )
            .register()
            .unwrap();
    }
    apply_runtime_scene_systems(&level, &registry);

    let advance = runtime.advance_time_by(Duration::from_millis(25), 8);
    level.tick(&runtime.handle(), advance).unwrap();

    assert_eq!(
        events.lock().unwrap().as_slice(),
        [
            "fixed-first",
            "fixed-update",
            "fixed-post-update",
            "fixed-first",
            "fixed-update",
            "fixed-post-update",
        ]
    );
}

#[test]
fn level_tick_skips_fixed_loop_stages_when_no_fixed_steps_are_drained() {
    let (runtime, level, events) = fixed_update_fixture();
    apply_fixed_timestep(&level, Duration::from_millis(10));
    install_fixed_update_recorder(&level, events.clone());

    let advance = runtime.advance_time_by(Duration::from_millis(5), 8);
    assert_eq!(advance.fixed_step_budget(), 8);

    level.tick(&runtime.handle(), advance).unwrap();

    assert!(events.lock().unwrap().is_empty());
}

#[test]
fn fixed_loop_clamps_to_max_steps_per_frame() {
    let (runtime, level, events) = fixed_update_fixture();
    apply_fixed_timestep(&level, Duration::from_millis(10));
    install_fixed_update_recorder(&level, events.clone());

    let advance = runtime.advance_time_by(Duration::from_millis(55), 4);
    assert_eq!(advance.fixed_step_budget(), 4);

    level.tick(&runtime.handle(), advance).unwrap();

    let events = events.lock().unwrap();
    assert_eq!(events.len(), 4);
    assert!(events.iter().all(|delta| (*delta - 0.01).abs() < 0.000_001));
}

#[test]
fn fixed_runtime_system_receives_each_committed_simulation_tick_context() {
    let (runtime, level, _) = fixed_update_fixture();
    apply_fixed_timestep(&level, Duration::from_millis(10));
    let observed = Arc::new(Mutex::new(Vec::<SystemTickContext>::new()));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("runtime03.fixed.context")
        .unwrap();
    let observed_for_system = Arc::clone(&observed);
    registry
        .register_runtime_scene_system(
            owner,
            "runtime03.fixed.context.recorder",
            SystemStage::FixedUpdate,
            move || {
                let observed = Arc::clone(&observed_for_system);
                move |context| {
                    observed.lock().unwrap().push(context.tick());
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();
    apply_runtime_scene_systems(&level, &registry);

    let advance = runtime.advance_time_by(Duration::from_millis(25), 8);
    level.tick(&runtime.handle(), advance).unwrap();

    let observed = observed.lock().unwrap();
    assert_eq!(observed.len(), 2);
    assert!(
        observed
            .iter()
            .all(|tick| tick.stage() == SystemStage::FixedUpdate)
    );
    assert!(
        observed
            .iter()
            .all(|tick| tick.clock_domain() == ClockDomainId::WorldFixed)
    );
    assert_eq!(observed[0].outer_frame_index(), 1);
    assert_eq!(
        observed[0].simulation_tick().map(|tick| tick.tick_index()),
        Some(1)
    );
    assert_eq!(
        observed[1].simulation_tick().map(|tick| tick.tick_index()),
        Some(2)
    );
    assert_eq!(observed[0].delta(), Duration::from_millis(10));
    assert_eq!(observed[0].elapsed(), Duration::from_millis(10));
    assert_eq!(observed[1].elapsed(), Duration::from_millis(20));
    assert_eq!(observed[0].world_generation(), level.world_generation());
}

#[test]
fn fixed_runtime_system_observes_only_the_prior_committed_clock_during_its_step() {
    let (runtime, level, _) = fixed_update_fixture();
    apply_fixed_timestep(&level, Duration::from_millis(10));
    let observed = Arc::new(Mutex::new(Vec::new()));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("runtime03.fixed.committed-clock")
        .unwrap();
    let observed_for_system = Arc::clone(&observed);
    registry
        .register_runtime_scene_system(
            owner,
            "runtime03.fixed.committed-clock.recorder",
            SystemStage::FixedUpdate,
            move || {
                let observed = Arc::clone(&observed_for_system);
                move |context| {
                    observed.lock().unwrap().push((
                        context.tick().elapsed(),
                        context.level.world_time().fixed_time().elapsed(),
                    ));
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();
    apply_runtime_scene_systems(&level, &registry);

    let advance = runtime.advance_time_by(Duration::from_millis(25), 8);
    level.tick(&runtime.handle(), advance).unwrap();

    assert_eq!(
        observed.lock().unwrap().as_slice(),
        [
            (Duration::from_millis(10), Duration::ZERO),
            (Duration::from_millis(20), Duration::from_millis(10)),
        ]
    );
}

#[test]
fn runtime_system_reads_interpolation_from_only_committed_fixed_states() {
    let (runtime, level, _) = fixed_update_fixture();
    apply_fixed_timestep(&level, Duration::from_millis(10));
    let observed = Arc::new(Mutex::new(Vec::new()));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("runtime03.fixed.interpolation")
        .unwrap();
    let observed_for_system = Arc::clone(&observed);
    registry
        .register_runtime_scene_system(
            owner,
            "runtime03.fixed.interpolation.recorder",
            SystemStage::Update,
            move || {
                let observed = Arc::clone(&observed_for_system);
                move |context| {
                    observed
                        .lock()
                        .unwrap()
                        .push(context.level.fixed_interpolation_context());
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();
    apply_runtime_scene_systems(&level, &registry);

    let advance = runtime.advance_time_by(Duration::from_millis(25), 8);
    level.tick(&runtime.handle(), advance).unwrap();

    let interpolation = observed
        .lock()
        .unwrap()
        .pop()
        .expect("update system should observe interpolation evidence");
    assert_eq!(
        interpolation
            .previous()
            .simulation_tick()
            .map(|tick| tick.tick_index()),
        Some(1)
    );
    assert_eq!(
        interpolation
            .current()
            .simulation_tick()
            .map(|tick| tick.tick_index()),
        Some(2)
    );
    assert_eq!(interpolation.remaining_debt(), Duration::from_millis(5));
    assert_eq!(interpolation.fraction(), 0.5);
}

#[test]
fn fixed_step_failure_commits_only_prior_steps_and_preserves_remaining_debt() {
    let (runtime, level, _) = fixed_update_fixture();
    apply_fixed_timestep(&level, Duration::from_millis(10));
    let calls = Arc::new(Mutex::new(0_u32));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("runtime03.fixed.failure")
        .unwrap();
    let calls_for_system = Arc::clone(&calls);
    registry
        .register_runtime_scene_system(
            owner,
            "runtime03.fixed.failure.injected",
            SystemStage::FixedUpdate,
            move || {
                let calls = Arc::clone(&calls_for_system);
                move |_| {
                    let mut calls = calls.lock().unwrap();
                    *calls = calls.saturating_add(1);
                    if *calls == 2 {
                        return Err(CoreError::Initialization(
                            "fixed-step-test".to_string(),
                            "injected second-step failure".to_string(),
                        ));
                    }
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();
    apply_runtime_scene_systems(&level, &registry);

    let advance = runtime.advance_time_by(Duration::from_millis(25), 8);
    let error = level
        .tick(&runtime.handle(), advance)
        .expect_err("second fixed step should fail");
    assert!(error.to_string().contains("injected second-step failure"));

    let receipt = error
        .fixed_step_receipt()
        .expect("fixed runtime failure must retain its typed receipt");
    assert_eq!(
        receipt.phase(),
        FixedStepFailurePhase::Stage(SystemStage::FixedUpdate)
    );
    assert_eq!(
        receipt.system_id(),
        Some("runtime03.fixed.failure.injected")
    );
    assert_eq!(receipt.tick().tick_index(), 2);
    assert_eq!(receipt.committed_steps(), 1);
    assert_eq!(receipt.remaining_debt(), Duration::from_millis(15));
    assert_eq!(
        receipt.tick().world_generation(),
        receipt.observed_world_generation()
    );

    let after_failure = level.world_time().fixed_time();
    assert_eq!(after_failure.frame_index(), 1);
    assert_eq!(after_failure.elapsed(), Duration::from_millis(10));
    assert_eq!(after_failure.overstep(), Duration::from_millis(15));

    let retry = runtime.advance_time_by(Duration::ZERO, 8);
    level
        .tick(&runtime.handle(), retry)
        .expect("the uncommitted second step should retry");
    let after_retry = level.world_time().fixed_time();
    assert_eq!(after_retry.frame_index(), 2);
    assert_eq!(after_retry.elapsed(), Duration::from_millis(20));
    assert_eq!(after_retry.overstep(), Duration::from_millis(5));
}

#[test]
fn fixed_runtime_panic_aborts_the_step_and_restores_the_callback_for_retry() {
    let (runtime, level, _) = fixed_update_fixture();
    apply_fixed_timestep(&level, Duration::from_millis(10));
    let panic_once = Arc::new(AtomicBool::new(true));
    let calls = Arc::new(AtomicU64::new(0));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("runtime22.fixed.panic-retry")
        .unwrap();
    let panic_once_for_system = Arc::clone(&panic_once);
    let calls_for_system = Arc::clone(&calls);
    registry
        .register_runtime_scene_system(
            owner,
            "runtime22.fixed.panic-retry.injected",
            SystemStage::FixedUpdate,
            move || {
                let panic_once = Arc::clone(&panic_once_for_system);
                let calls = Arc::clone(&calls_for_system);
                move |_| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    if panic_once.swap(false, Ordering::SeqCst) {
                        panic!("intentional fixed runtime panic");
                    }
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();
    apply_runtime_scene_systems(&level, &registry);

    let advance = runtime.advance_time_by(Duration::from_millis(10), 8);
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        level.tick(&runtime.handle(), advance)
    }));
    assert!(panic.is_err());
    let after_panic = level.world_time().fixed_time();
    assert_eq!(after_panic.frame_index(), 0);
    assert_eq!(after_panic.elapsed(), Duration::ZERO);
    assert_eq!(after_panic.overstep(), Duration::from_millis(10));

    let retry = runtime.advance_time_by(Duration::ZERO, 8);
    level
        .tick(&runtime.handle(), retry)
        .expect("the aborted fixed step should retry with the restored callback");
    let after_retry = level.world_time().fixed_time();
    assert_eq!(after_retry.frame_index(), 1);
    assert_eq!(after_retry.elapsed(), Duration::from_millis(10));
    assert_eq!(after_retry.overstep(), Duration::ZERO);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn world_replacement_during_a_fixed_step_aborts_the_uncommitted_transaction() {
    let (runtime, level, _) = fixed_update_fixture();
    apply_fixed_timestep(&level, Duration::from_millis(10));
    let original_generation = level.world_generation();
    let calls = Arc::new(AtomicU64::new(0));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("runtime03.fixed.replacement")
        .unwrap();
    let calls_for_system = Arc::clone(&calls);
    registry
        .register_runtime_scene_system(
            owner,
            "runtime03.fixed.replacement.injected",
            SystemStage::FixedUpdate,
            move || {
                let calls = Arc::clone(&calls_for_system);
                move |context| {
                    calls.fetch_add(1, Ordering::Relaxed);
                    context
                        .level
                        .replace_world_and_reset_runtime_state(World::empty());
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();
    apply_runtime_scene_systems(&level, &registry);

    let advance = runtime.advance_time_by(Duration::from_millis(10), 8);
    let error = level
        .tick(&runtime.handle(), advance)
        .expect_err("replacement must prevent the old World fixed step from committing");
    assert!(error.to_string().contains("World generation changed"));
    assert!(level.world_generation() > original_generation);

    let receipt = error
        .fixed_step_receipt()
        .expect("World replacement must retain its typed fixed-step receipt");
    assert_eq!(
        receipt.phase(),
        FixedStepFailurePhase::Stage(SystemStage::FixedUpdate)
    );
    assert_eq!(receipt.system_id(), None);
    assert_eq!(receipt.tick().world_generation(), original_generation);
    assert_eq!(
        receipt.observed_world_generation(),
        level.world_generation()
    );
    assert_eq!(receipt.committed_steps(), 0);
    assert_eq!(receipt.remaining_debt(), Duration::from_millis(10));

    let fixed = level.world_time().fixed_time();
    assert_eq!(fixed.frame_index(), 0);
    assert_eq!(fixed.elapsed(), Duration::ZERO);
    assert_eq!(fixed.overstep(), Duration::from_millis(10));
    assert_eq!(
        level.fixed_interpolation_context().remaining_debt(),
        Duration::from_millis(10)
    );
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(level.with_world_mut(|world| {
        world
            .schedule_mut()
            .take_runtime_system("runtime03.fixed.replacement.injected")
            .is_none()
    }));

    let retry = runtime.advance_time_by(Duration::ZERO, 8);
    level
        .tick(&runtime.handle(), retry)
        .expect("the replacement World should commit the retained fixed-step debt");
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    let fixed = level.world_time().fixed_time();
    assert_eq!(fixed.frame_index(), 1);
    assert_eq!(fixed.elapsed(), Duration::from_millis(10));
    assert_eq!(fixed.overstep(), Duration::ZERO);
}

#[test]
fn fixed_step_failure_system_id_allocation_is_error_only() {
    let source = include_str!("../../ecs/schedule_runner.rs");

    assert_eq!(source.matches("system_id.to_owned()").count(), 1);
    assert!(source.contains("fn runtime_system(system_id: &str, source: CoreError)"));
    assert!(source.contains(".map_err(|source| SceneStageRunError::runtime_system(id, source))?"));
}

#[test]
fn fixed_step_failure_receipt_classifies_each_fixed_stage() {
    for (stage, suffix) in [
        (SystemStage::FixedFirst, "first"),
        (SystemStage::FixedUpdate, "update"),
        (SystemStage::FixedPostUpdate, "post-update"),
    ] {
        let (runtime, level, _) = fixed_update_fixture();
        apply_fixed_timestep(&level, Duration::from_millis(10));
        let system_id = format!("runtime22.fixed-stage-failure.{suffix}");
        let expected_system_id = system_id.clone();
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry
            .intern_plugin_module("runtime22.fixed-stage-failure")
            .unwrap();
        registry
            .register_runtime_scene_system(owner, system_id, stage, move || {
                move |_| {
                    Err(CoreError::Initialization(
                        "runtime22.fixed-stage-failure".to_string(),
                        "injected".to_string(),
                    ))
                }
            })
            .register()
            .unwrap();
        apply_runtime_scene_systems(&level, &registry);

        let advance = runtime.advance_time_by(Duration::from_millis(10), 8);
        let error = level
            .tick(&runtime.handle(), advance)
            .expect_err("injected fixed-stage failure must abort the tick");
        let receipt = error
            .fixed_step_receipt()
            .expect("fixed-stage error must carry a receipt");
        assert_eq!(receipt.phase(), FixedStepFailurePhase::Stage(stage));
        assert_eq!(receipt.system_id(), Some(expected_system_id.as_str()));
        assert_eq!(receipt.committed_steps(), 0);
        assert_eq!(receipt.remaining_debt(), Duration::from_millis(10));
        assert_eq!(level.world_time().fixed_time().frame_index(), 0);
    }
}

#[test]
#[ignore = "release profiling matrix; run explicitly with --ignored --nocapture"]
fn runtime22_performance_fixed_step_transaction_profile_matrix() {
    const WARMUP_SAMPLES: usize = 10;
    const MEASURED_SAMPLES: usize = 50;

    for system_count in [1_u32, 100, 1_000] {
        for (workload, delta, budget, executed_steps) in [
            ("zero", Duration::ZERO, 8, 0_u64),
            ("one", Duration::from_millis(10), 8, 1),
            ("eight", Duration::from_millis(80), 8, 8),
            ("capped", Duration::from_millis(160), 4, 4),
        ] {
            let (runtime, level, calls) = fixed_step_profile_fixture(system_count);
            let mut samples = Vec::with_capacity(MEASURED_SAMPLES);
            for sample_index in 0..WARMUP_SAMPLES + MEASURED_SAMPLES {
                let before_calls = calls.load(Ordering::Relaxed);
                let advance = runtime.advance_time_by(delta, budget);
                let started_at = Instant::now();
                level
                    .tick(&runtime.handle(), advance)
                    .expect("profile workload should complete");
                let elapsed = started_at.elapsed();
                assert_eq!(
                    calls.load(Ordering::Relaxed) - before_calls,
                    executed_steps * u64::from(system_count),
                    "profile workload must execute the requested fixed-stage callbacks"
                );
                if sample_index >= WARMUP_SAMPLES {
                    samples.push(elapsed);
                }
            }
            samples.sort_unstable();
            let p50 = samples[MEASURED_SAMPLES / 2];
            let p95 = samples[(MEASURED_SAMPLES * 95).div_ceil(100) - 1];
            println!(
                "runtime22.fixed_step workload={workload} systems={system_count} samples={MEASURED_SAMPLES} p50_us={:.3} p95_us={:.3}",
                p50.as_secs_f64() * 1_000_000.0,
                p95.as_secs_f64() * 1_000_000.0,
            );
        }
    }
}

fn fixed_step_profile_fixture(
    system_count: u32,
) -> (CoreRuntime, crate::scene::LevelSystem, Arc<AtomicU64>) {
    let (runtime, level, _) = fixed_update_fixture();
    apply_fixed_timestep(&level, Duration::from_millis(10));
    let calls = Arc::new(AtomicU64::new(0));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("runtime22.fixed-step-profile")
        .unwrap();
    for system_index in 0..system_count {
        let calls_for_system = Arc::clone(&calls);
        registry
            .register_runtime_scene_system(
                owner,
                format!("runtime22.fixed-step-profile.system.{system_index:04}"),
                SystemStage::FixedUpdate,
                move || {
                    let calls = Arc::clone(&calls_for_system);
                    move |_| {
                        calls.fetch_add(1, Ordering::Relaxed);
                        Ok(())
                    }
                },
            )
            .register()
            .unwrap();
    }
    apply_runtime_scene_systems(&level, &registry);
    (runtime, level, calls)
}

fn fixed_update_fixture() -> (
    CoreRuntime,
    crate::scene::LevelSystem,
    Arc<Mutex<Vec<Real>>>,
) {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    (runtime, level, Arc::new(Mutex::new(Vec::new())))
}

fn apply_fixed_timestep(level: &crate::scene::LevelSystem, timestep: Duration) {
    let policy: TimePolicy = level.time_policy().with_fixed_timestep(timestep);
    level
        .apply_time_policy(TimePolicyTransaction::new(policy))
        .expect("test fixed timestep should commit");
}

fn install_fixed_update_recorder(level: &crate::scene::LevelSystem, events: Arc<Mutex<Vec<Real>>>) {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("runtime03.runtime").unwrap();
    registry
        .register_runtime_scene_system(
            owner,
            "runtime03.fixed-update.recorder",
            SystemStage::FixedUpdate,
            move || {
                let events = Arc::clone(&events);
                move |context| {
                    events.lock().unwrap().push(context.tick().delta_seconds());
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();
    apply_runtime_scene_systems(level, &registry);
}

fn apply_runtime_scene_systems(
    level: &crate::scene::LevelSystem,
    registry: &RuntimeExtensionRegistry,
) {
    let plan = registry.world_runtime_extension_plan().unwrap();
    level
        .with_world_mut(|world| plan.apply_to_world(world))
        .unwrap();
}
