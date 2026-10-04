use std::fmt;

use super::{
    source_cubemap_mip_count, source_cubemap_mip_size, RenderEnvironmentCaptureHandle,
    RenderEnvironmentCaptureOutputIdentity, RGBA16F_TEXEL_SIZE_BYTES, SOURCE_CUBEMAP_FACE_COUNT,
};

/// Consuming CPU result for an explicitly persisted environment capture.
///
/// This payload is deliberately not `Clone`: a 1024 source pyramid is about 64 MiB and must move
/// from the renderer to the asset/editor owner exactly once.
pub struct RenderEnvironmentCaptureSourcePayload {
    handle: RenderEnvironmentCaptureHandle,
    output: RenderEnvironmentCaptureOutputIdentity,
    face_size: u32,
    mip_count: u32,
    source_rgba16f: Vec<u8>,
}

impl RenderEnvironmentCaptureSourcePayload {
    /// 仅接受带持久化目标、完整 mip 金字塔及精确字节数的捕获结果；
    /// 渲染器读回后将所有权一次性交给资产或编辑器持久化端。
    pub fn new(
        handle: RenderEnvironmentCaptureHandle,
        output: RenderEnvironmentCaptureOutputIdentity,
        face_size: u32,
        mip_count: u32,
        source_rgba16f: Vec<u8>,
    ) -> Result<Self, RenderEnvironmentCaptureSourcePayloadError> {
        if output.persistence_output_uri().is_none() {
            return Err(RenderEnvironmentCaptureSourcePayloadError::MissingPersistenceTarget);
        }
        let expected_mip_count = source_cubemap_mip_count(face_size);
        if face_size == 0 || mip_count != expected_mip_count {
            return Err(RenderEnvironmentCaptureSourcePayloadError::InvalidLayout {
                face_size,
                expected_mip_count,
                actual_mip_count: mip_count,
            });
        }
        let expected_bytes = checked_source_payload_byte_len(face_size, mip_count).ok_or(
            RenderEnvironmentCaptureSourcePayloadError::ExtentTooLarge {
                face_size,
                mip_count,
            },
        )?;
        if source_rgba16f.len() != expected_bytes {
            return Err(
                RenderEnvironmentCaptureSourcePayloadError::PayloadLengthMismatch {
                    expected: expected_bytes,
                    actual: source_rgba16f.len(),
                },
            );
        }
        Ok(Self {
            handle,
            output,
            face_size,
            mip_count,
            source_rgba16f,
        })
    }

    pub const fn handle(&self) -> RenderEnvironmentCaptureHandle {
        self.handle
    }

    pub const fn output(&self) -> &RenderEnvironmentCaptureOutputIdentity {
        &self.output
    }

    pub const fn face_size(&self) -> u32 {
        self.face_size
    }

    pub const fn mip_count(&self) -> u32 {
        self.mip_count
    }

    pub fn source_rgba16f_bytes(&self) -> &[u8] {
        &self.source_rgba16f
    }

    pub fn into_source_rgba16f_bytes(self) -> Vec<u8> {
        self.source_rgba16f
    }
}

impl fmt::Debug for RenderEnvironmentCaptureSourcePayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RenderEnvironmentCaptureSourcePayload")
            .field("handle", &self.handle)
            .field("output", &self.output)
            .field("face_size", &self.face_size)
            .field("mip_count", &self.mip_count)
            .field("source_rgba16f_bytes", &self.source_rgba16f.len())
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderEnvironmentCaptureSourcePayloadError {
    MissingPersistenceTarget,
    InvalidLayout {
        face_size: u32,
        expected_mip_count: u32,
        actual_mip_count: u32,
    },
    ExtentTooLarge {
        face_size: u32,
        mip_count: u32,
    },
    PayloadLengthMismatch {
        expected: usize,
        actual: usize,
    },
}

impl fmt::Display for RenderEnvironmentCaptureSourcePayloadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPersistenceTarget => write!(
                formatter,
                "environment capture source payload has no persistence target"
            ),
            Self::InvalidLayout {
                face_size,
                expected_mip_count,
                actual_mip_count,
            } => write!(
                formatter,
                "invalid environment capture source layout face_size={face_size}, expected mip_count={expected_mip_count}, found {actual_mip_count}"
            ),
            Self::ExtentTooLarge {
                face_size,
                mip_count,
            } => write!(
                formatter,
                "environment capture source layout is too large: face_size={face_size}, mip_count={mip_count}"
            ),
            Self::PayloadLengthMismatch { expected, actual } => write!(
                formatter,
                "environment capture source payload length mismatch: expected {expected} bytes, found {actual}"
            ),
        }
    }
}

impl std::error::Error for RenderEnvironmentCaptureSourcePayloadError {}

fn checked_source_payload_byte_len(face_size: u32, mip_count: u32) -> Option<usize> {
    let mut per_face = 0_usize;
    for mip_level in 0..mip_count {
        let mip_size = usize::try_from(source_cubemap_mip_size(face_size, mip_level)).ok()?;
        per_face = per_face.checked_add(mip_size.checked_mul(mip_size)?)?;
    }
    per_face
        .checked_mul(SOURCE_CUBEMAP_FACE_COUNT)?
        .checked_mul(RGBA16F_TEXEL_SIZE_BYTES)
}

#[cfg(test)]
#[path = "tests/environment_capture_source_payload.rs"]
mod tests;
