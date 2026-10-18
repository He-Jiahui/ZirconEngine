use crate::{validate_command_payload, ProtocolError, CHALLENGE_RESPONSE_COMMAND_ID};

const CHALLENGE_STRING_COUNT: usize = 3;
const CHALLENGE_LENGTH_PREFIX_BYTES: usize = 4;
const CHALLENGE_MAX_UTF8_BYTES: usize = 256;
const CHALLENGE_STRING_CONTEXTS: [&str; CHALLENGE_STRING_COUNT] = [
    "ChallengeResponseCommandPayload.nonce",
    "ChallengeResponseCommandPayload.response",
    "ChallengeResponseCommandPayload.signature",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChallengeResponseCommandPayload {
    pub nonce: String,
    pub response: String,
    pub signature: String,
}

impl ChallengeResponseCommandPayload {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let values = [
            (self.nonce.as_str(), CHALLENGE_STRING_CONTEXTS[0]),
            (self.response.as_str(), CHALLENGE_STRING_CONTEXTS[1]),
            (self.signature.as_str(), CHALLENGE_STRING_CONTEXTS[2]),
        ];
        let mut bytes = Vec::with_capacity(
            values
                .iter()
                .map(|(value, _)| CHALLENGE_LENGTH_PREFIX_BYTES + value.len())
                .sum(),
        );
        for (value, context) in values {
            write_challenge_string(value, context, &mut bytes)?;
        }
        validate_command_payload(CHALLENGE_RESPONSE_COMMAND_ID, &bytes)?;
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        validate_command_payload(CHALLENGE_RESPONSE_COMMAND_ID, bytes)?;
        let (nonce, consumed) = read_challenge_string(bytes, 0, CHALLENGE_STRING_CONTEXTS[0])?;
        let (response, consumed) =
            read_challenge_string(bytes, consumed, CHALLENGE_STRING_CONTEXTS[1])?;
        let (signature, consumed) =
            read_challenge_string(bytes, consumed, CHALLENGE_STRING_CONTEXTS[2])?;
        reject_challenge_trailing(bytes, consumed)?;
        Ok(Self {
            nonce: nonce.to_owned(),
            response: response.to_owned(),
            signature: signature.to_owned(),
        })
    }
}

pub(crate) fn validate_challenge_response_payload(bytes: &[u8]) -> Result<(), ProtocolError> {
    let mut consumed = 0;
    for context in CHALLENGE_STRING_CONTEXTS {
        consumed = read_challenge_string(bytes, consumed, context)?.1;
    }
    reject_challenge_trailing(bytes, consumed)
}

fn write_challenge_string(
    value: &str,
    context: &'static str,
    bytes: &mut Vec<u8>,
) -> Result<(), ProtocolError> {
    if value.len() > CHALLENGE_MAX_UTF8_BYTES {
        return Err(ProtocolError::CollectionTooLarge {
            context,
            actual: value.len(),
            maximum: CHALLENGE_MAX_UTF8_BYTES,
        });
    }
    bytes.extend_from_slice(
        &u32::try_from(value.len())
            .expect("bounded challenge string length fits u32")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn read_challenge_string<'a>(
    bytes: &'a [u8],
    offset: usize,
    context: &'static str,
) -> Result<(&'a str, usize), ProtocolError> {
    let length = u32::from_le_bytes(
        take_challenge(bytes, offset, CHALLENGE_LENGTH_PREFIX_BYTES, context)?
            .try_into()
            .expect("challenge string length prefix has four bytes"),
    ) as usize;
    if length > CHALLENGE_MAX_UTF8_BYTES {
        return Err(ProtocolError::CollectionTooLarge {
            context,
            actual: length,
            maximum: CHALLENGE_MAX_UTF8_BYTES,
        });
    }
    let value_offset = offset.checked_add(CHALLENGE_LENGTH_PREFIX_BYTES).ok_or(
        ProtocolError::TruncatedPayload {
            context,
            needed: CHALLENGE_LENGTH_PREFIX_BYTES,
            remaining: bytes.len().saturating_sub(offset),
        },
    )?;
    let value = take_challenge(bytes, value_offset, length, context)?;
    let value = std::str::from_utf8(value).map_err(|_| ProtocolError::InvalidUtf8 { context })?;
    Ok((value, value_offset + length))
}

fn take_challenge<'a>(
    bytes: &'a [u8],
    offset: usize,
    length: usize,
    context: &'static str,
) -> Result<&'a [u8], ProtocolError> {
    let end = offset
        .checked_add(length)
        .ok_or(ProtocolError::TruncatedPayload {
            context,
            needed: length,
            remaining: bytes.len().saturating_sub(offset),
        })?;
    bytes
        .get(offset..end)
        .ok_or(ProtocolError::TruncatedPayload {
            context,
            needed: length,
            remaining: bytes.len().saturating_sub(offset),
        })
}

fn reject_challenge_trailing(bytes: &[u8], consumed: usize) -> Result<(), ProtocolError> {
    let remaining = bytes.len().saturating_sub(consumed);
    if remaining == 0 {
        Ok(())
    } else {
        Err(ProtocolError::TrailingPayload { remaining })
    }
}
