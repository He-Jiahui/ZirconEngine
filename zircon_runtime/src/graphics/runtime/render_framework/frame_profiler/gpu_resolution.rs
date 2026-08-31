use std::sync::Arc;

use crate::core::framework::render::{
    RenderFrameBudget, RenderFrameProfile, RenderGpuTimingStatus, RenderPassPipelineStatistics,
    RenderPassProfileEntry,
};
use crate::graphics::backend::{GpuPipelineStatisticsFrameResult, GpuTimerFrameResult};

use super::{saturating_u32, FrameProfiler};

pub(in crate::graphics::runtime::render_framework) struct FrameProfileWrite {
    pub(in crate::graphics::runtime::render_framework) capture_profile: Arc<RenderFrameProfile>,
    pub(in crate::graphics::runtime::render_framework) resolved_gpu_profiles:
        Vec<Arc<RenderFrameProfile>>,
    pub(in crate::graphics::runtime::render_framework) resolved_gpu_profile:
        Option<Arc<RenderFrameProfile>>,
}

impl FrameProfiler {
    pub(super) fn merge_gpu_timer_result(
        &mut self,
        result: &GpuTimerFrameResult,
        current_frame_generation: u64,
    ) -> Option<u64> {
        // A published snapshot is cloned only when a late GPU result still shares it.
        let profile_index = self
            .pending_profiles
            .iter()
            .position(|profile| profile.frame_generation == result.frame_generation)?;
        {
            let profile = Arc::make_mut(&mut self.pending_profiles[profile_index]);

            let mut matched_passes = vec![false; profile.passes.len()];
            for timing in &result.pass_timings {
                if let Some(pass_index) = take_next_pass_profile_index(
                    &profile.passes,
                    &mut matched_passes,
                    &timing.pass_name,
                ) {
                    profile.passes[pass_index].gpu_time_us = Some(timing.gpu_time_us);
                }
            }

            profile.gpu_frame_time_us = (!result.pass_timings.is_empty()).then(|| {
                result.pass_timings.iter().fold(0_u64, |total, timing| {
                    total.saturating_add(timing.gpu_time_us)
                })
            });
            profile.profile_latency_frames = saturating_u32_from_u64(
                current_frame_generation.saturating_sub(result.frame_generation),
            );
            if profile.gpu_timing_status != RenderGpuTimingStatus::CapacityExhausted {
                profile.gpu_timing_status = RenderGpuTimingStatus::Measured;
            }
            update_subsystem_gpu_times(profile, &self.budget);
            profile.budget_warning_count = profile
                .budget_warning_count
                .saturating_add(gpu_budget_warning_count(profile, &self.budget));
        }

        Some(result.frame_generation)
    }

    pub(super) fn merge_gpu_pipeline_statistics_result(
        &mut self,
        result: &GpuPipelineStatisticsFrameResult,
        current_frame_generation: u64,
    ) -> Option<u64> {
        let profile_index = self
            .pending_profiles
            .iter()
            .position(|profile| profile.frame_generation == result.frame_generation)?;
        {
            let profile = Arc::make_mut(&mut self.pending_profiles[profile_index]);
            let mut matched_passes = vec![false; profile.passes.len()];
            for statistics in &result.pass_statistics {
                if let Some(pass_index) = take_next_pass_profile_index(
                    &profile.passes,
                    &mut matched_passes,
                    &statistics.pass_name,
                ) {
                    profile.passes[pass_index].pipeline_statistics =
                        Some(RenderPassPipelineStatistics {
                            vertex_shader_invocations: statistics
                                .statistics
                                .vertex_shader_invocations,
                            clipper_invocations: statistics.statistics.clipper_invocations,
                            clipper_primitives_out: statistics.statistics.clipper_primitives_out,
                            fragment_shader_invocations: statistics
                                .statistics
                                .fragment_shader_invocations,
                            compute_shader_invocations: statistics
                                .statistics
                                .compute_shader_invocations,
                        });
                }
            }
            profile.profile_latency_frames =
                profile.profile_latency_frames.max(saturating_u32_from_u64(
                    current_frame_generation.saturating_sub(result.frame_generation),
                ));
        }
        Some(result.frame_generation)
    }
}

fn take_next_pass_profile_index(
    passes: &[RenderPassProfileEntry],
    matched_passes: &mut [bool],
    pass_name: &str,
) -> Option<usize> {
    for (index, pass) in passes.iter().enumerate() {
        if !matched_passes[index] && pass.pass_name == pass_name {
            matched_passes[index] = true;
            return Some(index);
        }
    }
    None
}

fn update_subsystem_gpu_times(profile: &mut RenderFrameProfile, budget: &RenderFrameBudget) {
    let passes = &profile.passes;
    for subsystem in &mut profile.subsystems {
        let mut has_gpu_timing = false;
        let gpu_time_us = passes
            .iter()
            .filter(|pass| pass.budget_key == subsystem.key)
            .filter_map(|pass| pass.gpu_time_us)
            .inspect(|_| has_gpu_timing = true)
            .fold(0_u64, u64::saturating_add);
        subsystem.gpu_time_us = has_gpu_timing.then_some(gpu_time_us);
        subsystem.over_budget = subsystem
            .gpu_time_us
            .is_some_and(|gpu_time_us| gpu_time_us > budget.budget_us(subsystem.key));
    }
}

fn gpu_budget_warning_count(profile: &RenderFrameProfile, budget: &RenderFrameBudget) -> u32 {
    let subsystem_warning_count = profile
        .subsystems
        .iter()
        .filter(|subsystem| subsystem.over_budget)
        .count();
    let frame_warning_count = profile
        .gpu_frame_time_us
        .is_some_and(|gpu_time_us| gpu_time_us > budget.total_budget_us())
        as usize;
    saturating_u32(subsystem_warning_count.saturating_add(frame_warning_count))
}

fn saturating_u32_from_u64(value: u64) -> u32 {
    value.min(u64::from(u32::MAX)) as u32
}
