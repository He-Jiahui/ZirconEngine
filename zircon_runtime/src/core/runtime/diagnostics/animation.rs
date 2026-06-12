use crate::core::framework::animation::AnimationPlaybackSettings;

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
