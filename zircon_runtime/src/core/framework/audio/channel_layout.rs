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
pub enum AudioSpeakerChannel {
    FrontLeft,
    FrontRight,
    FrontCenter,
    LowFrequency,
    BackLeft,
    BackRight,
    SideLeft,
    SideRight,
}

fn named_layout_contract(name: &str) -> Option<(u16, &'static [AudioSpeakerChannel])> {
    use AudioSpeakerChannel::{
        BackLeft, BackRight, FrontCenter, FrontLeft, FrontRight, LowFrequency, SideLeft, SideRight,
    };

    match name {
        "mono" => Some((1, &[FrontCenter])),
        "stereo" => Some((2, &[FrontLeft, FrontRight])),
        "quad" => Some((4, &[FrontLeft, FrontRight, BackLeft, BackRight])),
        "surround_5_0" => Some((
            5,
            &[FrontLeft, FrontRight, FrontCenter, BackLeft, BackRight],
        )),
        "surround_5_1" => Some((
            6,
            &[
                FrontLeft,
                FrontRight,
                FrontCenter,
                LowFrequency,
                BackLeft,
                BackRight,
            ],
        )),
        "surround_5_1_side" => Some((
            6,
            &[
                FrontLeft,
                FrontRight,
                FrontCenter,
                LowFrequency,
                SideLeft,
                SideRight,
            ],
        )),
        "surround_7_0" => Some((
            7,
            &[
                FrontLeft,
                FrontRight,
                FrontCenter,
                BackLeft,
                BackRight,
                SideLeft,
                SideRight,
            ],
        )),
        "surround_7_1" => Some((
            8,
            &[
                FrontLeft,
                FrontRight,
                FrontCenter,
                LowFrequency,
                BackLeft,
                BackRight,
                SideLeft,
                SideRight,
            ],
        )),
        _ => None,
    }
}

fn canonical_discrete_name_matches(name: &str, channel_count: u16) -> bool {
    let Some(suffix) = name.strip_prefix("discrete_") else {
        return false;
    };
    if suffix.is_empty() || (suffix.len() > 1 && suffix.as_bytes()[0] == b'0') {
        return false;
    }

    let mut parsed = 0_u16;
    for byte in suffix.bytes() {
        if !byte.is_ascii_digit() {
            return false;
        }
        let digit = u16::from(byte - b'0');
        let Some(next) = parsed
            .checked_mul(10)
            .and_then(|value| value.checked_add(digit))
        else {
            return false;
        };
        parsed = next;
    }
    parsed == channel_count
}

fn speaker_bit(speaker: AudioSpeakerChannel) -> u8 {
    match speaker {
        AudioSpeakerChannel::FrontLeft => 1 << 0,
        AudioSpeakerChannel::FrontRight => 1 << 1,
        AudioSpeakerChannel::FrontCenter => 1 << 2,
        AudioSpeakerChannel::LowFrequency => 1 << 3,
        AudioSpeakerChannel::BackLeft => 1 << 4,
        AudioSpeakerChannel::BackRight => 1 << 5,
        AudioSpeakerChannel::SideLeft => 1 << 6,
        AudioSpeakerChannel::SideRight => 1 << 7,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioChannelLayout {
    pub name: String,
    pub channel_count: u16,
    pub speakers: Vec<AudioSpeakerChannel>,
}

impl AudioChannelLayout {
    pub fn mono() -> Self {
        Self {
            name: "mono".to_string(),
            channel_count: 1,
            speakers: vec![AudioSpeakerChannel::FrontCenter],
        }
    }

    pub fn stereo() -> Self {
        Self {
            name: "stereo".to_string(),
            channel_count: 2,
            speakers: vec![
                AudioSpeakerChannel::FrontLeft,
                AudioSpeakerChannel::FrontRight,
            ],
        }
    }

    pub fn quad() -> Self {
        Self {
            name: "quad".to_string(),
            channel_count: 4,
            speakers: vec![
                AudioSpeakerChannel::FrontLeft,
                AudioSpeakerChannel::FrontRight,
                AudioSpeakerChannel::BackLeft,
                AudioSpeakerChannel::BackRight,
            ],
        }
    }

    pub fn surround_5_1() -> Self {
        Self {
            name: "surround_5_1".to_string(),
            channel_count: 6,
            speakers: vec![
                AudioSpeakerChannel::FrontLeft,
                AudioSpeakerChannel::FrontRight,
                AudioSpeakerChannel::FrontCenter,
                AudioSpeakerChannel::LowFrequency,
                AudioSpeakerChannel::BackLeft,
                AudioSpeakerChannel::BackRight,
            ],
        }
    }

    pub fn surround_5_0() -> Self {
        Self {
            name: "surround_5_0".to_string(),
            channel_count: 5,
            speakers: vec![
                AudioSpeakerChannel::FrontLeft,
                AudioSpeakerChannel::FrontRight,
                AudioSpeakerChannel::FrontCenter,
                AudioSpeakerChannel::BackLeft,
                AudioSpeakerChannel::BackRight,
            ],
        }
    }

    pub fn surround_5_1_side() -> Self {
        Self {
            name: "surround_5_1_side".to_string(),
            channel_count: 6,
            speakers: vec![
                AudioSpeakerChannel::FrontLeft,
                AudioSpeakerChannel::FrontRight,
                AudioSpeakerChannel::FrontCenter,
                AudioSpeakerChannel::LowFrequency,
                AudioSpeakerChannel::SideLeft,
                AudioSpeakerChannel::SideRight,
            ],
        }
    }

    pub fn surround_7_0() -> Self {
        Self {
            name: "surround_7_0".to_string(),
            channel_count: 7,
            speakers: vec![
                AudioSpeakerChannel::FrontLeft,
                AudioSpeakerChannel::FrontRight,
                AudioSpeakerChannel::FrontCenter,
                AudioSpeakerChannel::BackLeft,
                AudioSpeakerChannel::BackRight,
                AudioSpeakerChannel::SideLeft,
                AudioSpeakerChannel::SideRight,
            ],
        }
    }

    pub fn surround_7_1() -> Self {
        Self {
            name: "surround_7_1".to_string(),
            channel_count: 8,
            speakers: vec![
                AudioSpeakerChannel::FrontLeft,
                AudioSpeakerChannel::FrontRight,
                AudioSpeakerChannel::FrontCenter,
                AudioSpeakerChannel::LowFrequency,
                AudioSpeakerChannel::BackLeft,
                AudioSpeakerChannel::BackRight,
                AudioSpeakerChannel::SideLeft,
                AudioSpeakerChannel::SideRight,
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
        let mut seen = 0_u8;
        self.speakers.iter().copied().all(|speaker| {
            let bit = speaker_bit(speaker);
            let unique = seen & bit == 0;
            seen |= bit;
            unique
        })
    }

    pub fn is_canonical_named_layout(&self) -> bool {
        named_layout_contract(&self.name)
            .map(|(channel_count, speakers)| {
                channel_count == self.channel_count && speakers == self.speakers.as_slice()
            })
            .unwrap_or_default()
    }

    pub fn is_canonical_discrete_layout(&self) -> bool {
        self.speakers.is_empty()
            && self.channel_count > 0
            && canonical_discrete_name_matches(&self.name, self.channel_count)
    }

    pub fn is_valid_contract_layout(&self) -> bool {
        if self.channel_count == 0 || self.name.trim() != self.name.as_str() || self.name.is_empty()
        {
            return false;
        }
        if named_layout_contract(&self.name).is_some() {
            return self.is_canonical_named_layout();
        }
        if self.name.starts_with("discrete_") {
            return self.is_canonical_discrete_layout();
        }
        if self.speakers.is_empty() {
            return self.is_canonical_discrete_layout();
        }
        self.has_matching_speaker_count() && self.has_unique_speakers()
    }
}

#[cfg(test)]
#[path = "channel_layout/allocation_free_discrete_name_tests.rs"]
mod allocation_free_discrete_name_tests;
