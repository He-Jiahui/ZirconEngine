use serde::{Deserialize, Serialize};

use super::{
    SoundAutomationBindingId, SoundEffectId, SoundListenerId, SoundParameterId, SoundSourceId,
    SoundTrackId, SoundVolumeId,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoundAutomationBinding {
    pub id: SoundAutomationBindingId,
    pub timeline_track_path: String,
    pub target: SoundAutomationTarget,
    pub parameter: SoundParameterId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundAutomationTarget {
    Track(SoundTrackId),
    Effect {
        track: SoundTrackId,
        effect: SoundEffectId,
    },
    Source(SoundSourceId),
    Listener(SoundListenerId),
    Volume(SoundVolumeId),
    SynthParameter(SoundParameterId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoundDynamicEventCatalog {
    pub namespace: String,
    pub version: u32,
    pub events: Vec<SoundDynamicEventDescriptor>,
}

impl SoundDynamicEventCatalog {
    pub fn empty() -> Self {
        Self {
            namespace: "sound.dynamic_events".to_string(),
            version: 1,
            events: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoundDynamicEventDescriptor {
    pub id: String,
    pub display_name: String,
    pub payload_schema: String,
}
