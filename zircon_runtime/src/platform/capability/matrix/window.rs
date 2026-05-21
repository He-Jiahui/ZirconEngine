use crate::RuntimeTargetMode;

use super::super::backends::{
    CursorBoundaryBackend, CursorOptionsBackend, FileDragDropBackend, ImeBackend, MonitorBackend,
    PointerPositionBackend, RawMouseMotionBackend, WindowBackend, WindowEventBackend,
    WindowLifecycleBackend, WindowMetricsBackend,
};
use super::super::status::CapabilityStatus;
use super::PlatformCapabilityMatrix;
use crate::platform::PlatformTarget;

impl PlatformCapabilityMatrix {
    pub(super) fn window_backend(
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

    pub(super) fn monitor_inventory_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<MonitorBackend> {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no monitor inventory backend",
            };
        }

        match self.window_backend(target, target_mode) {
            CapabilityStatus::Supported(WindowBackend::Winit) => {
                CapabilityStatus::Supported(MonitorBackend::WinitMonitorHandles)
            }
            CapabilityStatus::Supported(WindowBackend::BrowserCanvas) => {
                CapabilityStatus::Unavailable {
                    reason: "browser monitor inventory host backend is not implemented yet",
                }
            }
            CapabilityStatus::Supported(WindowBackend::Headless) => CapabilityStatus::Unavailable {
                reason: "headless target has no monitor inventory backend",
            },
            CapabilityStatus::FeatureDisabled { feature } => {
                CapabilityStatus::FeatureDisabled { feature }
            }
            CapabilityStatus::Unavailable { reason } => CapabilityStatus::Unavailable { reason },
        }
    }

    pub(super) fn window_event_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<WindowEventBackend> {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no window event host backend",
            };
        }

        match self.window_backend(target, target_mode) {
            CapabilityStatus::Supported(WindowBackend::Winit) => {
                CapabilityStatus::Supported(WindowEventBackend::WinitWindowEvents)
            }
            CapabilityStatus::Supported(WindowBackend::BrowserCanvas) => {
                CapabilityStatus::Unavailable {
                    reason: "browser window event host backend is not implemented yet",
                }
            }
            CapabilityStatus::Supported(WindowBackend::Headless) => CapabilityStatus::Unavailable {
                reason: "headless target has no window event host backend",
            },
            CapabilityStatus::FeatureDisabled { feature } => {
                CapabilityStatus::FeatureDisabled { feature }
            }
            CapabilityStatus::Unavailable { reason } => CapabilityStatus::Unavailable { reason },
        }
    }

    pub(super) fn window_lifecycle_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<WindowLifecycleBackend> {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no window lifecycle host backend",
            };
        }

        match self.window_backend(target, target_mode) {
            CapabilityStatus::Supported(WindowBackend::Winit) => {
                CapabilityStatus::Supported(WindowLifecycleBackend::WinitWindowEvents)
            }
            CapabilityStatus::Supported(WindowBackend::BrowserCanvas) => {
                CapabilityStatus::Unavailable {
                    reason: "browser window lifecycle host backend is not implemented yet",
                }
            }
            CapabilityStatus::Supported(WindowBackend::Headless) => CapabilityStatus::Unavailable {
                reason: "headless target has no window lifecycle host backend",
            },
            CapabilityStatus::FeatureDisabled { feature } => {
                CapabilityStatus::FeatureDisabled { feature }
            }
            CapabilityStatus::Unavailable { reason } => CapabilityStatus::Unavailable { reason },
        }
    }

    pub(super) fn window_metrics_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<WindowMetricsBackend> {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no window metrics host backend",
            };
        }

        match self.window_backend(target, target_mode) {
            CapabilityStatus::Supported(WindowBackend::Winit) => {
                CapabilityStatus::Supported(WindowMetricsBackend::WinitWindowEvents)
            }
            CapabilityStatus::Supported(WindowBackend::BrowserCanvas) => {
                CapabilityStatus::Unavailable {
                    reason: "browser window metrics host backend is not implemented yet",
                }
            }
            CapabilityStatus::Supported(WindowBackend::Headless) => CapabilityStatus::Unavailable {
                reason: "headless target has no window metrics host backend",
            },
            CapabilityStatus::FeatureDisabled { feature } => {
                CapabilityStatus::FeatureDisabled { feature }
            }
            CapabilityStatus::Unavailable { reason } => CapabilityStatus::Unavailable { reason },
        }
    }

    pub(super) fn ime_backend(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> CapabilityStatus<ImeBackend> {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return CapabilityStatus::Unavailable {
                reason: "headless target has no ime host backend",
            };
        }

        match self.window_backend(target, target_mode) {
            CapabilityStatus::Supported(WindowBackend::Winit) if target.is_desktop() => {
                CapabilityStatus::Supported(ImeBackend::WinitIme)
            }
            CapabilityStatus::Supported(WindowBackend::Winit) => CapabilityStatus::Unavailable {
                reason: "mobile ime host backend is not implemented yet",
            },
            CapabilityStatus::Supported(WindowBackend::BrowserCanvas) => {
                CapabilityStatus::Unavailable {
                    reason: "browser ime host backend is not implemented yet",
                }
            }
            CapabilityStatus::Supported(WindowBackend::Headless) => CapabilityStatus::Unavailable {
                reason: "headless target has no ime host backend",
            },
            CapabilityStatus::FeatureDisabled { feature } => {
                CapabilityStatus::FeatureDisabled { feature }
            }
            CapabilityStatus::Unavailable { reason } => CapabilityStatus::Unavailable { reason },
        }
    }

    pub(super) fn cursor_boundary_backend(
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

    pub(super) fn cursor_options_backend(
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
                CapabilityStatus::Unavailable {
                    reason: "desktop cursor options host-request backend is not implemented yet",
                }
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

    pub(super) fn pointer_position_backend(
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

    pub(super) fn raw_mouse_motion_backend(
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

    pub(super) fn file_drag_drop_backend(
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
}
