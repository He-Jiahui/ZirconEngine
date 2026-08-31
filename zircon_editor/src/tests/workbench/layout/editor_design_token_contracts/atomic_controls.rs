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
fn atomic_controls_share_editor_visual_and_density_tokens() {
    assert_tokenized_assets(&[
        (
            "workbench_button.zui",
            workbench_asset!("primitives/inputs/workbench_button.zui"),
            &[
                "$editor.surface.1",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.control",
                "$editor.density.gap.large",
                "$editor.density.gap.medium",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_icon_button.zui",
            workbench_asset!("primitives/inputs/workbench_icon_button.zui"),
            &[
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.control",
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.control.height.default",
            ],
        ),
        (
            "workbench_rail_button.zui",
            workbench_asset!("primitives/chrome/workbench_rail_button.zui"),
            &[
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.control",
                "$editor.density.gap.medium",
                "$editor.control.height.large",
            ],
        ),
        (
            "workbench_axis_value_field.zui",
            workbench_asset!("primitives/chrome/workbench_axis_value_field.zui"),
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
                "$editor.density.gap.small",
                "$editor.density.gap.medium",
                "$editor.density.axis_value_field.min_width",
                "$editor.density.axis_value_field.preferred_width",
                "$editor.density.axis_value_field.max_width",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_tab.zui",
            workbench_asset!("primitives/inputs/workbench_tab.zui"),
            &[
                "$editor.surface.2",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.secondary",
                "$editor.text.disabled",
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
            "workbench_divider.zui",
            workbench_asset!("primitives/data/workbench_divider.zui"),
            &[
                "$editor.separator.soft",
                "$editor.border.disabled",
                "$editor.control.border_width",
                "$editor.density.gap.medium",
            ],
        ),
        (
            "workbench_search_input.zui",
            workbench_asset!("primitives/inputs/workbench_search_input.zui"),
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
                "$editor.density.gap.small",
                "$editor.density.gap.large",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_dropdown.zui",
            workbench_asset!("primitives/inputs/workbench_dropdown.zui"),
            &[
                "$editor.surface.recessed",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.accent.soft",
                "$editor.surface.disabled",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.control",
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.density.gap.large",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.caption.size",
                "$editor.typography.overlay.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_field.zui",
            workbench_asset!("primitives/inputs/workbench_field.zui"),
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
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_number_field.zui",
            workbench_asset!("primitives/inputs/workbench_number_field.zui"),
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
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_toggle.zui",
            workbench_asset!("primitives/inputs/workbench_toggle.zui"),
            &[
                "$editor.surface.2",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.separator.strong",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.density.gap.medium",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_segmented_control.zui",
            workbench_asset!("primitives/inputs/workbench_segmented_control.zui"),
            &[
                "$editor.surface.2",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.border",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.control",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
    ]);
}

#[test]
fn workbench_tab_uses_tokenized_states_and_stretch_width() {
    let asset = workbench_asset!("primitives/inputs/workbench_tab.zui");

    assert!(
        asset.contains("width = { stretch = \"Stretch\" }"),
        "the reusable tab must let its parent allocate horizontal width"
    );
    assert!(
        [
            "layout_padding_right = 10.0",
            "layout_padding_top = 3.0",
            "layout_min_width = 72.0",
            "layout_icon_size = 14.0",
            "width = { min = 88.0, preferred = 96.0, max = 140.0, stretch = \"Fixed\" }",
        ]
        .iter()
        .all(|legacy_metric| !asset.contains(legacy_metric)),
        "the reusable tab must not retain local spacing, icon, or width metrics"
    );
}

#[test]
fn workbench_rail_button_uses_shared_compact_control_geometry() {
    let asset = workbench_asset!("primitives/chrome/workbench_rail_button.zui");

    assert!(
        asset.matches("$editor.control.height.compact").count() >= 4,
        "the activity-rail button must use the shared compact control size for both hit axes"
    );
    assert!(
        [
            "layout_spacing = 0.0",
            "layout_min_width = 44.0",
            "layout_min_height = 44.0",
            "layout_icon_size = 22.0",
            "width = { min = 46.0, preferred = 48.0, max = 50.0, stretch = \"Fixed\" }",
            "height = { min = 46.0, preferred = 48.0, max = 50.0, stretch = \"Fixed\" }",
        ]
        .iter()
        .all(|legacy_metric| !asset.contains(legacy_metric)),
        "the activity-rail button must not retain private fixed geometry"
    );
}

#[test]
fn component_drawer_button_samples_inherit_atomic_visuals_without_pixel_nudges() {
    let asset = workbench_asset!("shell/workbench_component_drawer.zui");
    let document = asset
        .parse::<Value>()
        .expect("component drawer must remain valid ZUI TOML");
    let nodes = document
        .get("nodes")
        .and_then(Value::as_table)
        .expect("component drawer must expose nodes");
    let mut button_count = 0usize;
    let mut icon_button_count = 0usize;
    let mut overrides = Vec::new();

    for (node_id, node) in nodes {
        let component = node.get("component").and_then(Value::as_str);
        let Some(kind @ ("WorkbenchButton" | "WorkbenchIconButton")) = component else {
            continue;
        };
        if kind == "WorkbenchButton" {
            button_count += 1;
        } else {
            icon_button_count += 1;
        }

        let props = node
            .get("props")
            .and_then(Value::as_table)
            .expect("drawer control samples must expose a property table");
        for property in [
            "background_color",
            "border_color",
            "foreground_color",
            "icon_color",
            "corner_radius",
            "border_width",
            "layout_offset_x",
            "layout_offset_y",
            "visual_brightness",
            "disabled_opacity",
        ] {
            if props.contains_key(property) {
                overrides.push(format!("{kind} `{node_id}` overrides `{property}`"));
            }
        }
    }

    assert_eq!(
        button_count, 7,
        "component drawer must retain its button samples"
    );
    assert_eq!(
        icon_button_count, 8,
        "component drawer must retain its icon-button samples"
    );
    assert!(
        overrides.is_empty(),
        "component drawer samples must inherit shared atomic visuals: {overrides:#?}"
    );
}

#[test]
fn component_drawer_input_samples_inherit_atomic_metrics_and_state_visuals() {
    let asset = workbench_asset!("shell/workbench_component_drawer.zui");
    let document = asset
        .parse::<Value>()
        .expect("component drawer must remain valid ZUI TOML");
    let nodes = document
        .get("nodes")
        .and_then(Value::as_table)
        .expect("component drawer must expose nodes");
    let atomic_kinds = [
        "WorkbenchCheckbox",
        "WorkbenchDropdown",
        "WorkbenchField",
        "WorkbenchRadio",
        "WorkbenchRangeSlider",
        "WorkbenchSegmentedControl",
        "WorkbenchSlider",
        "WorkbenchTab",
        "WorkbenchToggle",
    ];
    let forbidden_properties = [
        "background_color",
        "border_color",
        "foreground_color",
        "text_color",
        "icon_color",
        "corner_radius",
        "border_width",
        "disabled_opacity",
        "label_color",
        "label_brightness",
        "dot_color",
        "dot_size",
        "layout_icon_size",
        "layout_offset_x",
        "layout_offset_y",
        "layout_spacing",
        "selected_segment_border_width",
        "selected_segment_underline_color",
        "track_height",
        "track_offset_x",
        "track_width",
        "track_width_delta",
        "thumb_size",
        "visual_brightness",
        "value_color",
        "arrow_color",
        "selected_segment_underline_height",
    ];
    let mut sample_count = 0usize;
    let mut overrides = Vec::new();

    for (node_id, node) in nodes {
        let Some(kind) = node.get("component").and_then(Value::as_str) else {
            continue;
        };
        if !atomic_kinds.contains(&kind) {
            continue;
        }

        sample_count += 1;
        let props = node
            .get("props")
            .and_then(Value::as_table)
            .expect("drawer control samples must expose a property table");
        for property in forbidden_properties {
            if props.contains_key(property) {
                overrides.push(format!("{kind} `{node_id}` overrides `{property}`"));
            }
        }
    }

    assert_eq!(
        sample_count, 20,
        "component drawer must retain its input and selection samples"
    );
    assert!(
        overrides.is_empty(),
        "component drawer input samples must inherit shared atomic metrics and state visuals: {overrides:#?}"
    );
}
