macro_rules! workbench_asset {
    ($path:literal) => {
        include_str!(concat!(
            "../../../../assets/ui/editor/components/workbench/",
            $path
        ))
    };
}

mod atomic_controls;

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

#[test]
fn shell_chrome_roots_share_tokenized_surface_typography_and_density_inputs() {
    assert_tokenized_assets(&[
        (
            "workbench_top_toolbar.zui",
            workbench_asset!("shell/workbench_top_toolbar.zui"),
            &[
                "$editor.surface.2",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.separator.soft",
                "$editor.border",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_status_bar.zui",
            workbench_asset!("shell/workbench_status_bar.zui"),
            &[
                "$editor.surface.0",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.separator.soft",
                "$editor.border",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
    ]);
}

#[test]
fn activity_drawer_window_uses_relative_shell_constraints() {
    let asset = workbench_asset!("shell/activity_drawer_window.zui");
    assert_tokenized_assets(&[(
        "activity_drawer_window.zui",
        asset,
        &[
            "$editor.density.compact_side_min_width",
            "$editor.density.left_drawer_width",
            "$editor.density.right_drawer_width",
            "$editor.density.ultra_compact_bottom_min_height",
            "$editor.density.bottom_output_height",
        ],
    )]);

    for stretch_slot in [
        "ActivityDrawerWindowContentSlot",
        "ActivityDrawerWindowLeftTopSlot",
        "ActivityDrawerWindowLeftBottomSlot",
        "ActivityDrawerWindowRightTopSlot",
        "ActivityDrawerWindowRightBottomSlot",
        "ActivityDrawerWindowBottomLeftSlot",
        "ActivityDrawerWindowBottomRightSlot",
    ] {
        assert!(
            asset.contains(stretch_slot),
            "activity drawer window must retain the {stretch_slot} relative slot"
        );
    }
    assert!(
        asset.matches("stretch = \"Stretch\"").count() >= 11,
        "activity drawer window must keep its root, bands, and slots stretch-based"
    );
    assert!(
        [
            "min = 240.0",
            "preferred = 280.0",
            "max = 420.0",
            "preferred = 300.0",
            "max = 460.0",
            "min = 170.0",
            "preferred = 220.0",
            "max = 320.0",
        ]
        .iter()
        .all(|legacy_extent| !asset.contains(legacy_extent)),
        "activity drawer window must not retain local drawer or bottom-band geometry"
    );
}

#[test]
fn activity_rail_uses_shared_component_spacing() {
    let asset = workbench_asset!("shell/workbench_activity_rail.zui");
    assert_tokenized_assets(&[(
        "workbench_activity_rail.zui",
        asset,
        &[
            "$editor.density.gap.medium",
            "$editor.density.activity_rail_width",
            "$editor.control.height.large",
        ],
    )]);
    assert!(
        !asset.contains("gap = 10.0")
            && !asset.contains("min = 72.0")
            && !asset.contains("preferred = 72.0")
            && !asset.contains("max = 72.0")
            && !asset.contains("min = 48.0")
            && !asset.contains("preferred = 48.0")
            && !asset.contains("max = 48.0"),
        "activity rail must not keep local gap, shell-width, or item-size geometry"
    );
}

fn panel_header_uses_shared_container_surface_and_density() {
    let asset = workbench_asset!("composites/chrome/workbench_panel_header.zui");
    assert_tokenized_assets(&[(
        "workbench_panel_header.zui",
        asset,
        &[
            "$editor.surface.1",
            "$editor.separator.soft",
            "$editor.control.border_width",
            "$editor.density.gap.small",
            "$editor.control.height.dense",
            "$editor.control.height.compact",
        ],
    )]);
    assert!(
        asset.contains("name = \"title\"")
            && asset.contains("name = \"actions\"")
            && asset.matches("stretch = \"Stretch\"").count() >= 4,
        "panel header must preserve its stretch title/actions slot contract"
    );
    assert!(
        !asset.contains("gap = 2.0")
            && !asset.contains("min = 28.0")
            && !asset.contains("max = 30.0"),
        "panel header must not retain local density metrics"
    );
}

fn assert_tokenized_assets(assets: &[(&str, &str, &[&str])]) {
    for &(asset_name, asset_source, required_tokens) in assets {
        assert!(
            asset_source.contains("res://ui/editor/theme/editor_tokens.zui"),
            "{asset_name} must import the editor token asset"
        );
        for &token in required_tokens {
            assert!(
                asset_source.contains(token),
                "{asset_name} must use {token} instead of a local component value"
            );
        }
        assert!(
            !contains_hex_color(asset_source),
            "{asset_name} must not reintroduce a naked hex color"
        );
    }
}

fn contains_hex_color(source: &str) -> bool {
    source
        .as_bytes()
        .windows(7)
        .any(|window| window[0] == b'#' && window[1..].iter().all(u8::is_ascii_hexdigit))
}
