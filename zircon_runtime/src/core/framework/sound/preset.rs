use serde::{Deserialize, Serialize};

use super::SoundMixerGraph;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundMixerPresetDescriptor {
    pub locator: String,
    pub display_name: String,
    pub graph: SoundMixerGraph,
}

impl SoundMixerPresetDescriptor {
    pub fn new(
        locator: impl Into<String>,
        display_name: impl Into<String>,
        graph: SoundMixerGraph,
    ) -> Self {
        Self {
            locator: locator.into(),
            display_name: display_name.into(),
            graph,
        }
    }
}
