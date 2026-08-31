use super::super::{RenderArtifactBlockCodec, RenderArtifactBlockDescriptor};
use super::contract::{
    RenderArtifactBlockAdmissionError, RenderArtifactBlockLoaderInitError,
    RenderArtifactBlockLoaderLimits,
};

const ENTRY_METADATA_BYTES: usize = 256;

pub(super) fn quote_retained_bytes(
    descriptor: &RenderArtifactBlockDescriptor,
    limits: RenderArtifactBlockLoaderLimits,
) -> Result<usize, RenderArtifactBlockAdmissionError> {
    if descriptor.encoded_bytes() == 0 || descriptor.decoded_bytes() == 0 {
        return Err(RenderArtifactBlockAdmissionError::InvalidBlockDescriptor {
            reason: "encoded and decoded byte counts must be non-zero",
        });
    }
    if descriptor.codec() == RenderArtifactBlockCodec::Raw
        && descriptor.encoded_bytes() != descriptor.decoded_bytes()
    {
        return Err(RenderArtifactBlockAdmissionError::InvalidBlockDescriptor {
            reason: "raw block encoded and decoded byte counts must match",
        });
    }
    if descriptor.encoded_bytes() > limits.store_limits().max_encoded_block_bytes() {
        return Err(
            RenderArtifactBlockAdmissionError::EncodedBlockLimitExceeded {
                actual: descriptor.encoded_bytes(),
                limit: limits.store_limits().max_encoded_block_bytes(),
            },
        );
    }
    if descriptor.decoded_bytes() > limits.max_decoded_block_bytes() {
        return Err(
            RenderArtifactBlockAdmissionError::DecodedBlockLimitExceeded {
                actual: descriptor.decoded_bytes(),
                limit: limits.max_decoded_block_bytes(),
            },
        );
    }
    let encoded = usize::try_from(descriptor.encoded_bytes())
        .map_err(|_| RenderArtifactBlockAdmissionError::RetainedBytesOverflow)?;
    let decoded = usize::try_from(descriptor.decoded_bytes())
        .map_err(|_| RenderArtifactBlockAdmissionError::RetainedBytesOverflow)?;
    encoded
        .checked_add(decoded)
        .and_then(|bytes| bytes.checked_add(ENTRY_METADATA_BYTES))
        .ok_or(RenderArtifactBlockAdmissionError::RetainedBytesOverflow)
}

pub(super) fn validate_limits(
    limits: RenderArtifactBlockLoaderLimits,
) -> Result<(), RenderArtifactBlockLoaderInitError> {
    for (name, value) in [
        ("max_entries", limits.max_entries()),
        ("max_total_tickets", limits.max_total_tickets()),
        ("max_tickets_per_entry", limits.max_tickets_per_entry()),
        ("max_retained_bytes", limits.max_retained_bytes()),
    ] {
        if value == 0 {
            return Err(RenderArtifactBlockLoaderInitError::ZeroLimit { limit: name });
        }
    }
    for (name, value) in [
        ("max_decoded_block_bytes", limits.max_decoded_block_bytes()),
        (
            "max_manifest_bytes",
            limits.store_limits().max_manifest_bytes(),
        ),
        (
            "max_encoded_block_bytes",
            limits.store_limits().max_encoded_block_bytes(),
        ),
    ] {
        if value == 0 {
            return Err(RenderArtifactBlockLoaderInitError::ZeroLimit { limit: name });
        }
    }
    Ok(())
}
