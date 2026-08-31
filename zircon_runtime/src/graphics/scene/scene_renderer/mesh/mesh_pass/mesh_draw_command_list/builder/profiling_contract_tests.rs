const BUILDER_SOURCE: &str = include_str!("../builder.rs");
const PARALLEL_ADMISSION_SOURCE: &str = include_str!("parallel_admission.rs");
const PARALLEL_PREPARATION_SOURCE: &str = include_str!("parallel_preparation.rs");

use super::parallel_admission::ParallelPreparationMode;

#[test]
fn cached_prepare_profile_stages_preserve_owner_boundaries() {
    let product_serial_entry = source_between(
        BUILDER_SOURCE,
        "pub(crate) fn build_mesh_pass_command_buffers_cached",
        "pub(crate) fn build_mesh_pass_command_buffers_cached_parallel",
    );
    let generic_serial_entry = source_between(
        BUILDER_SOURCE,
        "pub(super) fn build_mesh_pass_command_buffers_from_batches_cached",
        "fn build_mesh_pass_command_buffers_from_ordered_batches_cached",
    );
    let serial_builder = source_between(
        BUILDER_SOURCE,
        "fn build_mesh_pass_command_buffers_from_ordered_batches_cached",
        "pub(super) fn build_mesh_pass_command_buffers_from_batches_cached_parallel",
    );
    let parallel_builder = source_between(
        PARALLEL_PREPARATION_SOURCE,
        "pub(super) fn build_mesh_pass_command_buffers_from_batches_cached_parallel",
        "fn prepare_batch_plan",
    );
    let source_order_normalization = source_between(
        BUILDER_SOURCE,
        "fn collect_batches_in_source_order",
        "fn prepare_batch_plan",
    );

    assert!(product_serial_entry
        .contains("build_mesh_pass_command_buffers_from_ordered_batches_cached"));
    assert!(!product_serial_entry.contains("collect_batches_in_source_order"));
    assert_stage_order(
        generic_serial_entry,
        &[
            "collect_batches_in_source_order",
            "build_mesh_pass_command_buffers_from_ordered_batches_cached",
        ],
    );
    assert_stage_order(
        serial_builder,
        &[
            "prepare_cached_serial",
            "serial_prepare_and_project",
            "record_preparation_result_profile",
            "seal_phase_buffers",
        ],
    );
    assert_stage_order(
        parallel_builder,
        &[
            "collect_batches_in_source_order",
            "parallel_admission",
            "owner_transaction",
            "worker_projection_wait",
            "ordered_merge",
            "record_preparation_result_profile",
            "seal_phase_buffers",
        ],
    );

    let builder_sources = format!("{BUILDER_SOURCE}\n{PARALLEL_PREPARATION_SOURCE}");
    for stage in [
        "normalize_source_order",
        "serial_prepare_and_project",
        "parallel_admission",
        "owner_transaction",
        "worker_projection_wait",
        "ordered_merge",
        "seal_phase_buffers",
    ] {
        let needle = format!("\"{stage}\"");
        assert_eq!(
            builder_sources.matches(needle.as_str()).count(),
            if stage == "seal_phase_buffers" { 2 } else { 1 },
            "profiling stage must have the expected single owner: {stage}"
        );
    }
    assert!(source_order_normalization.contains("\"normalize_source_order\""));
}

#[test]
fn cached_prepare_profile_counters_remain_single_observation_points() {
    let uncached_builder = source_between(
        BUILDER_SOURCE,
        "fn build_mesh_pass_command_buffers_from_batches_uncached",
        "pub(super) fn build_mesh_pass_command_buffers_from_batches_cached",
    );
    let cached_serial_builder = source_between(
        BUILDER_SOURCE,
        "fn build_mesh_pass_command_buffers_from_ordered_batches_cached",
        "pub(super) fn build_mesh_pass_command_buffers_from_batches_cached_parallel",
    );
    let parallel_builder = source_between(
        PARALLEL_PREPARATION_SOURCE,
        "pub(super) fn build_mesh_pass_command_buffers_from_batches_cached_parallel",
        "fn prepare_batch_plan",
    );

    for counter in [
        "mesh_commands.batch_count",
        "mesh_commands.worker_count",
        "mesh_commands.parallel_enabled",
        "mesh_commands.dispatch_reason_code",
    ] {
        assert_eq!(
            parallel_builder.matches(counter).count(),
            1,
            "cached parallel dispatch counter must have one owner: {counter}"
        );
    }
    assert!(parallel_builder.contains("record_counter_batch("));
    assert_eq!(
        parallel_builder.matches("profile_counter!(").count(),
        0,
        "parallel dispatch metadata must take one recorder lock"
    );
    assert!(
        !uncached_builder.contains("record_preparation_result_profile"),
        "cache result counters do not belong to the uncached builder"
    );
    assert_eq!(
        cached_serial_builder
            .matches("record_preparation_result_profile(&commands, &cache_stats);")
            .count(),
        1,
        "cached serial preparation must publish the shared result schema"
    );
    assert_eq!(
        cached_serial_builder
            .matches("\"seal_phase_buffers\"")
            .count(),
        1,
        "cached serial preparation must isolate phase partitioning and sorting"
    );
    assert_eq!(
        parallel_builder
            .matches("record_preparation_result_profile(&commands, &cache_stats);")
            .count(),
        1,
        "cached parallel preparation must publish the shared result schema"
    );

    let result_profile = source_between(
        BUILDER_SOURCE,
        "fn record_preparation_result_profile",
        "fn collect_batches_in_source_order",
    );
    for counter in [
        "mesh_commands.cache_hit_count",
        "mesh_commands.cache_miss_count",
        "mesh_commands.command_rebuild_count",
        "mesh_commands.command_count",
    ] {
        assert_eq!(
            result_profile.matches(counter).count(),
            1,
            "cached preparation result counter must have one owner: {counter}"
        );
    }
    assert!(result_profile.contains("record_counter_batch("));
    assert_eq!(
        result_profile.matches("profile_counter!(").count(),
        0,
        "cached preparation results must take one recorder lock"
    );
    assert_eq!(
        BUILDER_SOURCE
            .matches("record_preparation_result_profile(&commands, &cache_stats);")
            .count(),
        2,
        "serial and parallel builders must publish the same preparation result schema"
    );
}

#[test]
fn dispatch_reason_profile_codes_remain_stable() {
    assert_eq!(ParallelPreparationMode::Parallel.profile_code(), 0);
    assert_eq!(ParallelPreparationMode::SingleWorker.profile_code(), 1);
    assert_eq!(ParallelPreparationMode::SmallBatch.profile_code(), 2);
    assert_eq!(ParallelPreparationMode::DuplicateCacheKey.profile_code(), 3);
}

#[test]
fn parallel_admission_uses_the_transaction_shader_quality() {
    let parallel_builder = source_between(
        PARALLEL_PREPARATION_SOURCE,
        "pub(super) fn build_mesh_pass_command_buffers_from_batches_cached_parallel",
        "fn prepare_batch_plan",
    );
    assert!(parallel_builder
        .contains("ParallelPreparationMode::select(&batches, shader_quality, task_pool)"));

    let selector = source_between(
        PARALLEL_ADMISSION_SOURCE,
        "pub(super) fn select",
        "pub(super) const fn is_parallel",
    );
    assert!(selector.contains("shader_quality: ShaderQualityTier"));
    assert!(selector.contains("has_duplicate_cache_keys(batches, shader_quality)"));

    let duplicate_scan = PARALLEL_ADMISSION_SOURCE
        .split_once("fn has_duplicate_cache_keys")
        .map(|(_, body)| body)
        .expect("duplicate cache-key scan source boundary");
    assert!(duplicate_scan.contains("shader_quality: ShaderQualityTier"));
    assert!(duplicate_scan.contains("from_batch_phase(batch, phase, shader_quality)"));
}

#[test]
fn cached_parallel_worker_does_not_create_per_batch_timeline_spans() {
    let worker = source_between(
        PARALLEL_PREPARATION_SOURCE,
        "fn build_prepared_batch_chunk",
        "struct PreparedBatchPlan",
    );

    assert!(
        !worker.contains("profile_scope!"),
        "per-batch timeline spans would make profiling overhead scale with draw count"
    );
    assert!(BUILDER_SOURCE.contains("\"prepare_cached_serial\""));
}

#[test]
fn parallel_preparation_has_one_folder_backed_owner() {
    assert!(BUILDER_SOURCE.contains("mod parallel_preparation;"));
    assert!(BUILDER_SOURCE.contains("pub(super) use parallel_preparation::"));
    assert!(BUILDER_SOURCE
        .contains("build_mesh_pass_command_buffers_from_batches_cached_parallel;"));
    for implementation_detail in [
        "ParallelSliceExecutor",
        "PreparedBatchPlan",
        "PreparedBatchChunk",
        "PreparedCacheStore",
    ] {
        assert!(
            !BUILDER_SOURCE.contains(implementation_detail),
            "builder root must not retain parallel preparation detail: {implementation_detail}"
        );
        assert!(
            PARALLEL_PREPARATION_SOURCE.contains(implementation_detail),
            "parallel preparation owner must retain its implementation detail: {implementation_detail}"
        );
    }
}

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .and_then(|(_, tail)| tail.split_once(end).map(|(body, _)| body))
        .unwrap_or_else(|| panic!("source boundary is missing: {start} .. {end}"))
}

fn assert_stage_order(source: &str, stages: &[&str]) {
    let mut previous_position = None;
    for stage in stages {
        let position = source
            .find(stage)
            .unwrap_or_else(|| panic!("profiling stage is missing: {stage}"));
        if let Some(previous_position) = previous_position {
            assert!(
                previous_position < position,
                "profiling stages must preserve execution order"
            );
        }
        previous_position = Some(position);
    }
}
