use std::cmp::Ordering;

use crate::{limit, MessageKind, ProtocolError, PROTOCOL_VERSION, SCHEMA_FINGERPRINT_BYTES};

const FRAME_MAGIC: [u8; 4] = *b"WOC1";
pub const FRAME_HEADER_BYTES: usize = 44;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodeLimits {
    pub max_payload_bytes: usize,
}

impl Default for DecodeLimits {
    fn default() -> Self {
        Self {
            max_payload_bytes: usize::try_from(limit::FRAME_PAYLOAD_BYTES)
                .expect("frame payload limit must fit usize"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub kind: MessageKind,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn new(kind: MessageKind, payload: Vec<u8>) -> Self {
        Self { kind, payload }
    }
}

pub fn encode_frame(frame: &Frame, limits: DecodeLimits) -> Result<Vec<u8>, ProtocolError> {
    validate_payload_length(frame.payload.len(), limits)?;
    let payload_length =
        u32::try_from(frame.payload.len()).map_err(|_| ProtocolError::PayloadTooLarge {
            actual: frame.payload.len(),
            maximum: u32::MAX as usize,
        })?;
    let mut encoded = Vec::with_capacity(FRAME_HEADER_BYTES + frame.payload.len());
    encoded.extend_from_slice(&FRAME_MAGIC);
    encoded.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
    encoded.extend_from_slice(&(frame.kind as u16).to_le_bytes());
    encoded.extend_from_slice(&SCHEMA_FINGERPRINT_BYTES);
    encoded.extend_from_slice(&payload_length.to_le_bytes());
    encoded.extend_from_slice(&frame.payload);
    Ok(encoded)
}

pub fn decode_frame(bytes: &[u8], limits: DecodeLimits) -> Result<Frame, ProtocolError> {
    if bytes.len() < FRAME_HEADER_BYTES {
        return Err(ProtocolError::TruncatedHeader {
            actual: bytes.len(),
            minimum: FRAME_HEADER_BYTES,
        });
    }
    let magic: [u8; 4] = bytes[0..4].try_into().expect("fixed magic slice");
    if magic != FRAME_MAGIC {
        return Err(ProtocolError::InvalidMagic { actual: magic });
    }
    let version = u16::from_le_bytes(bytes[4..6].try_into().expect("fixed version slice"));
    if version != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion {
            actual: version,
            expected: PROTOCOL_VERSION,
        });
    }
    let kind = MessageKind::try_from(u16::from_le_bytes(
        bytes[6..8].try_into().expect("fixed kind slice"),
    ))?;
    let fingerprint: [u8; 32] = bytes[8..40].try_into().expect("fixed fingerprint slice");
    if fingerprint != SCHEMA_FINGERPRINT_BYTES {
        return Err(ProtocolError::SchemaMismatch {
            actual: fingerprint,
        });
    }
    let declared =
        u32::from_le_bytes(bytes[40..44].try_into().expect("fixed length slice")) as usize;
    validate_payload_length(declared, limits)?;
    let actual = bytes.len() - FRAME_HEADER_BYTES;
    if actual != declared {
        return Err(ProtocolError::LengthMismatch { declared, actual });
    }
    Ok(Frame::new(kind, bytes[FRAME_HEADER_BYTES..].to_vec()))
}

pub fn require_finite(field: &'static str, value: f64) -> Result<f64, ProtocolError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(ProtocolError::NonFinite { field, value })
    }
}

pub fn canonical_pairs<K: Ord, V>(
    values: impl IntoIterator<Item = (K, V)>,
) -> Result<Vec<(K, V)>, ProtocolError> {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort_by(|left, right| left.0.cmp(&right.0));
    if values
        .windows(2)
        .any(|window| window[0].0.cmp(&window[1].0) == Ordering::Equal)
    {
        return Err(ProtocolError::DuplicateCanonicalKey);
    }
    Ok(values)
}

fn validate_payload_length(actual: usize, limits: DecodeLimits) -> Result<(), ProtocolError> {
    if actual > limits.max_payload_bytes {
        return Err(ProtocolError::PayloadTooLarge {
            actual,
            maximum: limits.max_payload_bytes,
        });
    }
    Ok(())
}
