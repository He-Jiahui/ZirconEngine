use crate::RuntimeTargetMode;

use super::super::backends::{GamepadBackend, GamepadEventBackend, GamepadRumbleBackend};
use super::super::status::CapabilityStatus;
use super::PlatformCapabilityMatrix;
use crate::platform::PlatformTarget;

impl PlatformCapabilityMatrix {
    pub(super) fn gamepad_backend(
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

    pub(super) fn gamepad_event_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<GamepadEventBackend> {
        if !self.features.input_gamepad {
            return CapabilityStatus::FeatureDisabled {
                feature: "input-gamepad",
            };
        }

        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no physical gamepad event backend",
            };
        }

        match target {
            PlatformTarget::Windows | PlatformTarget::Linux | PlatformTarget::Macos => {
                if self.features.gamepad_gilrs {
                    CapabilityStatus::Supported(GamepadEventBackend::GilrsEventPolling)
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
                    CapabilityStatus::Supported(GamepadEventBackend::BrowserGamepadApiPolling)
                } else {
                    CapabilityStatus::FeatureDisabled {
                        feature: "gamepad-browser",
                    }
                }
            }
            PlatformTarget::Android | PlatformTarget::Ios => CapabilityStatus::Unavailable {
                reason: "mobile gamepad event host backend is not implemented yet",
            },
            PlatformTarget::Headless => CapabilityStatus::Unavailable {
                reason: "headless target has no physical gamepad event backend",
            },
        }
    }

    pub(super) fn gamepad_rumble_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<GamepadRumbleBackend> {
        if !self.features.input_gamepad {
            return CapabilityStatus::FeatureDisabled {
                feature: "input-gamepad",
            };
        }

        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no physical gamepad rumble backend",
            };
        }

        match target {
            PlatformTarget::Windows | PlatformTarget::Linux | PlatformTarget::Macos => {
                if self.features.gamepad_gilrs {
                    CapabilityStatus::Unavailable {
                        reason: "desktop gamepad rumble host backend is not implemented yet",
                    }
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
                    CapabilityStatus::Unavailable {
                        reason: "browser gamepad rumble host backend is not implemented yet",
                    }
                } else {
                    CapabilityStatus::FeatureDisabled {
                        feature: "gamepad-browser",
                    }
                }
            }
            PlatformTarget::Android | PlatformTarget::Ios => CapabilityStatus::Unavailable {
                reason: "mobile gamepad rumble host backend is not implemented yet",
            },
            PlatformTarget::Headless => CapabilityStatus::Unavailable {
                reason: "headless target has no physical gamepad rumble backend",
            },
        }
    }
}
