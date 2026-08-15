use super::support::assert_contains_all;

pub(super) fn assert_runtime_04_mirror_docs() {
    let mirror_docs = [
        (
            "Runtime 04 plan",
            include_str!(
                "../../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "asset facade doc",
            include_str!("../../../../../docs/zircon_runtime/asset/facade.md"),
        ),
        (
            "asset worker pool doc",
            include_str!("../../../../../docs/zircon_runtime/asset/worker_pool.md"),
        ),
        (
            "asset watcher doc",
            include_str!("../../../../../docs/zircon_runtime/asset/watcher.md"),
        ),
        (
            "asset artifact doc",
            include_str!("../../../../../docs/zircon_runtime/asset/artifact.md"),
        ),
        (
            "core resource doc",
            include_str!("../../../../../docs/zircon_runtime/core/resource.md"),
        ),
        (
            "M0 review",
            include_str!(
                "../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
            ),
        ),
        (
            "runtime-interface convergence",
            include_str!(
                "../../../../../docs/engine-architecture/runtime-interface-convergence.md"
            ),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        assert_contains_all(
            doc_name,
            doc_source,
            super::inventory::ASSET_PIPELINE_MIRROR_DOC_ANCHORS,
        );
    }
}

#[test]
fn runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation() {
    let plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
    );
    let mut plan_lines = plan.lines();
    assert_eq!(
        plan_lines.next().map(str::trim),
        Some("---"),
        "Runtime 04 plan should start with YAML frontmatter"
    );
    let mut frontmatter_status = None;
    let mut frontmatter_closed = false;
    for line in plan_lines.by_ref() {
        if line.trim() == "---" {
            frontmatter_closed = true;
            break;
        }
        if line.trim() == "status: in_progress" {
            frontmatter_status = Some("in_progress");
        }
    }
    assert!(
        frontmatter_closed,
        "Runtime 04 plan should close its YAML frontmatter"
    );

    assert!(
        frontmatter_status.is_some(),
        "Runtime 04 must remain in progress until broader asset validation closes"
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
        let row = plan
            .lines()
            .find(|line| line.contains(&row_anchor))
            .unwrap_or_else(|| panic!("Runtime 04 should keep status row `{row_name}`"));
        assert_contains_all("Runtime 04 pending status row", row, &["Cargo", "待"]);
    }
    assert_contains_all(
        "Runtime 04 validation gate commands",
        plan,
        &[
            "cargo test -p zircon_runtime --lib load_state --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib resource --locked",
            "cargo test -p zircon_runtime --lib asset:: --locked",
            "cargo test -p zircon_runtime --lib worker_pool --locked -- --nocapture",
            "cargo test -p zircon_runtime --lib watch --locked -- --nocapture",
            "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
        ],
    );
    assert_runtime_04_mirror_docs();
}
