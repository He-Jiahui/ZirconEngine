use super::source_assertions::assert_source_order;

fn runtime_window_lifecycle_source() -> String {
    [
        include_str!("../runtime_entry_app/window_lifecycle/mod.rs"),
        include_str!("../runtime_entry_app/window_lifecycle/close.rs"),
        include_str!("../runtime_entry_app/window_lifecycle/focus.rs"),
        include_str!("../runtime_entry_app/window_lifecycle/scale_factor.rs"),
        include_str!("../runtime_entry_app/window_lifecycle/status.rs"),
    ]
    .join("\n")
}

fn runtime_window_events_source() -> String {
    [
        include_str!("../runtime_entry_app/window_events/mod.rs"),
        include_str!("../runtime_entry_app/window_events/dispatch.rs"),
    ]
    .join("\n")
}

#[test]
fn runtime_entry_keeps_window_lifecycle_policy_source_visible() {
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_window_events_source = runtime_window_events_source();
    let runtime_window_creation_source = include_str!("../runtime_entry_app/window_creation.rs");
    let runtime_window_lifecycle_root_source =
        include_str!("../runtime_entry_app/window_lifecycle/mod.rs");
    let runtime_window_lifecycle_source = runtime_window_lifecycle_source();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("entry")
        .join("runtime_entry_app");

    assert!(
        runtime_app_source.contains("mod window_lifecycle;"),
        "runtime entry app should keep window lifecycle/status handling in a child module"
    );
    assert!(
        runtime_app_source.contains("mod window_events;"),
        "runtime entry app should keep concrete window-event dispatch in a child module"
    );
    for required in [
        "mod close;",
        "mod focus;",
        "mod scale_factor;",
        "mod status;",
    ] {
        assert!(
            runtime_window_lifecycle_root_source.contains(required),
            "runtime window lifecycle root should preserve structural wiring `{required}`"
        );
    }
    assert!(
        !root.join("window_lifecycle.rs").exists(),
        "runtime window lifecycle should stay folder-backed instead of returning to an umbrella window_lifecycle.rs file"
    );

    assert_source_order(
        runtime_window_creation_source,
        &[
            "fn create_primary_window_surface",
            "self.window_descriptor.primary_window.is_none()",
            "return;",
            "let window_attributes = runtime_window_attributes(&self.window_descriptor, event_loop);",
        ],
        "runtime entry should honor no-primary-window host policy before creating winit attributes",
    );
    assert_source_order(
        runtime_window_events_source.as_str(),
        &[
            "WindowEvent::CloseRequested",
            "self.handle_window_close_requested(event_loop);",
        ],
        "runtime entry should delegate close-request policy handling to the window lifecycle module",
    );
    assert_source_order(
        runtime_window_lifecycle_source.as_str(),
        &[
            "fn handle_window_close_requested",
            "ZrRuntimeEventV1::window_close_requested",
            "self.session.handle_event(event).is_err()",
            "event_loop.exit();",
            "return;",
            "self.window_lifecycle_policy.should_close_on_request()",
            "self.close_primary_window_after_request();",
            ".should_exit_after_primary_close()",
            "event_loop.exit();",
        ],
        "runtime entry should notify the runtime about close requests before applying the configurable close policy",
    );
    for (event, handler_call, helper, constructor) in [
        (
            "WindowEvent::Destroyed",
            "self.handle_window_destroyed(event_loop);",
            "fn handle_window_destroyed",
            "ZrRuntimeEventV1::window_destroyed",
        ),
        (
            "WindowEvent::Moved(position)",
            "self.handle_window_moved(event_loop, position);",
            "fn handle_window_moved",
            "ZrRuntimeEventV1::window_moved",
        ),
        (
            "WindowEvent::Occluded(occluded)",
            "self.handle_window_occluded(event_loop, occluded);",
            "fn handle_window_occluded",
            "ZrRuntimeEventV1::window_occluded",
        ),
        (
            "WindowEvent::ThemeChanged(theme)",
            "self.handle_window_theme_changed(event_loop, theme);",
            "fn handle_window_theme_changed",
            "ZrRuntimeEventV1::window_theme_changed",
        ),
    ] {
        assert_source_order(
            runtime_window_events_source.as_str(),
            &[event, handler_call],
            "runtime entry should delegate window status event forwarding to the window lifecycle module",
        );
        assert_source_order(
            runtime_window_lifecycle_source.as_str(),
            &[
                helper,
                constructor,
                "self.session.handle_event(event).is_err()",
            ],
            "runtime entry should keep window status event forwarding source-visible",
        );
    }
    assert_source_order(
        runtime_window_events_source.as_str(),
        &[
            "WindowEvent::ScaleFactorChanged { scale_factor, .. }",
            "self.handle_window_scale_factor_changed(event_loop, scale_factor);",
        ],
        "runtime entry should delegate scale-factor status forwarding to the window lifecycle module",
    );
    assert_source_order(
        runtime_window_lifecycle_source.as_str(),
        &[
            "fn handle_window_scale_factor_changed",
            "ZrRuntimeEventV1::window_backend_scale_factor_changed",
            "self.session.handle_event(backend_event).is_err()",
            "return;",
            "ZrRuntimeEventV1::window_scale_factor_changed",
            "self.session.handle_event(logical_event).is_err()",
        ],
        "runtime entry should forward backend scale-factor changes before logical scale-factor changes",
    );
    assert_source_order(
        runtime_window_events_source.as_str(),
        &[
            "WindowEvent::Focused(focused)",
            "self.handle_window_focus_changed(event_loop, focused);",
        ],
        "runtime entry should delegate focus lifecycle forwarding to the window lifecycle module",
    );
    assert_source_order(
        runtime_window_lifecycle_source.as_str(),
        &[
            "fn handle_window_focus_changed",
            "let state = if focused",
            "ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1",
            "ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1",
            "ZrRuntimeEventV1::lifecycle",
            "self.session.handle_event(event).is_err()",
        ],
        "runtime entry should translate focus changes into runtime foreground/background lifecycle events",
    );
}
