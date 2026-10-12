#[path = "../../../../window/effective/exclusive_fullscreen_fallback.rs"]
mod exclusive_fullscreen_fallback;
#[path = "../../../../window/effective/exclusive_fullscreen_fallback_reason.rs"]
mod exclusive_fullscreen_fallback_reason;
#[path = "mode.rs"]
mod mode;
#[path = "placement.rs"]
mod placement;
#[path = "state.rs"]
mod state;
#[path = "error.rs"]
mod window_effective_state_error;

pub use exclusive_fullscreen_fallback::WindowExclusiveFullscreenFallback;
pub use exclusive_fullscreen_fallback_reason::WindowExclusiveFullscreenFallbackReason;
pub use mode::WindowEffectiveMode;
pub use placement::WindowEffectivePlacement;
pub use state::WindowEffectiveState;
pub use window_effective_state_error::WindowEffectiveStateError;
