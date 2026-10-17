//! Versioned gamepad payloads carried by the unchanged Runtime event V1 envelope.
//! Generation is captured by the producer, never reconstructed from receiver state.
//! Samples have exactly one header; connections append an optional UTF-8 name.
use std::num::NonZeroU64;

use crate::ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1;

pub const ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V2: u32 = 21;
pub const ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V2: u32 = 22;
pub const ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V2: u32 = 23;
pub const ZR_RUNTIME_GAMEPAD_GENERATION_V2_HEADER_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrRuntimeGamepadGenerationV2Error {
    PayloadTooShort,
    PayloadTooLong,
    InvalidMagic,
    UnsupportedVersion,
    NonzeroReserved,
    ZeroGeneration,
    UnexpectedSamplePayloadLength,
    InvalidUtf8Name,
}

/// Borrowed connection decoding; the caller owns the bytes for this view's lifetime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeGamepadGenerationV2<'a> {
    pub generation: NonZeroU64,
    pub name: Option<&'a str>,
}

impl<'a> ZrRuntimeGamepadGenerationV2<'a> {
    pub fn encode_sample(generation: NonZeroU64) -> [u8; ZR_RUNTIME_GAMEPAD_GENERATION_V2_HEADER_BYTES] {
        let mut bytes = [0_u8; ZR_RUNTIME_GAMEPAD_GENERATION_V2_HEADER_BYTES];
        bytes[..4].copy_from_slice(b"ZRG2");
        bytes[4..6].copy_from_slice(&2_u16.to_le_bytes());
        bytes[8..16].copy_from_slice(&generation.get().to_le_bytes());
        bytes
    }

    pub fn encode_connection(
        generation: NonZeroU64,
        name: Option<&str>,
    ) -> Result<Vec<u8>, ZrRuntimeGamepadGenerationV2Error> {
        let name = name.unwrap_or("").as_bytes();
        let length = ZR_RUNTIME_GAMEPAD_GENERATION_V2_HEADER_BYTES
            .checked_add(name.len())
            .ok_or(ZrRuntimeGamepadGenerationV2Error::PayloadTooLong)?;
        if length > ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1 {
            return Err(ZrRuntimeGamepadGenerationV2Error::PayloadTooLong);
        }
        let mut bytes = Vec::with_capacity(length);
        bytes.extend_from_slice(&Self::encode_sample(generation));
        bytes.extend_from_slice(name);
        Ok(bytes)
    }

    pub fn decode_sample(bytes: &[u8]) -> Result<NonZeroU64, ZrRuntimeGamepadGenerationV2Error> {
        let generation = decode_header(bytes)?;
        if bytes.len() != ZR_RUNTIME_GAMEPAD_GENERATION_V2_HEADER_BYTES {
            return Err(ZrRuntimeGamepadGenerationV2Error::UnexpectedSamplePayloadLength);
        }
        Ok(generation)
    }

    pub fn decode_connection(bytes: &'a [u8]) -> Result<Self, ZrRuntimeGamepadGenerationV2Error> {
        let generation = decode_header(bytes)?;
        let name = &bytes[ZR_RUNTIME_GAMEPAD_GENERATION_V2_HEADER_BYTES..];
        let name = if name.is_empty() {
            None
        } else {
            Some(std::str::from_utf8(name)
                .map_err(|_| ZrRuntimeGamepadGenerationV2Error::InvalidUtf8Name)?)
        };
        Ok(Self { generation, name })
    }
}

fn decode_header(bytes: &[u8]) -> Result<NonZeroU64, ZrRuntimeGamepadGenerationV2Error> {
    use ZrRuntimeGamepadGenerationV2Error as Error;
    if bytes.len() < ZR_RUNTIME_GAMEPAD_GENERATION_V2_HEADER_BYTES {
        return Err(Error::PayloadTooShort);
    }
    if bytes.len() > ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1 {
        return Err(Error::PayloadTooLong);
    }
    if &bytes[..4] != b"ZRG2" {
        return Err(Error::InvalidMagic);
    }
    if u16::from_le_bytes([bytes[4], bytes[5]]) != 2 {
        return Err(Error::UnsupportedVersion);
    }
    if bytes[6] != 0 || bytes[7] != 0 {
        return Err(Error::NonzeroReserved);
    }
    NonZeroU64::new(u64::from_le_bytes([
        bytes[8], bytes[9], bytes[10], bytes[11],
        bytes[12], bytes[13], bytes[14], bytes[15],
    ])).ok_or(Error::ZeroGeneration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamepad_v2_round_trips_nonzero_generation_and_borrowed_name() {
        let generation = NonZeroU64::new(u64::MAX).unwrap();
        let bytes = ZrRuntimeGamepadGenerationV2::encode_connection(generation, Some("pad")).unwrap();
        let decoded = ZrRuntimeGamepadGenerationV2::decode_connection(&bytes).unwrap();
        assert_eq!(decoded.generation, generation);
        assert_eq!(decoded.name, Some("pad"));
        assert_eq!(decoded.name.unwrap().as_ptr(), bytes[16..].as_ptr());
        let sample = ZrRuntimeGamepadGenerationV2::encode_sample(generation);
        assert_eq!(ZrRuntimeGamepadGenerationV2::decode_sample(&sample), Ok(generation));
    }

    #[test]
    fn gamepad_v2_rejects_invalid_generation_payload() {
        use ZrRuntimeGamepadGenerationV2Error as Error;
        let sample = ZrRuntimeGamepadGenerationV2::encode_sample(NonZeroU64::new(9).unwrap());
        assert_eq!(ZrRuntimeGamepadGenerationV2::decode_sample(&sample[..15]), Err(Error::PayloadTooShort));
        let mut invalid = sample;
        invalid[0] = 0;
        assert_eq!(ZrRuntimeGamepadGenerationV2::decode_sample(&invalid), Err(Error::InvalidMagic));
        invalid = sample;
        invalid[4] = 1;
        assert_eq!(ZrRuntimeGamepadGenerationV2::decode_sample(&invalid), Err(Error::UnsupportedVersion));
        invalid = sample;
        invalid[6] = 1;
        assert_eq!(ZrRuntimeGamepadGenerationV2::decode_sample(&invalid), Err(Error::NonzeroReserved));
        invalid = sample;
        invalid[8..16].fill(0);
        assert_eq!(ZrRuntimeGamepadGenerationV2::decode_sample(&invalid), Err(Error::ZeroGeneration));
        let mut trailing = sample.to_vec();
        trailing.push(0);
        assert_eq!(ZrRuntimeGamepadGenerationV2::decode_sample(&trailing), Err(Error::UnexpectedSamplePayloadLength));
        trailing[16] = 0xff;
        assert_eq!(ZrRuntimeGamepadGenerationV2::decode_connection(&trailing), Err(Error::InvalidUtf8Name));
        let oversized = vec![0; ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1 + 1];
        assert_eq!(ZrRuntimeGamepadGenerationV2::decode_connection(&oversized), Err(Error::PayloadTooLong));
    }
}
