use super::sources::entry_root;

#[test]
fn entry_subsystem_is_split_into_builtin_modules_run_modes_and_runtime_app_tree() {
    let root = entry_root();

    for relative in [
        "builtin_modules.rs",
        "entry_runner/mod.rs",
        "entry_runner/editor.rs",
        "entry_runner/runtime.rs",
        "entry_runner/headless.rs",
        "runtime_entry_app/application_handler/mod.rs",
        "runtime_entry_app/application_handler/hooks.rs",
        "runtime_entry_app/config/mod.rs",
        "runtime_entry_app/config/app_config.rs",
        "runtime_entry_app/converters/mod.rs",
        "runtime_entry_app/converters/abi.rs",
        "runtime_entry_app/converters/keyboard.rs",
        "runtime_entry_app/converters/pointer.rs",
        "runtime_entry_app/converters/window.rs",
        "runtime_entry_app/device_events/mod.rs",
        "runtime_entry_app/device_events/dispatch.rs",
        "runtime_entry_app/event_loop_policy/mod.rs",
        "runtime_entry_app/event_loop_policy/control_flow.rs",
        "runtime_entry_app/file_drag_drop/mod.rs",
        "runtime_entry_app/file_drag_drop/cancelled.rs",
        "runtime_entry_app/file_drag_drop/dropped.rs",
        "runtime_entry_app/file_drag_drop/hovered.rs",
        "runtime_entry_app/frame_loop.rs",
        "runtime_entry_app/gamepad/mod.rs",
        "runtime_entry_app/gamepad/codes.rs",
        "runtime_entry_app/gamepad/events.rs",
        "runtime_entry_app/gamepad/host.rs",
        "runtime_entry_app/gamepad/polling.rs",
        "runtime_entry_app/gamepad/rumble.rs",
        "runtime_entry_app/host_requests/mod.rs",
        "runtime_entry_app/host_requests/drain.rs",
        "runtime_entry_app/host_requests/routing.rs",
        "runtime_entry_app/host_requests/ime/mod.rs",
        "runtime_entry_app/host_requests/ime/enable.rs",
        "runtime_entry_app/host_requests/ime/geometry.rs",
        "runtime_entry_app/host_requests/ime/request.rs",
        "runtime_entry_app/host_requests/ime/surrounding_text.rs",
        "runtime_entry_app/ime_input/mod.rs",
        "runtime_entry_app/ime_input/composition.rs",
        "runtime_entry_app/ime_input/deletion.rs",
        "runtime_entry_app/ime_input/lifecycle.rs",
        "runtime_entry_app/ime_input/routing.rs",
        "runtime_entry_app/keyboard_input/mod.rs",
        "runtime_entry_app/keyboard_input/event.rs",
        "runtime_entry_app/keyboard_input/payload.rs",
        "runtime_entry_app/mod.rs",
        "runtime_entry_app/pointer_input/mod.rs",
        "runtime_entry_app/pointer_input/button.rs",
        "runtime_entry_app/pointer_input/cursor.rs",
        "runtime_entry_app/pointer_input/device.rs",
        "runtime_entry_app/pointer_input/motion.rs",
        "runtime_entry_app/pointer_input/wheel.rs",
        "runtime_entry_app/surface_present/mod.rs",
        "runtime_entry_app/surface_present/binding.rs",
        "runtime_entry_app/surface_present/fallback.rs",
        "runtime_entry_app/surface_present/lifecycle.rs",
        "runtime_entry_app/surface_present/redraw.rs",
        "runtime_entry_app/surface_present/resize.rs",
        "runtime_entry_app/window_attributes/mod.rs",
        "runtime_entry_app/window_attributes/builder.rs",
        "runtime_entry_app/window_attributes/fullscreen.rs",
        "runtime_entry_app/window_attributes/monitor.rs",
        "runtime_entry_app/window_attributes/position.rs",
        "runtime_entry_app/window_attributes/video_mode.rs",
        "runtime_entry_app/window_creation.rs",
        "runtime_entry_app/window_events/mod.rs",
        "runtime_entry_app/window_events/dispatch.rs",
        "runtime_entry_app/window_lifecycle/mod.rs",
        "runtime_entry_app/window_lifecycle/close.rs",
        "runtime_entry_app/window_lifecycle/focus.rs",
        "runtime_entry_app/window_lifecycle/scale_factor.rs",
        "runtime_entry_app/window_lifecycle/status.rs",
        "runtime_entry_app/window_surface/mod.rs",
        "runtime_entry_app/window_surface/native_target.rs",
        "runtime_library/mod.rs",
        "runtime_library/loaded_runtime.rs",
        "runtime_library/runtime_session.rs",
        "tests/mod.rs",
        "tests/runtime_entry_device_guards/mod.rs",
        "tests/runtime_entry_device_guards/dispatch.rs",
        "tests/runtime_entry_device_guards/sources.rs",
        "tests/runtime_entry_device_guards/structure.rs",
        "tests/runtime_entry_input_guards/mod.rs",
        "tests/runtime_entry_input_guards/file_drag_drop.rs",
        "tests/runtime_entry_input_guards/ime.rs",
        "tests/runtime_entry_input_guards/keyboard.rs",
        "tests/runtime_entry_input_guards/pointer.rs",
        "tests/runtime_entry_input_guards/protocol.rs",
        "tests/runtime_entry_input_guards/sources.rs",
        "tests/runtime_entry_source_guards/mod.rs",
        "tests/runtime_entry_source_guards/application_handler.rs",
        "tests/runtime_entry_source_guards/config.rs",
        "tests/runtime_entry_source_guards/converters.rs",
        "tests/runtime_entry_source_guards/entry_tree.rs",
        "tests/runtime_entry_source_guards/event_loop_policy.rs",
        "tests/runtime_entry_source_guards/frame_loop.rs",
        "tests/runtime_entry_source_guards/runtime_session.rs",
        "tests/runtime_entry_source_guards/sources.rs",
        "tests/runtime_entry_source_guards/viewport.rs",
        "tests/runtime_entry_source_guards/window_attributes.rs",
        "tests/runtime_entry_source_guards/window_events.rs",
        "tests/runtime_entry_surface_present_guards/mod.rs",
        "tests/runtime_entry_surface_present_guards/dynamic_api.rs",
        "tests/runtime_entry_surface_present_guards/fallback.rs",
        "tests/runtime_entry_surface_present_guards/resize_redraw.rs",
        "tests/runtime_entry_surface_present_guards/sources.rs",
        "tests/runtime_entry_surface_present_guards/structure.rs",
        "tests/runtime_entry_window_lifecycle_guards/mod.rs",
        "tests/runtime_entry_window_lifecycle_guards/close.rs",
        "tests/runtime_entry_window_lifecycle_guards/focus.rs",
        "tests/runtime_entry_window_lifecycle_guards/scale_factor.rs",
        "tests/runtime_entry_window_lifecycle_guards/sources.rs",
        "tests/runtime_entry_window_lifecycle_guards/status.rs",
        "tests/runtime_entry_window_lifecycle_guards/structure.rs",
        "tests/source_assertions.rs",
        "tests/profile_bootstrap.rs",
        "tests/builtin_engine_entry.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected entry module {relative} under {:?}",
            root
        );
    }
    assert!(
        !root.join("tests/runtime_entry_device_guards.rs")
            .exists(),
        "runtime entry device-event guards should stay folder-backed instead of returning to an umbrella runtime_entry_device_guards.rs file"
    );
    assert!(
        !root.join("tests/runtime_entry_source_guards.rs").exists(),
        "runtime entry source guards should stay folder-backed instead of returning to an umbrella runtime_entry_source_guards.rs file"
    );
    assert!(
        !root.join("tests/runtime_entry_input_guards.rs").exists(),
        "runtime entry input guards should stay folder-backed instead of returning to an umbrella runtime_entry_input_guards.rs file"
    );
    assert!(
        !root.join("tests/runtime_entry_surface_present_guards.rs")
            .exists(),
        "runtime entry surface-present guards should stay folder-backed instead of returning to an umbrella runtime_entry_surface_present_guards.rs file"
    );
    assert!(
        !root.join("tests/runtime_entry_window_lifecycle_guards.rs")
            .exists(),
        "runtime entry window-lifecycle guards should stay folder-backed instead of returning to an umbrella runtime_entry_window_lifecycle_guards.rs file"
    );
    for (relative, message) in [
        (
            "runtime_entry_app/application_handler.rs",
            "runtime application handler should stay folder-backed instead of returning to an umbrella application_handler.rs file",
        ),
        (
            "runtime_entry_app/window_attributes.rs",
            "runtime window attributes should stay folder-backed instead of returning to an umbrella window_attributes.rs file",
        ),
        (
            "runtime_entry_app/pointer_input.rs",
            "runtime pointer input should stay folder-backed instead of returning to an umbrella pointer_input.rs file",
        ),
        (
            "runtime_entry_app/ime_input.rs",
            "runtime IME input should stay folder-backed instead of returning to an umbrella ime_input.rs file",
        ),
        (
            "runtime_entry_app/keyboard_input.rs",
            "runtime keyboard input should stay folder-backed instead of returning to an umbrella keyboard_input.rs file",
        ),
        (
            "runtime_entry_app/host_requests.rs",
            "runtime host requests should stay folder-backed instead of returning to an umbrella host_requests.rs file",
        ),
        (
            "runtime_entry_app/config.rs",
            "runtime config should stay folder-backed instead of returning to an umbrella config.rs file",
        ),
        (
            "runtime_entry_app/surface_present.rs",
            "runtime surface present should stay folder-backed instead of returning to an umbrella surface_present.rs file",
        ),
        (
            "runtime_entry_app/window_surface.rs",
            "runtime window surface should stay folder-backed instead of returning to an umbrella window_surface.rs file",
        ),
        (
            "runtime_entry_app/event_loop_policy.rs",
            "runtime event-loop policy should stay folder-backed instead of returning to an umbrella event_loop_policy.rs file",
        ),
        (
            "runtime_entry_app/file_drag_drop.rs",
            "runtime file drag/drop should stay folder-backed instead of returning to an umbrella file_drag_drop.rs file",
        ),
        (
            "runtime_entry_app/window_lifecycle.rs",
            "runtime window lifecycle should stay folder-backed instead of returning to an umbrella window_lifecycle.rs file",
        ),
        (
            "runtime_entry_app/window_events.rs",
            "runtime window events should stay folder-backed instead of returning to an umbrella window_events.rs file",
        ),
        (
            "runtime_entry_app/gamepad.rs",
            "runtime gamepad should stay folder-backed instead of returning to an umbrella gamepad.rs file",
        ),
    ] {
        assert!(!root.join(relative).exists(), "{message}");
    }
}

#[test]
fn entry_uses_runtime_owned_builtin_module_list_without_manual_graphics_insertion() {
    let builtin_modules_source = include_str!("../../builtin_modules.rs");

    assert!(
        builtin_modules_source.contains("runtime_modules_for_target"),
        "entry bootstrap should source runtime modules through target-aware runtime loader"
    );
    for forbidden in [
        "use zircon_runtime::graphics::GraphicsModule;",
        "modules.insert(4, Arc::new(GraphicsModule));",
    ] {
        assert!(
            !builtin_modules_source.contains(forbidden),
            "entry builtin module bootstrap should stop keeping runtime-owned graphics registration detail `{forbidden}`"
        );
    }
}
