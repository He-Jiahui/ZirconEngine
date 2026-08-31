use crate::core::framework::window::DisplayId;

use super::super::WindowVideoModeRequest;
use super::WindowExclusiveFullscreenFallback;

/// The mode actually accepted by the platform host. It carries the resolved
/// stable display instead of a transient monitor index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowEffectiveMode {
    Windowed,
    BorderlessFullscreen {
        output: DisplayId,
        exclusive_fallback: Option<WindowExclusiveFullscreenFallback>,
    },
    ExclusiveFullscreen {
        output: DisplayId,
        video_mode: WindowVideoModeRequest,
    },
}

impl WindowEffectiveMode {
    pub const fn is_fullscreen(&self) -> bool {
        !matches!(self, Self::Windowed)
    }

    pub fn output(&self) -> Option<&DisplayId> {
        match self {
            Self::Windowed => None,
            Self::BorderlessFullscreen { output, .. }
            | Self::ExclusiveFullscreen { output, .. } => Some(output),
        }
    }

    pub const fn exclusive_fallback(&self) -> Option<WindowExclusiveFullscreenFallback> {
        match self {
            Self::BorderlessFullscreen {
                exclusive_fallback, ..
            } => *exclusive_fallback,
            Self::Windowed | Self::ExclusiveFullscreen { .. } => None,
        }
    }
}
