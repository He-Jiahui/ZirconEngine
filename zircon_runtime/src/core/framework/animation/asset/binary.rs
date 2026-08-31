//! Shared binary envelope for animation resource schemas.

use bincode::Options;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::error::{AnimationAssetError, AnimationAssetResult};

const ANIMATION_BINARY_MAGIC: [u8; 8] = *b"ZRANIM01";
const ANIMATION_BINARY_VERSION: u32 = 1;
const KIBIBYTE: usize = 1024;
const MEBIBYTE: usize = KIBIBYTE * KIBIBYTE;
// Animation authoring schemas are compact binary metadata. Keep an input cap aligned with the
// existing untrusted-font source policy while preventing malformed vector lengths from consuming
// an unbounded amount of memory during bincode deserialization or version fallback attempts.
const ANIMATION_BINARY_MAX_DECODE_BYTES: usize = 64 * MEBIBYTE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum AnimationBinaryAssetKind {
    Skeleton,
    Clip,
    Sequence,
    Graph,
    StateMachine,
}

impl AnimationBinaryAssetKind {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Skeleton => "skeleton",
            Self::Clip => "clip",
            Self::Sequence => "sequence",
            Self::Graph => "graph",
            Self::StateMachine => "state_machine",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationBinaryDocument<T> {
    magic: [u8; 8],
    version: u32,
    kind: AnimationBinaryAssetKind,
    payload: T,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct AnimationBinaryHeader {
    magic: [u8; 8],
    version: u32,
    kind: AnimationBinaryAssetKind,
}

pub(super) fn encode_binary_asset<T>(
    kind: AnimationBinaryAssetKind,
    payload: &T,
) -> AnimationAssetResult<Vec<u8>>
where
    T: Serialize + Clone,
{
    bincode::serialize(&AnimationBinaryDocument {
        magic: ANIMATION_BINARY_MAGIC,
        version: ANIMATION_BINARY_VERSION,
        kind,
        payload: payload.clone(),
    })
    .map_err(|source| AnimationAssetError::Serialize {
        kind: kind.as_str(),
        source,
    })
}

pub(super) fn decode_binary_asset<T>(
    kind: AnimationBinaryAssetKind,
    bytes: &[u8],
) -> AnimationAssetResult<T>
where
    T: DeserializeOwned,
{
    validate_binary_input_size(kind, bytes)?;
    match decode_binary_document_asset(kind, bytes) {
        Ok(payload) => Ok(payload),
        Err(document_error) => decode_binary_stream_asset(kind, bytes).map_err(|stream_error| {
            AnimationAssetError::DocumentAndStreamDecode {
                kind: kind.as_str(),
                document: Box::new(document_error),
                stream: Box::new(stream_error),
            }
        }),
    }
}

pub(super) fn decode_binary_asset_with_v1_payload_fallback<T, V1>(
    kind: AnimationBinaryAssetKind,
    bytes: &[u8],
) -> AnimationAssetResult<T>
where
    T: DeserializeOwned,
    V1: DeserializeOwned + TryInto<T>,
    <V1 as TryInto<T>>::Error: Into<AnimationAssetError>,
{
    match decode_binary_asset(kind, bytes) {
        Ok(payload) => Ok(payload),
        Err(error @ AnimationAssetError::InputTooLarge { .. }) => Err(error),
        Err(primary_error) => {
            let v1_payload = decode_binary_asset::<V1>(kind, bytes)
                .and_then(|payload| payload.try_into().map_err(Into::into))
                .map_err(|v1_error| AnimationAssetError::CurrentAndV1PayloadDecode {
                    kind: kind.as_str(),
                    current: Box::new(primary_error),
                    v1: Box::new(v1_error),
                })?;
            Ok(v1_payload)
        }
    }
}

pub(super) fn decode_binary_asset_with_v3_v2_v1_payload_fallback<T, V3, V2, V1>(
    kind: AnimationBinaryAssetKind,
    bytes: &[u8],
) -> AnimationAssetResult<T>
where
    T: DeserializeOwned,
    V3: DeserializeOwned + TryInto<T>,
    <V3 as TryInto<T>>::Error: Into<AnimationAssetError>,
    V2: DeserializeOwned + TryInto<T>,
    <V2 as TryInto<T>>::Error: Into<AnimationAssetError>,
    V1: DeserializeOwned + TryInto<T>,
    <V1 as TryInto<T>>::Error: Into<AnimationAssetError>,
{
    match decode_binary_asset(kind, bytes) {
        Ok(payload) => Ok(payload),
        Err(error @ AnimationAssetError::InputTooLarge { .. }) => Err(error),
        Err(current_error) => match decode_binary_asset::<V3>(kind, bytes)
            .and_then(|payload| payload.try_into().map_err(Into::into))
        {
            Ok(payload) => Ok(payload),
            Err(v3_error) => match decode_binary_asset::<V2>(kind, bytes)
                .and_then(|payload| payload.try_into().map_err(Into::into))
            {
                Ok(payload) => Ok(payload),
                Err(v2_error) => decode_binary_asset::<V1>(kind, bytes)
                    .and_then(|payload| payload.try_into().map_err(Into::into))
                    .map_err(
                        |v1_error| AnimationAssetError::CurrentV3V2AndV1PayloadDecode {
                            kind: kind.as_str(),
                            current: Box::new(current_error),
                            v3: Box::new(v3_error),
                            v2: Box::new(v2_error),
                            v1: Box::new(v1_error),
                        },
                    ),
            },
        },
    }
}

fn decode_binary_document_asset<T>(
    kind: AnimationBinaryAssetKind,
    bytes: &[u8],
) -> AnimationAssetResult<T>
where
    T: DeserializeOwned,
{
    let document: AnimationBinaryDocument<T> = bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .allow_trailing_bytes()
        .with_limit(ANIMATION_BINARY_MAX_DECODE_BYTES as u64)
        .deserialize(bytes)
        .map_err(|source| AnimationAssetError::DocumentDeserialize {
            kind: kind.as_str(),
            source,
        })?;
    validate_binary_header(kind, document.magic, document.version, document.kind)?;
    Ok(document.payload)
}

fn decode_binary_stream_asset<T>(
    kind: AnimationBinaryAssetKind,
    bytes: &[u8],
) -> AnimationAssetResult<T>
where
    T: DeserializeOwned,
{
    let mut cursor = std::io::Cursor::new(bytes);
    let header: AnimationBinaryHeader = bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .allow_trailing_bytes()
        .with_limit(ANIMATION_BINARY_MAX_DECODE_BYTES as u64)
        .deserialize_from(&mut cursor)
        .map_err(|source| AnimationAssetError::StreamHeaderDeserialize {
            kind: kind.as_str(),
            source,
        })?;
    validate_binary_header(kind, header.magic, header.version, header.kind)?;

    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .allow_trailing_bytes()
        .with_limit(ANIMATION_BINARY_MAX_DECODE_BYTES as u64)
        .deserialize_from(&mut cursor)
        .map_err(|source| AnimationAssetError::StreamPayloadDeserialize {
            kind: kind.as_str(),
            source,
        })
}

fn validate_binary_input_size(
    kind: AnimationBinaryAssetKind,
    bytes: &[u8],
) -> AnimationAssetResult<()> {
    validate_binary_input_len(kind, bytes.len())
}

fn validate_binary_input_len(
    kind: AnimationBinaryAssetKind,
    actual_bytes: usize,
) -> AnimationAssetResult<()> {
    if actual_bytes > ANIMATION_BINARY_MAX_DECODE_BYTES {
        return Err(AnimationAssetError::InputTooLarge {
            kind: kind.as_str(),
            actual_bytes: actual_bytes as u64,
            limit_bytes: ANIMATION_BINARY_MAX_DECODE_BYTES as u64,
        });
    }
    Ok(())
}

fn validate_binary_header(
    kind: AnimationBinaryAssetKind,
    magic: [u8; 8],
    version: u32,
    actual_kind: AnimationBinaryAssetKind,
) -> AnimationAssetResult<()> {
    if magic != ANIMATION_BINARY_MAGIC {
        return Err(AnimationAssetError::InvalidMagic);
    }
    if version != ANIMATION_BINARY_VERSION {
        return Err(AnimationAssetError::UnsupportedVersion { version });
    }
    if actual_kind != kind {
        return Err(AnimationAssetError::KindMismatch {
            expected: kind.as_str(),
            actual: actual_kind.as_str(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        decode_binary_asset, encode_binary_asset, validate_binary_input_len,
        AnimationBinaryAssetKind, ANIMATION_BINARY_MAX_DECODE_BYTES,
    };
    use crate::core::framework::animation::AnimationAssetError;

    #[test]
    fn animation_binary_rejects_oversized_input_before_deserialization() {
        let error = validate_binary_input_len(
            AnimationBinaryAssetKind::Graph,
            ANIMATION_BINARY_MAX_DECODE_BYTES + 1,
        )
        .expect_err("oversized input must be rejected before deserialize");

        assert!(matches!(
            error,
            AnimationAssetError::InputTooLarge {
                kind: "graph",
                actual_bytes,
                limit_bytes,
            } if actual_bytes == limit_bytes + 1
        ));
    }

    #[test]
    fn animation_binary_budgeting_preserves_legacy_trailing_byte_decoding() {
        let mut bytes = encode_binary_asset(AnimationBinaryAssetKind::Graph, &7_u8)
            .expect("fixture serialization succeeds");
        bytes.push(0);

        let decoded = decode_binary_asset::<u8>(AnimationBinaryAssetKind::Graph, &bytes)
            .expect("legacy trailing bytes remain accepted");

        assert_eq!(decoded, 7);
    }
}
