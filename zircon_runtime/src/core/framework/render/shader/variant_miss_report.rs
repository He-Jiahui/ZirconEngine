use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};

use super::ShaderVariantKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderPipelineDiagnosticStage {
    SourceAssembly,
    WgslValidation,
    PipelineCreation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderPipelineDiagnostic {
    pub variant_key: String,
    pub stage: ShaderPipelineDiagnosticStage,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantMissReport {
    pub request_count: usize,
    pub memory_hit_count: usize,
    pub disk_hit_count: usize,
    pub compile_miss_count: usize,
    pub disk_write_count: usize,
    pub disk_error_count: usize,
    #[serde(default)]
    pub dimension_summary: ShaderVariantRuntimeDimensionSummary,
    #[serde(default, deserialize_with = "deserialize_pipeline_diagnostics")]
    pipeline_diagnostics: Vec<ShaderPipelineDiagnostic>,
}

impl ShaderVariantMissReport {
    pub const MAX_PIPELINE_DIAGNOSTICS: usize = 8;

    pub fn record_request(&mut self, key: &ShaderVariantKey) {
        self.request_count += 1;
        self.dimension_summary
            .record(key, ShaderVariantRuntimeOutcome::Request);
    }

    pub fn record_memory_hit(&mut self, key: &ShaderVariantKey) {
        self.request_count += 1;
        self.memory_hit_count += 1;
        self.dimension_summary
            .record(key, ShaderVariantRuntimeOutcome::MemoryHit);
    }

    pub fn record_disk_hit(&mut self, key: &ShaderVariantKey) {
        self.disk_hit_count += 1;
        self.dimension_summary
            .record(key, ShaderVariantRuntimeOutcome::DiskHit);
    }

    pub fn record_compile_miss(&mut self, key: &ShaderVariantKey) {
        self.compile_miss_count += 1;
        self.dimension_summary
            .record(key, ShaderVariantRuntimeOutcome::CompileMiss);
    }

    pub fn record_disk_write(&mut self, key: &ShaderVariantKey) {
        self.disk_write_count += 1;
        self.dimension_summary
            .record(key, ShaderVariantRuntimeOutcome::DiskWrite);
    }

    pub fn record_disk_error(&mut self, key: &ShaderVariantKey) {
        self.disk_error_count += 1;
        self.dimension_summary
            .record(key, ShaderVariantRuntimeOutcome::DiskError);
    }

    pub fn record_pipeline_diagnostic(
        &mut self,
        key: &ShaderVariantKey,
        stage: ShaderPipelineDiagnosticStage,
        message: impl Into<String>,
    ) {
        self.push_pipeline_diagnostic(ShaderPipelineDiagnostic {
            variant_key: key.canonical_string(),
            stage,
            message: bounded_pipeline_diagnostic_message(message.into()),
        });
    }

    pub fn pipeline_diagnostics(&self) -> &[ShaderPipelineDiagnostic] {
        &self.pipeline_diagnostics
    }

    pub fn accumulate(&mut self, other: Self) {
        self.request_count += other.request_count;
        self.memory_hit_count += other.memory_hit_count;
        self.disk_hit_count += other.disk_hit_count;
        self.compile_miss_count += other.compile_miss_count;
        self.disk_write_count += other.disk_write_count;
        self.disk_error_count += other.disk_error_count;
        self.dimension_summary.accumulate(&other.dimension_summary);
        for diagnostic in other.pipeline_diagnostics {
            self.push_pipeline_diagnostic(diagnostic);
        }
    }

    fn push_pipeline_diagnostic(&mut self, diagnostic: ShaderPipelineDiagnostic) {
        let diagnostic = ShaderPipelineDiagnostic {
            message: bounded_pipeline_diagnostic_message(diagnostic.message),
            ..diagnostic
        };
        if self.pipeline_diagnostics.len() >= Self::MAX_PIPELINE_DIAGNOSTICS
            || self
                .pipeline_diagnostics
                .iter()
                .any(|current| current == &diagnostic)
        {
            return;
        }
        self.pipeline_diagnostics.push(diagnostic);
    }
}

const MAX_PIPELINE_DIAGNOSTIC_MESSAGE_CHARS: usize = 2048;

fn bounded_pipeline_diagnostic_message(mut message: String) -> String {
    const TRUNCATION_SUFFIX: &str = "...";
    let retained_char_count =
        MAX_PIPELINE_DIAGNOSTIC_MESSAGE_CHARS.saturating_sub(TRUNCATION_SUFFIX.chars().count());
    let Some((end, _)) = message.char_indices().nth(retained_char_count) else {
        return message;
    };
    message.truncate(end);
    message.push_str(TRUNCATION_SUFFIX);
    message
}

fn deserialize_pipeline_diagnostics<'de, D>(
    deserializer: D,
) -> Result<Vec<ShaderPipelineDiagnostic>, D::Error>
where
    D: Deserializer<'de>,
{
    let diagnostics = Vec::<ShaderPipelineDiagnostic>::deserialize(deserializer)?;
    let mut report = ShaderVariantMissReport::default();
    for diagnostic in diagnostics {
        report.push_pipeline_diagnostic(diagnostic);
    }
    Ok(report.pipeline_diagnostics)
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantRuntimeDimensionSummary {
    pub pass_types: BTreeMap<String, ShaderVariantRuntimeDimensionCount>,
    pub geometry_source_ids: BTreeMap<String, ShaderVariantRuntimeDimensionCount>,
    pub shading_model_ids: BTreeMap<String, ShaderVariantRuntimeDimensionCount>,
    pub quality_tiers: BTreeMap<String, ShaderVariantRuntimeDimensionCount>,
}

impl ShaderVariantRuntimeDimensionSummary {
    fn record(&mut self, key: &ShaderVariantKey, outcome: ShaderVariantRuntimeOutcome) {
        record_dimension(&mut self.pass_types, key.pass_type.token(), outcome);
        let geometry_source_id = key.geometry_source.value().to_string();
        record_dimension(&mut self.geometry_source_ids, &geometry_source_id, outcome);
        let shading_model_id = key.shading_model.value().to_string();
        record_dimension(&mut self.shading_model_ids, &shading_model_id, outcome);
        record_dimension(&mut self.quality_tiers, key.quality.token(), outcome);
    }

    fn accumulate(&mut self, other: &Self) {
        accumulate_dimensions(&mut self.pass_types, &other.pass_types);
        accumulate_dimensions(&mut self.geometry_source_ids, &other.geometry_source_ids);
        accumulate_dimensions(&mut self.shading_model_ids, &other.shading_model_ids);
        accumulate_dimensions(&mut self.quality_tiers, &other.quality_tiers);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantRuntimeDimensionCount {
    pub request_count: usize,
    pub memory_hit_count: usize,
    pub disk_hit_count: usize,
    pub compile_miss_count: usize,
    pub disk_write_count: usize,
    pub disk_error_count: usize,
}

impl ShaderVariantRuntimeDimensionCount {
    fn record(&mut self, outcome: ShaderVariantRuntimeOutcome) {
        match outcome {
            ShaderVariantRuntimeOutcome::Request => self.request_count += 1,
            ShaderVariantRuntimeOutcome::MemoryHit => {
                self.request_count += 1;
                self.memory_hit_count += 1;
            }
            ShaderVariantRuntimeOutcome::DiskHit => self.disk_hit_count += 1,
            ShaderVariantRuntimeOutcome::CompileMiss => self.compile_miss_count += 1,
            ShaderVariantRuntimeOutcome::DiskWrite => self.disk_write_count += 1,
            ShaderVariantRuntimeOutcome::DiskError => self.disk_error_count += 1,
        }
    }

    fn accumulate(&mut self, other: Self) {
        self.request_count += other.request_count;
        self.memory_hit_count += other.memory_hit_count;
        self.disk_hit_count += other.disk_hit_count;
        self.compile_miss_count += other.compile_miss_count;
        self.disk_write_count += other.disk_write_count;
        self.disk_error_count += other.disk_error_count;
    }
}

#[derive(Clone, Copy, Debug)]
enum ShaderVariantRuntimeOutcome {
    Request,
    MemoryHit,
    DiskHit,
    CompileMiss,
    DiskWrite,
    DiskError,
}

fn record_dimension(
    counts: &mut BTreeMap<String, ShaderVariantRuntimeDimensionCount>,
    key: &str,
    outcome: ShaderVariantRuntimeOutcome,
) {
    if let Some(count) = counts.get_mut(key) {
        count.record(outcome);
        return;
    }

    let mut count = ShaderVariantRuntimeDimensionCount::default();
    count.record(outcome);
    counts.insert(key.to_owned(), count);
}

fn accumulate_dimensions(
    counts: &mut BTreeMap<String, ShaderVariantRuntimeDimensionCount>,
    other: &BTreeMap<String, ShaderVariantRuntimeDimensionCount>,
) {
    for (key, other_count) in other {
        counts
            .entry(key.clone())
            .or_default()
            .accumulate(*other_count);
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::resource::ResourceId;

    use crate::core::framework::render::{
        GeometrySourceId, ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
        SHADING_MODEL_ID_STANDARD_PBR,
    };

    use super::{ShaderPipelineDiagnosticStage, ShaderVariantMissReport};

    #[test]
    fn shader_variant_miss_report_groups_runtime_outcomes_by_variant_dimensions() {
        let key = ShaderVariantKey {
            material_shader: ResourceId::from_stable_label("res://materials/runtime-hit.zshader"),
            material_revision: 11,
            material_layout_hash: 0,
            material_option_bits: 0,
            geometry_source: GeometrySourceId::new(3),
            shading_model: SHADING_MODEL_ID_STANDARD_PBR,
            pass_type: ShaderPassType::Velocity,
            features: ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST),
            quality: ShaderQualityTier::High,
            platform_token: "wgpu-runtime".to_string(),
        };
        let mut report = ShaderVariantMissReport::default();

        report.record_request(&key);
        report.record_disk_hit(&key);
        report.record_compile_miss(&key);
        report.record_disk_write(&key);
        report.record_disk_error(&key);

        assert_eq!(report.request_count, 1);
        assert_eq!(report.disk_hit_count, 1);
        assert_eq!(report.compile_miss_count, 1);
        assert_eq!(report.disk_write_count, 1);
        assert_eq!(report.disk_error_count, 1);

        let pass = report
            .dimension_summary
            .pass_types
            .get("velocity")
            .expect("velocity runtime dimension");
        assert_eq!(pass.request_count, 1);
        assert_eq!(pass.disk_hit_count, 1);
        assert_eq!(pass.compile_miss_count, 1);
        assert_eq!(pass.disk_write_count, 1);
        assert_eq!(pass.disk_error_count, 1);
        assert_eq!(
            report.dimension_summary.geometry_source_ids["3"].disk_hit_count,
            1
        );
        assert_eq!(
            report.dimension_summary.shading_model_ids["2"].compile_miss_count,
            1
        );
        assert_eq!(
            report.dimension_summary.quality_tiers["high"].disk_write_count,
            1
        );
    }

    #[test]
    fn shader_variant_miss_report_deduplicates_and_bounds_pipeline_diagnostics() {
        let key = ShaderVariantKey {
            material_shader: ResourceId::from_stable_label("res://materials/diagnostic.zshader"),
            material_revision: 12,
            material_layout_hash: 0,
            material_option_bits: 0,
            geometry_source: GeometrySourceId::new(3),
            shading_model: SHADING_MODEL_ID_STANDARD_PBR,
            pass_type: ShaderPassType::Forward,
            features: ShaderFeatureBits::default(),
            quality: ShaderQualityTier::High,
            platform_token: "wgpu-runtime".to_string(),
        };
        let mut report = ShaderVariantMissReport::default();

        report.record_pipeline_diagnostic(
            &key,
            ShaderPipelineDiagnosticStage::SourceAssembly,
            "missing surface entry",
        );
        report.record_pipeline_diagnostic(
            &key,
            ShaderPipelineDiagnosticStage::SourceAssembly,
            "missing surface entry",
        );
        for index in 0..ShaderVariantMissReport::MAX_PIPELINE_DIAGNOSTICS {
            report.record_pipeline_diagnostic(
                &key,
                ShaderPipelineDiagnosticStage::WgslValidation,
                format!("validation failure {index}"),
            );
        }

        assert_eq!(
            report.pipeline_diagnostics().len(),
            ShaderVariantMissReport::MAX_PIPELINE_DIAGNOSTICS
        );
        assert_eq!(
            report.pipeline_diagnostics()[0].stage,
            ShaderPipelineDiagnosticStage::SourceAssembly
        );
        assert!(report.pipeline_diagnostics()[0]
            .variant_key
            .contains("res://materials/diagnostic.zshader"));
    }

    #[test]
    fn shader_variant_miss_report_deserialization_enforces_pipeline_diagnostic_limits() {
        let mut document = serde_json::to_value(ShaderVariantMissReport::default())
            .expect("default report serializes");
        let diagnostics = (0..=ShaderVariantMissReport::MAX_PIPELINE_DIAGNOSTICS)
            .map(|index| {
                serde_json::json!({
                    "variant_key": format!("variant-{index}"),
                    "stage": "pipeline_creation",
                    "message": "x".repeat(4096),
                })
            })
            .collect();
        document["pipeline_diagnostics"] = serde_json::Value::Array(diagnostics);

        let report: ShaderVariantMissReport =
            serde_json::from_value(document).expect("diagnostics deserialize");

        assert_eq!(
            report.pipeline_diagnostics().len(),
            ShaderVariantMissReport::MAX_PIPELINE_DIAGNOSTICS
        );
        assert!(
            report.pipeline_diagnostics()[0].message.chars().count()
                <= super::MAX_PIPELINE_DIAGNOSTIC_MESSAGE_CHARS
        );
    }

    #[test]
    fn shader_variant_miss_report_accumulation_normalizes_foreign_diagnostics() {
        let mut destination = ShaderVariantMissReport::default();
        let mut source = ShaderVariantMissReport::default();
        source
            .pipeline_diagnostics
            .push(super::ShaderPipelineDiagnostic {
                variant_key: "foreign-variant".to_string(),
                stage: ShaderPipelineDiagnosticStage::PipelineCreation,
                message: "x".repeat(4096),
            });

        destination.accumulate(source);

        assert_eq!(destination.pipeline_diagnostics().len(), 1);
        assert!(
            destination.pipeline_diagnostics()[0]
                .message
                .chars()
                .count()
                <= super::MAX_PIPELINE_DIAGNOSTIC_MESSAGE_CHARS
        );
    }
}
