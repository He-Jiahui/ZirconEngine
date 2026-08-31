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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderPipelineFallbackState {
    Deferred,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderPipelineFallbackAction {
    DeferDraw,
    RejectDraw,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderPipelineFallbackDiagnostic {
    pub variant_key: String,
    pub pipeline_variant_id: u32,
    pub entity_id: u64,
    pub consumer: String,
    pub state: ShaderPipelineFallbackState,
    pub action: ShaderPipelineFallbackAction,
    pub reason: String,
    pub state_age_microseconds: u64,
    pub occurrence_count: usize,
}

pub const SHADER_PIPELINE_TARGET_COUNT: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum ShaderPipelineTarget {
    Base = 0,
    GBuffer = 1,
    DepthPrepass = 2,
    HitProxy = 3,
    ShadowDepth = 4,
    ShadowDepthAlphaMask = 5,
    Velocity = 6,
    TaaReactiveMask = 7,
    TaaReactiveMaterialMask = 8,
    Oit = 9,
}

impl ShaderPipelineTarget {
    pub const ALL: [Self; SHADER_PIPELINE_TARGET_COUNT] = [
        Self::Base,
        Self::GBuffer,
        Self::DepthPrepass,
        Self::HitProxy,
        Self::ShadowDepth,
        Self::ShadowDepthAlphaMask,
        Self::Velocity,
        Self::TaaReactiveMask,
        Self::TaaReactiveMaterialMask,
        Self::Oit,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn token(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::GBuffer => "gbuffer",
            Self::DepthPrepass => "depth_prepass",
            Self::HitProxy => "hit_proxy",
            Self::ShadowDepth => "shadow_depth",
            Self::ShadowDepthAlphaMask => "shadow_depth_alpha_mask",
            Self::Velocity => "velocity",
            Self::TaaReactiveMask => "taa_reactive_mask",
            Self::TaaReactiveMaterialMask => "taa_reactive_material_mask",
            Self::Oit => "oit",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderPipelineTargetMetrics {
    pub registered_pipeline_variant_count: usize,
    pub unique_shader_source_count: usize,
    pub render_pipeline_creation_count: usize,
    pub shader_module_creation_count: usize,
    pub render_pipeline_creation_cpu_microseconds: u64,
    pub shader_module_creation_cpu_microseconds: u64,
}

impl ShaderPipelineTargetMetrics {
    fn accumulate_snapshot(&mut self, other: Self) {
        self.registered_pipeline_variant_count = self
            .registered_pipeline_variant_count
            .max(other.registered_pipeline_variant_count);
        self.unique_shader_source_count = self
            .unique_shader_source_count
            .max(other.unique_shader_source_count);
        if creation_snapshot_is_greater(
            self.render_pipeline_creation_count,
            self.render_pipeline_creation_cpu_microseconds,
            other.render_pipeline_creation_count,
            other.render_pipeline_creation_cpu_microseconds,
        ) {
            self.render_pipeline_creation_count = other.render_pipeline_creation_count;
            self.render_pipeline_creation_cpu_microseconds =
                other.render_pipeline_creation_cpu_microseconds;
        }
        if creation_snapshot_is_greater(
            self.shader_module_creation_count,
            self.shader_module_creation_cpu_microseconds,
            other.shader_module_creation_count,
            other.shader_module_creation_cpu_microseconds,
        ) {
            self.shader_module_creation_count = other.shader_module_creation_count;
            self.shader_module_creation_cpu_microseconds =
                other.shader_module_creation_cpu_microseconds;
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderSourceValidationMetrics {
    pub queued_count: usize,
    pub already_pending_count: usize,
    pub full_count: usize,
    pub worker_unavailable_count: usize,
    pub job_count: usize,
    pub unique_source_count: usize,
    pub duplicate_job_count: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub queue_wait_microseconds: u64,
    pub validation_cpu_microseconds: u64,
}

impl ShaderSourceValidationMetrics {
    fn accumulate_snapshot(&mut self, other: Self) {
        if other.snapshot_order() > self.snapshot_order() {
            *self = other;
        }
    }

    fn snapshot_order(self) -> (usize, usize, usize, u64, u64) {
        (
            self.queued_count
                .saturating_add(self.already_pending_count)
                .saturating_add(self.full_count)
                .saturating_add(self.worker_unavailable_count),
            self.job_count,
            self.success_count.saturating_add(self.failure_count),
            self.validation_cpu_microseconds,
            self.queue_wait_microseconds,
        )
    }
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
    pub registered_pipeline_variant_count: usize,
    #[serde(default)]
    pub registered_shader_variant_count: usize,
    #[serde(default)]
    pub texture_presence_normalized_pipeline_variant_count: usize,
    #[serde(default)]
    pub texture_presence_equivalent_pipeline_variant_count: usize,
    #[serde(default)]
    pub cached_render_pipeline_count: usize,
    #[serde(default)]
    pub cached_shader_module_count: usize,
    #[serde(default)]
    pub render_pipeline_creation_count: usize,
    #[serde(default)]
    pub shader_module_creation_count: usize,
    #[serde(default)]
    pub render_pipeline_creation_cpu_microseconds: u64,
    #[serde(default)]
    pub shader_module_creation_cpu_microseconds: u64,
    #[serde(default)]
    pub async_base_pipeline_queue_wait_count: usize,
    #[serde(default)]
    pub async_base_pipeline_queue_wait_microseconds: u64,
    #[serde(default)]
    pub shader_source_validation_metrics: ShaderSourceValidationMetrics,
    #[serde(default)]
    pub pipeline_deferred_draw_count: usize,
    #[serde(default)]
    pub pipeline_failed_draw_count: usize,
    #[serde(default)]
    pipeline_target_metrics: [ShaderPipelineTargetMetrics; SHADER_PIPELINE_TARGET_COUNT],
    #[serde(default)]
    pub dimension_summary: ShaderVariantRuntimeDimensionSummary,
    #[serde(default, deserialize_with = "deserialize_pipeline_diagnostics")]
    pipeline_diagnostics: Vec<ShaderPipelineDiagnostic>,
    #[serde(default, deserialize_with = "deserialize_pipeline_fallbacks")]
    pipeline_fallbacks: Vec<ShaderPipelineFallbackDiagnostic>,
}

impl ShaderVariantMissReport {
    pub const MAX_PIPELINE_DIAGNOSTICS: usize = MAX_SHADER_VARIANT_REPORT_DIAGNOSTICS;
    pub const MAX_PIPELINE_FALLBACKS: usize = MAX_SHADER_VARIANT_REPORT_DIAGNOSTICS;

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

    pub fn record_registered_variant_counts(
        &mut self,
        pipeline_variant_count: usize,
        shader_variant_count: usize,
        texture_presence_normalized_pipeline_variant_count: usize,
    ) {
        self.registered_pipeline_variant_count = pipeline_variant_count;
        self.registered_shader_variant_count = shader_variant_count;
        self.texture_presence_normalized_pipeline_variant_count =
            texture_presence_normalized_pipeline_variant_count;
        self.texture_presence_equivalent_pipeline_variant_count = pipeline_variant_count
            .saturating_sub(texture_presence_normalized_pipeline_variant_count);
    }

    pub fn record_cached_gpu_object_counts(
        &mut self,
        render_pipeline_count: usize,
        shader_module_count: usize,
    ) {
        self.cached_render_pipeline_count = render_pipeline_count;
        self.cached_shader_module_count = shader_module_count;
    }

    pub fn record_gpu_object_creation_totals(
        &mut self,
        render_pipeline_count: usize,
        shader_module_count: usize,
        render_pipeline_cpu_microseconds: u64,
        shader_module_cpu_microseconds: u64,
    ) {
        self.render_pipeline_creation_count = render_pipeline_count;
        self.shader_module_creation_count = shader_module_count;
        self.render_pipeline_creation_cpu_microseconds = render_pipeline_cpu_microseconds;
        self.shader_module_creation_cpu_microseconds = shader_module_cpu_microseconds;
    }

    pub fn record_async_base_pipeline_queue_wait_totals(
        &mut self,
        count: usize,
        cpu_microseconds: u64,
    ) {
        self.async_base_pipeline_queue_wait_count = count;
        self.async_base_pipeline_queue_wait_microseconds = cpu_microseconds;
    }

    pub fn record_shader_source_validation_metrics(
        &mut self,
        metrics: ShaderSourceValidationMetrics,
    ) {
        self.shader_source_validation_metrics = metrics;
    }

    pub fn pipeline_target_metrics(
        &self,
        target: ShaderPipelineTarget,
    ) -> ShaderPipelineTargetMetrics {
        self.pipeline_target_metrics[target.index()]
    }

    pub fn record_registered_pipeline_target_variant_count(
        &mut self,
        target: ShaderPipelineTarget,
        count: usize,
    ) {
        self.pipeline_target_metrics[target.index()].registered_pipeline_variant_count = count;
    }

    pub fn record_pipeline_target_runtime_metrics(
        &mut self,
        target: ShaderPipelineTarget,
        metrics: ShaderPipelineTargetMetrics,
    ) {
        let target_metrics = &mut self.pipeline_target_metrics[target.index()];
        let registered_pipeline_variant_count = target_metrics.registered_pipeline_variant_count;
        *target_metrics = metrics;
        target_metrics.registered_pipeline_variant_count =
            registered_pipeline_variant_count.max(metrics.registered_pipeline_variant_count);
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

    #[allow(clippy::too_many_arguments)]
    pub fn record_pipeline_fallback(
        &mut self,
        key: &ShaderVariantKey,
        pipeline_variant_id: u32,
        entity_id: u64,
        consumer: &str,
        state: ShaderPipelineFallbackState,
        action: ShaderPipelineFallbackAction,
        reason: &str,
        state_age_microseconds: u64,
    ) {
        self.record_pipeline_fallback_with_identity(
            || key.canonical_string(),
            pipeline_variant_id,
            entity_id,
            consumer,
            state,
            action,
            reason,
            state_age_microseconds,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_unresolved_pipeline_fallback(
        &mut self,
        pipeline_variant_id: u32,
        entity_id: u64,
        consumer: &str,
        state: ShaderPipelineFallbackState,
        action: ShaderPipelineFallbackAction,
        reason: &str,
        state_age_microseconds: u64,
    ) {
        self.record_pipeline_fallback_with_identity(
            || format!("unresolved-pipeline-variant:{pipeline_variant_id}"),
            pipeline_variant_id,
            entity_id,
            consumer,
            state,
            action,
            reason,
            state_age_microseconds,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn record_pipeline_fallback_with_identity(
        &mut self,
        variant_key: impl FnOnce() -> String,
        pipeline_variant_id: u32,
        entity_id: u64,
        consumer: &str,
        state: ShaderPipelineFallbackState,
        action: ShaderPipelineFallbackAction,
        reason: &str,
        state_age_microseconds: u64,
    ) {
        match state {
            ShaderPipelineFallbackState::Deferred => self.pipeline_deferred_draw_count += 1,
            ShaderPipelineFallbackState::Failed => self.pipeline_failed_draw_count += 1,
        }
        if let Some(current) = self.pipeline_fallbacks.iter_mut().find(|current| {
            current.pipeline_variant_id == pipeline_variant_id
                && current.entity_id == entity_id
                && current.consumer == consumer
                && current.state == state
                && current.action == action
                && current.reason == reason
        }) {
            current.occurrence_count = current.occurrence_count.saturating_add(1);
            current.state_age_microseconds =
                current.state_age_microseconds.max(state_age_microseconds);
            return;
        }
        if self.pipeline_fallbacks.len() >= Self::MAX_PIPELINE_FALLBACKS {
            return;
        }
        self.push_pipeline_fallback(ShaderPipelineFallbackDiagnostic {
            variant_key: variant_key(),
            pipeline_variant_id,
            entity_id,
            consumer: consumer.to_owned(),
            state,
            action,
            reason: reason.to_owned(),
            state_age_microseconds,
            occurrence_count: 1,
        });
    }

    pub fn pipeline_fallbacks(&self) -> &[ShaderPipelineFallbackDiagnostic] {
        &self.pipeline_fallbacks
    }

    pub fn accumulate(&mut self, other: Self) {
        self.request_count += other.request_count;
        self.memory_hit_count += other.memory_hit_count;
        self.disk_hit_count += other.disk_hit_count;
        self.compile_miss_count += other.compile_miss_count;
        self.disk_write_count += other.disk_write_count;
        self.disk_error_count += other.disk_error_count;
        self.pipeline_deferred_draw_count += other.pipeline_deferred_draw_count;
        self.pipeline_failed_draw_count += other.pipeline_failed_draw_count;
        if other.registered_pipeline_variant_count > self.registered_pipeline_variant_count {
            self.registered_pipeline_variant_count = other.registered_pipeline_variant_count;
            self.registered_shader_variant_count = other.registered_shader_variant_count;
            self.texture_presence_normalized_pipeline_variant_count =
                other.texture_presence_normalized_pipeline_variant_count;
            self.texture_presence_equivalent_pipeline_variant_count =
                other.texture_presence_equivalent_pipeline_variant_count;
        }
        self.cached_render_pipeline_count = self
            .cached_render_pipeline_count
            .max(other.cached_render_pipeline_count);
        self.cached_shader_module_count = self
            .cached_shader_module_count
            .max(other.cached_shader_module_count);
        if creation_snapshot_is_greater(
            self.render_pipeline_creation_count,
            self.render_pipeline_creation_cpu_microseconds,
            other.render_pipeline_creation_count,
            other.render_pipeline_creation_cpu_microseconds,
        ) {
            self.render_pipeline_creation_count = other.render_pipeline_creation_count;
            self.render_pipeline_creation_cpu_microseconds =
                other.render_pipeline_creation_cpu_microseconds;
        }
        if creation_snapshot_is_greater(
            self.shader_module_creation_count,
            self.shader_module_creation_cpu_microseconds,
            other.shader_module_creation_count,
            other.shader_module_creation_cpu_microseconds,
        ) {
            self.shader_module_creation_count = other.shader_module_creation_count;
            self.shader_module_creation_cpu_microseconds =
                other.shader_module_creation_cpu_microseconds;
        }
        if creation_snapshot_is_greater(
            self.async_base_pipeline_queue_wait_count,
            self.async_base_pipeline_queue_wait_microseconds,
            other.async_base_pipeline_queue_wait_count,
            other.async_base_pipeline_queue_wait_microseconds,
        ) {
            self.async_base_pipeline_queue_wait_count = other.async_base_pipeline_queue_wait_count;
            self.async_base_pipeline_queue_wait_microseconds =
                other.async_base_pipeline_queue_wait_microseconds;
        }
        self.shader_source_validation_metrics
            .accumulate_snapshot(other.shader_source_validation_metrics);
        for target in ShaderPipelineTarget::ALL {
            self.pipeline_target_metrics[target.index()]
                .accumulate_snapshot(other.pipeline_target_metrics[target.index()]);
        }
        self.dimension_summary.accumulate(&other.dimension_summary);
        for diagnostic in other.pipeline_diagnostics {
            self.push_pipeline_diagnostic(diagnostic);
        }
        for fallback in other.pipeline_fallbacks {
            self.push_pipeline_fallback(fallback);
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

    fn push_pipeline_fallback(&mut self, fallback: ShaderPipelineFallbackDiagnostic) {
        if let Some(current) = self.pipeline_fallbacks.iter_mut().find(|current| {
            current.variant_key == fallback.variant_key
                && current.pipeline_variant_id == fallback.pipeline_variant_id
                && current.entity_id == fallback.entity_id
                && current.consumer == fallback.consumer
                && current.state == fallback.state
                && current.action == fallback.action
                && current.reason == fallback.reason
        }) {
            current.occurrence_count = current
                .occurrence_count
                .saturating_add(fallback.occurrence_count);
            current.state_age_microseconds = current
                .state_age_microseconds
                .max(fallback.state_age_microseconds);
            return;
        }
        if self.pipeline_fallbacks.len() < Self::MAX_PIPELINE_FALLBACKS {
            self.pipeline_fallbacks.push(fallback);
        }
    }
}

const MAX_SHADER_VARIANT_REPORT_DIAGNOSTICS: usize = 8;

fn creation_snapshot_is_greater(
    current_count: usize,
    current_cpu_microseconds: u64,
    candidate_count: usize,
    candidate_cpu_microseconds: u64,
) -> bool {
    candidate_count > current_count
        || (candidate_count == current_count
            && candidate_cpu_microseconds > current_cpu_microseconds)
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

fn deserialize_pipeline_fallbacks<'de, D>(
    deserializer: D,
) -> Result<Vec<ShaderPipelineFallbackDiagnostic>, D::Error>
where
    D: Deserializer<'de>,
{
    let fallbacks = Vec::<ShaderPipelineFallbackDiagnostic>::deserialize(deserializer)?;
    let mut report = ShaderVariantMissReport::default();
    for fallback in fallbacks {
        report.push_pipeline_fallback(fallback);
    }
    Ok(report.pipeline_fallbacks)
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
#[path = "variant_miss_report/tests.rs"]
mod tests;
