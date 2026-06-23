use super::*;

#[test]
fn schedule_uses_bevy_style_stage_order_and_builtin_post_update_systems() {
    let schedule = Schedule::default();

    assert_eq!(
        schedule.stages(),
        vec![
            SystemStage::First,
            SystemStage::PreUpdate,
            SystemStage::FixedFirst,
            SystemStage::FixedUpdate,
            SystemStage::FixedPostUpdate,
            SystemStage::Update,
            SystemStage::PostUpdate,
            SystemStage::Last,
            SystemStage::RenderExtract,
        ]
    );
    let first = schedule
        .systems_for_stage(SystemStage::First)
        .map(|system| system.system())
        .collect::<Vec<_>>();
    assert_eq!(first, vec![InternalSceneSystem::UpdateEvents]);

    let post_update = schedule
        .systems_for_stage(SystemStage::PostUpdate)
        .map(|system| system.system())
        .collect::<Vec<_>>();
    assert_eq!(
        post_update,
        vec![
            InternalSceneSystem::HierarchyValidity,
            InternalSceneSystem::ActiveHierarchy,
            InternalSceneSystem::WorldTransform,
            InternalSceneSystem::NodeCache,
        ]
    );

    let render_extract = schedule
        .systems_for_stage(SystemStage::RenderExtract)
        .map(|system| system.system())
        .collect::<Vec<_>>();
    assert_eq!(
        render_extract,
        vec![InternalSceneSystem::RenderExtractPrepare]
    );
}

#[test]
fn system_set_intern_is_stable() {
    let mut registry = SystemSetRegistry::default();

    let first = registry.intern("physics.main").unwrap();
    let repeated = registry.intern("physics.main").unwrap();
    let second = registry.intern("animation.main").unwrap();

    assert_eq!(first, repeated);
    assert_ne!(first, second);
    assert_eq!(registry.name(first), Some("physics.main"));
    assert_eq!(registry.name(second), Some("animation.main"));
}

#[test]
fn stage_plan_orders_by_constraints_then_order() {
    let mut schedule = Schedule::default();

    schedule
        .register_system(
            SceneSystemDescriptor::new(
                "plugin.physics.step",
                SystemStage::Update,
                InternalSceneSystem::NodeCache,
            )
            .with_order(100),
        )
        .unwrap();
    schedule
        .register_system(
            SceneSystemDescriptor::new(
                "plugin.animation.evaluate",
                SystemStage::Update,
                InternalSceneSystem::NodeCache,
            )
            .with_order(-100)
            .after(SystemRef::System("plugin.physics.step".to_string())),
        )
        .unwrap();

    let plan = schedule.stage_plan();
    let ids = plan
        .internal_systems_for_stage(SystemStage::Update)
        .iter()
        .map(|system| system.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ids,
        vec!["plugin.physics.step", "plugin.animation.evaluate"]
    );
}

#[test]
fn schedule_stage_plan_orders_steps_by_explicit_declaration_not_registration() {
    fn ordered_update_ids(register_animation_first: bool) -> Vec<String> {
        let mut schedule = Schedule::default();
        let physics = SceneSystemDescriptor::new(
            "plugin.physics.step",
            SystemStage::Update,
            InternalSceneSystem::NodeCache,
        );
        let animation = SceneSystemDescriptor::new(
            "plugin.animation.evaluate",
            SystemStage::Update,
            InternalSceneSystem::NodeCache,
        )
        .after(SystemRef::System("plugin.physics.step".to_string()));

        if register_animation_first {
            schedule.register_system(animation).unwrap();
            schedule.register_system(physics).unwrap();
        } else {
            schedule.register_system(physics).unwrap();
            schedule.register_system(animation).unwrap();
        }

        schedule
            .stage_plan()
            .internal_systems_for_stage(SystemStage::Update)
            .iter()
            .map(|system| system.id.clone())
            .collect::<Vec<_>>()
    }

    let expected = vec![
        "plugin.physics.step".to_string(),
        "plugin.animation.evaluate".to_string(),
    ];
    assert_eq!(ordered_update_ids(false), expected);
    assert_eq!(ordered_update_ids(true), expected);
}

#[test]
fn ordering_cycle_reports_chain() {
    let mut schedule = Schedule::default();

    schedule
        .register_system(
            SceneSystemDescriptor::new(
                "plugin.a",
                SystemStage::Update,
                InternalSceneSystem::NodeCache,
            )
            .after(SystemRef::System("plugin.b".to_string())),
        )
        .unwrap();
    let error = schedule
        .register_system(
            SceneSystemDescriptor::new(
                "plugin.b",
                SystemStage::Update,
                InternalSceneSystem::NodeCache,
            )
            .after(SystemRef::System("plugin.a".to_string())),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        crate::scene::ecs::ScheduleError::OrderingCycle { .. }
    ));
    assert!(error.to_string().contains("ordering cycle"));
    assert!(error.to_string().contains("plugin.a"));
    assert!(error.to_string().contains("plugin.b"));
}

#[test]
fn cross_stage_constraint_rejected() {
    let mut schedule = Schedule::default();

    schedule
        .register_system(SceneSystemDescriptor::new(
            "plugin.update",
            SystemStage::Update,
            InternalSceneSystem::NodeCache,
        ))
        .unwrap();
    let error = schedule
        .register_system(
            SceneSystemDescriptor::new(
                "plugin.post_update",
                SystemStage::PostUpdate,
                InternalSceneSystem::NodeCache,
            )
            .with_constraint(SystemOrderingConstraint::After(SystemRef::System(
                "plugin.update".to_string(),
            ))),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        crate::scene::ecs::ScheduleError::CrossStageConstraint { .. }
    ));
    assert!(error.to_string().contains("cross-stage"));
    assert!(error.to_string().contains("plugin.post_update"));
    assert!(error.to_string().contains("plugin.update"));
}

#[test]
fn schedule_maintains_executor_stage_plan_after_registration_and_load() {
    let mut schedule = Schedule::default();
    assert!(schedule
        .stage_plan()
        .internal_systems_for_stage(SystemStage::Update)
        .is_empty());

    schedule
        .register_system(
            SceneSystemDescriptor::new(
                "zircon.test.cached_update",
                SystemStage::Update,
                InternalSceneSystem::NodeCache,
            )
            .with_order(7),
        )
        .unwrap();
    assert!(schedule
        .stage_plan()
        .internal_systems_for_stage(SystemStage::Update)
        .iter()
        .any(|system| system.id == "zircon.test.cached_update"));

    let serialized = serde_json::to_string(&schedule).expect("schedule serializes");
    let loaded: Schedule = serde_json::from_str(&serialized).expect("schedule deserializes");

    assert!(loaded
        .stage_plan()
        .internal_systems_for_stage(SystemStage::Update)
        .iter()
        .any(|system| system.id == "zircon.test.cached_update"));
    assert!(loaded
        .stage_plan()
        .native_steps_for_stage(SystemStage::Update)
        .is_empty());
}

#[test]
fn schedule_defers_executor_plan_refresh_while_native_system_is_taken() {
    let mut world = World::empty();
    world
        .register_native_system::<(), _>(
            "zircon.test.running_native",
            SystemStage::Update,
            0,
            |()| {},
        )
        .unwrap();

    let running_system = world
        .schedule_mut()
        .take_native_system("zircon.test.running_native")
        .expect("registered native system can be taken");
    world
        .schedule_mut()
        .register_system(SceneSystemDescriptor::new(
            "zircon.test.registered_while_native_runs",
            SystemStage::Update,
            InternalSceneSystem::NodeCache,
        ))
        .unwrap();

    assert!(!world
        .schedule()
        .stage_plan()
        .internal_systems_for_stage(SystemStage::Update)
        .iter()
        .any(|system| system.id == "zircon.test.registered_while_native_runs"));

    world.schedule_mut().restore_native_system(running_system);

    let plan = world.schedule().stage_plan();
    assert!(plan
        .internal_systems_for_stage(SystemStage::Update)
        .iter()
        .any(|system| system.id == "zircon.test.registered_while_native_runs"));
    assert!(plan
        .native_steps_for_stage(SystemStage::Update)
        .iter()
        .any(|step| match step {
            crate::scene::ecs::ScheduledSceneStep::Native { id, .. } =>
                id == "zircon.test.running_native",
            crate::scene::ecs::ScheduledSceneStep::Runtime { .. } => false,
            crate::scene::ecs::ScheduledSceneStep::ApplyDeferred { .. } => false,
        }));
}

#[test]
fn schedule_keeps_taken_native_system_ids_reserved() {
    let mut world = World::empty();
    world
        .register_native_system::<(), _>(
            "zircon.test.reserved_native",
            SystemStage::Update,
            0,
            |()| {},
        )
        .unwrap();

    let running_system = world
        .schedule_mut()
        .take_native_system("zircon.test.reserved_native")
        .expect("registered native system can be taken");
    let duplicate = world
        .schedule_mut()
        .register_system(SceneSystemDescriptor::new(
            "zircon.test.reserved_native",
            SystemStage::Update,
            InternalSceneSystem::NodeCache,
        ))
        .unwrap_err();

    assert!(duplicate
        .to_string()
        .contains("system zircon.test.reserved_native already registered"));

    world.schedule_mut().restore_native_system(running_system);
}

#[test]
fn schedule_rejects_duplicate_and_blank_system_ids() {
    let mut schedule = Schedule::default();

    let duplicate = schedule
        .register_system(SceneSystemDescriptor::new(
            "zircon.scene.node_cache",
            SystemStage::PostUpdate,
            InternalSceneSystem::NodeCache,
        ))
        .unwrap_err();
    assert!(duplicate
        .to_string()
        .contains("system zircon.scene.node_cache already registered"));

    let blank = schedule
        .register_system(SceneSystemDescriptor::new(
            " ",
            SystemStage::Update,
            InternalSceneSystem::NodeCache,
        ))
        .unwrap_err();
    assert_eq!(blank.to_string(), "system id cannot be empty");
}

#[test]
fn schedule_rejects_duplicate_native_and_builtin_system_ids() {
    let mut world = crate::scene::World::empty();
    let duplicate_builtin = world
        .register_native_system::<(), _>("zircon.scene.node_cache", SystemStage::Update, 0, |_| {})
        .unwrap_err();
    assert!(duplicate_builtin
        .to_string()
        .contains("system zircon.scene.node_cache already registered"));

    world
        .register_native_system::<(), _>("gameplay.first", SystemStage::Update, 0, |_| {})
        .unwrap();
    let duplicate_native = world
        .register_native_system::<(), _>("gameplay.first", SystemStage::Update, 1, |_| {})
        .unwrap_err();
    assert!(duplicate_native
        .to_string()
        .contains("system gameplay.first already registered"));
}

#[test]
fn native_system_registration_reports_missing_required_resources() {
    let mut world = crate::scene::World::empty();
    let error = world
        .register_native_system::<crate::scene::ecs::ResParam<MissingScheduleResource>, _>(
            "gameplay.requires_missing_resource",
            SystemStage::Update,
            0,
            |_: crate::scene::ecs::Res<'_, MissingScheduleResource>| {},
        )
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("system gameplay.requires_missing_resource failed to initialize params"));
    assert!(error
        .to_string()
        .contains(std::any::type_name::<MissingScheduleResource>()));
}

#[derive(Debug, PartialEq, Eq)]
struct MissingScheduleResource;

impl crate::scene::ecs::Resource for MissingScheduleResource {}
