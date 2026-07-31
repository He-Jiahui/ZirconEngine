use crate::{
    require_finite, validate_command_payload, ProtocolError, VALE_CUP_BET_COMMAND_ID,
    VALE_CUP_PRACTICE_COMMAND_ID, VALE_CUP_QUEUE_COMMAND_ID, VALE_CUP_ROLE_COMMAND_ID,
};

const VALE_CUP_QUEUE_BYTES: usize = 4;
const VALE_CUP_ROLE_BYTES: usize = 1;
const VALE_CUP_BET_BYTES: usize = 9;
const VALE_CUP_PRACTICE_BYTES: usize = 1;
const NUMBER_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValeCupBracket {
    One,
    Two,
    Three,
    Four,
    Five,
}

impl ValeCupBracket {
    const fn wire_code(self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
        }
    }

    fn from_wire_code(code: u8) -> Result<Self, ProtocolError> {
        match code {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            invalid => Err(ProtocolError::InvalidValeCupBracket(invalid)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValeCupNation {
    Vale,
    Mirefen,
    Thornpeak,
    Coliseum,
    Choir,
    Ogre,
    Moon,
    Copperdig,
}

impl ValeCupNation {
    const fn wire_code(self) -> u8 {
        match self {
            Self::Vale => 0,
            Self::Mirefen => 1,
            Self::Thornpeak => 2,
            Self::Coliseum => 3,
            Self::Choir => 4,
            Self::Ogre => 5,
            Self::Moon => 6,
            Self::Copperdig => 7,
        }
    }

    fn from_wire_code(code: u8) -> Result<Self, ProtocolError> {
        match code {
            0 => Ok(Self::Vale),
            1 => Ok(Self::Mirefen),
            2 => Ok(Self::Thornpeak),
            3 => Ok(Self::Coliseum),
            4 => Ok(Self::Choir),
            5 => Ok(Self::Ogre),
            6 => Ok(Self::Moon),
            7 => Ok(Self::Copperdig),
            invalid => Err(ProtocolError::InvalidValeCupNation(invalid)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValeCupRole {
    Allrounder,
    Striker,
    Sweeper,
    Keeper,
}

impl ValeCupRole {
    const fn wire_code(self) -> u8 {
        match self {
            Self::Allrounder => 0,
            Self::Striker => 1,
            Self::Sweeper => 2,
            Self::Keeper => 3,
        }
    }

    fn from_wire_code(code: u8) -> Result<Self, ProtocolError> {
        match code {
            0 => Ok(Self::Allrounder),
            1 => Ok(Self::Striker),
            2 => Ok(Self::Sweeper),
            3 => Ok(Self::Keeper),
            invalid => Err(ProtocolError::InvalidValeCupRole(invalid)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValeCupSide {
    A,
    B,
}

impl ValeCupSide {
    const fn wire_code(self) -> u8 {
        match self {
            Self::A => 0,
            Self::B => 1,
        }
    }

    fn from_wire_code(code: u8) -> Result<Self, ProtocolError> {
        match code {
            0 => Ok(Self::A),
            1 => Ok(Self::B),
            invalid => Err(ProtocolError::InvalidValeCupSide(invalid)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValeCupQueueCommandPayload {
    pub bracket: ValeCupBracket,
    pub nation: ValeCupNation,
    pub role: ValeCupRole,
    pub enter_as_guild: bool,
}

impl ValeCupQueueCommandPayload {
    pub fn encode(self) -> Result<[u8; VALE_CUP_QUEUE_BYTES], ProtocolError> {
        let bytes = [
            self.bracket.wire_code(),
            self.nation.wire_code(),
            self.role.wire_code(),
            u8::from(self.enter_as_guild),
        ];
        validate_command_payload(VALE_CUP_QUEUE_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(VALE_CUP_QUEUE_COMMAND_ID, bytes)?;
        Ok(Self {
            bracket: ValeCupBracket::from_wire_code(bytes[0])?,
            nation: ValeCupNation::from_wire_code(bytes[1])?,
            role: ValeCupRole::from_wire_code(bytes[2])?,
            enter_as_guild: decode_boolean(bytes[3])?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValeCupRoleCommandPayload {
    pub role: ValeCupRole,
}

impl ValeCupRoleCommandPayload {
    pub fn encode(self) -> Result<[u8; VALE_CUP_ROLE_BYTES], ProtocolError> {
        let bytes = [self.role.wire_code()];
        validate_command_payload(VALE_CUP_ROLE_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(VALE_CUP_ROLE_COMMAND_ID, bytes)?;
        Ok(Self {
            role: ValeCupRole::from_wire_code(bytes[0])?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValeCupBetCommandPayload {
    pub side: ValeCupSide,
    pub amount: f64,
}

impl ValeCupBetCommandPayload {
    pub fn encode(self) -> Result<[u8; VALE_CUP_BET_BYTES], ProtocolError> {
        let amount = canonical_finite_f64("ValeCupBetCommandPayload.amount", self.amount)?;
        let mut bytes = [0; VALE_CUP_BET_BYTES];
        bytes[0] = self.side.wire_code();
        bytes[1..].copy_from_slice(&amount.to_le_bytes());
        validate_command_payload(VALE_CUP_BET_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(VALE_CUP_BET_COMMAND_ID, bytes)?;
        Ok(Self {
            side: ValeCupSide::from_wire_code(bytes[0])?,
            amount: read_finite_f64(bytes, 1, "ValeCupBetCommandPayload.amount")?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValeCupPracticeCommandPayload {
    pub bracket: ValeCupBracket,
}

impl ValeCupPracticeCommandPayload {
    pub fn encode(self) -> Result<[u8; VALE_CUP_PRACTICE_BYTES], ProtocolError> {
        let bytes = [self.bracket.wire_code()];
        validate_command_payload(VALE_CUP_PRACTICE_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(VALE_CUP_PRACTICE_COMMAND_ID, bytes)?;
        Ok(Self {
            bracket: ValeCupBracket::from_wire_code(bytes[0])?,
        })
    }
}

pub(crate) fn validate_vale_cup_queue_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = ValeCupBracket::from_wire_code(bytes[0])?;
    let _ = ValeCupNation::from_wire_code(bytes[1])?;
    let _ = ValeCupRole::from_wire_code(bytes[2])?;
    let _ = decode_boolean(bytes[3])?;
    Ok(())
}

pub(crate) fn validate_vale_cup_role_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = ValeCupRole::from_wire_code(bytes[0])?;
    Ok(())
}

pub(crate) fn validate_vale_cup_bet_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = ValeCupSide::from_wire_code(bytes[0])?;
    let _ = read_finite_f64(bytes, 1, "ValeCupBetCommandPayload.amount")?;
    Ok(())
}

pub(crate) fn validate_vale_cup_practice_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let _ = ValeCupBracket::from_wire_code(bytes[0])?;
    Ok(())
}

fn canonical_finite_f64(context: &'static str, value: f64) -> Result<f64, ProtocolError> {
    let value = require_finite(context, value)?;
    Ok(if value == 0.0 { 0.0 } else { value })
}

fn read_finite_f64(
    bytes: &[u8],
    offset: usize,
    context: &'static str,
) -> Result<f64, ProtocolError> {
    let value = f64::from_le_bytes(
        bytes[offset..offset + NUMBER_BYTES]
            .try_into()
            .expect("validated Vale Cup payload contains a complete f64"),
    );
    canonical_finite_f64(context, value)
}

fn decode_boolean(value: u8) -> Result<bool, ProtocolError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        invalid => Err(ProtocolError::InvalidBoolean(invalid)),
    }
}
