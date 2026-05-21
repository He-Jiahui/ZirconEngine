use crate::RuntimeTargetMode;

use super::super::backends::{
    GestureEventBackend, InputBackend, KeyboardEventBackend, MouseButtonBackend, MouseWheelBackend,
    TouchEventBackend, WindowBackend,
};
use super::super::status::CapabilityStatus;
use super::PlatformCapabilityMatrix;
use crate::platform::PlatformTarget;

impl PlatformCapabilityMatrix {
    pub(super) fn mouse_wheel_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<MouseWheelBackend> {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no mouse wheel host backend",
            };
        }

        if !self.features.input_mouse {
            return CapabilityStatus::FeatureDisabled {
                feature: "input-mouse",
            };
        }

        match self.window_backend(target, target_mode) {
            CapabilityStatus::Supported(WindowBackend::Winit) => {
                CapabilityStatus::Supported(MouseWheelBackend::WinitWindowEvents)
            }
            CapabilityStatus::Supported(WindowBackend::BrowserCanvas) => {
                CapabilityStatus::Unavailable {
                    reason: "browser mouse wheel host backend is not implemented yet",
                }
            }
            CapabilityStatus::Supported(WindowBackend::Headless) => CapabilityStatus::Unavailable {
                reason: "headless target has no mouse wheel host backend",
            },
            CapabilityStatus::FeatureDisabled { feature } => {
                CapabilityStatus::FeatureDisabled { feature }
            }
            CapabilityStatus::Unavailable { reason } => CapabilityStatus::Unavailable { reason },
        }
    }

    pub(super) fn keyboard_event_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<KeyboardEventBackend> {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no keyboard event host backend",
            };
        }

        if !self.features.input_keyboard {
            return CapabilityStatus::FeatureDisabled {
                feature: "input-keyboard",
            };
        }

        match self.window_backend(target, target_mode) {
            CapabilityStatus::Supported(WindowBackend::Winit) => {
                CapabilityStatus::Supported(KeyboardEventBackend::WinitWindowEvents)
            }
            CapabilityStatus::Supported(WindowBackend::BrowserCanvas) => {
                CapabilityStatus::Unavailable {
                    reason: "browser keyboard event host backend is not implemented yet",
                }
            }
            CapabilityStatus::Supported(WindowBackend::Headless) => CapabilityStatus::Unavailable {
                reason: "headless target has no keyboard event host backend",
            },
            CapabilityStatus::FeatureDisabled { feature } => {
                CapabilityStatus::FeatureDisabled { feature }
            }
            CapabilityStatus::Unavailable { reason } => CapabilityStatus::Unavailable { reason },
        }
    }

    pub(super) fn mouse_button_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<MouseButtonBackend> {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no mouse button host backend",
            };
        }

        if !self.features.input_mouse {
            return CapabilityStatus::FeatureDisabled {
                feature: "input-mouse",
            };
        }

        match self.window_backend(target, target_mode) {
            CapabilityStatus::Supported(WindowBackend::Winit) => {
                CapabilityStatus::Supported(MouseButtonBackend::WinitWindowEvents)
            }
            CapabilityStatus::Supported(WindowBackend::BrowserCanvas) => {
                CapabilityStatus::Unavailable {
                    reason: "browser mouse button host backend is not implemented yet",
                }
            }
            CapabilityStatus::Supported(WindowBackend::Headless) => CapabilityStatus::Unavailable {
                reason: "headless target has no mouse button host backend",
            },
            CapabilityStatus::FeatureDisabled { feature } => {
                CapabilityStatus::FeatureDisabled { feature }
            }
            CapabilityStatus::Unavailable { reason } => CapabilityStatus::Unavailable { reason },
        }
    }

    pub(super) fn touch_event_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<TouchEventBackend> {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no touch event host backend",
            };
        }

        if !self.features.input_touch {
            return CapabilityStatus::FeatureDisabled {
                feature: "input-touch",
            };
        }

        match self.window_backend(target, target_mode) {
            CapabilityStatus::Supported(WindowBackend::Winit) => {
                CapabilityStatus::Supported(TouchEventBackend::WinitWindowEvents)
            }
            CapabilityStatus::Supported(WindowBackend::BrowserCanvas) => {
                CapabilityStatus::Unavailable {
                    reason: "browser touch event host backend is not implemented yet",
                }
            }
            CapabilityStatus::Supported(WindowBackend::Headless) => CapabilityStatus::Unavailable {
                reason: "headless target has no touch event host backend",
            },
            CapabilityStatus::FeatureDisabled { feature } => {
                CapabilityStatus::FeatureDisabled { feature }
            }
            CapabilityStatus::Unavailable { reason } => CapabilityStatus::Unavailable { reason },
        }
    }

    pub(super) fn gesture_event_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<GestureEventBackend> {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no gesture event host backend",
            };
        }

        if !self.features.input_gestures {
            return CapabilityStatus::FeatureDisabled {
                feature: "input-gestures",
            };
        }

        match self.window_backend(target, target_mode) {
            CapabilityStatus::Supported(WindowBackend::Winit)
                if matches!(target, PlatformTarget::Macos | PlatformTarget::Ios) =>
            {
                CapabilityStatus::Unavailable {
                    reason: "winit gesture event host backend is not implemented yet",
                }
            }
            CapabilityStatus::Supported(WindowBackend::Winit) => CapabilityStatus::Unavailable {
                reason: "winit gesture events are only declared for macOS and iOS targets",
            },
            CapabilityStatus::Supported(WindowBackend::BrowserCanvas) => {
                CapabilityStatus::Unavailable {
                    reason: "browser gesture event host backend is not implemented yet",
                }
            }
            CapabilityStatus::Supported(WindowBackend::Headless) => CapabilityStatus::Unavailable {
                reason: "headless target has no gesture event host backend",
            },
            CapabilityStatus::FeatureDisabled { feature } => {
                CapabilityStatus::FeatureDisabled { feature }
            }
            CapabilityStatus::Unavailable { reason } => CapabilityStatus::Unavailable { reason },
        }
    }

    pub(super) fn input_backend(
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
}
