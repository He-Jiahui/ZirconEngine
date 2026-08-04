use super::*;

#[test]
fn runtime_15_asset_watcher_shutdown_on_drop_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let asset_watch_dir = manifest_root.join("src/asset/watch");
    let retired_drop_impl = asset_watch_dir.join("drop_impl.rs");
    let asset_watch_mod = read_text(
        &asset_watch_dir.join("mod.rs"),
        "asset watcher module entry should be readable",
    );
    let shutdown_on_drop = read_text(
        &asset_watch_dir.join("shutdown_on_drop.rs"),
        "asset watcher shutdown-on-drop owner should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let asset_watcher_doc = read_repo_text(manifest_root, "docs/zircon_runtime/asset/watcher.md");

    assert!(
        !retired_drop_impl.exists(),
        "asset watcher should not keep banned-name module file {:?}",
        retired_drop_impl
    );
    assert_contains_all(
        "asset watcher module entry",
        &asset_watch_mod,
        &["mod shutdown_on_drop;"],
    );
    assert!(
        !asset_watch_mod.contains("mod drop_impl;"),
        "asset/watch/mod.rs should not preserve the banned drop_impl module name"
    );
    assert_contains_all(
        "asset watcher shutdown-on-drop owner",
        &shutdown_on_drop,
        &[
            "impl Drop for AssetWatcher",
            "self.stop_tx.send(())",
            "join.join()",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("asset watcher doc", asset_watcher_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 asset watcher shutdown-on-drop module naming hard cutover",
                "runtime_15_asset_watcher_shutdown_on_drop_naming_hard_cutover_static_passed_cargo_deferred",
                "asset/watch/shutdown_on_drop.rs",
                "runtime_15_asset_watcher_shutdown_on_drop_uses_owner_name",
            ],
        );
    }
}

#[test]
fn runtime_15_asset_change_construction_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let asset_watch_dir = manifest_root.join("src/asset/watch");
    let retired_asset_change_new = asset_watch_dir.join("asset_change_new.rs");
    let asset_watch_mod = read_text(
        &asset_watch_dir.join("mod.rs"),
        "asset watcher module entry should be readable",
    );
    let asset_change_construction = read_text(
        &asset_watch_dir.join("asset_change_construction.rs"),
        "asset change construction owner should be readable",
    );
    let fold_events = read_text(
        &asset_watch_dir.join("fold_events.rs"),
        "asset watcher fold events owner should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let asset_watcher_doc = read_repo_text(manifest_root, "docs/zircon_runtime/asset/watcher.md");

    assert!(
        !retired_asset_change_new.exists(),
        "asset watcher should not keep *_new construction owner file {:?}",
        retired_asset_change_new
    );
    assert_contains_all(
        "asset watcher module entry",
        &asset_watch_mod,
        &["mod asset_change_construction;"],
    );
    assert!(
        !asset_watch_mod.contains("mod asset_change_new;"),
        "asset/watch/mod.rs should not preserve the retired asset_change_new module name"
    );
    assert_contains_all(
        "asset change construction owner",
        &asset_change_construction,
        &[
            "impl AssetChange",
            "pub fn new(",
            "kind: AssetChangeKind",
            "previous_uri: Option<AssetUri>",
        ],
    );
    assert_contains_all(
        "asset watcher fold events owner",
        &fold_events,
        &["AssetChange::new("],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("asset watcher doc", asset_watcher_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 asset change construction module naming hard cutover",
                "runtime_15_asset_change_construction_naming_hard_cutover_static_passed_cargo_deferred",
                "asset/watch/asset_change_construction.rs",
                "runtime_15_asset_change_construction_uses_owner_name",
            ],
        );
    }
}
