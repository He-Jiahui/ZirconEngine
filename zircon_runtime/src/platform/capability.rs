use crate::RuntimeTargetMode;

use super::{PlatformFeatureSelection, PlatformTarget};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilityStatus<T> {
    Supported(T),
    FeatureDisabled { feature: &'static str },
    Unavailable { reason: &'static str },
}

impl<T> CapabilityStatus<T> {
    pub const fn is_supported(&self) -> bool {
        matches!(self, Self::Supported(_))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowBackend {
    Winit,
    BrowserCanvas,
    Headless,
}

impl WindowBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Winit => "winit",
            Self::BrowserCanvas => "browser_canvas",
            Self::Headless => "headless",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputBackend {
    WinitWindowEvents,
    BrowserEvents,
    SyntheticOnly,
}

impl InputBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitWindowEvents => "winit_window_events",
            Self::BrowserEvents => "browser_events",
            Self::SyntheticOnly => "synthetic_only",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadBackend {
    Gilrs,
    BrowserGamepadApi,
}

impl GamepadBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Gilrs => "gilrs",
            Self::BrowserGamepadApi => "browser_gamepad_api",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileDragDropBackend {
    WinitWindowEvents,
    BrowserDragEvents,
}

impl FileDragDropBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WinitWindowEvents => "winit_window_events",
            Self::BrowserDragEvents => "browser_drag_events",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinuxWindowProtocol {
    X11,
    Wayland,
}

impl LinuxWindowProtocol {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::X11 => "x11",
            Self::Wayland => "wayland",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventLoopPolicy {
    Game,
    DesktopApp,
    Mobile,
    Headless,
}

impl EventLoopPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Game => "game",
            Self::DesktopApp => "desktop_app",
            Self::Mobile => "mobile",
            Self::Headless => "headless",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformCapabilityReport {
    pub target: PlatformTarget,
    pub target_mode: RuntimeTargetMode,
    pub window_backend: CapabilityStatus<WindowBackend>,
    pub event_loop_policy: EventLoopPolicy,
    pub mouse_input: CapabilityStatus<InputBackend>,
    pub keyboard_input: CapabilityStatus<InputBackend>,
    pub touch_input: CapabilityStatus<InputBackend>,
    pub gesture_input: CapabilityStatus<InputBackend>,
    pub gamepad_input: CapabilityStatus<GamepadBackend>,
    pub file_drag_drop: CapabilityStatus<FileDragDropBackend>,
    pub linux_x11: CapabilityStatus<LinuxWindowProtocol>,
    pub linux_wayland: CapabilityStatus<LinuxWindowProtocol>,
}

impl PlatformCapabilityReport {
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
                "platform.file_drag_drop={}",
                format_capability(self.file_drag_drop, FileDragDropBackend::as_str)
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

    fn window_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<WindowBackend> {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return if self.features.platform_headless {
                CapabilityStatus::Supported(WindowBackend::Headless)
            } else {
                CapabilityStatus::FeatureDisabled {
                    feature: "platform-headless",
                }
            };
        }

        if !self.features.platform_window {
            return CapabilityStatus::FeatureDisabled {
                feature: "platform-window",
            };
        }

        match target {
            PlatformTarget::Windows | PlatformTarget::Linux | PlatformTarget::Macos => {
                if self.features.platform_winit {
                    CapabilityStatus::Supported(WindowBackend::Winit)
                } else {
                    CapabilityStatus::FeatureDisabled {
                        feature: "platform-winit",
                    }
                }
            }
            PlatformTarget::Android => {
                if !self.features.platform_winit {
                    CapabilityStatus::FeatureDisabled {
                        feature: "platform-winit",
                    }
                } else if self.features.platform_android_game_activity
                    || self.features.platform_android_native_activity
                {
                    CapabilityStatus::Supported(WindowBackend::Winit)
                } else {
                    CapabilityStatus::FeatureDisabled {
                        feature: "platform-android-game-activity",
                    }
                }
            }
            PlatformTarget::Ios => {
                if self.features.platform_winit {
                    CapabilityStatus::Supported(WindowBackend::Winit)
                } else {
                    CapabilityStatus::FeatureDisabled {
                        feature: "platform-winit",
                    }
                }
            }
            PlatformTarget::WebGpu | PlatformTarget::Wasm => {
                if self.features.platform_web {
                    CapabilityStatus::Supported(WindowBackend::BrowserCanvas)
                } else {
                    CapabilityStatus::FeatureDisabled {
                        feature: "platform-web",
                    }
                }
            }
            PlatformTarget::Headless => CapabilityStatus::Supported(WindowBackend::Headless),
        }
    }

    fn event_loop_policy(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> EventLoopPolicy {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            EventLoopPolicy::Headless
        } else if target.is_mobile() {
            EventLoopPolicy::Mobile
        } else if target_mode == RuntimeTargetMode::EditorHost {
            EventLoopPolicy::DesktopApp
        } else {
            EventLoopPolicy::Game
        }
    }

    fn input_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
        feature_enabled: bool,
        feature: &'static str,
    ) -> CapabilityStatus<InputBackend> {
        if !feature_enabled {
            return CapabilityStatus::FeatureDisabled { feature };
        }

        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Supported(InputBackend::SyntheticOnly);
        }

        match self.window_backend(target, target_mode) {
            CapabilityStatus::Supported(WindowBackend::Winit) => {
                CapabilityStatus::Supported(InputBackend::WinitWindowEvents)
            }
            CapabilityStatus::Supported(WindowBackend::BrowserCanvas) => {
                CapabilityStatus::Supported(InputBackend::BrowserEvents)
            }
            CapabilityStatus::Supported(WindowBackend::Headless) => {
                CapabilityStatus::Supported(InputBackend::SyntheticOnly)
            }
            CapabilityStatus::FeatureDisabled { feature } => {
                CapabilityStatus::FeatureDisabled { feature }
            }
            CapabilityStatus::Unavailable { reason } => CapabilityStatus::Unavailable { reason },
        }
    }

    fn file_drag_drop_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<FileDragDropBackend> {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no file drag/drop host backend",
            };
        }

        match self.window_backend(target, target_mode) {
            CapabilityStatus::Supported(WindowBackend::Winit) if target.is_desktop() => {
                CapabilityStatus::Supported(FileDragDropBackend::WinitWindowEvents)
            }
            CapabilityStatus::Supported(WindowBackend::Winit) => CapabilityStatus::Unavailable {
                reason: "mobile file drag/drop host backend is not implemented yet",
            },
            CapabilityStatus::Supported(WindowBackend::BrowserCanvas) => {
                CapabilityStatus::Unavailable {
                    reason: "browser file drag/drop host backend is not implemented yet",
                }
            }
            CapabilityStatus::Supported(WindowBackend::Headless) => CapabilityStatus::Unavailable {
                reason: "headless target has no file drag/drop host backend",
            },
            CapabilityStatus::FeatureDisabled { feature } => {
                CapabilityStatus::FeatureDisabled { feature }
            }
            CapabilityStatus::Unavailable { reason } => CapabilityStatus::Unavailable { reason },
        }
    }

    fn gamepad_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<GamepadBackend> {
        if !self.features.input_gamepad {
            return CapabilityStatus::FeatureDisabled {
                feature: "input-gamepad",
            };
        }

        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no physical gamepad backend",
            };
        }

        match target {
            PlatformTarget::Windows | PlatformTarget::Linux | PlatformTarget::Macos => {
                if self.features.gamepad_gilrs {
                    CapabilityStatus::Supported(GamepadBackend::Gilrs)
                } else {
                    CapabilityStatus::FeatureDisabled {
                        feature: "gamepad-gilrs",
                    }
                }
            }
            PlatformTarget::WebGpu | PlatformTarget::Wasm => {
                if !self.features.platform_web {
                    CapabilityStatus::FeatureDisabled {
                        feature: "platform-web",
                    }
                } else if self.features.gamepad_browser {
                    CapabilityStatus::Supported(GamepadBackend::BrowserGamepadApi)
                } else {
                    CapabilityStatus::FeatureDisabled {
                        feature: "gamepad-browser",
                    }
                }
            }
            PlatformTarget::Android | PlatformTarget::Ios => CapabilityStatus::Unavailable {
                reason: "mobile gamepad host backend is not implemented yet",
            },
            PlatformTarget::Headless => CapabilityStatus::Unavailable {
                reason: "headless target has no physical gamepad backend",
            },
        }
    }

    fn linux_protocol(
        self,
        target: PlatformTarget,
        feature_enabled: bool,
        feature: &'static str,
        protocol: LinuxWindowProtocol,
    ) -> CapabilityStatus<LinuxWindowProtocol> {
        if target != PlatformTarget::Linux {
            return CapabilityStatus::Unavailable {
                reason: "protocol is linux-specific",
            };
        }

        if feature_enabled {
            CapabilityStatus::Supported(protocol)
        } else {
            CapabilityStatus::FeatureDisabled { feature }
        }
    }
}

fn format_capability<T>(
    status: CapabilityStatus<T>,
    supported_value: impl FnOnce(T) -> &'static str,
) -> String {
    match status {
        CapabilityStatus::Supported(value) => format!("supported:{}", supported_value(value)),
        CapabilityStatus::FeatureDisabled { feature } => {
            format!("feature_disabled:{feature}")
        }
        CapabilityStatus::Unavailable { reason } => format!("unavailable:{reason}"),
    }
}

const fn runtime_target_mode_as_str(mode: RuntimeTargetMode) -> &'static str {
    match mode {
        RuntimeTargetMode::ClientRuntime => "client_runtime",
        RuntimeTargetMode::ServerRuntime => "server_runtime",
        RuntimeTargetMode::EditorHost => "editor_host",
    }
}
