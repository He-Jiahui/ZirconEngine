use crate::{validate_command_payload, ProtocolError, DELVE_RITE_CHOOSE_COMMAND_ID};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DelveRiteIntensity {
    Easy,
    Medium,
    Hard,
}

impl DelveRiteIntensity {
    const fn code(self) -> u8 {
        match self {
            Self::Easy => 0,
            Self::Medium => 1,
            Self::Hard => 2,
        }
    }

    fn decode(code: u8) -> Result<Self, ProtocolError> {
        match code {
            0 => Ok(Self::Easy),
            1 => Ok(Self::Medium),
            2 => Ok(Self::Hard),
            other => Err(ProtocolError::InvalidDelveRiteIntensity(other)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DelveRiteChoosePayload {
    pub intensity: DelveRiteIntensity,
}

impl DelveRiteChoosePayload {
    pub fn encode(self) -> Result<[u8; 1], ProtocolError> {
        let bytes = [self.intensity.code()];
        validate_command_payload(DELVE_RITE_CHOOSE_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(DELVE_RITE_CHOOSE_COMMAND_ID, bytes)?;
        Ok(Self {
            intensity: DelveRiteIntensity::decode(bytes[0])?,
        })
    }
}

pub(crate) fn validate_delve_rite_intensity_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = DelveRiteIntensity::decode(bytes[0])?;
    Ok(())
}
