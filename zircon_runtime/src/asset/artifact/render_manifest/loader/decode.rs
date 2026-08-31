use std::io::{self, Read};
use std::sync::Arc;

use super::contract::{RenderArtifactBlockFailure, RenderArtifactBlockFailureCode};

pub(super) fn decode_zstd_block(
    encoded: &Arc<[u8]>,
    expected_bytes: u64,
) -> Result<Arc<[u8]>, RenderArtifactBlockFailure> {
    let capacity = usize::try_from(expected_bytes).map_err(|_| {
        RenderArtifactBlockFailure::new(
            RenderArtifactBlockFailureCode::DecodedSizeMismatch,
            "decoded block size does not fit this address space",
        )
    })?;
    let decoder = zstd::stream::read::Decoder::new(encoded.as_ref()).map_err(decode_failure)?;
    let mut decoded = Vec::with_capacity(capacity);
    decoder
        .take(expected_bytes.saturating_add(1))
        .read_to_end(&mut decoded)
        .map_err(decode_failure)?;
    let actual_bytes = u64::try_from(decoded.len()).map_err(|_| {
        RenderArtifactBlockFailure::new(
            RenderArtifactBlockFailureCode::DecodedSizeMismatch,
            "decoded block size does not fit the artifact contract",
        )
    })?;
    if actual_bytes != expected_bytes {
        return Err(RenderArtifactBlockFailure::new(
            RenderArtifactBlockFailureCode::DecodedSizeMismatch,
            format!("expected {expected_bytes} decoded bytes, received {actual_bytes}"),
        ));
    }
    Ok(decoded.into())
}

fn decode_failure(error: io::Error) -> RenderArtifactBlockFailure {
    RenderArtifactBlockFailure::new(
        RenderArtifactBlockFailureCode::DecodeFailed,
        error.to_string(),
    )
}
