#[test]
fn editor_manager_ui_asset_sessions_are_split_by_host_orchestration_behaviors() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("host")
        .join("asset_editor_sessions");

    for relative in [
        "mod.rs",
        "open.rs",
        "save.rs",
        "lifecycle.rs",
        "sync.rs",
        "imports.rs",
        "hydration.rs",
        "preview_refresh.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected host ui asset session module {relative} under {:?}",
            root
        );
    }
}
