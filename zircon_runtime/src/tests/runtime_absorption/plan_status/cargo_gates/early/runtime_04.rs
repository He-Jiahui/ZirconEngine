#[test]
fn runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation() {
    let runtime_04_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let asset_facade_doc = include_str!("../../../../../../../docs/zircon_runtime/asset/facade.md");
    let asset_worker_doc =
        include_str!("../../../../../../../docs/zircon_runtime/asset/worker_pool.md");
    let review = include_str!(
        "../../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );

    assert_eq!(
        frontmatter_status(runtime_04_plan),
        Some("in_progress"),
        "Runtime 04 should stay in progress until broader asset validation closes"
    );

    for row_name in [
        "1.1 句柄语义裁决",
        "1.2 转移表测试",
        "2.1 背压策略",
        "2.2 请求去重",
        "2.3 诊断计数",
        "worker pool 当前状态守卫",
    ] {
        let row_anchor = format!("| {row_name} |");
        let row = runtime_04_plan
            .lines()
            .find(|line| line.contains(&row_anchor))
            .unwrap_or_else(|| panic!("Runtime 04 should keep status row `{row_name}`"));
        assert_contains_all("Runtime 04 pending status row", row, &["Cargo", "待"]);
    }

    assert_contains_all(
        "Runtime 04 validation gate commands",
        runtime_04_plan,
        &[
            "cargo test -p zircon_runtime --lib load_state --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib resource --locked",
            "cargo test -p zircon_runtime --lib asset:: --locked",
            "cargo test -p zircon_runtime --lib worker_pool --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib watch --locked -- --nocapture",
            "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
        ],
    );

    let runtime_04_index_row =
        runtime_index_row_for(runtime_index, "04-asset-pipeline-alignment.md");
    assert_contains_all(
        "Runtime 04 index row",
        runtime_04_index_row,
        &[
            "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
            "broader `asset::` / `worker_pool` Cargo filters",
            "Cargo 待",
        ],
    );

    let runtime_04_problem_row =
        runtime_index_problem_row_for(runtime_index, "P7", "asset pipeline");
    assert_contains_all(
        "Runtime index P7 row",
        runtime_04_problem_row,
        &[
            "asset_worker_pool_matches_runtime_04_and_11_decisions",
            "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
            "broader `asset::` / `worker_pool` Cargo filters",
        ],
    );

    assert_contains_all(
        "Runtime asset facade doc",
        asset_facade_doc,
        &[
            "Reference Asset Stack Gap Table",
            "dangling_handle_queries_report_not_loaded_instead_of_panicking",
            "failed_asset_exposes_failure_reason_through_facade",
        ],
    );
    assert_contains_all(
        "Runtime asset worker doc",
        asset_worker_doc,
        &[
            "AssetWorkerPoolOptions",
            "Backpressure",
            "Request De-Duplication",
            "asset.worker.budgeted_threads",
        ],
    );
    assert_contains_all(
        "Runtime architecture review Runtime 04 gate",
        review,
        &[
            "Runtime 04 Asset Pipeline Guard",
            "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
            "asset:: / worker_pool",
        ],
    );
}
