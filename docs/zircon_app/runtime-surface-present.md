---
related_code:
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/mod.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/hooks.rs
  - zircon_app/src/entry/runtime_entry_app/config/mod.rs
  - zircon_app/src/entry/runtime_entry_app/config/app_config.rs
  - zircon_app/src/entry/runtime_entry_app/construct.rs
  - zircon_app/src/entry/runtime_entry_app/converters/mod.rs
  - zircon_app/src/entry/runtime_entry_app/converters/abi.rs
  - zircon_app/src/entry/runtime_entry_app/converters/keyboard.rs
  - zircon_app/src/entry/runtime_entry_app/converters/pointer.rs
  - zircon_app/src/entry/runtime_entry_app/converters/window.rs
  - zircon_app/src/entry/runtime_entry_app/device_events/mod.rs
  - zircon_app/src/entry/runtime_entry_app/device_events/dispatch.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/mod.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/control_flow.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/mod.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/cancelled.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/dropped.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/hovered.rs
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/mod.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/codes.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/events.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/host.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/rumble.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/mod.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/drain.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/mod.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/enable.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/geometry.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/request.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/surrounding_text.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/mod.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/composition.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/deletion.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/lifecycle.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/routing.rs
  - zircon_app/src/entry/runtime_entry_app/keyboard_input/mod.rs
  - zircon_app/src/entry/runtime_entry_app/keyboard_input/event.rs
  - zircon_app/src/entry/runtime_entry_app/keyboard_input/payload.rs
  - zircon_app/src/entry/runtime_entry_app/mod.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/mod.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/button.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/cursor.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/device.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/motion.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/wheel.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/mod.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/binding.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/fallback.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/lifecycle.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/redraw.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/resize.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/builder.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/fullscreen.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/monitor.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/position.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/video_mode.rs
  - zircon_app/src/entry/runtime_entry_app/window_creation.rs
  - zircon_app/src/entry/runtime_entry_app/window_events/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_events/dispatch.rs
  - zircon_app/src/entry/runtime_entry_app/window_lifecycle/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_lifecycle/close.rs
  - zircon_app/src/entry/runtime_entry_app/window_lifecycle/focus.rs
  - zircon_app/src/entry/runtime_entry_app/window_lifecycle/scale_factor.rs
  - zircon_app/src/entry/runtime_entry_app/window_lifecycle/status.rs
  - zircon_app/src/entry/runtime_entry_app/window_surface/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_surface/native_target.rs
  - zircon_runtime/src/core/framework/window/descriptor.rs
  - zircon_runtime/src/core/framework/window/lifecycle_policy.rs
  - zircon_runtime/src/core/framework/window/mode.rs
  - zircon_runtime/src/core/framework/window/monitor_selection.rs
  - zircon_runtime/src/core/framework/window/position.rs
  - zircon_runtime/src/core/framework/window/resolution.rs
  - zircon_runtime/src/core/framework/window/resize_constraints.rs
  - zircon_runtime/src/core/framework/window/video_mode_selection.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/tests/mod.rs
  - zircon_app/src/entry/tests/runtime_entry_device_guards/mod.rs
  - zircon_app/src/entry/tests/runtime_entry_device_guards/dispatch.rs
  - zircon_app/src/entry/tests/runtime_entry_device_guards/sources.rs
  - zircon_app/src/entry/tests/runtime_entry_device_guards/structure.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/mod.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/file_drag_drop.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/ime.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/keyboard.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/pointer.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/protocol.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/sources.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/mod.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/application_handler.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/config.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/converters.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/entry_tree.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/event_loop_policy.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/frame_loop.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/runtime_session.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/sources.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/viewport.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/window_attributes.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/window_events.rs
  - zircon_app/src/entry/tests/runtime_entry_surface_present_guards/mod.rs
  - zircon_app/src/entry/tests/runtime_entry_surface_present_guards/dynamic_api.rs
  - zircon_app/src/entry/tests/runtime_entry_surface_present_guards/fallback.rs
  - zircon_app/src/entry/tests/runtime_entry_surface_present_guards/resize_redraw.rs
  - zircon_app/src/entry/tests/runtime_entry_surface_present_guards/sources.rs
  - zircon_app/src/entry/tests/runtime_entry_surface_present_guards/structure.rs
  - zircon_app/src/entry/tests/source_assertions.rs
  - zircon_app/src/runtime_presenter.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime/src/dynamic_api/surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
implementation_files:
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/mod.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/hooks.rs
  - zircon_app/src/entry/runtime_entry_app/config/mod.rs
  - zircon_app/src/entry/runtime_entry_app/config/app_config.rs
  - zircon_app/src/entry/runtime_entry_app/construct.rs
  - zircon_app/src/entry/runtime_entry_app/converters/mod.rs
  - zircon_app/src/entry/runtime_entry_app/converters/abi.rs
  - zircon_app/src/entry/runtime_entry_app/converters/keyboard.rs
  - zircon_app/src/entry/runtime_entry_app/converters/pointer.rs
  - zircon_app/src/entry/runtime_entry_app/converters/window.rs
  - zircon_app/src/entry/runtime_entry_app/device_events/mod.rs
  - zircon_app/src/entry/runtime_entry_app/device_events/dispatch.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/mod.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/control_flow.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/mod.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/cancelled.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/dropped.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/hovered.rs
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/mod.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/codes.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/events.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/host.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/rumble.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/mod.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/drain.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/mod.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/enable.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/geometry.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/request.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/surrounding_text.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/mod.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/composition.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/deletion.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/lifecycle.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/routing.rs
  - zircon_app/src/entry/runtime_entry_app/keyboard_input/mod.rs
  - zircon_app/src/entry/runtime_entry_app/keyboard_input/event.rs
  - zircon_app/src/entry/runtime_entry_app/keyboard_input/payload.rs
  - zircon_app/src/entry/runtime_entry_app/mod.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/mod.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/button.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/cursor.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/device.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/motion.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/wheel.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/mod.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/binding.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/fallback.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/lifecycle.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/redraw.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/resize.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/builder.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/fullscreen.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/monitor.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/position.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/video_mode.rs
  - zircon_app/src/entry/runtime_entry_app/window_creation.rs
  - zircon_app/src/entry/runtime_entry_app/window_events/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_events/dispatch.rs
  - zircon_app/src/entry/runtime_entry_app/window_lifecycle/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_lifecycle/close.rs
  - zircon_app/src/entry/runtime_entry_app/window_lifecycle/focus.rs
  - zircon_app/src/entry/runtime_entry_app/window_lifecycle/scale_factor.rs
  - zircon_app/src/entry/runtime_entry_app/window_lifecycle/status.rs
  - zircon_app/src/entry/runtime_entry_app/window_surface/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_surface/native_target.rs
  - zircon_runtime/src/core/framework/window/lifecycle_policy.rs
  - zircon_runtime/src/core/framework/window/mode.rs
  - zircon_runtime/src/core/framework/window/monitor_selection.rs
  - zircon_runtime/src/core/framework/window/position.rs
  - zircon_runtime/src/core/framework/window/video_mode_selection.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/runtime_presenter.rs
plan_sources:
  - user: 2026-05-10 switch runtime preview to real wgpu surface present for ordinary RenderDoc capture
  - .codex/plans/ZirconEngine Bevy 式 Platform Window Input Gilrs 完成度计划.md
  - docs/superpowers/plans/2026-05-10-runtime-surface-present.md
  - dev/bevy/crates/bevy_window/src/lib.rs
  - dev/bevy/crates/bevy_window/src/raw_handle.rs
  - dev/bevy/crates/bevy_window/src/system.rs
  - dev/bevy/crates/bevy_window/src/event.rs
  - dev/bevy/crates/bevy_render/src/renderer/mod.rs
  - dev/bevy/crates/bevy_input/src/mouse.rs
  - dev/bevy/crates/bevy_winit/src/winit_windows.rs
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
tests:
  - zircon_app/src/entry/runtime_entry_app/application_handler/mod.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/hooks.rs
  - zircon_app/src/entry/runtime_entry_app/converters/mod.rs
  - zircon_app/src/entry/runtime_entry_app/converters/abi.rs
  - zircon_app/src/entry/runtime_entry_app/converters/keyboard.rs
  - zircon_app/src/entry/runtime_entry_app/converters/pointer.rs
  - zircon_app/src/entry/runtime_entry_app/converters/window.rs
  - zircon_app/src/entry/runtime_entry_app/device_events/mod.rs
  - zircon_app/src/entry/runtime_entry_app/device_events/dispatch.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/builder.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/fullscreen.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/monitor.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/position.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/video_mode.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/mod.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/control_flow.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/mod.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/drain.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/mod.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/enable.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/geometry.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/request.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/surrounding_text.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/mod.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/cancelled.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/dropped.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/hovered.rs
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/mod.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/codes.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/events.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/host.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/rumble.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/mod.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/composition.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/deletion.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/lifecycle.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/routing.rs
  - zircon_app/src/entry/runtime_entry_app/keyboard_input/mod.rs
  - zircon_app/src/entry/runtime_entry_app/keyboard_input/event.rs
  - zircon_app/src/entry/runtime_entry_app/keyboard_input/payload.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/mod.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/button.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/cursor.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/device.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/motion.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/wheel.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/mod.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/binding.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/fallback.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/lifecycle.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/redraw.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present/resize.rs
  - zircon_app/src/entry/runtime_entry_app/window_creation.rs
  - zircon_app/src/entry/runtime_entry_app/window_events/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_events/dispatch.rs
  - zircon_app/src/entry/runtime_entry_app/window_lifecycle/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_lifecycle/close.rs
  - zircon_app/src/entry/runtime_entry_app/window_lifecycle/focus.rs
  - zircon_app/src/entry/runtime_entry_app/window_lifecycle/scale_factor.rs
  - zircon_app/src/entry/runtime_entry_app/window_lifecycle/status.rs
  - zircon_app/src/entry/runtime_entry_app/window_surface/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_surface/native_target.rs
  - zircon_app/src/entry/runtime_entry_app/config/mod.rs
  - zircon_app/src/entry/runtime_entry_app/config/app_config.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/tests/mod.rs
  - zircon_app/src/entry/tests/runtime_entry_device_guards/mod.rs
  - zircon_app/src/entry/tests/runtime_entry_device_guards/dispatch.rs
  - zircon_app/src/entry/tests/runtime_entry_device_guards/sources.rs
  - zircon_app/src/entry/tests/runtime_entry_device_guards/structure.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/mod.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/file_drag_drop.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/ime.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/keyboard.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/pointer.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/protocol.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/sources.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/mod.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/application_handler.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/config.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/converters.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/entry_tree.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/event_loop_policy.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/frame_loop.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/runtime_session.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/sources.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/viewport.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/window_attributes.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/window_events.rs
  - zircon_app/src/entry/tests/runtime_entry_surface_present_guards/mod.rs
  - zircon_app/src/entry/tests/runtime_entry_surface_present_guards/dynamic_api.rs
  - zircon_app/src/entry/tests/runtime_entry_surface_present_guards/fallback.rs
  - zircon_app/src/entry/tests/runtime_entry_surface_present_guards/resize_redraw.rs
  - zircon_app/src/entry/tests/runtime_entry_surface_present_guards/sources.rs
  - zircon_app/src/entry/tests/runtime_entry_surface_present_guards/structure.rs
  - zircon_app/src/entry/tests/runtime_entry_window_lifecycle_guards/mod.rs
  - zircon_app/src/entry/tests/runtime_entry_window_lifecycle_guards/close.rs
  - zircon_app/src/entry/tests/runtime_entry_window_lifecycle_guards/focus.rs
  - zircon_app/src/entry/tests/runtime_entry_window_lifecycle_guards/scale_factor.rs
  - zircon_app/src/entry/tests/runtime_entry_window_lifecycle_guards/sources.rs
  - zircon_app/src/entry/tests/runtime_entry_window_lifecycle_guards/status.rs
  - zircon_app/src/entry/tests/runtime_entry_window_lifecycle_guards/structure.rs
  - zircon_app/src/entry/tests/source_assertions.rs
  - cargo test -p zircon_app runtime_entry_keeps_window_lifecycle_policy_source_visible --lib --no-default-features --features platform-x11,platform-wayland,input-mouse,input-keyboard,input-touch --locked
  - cargo test -p zircon_app --locked --verbose runtime_entry
  - cargo test -p zircon_app runtime_entry_maps_platform_event_loop_policy_to_winit_control_flow --lib --no-default-features --features platform-x11,platform-wayland,input-mouse,input-keyboard,input-touch --locked
  - cargo test -p zircon_app --locked --verbose
  - cargo check -p zircon_app --locked
doc_type: module-detail
---

# Runtime Surface Present

`zircon_app` owns the runtime-preview process entry, winit window, native handle extraction, and softbuffer fallback. It does not create WGPU devices, surfaces, swapchains, render pipelines, or scene renderer state. Those stay in `zircon_runtime` behind the dynamic runtime API.

## Startup Bind

Before creating the dynamic session, the runtime runner strips diagnostic log startup arguments and then accepts `--runtime-session-profile <runtime|editor|dev|minimal|headless>` or `--runtime-session-profile=dev`. The selected value is forwarded as `ZrRuntimeSessionConfigV1.profile` through `RuntimeSession::create_with_profile(...)`. `-h` and `--help` return a startup help page that lists the profile names, process log controls, and dynamic runtime library override without opening the winit window or loading the runtime library. In the current Bevy-style dev profile path, `dev` enables runtime-owned diagnostic-store log cadence from `zircon_runtime`; the app runner only forwards the startup policy and does not inspect or emit runtime diagnostics itself.

After the dynamic session is created, the same parsed session profile is projected into `RuntimeEntryAppConfig`. The default `runtime` profile keeps the visible primary window, `EventLoopPolicy::Game`, and `WindowLifecyclePolicy::default()` (`close_when_requested = true`, `WindowExitCondition::OnAllClosed`). The `editor` and `dev` profiles keep the primary window but select `EventLoopPolicy::DesktopApp`, matching Bevy's split between game-style continuous update and desktop-app reactive policy. The `minimal` and `headless` profiles select `WindowDescriptor::without_primary_window()`, `EventLoopPolicy::Headless`, and `WindowExitCondition::DontExit`, so `ApplicationHandler::can_create_surfaces()` returns before creating a concrete winit window or presenter and the process is not forced to exit by a missing primary window. The close-request setting mirrors Bevy `WindowPlugin::close_when_requested`: the preview host always forwards `window_close_requested` to the runtime, then closes and exits only when the configured lifecycle policy allows it.

Runtime host configuration is folder-backed under `runtime_entry_app/config/`. Root `mod.rs` only exposes `RuntimeEntryAppConfig`, while `app_config.rs` owns the neutral window descriptor, event-loop policy, lifecycle policy, builder methods, accessors, defaults, and focused unit coverage. This mirrors the Bevy configuration split at the host boundary: `WindowPlugin` owns primary-window, close-request, and exit-condition defaults in `bevy_window/src/lib.rs`, while `WinitSettings` owns update policy vocabulary in `bevy_winit/src/winit_config.rs`. Zircon keeps those choices in the app host config and still applies concrete winit behavior in `window_creation.rs`, `window_lifecycle/`, and `event_loop_policy/control_flow.rs`.

The runtime app keeps winit trait hook wiring folder-backed under `runtime_entry_app/application_handler/`. Root `mod.rs` is structural, and `hooks.rs` implements `ApplicationHandler` by profiling and delegating `can_create_surfaces()`, `window_event()`, `about_to_wait()`, and `device_event()` into the focused window-creation, window-event, frame-loop, and device-event modules. This mirrors Bevy's `bevy_winit/src/state.rs` runner shape where the `ApplicationHandler` implementation is the host hook surface and concrete event behavior is routed onward. `ApplicationHandler::can_create_surfaces()` delegates the actual primary-window startup flow to `runtime_entry_app/window_creation.rs`. That module applies the no-primary-window policy, creates the concrete winit window, reads the window surface size, stores a nonzero `ZrRuntimeViewportSizeV1`, and calls `resize_viewport()` before attempting any native surface bind. This ordering keeps runtime viewport metadata aligned with the first WGPU surface configuration without letting the hook file own window creation behavior.

Window creation now starts from the neutral runtime `WindowDescriptor` instead of hard-coded app literals. `runtime_entry_app/window_attributes/` translates the descriptor into winit `WindowAttributes`: title, physical surface size, resize constraints, resizable/decorated/visible/focused flags, explicit physical position, centered monitor placement, and fullscreen intent. The root `mod.rs` only exposes `runtime_window_attributes(...)`; `builder.rs` owns the final `WindowAttributes` assembly, `monitor.rs` owns active-event-loop monitor capture and selection, `position.rs` owns centered/explicit physical placement, `fullscreen.rs` owns borderless/exclusive fullscreen policy, and `video_mode.rs` owns current/specific video-mode matching. `WindowPosition::Centered` / `WindowMode::BorderlessFullscreen` / `WindowMode::Fullscreen` keep their primary-monitor defaults, while `WindowPosition::CenteredOn`, `WindowMode::BorderlessFullscreenOn`, and `WindowMode::FullscreenOn { monitor, video_mode }` expose Bevy-style monitor and video-mode selection through neutral runtime DTOs. `WindowMonitorSelection::Current` intentionally maps to winit's current-monitor fallback during creation because there is not yet a concrete window current monitor; `Primary` and `Index(n)` use the active event loop monitor inventory. `WindowVideoModeSelection::Specific` matches physical size first and treats bit depth and refresh rate as optional constraints, so descriptors can request a broad 1920x1080 mode or an exact 1920x1080@60Hz/32-bit mode. The centered-position math is pure and saturating: if the requested window is larger than the monitor, placement stays at the monitor origin instead of underflowing. This keeps the Bevy-style split intact: `zircon_runtime::core::framework::window` owns the window vocabulary, while `zircon_app` owns the concrete winit representation. The no-primary-window branch follows Bevy `WindowPlugin::primary_window: Option<Window>` in `dev/bevy/crates/bevy_window/src/lib.rs`, where `None` does not spawn the primary window; monitor-aware placement and fullscreen selection follow Bevy `bevy_winit/src/winit_windows.rs` and `bevy_winit/src/system.rs`; the event-loop policy side follows Bevy `WinitSettings` in `dev/bevy/crates/bevy_winit/src/winit_config.rs`.

Winit input and window-status scalar conversion is kept in the folder-backed `runtime_entry_app/converters/` module, following Bevy's `dev/bevy/crates/bevy_winit/src/converters.rs` split from `dev/bevy/crates/bevy_winit/src/state.rs`. The root `converters/mod.rs` only re-exports focused helper families: `abi.rs` owns byte-slice and usize-bound conversion, `keyboard.rs` owns key action and physical key-code conversion, `pointer.rs` owns pointer/touch IDs, button states, mouse buttons, and wheel units, and `window.rs` owns theme conversion. Native runtime host requests are folder-backed under `runtime_entry_app/host_requests/`: after each dynamic runtime tick, `drain.rs` drains optional `ZrRuntimeHostRequestV1` batches, `routing.rs` dispatches host request families, and `ime/` applies desktop IME enable/update/disable requests through winit `Window::request_ime_update`. Concrete `WindowEvent` dispatch is folder-backed under `runtime_entry_app/window_events/`: `mod.rs` is structural, `application_handler/hooks.rs` profiles the winit hook and delegates the event to that child module, while `dispatch.rs` matches the concrete winit arms and routes them to keyboard-input, IME-input, file-drag/drop, pointer-input, window-lifecycle, and surface-present helpers. Keyboard forwarding is folder-backed under `runtime_entry_app/keyboard_input/`: `event.rs` owns winit key action conversion, physical key-code conversion, runtime event construction, and `keyboard` submission, while `payload.rs` owns text payload byte-slice construction. IME input forwarding is folder-backed under `runtime_entry_app/ime_input/`: `routing.rs` owns the winit `Ime` dispatch, `lifecycle.rs` owns enabled/disabled runtime events, `composition.rs` owns preedit/commit byte-slice and cursor sentinel handling, and `deletion.rs` owns delete-surrounding byte-count clamping. File drag/drop forwarding is folder-backed under `runtime_entry_app/file_drag_drop/`: `hovered.rs` owns hover path string conversion and `file_hovered` submission, `dropped.rs` owns drop path string conversion and `file_dropped` submission, `cancelled.rs` owns cancellation forwarding, and root `mod.rs` is structural wiring. This remains app-host input forwarding and does not take ownership of asset import or hot-reload behavior. Pointer and mouse host event forwarding is kept in folder-backed `runtime_entry_app/pointer_input/`: `cursor.rs` owns cursor enter/leave and touch cancellation, `motion.rs` owns pointer moved and touch move conversion, `button.rs` owns pointer button conversion to touch/mouse ABI events, `wheel.rs` owns mouse wheel units, and `device.rs` owns raw device pointer motion. Native surface presentation is folder-backed under `runtime_entry_app/surface_present/`: `binding.rs` owns `runtime_native_surface_target(...)` usage plus optional ABI bind calls, `lifecycle.rs` owns enable/fallback/failure/unbind diagnostics and Drop cleanup, `fallback.rs` owns `SoftbufferRuntimePresenter` creation, `redraw.rs` owns redraw-time `present_viewport()` / `capture_frame()` selection, `resize.rs` owns resize-time viewport/surface/presenter synchronization, and root `mod.rs` is structural wiring. Native window-surface target extraction is folder-backed under `runtime_entry_app/window_surface/`: root `mod.rs` only re-exports `runtime_native_surface_target(...)`, while `native_target.rs` owns raw-window/display-handle extraction and Win32 ABI target construction. Primary window creation is kept in `runtime_entry_app/window_creation.rs`, which owns `ActiveEventLoop::create_window(...)`, initial viewport resize, initial native-surface bind, and fallback presenter seeding. Window lifecycle and window-status forwarding are kept in `runtime_entry_app/window_lifecycle/`: `close.rs` owns close-request policy, `status.rs` owns destroyed/moved/occluded/theme forwarding, `scale_factor.rs` owns backend-before-logical scale-factor forwarding, and `focus.rs` owns focus-to-lifecycle mapping. The application-handler folder remains the winit trait hook surface for can-create-surfaces, window-event, about-to-wait, and device-event handoff instead of owning each backend behavior family itself.

Window lifecycle policy follows the same Bevy source split: `dev/bevy/crates/bevy_window/src/lib.rs` registers close, destroyed, focused, occluded, moved, theme, and scale-factor messages, while `dev/bevy/crates/bevy_winit/src/state.rs` translates concrete winit events into those messages. Zircon routes the concrete winit arms from `window_events/dispatch.rs` into folder-backed `runtime_entry_app/window_lifecycle/`: root `mod.rs` is structural, `close.rs` sends `window_close_requested` before consulting `WindowLifecyclePolicy::should_close_on_request()` and `should_exit_after_primary_close()`, `status.rs` forwards destroyed/moved/occluded/theme runtime events, `scale_factor.rs` forwards backend scale-factor before logical scale-factor, and `focus.rs` maps focus changes to runtime foreground/background lifecycle states.

Frame-loop pumping is kept in `runtime_entry_app/frame_loop.rs`: `ApplicationHandler::about_to_wait()` profiles the winit hook and delegates to `pump_frame_loop(...)`, while the child module owns event-loop policy application, optional gilrs polling, dynamic runtime `tick_frame()`, host-request drain, and redraw scheduling. Event-loop policy mapping is folder-backed under `runtime_entry_app/event_loop_policy/`: root `mod.rs` is structural, and `control_flow.rs` owns the `EventLoopPolicy` to winit `ControlFlow` mapping. `Game` and `Continuous` use `ControlFlow::Poll`; `DesktopApp`, `Mobile`, and `Headless` use `ControlFlow::Wait`. This follows Bevy's split where `WinitSettings` / `UpdateMode` define game, desktop-app, mobile, continuous, and reactive policy vocabulary in `bevy_winit/src/winit_config.rs`, while `bevy_winit/src/state.rs` applies the resulting `ControlFlow` to the active event loop.

Raw device-event dispatch is folder-backed under `runtime_entry_app/device_events/`. `ApplicationHandler::device_event()` still only profiles and delegates, root `mod.rs` is structural, and `dispatch.rs` owns the narrow handoff into `pointer_input/device.rs`. This follows Bevy's source boundary where `bevy_winit/src/state.rs` matches `DeviceEvent::MouseMotion` and emits the stable `bevy_input::mouse::MouseMotion` vocabulary; Zircon keeps runtime ABI construction in the pointer-input module and does not introduce UI picking, camera control, or render ownership in the dispatcher.

The optional gilrs host is folder-backed under `runtime_entry_app/gamepad/`, following Bevy's split between `bevy_gilrs/src/lib.rs`, `bevy_gilrs/src/gilrs_system.rs`, and `bevy_gilrs/src/rumble.rs`. The root `gamepad/mod.rs` is structural; `host.rs` owns `GilrsBuilder` startup and warning output, `polling.rs` owns connection announcement plus per-frame `EventType` draining, `events.rs` owns `ZrRuntimeEventV1::gamepad_connection_with_ids` / `gamepad_button` / `gamepad_axis` construction, `codes.rs` owns gilrs button/axis to runtime ABI code mapping, and `rumble.rs` owns gilrs force-feedback effect construction, error classification, lifetime tracking, disconnect cleanup, and shutdown cleanup. This keeps the app-host backend polling distinct from the neutral runtime gamepad ABI and from higher-level UI picking or input-focus ownership.

After resize, `surface_present/binding.rs` calls `runtime_native_surface_target(window)` to convert the winit window and display handles into a `ZrRuntimeNativeSurfaceTargetV1`. The current app-side extractor emits only Win32 descriptors from `window_surface/native_target.rs`. This mirrors Bevy's separation where `bevy_window/src/raw_handle.rs` wraps raw window/display handles and `bevy_render/src/renderer/mod.rs` consumes the wrapper through `Instance::create_surface(...)`: Zircon keeps the app host responsible only for opaque ABI target extraction, while concrete WGPU surface creation stays inside the dynamic runtime. If extraction fails, or if the loaded runtime does not expose a coherent optional bind/unbind/present ABI set, the app skips native present and `surface_present/fallback.rs` creates `SoftbufferRuntimePresenter` for the existing CPU readback path.

When extraction succeeds, `RuntimeSession::bind_viewport_surface()` sends a `ZrRuntimeBindViewportSurfaceRequestV1::new(...)` request through the optional dynamic ABI. A successful bind calls `enable_surface_present()` and logs `runtime_surface_present_enabled`. An unavailable bind logs `runtime_surface_present_fallback`. A bind error logs `runtime_surface_present_failed`, unbinds best-effort when a bind was attempted, and then uses the fallback presenter.

## Redraw Flow

`WindowEvent::RedrawRequested` delegates to `surface_present/redraw.rs`, which first checks `surface_present_enabled && !surface_present_failed`. While that gate is true, redraw calls `RuntimeSession::present_viewport()` and returns immediately on `Ok(true)`. This is the app-side entry to the runtime WGPU path that eventually calls `SurfaceTexture::present()`.

If `present_viewport()` returns `Ok(false)` or an error, the app marks `surface_present_failed`, logs `runtime_surface_present_failed`, disables/unbinds the native path, creates the softbuffer presenter on demand, and continues through the existing fallback branch in the same redraw event. The fallback branch calls `RuntimeSession::capture_frame()` and passes the CPU RGBA frame to `SoftbufferRuntimePresenter::present()`.

`about_to_wait()` delegates to `frame_loop.rs`, which applies the current platform event-loop policy through `event_loop_policy/control_flow.rs` before runtime ticks and redraw scheduling. The runtime-preview host maps `EventLoopPolicy::Game` and `EventLoopPolicy::Continuous` to winit `ControlFlow::Poll`, and maps `DesktopApp`, `Mobile`, and `Headless` to `ControlFlow::Wait`. The preview app still calls `request_redraw()` on the window each loop after the runtime tick and host-request drain so both native surface present and softbuffer fallback remain frame-driven.

## Resize And Teardown

`WindowEvent::SurfaceResized` delegates to `surface_present/resize.rs`. The helper clamps the size through `ZrRuntimeViewportSizeV1`, forwards it through `resize_viewport()`, rebinds the active native surface only when surface-present is enabled and has not failed, and resizes the softbuffer presenter when it exists. Keeping resize under the same folder as bind/present prevents the event handler from duplicating surface-present state transitions.

`RuntimeEntryApp::drop()` calls `disable_surface_present()`, which unbinds the viewport surface when native present is enabled or was attempted. `RuntimeSession::drop()` also performs a best-effort unbind for the current default runtime viewport before destroying the session. That duplicate cleanup is intentionally harmless because the ABI unbind path is optional and best-effort.

## RenderDoc Launch

For manual Windows validation, launch the runtime preview from RenderDoc or from a shell with the same environment:

```powershell
$env:WGPU_BACKEND='dx12'
$env:WGPU_DEBUG='1'
$env:WGPU_VALIDATION='1'
cargo run -p zircon_app --bin zircon_runtime --locked
```

If testing with explicit features instead of defaults, use `cargo run -p zircon_app --no-default-features --features target-client --bin zircon_runtime --locked`.

RenderDoc ordinary `Capture Frame` should show the runtime process as presenting and the captured frame should end in a swapchain present after `zircon-present-blit-pass`. If the app falls back to softbuffer, inspect diagnostics in this order: native handle extraction, optional ABI availability, runtime surface bind, backend surface creation/configuration, and present call status.

## Scope

This path is for runtime preview. Editor viewport embedding remains on offscreen readback until a separate editor GPU embedding milestone. The fallback path is not legacy dead code; it remains required for unsupported native surfaces, dynamic runtimes without the optional ABI fields, headless/test workflows, and editor viewport imports.

## Validation

The 2026-05-12 app validation ran `cargo test -p zircon_app --locked --verbose runtime_entry`, `cargo check -p zircon_app --locked`, `cargo fmt -p zircon_app --check`, and the full package command `cargo test -p zircon_app --locked --verbose`. The final full package run passed with `41 passed; 0 failed`; the runtime-preview binary test target and doc tests both ran zero tests successfully. Manual RenderDoc validation remains separate because it needs an interactive Windows GPU/window capture session.

Workspace validation has also exercised this app path through `./.opencode/skills/zircon-dev/scripts/validate-matrix.ps1 -VerboseOutput`: the workspace build phase passed, and the later test phase reached `zircon_editor --lib` before failing in retained host template projection. No app-side runtime-surface regression was identified in that validator run.

The runtime session profile forwarding slice adds a focused app entry source guard plus parser unit coverage for the new startup argument and help output. Current slice validation is limited to Rust formatting and source hygiene while concurrent Cargo/rustc jobs are active; package-level `zircon_app` validation must be rerun at the milestone testing stage.

The runtime-entry host-config slice adds `RuntimeEntryAppConfig` unit coverage, runtime-runner profile-mapping unit coverage, and app entry source guards for the profile-to-host projection. It was implemented after reading Bevy `WindowPlugin::primary_window` and `WinitSettings` source references above; focused Cargo validation remains part of the M3 testing stage when current unrelated workspace blockers are quiet.

The runtime-entry converter split first moved pure scalar conversion out of `application_handler.rs`, then the converter-family split converted the old umbrella `converters.rs` into `runtime_entry_app/converters/`. Module-local tests now sit beside the focused helper families for button/key state constants, touch ID preservation, mouse button and wheel units, theme/key-code mapping, byte-slice construction, and usize saturation. App entry source guards read the converter root plus all child modules when checking the full winit-to-runtime event translation surface.

The runtime host-request split moves native IME request application from `application_handler.rs` into `runtime_entry_app/host_requests/`. The host-request family is now folder-backed: `mod.rs` is structural, `drain.rs` owns `RuntimeSession::drain_host_requests()` and request iteration, `routing.rs` owns `ZrRuntimeHostRequestV1` family dispatch and warning emission, `ime/request.rs` owns IME host-request kind mapping, `ime/enable.rs` owns enable capabilities plus default IME data, `ime/geometry.rs` owns cursor-area logical conversion, and `ime/surrounding_text.rs` owns surrounding-text construction and validation. Gamepad rumble requests now route through the same `ZrRuntimeHostRequestV1` dispatch surface as `GamepadRumble`, and the native preview host executes them through gilrs force-feedback effects (with per-gamepad effect lifetime tracking plus stop, disconnect, and shutdown cleanup). Source guards require the folder wiring, reject returning to `runtime_entry_app/host_requests.rs`, and keep `drain_host_requests`, `ZrRuntimeHostRequestV1`, `request_ime_update`, `ImeCapabilities`, and `ImeSurroundingText` source-visible in focused child modules without letting the winit `ApplicationHandler` file grow another behavior family.

The runtime surface-present split first moved native surface binding, optional ABI bind/unbind, resize-time viewport/surface/presenter synchronization, redraw-time `present_viewport()` / `capture_frame()` selection, surface-present diagnostics, and softbuffer presenter creation from `application_handler.rs` into a child module. The surface-present family is now folder-backed under `runtime_entry_app/surface_present/`: `mod.rs` is structural, `binding.rs` owns optional ABI bind setup, `lifecycle.rs` owns enable/fallback/failure/unbind diagnostics plus Drop cleanup, `fallback.rs` owns softbuffer presenter construction, `redraw.rs` owns redraw-time native present versus capture fallback, and `resize.rs` owns resize-time viewport/native/presenter synchronization. Source guards now require the folder wiring, reject returning to `runtime_entry_app/surface_present.rs`, verify that the handler no longer owns `fn bind_window_surface`, and keep the resize, bind, present, failure, unbind, diagnostic, and fallback paths source-visible while `window_events/dispatch.rs` owns winit resize/redraw event routing.

The native window-surface target split moves the raw-window/display-handle extractor from the flat `runtime_entry_app/window_surface.rs` file into folder-backed `runtime_entry_app/window_surface/`. The root stays structural and re-exports the stable `runtime_native_surface_target(...)` helper consumed by `surface_present/binding.rs`; `native_target.rs` owns the `HasWindowHandle` / `HasDisplayHandle` reads, the Win32 display/window handle match, and `ZrRuntimeNativeSurfaceTargetV1::win32(...)` construction. Source guards reject returning to `runtime_entry_app/window_surface.rs`, require the folder wiring, and keep the raw handle path source-visible without introducing direct `wgpu` ownership in `zircon_app`.

The runtime window-creation split moves the concrete `ActiveEventLoop::create_window(...)` startup path from `application_handler.rs` into `runtime_entry_app/window_creation.rs`. Source guards now require `mod window_creation;`, verify that `ApplicationHandler::can_create_surfaces()` delegates to `create_primary_window_surface(...)`, and keep no-primary-window policy, monitor-aware attribute creation, initial nonzero viewport sizing, resize-before-bind ordering, and fallback presenter seeding source-visible in the child module.

The window lifecycle policy guard adds source-visible coverage for no-primary-window creation skip, close-before-policy-exit ordering, destroyed/moved/occluded/theme forwarding, backend-before-logical scale-factor forwarding, and focus-to-lifecycle mapping. It is now folder-backed under `runtime_entry_window_lifecycle_guards/`: `mod.rs` stays structural, `sources.rs` centralizes source fixtures, and focused child files own close, status, scale-factor, focus, and lifecycle structure assertions. The guards read the `window_lifecycle/` subtree directly so those policy and status transitions stay covered after leaving `application_handler.rs`, while source guards reject returning to umbrella `runtime_entry_window_lifecycle_guards.rs` or `runtime_entry_app/window_lifecycle.rs` files. `RuntimeEntryAppConfig` unit coverage also guards the default `WindowLifecyclePolicy`, disabled close requests, full policy override, and headless `WindowExitCondition::DontExit` projection. It cites Bevy `WindowPlugin`/window events and `bevy_winit` state handling as the parity anchors.

The pointer/mouse input split first moved cursor boundary forwarding, pointer moved forwarding, touch move/cancel conversion, mouse button conversion, wheel unit conversion, and raw pointer device motion from `application_handler.rs` into a child module. The pointer-family split now keeps that child module folder-backed under `runtime_entry_app/pointer_input/`: `mod.rs` is structural, `cursor.rs` owns cursor enter/leave plus touch cancellation, `motion.rs` owns pointer move/touch move forwarding, `button.rs` owns touch and mouse button forwarding, `wheel.rs` owns line/pixel wheel forwarding, and `device.rs` owns raw pointer device motion. Source guards require the folder wiring, reject returning to `runtime_entry_app/pointer_input.rs`, verify `window_events/dispatch.rs` delegates pointer/mouse winit arms and the handler delegates device events to helper methods, and keep the runtime event constructors plus converter calls source-visible in focused child modules. This mirrors the Bevy split where concrete winit pointer and mouse events are translated in `bevy_winit/src/state.rs` while stable input vocabulary lives in `bevy_window` cursor events and `bevy_input` mouse/touch modules.

The device-event dispatcher split moves the final raw-device routing wrapper from the flat `runtime_entry_app/device_events.rs` file into folder-backed `runtime_entry_app/device_events/`. Source guards reject returning to `runtime_entry_app/device_events.rs`, require structural root wiring, and keep the `handle_device_event(...)` to `handle_pointer_device_event(...)` handoff visible without moving raw pointer-motion ABI construction out of `pointer_input/device.rs`. The device-event guard suite is also folder-backed under `runtime_entry_device_guards/`: `mod.rs` stays structural, `sources.rs` centralizes include fixtures, `structure.rs` owns path/absent-file assertions, and `dispatch.rs` owns hook-to-pointer dispatch assertions.

The file drag/drop split first moved `WindowEvent::DragEntered`, `DragDropped`, and `DragLeft` forwarding out of `application_handler.rs`. The file-drag/drop family split now keeps that child module folder-backed under `runtime_entry_app/file_drag_drop/`: `mod.rs` is structural, `hovered.rs` owns hovered path string conversion and `file_hovered`, `dropped.rs` owns dropped path string conversion and `file_dropped`, and `cancelled.rs` owns `file_drag_cancelled`. Source guards require the folder wiring, reject returning to `runtime_entry_app/file_drag_drop.rs`, verify `window_events/dispatch.rs` delegates those arms, and keep `file_hovered`, `file_dropped`, `file_drag_cancelled`, path string conversion, and `byte_slice(...)` source-visible in focused child modules. This remains an app-host input forwarding boundary only; asset import and hot-reload behavior stay in the asset milestones.

The keyboard and IME input splits move `WindowEvent::KeyboardInput` forwarding into `runtime_entry_app/keyboard_input/` and `WindowEvent::Ime` forwarding into `runtime_entry_app/ime_input/`. The keyboard family is now folder-backed: `event.rs` owns the winit `KeyEvent` forwarding path, runtime `keyboard` event construction, key action conversion, physical key-code conversion, and session submission, while `payload.rs` owns text byte-slice construction. The IME family is also folder-backed: `routing.rs` owns the winit `Ime` match, `lifecycle.rs` owns enabled/disabled forwarding, `composition.rs` owns preedit/commit byte-slice and cursor sentinel handling, and `deletion.rs` owns delete-surrounding byte-count clamping. Source guards now require `mod keyboard_input;` and `mod ime_input;`, reject returning to `runtime_entry_app/keyboard_input.rs` or `runtime_entry_app/ime_input.rs`, verify `window_events/dispatch.rs` delegates both arms, and keep `keyboard`, IME event construction, text byte-slice handling, physical key-code conversion, and byte-count clamping source-visible in focused child modules. This keeps the event routing aligned with Bevy's `bevy_winit` shape while leaving UI focus, text-edit widget semantics, and accessibility ownership to later milestones.

The frame-loop split moves `about_to_wait()` pump behavior into `runtime_entry_app/frame_loop.rs`. Source guards now require `mod frame_loop;`, verify the handler delegates the hook, and keep event-loop policy application, optional gilrs polling, dynamic runtime `tick_frame()`, host-request drain, and redraw scheduling source-visible in the child module. The runtime-entry app source guards are split under `zircon_app/src/entry/tests/` so `tests/mod.rs` stays structural while input/event translation checks live in the folder-backed `runtime_entry_input_guards/` suite, surface-present and native window-surface ownership checks live in the folder-backed `runtime_entry_surface_present_guards/` suite, window lifecycle policy/status checks live in the folder-backed `runtime_entry_window_lifecycle_guards/` suite, and broader runtime-entry ownership checks live in the folder-backed `runtime_entry_source_guards/` suite. The input guard suite keeps `mod.rs` structural, centralizes source fixtures in `sources.rs`, and moves pointer, file drag/drop, keyboard, IME, and cross-protocol translation guards into focused child files. The surface-present guard suite also keeps `mod.rs` structural, centralizes source fixtures in `sources.rs`, and moves dynamic-API ownership, softbuffer fallback, folder-backed surface structure, and resize/redraw/teardown ordering guards into focused child files. The broader source guard suite keeps `mod.rs` structural and moves broad guards into focused child files for application-handler hooks, config/profile projection, converter-family structure, entry-tree structure, event-loop policy mapping, frame-loop order, runtime-session profile forwarding, viewport ownership, monitor-aware window attributes, and concrete window-event dispatch.

The gamepad-family split keeps optional desktop gamepad polling source-visible without leaving a broad `gamepad.rs` file. Source guards now read `gamepad/mod.rs`, `codes.rs`, `events.rs`, `host.rs`, and `polling.rs`; require root structural wiring; reject returning to `runtime_entry_app/gamepad.rs`; and preserve gilrs startup, connection announcement, button/axis event forwarding, and runtime ABI code mapping as separate helper families.

The window-event dispatch split first moved the concrete `WindowEvent` match from `application_handler.rs` into `runtime_entry_app/window_events.rs`. The dispatcher is now folder-backed under `runtime_entry_app/window_events/`: `mod.rs` is structural and `dispatch.rs` owns `handle_window_event(...)`, matching concrete winit arms and delegating to the focused window-lifecycle, surface-present, pointer-input, file-drag/drop, keyboard-input, and IME-input helper families. Source guards now require `mod window_events;`, require root `mod dispatch;`, reject returning to `runtime_entry_app/window_events.rs`, verify that `ApplicationHandler::window_event()` only profiles and delegates to `handle_window_event(...)`, reject concrete `WindowEvent::*` arms in the handler, and keep close/destroy/move/occlusion/theme/scale/focus, surface resize/redraw, pointer/mouse, drag/drop, keyboard, and IME dispatch source-visible in `dispatch.rs`.

The application-handler hook split replaces the flat `runtime_entry_app/application_handler.rs` file with folder-backed `runtime_entry_app/application_handler/`. Source guards now reject returning to the old flat file, require structural root wiring through `mod hooks;`, and keep `ApplicationHandler::{can_create_surfaces, window_event, about_to_wait, device_event}` source-visible in `hooks.rs` as profile-and-delegate hooks. This keeps the same call surface for `window_creation.rs`, `window_events/dispatch.rs`, `frame_loop.rs`, and `device_events/dispatch.rs` while matching Bevy's `bevy_winit/src/state.rs` runner-hook layout.

Current M67 validation ran scoped `rustfmt --edition 2021 --check`, `cargo metadata --locked --no-deps --format-version 1`, a 20-marker source contract, scoped `git diff --check`, and trailing-whitespace scans over the application-handler split plus source guards, docs, plan, and session note. Package-level `zircon_app` checks remain deferred to the milestone testing stage or a quieter compile window because active Cargo/rustc work was still present during this implementation slice.

Current M68 validation ran scoped `rustfmt --edition 2021 --check`, `cargo metadata --locked --no-deps --format-version 1`, a 44-marker source contract, scoped `git diff --check`, live old-include scanning, and trailing-whitespace/conflict scans over the window-event folder split plus app source guards, docs, plan, and session note. Package-level `zircon_app` checks remain deferred to the milestone testing stage or a quieter compile window because active Cargo/rustc work was still present during this implementation slice.

The monitor-aware window creation slice makes `runtime_window_attributes(...)` consume the active event-loop monitor context. Source and module-local guards cover centered placement, selected-monitor policy, centered physical-position math, exclusive fullscreen preference, specific video-mode matching, borderless fallback, and the no-monitor fallback for tests/headless hosts. Bevy anchors are `dev/bevy/crates/bevy_window/src/window.rs` for `WindowPosition` / `WindowMode` / `MonitorSelection` / `VideoModeSelection` vocabulary and `dev/bevy/crates/bevy_winit/src/winit_windows.rs` for monitor/video-mode selection at window creation.

The window-attribute family split keeps that Bevy-style host boundary readable as it grows. Source guards now reject returning to a broad `window_attributes.rs` file and require the folder-backed `builder`, `monitor`, `position`, `fullscreen`, and `video_mode` files. This is a structure-only refactor: it preserves the same public helper consumed by `window_creation.rs`, the same no-monitor fallback for tests/headless hosts, and the same descriptor-to-winit semantics.
