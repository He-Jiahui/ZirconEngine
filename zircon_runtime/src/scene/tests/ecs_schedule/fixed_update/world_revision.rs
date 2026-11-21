use super::*;
use crate::core::math::Vec3;
use crate::scene::components::{NodeKind, WorldMatrix};
use crate::scene::LevelTickError;

const DT: Duration = Duration::from_millis(10);

#[test]
fn fixed_steps_commit_same_world_authored_revisions() {
    let (runtime, level, _) = fixed_update_fixture();
    apply_fixed_timestep(&level, DT);
    let node = level.with_world_mut(|world| world.spawn_node(NodeKind::Empty).unwrap());
    let initial_gen = level.world_generation();
    let initial_epoch = level.capture_world_replacement_epoch();
    let before_plan = level.with_world(|world| world.schedule().stage_plan());
    let observed = Arc::new(Mutex::new(Vec::new()));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("fixed-revision.author")
        .unwrap();
    for (stage, name) in [
        (SystemStage::FixedFirst, "fixed-revision.author.first"),
        (SystemStage::FixedUpdate, "fixed-revision.author.update"),
        (SystemStage::FixedPostUpdate, "fixed-revision.author.post"),
    ] {
        let observed = Arc::clone(&observed);
        registry
            .register_runtime_scene_system(owner, name, stage, move || {
                let observed = Arc::clone(&observed);
                move |context| {
                    let tick = context.tick().simulation_tick().unwrap();
                    let generation = context.level.world_generation();
                    observed.lock().unwrap().push((
                        stage,
                        tick,
                        generation,
                        context.tick().world_generation(),
                    ));
                    if stage == SystemStage::FixedPostUpdate {
                        context.level.with_world_mut(|world| {
                            let mut local = world.local_transform(node).unwrap();
                            local.translation += Vec3::X;
                            assert!(world.update_transform(node, local).unwrap());
                        });
                    }
                    Ok(())
                }
            })
            .register()
            .unwrap();
    }
    apply_runtime_scene_systems(&level, &registry);
    let pinned = level.with_world(|world| world.schedule().stage_plan());
    assert!(!Arc::ptr_eq(&before_plan, &pinned));
    // Registry installation can legitimately consume pre-frame revisions.
    let generation = level.world_generation();
    assert!(generation >= initial_gen);
    let result = level.tick(&runtime.handle(), runtime.advance_time_by(2 * DT, 2));
    assert_eq!(level.capture_world_replacement_epoch(), initial_epoch);
    assert!(level.with_world(|world| Arc::ptr_eq(&pinned, &world.schedule().stage_plan())));
    let (local, affine, cached) = level.with_world(|w| {
        (
            w.local_transform(node).unwrap().translation,
            w.world_matrix(node).unwrap().w_axis.truncate(),
            w.get::<WorldMatrix>(node)
                .map(|row| row.0.w_axis.truncate()),
        )
    });
    let clock = level.world_time().fixed_time();
    let observed = observed.lock().unwrap();
    if let Err(error) = result {
        let LevelTickError::FixedStep {
            receipt,
            source: CoreError::Initialization(owner, detail),
        } = error
        else {
            panic!("unexpected route: {error:?}");
        };
        assert_eq!(owner, "WorldDriver fixed-step world generation");
        assert_eq!(
            detail,
            format!(
                "World generation changed during FixedPostUpdate: expected {generation}, found {}",
                generation + 1
            )
        );
        assert_eq!(
            receipt.phase(),
            FixedStepFailurePhase::Stage(SystemStage::FixedPostUpdate)
        );
        assert_eq!(receipt.system_id(), None);
        assert_eq!(receipt.tick(), observed[0].1);
        assert_eq!(
            (
                observed.len(),
                clock.frame_index(),
                receipt.committed_steps()
            ),
            (3, 0, 0)
        );
        assert_eq!(receipt.remaining_debt(), 2 * DT);
        assert_eq!(level.world_generation(), generation + 1);
        assert_eq!((local, affine), (Vec3::X, Vec3::X));
        println!("FIXED_WORLD label=authored status=RevisionRejected callbacks=3 fixed=0 debt_ns={} generation={generation}->{} x={} same_owner=true", clock.overstep().as_nanos(), level.world_generation(), local.x);
        panic!("same-world authoring must not abort real fixed stages");
    }
    assert_eq!(
        (local, affine, cached),
        (2.0 * Vec3::X, 2.0 * Vec3::X, Some(2.0 * Vec3::X))
    );
    assert_eq!(level.world_generation(), generation + 2);
    assert_eq!(observed.len(), 6);
    for (index, group) in observed.chunks_exact(3).enumerate() {
        let id = group[0].1;
        assert_eq!(id.tick_index(), index as u64 + 1);
        assert_eq!(id.fixed_epoch(), observed[0].1.fixed_epoch());
        // The admitted tick's initial revision is a stamp, not a live equality fence.
        assert_eq!(id.world_generation(), group[0].2);
        assert_eq!(
            group.iter().map(|r| r.0).collect::<Vec<_>>(),
            SystemStage::FIXED_LOOP
        );
        assert!(group
            .iter()
            .all(|r| r.1 == id && r.3 == id.world_generation()));
    }
    let interp = level.fixed_interpolation_context();
    assert_eq!(interp.current().simulation_tick(), Some(observed[3].1));
    assert_eq!(interp.previous().simulation_tick(), Some(observed[0].1));
    assert_eq!(
        (clock.frame_index(), clock.elapsed(), clock.overstep()),
        (2, 2 * DT, Duration::ZERO)
    );
    println!("FIXED_WORLD label=authored status=Committed callbacks=6 fixed=2 debt_ns=0 generation={generation}->{} x={} same_owner=true", level.world_generation(), local.x);
}

#[test]
fn schedule_only_replacement_still_refuses_the_old_fixed_step() {
    let (runtime, level, _) = fixed_update_fixture();
    apply_fixed_timestep(&level, DT);
    let downstream = Arc::new(AtomicU64::new(0));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("fixed-revision.schedule")
        .unwrap();
    registry
        .register_runtime_scene_system(
            owner,
            "fixed-revision.schedule.replace",
            SystemStage::FixedPostUpdate,
            || {
                |context| {
                    context
                        .level
                        .with_world_mut(|world| *world.schedule_mut() = Default::default());
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();
    let downstream_f = Arc::clone(&downstream);
    registry
        .register_runtime_scene_system(
            owner,
            "fixed-revision.schedule.downstream",
            SystemStage::PostUpdate,
            move || {
                let downstream = Arc::clone(&downstream_f);
                move |_| {
                    downstream.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();
    apply_runtime_scene_systems(&level, &registry);
    let generation = level.world_generation();
    let epoch = level.capture_world_replacement_epoch();
    let before = level.with_world(|w| w.schedule().stage_plan());
    let error = level
        .tick(&runtime.handle(), runtime.advance_time_by(DT, 1))
        .unwrap_err();
    assert!(
        matches!(&error, LevelTickError::FixedStep { source: CoreError::Initialization(owner, reason), .. }
        if owner == "SceneScheduleRunner schedule owner" && reason == "World schedule was replaced during the active tick")
    );
    let receipt = error.fixed_step_receipt().unwrap();
    assert_eq!(
        receipt.phase(),
        FixedStepFailurePhase::Stage(SystemStage::FixedPostUpdate)
    );
    assert_eq!(receipt.system_id(), Some("fixed-revision.schedule.replace"));
    assert_eq!(receipt.committed_steps(), 0);
    assert_eq!(receipt.remaining_debt(), DT);
    assert_eq!(receipt.tick().tick_index(), 1);
    assert_eq!(level.world_generation(), generation);
    assert_eq!(level.capture_world_replacement_epoch(), epoch);
    assert!(level.with_world(|w| !Arc::ptr_eq(&before, &w.schedule().stage_plan())));
    let clock = level.world_time().fixed_time();
    assert_eq!(
        (clock.frame_index(), clock.elapsed(), clock.overstep()),
        (0, Duration::ZERO, DT)
    );
    assert_eq!(downstream.load(Ordering::Relaxed), 0);
    level
        .tick(
            &runtime.handle(),
            runtime.advance_time_by(Duration::ZERO, 1),
        )
        .expect("remaining debt admits the fresh schedule");
    let clock = level.world_time().fixed_time();
    assert_eq!(
        (clock.frame_index(), clock.elapsed(), clock.overstep()),
        (1, DT, Duration::ZERO)
    );
    assert_eq!(downstream.load(Ordering::Relaxed), 0);
    println!("FIXED_WORLD label=schedule status=Retired debt_ns=10000000 mutation_revision_unchanged=true retry=1");
}

#[test]
fn staged_level_reset_retires_the_step_even_when_its_schedule_remains_pinned() {
    let (runtime, level, _) = fixed_update_fixture();
    apply_fixed_timestep(&level, DT);
    let once = Arc::new(AtomicBool::new(true));
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("fixed-revision.staged")
        .unwrap();
    registry
        .register_runtime_scene_system(
            owner,
            "fixed-revision.staged.reset",
            SystemStage::FixedPostUpdate,
            move || {
                let once = Arc::clone(&once);
                move |context| {
                    if once.swap(false, Ordering::SeqCst) {
                        let (staged, _) = context.level.with_world_mut(|w| {
                            w.clone_for_dynamic_scene_staging(1024 * 1024).unwrap()
                        });
                        let generation = context.level.world_generation();
                        context
                            .level
                            .replace_world_if_generation(generation, staged)
                            .unwrap();
                    }
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();
    apply_runtime_scene_systems(&level, &registry);
    let generation = level.world_generation();
    let epoch = level.capture_world_replacement_epoch();
    let plan = level.with_world(|w| w.schedule().stage_plan());
    let error = level
        .tick(&runtime.handle(), runtime.advance_time_by(DT, 1))
        .unwrap_err();
    let receipt = error.fixed_step_receipt().unwrap();
    assert_eq!(
        receipt.phase(),
        FixedStepFailurePhase::Stage(SystemStage::FixedPostUpdate)
    );
    assert_eq!(receipt.system_id(), None);
    assert_eq!(receipt.tick().tick_index(), 1);
    assert_eq!(receipt.tick().world_generation(), generation);
    assert_eq!(receipt.committed_steps(), 0);
    assert_eq!(receipt.remaining_debt(), DT);
    assert_eq!(level.capture_world_replacement_epoch(), epoch + 1);
    assert!(level.world_generation() > generation);
    assert!(level.with_world(|w| Arc::ptr_eq(&plan, &w.schedule().stage_plan())));
    assert_eq!(
        level
            .fixed_interpolation_context()
            .current()
            .simulation_tick(),
        None
    );
    level
        .tick(
            &runtime.handle(),
            runtime.advance_time_by(Duration::ZERO, 1),
        )
        .unwrap();
    let interp = level.fixed_interpolation_context();
    let tick = interp.current().simulation_tick().unwrap();
    assert_eq!(tick.tick_index(), 1);
    assert_eq!(tick.world_generation(), level.world_generation());
    assert_eq!(
        (interp.current().elapsed(), interp.remaining_debt()),
        (DT, Duration::ZERO)
    );
    println!("FIXED_WORLD label=staged status=Retired debt_ns=10000000 same_plan=true epoch_changed=true retry=1");
}
