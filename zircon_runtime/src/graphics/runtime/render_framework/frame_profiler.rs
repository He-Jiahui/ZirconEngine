//! 帧分析器合并 CPU 提交、GPU 查询和预算诊断，发布时须对应同一代际的完成票据。
use crate::core::framework::render::{
    RenderBudgetKey, RenderFrameBudget, RenderFrameProfile, RenderGpuTimingStatus,
    RenderPassProfileEntry, RenderStats, RenderSubsystemProfileEntry,
};
use crate::graphics::backend::{GpuPipelineStatisticsFrameResult, GpuTimerFrameResult};
use std::{collections::VecDeque, sync::Arc, time::Instant};

use super::budget::{memory_budget_warning_count, BudgetDegradeLadder, GpuMemoryBudget};

mod gpu_resolution;
mod mesh_submission;

pub(in crate::graphics::runtime::render_framework) use gpu_resolution::FrameProfileWrite;
use mesh_submission::mesh_submission_profile;

const MAX_PENDING_FRAME_PROFILES: usize = 4;

pub(in crate::graphics::runtime::render_framework) struct FrameProfiler {
    budget: RenderFrameBudget,
    last_compiled_graph_cache_hit_count: usize,
    pending_profiles: VecDeque<Arc<RenderFrameProfile>>,
}

impl Default for FrameProfiler {
    fn default() -> Self {
        Self {
            budget: RenderFrameBudget::reference_1080p_mid(),
            last_compiled_graph_cache_hit_count: 0,
            // Keep the timer's three in-flight frames plus the profile being assembled.
            pending_profiles: VecDeque::with_capacity(MAX_PENDING_FRAME_PROFILES),
        }
    }
}

impl FrameProfiler {
    pub(in crate::graphics::runtime::render_framework) fn elapsed_micros(
        submit_started: Instant,
    ) -> u64 {
        submit_started.elapsed().as_micros().min(u64::MAX.into()) as u64
    }

    pub(in crate::graphics::runtime::render_framework) fn write_frame_profile(
        &mut self,
        stats: &mut RenderStats,
        frame_generation: u64,
        cpu_submit_time_us: u64,
        gpu_timing_status: RenderGpuTimingStatus,
        gpu_timer_frame_result: Option<&GpuTimerFrameResult>,
        gpu_pipeline_statistics_frame_result: Option<&GpuPipelineStatisticsFrameResult>,
        memory_budget: &GpuMemoryBudget,
        degrade_ladder: &mut BudgetDegradeLadder,
        store_lint_count: u32,
        persistent_texture_resident_bytes: u64,
    ) -> FrameProfileWrite {
        let compiled_graph_cache_hit =
            stats.last_graph_compiled_cache_hit_count > self.last_compiled_graph_cache_hit_count;
        self.last_compiled_graph_cache_hit_count = stats.last_graph_compiled_cache_hit_count;

        let passes = stats
            .last_graph_execution_profile_report
            .pass_profiles
            .iter()
            .map(|record| RenderPassProfileEntry {
                pass_name: record.pass_name.clone(),
                executor_id: record.executor_id.clone(),
                budget_key: record.budget_key,
                cpu_elapsed_micros: record.cpu_elapsed_micros,
                gpu_time_us: None,
                pipeline_statistics: None,
                draw_count: record.draw_count,
                instance_count: record.instance_count,
                state_change_count: record.state_change_count,
                upload_bytes: record.upload_bytes,
                dispatch_count: record.dispatch_count,
                native_resource_creates: record.native_resource_creates,
            })
            .collect::<Vec<_>>();
        let staging_total_bytes = passes
            .iter()
            .map(|pass| pass.upload_bytes)
            .fold(0_u64, u64::saturating_add);
        let mut pending_profile = RenderFrameProfile {
            frame_generation,
            gpu_frame_time_us: None,
            gpu_timing_status,
            cpu_submit_time_us,
            parallel_recording_eligible_stage_count: saturating_u32(
                stats
                    .last_graph_parallel_recording_report
                    .eligible_stage_count,
            ),
            parallel_recording_eligible_bucket_count: saturating_u32(
                stats
                    .last_graph_parallel_recording_report
                    .eligible_bucket_count,
            ),
            parallel_recording_executed_stage_count: saturating_u32(
                stats
                    .last_graph_parallel_recording_report
                    .executed_stage_count,
            ),
            parallel_recording_executed_bucket_count: saturating_u32(
                stats
                    .last_graph_parallel_recording_report
                    .executed_bucket_count,
            ),
            profile_latency_frames: 0,
            passes,
            subsystems: RenderBudgetKey::ALL
                .into_iter()
                .map(|key| RenderSubsystemProfileEntry {
                    key,
                    gpu_time_us: None,
                    budget_us: self.budget.budget_us(key),
                    over_budget: false,
                })
                .collect(),
            mesh_submission: mesh_submission_profile(stats),
            transient_texture_peak_bytes: stats.last_graph_transient_texture_bytes_reserved,
            transient_buffer_peak_bytes: stats.last_graph_transient_buffer_bytes_reserved,
            staging_total_bytes,
            persistent_texture_resident_bytes,
            compiled_graph_cache_hit,
            variant_miss_count: saturating_u32(
                stats.last_shader_variant_miss_report.compile_miss_count,
            ),
            store_lint_count,
            budget_warning_count: 0,
            degrade_step_active: 0,
        };
        pending_profile.budget_warning_count =
            memory_budget_warning_count(&pending_profile, *memory_budget);
        degrade_ladder.evaluate(&pending_profile, memory_budget);
        pending_profile.degrade_step_active = saturating_u32(degrade_ladder.active_level());
        let pending_profile = Arc::new(pending_profile);
        let capture_profile = Arc::clone(&pending_profile);
        self.pending_profiles.push_back(pending_profile);

        // Merge before eviction: the oldest of three in-flight readbacks can resolve while the
        // fourth profile is being assembled, and must remain addressable for this update.
        let mut resolved_gpu_generations = [
            gpu_timer_frame_result
                .and_then(|result| self.merge_gpu_timer_result(result, frame_generation)),
            gpu_pipeline_statistics_frame_result.and_then(|result| {
                self.merge_gpu_pipeline_statistics_result(result, frame_generation)
            }),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        resolved_gpu_generations.sort_unstable();
        resolved_gpu_generations.dedup();
        let resolved_gpu_profiles = resolved_gpu_generations
            .into_iter()
            .filter_map(|generation| {
                self.pending_profiles
                    .iter()
                    .find(|profile| profile.frame_generation == generation)
                    .cloned()
            })
            .collect::<Vec<_>>();
        let resolved_gpu_profile = resolved_gpu_profiles.last().cloned();
        while self.pending_profiles.len() > MAX_PENDING_FRAME_PROFILES {
            self.pending_profiles.pop_front();
        }
        stats.last_budget_warning_count = capture_profile.budget_warning_count;
        stats.last_store_lint_count = capture_profile.store_lint_count;
        stats.last_frame_profile = Arc::clone(&capture_profile);
        if let Some(profile) = &resolved_gpu_profile {
            stats.last_resolved_gpu_frame_profile = Some(Arc::clone(profile));
        }

        FrameProfileWrite {
            capture_profile,
            resolved_gpu_profiles,
            resolved_gpu_profile,
        }
    }
}

fn saturating_u32(value: usize) -> u32 {
    value.min(u32::MAX as usize) as u32
}

#[cfg(test)]
#[path = "tests/frame_profiler.rs"]
mod tests;
