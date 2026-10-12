use super::super::WindowVideoModeRequest;
use super::WindowExclusiveFullscreenFallbackReason;

/// The requested exclusive video mode retained in an effective fallback
/// report, so diagnostics never hide what the host could not realize.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowExclusiveFullscreenFallback {
    requested_video_mode: WindowVideoModeRequest,
    reason: WindowExclusiveFullscreenFallbackReason,
}

impl WindowExclusiveFullscreenFallback {
    pub const fn new(
        requested_video_mode: WindowVideoModeRequest,
        reason: WindowExclusiveFullscreenFallbackReason,
    ) -> Self {
        Self {
            requested_video_mode,
            reason,
        }
    }

    pub const fn requested_video_mode(self) -> WindowVideoModeRequest {
        self.requested_video_mode
    }

    pub const fn reason(self) -> WindowExclusiveFullscreenFallbackReason {
        self.reason
    }
}
