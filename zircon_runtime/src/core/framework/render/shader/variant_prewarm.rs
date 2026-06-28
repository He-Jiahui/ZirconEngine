use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::ShaderVariantKey;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmManifest {
    pub schema_version: u32,
    pub variants: Vec<ShaderVariantPrewarmRequest>,
}

impl ShaderVariantPrewarmManifest {
    pub const SCHEMA_VERSION: u32 = 1;

    pub fn new(variants: Vec<ShaderVariantPrewarmRequest>) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            variants,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmRequest {
    pub key: ShaderVariantKey,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source_label: String,
    pub wgsl_source: String,
    pub include_content_hashes: Vec<String>,
    pub template_revision: String,
    pub naga_version: String,
    pub wgpu_version: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmReport {
    pub requested_count: usize,
    pub written_count: usize,
    pub failed_count: usize,
    #[serde(default)]
    pub written_variants: Vec<ShaderVariantPrewarmWrittenVariant>,
    #[serde(default)]
    pub wgpu_module_validation: ShaderVariantPrewarmWgpuModuleValidationSummary,
    #[serde(default)]
    pub dimension_summary: ShaderVariantPrewarmDimensionSummary,
    #[serde(default)]
    pub source_provenance: ShaderVariantPrewarmSourceProvenanceSummary,
    pub failures: Vec<ShaderVariantPrewarmFailure>,
}

impl ShaderVariantPrewarmReport {
    pub fn record_written(&mut self) {
        self.requested_count += 1;
        self.written_count += 1;
    }

    pub fn record_written_variant(&mut self, key: &ShaderVariantKey) {
        self.record_written();
        self.dimension_summary
            .record(key, ShaderVariantPrewarmOutcome::Written);
    }

    pub fn record_written_request(&mut self, request: &ShaderVariantPrewarmRequest) {
        self.record_written_variant(&request.key);
        self.source_provenance
            .record(request, ShaderVariantPrewarmOutcome::Written);
    }

    pub fn record_written_cache_entry(
        &mut self,
        request: &ShaderVariantPrewarmRequest,
        cache_hash: impl Into<String>,
        canonical_string: impl Into<String>,
    ) {
        self.record_written_request(request);
        self.written_variants
            .push(ShaderVariantPrewarmWrittenVariant {
                cache_hash: cache_hash.into(),
                canonical_string: canonical_string.into(),
                source_label: request.provenance_source_label(),
                template_revision: request.template_revision.clone(),
                naga_version: request.naga_version.clone(),
                wgpu_version: request.wgpu_version.clone(),
            });
    }

    pub fn record_failure(&mut self, variant_index: usize, error: impl Into<String>) {
        self.requested_count += 1;
        self.failed_count += 1;
        self.failures.push(ShaderVariantPrewarmFailure {
            variant_index,
            error: error.into(),
        });
    }

    pub fn record_failure_variant(
        &mut self,
        variant_index: usize,
        key: &ShaderVariantKey,
        error: impl Into<String>,
    ) {
        self.record_failure(variant_index, error);
        self.dimension_summary
            .record(key, ShaderVariantPrewarmOutcome::Failed);
    }

    pub fn record_failure_request(
        &mut self,
        variant_index: usize,
        request: &ShaderVariantPrewarmRequest,
        error: impl Into<String>,
    ) {
        self.record_failure_variant(variant_index, &request.key, error);
        self.source_provenance
            .record(request, ShaderVariantPrewarmOutcome::Failed);
    }

    pub fn enable_wgpu_module_validation(&mut self, requested_count: usize) {
        self.wgpu_module_validation.enabled = true;
        self.wgpu_module_validation.requested_count = requested_count;
    }

    pub fn record_wgpu_module_validation_passed(&mut self) {
        self.wgpu_module_validation.validated_count += 1;
    }

    pub fn record_wgpu_module_validation_failed(&mut self) {
        self.wgpu_module_validation.failed_count += 1;
    }

    pub fn record_wgpu_module_validation_skipped(&mut self) {
        self.wgpu_module_validation.skipped_count += 1;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmWgpuModuleValidationSummary {
    pub enabled: bool,
    pub requested_count: usize,
    pub validated_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmDimensionSummary {
    pub pass_types: BTreeMap<String, ShaderVariantPrewarmDimensionCount>,
    pub geometry_source_ids: BTreeMap<String, ShaderVariantPrewarmDimensionCount>,
    pub shading_model_ids: BTreeMap<String, ShaderVariantPrewarmDimensionCount>,
    pub quality_tiers: BTreeMap<String, ShaderVariantPrewarmDimensionCount>,
}

impl ShaderVariantPrewarmDimensionSummary {
    fn record(&mut self, key: &ShaderVariantKey, outcome: ShaderVariantPrewarmOutcome) {
        record_dimension(
            &mut self.pass_types,
            key.pass_type.token().to_string(),
            outcome,
        );
        record_dimension(
            &mut self.geometry_source_ids,
            key.geometry_source.value().to_string(),
            outcome,
        );
        record_dimension(
            &mut self.shading_model_ids,
            key.shading_model.value().to_string(),
            outcome,
        );
        record_dimension(
            &mut self.quality_tiers,
            key.quality.token().to_string(),
            outcome,
        );
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmDimensionCount {
    pub requested_count: usize,
    pub written_count: usize,
    pub failed_count: usize,
}

impl ShaderVariantPrewarmDimensionCount {
    fn record(&mut self, outcome: ShaderVariantPrewarmOutcome) {
        self.requested_count += 1;
        match outcome {
            ShaderVariantPrewarmOutcome::Written => self.written_count += 1,
            ShaderVariantPrewarmOutcome::Failed => self.failed_count += 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmFailure {
    pub variant_index: usize,
    pub error: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmWrittenVariant {
    pub cache_hash: String,
    pub canonical_string: String,
    pub source_label: String,
    pub template_revision: String,
    pub naga_version: String,
    pub wgpu_version: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmSourceProvenanceSummary {
    pub source_count: usize,
    pub variant_count: usize,
    pub sources: BTreeMap<String, ShaderVariantPrewarmSourceProvenanceEntry>,
}

impl ShaderVariantPrewarmSourceProvenanceSummary {
    fn record(
        &mut self,
        request: &ShaderVariantPrewarmRequest,
        outcome: ShaderVariantPrewarmOutcome,
    ) {
        self.variant_count += 1;
        let source_label = request.provenance_source_label();
        let source_hash = shader_prewarm_source_hash(&request.wgsl_source);
        let key =
            shader_prewarm_provenance_key(&source_label, &source_hash, &request.template_revision);
        let entry =
            self.sources
                .entry(key)
                .or_insert_with(|| ShaderVariantPrewarmSourceProvenanceEntry {
                    source_label,
                    source_hash,
                    include_content_hashes: request.include_content_hashes.clone(),
                    template_revision: request.template_revision.clone(),
                    naga_version: request.naga_version.clone(),
                    wgpu_version: request.wgpu_version.clone(),
                    ..Default::default()
                });
        entry.record(outcome);
        self.source_count = self.sources.len();
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmSourceProvenanceEntry {
    pub source_label: String,
    pub source_hash: String,
    pub include_content_hashes: Vec<String>,
    pub template_revision: String,
    pub naga_version: String,
    pub wgpu_version: String,
    pub requested_count: usize,
    pub written_count: usize,
    pub failed_count: usize,
}

impl ShaderVariantPrewarmSourceProvenanceEntry {
    fn record(&mut self, outcome: ShaderVariantPrewarmOutcome) {
        self.requested_count += 1;
        match outcome {
            ShaderVariantPrewarmOutcome::Written => self.written_count += 1,
            ShaderVariantPrewarmOutcome::Failed => self.failed_count += 1,
        }
    }
}

impl ShaderVariantPrewarmRequest {
    fn provenance_source_label(&self) -> String {
        let source_label = self.source_label.trim();
        if source_label.is_empty() {
            self.key.material_shader.to_string()
        } else {
            source_label.to_string()
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ShaderVariantPrewarmOutcome {
    Written,
    Failed,
}

fn record_dimension(
    counts: &mut BTreeMap<String, ShaderVariantPrewarmDimensionCount>,
    key: String,
    outcome: ShaderVariantPrewarmOutcome,
) {
    counts.entry(key).or_default().record(outcome);
}

fn shader_prewarm_source_hash(source: &str) -> String {
    blake3::hash(source.as_bytes()).to_hex().to_string()
}

fn shader_prewarm_provenance_key(
    source_label: &str,
    source_hash: &str,
    template_revision: &str,
) -> String {
    format!("{source_label}#{source_hash}#{template_revision}")
}
