use thiserror::Error;

use crate::core::framework::render::{
    IblBakeArtifactBlob, IblBakeArtifactPayload, IblBakeArtifactReadbackSections,
    IblBakeArtifactRequest, IblBakeArtifactResolvedPayload, IblBakeArtifactSource,
};

use super::{
    write_ibl_bake_artifact_runtime_readback, IblBakeArtifactCacheError, IblBakeArtifactCacheRead,
    IblBakeArtifactCacheStore, IblBakeArtifactRuntimeWritebackError,
    IblBakeArtifactRuntimeWritebackReport, IblBakeArtifactRuntimeWritebackStatus,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IblBakeArtifactRuntimeDispatchReport {
    cache_read: IblBakeArtifactCacheRead,
    resolved: IblBakeArtifactResolvedPayload,
}

impl IblBakeArtifactRuntimeDispatchReport {
    pub const fn source(&self) -> IblBakeArtifactSource {
        self.resolved.source()
    }

    pub fn cache_read(&self) -> &IblBakeArtifactCacheRead {
        &self.cache_read
    }

    pub fn resolved(&self) -> &IblBakeArtifactResolvedPayload {
        &self.resolved
    }

    pub fn payload(&self) -> Option<&IblBakeArtifactPayload> {
        self.resolved.payload()
    }

    pub const fn rejected_candidate_count(&self) -> usize {
        self.resolved.rejected_candidate_count()
    }

    pub const fn environment_compute_dispatch_count(&self) -> u32 {
        self.resolved.environment_compute_dispatch_count()
    }

    pub const fn requires_runtime_compute(&self) -> bool {
        self.resolved.requires_runtime_compute()
    }
}

/// 先读取与请求描述符绑定的运行时缓存，再把资产派生候选和缓存命中交给解析器；报告保留读取结果，供渲染调用方区分命中、拒绝和运行时计算。
pub fn resolve_ibl_bake_artifact_runtime_dispatch(
    store: &IblBakeArtifactCacheStore,
    request: &IblBakeArtifactRequest,
    asset_derived_blobs: &[IblBakeArtifactBlob],
) -> Result<IblBakeArtifactRuntimeDispatchReport, IblBakeArtifactRuntimeDispatchError> {
    let cache_read = store
        .read_runtime_cache(request)
        .map_err(IblBakeArtifactRuntimeDispatchError::Cache)?;
    let runtime_cache_blob = match &cache_read {
        IblBakeArtifactCacheRead::Hit(blob) => Some(blob),
        IblBakeArtifactCacheRead::Missing | IblBakeArtifactCacheRead::Rejected(_) => None,
    };
    let resolved = IblBakeArtifactResolvedPayload::resolve_borrowed_blob_sources(
        request,
        asset_derived_blobs,
        runtime_cache_blob,
    );
    Ok(IblBakeArtifactRuntimeDispatchReport {
        cache_read,
        resolved,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IblBakeArtifactRuntimeDispatchReadbackStatus {
    Written,
    SkippedRuntimeComputeNotRequired,
    SkippedDescriptorNotCurrent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IblBakeArtifactRuntimeDispatchReadbackReport {
    status: IblBakeArtifactRuntimeDispatchReadbackStatus,
    writeback: Option<IblBakeArtifactRuntimeWritebackReport>,
}

impl IblBakeArtifactRuntimeDispatchReadbackReport {
    pub const fn status(&self) -> IblBakeArtifactRuntimeDispatchReadbackStatus {
        self.status
    }

    pub fn writeback(&self) -> Option<&IblBakeArtifactRuntimeWritebackReport> {
        self.writeback.as_ref()
    }

    pub const fn wrote_cache(&self) -> bool {
        matches!(
            self.status,
            IblBakeArtifactRuntimeDispatchReadbackStatus::Written
        )
    }
}

/// 只有解析结果声明需要运行时计算时才写回 readback；写回层仍会检查描述符是否为当前版本，并把跳过原因映射成可观察状态。
pub fn write_ibl_bake_artifact_runtime_dispatch_readback(
    store: &IblBakeArtifactCacheStore,
    request: &IblBakeArtifactRequest,
    dispatch: &IblBakeArtifactRuntimeDispatchReport,
    readback: IblBakeArtifactReadbackSections,
) -> Result<IblBakeArtifactRuntimeDispatchReadbackReport, IblBakeArtifactRuntimeDispatchError> {
    if !dispatch.requires_runtime_compute() {
        return Ok(IblBakeArtifactRuntimeDispatchReadbackReport {
            status: IblBakeArtifactRuntimeDispatchReadbackStatus::SkippedRuntimeComputeNotRequired,
            writeback: None,
        });
    }

    let writeback = write_ibl_bake_artifact_runtime_readback(store, request, readback)
        .map_err(IblBakeArtifactRuntimeDispatchError::Writeback)?;
    let status = match writeback.status() {
        IblBakeArtifactRuntimeWritebackStatus::Written => {
            IblBakeArtifactRuntimeDispatchReadbackStatus::Written
        }
        IblBakeArtifactRuntimeWritebackStatus::SkippedDescriptorNotCurrent => {
            IblBakeArtifactRuntimeDispatchReadbackStatus::SkippedDescriptorNotCurrent
        }
    };

    Ok(IblBakeArtifactRuntimeDispatchReadbackReport {
        status,
        writeback: Some(writeback),
    })
}

#[derive(Debug, Error)]
pub enum IblBakeArtifactRuntimeDispatchError {
    #[error("read IBL bake artifact runtime cache: {0}")]
    Cache(IblBakeArtifactCacheError),
    #[error("write IBL bake artifact runtime readback: {0}")]
    Writeback(IblBakeArtifactRuntimeWritebackError),
}
