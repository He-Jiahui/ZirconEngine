use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::ShaderVariantKey;

mod budget;
mod source;
mod toolchain;

pub use budget::{
    ShaderVariantPrewarmExecutionBudget, ShaderVariantPrewarmExecutionBudgetError,
    ShaderVariantPrewarmExecutionBudgetSummary,
};
pub use source::{
    ShaderVariantPrewarmSource, ShaderVariantPrewarmSourceId, ShaderVariantPrewarmSourceTable,
};
pub use toolchain::{SHADER_VARIANT_CACHE_NAGA_VERSION, SHADER_VARIANT_CACHE_WGPU_VERSION};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShaderVariantPrewarmManifest {
    pub schema_version: u32,
    pub sources: Vec<ShaderVariantPrewarmSource>,
    pub variants: Vec<ShaderVariantPrewarmRequest>,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ShaderVariantPrewarmManifestIntegrityError {
    #[error("shader prewarm source {source_id} does not match its content-addressed id")]
    NonCanonicalSourceId { source_id: String },
    #[error("shader prewarm source id {source_id} occurs more than once")]
    DuplicateSourceId { source_id: String },
    #[error("shader prewarm variant {canonical_key} references missing source {source_id}")]
    MissingVariantSource {
        canonical_key: String,
        source_id: String,
    },
}

impl ShaderVariantPrewarmManifest {
    pub const SCHEMA_VERSION: u32 = 3;

    pub fn new(
        sources: Vec<ShaderVariantPrewarmSource>,
        variants: Vec<ShaderVariantPrewarmRequest>,
    ) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            sources,
            variants,
        }
    }

    pub fn empty() -> Self {
        Self::new(Vec::new(), Vec::new())
    }

    pub fn source_for(
        &self,
        request: &ShaderVariantPrewarmRequest,
    ) -> Option<&ShaderVariantPrewarmSource> {
        self.sources
            .iter()
            .find(|source| source.id == request.source_id)
    }

    /// Builds one borrowed lookup table for repeated request-to-source resolution.
    pub fn source_table(&self) -> ShaderVariantPrewarmSourceTable<'_> {
        ShaderVariantPrewarmSourceTable::new(&self.sources)
    }

    /// Returns the exact heap footprint of the persisted source table.
    pub fn source_table_resident_bytes(&self) -> Option<usize> {
        self.sources.iter().try_fold(0usize, |total, source| {
            total.checked_add(source.resident_bytes())
        })
    }

    pub fn validate_integrity(&self) -> Result<(), ShaderVariantPrewarmManifestIntegrityError> {
        let mut source_ids = HashSet::with_capacity(self.sources.len());
        for source in &self.sources {
            if !source.has_canonical_id() {
                return Err(
                    ShaderVariantPrewarmManifestIntegrityError::NonCanonicalSourceId {
                        source_id: source.id.as_str().to_string(),
                    },
                );
            }
            if !source_ids.insert(source.id.clone()) {
                return Err(
                    ShaderVariantPrewarmManifestIntegrityError::DuplicateSourceId {
                        source_id: source.id.as_str().to_string(),
                    },
                );
            }
        }
        for request in &self.variants {
            if !source_ids.contains(&request.source_id) {
                return Err(
                    ShaderVariantPrewarmManifestIntegrityError::MissingVariantSource {
                        canonical_key: request.key.canonical_string(),
                        source_id: request.source_id.as_str().to_string(),
                    },
                );
            }
        }
        Ok(())
    }

    pub fn replace_variant_source(
        &mut self,
        variant_index: usize,
        source: ShaderVariantPrewarmSource,
    ) -> bool {
        if variant_index >= self.variants.len() {
            return false;
        }
        let source_id = source.id.clone();
        if !self.sources.iter().any(|existing| existing.id == source.id) {
            self.sources.push(source);
        }
        self.variants[variant_index].source_id = source_id;
        let referenced_source_ids = self
            .variants
            .iter()
            .map(|request| request.source_id.clone())
            .collect::<HashSet<_>>();
        self.sources
            .retain(|existing| referenced_source_ids.contains(&existing.id));
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShaderVariantPrewarmRequest {
    pub key: ShaderVariantKey,
    /// Exact render-pipeline state not represented by [`ShaderVariantKey`].
    ///
    /// Offline shader-cache consumers may omit this field. Runtime PSO prewarm
    /// requires it so a compiled pipeline is inserted under the same complete
    /// key that draw submission will resolve.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipeline_state: Option<ShaderPipelinePrewarmState>,
    pub source_id: ShaderVariantPrewarmSourceId,
}

/// Render-pipeline state not already represented by [`ShaderVariantKey`].
///
/// Standard material binding presence is excluded: fixed texture bindings use
/// neutral fallback resources and do not change shader or PSO identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShaderPipelinePrewarmState {
    pub alpha_blend: bool,
    pub alpha_cutoff_bits: Option<u32>,
    pub unlit: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmReport {
    pub requested_count: usize,
    pub written_count: usize,
    pub failed_count: usize,
    /// A failure that prevented prewarm work from starting for the whole manifest.
    ///
    /// Unlike [`Self::failures`], this is not attributed to a synthetic variant
    /// index and does not affect the per-variant counters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preflight_error: Option<String>,
    #[serde(default)]
    pub written_variants: Vec<ShaderVariantPrewarmWrittenVariant>,
    #[serde(default)]
    pub wgpu_module_validation: ShaderVariantPrewarmWgpuModuleValidationSummary,
    #[serde(default)]
    pub wgpu_pipeline_validation: ShaderVariantPrewarmWgpuPipelineValidationSummary,
    #[serde(default)]
    pub dimension_summary: ShaderVariantPrewarmDimensionSummary,
    #[serde(default)]
    pub source_provenance: ShaderVariantPrewarmSourceProvenanceSummary,
    #[serde(default)]
    pub execution_budget: ShaderVariantPrewarmExecutionBudgetSummary,
    pub failures: Vec<ShaderVariantPrewarmFailure>,
}

impl ShaderVariantPrewarmReport {
    pub fn record_preflight_error(&mut self, error: impl Into<String>) {
        self.preflight_error = Some(error.into());
    }

    pub fn record_written(&mut self) {
        self.requested_count += 1;
        self.written_count += 1;
    }

    pub fn record_written_variant(&mut self, key: &ShaderVariantKey) {
        self.record_written();
        self.dimension_summary
            .record(key, ShaderVariantPrewarmOutcome::Written);
    }

    pub fn record_written_request(
        &mut self,
        request: &ShaderVariantPrewarmRequest,
        source: &ShaderVariantPrewarmSource,
    ) {
        self.record_written_variant(&request.key);
        self.source_provenance
            .record(source, ShaderVariantPrewarmOutcome::Written);
    }

    pub fn record_written_cache_entry(
        &mut self,
        request: &ShaderVariantPrewarmRequest,
        source: &ShaderVariantPrewarmSource,
        cache_hash: impl Into<String>,
        canonical_string: impl Into<String>,
    ) {
        self.record_written_request(request, source);
        self.written_variants
            .push(ShaderVariantPrewarmWrittenVariant {
                cache_hash: cache_hash.into(),
                canonical_string: canonical_string.into(),
                source_id: source.id.clone(),
                source_label: source.provenance_source_label(),
                template_revision: source.template_revision.clone(),
                naga_version: source.naga_version.clone(),
                wgpu_version: source.wgpu_version.clone(),
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
        source: &ShaderVariantPrewarmSource,
        error: impl Into<String>,
    ) {
        self.record_failure_variant(variant_index, &request.key, error);
        self.source_provenance
            .record(source, ShaderVariantPrewarmOutcome::Failed);
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

    pub fn enable_wgpu_pipeline_validation(&mut self, requested_count: usize) {
        self.wgpu_pipeline_validation.enabled = true;
        self.wgpu_pipeline_validation.requested_count = requested_count;
    }

    pub fn record_wgpu_pipeline_validation_passed(&mut self) {
        self.wgpu_pipeline_validation.validated_count += 1;
    }

    pub fn record_wgpu_pipeline_validation_failed(&mut self) {
        self.wgpu_pipeline_validation.failed_count += 1;
    }

    pub fn record_wgpu_pipeline_validation_skipped(&mut self) {
        self.wgpu_pipeline_validation.skipped_count += 1;
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmWgpuPipelineValidationSummary {
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
    pub source_id: ShaderVariantPrewarmSourceId,
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
        source: &ShaderVariantPrewarmSource,
        outcome: ShaderVariantPrewarmOutcome,
    ) {
        self.variant_count += 1;
        if let Some(entry) = self.sources.get_mut(source.id.as_str()) {
            entry.record(outcome);
            return;
        }

        let mut entry = ShaderVariantPrewarmSourceProvenanceEntry {
            source_id: source.id.clone(),
            source_label: source.provenance_source_label(),
            source_hash: source.source_hash(),
            include_content_hashes: source.include_content_hashes.clone(),
            template_revision: source.template_revision.clone(),
            naga_version: source.naga_version.clone(),
            wgpu_version: source.wgpu_version.clone(),
            ..Default::default()
        };
        entry.record(outcome);
        self.sources.insert(source.id.as_str().to_string(), entry);
        self.source_count = self.sources.len();
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantPrewarmSourceProvenanceEntry {
    pub source_id: ShaderVariantPrewarmSourceId,
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

impl ShaderVariantPrewarmSource {
    fn provenance_source_label(&self) -> String {
        let source_label = self.source_label.trim();
        if source_label.is_empty() {
            "<unlabeled>".to_string()
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

#[cfg(test)]
mod tests {
    use super::{
        ShaderVariantPrewarmManifest, ShaderVariantPrewarmManifestIntegrityError,
        ShaderVariantPrewarmOutcome, ShaderVariantPrewarmSource,
        ShaderVariantPrewarmSourceProvenanceSummary,
    };

    #[test]
    fn source_provenance_aggregates_repeated_source_outcomes() {
        let source = ShaderVariantPrewarmSource::new(
            "res://shared.wgsl",
            "fn main() {}",
            vec!["include-a".to_string()],
            "template-r1",
            "naga-r1",
            "wgpu-r1",
        );
        let mut summary = ShaderVariantPrewarmSourceProvenanceSummary::default();

        summary.record(&source, ShaderVariantPrewarmOutcome::Written);
        summary.record(&source, ShaderVariantPrewarmOutcome::Failed);

        assert_eq!(summary.source_count, 1);
        assert_eq!(summary.variant_count, 2);
        let entry = summary
            .sources
            .get(source.id.as_str())
            .expect("shared source should have one provenance entry");
        assert_eq!(entry.source_hash, source.source_hash());
        assert_eq!(entry.requested_count, 2);
        assert_eq!(entry.written_count, 1);
        assert_eq!(entry.failed_count, 1);
    }

    #[test]
    fn manifest_integrity_reports_duplicate_source_id_as_typed_error() {
        let source = ShaderVariantPrewarmSource::new(
            "res://shared.wgsl",
            "fn main() {}",
            Vec::new(),
            "template-r1",
            "naga-r1",
            "wgpu-r1",
        );

        let error = ShaderVariantPrewarmManifest::new(vec![source.clone(), source], Vec::new())
            .validate_integrity()
            .expect_err("duplicate source ids must fail manifest integrity validation");
        assert!(matches!(
            error,
            ShaderVariantPrewarmManifestIntegrityError::DuplicateSourceId { .. }
        ));
    }
}
