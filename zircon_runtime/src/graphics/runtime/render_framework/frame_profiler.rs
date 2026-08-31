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
mod tests {
    use std::sync::Arc;

    use crate::core::framework::render::{
        RenderBudgetKey, RenderGpuTimingStatus, RenderGraphExecutionProfileReport,
        RenderGraphParallelRecordingReport, RenderGraphPassProfileMetrics,
        RenderGraphPassProfileRecord, RenderStats,
    };
    use crate::graphics::backend::{
        GpuPassPipelineStatistics, GpuPassTiming, GpuPipelineStatistics,
        GpuPipelineStatisticsFrameResult, GpuTimerFrameResult,
    };

    use super::{FrameProfileWrite, FrameProfiler};
    use crate::graphics::runtime::render_framework::budget::{
        BudgetDegradeLadder, GpuMemoryBudget,
    };

    fn write_profile(
        profiler: &mut FrameProfiler,
        stats: &mut RenderStats,
        frame_generation: u64,
        cpu_submit_time_us: u64,
        gpu_timer_frame_result: Option<&GpuTimerFrameResult>,
    ) -> FrameProfileWrite {
        write_profile_with_gpu_timing_status(
            profiler,
            stats,
            frame_generation,
            cpu_submit_time_us,
            RenderGpuTimingStatus::Disabled,
            gpu_timer_frame_result,
        )
    }

    fn write_profile_with_gpu_timing_status(
        profiler: &mut FrameProfiler,
        stats: &mut RenderStats,
        frame_generation: u64,
        cpu_submit_time_us: u64,
        gpu_timing_status: RenderGpuTimingStatus,
        gpu_timer_frame_result: Option<&GpuTimerFrameResult>,
    ) -> FrameProfileWrite {
        let memory_budget = GpuMemoryBudget::default();
        let mut degrade_ladder = BudgetDegradeLadder::default();
        profiler.write_frame_profile(
            stats,
            frame_generation,
            cpu_submit_time_us,
            gpu_timing_status,
            gpu_timer_frame_result,
            None,
            &memory_budget,
            &mut degrade_ladder,
            0,
            0,
        )
    }

    #[test]
    fn render_perf_frame_profile_matches_flat_stats() {
        let mut stats = RenderStats::default();
        stats.last_graph_execution_profile_report = RenderGraphExecutionProfileReport::new(vec![
            RenderGraphPassProfileRecord::new("opaque", "mesh.opaque", 41)
                .with_budget_key(RenderBudgetKey::BasePass)
                .with_render_metrics(RenderGraphPassProfileMetrics::new(5, 0, 9)),
            RenderGraphPassProfileRecord::new("ui", "ui", 7)
                .with_budget_key(RenderBudgetKey::Ui)
                .with_compute_metrics(2, 192),
        ]);
        stats.last_graph_executed_pass_count = 2;
        stats.last_graph_executed_passes = vec!["opaque".to_owned(), "ui".to_owned()];
        stats.last_graph_transient_texture_bytes_reserved = 128;
        stats.last_graph_transient_buffer_bytes_reserved = 64;
        stats.last_graph_transient_dense_bytes_reserved = 192;
        stats.last_shader_variant_miss_report.compile_miss_count = 3;
        stats.last_graph_parallel_recording_report =
            RenderGraphParallelRecordingReport::new(1, 3, 1, 2);

        let mut profiler = FrameProfiler::default();
        let capture_profile =
            write_profile(&mut profiler, &mut stats, 17, 1234, None).capture_profile;

        let profile = &stats.last_frame_profile;
        assert_eq!(profile.frame_generation, 17);
        assert_eq!(profile.cpu_submit_time_us, 1234);
        assert_eq!(profile.gpu_frame_time_us, None);
        assert_eq!(profile.profile_latency_frames, 0);
        assert_eq!(profile.transient_texture_peak_bytes, 128);
        assert_eq!(profile.transient_buffer_peak_bytes, 64);
        assert_eq!(
            profile
                .transient_texture_peak_bytes
                .saturating_add(profile.transient_buffer_peak_bytes),
            stats.last_graph_transient_dense_bytes_reserved
        );
        assert_eq!(profile.variant_miss_count, 3);
        assert_eq!(profile.parallel_recording_eligible_stage_count, 1);
        assert_eq!(profile.parallel_recording_eligible_bucket_count, 3);
        assert_eq!(profile.parallel_recording_executed_stage_count, 1);
        assert_eq!(profile.parallel_recording_executed_bucket_count, 2);
        assert_eq!(profile.passes.len(), 2);
        assert_eq!(profile.passes.len(), stats.last_graph_executed_pass_count);
        assert_eq!(
            profile
                .passes
                .iter()
                .map(|pass| pass.pass_name.as_str())
                .collect::<Vec<_>>(),
            stats
                .last_graph_executed_passes
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        );
        assert_eq!(profile.passes[0].pass_name, "opaque");
        assert_eq!(profile.passes[0].budget_key, RenderBudgetKey::BasePass);
        assert_eq!(profile.passes[0].cpu_elapsed_micros, 41);
        assert_eq!(profile.passes[0].draw_count, 5);
        assert_eq!(profile.passes[0].instance_count, 0);
        assert_eq!(profile.passes[0].state_change_count, 9);
        assert_eq!(profile.passes[1].pass_name, "ui");
        assert_eq!(profile.passes[1].budget_key, RenderBudgetKey::Ui);
        assert_eq!(profile.passes[1].cpu_elapsed_micros, 7);
        assert_eq!(profile.passes[1].dispatch_count, 2);
        assert_eq!(profile.passes[1].upload_bytes, 192);
        assert!(profile.passes.iter().all(|pass| pass.gpu_time_us.is_none()));
        assert_eq!(profile.subsystems.len(), RenderBudgetKey::ALL.len());
        assert!(profile.subsystems.iter().all(|entry| !entry.over_budget));
        assert!(Arc::ptr_eq(&capture_profile, &stats.last_frame_profile));
        assert_eq!(capture_profile, stats.last_frame_profile);
        let copied_stats = stats.clone();
        assert!(Arc::ptr_eq(
            &stats.last_frame_profile,
            &copied_stats.last_frame_profile
        ));
    }

    #[test]
    fn cache_hit_is_observed_once_per_stats_update() {
        let mut stats = RenderStats::default();
        let mut profiler = FrameProfiler::default();

        write_profile(&mut profiler, &mut stats, 1, 1, None);
        assert!(!stats.last_frame_profile.compiled_graph_cache_hit);

        stats.last_graph_compiled_cache_hit_count = 1;
        write_profile(&mut profiler, &mut stats, 2, 1, None);
        assert!(stats.last_frame_profile.compiled_graph_cache_hit);

        write_profile(&mut profiler, &mut stats, 3, 1, None);
        assert!(!stats.last_frame_profile.compiled_graph_cache_hit);
    }

    #[test]
    fn render_perf_gpu_timer_latency_within_three_frames() {
        let mut stats = RenderStats::default();
        stats.last_graph_execution_profile_report =
            RenderGraphExecutionProfileReport::new(vec![RenderGraphPassProfileRecord::new(
                "opaque",
                "mesh.opaque",
                1,
            )
            .with_budget_key(RenderBudgetKey::BasePass)]);
        let mut profiler = FrameProfiler::default();

        let first_capture = write_profile(&mut profiler, &mut stats, 7, 111, None).capture_profile;
        assert_eq!(first_capture.frame_generation, 7);
        assert_eq!(first_capture.gpu_frame_time_us, None);

        let delayed_result = GpuTimerFrameResult {
            frame_generation: 7,
            pass_timings: vec![GpuPassTiming {
                pass_name: "opaque".to_owned(),
                gpu_time_us: 4_000,
            }],
        };
        let write = write_profile(&mut profiler, &mut stats, 9, 222, Some(&delayed_result));
        let current_capture = write.capture_profile;

        assert_eq!(
            first_capture.gpu_frame_time_us, None,
            "a late GPU result must not mutate the snapshot previously returned to a caller"
        );
        assert_eq!(current_capture.frame_generation, 9);
        assert_eq!(current_capture.gpu_frame_time_us, None);
        assert_eq!(stats.last_frame_profile.frame_generation, 9);
        assert_eq!(stats.last_frame_profile.gpu_frame_time_us, None);
        let resolved = stats
            .last_resolved_gpu_frame_profile
            .as_ref()
            .expect("late GPU result remains queryable separately");
        assert_eq!(resolved.frame_generation, 7);
        assert_eq!(resolved.gpu_frame_time_us, Some(4_000));
        assert_eq!(resolved.profile_latency_frames, 2);
        assert_eq!(resolved.passes[0].gpu_time_us, Some(4_000));
        let base_pass = stats
            .last_resolved_gpu_frame_profile
            .as_ref()
            .expect("late GPU result remains queryable separately")
            .subsystems
            .iter()
            .find(|subsystem| subsystem.key == RenderBudgetKey::BasePass)
            .expect("base-pass budget entry exists");
        assert_eq!(base_pass.gpu_time_us, Some(4_000));
        assert!(base_pass.over_budget);
        assert_eq!(stats.last_budget_warning_count, 0);

        write_profile(&mut profiler, &mut stats, 10, 333, None);
        assert_eq!(stats.last_frame_profile.frame_generation, 10);
        assert_eq!(
            stats
                .last_resolved_gpu_frame_profile
                .as_ref()
                .expect("last resolved GPU profile persists until replaced")
                .frame_generation,
            7
        );
    }

    #[test]
    fn capacity_limited_timer_result_never_becomes_a_comparable_measurement() {
        let mut stats = RenderStats::default();
        stats.last_graph_execution_profile_report =
            RenderGraphExecutionProfileReport::new(vec![RenderGraphPassProfileRecord::new(
                "opaque",
                "mesh.opaque",
                1,
            )
            .with_budget_key(RenderBudgetKey::BasePass)]);
        let mut profiler = FrameProfiler::default();

        let capture = write_profile_with_gpu_timing_status(
            &mut profiler,
            &mut stats,
            7,
            10,
            RenderGpuTimingStatus::CapacityExhausted,
            None,
        )
        .capture_profile;
        assert_eq!(
            capture.gpu_timing_status,
            RenderGpuTimingStatus::CapacityExhausted
        );

        let result = GpuTimerFrameResult {
            frame_generation: 7,
            pass_timings: vec![GpuPassTiming {
                pass_name: "opaque".to_owned(),
                gpu_time_us: 17,
            }],
        };
        let resolved = write_profile(&mut profiler, &mut stats, 8, 10, Some(&result))
            .resolved_gpu_profile
            .expect("partial timer data still backfills its source generation");

        assert_eq!(resolved.gpu_frame_time_us, Some(17));
        assert_eq!(
            resolved.gpu_timing_status,
            RenderGpuTimingStatus::CapacityExhausted
        );
    }

    #[test]
    fn gpu_timer_matches_repeated_graph_pass_names_by_occurrence() {
        let mut stats = RenderStats::default();
        stats.last_graph_execution_profile_report = RenderGraphExecutionProfileReport::new(vec![
            RenderGraphPassProfileRecord::new("blur", "blur.horizontal", 1)
                .with_budget_key(RenderBudgetKey::PostProcess),
            RenderGraphPassProfileRecord::new("blur", "blur.vertical", 1)
                .with_budget_key(RenderBudgetKey::PostProcess),
        ]);
        let mut profiler = FrameProfiler::default();

        write_profile(&mut profiler, &mut stats, 5, 10, None);
        let delayed_result = GpuTimerFrameResult {
            frame_generation: 5,
            pass_timings: vec![
                GpuPassTiming {
                    pass_name: "blur".to_owned(),
                    gpu_time_us: 11,
                },
                GpuPassTiming {
                    pass_name: "blur".to_owned(),
                    gpu_time_us: 29,
                },
            ],
        };

        let resolved = write_profile(&mut profiler, &mut stats, 6, 10, Some(&delayed_result))
            .resolved_gpu_profile
            .expect("delayed repeated-name timings resolve the prior profile");

        assert_eq!(resolved.gpu_frame_time_us, Some(40));
        assert_eq!(resolved.passes[0].gpu_time_us, Some(11));
        assert_eq!(resolved.passes[1].gpu_time_us, Some(29));
    }

    #[test]
    fn oldest_in_flight_profile_merges_before_pending_ring_eviction() {
        let mut stats = RenderStats::default();
        stats.last_graph_execution_profile_report =
            RenderGraphExecutionProfileReport::new(vec![RenderGraphPassProfileRecord::new(
                "opaque",
                "mesh.opaque",
                1,
            )
            .with_budget_key(RenderBudgetKey::BasePass)]);
        let mut profiler = FrameProfiler::default();
        for generation in 1..=4 {
            write_profile(&mut profiler, &mut stats, generation, 10, None);
        }
        let first_result = GpuTimerFrameResult {
            frame_generation: 1,
            pass_timings: vec![GpuPassTiming {
                pass_name: "opaque".to_owned(),
                gpu_time_us: 17,
            }],
        };

        let write = write_profile(&mut profiler, &mut stats, 5, 10, Some(&first_result));

        assert_eq!(write.capture_profile.frame_generation, 5);
        let resolved = write
            .resolved_gpu_profile
            .expect("the oldest timer result is merged before eviction");
        assert_eq!(resolved.frame_generation, 1);
        assert_eq!(resolved.gpu_frame_time_us, Some(17));
        assert_eq!(stats.last_frame_profile.frame_generation, 5);
    }

    #[test]
    fn pipeline_statistics_merge_is_available_without_changing_gpu_time_budgeting() {
        let mut stats = RenderStats::default();
        stats.last_graph_execution_profile_report =
            RenderGraphExecutionProfileReport::new(vec![RenderGraphPassProfileRecord::new(
                "hzb.build",
                "hzb.build",
                1,
            )
            .with_budget_key(RenderBudgetKey::Hzb)]);
        let mut profiler = FrameProfiler::default();

        write_profile(&mut profiler, &mut stats, 3, 10, None);
        let statistics = GpuPipelineStatisticsFrameResult {
            frame_generation: 3,
            pass_statistics: vec![GpuPassPipelineStatistics {
                pass_name: "hzb.build".to_string(),
                statistics: GpuPipelineStatistics {
                    compute_shader_invocations: 512,
                    ..GpuPipelineStatistics::default()
                },
            }],
        };
        let memory_budget = GpuMemoryBudget::default();
        let mut degrade_ladder = BudgetDegradeLadder::default();
        let write = profiler.write_frame_profile(
            &mut stats,
            4,
            10,
            RenderGpuTimingStatus::Disabled,
            None,
            Some(&statistics),
            &memory_budget,
            &mut degrade_ladder,
            0,
            0,
        );

        let resolved = write
            .resolved_gpu_profile
            .expect("late pipeline statistics should resolve the prior profile");
        assert_eq!(resolved.frame_generation, 3);
        assert_eq!(resolved.gpu_frame_time_us, None);
        assert_eq!(
            resolved.passes[0]
                .pipeline_statistics
                .as_ref()
                .expect("HZB statistics should be attached")
                .compute_shader_invocations,
            512
        );
    }

    #[test]
    fn simultaneous_late_gpu_results_backfill_each_matching_capture() {
        let mut stats = RenderStats::default();
        stats.last_graph_execution_profile_report =
            RenderGraphExecutionProfileReport::new(vec![RenderGraphPassProfileRecord::new(
                "opaque",
                "mesh.opaque",
                1,
            )
            .with_budget_key(RenderBudgetKey::BasePass)]);
        let mut profiler = FrameProfiler::default();
        write_profile(&mut profiler, &mut stats, 4, 10, None);
        write_profile(&mut profiler, &mut stats, 5, 10, None);
        let timer = GpuTimerFrameResult {
            frame_generation: 4,
            pass_timings: vec![GpuPassTiming {
                pass_name: "opaque".to_owned(),
                gpu_time_us: 17,
            }],
        };
        let statistics = GpuPipelineStatisticsFrameResult {
            frame_generation: 5,
            pass_statistics: vec![GpuPassPipelineStatistics {
                pass_name: "opaque".to_owned(),
                statistics: GpuPipelineStatistics {
                    compute_shader_invocations: 512,
                    ..GpuPipelineStatistics::default()
                },
            }],
        };
        let memory_budget = GpuMemoryBudget::default();
        let mut degrade_ladder = BudgetDegradeLadder::default();

        let write = profiler.write_frame_profile(
            &mut stats,
            6,
            10,
            RenderGpuTimingStatus::Disabled,
            Some(&timer),
            Some(&statistics),
            &memory_budget,
            &mut degrade_ladder,
            0,
            0,
        );

        assert_eq!(
            write
                .resolved_gpu_profiles
                .iter()
                .map(|profile| profile.frame_generation)
                .collect::<Vec<_>>(),
            vec![4, 5]
        );
        assert_eq!(write.resolved_gpu_profiles[0].gpu_frame_time_us, Some(17));
        assert_eq!(
            write.resolved_gpu_profiles[1].passes[0]
                .pipeline_statistics
                .as_ref()
                .expect("pipeline statistics should be attached to generation five")
                .compute_shader_invocations,
            512
        );
        assert_eq!(
            write
                .resolved_gpu_profile
                .as_ref()
                .map(|profile| profile.frame_generation),
            Some(5)
        );
    }

    #[test]
    fn same_generation_timer_and_statistics_share_one_resolved_profile() {
        let mut stats = RenderStats::default();
        stats.last_graph_execution_profile_report =
            RenderGraphExecutionProfileReport::new(vec![RenderGraphPassProfileRecord::new(
                "opaque",
                "mesh.opaque",
                1,
            )
            .with_budget_key(RenderBudgetKey::BasePass)]);
        let mut profiler = FrameProfiler::default();
        write_profile(&mut profiler, &mut stats, 4, 10, None);
        let timer = GpuTimerFrameResult {
            frame_generation: 4,
            pass_timings: vec![GpuPassTiming {
                pass_name: "opaque".to_owned(),
                gpu_time_us: 17,
            }],
        };
        let statistics = GpuPipelineStatisticsFrameResult {
            frame_generation: 4,
            pass_statistics: vec![GpuPassPipelineStatistics {
                pass_name: "opaque".to_owned(),
                statistics: GpuPipelineStatistics {
                    compute_shader_invocations: 512,
                    ..GpuPipelineStatistics::default()
                },
            }],
        };
        let memory_budget = GpuMemoryBudget::default();
        let mut degrade_ladder = BudgetDegradeLadder::default();

        let write = profiler.write_frame_profile(
            &mut stats,
            5,
            10,
            RenderGpuTimingStatus::Disabled,
            Some(&timer),
            Some(&statistics),
            &memory_budget,
            &mut degrade_ladder,
            0,
            0,
        );

        assert_eq!(write.resolved_gpu_profiles.len(), 1);
        let resolved = &write.resolved_gpu_profiles[0];
        assert_eq!(resolved.frame_generation, 4);
        assert_eq!(resolved.gpu_frame_time_us, Some(17));
        assert_eq!(
            resolved.passes[0]
                .pipeline_statistics
                .as_ref()
                .expect("same-generation pipeline statistics should be retained")
                .compute_shader_invocations,
            512
        );
    }

    #[test]
    fn pipeline_statistics_match_repeated_graph_pass_names_by_occurrence() {
        let mut stats = RenderStats::default();
        stats.last_graph_execution_profile_report = RenderGraphExecutionProfileReport::new(vec![
            RenderGraphPassProfileRecord::new("blur", "blur.horizontal", 1),
            RenderGraphPassProfileRecord::new("blur", "blur.vertical", 1),
        ]);
        let mut profiler = FrameProfiler::default();

        write_profile(&mut profiler, &mut stats, 3, 10, None);
        let statistics = GpuPipelineStatisticsFrameResult {
            frame_generation: 3,
            pass_statistics: vec![
                GpuPassPipelineStatistics {
                    pass_name: "blur".to_owned(),
                    statistics: GpuPipelineStatistics {
                        compute_shader_invocations: 11,
                        ..GpuPipelineStatistics::default()
                    },
                },
                GpuPassPipelineStatistics {
                    pass_name: "blur".to_owned(),
                    statistics: GpuPipelineStatistics {
                        compute_shader_invocations: 29,
                        ..GpuPipelineStatistics::default()
                    },
                },
            ],
        };
        let memory_budget = GpuMemoryBudget::default();
        let mut degrade_ladder = BudgetDegradeLadder::default();

        let resolved = profiler
            .write_frame_profile(
                &mut stats,
                4,
                10,
                RenderGpuTimingStatus::Disabled,
                None,
                Some(&statistics),
                &memory_budget,
                &mut degrade_ladder,
                0,
                0,
            )
            .resolved_gpu_profile
            .expect("repeated-name pipeline statistics resolve the prior profile");

        assert_eq!(
            resolved
                .passes
                .iter()
                .map(|pass| {
                    pass.pipeline_statistics
                        .as_ref()
                        .map(|statistics| statistics.compute_shader_invocations)
                })
                .collect::<Vec<_>>(),
            vec![Some(11), Some(29)]
        );
    }

    #[test]
    fn render_perf_profile_feeds_memory_budget_lint_and_degrade_state() {
        let mut stats = RenderStats::default();
        stats.last_graph_execution_profile_report =
            RenderGraphExecutionProfileReport::new(vec![RenderGraphPassProfileRecord::new(
                "upload", "upload", 1,
            )
            .with_compute_metrics(1, 128)]);
        stats.last_graph_transient_texture_bytes_reserved = 65;
        let memory_budget = GpuMemoryBudget::new(64, 64, 64);
        let mut degrade_ladder = BudgetDegradeLadder::with_hysteresis_frames(2);
        let mut profiler = FrameProfiler::default();

        let write = profiler.write_frame_profile(
            &mut stats,
            11,
            10,
            RenderGpuTimingStatus::Disabled,
            None,
            None,
            &memory_budget,
            &mut degrade_ladder,
            3,
            0,
        );

        assert_eq!(write.capture_profile.staging_total_bytes, 128);
        assert_eq!(write.capture_profile.store_lint_count, 3);
        assert_eq!(write.capture_profile.budget_warning_count, 2);
        assert_eq!(write.capture_profile.degrade_step_active, 1);
        assert_eq!(stats.last_budget_warning_count, 2);
        assert_eq!(stats.last_store_lint_count, 3);
    }
}
