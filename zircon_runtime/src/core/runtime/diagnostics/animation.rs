use crate::core::framework::animation::AnimationPlaybackSettings;

use super::FrameDiagnostics;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeAnimationDiagnostics {
    pub available: bool,
    pub playback_settings: Option<AnimationPlaybackSettings>,
    pub error: Option<String>,
}

impl RuntimeAnimationDiagnostics {
    pub fn unavailable(error: impl Into<String>) -> Self {
        Self {
            available: false,
            playback_settings: None,
            error: Some(error.into()),
        }
    }
}

impl FrameDiagnostics for RuntimeAnimationDiagnostics {
    fn diagnostics_domain(&self) -> &'static str {
        "animation"
    }

    fn diagnostics_available(&self) -> bool {
        self.available
    }

    fn diagnostics_error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}
