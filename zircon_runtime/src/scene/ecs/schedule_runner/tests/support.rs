use super::*;

pub(super) fn register_timed_external_system(
    registry: &mut RuntimeExtensionRegistry,
    owner: crate::plugin::PluginModuleId,
    system_id: &'static str,
    resource_id: &'static str,
    active: Arc<AtomicUsize>,
    max_active: Arc<AtomicUsize>,
) {
    registry
        .register_external_native_system(
            owner,
            system_id,
            SystemStage::Update,
            SceneSystemThreadAffinity::WorkerSafe,
            move |world| {
                let resource_id = world.external_resource_id(resource_id);
                let mut access = SystemParamAccess::default();
                access
                    .add_resource_write(resource_id)
                    .map_err(|error| error.to_string())?;
                Ok(access)
            },
            move || {
                let active = Arc::clone(&active);
                let max_active = Arc::clone(&max_active);
                move || {
                    let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                    max_active.fetch_max(current, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(30));
                    active.fetch_sub(1, Ordering::SeqCst);
                }
            },
        )
        .register()
        .unwrap();
}

pub(super) fn test_level(mut registry: RuntimeExtensionRegistry) -> LevelSystem {
    let mut world = World::empty();
    registry.apply_to_world(&mut world).unwrap();
    LevelSystem::new(
        WorldHandle::new(1),
        Arc::new(Mutex::new(world)),
        LevelMetadata::default(),
    )
}

pub(super) fn run_test_stage(core: &CoreHandle, level: &LevelSystem) {
    let schedule = level.with_world(|world| world.schedule().stage_plan());
    let world_generation = level.world_generation();
    let tick_contexts = SceneStageTickContexts::new(
        test_tick_context(
            crate::core::framework::time::ClockDomainId::WorldVirtual,
            world_generation,
        ),
        test_tick_context(
            crate::core::framework::time::ClockDomainId::MonotonicReal,
            world_generation,
        ),
        test_tick_context(
            crate::core::framework::time::ClockDomainId::WorldFixed,
            world_generation,
        ),
    );
    SceneScheduleRunner::run_stage(
        core,
        level,
        SystemStage::Update,
        tick_contexts,
        false,
        schedule.internal_systems_for_stage(SystemStage::Update),
        schedule.native_steps_for_stage(SystemStage::Update),
        schedule.native_conflict_graph_for_stage(SystemStage::Update),
    )
    .unwrap();
}

fn test_tick_context(
    domain: crate::core::framework::time::ClockDomainId,
    world_generation: u64,
) -> crate::scene::ecs::SystemTickContext {
    crate::scene::ecs::SystemTickContext::new(
        SystemStage::Update,
        crate::core::framework::time::ClockDomainStamp::initial(domain),
        0,
        None,
        Duration::ZERO,
        Duration::ZERO,
        world_generation,
    )
}
