use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationPlaybackSettings {
    pub enabled: bool,
    pub property_tracks: bool,
    pub skeletal_clips: bool,
    pub graphs: bool,
    pub state_machines: bool,
}

impl Default for AnimationPlaybackSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            property_tracks: true,
            skeletal_clips: true,
            graphs: true,
            state_machines: true,
        }
    }
}
