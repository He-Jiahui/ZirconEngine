use crate::core::framework::platform::{PreferenceStorageBackendKind, RuntimeTargetMode};

use super::backends::{
    CursorBoundaryBackend, CursorOptionsBackend, EventLoopPolicy, FileDragDropBackend,
    GamepadBackend, GamepadEventBackend, GamepadRumbleBackend, GestureEventBackend, ImeBackend,
    InputBackend, KeyboardEventBackend, LinuxWindowProtocol, MonitorBackend, MouseButtonBackend,
    MouseWheelBackend, PointerPositionBackend, RawMouseMotionBackend, TouchEventBackend,
    WindowBackend, WindowEventBackend, WindowLifecycleBackend, WindowMetricsBackend,
};
use super::status::{format_capability, CapabilityStatus};
use crate::platform::PlatformTarget;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformCapabilityReport {
    pub target: PlatformTarget,
    pub target_mode: RuntimeTargetMode,
    pub window_backend: CapabilityStatus<WindowBackend>,
    pub monitor_inventory: CapabilityStatus<MonitorBackend>,
    pub window_events: CapabilityStatus<WindowEventBackend>,
    pub window_lifecycle: CapabilityStatus<WindowLifecycleBackend>,
    pub window_metrics: CapabilityStatus<WindowMetricsBackend>,
    pub ime: CapabilityStatus<ImeBackend>,
    pub keyboard_events: CapabilityStatus<KeyboardEventBackend>,
    pub cursor_boundary: CapabilityStatus<CursorBoundaryBackend>,
    pub cursor_options: CapabilityStatus<CursorOptionsBackend>,
    pub mouse_buttons: CapabilityStatus<MouseButtonBackend>,
    pub mouse_wheel: CapabilityStatus<MouseWheelBackend>,
    pub touch_events: CapabilityStatus<TouchEventBackend>,
    pub gesture_events: CapabilityStatus<GestureEventBackend>,
    pub pointer_position: CapabilityStatus<PointerPositionBackend>,
    pub raw_mouse_motion: CapabilityStatus<RawMouseMotionBackend>,
    pub event_loop_policy: EventLoopPolicy,
    pub mouse_input: CapabilityStatus<InputBackend>,
    pub keyboard_input: CapabilityStatus<InputBackend>,
    pub touch_input: CapabilityStatus<InputBackend>,
    pub gesture_input: CapabilityStatus<InputBackend>,
    pub gamepad_input: CapabilityStatus<GamepadBackend>,
    pub gamepad_events: CapabilityStatus<GamepadEventBackend>,
    pub gamepad_rumble: CapabilityStatus<GamepadRumbleBackend>,
    pub file_drag_drop: CapabilityStatus<FileDragDropBackend>,
    pub persistent_preferences: CapabilityStatus<PreferenceStorageBackendKind>,
    pub linux_x11: CapabilityStatus<LinuxWindowProtocol>,
    pub linux_wayland: CapabilityStatus<LinuxWindowProtocol>,
}

impl PlatformCapabilityReport {
    pub(in crate::platform) fn disabled_by_platform(mut self) -> Self {
        self.window_backend = CapabilityStatus::platform_disabled();
        self.monitor_inventory = CapabilityStatus::platform_disabled();
        self.window_events = CapabilityStatus::platform_disabled();
        self.window_lifecycle = CapabilityStatus::platform_disabled();
        self.window_metrics = CapabilityStatus::platform_disabled();
        self.ime = CapabilityStatus::platform_disabled();
        self.keyboard_events = CapabilityStatus::platform_disabled();
        self.cursor_boundary = CapabilityStatus::platform_disabled();
        self.cursor_options = CapabilityStatus::platform_disabled();
        self.mouse_buttons = CapabilityStatus::platform_disabled();
        self.mouse_wheel = CapabilityStatus::platform_disabled();
        self.touch_events = CapabilityStatus::platform_disabled();
        self.gesture_events = CapabilityStatus::platform_disabled();
        self.pointer_position = CapabilityStatus::platform_disabled();
        self.raw_mouse_motion = CapabilityStatus::platform_disabled();
        self.event_loop_policy = EventLoopPolicy::Headless;
        self.mouse_input = CapabilityStatus::platform_disabled();
        self.keyboard_input = CapabilityStatus::platform_disabled();
        self.touch_input = CapabilityStatus::platform_disabled();
        self.gesture_input = CapabilityStatus::platform_disabled();
        self.gamepad_input = CapabilityStatus::platform_disabled();
        self.gamepad_events = CapabilityStatus::platform_disabled();
        self.gamepad_rumble = CapabilityStatus::platform_disabled();
        self.file_drag_drop = CapabilityStatus::platform_disabled();
        self.persistent_preferences = CapabilityStatus::platform_disabled();
        self.linux_x11 = CapabilityStatus::platform_disabled();
        self.linux_wayland = CapabilityStatus::platform_disabled();
        self
    }

    pub fn with_preference_storage_backend(
        mut self,
        backend: PreferenceStorageBackendKind,
    ) -> Self {
        if backend.is_persistent() {
            self.persistent_preferences = CapabilityStatus::Supported(backend);
        }
        self
    }

    pub fn diagnostic_lines(&self) -> Vec<String> {
        vec![
            format!("platform.target={}", self.target.as_str()),
            format!(
                "platform.target_mode={}",
                runtime_target_mode_as_str(self.target_mode)
            ),
            format!(
                "platform.window_backend={}",
                format_capability(self.window_backend, WindowBackend::as_str)
            ),
            format!(
                "platform.monitor_inventory={}",
                format_capability(self.monitor_inventory, MonitorBackend::as_str)
            ),
            format!(
                "platform.window_events={}",
                format_capability(self.window_events, WindowEventBackend::as_str)
            ),
            format!(
                "platform.window_lifecycle={}",
                format_capability(self.window_lifecycle, WindowLifecycleBackend::as_str)
            ),
            format!(
                "platform.window_metrics={}",
                format_capability(self.window_metrics, WindowMetricsBackend::as_str)
            ),
            format!(
                "platform.ime={}",
                format_capability(self.ime, ImeBackend::as_str)
            ),
            format!(
                "platform.keyboard_events={}",
                format_capability(self.keyboard_events, KeyboardEventBackend::as_str)
            ),
            format!(
                "platform.cursor_boundary={}",
                format_capability(self.cursor_boundary, CursorBoundaryBackend::as_str)
            ),
            format!(
                "platform.cursor_options={}",
                format_capability(self.cursor_options, CursorOptionsBackend::as_str)
            ),
            format!(
                "platform.mouse_buttons={}",
                format_capability(self.mouse_buttons, MouseButtonBackend::as_str)
            ),
            format!(
                "platform.mouse_wheel={}",
                format_capability(self.mouse_wheel, MouseWheelBackend::as_str)
            ),
            format!(
                "platform.touch_events={}",
                format_capability(self.touch_events, TouchEventBackend::as_str)
            ),
            format!(
                "platform.gesture_events={}",
                format_capability(self.gesture_events, GestureEventBackend::as_str)
            ),
            format!(
                "platform.pointer_position={}",
                format_capability(self.pointer_position, PointerPositionBackend::as_str)
            ),
            format!(
                "platform.raw_mouse_motion={}",
                format_capability(self.raw_mouse_motion, RawMouseMotionBackend::as_str)
            ),
            format!(
                "platform.event_loop_policy={}",
                self.event_loop_policy.as_str()
            ),
            format!(
                "platform.mouse_input={}",
                format_capability(self.mouse_input, InputBackend::as_str)
            ),
            format!(
                "platform.keyboard_input={}",
                format_capability(self.keyboard_input, InputBackend::as_str)
            ),
            format!(
                "platform.touch_input={}",
                format_capability(self.touch_input, InputBackend::as_str)
            ),
            format!(
                "platform.gesture_input={}",
                format_capability(self.gesture_input, InputBackend::as_str)
            ),
            format!(
                "platform.gamepad_input={}",
                format_capability(self.gamepad_input, GamepadBackend::as_str)
            ),
            format!(
                "platform.gamepad_events={}",
                format_capability(self.gamepad_events, GamepadEventBackend::as_str)
            ),
            format!(
                "platform.gamepad_rumble={}",
                format_capability(self.gamepad_rumble, GamepadRumbleBackend::as_str)
            ),
            format!(
                "platform.file_drag_drop={}",
                format_capability(self.file_drag_drop, FileDragDropBackend::as_str)
            ),
            format!(
                "platform.persistent_preferences={}",
                format_capability(
                    self.persistent_preferences,
                    PreferenceStorageBackendKind::as_str
                )
            ),
            format!(
                "platform.linux_x11={}",
                format_capability(self.linux_x11, LinuxWindowProtocol::as_str)
            ),
            format!(
                "platform.linux_wayland={}",
                format_capability(self.linux_wayland, LinuxWindowProtocol::as_str)
            ),
        ]
    }

    pub fn format_diagnostics(&self) -> String {
        self.diagnostic_lines().join("\n")
    }
}

const fn runtime_target_mode_as_str(mode: RuntimeTargetMode) -> &'static str {
    match mode {
        RuntimeTargetMode::ClientRuntime => "client_runtime",
        RuntimeTargetMode::ServerRuntime => "server_runtime",
        RuntimeTargetMode::EditorHost => "editor_host",
    }
}
