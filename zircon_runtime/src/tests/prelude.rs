use crate::prelude::*;

#[test]
fn runtime_prelude_exports_core_lifecycle_and_module_contracts() {
    let runtime = CoreRuntime::new();
    let descriptor = ModuleDescriptor::new("PreludeModule", "prelude smoke module");
    let registry_name = RegistryName::from_parts("PreludeModule", ServiceKind::Driver, "Driver");
    let dependency = DependencySpec::named(registry_name.clone());

    runtime.register_module(descriptor).unwrap();

    assert_eq!(registry_name.as_str(), "PreludeModule.Driver.Driver");
    assert_eq!(dependency.name, registry_name);
    assert_eq!(StartupMode::Immediate, StartupMode::Immediate);
    assert_eq!(LifecycleState::Registered, LifecycleState::Registered);
}

#[test]
fn runtime_prelude_exports_time_diagnostics_log_and_runtime_profile_types() {
    let mut real_time = Time::<Real>::default();
    real_time.advance_by(std::time::Duration::from_millis(16));
    let mut virtual_time = Time::<Virtual>::default();
    virtual_time.pause();
    virtual_time.advance_from_real_delta(std::time::Duration::from_millis(16));
    let mut fixed_time = Time::<Fixed>::from_duration(std::time::Duration::from_millis(8));
    fixed_time.accumulate_overstep(std::time::Duration::from_millis(18));
    let fixed_plan: FixedStepPlan = fixed_time.drain_steps(4);

    let mut diagnostics = DiagnosticStore::default();
    diagnostics.record(
        DiagnosticPath::from("frame/fps"),
        1,
        60.0,
        Some("hz"),
        ["frame"],
    );
    let log_filter = DiagnosticLogFilter::parse("debug").unwrap();
    let profile = RuntimeProfileDescriptor::for_id(RuntimeProfileId::Client3d);

    assert_eq!(real_time.frame_index(), 1);
    assert_eq!(virtual_time.delta(), std::time::Duration::ZERO);
    assert_eq!(fixed_plan.step_count, 2);
    assert_eq!(diagnostics.snapshot().series.len(), 1);
    assert!(log_filter.allows(DiagnosticLogLevel::Error));
    assert_eq!(profile.id, RuntimeProfileId::Client3d);
}

#[test]
fn runtime_prelude_exports_task_and_builtin_module_facades() {
    let scheduler = JobScheduler::default();
    let module_names = [
        FoundationModule.module_name(),
        TasksModule.module_name(),
        TimeModule.module_name(),
        FrameCountModule.module_name(),
        DiagnosticsCoreModule.module_name(),
    ];

    assert!(scheduler.parallelism() >= 1);
    assert_eq!(module_names[0], FOUNDATION_MODULE_NAME);
    assert_eq!(module_names[1], TASKS_MODULE_NAME);
    assert_eq!(module_names[2], TIME_MODULE_NAME);
    assert_eq!(module_names[3], FRAME_COUNT_MODULE_NAME);
    assert_eq!(module_names[4], DIAGNOSTICS_CORE_MODULE_NAME);
}

#[test]
fn runtime_prelude_exports_state_contracts() {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    enum PreludeState {
        #[default]
        Loading,
        Running,
    }

    let runtime = CoreRuntime::new();
    let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let on_enter = events.clone();

    runtime.register_on_enter(OnEnter::new(PreludeState::Running), move |_| {
        on_enter.lock().unwrap().push("running");
    });
    let initial = runtime.init_state::<PreludeState>();
    runtime.set_next_state(PreludeState::Running);
    let transition: StateTransitionEvent<PreludeState> = runtime.apply_state_transition().unwrap();
    let current: State<PreludeState> = runtime.state().unwrap();
    let next: NextState<PreludeState> = runtime.next_state();

    assert_eq!(
        <PreludeState as StateSpec>::state_name(),
        std::any::type_name::<PreludeState>()
    );
    assert_eq!(initial.entered, Some(PreludeState::Loading));
    assert_eq!(transition.entered, Some(PreludeState::Running));
    assert_eq!(current.get(), &PreludeState::Running);
    assert_eq!(next, NextState::Unchanged);
    assert_eq!(*events.lock().unwrap(), vec!["running"]);
}
