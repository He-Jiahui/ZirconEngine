#[test]
fn ui_asset_editor_subsystem_is_grouped_by_domain_folders() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("asset_editor");

    for relative in [
        "binding/mod.rs",
        "binding/binding_inspector.rs",
        "preview/mod.rs",
        "preview/preview_host.rs",
        "preview/preview_mock.rs",
        "preview/preview_projection.rs",
        "source/mod.rs",
        "source/source_buffer.rs",
        "source/source_sync.rs",
        "style/mod.rs",
        "style/inspector_fields.rs",
        "style/inspector_semantics.rs",
        "style/matched_rule_inspection.rs",
        "style/style_rule_declarations.rs",
        "tree/mod.rs",
        "tree/tree_editing.rs",
        "tree/drag_drop_policy.rs",
        "tree/palette_drop/mod.rs",
        "tree/palette_drop/resolution.rs",
        "tree/palette_drop/overlay_slots.rs",
        "tree/palette_drop/grid_slots.rs",
        "tree/palette_drop/flow_slots.rs",
        "session/mod.rs",
        "session/ui_asset_editor_session.rs",
        "session/session_state.rs",
        "session/preview_compile.rs",
        "session/style_inspection.rs",
        "session/hierarchy_projection.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected ui asset editor module {relative} under {:?}",
            root
        );
    }
}
