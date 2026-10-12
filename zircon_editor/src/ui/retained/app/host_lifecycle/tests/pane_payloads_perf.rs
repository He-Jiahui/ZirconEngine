#[test]
fn editor_pane_payloads_consume_one_targeted_identity_projection() {
    let sources = [
        include_str!("../pane_payloads.rs"),
        include_str!("../pane_payloads/editor_panes.rs"),
    ]
    .concat();

    assert_eq!(sources.matches("editor_pane_instance_ids(").count(), 1);
    assert!(!sources.contains("current_view_instances"));
    assert!(sources.contains("collect_ui_asset_panes || collect_animation_panes"));
}

#[test]
fn shell_content_payloads_are_gated_by_the_target_kind() {
    let source = include_str!("../pane_payloads.rs");
    let targeted = source
        .split("fn collect_shell_content_pane_payloads")
        .nth(1)
        .and_then(|body| body.split("fn collect_host_lifecycle_pane_payloads").next())
        .expect("targeted shell content payload collector");

    for kind in [
        "ViewContentKind::UiAssetEditor",
        "ViewContentKind::AnimationSequenceEditor",
        "ViewContentKind::AnimationGraphEditor",
        "ViewContentKind::RuntimeDiagnostics",
        "ViewContentKind::PerformanceTimeline",
        "ViewContentKind::ModulePlugins",
        "ViewContentKind::BuildExport",
    ] {
        assert!(targeted.contains(kind), "missing targeted gate for {kind}");
    }
    assert!(!targeted.contains("should_collect_payload_for_kind"));
    assert!(!targeted.contains("current_view_instances"));
    assert!(!targeted.contains("ui_template_pane_data_snapshots()"));
    assert!(targeted.contains("target_instance_id"));
}
