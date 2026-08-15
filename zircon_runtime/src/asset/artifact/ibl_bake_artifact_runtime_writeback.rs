use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::core::framework::render::{
    IblBakeArtifactBlob, IblBakeArtifactDescriptor, IblBakeArtifactReadbackError,
    IblBakeArtifactReadbackSections, IblBakeArtifactRequest,
};

use super::{IblBakeArtifactCacheError, IblBakeArtifactCacheStore};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IblBakeArtifactRuntimeWritebackStatus {
    Written,
    SkippedDescriptorNotCurrent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IblBakeArtifactRuntimeWritebackReport {
    status: IblBakeArtifactRuntimeWritebackStatus,
    descriptor: IblBakeArtifactDescriptor,
    path: Option<PathBuf>,
    encoded_len: usize,
    payload_len: usize,
}

impl IblBakeArtifactRuntimeWritebackReport {
    pub const fn status(&self) -> IblBakeArtifactRuntimeWritebackStatus {
        self.status
    }

    pub const fn descriptor(&self) -> IblBakeArtifactDescriptor {
        self.descriptor
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub const fn encoded_len(&self) -> usize {
        self.encoded_len
    }

    pub const fn payload_len(&self) -> usize {
        self.payload_len
    }

    pub const fn wrote_cache(&self) -> bool {
        matches!(self.status, IblBakeArtifactRuntimeWritebackStatus::Written)
    }
}

pub fn write_ibl_bake_artifact_runtime_readback(
    store: &IblBakeArtifactCacheStore,
    request: &IblBakeArtifactRequest,
    readback: IblBakeArtifactReadbackSections,
) -> Result<IblBakeArtifactRuntimeWritebackReport, IblBakeArtifactRuntimeWritebackError> {
    let descriptor = readback.descriptor();
    if !descriptor.is_current_runtime_cache_for(request) {
        return Ok(IblBakeArtifactRuntimeWritebackReport {
            status: IblBakeArtifactRuntimeWritebackStatus::SkippedDescriptorNotCurrent,
            descriptor,
            path: None,
            encoded_len: 0,
            payload_len: 0,
        });
    }

    let payload = readback
        .into_payload()
        .map_err(IblBakeArtifactRuntimeWritebackError::Readback)?;
    let payload_len = payload.bytes().len();
    let blob = IblBakeArtifactBlob::from_payload(payload);
    let encoded_len = blob.encoded_len();
    let path = store
        .write_runtime_cache(&blob)
        .map_err(IblBakeArtifactRuntimeWritebackError::Cache)?;

    Ok(IblBakeArtifactRuntimeWritebackReport {
        status: IblBakeArtifactRuntimeWritebackStatus::Written,
        descriptor,
        path: Some(path),
        encoded_len,
        payload_len,
    })
}

#[derive(Debug, Error)]
pub enum IblBakeArtifactRuntimeWritebackError {
    #[error("assemble IBL bake artifact readback sections: {0:?}")]
    Readback(IblBakeArtifactReadbackError),
    #[error("write IBL bake artifact runtime cache: {0}")]
    Cache(IblBakeArtifactCacheError),
}
