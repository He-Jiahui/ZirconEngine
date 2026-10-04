use zircon_runtime::ui::v2::UiV2AssetLoader;

const ASSETS_ACTIVITY_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/assets_activity.zui"
));

#[test]
fn assets_activity_static_text_uses_central_typography_tokens() {
    for token in [
        "$editor.typography.body.size",
        "$editor.typography.caption.size",
        "$editor.typography.overlay.size",
        "$editor.typography.strong.weight",
        "$editor.typography.emphasis.weight",
    ] {
        assert!(
            ASSETS_ACTIVITY_LAYOUT_TOML.contains(token),
            "assets activity text should reference the central token `{token}`"
        );
    }
    for raw_value in [
        "font_size = 9.0",
        "font_size = 10.0",
        "font_size = 11.0",
        "font_size = 12.0",
        "font_weight = 600",
        "font_weight = 700",
    ] {
        assert!(
            !ASSETS_ACTIVITY_LAYOUT_TOML.contains(raw_value),
            "assets activity must not retain a local typography value `{raw_value}`"
        );
    }
}

#[test]
fn assets_activity_standard_container_metrics_use_central_tokens() {
    for token in [
        "$editor.control.border_width",
        "$editor.control.height.default",
        "$editor.control.height.dense",
        "$editor.control.radius.small",
        "$editor.density.gap.xsmall",
        "$editor.density.gap.small",
        "$editor.density.gap.medium",
        "$editor.density.gap.large",
        "$editor.density.row_height",
    ] {
        assert!(
            ASSETS_ACTIVITY_LAYOUT_TOML.contains(token),
            "assets activity containers should reference the central token `{token}`"
        );
    }
    for raw_value in [
        "radius = 3.0",
        "radius = 4.0",
        "border_width = 1.0",
        "gap = 2.0",
        "gap = 3.0",
        "gap = 4.0",
        "gap = 8.0",
        "gap = 10.0",
        "gap = 12.0",
        "height = { min = 28.0, preferred = 28.0, max = 28.0",
        "height = { min = 32.0, preferred = 32.0, max = 32.0",
    ] {
        assert!(
            !ASSETS_ACTIVITY_LAYOUT_TOML.contains(raw_value),
            "assets activity must not retain a local container metric `{raw_value}`"
        );
    }
}

#[test]
fn assets_activity_bootstrap_layout_self_hosts_shell_sections() {
    let layout = UiV2AssetLoader::load_toml_str(ASSETS_ACTIVITY_LAYOUT_TOML)
        .expect("assets activity layout");

    for required_node in [
        "assets_activity_root",
        "toolbar_panel",
        "toolbar_title_row",
        "toolbar_title_text",
        "toolbar_open_browser_button",
        "toolbar_subtitle_row",
        "toolbar_subtitle_text",
        "toolbar_search_row",
        "toolbar_search_field",
        "toolbar_filter_row",
        "toolbar_kind_filter_dropdown",
        "toolbar_view_mode_list_button",
        "toolbar_view_mode_thumb_button",
        "main_panel",
        "tree_panel",
        "tree_header_panel",
        "tree_title_text",
        "tree_subtitle_text",
        "tree_divider",
        "tree_scroll_body",
        "tree_row_panel",
        "tree_row_icon",
        "tree_row_name_text",
        "tree_row_count_text",
        "content_panel",
        "utility_panel",
        "utility_tabs_row",
        "utility_preview_button",
        "utility_references_button",
        "utility_selection_text",
        "utility_tabs_divider",
        "utility_content_panel",
        "preview_panel",
        "reference_left_panel",
        "reference_right_panel",
    ] {
        assert!(
            layout.nodes.contains_key(required_node),
            "assets activity bootstrap layout should include `{required_node}`"
        );
    }
}
