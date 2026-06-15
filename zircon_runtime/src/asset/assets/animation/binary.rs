use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

const ANIMATION_BINARY_MAGIC: [u8; 8] = *b"ZRANIM01";
const ANIMATION_BINARY_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum AnimationBinaryAssetKind {
    Skeleton,
    Clip,
    Sequence,
    Graph,
    StateMachine,
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
) -> Result<Vec<u8>, String>
where
    T: Serialize + Clone,
{
    bincode::serialize(&AnimationBinaryDocument {
        magic: ANIMATION_BINARY_MAGIC,
        version: ANIMATION_BINARY_VERSION,
        kind,
        payload: payload.clone(),
    })
    .map_err(|error| error.to_string())
}

pub(super) fn decode_binary_asset<T>(
    kind: AnimationBinaryAssetKind,
    bytes: &[u8],
) -> Result<T, String>
where
    T: DeserializeOwned,
{
    match decode_binary_document_asset(kind, bytes) {
        Ok(payload) => Ok(payload),
        Err(document_error) => decode_binary_stream_asset(kind, bytes).map_err(|stream_error| {
            format!("{document_error}; animation stream decode failed: {stream_error}")
        }),
    }
}

pub(super) fn decode_binary_asset_with_v1_payload_fallback<T, V1>(
    kind: AnimationBinaryAssetKind,
    bytes: &[u8],
) -> Result<T, String>
where
    T: DeserializeOwned,
    V1: DeserializeOwned + TryInto<T>,
    <V1 as TryInto<T>>::Error: std::fmt::Display,
{
    match decode_binary_asset(kind, bytes) {
        Ok(payload) => Ok(payload),
        Err(primary_error) => {
            let v1_payload = decode_binary_asset::<V1>(kind, bytes)
                .and_then(|payload| payload.try_into().map_err(|error| error.to_string()))
                .map_err(|v1_error| {
                    format!("{primary_error}; v1 animation asset decode failed: {v1_error}")
                })?;
            Ok(v1_payload)
        }
    }
}

fn decode_binary_document_asset<T>(
    kind: AnimationBinaryAssetKind,
    bytes: &[u8],
) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let document: AnimationBinaryDocument<T> =
        bincode::deserialize(bytes).map_err(|error| error.to_string())?;
    validate_binary_header(kind, document.magic, document.version, document.kind)?;
    Ok(document.payload)
}

fn decode_binary_stream_asset<T>(kind: AnimationBinaryAssetKind, bytes: &[u8]) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let mut cursor = std::io::Cursor::new(bytes);
    let header: AnimationBinaryHeader =
        bincode::deserialize_from(&mut cursor).map_err(|error| error.to_string())?;
    validate_binary_header(kind, header.magic, header.version, header.kind)?;

    bincode::deserialize_from(&mut cursor).map_err(|error| error.to_string())
}

fn validate_binary_header(
    kind: AnimationBinaryAssetKind,
    magic: [u8; 8],
    version: u32,
    actual_kind: AnimationBinaryAssetKind,
) -> Result<(), String> {
    if magic != ANIMATION_BINARY_MAGIC {
        return Err("invalid animation asset magic".to_string());
    }
    if version != ANIMATION_BINARY_VERSION {
        return Err(format!("unsupported animation asset version {}", version));
    }
    if actual_kind != kind {
        return Err(format!(
            "animation asset kind mismatch: expected {:?}, found {:?}",
            kind, actual_kind
        ));
    }
    Ok(())
}
