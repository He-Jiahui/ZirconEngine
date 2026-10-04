use crate::ui::dispatch::{
    UiImePreeditClause, UiImePreeditClauseError, UiImePreeditClauseKind, UiTextByteRange,
};

pub const ZR_RUNTIME_IME_COMPOSITION_V2_MAGIC: [u8; 4] = *b"ZIME";
pub const ZR_RUNTIME_IME_COMPOSITION_V2_SCHEMA: u16 = 2;
pub const ZR_RUNTIME_IME_COMPOSITION_V2_HEADER_BYTES: usize = 52;
pub const ZR_RUNTIME_IME_COMPOSITION_V2_CLAUSE_BYTES: usize = 12;
pub const ZR_RUNTIME_IME_COMPOSITION_V2_MAX_CLAUSES: usize = 4_096;

/// Empty clauses can mean either "the platform had no attributes" or "the platform reported
/// an empty composition". Keep that distinction in the negotiated envelope instead of guessing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ZrRuntimeImePreeditClauseAvailabilityV2 {
    Unavailable = 0,
    Available = 1,
}

impl TryFrom<u32> for ZrRuntimeImePreeditClauseAvailabilityV2 {
    type Error = ZrRuntimeImeCompositionV2Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Unavailable),
            1 => Ok(Self::Available),
            _ => Err(ZrRuntimeImeCompositionV2Error::UnknownClauseAvailability),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum ZrRuntimeImeCompositionOperationV2 {
    Preedit = 1,
    Commit = 2,
    Cancel = 3,
}

impl TryFrom<u16> for ZrRuntimeImeCompositionOperationV2 {
    type Error = ZrRuntimeImeCompositionV2Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Preedit),
            2 => Ok(Self::Commit),
            3 => Ok(Self::Cancel),
            _ => Err(ZrRuntimeImeCompositionV2Error::UnknownOperation),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ZrRuntimeImeCompositionContextV2 {
    pub window_generation: u64,
    pub focus_generation: u64,
    pub composition_generation: u64,
}

impl ZrRuntimeImeCompositionContextV2 {
    fn is_valid(self) -> bool {
        self.window_generation != 0
            && self.focus_generation != 0
            && self.composition_generation != 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZrRuntimeImeCompositionV2 {
    pub operation: ZrRuntimeImeCompositionOperationV2,
    pub context: ZrRuntimeImeCompositionContextV2,
    pub text: String,
    pub cursor_range: Option<UiTextByteRange>,
    pub clauses: Vec<UiImePreeditClause>,
    pub clause_availability: ZrRuntimeImePreeditClauseAvailabilityV2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrRuntimeImeCompositionV2Error {
    PayloadTooLarge,
    HeaderTruncated,
    BadMagic,
    UnsupportedSchema,
    UnknownOperation,
    InvalidContext,
    CountLimitExceeded,
    LengthOverflow,
    LengthMismatch,
    InvalidUtf8,
    UnknownClauseKind,
    UnknownClauseAvailability,
    NonZeroReservedClauseField,
    InvalidOperationPayload,
    InvalidPreeditRange(UiImePreeditClauseError),
}

impl ZrRuntimeImeCompositionV2 {
    pub fn encode(&self) -> Result<Vec<u8>, ZrRuntimeImeCompositionV2Error> {
        self.validate()?;
        let text_len = u32::try_from(self.text.len())
            .map_err(|_| ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
        let clause_count = u32::try_from(self.clauses.len())
            .map_err(|_| ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
        let clause_bytes = self
            .clauses
            .len()
            .checked_mul(ZR_RUNTIME_IME_COMPOSITION_V2_CLAUSE_BYTES)
            .ok_or(ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
        let total_bytes = ZR_RUNTIME_IME_COMPOSITION_V2_HEADER_BYTES
            .checked_add(clause_bytes)
            .and_then(|bytes| bytes.checked_add(self.text.len()))
            .ok_or(ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
        if total_bytes > crate::ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1 {
            return Err(ZrRuntimeImeCompositionV2Error::PayloadTooLarge);
        }

        let mut output = Vec::with_capacity(total_bytes);
        output.extend_from_slice(&ZR_RUNTIME_IME_COMPOSITION_V2_MAGIC);
        output.extend_from_slice(&ZR_RUNTIME_IME_COMPOSITION_V2_SCHEMA.to_le_bytes());
        output.extend_from_slice(&(self.operation as u16).to_le_bytes());
        output.extend_from_slice(&self.context.window_generation.to_le_bytes());
        output.extend_from_slice(&self.context.focus_generation.to_le_bytes());
        output.extend_from_slice(&self.context.composition_generation.to_le_bytes());
        output.extend_from_slice(&text_len.to_le_bytes());
        let (cursor_start, cursor_end) = self
            .cursor_range
            .map(|range| (range.start_byte, range.end_byte))
            .unwrap_or((u32::MAX, u32::MAX));
        output.extend_from_slice(&cursor_start.to_le_bytes());
        output.extend_from_slice(&cursor_end.to_le_bytes());
        output.extend_from_slice(&clause_count.to_le_bytes());
        output.extend_from_slice(&(self.clause_availability as u32).to_le_bytes());
        for clause in &self.clauses {
            output.extend_from_slice(&clause.range.start_byte.to_le_bytes());
            output.extend_from_slice(&clause.range.end_byte.to_le_bytes());
            output.extend_from_slice(&clause_kind_to_wire(clause.kind).to_le_bytes());
            output.extend_from_slice(&0_u16.to_le_bytes());
        }
        output.extend_from_slice(self.text.as_bytes());
        debug_assert_eq!(output.len(), total_bytes);
        Ok(output)
    }

    pub fn decode(payload: &[u8]) -> Result<Self, ZrRuntimeImeCompositionV2Error> {
        if payload.len() > crate::ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1 {
            return Err(ZrRuntimeImeCompositionV2Error::PayloadTooLarge);
        }
        if payload.len() < ZR_RUNTIME_IME_COMPOSITION_V2_HEADER_BYTES {
            return Err(ZrRuntimeImeCompositionV2Error::HeaderTruncated);
        }
        if payload.get(..4) != Some(&ZR_RUNTIME_IME_COMPOSITION_V2_MAGIC[..]) {
            return Err(ZrRuntimeImeCompositionV2Error::BadMagic);
        }
        if read_u16(payload, 4)? != ZR_RUNTIME_IME_COMPOSITION_V2_SCHEMA {
            return Err(ZrRuntimeImeCompositionV2Error::UnsupportedSchema);
        }
        let operation = ZrRuntimeImeCompositionOperationV2::try_from(read_u16(payload, 6)?)?;
        let context = ZrRuntimeImeCompositionContextV2 {
            window_generation: read_u64(payload, 8)?,
            focus_generation: read_u64(payload, 16)?,
            composition_generation: read_u64(payload, 24)?,
        };
        if !context.is_valid() {
            return Err(ZrRuntimeImeCompositionV2Error::InvalidContext);
        }
        let text_len = read_u32(payload, 32)? as usize;
        let cursor_start = read_u32(payload, 36)?;
        let cursor_end = read_u32(payload, 40)?;
        let clause_count = read_u32(payload, 44)? as usize;
        let clause_availability =
            ZrRuntimeImePreeditClauseAvailabilityV2::try_from(read_u32(payload, 48)?)?;
        if clause_count > ZR_RUNTIME_IME_COMPOSITION_V2_MAX_CLAUSES {
            return Err(ZrRuntimeImeCompositionV2Error::CountLimitExceeded);
        }
        let clause_bytes = clause_count
            .checked_mul(ZR_RUNTIME_IME_COMPOSITION_V2_CLAUSE_BYTES)
            .ok_or(ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
        let text_start = ZR_RUNTIME_IME_COMPOSITION_V2_HEADER_BYTES
            .checked_add(clause_bytes)
            .ok_or(ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
        let expected_len = text_start
            .checked_add(text_len)
            .ok_or(ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
        if expected_len != payload.len() {
            return Err(ZrRuntimeImeCompositionV2Error::LengthMismatch);
        }

        let cursor_range = match (cursor_start, cursor_end) {
            (u32::MAX, u32::MAX) => None,
            (u32::MAX, _) | (_, u32::MAX) => {
                return Err(ZrRuntimeImeCompositionV2Error::InvalidOperationPayload);
            }
            (start_byte, end_byte) => Some(UiTextByteRange::new(start_byte, end_byte)),
        };
        let mut clauses = Vec::with_capacity(clause_count);
        for index in 0..clause_count {
            let offset = ZR_RUNTIME_IME_COMPOSITION_V2_HEADER_BYTES
                + index * ZR_RUNTIME_IME_COMPOSITION_V2_CLAUSE_BYTES;
            let kind = wire_to_clause_kind(read_u16(payload, offset + 8)?)?;
            if read_u16(payload, offset + 10)? != 0 {
                return Err(ZrRuntimeImeCompositionV2Error::NonZeroReservedClauseField);
            }
            clauses.push(UiImePreeditClause::new(
                UiTextByteRange::new(read_u32(payload, offset)?, read_u32(payload, offset + 4)?),
                kind,
            ));
        }
        let text = String::from_utf8(payload[text_start..].to_vec())
            .map_err(|_| ZrRuntimeImeCompositionV2Error::InvalidUtf8)?;
        let decoded = Self {
            operation,
            context,
            text,
            cursor_range,
            clauses,
            clause_availability,
        };
        decoded.validate()?;
        Ok(decoded)
    }

    fn validate(&self) -> Result<(), ZrRuntimeImeCompositionV2Error> {
        if !self.context.is_valid() {
            return Err(ZrRuntimeImeCompositionV2Error::InvalidContext);
        }
        if self.clauses.len() > ZR_RUNTIME_IME_COMPOSITION_V2_MAX_CLAUSES {
            return Err(ZrRuntimeImeCompositionV2Error::CountLimitExceeded);
        }
        match self.operation {
            ZrRuntimeImeCompositionOperationV2::Preedit => {
                if self.clause_availability == ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable
                    && !self.clauses.is_empty()
                {
                    return Err(ZrRuntimeImeCompositionV2Error::InvalidOperationPayload);
                }
                UiImePreeditClause::validate_preedit_payload(
                    &self.text,
                    self.cursor_range,
                    &self.clauses,
                )
                .map_err(ZrRuntimeImeCompositionV2Error::InvalidPreeditRange)?;
            }
            ZrRuntimeImeCompositionOperationV2::Commit => {
                if self.cursor_range.is_some()
                    || !self.clauses.is_empty()
                    || self.clause_availability
                        != ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable
                {
                    return Err(ZrRuntimeImeCompositionV2Error::InvalidOperationPayload);
                }
            }
            ZrRuntimeImeCompositionOperationV2::Cancel => {
                if !self.text.is_empty()
                    || self.cursor_range.is_some()
                    || !self.clauses.is_empty()
                    || self.clause_availability
                        != ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable
                {
                    return Err(ZrRuntimeImeCompositionV2Error::InvalidOperationPayload);
                }
            }
        }
        Ok(())
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, ZrRuntimeImeCompositionV2Error> {
    let end = offset
        .checked_add(2)
        .ok_or(ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
    let field = bytes
        .get(offset..end)
        .ok_or(ZrRuntimeImeCompositionV2Error::HeaderTruncated)?;
    Ok(u16::from_le_bytes([field[0], field[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, ZrRuntimeImeCompositionV2Error> {
    let end = offset
        .checked_add(4)
        .ok_or(ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
    let field = bytes
        .get(offset..end)
        .ok_or(ZrRuntimeImeCompositionV2Error::HeaderTruncated)?;
    Ok(u32::from_le_bytes([field[0], field[1], field[2], field[3]]))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, ZrRuntimeImeCompositionV2Error> {
    let end = offset
        .checked_add(8)
        .ok_or(ZrRuntimeImeCompositionV2Error::LengthOverflow)?;
    let field = bytes
        .get(offset..end)
        .ok_or(ZrRuntimeImeCompositionV2Error::HeaderTruncated)?;
    Ok(u64::from_le_bytes([
        field[0], field[1], field[2], field[3], field[4], field[5], field[6], field[7],
    ]))
}

fn clause_kind_to_wire(kind: UiImePreeditClauseKind) -> u16 {
    match kind {
        UiImePreeditClauseKind::Input => 1,
        UiImePreeditClauseKind::Converted => 2,
        UiImePreeditClauseKind::TargetConverted => 3,
        UiImePreeditClauseKind::TargetNotConverted => 4,
    }
}

fn wire_to_clause_kind(
    value: u16,
) -> Result<UiImePreeditClauseKind, ZrRuntimeImeCompositionV2Error> {
    match value {
        1 => Ok(UiImePreeditClauseKind::Input),
        2 => Ok(UiImePreeditClauseKind::Converted),
        3 => Ok(UiImePreeditClauseKind::TargetConverted),
        4 => Ok(UiImePreeditClauseKind::TargetNotConverted),
        _ => Err(ZrRuntimeImeCompositionV2Error::UnknownClauseKind),
    }
}

#[cfg(test)]
#[path = "tests/ime_composition_v2.rs"]
mod tests;
