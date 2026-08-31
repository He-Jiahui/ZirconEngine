use std::num::{NonZeroU16, NonZeroU32};

use super::{WindowDisplayTarget, WindowPhysicalExtent};

/// Explicit policy for a requested exclusive mode that the host cannot find.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowFullscreenFallback {
    Exact,
    AllowFallback,
}

/// A strict exclusive-fullscreen video-mode request. Zero dimensions, bit
/// depth, and refresh values cannot enter this representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowVideoModeRequest {
    physical_extent: WindowPhysicalExtent,
    bit_depth: Option<NonZeroU16>,
    refresh_rate_millihertz: Option<NonZeroU32>,
}

impl WindowVideoModeRequest {
    pub const fn new(
        physical_extent: WindowPhysicalExtent,
        bit_depth: Option<NonZeroU16>,
        refresh_rate_millihertz: Option<NonZeroU32>,
    ) -> Self {
        Self {
            physical_extent,
            bit_depth,
            refresh_rate_millihertz,
        }
    }

    pub const fn physical_extent(self) -> WindowPhysicalExtent {
        self.physical_extent
    }

    pub const fn bit_depth(self) -> Option<u16> {
        match self.bit_depth {
            Some(bit_depth) => Some(bit_depth.get()),
            None => None,
        }
    }

    pub const fn refresh_rate_millihertz(self) -> Option<u32> {
        match self.refresh_rate_millihertz {
            Some(refresh_rate) => Some(refresh_rate.get()),
            None => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowExclusiveFullscreenRequest {
    output: WindowDisplayTarget,
    video_mode: WindowVideoModeRequest,
    fallback: WindowFullscreenFallback,
}

impl WindowExclusiveFullscreenRequest {
    pub fn new(
        output: WindowDisplayTarget,
        video_mode: WindowVideoModeRequest,
        fallback: WindowFullscreenFallback,
    ) -> Self {
        Self {
            output,
            video_mode,
            fallback,
        }
    }

    pub fn output(&self) -> &WindowDisplayTarget {
        &self.output
    }

    pub fn video_mode(&self) -> WindowVideoModeRequest {
        self.video_mode
    }

    pub fn fallback(&self) -> WindowFullscreenFallback {
        self.fallback
    }
}

/// Requested window mode with stable display targeting and no implicit
/// fallback behavior.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowRequestedMode {
    Windowed,
    BorderlessFullscreen { output: WindowDisplayTarget },
    ExclusiveFullscreen(WindowExclusiveFullscreenRequest),
}

impl WindowRequestedMode {
    pub fn requires_exact_video_mode(&self) -> bool {
        matches!(
            self,
            Self::ExclusiveFullscreen(WindowExclusiveFullscreenRequest {
                fallback: WindowFullscreenFallback::Exact,
                ..
            })
        )
    }

    pub fn video_mode(&self) -> Option<WindowVideoModeRequest> {
        match self {
            Self::ExclusiveFullscreen(request) => Some(request.video_mode()),
            Self::Windowed | Self::BorderlessFullscreen { .. } => None,
        }
    }
}
