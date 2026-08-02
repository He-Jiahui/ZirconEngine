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

#[test]
fn feedback_selection_and_label_primitives_share_editor_tokens() {
    assert_tokenized_assets(&[
        (
            "workbench_tooltip.zui",
            workbench_asset!("primitives/feedback/workbench_tooltip.zui"),
            &[
                "$editor.popup",
                "$editor.separator.soft",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.accent",
                "$editor.control.border_width",
                "$editor.control.radius.small",
            ],
        ),
        (
            "workbench_alert.zui",
            workbench_asset!("primitives/feedback/workbench_alert.zui"),
            &[
                "$editor.semantic.warning",
                "$editor.semantic.warning.container",
                "$editor.control.border_width",
                "$editor.control.radius.small",
                "$editor.control.height.compact",
                "$editor.control.height.default",
            ],
        ),
        (
            "workbench_toast.zui",
            workbench_asset!("primitives/feedback/workbench_toast.zui"),
            &[
                "$editor.semantic.info",
                "$editor.semantic.info.container",
                "$editor.control.border_width",
                "$editor.control.radius.small",
                "$editor.control.height.compact",
                "$editor.control.height.default",
            ],
        ),
        (
            "workbench_progress_bar.zui",
            workbench_asset!("primitives/feedback/workbench_progress_bar.zui"),
            &[
                "$editor.surface.recessed",
                "$editor.surface.disabled",
                "$editor.accent",
                "$editor.separator.soft",
                "$editor.border.disabled",
                "$editor.text.primary",
                "$editor.text.disabled",
                "$editor.semantic.warning",
                "$editor.semantic.error",
                "$editor.control.border_width",
                "$editor.control.radius.small",
                "$editor.density.gap.small",
                "$editor.density.gap.medium",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_drag_overlay.zui",
            workbench_asset!("primitives/feedback/workbench_drag_overlay.zui"),
            &[
                "$editor.accent.soft",
                "$editor.semantic.error",
                "$editor.semantic.error.container",
                "$editor.text.primary",
                "$editor.control.border_width",
                "$editor.control.radius.control",
                "$editor.density.gap.large",
                "$editor.typography.overlay.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_notification_center.zui",
            workbench_asset!("primitives/feedback/workbench_notification_center.zui"),
            &[
                "$editor.popup",
                "$editor.border",
                "$editor.surface.1",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.separator.soft",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.accent",
                "$editor.semantic.success",
                "$editor.semantic.warning",
                "$editor.semantic.error",
                "$editor.control.border_width",
                "$editor.control.radius.small",
                "$editor.control.radius.panel",
                "$editor.typography.body.size",
                "$editor.typography.overlay.size",
                "$editor.typography.caption.size",
            ],
        ),
        (
            "workbench_dialog.zui",
            workbench_asset!("primitives/feedback/workbench_dialog.zui"),
            &[
                "$editor.popup",
                "$editor.border",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.semantic.info",
                "$editor.semantic.info.container",
                "$editor.semantic.warning",
                "$editor.semantic.warning.container",
                "$editor.semantic.error",
                "$editor.semantic.error.container",
                "$editor.control.border_width",
                "$editor.control.radius.panel",
                "$editor.typography.title.size",
                "$editor.typography.caption.size",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_confirm_dialog.zui",
            workbench_asset!("primitives/feedback/workbench_confirm_dialog.zui"),
            &[
                "$editor.popup",
                "$editor.border",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.semantic.info",
                "$editor.semantic.info.container",
                "$editor.semantic.warning",
                "$editor.semantic.warning.container",
                "$editor.semantic.error",
                "$editor.semantic.error.container",
                "$editor.control.border_width",
                "$editor.control.radius.panel",
                "$editor.typography.title.size",
                "$editor.typography.caption.size",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_command_palette.zui",
            workbench_asset!("primitives/feedback/workbench_command_palette.zui"),
            &[
                "$editor.popup",
                "$editor.surface.recessed",
                "$editor.border",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.control.border_width",
                "$editor.control.radius.small",
                "$editor.control.radius.panel",
                "$editor.control.height.compact",
                "$editor.density.gap.medium",
                "$editor.density.gap.large",
                "$editor.density.row_height",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_skeleton.zui",
            workbench_asset!("primitives/feedback/workbench_skeleton.zui"),
            &[
                "$editor.surface.3",
                "$editor.surface.disabled",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.small",
                "$editor.typography.title.size",
                "$editor.control.height.dense",
            ],
        ),
        (
            "workbench_radio.zui",
            workbench_asset!("primitives/inputs/workbench_radio.zui"),
            &[
                "$editor.surface.recessed",
                "$editor.surface.2",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.separator.strong",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.accent",
                "$editor.control.border_width",
                "$editor.density.gap.medium",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_checkbox.zui",
            workbench_asset!("primitives/inputs/workbench_checkbox.zui"),
            &[
                "$editor.surface.recessed",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.separator.strong",
                "$editor.border.disabled",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.accent",
                "$editor.control.border_width",
                "$editor.control.radius.small",
                "$editor.density.gap.medium",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_icon.zui",
            workbench_asset!("primitives/data/workbench_icon.zui"),
            &[
                "$editor.text.secondary",
                "$editor.typography.title.size",
                "$editor.control.height.dense",
            ],
        ),
        (
            "workbench_label.zui",
            workbench_asset!("primitives/data/workbench_label.zui"),
            &[
                "$editor.typography.body.size",
                "$editor.typography.medium.weight",
                "$editor.text.primary",
                "$editor.density.gap.xsmall",
            ],
        ),
    ]);
}

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
                "$editor.density.gap.xsmall",
            ],
        ),
        (
            "workbench_section_title.zui",
            workbench_asset!("primitives/chrome/workbench_section_title.zui"),
            &[
                "$editor.typography.body.size",
                "$editor.typography.strong.weight",
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
fn chip_slider_and_status_primitives_share_editor_tokens() {
    assert_tokenized_assets(&[
        (
            "workbench_chip.zui",
            workbench_asset!("primitives/chrome/workbench_chip.zui"),
            &[
                "$editor.typography.body.size",
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
                "$editor.control.radius.small",
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
                "$editor.control.radius.small",
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
                "$editor.control.radius.small",
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
