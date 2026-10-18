use crate::{validate_command_payload, ProtocolError, EMOTE_COMMAND_ID};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmoteId {
    Wave,
    Laugh,
    Question,
    Cheer,
    Dance,
    Point,
    Flex,
    Salute,
    Cry,
    Bow,
    Clap,
    Roar,
    Kneel,
}

impl EmoteId {
    pub const fn wire_code(self) -> u8 {
        match self {
            Self::Wave => 1,
            Self::Laugh => 2,
            Self::Question => 3,
            Self::Cheer => 4,
            Self::Dance => 5,
            Self::Point => 6,
            Self::Flex => 7,
            Self::Salute => 8,
            Self::Cry => 9,
            Self::Bow => 10,
            Self::Clap => 11,
            Self::Roar => 12,
            Self::Kneel => 13,
        }
    }

    pub const fn source_id(self) -> &'static str {
        match self {
            Self::Wave => "wave",
            Self::Laugh => "laugh",
            Self::Question => "question",
            Self::Cheer => "cheer",
            Self::Dance => "dance",
            Self::Point => "point",
            Self::Flex => "flex",
            Self::Salute => "salute",
            Self::Cry => "cry",
            Self::Bow => "bow",
            Self::Clap => "clap",
            Self::Roar => "roar",
            Self::Kneel => "kneel",
        }
    }

    pub fn from_wire_code(code: u8) -> Result<Self, ProtocolError> {
        match code {
            1 => Ok(Self::Wave),
            2 => Ok(Self::Laugh),
            3 => Ok(Self::Question),
            4 => Ok(Self::Cheer),
            5 => Ok(Self::Dance),
            6 => Ok(Self::Point),
            7 => Ok(Self::Flex),
            8 => Ok(Self::Salute),
            9 => Ok(Self::Cry),
            10 => Ok(Self::Bow),
            11 => Ok(Self::Clap),
            12 => Ok(Self::Roar),
            13 => Ok(Self::Kneel),
            value => Err(ProtocolError::InvalidEmoteId(value)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmoteCommandPayload {
    pub emote: EmoteId,
}

impl EmoteCommandPayload {
    pub fn encode(self) -> [u8; 1] {
        [self.emote.wire_code()]
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(EMOTE_COMMAND_ID, bytes)?;
        Ok(Self {
            emote: EmoteId::from_wire_code(bytes[0])?,
        })
    }
}

#[cfg(test)]
#[path = "tests/emote_payload.rs"]
mod tests;
