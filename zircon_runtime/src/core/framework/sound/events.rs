use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundDynamicEventInvocation {
    pub event_id: String,
    pub source_path: Option<String>,
    pub time_seconds: f32,
    pub payload_schema: String,
    pub payload: Vec<u8>,
}
