use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::asset::{AssetReference, AssetUri};
use crate::core::framework::render::{
    RenderShaderBindingResourceType, RenderShaderDefinitionValue, RenderShaderStage,
    ShaderAssetKind,
};
use crate::core::resource::ResourceId;

use super::{ShaderAsset, ShaderSourceLanguage};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderReadinessReport {
    pub uri: AssetUri,
    #[serde(default = "default_shader_asset_kind")]
    pub kind: ShaderAssetKind,
    #[serde(default)]
    pub kind_diagnostic: Option<String>,
    pub runtime_source: ShaderRuntimeSourceReadiness,
    pub imports: Vec<ShaderImportReadiness>,
    pub entry_points: Vec<ShaderEntryPointReadiness>,
    pub shader_defs: Vec<ShaderDefinitionReadiness>,
    pub validation_diagnostics: Vec<String>,
    pub dependency_count: usize,
    pub has_pipeline_layout: bool,
    #[serde(default)]
    pub pipeline_layout: ShaderPipelineLayoutReadiness,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderAssetReadinessSummary {
    pub ready: bool,
    #[serde(default = "default_shader_asset_kind")]
    pub kind: ShaderAssetKind,
    #[serde(default)]
    pub kind_diagnostic_count: usize,
    pub uses_runtime_wgsl: bool,
    pub runtime_source_kind: ShaderRuntimeSourceKind,
    pub import_count: usize,
    pub redirected_import_count: usize,
    pub entry_point_count: usize,
    pub entry_point_diagnostic_count: usize,
    pub shader_definition_count: usize,
    pub shader_definition_diagnostic_count: usize,
    pub validation_diagnostic_count: usize,
    pub dependency_count: usize,
    pub has_pipeline_layout: bool,
    pub bind_group_count: usize,
    pub binding_count: usize,
    pub push_constant_range_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderAssetManagementRecord {
    pub shader_id: ResourceId,
    pub summary: ShaderAssetReadinessSummary,
    pub report: ShaderReadinessReport,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderAssetManagementRecordSetSummary {
    pub shader_count: usize,
    pub ready_count: usize,
    pub not_ready_count: usize,
    pub runtime_wgsl_count: usize,
    pub unavailable_runtime_source_count: usize,
    #[serde(default)]
    pub kind_diagnostic_count: usize,
    pub redirected_import_count: usize,
    pub dependency_count: usize,
    pub entry_point_diagnostic_count: usize,
    pub shader_definition_diagnostic_count: usize,
    pub validation_diagnostic_count: usize,
    pub pipeline_layout_count: usize,
    pub bind_group_count: usize,
    pub binding_count: usize,
    pub push_constant_range_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderAssetManagementRecordSet {
    pub records: Vec<ShaderAssetManagementRecord>,
    pub summary: ShaderAssetManagementRecordSetSummary,
}

impl ShaderAssetManagementRecordSetSummary {
    pub fn from_records(records: &[ShaderAssetManagementRecord]) -> Self {
        let mut summary = Self {
            shader_count: records.len(),
            ..Self::default()
        };
        for record in records {
            let record = &record.summary;
            summary.ready_count += usize::from(record.ready);
            summary.not_ready_count += usize::from(!record.ready);
            summary.runtime_wgsl_count += usize::from(record.uses_runtime_wgsl);
            summary.unavailable_runtime_source_count +=
                usize::from(record.runtime_source_kind == ShaderRuntimeSourceKind::Unavailable);
            summary.kind_diagnostic_count += record.kind_diagnostic_count;
            summary.redirected_import_count += record.redirected_import_count;
            summary.dependency_count += record.dependency_count;
            summary.entry_point_diagnostic_count += record.entry_point_diagnostic_count;
            summary.shader_definition_diagnostic_count += record.shader_definition_diagnostic_count;
            summary.validation_diagnostic_count += record.validation_diagnostic_count;
            summary.pipeline_layout_count += usize::from(record.has_pipeline_layout);
            summary.bind_group_count += record.bind_group_count;
            summary.binding_count += record.binding_count;
            summary.push_constant_range_count += record.push_constant_range_count;
        }
        summary
    }

    pub fn issue_row_count(&self) -> usize {
        self.unavailable_runtime_source_count
            + self.kind_diagnostic_count
            + self.entry_point_diagnostic_count
            + self.shader_definition_diagnostic_count
            + self.validation_diagnostic_count
    }
}

impl ShaderAssetManagementRecordSet {
    pub fn from_records(mut records: Vec<ShaderAssetManagementRecord>) -> Self {
        records.sort_by_key(|record| record.shader_id);
        let summary = ShaderAssetManagementRecordSetSummary::from_records(&records);
        Self { records, summary }
    }
}

impl ShaderReadinessReport {
    pub fn is_ready(&self) -> bool {
        self.kind_diagnostic.is_none()
            && self.uses_runtime_wgsl()
            && self
                .entry_points
                .iter()
                .all(|entry| entry.diagnostic.is_none())
            && self
                .shader_defs
                .iter()
                .all(|definition| definition.diagnostic.is_none())
            && self.validation_diagnostics.is_empty()
    }

    pub fn uses_runtime_wgsl(&self) -> bool {
        matches!(
            self.runtime_source.source_kind,
            ShaderRuntimeSourceKind::EmittedWgsl | ShaderRuntimeSourceKind::RawWgslSource
        )
    }

    pub fn has_redirected_import_dependencies(&self) -> bool {
        self.imports
            .iter()
            .any(|import| import.contributes_dependency)
    }

    pub fn summary(&self) -> ShaderAssetReadinessSummary {
        ShaderAssetReadinessSummary {
            ready: self.is_ready(),
            kind: self.kind,
            kind_diagnostic_count: usize::from(self.kind_diagnostic.is_some()),
            uses_runtime_wgsl: self.uses_runtime_wgsl(),
            runtime_source_kind: self.runtime_source.source_kind,
            import_count: self.imports.len(),
            redirected_import_count: self
                .imports
                .iter()
                .filter(|import| import.contributes_dependency)
                .count(),
            entry_point_count: self.entry_points.len(),
            entry_point_diagnostic_count: self
                .entry_points
                .iter()
                .filter(|entry| entry.diagnostic.is_some())
                .count(),
            shader_definition_count: self.shader_defs.len(),
            shader_definition_diagnostic_count: self
                .shader_defs
                .iter()
                .filter(|definition| definition.diagnostic.is_some())
                .count(),
            validation_diagnostic_count: self.validation_diagnostics.len(),
            dependency_count: self.dependency_count,
            has_pipeline_layout: self.has_pipeline_layout,
            bind_group_count: self.pipeline_layout.bind_group_count,
            binding_count: self.pipeline_layout.binding_count,
            push_constant_range_count: self.pipeline_layout.push_constant_range_count,
        }
    }

    pub fn management_record(&self, shader_id: ResourceId) -> ShaderAssetManagementRecord {
        ShaderAssetManagementRecord {
            shader_id,
            summary: self.summary(),
            report: self.clone(),
        }
    }

    fn from_shader(shader: &ShaderAsset) -> Self {
        let entry_points = entry_point_readiness(shader);
        Self {
            uri: shader.uri.clone(),
            kind: shader.kind,
            kind_diagnostic: shader_kind_diagnostic(shader, &entry_points),
            runtime_source: runtime_source_readiness(shader),
            imports: import_readiness(shader),
            entry_points,
            shader_defs: shader_definition_readiness(shader),
            validation_diagnostics: shader.validation_diagnostics.clone(),
            dependency_count: shader.dependencies.len(),
            has_pipeline_layout: !shader.pipeline_layout.bind_groups.is_empty()
                || !shader.pipeline_layout.push_constant_ranges.is_empty(),
            pipeline_layout: pipeline_layout_readiness(shader),
        }
    }
}

impl ShaderAssetReadinessSummary {
    pub fn issue_row_count(&self) -> usize {
        (if self.runtime_source_kind == ShaderRuntimeSourceKind::Unavailable {
            1
        } else {
            0
        }) + self.kind_diagnostic_count
            + self.entry_point_diagnostic_count
            + self.shader_definition_diagnostic_count
            + self.validation_diagnostic_count
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderRuntimeSourceReadiness {
    pub source_language: ShaderSourceLanguage,
    pub source_kind: ShaderRuntimeSourceKind,
    pub diagnostic: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderRuntimeSourceKind {
    EmittedWgsl,
    RawWgslSource,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderImportReadiness {
    pub source: String,
    pub redirect: Option<AssetReference>,
    pub contributes_dependency: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_diagnostic: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderEntryPointReadiness {
    pub name: String,
    pub stage: String,
    pub canonical_stage: Option<RenderShaderStage>,
    pub diagnostic: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderDefinitionReadiness {
    pub raw_name: String,
    pub normalized_name: String,
    pub value: RenderShaderDefinitionValue,
    pub diagnostic: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderPipelineLayoutReadiness {
    pub has_layout: bool,
    pub bind_group_count: usize,
    pub binding_count: usize,
    pub push_constant_range_count: usize,
    pub bind_groups: Vec<ShaderBindGroupLayoutReadiness>,
    pub push_constant_ranges: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderBindGroupLayoutReadiness {
    pub group: u32,
    pub label: Option<String>,
    pub binding_count: usize,
    pub bindings: Vec<ShaderBindingLayoutReadiness>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderBindingLayoutReadiness {
    pub binding: u32,
    pub label: Option<String>,
    pub resource_type: RenderShaderBindingResourceType,
    pub visibility: Vec<RenderShaderStage>,
}

impl ShaderAsset {
    pub fn readiness_report(&self) -> ShaderReadinessReport {
        ShaderReadinessReport::from_shader(self)
    }

    pub fn readiness_summary(&self) -> ShaderAssetReadinessSummary {
        self.readiness_report().summary()
    }

    pub fn management_record(&self, shader_id: ResourceId) -> ShaderAssetManagementRecord {
        self.readiness_report().management_record(shader_id)
    }
}

fn runtime_source_readiness(shader: &ShaderAsset) -> ShaderRuntimeSourceReadiness {
    if !shader.wgsl_source.trim().is_empty() {
        ShaderRuntimeSourceReadiness {
            source_language: shader.source_language,
            source_kind: ShaderRuntimeSourceKind::EmittedWgsl,
            diagnostic: None,
        }
    } else if shader.source_language == ShaderSourceLanguage::Wgsl
        && !shader.source.trim().is_empty()
    {
        ShaderRuntimeSourceReadiness {
            source_language: shader.source_language,
            source_kind: ShaderRuntimeSourceKind::RawWgslSource,
            diagnostic: None,
        }
    } else {
        ShaderRuntimeSourceReadiness {
            source_language: shader.source_language,
            source_kind: ShaderRuntimeSourceKind::Unavailable,
            diagnostic: Some(format!(
                "shader `{}` does not provide emitted WGSL and cannot use `{}` source directly at runtime",
                shader.uri,
                shader.source_language.as_str()
            )),
        }
    }
}

fn shader_kind_diagnostic(
    shader: &ShaderAsset,
    entry_points: &[ShaderEntryPointReadiness],
) -> Option<String> {
    match shader.kind {
        ShaderAssetKind::Module => None,
        ShaderAssetKind::Surface => {
            if !shader
                .shading_model
                .as_deref()
                .is_some_and(|model| !model.trim().is_empty())
            {
                return Some("surface shader requires a non-empty shading model".to_string());
            }
            shader
                .surface_source_contract()
                .err()
                .map(|error| error.to_string())
        }
        ShaderAssetKind::Include => (!entry_points.is_empty()).then(|| {
            format!(
                "include shader must not own entry points (found {})",
                entry_points.len()
            )
        }),
        ShaderAssetKind::Compute => executable_kind_diagnostic(
            ShaderAssetKind::Compute,
            RenderShaderStage::Compute,
            entry_points,
        ),
        ShaderAssetKind::Fullscreen => executable_kind_diagnostic(
            ShaderAssetKind::Fullscreen,
            RenderShaderStage::Fragment,
            entry_points,
        ),
    }
}

fn executable_kind_diagnostic(
    kind: ShaderAssetKind,
    expected_stage: RenderShaderStage,
    entry_points: &[ShaderEntryPointReadiness],
) -> Option<String> {
    if entry_points.is_empty() {
        return Some(format!(
            "{} shader requires at least one entry point",
            kind.token()
        ));
    }
    entry_points
        .iter()
        .find(|entry| {
            entry
                .canonical_stage
                .is_some_and(|stage| stage != expected_stage)
        })
        .map(|entry| {
            format!(
                "{} shader entry point `{}` uses `{}` but expected `{}`",
                kind.token(),
                entry.name,
                entry.stage,
                render_shader_stage_token(expected_stage)
            )
        })
}

fn render_shader_stage_token(stage: RenderShaderStage) -> &'static str {
    match stage {
        RenderShaderStage::Vertex => "vertex",
        RenderShaderStage::Fragment => "fragment",
        RenderShaderStage::Compute => "compute",
    }
}

fn default_shader_asset_kind() -> ShaderAssetKind {
    ShaderAssetKind::Surface
}

fn import_readiness(shader: &ShaderAsset) -> Vec<ShaderImportReadiness> {
    shader
        .imports
        .iter()
        .map(|import| ShaderImportReadiness {
            source: import.source.clone(),
            redirect: import.redirect.clone(),
            contributes_dependency: import.redirect.is_some(),
            source_diagnostic: import.redirect.as_ref().map(|redirect| {
                format!(
                    "shader import `{}` is redirected to `{}`",
                    import.source, redirect.locator
                )
            }),
        })
        .collect()
}

fn entry_point_readiness(shader: &ShaderAsset) -> Vec<ShaderEntryPointReadiness> {
    shader
        .entry_points
        .iter()
        .map(|entry| match entry.descriptor() {
            Some(descriptor) => ShaderEntryPointReadiness {
                name: entry.name.clone(),
                stage: entry.stage.clone(),
                canonical_stage: Some(descriptor.stage),
                diagnostic: None,
            },
            None => ShaderEntryPointReadiness {
                name: entry.name.clone(),
                stage: entry.stage.clone(),
                canonical_stage: None,
                diagnostic: Some(format!(
                    "shader entry point `{}` uses unsupported stage `{}`",
                    entry.name, entry.stage
                )),
            },
        })
        .collect()
}

fn shader_definition_readiness(shader: &ShaderAsset) -> Vec<ShaderDefinitionReadiness> {
    let mut seen = HashSet::with_capacity(shader.shader_defs.len());
    shader
        .shader_defs
        .iter()
        .map(|definition| {
            let normalized_name = definition.name().trim();
            let diagnostic = if normalized_name.is_empty() {
                Some("shader definition is empty after trimming".to_string())
            } else if !seen.insert(normalized_name) {
                Some(format!(
                    "shader definition `{}` is duplicated",
                    normalized_name
                ))
            } else {
                None
            };

            ShaderDefinitionReadiness {
                raw_name: definition.name().to_string(),
                normalized_name: normalized_name.to_string(),
                value: definition.clone(),
                diagnostic,
            }
        })
        .collect()
}

fn pipeline_layout_readiness(shader: &ShaderAsset) -> ShaderPipelineLayoutReadiness {
    let bind_groups = shader
        .pipeline_layout
        .bind_groups
        .iter()
        .map(|group| ShaderBindGroupLayoutReadiness {
            group: group.group,
            label: group.label.clone(),
            binding_count: group.bindings.len(),
            bindings: group
                .bindings
                .iter()
                .map(|binding| ShaderBindingLayoutReadiness {
                    binding: binding.binding,
                    label: binding.label.clone(),
                    resource_type: binding.resource_type,
                    visibility: binding.visibility.clone(),
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let binding_count: usize = bind_groups.iter().map(|group| group.binding_count).sum();
    let push_constant_ranges = shader.pipeline_layout.push_constant_ranges.clone();

    ShaderPipelineLayoutReadiness {
        has_layout: !bind_groups.is_empty() || !push_constant_ranges.is_empty(),
        bind_group_count: bind_groups.len(),
        binding_count,
        push_constant_range_count: push_constant_ranges.len(),
        bind_groups,
        push_constant_ranges,
    }
}

#[cfg(test)]
#[path = "readiness/optimization_tests.rs"]
mod optimization_tests;
