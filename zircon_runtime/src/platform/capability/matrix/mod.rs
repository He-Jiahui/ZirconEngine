mod gamepad;
mod input;
mod linux;
mod policy;
mod window;

use crate::RuntimeTargetMode;

use super::backends::{EventLoopPolicy, LinuxWindowProtocol};
use super::report::PlatformCapabilityReport;
use crate::platform::{PlatformFeatureSelection, PlatformTarget};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformCapabilityMatrix {
    pub features: PlatformFeatureSelection,
}

impl PlatformCapabilityMatrix {
    pub const fn new(features: PlatformFeatureSelection) -> Self {
        Self { features }
    }

    pub fn compiled() -> Self {
        Self::new(PlatformFeatureSelection::from_compiled_features())
    }

    pub fn report(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> PlatformCapabilityReport {
        let window_backend = self.window_backend(target, target_mode);
        PlatformCapabilityReport {
            target,
            target_mode,
            monitor_inventory: self.monitor_inventory_backend(target, target_mode),
            window_events: self.window_event_backend(target, target_mode),
            window_lifecycle: self.window_lifecycle_backend(target, target_mode),
            window_metrics: self.window_metrics_backend(target, target_mode),
            ime: self.ime_backend(target, target_mode),
            keyboard_events: self.keyboard_event_backend(target, target_mode),
            cursor_boundary: self.cursor_boundary_backend(target, target_mode),
            cursor_options: self.cursor_options_backend(target, target_mode),
            mouse_buttons: self.mouse_button_backend(target, target_mode),
            mouse_wheel: self.mouse_wheel_backend(target, target_mode),
            touch_events: self.touch_event_backend(target, target_mode),
            gesture_events: self.gesture_event_backend(target, target_mode),
            pointer_position: self.pointer_position_backend(target, target_mode),
            raw_mouse_motion: self.raw_mouse_motion_backend(target, target_mode),
            event_loop_policy: self.event_loop_policy(target, target_mode),
            mouse_input: self.input_backend(
                target,
                target_mode,
                self.features.input_mouse,
                "input-mouse",
            ),
            keyboard_input: self.input_backend(
                target,
                target_mode,
                self.features.input_keyboard,
                "input-keyboard",
            ),
            touch_input: self.input_backend(
                target,
                target_mode,
                self.features.input_touch,
                "input-touch",
            ),
            gesture_input: self.input_backend(
                target,
                target_mode,
                self.features.input_gestures,
                "input-gestures",
            ),
            gamepad_input: self.gamepad_backend(target, target_mode),
            gamepad_events: self.gamepad_event_backend(target, target_mode),
            gamepad_rumble: self.gamepad_rumble_backend(target, target_mode),
            file_drag_drop: self.file_drag_drop_backend(target, target_mode),
            linux_x11: self.linux_protocol(
                target,
                self.features.platform_x11,
                "platform-x11",
                LinuxWindowProtocol::X11,
            ),
            linux_wayland: self.linux_protocol(
                target,
                self.features.platform_wayland,
                "platform-wayland",
                LinuxWindowProtocol::Wayland,
            ),
            window_backend,
        }
    }

    /// Builds a report with an explicit Bevy-style update policy while keeping
    /// server/headless topology authoritative.
    pub fn report_with_event_loop_policy(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
        event_loop_policy: EventLoopPolicy,
    ) -> PlatformCapabilityReport {
        let mut report = self.report(target, target_mode);
        report.event_loop_policy =
            self.explicit_event_loop_policy(target, target_mode, event_loop_policy);
        report
    }
}
