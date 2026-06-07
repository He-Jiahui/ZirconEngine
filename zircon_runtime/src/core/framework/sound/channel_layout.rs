use serde::{Deserialize, Serialize};

const NAMED_CHANNEL_LAYOUT_NAMES: &[&str] = &[
    "mono",
    "stereo",
    "quad",
    "surround_5_0",
    "surround_5_1",
    "surround_5_1_side",
    "surround_7_0",
    "surround_7_1",
];

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

    pub fn quad() -> Self {
        Self {
            name: "quad".to_string(),
            channel_count: 4,
            speakers: vec![
                SoundSpeakerChannel::FrontLeft,
                SoundSpeakerChannel::FrontRight,
                SoundSpeakerChannel::BackLeft,
                SoundSpeakerChannel::BackRight,
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

    pub fn surround_5_0() -> Self {
        Self {
            name: "surround_5_0".to_string(),
            channel_count: 5,
            speakers: vec![
                SoundSpeakerChannel::FrontLeft,
                SoundSpeakerChannel::FrontRight,
                SoundSpeakerChannel::FrontCenter,
                SoundSpeakerChannel::BackLeft,
                SoundSpeakerChannel::BackRight,
            ],
        }
    }

    pub fn surround_5_1_side() -> Self {
        Self {
            name: "surround_5_1_side".to_string(),
            channel_count: 6,
            speakers: vec![
                SoundSpeakerChannel::FrontLeft,
                SoundSpeakerChannel::FrontRight,
                SoundSpeakerChannel::FrontCenter,
                SoundSpeakerChannel::LowFrequency,
                SoundSpeakerChannel::SideLeft,
                SoundSpeakerChannel::SideRight,
            ],
        }
    }

    pub fn surround_7_0() -> Self {
        Self {
            name: "surround_7_0".to_string(),
            channel_count: 7,
            speakers: vec![
                SoundSpeakerChannel::FrontLeft,
                SoundSpeakerChannel::FrontRight,
                SoundSpeakerChannel::FrontCenter,
                SoundSpeakerChannel::BackLeft,
                SoundSpeakerChannel::BackRight,
                SoundSpeakerChannel::SideLeft,
                SoundSpeakerChannel::SideRight,
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
            4 => Self::quad(),
            5 => Self::surround_5_0(),
            6 => Self::surround_5_1(),
            7 => Self::surround_7_0(),
            8 => Self::surround_7_1(),
            _ => Self::discrete(channel_count),
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "mono" => Some(Self::mono()),
            "stereo" => Some(Self::stereo()),
            "quad" => Some(Self::quad()),
            "surround_5_0" => Some(Self::surround_5_0()),
            "surround_5_1" => Some(Self::surround_5_1()),
            "surround_5_1_side" => Some(Self::surround_5_1_side()),
            "surround_7_0" => Some(Self::surround_7_0()),
            "surround_7_1" => Some(Self::surround_7_1()),
            _ => None,
        }
    }

    pub fn named_layout_names() -> &'static [&'static str] {
        NAMED_CHANNEL_LAYOUT_NAMES
    }

    pub fn matches_channel_count(&self, channel_count: u16) -> bool {
        self.channel_count == channel_count
    }

    pub fn has_matching_speaker_count(&self) -> bool {
        self.speakers.is_empty() || self.speakers.len() == usize::from(self.channel_count)
    }

    pub fn has_unique_speakers(&self) -> bool {
        self.speakers
            .iter()
            .enumerate()
            .all(|(index, speaker)| !self.speakers[index + 1..].contains(speaker))
    }

    pub fn is_canonical_named_layout(&self) -> bool {
        Self::from_name(&self.name)
            .map(|layout| {
                layout.channel_count == self.channel_count && layout.speakers == self.speakers
            })
            .unwrap_or_default()
    }

    pub fn is_canonical_discrete_layout(&self) -> bool {
        self.speakers.is_empty()
            && self.channel_count > 0
            && self.name == format!("discrete_{}", self.channel_count)
    }

    pub fn is_valid_contract_layout(&self) -> bool {
        if self.channel_count == 0 || self.name.trim() != self.name.as_str() || self.name.is_empty()
        {
            return false;
        }
        if Self::from_name(&self.name).is_some() {
            return self.is_canonical_named_layout();
        }
        if self
            .name
            .strip_prefix("discrete_")
            .and_then(|suffix| suffix.parse::<u16>().ok())
            .is_some()
        {
            return self.is_canonical_discrete_layout();
        }
        if self.speakers.is_empty() {
            return self.is_canonical_discrete_layout();
        }
        self.has_matching_speaker_count() && self.has_unique_speakers()
    }
}
