#[test]
fn recompute_preserves_pending_render_work_until_render_submission_consumes_it() {
    let decision_source = include_str!("../recompute/invalidation/decision.rs");
    let recompute_source = include_str!("../recompute.rs");
    let production = recompute_source
        .split("#[cfg(test)]")
        .next()
        .expect("recompute source should expose its production section");

    assert!(decision_source.contains("pending_reasons.union(legacy_dirty_reasons)"));
    assert!(
        !production.contains("self.render_dirty = false"),
        "only render submission may consume a pending render request"
    );
}

#[test]
fn scoped_view_presentation_returns_before_the_full_shell_rebuild() {
    let source = include_str!("../recompute.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("recompute source should expose its production section");
    let scoped_branch = production
        .split_once("RecomputeInvalidationTarget::ViewPresentation")
        .and_then(|(_, tail)| tail.split_once("if paint_only_reasons.requires_layout()"))
        .map(|(_, tail)| tail)
        .expect("scoped presentation branch should remain before full recompute");

    assert!(scoped_branch.contains("self.apply_scoped_ui_asset_presentation(view_ids)"));
    assert!(scoped_branch.contains("return;"));
    assert!(!scoped_branch.contains("build_recompute_shell_snapshot"));
}

#[test]
fn workbench_projection_patch_runs_before_the_full_shell_rebuild() {
    let source = include_str!("../recompute.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("recompute production source");
    let projection = production
        .find("apply_workbench_projection_presentation")
        .expect("workbench projection fast path");
    let shell = production
        .find("build_recompute_shell_snapshot")
        .expect("full shell fallback");

    assert!(projection < shell);
}

#[test]
fn shell_content_patch_runs_before_the_full_shell_snapshot_build() {
    let source = include_str!("../recompute.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("recompute production source");
    let shell_content = production
        .find("apply_committed_shell_content_presentation")
        .expect("shell content fast path");
    let shell_snapshot = production
        .find("build_recompute_shell_snapshot")
        .expect("full shell snapshot fallback");

    assert!(shell_content < shell_snapshot);
}

#[test]
fn window_metrics_stage_cache_runs_before_the_full_shell_snapshot_build() {
    let source = include_str!("../recompute.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("recompute production source");
    let metrics = production
        .find("build_window_metrics_shell_snapshot")
        .expect("window metrics stage cache");
    let full = production
        .find("build_recompute_shell_snapshot(false)")
        .expect("full shell fallback");

    assert!(metrics < full);
}

#[test]
fn stable_shell_content_passes_layout_reuse_to_the_shell_builder() {
    let source = include_str!("../recompute.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("recompute production source");

    assert!(production.contains("recompute_decision.reuse_shell_layout"));
    assert!(production.contains("build_recompute_shell_snapshot(requested_shell_layout_reuse)"));
    assert!(production.contains("shell.reuse_shell_layout"));
}

#[test]
fn window_metrics_reuses_stable_shell_before_pane_payload_collection() {
    let source = include_str!("../recompute.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("recompute production source");
    let geometry_fast_path = production
        .find("apply_window_metrics_geometry_presentation")
        .expect("window metrics geometry fast path");
    let payload_collection = production
        .find("collect_host_lifecycle_pane_payloads")
        .expect("pane payload collection fallback");

    assert!(geometry_fast_path < payload_collection);
    assert!(production.contains("retained_shell_presentation.as_ref()"));
    assert!(production.contains("retained_pane_payloads.as_ref()"));
    assert!(production.contains("retained_shell_presentation: shell.retained_shell_presentation"));
}
