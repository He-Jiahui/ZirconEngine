use crate::core::framework::platform::RuntimeTargetMode;
use crate::platform::PlatformTarget;

use super::super::super::backends::{
    CursorBoundaryBackend, CursorOptionsBackend, PointerPositionBackend, RawMouseMotionBackend,
    WindowBackend,
};
use super::super::super::status::CapabilityStatus;
use super::super::PlatformCapabilityMatrix;

// 先拒绝 server/headless，再由窗口后端筛选光标能力；进入该判定的分支原样
// 传播 FeatureDisabled/Unavailable；选项与原始运动还各有目标或输入 gate 限制。
impl PlatformCapabilityMatrix {
    pub(in crate::platform::capability::matrix) fn cursor_boundary_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<CursorBoundaryBackend> {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no cursor boundary host backend",
            };
        }

        match self.window_backend(target, target_mode) {
            CapabilityStatus::Supported(WindowBackend::Winit) => {
                CapabilityStatus::Supported(CursorBoundaryBackend::WinitWindowEvents)
            }
            CapabilityStatus::Supported(WindowBackend::BrowserCanvas) => {
                CapabilityStatus::Unavailable {
                    reason: "browser cursor boundary host backend is not implemented yet",
                }
            }
            CapabilityStatus::Supported(WindowBackend::Headless) => CapabilityStatus::Unavailable {
                reason: "headless target has no cursor boundary host backend",
            },
            CapabilityStatus::FeatureDisabled { feature } => {
                CapabilityStatus::FeatureDisabled { feature }
            }
            CapabilityStatus::Unavailable { reason } => CapabilityStatus::Unavailable { reason },
        }
    }

    pub(in crate::platform::capability::matrix) fn cursor_options_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<CursorOptionsBackend> {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no cursor options host backend",
            };
        }

        match self.window_backend(target, target_mode) {
            CapabilityStatus::Supported(WindowBackend::Winit) if target.is_desktop() => {
                CapabilityStatus::Supported(CursorOptionsBackend::WinitWindowOptions)
            }
            CapabilityStatus::Supported(WindowBackend::Winit) => CapabilityStatus::Unavailable {
                reason: "mobile cursor options host-request backend is not implemented yet",
            },
            CapabilityStatus::Supported(WindowBackend::BrowserCanvas) => {
                CapabilityStatus::Unavailable {
                    reason: "browser cursor options host backend is not implemented yet",
                }
            }
            CapabilityStatus::Supported(WindowBackend::Headless) => CapabilityStatus::Unavailable {
                reason: "headless target has no cursor options host backend",
            },
            CapabilityStatus::FeatureDisabled { feature } => {
                CapabilityStatus::FeatureDisabled { feature }
            }
            CapabilityStatus::Unavailable { reason } => CapabilityStatus::Unavailable { reason },
        }
    }

    pub(in crate::platform::capability::matrix) fn pointer_position_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<PointerPositionBackend> {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no pointer position host backend",
            };
        }

        match self.window_backend(target, target_mode) {
            CapabilityStatus::Supported(WindowBackend::Winit) => {
                CapabilityStatus::Supported(PointerPositionBackend::WinitWindowEvents)
            }
            CapabilityStatus::Supported(WindowBackend::BrowserCanvas) => {
                CapabilityStatus::Unavailable {
                    reason: "browser pointer position host backend is not implemented yet",
                }
            }
            CapabilityStatus::Supported(WindowBackend::Headless) => CapabilityStatus::Unavailable {
                reason: "headless target has no pointer position host backend",
            },
            CapabilityStatus::FeatureDisabled { feature } => {
                CapabilityStatus::FeatureDisabled { feature }
            }
            CapabilityStatus::Unavailable { reason } => CapabilityStatus::Unavailable { reason },
        }
    }

    pub(in crate::platform::capability::matrix) fn raw_mouse_motion_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<RawMouseMotionBackend> {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no raw mouse motion host backend",
            };
        }

        if !self.features.input_mouse {
            return CapabilityStatus::FeatureDisabled {
                feature: "input-mouse",
            };
        }

        match self.window_backend(target, target_mode) {
            CapabilityStatus::Supported(WindowBackend::Winit) if target.is_desktop() => {
                CapabilityStatus::Supported(RawMouseMotionBackend::WinitDeviceEvents)
            }
            CapabilityStatus::Supported(WindowBackend::Winit) => CapabilityStatus::Unavailable {
                reason: "mobile raw mouse motion host backend is not implemented yet",
            },
            CapabilityStatus::Supported(WindowBackend::BrowserCanvas) => {
                CapabilityStatus::Unavailable {
                    reason: "browser raw mouse motion host backend is not implemented yet",
                }
            }
            CapabilityStatus::Supported(WindowBackend::Headless) => CapabilityStatus::Unavailable {
                reason: "headless target has no raw mouse motion host backend",
            },
            CapabilityStatus::FeatureDisabled { feature } => {
                CapabilityStatus::FeatureDisabled { feature }
            }
            CapabilityStatus::Unavailable { reason } => CapabilityStatus::Unavailable { reason },
        }
    }
}
