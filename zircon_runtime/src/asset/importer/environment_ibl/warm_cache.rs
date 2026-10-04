use std::path::Path;
use std::time::Instant;

use super::{
    decode_texture_source_image_metadata, environment_ibl_import_mode,
    environment_ibl_request_for_source_image, source_image_identity, AssetImportContext,
    EnvironmentIblImportMode, EnvironmentIblSourceStagingError, EnvironmentIblSourceStagingOutput,
    EnvironmentIblSourceStagingReport, EnvironmentIblSourceStagingStatus,
    EnvironmentIblSourceStagingTiming, EnvironmentIblStagingPhase, IblSourceCubemapStagingStore,
    PreparedEnvironmentIblSourceStaging,
};

pub(super) enum EnvironmentIblWarmCacheProbe {
    Finished(PreparedEnvironmentIblSourceStaging),
    Miss {
        source_image: crate::asset::artifact::IblSourceImageIdentity,
        timing: EnvironmentIblSourceStagingTiming,
    },
}

// 先读解码格式和尺寸以构造与冷路径相同的请求，再用 manifest 与文件元数据作快速命中判断；
// 此入口不读取 payload，完整内容恢复由 artifact store 的读取路径负责。
pub(super) fn probe_environment_ibl_warm_cache(
    context: &AssetImportContext,
    cache_root: &Path,
) -> Result<EnvironmentIblWarmCacheProbe, EnvironmentIblSourceStagingError> {
    let classify_started = Instant::now();
    let mode = {
        let _phase = EnvironmentIblStagingPhase::SourceClassify.enter();
        environment_ibl_import_mode(context)?
    };
    let source_classify = classify_started.elapsed();
    let store = IblSourceCubemapStagingStore::new(cache_root);
    if mode == EnvironmentIblImportMode::Disabled || !mode.applies_to(context) {
        return Ok(EnvironmentIblWarmCacheProbe::Finished(
            PreparedEnvironmentIblSourceStaging {
                store,
                writes: Vec::new(),
                report: EnvironmentIblSourceStagingReport::skipped(),
            },
        ));
    }

    let metadata_started = Instant::now();
    let metadata = {
        let _phase = EnvironmentIblStagingPhase::SourceDecode.enter();
        decode_texture_source_image_metadata(context)
            .map_err(EnvironmentIblSourceStagingError::Decode)?
    };
    let source_image = source_image_identity(metadata);
    let mut timing = EnvironmentIblSourceStagingTiming {
        source_classify,
        source_decode: metadata_started.elapsed(),
        ..Default::default()
    };
    let identity_started = Instant::now();
    let request = {
        let _phase = EnvironmentIblStagingPhase::SourceIdentity.enter();
        environment_ibl_request_for_source_image(context, source_image)?
    };
    timing.source_identity = identity_started.elapsed();
    let Some(request) = request else {
        return Ok(EnvironmentIblWarmCacheProbe::Finished(
            PreparedEnvironmentIblSourceStaging {
                store,
                writes: Vec::new(),
                report: EnvironmentIblSourceStagingReport::skipped(),
            },
        ));
    };

    let source_path = store.source_cubemap_path(&request);
    let derived_path = store.asset_derived_store().asset_derived_path(&request);
    let probe_started = Instant::now();
    let current = {
        let _phase = EnvironmentIblStagingPhase::CacheProbe.enter();
        store.current_bundle_manifest_matches(&request, source_image)?
    };
    timing.cache_probe = probe_started.elapsed();
    if !current {
        return Ok(EnvironmentIblWarmCacheProbe::Miss {
            source_image,
            timing,
        });
    }

    let output = EnvironmentIblSourceStagingOutput::from_reused_paths(&source_path, &derived_path)?;
    Ok(EnvironmentIblWarmCacheProbe::Finished(
        PreparedEnvironmentIblSourceStaging {
            store,
            writes: Vec::new(),
            report: EnvironmentIblSourceStagingReport::current(
                EnvironmentIblSourceStagingStatus::Reused,
                request,
                source_path,
                derived_path,
                timing,
                output,
            ),
        },
    ))
}

#[cfg(test)]
#[path = "tests/warm_cache.rs"]
mod tests;
