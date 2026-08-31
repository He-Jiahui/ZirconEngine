use super::{AssetImportContext, EnvironmentIblSourceStagingError};
use crate::core::framework::render::{
    source_cubemap_mip_count, IblBakeArtifactContents, SOURCE_CUBEMAP_MAX_FACE_SIZE,
    SOURCE_CUBEMAP_MIN_FACE_SIZE, SOURCE_CUBEMAP_PMREM_FACE_SIZE, SOURCE_CUBEMAP_PMREM_MIP_COUNT,
};

pub const ENVIRONMENT_IBL_IMPORT_SETTING: &str = "environment_ibl";
pub const ENVIRONMENT_IBL_FACE_SIZE_IMPORT_SETTING: &str = "environment_ibl_face_size";
pub const ENVIRONMENT_IBL_PMREM_FACE_SIZE_IMPORT_SETTING: &str = "environment_ibl_pmrem_face_size";
pub const ENVIRONMENT_IBL_IRRADIANCE_CUBE_IMPORT_SETTING: &str = "environment_ibl_irradiance_cube";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EnvironmentIblImportMode {
    Disabled,
    Automatic,
    Enabled,
}

impl EnvironmentIblImportMode {
    pub(super) fn applies_to(self, context: &AssetImportContext) -> bool {
        if self == Self::Enabled {
            return true;
        }
        context
            .source_path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(is_environment_ibl_extension)
    }
}

fn is_environment_ibl_extension(extension: &str) -> bool {
    extension.eq_ignore_ascii_case("hdr") || extension.eq_ignore_ascii_case("exr")
}

pub(super) fn environment_ibl_import_mode(
    context: &AssetImportContext,
) -> Result<EnvironmentIblImportMode, EnvironmentIblSourceStagingError> {
    let Some(value) = context.import_settings.get(ENVIRONMENT_IBL_IMPORT_SETTING) else {
        return Ok(EnvironmentIblImportMode::Automatic);
    };
    if let Some(enabled) = value.as_bool() {
        return Ok(if enabled {
            EnvironmentIblImportMode::Enabled
        } else {
            EnvironmentIblImportMode::Disabled
        });
    }
    if value
        .as_str()
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("auto"))
    {
        return Ok(EnvironmentIblImportMode::Automatic);
    }
    Err(EnvironmentIblSourceStagingError::InvalidSetting {
        key: ENVIRONMENT_IBL_IMPORT_SETTING,
        reason: "expected true, false, or \"auto\"".to_string(),
    })
}

pub(super) fn requested_artifact_contents(
    context: &AssetImportContext,
) -> Result<IblBakeArtifactContents, EnvironmentIblSourceStagingError> {
    requested_artifact_contents_from_value(
        context
            .import_settings
            .get(ENVIRONMENT_IBL_IRRADIANCE_CUBE_IMPORT_SETTING),
    )
}

pub(super) fn requested_artifact_contents_from_value(
    value: Option<&toml::Value>,
) -> Result<IblBakeArtifactContents, EnvironmentIblSourceStagingError> {
    let Some(value) = value else {
        return Ok(IblBakeArtifactContents::PMREM_SH9);
    };
    let Some(enabled) = value.as_bool() else {
        return Err(EnvironmentIblSourceStagingError::InvalidSetting {
            key: ENVIRONMENT_IBL_IRRADIANCE_CUBE_IMPORT_SETTING,
            reason: "expected true or false".to_string(),
        });
    };
    Ok(if enabled {
        IblBakeArtifactContents::PMREM_SH9_IEM
    } else {
        IblBakeArtifactContents::PMREM_SH9
    })
}

pub(super) fn requested_face_size(
    context: &AssetImportContext,
    natural_face_size: u32,
) -> Result<u32, EnvironmentIblSourceStagingError> {
    let Some(value) = context
        .import_settings
        .get(ENVIRONMENT_IBL_FACE_SIZE_IMPORT_SETTING)
    else {
        return Ok(natural_face_size);
    };
    let Some(value) = value.as_integer() else {
        return Err(EnvironmentIblSourceStagingError::InvalidSetting {
            key: ENVIRONMENT_IBL_FACE_SIZE_IMPORT_SETTING,
            reason: "expected an integer power-of-two face size".to_string(),
        });
    };
    let face_size = u32::try_from(value).map_err(|_| {
        EnvironmentIblSourceStagingError::InvalidSetting {
            key: ENVIRONMENT_IBL_FACE_SIZE_IMPORT_SETTING,
            reason: format!(
                "face size must be in {SOURCE_CUBEMAP_MIN_FACE_SIZE}..={SOURCE_CUBEMAP_MAX_FACE_SIZE}"
            ),
        }
    })?;
    if !face_size.is_power_of_two()
        || !(SOURCE_CUBEMAP_MIN_FACE_SIZE..=SOURCE_CUBEMAP_MAX_FACE_SIZE).contains(&face_size)
    {
        return Err(EnvironmentIblSourceStagingError::InvalidSetting {
            key: ENVIRONMENT_IBL_FACE_SIZE_IMPORT_SETTING,
            reason: format!(
                "face size must be a power of two in {SOURCE_CUBEMAP_MIN_FACE_SIZE}..={SOURCE_CUBEMAP_MAX_FACE_SIZE}"
            ),
        });
    }
    Ok(face_size.min(natural_face_size))
}

pub(super) fn requested_pmrem_layout(
    context: &AssetImportContext,
    source_face_size: u32,
) -> Result<(u32, u32), EnvironmentIblSourceStagingError> {
    let Some(value) = context
        .import_settings
        .get(ENVIRONMENT_IBL_PMREM_FACE_SIZE_IMPORT_SETTING)
    else {
        return Ok((
            SOURCE_CUBEMAP_PMREM_FACE_SIZE,
            SOURCE_CUBEMAP_PMREM_MIP_COUNT,
        ));
    };
    let Some(value) = value.as_integer() else {
        return Err(EnvironmentIblSourceStagingError::InvalidSetting {
            key: ENVIRONMENT_IBL_PMREM_FACE_SIZE_IMPORT_SETTING,
            reason: "expected an integer power-of-two face size".to_string(),
        });
    };
    let face_size = u32::try_from(value).map_err(|_| {
        EnvironmentIblSourceStagingError::InvalidSetting {
            key: ENVIRONMENT_IBL_PMREM_FACE_SIZE_IMPORT_SETTING,
            reason: format!(
                "face size must be in {SOURCE_CUBEMAP_MIN_FACE_SIZE}..={SOURCE_CUBEMAP_MAX_FACE_SIZE}"
            ),
        }
    })?;
    if !face_size.is_power_of_two()
        || !(SOURCE_CUBEMAP_MIN_FACE_SIZE..=SOURCE_CUBEMAP_MAX_FACE_SIZE).contains(&face_size)
    {
        return Err(EnvironmentIblSourceStagingError::InvalidSetting {
            key: ENVIRONMENT_IBL_PMREM_FACE_SIZE_IMPORT_SETTING,
            reason: format!(
                "face size must be a power of two in {SOURCE_CUBEMAP_MIN_FACE_SIZE}..={SOURCE_CUBEMAP_MAX_FACE_SIZE}"
            ),
        });
    }
    if face_size > source_face_size {
        return Err(EnvironmentIblSourceStagingError::InvalidSetting {
            key: ENVIRONMENT_IBL_PMREM_FACE_SIZE_IMPORT_SETTING,
            reason: format!(
                "face size {face_size} must not exceed source face size {source_face_size}"
            ),
        });
    }
    Ok((face_size, source_cubemap_mip_count(face_size)))
}

#[cfg(test)]
mod plugins07_environment_extension_hotpath_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const CHECKS_PER_SAMPLE: usize = 240_000;
    const EXTENSIONS: [&str; 6] = ["hdr", "HDR", "ExR", "png", "ktx2", "jpeg"];

    #[test]
    fn borrowed_import_keyword_contract_environment_ibl_extension() {
        assert!(is_environment_ibl_extension("HDR"));
        assert!(is_environment_ibl_extension("eXr"));
        assert!(!is_environment_ibl_extension("png"));
        assert!(!is_environment_ibl_extension("hdr.backup"));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn borrowed_import_keyword_performance_release_environment_ibl_extension() {
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
                (measure_legacy(), measure_borrowed())
            } else {
                let optimized_ns = measure_borrowed();
                (measure_legacy(), optimized_ns)
            };
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT plugins07_environment_ibl_borrowed_extension sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=50 legacy_allocations_per_sample={CHECKS_PER_SAMPLE} optimized_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            improvement_percent >= 50,
            "borrowed environment IBL extension classification must improve P95 by at least 50%"
        );
    }

    fn measure_legacy() -> u128 {
        let started = Instant::now();
        let mut matched = 0_u64;
        for check in 0..CHECKS_PER_SAMPLE {
            let normalized = black_box(EXTENSIONS[check % EXTENSIONS.len()]).to_ascii_lowercase();
            matched += u64::from(matches!(normalized.as_str(), "hdr" | "exr"));
            black_box(normalized);
        }
        black_box(matched);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_borrowed() -> u128 {
        let started = Instant::now();
        let mut matched = 0_u64;
        for check in 0..CHECKS_PER_SAMPLE {
            matched += u64::from(is_environment_ibl_extension(black_box(
                EXTENSIONS[check % EXTENSIONS.len()],
            )));
        }
        black_box(matched);
        started.elapsed().as_nanos().max(1)
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
