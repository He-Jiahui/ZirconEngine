#[test]
fn ui_asset_editor_host_genericizes_collection_event_dispatch() {
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));
    let ui_asset_editor = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/ui_asset_editor.rs"
    ));
    assert!(
        wiring.contains("pane_surface_host.on_ui_asset_collection_event("),
        "callback wiring should converge PaneSurfaceHostContext UI asset selection callbacks into a generic collection event hook"
    );

    for legacy_wiring in [
        "ui.on_ui_asset_matched_style_rule_selected(",
        "ui.on_ui_asset_palette_selected(",
        "ui.on_ui_asset_palette_target_candidate_selected(",
        "ui.on_ui_asset_hierarchy_selected(",
        "ui.on_ui_asset_hierarchy_activated(",
        "ui.on_ui_asset_preview_selected(",
        "ui.on_ui_asset_preview_activated(",
        "ui.on_ui_asset_source_outline_selected(",
        "ui.on_ui_asset_preview_mock_selected(",
        "ui.on_ui_asset_binding_selected(",
        "ui.on_ui_asset_binding_event_selected(",
        "ui.on_ui_asset_binding_action_kind_selected(",
        "ui.on_ui_asset_binding_payload_selected(",
        "ui.on_ui_asset_slot_semantic_selected(",
        "ui.on_ui_asset_layout_semantic_selected(",
    ] {
        assert!(
            !wiring.contains(legacy_wiring),
            "callback wiring should drop legacy UI asset collection hook `{legacy_wiring}`"
        );
    }

    assert!(
        ui_asset_editor.contains("pub(super) fn dispatch_ui_asset_collection_event("),
        "ui asset editor host dispatch should expose a generic collection event dispatcher"
    );

    for legacy_dispatch in [
        "pub(super) fn dispatch_ui_asset_matched_style_rule_selected(",
        "pub(super) fn dispatch_ui_asset_palette_selected(",
        "pub(super) fn dispatch_ui_asset_palette_target_candidate_selected(",
        "pub(super) fn dispatch_ui_asset_hierarchy_selected(",
        "pub(super) fn dispatch_ui_asset_hierarchy_activated(",
        "pub(super) fn dispatch_ui_asset_preview_selected(",
        "pub(super) fn dispatch_ui_asset_preview_activated(",
        "pub(super) fn dispatch_ui_asset_source_outline_selected(",
        "pub(super) fn dispatch_ui_asset_preview_mock_selected(",
        "pub(super) fn dispatch_ui_asset_binding_selected(",
        "pub(super) fn dispatch_ui_asset_binding_event_selected(",
        "pub(super) fn dispatch_ui_asset_binding_action_kind_selected(",
        "pub(super) fn dispatch_ui_asset_binding_payload_selected(",
        "pub(super) fn dispatch_ui_asset_slot_semantic_selected(",
        "pub(super) fn dispatch_ui_asset_layout_semantic_selected(",
    ] {
        assert!(
            !ui_asset_editor.contains(legacy_dispatch),
            "ui asset editor host dispatch should remove legacy collection handler `{legacy_dispatch}`"
        );
    }

    for manager_call in [
        ".select_ui_asset_editor_matched_style_rule(",
        ".select_ui_asset_editor_palette_index(",
        ".select_ui_asset_editor_palette_target_candidate(",
        ".select_ui_asset_editor_hierarchy_index(",
        ".activate_ui_asset_editor_hierarchy_index(",
        ".select_ui_asset_editor_preview_index(",
        ".activate_ui_asset_editor_preview_index(",
        ".select_ui_asset_editor_source_outline_index(",
        ".select_ui_asset_editor_preview_mock_property(",
        ".select_ui_asset_editor_binding(",
        ".select_ui_asset_editor_binding_event_option(",
        ".select_ui_asset_editor_binding_action_kind(",
        ".select_ui_asset_editor_binding_payload(",
        ".select_ui_asset_editor_slot_semantic(",
        ".select_ui_asset_editor_layout_semantic(",
    ] {
        assert!(
            ui_asset_editor.contains(manager_call),
            "generic collection dispatch should still route through `{manager_call}`"
        );
    }
}

#[test]
fn ui_asset_editor_host_genericizes_detail_event_dispatch() {
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));
    let ui_asset_editor = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/ui_asset_editor.rs"
    ));
    let component_adapter = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/template_runtime/component_adapter/asset_editor.rs"
    ));

    assert!(
        wiring.contains("pane_surface_host.on_ui_asset_detail_event("),
        "callback wiring should converge PaneSurfaceHostContext UI asset detail callbacks into a generic detail event hook"
    );

    for legacy_wiring in [
        "ui.on_ui_asset_inspector_widget_action(",
        "ui.on_ui_asset_style_rule_action(",
        "ui.on_ui_asset_style_rule_declaration_action(",
        "ui.on_ui_asset_style_token_action(",
        "ui.on_ui_asset_preview_mock_action(",
        "ui.on_ui_asset_binding_payload_action(",
    ] {
        assert!(
            !wiring.contains(legacy_wiring),
            "callback wiring should drop legacy detail hook `{legacy_wiring}`"
        );
    }

    assert!(
        ui_asset_editor.contains("pub(super) fn dispatch_ui_asset_detail_event("),
        "ui asset editor host dispatch should expose a generic detail event dispatcher"
    );

    for legacy_dispatch in [
        "pub(super) fn dispatch_ui_asset_inspector_widget_action(",
        "pub(super) fn dispatch_ui_asset_style_rule_action(",
        "pub(super) fn dispatch_ui_asset_style_rule_declaration_action(",
        "pub(super) fn dispatch_ui_asset_style_token_action(",
        "pub(super) fn dispatch_ui_asset_preview_mock_action(",
        "pub(super) fn dispatch_ui_asset_binding_payload_action(",
    ] {
        assert!(
            !ui_asset_editor.contains(legacy_dispatch),
            "ui asset editor host dispatch should drop legacy detail dispatcher `{legacy_dispatch}`"
        );
    }

    for host_adapter_route in [
        "dispatch_ui_asset_component_adapter_commit(",
        "\"widget.control_id\"",
        "\"widget.text\"",
        "\"component.root_class_policy\"",
        "\"slot.mount\"",
        "\"slot.width_preferred\"",
        "\"slot.height_preferred\"",
        "\"layout.width_preferred\"",
        "\"layout.height_preferred\"",
        "\"slot.semantic.value\"",
        "\"layout.semantic.value\"",
        "\"binding.id\"",
        "\"binding.event\"",
        "\"binding.route\"",
        "\"binding.route_target\"",
        "\"binding.action_target\"",
    ] {
        assert!(
            ui_asset_editor.contains(host_adapter_route),
            "generic detail dispatch should route authored field commits through component adapter `{host_adapter_route}`"
        );
    }

    for manager_call in [
        ".set_ui_asset_editor_selected_widget_control_id(",
        ".set_ui_asset_editor_selected_widget_text_property(",
        ".set_ui_asset_editor_selected_component_root_class_policy(",
        ".set_ui_asset_editor_selected_slot_mount(",
        ".set_ui_asset_editor_selected_slot_width_preferred(",
        ".set_ui_asset_editor_selected_layout_width_preferred(",
        ".set_ui_asset_editor_selected_binding_id(",
        ".set_ui_asset_editor_selected_binding_event(",
        ".set_ui_asset_editor_selected_binding_route(",
        ".set_ui_asset_editor_selected_binding_route_target(",
        ".set_ui_asset_editor_selected_binding_action_target(",
    ] {
        assert!(
            component_adapter.contains(manager_call),
            "component adapter should still route field commits through `{manager_call}`"
        );
    }

    for manager_call in [
        ".set_ui_asset_editor_selected_promote_widget_asset_id(",
        ".rename_ui_asset_editor_selected_stylesheet_rule(",
        ".upsert_ui_asset_editor_selected_style_rule_declaration(",
        ".upsert_ui_asset_editor_style_token(",
        ".set_ui_asset_editor_selected_preview_mock_value(",
        ".upsert_ui_asset_editor_selected_binding_payload(",
    ] {
        assert!(
            ui_asset_editor.contains(manager_call),
            "generic detail dispatch should still route direct detail action through `{manager_call}`"
        );
    }

    for detail_id in [
        "\"widget\" => self.handle_ui_asset_widget_detail(",
        "\"widget_promote\" =>",
        "\"slot\" => self.handle_ui_asset_slot_detail(",
        "\"layout\" => self.handle_ui_asset_layout_detail(",
        "\"binding\" => self.handle_ui_asset_binding_detail(",
    ] {
        assert!(
            ui_asset_editor.contains(detail_id),
            "generic detail dispatch should include split detail route `{detail_id}`"
        );
    }
}
