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
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderSubsystemProfileEntry {
    pub key: RenderBudgetKey,
    pub gpu_time_us: Option<u64>,
    pub budget_us: u64,
    pub over_budget: bool,
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
        RenderPassProfileEntry,
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
        };
        let mut legacy = serde_json::to_value(entry).expect("pass profile serializes");
        legacy
            .as_object_mut()
            .expect("pass profile is a JSON object")
            .remove("cpu_elapsed_micros");

        let decoded: RenderPassProfileEntry =
            serde_json::from_value(legacy).expect("legacy pass profile remains readable");

        assert_eq!(decoded.cpu_elapsed_micros, 0);
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
