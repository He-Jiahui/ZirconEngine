use crate::core::framework::platform::RuntimeTargetMode;
use crate::platform::PlatformTarget;

use super::super::super::backends::WindowBackend;
use super::super::super::status::CapabilityStatus;
use super::super::PlatformCapabilityMatrix;

// window_backend 是窗口能力的根判定：先处理 ServerRuntime/Headless，再处理
// platform-window 总开关，最后按桌面、移动、浏览器目标选择 winit、canvas 或 headless。
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
