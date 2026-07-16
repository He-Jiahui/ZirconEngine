use crate::core::framework::platform::RuntimeTargetMode;
use crate::diagnostic_log::{DiagnosticStoreLogSchedule, DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT};

const DEFAULT_DYNAMIC_RUNTIME_MAX_FIXED_STEPS_PER_FRAME: u32 = 8;
const RUNTIME_SESSION_PROFILE_RUNTIME: &[u8] = b"runtime";
const RUNTIME_SESSION_PROFILE_EDITOR: &[u8] = b"editor";
const RUNTIME_SESSION_PROFILE_DEV: &[u8] = b"dev";
const RUNTIME_SESSION_PROFILE_MINIMAL: &[u8] = b"minimal";
const RUNTIME_SESSION_PROFILE_HEADLESS: &[u8] = b"headless";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RuntimeDynamicSessionProfile {
    Runtime,
    Editor,
    Dev,
    Minimal,
    Headless,
}

impl RuntimeDynamicSessionProfile {
    pub(super) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            [] | RUNTIME_SESSION_PROFILE_RUNTIME => Some(Self::Runtime),
            RUNTIME_SESSION_PROFILE_EDITOR => Some(Self::Editor),
            RUNTIME_SESSION_PROFILE_DEV => Some(Self::Dev),
            RUNTIME_SESSION_PROFILE_MINIMAL => Some(Self::Minimal),
            RUNTIME_SESSION_PROFILE_HEADLESS => Some(Self::Headless),
            _ => None,
        }
    }

    pub(super) fn max_fixed_steps_per_frame(self) -> u32 {
        DEFAULT_DYNAMIC_RUNTIME_MAX_FIXED_STEPS_PER_FRAME
    }

    pub(super) fn diagnostic_log_schedule(self) -> DiagnosticStoreLogSchedule {
        match self {
            Self::Dev => DiagnosticStoreLogSchedule::repeating(DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT),
            Self::Runtime | Self::Editor | Self::Minimal | Self::Headless => {
                DiagnosticStoreLogSchedule::disabled()
            }
        }
    }

    pub(super) fn uses_render_bridge(self) -> bool {
        matches!(self, Self::Runtime | Self::Editor | Self::Dev)
    }

    pub(super) fn target_mode(self) -> RuntimeTargetMode {
        match self {
            Self::Editor => RuntimeTargetMode::EditorHost,
            Self::Runtime | Self::Dev | Self::Minimal | Self::Headless => {
                RuntimeTargetMode::ClientRuntime
            }
        }
    }
}
