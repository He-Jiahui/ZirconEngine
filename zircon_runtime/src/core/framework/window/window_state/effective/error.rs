use std::error::Error;
use std::fmt;

use crate::core::framework::window::DisplayId;

/// A host-effective state cannot name two outputs for one native window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowEffectiveStateError {
    FullscreenOutputMismatch {
        placement_display: DisplayId,
        mode_output: DisplayId,
    },
}

impl fmt::Display for WindowEffectiveStateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FullscreenOutputMismatch {
                placement_display,
                mode_output,
            } => write!(
                formatter,
                "fullscreen output {} does not match effective placement display {}",
                mode_output.as_str(),
                placement_display.as_str()
            ),
        }
    }
}

impl Error for WindowEffectiveStateError {}
