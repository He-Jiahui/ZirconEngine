//! IBL 缓存命中直接回填已烘焙资源；未命中则为本帧图编译保留唯一烘焙请求及失败重试权。
use std::sync::{Arc, Mutex};

use crate::asset::artifact::resolve_ibl_bake_artifact_runtime_dispatch;
use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    source_cubemap_environment_with_bake_artifact, IblBakeArtifactContents, RenderFrameExtract,
    RenderFrameworkError, SourceCubemapEnvironment,
};
use crate::graphics::runtime::render_framework::render_framework_state::EnvironmentIblHydrationCache;
use crate::graphics::RenderPipelineCompileOptions;

enum EnvironmentIblCacheResolutionKind {
    RuntimeComputeRequired(crate::graphics::EnvironmentIblBakeReservation),
    RuntimeBakePending,
    Hydrated,
    RuntimeBakeUnavailable,
}

pub(super) struct EnvironmentIblCacheResolution {
    kind: EnvironmentIblCacheResolutionKind,
    hydrated_source_cubemap: Option<SourceCubemapEnvironment>,
}

impl EnvironmentIblCacheResolution {
    fn from_kind(kind: EnvironmentIblCacheResolutionKind) -> Self {
        Self {
            kind,
            hydrated_source_cubemap: None,
        }
    }

    fn hydrated(source_cubemap: Option<SourceCubemapEnvironment>) -> Self {
        Self {
            kind: EnvironmentIblCacheResolutionKind::Hydrated,
            hydrated_source_cubemap: source_cubemap,
        }
    }

    const fn requires_runtime_compute(&self) -> bool {
        matches!(
            self.kind,
            EnvironmentIblCacheResolutionKind::RuntimeComputeRequired(_)
        )
    }

    pub(super) fn into_submission_parts(
        self,
    ) -> (
        Option<crate::graphics::EnvironmentIblBakeReservation>,
        Option<SourceCubemapEnvironment>,
    ) {
        let reservation = match self.kind {
            EnvironmentIblCacheResolutionKind::RuntimeComputeRequired(reservation) => {
                Some(reservation)
            }
            EnvironmentIblCacheResolutionKind::RuntimeBakePending
            | EnvironmentIblCacheResolutionKind::Hydrated
            | EnvironmentIblCacheResolutionKind::RuntimeBakeUnavailable => None,
        };
        (reservation, self.hydrated_source_cubemap)
    }

    #[cfg(test)]
    fn hydrated_source_cubemap(&self) -> Option<&SourceCubemapEnvironment> {
        self.hydrated_source_cubemap.as_ref()
    }
}

pub(super) fn resolve_and_rehydrate_environment_ibl_cache(
    asset_manager: &ProjectAssetManager,
    hydration_cache: &Arc<Mutex<EnvironmentIblHydrationCache>>,
    extract: &RenderFrameExtract,
) -> Result<Option<EnvironmentIblCacheResolution>, RenderFrameworkError> {
    let Some(request) = extract
        .environment
        .source_cubemap_ibl_bake_request(IblBakeArtifactContents::PMREM_SH9)
    else {
        return Ok(None);
    };
    let Some(environment) = extract.environment.skybox.source_cubemap.as_ref() else {
        return Ok(None);
    };
    if environment
        .accepted_bake_artifact_descriptor()
        .is_some_and(|descriptor| descriptor.is_current_for(&request))
    {
        hydration_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear_pending_runtime_bake(&request);
        return Ok(Some(EnvironmentIblCacheResolution::hydrated(None)));
    }
    let cached = hydration_cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(&request, environment);
    if let Some(cached) = cached {
        return Ok(Some(EnvironmentIblCacheResolution::hydrated(Some(cached))));
    }
    let Some(store) = asset_manager.ibl_bake_artifact_cache_store() else {
        return Ok(Some(EnvironmentIblCacheResolution::from_kind(
            EnvironmentIblCacheResolutionKind::RuntimeBakeUnavailable,
        )));
    };
    let dispatch = resolve_ibl_bake_artifact_runtime_dispatch(&store, &request, &[])
        .map_err(|error| RenderFrameworkError::Backend(error.to_string()))?;
    let Some(payload) = dispatch.payload() else {
        let resolution =
            EnvironmentIblHydrationCache::reserve_runtime_bake(hydration_cache, request)
                .map(EnvironmentIblCacheResolutionKind::RuntimeComputeRequired)
                .unwrap_or(EnvironmentIblCacheResolutionKind::RuntimeBakePending);
        return Ok(Some(EnvironmentIblCacheResolution::from_kind(resolution)));
    };
    let hydrated =
        source_cubemap_environment_with_bake_artifact(environment, payload).map_err(|error| {
            RenderFrameworkError::Backend(format!("rehydrate environment IBL artifact: {error:?}"))
        })?;
    hydration_cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(request, hydrated.clone());
    Ok(Some(EnvironmentIblCacheResolution::hydrated(Some(
        hydrated,
    ))))
}

pub(super) fn compile_options_with_environment_ibl_bake_request(
    extract: &RenderFrameExtract,
    options: RenderPipelineCompileOptions,
    resolution: Option<&EnvironmentIblCacheResolution>,
) -> Result<RenderPipelineCompileOptions, RenderFrameworkError> {
    let Some(request) = extract
        .environment
        .source_cubemap_ibl_bake_request(IblBakeArtifactContents::PMREM_SH9)
    else {
        return Ok(options.without_environment_ibl_bake_request());
    };
    let Some(resolution) = resolution else {
        return Ok(options.without_environment_ibl_bake_request());
    };
    if resolution.requires_runtime_compute() {
        Ok(options.with_environment_ibl_bake_request(request))
    } else {
        Ok(options.without_environment_ibl_bake_request())
    }
}

#[cfg(test)]
#[path = "tests/environment_ibl_compile_options.rs"]
mod tests;
