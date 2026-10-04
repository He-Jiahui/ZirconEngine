use std::fmt::{Display, Formatter};

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::ShadingModelDescriptor;
use crate::core::resource::{ResourceKind, ResourceLocator, ResourceRecord};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShadingModelIncludeSource {
    pub(crate) token: String,
    pub(crate) source: String,
}

impl ShadingModelIncludeSource {
    fn new(token: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            source: source.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ShadingModelIncludeSourceSet {
    forward: Vec<ShadingModelIncludeSource>,
    gbuffer: Vec<ShadingModelIncludeSource>,
    deferred: Vec<ShadingModelIncludeSource>,
}

impl ShadingModelIncludeSourceSet {
    pub(crate) fn from_project_asset_manager(
        asset_manager: &ProjectAssetManager,
        descriptors: &[ShadingModelDescriptor],
    ) -> Result<Self, ShadingModelIncludeSourceError> {
        Self::from_project_asset_manager_iter(asset_manager, descriptors.iter())
    }

    pub(crate) fn from_project_asset_manager_iter<'a>(
        asset_manager: &ProjectAssetManager,
        descriptors: impl IntoIterator<Item = &'a ShadingModelDescriptor>,
    ) -> Result<Self, ShadingModelIncludeSourceError> {
        let shader_records = asset_manager
            .resource_manager()
            .ready_records_for_kind(ResourceKind::Shader);
        let mut set = Self::default();
        for descriptor in descriptors
            .into_iter()
            .filter(|descriptor| descriptor.id.is_plugin_range())
        {
            if !runtime_owns_include(IncludePass::Forward, &descriptor.forward_include) {
                set.forward.push(resolve_include_source(
                    asset_manager,
                    &shader_records,
                    &descriptor.forward_include,
                )?);
            }
            if !runtime_owns_include(IncludePass::GBuffer, &descriptor.gbuffer_encode_include) {
                set.gbuffer.push(resolve_include_source(
                    asset_manager,
                    &shader_records,
                    &descriptor.gbuffer_encode_include,
                )?);
            }
            if !runtime_owns_include(IncludePass::Deferred, &descriptor.deferred_include) {
                set.deferred.push(resolve_include_source(
                    asset_manager,
                    &shader_records,
                    &descriptor.deferred_include,
                )?);
            }
        }
        Ok(set)
    }

    pub(crate) fn forward(&self) -> &[ShadingModelIncludeSource] {
        &self.forward
    }

    pub(crate) fn gbuffer(&self) -> &[ShadingModelIncludeSource] {
        &self.gbuffer
    }

    pub(crate) fn deferred(&self) -> &[ShadingModelIncludeSource] {
        &self.deferred
    }
}

#[derive(Clone, Copy)]
enum IncludePass {
    Forward,
    GBuffer,
    Deferred,
}

fn runtime_owns_include(pass: IncludePass, token: &str) -> bool {
    let token = normalize_include_token(token);
    match pass {
        IncludePass::Forward => token == "zr_shading_standard_pbr",
        IncludePass::GBuffer => matches!(
            token.as_str(),
            "zr_gbuffer_encode_standard_pbr" | "zr_gbuffer_encode_subsurface"
        ),
        IncludePass::Deferred => matches!(
            token.as_str(),
            "zr_shade_deferred_standard_pbr"
                | "zr_shade_deferred_blinn_phong"
                | "zr_shade_deferred_unlit"
                | "zr_shade_deferred_subsurface"
        ),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ShadingModelIncludeSourceError {
    MissingInclude {
        token: String,
    },
    DuplicateIncludeToken {
        token: String,
        first_locator: String,
        second_locator: String,
    },
    MissingRuntimeSource {
        token: String,
        locator: String,
    },
    LoadShader {
        token: String,
        locator: String,
        message: String,
    },
}

impl Display for ShadingModelIncludeSourceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingInclude { token } => {
                write!(
                    f,
                    "shading model include `{token}` was not found in ready shader assets"
                )
            }
            Self::DuplicateIncludeToken {
                token,
                first_locator,
                second_locator,
            } => write!(
                f,
                "shading model include `{token}` matched multiple shader assets: {first_locator} and {second_locator}"
            ),
            Self::MissingRuntimeSource { token, locator } => write!(
                f,
                "shading model include `{token}` matched {locator}, but the shader has no runtime WGSL source"
            ),
            Self::LoadShader {
                token,
                locator,
                message,
            } => write!(
                f,
                "failed to load shader asset {locator} for shading model include `{token}`: {message}"
            ),
        }
    }
}

impl std::error::Error for ShadingModelIncludeSourceError {}

fn resolve_include_source(
    asset_manager: &ProjectAssetManager,
    shader_records: &[ResourceRecord],
    token: &str,
) -> Result<ShadingModelIncludeSource, ShadingModelIncludeSourceError> {
    let normalized_token = normalize_include_token(token);
    let mut matches = shader_records
        .iter()
        .filter(|record| record_matches_include_token(record, &normalized_token));
    let Some(record) = matches.next() else {
        return Err(ShadingModelIncludeSourceError::MissingInclude {
            token: token.to_string(),
        });
    };
    if let Some(second) = matches.next() {
        return Err(ShadingModelIncludeSourceError::DuplicateIncludeToken {
            token: token.to_string(),
            first_locator: record.primary_locator.to_string(),
            second_locator: second.primary_locator.to_string(),
        });
    }

    let shader = asset_manager
        .load_shader_asset(record.id)
        .map_err(|error| ShadingModelIncludeSourceError::LoadShader {
            token: token.to_string(),
            locator: record.primary_locator.to_string(),
            message: error.to_string(),
        })?;
    let Some(source) = shader.runtime_wgsl_source() else {
        return Err(ShadingModelIncludeSourceError::MissingRuntimeSource {
            token: token.to_string(),
            locator: record.primary_locator.to_string(),
        });
    };
    Ok(ShadingModelIncludeSource::new(token, source))
}

fn record_matches_include_token(record: &ResourceRecord, normalized_token: &str) -> bool {
    locator_matches_include_token(&record.primary_locator, normalized_token)
        || record
            .artifact_locator
            .as_ref()
            .is_some_and(|locator| locator_matches_include_token(locator, normalized_token))
}

fn locator_matches_include_token(locator: &ResourceLocator, normalized_token: &str) -> bool {
    path_matches_normalized_include_token(locator.path(), normalized_token)
}

fn path_matches_normalized_include_token(path: &str, normalized_token: &str) -> bool {
    let path = normalize_include_token(path);
    path == normalized_token
        || path
            .strip_suffix(normalized_token)
            .is_some_and(|prefix| prefix.ends_with('/'))
}

fn normalize_include_token(value: &str) -> String {
    let normalized = value.trim().replace('\\', "/").to_ascii_lowercase();
    normalized
        .strip_suffix(".wgsl")
        .unwrap_or(normalized.as_str())
        .to_string()
}

#[cfg(test)]
#[path = "include_sources/tests/token_hoist_tests.rs"]
mod token_hoist_tests;

#[cfg(test)]
#[path = "tests/include_sources.rs"]
mod tests;
