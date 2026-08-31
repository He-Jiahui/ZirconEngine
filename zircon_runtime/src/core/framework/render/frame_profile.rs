use serde::{Deserialize, Serialize};

mod gpu_timing_status;

pub use gpu_timing_status::RenderGpuTimingStatus;

/// Stable budget categories shared by graph-pass, subsystem, and frame observations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RenderBudgetKey {
    Shadow,
    DepthPrepass,
    Hzb,
    GpuSceneUpdate,
    BasePass,
    LightGrid,
    DeferredLighting,
    Ssao,
    Transparent,
    PostProcess,
    TemporalAa,
    Ui,
    #[default]
    Other,
}

impl RenderBudgetKey {
    pub const ALL: [Self; 13] = [
        Self::Shadow,
        Self::DepthPrepass,
        Self::Hzb,
        Self::GpuSceneUpdate,
        Self::BasePass,
        Self::LightGrid,
        Self::DeferredLighting,
        Self::Ssao,
        Self::Transparent,
        Self::PostProcess,
        Self::TemporalAa,
        Self::Ui,
        Self::Other,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Shadow => "shadow",
            Self::DepthPrepass => "depth_prepass",
            Self::Hzb => "hzb",
            Self::GpuSceneUpdate => "gpu_scene_update",
            Self::BasePass => "base_pass",
            Self::LightGrid => "light_grid",
            Self::DeferredLighting => "deferred_lighting",
            Self::Ssao => "ssao",
            Self::Transparent => "transparent",
            Self::PostProcess => "post_process",
            Self::TemporalAa => "temporal_aa",
            Self::Ui => "ui",
            Self::Other => "other",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderPassPipelineStatistics {
    pub vertex_shader_invocations: u64,
    pub clipper_invocations: u64,
    pub clipper_primitives_out: u64,
    pub fragment_shader_invocations: u64,
    pub compute_shader_invocations: u64,
}

/// Native resource creation observed while recording one render-graph pass.
///
/// These counters identify pass-time allocation and pipeline compilation pressure. They are work
/// counts, not elapsed-time or proof that a particular cache would improve performance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderPassNativeResourceCreateMetrics {
    pub buffer_count: u32,
    pub bind_group_count: u32,
    pub bind_group_layout_count: u32,
    pub shader_module_count: u32,
    pub pipeline_layout_count: u32,
    pub compute_pipeline_count: u32,
    pub render_pipeline_count: u32,
}

impl RenderPassNativeResourceCreateMetrics {
    pub const fn new(
        buffer_count: u32,
        bind_group_count: u32,
        bind_group_layout_count: u32,
        shader_module_count: u32,
        pipeline_layout_count: u32,
        compute_pipeline_count: u32,
        render_pipeline_count: u32,
    ) -> Self {
        Self {
            buffer_count,
            bind_group_count,
            bind_group_layout_count,
            shader_module_count,
            pipeline_layout_count,
            compute_pipeline_count,
            render_pipeline_count,
        }
    }

    pub const fn total_count(self) -> u32 {
        self.buffer_count
            .saturating_add(self.bind_group_count)
            .saturating_add(self.bind_group_layout_count)
            .saturating_add(self.shader_module_count)
            .saturating_add(self.pipeline_layout_count)
            .saturating_add(self.compute_pipeline_count)
            .saturating_add(self.render_pipeline_count)
    }

    pub const fn saturating_add(self, other: Self) -> Self {
        Self {
            buffer_count: self.buffer_count.saturating_add(other.buffer_count),
            bind_group_count: self.bind_group_count.saturating_add(other.bind_group_count),
            bind_group_layout_count: self
                .bind_group_layout_count
                .saturating_add(other.bind_group_layout_count),
            shader_module_count: self
                .shader_module_count
                .saturating_add(other.shader_module_count),
            pipeline_layout_count: self
                .pipeline_layout_count
                .saturating_add(other.pipeline_layout_count),
            compute_pipeline_count: self
                .compute_pipeline_count
                .saturating_add(other.compute_pipeline_count),
            render_pipeline_count: self
                .render_pipeline_count
                .saturating_add(other.render_pipeline_count),
        }
    }

    pub(crate) fn record_buffer(&mut self) {
        self.buffer_count = self.buffer_count.saturating_add(1);
    }

    pub(crate) fn record_bind_group(&mut self) {
        self.bind_group_count = self.bind_group_count.saturating_add(1);
    }

    pub(crate) fn record_bind_group_layout(&mut self) {
        self.bind_group_layout_count = self.bind_group_layout_count.saturating_add(1);
    }

    pub(crate) fn record_shader_module(&mut self) {
        self.shader_module_count = self.shader_module_count.saturating_add(1);
    }

    pub(crate) fn record_pipeline_layout(&mut self) {
        self.pipeline_layout_count = self.pipeline_layout_count.saturating_add(1);
    }

    pub(crate) fn record_compute_pipeline(&mut self) {
        self.compute_pipeline_count = self.compute_pipeline_count.saturating_add(1);
    }

    pub(crate) fn record_render_pipeline(&mut self) {
        self.render_pipeline_count = self.render_pipeline_count.saturating_add(1);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderPassProfileEntry {
    pub pass_name: String,
    pub executor_id: String,
    pub budget_key: RenderBudgetKey,
    /// CPU time spent recording this pass, retained separately from asynchronous GPU timing.
    #[serde(default)]
    pub cpu_elapsed_micros: u64,
    pub gpu_time_us: Option<u64>,
    pub pipeline_statistics: Option<RenderPassPipelineStatistics>,
    pub draw_count: u32,
    pub instance_count: u32,
    pub state_change_count: u32,
    pub upload_bytes: u64,
    pub dispatch_count: u32,
    #[serde(default)]
    pub native_resource_creates: RenderPassNativeResourceCreateMetrics,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderSubsystemProfileEntry {
    pub key: RenderBudgetKey,
    pub gpu_time_us: Option<u64>,
    pub budget_us: u64,
    pub over_budget: bool,
}

/// Frame-qualified mesh submission counters used to correlate cache and indirect work with timing.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMeshSubmissionProfile {
    pub draw_count: u32,
    pub command_count: u32,
    /// Commands routed through the ordinary deferred-opaque queue.
    #[serde(default)]
    pub opaque_command_count: u32,
    /// Opaque commands routed through the late Forward advanced-PBR queue.
    #[serde(default)]
    pub advanced_pbr_opaque_command_count: u32,
    pub cached_command_hit_count: u32,
    pub command_rebuild_count: u32,
    pub dynamic_command_count: u32,
    pub static_command_cache_skipped_draw_count: u32,
    pub static_command_cache_visibility_pruned_draw_count: u32,
    pub indirect_batch_count: u32,
    pub indirect_batched_draw_count: u32,
    pub indirect_fallback_draw_count: u32,
    pub indirect_workspace_uploaded_bytes: u64,
    pub replay_state_change_count: u32,
    pub replay_bind_skip_count: u32,
    pub material_bind_group_set_count: u32,
    pub material_bind_group_skip_count: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderFrameProfile {
    pub frame_generation: u64,
    pub gpu_frame_time_us: Option<u64>,
    #[serde(default)]
    pub gpu_timing_status: RenderGpuTimingStatus,
    pub cpu_submit_time_us: u64,
    #[serde(default)]
    pub parallel_recording_eligible_stage_count: u32,
    #[serde(default)]
    pub parallel_recording_eligible_bucket_count: u32,
    #[serde(default)]
    pub parallel_recording_executed_stage_count: u32,
    #[serde(default)]
    pub parallel_recording_executed_bucket_count: u32,
    pub profile_latency_frames: u32,
    pub passes: Vec<RenderPassProfileEntry>,
    pub subsystems: Vec<RenderSubsystemProfileEntry>,
    #[serde(default)]
    pub mesh_submission: RenderMeshSubmissionProfile,
    pub transient_texture_peak_bytes: u64,
    pub transient_buffer_peak_bytes: u64,
    pub staging_total_bytes: u64,
    /// Persistent scene texture residency, independent of graph-owned transient attachments.
    pub persistent_texture_resident_bytes: u64,
    pub compiled_graph_cache_hit: bool,
    pub variant_miss_count: u32,
    pub store_lint_count: u32,
    pub budget_warning_count: u32,
    pub degrade_step_active: u32,
}

/// Reference GPU budgets are observations, not frame-dropping policy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderFrameBudget {
    total_budget_us: u64,
    per_key_budget_us: Vec<(RenderBudgetKey, u64)>,
}

impl RenderFrameBudget {
    pub fn reference_1080p_mid() -> Self {
        const MILLISECONDS_TO_MICROSECONDS: u64 = 1_000;
        let per_key_budget_us = vec![
            (RenderBudgetKey::Shadow, 2_200),
            (RenderBudgetKey::DepthPrepass, 700),
            (RenderBudgetKey::Hzb, 250),
            (RenderBudgetKey::GpuSceneUpdate, 400),
            (RenderBudgetKey::BasePass, 3_200),
            (RenderBudgetKey::LightGrid, 350),
            (RenderBudgetKey::DeferredLighting, 2_200),
            (RenderBudgetKey::Ssao, 800),
            (RenderBudgetKey::Transparent, 1_200),
            (RenderBudgetKey::PostProcess, 1_600),
            (RenderBudgetKey::TemporalAa, 700),
            (RenderBudgetKey::Ui, 400),
            (RenderBudgetKey::Other, 0),
        ];
        Self {
            total_budget_us: 14 * MILLISECONDS_TO_MICROSECONDS,
            per_key_budget_us,
        }
    }

    pub const fn total_budget_us(&self) -> u64 {
        self.total_budget_us
    }

    pub fn budget_us(&self, key: RenderBudgetKey) -> u64 {
        self.per_key_budget_us
            .iter()
            .find_map(|(candidate, budget_us)| (*candidate == key).then_some(*budget_us))
            .unwrap_or_default()
    }

    pub fn entries(&self) -> &[(RenderBudgetKey, u64)] {
        &self.per_key_budget_us
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RenderBudgetKey, RenderFrameBudget, RenderFrameProfile, RenderGpuTimingStatus,
        RenderMeshSubmissionProfile, RenderPassProfileEntry,
    };

    #[test]
    fn reference_1080p_mid_budget_covers_each_profile_category() {
        let budget = RenderFrameBudget::reference_1080p_mid();

        assert_eq!(budget.total_budget_us(), 14_000);
        assert_eq!(budget.entries().len(), RenderBudgetKey::ALL.len());
        assert_eq!(
            budget
                .entries()
                .iter()
                .map(|(_, budget_us)| *budget_us)
                .sum::<u64>(),
            budget.total_budget_us()
        );
        assert_eq!(budget.budget_us(RenderBudgetKey::Shadow), 2_200);
        assert_eq!(budget.budget_us(RenderBudgetKey::BasePass), 3_200);
        assert_eq!(budget.budget_us(RenderBudgetKey::Other), 0);
    }

    #[test]
    fn empty_frame_profile_reports_no_gpu_timing_until_the_timer_resolves() {
        let profile = RenderFrameProfile::default();

        assert_eq!(profile.gpu_frame_time_us, None);
        assert_eq!(profile.gpu_timing_status, RenderGpuTimingStatus::Disabled);
        assert!(profile.passes.is_empty());
        assert!(profile.subsystems.is_empty());
    }

    #[test]
    fn legacy_frame_profile_json_defaults_missing_mesh_submission_metrics() {
        let profile = RenderFrameProfile {
            mesh_submission: RenderMeshSubmissionProfile {
                command_count: 9,
                cached_command_hit_count: 5,
                ..RenderMeshSubmissionProfile::default()
            },
            ..RenderFrameProfile::default()
        };
        let mut legacy = serde_json::to_value(profile).expect("frame profile serializes");
        legacy
            .as_object_mut()
            .expect("frame profile is a JSON object")
            .remove("mesh_submission");

        let decoded: RenderFrameProfile =
            serde_json::from_value(legacy).expect("legacy frame profile remains readable");

        assert_eq!(
            decoded.mesh_submission,
            RenderMeshSubmissionProfile::default()
        );
    }

    #[test]
    fn mesh_submission_profile_json_defaults_missing_opaque_phase_counts() {
        let profile = RenderFrameProfile {
            mesh_submission: RenderMeshSubmissionProfile {
                opaque_command_count: 2,
                advanced_pbr_opaque_command_count: 1,
                ..RenderMeshSubmissionProfile::default()
            },
            ..RenderFrameProfile::default()
        };
        let mut serialized = serde_json::to_value(profile).expect("frame profile serializes");
        let mesh_submission = serialized
            .as_object_mut()
            .and_then(|profile| profile.get_mut("mesh_submission"))
            .and_then(serde_json::Value::as_object_mut)
            .expect("serialized frame profile contains mesh submission metrics");
        mesh_submission.remove("opaque_command_count");
        mesh_submission.remove("advanced_pbr_opaque_command_count");

        let decoded: RenderFrameProfile =
            serde_json::from_value(serialized).expect("legacy frame profile remains readable");

        assert_eq!(decoded.mesh_submission.opaque_command_count, 0);
        assert_eq!(decoded.mesh_submission.advanced_pbr_opaque_command_count, 0);
    }

    #[test]
    fn legacy_pass_profile_json_defaults_missing_cpu_time() {
        let entry = RenderPassProfileEntry {
            pass_name: "opaque".to_owned(),
            executor_id: "mesh.opaque".to_owned(),
            budget_key: RenderBudgetKey::BasePass,
            cpu_elapsed_micros: 41,
            gpu_time_us: None,
            pipeline_statistics: None,
            draw_count: 1,
            instance_count: 1,
            state_change_count: 0,
            upload_bytes: 0,
            dispatch_count: 0,
            native_resource_creates: RenderPassNativeResourceCreateMetrics::new(
                1, 2, 3, 4, 5, 6, 7,
            ),
        };
        let mut legacy = serde_json::to_value(entry).expect("pass profile serializes");
        let legacy = legacy
            .as_object_mut()
            .expect("pass profile is a JSON object");
        legacy.remove("cpu_elapsed_micros");
        legacy.remove("native_resource_creates");

        let decoded: RenderPassProfileEntry =
            serde_json::from_value(legacy).expect("legacy pass profile remains readable");

        assert_eq!(decoded.cpu_elapsed_micros, 0);
        assert_eq!(
            decoded.native_resource_creates,
            RenderPassNativeResourceCreateMetrics::default()
        );
    }

    #[test]
    fn native_resource_create_metrics_keep_categories_and_saturating_total() {
        let metrics = RenderPassNativeResourceCreateMetrics::new(1, 2, 3, 4, 5, 6, u32::MAX);

        assert_eq!(metrics.buffer_count, 1);
        assert_eq!(metrics.bind_group_count, 2);
        assert_eq!(metrics.bind_group_layout_count, 3);
        assert_eq!(metrics.shader_module_count, 4);
        assert_eq!(metrics.pipeline_layout_count, 5);
        assert_eq!(metrics.compute_pipeline_count, 6);
        assert_eq!(metrics.render_pipeline_count, u32::MAX);
        assert_eq!(metrics.total_count(), u32::MAX);
    }

    #[test]
    fn legacy_frame_profile_json_defaults_missing_parallel_recording_counts() {
        let profile = RenderFrameProfile {
            parallel_recording_eligible_stage_count: 1,
            parallel_recording_eligible_bucket_count: 3,
            parallel_recording_executed_stage_count: 1,
            parallel_recording_executed_bucket_count: 2,
            ..RenderFrameProfile::default()
        };
        let mut legacy = serde_json::to_value(profile).expect("frame profile serializes");
        let object = legacy
            .as_object_mut()
            .expect("frame profile is a JSON object");
        for field in [
            "gpu_timing_status",
            "parallel_recording_eligible_stage_count",
            "parallel_recording_eligible_bucket_count",
            "parallel_recording_executed_stage_count",
            "parallel_recording_executed_bucket_count",
        ] {
            object.remove(field);
        }

        let decoded: RenderFrameProfile =
            serde_json::from_value(legacy).expect("legacy frame profile remains readable");

        assert_eq!(decoded.gpu_timing_status, RenderGpuTimingStatus::Disabled);
        assert_eq!(decoded.parallel_recording_eligible_stage_count, 0);
        assert_eq!(decoded.parallel_recording_eligible_bucket_count, 0);
        assert_eq!(decoded.parallel_recording_executed_stage_count, 0);
        assert_eq!(decoded.parallel_recording_executed_bucket_count, 0);
    }
}
