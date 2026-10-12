/// Why an explicitly permissive exclusive-fullscreen request became
/// borderless. Exact requests never construct this value: they fail instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowExclusiveFullscreenFallbackReason {
    VideoModeUnavailable,
    ExclusiveFullscreenUnavailable,
}
