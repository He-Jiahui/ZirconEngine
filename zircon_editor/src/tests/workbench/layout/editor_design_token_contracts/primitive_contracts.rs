use super::support::assert_tokenized_assets;
use toml::Value;

macro_rules! workbench_asset {
    ($path:literal) => {
        include_str!(concat!(
            "../../../../../assets/ui/editor/components/workbench/",
            $path
        ))
    };
}

mod foundational;

#[test]
fn component_drawer_list_and_table_samples_inherit_shared_row_geometry_and_states() {
    let asset = workbench_asset!("shell/workbench_component_drawer.zui");
    let document = asset
        .parse::<Value>()
        .expect("component drawer must remain valid ZUI TOML");
    let nodes = document
        .get("nodes")
        .and_then(Value::as_table)
        .expect("component drawer must expose nodes");
    let row_kinds = ["WorkbenchListRow", "WorkbenchTableRow"];
    let forbidden_properties = [
        "background_color",
        "border_color",
        "foreground_color",
        "text_color",
        "icon_color",
        "fourth_cell_text_color",
        "layout_content_offset_x",
        "layout_content_offset_y",
        "layout_first_cell_offset_x",
        "layout_fourth_cell_offset_x",
        "layout_offset_x",
        "layout_offset_y",
        "layout_padding_bottom",
        "layout_padding_top",
        "layout_second_cell_offset_x",
    ];
    let mut list_count = 0usize;
    let mut table_count = 0usize;
    let mut overrides = Vec::new();

    for (node_id, node) in nodes {
        let Some(kind) = node.get("component").and_then(Value::as_str) else {
            continue;
        };
        if !row_kinds.contains(&kind) {
            continue;
        }
        if kind == "WorkbenchListRow" {
            list_count += 1;
        } else {
            table_count += 1;
        }

        let props = node
            .get("props")
            .and_then(Value::as_table)
            .expect("drawer row samples must expose a property table");
        for property in forbidden_properties {
            if props.contains_key(property) {
                overrides.push(format!("{kind} `{node_id}` overrides `{property}`"));
            }
        }
    }

    assert_eq!(
        list_count, 3,
        "component drawer must retain list-row samples"
    );
    assert_eq!(
        table_count, 4,
        "component drawer must retain table-row samples"
    );
    assert!(
        overrides.is_empty(),
        "component drawer rows must inherit shared geometry and state visuals: {overrides:#?}"
    );
}

#[test]
fn component_drawer_feedback_and_text_samples_inherit_primitive_visuals() {
    let asset = workbench_asset!("shell/workbench_component_drawer.zui");
    let document = asset
        .parse::<Value>()
        .expect("component drawer must remain valid ZUI TOML");
    let nodes = document
        .get("nodes")
        .and_then(Value::as_table)
        .expect("component drawer must expose nodes");
    let contracts: [(&str, &[&str]); 10] = [
        ("sliders_title", &["label_color"]),
        ("selection_title", &["label_color"]),
        ("labs_tabs", &["background_color"]),
        ("visual_label", &["foreground_color"]),
        (
            "feedback_tooltip",
            &[
                "arrow_color",
                "arrow_size",
                "background_color",
                "border_color",
                "icon_color",
                "label_color",
                "layout_content_offset_y",
                "layout_icon_size",
                "text_color",
            ],
        ),
        (
            "feedback_skeleton",
            &["background_color", "highlight_color"],
        ),
        ("feedback_toast_spacer", &["border_width", "corner_radius"]),
        ("list_group", &["border_width", "corner_radius"]),
        ("table_group", &["border_width", "corner_radius"]),
        ("popup_menu", &["layout_offset_y"]),
    ];

    for (node_id, forbidden_properties) in contracts {
        let node = nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("component drawer must retain `{node_id}`"));
        let props = node
            .get("props")
            .and_then(Value::as_table)
            .unwrap_or_else(|| panic!("component drawer node `{node_id}` must expose props"));
        for property in forbidden_properties {
            assert!(
                !props.contains_key(*property),
                "component drawer node `{node_id}` must inherit `{property}` from its primitive"
            );
        }
    }
}

#[test]
fn text_and_property_primitives_share_typography_and_density_tokens() {
    assert_tokenized_assets(&[
        (
            "workbench_caption.zui",
            workbench_asset!("primitives/data/workbench_caption.zui"),
            &[
                "$editor.typography.caption.size",
                "$editor.typography.strong.weight",
                "$editor.typography.line_height",
                "$editor.density.gap.xsmall",
            ],
        ),
        (
            "workbench_section_title.zui",
            workbench_asset!("primitives/chrome/workbench_section_title.zui"),
            &[
                "$editor.typography.body.size",
                "$editor.typography.strong.weight",
                "$editor.typography.line_height",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
            ],
        ),
        (
            "workbench_property_row.zui",
            workbench_asset!("primitives/data/workbench_property_row.zui"),
            &[
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.control.height.dense",
            ],
        ),
        (
            "workbench_component_property_row.zui",
            workbench_asset!("primitives/data/workbench_component_property_row.zui"),
            &[
                "$editor.surface.recessed",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.disabled",
                "$editor.separator.soft",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.control",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.control.height.dense",
            ],
        ),
    ]);
}

#[test]
fn label_primitives_bind_runtime_text_line_height_on_their_roots() {
    for (asset_name, asset_source, root_id) in [
        (
            "workbench_label.zui",
            workbench_asset!("primitives/data/workbench_label.zui"),
            "root",
        ),
        (
            "workbench_caption.zui",
            workbench_asset!("primitives/data/workbench_caption.zui"),
            "root",
        ),
        (
            "workbench_section_title.zui",
            workbench_asset!("primitives/chrome/workbench_section_title.zui"),
            "root",
        ),
        (
            "workbench_chip.zui",
            workbench_asset!("primitives/chrome/workbench_chip.zui"),
            "root",
        ),
    ] {
        let document = asset_source
            .parse::<Value>()
            .unwrap_or_else(|error| panic!("{asset_name} must remain valid ZUI TOML: {error}"));
        let root = document
            .get("nodes")
            .and_then(Value::as_table)
            .and_then(|nodes| nodes.get(root_id))
            .and_then(Value::as_table)
            .unwrap_or_else(|| panic!("{asset_name} must expose its `{root_id}` root node"));
        assert_eq!(root.get("component").and_then(Value::as_str), Some("Label"));
        assert_eq!(
            root.get("props")
                .and_then(Value::as_table)
                .and_then(|props| props.get("line_height_ratio"))
                .and_then(Value::as_str),
            Some("$editor.typography.line_height"),
            "{asset_name} must bind Runtime Text line height directly on its Label root"
        );
    }
}

#[test]
fn component_property_row_uses_shared_input_insets() {
    let asset = workbench_asset!("primitives/data/workbench_component_property_row.zui");
    assert_tokenized_assets(&[(
        "workbench_component_property_row.zui",
        asset,
        &[
            "$editor.density.gap.medium",
            "$editor.density.gap.small",
            "$editor.typography.body.size",
            "$editor.typography.line_height",
        ],
    )]);
    assert!(
        !asset.contains("layout_padding_left = 0.0")
            && !asset.contains("layout_padding_right = 0.0")
            && !asset.contains("layout_padding_top = 0.0")
            && !asset.contains("layout_padding_bottom = 0.0"),
        "component property row must not retain local input padding"
    );
}

#[test]
fn compact_text_primitives_use_xsmall_vertical_padding() {
    for (asset_name, asset_source) in [
        (
            "workbench_caption.zui",
            workbench_asset!("primitives/data/workbench_caption.zui"),
        ),
        (
            "workbench_label.zui",
            workbench_asset!("primitives/data/workbench_label.zui"),
        ),
    ] {
        assert!(
            asset_source.contains("$editor.density.gap.xsmall"),
            "{asset_name} must use the xsmall density token for compact text padding"
        );
        assert!(
            !asset_source.contains("layout_padding_left = 0.0")
                && !asset_source.contains("layout_padding_right = 0.0")
                && !asset_source.contains("layout_padding_top = 2.0")
                && !asset_source.contains("layout_padding_bottom = 2.0"),
            "{asset_name} must not keep local compact text padding"
        );
    }
}

#[test]
fn label_uses_shared_dense_control_height_cap() {
    let asset = workbench_asset!("primitives/data/workbench_label.zui");

    assert!(
        asset.contains("max = \"$editor.control.height.dense\""),
        "label must cap its compact row height through the shared dense control token"
    );
    assert!(
        !asset.contains("max = 28.0"),
        "label must not retain the fixed dense control-height value"
    );
}

#[test]
fn feedback_primitives_use_shared_spacing_and_control_height_tokens() {
    let tooltip = workbench_asset!("primitives/feedback/workbench_tooltip.zui");
    let drag_overlay = workbench_asset!("primitives/feedback/workbench_drag_overlay.zui");

    assert!(
        tooltip.contains("arrow_size = \"$editor.density.gap.medium\"")
            && !tooltip.contains("arrow_size = 8.0"),
        "tooltip arrow size must follow the shared medium spacing token"
    );
    assert!(
        drag_overlay.contains("drop_target_height = \"$editor.control.height.compact\"")
            && !drag_overlay.contains("drop_target_height = 30.0"),
        "drag-overlay target height must follow the compact control height token"
    );
}

#[test]
fn property_editor_row_composite_inherits_shared_density_and_height_tokens() {
    let asset = workbench_asset!("composites/inputs/workbench_property_editor_row.zui");

    assert!(
        asset.contains("styles = [\"res://ui/editor/theme/editor_tokens.zui\"]"),
        "property-editor row must import the shared editor token registry"
    );
    assert!(
        asset.contains(
            "container = { kind = \"HorizontalBox\", gap = \"$editor.density.gap.small\" }"
        ) && asset.contains("min = \"$editor.control.height.dense\"")
            && asset.contains("preferred = \"$editor.control.height.compact\"")
            && asset.contains("max = \"$editor.control.height.default\""),
        "property-editor row must use shared spacing and three-tier control heights"
    );
    for fixed_metric in ["gap = 4.0", "min = 28.0", "preferred = 30.0", "max = 32.0"] {
        assert!(
            !asset.contains(fixed_metric),
            "property-editor row must not retain fixed metric `{fixed_metric}`"
        );
    }
}

#[test]
fn floating_composites_inherit_shared_spacing_and_control_height_tokens() {
    let command_palette = workbench_asset!("floating/workbench_command_palette.zui");
    let preferences = workbench_asset!("floating/workbench_preferences.zui");

    assert!(
        command_palette.contains(
            "container = { kind = \"VerticalBox\", gap = \"$editor.density.gap.small\" }"
        ) && command_palette.contains(
            "min = \"$editor.control.height.default\", preferred = \"$editor.control.height.default\", max = \"$editor.control.height.default\""
        ) && command_palette.contains(
            "min = \"$editor.control.height.dense\", preferred = \"$editor.control.height.dense\", max = \"$editor.control.height.dense\""
        ),
        "command palette must inherit shared spacing, search, and result-row metrics"
    );
    assert!(
        !command_palette.contains("container = { kind = \"VerticalBox\", gap = 4.0 }")
            && !command_palette.contains("min = 32.0, preferred = 32.0, max = 32.0")
            && !command_palette.contains("min = 28.0, preferred = 28.0, max = 28.0"),
        "command palette must not retain fixed shared density metrics"
    );

    for required in [
        "container = { kind = \"HorizontalBox\", gap = \"$editor.density.gap.medium\" }",
        "selected_category_id = \"\"",
        "categories = []",
        "settings = []",
        "plugin_pages = []",
    ] {
        assert!(
            preferences.contains(required),
            "preferences must retain dynamic settings-window contract `{required}`"
        );
    }
    assert!(
        !preferences.contains("WorkbenchPreferencesGeneral")
            && !preferences.contains("WorkbenchPreferencesLayout"),
        "preferences must not retain fixed legacy category rows"
    );
    for fixed_metric in [
        "container = { kind = \"HorizontalBox\", gap = 8.0 }",
        "container = { kind = \"VerticalBox\", gap = 2.0 }",
        "container = { kind = \"VerticalBox\", gap = 8.0 }",
        "min = 28.0, preferred = 28.0, max = 28.0",
        "min = 32.0, preferred = 32.0, max = 32.0",
    ] {
        assert!(
            !preferences.contains(fixed_metric),
            "preferences must not retain fixed floating-window metric `{fixed_metric}`"
        );
    }
}

#[test]
fn chip_slider_and_status_primitives_share_editor_tokens() {
    assert_tokenized_assets(&[
        (
            "workbench_chip.zui",
            workbench_asset!("primitives/chrome/workbench_chip.zui"),
            &[
                "$editor.typography.body.size",
                "$editor.typography.line_height",
                "$editor.density.gap.small",
                "$editor.density.gap.medium",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
            ],
        ),
        (
            "workbench_slider.zui",
            workbench_asset!("primitives/inputs/workbench_slider.zui"),
            &[
                "$editor.surface.0",
                "$editor.surface.disabled",
                "$editor.separator.strong",
                "$editor.separator.soft",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.semantic.warning",
                "$editor.semantic.error",
                "$editor.control.border_width",
                "$editor.control.radius.small",
                "$editor.density.gap.small",
                "$editor.density.gap.medium",
                "$editor.density.gap.large",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_range_slider.zui",
            workbench_asset!("primitives/inputs/workbench_range_slider.zui"),
            &[
                "$editor.surface.0",
                "$editor.surface.disabled",
                "$editor.separator.strong",
                "$editor.separator.soft",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.semantic.warning",
                "$editor.semantic.error",
                "$editor.control.border_width",
                "$editor.control.radius.small",
                "$editor.density.gap.small",
                "$editor.density.gap.medium",
                "$editor.density.gap.large",
                "$editor.control.height.default",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_status_item.zui",
            workbench_asset!("primitives/feedback/workbench_status_item.zui"),
            &[
                "$editor.typography.body.size",
                "$editor.typography.line_height",
                "$editor.text.primary",
                "$editor.text.disabled",
            ],
        ),
    ]);
}

#[test]
fn chip_uses_shared_horizontal_density() {
    let asset = workbench_asset!("primitives/chrome/workbench_chip.zui");
    assert_tokenized_assets(&[(
        "workbench_chip.zui",
        asset,
        &[
            "$editor.density.gap.medium",
            "$editor.density.gap.small",
            "$editor.control.height.dense",
            "$editor.control.height.compact",
            "$editor.control.height.default",
        ],
    )]);
    assert!(
        !asset.contains("layout_padding_left = 10.0")
            && !asset.contains("layout_padding_right = 10.0"),
        "chip must not retain local horizontal padding"
    );
}

#[test]
fn menu_and_collection_composites_share_editor_tokens() {
    assert_tokenized_assets(&[
        (
            "workbench_popup_menu.zui",
            workbench_asset!("primitives/feedback/workbench_popup_menu.zui"),
            &[
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.control.height.dense",
                "$editor.control.border_width",
                "$editor.control.radius.panel",
            ],
        ),
        (
            "workbench_dropdown_popup.zui",
            workbench_asset!("primitives/feedback/workbench_dropdown_popup.zui"),
            &[
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.control.height.dense",
                "$editor.control.border_width",
                "$editor.control.radius.panel",
            ],
        ),
        (
            "workbench_context_menu.zui",
            workbench_asset!("primitives/feedback/workbench_context_menu.zui"),
            &[
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.control.height.dense",
                "$editor.control.border_width",
                "$editor.control.radius.panel",
            ],
        ),
        (
            "workbench_list_row.zui",
            workbench_asset!("primitives/data/workbench_list_row.zui"),
            &[
                "$editor.surface.1",
                "$editor.surface.hover",
                "$editor.surface.selected",
                "$editor.surface.3",
                "$editor.surface.disabled",
                "$editor.accent.soft",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.separator.soft",
                "$editor.control.border_width",
                "$editor.control.radius.small",
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.control.height.dense",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_tree_row.zui",
            workbench_asset!("primitives/data/workbench_tree_row.zui"),
            &[
                "$editor.surface.1",
                "$editor.surface.hover",
                "$editor.surface.selected",
                "$editor.surface.3",
                "$editor.surface.disabled",
                "$editor.accent.soft",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.separator.soft",
                "$editor.control.border_width",
                "$editor.control.radius.small",
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.control.height.dense",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_table_row.zui",
            workbench_asset!("primitives/data/workbench_table_row.zui"),
            &[
                "$editor.surface.1",
                "$editor.surface.hover",
                "$editor.surface.selected",
                "$editor.surface.3",
                "$editor.surface.disabled",
                "$editor.accent.soft",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.separator.soft",
                "$editor.control.border_width",
                "$editor.control.radius.small",
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.control.height.dense",
                "$editor.typography.caption.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_tab_strip.zui",
            workbench_asset!("primitives/inputs/workbench_tab_strip.zui"),
            &[
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
            ],
        ),
    ]);
}

#[test]
fn popup_menu_anchor_tracks_the_compact_control_height_token() {
    let asset = workbench_asset!("primitives/feedback/workbench_popup_menu.zui");

    assert!(
        asset.contains("popup_anchor_y = \"$editor.control.height.compact\""),
        "popup-menu anchor must follow the compact control height token"
    );
    assert!(
        !asset.contains("popup_anchor_y = 30.0"),
        "popup-menu anchor must not retain the old fixed compact-height value"
    );
}

#[test]
fn dropdown_popup_anchor_tracks_shared_control_height_tokens() {
    let asset = workbench_asset!("primitives/feedback/workbench_dropdown_popup.zui");

    assert!(
        asset.contains("popup_anchor_x = \"$editor.density.gap.large\"")
            && asset.contains("popup_anchor_y = \"$editor.control.height.default\"")
            && asset.contains("popup_anchor_height = \"$editor.control.height.compact\""),
        "dropdown-popup anchor must follow the shared spacing and control height tokens"
    );
    assert!(
        !asset.contains("popup_anchor_x = 12.0")
            && !asset.contains("popup_anchor_y = 32.0")
            && !asset.contains("popup_anchor_height = 30.0"),
        "dropdown-popup anchor must not retain old fixed spacing or control-height values"
    );
}
