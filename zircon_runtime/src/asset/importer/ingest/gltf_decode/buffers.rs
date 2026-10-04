use crate::asset::AssetImportError;

use super::super::auxiliary_source::AuxiliarySourceResolver;
use super::super::gltf_meshopt::buffer_is_meshopt_fallback;
use super::budget::DecodedBudget;
use super::sources::ExternalSources;
use super::{gltf_buffer_source_name, gltf_parse_error, is_data_uri};

const GLTF_BUFFER_ALIGNMENT: usize = 4;
const GLTF_BASE64: GeneralPurpose = GeneralPurpose::new(
    &base64::alphabet::STANDARD,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

pub(super) fn load_buffers(
    document: &gltf::Document,
    mut blob: Option<Vec<u8>>,
    sources: &mut ExternalSources<'_>,
    budget: &mut DecodedBudget,
) -> Result<Vec<gltf::buffer::Data>, AssetImportError> {
    if document.buffers().len() > AuxiliarySourceResolver::MAX_SNAPSHOT_FILES {
        return Err(gltf_parse_error(
            "gltf decoded buffer count exceeds its cumulative limit",
        ));
    }
    // Reject declared aggregate growth before reading any sources or allocating fallback storage.
    let mut declared = 0_u64;
    for buffer in document.buffers() {
        declared = declared
            .checked_add(padded_length(buffer.length())? as u64)
            .ok_or_else(|| gltf_parse_error("gltf decoded buffer budget overflow"))?;
        budget.check(declared, "buffer")?;
    }
    let mut buffers = Vec::with_capacity(document.buffers().len());
    for buffer in document.buffers() {
        let data = if buffer_is_meshopt_fallback(&buffer)? {
            let length = padded_length(buffer.length())?;
            budget.charge(length as u64, "buffer")?;
            allocate_padded(&[], length)?
        } else {
            match buffer.source() {
                gltf::buffer::Source::Uri(uri) if !is_data_uri(uri) => {
                    let remaining = budget.remaining() / GLTF_BUFFER_ALIGNMENT as u64
                        * GLTF_BUFFER_ALIGNMENT as u64;
                    let (_, bytes) = sources.read(uri, remaining).map_err(|error| {
                        gltf_parse_error(format!(
                            "gltf decoded buffer budget/source admission: {error}"
                        ))
                    })?;
                    let length = padded_length(bytes.len())?;
                    budget.charge(length as u64, "buffer")?;
                    allocate_padded(bytes, length)?
                }
                gltf::buffer::Source::Uri(uri) => {
                    let bytes = decode_data_uri(uri, budget.remaining())?;
                    budget.charge(bytes.len() as u64, "buffer")?;
                    bytes
                }
                gltf::buffer::Source::Bin => {
                    let mut bytes = blob.take().ok_or_else(|| {
                        gltf_parse_error(format!(
                            "load gltf Buffer{} from {}: missing binary chunk",
                            buffer.index(),
                            gltf_buffer_source_name(buffer.source())
                        ))
                    })?;
                    let length = padded_length(bytes.len())?;
                    budget.charge(length as u64, "buffer")?;
                    bytes
                        .try_reserve_exact(length - bytes.len())
                        .map_err(|error| {
                            gltf_parse_error(format!("allocate gltf binary buffer: {error}"))
                        })?;
                    bytes.resize(length, 0);
                    bytes
                }
            }
        };
        if data.len() < buffer.length() {
            return Err(gltf_parse_error(format!(
                "gltf Buffer{} declares {} bytes but its source contains {}",
                buffer.index(),
                buffer.length(),
                data.len()
            )));
        }
        buffers.push(gltf::buffer::Data(data));
    }
    Ok(buffers)
}

pub(super) fn decode_data_uri(uri: &str, limit: u64) -> Result<Vec<u8>, AssetImportError> {
    let rest = uri
        .strip_prefix("data:")
        .ok_or_else(|| gltf_parse_error("unsupported gltf data URI scheme"))?;
    // Keep gltf's payload and optional-padding semantics while owning the bounded output allocation.
    let payload = rest.split(";base64,").nth(1).unwrap_or(rest);
    let padding = payload
        .bytes()
        .rev()
        .take_while(|byte| *byte == b'=')
        .take(2)
        .count();
    let decoded_length = payload
        .len()
        .checked_mul(3)
        .map(|length| length / 4)
        .and_then(|length| length.checked_sub(padding))
        .ok_or_else(|| gltf_parse_error("gltf data URI length overflow"))?;
    let padded = padded_length(decoded_length)?;
    if padded as u64 > limit {
        return Err(gltf_parse_error(
            "gltf decoded buffer budget exceeded by embedded data URI",
        ));
    }
    let mut data = allocate_padded(&[], padded)?;
    let decoded = GLTF_BASE64
        .decode_slice(payload, &mut data)
        .map_err(|error| gltf_parse_error(format!("decode gltf data URI: {error}")))?;
    data.truncate(padded_length(decoded)?);
    Ok(data)
}

fn padded_length(length: usize) -> Result<usize, AssetImportError> {
    length
        .checked_add(GLTF_BUFFER_ALIGNMENT - 1)
        .map(|length| length / GLTF_BUFFER_ALIGNMENT * GLTF_BUFFER_ALIGNMENT)
        .ok_or_else(|| gltf_parse_error("gltf decoded buffer budget overflow"))
}

fn allocate_padded(bytes: &[u8], length: usize) -> Result<Vec<u8>, AssetImportError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(length)
        .map_err(|error| gltf_parse_error(format!("allocate gltf buffer: {error}")))?;
    output.extend_from_slice(bytes);
    output.resize(length, 0);
    Ok(output)
}
use base64::engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};
use base64::Engine;
