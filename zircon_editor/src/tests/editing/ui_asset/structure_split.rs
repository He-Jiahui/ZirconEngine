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
        "session/lifecycle.rs",
        "session/command_entry.rs",
        "session/palette_state.rs",
        "session/binding_state.rs",
        "session/navigation_state.rs",
        "session/theme_state.rs",
        "session/promotion_state.rs",
        "session/style_state.rs",
        "session/presentation_state.rs",
        "session/session_state.rs",
        "session/preview_compile.rs",
        "session/preview_state.rs",
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

#[test]
fn ui_asset_editor_lifecycle_owns_document_validation_and_apply_state() {
    let session_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("asset_editor")
        .join("session");
    let lifecycle_source =
        std::fs::read_to_string(session_root.join("lifecycle.rs")).expect("read lifecycle source");
    let session_source = std::fs::read_to_string(session_root.join("ui_asset_editor_session.rs"))
        .expect("read session source");

    assert!(
        lifecycle_source.contains("pub(super) fn revalidate("),
        "expected lifecycle.rs to own revalidate after the split",
    );
    assert!(
        lifecycle_source.contains("pub(super) fn apply_valid_document("),
        "expected lifecycle.rs to own apply_valid_document after the split",
    );
    assert!(
        !session_source.contains("pub(super) fn revalidate("),
        "expected ui_asset_editor_session.rs to stop owning revalidate after the split",
    );
    assert!(
        !session_source.contains("pub(super) fn apply_valid_document("),
        "expected ui_asset_editor_session.rs to stop owning apply_valid_document after the split",
    );
}

#[test]
fn ui_asset_editor_theme_state_owns_theme_replay_helpers() {
    let session_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("asset_editor")
        .join("session");
    let theme_source =
        std::fs::read_to_string(session_root.join("theme_state.rs")).expect("read theme source");
    let session_source = std::fs::read_to_string(session_root.join("ui_asset_editor_session.rs"))
        .expect("read session source");

    assert!(
        theme_source.contains("pub(super) fn theme_document_replay_bundle("),
        "expected theme_state.rs to own theme_document_replay_bundle after the split",
    );
    assert!(
        !session_source.contains("pub(super) fn theme_document_replay_bundle("),
        "expected ui_asset_editor_session.rs to stop owning theme_document_replay_bundle after the split",
    );
}

#[test]
fn ui_asset_editor_style_state_owns_style_replay_helpers() {
    let session_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("asset_editor")
        .join("session");
    let style_source =
        std::fs::read_to_string(session_root.join("style_state.rs")).expect("read style source");
    let session_source = std::fs::read_to_string(session_root.join("ui_asset_editor_session.rs"))
        .expect("read session source");

    assert!(
        style_source.contains("pub(super) fn editable_stylesheet("),
        "expected style_state.rs to own editable_stylesheet after the split",
    );
    assert!(
        style_source.contains("pub(super) fn style_rule_insert_replay_bundle("),
        "expected style_state.rs to own style_rule_insert_replay_bundle after the split",
    );
    assert!(
        !session_source.contains("pub(super) fn editable_stylesheet("),
        "expected ui_asset_editor_session.rs to stop owning editable_stylesheet after the split",
    );
    assert!(
        !session_source.contains("pub(super) fn style_rule_insert_replay_bundle("),
        "expected ui_asset_editor_session.rs to stop owning style_rule_insert_replay_bundle after the split",
    );
}

#[test]
fn ui_asset_editor_promotion_state_owns_promotion_helpers() {
    let session_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("asset_editor")
        .join("session");
    let promotion_source = std::fs::read_to_string(session_root.join("promotion_state.rs"))
        .expect("read promotion source");
    let session_source = std::fs::read_to_string(session_root.join("ui_asset_editor_session.rs"))
        .expect("read session source");

    for expected in [
        "pub(super) fn normalized_promote_asset_id(",
        "pub(super) fn normalized_promote_component_name(",
        "pub(super) fn normalized_promote_document_id(",
        "pub(super) fn normalized_promote_display_name(",
        "pub(super) fn reference_asset_id(",
        "pub(super) fn restore_or_remove_external_asset_source(",
    ] {
        assert!(
            promotion_source.contains(expected),
            "expected promotion_state.rs to own {expected} after the split",
        );
        assert!(
            !session_source.contains(expected),
            "expected ui_asset_editor_session.rs to stop owning {expected} after the split",
        );
    }
}

#[test]
fn ui_asset_editor_command_entry_owns_document_replay_helpers() {
    let session_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("asset_editor")
        .join("session");
    let command_entry_source = std::fs::read_to_string(session_root.join("command_entry.rs"))
        .expect("read command entry source");
    let session_source = std::fs::read_to_string(session_root.join("ui_asset_editor_session.rs"))
        .expect("read session source");

    for expected in [
        "pub(super) fn tree_document_replay_bundle(",
        "pub(super) fn binding_document_replay_bundle(",
    ] {
        assert!(
            command_entry_source.contains(expected),
            "expected command_entry.rs to own {expected} after the split",
        );
        assert!(
            !session_source.contains(expected),
            "expected ui_asset_editor_session.rs to stop owning {expected} after the split",
        );
    }
}

#[test]
fn ui_asset_editor_presentation_state_owns_view_projection() {
    let session_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("asset_editor")
        .join("session");
    let presentation_source = std::fs::read_to_string(session_root.join("presentation_state.rs"))
        .expect("read presentation state source");
    let session_source = std::fs::read_to_string(session_root.join("ui_asset_editor_session.rs"))
        .expect("read session source");

    for expected in [
        "pub fn reflection_model(&self) -> UiAssetEditorReflectionModel",
        "pub fn pane_presentation(&self) -> UiAssetEditorPanePresentation",
    ] {
        assert!(
            presentation_source.contains(expected),
            "expected presentation_state.rs to own {expected} after the split",
        );
        assert!(
            !session_source.contains(expected),
            "expected ui_asset_editor_session.rs to stop owning {expected} after the split",
        );
    }
}
