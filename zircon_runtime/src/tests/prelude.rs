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
    let runtime = CoreRuntime::new();
    let runtime_advance: RuntimeTimeAdvance =
        runtime.advance_time_by(std::time::Duration::from_millis(16), 4);
    let runtime_clocks: RuntimeTimeClocks = runtime.time_clocks();

    let mut diagnostics = DiagnosticStore::default();
    diagnostics.record(
        DiagnosticPath::from("frame/fps"),
        1,
        60.0,
        Some("hz"),
        ["frame"],
    );
    let log_filter = DiagnosticLogFilter::parse("debug").unwrap();
    let formatted_diagnostics = format_diagnostic_store_snapshot(&diagnostics.snapshot());
    let mut diagnostic_log_schedule =
        DiagnosticStoreLogSchedule::repeating(DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT);
    let diagnostic_log_settings = DiagnosticLogSettings::new("prelude")
        .with_location(DiagnosticLogLocation::LocalFirst)
        .with_console_enabled(true)
        .with_file_enabled(false);
    let _log_settings: LogSettings = diagnostic_log_settings.clone();
    let profile = RuntimeProfileDescriptor::for_id(RuntimeProfileId::Client3d);
    let minimal_core_profile = RuntimeCoreProfile::minimal();

    assert_eq!(real_time.frame_index(), 1);
    assert_eq!(virtual_time.delta(), std::time::Duration::ZERO);
    assert_eq!(fixed_plan.step_count, 2);
    assert_eq!(runtime_advance.fixed_step_plan().step_count, 1);
    assert_eq!(
        runtime_clocks.real().delta(),
        std::time::Duration::from_millis(16)
    );
    assert_eq!(TIME_FRAME_COUNT_DIAGNOSTIC, "time.frame_count");
    assert_eq!(TIME_FIXED_STEPS_DIAGNOSTIC, "time.fixed_steps");
    assert_eq!(TIME_FRAME_TIME_DIAGNOSTIC, "time.frame_time");
    assert_eq!(TIME_FPS_DIAGNOSTIC, "time.fps");
    assert_eq!(DIAGNOSTIC_LOG_FILTER_ENV, "ZIRCON_LOG_FILTER");
    assert_eq!(DIAGNOSTIC_LOG_ENV, "ZIRCON_LOG");
    assert_eq!(RUST_LOG_ENV, "RUST_LOG");
    assert!(diagnostic_log_settings
        .format_diagnostics()
        .contains("diagnostic_log.file_enabled=false"));
    assert_eq!(diagnostics.snapshot().series.len(), 1);
    assert!(formatted_diagnostics
        .iter()
        .any(|line| line.starts_with("frame/fps: 60.000000hz")));
    assert_eq!(
        diagnostic_log_schedule.wait_duration(),
        std::time::Duration::from_secs(1)
    );
    assert!(diagnostic_log_schedule.tick(std::time::Duration::from_secs(1)));
    assert!(log_filter.allows(DiagnosticLogLevel::Error));
    assert_eq!(profile.id, RuntimeProfileId::Client3d);
    assert_eq!(profile.minimum_maturity, PluginMaturity::Beta);
    assert!(minimal_core_profile
        .required_capabilities
        .contains(&"runtime.core.tasks".to_string()));
}

#[test]
fn runtime_prelude_exports_task_and_builtin_module_facades() {
    let scheduler = JobScheduler::default();
    let task_pools = TaskPoolOptions::with_num_threads(3).create_pools();
    let counts: TaskPoolThreadCounts = task_pools.thread_counts();
    let task_report: TaskPoolReport = task_pools.report();
    let compute_pool: &TaskPool = task_pools.get(TaskPoolKind::Compute);
    let module_names = [
        FoundationModule.module_name(),
        LogModule.module_name(),
        TasksModule.module_name(),
        TimeModule.module_name(),
        FrameCountModule.module_name(),
        DiagnosticsCoreModule.module_name(),
        LogDiagnosticsModule.module_name(),
    ];

    assert!(scheduler.parallelism() >= 1);
    assert_eq!(counts.total_threads, 3);
    assert!(task_report.format_diagnostics().contains("tasks.pools=3"));
    assert_eq!(compute_pool.kind(), TaskPoolKind::Compute);
    assert_eq!(module_names[0], FOUNDATION_MODULE_NAME);
    assert_eq!(module_names[1], LOG_MODULE_NAME);
    assert_eq!(module_names[2], TASKS_MODULE_NAME);
    assert_eq!(module_names[3], TIME_MODULE_NAME);
    assert_eq!(module_names[4], FRAME_COUNT_MODULE_NAME);
    assert_eq!(module_names[5], DIAGNOSTICS_CORE_MODULE_NAME);
    assert_eq!(module_names[6], LOG_DIAGNOSTICS_MODULE_NAME);
}

#[test]
fn runtime_prelude_exports_platform_window_and_input_contracts() {
    let report: PlatformCapabilityReport =
        PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform())
            .report(PlatformTarget::Windows, RuntimeTargetMode::ClientRuntime);
    let input_event = InputEvent::ButtonPressed(InputButton::MouseLeft);
    let file_event = FileDragDropEvent::Hovered {
        path: "C:/tmp/asset.png".to_string(),
    };
    let gamepad_rumble = GamepadRumbleRequest::add(GamepadId(7), GamepadRumbleIntensity::MAX, 100);
    let ime_request = ImeHostRequest::SetSurroundingText(ImeSurroundingText::new("search", 6, 0));
    let window_descriptor = WindowDescriptor::default()
        .with_primary_window(PrimaryWindowHandle::new(7))
        .with_title("Prelude Window")
        .with_present_mode(WindowPresentMode::Fifo)
        .with_position(WindowPosition::CenteredOn(WindowMonitorSelection::Primary))
        .with_mode(WindowMode::FullscreenOn {
            monitor: WindowMonitorSelection::Index(0),
            video_mode: WindowVideoModeSelection::Specific(
                WindowVideoMode::new(1920, 1080).with_refresh_rate_millihertz(60_000),
            ),
        });
    let window_lifecycle_policy =
        WindowLifecyclePolicy::default().with_exit_condition(WindowExitCondition::DontExit);
    let window_event = WindowStatusEvent::ThemeChanged(WindowTheme::Light);
    let module_names = [PlatformModule.module_name(), InputModule.module_name()];

    assert_eq!(
        report.window_backend,
        CapabilityStatus::Supported(WindowBackend::Winit)
    );
    assert_eq!(
        report.monitor_inventory,
        CapabilityStatus::Supported(MonitorBackend::WinitMonitorHandles)
    );
    assert_eq!(
        report.window_events,
        CapabilityStatus::Supported(WindowEventBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.window_lifecycle,
        CapabilityStatus::Supported(WindowLifecycleBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.window_metrics,
        CapabilityStatus::Supported(WindowMetricsBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.ime,
        CapabilityStatus::Supported(ImeBackend::WinitIme)
    );
    assert_eq!(
        report.keyboard_events,
        CapabilityStatus::Supported(KeyboardEventBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.cursor_boundary,
        CapabilityStatus::Supported(CursorBoundaryBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.cursor_options,
        CapabilityStatus::Unavailable {
            reason: "desktop cursor options host-request backend is not implemented yet"
        }
    );
    assert_eq!(
        report.mouse_buttons,
        CapabilityStatus::Supported(MouseButtonBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.mouse_wheel,
        CapabilityStatus::Supported(MouseWheelBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.touch_events,
        CapabilityStatus::Supported(TouchEventBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.pointer_position,
        CapabilityStatus::Supported(PointerPositionBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.raw_mouse_motion,
        CapabilityStatus::Supported(RawMouseMotionBackend::WinitDeviceEvents)
    );
    assert_eq!(report.event_loop_policy, EventLoopPolicy::Game);
    assert_eq!(
        report.mouse_input,
        CapabilityStatus::Supported(InputBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.gamepad_input,
        CapabilityStatus::Supported(GamepadBackend::Gilrs)
    );
    assert_eq!(
        report.gamepad_events,
        CapabilityStatus::Supported(GamepadEventBackend::GilrsEventPolling)
    );
    assert_eq!(
        report.gamepad_rumble,
        CapabilityStatus::Supported(GamepadRumbleBackend::GilrsForceFeedback)
    );
    assert_eq!(
        report.file_drag_drop,
        CapabilityStatus::Supported(FileDragDropBackend::WinitWindowEvents)
    );
    assert!(matches!(input_event, InputEvent::ButtonPressed(_)));
    assert!(matches!(file_event, FileDragDropEvent::Hovered { .. }));
    assert_eq!(gamepad_rumble.gamepad(), GamepadId(7));
    assert_eq!(
        GamepadButtonSettings::default().press_threshold,
        GAMEPAD_BUTTON_PRESS_THRESHOLD
    );
    assert_eq!(
        GamepadButtonAxisSettings::default().change_threshold,
        GAMEPAD_BUTTON_AXIS_CHANGE_THRESHOLD
    );
    assert_eq!(
        GamepadAxisSettings::default().deadzone_lowerbound,
        GAMEPAD_AXIS_DEADZONE_LOWER
    );
    assert_eq!(
        GamepadAxisSettings::default().deadzone_upperbound,
        GAMEPAD_AXIS_DEADZONE_UPPER
    );
    assert_eq!(
        GamepadAxisSettings::default().livezone_lowerbound,
        GAMEPAD_AXIS_LIVEZONE_LOWER
    );
    assert_eq!(
        GamepadAxisSettings::default().livezone_upperbound,
        GAMEPAD_AXIS_LIVEZONE_UPPER
    );
    assert!(matches!(ime_request, ImeHostRequest::SetSurroundingText(_)));
    assert_eq!(DEFAULT_WINDOW_TITLE, "Zircon Runtime");
    assert_eq!(window_descriptor.title, "Prelude Window");
    assert_eq!(window_descriptor.primary_window.unwrap().raw(), 7);
    assert_eq!(
        window_descriptor.mode,
        WindowMode::FullscreenOn {
            monitor: WindowMonitorSelection::Index(0),
            video_mode: WindowVideoModeSelection::Specific(
                WindowVideoMode::new(1920, 1080).with_refresh_rate_millihertz(60_000),
            ),
        }
    );
    assert_eq!(
        window_descriptor.position,
        WindowPosition::CenteredOn(WindowMonitorSelection::Primary)
    );
    assert_eq!(window_descriptor.present_mode, WindowPresentMode::Fifo);
    assert_eq!(WindowResolution::default().physical_size().x, 1280);
    assert_eq!(WindowResizeConstraints::default().min_width, 180.0);
    assert!(window_lifecycle_policy.should_close_on_request());
    assert!(!window_lifecycle_policy.should_exit_after_primary_close());
    assert_eq!(
        PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY,
        "runtime.window.primary_descriptor"
    );
    assert!(matches!(
        window_event,
        WindowStatusEvent::ThemeChanged(WindowTheme::Light)
    ));
    assert_eq!(module_names[0], PLATFORM_MODULE_NAME);
    assert_eq!(module_names[1], INPUT_MODULE_NAME);
    assert_eq!(PLATFORM_CONFIG_KEY, "runtime.platform.config");
    assert_eq!(
        [PLATFORM_DRIVER_NAME, PLATFORM_MANAGER_NAME],
        [
            "PlatformModule.Driver.PlatformDriver",
            "PlatformModule.Manager.PlatformManager",
        ]
    );
    assert_eq!(
        [INPUT_DRIVER_NAME, INPUT_MANAGER_NAME],
        [
            "InputModule.Driver.InputDriver",
            "InputModule.Manager.InputManager",
        ]
    );
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
