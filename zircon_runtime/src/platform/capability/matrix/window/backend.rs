use crate::core::framework::platform::RuntimeTargetMode;
use crate::platform::PlatformTarget;

use super::super::super::backends::WindowBackend;
use super::super::super::status::CapabilityStatus;
use super::super::PlatformCapabilityMatrix;

impl PlatformCapabilityMatrix {
    pub(in crate::platform::capability::matrix) fn window_backend(
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
}
