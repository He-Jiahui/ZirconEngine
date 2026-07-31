use crate::{validate_command_payload, ProtocolError, SET_DUNGEON_DIFFICULTY_COMMAND_ID};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DungeonDifficulty {
    Normal,
    Heroic,
}

impl DungeonDifficulty {
    const fn code(self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::Heroic => 1,
        }
    }

    fn decode(code: u8) -> Result<Self, ProtocolError> {
        match code {
            0 => Ok(Self::Normal),
            1 => Ok(Self::Heroic),
            other => Err(ProtocolError::InvalidDungeonDifficulty(other)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DungeonDifficultyPayload {
    pub difficulty: DungeonDifficulty,
}

impl DungeonDifficultyPayload {
    pub fn encode(self) -> Result<[u8; 1], ProtocolError> {
        let bytes = [self.difficulty.code()];
        validate_command_payload(SET_DUNGEON_DIFFICULTY_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(SET_DUNGEON_DIFFICULTY_COMMAND_ID, bytes)?;
        Ok(Self {
            difficulty: DungeonDifficulty::decode(bytes[0])?,
        })
    }
}

pub(crate) fn validate_dungeon_difficulty_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = DungeonDifficulty::decode(bytes[0])?;
    Ok(())
}
