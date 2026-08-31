use super::super::super::frame_profiler::FrameProfileWrite;
use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::submission_record_update::SubmissionRecordUpdate;
use super::base_stats::update_base_stats;
use super::hybrid_gi_stats::{reset_hybrid_gi_stats, update_hybrid_gi_stats};
use super::particle_stats::update_particle_stats;
use super::quality_profile::update_quality_profile;
use super::shared_product_reports::SharedViewportProductReports;
use super::virtual_geometry_stats::{reset_virtual_geometry_stats, update_virtual_geometry_stats};
pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn update_stats(
    state: &mut RenderFrameworkState,
    context: &FrameSubmissionContext,
    record_update: &SubmissionRecordUpdate,
    frame_generation: u64,
    cpu_submit_time_us: u64,
    shared_product_reports: SharedViewportProductReports,
) -> FrameProfileWrite {
    update_base_stats(
        state,
        context,
        record_update,
        frame_generation,
        shared_product_reports,
    );
    update_particle_stats(state, record_update);

    if context.hybrid_gi_enabled() {
        update_hybrid_gi_stats(state, context, record_update);
    } else {
        reset_hybrid_gi_stats(state);
    }

    if context.virtual_geometry_enabled() {
        update_virtual_geometry_stats(state, context, record_update);
    } else {
        reset_virtual_geometry_stats(state);
    }

    update_quality_profile(state, context);
    let store_lint_count = context
        .compiled_pipeline()
        .graph()
        .store_lint_report()
        .count();
    let store_lint_count = store_lint_count.min(u32::MAX as usize) as u32;
    let persistent_texture_resident_bytes = state.renderer.persistent_texture_resident_bytes();
    let RenderFrameworkState {
        renderer,
        stats,
        frame_profiler,
        memory_budget,
        degrade_ladder,
        ..
    } = state;
    let gpu_timing_status = renderer.last_gpu_timing_status();
    let (gpu_timer_frame_result, gpu_pipeline_statistics_frame_result) =
        borrowed_gpu_profile_reports(
            renderer.last_gpu_timer_frame_result(),
            renderer.last_gpu_pipeline_statistics_frame_result(),
        );
    frame_profiler.write_frame_profile(
        stats,
        frame_generation,
        cpu_submit_time_us,
        gpu_timing_status,
        gpu_timer_frame_result,
        gpu_pipeline_statistics_frame_result,
        memory_budget,
        degrade_ladder,
        store_lint_count,
        persistent_texture_resident_bytes,
    )
}

fn borrowed_gpu_profile_reports<'a>(
    timer: Option<&'a crate::graphics::backend::GpuTimerFrameResult>,
    statistics: Option<&'a crate::graphics::backend::GpuPipelineStatisticsFrameResult>,
) -> (
    Option<&'a crate::graphics::backend::GpuTimerFrameResult>,
    Option<&'a crate::graphics::backend::GpuPipelineStatisticsFrameResult>,
) {
    (timer, statistics)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::graphics::backend::{
        GpuPassPipelineStatistics, GpuPassTiming, GpuPipelineStatistics,
        GpuPipelineStatisticsFrameResult, GpuTimerFrameResult,
    };

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const REPORT_READS_PER_SAMPLE: usize = 16_384;

    #[test]
    fn optimization_batch_fd_runtime462_gpu_profile_reports_are_borrowed() {
        let (timer, statistics) = representative_reports();
        let (borrowed_timer, borrowed_statistics) =
            borrowed_gpu_profile_reports(Some(&timer), Some(&statistics));

        assert!(std::ptr::eq(borrowed_timer.unwrap(), &timer));
        assert!(std::ptr::eq(borrowed_statistics.unwrap(), &statistics));
        assert_eq!(borrowed_gpu_profile_reports(None, None), (None, None));

        let production = include_str!("update.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        assert!(!production.contains("last_gpu_timer_frame_result().cloned()"));
        assert!(
            !production.contains("last_gpu_pipeline_statistics_frame_result()\n        .cloned()")
        );
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fd_runtime462_borrowed_gpu_profile_reports_benchmark() {
        let (timer, statistics) = representative_reports();
        for _ in 0..4 {
            black_box(measure_legacy(&timer, &statistics));
            black_box(measure_optimized(&timer, &statistics));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_legacy(&timer, &statistics));
                optimized_samples.push(measure_optimized(&timer, &statistics));
            } else {
                optimized_samples.push(measure_optimized(&timer, &statistics));
                legacy_samples.push(measure_legacy(&timer, &statistics));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn representative_reports() -> (GpuTimerFrameResult, GpuPipelineStatisticsFrameResult) {
        let pass_timings = (0..16)
            .map(|index| GpuPassTiming {
                pass_name: format!("pass.{index}"),
                gpu_time_us: 100 + index,
            })
            .collect();
        let pass_statistics = (0..16)
            .map(|index| GpuPassPipelineStatistics {
                pass_name: format!("pass.{index}"),
                statistics: GpuPipelineStatistics {
                    vertex_shader_invocations: 10_000 + index,
                    fragment_shader_invocations: 20_000 + index,
                    compute_shader_invocations: 30_000 + index,
                    ..GpuPipelineStatistics::default()
                },
            })
            .collect();
        (
            GpuTimerFrameResult {
                frame_generation: 42,
                pass_timings,
            },
            GpuPipelineStatisticsFrameResult {
                frame_generation: 42,
                pass_statistics,
            },
        )
    }

    fn measure_legacy(
        timer: &GpuTimerFrameResult,
        statistics: &GpuPipelineStatisticsFrameResult,
    ) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..REPORT_READS_PER_SAMPLE {
            let timer = black_box(Some(timer)).cloned();
            let statistics = black_box(Some(statistics)).cloned();
            checksum = checksum
                .wrapping_add(timer.as_ref().unwrap().pass_timings.len())
                .wrapping_add(statistics.as_ref().unwrap().pass_statistics.len());
            black_box((timer, statistics));
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(
        timer: &GpuTimerFrameResult,
        statistics: &GpuPipelineStatisticsFrameResult,
    ) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..REPORT_READS_PER_SAMPLE {
            let reports =
                borrowed_gpu_profile_reports(black_box(Some(timer)), black_box(Some(statistics)));
            checksum = checksum
                .wrapping_add(reports.0.unwrap().pass_timings.len())
                .wrapping_add(reports.1.unwrap().pass_statistics.len());
            black_box(reports);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME462_BORROWED_GPU_PROFILE_REPORTS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} report_reads_per_sample={REPORT_READS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=90",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95 / 10,
            "borrowed GPU profile reports must reduce P95 by at least 90%"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
