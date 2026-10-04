const SOURCE: &str = include_str!("../environment_capture_filter_wgpu.rs");

fn production_source() -> &'static str {
    SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("environment capture filtering must retain a test boundary")
}

#[test]
fn capture_filter_records_source_mips_pmrem_and_sh9_into_one_encoder() {
    let source = production_source();

    assert!(source.contains("fn record("));
    assert!(source.contains("record_source_mip_chain("));
    assert!(source.contains("ibl_bake_pmrem_kernel_plan_with_quality("));
    assert!(source.contains("ibl_bake_irradiance_sh9_kernel_plan("));
    assert!(source.contains("encode_ibl_bake_wgpu_compute_dispatch("));
    assert!(!source.contains("submit_graphics_command_buffers("));
}

#[test]
fn capture_filter_reuses_renderer_owned_pipelines_and_reports_exact_work() {
    let source = production_source();

    assert!(source.contains("pipelines: &RealtimeIblCaptureWgpuPipelines"));
    assert!(source.contains("pipeline_cache: &mut IblBakeWgpuPipelineCache"));
    assert!(source.contains("source_mip_dispatch_count"));
    assert!(source.contains("pmrem_dispatch_count"));
    assert!(source.contains("sh9_dispatch_count"));
    assert!(!source.contains("RealtimeIblCaptureWgpuPipelines::new("));
    assert!(!source.contains("IblBakeWgpuPipelineCache::new("));
}

#[test]
fn capture_filter_publishes_existing_work_report_to_the_profiler() {
    let source = production_source();

    for counter in [
        "environment_capture_source_mip_dispatch_count",
        "environment_capture_pmrem_dispatch_count",
        "environment_capture_sh9_dispatch_count",
        "environment_capture_source_mip_binding_creation_micros",
        "environment_capture_ibl_bind_group_creation_count",
    ] {
        assert!(
            source.contains(counter),
            "missing profile counter {counter}"
        );
    }
    assert!(source.contains("report.emit_profile_counters()"));
}
