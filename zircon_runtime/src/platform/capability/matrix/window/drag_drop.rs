use crate::core::framework::platform::RuntimeTargetMode;
use crate::platform::PlatformTarget;

use super::super::super::backends::{FileDragDropBackend, WindowBackend};
use super::super::super::status::CapabilityStatus;
use super::super::PlatformCapabilityMatrix;

impl PlatformCapabilityMatrix {
    pub(in crate::platform::capability::matrix) fn file_drag_drop_backend(
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
