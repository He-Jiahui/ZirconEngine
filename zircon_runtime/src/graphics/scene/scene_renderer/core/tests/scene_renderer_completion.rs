#[test]
fn completion_owner_polls_once_before_routing_all_cpu_deliveries() {
    let source = include_str!("../scene_renderer_completion.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("completion owner must retain a test boundary");
    let poll = production
        .find("self.backend.poll_submission_completions()")
        .expect("completion owner must poll the backend");
    let submission_journal = production
        .find("journal.observe(poll_receipt")
        .expect("scene submission journal must consume the poll receipt");
    let residency = production
        .find("streamer.maintain_render_asset_gpu_residency_after_rhi_poll")
        .expect("render asset residency must consume the same poll receipt");
    let ibl = production
        .find(".ibl_bake_runtime_writebacks")
        .expect("IBL artifact callbacks must drain after polling");
    let typed_queries = production
        .find("backend.drain_product_diagnostic_query_results()")
        .expect("typed queries must drain after IBL artifact callbacks");
    let timer = production
        .find("GpuPassTimer::try_collect")
        .expect("timer results must collect after typed query routing");
    let statistics = production
        .find("GpuPipelineStatisticsTimer::try_collect")
        .expect("statistics must collect after typed query routing");

    assert_eq!(
        production.matches("poll_submission_completions()").count(),
        1
    );
    assert_eq!(
        production
            .matches("append_submission_statuses(tickets, statuses)")
            .count(),
        1
    );
    assert!(poll < ibl);
    assert!(poll < submission_journal);
    assert!(submission_journal < residency);
    assert!(residency < ibl);
    assert!(submission_journal < ibl);
    assert!(ibl < typed_queries);
    assert!(typed_queries < timer);
    assert!(typed_queries < statistics);
    assert!(!production.contains("GpuReadbackQueue"));
    assert!(!production.contains("device.poll("));
}
