use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundSpeakerChannel {
    FrontLeft,
    FrontRight,
    FrontCenter,
    LowFrequency,
    BackLeft,
    BackRight,
    SideLeft,
    SideRight,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoundChannelLayout {
    pub name: String,
    pub channel_count: u16,
    pub speakers: Vec<SoundSpeakerChannel>,
}

impl SoundChannelLayout {
    pub fn mono() -> Self {
        Self {
            name: "mono".to_string(),
            channel_count: 1,
            speakers: vec![SoundSpeakerChannel::FrontCenter],
        }
    }

    pub fn stereo() -> Self {
        Self {
            name: "stereo".to_string(),
            channel_count: 2,
            speakers: vec![
                SoundSpeakerChannel::FrontLeft,
                SoundSpeakerChannel::FrontRight,
            ],
        }
    }

    pub fn surround_5_1() -> Self {
        Self {
            name: "surround_5_1".to_string(),
            channel_count: 6,
            speakers: vec![
                SoundSpeakerChannel::FrontLeft,
                SoundSpeakerChannel::FrontRight,
                SoundSpeakerChannel::FrontCenter,
                SoundSpeakerChannel::LowFrequency,
                SoundSpeakerChannel::BackLeft,
                SoundSpeakerChannel::BackRight,
            ],
        }
    }

    pub fn surround_7_1() -> Self {
        Self {
            name: "surround_7_1".to_string(),
            channel_count: 8,
            speakers: vec![
                SoundSpeakerChannel::FrontLeft,
                SoundSpeakerChannel::FrontRight,
                SoundSpeakerChannel::FrontCenter,
                SoundSpeakerChannel::LowFrequency,
                SoundSpeakerChannel::BackLeft,
                SoundSpeakerChannel::BackRight,
                SoundSpeakerChannel::SideLeft,
                SoundSpeakerChannel::SideRight,
            ],
        }
    }

    pub fn discrete(channel_count: u16) -> Self {
        Self {
            name: format!("discrete_{channel_count}"),
            channel_count,
            speakers: Vec::new(),
        }
    }

    pub fn for_channel_count(channel_count: u16) -> Self {
        match channel_count {
            1 => Self::mono(),
            2 => Self::stereo(),
            6 => Self::surround_5_1(),
            8 => Self::surround_7_1(),
            _ => Self::discrete(channel_count),
        }
    }

    pub fn matches_channel_count(&self, channel_count: u16) -> bool {
        self.channel_count == channel_count
    }
}
