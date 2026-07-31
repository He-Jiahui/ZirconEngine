use crate::{
    require_finite, validate_command_payload, ProtocolError,
    DUNGEON_FINDER_APPLICATION_RESPONSE_COMMAND_ID, DUNGEON_FINDER_APPLY_COMMAND_ID,
    DUNGEON_FINDER_LIST_CREATE_COMMAND_ID, DUNGEON_FINDER_QUEUE_COMMAND_ID,
    DUNGEON_FINDER_ROLES_COMMAND_ID,
};

const COUNT_BYTES: usize = 1;
const F64_BYTES: usize = 8;
const MAX_ROLE_COUNT: usize = 3;
const MAX_ACTIVITY_COUNT: usize = 16;
const MAX_TAG_COUNT: usize = 8;
const MAX_ACTIVITY_UTF16_CODE_UNITS: usize = 64;
const MAX_ACTIVITY_UTF8_BYTES: usize = MAX_ACTIVITY_UTF16_CODE_UNITS * 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DungeonFinderRole {
    Tank,
    Healer,
    Dps,
}

impl DungeonFinderRole {
    const fn wire_code(self) -> u8 {
        match self {
            Self::Tank => 0,
            Self::Healer => 1,
            Self::Dps => 2,
        }
    }

    fn from_wire_code(code: u8) -> Result<Self, ProtocolError> {
        match code {
            0 => Ok(Self::Tank),
            1 => Ok(Self::Healer),
            2 => Ok(Self::Dps),
            invalid => Err(ProtocolError::InvalidDungeonFinderRole(invalid)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DungeonFinderListingTag {
    FirstRun,
    QuestRun,
    FullClear,
    Learning,
    FastRun,
}

impl DungeonFinderListingTag {
    const fn wire_code(self) -> u8 {
        match self {
            Self::FirstRun => 0,
            Self::QuestRun => 1,
            Self::FullClear => 2,
            Self::Learning => 3,
            Self::FastRun => 4,
        }
    }

    fn from_wire_code(code: u8) -> Result<Self, ProtocolError> {
        match code {
            0 => Ok(Self::FirstRun),
            1 => Ok(Self::QuestRun),
            2 => Ok(Self::FullClear),
            3 => Ok(Self::Learning),
            4 => Ok(Self::FastRun),
            invalid => Err(ProtocolError::InvalidDungeonFinderListingTag(invalid)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DungeonFinderRolesPayload {
    pub roles: Vec<DungeonFinderRole>,
}

impl DungeonFinderRolesPayload {
    pub fn encode(self) -> Result<Vec<u8>, ProtocolError> {
        validate_count(
            self.roles.len(),
            MAX_ROLE_COUNT,
            "DungeonFinderRolesPayload.roles",
        )?;
        let mut bytes = Vec::with_capacity(COUNT_BYTES + self.roles.len());
        bytes.push(self.roles.len() as u8);
        bytes.extend(self.roles.into_iter().map(DungeonFinderRole::wire_code));
        validate_command_payload(DUNGEON_FINDER_ROLES_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(DUNGEON_FINDER_ROLES_COMMAND_ID, bytes)?;
        decode_roles(bytes)
    }
}

fn decode_roles(bytes: &[u8]) -> Result<DungeonFinderRolesPayload, ProtocolError> {
    let mut offset = 0;
    let count = read_count(
        bytes,
        &mut offset,
        MAX_ROLE_COUNT,
        "DungeonFinderRolesPayload.roles",
    )?;
    let mut roles = Vec::with_capacity(count);
    for _ in 0..count {
        roles.push(DungeonFinderRole::from_wire_code(read_u8(
            bytes,
            &mut offset,
            "DungeonFinderRolesPayload.roles",
        )?)?);
    }
    reject_trailing(bytes, offset)?;
    Ok(DungeonFinderRolesPayload { roles })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DungeonFinderActivitiesPayload {
    pub activities: Vec<String>,
}

impl DungeonFinderActivitiesPayload {
    pub fn encode(self) -> Result<Vec<u8>, ProtocolError> {
        validate_count(
            self.activities.len(),
            MAX_ACTIVITY_COUNT,
            "DungeonFinderActivitiesPayload.activities",
        )?;
        let mut bytes = Vec::new();
        bytes.push(self.activities.len() as u8);
        for activity in &self.activities {
            write_activity(
                &mut bytes,
                activity,
                "DungeonFinderActivitiesPayload.activity",
            )?;
        }
        validate_command_payload(DUNGEON_FINDER_QUEUE_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(DUNGEON_FINDER_QUEUE_COMMAND_ID, bytes)?;
        decode_activities(bytes)
    }
}

fn decode_activities(bytes: &[u8]) -> Result<DungeonFinderActivitiesPayload, ProtocolError> {
    let mut offset = 0;
    let count = read_count(
        bytes,
        &mut offset,
        MAX_ACTIVITY_COUNT,
        "DungeonFinderActivitiesPayload.activities",
    )?;
    let mut activities = Vec::with_capacity(count);
    for _ in 0..count {
        activities.push(read_activity(
            bytes,
            &mut offset,
            "DungeonFinderActivitiesPayload.activity",
        )?);
    }
    reject_trailing(bytes, offset)?;
    Ok(DungeonFinderActivitiesPayload { activities })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DungeonFinderListingPayload {
    pub activity: String,
    pub tags: Vec<DungeonFinderListingTag>,
}

impl DungeonFinderListingPayload {
    pub fn encode(self) -> Result<Vec<u8>, ProtocolError> {
        validate_count(
            self.tags.len(),
            MAX_TAG_COUNT,
            "DungeonFinderListingPayload.tags",
        )?;
        let mut bytes =
            Vec::with_capacity(COUNT_BYTES + self.activity.len() + COUNT_BYTES + self.tags.len());
        write_activity(
            &mut bytes,
            &self.activity,
            "DungeonFinderListingPayload.activity",
        )?;
        bytes.push(self.tags.len() as u8);
        bytes.extend(
            self.tags
                .into_iter()
                .map(DungeonFinderListingTag::wire_code),
        );
        validate_command_payload(DUNGEON_FINDER_LIST_CREATE_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(DUNGEON_FINDER_LIST_CREATE_COMMAND_ID, bytes)?;
        decode_listing(bytes)
    }
}

fn decode_listing(bytes: &[u8]) -> Result<DungeonFinderListingPayload, ProtocolError> {
    let mut offset = 0;
    let activity = read_activity(bytes, &mut offset, "DungeonFinderListingPayload.activity")?;
    let count = read_count(
        bytes,
        &mut offset,
        MAX_TAG_COUNT,
        "DungeonFinderListingPayload.tags",
    )?;
    let mut tags = Vec::with_capacity(count);
    for _ in 0..count {
        tags.push(DungeonFinderListingTag::from_wire_code(read_u8(
            bytes,
            &mut offset,
            "DungeonFinderListingPayload.tags",
        )?)?);
    }
    reject_trailing(bytes, offset)?;
    Ok(DungeonFinderListingPayload { activity, tags })
}

#[derive(Clone, Debug, PartialEq)]
pub struct DungeonFinderListingIdPayload {
    pub listing_id: f64,
}

impl DungeonFinderListingIdPayload {
    pub fn encode(self) -> Result<[u8; F64_BYTES], ProtocolError> {
        let listing_id =
            canonical_finite_f64("DungeonFinderListingIdPayload.listing_id", self.listing_id)?;
        let bytes = listing_id.to_le_bytes();
        validate_command_payload(DUNGEON_FINDER_APPLY_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(DUNGEON_FINDER_APPLY_COMMAND_ID, bytes)?;
        Ok(Self {
            listing_id: read_finite_f64(bytes, 0, "DungeonFinderListingIdPayload.listing_id")?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DungeonFinderApplicationResponsePayload {
    pub applicant_id: f64,
    pub accept: bool,
}

impl DungeonFinderApplicationResponsePayload {
    pub fn encode(self) -> Result<[u8; F64_BYTES + COUNT_BYTES], ProtocolError> {
        let applicant_id = canonical_finite_f64(
            "DungeonFinderApplicationResponsePayload.applicant_id",
            self.applicant_id,
        )?;
        let mut bytes = [0; F64_BYTES + COUNT_BYTES];
        bytes[..F64_BYTES].copy_from_slice(&applicant_id.to_le_bytes());
        bytes[F64_BYTES] = u8::from(self.accept);
        validate_command_payload(DUNGEON_FINDER_APPLICATION_RESPONSE_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(DUNGEON_FINDER_APPLICATION_RESPONSE_COMMAND_ID, bytes)?;
        let applicant_id = read_finite_f64(
            bytes,
            0,
            "DungeonFinderApplicationResponsePayload.applicant_id",
        )?;
        let accept = read_bool(
            bytes,
            F64_BYTES,
            "DungeonFinderApplicationResponsePayload.accept",
        )?;
        Ok(Self {
            applicant_id,
            accept,
        })
    }
}

pub(crate) fn validate_dungeon_finder_roles_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    decode_roles(bytes).map(|_| ())
}

pub(crate) fn validate_dungeon_finder_activities_payload(
    bytes: &[u8],
) -> Result<(), ProtocolError> {
    decode_activities(bytes).map(|_| ())
}

pub(crate) fn validate_dungeon_finder_listing_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    decode_listing(bytes).map(|_| ())
}

pub(crate) fn validate_dungeon_finder_listing_id_payload(
    bytes: &[u8],
) -> Result<(), ProtocolError> {
    let _ = read_finite_f64(bytes, 0, "DungeonFinderListingIdPayload.listing_id")?;
    Ok(())
}

pub(crate) fn validate_dungeon_finder_application_response_payload(
    bytes: &[u8],
) -> Result<(), ProtocolError> {
    let _ = read_finite_f64(
        bytes,
        0,
        "DungeonFinderApplicationResponsePayload.applicant_id",
    )?;
    let _ = read_bool(
        bytes,
        F64_BYTES,
        "DungeonFinderApplicationResponsePayload.accept",
    )?;
    Ok(())
}

fn write_activity(
    bytes: &mut Vec<u8>,
    activity: &str,
    context: &'static str,
) -> Result<(), ProtocolError> {
    validate_activity(activity, context)?;
    bytes.push(activity.len() as u8);
    bytes.extend_from_slice(activity.as_bytes());
    Ok(())
}

fn read_activity(
    bytes: &[u8],
    offset: &mut usize,
    context: &'static str,
) -> Result<String, ProtocolError> {
    let byte_length = read_u8(bytes, offset, context)? as usize;
    if byte_length > MAX_ACTIVITY_UTF8_BYTES {
        return Err(ProtocolError::CollectionTooLarge {
            context,
            actual: byte_length,
            maximum: MAX_ACTIVITY_UTF8_BYTES,
        });
    }
    let start = *offset;
    let raw = take(bytes, offset, byte_length, context)?;
    let activity = std::str::from_utf8(raw)
        .map_err(|_| ProtocolError::InvalidUtf8 { context })?
        .to_owned();
    validate_activity(&activity, context)?;
    debug_assert_eq!(start + byte_length, *offset);
    Ok(activity)
}

fn validate_activity(activity: &str, context: &'static str) -> Result<(), ProtocolError> {
    let utf16_units = activity.encode_utf16().count();
    if utf16_units > MAX_ACTIVITY_UTF16_CODE_UNITS {
        return Err(ProtocolError::CollectionTooLarge {
            context,
            actual: utf16_units,
            maximum: MAX_ACTIVITY_UTF16_CODE_UNITS,
        });
    }
    if activity.len() > MAX_ACTIVITY_UTF8_BYTES {
        return Err(ProtocolError::CollectionTooLarge {
            context,
            actual: activity.len(),
            maximum: MAX_ACTIVITY_UTF8_BYTES,
        });
    }
    Ok(())
}

fn validate_count(
    actual: usize,
    maximum: usize,
    context: &'static str,
) -> Result<(), ProtocolError> {
    if actual > maximum {
        return Err(ProtocolError::CollectionTooLarge {
            context,
            actual,
            maximum,
        });
    }
    Ok(())
}

fn read_count(
    bytes: &[u8],
    offset: &mut usize,
    maximum: usize,
    context: &'static str,
) -> Result<usize, ProtocolError> {
    let count = read_u8(bytes, offset, context)? as usize;
    validate_count(count, maximum, context)?;
    Ok(count)
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
        bytes[offset..offset + F64_BYTES]
            .try_into()
            .expect("validated Dungeon Finder payload contains a complete f64"),
    );
    canonical_finite_f64(context, value)
}

fn read_bool(bytes: &[u8], offset: usize, _context: &'static str) -> Result<bool, ProtocolError> {
    match bytes[offset] {
        0 => Ok(false),
        1 => Ok(true),
        invalid => Err(ProtocolError::InvalidBoolean(invalid)),
    }
}

fn read_u8(bytes: &[u8], offset: &mut usize, context: &'static str) -> Result<u8, ProtocolError> {
    let value = take(bytes, offset, 1, context)?[0];
    Ok(value)
}

fn take<'a>(
    bytes: &'a [u8],
    offset: &mut usize,
    length: usize,
    context: &'static str,
) -> Result<&'a [u8], ProtocolError> {
    let remaining = bytes.len().saturating_sub(*offset);
    if remaining < length {
        return Err(ProtocolError::TruncatedPayload {
            context,
            needed: length,
            remaining,
        });
    }
    let start = *offset;
    *offset += length;
    Ok(&bytes[start..*offset])
}

fn reject_trailing(bytes: &[u8], offset: usize) -> Result<(), ProtocolError> {
    let remaining = bytes.len().saturating_sub(offset);
    if remaining == 0 {
        Ok(())
    } else {
        Err(ProtocolError::TrailingPayload { remaining })
    }
}
