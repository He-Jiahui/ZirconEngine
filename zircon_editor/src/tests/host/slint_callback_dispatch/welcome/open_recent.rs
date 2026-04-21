use super::super::support::*;

#[test]
fn builtin_welcome_surface_open_recent_dispatches_dynamic_host_event_from_template() {
    let _guard = env_lock().lock().unwrap();

    let bridge = BuiltinWelcomeSurfaceTemplateBridge::new().unwrap();

    let event = dispatch_builtin_welcome_surface_control(
        &bridge,
        "OpenRecentProject",
        UiEventKind::Click,
        vec![zircon_runtime::ui::binding::UiBindingValue::string(
            "E:/Projects/Sandbox",
        )],
    )
    .expect("welcome open recent control should resolve through template bridge")
    .unwrap();

    assert_eq!(
        event,
        WelcomeHostEvent::OpenRecentProject {
            path: "E:/Projects/Sandbox".to_string(),
        }
    );
}

#[test]
fn builtin_welcome_surface_open_recent_matches_direct_binding_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let bridge = BuiltinWelcomeSurfaceTemplateBridge::new().unwrap();
    let builtin_event = dispatch_builtin_welcome_surface_control(
        &bridge,
        "OpenRecentProject",
        UiEventKind::Click,
        vec![zircon_runtime::ui::binding::UiBindingValue::string(
            "E:/Projects/Sandbox",
        )],
    )
    .expect("welcome open recent control should resolve through template bridge")
    .unwrap();

    let legacy_event = dispatch_welcome_binding(&EditorUiBinding::new(
        "WelcomeSurface",
        "OpenRecentProject",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::welcome_command(WelcomeCommand::OpenRecentProject {
            path: "E:/Projects/Sandbox".to_string(),
        }),
    ))
    .unwrap();

    assert_eq!(builtin_event, legacy_event);
}
