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
        matches!(
            context
                .source_path
                .extension()
                .and_then(|extension| extension.to_str())
                .map(str::to_ascii_lowercase)
                .as_deref(),
            Some("hdr" | "exr")
        )
    }
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
