use super::super::support::{rust_files_under, top_level_entry_names};
use super::{
    EXPECTED_SURFACE_ENTRY_COUNT, EXPECTED_SURFACE_SOURCE_ENTRY_COUNT, EXPECTED_UI_ENTRY_COUNT,
};

pub(super) fn assert_current_inventory(architecture_doc: &str) {
    let ui_entries = top_level_entry_names("zircon_runtime/src/ui", false);
    let surface_entries = top_level_entry_names("zircon_runtime/src/ui/surface", true);
    let expected_ui = [
        "accessibility",
        "action",
        "asset",
        "binding",
        "catalog",
        "component",
        "dispatch",
        "editable_text_composition.rs",
        "event_ui",
        "icon_atlas",
        "keyboard",
        "layout",
        "module",
        "module.rs",
        "platform_input",
        "prelude.rs",
        "public_runtime_frame.rs",
        "render",
        "secure_text_policy.rs",
        "style",
        "style.rs",
        "surface",
        "template",
        "tests",
        "text",
        "theme",
        "tree",
        "v2",
    ];
    let expected_surface = [
        "arranged.rs",
        "arranged_visibility.rs",
        "binding_targets.rs",
        "binding_transaction.rs",
        "clipboard_transfers.rs",
        "component_state.rs",
        "control_index.rs",
        "default_interactions",
        "diagnostics.rs",
        "ecs_projection.rs",
        "focus",
        "focus.rs",
        "frame_hit_test",
        "frame_hit_test.rs",
        "host_font_assets.rs",
        "input",
        "interaction_gate.rs",
        "invalidation",
        "invalidation.rs",
        "localized_text",
        "localized_text.rs",
        "mod.rs",
        "mutation_snapshot.rs",
        "navigation_index",
        "navigation_index.rs",
        "node_pool.rs",
        "pointer_component_events",
        "popup_stack.rs",
        "property_mutation",
        "property_mutation.rs",
        "reflection_snapshot",
        "reflection_snapshot.rs",
        "render",
        "render_fonts.rs",
        "secure_text_values.rs",
        "session_identity.rs",
        "slots.rs",
        "subtree_mount.rs",
        "surface",
        "surface.rs",
        "tests",
        "text_artifact",
        "text_artifact.rs",
        "text_geometry.rs",
        "text_shape.rs",
        "timeline",
        "timeline.rs",
        "virtual_list_materialization",
        "virtual_list_materialization.rs",
        "virtual_list_prototype_pool.rs",
    ];
    assert_eq!(ui_entries.len(), EXPECTED_UI_ENTRY_COUNT);
    assert_eq!(surface_entries.len(), EXPECTED_SURFACE_ENTRY_COUNT);
    assert_eq!(ui_entries, expected_ui);
    assert_eq!(surface_entries, expected_surface);
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let source_surface_entries = surface_entries
        .iter()
        .filter(|entry| {
            let path = repo_root.join("zircon_runtime/src/ui/surface").join(entry);
            (path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
                || (path.is_dir()
                    && !rust_files_under(&format!("zircon_runtime/src/ui/surface/{entry}"))
                        .is_empty())
        })
        .count();
    assert_eq!(source_surface_entries, EXPECTED_SURFACE_SOURCE_ENTRY_COUNT);
    assert_eq!(surface_entries.len() - source_surface_entries, 0);
    for anchor in [
        "Module Boundary Map",
        "expected_ui_entry_count = 28",
        "expected_surface_entry_count = 50",
        "surface_source_module_entry_count = 50",
        "surface_empty_directory_count = 0",
    ] {
        assert!(
            architecture_doc.contains(anchor),
            "current UI inventory: {anchor}"
        );
    }
}
